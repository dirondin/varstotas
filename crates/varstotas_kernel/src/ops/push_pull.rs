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
use glam::{DVec2, DVec3};
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
                // A stretched neighbour must stay a valid face: sliding must not drag one of
                // its loops across another (e.g. a hole pulled out through the outer loop) or
                // across itself. Such vertices are extruded instead (new side faces, whose
                // overlaps with the neighbour are then resolved), and the analysis repeats.
                // Collapse check for stretched vertices (an edge along the motion would vanish
                // or flip): refused outright.
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
                let bad = self.stretch_breaks_faces(f, &slid, off);
                if bad.is_empty() {
                    break;
                }
                for v in bad {
                    slid.insert(v, false);
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
        // (segment, is a new edge from an original vertex to its moved copy)
        let mut merge_segs: Vec<(DVec3, DVec3, bool)> = Vec::new();
        let mut created: Vec<FaceId> = vec![cap];
        for (e, a, b) in &bedges {
            if slid[a] && slid[b] {
                continue;
            }
            let (a2, b2) = (map[a], map[b]);
            let side = if cap_flip { vec![*b, *a, a2, b2] } else { vec![*a, *b, b2, a2] };
            created.push(self.f_new(side, Vec::new(), &Face::template()));
            if !lone {
                let (p, q) = self.edge_segment(*e);
                merge_segs.push((p, q, false));
            }
            for (x, y) in [(*a, a2), (*b, b2)] {
                if x != y {
                    merge_segs.push((self.pos(x), self.pos(y), true));
                }
            }
        }
        self.soften_sides(&loops, &map);
        let new_verts: Vec<VertexId> = loops.iter().flatten().filter(|v| !slid[*v]).map(|v| map[v]).collect();
        // Slid vertices in loop order (not hash order) so the result is deterministic.
        let mut seen = HashSet::new();
        let slid_verts: Vec<VertexId> = loops.iter().flatten().copied().filter(|v| slid[v] && seen.insert(*v)).collect();
        let moved: Vec<VertexId> = new_verts.iter().chain(&slid_verts).copied().collect();
        let mut stretched: Vec<FaceId> = Vec::new();
        for v in &slid_verts {
            stretched.extend(self.faces_of_vertex(*v));
        }
        let split_at = self.weld(&moved);
        // Stretched neighbours grew and may now overlap coplanar faces too.
        for g in stretched {
            if !created.contains(&g) && self.faces.contains_key(g) {
                created.push(g);
            }
        }
        let mut touched = self.resolve_overlaps(&created, false);
        // Faces whose boundary got a new vertex may now have an existing edge as a chord.
        let mut check = created.clone();
        for v in split_at {
            if self.vertices.contains_key(v) {
                for g in self.faces_of_vertex(v) {
                    if !check.contains(&g) {
                        check.push(g);
                    }
                }
            }
        }
        self.split_faces_by_inner_edges(&check);
        let mut cands: Vec<EdgeId> = Vec::new();
        let mut new_pieces: Vec<EdgeId> = Vec::new();
        for (a, b, fresh) in &merge_segs {
            for e in self.cache.egrid.query_segment(*a, *b) {
                if cands.contains(&e) || !self.edges.contains_key(e) {
                    continue;
                }
                let (p, q) = self.edge_segment(e);
                if geom::point_seg_dist(p, *a, *b).0 <= TOL && geom::point_seg_dist(q, *a, *b).0 <= TOL {
                    cands.push(e);
                    if *fresh {
                        new_pieces.push(e);
                    }
                }
            }
        }
        if !keep_original {
            self.merge_coplanar_at(&cands);
            // Pieces of a new side edge (split by crossings) left without faces once the
            // sides merged are not geometry anybody drew.
            for e in new_pieces {
                if self.edges.contains_key(e) && self.edge_faces(e).is_empty() {
                    touched.extend(self.edges[e].v);
                    self.e_remove(e);
                }
            }
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
        let mut sides = Vec::new();
        for (e, a, b) in bedges {
            let (a2, b2) = (map[a], map[b]);
            sides.push(self.f_new(vec![*a, *b, b2, a2], Vec::new(), &Face::template()));
            merge_candidates.push(*e);
        }
        let side_edges: Vec<EdgeId> = bedges
            .iter()
            .flat_map(|(_, a, b)| [(*a, map[a]), (map[a], map[b])])
            .filter_map(|(x, y)| self.edge_between(x, y))
            .collect();
        let split_at = self.split_crossings(side_edges);
        self.resolve_overlaps(&sides, false);
        let mut check = sides.clone();
        for v in split_at {
            if self.vertices.contains_key(v) {
                check.extend(self.faces_of_vertex(v));
            }
        }
        self.split_faces_by_inner_edges(&check);
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

    /// Vertices of `slid` (true entries) in faces other than `f` that would become invalid if
    /// those vertices moved by `off`: two non-adjacent edges of the face's loops would touch.
    fn stretch_breaks_faces(&self, f: FaceId, slid: &HashMap<VertexId, bool>, off: DVec3) -> Vec<VertexId> {
        let moved = |v: VertexId| slid.get(&v).copied().unwrap_or(false);
        let mut faces: Vec<FaceId> = Vec::new();
        for (v, s) in slid {
            if *s {
                for g in self.faces_of_vertex(*v) {
                    if g != f && !faces.contains(&g) {
                        faces.push(g);
                    }
                }
            }
        }
        let mut bad: Vec<VertexId> = Vec::new();
        for g in faces {
            let face = &self.faces[g];
            let pr = Projector::new(self.pos(face.outer[0]), face.normal);
            let p2 = |v: VertexId| pr.to2(self.pos(v) + if moved(v) { off } else { DVec3::ZERO });
            let segs: Vec<(VertexId, VertexId, DVec2, DVec2)> = face
                .loops()
                .flat_map(|l| (0..l.len()).map(move |i| (l[i], l[(i + 1) % l.len()])))
                .map(|(a, b)| (a, b, p2(a), p2(b)))
                .collect();
            let broken = segs.iter().enumerate().any(|(i, s1)| {
                (moved(s1.0) || moved(s1.1))
                    && segs.iter().enumerate().any(|(j, s2)| {
                        j != i
                            && s1.0 != s2.0
                            && s1.0 != s2.1
                            && s1.1 != s2.0
                            && s1.1 != s2.1
                            && geom::segs_intersect_2d(s1.2, s1.3, s2.2, s2.3, TOL)
                    })
            });
            if broken {
                bad.extend(face.loops().flatten().copied().filter(|v| moved(*v)));
            }
        }
        bad
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
    /// split their edges at existing vertices lying on them and at crossings with other edges.
    /// Returns the vertices at which edges were split.
    pub(crate) fn weld(&mut self, verts: &[VertexId]) -> Vec<VertexId> {
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
        let work: Vec<EdgeId> =
            alive.iter().filter(|v| self.vertices.contains_key(**v)).flat_map(|v| self.vertex_edges(*v)).collect();
        self.split_crossings(work)
    }

    /// Split edges of `work` and existing edges crossing them in their interiors at a shared
    /// vertex (plane arrangements need an edge graph without crossings).
    pub(crate) fn split_crossings(&mut self, mut work: Vec<EdgeId>) -> Vec<VertexId> {
        let mut at: Vec<VertexId> = Vec::new();
        let mut guard = 0;
        while let Some(e) = work.pop() {
            guard += 1;
            if guard > 100_000 {
                break;
            }
            let Some(ed) = self.edges.get(e).cloned() else { continue };
            let (a, b) = self.edge_segment(e);
            let interior = |p: DVec3, x: DVec3, y: DVec3| {
                geom::point_seg_dist(p, x, y).0 <= TOL && (p - x).length() > TOL && (p - y).length() > TOL
            };
            // Existing vertices inside the edge (T-junctions, collinear overlaps).
            let on_e = self.cache.vgrid.query_segment(a, b).into_iter().find(|w| {
                !ed.has(*w) && self.vertices.get(*w).is_some_and(|x| interior(x.pos, a, b))
            });
            if let Some(w) = on_e {
                at.push(w);
                work.extend(self.split_edge_at(e, w));
                continue;
            }
            // Endpoints of the edge inside other edges.
            let d = DVec3::splat(TOL);
            let mut t_hit = None;
            for (v, p) in [(ed.v[0], a), (ed.v[1], b)] {
                t_hit = self.cache.egrid.query_box(p - d, p + d).into_iter().find(|e2| {
                    self.edges.get(*e2).is_some_and(|x| !x.has(v)) && {
                        let (c, dd) = self.edge_segment(*e2);
                        interior(p, c, dd)
                    }
                }).map(|e2| (e2, v));
                if t_hit.is_some() {
                    break;
                }
            }
            if let Some((e2, v)) = t_hit {
                at.push(v);
                work.extend(self.split_edge_at(e2, v));
                work.push(e);
                continue;
            }
            let mut hit: Option<(EdgeId, DVec3)> = None;
            for e2 in self.cache.egrid.query_segment(a, b) {
                let Some(ed2) = self.edges.get(e2) else { continue };
                if e2 == e || ed2.has(ed.v[0]) || ed2.has(ed.v[1]) {
                    continue;
                }
                let (c, d) = self.edge_segment(e2);
                let Some((_, _, p1, p2)) = geom::seg_seg_closest(a, b, c, d) else { continue };
                if (p1 - p2).length() > TOL {
                    continue;
                }
                let far = |p: DVec3, x: DVec3, y: DVec3| (p - x).length() > TOL && (p - y).length() > TOL;
                if far(p1, a, b) && far(p2, c, d) {
                    hit = Some((e2, p1));
                    break;
                }
            }
            if let Some((e2, p)) = hit {
                let v = match self.v_find(p, TOL) {
                    Some(v) => v,
                    None => self.v_new(p),
                };
                at.push(v);
                if !self.edges[e2].has(v) {
                    work.extend(self.split_edge_at(e2, v));
                }
                if !self.edges[e].has(v) {
                    work.extend(self.split_edge_at(e, v));
                }
            }
        }
        at
    }

    /// Split faces of `faces` by existing in-plane edges lying inside them (an edge
    /// drawn across a face splits it; new faces must honour edges already there).
    /// Returns how many faces were rebuilt.
    pub(crate) fn split_faces_by_inner_edges(&mut self, faces: &[FaceId]) -> usize {
        let mut rebuilt = 0;
        for &f in faces {
            if !self.faces.contains_key(f) {
                continue;
            }
            let plane = self.face_plane(f);
            let Some(bb) = self.face_aabb(f) else { continue };
            let bb = bb.inflate(TOL);
            let fe: HashSet<VertexId> = self.faces[f].loops().flatten().copied().collect();
            let inner = self.cache.egrid.query_box(bb.min, bb.max).into_iter().any(|e| {
                let Some(ed) = self.edges.get(e) else { return false };
                let (a, b) = self.edge_segment(e);
                if plane.distance(a).abs() > PTOL || plane.distance(b).abs() > PTOL {
                    return false;
                }
                if fe.contains(&ed.v[0]) && fe.contains(&ed.v[1]) && self.edge_faces(e).contains(&f) {
                    return false;
                }
                [(a + b) * 0.5, a, b].iter().any(|p| self.face_point_loc(f, *p) == PointLoc::Inside)
            });
            // A loop running along an edge twice (a stretched face collapsed onto itself).
            let mut seen = HashSet::new();
            let folded = self.faces[f].loops().any(|l| {
                (0..l.len()).any(|i| {
                    let (a, b) = (l[i], l[(i + 1) % l.len()]);
                    !seen.insert(if a < b { (a, b) } else { (b, a) })
                })
            });
            if inner || folded {
                let mut seen = HashSet::new();
                let seeds: Vec<VertexId> = self.faces[f].loops().flatten().copied().filter(|v| seen.insert(*v)).collect();
                self.refresh_plane(plane, &seeds, &[f], &Classify::None, Orient::auto());
                rebuilt += 1;
            }
        }
        rebuilt
    }

    /// `s` and the coplanar faces whose area overlaps it.
    pub(crate) fn overlap_group(&self, s: FaceId) -> Vec<FaceId> {
        let plane = self.face_plane(s);
        let n = plane.normal;
        let pr = Projector::new(plane.origin, n);
        let to2 = |l: &[VertexId]| l.iter().map(|v| pr.to2(self.pos(*v))).collect::<Vec<_>>();
        let so = to2(&self.faces[s].outer);
        let sh: Vec<_> = self.faces[s].holes.iter().map(|h| to2(h)).collect();
        let ss = crate::topology::interior_sample(&so, &sh);
        let mut group = vec![s];
        let Some(bb) = self.face_aabb(s) else { return group };
        let bb = bb.inflate(TOL);
        for g in self.cache.fgrid.query_box(bb.min, bb.max) {
            let Some(gf) = self.faces.get(g) else { continue };
            if group.contains(&g) || gf.normal.dot(n).abs() < 1.0 - 1e-6 {
                continue;
            }
            if gf.loops().flatten().any(|v| plane.distance(self.pos(*v)).abs() > PTOL) {
                continue;
            }
            let go = to2(&gf.outer);
            let gh: Vec<_> = gf.holes.iter().map(|h| to2(h)).collect();
            let gs = crate::topology::interior_sample(&go, &gh);
            if regions_overlap((&so, &sh, ss), (&go, &gh, gs)) {
                group.push(g);
            }
        }
        group
    }

    /// Overlay each face of `faces` with existing coplanar faces it overlaps: coverage with
    /// opposite orientations cancels (with `union`, nothing cancels: the faces are merged into
    /// one partition, the smallest covering face winning on each overlap). Returns the vertices
    /// involved.
    pub(crate) fn resolve_overlaps(&mut self, faces: &[FaceId], union: bool) -> Vec<VertexId> {
        let mut touched = Vec::new();
        let mut segs: Vec<(DVec3, DVec3)> = Vec::new();
        // A work queue: overlaying one group rebuilds the plane around it, which may replace
        // other faces still to be checked (new ids), so the rebuilt faces are checked too.
        let mut queue: std::collections::VecDeque<FaceId> = faces.iter().copied().collect();
        let mut budget = 4 * faces.len() + 64;
        while let Some(s) = queue.pop_front() {
            if !self.faces.contains_key(s) {
                continue;
            }
            let mut group = self.overlap_group(s);
            if group.len() < 2 {
                continue;
            }
            // Overlaying rebuilds the plane around the group, so every pending overlap in the
            // same plane goes into the same overlay (else a neighbouring face still to be
            // checked would be rebuilt without its overlap being resolved).
            let plane = self.face_plane(s);
            let pending: Vec<FaceId> = queue.iter().copied().filter(|q| !group.contains(q)).collect();
            for q in pending {
                let Some(qf) = self.faces.get(q) else { continue };
                if qf.normal.dot(plane.normal).abs() < 1.0 - 1e-6 || qf.outer.iter().any(|v| plane.distance(self.pos(*v)).abs() > PTOL) {
                    continue;
                }
                let more = self.overlap_group(q);
                if more.len() >= 2 {
                    for g in more {
                        if !group.contains(&g) {
                            group.push(g);
                        }
                    }
                }
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
            let rebuilt = self.refresh_plane(plane, &seeds, &[], &Classify::Overlay(&polys, union), Orient::auto());
            if budget > 0 {
                budget -= 1;
                queue.extend(rebuilt.iter().map(|r| r.face));
            }
            segs.extend(pairs.iter().map(|(a, b)| (self.pos(*a), self.pos(*b))));
            touched.extend(seeds);
        }
        // Edges along the boundaries of the overlaid faces left without any face (cancelled
        // coverage). Checked by geometry once all groups are done (later groups may split
        // them), so the result does not depend on the order of the groups.
        for (a, b) in segs {
            for e in self.cache.egrid.query_segment(a, b) {
                if !self.edges.contains_key(e) || !self.edge_faces(e).is_empty() {
                    continue;
                }
                let (p, q) = self.edge_segment(e);
                if geom::point_seg_dist(p, a, b).0 <= TOL && geom::point_seg_dist(q, a, b).0 <= TOL {
                    self.e_remove(e);
                }
            }
        }
        touched
    }
}

type Region2<'a> = (&'a [DVec2], &'a [Vec<DVec2>], DVec2);

/// Do two polygons with holes share interior area? Besides interior samples, partial overlaps
/// show up as a vertex or an edge midpoint of one lying strictly inside the other (edges are
/// split at crossings before this is used).
fn regions_overlap(s: Region2, g: Region2) -> bool {
    let inside = |p: DVec2, r: &Region2| geom::point_in_region(p, r.0, r.1, TOL) == PointLoc::Inside;
    let probes = |r: &Region2, other: &Region2| {
        std::iter::once(r.0).chain(r.1.iter().map(|h| h.as_slice())).any(|l| {
            let k = l.len();
            (0..k).any(|i| inside(l[i], other) || inside((l[i] + l[(i + 1) % k]) * 0.5, other))
        })
    };
    inside(s.2, &g) || inside(g.2, &s) || probes(&s, &g) || probes(&g, &s)
}
