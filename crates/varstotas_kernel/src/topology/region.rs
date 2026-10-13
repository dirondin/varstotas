//! Planar region recomputation: the heart of sticky face behaviour.
//!
//! For one plane we gather every in-plane edge connected to the seeds (plus components lying
//! inside their bounding box), build the planar arrangement (half-edges sorted by angle),
//! extract the bounded regions with their holes, and then decide which regions are faces:
//! - a region inside a previous face of this plane stays a face (inheriting attributes and
//!   orientation; the previous id is reused when possible);
//! - otherwise it becomes a new face according to [`Classify`].
//!
//! Previous faces of the plane that no longer correspond to any region are removed.

use crate::geom::{self, PointLoc, Projector};
use crate::mesh::{Mesh, PTOL};
use crate::triangulate::triangulate;
use crate::{Aabb, EdgeId, Face, FaceId, Plane, TOL, VertexId};
use glam::{DVec2, DVec3};
use std::collections::{HashMap, HashSet};

/// How regions that were not part of an existing face are classified.
pub(crate) enum Classify<'a> {
    /// Only regions inside previous faces survive (splits / merges).
    None,
    /// Additionally, regions whose outer loop uses one of these edges become faces.
    NewEdges(&'a HashSet<EdgeId>),
    /// Additionally, regions inside this polygon (minus holes) become faces.
    Region { outer: Vec<DVec3>, holes: Vec<Vec<DVec3>> },
}

/// Orientation policy for brand-new faces.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Orient {
    pub pref: Option<DVec3>,
    /// Ground-plane faces face down even if `pref` says otherwise.
    pub ground_first: bool,
}

impl Orient {
    pub fn auto() -> Self {
        Self { pref: None, ground_first: false }
    }
    pub fn prefer(n: DVec3) -> Self {
        Self { pref: Some(n), ground_first: false }
    }
}

/// A face produced (or kept) by a region refresh.
#[derive(Clone, Debug)]
pub(crate) struct RegionFace {
    pub face: FaceId,
    /// A point strictly inside the face.
    pub sample: DVec3,
    pub area: f64,
    #[allow(dead_code)]
    pub created: bool,
}

struct Loop {
    verts: Vec<usize>,
    area: f64,
    comp: usize,
}

struct Region {
    outer: Loop,
    holes: Vec<Loop>,
}

fn find(uf: &mut [usize], x: usize) -> usize {
    let mut r = x;
    while uf[r] != r {
        r = uf[r];
    }
    let mut c = x;
    while uf[c] != r {
        let n = uf[c];
        uf[c] = r;
        c = n;
    }
    r
}

/// Split a face cycle (list of half-edge ids) at bridges (half-edge pairs both in the cycle).
fn split_bridges(cycle: &[usize]) -> Vec<Vec<usize>> {
    let set: HashSet<usize> = cycle.iter().copied().collect();
    let mut opened: HashSet<usize> = HashSet::new();
    let mut stack: Vec<Vec<usize>> = vec![Vec::new()];
    let mut out = Vec::new();
    for &h in cycle {
        if set.contains(&(h ^ 1)) {
            let e = h / 2;
            if opened.insert(e) || stack.len() == 1 {
                stack.push(Vec::new());
            } else {
                let l = stack.pop().unwrap();
                if !l.is_empty() {
                    out.push(l);
                }
            }
        } else {
            stack.last_mut().unwrap().push(h);
        }
    }
    while let Some(l) = stack.pop() {
        if !l.is_empty() {
            out.push(l);
        }
    }
    out
}

impl Mesh {
    pub(crate) fn refresh_plane(
        &mut self,
        plane: Plane,
        seeds: &[VertexId],
        extra_old: &[FaceId],
        cls: &Classify,
        orient: Orient,
    ) -> Vec<RegionFace> {
        let n = plane.normal.normalize_or(DVec3::Z);
        let plane = Plane { origin: plane.origin, normal: n };
        let pr = Projector::new(plane.origin, n);
        let on_plane = |m: &Mesh, v: VertexId| plane.distance(m.pos(v)).abs() <= PTOL;
        let face_in_plane = |m: &Mesh, f: FaceId| {
            let face = &m.faces[f];
            face.normal.dot(n).abs() > 1.0 - 1e-6 && face.outer.iter().all(|v| on_plane(m, *v))
        };

        // ---- 1. gather in-plane edges and old faces ---------------------------
        let mut vset: HashSet<VertexId> = HashSet::new();
        let mut eset: HashSet<EdgeId> = HashSet::new();
        let mut evec: Vec<EdgeId> = Vec::new();
        let mut old: Vec<FaceId> = Vec::new();
        let mut queue: Vec<VertexId> = seeds.iter().copied().filter(|v| self.vertices.contains_key(*v) && on_plane(self, *v)).collect();
        for &f in extra_old {
            if self.faces.contains_key(f) && !old.contains(&f) {
                old.push(f);
                queue.extend(self.faces[f].loops().flatten().copied().filter(|v| on_plane(self, *v)));
            }
        }
        let mut bbox: Option<Aabb> = None;
        loop {
            while let Some(v) = queue.pop() {
                if !vset.insert(v) {
                    continue;
                }
                let p = self.pos(v);
                match &mut bbox {
                    Some(b) => b.include(p),
                    None => bbox = Some(Aabb::from_point(p)),
                }
                for e in self.vertex_edges(v) {
                    if eset.contains(&e) {
                        continue;
                    }
                    let o = self.edges[e].other(v);
                    if !on_plane(self, o) {
                        continue;
                    }
                    eset.insert(e);
                    evec.push(e);
                    for f in self.edge_faces(e) {
                        if !old.contains(&f) && face_in_plane(self, f) {
                            old.push(f);
                            queue.extend(self.faces[f].loops().flatten().copied().filter(|v| on_plane(self, *v)));
                        }
                    }
                    if !vset.contains(&o) {
                        queue.push(o);
                    }
                }
            }
            let Some(bb) = bbox else { break };
            let bbi = bb.inflate(TOL);
            let mut added = false;
            // In-plane faces that contain gathered geometry (e.g. a loop drawn inside a face).
            for f in self.cache.fgrid.query_box(bbi.min, bbi.max) {
                if old.contains(&f) || !self.faces.contains_key(f) || !face_in_plane(self, f) {
                    continue;
                }
                let Some(fb) = self.face_aabb(f) else { continue };
                if !fb.inflate(TOL).intersects(&bbi) {
                    continue;
                }
                let touches = vset.iter().any(|v| {
                    let p = self.pos(*v);
                    fb.inflate(TOL).contains(p) && self.face_point_loc(f, p) != PointLoc::Outside
                });
                if touches {
                    old.push(f);
                    queue.extend(self.faces[f].loops().flatten().copied().filter(|v| on_plane(self, *v)));
                    added = true;
                }
            }
            for e in self.cache.egrid.query_box(bbi.min, bbi.max) {
                if eset.contains(&e) {
                    continue;
                }
                let Some(ed) = self.edges.get(e) else { continue };
                let [a, b] = ed.v;
                if vset.contains(&a) || vset.contains(&b) {
                    continue;
                }
                if on_plane(self, a) && on_plane(self, b) && bbi.contains(self.pos(a)) && bbi.contains(self.pos(b)) {
                    queue.push(a);
                    added = true;
                }
            }
            if !added {
                break;
            }
        }

        // ---- 2. prune dangling trees -----------------------------------------------
        let mut deg: HashMap<VertexId, usize> = HashMap::new();
        for e in &evec {
            for v in self.edges[*e].v {
                *deg.entry(v).or_default() += 1;
            }
        }
        let mut alive: HashSet<EdgeId> = eset.clone();
        let mut stack: Vec<VertexId> = deg.iter().filter(|(_, d)| **d <= 1).map(|(v, _)| *v).collect();
        while let Some(v) = stack.pop() {
            for e in self.vertex_edges(v) {
                if alive.remove(&e) {
                    let o = self.edges[e].other(v);
                    *deg.get_mut(&v).unwrap() -= 1;
                    let d = deg.get_mut(&o).unwrap();
                    *d -= 1;
                    if *d == 1 {
                        stack.push(o);
                    }
                }
            }
        }
        let edges: Vec<EdgeId> = evec.iter().copied().filter(|e| alive.contains(e)).collect();

        // ---- 3. arrangement --------------------------------------------------------
        let mut idx: HashMap<VertexId, usize> = HashMap::new();
        let mut vids: Vec<VertexId> = Vec::new();
        let mut p2: Vec<DVec2> = Vec::new();
        let mut he_from: Vec<usize> = Vec::with_capacity(edges.len() * 2);
        let mut he_to: Vec<usize> = Vec::with_capacity(edges.len() * 2);
        for e in &edges {
            let [a, b] = self.edges[*e].v;
            let mut ix = |v: VertexId| {
                *idx.entry(v).or_insert_with(|| {
                    vids.push(v);
                    p2.push(pr.to2(self.vertices[v].pos));
                    vids.len() - 1
                })
            };
            let (ia, ib) = (ix(a), ix(b));
            he_from.push(ia);
            he_to.push(ib);
            he_from.push(ib);
            he_to.push(ia);
        }
        let nv = vids.len();
        let nh = he_from.len();
        let mut out: Vec<Vec<usize>> = vec![Vec::new(); nv];
        for h in 0..nh {
            out[he_from[h]].push(h);
        }
        let angle = |h: usize| {
            let d = p2[he_to[h]] - p2[he_from[h]];
            d.y.atan2(d.x)
        };
        let mut slot = vec![0usize; nh];
        for l in out.iter_mut() {
            l.sort_by(|a, b| angle(*a).total_cmp(&angle(*b)));
            for (i, h) in l.iter().enumerate() {
                slot[*h] = i;
            }
        }
        let next = |h: usize| {
            let v = he_to[h];
            let tw = h ^ 1;
            let l = &out[v];
            let k = l.len();
            l[(slot[tw] + k - 1) % k]
        };
        let mut uf: Vec<usize> = (0..nv).collect();
        for h in (0..nh).step_by(2) {
            let (a, b) = (find(&mut uf, he_from[h]), find(&mut uf, he_to[h]));
            if a != b {
                uf[a] = b;
            }
        }
        let mut visited = vec![false; nh];
        let mut positives: Vec<Region> = Vec::new();
        let mut orphans: Vec<Loop> = Vec::new();
        let a_eps = 1e-9;
        for h0 in 0..nh {
            if visited[h0] {
                continue;
            }
            let mut cyc = Vec::new();
            let mut h = h0;
            let mut guard = 0;
            while !visited[h] {
                visited[h] = true;
                cyc.push(h);
                h = next(h);
                guard += 1;
                if guard > nh + 1 {
                    break;
                }
            }
            let comp = find(&mut uf, he_from[h0]);
            let mut pos_loops: Vec<Loop> = Vec::new();
            let mut neg_loops: Vec<Loop> = Vec::new();
            for sub in split_bridges(&cyc) {
                if sub.len() < 3 {
                    continue;
                }
                let verts: Vec<usize> = sub.iter().map(|h| he_from[*h]).collect();
                let pts: Vec<DVec2> = verts.iter().map(|i| p2[*i]).collect();
                let area = geom::signed_area(&pts);
                let lp = Loop { verts, area, comp };
                if area > a_eps {
                    pos_loops.push(lp);
                } else if area < -a_eps {
                    neg_loops.push(lp);
                }
            }
            if pos_loops.is_empty() {
                orphans.extend(neg_loops);
            } else {
                pos_loops.sort_by(|a, b| b.area.total_cmp(&a.area));
                let mut it = pos_loops.into_iter();
                let outer = it.next().unwrap();
                positives.push(Region { outer, holes: neg_loops });
                for extra in it {
                    positives.push(Region { outer: extra, holes: Vec::new() });
                }
            }
        }
        // Assign component boundaries to the smallest containing region of another component.
        for o in orphans {
            let tp = p2[o.verts[0]];
            let mut best: Option<(f64, usize)> = None;
            for (ri, r) in positives.iter().enumerate() {
                if r.outer.comp == o.comp {
                    continue;
                }
                if best.is_some_and(|(a, _)| r.outer.area >= a) {
                    continue;
                }
                let poly: Vec<DVec2> = r.outer.verts.iter().map(|i| p2[*i]).collect();
                if geom::point_in_polygon(tp, &poly, 0.0) == PointLoc::Inside {
                    best = Some((r.outer.area, ri));
                }
            }
            if let Some((_, ri)) = best {
                positives[ri].holes.push(o);
            }
        }

        // ---- 4. classify regions ----------------------------------------------------
        struct OldInfo {
            face: Face,
            outer2: Vec<DVec2>,
            holes2: Vec<Vec<DVec2>>,
            area: f64,
            lo: DVec2,
            hi: DVec2,
        }
        let old_info: HashMap<FaceId, OldInfo> = old
            .iter()
            .map(|f| {
                let face = self.faces[*f].clone();
                let outer2: Vec<DVec2> = face.outer.iter().map(|v| pr.to2(self.pos(*v))).collect();
                let holes2: Vec<Vec<DVec2>> = face.holes.iter().map(|h| h.iter().map(|v| pr.to2(self.pos(*v))).collect()).collect();
                let area = geom::signed_area(&outer2).abs();
                let lo = outer2.iter().fold(DVec2::splat(f64::MAX), |a, b| a.min(*b));
                let hi = outer2.iter().fold(DVec2::splat(f64::MIN), |a, b| a.max(*b));
                (*f, OldInfo { face, outer2, holes2, area, lo, hi })
            })
            .collect();
        let old_by_key: HashMap<Vec<VertexId>, FaceId> = old
            .iter()
            .map(|f| {
                let mut k = old_info[f].face.outer.clone();
                k.sort();
                (k, *f)
            })
            .collect();
        let cls_poly: Option<(Vec<DVec2>, Vec<Vec<DVec2>>)> = match cls {
            Classify::Region { outer, holes } => Some((
                outer.iter().map(|p| pr.to2(*p)).collect(),
                holes.iter().map(|h| h.iter().map(|p| pr.to2(*p)).collect()).collect(),
            )),
            _ => None,
        };

        #[derive(Clone, Copy, PartialEq)]
        enum Kind {
            Skip,
            Inherit(FaceId),
            New,
        }
        // (outer, holes, sample, net area, kind)
        type Reg = (Vec<VertexId>, Vec<Vec<VertexId>>, DVec2, f64, Kind);
        let mut regs: Vec<Reg> = Vec::new();
        for r in &positives {
            let outer_pts: Vec<DVec2> = r.outer.verts.iter().map(|i| p2[*i]).collect();
            let holes_pts: Vec<Vec<DVec2>> = r.holes.iter().map(|h| h.verts.iter().map(|i| p2[*i]).collect()).collect();
            let sample = interior_sample(&outer_pts, &holes_pts);
            let net = r.outer.area - r.holes.iter().map(|h| h.area.abs()).sum::<f64>();
            let mut inherit: Option<(f64, FaceId)> = None;
            let rkey = {
                let mut k: Vec<VertexId> = r.outer.verts.iter().map(|i| vids[*i]).collect();
                k.sort();
                k
            };
            if let Some(f) = old_by_key.get(&rkey) {
                inherit = Some((0.0, *f));
            } else {
                for f in &old {
                    let oi = &old_info[f];
                    if sample.x < oi.lo.x - TOL || sample.y < oi.lo.y - TOL || sample.x > oi.hi.x + TOL || sample.y > oi.hi.y + TOL {
                        continue;
                    }
                    if geom::point_in_region(sample, &oi.outer2, &oi.holes2, TOL) != PointLoc::Outside
                        && inherit.is_none_or(|(a, _)| oi.area < a)
                    {
                        inherit = Some((oi.area, *f));
                    }
                }
            }
            let kind = if let Some((_, f)) = inherit {
                Kind::Inherit(f)
            } else {
                match cls {
                    Classify::None => Kind::Skip,
                    Classify::NewEdges(set) => {
                        let k = r.outer.verts.len();
                        let uses_new = (0..k).any(|i| {
                            let (a, b) = (vids[r.outer.verts[i]], vids[r.outer.verts[(i + 1) % k]]);
                            self.edge_between(a, b).is_some_and(|e| set.contains(&e))
                        });
                        if uses_new { Kind::New } else { Kind::Skip }
                    }
                    Classify::Region { .. } => {
                        let (o, h) = cls_poly.as_ref().unwrap();
                        if geom::point_in_region(sample, o, h, TOL) == PointLoc::Inside { Kind::New } else { Kind::Skip }
                    }
                }
            };
            if kind == Kind::Skip {
                continue;
            }
            let outer: Vec<VertexId> = r.outer.verts.iter().map(|i| vids[*i]).collect();
            let holes: Vec<Vec<VertexId>> = r.holes.iter().map(|h| h.verts.iter().map(|i| vids[*i]).collect()).collect();
            regs.push((outer, holes, sample, net, kind));
        }

        // ---- 5. apply -----------------------------------------------------------------
        let key = |l: &[VertexId]| {
            let mut k = l.to_vec();
            k.sort();
            k
        };
        let old_keys: HashMap<Vec<VertexId>, FaceId> = old.iter().map(|f| (key(&old_info[f].face.outer), *f)).collect();
        let mut reuse: Vec<Option<FaceId>> = vec![None; regs.len()];
        let mut used: HashSet<FaceId> = HashSet::new();
        for (i, r) in regs.iter().enumerate() {
            if let Some(f) = old_keys.get(&key(&r.0))
                && !used.contains(f) {
                    used.insert(*f);
                    reuse[i] = Some(*f);
                }
        }
        let mut order: Vec<usize> = (0..regs.len()).collect();
        order.sort_by(|a, b| regs[*b].3.total_cmp(&regs[*a].3));
        for &i in &order {
            if reuse[i].is_some() {
                continue;
            }
            if let Kind::Inherit(f) = regs[i].4
                && used.insert(f) {
                    reuse[i] = Some(f);
                }
        }
        for f in &old {
            if !used.contains(f) {
                self.f_remove(*f);
            }
        }
        let mut result = Vec::new();
        // Reused faces first (so new faces can orient against them).
        for (i, r) in regs.iter().enumerate() {
            let Some(f) = reuse[i] else { continue };
            let (outer, holes) = (r.0.clone(), r.1.clone());
            let face = &self.faces[f];
            let same = key(&face.outer) == key(&outer) && {
                let mut a: Vec<_> = face.holes.iter().map(|h| key(h)).collect();
                let mut b: Vec<_> = holes.iter().map(|h| key(h)).collect();
                a.sort();
                b.sort();
                a == b
            };
            if !same {
                let flip = old_info[&f].face.normal.dot(n) < 0.0;
                let (o, h) = orient_loops(outer, holes, flip);
                self.f_set_loops(f, o, h);
            }
            result.push(RegionFace { face: f, sample: pr.to3(r.2), area: r.3, created: false });
        }
        for (i, r) in regs.iter().enumerate() {
            if reuse[i].is_some() {
                continue;
            }
            let (outer, holes) = (r.0.clone(), r.1.clone());
            let (tmpl, flip) = match r.4 {
                Kind::Inherit(f) => {
                    let t = old_info[&f].face.clone();
                    let flip = t.normal.dot(n) < 0.0;
                    (t, flip)
                }
                _ => (Face::template(), self.choose_flip(&plane, &outer, orient)),
            };
            let (o, h) = orient_loops(outer, holes, flip);
            let f = self.f_new(o, h, &tmpl);
            result.push(RegionFace { face: f, sample: pr.to3(r.2), area: r.3, created: true });
        }
        result
    }

    /// Decide whether a new face with outer loop `outer` (CCW about `plane.normal`) must be
    /// flipped (normal = -plane.normal).
    fn choose_flip(&self, plane: &Plane, outer: &[VertexId], orient: Orient) -> bool {
        let n = plane.normal;
        let is_ground = n.z.abs() > 1.0 - 1e-9 && plane.origin.z.abs() <= PTOL;
        if orient.ground_first && is_ground {
            return n.z > 0.0;
        }
        if let Some(p) = orient.pref {
            return p.dot(n) < 0.0;
        }
        let k = outer.len();
        let pairs: Vec<(VertexId, VertexId)> = (0..k).map(|i| (outer[i], outer[(i + 1) % k])).collect();
        // Coplanar neighbour face: same orientation.
        for &(a, b) in &pairs {
            let Some(e) = self.edge_between(a, b) else { continue };
            for g in self.edge_faces(e) {
                let gn = self.faces[g].normal;
                if gn.dot(n).abs() > 1.0 - 1e-6 {
                    return gn.dot(n) < 0.0;
                }
            }
        }
        if is_ground {
            return n.z > 0.0;
        }
        // Consistent orientation with non-coplanar neighbours (shared edge traversed oppositely).
        for &(a, b) in &pairs {
            let Some(e) = self.edge_between(a, b) else { continue };
            for g in self.edge_faces(e) {
                if let Some(same_dir) = self.face_traverses(g, a, b) {
                    return same_dir;
                }
            }
        }
        // Default: horizontal faces up, vertical faces towards -Y (front) or +X (right).
        let a = n.abs();
        let want = if a.z >= a.x && a.z >= a.y {
            DVec3::Z
        } else if a.y >= a.x {
            DVec3::NEG_Y
        } else {
            DVec3::X
        };
        want.dot(n) < 0.0
    }
}

/// Loops come CCW (outer) / CW (holes) about the plane normal; flip both if needed.
fn orient_loops(mut outer: Vec<VertexId>, mut holes: Vec<Vec<VertexId>>, flip: bool) -> (Vec<VertexId>, Vec<Vec<VertexId>>) {
    if flip {
        outer.reverse();
        for h in &mut holes {
            h.reverse();
        }
    }
    (outer, holes)
}

/// A point strictly inside a polygon with holes (centroid of the largest triangle).
pub(crate) fn interior_sample(outer: &[DVec2], holes: &[Vec<DVec2>]) -> DVec2 {
    let hr: Vec<&[DVec2]> = holes.iter().map(|h| h.as_slice()).collect();
    let tris = triangulate(outer, &hr);
    let all: Vec<DVec2> = outer.iter().chain(holes.iter().flatten()).copied().collect();
    let mut best: Option<(f64, DVec2)> = None;
    for t in tris {
        let (a, b, c) = (all[t[0]], all[t[1]], all[t[2]]);
        let area = (b - a).perp_dot(c - a).abs();
        if best.is_none_or(|(ba, _)| area > ba) {
            best = Some((area, (a + b + c) / 3.0));
        }
    }
    best.map(|(_, p)| p).unwrap_or_else(|| outer.iter().copied().sum::<DVec2>() / outer.len().max(1) as f64)
}
