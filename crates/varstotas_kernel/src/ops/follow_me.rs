//! Follow Me: sweep a profile face along a path of connected edges.

use crate::mesh::Mesh;
use crate::{EdgeId, EntityRef, Face, FaceId, OpError, OpResult, TOL, VertexId};
use glam::DVec3;
use std::collections::HashMap;

impl Mesh {
    /// Order path edges into a vertex chain. Returns `(vertices, closed)`.
    fn order_path(&self, path: &[EdgeId]) -> OpResult<(Vec<VertexId>, bool)> {
        if path.is_empty() {
            return Err(OpError::Invalid("empty path".into()));
        }
        let mut adj: HashMap<VertexId, Vec<(EdgeId, VertexId)>> = HashMap::new();
        for &e in path {
            let ed = self.edges.get(e).ok_or(OpError::NotFound)?;
            adj.entry(ed.v[0]).or_default().push((e, ed.v[1]));
            adj.entry(ed.v[1]).or_default().push((e, ed.v[0]));
        }
        if adj.values().any(|l| l.len() > 2) {
            return Err(OpError::Invalid("path branches".into()));
        }
        let ends: Vec<VertexId> = adj.iter().filter(|(_, l)| l.len() == 1).map(|(v, _)| *v).collect();
        let closed = ends.is_empty();
        if !closed && ends.len() != 2 {
            return Err(OpError::Invalid("path is not connected".into()));
        }
        let start = if closed { self.edges[path[0]].v[0] } else { ends[0] };
        let mut chain = vec![start];
        let mut used: Vec<EdgeId> = Vec::new();
        let mut cur = start;
        loop {
            let next = adj[&cur].iter().find(|(e, _)| !used.contains(e)).copied();
            let Some((e, w)) = next else { break };
            used.push(e);
            if closed && w == start {
                break;
            }
            chain.push(w);
            cur = w;
        }
        if used.len() != path.len() {
            return Err(OpError::Invalid("path is not connected".into()));
        }
        Ok((chain, closed))
    }

    /// Sweep a profile face along a path of connected edges (open or closed). The profile
    /// stays where it is: the sweep starts at the path end (or closed-path vertex) nearest to
    /// the profile and each section is projected along the path onto the miter plane of the
    /// next path vertex. Open paths get end caps (the profile itself is the start cap);
    /// closed paths produce a closed ring and consume the profile face.
    pub fn follow_me(&mut self, profile: FaceId, path: &[EdgeId]) -> OpResult<()> {
        self.ensure();
        if !self.faces.contains_key(profile) {
            return Err(OpError::NotFound);
        }
        let (mut chain, closed) = self.order_path(path)?;
        let face0 = self.faces[profile].clone();
        let centroid = {
            let p = self.loop_pos(&face0.outer);
            p.iter().copied().sum::<DVec3>() / p.len() as f64
        };
        if closed {
            let k = (0..chain.len())
                .min_by(|a, b| {
                    (self.pos(chain[*a]) - centroid).length().total_cmp(&(self.pos(chain[*b]) - centroid).length())
                })
                .unwrap();
            chain.rotate_left(k);
        } else {
            let (d0, d1) = (
                (self.pos(chain[0]) - centroid).length(),
                (self.pos(*chain.last().unwrap()) - centroid).length(),
            );
            if d1 < d0 {
                chain.reverse();
            }
        }
        let mut pts: Vec<DVec3> = chain.iter().map(|v| self.pos(*v)).collect();
        if closed {
            pts.push(pts[0]);
        }
        let m = pts.len() - 1; // number of segments
        let dirs: Vec<DVec3> = (0..m).map(|i| (pts[i + 1] - pts[i]).normalize_or_zero()).collect();
        if dirs.contains(&DVec3::ZERO) {
            return Err(OpError::Degenerate);
        }
        if face0.normal.dot(dirs[0]).abs() < 1e-6 {
            return Err(OpError::Invalid("profile is parallel to the path".into()));
        }
        // Start cap must face against the sweep direction.
        if face0.normal.dot(dirs[0]) > 0.0 {
            self.reverse_face(profile);
            let f = &mut self.faces[profile];
            std::mem::swap(&mut f.front, &mut f.back);
        }
        let face = self.faces[profile].clone();
        let loops: Vec<Vec<VertexId>> = face.loops().cloned().collect();
        let base: Vec<Vec<DVec3>> = loops.iter().map(|l| self.loop_pos(l)).collect();

        // Miter plane normal at path vertex i.
        let miter = |i: usize| -> OpResult<DVec3> {
            if closed {
                let prev = dirs[(i + m - 1) % m];
                let next = dirs[i % m];
                let s = prev + next;
                if s.length() < 1e-9 { Err(OpError::Invalid("path reverses".into())) } else { Ok(s.normalize()) }
            } else if i == 0 {
                Ok(dirs[0])
            } else if i == m {
                Ok(dirs[m - 1])
            } else {
                let s = dirs[i - 1] + dirs[i];
                if s.length() < 1e-9 { Err(OpError::Invalid("path reverses".into())) } else { Ok(s.normalize()) }
            }
        };
        let project = |pts_in: &[Vec<DVec3>], d: DVec3, origin: DVec3, nm: DVec3| -> OpResult<Vec<Vec<DVec3>>> {
            let den = d.dot(nm);
            if den.abs() < 1e-9 {
                return Err(OpError::Degenerate);
            }
            Ok(pts_in.iter().map(|l| l.iter().map(|x| *x + d * ((origin - *x).dot(nm) / den)).collect()).collect())
        };
        // Sections (positions) at each path vertex.
        let mut sections: Vec<Vec<Vec<DVec3>>> = Vec::with_capacity(m + 1);
        if closed {
            sections.push(project(&base, dirs[0], pts[0], miter(0)?)?);
        } else {
            sections.push(base.clone());
        }
        for i in 1..=m {
            let prev = sections[i - 1].clone();
            let nm = if closed && i == m { miter(0)? } else { miter(i)? };
            sections.push(project(&prev, dirs[i - 1], pts[i], nm)?);
        }
        // Vertex ids per section.
        let mut ids: Vec<Vec<Vec<VertexId>>> = Vec::with_capacity(m + 1);
        for (si, sec) in sections.iter().enumerate() {
            if si == 0 && !closed {
                ids.push(loops.clone());
                continue;
            }
            if si == m && closed {
                let first = ids[0].clone();
                ids.push(first);
                continue;
            }
            let mut sl = Vec::new();
            for l in sec {
                sl.push(l.iter().map(|p| self.v_get_or_new(*p)).collect::<Vec<_>>());
            }
            ids.push(sl);
        }
        let mut new_edges: Vec<EdgeId> = Vec::new();
        let tmpl = Face { front: face.front, back: face.back, ..Face::template() };
        for li in 0..loops.len() {
            let k = loops[li].len();
            for j in 0..k {
                for s in 0..m {
                    let (a0, b0) = (ids[s][li][j], ids[s][li][(j + 1) % k]);
                    let (a1, b1) = (ids[s + 1][li][j], ids[s + 1][li][(j + 1) % k]);
                    let quad = vec![b0, a0, a1, b1];
                    let mut q = quad.clone();
                    q.dedup();
                    if q.len() < 4 || q[0] == q[3] {
                        continue;
                    }
                    if crate::geom::newell(&self.loop_pos(&quad)).length() < TOL * TOL {
                        continue;
                    }
                    self.f_new(quad, Vec::new(), &tmpl);
                    for (x, y) in [(a0, a1), (b0, b1), (a1, b1)] {
                        if let Some(e) = self.edge_between(x, y) {
                            new_edges.push(e);
                        }
                    }
                }
            }
        }
        if closed {
            // The ring is closed: the profile (and its edges if unused) are consumed.
            let profile_loops_moved = ids[0] != loops;
            self.f_remove(profile);
            if profile_loops_moved {
                let es: Vec<EdgeId> = loops
                    .iter()
                    .flat_map(|l| (0..l.len()).filter_map(|i| self.edge_between(l[i], l[(i + 1) % l.len()])).collect::<Vec<_>>())
                    .filter(|e| self.edge_faces(*e).is_empty())
                    .collect();
                let ents: Vec<EntityRef> = es.into_iter().map(EntityRef::Edge).collect();
                self.erase(&ents);
            }
        } else {
            let mut end: Vec<Vec<VertexId>> = ids[m].clone();
            for l in &mut end {
                l.reverse();
            }
            let outer = end.remove(0);
            self.f_new(outer, end, &face);
        }
        self.merge_coplanar_at(&new_edges);
        Ok(())
    }
}
