//! Cloth: a planar face becomes a sheet of particles (a regular grid clipped to the face) that
//! falls under gravity and drapes over the rest of the visible model (rapier soft body with
//! structural, shear and bending edges). [`ClothSim`] never touches the model: its frames carry
//! the particle positions, and [`cloth_mesh`] turns a frame into a smooth triangle mesh.

use super::{MM, owner_body, to_m};
use glam::{DMat4, DVec2, DVec3};
use rapier3d::prelude::*;
use varstotas_kernel::io::mesh_from_triangles;
use varstotas_kernel::texture::carried_placement;
use varstotas_kernel::{EntityRef, FaceId, InstanceId, MaterialId, Mesh, Model, TexPlacement, tessellate::tessellate};

#[derive(Clone, Debug, PartialEq)]
pub struct ClothSettings {
    /// Distance between the particles, mm (the finer, the softer the folds and the slower).
    pub spacing: f64,
    /// Resistance to stretching, 0..1.
    pub stiffness: f64,
    /// Resistance to bending, 0..1 (silk low, leather high).
    pub bending: f64,
    /// The gap the cloth keeps from what it lies on, mm.
    pub thickness: f64,
    pub friction: f64,
    /// m/s², down along -Z.
    pub gravity: f64,
    pub floor: bool,
    /// The cloth does not pass through itself (slower).
    pub self_collision: bool,
    /// Simulated seconds at most.
    pub max_seconds: f64,
}

impl Default for ClothSettings {
    fn default() -> Self {
        Self {
            spacing: 10.0,
            stiffness: 0.8,
            bending: 0.1,
            thickness: 1.0,
            friction: 0.5,
            gravity: 9.81,
            floor: true,
            self_collision: false,
            max_seconds: 6.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClothError {
    /// Select one face, or a group / component holding one face.
    NoFace,
    /// The face is smaller than the spacing.
    TooSmall,
    /// More particles than [`MAX_PARTICLES`]: raise the spacing.
    TooFine(usize),
}

impl std::fmt::Display for ClothError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClothError::NoFace => write!(f, "select one face"),
            ClothError::TooSmall => write!(f, "the face is smaller than the spacing"),
            ClothError::TooFine(n) => write!(f, "{n} particles: raise the spacing"),
        }
    }
}

/// Particles at most (a 160 × 160 grid).
pub const MAX_PARTICLES: usize = 25_600;

/// The face a cloth is made from.
#[derive(Clone, Debug, PartialEq)]
pub struct ClothSource {
    /// The context of the face, and the face.
    pub path: Vec<InstanceId>,
    pub face: FaceId,
    /// The selected group / component holding the face (in the open context), if any.
    pub group: Option<InstanceId>,
    /// World outline and holes.
    pub outer: Vec<DVec3>,
    pub holes: Vec<Vec<DVec3>>,
    pub normal: DVec3,
    /// The face's look, carried over to the cloth: its materials, its texture placement (in the
    /// axes of its context: `to_local` maps world points there, `local_normal` is the face's
    /// own normal) and the material painted on its group.
    pub front: Option<MaterialId>,
    pub back: Option<MaterialId>,
    pub uv: Option<TexPlacement>,
    pub to_local: DMat4,
    pub local_normal: DVec3,
    pub material: Option<MaterialId>,
}

/// The cloth's source in `selection` of the context `path`: one face, or one group / component
/// whose own content is a single face.
pub fn cloth_source(model: &Model, path: &[InstanceId], selection: &[EntityRef]) -> Option<ClothSource> {
    let faces: Vec<FaceId> = selection.iter().filter_map(|e| if let EntityRef::Face(f) = e { Some(*f) } else { None }).collect();
    let insts: Vec<InstanceId> = selection.iter().filter_map(|e| if let EntityRef::Instance(i) = e { Some(*i) } else { None }).collect();
    let (ctx, face, group, material) = match (faces.as_slice(), insts.as_slice()) {
        ([f], []) => (path.to_vec(), *f, None, None),
        ([], [i]) => {
            let inst = model.mesh_at(path)?.instances.get(*i)?;
            if inst.locked {
                return None;
            }
            let inner = &model.defs.get(inst.def)?.mesh;
            if inner.faces.len() != 1 || !inner.instances.is_empty() {
                return None;
            }
            let mut ctx = path.to_vec();
            ctx.push(*i);
            (ctx, inner.faces.keys().next()?, Some(*i), inst.material)
        }
        _ => return None,
    };
    let mesh = model.mesh_at(&ctx)?;
    let f = mesh.faces.get(face)?;
    if f.locked {
        return None;
    }
    let xf = model.transform_at(&ctx);
    let pts = |l: &[varstotas_kernel::VertexId]| l.iter().map(|v| xf.transform_point3(mesh.vertices[*v].pos)).collect::<Vec<_>>();
    let normal = xf.transform_vector3(f.normal).normalize_or_zero();
    Some(ClothSource {
        path: ctx,
        face,
        group,
        outer: pts(&f.outer),
        holes: f.holes.iter().map(|h| pts(h)).collect(),
        normal,
        front: f.front,
        back: f.back,
        uv: f.uv,
        to_local: xf.inverse(),
        local_normal: f.normal,
        material,
    })
}

/// The cloth's particles at rest (world, mm) and its triangles.
#[derive(Clone, Debug, Default)]
pub struct ClothGrid {
    pub positions: Vec<DVec3>,
    pub triangles: Vec<[u32; 3]>,
    edges: Vec<[u32; 2]>,
    bend: Vec<[u32; 2]>,
}

fn inside(p: DVec2, poly: &[DVec2]) -> bool {
    let mut c = false;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            c = !c;
        }
    }
    c
}

/// A regular grid over the face, kept where it is inside the face (holes cut out).
pub fn cloth_grid(src: &ClothSource, spacing: f64) -> Result<ClothGrid, ClothError> {
    let n = src.normal;
    if src.outer.len() < 3 || n.length_squared() < 0.5 {
        return Err(ClothError::NoFace);
    }
    // the face's own axes: along its first edge (a rectangle's grid follows its sides)
    let u = (src.outer[1] - src.outer[0]).reject_from(n).normalize_or_zero();
    let v = n.cross(u);
    let o = src.outer[0];
    let flat = |p: &DVec3| DVec2::new((*p - o).dot(u), (*p - o).dot(v));
    let outer: Vec<DVec2> = src.outer.iter().map(flat).collect();
    let holes: Vec<Vec<DVec2>> = src.holes.iter().map(|h| h.iter().map(flat).collect()).collect();
    let (lo, hi) = outer.iter().fold((DVec2::splat(f64::MAX), DVec2::splat(f64::MIN)), |(a, b), p| (a.min(*p), b.max(*p)));
    let size = hi - lo;
    let spacing = spacing.max(0.1);
    if size.min_element() < spacing {
        return Err(ClothError::TooSmall);
    }
    let (nu, nv) = ((size.x / spacing).round() as usize + 1, (size.y / spacing).round() as usize + 1);
    if nu * nv > MAX_PARTICLES {
        return Err(ClothError::TooFine(nu * nv));
    }
    let (du, dv) = (size.x / (nu - 1) as f64, size.y / (nv - 1) as f64);
    let eps = 1e-6 * size.max_element();
    let at = |i: usize, j: usize| lo + DVec2::new(i as f64 * du, j as f64 * dv);
    // a point on the outline counts as inside: nudge it toward the centre before the test
    let centre = (lo + hi) * 0.5;
    let keep: Vec<bool> = (0..nu * nv)
        .map(|k| {
            let p = at(k / nv, k % nv);
            let q = p + (centre - p).normalize_or_zero() * eps;
            inside(q, &outer) && !holes.iter().any(|h| inside(p - (centre - p).normalize_or_zero() * eps, h))
        })
        .collect();
    let id = |i: usize, j: usize| i * nv + j;
    let mut used = vec![false; nu * nv];
    let mut quads = Vec::new();
    for i in 0..nu - 1 {
        for j in 0..nv - 1 {
            let c = [id(i, j), id(i + 1, j), id(i + 1, j + 1), id(i, j + 1)];
            if c.iter().all(|k| keep[*k]) {
                quads.push((i, j, c));
                for k in c {
                    used[k] = true;
                }
            }
        }
    }
    if quads.is_empty() {
        return Err(ClothError::TooSmall);
    }
    let mut index = vec![u32::MAX; nu * nv];
    let mut grid = ClothGrid::default();
    for k in 0..nu * nv {
        if used[k] {
            index[k] = grid.positions.len() as u32;
            let p = at(k / nv, k % nv);
            grid.positions.push(o + u * p.x + v * p.y);
        }
    }
    let ix = |k: usize| index[k];
    let mut edges = std::collections::BTreeSet::new();
    for (i, j, c) in &quads {
        let [a, b, cc, d] = c.map(ix);
        // CCW about the face's normal, alternating diagonals
        if (i + j) % 2 == 0 {
            grid.triangles.push([a, b, cc]);
            grid.triangles.push([a, cc, d]);
        } else {
            grid.triangles.push([a, b, d]);
            grid.triangles.push([b, cc, d]);
        }
        for e in [[a, b], [b, cc], [cc, d], [d, a], [a, cc], [b, d]] {
            edges.insert([e[0].min(e[1]), e[0].max(e[1])]);
        }
    }
    grid.edges = edges.into_iter().collect();
    for i in 0..nu {
        for j in 0..nv {
            let k = id(i, j);
            if !used[k] {
                continue;
            }
            if i + 2 < nu && used[id(i + 1, j)] && used[id(i + 2, j)] {
                grid.bend.push([ix(k), ix(id(i + 2, j))]);
            }
            if j + 2 < nv && used[id(i, j + 1)] && used[id(i, j + 2)] {
                grid.bend.push([ix(k), ix(id(i, j + 2))]);
            }
        }
    }
    Ok(grid)
}

/// A running cloth. `Send`: it can be stepped on a worker thread.
pub struct ClothSim {
    world: PhysicsWorld,
    body: SoftBodyHandle,
    pub grid: ClothGrid,
    settings: ClothSettings,
    time: f64,
    last: Vec<Vector>,
    /// Positions at the start of the current rest window, and when it started.
    checkpoint: Vec<Vector>,
    since: f64,
    /// Simulated seconds the cloth has barely moved.
    still: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClothFrame {
    /// Particle positions, world mm (in the order of [`ClothGrid::positions`]).
    pub positions: Vec<DVec3>,
    pub time: f64,
    /// At rest, or the time ran out.
    pub done: bool,
}

/// The cloth is at rest once no particle moved faster than `REST_SPEED` (m/s, averaged over
/// windows of `REST_WINDOW` s, so the solver's jitter does not count) for `REST_SECONDS`.
const REST_SECONDS: f64 = 0.5;
const REST_WINDOW: f64 = 0.25;
const REST_SPEED: f64 = 0.008;

impl ClothSim {
    pub fn new(model: &Model, src: &ClothSource, settings: ClothSettings) -> Result<Self, ClothError> {
        let grid = cloth_grid(src, settings.spacing)?;
        let tess = tessellate(model);
        let (mut points, mut tris) = (Vec::new(), Vec::new());
        for face in &tess.faces {
            let own = face.owner_path == src.path && face.face == Some(src.face);
            let in_group = src.group.is_some() && owner_body(&face.owner_path, &src.path[..src.path.len() - 1], &src.group.into_iter().collect::<Vec<_>>()).is_some();
            if own || in_group {
                continue;
            }
            for t in face.positions.as_chunks::<3>().0 {
                if (t[1] - t[0]).cross(t[2] - t[0]).length_squared() < 1e-12 {
                    continue;
                }
                let n = points.len() as u32;
                points.extend(t.iter().map(|p| to_m(*p)));
                tris.push([n, n + 1, n + 2]);
            }
        }
        let mut world = PhysicsWorld::new();
        world.gravity = Vector::new(0.0, 0.0, -settings.gravity as f32);
        let spacing = (settings.spacing * MM) as f32;
        world.integration_parameters.length_unit = spacing.clamp(0.001, 1.0);
        world.integration_parameters.dt = 1.0 / 120.0;
        let friction = settings.friction as f32;
        if !tris.is_empty()
            && let Ok(c) = ColliderBuilder::trimesh_with_flags(points, tris, TriMeshFlags::FIX_INTERNAL_EDGES | TriMeshFlags::MERGE_DUPLICATE_VERTICES)
        {
            world.colliders.insert(c.friction(friction));
        }
        if settings.floor {
            world.colliders.insert(ColliderBuilder::new(SharedShape::halfspace(Vector::Z)).friction(friction));
        }
        let positions: Vec<Vector> = grid.positions.iter().map(|p| to_m(*p)).collect();
        // stiffness 0..1 → natural frequency of the springs (Hz)
        let freq = |x: f64, lo: f64, hi: f64| (lo * (hi / lo).powf(x.clamp(0.0, 1.0))) as f32;
        let material = SoftBodyMaterial {
            edge_softness: SpringCoefficients::new(freq(settings.stiffness, 5.0, 120.0), 1.0),
            bend_softness: SpringCoefficients::new(freq(settings.bending, 0.2, 40.0), 1.0),
            ..SoftBodyMaterial::default()
        };
        let area = grid.triangles.len() as f32 * 0.5 * spacing * spacing;
        let builder = SoftBodyBuilder::new(positions.clone())
            .edges(grid.edges.clone())
            .bend_edges(grid.bend.clone())
            .surface(grid.triangles.clone())
            .oriented(false)
            // thinner than a tenth of the spacing, the cloth slips into what it lies on
            .particle_radius(((settings.thickness * MM) as f32).max(spacing * 0.1))
            .material(material)
            .mass((area * 0.2).max(1e-4))
            .linear_damping(0.5)
            .self_contacts(settings.self_collision)
            .surface_collider(ColliderBuilder::ball(1.0).friction(friction));
        let body = world.soft_bodies.insert(builder, &mut world.bodies, &mut world.colliders);
        Ok(Self { world, body, grid, settings, time: 0.0, checkpoint: positions.clone(), last: positions, since: 0.0, still: 0.0 })
    }

    pub fn time(&self) -> f64 {
        self.time
    }

    fn finished(&self) -> bool {
        self.time >= self.settings.max_seconds || self.still >= REST_SECONDS || self.world.soft_bodies[self.body].is_sleeping()
    }

    /// Advance by `steps` steps of 1/120 s and report the state.
    pub fn step(&mut self, steps: usize) -> ClothFrame {
        let dt = self.world.integration_parameters.dt as f64;
        for _ in 0..steps {
            if self.finished() {
                break;
            }
            self.world.step();
            self.time += dt;
            self.last = self.world.soft_bodies[self.body].particle_positions().collect();
            let window = self.time - self.since;
            if window >= REST_WINDOW - 1e-9 {
                let moved = self.last.iter().zip(&self.checkpoint).map(|(a, b)| (*a - *b).length()).fold(0.0f32, f32::max) as f64;
                // the first instants are the fall: rest only counts after half a second
                self.still = if moved / window < REST_SPEED && self.time > 0.5 { self.still + window } else { 0.0 };
                self.checkpoint = self.last.clone();
                self.since = self.time;
            }
        }
        self.frame()
    }

    pub fn frame(&self) -> ClothFrame {
        let positions = self.last.iter().map(|p| DVec3::new(p.x as f64, p.y as f64, p.z as f64) / MM).collect();
        ClothFrame { positions, time: self.time, done: self.finished() }
    }
}

/// The cloth of a frame as a mesh in the axes of `parent` (the world transform of the context
/// it goes into): smooth, its inner edges soft. Each triangle keeps the source face's materials
/// and its texture as it lay on the flat face (the texture follows the folds).
pub fn cloth_mesh(src: &ClothSource, grid: &ClothGrid, positions: &[DVec3], parent: DMat4) -> Mesh {
    let inv = parent.inverse();
    let local: Vec<DVec3> = positions.iter().map(|p| inv.transform_point3(*p)).collect();
    let mut normals = vec![DVec3::ZERO; local.len()];
    for t in &grid.triangles {
        let [a, b, c] = t.map(|i| local[i as usize]);
        let n = (b - a).cross(c - a);
        for i in t {
            normals[*i as usize] += n;
        }
    }
    for n in &mut normals {
        *n = n.normalize_or_zero();
    }
    let rest: Vec<DVec3> = grid.positions.iter().map(|p| src.to_local.transform_point3(*p)).collect();
    mesh_from_triangles(&local, &grid.triangles, &normals, |i, face| {
        face.front = src.front;
        face.back = src.back;
        let t = grid.triangles[i].map(|k| k as usize);
        face.uv = carried_placement(&t.map(|k| rest[k]), src.local_normal, src.uv.as_ref(), &t.map(|k| local[k]), face.normal);
    })
}

/// Replace the cloth's source with `mesh` (made by [`cloth_mesh`] with the transform of the
/// context `path`): the source group goes, or the face with the edges no other face uses. The
/// cloth becomes a new group of `path`.
pub fn place_cloth(model: &mut Model, path: &[InstanceId], src: &ClothSource, mesh: Mesh) -> Option<InstanceId> {
    match src.group {
        Some(g) => {
            model.remove_instance(path, g);
        }
        None => {
            let m = model.mesh_at_mut(&src.path)?;
            let mut gone = vec![EntityRef::Face(src.face)];
            gone.extend(m.face_edges(src.face).into_iter().filter(|e| m.edge_faces(*e).len() == 1).map(EntityRef::Edge));
            m.erase(&gone);
        }
    }
    // i18n-ignore-next
    let id = model.add_group_mesh(path, mesh, "Cloth", DMat4::IDENTITY)?;
    if let Some(inst) = model.mesh_at_mut(path).and_then(|m| m.instances.get_mut(id)) {
        inst.material = src.material;
    }
    Some(id)
}

/// Swap the mesh of a cloth placed by [`place_cloth`].
pub fn set_cloth_mesh(model: &mut Model, path: &[InstanceId], cloth: InstanceId, mesh: Mesh) {
    if let Some(def) = model.mesh_at(path).and_then(|m| m.instances.get(cloth)).map(|i| i.def)
        && let Some(d) = model.defs.get_mut(def)
    {
        d.mesh = mesh;
    }
}
