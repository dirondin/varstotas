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
//! - A face that is not the boundary of a solid (a "sheet": nothing behind it, e.g. one half
//!   of a split floor) is extruded like a free-standing face, keeping its base.
//! - New geometry is welded to coincident existing vertices/edges, and new faces overlapping
//!   existing coplanar faces are overlaid: opposite faces cancel (a wall shared by two
//!   adjacent rooms disappears where both sides are solid).

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
        let free = bedges.iter().all(|(e, _, _)| self.edge_faces(*e).iter().all(|g| *g == f));
        let lone = free || (!keep_original && !self.has_solid_behind(f, &bedges));

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
        // Merge candidates are kept as segments: welding may split the edges.
        let mut merge_segs: Vec<(DVec3, DVec3)> = Vec::new();
        let mut created: Vec<FaceId> = vec![cap];
        for (e, a, b) in &bedges {
            if slid[a] && slid[b] {
                continue;
            }
            let (a2, b2) = (map[a], map[b]);
            let side = if cap_flip { vec![*b, *a, a2, b2] } else { vec![*a, *b, b2, a2] };
            created.push(self.f_new(side, Vec::new(), &Face::template()));
            if !lone {
                merge_segs.push(self.edge_segment(*e));
            }
            for (x, y) in [(*a, a2), (*b, b2)] {
                if x != y {
                    merge_segs.push((self.pos(x), self.pos(y)));
                }
            }
        }
        self.soften_sides(&loops, &map);
        let new_verts: Vec<VertexId> = loops.iter().flatten().filter(|v| !slid[*v]).map(|v| map[v]).collect();
        self.weld(&new_verts);
        let mut touched = self.resolve_overlaps(&created);
        let mut cands: Vec<EdgeId> = Vec::new();
        for (a, b) in &merge_segs {
            for e in self.cache.egrid.query_segment(*a, *b) {
                if cands.contains(&e) || !self.edges.contains_key(e) {
                    continue;
                }
                let (p, q) = self.edge_segment(e);
                if geom::point_seg_dist(p, *a, *b).0 <= TOL && geom::point_seg_dist(q, *a, *b).0 <= TOL {
                    cands.push(e);
                }
            }
        }
        if !keep_original {
            self.merge_coplanar_at(&cands);
            touched.extend(loops.iter().flatten().copied());
            touched.extend(new_verts);
            self.cleanup_vertices(&touched);
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
        self.soften_sides(loops, &map);
        self.merge_coplanar_at(&merge_candidates);
        Ok(keep.map(|(_, f)| f).filter(|f| self.faces.contains_key(*f)).unwrap_or(h))
    }

    /// Vertical edges between two pieces of the same curve (circle/arc) are soft + smooth.
    fn soften_sides(&mut self, loops: &[Vec<VertexId>], map: &HashMap<VertexId, VertexId>) {
        for l in loops {
            let k = l.len();
            for i in 0..k {
                let (p, c, nx) = (l[(i + k - 1) % k], l[i], l[(i + 1) % k]);
                if map[&c] == c {
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
    }

    /// Is face `f` part of a solid's boundary (material behind it, against its normal)?
    /// True if a non-coplanar neighbour behind the face is consistently oriented with it, or a
    /// ray from just behind the face crosses the rest of the surface an odd number of times.
    fn has_solid_behind(&self, f: FaceId, bedges: &[(EdgeId, VertexId, VertexId)]) -> bool {
        let n = self.faces[f].normal;
        let o = self.pos(bedges[0].1);
        for (e, a, b) in bedges {
            for g in self.edge_faces(*e) {
                if g == f || self.faces[g].normal.dot(n).abs() > 1.0 - 1e-6 {
                    continue;
                }
                if self.face_traverses(g, *a, *b) != Some(false) {
                    continue;
                }
                let gl = self.loop_pos(&self.faces[g].outer);
                let c = gl.iter().copied().sum::<DVec3>() / gl.len() as f64;
                if (c - o).dot(n) < -TOL {
                    return true;
                }
            }
        }
        let (pr, o2, h2) = self.face_2d(f);
        let start = pr.to3(crate::topology::interior_sample(&o2, &h2)) - n * (10.0 * TOL);
        let dir = (-n + DVec3::new(0.0123, 0.0371, 0.0219)).normalize();
        let fp: *const Face = &self.faces[f];
        let mut origin = start;
        let mut hits = 0;
        for _ in 0..1000 {
            match self.raycast_faces_filtered(origin, dir, |x| !std::ptr::eq(x, fp)) {
                Some((_, t)) => {
                    hits += 1;
                    origin += dir * (t + TOL * 0.1);
                }
                None => break,
            }
        }
        hits % 2 == 1
    }

    /// Split edge `e` at vertex `v` (lying on it), updating the faces that use it. Pieces that
    /// coincide with existing edges reuse them.
    pub(crate) fn split_edge_at(&mut self, e: EdgeId, v: VertexId) -> [EdgeId; 2] {
        let tmpl = self.edges[e].clone();
        let [a, b] = tmpl.v;
        let faces = self.edge_faces(e);
        for f in &faces {
            self.f_unregister(*f);
            let face = &mut self.faces[*f];
            for l in std::iter::once(&mut face.outer).chain(face.holes.iter_mut()) {
                let k = l.len();
                if let Some(i) = (0..k).find(|i| {
                    let (x, y) = (l[*i], l[(*i + 1) % k]);
                    (x == a && y == b) || (x == b && y == a)
                }) {
                    l.insert(i + 1, v);
                    break;
                }
            }
        }
        self.e_remove(e);
        let e1 = match self.edge_between(a, v) {
            Some(x) => x,
            None => self.e_new(a, v, Some(&tmpl)),
        };
        let e2 = match self.edge_between(v, b) {
            Some(x) => x,
            None => self.e_new(v, b, Some(&tmpl)),
        };
        for f in faces {
            self.f_register(f);
        }
        [e1, e2]
    }

    /// Make new vertices sticky: merge them with coincident vertices, split edges they lie on,
    /// and split their edges at existing vertices lying on them.
    pub(crate) fn weld(&mut self, verts: &[VertexId]) {
        let d = DVec3::splat(TOL);
        let mut alive: Vec<VertexId> = Vec::new();
        for &v in verts {
            if !self.vertices.contains_key(v) {
                continue;
            }
            let p = self.pos(v);
            let w = self
                .cache
                .vgrid
                .query_box(p - d, p + d)
                .into_iter()
                .filter(|w| *w != v && self.vertices.get(*w).is_some_and(|x| (x.pos - p).length() <= TOL))
                .min_by(|x, y| (self.pos(*x) - p).length().total_cmp(&(self.pos(*y) - p).length()));
            match w {
                Some(w) => {
                    self.merge_vertices(w, v);
                    if !alive.contains(&w) {
                        alive.push(w);
                    }
                }
                None => {
                    if !alive.contains(&v) {
                        alive.push(v);
                    }
                }
            }
        }
        let interior = |m: &Mesh, p: DVec3, e: EdgeId| {
            let (a, b) = m.edge_segment(e);
            geom::point_seg_dist(p, a, b).0 <= TOL && (p - a).length() > TOL && (p - b).length() > TOL
        };
        for &v in &alive {
            loop {
                if !self.vertices.contains_key(v) {
                    break;
                }
                let p = self.pos(v);
                let hit = self
                    .cache
                    .egrid
                    .query_box(p - d, p + d)
                    .into_iter()
                    .find(|e| self.edges.get(*e).is_some_and(|ed| !ed.has(v)) && interior(self, p, *e));
                match hit {
                    Some(e) => {
                        self.split_edge_at(e, v);
                    }
                    None => break,
                }
            }
        }
        let mut work: Vec<EdgeId> =
            alive.iter().filter(|v| self.vertices.contains_key(**v)).flat_map(|v| self.vertex_edges(*v)).collect();
        while let Some(e) = work.pop() {
            let Some(ed) = self.edges.get(e).cloned() else { continue };
            let (a, b) = self.edge_segment(e);
            let mut best: Option<(f64, VertexId)> = None;
            for w in self.cache.vgrid.query_segment(a, b) {
                if ed.has(w) || !self.vertices.contains_key(w) {
                    continue;
                }
                let p = self.pos(w);
                if interior(self, p, e) {
                    let t = (p - a).length();
                    if best.is_none_or(|(bt, _)| t < bt) {
                        best = Some((t, w));
                    }
                }
            }
            if let Some((_, w)) = best {
                work.extend(self.split_edge_at(e, w));
            }
        }
    }

    /// Overlay each face of `faces` with existing coplanar faces it overlaps: coverage with
    /// opposite orientations cancels. Returns the vertices involved.
    pub(crate) fn resolve_overlaps(&mut self, faces: &[FaceId]) -> Vec<VertexId> {
        let mut touched = Vec::new();
        for &s in faces {
            if !self.faces.contains_key(s) {
                continue;
            }
            let plane = self.face_plane(s);
            let n = plane.normal;
            let pr = Projector::new(plane.origin, n);
            let to2 = |m: &Mesh, l: &[VertexId]| l.iter().map(|v| pr.to2(m.pos(*v))).collect::<Vec<_>>();
            let so = to2(self, &self.faces[s].outer);
            let sh: Vec<_> = self.faces[s].holes.iter().map(|h| to2(self, h)).collect();
            let ss = crate::topology::interior_sample(&so, &sh);
            let Some(bb) = self.face_aabb(s) else { continue };
            let bb = bb.inflate(TOL);
            let mut group = vec![s];
            for g in self.cache.fgrid.query_box(bb.min, bb.max) {
                let Some(gf) = self.faces.get(g) else { continue };
                if group.contains(&g) || gf.normal.dot(n).abs() < 1.0 - 1e-6 {
                    continue;
                }
                if gf.loops().flatten().any(|v| plane.distance(self.pos(*v)).abs() > PTOL) {
                    continue;
                }
                let go = to2(self, &gf.outer);
                let gh: Vec<_> = gf.holes.iter().map(|h| to2(self, h)).collect();
                let gs = crate::topology::interior_sample(&go, &gh);
                if geom::point_in_region(ss, &go, &gh, TOL) == PointLoc::Inside
                    || geom::point_in_region(gs, &so, &sh, TOL) == PointLoc::Inside
                {
                    group.push(g);
                }
            }
            if group.len() < 2 {
                continue;
            }
            let mut polys = Vec::new();
            let mut seeds: Vec<VertexId> = Vec::new();
            let mut pairs: Vec<(VertexId, VertexId)> = Vec::new();
            for &g in &group {
                let face = self.faces[g].clone();
                for l in face.loops() {
                    seeds.extend(l.iter().copied());
                    pairs.extend((0..l.len()).map(|i| (l[i], l[(i + 1) % l.len()])));
                }
                let outer = self.loop_pos(&face.outer);
                let holes = face.holes.iter().map(|h| self.loop_pos(h)).collect();
                polys.push((outer, holes, face));
                self.f_remove(g);
            }
            self.refresh_plane(plane, &seeds, &[], &Classify::Overlay(&polys), Orient::auto());
            for (a, b) in pairs {
                if let Some(e) = self.edge_between(a, b)
                    && self.edge_faces(e).is_empty()
                {
                    self.e_remove(e);
                }
            }
            touched.extend(seeds);
        }
        touched
    }
}
