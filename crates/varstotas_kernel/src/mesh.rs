//! `Mesh` storage, adjacency caches and read-only queries.
//!
//! Topology invariants maintained by every kernel operation:
//! - every pair of consecutive vertices in a face loop is connected by an edge;
//! - there is at most one edge between two vertices;
//! - `Face::normal` is the unit Newell normal of the outer loop (outer loop is CCW about it,
//!   holes are CW);
//! - adjacency caches (`vertex → edges`, `edge → faces`) and spatial grids are up to date.
//!
//! The entity maps are public for convenient reads and attribute edits (materials, flags,
//! tags). If you change topology or positions by hand, call [`Mesh::rebuild_caches`].

use crate::geom::{self, PointLoc, Projector};
use crate::grid::Grid;
use crate::{Aabb, Edge, EdgeId, EntityRef, Face, FaceId, Instance, InstanceId, Plane, Ray, TOL, Vertex, VertexId};
use glam::{DVec2, DVec3};
use serde::{Deserialize, Serialize};
use slotmap::{SecondaryMap, SlotMap};
use std::collections::{HashSet, VecDeque};

/// Private adjacency caches and spatial indices (rebuilt on load / when invalid).
#[derive(Clone, Default)]
pub(crate) struct Caches {
    pub valid: bool,
    pub vert_edges: SecondaryMap<VertexId, Vec<EdgeId>>,
    pub edge_faces: SecondaryMap<EdgeId, Vec<FaceId>>,
    pub vgrid: Grid<VertexId>,
    pub egrid: Grid<EdgeId>,
    pub fgrid: Grid<FaceId>,
    /// Edge count when the grids were last sized.
    pub sized_for: usize,
}

impl std::fmt::Debug for Caches {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Caches {{ valid: {} }}", self.valid)
    }
}

/// Sticky geometry of one context (model root, a group, or a component definition).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Mesh {
    pub vertices: SlotMap<VertexId, Vertex>,
    pub edges: SlotMap<EdgeId, Edge>,
    pub faces: SlotMap<FaceId, Face>,
    pub instances: SlotMap<InstanceId, Instance>,
    #[serde(skip)]
    pub(crate) cache: Caches,
}

/// Vertex-on-plane tolerance.
pub(crate) const PTOL: f64 = TOL;

impl Mesh {
    pub fn new() -> Self {
        let mut m = Self::default();
        m.cache.valid = true;
        m
    }

    // ---- cache management ------------------------------------------------

    /// Rebuild adjacency caches and spatial indices from scratch.
    pub fn rebuild_caches(&mut self) {
        let mut c = Caches::default();
        for (v, _) in &self.vertices {
            c.vert_edges.insert(v, Vec::new());
        }
        for (e, ed) in &self.edges {
            c.edge_faces.insert(e, Vec::new());
            for v in ed.v {
                if let Some(l) = c.vert_edges.get_mut(v)
                    && !l.contains(&e) {
                        l.push(e);
                    }
            }
        }
        // Grid cell size: ~2x median edge length, clamped.
        let mut lens: Vec<f64> = self
            .edges
            .values()
            .filter_map(|e| Some((self.vertices.get(e.v[0])?.pos - self.vertices.get(e.v[1])?.pos).length()))
            .collect();
        let cell = if lens.is_empty() {
            100.0
        } else {
            let mid = lens.len() / 2;
            let (_, m, _) = lens.select_nth_unstable_by(mid, |a, b| a.total_cmp(b));
            (*m * 2.0).clamp(0.5, 10_000.0)
        };
        c.vgrid = Grid::new(cell);
        c.egrid = Grid::new(cell);
        c.fgrid = Grid::new(cell);
        c.sized_for = self.edges.len();
        self.cache = c;
        self.cache.valid = true;
        for (v, vd) in &self.vertices {
            self.cache.vgrid.insert_point(v, vd.pos);
        }
        let eids: Vec<EdgeId> = self.edges.keys().collect();
        for e in eids {
            self.grid_edge(e);
        }
        let fids: Vec<FaceId> = self.faces.keys().collect();
        for f in fids {
            let face = &self.faces[f];
            let mut pairs = Vec::new();
            for lp in face.loops() {
                for i in 0..lp.len() {
                    pairs.push((lp[i], lp[(i + 1) % lp.len()]));
                }
            }
            for (a, b) in pairs {
                if let Some(e) = self.edge_between(a, b) {
                    let l = self.cache.edge_faces.get_mut(e).unwrap();
                    if !l.contains(&f) {
                        l.push(f);
                    }
                }
            }
            self.grid_face(f);
        }
    }

    /// Mark caches stale (they are rebuilt lazily by the next editing operation).
    pub fn invalidate_caches(&mut self) {
        self.cache.valid = false;
    }

    pub(crate) fn ensure(&mut self) {
        if !self.cache.valid {
            self.rebuild_caches();
        } else {
            let n = self.edges.len();
            if n >= 1024 && n > 4 * self.cache.sized_for.max(1) {
                self.rebuild_caches();
            }
        }
    }

    pub(crate) fn grid_edge(&mut self, e: EdgeId) {
        let ed = &self.edges[e];
        let (a, b) = (self.vertices[ed.v[0]].pos, self.vertices[ed.v[1]].pos);
        self.cache.egrid.insert_segment(e, a, b);
    }

    pub(crate) fn grid_face(&mut self, f: FaceId) {
        if let Some(b) = self.face_aabb(f) {
            self.cache.fgrid.insert_box(f, b.min, b.max);
        }
    }

    // ---- queries -------------------------------------------------------

    pub fn edge_faces(&self, e: EdgeId) -> Vec<FaceId> {
        if self.cache.valid {
            return self.cache.edge_faces.get(e).cloned().unwrap_or_default();
        }
        let Some(ed) = self.edges.get(e) else { return Vec::new() };
        let (a, b) = (ed.v[0], ed.v[1]);
        self.faces
            .iter()
            .filter(|(_, f)| f.loops().any(|l| loop_has_pair(l, a, b)))
            .map(|(id, _)| id)
            .collect()
    }

    pub fn vertex_edges(&self, v: VertexId) -> Vec<EdgeId> {
        if self.cache.valid {
            return self.cache.vert_edges.get(v).cloned().unwrap_or_default();
        }
        self.edges.iter().filter(|(_, e)| e.has(v)).map(|(id, _)| id).collect()
    }

    /// Edge connecting `a` and `b`, if any.
    pub fn edge_between(&self, a: VertexId, b: VertexId) -> Option<EdgeId> {
        if self.cache.valid {
            let l = self.cache.vert_edges.get(a)?;
            return l.iter().copied().find(|e| self.edges.get(*e).is_some_and(|ed| ed.has(b) && ed.other(a) == b));
        }
        self.edges.iter().find(|(_, e)| (e.v[0] == a && e.v[1] == b) || (e.v[0] == b && e.v[1] == a)).map(|(id, _)| id)
    }

    /// All edges bounding the face (outer + holes).
    pub fn face_edges(&self, f: FaceId) -> Vec<EdgeId> {
        let Some(face) = self.faces.get(f) else { return Vec::new() };
        let mut out = Vec::new();
        for lp in face.loops() {
            for i in 0..lp.len() {
                if let Some(e) = self.edge_between(lp[i], lp[(i + 1) % lp.len()]) {
                    out.push(e);
                }
            }
        }
        out
    }

    /// Vertices of all loops of a face.
    pub fn face_vertices(&self, f: FaceId) -> Vec<VertexId> {
        self.faces.get(f).map(|face| face.loops().flatten().copied().collect()).unwrap_or_default()
    }

    pub fn face_plane(&self, f: FaceId) -> Plane {
        let face = &self.faces[f];
        let origin = face.outer.first().map(|v| self.vertices[*v].pos).unwrap_or(DVec3::ZERO);
        Plane { origin, normal: face.normal }
    }

    pub fn face_area(&self, f: FaceId) -> f64 {
        let Some(face) = self.faces.get(f) else { return 0.0 };
        let area = |l: &Vec<VertexId>| geom::newell(&self.loop_pos(l)).length() * 0.5;
        area(&face.outer) - face.holes.iter().map(area).sum::<f64>()
    }

    pub fn edge_length(&self, e: EdgeId) -> f64 {
        let (a, b) = self.edge_segment(e);
        (b - a).length()
    }

    pub fn edge_segment(&self, e: EdgeId) -> (DVec3, DVec3) {
        let ed = &self.edges[e];
        (self.vertices[ed.v[0]].pos, self.vertices[ed.v[1]].pos)
    }

    pub(crate) fn pos(&self, v: VertexId) -> DVec3 {
        self.vertices[v].pos
    }

    pub(crate) fn loop_pos(&self, l: &[VertexId]) -> Vec<DVec3> {
        l.iter().map(|v| self.vertices[*v].pos).collect()
    }

    pub fn face_aabb(&self, f: FaceId) -> Option<Aabb> {
        let face = self.faces.get(f)?;
        Aabb::from_points(face.outer.iter().filter_map(|v| self.vertices.get(*v)).map(|v| v.pos))
    }

    /// Faces sharing at least one edge with `f`.
    pub fn face_neighbors(&self, f: FaceId) -> Vec<FaceId> {
        let mut out = Vec::new();
        for e in self.face_edges(f) {
            for g in self.edge_faces(e) {
                if g != f && !out.contains(&g) {
                    out.push(g);
                }
            }
        }
        out
    }

    /// Projector for a face's plane plus its loops in 2D.
    pub(crate) fn face_2d(&self, f: FaceId) -> (Projector, Vec<DVec2>, Vec<Vec<DVec2>>) {
        let face = &self.faces[f];
        let pr = Projector::new(self.pos(face.outer[0]), face.normal);
        let outer = face.outer.iter().map(|v| pr.to2(self.pos(*v))).collect();
        let holes = face.holes.iter().map(|h| h.iter().map(|v| pr.to2(self.pos(*v))).collect()).collect();
        (pr, outer, holes)
    }

    /// Location of a 3D point (assumed in the face plane) relative to the face region.
    pub fn face_point_loc(&self, f: FaceId, p: DVec3) -> PointLoc {
        let (pr, outer, holes) = self.face_2d(f);
        geom::point_in_region(pr.to2(p), &outer, &holes, TOL)
    }

    /// Does face `f` traverse the directed pair `a → b` in one of its loops?
    pub(crate) fn face_traverses(&self, f: FaceId, a: VertexId, b: VertexId) -> Option<bool> {
        let face = self.faces.get(f)?;
        for l in face.loops() {
            let n = l.len();
            for i in 0..n {
                let (x, y) = (l[i], l[(i + 1) % n]);
                if x == a && y == b {
                    return Some(true);
                }
                if x == b && y == a {
                    return Some(false);
                }
            }
        }
        None
    }

    /// All entities connected to `start` through edges/faces (triple-click).
    /// Returns edges and faces (plus `start` itself if it is an isolated vertex or an instance).
    pub fn connected(&self, start: EntityRef) -> HashSet<EntityRef> {
        let mut out = HashSet::new();
        let mut seen_v: HashSet<VertexId> = HashSet::new();
        let mut q: VecDeque<VertexId> = VecDeque::new();
        match start {
            EntityRef::Instance(_) => {
                out.insert(start);
                return out;
            }
            EntityRef::Vertex(v) => {
                if !self.vertices.contains_key(v) {
                    return out;
                }
                q.push_back(v);
            }
            EntityRef::Edge(e) => {
                let Some(ed) = self.edges.get(e) else { return out };
                q.push_back(ed.v[0]);
            }
            EntityRef::Face(f) => {
                let Some(face) = self.faces.get(f) else { return out };
                out.insert(start);
                q.extend(face.loops().flatten().copied());
            }
        }
        while let Some(v) = q.pop_front() {
            if !seen_v.insert(v) {
                continue;
            }
            for e in self.vertex_edges(v) {
                if out.insert(EntityRef::Edge(e)) {
                    let ed = &self.edges[e];
                    q.push_back(ed.other(v));
                    for f in self.edge_faces(e) {
                        if out.insert(EntityRef::Face(f)) {
                            q.extend(self.faces[f].loops().flatten().copied());
                        }
                    }
                }
            }
        }
        if out.is_empty() {
            out.insert(start);
        }
        out
    }

    /// Bounds of the vertices of this mesh (instances are not included; see `Model::bounds`).
    pub fn bounds(&self) -> Option<Aabb> {
        Aabb::from_points(self.vertices.values().map(|v| v.pos))
    }

    /// Closest face hit in this mesh (instances are NOT descended; see `Model::raycast`).
    pub fn raycast_faces(&self, ray: &Ray) -> Option<(FaceId, f64)> {
        self.raycast_faces_filtered(ray.origin, ray.dir, |_| true)
    }

    /// Raycast with an unnormalized direction (`t` is in units of `dir`) and a face filter.
    pub(crate) fn raycast_faces_filtered(
        &self,
        origin: DVec3,
        dir: DVec3,
        mut keep: impl FnMut(&Face) -> bool,
    ) -> Option<(FaceId, f64)> {
        let mut best: Option<(FaceId, f64)> = None;
        for (fid, face) in &self.faces {
            if face.outer.len() < 3 || !keep(face) {
                continue;
            }
            let n = face.normal;
            let den = dir.dot(n);
            if den.abs() < 1e-12 {
                continue;
            }
            let o = self.pos(face.outer[0]);
            let t = (o - origin).dot(n) / den;
            if t < 0.0 || best.is_some_and(|(_, bt)| t >= bt) {
                continue;
            }
            let p = origin + dir * t;
            if self.face_point_loc(fid, p) != PointLoc::Outside {
                best = Some((fid, t));
            }
        }
        best
    }

    /// Consistency check of all invariants (for tests and debugging).
    pub fn validate(&self) -> Result<(), String> {
        for (e, ed) in &self.edges {
            if ed.v[0] == ed.v[1] {
                return Err(format!("edge {e:?} is degenerate"));
            }
            for v in ed.v {
                if !self.vertices.contains_key(v) {
                    return Err(format!("edge {e:?} references missing vertex"));
                }
            }
        }
        let mut pairs = HashSet::new();
        for (_, ed) in &self.edges {
            let k = if ed.v[0] < ed.v[1] { (ed.v[0], ed.v[1]) } else { (ed.v[1], ed.v[0]) };
            if !pairs.insert(k) {
                return Err("duplicate edge".into());
            }
        }
        for (f, face) in &self.faces {
            if face.outer.len() < 3 {
                return Err(format!("face {f:?} has < 3 vertices"));
            }
            for l in face.loops() {
                for i in 0..l.len() {
                    let (a, b) = (l[i], l[(i + 1) % l.len()]);
                    let Some(e) = self.edge_between(a, b) else {
                        return Err(format!("face {f:?} loop pair has no edge"));
                    };
                    if self.cache.valid && !self.cache.edge_faces.get(e).is_some_and(|l| l.contains(&f)) {
                        return Err(format!("edge_faces cache missing face {f:?}"));
                    }
                }
            }
            let nn = geom::newell(&self.loop_pos(&face.outer));
            if nn.normalize_or_zero().dot(face.normal) < 0.999 {
                return Err(format!("face {f:?} normal inconsistent with outer loop"));
            }
            for h in &face.holes {
                if geom::newell(&self.loop_pos(h)).dot(face.normal) > 0.0 {
                    return Err(format!("face {f:?} hole not CW"));
                }
            }
            let pl = self.face_plane(f);
            for v in face.loops().flatten() {
                if pl.distance(self.pos(*v)).abs() > 10.0 * PTOL {
                    return Err(format!("face {f:?} not planar"));
                }
            }
        }
        if self.cache.valid {
            for (e, l) in &self.cache.edge_faces {
                if !self.edges.contains_key(e) {
                    return Err("edge_faces cache has dead edge".into());
                }
                for f in l {
                    if !self.faces.contains_key(*f) {
                        return Err("edge_faces cache has dead face".into());
                    }
                }
            }
            for (v, l) in &self.cache.vert_edges {
                if !self.vertices.contains_key(v) {
                    return Err("vert_edges cache has dead vertex".into());
                }
                for e in l {
                    if !self.edges.get(*e).is_some_and(|ed| ed.has(v)) {
                        return Err("vert_edges cache inconsistent".into());
                    }
                }
            }
            for (e, ed) in &self.edges {
                for v in ed.v {
                    if !self.cache.vert_edges.get(v).is_some_and(|l| l.contains(&e)) {
                        return Err("vert_edges cache missing edge".into());
                    }
                }
            }
        }
        Ok(())
    }
}

pub(crate) fn loop_has_pair(l: &[VertexId], a: VertexId, b: VertexId) -> bool {
    let n = l.len();
    (0..n).any(|i| {
        let (x, y) = (l[i], l[(i + 1) % n]);
        (x == a && y == b) || (x == b && y == a)
    })
}
