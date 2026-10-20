//! Intersect Faces (Edit/Context → Intersect Faces).
//!
//! Participating faces (loose faces of the active context and the faces inside selected or
//! nearby instances, at any depth) are brought into the active context's coordinates. For every
//! pair of faces whose boxes overlap, the intersection is computed:
//! - non-coplanar faces: the part of the planes' intersection line lying in both regions
//!   (holes excluded, boundaries included; a piece lying on the boundary of *both* faces is
//!   skipped: it is an existing edge or faces merely touching along an edge);
//! - coplanar faces: the boundary of each face lying strictly inside the other (imprinted
//!   only for faces outside the active context; active faces never overlap each other).
//!
//! The resulting segments become edges of the active context (also for faces
//! inside instances) through the sticky insertion machinery, and the faces of the active
//! context they lie on are split (a loop closing inside a face cuts out a new face). Loops in
//! empty space (no active face there, e.g. two groups) stay edges: no faces are invented.

use crate::geom::{self, PointLoc, Projector};
use crate::mesh::{Mesh, PTOL};
use crate::model::Model;
use crate::topology::{Classify, Orient};
use crate::{Aabb, DefId, EdgeId, EntityRef, FaceId, InstanceId, OpError, OpResult, Plane, TOL, VertexId};
use glam::{DMat4, DVec2, DVec3};
use std::collections::HashMap;

/// What the selection is intersected with (see [`Model::intersect_faces`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntersectScope {
    /// Selection against every visible face of the model (any context, any depth).
    Model,
    /// Selected faces/instances only against each other.
    Selection,
    /// Selection against the active context: its loose faces and the instances in it.
    Context,
}

slotmap::new_key_type! { struct PolyKey; }

/// A face polygon in the active context's coordinates.
pub(super) struct IPoly {
    outer: Vec<DVec3>,
    holes: Vec<Vec<DVec3>>,
    pub(super) plane: Plane,
    pr: Projector,
    outer2: Vec<DVec2>,
    holes2: Vec<Vec<DVec2>>,
    bbox: Aabb,
    /// Context (instance chain) the face lives in; faces of one context are not intersected
    /// with each other unless it is the active one.
    body: usize,
    active: bool,
    in_a: bool,
    in_b: bool,
}

impl IPoly {
    pub(super) fn new(outer: Vec<DVec3>, holes: Vec<Vec<DVec3>>, body: usize, active: bool) -> Option<Self> {
        let n = geom::newell(&outer);
        if outer.len() < 3 || !n.is_finite() || n.length() < 1e-12 {
            return None;
        }
        let n = n.normalize();
        let c = outer.iter().copied().sum::<DVec3>() / outer.len() as f64;
        let pr = Projector::new(c, n);
        let outer2 = outer.iter().map(|p| pr.to2(*p)).collect();
        let holes2 = holes.iter().map(|h| h.iter().map(|p| pr.to2(*p)).collect()).collect();
        let bbox = Aabb::from_points(outer.iter().copied())?;
        Some(Self { outer, holes, plane: Plane { origin: c, normal: n }, pr, outer2, holes2, bbox, body, active, in_a: false, in_b: false })
    }

    pub(super) fn loc(&self, p: DVec3) -> PointLoc {
        geom::point_in_region(self.pr.to2(p), &self.outer2, &self.holes2, TOL)
    }

    fn loops(&self) -> impl Iterator<Item = &Vec<DVec3>> {
        std::iter::once(&self.outer).chain(self.holes.iter())
    }

    fn loops2(&self) -> impl Iterator<Item = &Vec<DVec2>> {
        std::iter::once(&self.outer2).chain(self.holes2.iter())
    }
}

/// Merge consecutive accepted parameter intervals along a line into segments.
pub(super) fn emit_intervals(ts: &mut Vec<f64>, at: impl Fn(f64) -> DVec3, accept: impl Fn(DVec3) -> bool, out: &mut Vec<(DVec3, DVec3)>) {
    ts.sort_by(|a, b| a.total_cmp(b));
    ts.dedup_by(|b, a| (*b - *a).abs() < TOL);
    let mut cur: Option<(f64, f64)> = None;
    for w in ts.windows(2) {
        let (t0, t1) = (w[0], w[1]);
        if accept(at((t0 + t1) * 0.5)) {
            cur = Some(match cur {
                Some((s, _)) => (s, t1),
                None => (t0, t1),
            });
        } else if let Some((s, e)) = cur.take() {
            out.push((at(s), at(e)));
        }
    }
    if let Some((s, e)) = cur {
        out.push((at(s), at(e)));
    }
}

/// Parameters along the line `p0 + t dir` where the loops of `p` meet the plane `other`.
pub(super) fn line_params(p: &IPoly, other: &Plane, p0: DVec3, dir: DVec3, ts: &mut Vec<f64>) {
    for l in p.loops() {
        let k = l.len();
        for i in 0..k {
            let (u, v) = (l[i], l[(i + 1) % k]);
            let (du, dv) = (other.distance(u), other.distance(v));
            if du.abs() <= PTOL {
                ts.push((u - p0).dot(dir));
            } else if dv.abs() > PTOL && (du < 0.0) != (dv < 0.0) {
                let x = u + (v - u) * (du / (du - dv));
                ts.push((x - p0).dot(dir));
            }
        }
    }
}

/// Intersection segments of two faces.
fn pair_segments(a: &IPoly, b: &IPoly, out: &mut Vec<(DVec3, DVec3)>) {
    let (na, nb) = (a.plane.normal, b.plane.normal);
    let d = na.cross(nb);
    let s = d.length();
    if s < 1e-9 {
        let coplanar = a.outer.iter().all(|p| b.plane.distance(*p).abs() <= PTOL);
        if coplanar {
            if !a.active {
                imprint_boundary(a, b, out);
            }
            if !b.active {
                imprint_boundary(b, a, out);
            }
        }
        return;
    }
    // A point on both planes: n·x = c for each plane.
    let (ca, cb) = (na.dot(a.plane.origin), nb.dot(b.plane.origin));
    let p0 = (nb.cross(d) * ca + d.cross(na) * cb) / (s * s);
    let dir = d / s;
    let mut ts = Vec::new();
    line_params(a, &b.plane, p0, dir, &mut ts);
    if ts.is_empty() {
        return;
    }
    line_params(b, &a.plane, p0, dir, &mut ts);
    if ts.len() < 2 {
        return;
    }
    emit_intervals(
        &mut ts,
        |t| p0 + dir * t,
        |m| {
            let la = a.loc(m);
            if la == PointLoc::Outside {
                return false;
            }
            let lb = b.loc(m);
            lb != PointLoc::Outside && !(la == PointLoc::On && lb == PointLoc::On)
        },
        out,
    );
}

/// Pieces of `x`'s boundary lying strictly inside the coplanar face `y`.
fn imprint_boundary(x: &IPoly, y: &IPoly, out: &mut Vec<(DVec3, DVec3)>) {
    if !x.bbox.inflate(TOL).intersects(&y.bbox) {
        return;
    }
    for l in x.loops() {
        let k = l.len();
        for i in 0..k {
            let (p, q) = (l[i], l[(i + 1) % k]);
            let (p2, q2) = (y.pr.to2(p), y.pr.to2(q));
            let dd = q2 - p2;
            let len2 = dd.length_squared();
            if len2 < TOL * TOL {
                continue;
            }
            let param = |c: DVec2| (c - p2).dot(dd) / len2;
            let mut ts = vec![0.0, 1.0];
            for yl in y.loops2() {
                let m = yl.len();
                for j in 0..m {
                    let (c, e) = (yl[j], yl[(j + 1) % m]);
                    for w in [c, e] {
                        if geom::point_seg_dist2(w, p2, q2).0 <= TOL {
                            ts.push(param(w));
                        }
                    }
                    if geom::segs_intersect_2d(p2, q2, c, e, 1e-12)
                        && let Some(x) = geom::line_line_2d(p2, dd, c, e - c)
                    {
                        ts.push(param(x));
                    }
                }
            }
            ts.retain(|t| (-1e-9..=1.0 + 1e-9).contains(t));
            // Parameters are fractions of the edge: dedup in length units.
            let len = len2.sqrt();
            let mut ts: Vec<f64> = ts.into_iter().map(|t| t.clamp(0.0, 1.0) * len).collect();
            emit_intervals(&mut ts, |t| p + (q - p) * (t / len), |m| y.loc(m) == PointLoc::Inside, out);
        }
    }
}

/// Collected participants of one intersect operation.
#[derive(Default)]
struct Polys {
    polys: slotmap::SlotMap<PolyKey, IPoly>,
    index: HashMap<(usize, FaceId), PolyKey>,
    bodies: HashMap<Vec<InstanceId>, usize>,
}

impl Polys {
    fn body(&mut self, path: &[InstanceId]) -> usize {
        let n = self.bodies.len();
        *self.bodies.entry(path.to_vec()).or_insert(n)
    }

    /// Add a face of `mesh` (transformed by `m`) with the given membership.
    #[allow(clippy::too_many_arguments)]
    fn add(&mut self, mesh: &Mesh, f: FaceId, m: &DMat4, body: usize, active: bool, a: bool, b: bool) {
        let key = match self.index.get(&(body, f)) {
            Some(k) => *k,
            None => {
                let Some(face) = mesh.faces.get(f) else { return };
                let tr = |l: &Vec<VertexId>| -> Vec<DVec3> { l.iter().map(|v| m.transform_point3(mesh.vertices[*v].pos)).collect() };
                let Some(p) = IPoly::new(tr(&face.outer), face.holes.iter().map(tr).collect(), body, active) else { return };
                let k = self.polys.insert(p);
                self.index.insert((body, f), k);
                k
            }
        };
        let p = &mut self.polys[key];
        p.in_a |= a;
        p.in_b |= b;
    }

    /// Faces of definition `d` and (recursively) of its visible instances.
    #[allow(clippy::too_many_arguments)]
    fn add_def(&mut self, model: &Model, d: DefId, path: &mut Vec<InstanceId>, m: DMat4, active_path: &[InstanceId], a: bool, b: bool, depth: usize) {
        if depth > 64 {
            return;
        }
        let Some(def) = model.defs.get(d) else { return };
        let active = path.as_slice() == active_path;
        // Exact coordinates for the active context itself.
        let m = if active { DMat4::IDENTITY } else { m };
        let body = self.body(path);
        for (f, face) in &def.mesh.faces {
            if !face.hidden && model.tag_visible(d, face.tag) {
                self.add(&def.mesh, f, &m, body, active, a, b);
            }
        }
        for (iid, inst) in &def.mesh.instances {
            if inst.hidden || !model.tag_visible(d, inst.tag) {
                continue;
            }
            path.push(iid);
            self.add_def(model, inst.def, path, m * inst.transform, active_path, a, b, depth + 1);
            path.pop();
        }
    }

    /// All intersection segments between A members and B members.
    fn segments(&self, active_body: Option<usize>) -> Vec<(DVec3, DVec3)> {
        let mut out = Vec::new();
        if self.polys.is_empty() {
            return out;
        }
        // Broad phase: a hash grid of the boxes (cell ~ 2x the median box size).
        let mut sizes: Vec<f64> = self.polys.values().map(|p| p.bbox.size().max_element()).collect();
        let mid = sizes.len() / 2;
        let (_, med, _) = sizes.select_nth_unstable_by(mid, |a, b| a.total_cmp(b));
        let mut grid: crate::grid::Grid<PolyKey> = crate::grid::Grid::new((*med * 2.0).clamp(1.0, 1e5));
        for (k, p) in &self.polys {
            if p.in_b {
                grid.insert_box(k, p.bbox.min, p.bbox.max);
            }
        }
        let mut keys: Vec<PolyKey> = self.polys.keys().filter(|k| self.polys[*k].in_a).collect();
        keys.sort();
        for i in keys {
            let a = &self.polys[i];
            let bb = a.bbox.inflate(TOL * 2.0);
            for j in grid.query_box(bb.min, bb.max) {
                if j == i {
                    continue;
                }
                let b = &self.polys[j];
                // Each unordered A/A pair once.
                if b.in_a && j < i {
                    continue;
                }
                if a.body == b.body && Some(a.body) != active_body {
                    continue;
                }
                if !bb.intersects(&b.bbox) {
                    continue;
                }
                pair_segments(a, b, &mut out);
            }
        }
        out
    }
}

impl Mesh {
    /// Insert intersection segments as edges and split the faces they lie on (no new faces
    /// from closed loops). Returns the newly created edges that still exist.
    pub(crate) fn imprint_segments(&mut self, segs: &[(DVec3, DVec3)]) -> Vec<EdgeId> {
        self.ensure();
        let mut created: Vec<EdgeId> = Vec::new();
        for (a, b) in segs {
            let (_, c) = self.insert_segment(*a, *b);
            created.extend(c);
        }
        // Group the faces lying under the new edges by plane; one region rebuild per plane.
        let mut groups: Vec<(Plane, Vec<VertexId>, Vec<FaceId>)> = Vec::new();
        for &e in &created {
            if !self.edges.contains_key(e) {
                continue;
            }
            let (a, b) = self.edge_segment(e);
            for f in self.faces_containing_segment(a, b) {
                let pl = self.face_plane(f);
                let g = match groups
                    .iter()
                    .position(|(q, _, _)| q.normal.dot(pl.normal).abs() > 1.0 - 1e-9 && q.distance(pl.origin).abs() < PTOL)
                {
                    Some(g) => g,
                    None => {
                        groups.push((pl, Vec::new(), Vec::new()));
                        groups.len() - 1
                    }
                };
                let (_, seeds, faces) = &mut groups[g];
                seeds.extend(self.edges[e].v);
                if !faces.contains(&f) {
                    faces.push(f);
                }
            }
        }
        for (plane, mut seeds, faces) in groups {
            seeds.sort();
            seeds.dedup();
            let faces: Vec<FaceId> = faces.into_iter().filter(|f| self.faces.contains_key(*f)).collect();
            self.refresh_plane(plane, &seeds, &faces, &Classify::None, Orient::auto());
        }
        created.retain(|e| self.edges.contains_key(*e));
        created
    }

    /// Intersect faces `a` with faces `b` of this mesh (pairs within `a` included when they
    /// are also in `b`). Edges along the intersections are added and faces split.
    /// Returns the newly created edges.
    pub fn intersect_faces(&mut self, a: &[FaceId], b: &[FaceId]) -> Vec<EdgeId> {
        self.ensure();
        let mut ps = Polys::default();
        let body = ps.body(&[]);
        for &f in a {
            ps.add(self, f, &DMat4::IDENTITY, body, true, true, false);
        }
        for &f in b {
            ps.add(self, f, &DMat4::IDENTITY, body, true, false, true);
        }
        let segs = ps.segments(Some(body));
        self.imprint_segments(&segs)
    }

    /// Intersect every face of this mesh with every other one (self-intersections).
    pub fn intersect_all(&mut self) -> Vec<EdgeId> {
        let all: Vec<FaceId> = self.faces.keys().collect();
        self.intersect_faces(&all, &all)
    }
}

impl Model {
    /// Intersect Faces. `path` is the active context, `sel` the selection in it (faces and
    /// instances; other entities are ignored). The selection is intersected with the
    /// geometry chosen by `scope`; the resulting edges are created in the active context
    /// (also for faces inside instances) and split its faces.
    /// Hidden faces/instances and hidden tags do not take part (selected loose faces always
    /// do). Returns the edges created in the active context.
    pub fn intersect_faces(&mut self, path: &[InstanceId], sel: &[EntityRef], scope: IntersectScope) -> OpResult<Vec<EdgeId>> {
        let ad = self.def_at(path).ok_or(OpError::NotFound)?;
        let to_local = self.transform_at(path).inverse();
        let mut ps = Polys::default();
        let active_body = ps.body(path);
        let in_b_sel = scope == IntersectScope::Selection;
        {
            let mesh = &self.defs[ad].mesh;
            let mut abs: Vec<InstanceId> = path.to_vec();
            for e in sel {
                match *e {
                    EntityRef::Face(f) if mesh.faces.contains_key(f) => {
                        ps.add(mesh, f, &DMat4::IDENTITY, active_body, true, true, in_b_sel);
                    }
                    EntityRef::Instance(i) => {
                        let Some(inst) = mesh.instances.get(i) else { continue };
                        abs.push(i);
                        ps.add_def(self, inst.def, &mut abs, inst.transform, path, true, in_b_sel, 1);
                        abs.pop();
                    }
                    _ => {}
                }
            }
        }
        match scope {
            IntersectScope::Selection => {}
            IntersectScope::Context => {
                let mut abs = path.to_vec();
                ps.add_def(self, ad, &mut abs, DMat4::IDENTITY, path, false, true, 0);
            }
            IntersectScope::Model => {
                let mut abs = Vec::new();
                ps.add_def(self, self.root, &mut abs, to_local, path, false, true, 0);
            }
        }
        let segs = ps.segments(Some(active_body));
        let mesh = &mut self.defs[ad].mesh;
        Ok(mesh.imprint_segments(&segs))
    }

    /// [`Model::intersect_faces`] with [`IntersectScope::Model`].
    pub fn intersect_with_model(&mut self, path: &[InstanceId], sel: &[EntityRef]) -> OpResult<Vec<EdgeId>> {
        self.intersect_faces(path, sel, IntersectScope::Model)
    }

    /// [`Model::intersect_faces`] with [`IntersectScope::Selection`].
    pub fn intersect_with_selection(&mut self, path: &[InstanceId], sel: &[EntityRef]) -> OpResult<Vec<EdgeId>> {
        self.intersect_faces(path, sel, IntersectScope::Selection)
    }

    /// [`Model::intersect_faces`] with [`IntersectScope::Context`].
    pub fn intersect_with_context(&mut self, path: &[InstanceId], sel: &[EntityRef]) -> OpResult<Vec<EdgeId>> {
        self.intersect_faces(path, sel, IntersectScope::Context)
    }
}
