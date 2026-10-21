//! Follow Me: sweep a profile face along a path of connected edges.

use crate::mesh::Mesh;
use crate::{EdgeId, EntityRef, Face, FaceId, OpError, OpResult, TOL, VertexId};
use glam::DVec3;
use std::collections::HashMap;
use varstotas_i18n::tr;

impl Mesh {
    /// Order path edges into a vertex chain. Returns `(vertices, closed)`.
    fn order_path(&self, path: &[EdgeId]) -> OpResult<(Vec<VertexId>, bool)> {
        if path.is_empty() {
            return Err(OpError::Invalid(tr!("kernel-err-empty-path").into()));
        }
        let mut adj: HashMap<VertexId, Vec<(EdgeId, VertexId)>> = HashMap::new();
        for &e in path {
            let ed = self.edges.get(e).ok_or(OpError::NotFound)?;
            adj.entry(ed.v[0]).or_default().push((e, ed.v[1]));
            adj.entry(ed.v[1]).or_default().push((e, ed.v[0]));
        }
        if adj.values().any(|l| l.len() > 2) {
            return Err(OpError::Invalid(tr!("kernel-err-path-branches").into()));
        }
        let ends: Vec<VertexId> = adj.iter().filter(|(_, l)| l.len() == 1).map(|(v, _)| *v).collect();
        let closed = ends.is_empty();
        if !closed && ends.len() != 2 {
            return Err(OpError::Invalid(tr!("kernel-err-path-is-not-connected").into()));
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
            return Err(OpError::Invalid(tr!("kernel-err-path-is-not-connected").into()));
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
        if pts.windows(2).any(|w| (w[1] - w[0]).length() < 1e-9) {
            return Err(OpError::Degenerate);
        }
        let d0 = (pts[1] - pts[0]).normalize();
        if face0.normal.dot(d0).abs() < 1e-6 {
            return Err(OpError::Invalid(tr!("kernel-err-profile-is-parallel-to-the-path").into()));
        }
        // Start cap must face against the sweep direction.
        if face0.normal.dot(d0) > 0.0 {
            self.reverse_face(profile);
            let f = &mut self.faces[profile];
            std::mem::swap(&mut f.front, &mut f.back);
        }
        let face = self.faces[profile].clone();
        let loops: Vec<Vec<VertexId>> = face.loops().cloned().collect();
        let base: Vec<Vec<DVec3>> = loops.iter().map(|l| self.loop_pos(l)).collect();
        // A path vertex whose section would land behind the previous one (a profile wider than
        // the bend, or tilted against short end segments of a curve) folds the sweep inside out:
        // such vertices are skipped and the sweep turns there with one mitered section.
        let sections = loop {
            match sweep_sections(&pts, closed, &base)? {
                Ok(s) => break s,
                Err(fold) => {
                    pts.remove(fold);
                    chain.remove(fold);
                }
            }
        };
        let m = pts.len() - 1; // number of segments
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
        // A curve's inner vertex (path or profile) gives soft, smooth edges, as drawn curves do.
        let gentle = |a: DVec3, b: DVec3, c: DVec3| (b - a).normalize_or_zero().dot((c - b).normalize_or_zero()) > 0.85;
        let same_curve = |m: &Mesh, a: VertexId, b: VertexId, c: VertexId| {
            let cv = |x, y| m.edge_between(x, y).and_then(|e| m.edges[e].curve);
            cv(a, b).is_some_and(|k| cv(b, c) == Some(k)) && gentle(m.pos(a), m.pos(b), m.pos(c))
        };
        let path_smooth: Vec<bool> = (0..=m)
            .map(|i| {
                let (prev, next) = match (i, closed) {
                    (0, false) => return false,
                    (i, false) if i == m => return false,
                    (i, _) => (chain[(i + chain.len() - 1) % chain.len()], chain[(i + 1) % chain.len()]),
                };
                same_curve(self, prev, chain[i % chain.len()], next)
            })
            .collect();
        let profile_smooth: Vec<Vec<bool>> = loops
            .iter()
            .map(|l| (0..l.len()).map(|j| same_curve(self, l[(j + l.len() - 1) % l.len()], l[j], l[(j + 1) % l.len()])).collect())
            .collect();
        let mut soften: Vec<EdgeId> = Vec::new();
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
                    for (x, y, soft) in [(a0, a1, profile_smooth[li][j]), (a1, b1, path_smooth[s + 1]), (a0, b0, s == 0 && path_smooth[0])] {
                        if soft && let Some(e) = self.edge_between(x, y) {
                            soften.push(e);
                        }
                    }
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
        for e in soften {
            if let Some(ed) = self.edges.get_mut(e) {
                ed.soft = true;
                ed.smooth = true;
            }
        }
        self.merge_coplanar_at(&new_edges);
        Ok(())
    }
}

/// The sections of `base` at each path vertex: each one is the previous section projected along
/// its segment onto the miter plane of the next vertex (the closed path's first section is the
/// profile projected onto the miter of its start). `Ok(Err(i))`: the sweep folds back at path
/// vertex `i`, which can be dropped.
#[allow(clippy::type_complexity)]
fn sweep_sections(pts: &[DVec3], closed: bool, base: &[Vec<DVec3>]) -> OpResult<Result<Vec<Vec<Vec<DVec3>>>, usize>> {
    let m = pts.len() - 1;
    let dirs: Vec<DVec3> = (0..m).map(|i| (pts[i + 1] - pts[i]).normalize()).collect();
    let miter = |i: usize| -> OpResult<DVec3> {
        let (prev, next) = if closed {
            (dirs[(i + m - 1) % m], dirs[i % m])
        } else if i == 0 {
            return Ok(dirs[0]);
        } else if i == m {
            return Ok(dirs[m - 1]);
        } else {
            (dirs[i - 1], dirs[i])
        };
        let s = prev + next;
        if s.length() < 1e-9 { Err(OpError::Invalid(tr!("kernel-err-path-reverses").into())) } else { Ok(s.normalize()) }
    };
    let project = |pts_in: &[Vec<DVec3>], d: DVec3, origin: DVec3, nm: DVec3| -> OpResult<Vec<Vec<DVec3>>> {
        let den = d.dot(nm);
        if den.abs() < 1e-9 {
            return Err(OpError::Degenerate);
        }
        Ok(pts_in.iter().map(|l| l.iter().map(|x| *x + d * ((origin - *x).dot(nm) / den)).collect()).collect())
    };
    let mut sections: Vec<Vec<Vec<DVec3>>> = Vec::with_capacity(m + 1);
    sections.push(if closed { project(base, dirs[0], pts[0], miter(0)?)? } else { base.to_vec() });
    for i in 1..=m {
        let nm = if closed && i == m { miter(0)? } else { miter(i)? };
        let next = project(&sections[i - 1], dirs[i - 1], pts[i], nm)?;
        let back = sections[i - 1].iter().flatten().zip(next.iter().flatten()).any(|(a, b)| (*b - *a).dot(dirs[i - 1]) < -TOL);
        if back && m > 1 {
            // Drop the vertex the section was projected onto, or (on the last segment, where
            // that is the path's end) the one before it.
            let drop = if i < m { i } else { i - 1 };
            if drop > 0 {
                return Ok(Err(drop));
            }
        }
        sections.push(next);
    }
    Ok(Ok(sections))
}
