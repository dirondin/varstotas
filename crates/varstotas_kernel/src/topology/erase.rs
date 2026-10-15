//! Eraser semantics: erasing an edge removes the faces it bounds, except that an
//! edge separating two coplanar faces merges them; erasing a face keeps its edges.

use super::{Classify, Orient};
use crate::mesh::{Mesh, PTOL};
use crate::{EdgeId, EntityRef, FaceId, Plane, VertexId};
use std::collections::HashSet;

impl Mesh {
    /// Are two faces coplanar (either orientation)?
    pub(crate) fn coplanar(&self, f: FaceId, g: FaceId) -> bool {
        let (a, b) = (&self.faces[f], &self.faces[g]);
        if a.normal.dot(b.normal).abs() < 1.0 - 1e-6 {
            return false;
        }
        let pl = self.face_plane(f);
        b.outer.iter().all(|v| pl.distance(self.pos(*v)).abs() <= PTOL)
    }

    /// classic eraser semantics: erasing an edge removes faces bounded by it; an edge shared by
    /// exactly two coplanar faces is removed and the faces merge into one. Erasing a face keeps
    /// its edges. Erasing a vertex erases its edges. Collinear edge pieces left behind are healed
    /// and orphan vertices are removed.
    pub fn erase(&mut self, ents: &[EntityRef]) {
        self.ensure();
        let mut kill_faces: HashSet<FaceId> = HashSet::new();
        let mut kill_edges: Vec<EdgeId> = Vec::new();
        let mut touched: Vec<VertexId> = Vec::new();
        for ent in ents {
            match *ent {
                EntityRef::Face(f) => {
                    if self.faces.contains_key(f) {
                        kill_faces.insert(f);
                    }
                }
                EntityRef::Edge(e) => {
                    if self.edges.contains_key(e) && !kill_edges.contains(&e) {
                        kill_edges.push(e);
                    }
                }
                EntityRef::Vertex(v) => {
                    if self.vertices.contains_key(v) {
                        touched.push(v);
                        for e in self.vertex_edges(v) {
                            if !kill_edges.contains(&e) {
                                kill_edges.push(e);
                            }
                        }
                    }
                }
                EntityRef::Instance(i) => {
                    self.instances.remove(i);
                }
            }
        }
        // Decide merges vs face removal.
        let mut merges: Vec<(EdgeId, FaceId, FaceId)> = Vec::new();
        for &e in &kill_edges {
            let fs: Vec<FaceId> = self.edge_faces(e).into_iter().filter(|f| !kill_faces.contains(f)).collect();
            if fs.len() == 2 && fs[0] != fs[1] && self.coplanar(fs[0], fs[1]) {
                merges.push((e, fs[0], fs[1]));
            } else {
                kill_faces.extend(fs);
            }
        }
        merges.retain(|(_, a, b)| !kill_faces.contains(a) && !kill_faces.contains(b));
        // Edges whose two faces get merged by another erased edge but that are "boundary only"
        // are fine; collect the faces to re-evaluate per plane.
        let mut merge_faces: Vec<FaceId> = Vec::new();
        for (_, a, b) in &merges {
            for f in [*a, *b] {
                if !merge_faces.contains(&f) {
                    merge_faces.push(f);
                }
            }
        }
        for f in kill_faces {
            self.f_remove(f);
        }
        for &e in &kill_edges {
            if let Some(ed) = self.edges.get(e) {
                touched.extend(ed.v);
            }
            self.e_remove(e);
        }
        // Re-evaluate each affected plane: merged regions inherit from the old faces.
        let mut done: HashSet<FaceId> = HashSet::new();
        for &f in &merge_faces {
            if done.contains(&f) || !self.faces.contains_key(f) {
                continue;
            }
            let plane: Plane = self.face_plane(f);
            let group: Vec<FaceId> = merge_faces
                .iter()
                .copied()
                .filter(|g| self.faces.contains_key(*g) && !done.contains(g) && self.coplanar(f, *g))
                .collect();
            done.extend(group.iter().copied());
            let seeds: Vec<VertexId> = group.iter().flat_map(|g| self.face_vertices(*g)).collect();
            self.refresh_plane(plane, &seeds, &group, &Classify::None, Orient::auto());
        }
        self.cleanup_vertices(&touched);
    }

    /// Heal collinear degree-2 vertices and drop orphans among `vs`.
    pub(crate) fn cleanup_vertices(&mut self, vs: &[VertexId]) {
        for &v in vs {
            if !self.vertices.contains_key(v) {
                continue;
            }
            if self.vertex_edges(v).is_empty() {
                self.v_remove(v);
            } else {
                self.heal_vertex(v);
            }
        }
    }

    /// Merge edge-adjacent faces among `faces` that are coplanar with the same orientation and
    /// attributes (materials, tag, hidden) into single faces, like erasing the edges between
    /// them, but one region rebuild per merged group (importers produce many triangles).
    /// Returns the number of faces removed.
    pub(crate) fn merge_coplanar_faces(&mut self, faces: &[FaceId]) -> usize {
        self.ensure();
        let set: HashSet<FaceId> = faces.iter().copied().filter(|f| self.faces.contains_key(*f)).collect();
        let same_attrs = |a: &crate::Face, b: &crate::Face| {
            a.normal.dot(b.normal) >= 1.0 - 1e-6 && a.front == b.front && a.back == b.back && a.tag == b.tag && a.hidden == b.hidden
                && match (&a.uv, &b.uv) {
                    (Some(x), Some(y)) => x.approx_eq(y),
                    (x, y) => x.is_none() && y.is_none(),
                }
        };
        let mut done: HashSet<FaceId> = HashSet::new();
        let mut groups: Vec<(Vec<FaceId>, Vec<EdgeId>)> = Vec::new();
        for &f in faces {
            if !set.contains(&f) || done.contains(&f) {
                continue;
            }
            done.insert(f);
            // Flood over shared edges; every member must lie on the seed's plane (no drift
            // along gently curved surfaces).
            let plane = self.face_plane(f);
            let mut group = vec![f];
            let mut in_group: HashSet<FaceId> = HashSet::from([f]);
            let mut inner: Vec<EdgeId> = Vec::new();
            let mut inner_set: HashSet<EdgeId> = HashSet::new();
            let mut k = 0;
            while k < group.len() {
                let g = group[k];
                k += 1;
                let loops: Vec<Vec<VertexId>> = self.faces[g].loops().cloned().collect();
                for l in &loops {
                    for i in 0..l.len() {
                        let Some(e) = self.edge_between(l[i], l[(i + 1) % l.len()]) else { continue };
                        let fs = self.edge_faces(e);
                        if fs.len() != 2 || fs[0] == fs[1] {
                            continue;
                        }
                        let h = if fs[0] == g { fs[1] } else { fs[0] };
                        if !set.contains(&h) || !same_attrs(&self.faces[g], &self.faces[h]) {
                            continue;
                        }
                        if !done.contains(&h) {
                            if self.faces[h].loops().flatten().any(|v| plane.distance(self.pos(*v)).abs() > PTOL) {
                                continue;
                            }
                            done.insert(h);
                            in_group.insert(h);
                            group.push(h);
                        } else if !in_group.contains(&h) {
                            continue;
                        }
                        if inner_set.insert(e) {
                            inner.push(e);
                        }
                    }
                }
            }
            if group.len() > 1 {
                groups.push((group, inner));
            }
        }
        let mut removed = 0;
        for (group, inner) in groups {
            let plane = self.face_plane(group[0]);
            // Interior samples of the faces (strictly inside each, so strictly inside the
            // merged region containing it; boundary points of the union are never used).
            let mut samples = Vec::with_capacity(group.len());
            let mut seeds: Vec<VertexId> = Vec::new();
            for &g in &group {
                let face = self.faces[g].clone();
                seeds.extend(face.loops().flatten().copied());
                let pr = crate::geom::Projector::new(plane.origin, plane.normal);
                let o: Vec<_> = face.outer.iter().map(|v| pr.to2(self.pos(*v))).collect();
                let h: Vec<Vec<_>> = face.holes.iter().map(|l| l.iter().map(|v| pr.to2(self.pos(*v))).collect()).collect();
                samples.push((pr.to3(super::interior_sample(&o, &h)), face));
                self.f_remove(g);
            }
            for e in inner {
                if self.edges.contains_key(e) && self.edge_faces(e).is_empty() {
                    self.e_remove(e);
                }
            }
            seeds.sort();
            seeds.dedup();
            let made = self.refresh_plane(plane, &seeds, &[], &Classify::Covered(&samples), Orient::auto());
            removed += group.len().saturating_sub(made.len());
            self.cleanup_vertices(&seeds);
        }
        removed
    }

    /// For each edge in `edges` separating two coplanar faces with the same orientation,
    /// erase it so the faces merge (used after push/pull, follow-me and folding).
    pub(crate) fn merge_coplanar_at(&mut self, edges: &[EdgeId]) {
        let mergeable = |m: &Self, e: EdgeId| {
            let fs = m.edge_faces(e);
            if fs.len() != 2 || fs[0] == fs[1] {
                return false;
            }
            let (a, b) = (&m.faces[fs[0]], &m.faces[fs[1]]);
            a.normal.dot(b.normal) >= 1.0 - 1e-6 && a.front == b.front && a.back == b.back && m.coplanar(fs[0], fs[1])
        };
        let eligible: Vec<EdgeId> = edges.iter().copied().filter(|e| self.edges.contains_key(*e) && mergeable(self, *e)).collect();
        for &e in &eligible {
            if !self.edges.contains_key(e) {
                continue;
            }
            // Erasing an earlier edge may already have merged this edge's faces, leaving it
            // inside one face or without faces; it goes too, whatever the processing order.
            let fs = self.edge_faces(e);
            if mergeable(self, e) || fs.is_empty() || (fs.len() == 2 && fs[0] == fs[1]) {
                self.erase(&[EntityRef::Edge(e)]);
            }
        }
    }
}
