//! Solid Tools: Union, Intersect, Subtract, Trim, Split and Outer Shell of solid groups /
//! components (closed meshes: every edge bounds exactly two faces).
//!
//! Both solids are copied into one scratch mesh (welded), every face is intersected with every
//! other, and each resulting piece is classified: which solid's surface it lies on and whether
//! it is inside the other solid (ray parity, majority of three rays). The kept pieces are
//! oriented outward and rebuilt into a new mesh; edges between coplanar pieces are erased.

use crate::geom::{self, PointLoc};
use crate::mesh::Mesh;
use crate::model::{Definition, Model};
use crate::tessellate::triangulate_face;
use crate::{EntityRef, FaceId, Instance, InstanceId, MaterialId, OpError, OpResult, TOL, VertexId};
use glam::{DMat4, DVec2, DVec3};
use std::collections::HashMap;
use varstotas_i18n::tr;

/// A Solid Tools operation. "First" and "second" are in the order the solids were picked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolidOp {
    /// Everything inside either solid; the inner faces go.
    Union,
    /// Only the overlap.
    Intersect,
    /// The second minus the first; the first is removed.
    Subtract,
    /// The second minus the first; the first stays.
    Trim,
    /// Three pieces: first minus second, second minus first and the overlap.
    Split,
    /// Like Union (only the outside faces).
    OuterShell,
}

impl SolidOp {
    pub const ALL: [SolidOp; 6] =
        [SolidOp::OuterShell, SolidOp::Intersect, SolidOp::Union, SolidOp::Subtract, SolidOp::Trim, SolidOp::Split];

    pub fn name(self) -> String {
        varstotas_i18n::tr(match self {
            SolidOp::Union => "solid-union",
            SolidOp::Intersect => "solid-intersect",
            SolidOp::Subtract => "solid-subtract",
            SolidOp::Trim => "solid-trim",
            SolidOp::Split => "solid-split",
            SolidOp::OuterShell => "solid-outer-shell",
        })
    }
}

/// Boolean of two solids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Bool {
    Union,
    Intersect,
    /// First minus second.
    Minus,
}

impl Mesh {
    /// Closed mesh: has faces, no nested instances, every edge bounds exactly two faces.
    pub fn is_solid(&self) -> bool {
        !self.faces.is_empty()
            && self.instances.is_empty()
            && self.edges.keys().all(|e| self.cache.edge_faces.get(e).is_some_and(|l| l.len() == 2))
    }

    /// Volume enclosed by the faces (meaningful for solids; positive when faces look out).
    pub fn volume(&self) -> f64 {
        let mut v = 0.0;
        for f in self.faces.keys() {
            for t in triangulate_face(self, f) {
                v += t[0].dot(t[1].cross(t[2])) / 6.0;
            }
        }
        v
    }
}

/// A face polygon in world (context) coordinates, for "lies on the surface" tests.
struct SurfPoly {
    normal: DVec3,
    origin: DVec3,
    u: DVec3,
    v: DVec3,
    outer: Vec<DVec2>,
    holes: Vec<Vec<DVec2>>,
}

/// One solid placed by a transform: its surface polygons and triangles.
struct Solid {
    polys: Vec<SurfPoly>,
    tris: Vec<[DVec3; 3]>,
    scale: f64,
}

impl Solid {
    fn new(mesh: &Mesh, t: &DMat4) -> Self {
        let mirror = t.determinant() < 0.0;
        let mut polys = Vec::new();
        let mut tris = Vec::new();
        let mut lo = DVec3::splat(f64::MAX);
        let mut hi = DVec3::splat(f64::MIN);
        for (f, face) in &mesh.faces {
            let tp = |l: &[VertexId]| -> Vec<DVec3> { l.iter().map(|v| t.transform_point3(mesh.vertices[*v].pos)).collect() };
            let outer = tp(&face.outer);
            for p in &outer {
                lo = lo.min(*p);
                hi = hi.max(*p);
            }
            let mut n = geom::newell(&outer).normalize_or_zero();
            if mirror {
                n = -n;
            }
            let (u, v) = geom::plane_basis(n);
            let o = outer[0];
            let to2 = |p: &DVec3| DVec2::new((*p - o).dot(u), (*p - o).dot(v));
            polys.push(SurfPoly {
                normal: n,
                origin: o,
                u,
                v,
                outer: outer.iter().map(to2).collect(),
                holes: face.holes.iter().map(|h| tp(h).iter().map(to2).collect()).collect(),
            });
            for tri in triangulate_face(mesh, f) {
                tris.push(tri.map(|p| t.transform_point3(p)));
            }
        }
        Self { polys, tris, scale: (hi - lo).length().max(1.0) }
    }

    fn tol(&self) -> f64 {
        (TOL * 10.0).max(self.scale * 1e-9)
    }

    /// Outward normal of the surface at `p` if `p` lies on it.
    fn on_surface(&self, p: DVec3) -> Option<DVec3> {
        let tol = self.tol();
        self.polys.iter().find_map(|s| {
            if (p - s.origin).dot(s.normal).abs() > tol {
                return None;
            }
            let q = DVec2::new((p - s.origin).dot(s.u), (p - s.origin).dot(s.v));
            (geom::point_in_region(q, &s.outer, &s.holes, tol) != PointLoc::Outside).then_some(s.normal)
        })
    }

    /// Is `p` (not on the surface) inside? Majority of three ray parities.
    fn contains(&self, p: DVec3) -> bool {
        const DIRS: [DVec3; 3] = [
            DVec3::new(0.571_3, 0.618_033_988_7, 0.533_589_7),
            DVec3::new(-0.414_213_562_3, 0.732_050_807_5, -0.541_381_265_1),
            DVec3::new(0.301_1, -0.477_121_254_7, 0.825_745_754_2),
        ];
        let inside = DIRS.iter().filter(|d| self.crossings(p, d.normalize()) % 2 == 1).count();
        inside >= 2
    }

    fn crossings(&self, p: DVec3, d: DVec3) -> usize {
        let eps = self.tol();
        self.tris
            .iter()
            .filter(|t| {
                // Möller–Trumbore.
                let (e1, e2) = (t[1] - t[0], t[2] - t[0]);
                let h = d.cross(e2);
                let a = e1.dot(h);
                if a.abs() < 1e-12 {
                    return false;
                }
                let f = 1.0 / a;
                let s = p - t[0];
                let u = f * s.dot(h);
                if !(0.0..=1.0).contains(&u) {
                    return false;
                }
                let q = s.cross(e1);
                let v = f * d.dot(q);
                if v < 0.0 || u + v > 1.0 {
                    return false;
                }
                f * e2.dot(q) > eps
            })
            .count()
    }
}

/// Copy `src` (placed by `t`, faces without a material get `mat`) into `dst`. Returns the
/// vertices and faces that were added or reused.
fn append(dst: &mut Mesh, src: &Mesh, t: &DMat4, mat: Option<MaterialId>) -> (Vec<VertexId>, Vec<FaceId>) {
    let mirror = t.determinant() < 0.0;
    let mut vmap: HashMap<VertexId, VertexId> = HashMap::new();
    for (v, vd) in &src.vertices {
        vmap.insert(v, dst.v_get_or_new(t.transform_point3(vd.pos)));
    }
    for ed in src.edges.values() {
        let (a, b) = (vmap[&ed.v[0]], vmap[&ed.v[1]]);
        if a != b && dst.edge_between(a, b).is_none() {
            let mut tmpl = ed.clone();
            tmpl.curve = None;
            dst.e_new(a, b, Some(&tmpl));
        }
    }
    let mut faces = Vec::new();
    for face in src.faces.values() {
        let mut outer: Vec<VertexId> = face.outer.iter().map(|v| vmap[v]).collect();
        let mut holes: Vec<Vec<VertexId>> = face.holes.iter().map(|h| h.iter().map(|v| vmap[v]).collect()).collect();
        if mirror {
            outer.reverse();
            holes.iter_mut().for_each(|h| h.reverse());
        }
        if let Some(ex) = dst.find_face_with_outer(&outer) {
            faces.push(ex);
            continue;
        }
        let mut tmpl = face.clone();
        tmpl.normal = t.transform_vector3(face.normal).normalize_or(face.normal) * if mirror { -1.0 } else { 1.0 };
        tmpl.uv = None;
        if tmpl.front.is_none() {
            tmpl.front = mat;
        }
        faces.push(dst.f_new(outer, holes, &tmpl));
    }
    let mut verts: Vec<VertexId> = vmap.values().copied().collect();
    verts.sort();
    verts.dedup();
    (verts, faces)
}

/// Boolean `op` of solid `a` (placed by `ta`) and `b` (by `tb`); the result is in the
/// coordinates the transforms map to.
fn boolean(a: (&Mesh, &DMat4, Option<MaterialId>), b: (&Mesh, &DMat4, Option<MaterialId>), op: Bool) -> Mesh {
    let (sa, sb) = (Solid::new(a.0, a.1), Solid::new(b.0, b.1));
    let mut m = Mesh::new();
    let (mut verts, mut faces) = append(&mut m, a.0, a.1, a.2);
    let (vb, fb) = append(&mut m, b.0, b.1, b.2);
    verts.extend(vb);
    faces.extend(fb);
    verts.sort();
    verts.dedup();
    m.absorb(&verts, &faces);
    m.intersect_all();

    // Classify every piece: keep it with the wanted outward normal, or drop it.
    let mut keep: Vec<(FaceId, DVec3)> = Vec::new();
    for f in m.faces.keys() {
        let (pr, outer, holes) = m.face_2d(f);
        let p = pr.to3(crate::topology::interior_sample(&outer, &holes));
        let (na, nb) = (sa.on_surface(p), sb.on_surface(p));
        let normal = match (na, nb, op) {
            (Some(n), None, Bool::Union) => (!sb.contains(p)).then_some(n),
            (None, Some(n), Bool::Union) => (!sa.contains(p)).then_some(n),
            (Some(n), None, Bool::Intersect) => sb.contains(p).then_some(n),
            (None, Some(n), Bool::Intersect) => sa.contains(p).then_some(n),
            (Some(n), None, Bool::Minus) => (!sb.contains(p)).then_some(n),
            (None, Some(n), Bool::Minus) => sa.contains(p).then_some(-n),
            // Coplanar pieces: facing the same way they are one surface of a union or an
            // overlap; facing each other they are where the solids touch.
            (Some(x), Some(y), Bool::Union | Bool::Intersect) => (x.dot(y) > 0.0).then_some(x),
            (Some(x), Some(y), Bool::Minus) => (x.dot(y) < 0.0).then_some(x),
            (None, None, _) => None,
        };
        if let Some(n) = normal {
            keep.push((f, n));
        }
    }

    // Rebuild from the kept pieces.
    let mut r = Mesh::new();
    for (f, n) in keep {
        let face = m.faces[f].clone();
        let pos = |l: &[VertexId]| -> Vec<DVec3> { l.iter().map(|v| m.vertices[*v].pos).collect() };
        let flip = geom::newell(&pos(&face.outer)).dot(n) < 0.0;
        let mut ids = |l: &[VertexId]| -> Vec<VertexId> {
            let mut out: Vec<VertexId> = pos(l).into_iter().map(|p| r.v_get_or_new(p)).collect();
            if flip {
                out.reverse();
            }
            out
        };
        let outer = ids(&face.outer);
        let holes: Vec<Vec<VertexId>> = face.holes.iter().map(|h| ids(h)).collect();
        let mut tmpl = face.clone();
        tmpl.normal = n;
        if flip {
            std::mem::swap(&mut tmpl.front, &mut tmpl.back);
            // A flipped piece of the cutting solid shows that solid's paint inside the cut.
            if tmpl.front.is_none() {
                tmpl.front = face.front;
            }
        }
        r.f_new(outer, holes, &tmpl);
    }
    // Edge attributes from the scratch mesh (soft / smooth / hidden).
    let ids: Vec<_> = r.edges.keys().collect();
    for e in &ids {
        let (pa, pb) = r.edge_segment(*e);
        if let (Some(va), Some(vb)) = (m.v_find(pa, TOL), m.v_find(pb, TOL))
            && let Some(me) = m.edge_between(va, vb)
        {
            let src = m.edges[me].clone();
            let ed = &mut r.edges[*e];
            (ed.soft, ed.smooth, ed.hidden, ed.tag) = (src.soft, src.smooth, src.hidden, src.tag);
        }
    }
    // Edges left between coplanar pieces of one surface go (the pieces merge).
    let seams: Vec<EntityRef> = ids
        .into_iter()
        .filter(|e| {
            let fs = r.edge_faces(*e);
            fs.len() == 2 && {
                let (x, y) = (&r.faces[fs[0]], &r.faces[fs[1]]);
                x.normal.dot(y.normal) > 1.0 - 1e-9 && x.front == y.front && x.back == y.back
            }
        })
        .map(EntityRef::Edge)
        .collect();
    r.erase(&seams);
    r
}

/// `mesh` placed by `xf` with the solid `tool` placed by `txf` added (`add`) or taken away, back
/// in the mesh's own frame; `None` when the result is empty or `xf` cannot be inverted.
pub(crate) fn mesh_boolean(mesh: &Mesh, xf: &DMat4, tool: &Mesh, txf: &DMat4, add: bool) -> Option<Mesh> {
    let mut r = boolean((mesh, xf, None), (tool, txf, None), if add { Bool::Union } else { Bool::Minus });
    let inv = xf.inverse();
    if r.faces.is_empty() || !inv.is_finite() {
        return None;
    }
    let ids: Vec<VertexId> = r.vertices.keys().collect();
    for v in ids {
        let p = inv.transform_point3(r.vertices[v].pos);
        r.v_set_pos(v, p);
    }
    for f in r.faces.values_mut() {
        f.normal = inv.transform_vector3(f.normal).normalize_or(f.normal);
    }
    r.rebuild_caches();
    Some(r)
}

impl Model {
    /// Solid `inst` of the context `path`: its mesh, transform and material.
    fn solid_of(&self, path: &[InstanceId], inst: InstanceId) -> OpResult<(Mesh, DMat4, Instance)> {
        let mesh = self.mesh_at(path).ok_or(OpError::NotFound)?;
        let i = mesh.instances.get(inst).ok_or(OpError::NotFound)?.clone();
        let def = self.defs.get(i.def).ok_or(OpError::NotFound)?;
        if !def.mesh.is_solid() {
            return Err(OpError::Invalid(tr!(
                "kernel-err-is-not-a-solid-every-edge",
                name = if i.name.is_empty() { def.name.as_str() } else { i.name.as_str() }
            )));
        }
        Ok((def.mesh.clone(), i.transform, i))
    }

    /// Run a Solid Tools operation on two solid instances of the context `path` (`first` is the
    /// one picked first). Consumed instances are removed; the results are new groups (in the
    /// second solid's frame). Returns the new instances.
    pub fn solid_op_in(&mut self, path: &[InstanceId], op: SolidOp, first: InstanceId, second: InstanceId) -> OpResult<Vec<InstanceId>> {
        if first == second {
            return Err(OpError::Invalid(tr!("kernel-err-pick-two-different-solids")));
        }
        let (ma, ta, ia) = self.solid_of(path, first)?;
        let (mb, tb, ib) = self.solid_of(path, second)?;
        let a = (&ma, &ta, ia.material);
        let b = (&mb, &tb, ib.material);
        let results: Vec<Mesh> = match op {
            SolidOp::Union | SolidOp::OuterShell => vec![boolean(a, b, Bool::Union)],
            SolidOp::Intersect => vec![boolean(a, b, Bool::Intersect)],
            SolidOp::Subtract | SolidOp::Trim => vec![boolean(b, a, Bool::Minus)],
            SolidOp::Split => vec![boolean(a, b, Bool::Minus), boolean(b, a, Bool::Minus), boolean(a, b, Bool::Intersect)],
        };
        let results: Vec<Mesh> = results.into_iter().filter(|m| !m.faces.is_empty()).collect();
        if results.is_empty() {
            return Err(OpError::Invalid(tr!("kernel-err-the-solids-do-not-overlap")));
        }
        let inv = tb.inverse();
        if !inv.is_finite() {
            return Err(OpError::Invalid(tr!("kernel-err-the-second-solid-has-a-degenerate")));
        }
        let pd = self.def_at(path).ok_or(OpError::NotFound)?;
        let mut out = Vec::new();
        for mut r in results {
            let ids: Vec<VertexId> = r.vertices.keys().collect();
            for v in ids {
                let p = inv.transform_point3(r.vertices[v].pos);
                r.v_set_pos(v, p);
            }
            for f in r.faces.values_mut() {
                f.normal = inv.transform_vector3(f.normal).normalize_or(f.normal);
            }
            r.rebuild_caches();
            // i18n-ignore-next
            let def = self.defs.insert(Definition { name: "Group".into(), mesh: r, is_group: true, guides: Default::default(), params: None, generated: None, source: None, rig: None, rigged: None, preview: None });
            let inst = Instance { def, transform: tb, name: ib.name.clone(), material: None, tag: ib.tag, hidden: false, locked: false };
            out.push(self.defs[pd].mesh.instances.insert(inst));
        }
        let mesh = &mut self.defs[pd].mesh;
        if op != SolidOp::Trim {
            mesh.instances.remove(first);
        }
        mesh.instances.remove(second);
        self.purge_unused_defs();
        Ok(out)
    }

    /// Outer Shell / Union of several solids (picked order), folded pairwise.
    pub fn solid_union_all_in(&mut self, path: &[InstanceId], insts: &[InstanceId]) -> OpResult<InstanceId> {
        let (&first, rest) = insts.split_first().ok_or(OpError::Invalid("no solids".into()))?;
        if rest.is_empty() {
            return Err(OpError::Invalid(tr!("kernel-err-pick-at-least-two-solids")));
        }
        let mut acc = first;
        for &next in rest {
            acc = *self.solid_op_in(path, SolidOp::Union, acc, next)?.first().ok_or(OpError::NotFound)?;
        }
        Ok(acc)
    }
}
