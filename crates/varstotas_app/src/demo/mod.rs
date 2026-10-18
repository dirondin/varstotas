//! File → Open Demo: Country House — a furnished one-storey house on a lawn, built in code:
//! walls with openings, a gable roof with exposed beams, a living room with a fireplace, a
//! kitchen, a bedroom and a bathroom, a terrace and a garden. PBR materials with procedural
//! textures (`textures`), lights, render settings and scenes (day, dusk, rooms, plan).

mod textures;

use varstotas_kernel::glam::{DMat4, DQuat, DVec3};
use varstotas_kernel::{
    Definition, EFFECT_PRESETS, EntityRef, Environment, Face, Instance, Light, LightKind, Material, MaterialId, Mesh, Model, Pbr,
    RenderSettings, Scene, SceneSettings, SceneStyle, ShadowSettings, SkyKind, TagId,
};
use varstotas_kernel::{DefId, LengthUnit};
use std::collections::BTreeMap;
use textures::Maps;

/// Floor level (top of the slab).
const F: f64 = 450.0;
/// Height of the outer walls above the floor.
const H: f64 = 2700.0;
/// Top of the outer walls.
const ZW: f64 = F + H;
/// Roof slope (rise per run, ≈ 35°).
const SLOPE: f64 = 0.7;
const LX: f64 = 12000.0;
const LY: f64 = 8000.0;
/// Outer and partition wall thickness.
const WT: f64 = 250.0;
const PT: f64 = 120.0;

fn v(x: f64, y: f64, z: f64) -> DVec3 {
    DVec3::new(x, y, z)
}

/// Underside of the roof at `y` (the slabs rest on the inner edge of the walls).
fn roof_under(y: f64) -> f64 {
    ZW + (y.min(LY - y) - WT) * SLOPE
}

// ---- mesh primitives -------------------------------------------------------------------

/// Extrude the planar polygon `pts` with `holes` by `dir`.
fn prism(mesh: &mut Mesh, pts: &[DVec3], holes: &[Vec<DVec3>], dir: DVec3) {
    let before: Vec<_> = mesh.faces.keys().collect();
    let Some(f) = mesh.add_face(pts) else { return };
    let n0 = mesh.faces[f].normal;
    let mut cut = Vec::new();
    for h in holes {
        if let Some(hf) = mesh.add_face(h) {
            cut.push(EntityRef::Face(hf));
        }
    }
    mesh.erase(&cut);
    // The polygon's face: the largest new face in its plane.
    let on_plane = |mesh: &Mesh, id| {
        let face: &Face = &mesh.faces[id];
        face.normal.dot(n0).abs() > 0.999 && face.outer.iter().all(|&vi| (mesh.vertices[vi].pos - pts[0]).dot(n0).abs() < 0.01)
    };
    let Some(face) = mesh
        .faces
        .keys()
        .filter(|id| !before.contains(id) && on_plane(mesh, *id))
        .max_by(|a, b| mesh.face_area(*a).total_cmp(&mesh.face_area(*b)))
    else {
        return;
    };
    let d = dir.length();
    let d = if mesh.faces[face].normal.dot(dir) > 0.0 { d } else { -d };
    let _ = mesh.push_pull(face, d, false);
}

fn rect_z(a: DVec3, b: DVec3) -> [DVec3; 4] {
    [v(a.x, a.y, a.z), v(b.x, a.y, a.z), v(b.x, b.y, a.z), v(a.x, b.y, a.z)]
}

/// Axis-aligned box between corners `a` and `b`.
fn cuboid(mesh: &mut Mesh, a: DVec3, b: DVec3) {
    let (lo, hi) = (a.min(b), a.max(b));
    prism(mesh, &rect_z(lo, hi), &[], v(0.0, 0.0, hi.z - lo.z));
}

fn circle(c: DVec3, r: f64, n: usize) -> Vec<DVec3> {
    (0..n).map(|i| c + DVec3::new(r * (i as f64 * std::f64::consts::TAU / n as f64).cos(), r * (i as f64 * std::f64::consts::TAU / n as f64).sin(), 0.0)).collect()
}

/// Vertical cylinder standing on `c`.
fn cylinder(mesh: &mut Mesh, c: DVec3, r: f64, h: f64, n: usize) {
    prism(mesh, &circle(c, r, n), &[], v(0.0, 0.0, h));
}

/// Add `pts` as a face whose front looks away from `inside`.
fn face_out(mesh: &mut Mesh, pts: &[DVec3], inside: DVec3) {
    let c = pts.iter().copied().sum::<DVec3>() / pts.len() as f64;
    let n = varstotas_kernel::geom::newell(pts);
    if n.dot(c - inside) >= 0.0 {
        mesh.add_face_exact(pts);
    } else {
        let r: Vec<DVec3> = pts.iter().rev().copied().collect();
        mesh.add_face_exact(&r);
    }
}

/// A truncated cone on `c` from radius `r0` to `r1`, open (a lamp shade) or closed.
fn frustum(mesh: &mut Mesh, c: DVec3, r0: f64, r1: f64, h: f64, n: usize, closed: bool) {
    let (lo, hi) = (circle(c, r0, n), circle(c + v(0.0, 0.0, h), r1, n));
    let mid = c + v(0.0, 0.0, h / 2.0);
    for i in 0..n {
        let j = (i + 1) % n;
        face_out(mesh, &[lo[i], lo[j], hi[j], hi[i]], mid);
    }
    if closed {
        face_out(mesh, &lo, mid);
        face_out(mesh, &hi, mid);
    }
}

/// A lumpy ellipsoid of triangles (foliage, a bush): radii `r`, bumps `bump` (0..1).
fn blob(mesh: &mut Mesh, c: DVec3, r: DVec3, bump: f64, seed: u32) {
    let (nl, na) = (8usize, 14usize);
    let mut p = vec![vec![DVec3::ZERO; na]; nl + 1];
    for (i, row) in p.iter_mut().enumerate() {
        let lat = std::f64::consts::PI * (i as f64 / nl as f64 - 0.5);
        for (j, q) in row.iter_mut().enumerate() {
            let lon = std::f64::consts::TAU * j as f64 / na as f64;
            let k = if i == 0 || i == nl { 1.0 } else { 1.0 + bump * (rnd(seed, (i * 31 + j) as u32) - 0.5) };
            *q = c + DVec3::new(lat.cos() * lon.cos() * r.x, lat.cos() * lon.sin() * r.y, lat.sin() * r.z) * k;
        }
    }
    for i in 0..nl {
        for j in 0..na {
            let jn = (j + 1) % na;
            let (a, b, cc, d) = (p[i][j], p[i][jn], p[i + 1][jn], p[i + 1][j]);
            if i > 0 {
                face_out(mesh, &[a, b, cc], c);
            }
            if i + 1 < nl {
                face_out(mesh, &[a, cc, d], c);
            }
        }
    }
}

fn rnd(seed: u32, i: u32) -> f64 {
    let mut h = seed.wrapping_mul(0x9e37_79b9) ^ i.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    (h & 0xffff) as f64 / 65535.0
}

/// Paint the faces of `mesh` that `pick(normal, centre)` selects with `mat` (both sides).
fn paint(mesh: &mut Mesh, mat: MaterialId, pick: impl Fn(DVec3, DVec3) -> bool) {
    let ids: Vec<_> = mesh
        .faces
        .iter()
        .filter(|(_, f)| {
            let c = f.outer.iter().map(|&vi| mesh.vertices[vi].pos).sum::<DVec3>() / f.outer.len().max(1) as f64;
            pick(f.normal, c)
        })
        .map(|(id, _)| id)
        .collect();
    for id in ids {
        let f = &mut mesh.faces[id];
        f.front = Some(mat);
        f.back = Some(mat);
    }
}

// ---- the builder -----------------------------------------------------------------------

struct Builder {
    m: Model,
    mats: BTreeMap<&'static str, MaterialId>,
    roof_tag: TagId,
    furniture_tag: TagId,
    garden_tag: TagId,
}

impl Builder {
    fn mat(&self, name: &str) -> MaterialId {
        *self.mats.get(name).unwrap_or_else(|| panic!("demo material {name}"))
    }

    fn add_mat(&mut self, name: &'static str, color: [f32; 3], pbr: Pbr) {
        let id = self.m.add_material(name, [color[0], color[1], color[2], 1.0]);
        self.m.materials[id].pbr = Some(pbr);
        self.mats.insert(name, id);
    }

    /// A material with procedural maps: colour, and normal / roughness when drawn.
    fn add_tex(&mut self, name: &'static str, maps: Maps, mut pbr: Pbr) {
        let id = self.m.add_material(name, [maps.average[0], maps.average[1], maps.average[2], 1.0]);
        if maps.rough.is_some() {
            pbr.roughness = 1.0;
        }
        pbr.normal_map = maps.normal;
        pbr.roughness_map = maps.rough;
        let mat: &mut Material = &mut self.m.materials[id];
        mat.image = Some(maps.color);
        mat.pbr = Some(pbr);
        self.mats.insert(name, id);
    }

    fn def(&mut self, name: &str, is_group: bool) -> DefId {
        self.m.defs.insert(Definition {
            name: name.into(),
            mesh: Mesh::new(),
            is_group,
            guides: Default::default(),
            params: None,
            generated: None,
        })
    }

    fn instance(&mut self, parent: DefId, def: DefId, xf: DMat4, name: &str, mat: Option<MaterialId>, tag: Option<TagId>) {
        let inst = Instance { def, transform: xf, name: name.into(), material: mat, tag, hidden: false, locked: false };
        self.m.defs[parent].mesh.instances.insert(inst);
    }

    /// An empty group in `parent` to put parts into.
    fn group(&mut self, parent: DefId, name: &str, tag: Option<TagId>) -> DefId {
        let d = self.def(name, true);
        self.instance(parent, d, DMat4::IDENTITY, name, None, tag);
        d
    }

    /// A group in `parent` (identity transform) with geometry drawn by `build`, painted
    /// `mat`; curved surfaces are smoothed.
    fn solid(&mut self, parent: DefId, name: &str, mat: Option<&str>, build: impl FnOnce(&mut Mesh)) -> DefId {
        let d = self.def(name, true);
        let mesh = &mut self.m.defs[d].mesh;
        build(mesh);
        let edges: Vec<_> = mesh.edges.keys().collect();
        mesh.soften_by_angle(&edges, 36.0, true, false);
        let mat = mat.map(|n| self.mat(n));
        self.instance(parent, d, DMat4::IDENTITY, name, mat, None);
        d
    }

    fn part(&mut self, parent: DefId, mat: &str, a: DVec3, b: DVec3) {
        self.solid(parent, mat, Some(mat), |m| cuboid(m, a, b));
    }

    /// A component definition (no instance yet).
    fn component(&mut self, name: &str) -> DefId {
        self.def(name, false)
    }

    /// Place component `def` turned `deg` about the vertical so that its local footprint
    /// `0..size.x × 0..size.y` is centred on `at` (z = `at.z`).
    fn place(&mut self, parent: DefId, def: DefId, size: (f64, f64), at: DVec3, deg: f64, mat: Option<&str>) {
        let q = DQuat::from_rotation_z(deg.to_radians());
        let pos = at - q * v(size.0 / 2.0, size.1 / 2.0, 0.0);
        let name = self.m.defs[def].name.clone();
        let mat = mat.map(|n| self.mat(n));
        self.instance(parent, def, DMat4::from_rotation_translation(q, pos), &name, mat, Some(self.furniture_tag));
    }
}

/// The demo model.
pub fn country_house() -> Model {
    let mut m = Model::new();
    m.unit = LengthUnit::Millimeter;
    let roof_tag = m.add_tag("Roof");
    let furniture_tag = m.add_tag("Furniture");
    let garden_tag = m.add_tag("Garden");
    let mut b = Builder { m, mats: BTreeMap::new(), roof_tag, furniture_tag, garden_tag };
    materials(&mut b);
    let root = b.m.root;
    house(&mut b, root);
    let living = b.group(root, "Living Room", Some(b.furniture_tag));
    living_room(&mut b, living);
    let kitchen = b.group(root, "Kitchen", Some(b.furniture_tag));
    kitchen_furniture(&mut b, kitchen);
    let bedroom = b.group(root, "Bedroom", Some(b.furniture_tag));
    bedroom_furniture(&mut b, bedroom);
    let bath = b.group(root, "Bathroom", Some(b.furniture_tag));
    bathroom(&mut b, bath);
    let garden = b.group(root, "Garden", Some(b.garden_tag));
    garden_and_terrace(&mut b, garden);
    lights(&mut b.m);
    settings(&mut b.m);
    let mut m = b.m;
    m.rebuild_caches();
    m
}

fn materials(b: &mut Builder) {
    use textures as t;
    let p = |roughness: f32| Pbr { roughness, ..Pbr::default() };
    b.add_tex("Siding", t::siding([0.55, 0.62, 0.57]), p(0.6));
    b.add_tex("Plaster", t::plaster(), p(0.9));
    b.add_tex("Stone", t::stone(), p(0.85));
    b.add_tex("Roof Metal", t::roof_metal(), Pbr { metallic: 0.75, ..p(0.4) });
    b.add_tex("Pine Ceiling", t::pine_boards(), p(0.65));
    b.add_tex("Oak Floor", t::oak_floor(), p(0.45));
    b.add_tex("Larch Deck", t::decking(), p(0.75));
    b.add_tex("Floor Tile", t::tiles("floor_tile", 4.0, 1200.0, [0.78, 0.76, 0.72], [0.55, 0.53, 0.5]), p(0.2));
    b.add_tex("Wall Tile", t::subway(), p(0.1));
    b.add_tex("Marble", t::marble(), p(0.1));
    b.add_tex("Walnut", t::wood("walnut", [0.42, 0.27, 0.16], [0.24, 0.14, 0.08]), p(0.45));
    b.add_tex("Oak", t::wood("oak", [0.76, 0.58, 0.38], [0.58, 0.4, 0.24]), p(0.5));
    b.add_tex("Teak", t::wood("teak", [0.6, 0.42, 0.26], [0.45, 0.3, 0.18]), p(0.6));
    b.add_tex("Linen", t::fabric("linen", [0.78, 0.73, 0.64]), p(0.9));
    b.add_tex("Blue Wool", t::fabric("wool", [0.33, 0.42, 0.52]), p(0.95));
    b.add_tex("Cotton", t::fabric("cotton", [0.93, 0.91, 0.87]), p(0.9));
    b.add_tex("Rug", t::rug(), p(0.95));
    b.add_tex("Grass", t::grass(), p(0.9));
    b.add_tex("Gravel", t::gravel(), p(0.9));
    b.add_tex("Brick", t::brick(), p(0.85));
    b.add_mat("Fascia", [0.92, 0.92, 0.9], p(0.5));
    b.add_mat("Window Frame", [0.93, 0.93, 0.92], p(0.35));
    b.add_mat("Glass", [0.86, 0.93, 0.93], Pbr { transmission: 1.0, ior: 1.5, ..p(0.02) });
    let glass = b.mat("Glass");
    b.m.materials[glass].color[3] = 0.3;
    b.add_mat("Steel", [0.72, 0.72, 0.74], Pbr { metallic: 1.0, ..p(0.3) });
    b.add_mat("Chrome", [0.9, 0.9, 0.92], Pbr { metallic: 1.0, ..p(0.06) });
    b.add_mat("Mirror", [0.95, 0.95, 0.95], Pbr { metallic: 1.0, ..p(0.01) });
    b.add_mat("Black Metal", [0.06, 0.06, 0.06], Pbr { metallic: 0.9, ..p(0.45) });
    b.add_mat("Brass", [0.86, 0.66, 0.36], Pbr { metallic: 1.0, ..p(0.3) });
    b.add_mat("Ceramic", [0.95, 0.95, 0.94], p(0.08));
    b.add_mat("Black Glass", [0.03, 0.03, 0.03], p(0.04));
    b.add_mat("Cabinet", [0.34, 0.39, 0.31], p(0.5));
    b.add_mat("Leather", [0.45, 0.23, 0.11], p(0.4));
    b.add_mat("Soot", [0.05, 0.045, 0.04], p(1.0));
    b.add_mat("Concrete", [0.6, 0.6, 0.58], p(0.85));
    b.add_mat("Terracotta", [0.68, 0.37, 0.23], p(0.8));
    b.add_mat("Soil", [0.2, 0.14, 0.1], p(1.0));
    b.add_mat("Leaves", [0.16, 0.3, 0.09], p(0.75));
    b.add_mat("Dark Leaves", [0.11, 0.22, 0.08], p(0.75));
    b.add_mat("Bark", [0.3, 0.23, 0.17], p(0.9));
    let warm = [1.0, 0.78, 0.5];
    b.add_mat("Lamp Shade", [0.9, 0.85, 0.75], Pbr { emission: warm, emission_strength: 1.2, ..p(0.9) });
    b.add_mat("Bulb", [1.0, 0.95, 0.85], Pbr { emission: warm, emission_strength: 25.0, ..p(0.3) });
    b.add_mat("Fire", [1.0, 0.5, 0.15], Pbr { emission: [1.0, 0.38, 0.08], emission_strength: 6.0, ..p(0.8) });
    for (name, c) in [
        ("Book Red", [0.55, 0.15, 0.12]),
        ("Book Blue", [0.14, 0.24, 0.42]),
        ("Book Ochre", [0.78, 0.58, 0.22]),
        ("Book Green", [0.2, 0.36, 0.26]),
        ("Book Cream", [0.88, 0.84, 0.74]),
    ] {
        b.add_mat(name, c, p(0.6));
    }
}

// ---- the house -------------------------------------------------------------------------

/// An opening in a wall: along the wall from `a` to `b` (x for the front and back walls, y
/// for the sides), from `z0` to `z1` above the floor.
#[derive(Clone, Copy)]
struct Opening {
    a: f64,
    b: f64,
    z0: f64,
    z1: f64,
    kind: Fill,
}

#[derive(Clone, Copy, PartialEq)]
enum Fill {
    Window,
    GlassDoor,
    Door,
}

const fn op(a: f64, b: f64, z0: f64, z1: f64, kind: Fill) -> Opening {
    Opening { a, b, z0, z1, kind }
}

const FRONT: [Opening; 3] = [
    op(800.0, 3200.0, 300.0, 2400.0, Fill::Window),
    op(4300.0, 6300.0, 0.0, 2400.0, Fill::GlassDoor),
    op(8400.0, 10400.0, 700.0, 2300.0, Fill::Window),
];
const BACK: [Opening; 2] = [op(1800.0, 3000.0, 1000.0, 2100.0, Fill::Window), op(7500.0, 8500.0, 0.0, 2200.0, Fill::Door)];
const WEST: [Opening; 1] = [op(5600.0, 6800.0, 900.0, 2200.0, Fill::Window)];
const EAST: [Opening; 2] = [op(1300.0, 2900.0, 700.0, 2300.0, Fill::Window), op(6000.0, 7000.0, 1400.0, 2200.0, Fill::Window)];

fn house(b: &mut Builder, root: DefId) {
    let shell = b.group(root, "House", None);
    let (siding, oak) = (b.mat("Siding"), b.mat("Oak Floor"));

    // Floor slab on a stone plinth.
    let slab = b.solid(shell, "Floor Slab", Some("Stone"), |m| cuboid(m, v(0.0, 0.0, 0.0), v(LX, LY, F)));
    paint(&mut b.m.defs[slab].mesh, oak, |n, _| n.z > 0.5);

    // Outer walls: the outer face of each is siding, the rest plaster.
    let holes_x = |ops: &[Opening], y: f64| -> Vec<Vec<DVec3>> {
        ops.iter().map(|o| rect_xz(o.a, o.b, F + o.z0, F + o.z1, y).to_vec()).collect()
    };
    let holes_y = |ops: &[Opening], x: f64| -> Vec<Vec<DVec3>> {
        ops.iter().map(|o| rect_yz(o.a, o.b, F + o.z0, F + o.z1, x).to_vec()).collect()
    };
    let outer = move |n: DVec3, c: DVec3| {
        (n.y < -0.9 && c.y < 1.0) || (n.y > 0.9 && c.y > LY - 1.0) || (n.x < -0.9 && c.x < 1.0) || (n.x > 0.9 && c.x > LX - 1.0)
    };
    let walls = [
        ("Front Wall", rect_xz(0.0, LX, F, ZW, 0.0).to_vec(), holes_x(&FRONT, 0.0), v(0.0, WT, 0.0)),
        ("Back Wall", rect_xz(0.0, LX, F, ZW, LY).to_vec(), holes_x(&BACK, LY), v(0.0, -WT, 0.0)),
        ("West Wall", gable(0.0, WT, LY - WT), holes_y(&WEST, 0.0), v(WT, 0.0, 0.0)),
        ("East Wall", gable(LX, WT, LY - WT), holes_y(&EAST, LX), v(-WT, 0.0, 0.0)),
    ];
    for (name, pts, holes, dir) in walls {
        let d = b.solid(shell, name, Some("Plaster"), |m| prism(m, &pts, &holes, dir));
        paint(&mut b.m.defs[d].mesh, siding, outer);
    }

    // Partitions up to the roof, with door openings.
    let p1 = gable(7000.0 - PT / 2.0, WT, LY - WT);
    b.solid(shell, "Partition", Some("Plaster"), |m| prism(m, &p1, &[rect_yz(5200.0, 6100.0, F, F + 2100.0, 7000.0 - PT / 2.0).to_vec()], v(PT, 0.0, 0.0)));
    let y2 = 4500.0 - PT / 2.0;
    let p2 = rect_xz(7000.0 + PT / 2.0, LX - WT, F, roof_under(y2 + PT), y2);
    b.solid(shell, "Partition", Some("Plaster"), |m| prism(m, &p2, &[rect_xz(7400.0, 8300.0, F, F + 2100.0, y2).to_vec()], v(0.0, PT, 0.0)));
    let x3 = 9000.0 - PT / 2.0;
    let y3 = 4500.0 + PT / 2.0;
    let p3 = vec![v(x3, y3, F), v(x3, LY - WT, F), v(x3, LY - WT, roof_under(LY - WT)), v(x3, y3, roof_under(y3))];
    b.solid(shell, "Partition", Some("Plaster"), |m| prism(m, &p3, &[rect_yz(6300.0, 7100.0, F, F + 2100.0, x3).to_vec()], v(PT, 0.0, 0.0)));
    // Bathroom floor tiles.
    b.part(shell, "Floor Tile", v(9000.0 + PT / 2.0, 4500.0 + PT / 2.0, F), v(LX - WT, LY - WT, F + 8.0));

    // Windows and doors.
    for (ops, side) in [(&FRONT[..], 0), (&BACK[..], 1), (&WEST[..], 2), (&EAST[..], 3)] {
        for o in ops {
            let def = fill(b, o);
            // Local frame: width along +x, outside towards −y, the frame 70 deep.
            let (pos, deg) = match side {
                0 => (v(o.a, WT / 2.0 - 35.0, F + o.z0), 0.0),
                1 => (v(o.b, LY - WT / 2.0 + 35.0, F + o.z0), 180.0),
                2 => (v(WT / 2.0 - 35.0, o.b, F + o.z0), -90.0),
                _ => (v(LX - WT / 2.0 + 35.0, o.a, F + o.z0), 90.0),
            };
            let q = DQuat::from_rotation_z(f64::to_radians(deg));
            let name = b.m.defs[def].name.clone();
            b.instance(shell, def, DMat4::from_rotation_translation(q, pos), &name, None, None);
        }
    }

    // Roof: two slabs with a metal top, a pine ceiling below and white fascias.
    let roof = b.group(root, "Roof", Some(b.roof_tag));
    let (oh, gab, thick) = (600.0, 500.0, 250.0);
    let (metal, pine) = (b.mat("Roof Metal"), b.mat("Pine Ceiling"));
    for north in [false, true] {
        let y = |y: f64| if north { LY - y } else { y };
        let (ya, yb) = (-oh, LY / 2.0);
        let (za, zb) = (ZW + (ya - WT) * SLOPE, ZW + (yb - WT) * SLOPE);
        let pts = [v(-gab, y(ya), za), v(-gab, y(yb), zb), v(-gab, y(yb), zb + thick), v(-gab, y(ya), za + thick)];
        let d = b.solid(roof, "Roof Slab", Some("Fascia"), |m| prism(m, &pts, &[], v(LX + 2.0 * gab, 0.0, 0.0)));
        let mesh = &mut b.m.defs[d].mesh;
        paint(mesh, metal, |n, _| n.z > 0.3);
        paint(mesh, pine, |n, _| n.z < -0.3);
    }
    let zr = ZW + (LY / 2.0 - WT) * SLOPE + thick;
    b.solid(roof, "Ridge Cap", Some("Black Metal"), |m| {
        prism(m, &[v(-gab - 20.0, LY / 2.0 - 160.0, zr - 60.0), v(-gab - 20.0, LY / 2.0, zr + 40.0), v(-gab - 20.0, LY / 2.0 + 160.0, zr - 60.0)], &[], v(LX + 2.0 * gab + 40.0, 0.0, 0.0))
    });
    // Exposed timber: tie beams over the living room and a ridge beam.
    for x in [1500.0, 3500.0, 5500.0] {
        b.part(roof, "Walnut", v(x - 80.0, WT, ZW - 240.0), v(x + 80.0, LY - WT, ZW));
    }
    let zu = roof_under(LY / 2.0);
    b.part(roof, "Walnut", v(WT, LY / 2.0 - 100.0, zu - 320.0), v(LX - WT, LY / 2.0 + 100.0, zu));

    // Fireplace and chimney.
    let fp = b.group(shell, "Fireplace", None);
    b.part(fp, "Stone", v(WT, 1300.0, F), v(WT + 950.0, 3500.0, F + 80.0));
    let face = rect_yz(1500.0, 3300.0, F + 80.0, F + 1300.0, WT + 600.0);
    let opening = rect_yz(2000.0, 2800.0, F + 280.0, F + 900.0, WT + 600.0);
    b.solid(fp, "Fireplace", Some("Brick"), |m| prism(m, &face, &[opening.to_vec()], v(-600.0, 0.0, 0.0)));
    b.part(fp, "Soot", v(WT, 1990.0, F + 80.0), v(WT + 120.0, 2810.0, F + 1300.0));
    b.part(fp, "Soot", v(WT, 1990.0, F + 900.0), v(WT + 600.0, 2810.0, F + 1300.0));
    b.part(fp, "Soot", v(WT, 1990.0, F + 80.0), v(WT + 600.0, 2810.0, F + 280.0));
    b.part(fp, "Oak", v(WT, 1450.0, F + 1300.0), v(WT + 680.0, 3350.0, F + 1370.0));
    let top = ZW + (LY / 2.0 - WT) * SLOPE + 900.0;
    b.part(fp, "Brick", v(WT + 20.0, 2000.0, F + 1370.0), v(WT + 560.0, 2800.0, top));
    b.part(fp, "Concrete", v(WT - 30.0, 1950.0, top), v(WT + 610.0, 2850.0, top + 80.0));
    // Logs and the fire.
    for (i, y) in [2150.0, 2350.0, 2550.0].into_iter().enumerate() {
        let z = F + 330.0 + (i % 2) as f64 * 40.0;
        b.solid(fp, "Log", Some("Bark"), |m| {
            prism(m, &circle_yz(v(WT + 160.0, y, z), 55.0, 10), &[], v(380.0, 0.0, 0.0));
        });
    }
    b.solid(fp, "Fire", Some("Fire"), |m| blob(m, v(WT + 360.0, 2400.0, F + 470.0), v(170.0, 300.0, 160.0), 0.6, 7));
}

/// The window, glass door or door component for `o` (one per size).
fn fill(b: &mut Builder, o: &Opening) -> DefId {
    let (w, h) = (o.b - o.a, o.z1 - o.z0);
    let name = match o.kind {
        Fill::Window => format!("Window {:.0}×{:.0}", w, h),
        Fill::GlassDoor => format!("Glass Door {:.0}×{:.0}", w, h),
        Fill::Door => format!("Door {:.0}×{:.0}", w, h),
    };
    if let Some((id, _)) = b.m.defs.iter().find(|(_, d)| d.name == name) {
        return id;
    }
    let d = b.component(&name);
    let (fw, depth) = (60.0, 70.0);
    if o.kind == Fill::Door {
        // Frame and a walnut leaf with a brass handle.
        b.part(d, "Window Frame", v(0.0, 0.0, 0.0), v(fw, depth, h));
        b.part(d, "Window Frame", v(w - fw, 0.0, 0.0), v(w, depth, h));
        b.part(d, "Window Frame", v(0.0, 0.0, h - fw), v(w, depth, h));
        b.part(d, "Walnut", v(fw, 10.0, 0.0), v(w - fw, 60.0, h - fw));
        b.part(d, "Brass", v(w - fw - 120.0, -20.0, 1000.0), v(w - fw - 90.0, 10.0, 1150.0));
        return d;
    }
    b.part(d, "Window Frame", v(0.0, 0.0, 0.0), v(fw, depth, h));
    b.part(d, "Window Frame", v(w - fw, 0.0, 0.0), v(w, depth, h));
    b.part(d, "Window Frame", v(fw, 0.0, h - fw), v(w - fw, depth, h));
    b.part(d, "Window Frame", v(fw, 0.0, 0.0), v(w - fw, depth, fw));
    let panes = if w > 1500.0 { 2 } else { 1 };
    let pw = (w - 2.0 * fw - (panes - 1) as f64 * fw) / panes as f64;
    for i in 0..panes {
        let x0 = fw + i as f64 * (pw + fw);
        if i > 0 {
            b.part(d, "Window Frame", v(x0 - fw, 0.0, fw), v(x0, depth, h - fw));
        }
        b.part(d, "Glass", v(x0, 30.0, fw), v(x0 + pw, 40.0, h - fw));
    }
    if o.kind == Fill::Window {
        // A sill outside.
        b.part(d, "Black Metal", v(-40.0, -110.0, -30.0), v(w + 40.0, 0.0, 0.0));
    }
    d
}

fn rect_xz(x0: f64, x1: f64, z0: f64, z1: f64, y: f64) -> [DVec3; 4] {
    [v(x0, y, z0), v(x1, y, z0), v(x1, y, z1), v(x0, y, z1)]
}

fn rect_yz(y0: f64, y1: f64, z0: f64, z1: f64, x: f64) -> [DVec3; 4] {
    [v(x, y0, z0), v(x, y1, z0), v(x, y1, z1), v(x, y0, z1)]
}

fn circle_yz(c: DVec3, r: f64, n: usize) -> Vec<DVec3> {
    (0..n).map(|i| c + DVec3::new(0.0, r * (i as f64 * std::f64::consts::TAU / n as f64).cos(), r * (i as f64 * std::f64::consts::TAU / n as f64).sin())).collect()
}

/// A wall in the plane `x` from `y0` to `y1` up to the roof (a pentagon with the gable).
fn gable(x: f64, y0: f64, y1: f64) -> Vec<DVec3> {
    vec![v(x, y0, F), v(x, y1, F), v(x, y1, roof_under(y1)), v(x, LY / 2.0, roof_under(LY / 2.0)), v(x, y0, roof_under(y0))]
}

// ---- furniture -------------------------------------------------------------------------

fn sofa(b: &mut Builder) -> DefId {
    let d = b.component("Sofa");
    let w = 2600.0;
    for (x, y) in [(60.0, 160.0), (w - 60.0, 160.0), (60.0, 890.0), (w - 60.0, 890.0)] {
        b.solid(d, "Leg", Some("Walnut"), |m| cylinder(m, v(x, y, 0.0), 22.0, 150.0, 12));
    }
    b.part(d, "Linen", v(0.0, 100.0, 150.0), v(w, 950.0, 420.0));
    b.part(d, "Linen", v(0.0, 760.0, 420.0), v(w, 950.0, 850.0));
    b.part(d, "Linen", v(0.0, 100.0, 420.0), v(180.0, 760.0, 640.0));
    b.part(d, "Linen", v(w - 180.0, 100.0, 420.0), v(w, 760.0, 640.0));
    for (x0, x1) in [(185.0, w / 2.0 - 3.0), (w / 2.0 + 3.0, w - 185.0)] {
        b.part(d, "Linen", v(x0, 70.0, 420.0), v(x1, 755.0, 560.0));
        b.part(d, "Linen", v(x0 + 20.0, 610.0, 560.0), v(x1 - 20.0, 755.0, 820.0));
    }
    b.part(d, "Blue Wool", v(300.0, 520.0, 560.0), v(750.0, 600.0, 960.0));
    d
}

fn armchair(b: &mut Builder) -> DefId {
    let d = b.component("Armchair");
    for (x, y) in [(50.0, 50.0), (800.0, 50.0), (50.0, 800.0), (800.0, 800.0)] {
        b.solid(d, "Leg", Some("Walnut"), |m| cylinder(m, v(x, y, 0.0), 20.0, 160.0, 12));
    }
    b.part(d, "Leather", v(0.0, 0.0, 160.0), v(850.0, 850.0, 400.0));
    b.part(d, "Leather", v(0.0, 650.0, 400.0), v(850.0, 850.0, 820.0));
    b.part(d, "Leather", v(0.0, 0.0, 400.0), v(130.0, 650.0, 600.0));
    b.part(d, "Leather", v(720.0, 0.0, 400.0), v(850.0, 650.0, 600.0));
    b.part(d, "Leather", v(135.0, 20.0, 400.0), v(715.0, 645.0, 500.0));
    d
}

fn table(b: &mut Builder, name: &str, w: f64, depth: f64, h: f64, leg: f64) -> DefId {
    let d = b.component(name);
    let solid = |b: &mut Builder, a: DVec3, c: DVec3| {
        b.solid(d, "Part", None, |m| cuboid(m, a, c));
    };
    solid(b, v(0.0, 0.0, h - 40.0), v(w, depth, h));
    let i = 60.0;
    for (x, y) in [(i, i), (w - i - leg, i), (i, depth - i - leg), (w - i - leg, depth - i - leg)] {
        solid(b, v(x, y, 0.0), v(x + leg, y + leg, h - 40.0));
    }
    d
}

/// A chair facing −y (the back on +y); one material from the instance.
fn chair(b: &mut Builder) -> DefId {
    let d = b.component("Chair");
    let solid = |b: &mut Builder, a: DVec3, c: DVec3| {
        b.solid(d, "Part", None, |m| cuboid(m, a, c));
    };
    let (w, dp) = (450.0, 480.0);
    for (x, y) in [(0.0, 0.0), (w - 35.0, 0.0)] {
        solid(b, v(x, y, 0.0), v(x + 35.0, y + 35.0, 430.0));
    }
    for x in [0.0, w - 35.0] {
        solid(b, v(x, dp - 35.0, 0.0), v(x + 35.0, dp, 880.0));
    }
    solid(b, v(0.0, 0.0, 430.0), v(w, dp - 36.0, 465.0));
    solid(b, v(36.0, dp - 30.0, 600.0), v(w - 36.0, dp - 5.0, 860.0));
    d
}

/// A floor lamp with an open shade; returns the component and the shade centre height.
fn floor_lamp(b: &mut Builder) -> (DefId, f64) {
    let d = b.component("Floor Lamp");
    let c = v(200.0, 200.0, 0.0);
    b.solid(d, "Base", Some("Black Metal"), |m| cylinder(m, c, 170.0, 25.0, 24));
    b.solid(d, "Pole", Some("Brass"), |m| cylinder(m, c + v(0.0, 0.0, 25.0), 12.0, 1400.0, 10));
    b.solid(d, "Shade", Some("Lamp Shade"), |m| frustum(m, c + v(0.0, 0.0, 1330.0), 220.0, 160.0, 320.0, 28, false));
    b.solid(d, "Bulb", Some("Bulb"), |m| blob(m, c + v(0.0, 0.0, 1420.0), v(40.0, 40.0, 55.0), 0.0, 1));
    (d, 1480.0)
}

/// A pendant: a black cone shade with a bulb, hung by a cord `cord` long.
fn pendant(b: &mut Builder, cord: f64) -> DefId {
    let name = format!("Pendant {cord:.0}");
    let d = b.component(&name);
    let c = v(200.0, 200.0, 0.0);
    b.solid(d, "Shade", Some("Black Metal"), |m| frustum(m, c, 200.0, 50.0, 200.0, 28, false));
    b.solid(d, "Bulb", Some("Bulb"), |m| blob(m, c + v(0.0, 0.0, 110.0), v(45.0, 45.0, 55.0), 0.0, 1));
    b.solid(d, "Cord", Some("Black Metal"), |m| cylinder(m, c + v(0.0, 0.0, 200.0), 5.0, cord, 6));
    d
}

/// A potted plant `h` tall.
fn plant(b: &mut Builder, name: &str, h: f64, seed: u32) -> DefId {
    let d = b.component(name);
    let c = v(300.0, 300.0, 0.0);
    b.solid(d, "Pot", Some("Terracotta"), |m| frustum(m, c, 190.0, 240.0, 420.0, 24, true));
    b.solid(d, "Soil", Some("Soil"), |m| cylinder(m, c + v(0.0, 0.0, 420.0), 225.0, 5.0, 24));
    b.solid(d, "Stem", Some("Bark"), |m| cylinder(m, c + v(0.0, 0.0, 420.0), 18.0, h * 0.5, 8));
    for i in 0..4u32 {
        let a = i as f64 * 1.7 + seed as f64;
        let off = v(a.cos() * 120.0, a.sin() * 120.0, 0.0);
        let z = 420.0 + h * (0.35 + 0.15 * i as f64);
        let mat = if i % 2 == 0 { "Leaves" } else { "Dark Leaves" };
        b.solid(d, "Leaves", Some(mat), |m| blob(m, c + off + v(0.0, 0.0, z), v(260.0, 240.0, 220.0), 0.5, seed * 10 + i));
    }
    d
}

fn rug(b: &mut Builder) -> DefId {
    let d = b.component("Rug");
    b.part(d, "Rug", v(0.0, 0.0, 0.0), v(2400.0, 1700.0, 12.0));
    d
}

fn bookshelf(b: &mut Builder) -> DefId {
    let d = b.component("Bookshelf");
    let (w, dp, h, t) = (1000.0, 340.0, 2000.0, 22.0);
    b.part(d, "Walnut", v(0.0, 0.0, 0.0), v(t, dp, h));
    b.part(d, "Walnut", v(w - t, 0.0, 0.0), v(w, dp, h));
    b.part(d, "Walnut", v(t, dp - 12.0, 0.0), v(w - t, dp, h));
    let shelves = 5;
    let mut books: BTreeMap<&'static str, Vec<(DVec3, DVec3)>> = BTreeMap::new();
    let colors = ["Book Red", "Book Blue", "Book Ochre", "Book Green", "Book Cream"];
    for s in 0..=shelves {
        let z = s as f64 * (h - t) / shelves as f64;
        b.part(d, "Walnut", v(t, 0.0, z), v(w - t, dp - 12.0, z + t));
        if s == shelves {
            break;
        }
        // A row of books, leaving a gap at one end.
        let mut x = t + 4.0;
        let mut k = s as u32 * 97;
        let end = w - t - 80.0 - 120.0 * rnd(5, s as u32);
        while x < end {
            k += 1;
            let bw = 22.0 + 28.0 * rnd(11, k);
            let bh = 190.0 + 90.0 * rnd(12, k);
            let bd = 190.0 + 50.0 * rnd(13, k);
            if x + bw > end {
                break;
            }
            let col = colors[(rnd(14, k) * colors.len() as f64) as usize % colors.len()];
            books.entry(col).or_default().push((v(x, dp - 14.0 - bd, z + t), v(x + bw, dp - 14.0, z + t + bh)));
            x += bw + 2.0;
        }
    }
    for (col, list) in books {
        b.solid(d, "Books", Some(col), |m| {
            for (a, c) in list {
                cuboid(m, a, c);
            }
        });
    }
    d
}

fn living_room(b: &mut Builder, g: DefId) {
    let s = sofa(b);
    b.place(g, s, (2600.0, 950.0), v(3480.0, 2400.0, F), -90.0, None);
    let a = armchair(b);
    b.place(g, a, (850.0, 850.0), v(1900.0, 780.0, F), 180.0, None);
    b.place(g, a, (850.0, 850.0), v(1900.0, 4020.0, F), 0.0, None);
    let ct = table(b, "Coffee Table", 1200.0, 650.0, 400.0, 40.0);
    b.place(g, ct, (1200.0, 650.0), v(2150.0, 2400.0, F + 12.0), 90.0, Some("Walnut"));
    let r = rug(b);
    b.place(g, r, (2400.0, 1700.0), v(2150.0, 2400.0, F), 90.0, None);
    let (lamp, _) = floor_lamp(b);
    b.place(g, lamp, (400.0, 400.0), v(3550.0, 700.0, F), 0.0, None);
    let shelf = bookshelf(b);
    b.place(g, shelf, (1000.0, 340.0), v(3750.0, WT + 170.0, F), 180.0, None);
    let p = plant(b, "Fiddle Leaf Plant", 1500.0, 3);
    b.place(g, p, (600.0, 600.0), v(600.0, 4300.0, F), 0.0, None);
    b.place(g, p, (600.0, 600.0), v(6550.0, 650.0, F), 40.0, None);
    // Dining: a table for six under two pendants.
    let dt = table(b, "Dining Table", 2000.0, 950.0, 750.0, 80.0);
    b.place(g, dt, (2000.0, 950.0), v(5300.0, 2500.0, F), 0.0, Some("Oak"));
    let ch = chair(b);
    for x in [4700.0, 5300.0, 5900.0] {
        b.place(g, ch, (450.0, 480.0), v(x, 1850.0, F), 180.0, Some("Walnut"));
        b.place(g, ch, (450.0, 480.0), v(x, 3150.0, F), 0.0, Some("Walnut"));
    }
    for x in [4850.0, 5750.0] {
        let bottom = F + 1700.0;
        let pd = pendant(b, roof_under(2500.0) - bottom - 200.0);
        b.place(g, pd, (400.0, 400.0), v(x, 2500.0, bottom), 0.0, None);
    }
}

fn kitchen_furniture(b: &mut Builder, g: DefId) {
    let (y0, y1) = (LY - WT - 620.0, LY - WT);
    let (x0, x1) = (WT + 50.0, 5150.0);
    // Base cabinets with doors, plinth and a marble top.
    b.part(g, "Black Metal", v(x0, y0 + 60.0, F), v(x1, y1, F + 100.0));
    b.part(g, "Cabinet", v(x0, y0 + 20.0, F + 100.0), v(x1, y1, F + 870.0));
    let n = ((x1 - x0) / 600.0).floor() as usize;
    let dw = (x1 - x0) / n as f64;
    for i in 0..n {
        let a = x0 + i as f64 * dw;
        b.part(g, "Cabinet", v(a + 2.0, y0, F + 105.0), v(a + dw - 2.0, y0 + 20.0, F + 865.0));
        b.part(g, "Brass", v(a + dw / 2.0 - 80.0, y0 - 15.0, F + 820.0), v(a + dw / 2.0 + 80.0, y0, F + 835.0));
    }
    b.part(g, "Marble", v(x0, y0 - 20.0, F + 870.0), v(x1, y1, F + 910.0));
    // Sink, tap, hob and hood.
    b.part(g, "Steel", v(2150.0, y0 + 80.0, F + 910.0), v(2650.0, y0 + 480.0, F + 914.0));
    b.solid(g, "Tap", Some("Chrome"), |m| cylinder(m, v(2400.0, y1 - 80.0, F + 910.0), 18.0, 320.0, 12));
    b.part(g, "Chrome", v(2385.0, y1 - 300.0, F + 1200.0), v(2415.0, y1 - 70.0, F + 1230.0));
    b.part(g, "Black Glass", v(3700.0, y0 + 60.0, F + 910.0), v(4300.0, y0 + 560.0, F + 916.0));
    b.part(g, "Steel", v(3650.0, y0 + 100.0, F + 1600.0), v(4350.0, y1, F + 1700.0));
    b.part(g, "Steel", v(3880.0, y1 - 260.0, F + 1700.0), v(4120.0, y1, roof_under(y1) - 1.0));
    // Backsplash around the window, upper cabinets.
    let (wa, wb) = (BACK[0].a, BACK[0].b);
    let ty = y1 - 10.0;
    b.part(g, "Wall Tile", v(x0, ty, F + 910.0), v(wa, y1, F + 1500.0));
    b.part(g, "Wall Tile", v(wb, ty, F + 910.0), v(x1, y1, F + 1500.0));
    b.part(g, "Wall Tile", v(wa, ty, F + 910.0), v(wb, y1, F + BACK[0].z0));
    for (a, c) in [(x0, 1650.0), (4400.0, x1)] {
        b.part(g, "Cabinet", v(a, y1 - 350.0, F + 1500.0), v(c, y1, F + 2250.0));
        b.part(g, "Black Metal", v(a, y1 - 352.0, F + 1500.0), v(c, y1 - 349.0, F + 1505.0));
    }
    // Tall units: fridge and oven.
    let (t0, t1) = (5200.0, 7000.0 - PT / 2.0 - 30.0);
    b.part(g, "Cabinet", v(t0, y0 + 20.0, F), v(t1, y1, F + 2250.0));
    let tm = (t0 + t1) / 2.0;
    b.part(g, "Steel", v(t0 + 3.0, y0, F + 100.0), v(tm - 2.0, y0 + 20.0, F + 1300.0));
    b.part(g, "Steel", v(t0 + 3.0, y0, F + 1305.0), v(tm - 2.0, y0 + 20.0, F + 2245.0));
    b.part(g, "Cabinet", v(tm + 2.0, y0, F + 100.0), v(t1 - 3.0, y0 + 20.0, F + 850.0));
    b.part(g, "Black Glass", v(tm + 2.0, y0, F + 855.0), v(t1 - 3.0, y0 + 20.0, F + 1450.0));
    b.part(g, "Cabinet", v(tm + 2.0, y0, F + 1455.0), v(t1 - 3.0, y0 + 20.0, F + 2245.0));
    // Island with an overhang for the stools.
    let (ia, ib, iy0, iy1) = (1800.0, 4400.0, 5350.0, 6250.0);
    b.part(g, "Black Metal", v(ia + 40.0, iy0 + 300.0, F), v(ib - 40.0, iy1 - 40.0, F + 100.0));
    b.part(g, "Cabinet", v(ia, iy0 + 280.0, F + 100.0), v(ib, iy1, F + 880.0));
    b.part(g, "Marble", v(ia - 40.0, iy0, F + 880.0), v(ib + 40.0, iy1 + 30.0, F + 920.0));
    let st = b.component("Bar Stool");
    let c = v(200.0, 200.0, 0.0);
    b.solid(st, "Seat", Some("Walnut"), |m| cylinder(m, c + v(0.0, 0.0, 660.0), 190.0, 45.0, 28));
    b.solid(st, "Ring", Some("Black Metal"), |m| cylinder(m, c + v(0.0, 0.0, 250.0), 170.0, 15.0, 24));
    for k in 0..4 {
        let a = std::f64::consts::FRAC_PI_4 + k as f64 * std::f64::consts::FRAC_PI_2;
        b.solid(st, "Leg", Some("Black Metal"), |m| cylinder(m, c + v(a.cos() * 140.0, a.sin() * 140.0, 0.0), 12.0, 660.0, 8));
    }
    for x in [2400.0, 3100.0, 3800.0] {
        b.place(g, st, (400.0, 400.0), v(x, iy0 - 120.0, F), 0.0, None);
        let bottom = F + 1750.0;
        let pd = pendant(b, roof_under(5800.0) - bottom - 200.0);
        b.place(g, pd, (400.0, 400.0), v(x, 5800.0, bottom), 0.0, None);
    }
}

fn bedroom_furniture(b: &mut Builder, g: DefId) {
    let bed = b.component("Bed");
    let (w, l) = (1800.0, 2150.0);
    for (x, y) in [(40.0, 40.0), (w - 100.0, 40.0), (40.0, l - 160.0), (w - 100.0, l - 160.0)] {
        b.solid(bed, "Leg", Some("Walnut"), |m| cuboid(m, v(x, y, 0.0), v(x + 60.0, y + 60.0, 120.0)));
    }
    b.part(bed, "Walnut", v(0.0, 0.0, 120.0), v(w, l - 100.0, 320.0));
    b.part(bed, "Cotton", v(30.0, 30.0, 320.0), v(w - 30.0, l - 110.0, 540.0));
    b.part(bed, "Cotton", v(10.0, 10.0, 470.0), v(w - 10.0, 1500.0, 580.0));
    b.part(bed, "Blue Wool", v(-10.0, 150.0, 580.0), v(w + 10.0, 650.0, 600.0));
    b.part(bed, "Blue Wool", v(-10.0, 150.0, 300.0), v(0.0, 650.0, 580.0));
    b.part(bed, "Blue Wool", v(w, 150.0, 300.0), v(w + 10.0, 650.0, 580.0));
    for x in [150.0, 960.0] {
        b.part(bed, "Cotton", v(x, 1620.0, 540.0), v(x + 690.0, l - 130.0, 680.0));
    }
    b.part(bed, "Blue Wool", v(-60.0, l - 100.0, 0.0), v(w + 60.0, l, 1150.0));
    b.place(g, bed, (w, l), v(9200.0, 4440.0 - l / 2.0, F), 0.0, None);
    // Nightstands with lamps.
    let ns = b.component("Nightstand");
    b.part(ns, "Walnut", v(0.0, 0.0, 0.0), v(480.0, 400.0, 520.0));
    b.part(ns, "Brass", v(190.0, -12.0, 400.0), v(290.0, 0.0, 412.0));
    let lamp = b.component("Bedside Lamp");
    let c = v(150.0, 150.0, 0.0);
    b.solid(lamp, "Base", Some("Ceramic"), |m| frustum(m, c, 90.0, 60.0, 300.0, 24, true));
    b.solid(lamp, "Shade", Some("Lamp Shade"), |m| frustum(m, c + v(0.0, 0.0, 280.0), 170.0, 130.0, 230.0, 28, false));
    b.solid(lamp, "Bulb", Some("Bulb"), |m| blob(m, c + v(0.0, 0.0, 380.0), v(30.0, 30.0, 40.0), 0.0, 1));
    for x in [7980.0, 10420.0] {
        b.place(g, ns, (480.0, 400.0), v(x, 4240.0, F), 0.0, None);
        b.place(g, lamp, (300.0, 300.0), v(x, 4260.0, F + 520.0), 0.0, None);
    }
    let r = rug(b);
    b.place(g, r, (2400.0, 1700.0), v(9200.0, 2700.0, F), 0.0, None);
    // A wardrobe along the partition.
    let wd = b.component("Wardrobe");
    b.part(wd, "Oak", v(0.0, 0.0, 0.0), v(2000.0, 600.0, 2300.0));
    for i in 0..4 {
        let a = i as f64 * 500.0;
        b.part(wd, "Oak", v(a + 3.0, -18.0, 60.0), v(a + 497.0, 0.0, 2290.0));
        let hx = if i % 2 == 0 { a + 470.0 } else { a + 20.0 };
        b.part(wd, "Black Metal", v(hx, -40.0, 950.0), v(hx + 12.0, -18.0, 1350.0));
    }
    b.place(g, wd, (2000.0, 600.0), v(7000.0 + PT / 2.0 + 310.0, 1650.0, F), 90.0, None);
    let bench = table(b, "Bench", 1400.0, 420.0, 450.0, 50.0);
    b.place(g, bench, (1400.0, 420.0), v(9200.0, 1950.0, F), 0.0, Some("Oak"));
    let p = plant(b, "Olive Tree", 1200.0, 5);
    b.place(g, p, (600.0, 600.0), v(11350.0, 3950.0, F), 0.0, None);
    // Hall: a bench and coat hooks by the door.
    let hb = table(b, "Hall Bench", 1100.0, 380.0, 450.0, 45.0);
    b.place(g, hb, (1100.0, 380.0), v(7060.0 + 210.0, 6300.0, F), 90.0, Some("Oak"));
    for y in [5900.0, 6150.0, 6400.0, 6650.0] {
        b.part(g, "Brass", v(7060.0, y, F + 1700.0), v(7130.0, y + 15.0, F + 1715.0));
    }
}

fn bathroom(b: &mut Builder, g: DefId) {
    let (x0, y0) = (9000.0 + PT / 2.0, 4500.0 + PT / 2.0);
    let (x1, y1) = (LX - WT, LY - WT);
    let t = 10.0;
    let zt = F + 1400.0;
    // Tiled walls to 1.4 m.
    b.part(g, "Wall Tile", v(x1 - t, y0, F), v(x1, y1, zt));
    b.part(g, "Wall Tile", v(x0, y1 - t, F), v(x1 - t, y1, zt));
    b.part(g, "Wall Tile", v(x0, y0, F), v(x1 - t, y0 + t, zt));
    b.part(g, "Wall Tile", v(x0, y0 + t, F), v(x0 + t, 6300.0, zt));
    b.part(g, "Wall Tile", v(x0, 7100.0, F), v(x0 + t, y1 - t, zt));
    // A bath: a ceramic shell (a ring with a floor), and a tap.
    let tub = b.component("Bath");
    let (w, d, h) = (1700.0, 750.0, 560.0);
    let outer = rect_z(v(0.0, 0.0, 0.0), v(w, d, 0.0));
    let inner = rect_z(v(70.0, 70.0, 0.0), v(w - 70.0, d - 70.0, 0.0));
    b.solid(tub, "Shell", Some("Ceramic"), |m| prism(m, &outer, &[inner.to_vec()], v(0.0, 0.0, h)));
    b.part(tub, "Ceramic", v(70.0, 70.0, 0.0), v(w - 70.0, d - 70.0, 120.0));
    b.solid(tub, "Tap", Some("Chrome"), |m| cylinder(m, v(w - 35.0, d / 2.0, h), 16.0, 200.0, 12));
    b.part(tub, "Chrome", v(w - 180.0, d / 2.0 - 12.0, h + 180.0), v(w - 35.0, d / 2.0 + 12.0, h + 205.0));
    b.place(g, tub, (w, d), v(x1 - t - d / 2.0, y0 + t + 50.0 + w / 2.0, F + 8.0), 90.0, None);
    // A vanity with a basin, a tap and a mirror.
    b.part(g, "Oak", v(9500.0, y1 - t - 480.0, F + 300.0), v(10700.0, y1 - t, F + 820.0));
    b.part(g, "Ceramic", v(9480.0, y1 - t - 500.0, F + 820.0), v(10720.0, y1 - t, F + 850.0));
    let bowl = rect_z(v(9850.0, y1 - t - 440.0, F + 850.0), v(10350.0, y1 - t - 90.0, F + 850.0));
    let hole = rect_z(v(9880.0, y1 - t - 410.0, F + 850.0), v(10320.0, y1 - t - 120.0, F + 850.0));
    b.solid(g, "Basin", Some("Ceramic"), |m| prism(m, &bowl, &[hole.to_vec()], v(0.0, 0.0, 140.0)));
    b.solid(g, "Tap", Some("Chrome"), |m| cylinder(m, v(10100.0, y1 - t - 50.0, F + 850.0), 15.0, 260.0, 12));
    b.part(g, "Chrome", v(10088.0, y1 - t - 200.0, F + 1085.0), v(10112.0, y1 - t - 40.0, F + 1110.0));
    b.part(g, "Mirror", v(9600.0, y1 - t - 15.0, F + 1150.0), v(10600.0, y1 - t, F + 1950.0));
    // A toilet against the bedroom partition.
    let wc = b.component("Toilet");
    b.part(wc, "Ceramic", v(0.0, 0.0, 400.0), v(400.0, 200.0, 800.0));
    b.solid(wc, "Bowl", Some("Ceramic"), |m| frustum(m, v(200.0, 420.0, 0.0), 150.0, 190.0, 400.0, 28, true));
    b.part(wc, "Ceramic", v(100.0, 150.0, 0.0), v(300.0, 300.0, 400.0));
    b.place(g, wc, (400.0, 620.0), v(9750.0, y0 + t + 310.0, F + 8.0), 0.0, None);
    let p = plant(b, "Fern", 500.0, 9);
    b.place(g, p, (600.0, 600.0), v(10950.0, 7350.0, F + 8.0), 0.0, None);
}

fn garden_and_terrace(b: &mut Builder, g: DefId) {
    // Lawn, a gravel path, the terrace with steps and the entrance landing.
    b.solid(g, "Lawn", Some("Grass"), |m| {
        let r = 150_000.0;
        m.add_face_exact(&[v(-r, -r, 2.0), v(r, -r, 2.0), v(r, r, 2.0), v(-r, r, 2.0)]);
    });
    b.part(g, "Gravel", v(3000.0, -16000.0, 0.0), v(4000.0, -3700.0, 8.0));
    b.part(g, "Gravel", v(7400.0, LY + 1300.0, 0.0), v(8600.0, LY + 9000.0, 8.0));
    b.part(g, "Larch Deck", v(0.0, -3200.0, 0.0), v(7000.0, 0.0, F - 30.0));
    b.part(g, "Larch Deck", v(2700.0, -3450.0, 0.0), v(4300.0, -3200.0, 280.0));
    b.part(g, "Larch Deck", v(2700.0, -3700.0, 0.0), v(4300.0, -3450.0, 140.0));
    b.part(g, "Concrete", v(7300.0, LY, 0.0), v(8700.0, LY + 1000.0, F - 30.0));
    b.part(g, "Concrete", v(7300.0, LY + 1000.0, 0.0), v(8700.0, LY + 1300.0, 210.0));
    // Terrace furniture.
    let gt = table(b, "Garden Table", 1600.0, 900.0, 740.0, 70.0);
    b.place(g, gt, (1600.0, 900.0), v(1900.0, -1700.0, F - 30.0), 0.0, Some("Teak"));
    let ch = b.m.defs.iter().find(|(_, d)| d.name == "Chair").map(|(id, _)| id).unwrap_or_else(|| chair(b));
    for x in [1500.0, 2300.0] {
        b.place(g, ch, (450.0, 480.0), v(x, -2430.0, F - 30.0), 180.0, Some("Teak"));
        b.place(g, ch, (450.0, 480.0), v(x, -970.0, F - 30.0), 0.0, Some("Teak"));
    }
    let p = plant(b, "Boxwood Planter", 700.0, 13);
    for x in [400.0, 6600.0] {
        b.place(g, p, (600.0, 600.0), v(x, -2850.0, F - 30.0), 0.0, None);
    }
    // Wall lights by the doors.
    let wl = b.component("Wall Light");
    b.part(wl, "Black Metal", v(0.0, 0.0, 0.0), v(140.0, 120.0, 260.0));
    b.part(wl, "Bulb", v(20.0, -2.0, 30.0), v(120.0, 0.0, 230.0));
    b.place(g, wl, (140.0, 120.0), v(6600.0, -60.0, F + 1900.0), 0.0, None);
    b.place(g, wl, (140.0, 120.0), v(8800.0, LY + 60.0, F + 1900.0), 180.0, None);
    // Trees and shrubs.
    for (i, (x, y, s)) in [(-6000.0, -5000.0, 1.0), (17500.0, -3500.0, 1.2), (16500.0, 12000.0, 0.9), (-5500.0, 13000.0, 1.1), (11000.0, -14000.0, 0.8)].into_iter().enumerate() {
        let t = b.group(g, "Tree", None);
        let s: f64 = s;
        b.solid(t, "Trunk", Some("Bark"), |m| frustum(m, v(x, y, 0.0), 220.0 * s, 140.0 * s, 3800.0 * s, 12, true));
        for k in 0..3u32 {
            let a = k as f64 * 2.1 + i as f64;
            let c = v(x + a.cos() * 900.0 * s, y + a.sin() * 900.0 * s, (4200.0 + 700.0 * k as f64) * s);
            let mat = if k == 1 { "Dark Leaves" } else { "Leaves" };
            b.solid(t, "Crown", Some(mat), |m| blob(m, c, v(2000.0, 1900.0, 1700.0) * s, 0.35, i as u32 * 7 + k));
        }
    }
    for (i, x) in [7600.0, 8800.0, 10000.0, 11200.0].into_iter().enumerate() {
        b.solid(g, "Shrub", Some(if i % 2 == 0 { "Dark Leaves" } else { "Leaves" }), |m| {
            blob(m, v(x, -700.0, 380.0), v(560.0, 480.0, 420.0), 0.3, 40 + i as u32)
        });
    }
}

// ---- lights, environment, scenes -------------------------------------------------------

fn lights(m: &mut Model) {
    let warm = [1.0, 0.8, 0.55];
    let mut add = |name: &str, kind: LightKind, p: DVec3, power: f32, color: [f32; 3], radius: f32| {
        let mut l = Light::new(kind, p);
        l.name = name.into();
        l.power = power;
        l.color = color;
        l.radius = radius;
        m.lights.push(l.sanitized());
    };
    add("Fireplace", LightKind::Point, v(WT + 380.0, 2400.0, F + 520.0), 220.0, [1.0, 0.5, 0.2], 120.0);
    add("Floor Lamp", LightKind::Point, v(3550.0, 700.0, F + 1480.0), 60.0, warm, 50.0);
    for x in [4850.0, 5750.0] {
        add("Dining Pendant", LightKind::Point, v(x, 2500.0, F + 1700.0 - 40.0), 50.0, warm, 40.0);
    }
    for x in [2400.0, 3100.0, 3800.0] {
        add("Island Pendant", LightKind::Point, v(x, 5800.0, F + 1750.0 - 40.0), 35.0, warm, 40.0);
    }
    for x in [7980.0, 10420.0] {
        add("Bedside Lamp", LightKind::Point, v(x, 4260.0, F + 520.0 + 400.0), 25.0, warm, 30.0);
    }
    add("Hall", LightKind::Point, v(8000.0, 6200.0, ZW - 100.0), 60.0, warm, 60.0);
    add("Bathroom", LightKind::Point, v(10400.0, 6200.0, ZW - 100.0), 70.0, [1.0, 0.92, 0.8], 60.0);
    add("Terrace Light", LightKind::Point, v(6600.0, -250.0, F + 2030.0), 30.0, warm, 40.0);
    add("Entrance Light", LightKind::Point, v(8800.0, LY + 250.0, F + 2030.0), 30.0, warm, 40.0);
}

fn settings(m: &mut Model) {
    let mut env = Environment { sky: SkyKind::Physical, turbidity: 2.5, sun: 3.0, exposure: 0.0, ..Environment::default() };
    if let Some(fx) = env.effects.with_preset(EFFECT_PRESETS[2]) {
        env.effects = fx;
    }
    m.environment = env;
    let day = ShadowSettings { enabled: true, day: varstotas_kernel::shadows::day_of_year(6, 20), minutes: 16 * 60, ..ShadowSettings::default() };
    m.shadows = day;
    m.render = RenderSettings { width: 1280, height: 720, samples: 256, bounces: 6, denoise: true, match_view: true, ..RenderSettings::default() };
    m.scene_settings = SceneSettings { transitions: true, transition_secs: 3.0, delay_secs: 1.5 };

    let style = Some(SceneStyle { face_style: "realistic".into(), xray: false, edges: false, profiles: false });
    let interior = Environment { exposure: 1.3, ..env };
    let dusk_env = Environment { exposure: 2.2, turbidity: 3.5, ..env };
    let dusk = ShadowSettings { minutes: 21 * 60 + 40, ..day };
    let morning = ShadowSettings { minutes: 10 * 60 + 30, ..day };
    let roof = m.tags.iter().find(|(_, t)| t.name == "Roof").map(|(id, _)| id);
    let mut scene = |name: &str, eye: DVec3, target: DVec3, fov: f64, env: Environment, sh: ShadowSettings| {
        let mut s = Scene::camera(name, eye, target, DVec3::Z, fov, false);
        s.style = style.clone();
        s.environment = Some(env);
        s.shadows = Some(sh);
        s.hidden_tags = Some(Vec::new());
        m.scenes.push(s);
    };
    scene("Exterior", v(-7500.0, -13500.0, 1900.0), v(6000.0, 3000.0, 2300.0), 38.0, env, day);
    scene("Terrace", v(10500.0, -8500.0, 1800.0), v(3000.0, -800.0, 1300.0), 45.0, env, day);
    scene("Living Room", v(6600.0, 4300.0, F + 1450.0), v(600.0, 2200.0, F + 1000.0), 60.0, interior, day);
    scene("Kitchen", v(5900.0, 3300.0, F + 1650.0), v(2600.0, 7300.0, F + 1000.0), 60.0, interior, morning);
    scene("Bedroom", v(11350.0, 500.0, F + 1550.0), v(8600.0, 4200.0, F + 800.0), 62.0, interior, morning);
    scene("Bathroom", v(9300.0, 4800.0, F + 1650.0), v(11200.0, 7400.0, F + 800.0), 70.0, Environment { exposure: 0.5, ..env }, morning);
    scene("Dusk", v(-7500.0, -13500.0, 1900.0), v(6000.0, 3000.0, 2300.0), 38.0, dusk_env, dusk);
    let mut plan = Scene::camera("Plan", v(6000.0, 2000.0, 38000.0), v(6000.0, 2000.0, 0.0), DVec3::Y, 35.0, false);
    plan.style = style;
    let mut plan_env = Environment { exposure: 0.3, ..env };
    plan_env.effects.bloom = false;
    plan.environment = Some(plan_env);
    plan.shadows = Some(ShadowSettings { minutes: 13 * 60, ..day });
    plan.hidden_tags = Some(roof.into_iter().collect());
    plan.in_animation = false;
    m.scenes.push(plan);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_demo_house_is_complete() {
        let m = country_house();
        assert!(m.materials.len() > 40);
        assert!(m.materials.values().filter(|x| x.image.is_some()).count() >= 18);
        assert_eq!(m.scenes.len(), 8);
        assert!(m.lights.len() >= 12);
        let b = m.bounds().expect("bounds");
        assert!(b.max.z > ZW + 2500.0, "{b:?}");
        // Every group got geometry; the outer walls are solids with their openings.
        for (_, d) in &m.defs {
            if d.is_group && d.mesh.instances.is_empty() {
                assert!(!d.mesh.faces.is_empty(), "empty group {}", d.name);
            }
        }
        let front = m.defs.values().find(|d| d.name == "Front Wall").unwrap();
        let holes: usize = front.mesh.faces.values().map(|f| f.holes.len()).sum();
        assert_eq!(holes, 4, "two windows through two faces (the door notches the outline)");
        // The file round-trips.
        let dir = std::env::temp_dir().join(format!("varstotas_demo_{}.vrst", std::process::id()));
        varstotas_kernel::io::save(&m, &dir).unwrap();
        let back = varstotas_kernel::io::load(&dir).unwrap();
        let _ = std::fs::remove_file(&dir);
        assert_eq!(back.materials.len(), m.materials.len());
        assert_eq!(back.scenes.len(), 8);
    }
}
