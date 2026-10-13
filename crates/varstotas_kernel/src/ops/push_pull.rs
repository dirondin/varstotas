//! Push/Pull (semantics).
//!
//! - A free-standing face (no boundary edge shared with another face) is extruded into a
//!   closed prism: the original face stays as the base (facing away from the motion), the cap
//!   faces the motion direction.
//! - A face connected to other faces is *moved*: its original location is left open (pulling
//!   out of a solid / pocketing into it). Boundary edges whose neighbours are perpendicular
//!   to the face (and whose corners only have edges parallel to the motion) are *stretched*
//!   instead of getting new side faces — e.g. pulling the top of a box just makes it taller.
//! - New side faces that end up coplanar with a neighbour are merged into it.
//! - Pushing a face (negative distance) exactly onto an opposite parallel face of the solid
//!   cuts a through-hole.
//! - `keep_original` (Ctrl) keeps the original face and always builds side faces.

use crate::geom::{self, PointLoc, Projector};
use crate::mesh::{Mesh, PTOL};
use crate::topology::{Classify, Orient};
use crate::{EdgeId, Face, FaceId, OpError, OpResult, TOL, VertexId};
use glam::DVec3;
use std::collections::{HashMap, HashSet};

impl Mesh {
    /// Push/Pull a face along its normal by `distance` (negative = into the solid).
    /// Creates side faces, removes the original face unless `keep_original` (or the face is
    /// free-standing, in which case it becomes the base of the new prism).
    /// Pushing fully through a solid to a parallel face cuts a hole.
    /// Returns the moved (cap) face; for a through-hole, the face that received the hole.
    pub fn push_pull(&mut self, f: FaceId, distance: f64, keep_original: bool) -> OpResult<FaceId> {
        self.ensure();
        let face = self.faces.get(f).ok_or(OpError::NotFound)?.clone();
        if !distance.is_finite() {
            return Err(OpError::Invalid("distance is not finite".into()));
        }
        if distance.abs() < TOL {
            return Ok(f);
        }
        let n = face.normal;
        let off = n * distance;
        let loops: Vec<Vec<VertexId>> = face.loops().cloned().collect();
        let mut bedges: Vec<(EdgeId, VertexId, VertexId)> = Vec::new();
        for l in &loops {
            for i in 0..l.len() {
                let (a, b) = (l[i], l[(i + 1) % l.len()]);
                let e = self.edge_between(a, b).ok_or_else(|| OpError::Invalid("face loop without edge".into()))?;
                bedges.push((e, a, b));
            }
        }
        let bset: HashSet<EdgeId> = bedges.iter().map(|x| x.0).collect();
        let lone = bedges.iter().all(|(e, _, _)| self.edge_faces(*e).iter().all(|g| *g == f));

        // ---- stretch analysis -------------------------------------------------------
        let mut slid: HashMap<VertexId, bool> = HashMap::new();
        for v in loops.iter().flatten() {
            slid.insert(*v, false);
        }
        if !lone && !keep_original {
            let cand: HashMap<EdgeId, bool> = bedges
                .iter()
                .map(|(e, _, _)| {
                    let others: Vec<FaceId> = self.edge_faces(*e).into_iter().filter(|g| *g != f).collect();
                    let ok = !others.is_empty() && others.iter().all(|g| self.faces[*g].normal.dot(n).abs() < 1e-6);
                    (*e, ok)
                })
                .collect();
            for v in loops.iter().flatten() {
                let es = self.vertex_edges(*v);
                let non_f: Vec<EdgeId> = es.iter().copied().filter(|e| !bset.contains(e)).collect();
                let ok = !non_f.is_empty()
                    && non_f.iter().all(|e| {
                        let (a, b) = self.edge_segment(*e);
                        (b - a).normalize_or_zero().dot(n).abs() > 1.0 - 1e-9
                    })
                    && es.iter().filter(|e| bset.contains(e)).all(|e| cand[e]);
                slid.insert(*v, ok);
            }
            loop {
                let mut changed = false;
                for (e, a, b) in &bedges {
                    if !(cand[e] && slid[a] && slid[b]) {
                        for v in [a, b] {
                            if slid[v] {
                                slid.insert(*v, false);
                                changed = true;
                            }
                        }
                    }
                }
                if !changed {
                    break;
                }
            }
        }
        let any_slid = slid.values().any(|x| *x);

        // ---- through-hole ---------------------------------------------------------------
        if !lone
            && !keep_original
            && distance < 0.0
            && !any_slid
            && let Some(h) = self.find_through_target(f, &loops, off)
        {
            return self.push_through(f, &loops, &bedges, off, h);
        }

        // ---- collapse check for stretched vertices --------------------------------------
        for (v, s) in &slid {
            if !*s {
                continue;
            }
            let pv = self.pos(*v);
            for e in self.vertex_edges(*v) {
                if bset.contains(&e) {
                    continue;
                }
                let w = self.pos(self.edges[e].other(*v));
                let (old, new) = (w - pv, w - (pv + off));
                if new.dot(old) <= 0.0 || new.length() < TOL {
                    return Err(OpError::Invalid("push/pull would collapse geometry".into()));
                }
            }
        }

        // ---- build ------------------------------------------------------------------------
        let mut map: HashMap<VertexId, VertexId> = HashMap::new();
        for v in loops.iter().flatten() {
            if map.contains_key(v) {
                continue;
            }
            let nv = if slid[v] { *v } else { self.v_new(self.pos(*v) + off) };
            map.insert(*v, nv);
        }
        for (v, s) in &slid {
            if *s {
                let p = self.pos(*v) + off;
                self.v_set_pos(*v, p);
            }
        }
        let cap_flip = lone && distance < 0.0;
        let mut cap_loops: Vec<Vec<VertexId>> = loops.iter().map(|l| l.iter().map(|v| map[v]).collect()).collect();
        if cap_flip {
            for l in &mut cap_loops {
                l.reverse();
            }
        }
        let cap_outer = cap_loops.remove(0);
        let cap = if lone || keep_original {
            if lone && distance > 0.0 {
                // The base must face away from the prism.
                let fm = &mut self.faces[f];
                fm.outer.reverse();
                for h in &mut fm.holes {
                    h.reverse();
                }
                self.f_fix_orientation(f);
            }
            self.f_new(cap_outer, cap_loops, &face)
        } else {
            self.f_set_loops(f, cap_outer, cap_loops);
            f
        };
        let mut merge_candidates: Vec<EdgeId> = Vec::new();
        for (e, a, b) in &bedges {
            if slid[a] && slid[b] {
                continue;
            }
            let (a2, b2) = (map[a], map[b]);
            let side = if cap_flip { vec![*b, *a, a2, b2] } else { vec![*a, *b, b2, a2] };
            self.f_new(side, Vec::new(), &Face::template());
            merge_candidates.push(*e);
            for (x, y) in [(*a, a2), (*b, b2)] {
                if let Some(ev) = self.edge_between(x, y)
                    && !merge_candidates.contains(&ev)
                {
                    merge_candidates.push(ev);
                }
            }
        }
        // Vertical edges between two pieces of the same curve (circle/arc) are soft + smooth.
        for l in &loops {
            let k = l.len();
            for i in 0..k {
                let (p, c, nx) = (l[(i + k - 1) % k], l[i], l[(i + 1) % k]);
                if slid[&c] {
                    continue;
                }
                let curve = |x: VertexId, y: VertexId| self.edge_between(x, y).and_then(|e| self.edges[e].curve);
                let (c1, c2) = (curve(p, c), curve(c, nx));
                if c1.is_some()
                    && c1 == c2
                    && let Some(ev) = self.edge_between(c, map[&c])
                {
                    let ed = &mut self.edges[ev];
                    ed.soft = true;
                    ed.smooth = true;
                }
            }
        }
        if !keep_original && !lone {
            self.merge_coplanar_at(&merge_candidates);
            let vs: Vec<VertexId> = loops.iter().flatten().copied().collect();
            self.cleanup_vertices(&vs);
        }
        Ok(if self.faces.contains_key(cap) { cap } else { f })
    }

    fn find_through_target(&self, f: FaceId, loops: &[Vec<VertexId>], off: DVec3) -> Option<FaceId> {
        let n = self.faces[f].normal;
        let cap: Vec<DVec3> = loops[0].iter().map(|v| self.pos(*v) + off).collect();
        let b = crate::Aabb::from_points(cap.iter().copied())?;
        let mut cands = self.cache.fgrid.query_box(b.min, b.max);
        cands.retain(|g| *g != f);
        cands.into_iter().find(|g| {
            let gf = &self.faces[*g];
            if gf.normal.dot(n) > -1.0 + 1e-6 {
                return false;
            }
            let pl = self.face_plane(*g);
            if pl.distance(cap[0]).abs() > PTOL {
                return false;
            }
            cap.iter().all(|p| self.face_point_loc(*g, *p) != PointLoc::Outside)
        })
    }

    fn push_through(
        &mut self,
        f: FaceId,
        loops: &[Vec<VertexId>],
        bedges: &[(EdgeId, VertexId, VertexId)],
        off: DVec3,
        h: FaceId,
    ) -> OpResult<FaceId> {
        let hplane = self.face_plane(h);
        let mut created = Vec::new();
        let mut seeds = Vec::new();
        let cap_pos: Vec<Vec<DVec3>> = loops.iter().map(|l| l.iter().map(|v| self.pos(*v) + off).collect()).collect();
        for l in &cap_pos {
            for i in 0..l.len() {
                let (p, c) = self.insert_segment(l[i], l[(i + 1) % l.len()]);
                for e in p {
                    seeds.extend(self.edges[e].v);
                }
                created.extend(c);
            }
        }
        let set: HashSet<EdgeId> = created.into_iter().collect();
        let regs = self.refresh_plane(hplane, &seeds, &[], &Classify::NewEdges(&set), Orient::prefer(hplane.normal));
        let pr = Projector::new(hplane.origin, hplane.normal);
        let o2: Vec<_> = cap_pos[0].iter().map(|p| pr.to2(*p)).collect();
        let h2: Vec<Vec<_>> = cap_pos[1..].iter().map(|l| l.iter().map(|p| pr.to2(*p)).collect()).collect();
        let mut keep: Option<(f64, FaceId)> = None;
        for r in &regs {
            if !self.faces.contains_key(r.face) {
                continue;
            }
            if geom::point_in_region(pr.to2(r.sample), &o2, &h2, TOL) == PointLoc::Inside {
                self.f_remove(r.face);
            } else if keep.is_none_or(|(a, _)| r.area > a) {
                keep = Some((r.area, r.face));
            }
        }
        let mut map: HashMap<VertexId, VertexId> = HashMap::new();
        for v in loops.iter().flatten() {
            let p = self.pos(*v) + off;
            let w = self.v_find(p, TOL).ok_or_else(|| OpError::Invalid("through-hole: cap vertex missing".into()))?;
            map.insert(*v, w);
        }
        self.f_remove(f);
        let mut merge_candidates = Vec::new();
        for (e, a, b) in bedges {
            let (a2, b2) = (map[a], map[b]);
            self.f_new(vec![*a, *b, b2, a2], Vec::new(), &Face::template());
            merge_candidates.push(*e);
        }
        self.merge_coplanar_at(&merge_candidates);
        Ok(keep.map(|(_, f)| f).filter(|f| self.faces.contains_key(*f)).unwrap_or(h))
    }
}
