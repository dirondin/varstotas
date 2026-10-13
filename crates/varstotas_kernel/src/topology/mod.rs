//! Low-level sticky-geometry primitives. Every function keeps the caches consistent.

mod erase;
mod insert;
mod region;

pub(crate) use region::{Classify, Orient};

use crate::geom;
use crate::mesh::Mesh;
use crate::{Edge, EdgeId, Face, FaceId, TOL, Vertex, VertexId};
use glam::DVec3;

impl Mesh {
    // ---- vertices ------------------------------------------------------------

    pub(crate) fn v_new(&mut self, p: DVec3) -> VertexId {
        let v = self.vertices.insert(Vertex { pos: p });
        self.cache.vert_edges.insert(v, Vec::new());
        self.cache.vgrid.insert_point(v, p);
        v
    }

    /// Nearest existing vertex within `tol` of `p`.
    pub(crate) fn v_find(&self, p: DVec3, tol: f64) -> Option<VertexId> {
        let d = DVec3::splat(tol);
        let mut best: Option<(f64, VertexId)> = None;
        for v in self.cache.vgrid.query_box(p - d, p + d) {
            let Some(vd) = self.vertices.get(v) else { continue };
            let dist = (vd.pos - p).length();
            if dist <= tol && best.is_none_or(|(bd, _)| dist < bd) {
                best = Some((dist, v));
            }
        }
        best.map(|(_, v)| v)
    }

    pub(crate) fn v_get_or_new(&mut self, p: DVec3) -> VertexId {
        self.v_find(p, TOL).unwrap_or_else(|| self.v_new(p))
    }

    /// Remove a vertex that has no edges.
    pub(crate) fn v_remove(&mut self, v: VertexId) {
        self.cache.vgrid.remove(v);
        self.cache.vert_edges.remove(v);
        self.vertices.remove(v);
    }

    pub(crate) fn remove_if_orphan(&mut self, v: VertexId) {
        if self.vertices.contains_key(v) && self.cache.vert_edges.get(v).is_none_or(|l| l.is_empty()) {
            self.v_remove(v);
        }
    }

    /// Move one vertex, updating spatial indices of everything attached.
    pub(crate) fn v_set_pos(&mut self, v: VertexId, p: DVec3) {
        self.vertices[v].pos = p;
        self.cache.vgrid.insert_point(v, p);
        let mut faces = Vec::new();
        for e in self.vertex_edges(v) {
            self.grid_edge(e);
            for f in self.edge_faces(e) {
                if !faces.contains(&f) {
                    faces.push(f);
                }
            }
        }
        for f in faces {
            self.grid_face(f);
        }
    }

    pub(crate) fn faces_of_vertex(&self, v: VertexId) -> Vec<FaceId> {
        let mut out = Vec::new();
        for e in self.vertex_edges(v) {
            for f in self.edge_faces(e) {
                if !out.contains(&f) {
                    out.push(f);
                }
            }
        }
        out
    }

    // ---- edges ---------------------------------------------------------------

    pub(crate) fn e_new(&mut self, a: VertexId, b: VertexId, template: Option<&Edge>) -> EdgeId {
        let mut ed = template.cloned().unwrap_or_else(|| Edge::new(a, b));
        ed.v = [a, b];
        let e = self.edges.insert(ed);
        self.cache.vert_edges.entry(a).unwrap().or_default().push(e);
        self.cache.vert_edges.entry(b).unwrap().or_default().push(e);
        self.cache.edge_faces.insert(e, Vec::new());
        self.grid_edge(e);
        e
    }

    /// Existing edge between `a` and `b`, or a new one. Returns `(edge, created)`.
    pub(crate) fn e_ensure(&mut self, a: VertexId, b: VertexId) -> (EdgeId, bool) {
        match self.edge_between(a, b) {
            Some(e) => (e, false),
            None => (self.e_new(a, b, None), true),
        }
    }

    /// Remove an edge (faces referencing it must be removed/updated by the caller).
    pub(crate) fn e_remove(&mut self, e: EdgeId) {
        let Some(ed) = self.edges.remove(e) else { return };
        for v in ed.v {
            if let Some(l) = self.cache.vert_edges.get_mut(v) {
                l.retain(|x| *x != e);
            }
        }
        self.cache.edge_faces.remove(e);
        self.cache.egrid.remove(e);
    }

    /// Re-point one endpoint of an edge.
    fn e_repoint(&mut self, e: EdgeId, from: VertexId, to: VertexId) {
        if let Some(l) = self.cache.vert_edges.get_mut(from) {
            l.retain(|x| *x != e);
        }
        let ed = &mut self.edges[e];
        if ed.v[0] == from {
            ed.v[0] = to;
        } else {
            ed.v[1] = to;
        }
        self.cache.vert_edges.entry(to).unwrap().or_default().push(e);
        self.grid_edge(e);
    }

    /// Split edge `e` at existing vertex `v` (which lies on it). Face loops get `v` inserted.
    /// `e` keeps the first half (`a → v`); returns the new second half (`v → b`).
    pub(crate) fn e_split(&mut self, e: EdgeId, v: VertexId) -> EdgeId {
        let tmpl = self.edges[e].clone();
        let [a, b] = tmpl.v;
        self.e_repoint(e, b, v);
        // e_repoint replaced endpoint b by v: e is now (a, v)
        let e2 = self.e_new(v, b, Some(&tmpl));
        let faces = self.cache.edge_faces.get(e).cloned().unwrap_or_default();
        self.cache.edge_faces.insert(e2, faces.clone());
        for f in faces {
            let face = &mut self.faces[f];
            for l in std::iter::once(&mut face.outer).chain(face.holes.iter_mut()) {
                let n = l.len();
                for i in 0..n {
                    let (x, y) = (l[i], l[(i + 1) % n]);
                    if (x == a && y == b) || (x == b && y == a) {
                        l.insert(i + 1, v);
                        break;
                    }
                }
            }
        }
        e2
    }

    /// If `v` joins exactly two collinear, attribute-compatible edges bounding the same faces,
    /// merge them into one edge and delete `v`. Returns true if healed.
    pub(crate) fn heal_vertex(&mut self, v: VertexId) -> bool {
        if !self.vertices.contains_key(v) {
            return false;
        }
        let es = self.vertex_edges(v);
        if es.len() != 2 {
            return false;
        }
        let (e1, e2) = (es[0], es[1]);
        let (a, b) = (self.edges[e1].other(v), self.edges[e2].other(v));
        if a == b || self.edge_between(a, b).is_some() {
            return false;
        }
        let (pa, pv, pb) = (self.pos(a), self.pos(v), self.pos(b));
        let (d1, d2) = ((pv - pa).normalize_or_zero(), (pb - pv).normalize_or_zero());
        if d1.dot(d2) < 1.0 - 1e-10 {
            return false;
        }
        // Distance of v from line a-b must be tiny too.
        if geom::point_seg_dist(pv, pa, pb).0 > TOL * 0.1 {
            return false;
        }
        let (x, y) = (&self.edges[e1], &self.edges[e2]);
        if x.soft != y.soft || x.smooth != y.smooth || x.hidden != y.hidden || x.tag != y.tag || x.curve != y.curve {
            return false;
        }
        let mut f1 = self.edge_faces(e1);
        let mut f2 = self.edge_faces(e2);
        f1.sort();
        f2.sort();
        if f1 != f2 {
            return false;
        }
        for f in &f1 {
            self.f_unregister(*f);
        }
        self.e_remove(e2);
        self.e_repoint(e1, v, b);
        for f in &f1 {
            let face = &mut self.faces[*f];
            for l in std::iter::once(&mut face.outer).chain(face.holes.iter_mut()) {
                l.retain(|x| *x != v);
            }
        }
        for f in f1 {
            self.f_register(f);
        }
        self.v_remove(v);
        true
    }

    /// Merge vertex `gone` into `keep` (used when moved vertices land on others).
    pub(crate) fn merge_vertices(&mut self, keep: VertexId, gone: VertexId) {
        if keep == gone || !self.vertices.contains_key(keep) || !self.vertices.contains_key(gone) {
            return;
        }
        let faces = self.faces_of_vertex(gone);
        for f in &faces {
            self.f_unregister(*f);
        }
        for e in self.vertex_edges(gone) {
            let other = self.edges[e].other(gone);
            if other == keep || self.edge_between(keep, other).is_some() {
                self.e_remove(e);
            } else {
                self.e_repoint(e, gone, keep);
            }
        }
        for f in faces {
            let face = &mut self.faces[f];
            for l in std::iter::once(&mut face.outer).chain(face.holes.iter_mut()) {
                for x in l.iter_mut() {
                    if *x == gone {
                        *x = keep;
                    }
                }
                dedup_cyclic(l);
            }
            face.holes.retain(|h| h.len() >= 3);
            if face.outer.len() < 3 {
                self.faces.remove(f);
                continue;
            }
            let n = geom::newell(&self.loop_pos(&self.faces[f].outer));
            if n.length() < 1e-12 {
                self.faces.remove(f);
                continue;
            }
            self.f_register(f);
        }
        self.v_remove(gone);
    }

    // ---- faces ---------------------------------------------------------------

    /// Register a face in edge→face caches (creating missing edges) and the face grid.
    pub(crate) fn f_register(&mut self, f: FaceId) {
        let mut pairs = Vec::new();
        for l in self.faces[f].loops() {
            for i in 0..l.len() {
                pairs.push((l[i], l[(i + 1) % l.len()]));
            }
        }
        for (a, b) in pairs {
            let (e, _) = self.e_ensure(a, b);
            let l = self.cache.edge_faces.entry(e).unwrap().or_default();
            if !l.contains(&f) {
                l.push(f);
            }
        }
        self.f_fix_orientation(f);
        self.grid_face(f);
    }

    pub(crate) fn f_unregister(&mut self, f: FaceId) {
        let mut pairs = Vec::new();
        for l in self.faces[f].loops() {
            for i in 0..l.len() {
                pairs.push((l[i], l[(i + 1) % l.len()]));
            }
        }
        for (a, b) in pairs {
            if let Some(e) = self.edge_between(a, b)
                && let Some(l) = self.cache.edge_faces.get_mut(e) {
                    l.retain(|x| *x != f);
                }
        }
        self.cache.fgrid.remove(f);
    }

    /// Recompute the normal from the outer loop and make holes clockwise.
    pub(crate) fn f_fix_orientation(&mut self, f: FaceId) {
        let n = geom::newell(&self.loop_pos(&self.faces[f].outer)).normalize_or_zero();
        let n = if n == DVec3::ZERO { self.faces[f].normal } else { n };
        let flips: Vec<bool> =
            self.faces[f].holes.iter().map(|h| geom::newell(&self.loop_pos(h)).dot(n) > 0.0).collect();
        let face = &mut self.faces[f];
        face.normal = n;
        for (h, flip) in face.holes.iter_mut().zip(flips) {
            if flip {
                h.reverse();
            }
        }
    }

    /// Create a face with loops (edges are created if missing) and attributes from `tmpl`.
    /// The outer loop order defines the normal.
    pub(crate) fn f_new(&mut self, outer: Vec<VertexId>, holes: Vec<Vec<VertexId>>, tmpl: &Face) -> FaceId {
        let face = Face {
            outer,
            holes,
            normal: tmpl.normal,
            front: tmpl.front,
            back: tmpl.back,
            hidden: tmpl.hidden,
            tag: tmpl.tag,
        };
        let f = self.faces.insert(face);
        self.f_register(f);
        f
    }

    pub(crate) fn f_set_loops(&mut self, f: FaceId, outer: Vec<VertexId>, holes: Vec<Vec<VertexId>>) {
        self.f_unregister(f);
        let face = &mut self.faces[f];
        face.outer = outer;
        face.holes = holes;
        self.f_register(f);
    }

    pub(crate) fn f_remove(&mut self, f: FaceId) {
        if !self.faces.contains_key(f) {
            return;
        }
        self.f_unregister(f);
        self.faces.remove(f);
    }

    /// Existing face whose outer loop has exactly the same vertex set as `l`.
    pub(crate) fn find_face_with_outer(&self, l: &[VertexId]) -> Option<FaceId> {
        let first = *l.first()?;
        let mut key: Vec<VertexId> = l.to_vec();
        key.sort();
        for f in self.faces_of_vertex(first) {
            let mut k2 = self.faces[f].outer.clone();
            if k2.len() != key.len() {
                continue;
            }
            k2.sort();
            if k2 == key {
                return Some(f);
            }
        }
        None
    }

    /// Next unused curve id.
    pub(crate) fn next_curve_id(&self) -> u32 {
        self.edges.values().filter_map(|e| e.curve).max().map_or(1, |m| m + 1)
    }
}

/// Remove consecutive duplicates in a cyclic loop.
pub(crate) fn dedup_cyclic(l: &mut Vec<VertexId>) {
    l.dedup();
    while l.len() > 1 && l.first() == l.last() {
        l.pop();
    }
}
