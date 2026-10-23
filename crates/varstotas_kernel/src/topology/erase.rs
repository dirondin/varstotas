//! Eraser semantics: erasing an edge removes the faces it bounds, except that an
//! edge separating two coplanar faces merges them; erasing a face keeps its edges.

use super::{Classify, Orient};
use crate::mesh::{Mesh, PTOL};
use crate::{EdgeId, EntityRef, Face, FaceId, Plane, VertexId};
use std::collections::{HashMap,HashSet};

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

    /// Join only existing manifold boundaries. The interactive planar region finder can erase
    /// unrelated overlapping faces, so import simplification must never call it.
    pub(crate) fn merge_coplanar_faces(&mut self, faces: &[FaceId]) -> usize {
        self.ensure();
        let mut removed = 0;
        let eligible: HashSet<_> = faces.iter().copied().collect();
        let mut seen = HashSet::new();
        for &seed in faces {
            if !self.faces.contains_key(seed) || !seen.insert(seed) {
                continue;
            }
            let template = self.faces[seed].clone();
            let origin = self.vertices[template.outer[0]].pos;
            let attrs = |a: &Face, b: &Face| {
                a.normal.dot(b.normal) > 1.0 - 1e-12
                    && a.front == b.front
                    && a.back == b.back
                    && a.hidden == b.hidden
                    && a.locked == b.locked
                    && a.tag == b.tag
                    && match (a.uv, b.uv) {
                        (Some(a), Some(b)) => a.approx_eq(&b),
                        (None, None) => true,
                        _ => false,
                    }
            };
            let mut group = vec![seed];
            let mut i = 0;
            while i < group.len() {
                for e in self.face_edges(group[i]) {
                    let fs = self.edge_faces(e);
                    if fs.len() != 2 {
                        continue;
                    }
                    for f in fs {
                        if !eligible.contains(&f)
                            || seen.contains(&f)
                            || !attrs(&template, &self.faces[f])
                        {
                            continue;
                        }
                        if self.faces[f].loops().flatten().any(|v| {
                            let p = self.vertices[*v].pos - origin;
                            p.dot(template.normal).abs() > 1e-10 * p.length().max(1.0)
                        }) {
                            continue;
                        }
                        seen.insert(f);
                        group.push(f);
                    }
                }
                i += 1;
            }
            if group.len() < 2 {
                continue;
            }
            let mut boundary = HashSet::new();
            let mut invalid = false;
            for &f in &group {
                for lp in self.faces[f].loops() {
                    for i in 0..lp.len() {
                        let (a, b) = (lp[i], lp[(i + 1) % lp.len()]);
                        if !boundary.remove(&(b, a)) && !boundary.insert((a, b)) {
                            invalid = true;
                        }
                    }
                }
            }
            let mut next = HashMap::new();
            let mut ends = HashSet::new();
            for (a, b) in boundary {
                if next.insert(a, b).is_some() || !ends.insert(b) {
                    invalid = true;
                }
            }
            if invalid || next.keys().any(|v| !ends.contains(v)) {
                continue;
            }
            let mut loops = Vec::new();
            while let Some(&start) = next.keys().min() {
                let mut lp = vec![start];
                let mut v = start;
                loop {
                    let Some(w) = next.remove(&v) else {
                        invalid = true;
                        break;
                    };
                    if w == start {
                        break;
                    }
                    lp.push(w);
                    v = w;
                }
                if invalid || lp.len() < 3 {
                    invalid = true;
                    break;
                }
                loops.push(lp);
            }
            if invalid {
                continue;
            }
            let is_outer = |lp: &Vec<VertexId>| {
                let ps: Vec<_> = lp.iter().map(|v| self.vertices[*v].pos - origin).collect();
                crate::geom::newell(&ps).dot(template.normal) > 0.0
            };
            let outers: Vec<_> = loops
                .iter()
                .enumerate()
                .filter(|(_, lp)| is_outer(lp))
                .map(|(i, _)| i)
                .collect();
            if outers.len() != 1 {
                continue;
            }
            let outer = loops.remove(outers[0]);
            let edges: HashSet<_> = group.iter().flat_map(|f| self.face_edges(*f)).collect();
            let seeds: Vec<_> = group
                .iter()
                .flat_map(|f| self.faces[*f].loops().flatten().copied())
                .collect();
            removed += group.len() - 1;
            for f in group {
                self.f_remove(f);
            }
            self.f_new(outer, loops, &template);
            for e in edges {
                if self.edge_faces(e).is_empty() {
                    self.e_remove(e);
                }
            }
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
