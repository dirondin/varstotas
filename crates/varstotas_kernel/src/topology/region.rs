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

/// A face polygon for [`Classify::Overlay`]: outer, holes and the face (attributes, normal).
pub(crate) type OverlayPoly = (Vec<DVec3>, Vec<Vec<DVec3>>, Face);

/// How regions that were not part of an existing face are classified.
pub(crate) enum Classify<'a> {
    /// Only regions inside previous faces survive (splits / merges).
    None,
    /// Additionally, regions whose outer loop uses one of these edges become faces.
    NewEdges(&'a HashSet<EdgeId>),
    /// Additionally, regions inside this polygon (minus holes) become faces.
    Region { outer: Vec<DVec3>, holes: Vec<Vec<DVec3>> },
    /// Overlay of coplanar faces (outer, holes, attributes incl. normal): regions get the net
    /// orientation of the faces covering them; opposite faces cancel out (no face).
    /// With `union = true` nothing cancels: every covered region becomes a face with the
    /// attributes and orientation of the smallest polygon covering it.
    Overlay(&'a [OverlayPoly], bool),
    /// Regions strictly containing one of these points become faces with that face's
    /// attributes and orientation (merging faces whose shared edges were removed: the regions
    /// of the remaining boundary are unions of the old faces, each containing their samples).
    Covered(&'a [(DVec3, Face)]),
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

/// Validation states of a gathered region (see `refresh_plane`).
const EXACT: u8 = 0;
const PARTIAL: u8 = 1;
const COVERED: u8 = 2;

#[cfg(test)]
thread_local! {
    /// Test-only reference mode: gather the whole connected in-plane component (the original,
    /// non-localized algorithm) so localized results can be compared against it.
    static FULL_GATHER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn full_gather() -> bool {
    FULL_GATHER.with(|c| c.get())
}

#[cfg(not(test))]
#[inline(always)]
fn full_gather() -> bool {
    false
}

/// Gathered subset of one plane for [`Mesh::refresh_plane`].
#[derive(Default)]
struct Gather {
    /// Fully visited vertices (all their in-plane edges are gathered).
    visited: HashSet<VertexId>,
    /// All vertices of gathered edges/faces, in insertion order (+ set).
    vlist: Vec<VertexId>,
    vset: HashSet<VertexId>,
    eset: HashSet<EdgeId>,
    evec: Vec<EdgeId>,
    old: Vec<FaceId>,
    old_set: HashSet<FaceId>,
    bbox: Option<Aabb>,
}

impl Gather {
    fn add_vertex(&mut self, m: &Mesh, v: VertexId) {
        if self.vset.insert(v) {
            self.vlist.push(v);
            let p = m.pos(v);
            match &mut self.bbox {
                Some(b) => b.include(p),
                None => self.bbox = Some(Aabb::from_point(p)),
            }
        }
    }
    fn add_edge(&mut self, m: &Mesh, e: EdgeId) {
        if self.eset.insert(e) {
            self.evec.push(e);
            let [a, b] = m.edges[e].v;
            self.add_vertex(m, a);
            self.add_vertex(m, b);
        }
    }
    /// Add an in-plane face and all edges of its loops.
    fn add_face(&mut self, m: &Mesh, f: FaceId) {
        if !self.old_set.insert(f) {
            return;
        }
        self.old.push(f);
        for l in m.faces[f].loops() {
            let k = l.len();
            for i in 0..k {
                if let Some(e) = m.edge_between(l[i], l[(i + 1) % k]) {
                    self.add_edge(m, e);
                }
            }
        }
    }
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
        // Localized gathering. "Full" visits of a vertex add every in-plane edge at it (and the
        // in-plane faces of those edges, whose loops are added as a whole). The flood only
        // continues across edges that are not shared by two in-plane faces: an edge with faces
        // on both sides cannot bound a non-face region, so the regions beyond it are unchanged
        // faces. Faces that strictly contain a gathered vertex are gathered, and every bounded
        // region of the gathered arrangement is validated: in-plane edges lying inside it
        // (islands, chords) are fully visited and the arrangement is rebuilt, so each bounded
        // region equals a region of the full plane arrangement. A region whose boundary was
        // gathered only as loops of neighbouring faces may coincide with an ungathered face;
        // classification inherits such a face instead of duplicating it.
        let mut g = Gather::default();
        let mut queue: Vec<VertexId> = seeds.iter().copied().filter(|v| self.vertices.contains_key(*v) && on_plane(self, *v)).collect();
        for &f in extra_old {
            if self.faces.contains_key(f) {
                g.add_face(self, f);
                queue.extend(self.faces[f].loops().flatten().copied().filter(|v| on_plane(self, *v)));
            }
        }
        let cls_poly: Option<(Vec<DVec2>, Vec<Vec<DVec2>>)> = match cls {
            Classify::Region { outer, holes } => Some((
                outer.iter().map(|p| pr.to2(*p)).collect(),
                holes.iter().map(|h| h.iter().map(|p| pr.to2(*p)).collect()).collect(),
            )),
            _ => None,
        };
        // Is a 2D point inside or on the classification polygon (Region only)?
        let in_cls_poly = |p: DVec2| {
            let (o, h) = cls_poly.as_ref().unwrap();
            geom::point_in_region(p, o, h, TOL) != PointLoc::Outside
        };
        // Non-face areas only matter where new faces may appear: everywhere for NewEdges and
        // Overlay, inside the polygon for Region, nowhere for None.
        let full = full_gather();
        let flood_across = |m: &Mesh, e: EdgeId| full || match cls {
            Classify::None => false,
            Classify::Region { .. } => {
                let (a, b) = m.edge_segment(e);
                in_cls_poly(pr.to2((a + b) * 0.5))
            }
            _ => true,
        };
        let mut checked = 0usize;
        let (vids, p2, positives, state) = loop {
            while let Some(v) = queue.pop() {
                if !g.visited.insert(v) {
                    continue;
                }
                g.add_vertex(self, v);
                for e in self.vertex_edges(v) {
                    let o = self.edges[e].other(v);
                    if !on_plane(self, o) {
                        continue;
                    }
                    g.add_edge(self, e);
                    let mut cnt = 0;
                    for f in self.edge_faces(e) {
                        if face_in_plane(self, f) {
                            cnt += 1;
                            g.add_face(self, f);
                        }
                    }
                    if (cnt < 2 || full) && !g.visited.contains(&o) && flood_across(self, e) {
                        queue.push(o);
                    }
                }
            }
            // In-plane faces that contain gathered geometry (e.g. a loop drawn inside a face).
            while checked < g.vlist.len() {
                let v = g.vlist[checked];
                checked += 1;
                let p = self.pos(v);
                let d = DVec3::splat(TOL);
                for f in self.cache.fgrid.query_box(p - d, p + d) {
                    if g.old_set.contains(&f) || !self.faces.contains_key(f) || !face_in_plane(self, f) {
                        continue;
                    }
                    let Some(fb) = self.face_aabb(f) else { continue };
                    if !fb.inflate(TOL).contains(p) {
                        continue;
                    }
                    if self.face_point_loc(f, p) == PointLoc::Inside {
                        g.add_face(self, f);
                    }
                }
            }
            // Validate the bounded regions of the gathered arrangement against ungathered
            // in-plane edges lying inside them:
            // - an edge with faces on both sides means the region is a union of untouched
            //   existing faces (COVERED): it is left alone;
            // - otherwise (islands, free or dangling edges) the edge is fully visited and the
            //   arrangement rebuilt, if the region may become a new face; if it may not, the
            //   region is only allowed to inherit a gathered face (PARTIAL: dangling edges inside
            //   a face are harmless, but an ungathered face inside must not be reshaped).
            let (vids_, p2_, positives_) = self.arrangement(&pr, &g.evec, &g.eset);
            let mut state_ = vec![EXACT; positives_.len()];
            for (ri, r) in positives_.iter().enumerate() {
                let outer: Vec<DVec2> = r.outer.verts.iter().map(|i| p2_[*i]).collect();
                let holes: Vec<Vec<DVec2>> = r.holes.iter().map(|h| h.verts.iter().map(|i| p2_[*i]).collect()).collect();
                // Could this region become a new face? Only then must it be exact; otherwise a
                // region containing ungathered edges is simply left alone.
                let may_be_new = full || match cls {
                    Classify::None => false,
                    Classify::NewEdges(set) => std::iter::once(&r.outer).chain(r.holes.iter()).any(|l| {
                        let k = l.verts.len();
                        (0..k).any(|i| {
                            self.edge_between(vids_[l.verts[i]], vids_[l.verts[(i + 1) % k]]).is_some_and(|e| set.contains(&e))
                        })
                    }),
                    Classify::Region { .. } => {
                        let k = outer.len();
                        (0..k).all(|i| in_cls_poly((outer[i] + outer[(i + 1) % k]) * 0.5))
                    }
                    Classify::Overlay(..) | Classify::Covered(_) => true,
                };
                let Some(bb) = Aabb::from_points(r.outer.verts.iter().map(|i| self.pos(vids_[*i]))) else { continue };
                let bb = bb.inflate(TOL);
                let (mut has_two, mut pushed) = (false, false);
                let mut in_old_face: Option<bool> = None;
                for e in self.cache.egrid.query_box(bb.min, bb.max) {
                    if g.eset.contains(&e) {
                        continue;
                    }
                    let Some(ed) = self.edges.get(e) else { continue };
                    let [a, b] = ed.v;
                    let (pa, pb) = (self.pos(a), self.pos(b));
                    if !bb.contains(pa) || !bb.contains(pb) {
                        continue;
                    }
                    if !on_plane(self, a) || !on_plane(self, b) {
                        continue;
                    }
                    let (pa, pb) = (pr.to2(pa), pr.to2(pb));
                    let inside = [(pa + pb) * 0.5, pa, pb]
                        .iter()
                        .any(|p| geom::point_in_region(*p, &outer, &holes, TOL) == PointLoc::Inside);
                    if !inside {
                        continue;
                    }
                    let two_sided = self.edge_faces(e).into_iter().filter(|f| face_in_plane(self, *f)).count() >= 2;
                    match (two_sided, may_be_new) {
                        (true, false) => {
                            state_[ri] = COVERED;
                            break;
                        }
                        (false, false) => {
                            // A dangling edge inside a face is harmless, but new edges may have
                            // connected it: regions inside a gathered face must be exact.
                            let in_old = *in_old_face.get_or_insert_with(|| {
                                let sample = pr.to3(interior_sample(&outer, &holes));
                                g.old.iter().any(|f| {
                                    self.face_aabb(*f).is_some_and(|b| b.inflate(TOL).contains(sample))
                                        && self.face_point_loc(*f, sample) == PointLoc::Inside
                                })
                            });
                            if in_old {
                                pushed = true;
                                queue.push(a);
                                queue.push(b);
                            } else {
                                state_[ri] = PARTIAL;
                            }
                        }
                        (true, true) => has_two = true,
                        (false, true) => {
                            pushed = true;
                            queue.push(a);
                            queue.push(b);
                        }
                    }
                }
                if has_two && !pushed {
                    state_[ri] = COVERED;
                }
            }
            if queue.is_empty() {
                break (vids_, p2_, positives_, state_);
            }
        };
        let Gather { mut old, mut old_set, .. } = g;

        // ---- 4. classify regions ----------------------------------------------------
        struct OldInfo {
            face: Face,
            outer2: Vec<DVec2>,
            holes2: Vec<Vec<DVec2>>,
            area: f64,
            lo: DVec2,
            hi: DVec2,
        }
        let mk_info = |m: &Mesh, f: FaceId| {
            let face = m.faces[f].clone();
            let outer2: Vec<DVec2> = face.outer.iter().map(|v| pr.to2(m.pos(*v))).collect();
            let holes2: Vec<Vec<DVec2>> = face.holes.iter().map(|h| h.iter().map(|v| pr.to2(m.pos(*v))).collect()).collect();
            let area = geom::signed_area(&outer2).abs();
            let lo = outer2.iter().fold(DVec2::splat(f64::MAX), |a, b| a.min(*b));
            let hi = outer2.iter().fold(DVec2::splat(f64::MIN), |a, b| a.max(*b));
            OldInfo { face, outer2, holes2, area, lo, hi }
        };
        let mut old_info: HashMap<FaceId, OldInfo> = old.iter().map(|f| (*f, mk_info(self, *f))).collect();
        let old_by_key: HashMap<Vec<VertexId>, FaceId> = old
            .iter()
            .map(|f| {
                let mut k = old_info[f].face.outer.clone();
                k.sort();
                (k, *f)
            })
            .collect();
        type Poly2 = (Vec<DVec2>, Vec<Vec<DVec2>>, i32);
        let overlay: Vec<Poly2> = match cls {
            Classify::Overlay(polys, _) => polys
                .iter()
                .map(|(o, h, f)| {
                    (
                        o.iter().map(|p| pr.to2(*p)).collect(),
                        h.iter().map(|l| l.iter().map(|p| pr.to2(*p)).collect()).collect(),
                        if f.normal.dot(n) >= 0.0 { 1 } else { -1 },
                    )
                })
                .collect(),
            _ => Vec::new(),
        };

        let overlay_area: Vec<f64> =
            overlay.iter().map(|(o, h, _)| geom::signed_area(o).abs() - h.iter().map(|l| geom::signed_area(l).abs()).sum::<f64>()).collect();
        #[derive(Clone, Copy, PartialEq)]
        enum Kind {
            Skip,
            Inherit(FaceId),
            New,
            /// New face with a forced flip and the attribute template of overlay polygon `idx`.
            Forced(bool, usize),
        }
        // (outer, holes, sample, net area, kind)
        type Reg = (Vec<VertexId>, Vec<Vec<VertexId>>, DVec2, f64, Kind);
        let mut regs: Vec<Reg> = Vec::new();
        for (r, st) in positives.iter().zip(&state).filter(|(_, st)| **st != COVERED) {
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
                if inherit.is_none() && *st == EXACT {
                    // Safety net for the localized gathering: an existing in-plane face that
                    // was not gathered but contains this region (its boundary was gathered
                    // only as loops of neighbouring faces) is inherited, never duplicated.
                    let p3 = pr.to3(sample);
                    let d = DVec3::splat(TOL);
                    for f in self.cache.fgrid.query_box(p3 - d, p3 + d) {
                        if old_set.contains(&f) || !self.faces.contains_key(f) || !face_in_plane(self, f) {
                            continue;
                        }
                        let oi = mk_info(self, f);
                        if geom::point_in_region(sample, &oi.outer2, &oi.holes2, TOL) == PointLoc::Inside
                            && inherit.is_none_or(|(a, _)| oi.area < a)
                        {
                            inherit = Some((oi.area, f));
                        }
                    }
                    if let Some((_, f)) = inherit {
                        old_set.insert(f);
                        old.push(f);
                        old_info.insert(f, mk_info(self, f));
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
                    Classify::Covered(pts) => {
                        let (lo, hi) = outer_pts.iter().fold((DVec2::splat(f64::MAX), DVec2::splat(f64::MIN)), |(a, b), p| (a.min(*p), b.max(*p)));
                        let hit = pts.iter().position(|(p, _)| {
                            let q = pr.to2(*p);
                            q.cmpge(lo).all() && q.cmple(hi).all() && geom::point_in_region(q, &outer_pts, &holes_pts, TOL) == PointLoc::Inside
                        });
                        match hit {
                            Some(i) => Kind::Forced(pts[i].1.normal.dot(n) < 0.0, i),
                            None => Kind::Skip,
                        }
                    }
                    Classify::Overlay(_, union) => {
                        let inside: Vec<usize> = (0..overlay.len())
                            .filter(|i| {
                                let (o, h, _) = &overlay[*i];
                                geom::point_in_region(sample, o, h, TOL) == PointLoc::Inside
                            })
                            .collect();
                        let net: i32 = inside.iter().map(|i| overlay[*i].2).sum();
                        if *union {
                            // Smallest covering face wins; equal ones (coincident faces) are
                            // decided by a canonical normal direction (order independent).
                            let canon = |i: usize| {
                                let nn = if let Classify::Overlay(p, _) = cls { p[i].2.normal } else { DVec3::Z };
                                let c = [nn.x, nn.y, nn.z].into_iter().find(|c| c.abs() > 1e-9).unwrap_or(0.0);
                                c > 0.0
                            };
                            let best = inside.iter().copied().min_by(|a, b| {
                                let (x, y) = (overlay_area[*a], overlay_area[*b]);
                                if (x - y).abs() <= 1e-9 * x.max(y).max(1.0) {
                                    canon(*b).cmp(&canon(*a))
                                } else {
                                    x.total_cmp(&y)
                                }
                            });
                            match best {
                                Some(i) => Kind::Forced(overlay[i].2 < 0, i),
                                None => Kind::Skip,
                            }
                        } else if net == 0 {
                            Kind::Skip
                        } else {
                            let idx = inside.iter().copied().find(|i| overlay[*i].2 == net.signum()).unwrap();
                            Kind::Forced(net < 0, idx)
                        }
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
                Kind::Forced(flip, idx) => match cls {
                    Classify::Overlay(polys, _) => (polys[idx].2.clone(), flip),
                    Classify::Covered(pts) => (pts[idx].1.clone(), flip),
                    _ => unreachable!(),
                },
                _ => (Face::template(), self.choose_flip(&plane, &outer, orient)),
            };
            let (o, h) = orient_loops(outer, holes, flip);
            let f = self.f_new(o, h, &tmpl);
            result.push(RegionFace { face: f, sample: pr.to3(r.2), area: r.3, created: true });
        }
        result
    }

    /// Prune dangling trees from the gathered edges and extract the bounded regions (with
    /// holes) of their planar arrangement. Returns (vertex ids, 2D positions, regions).
    fn arrangement(&self, pr: &Projector, evec: &[EdgeId], eset: &HashSet<EdgeId>) -> (Vec<VertexId>, Vec<DVec2>, Vec<Region>) {
    // ---- 2. prune dangling trees -----------------------------------------------
    let mut deg: HashMap<VertexId, usize> = HashMap::new();
    for e in evec {
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

        (vids, p2, positives)
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

#[cfg(test)]
mod tests {
    use super::FULL_GATHER;
    use crate::mesh::Mesh;
    use crate::{EntityRef, TOL};
    use glam::DVec3;

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }

    fn q(p: DVec3) -> [i64; 3] {
        let r = |x: f64| (x / (TOL * 10.0)).round() as i64;
        [r(p.x), r(p.y), r(p.z)]
    }

    /// Canonical rotation of a loop of positions (start at the smallest point).
    fn canon(l: Vec<[i64; 3]>) -> Vec<[i64; 3]> {
        let i = (0..l.len()).min_by_key(|i| l[*i]).unwrap_or(0);
        l[i..].iter().chain(l[..i].iter()).copied().collect()
    }

    type FaceKey = (Vec<[i64; 3]>, Vec<Vec<[i64; 3]>>, [i64; 3]);

    fn face_keys(m: &Mesh) -> Vec<FaceKey> {
        let mut out: Vec<FaceKey> = m
            .faces
            .values()
            .map(|f| {
                let outer = canon(f.outer.iter().map(|v| q(m.vertices[*v].pos)).collect());
                let mut holes: Vec<_> = f.holes.iter().map(|h| canon(h.iter().map(|v| q(m.vertices[*v].pos)).collect())).collect();
                holes.sort();
                let n = f.normal;
                (outer, holes, [n.x.round() as i64, n.y.round() as i64, n.z.round() as i64])
            })
            .collect();
        out.sort();
        out
    }

    fn edge_keys(m: &Mesh) -> Vec<[[i64; 3]; 2]> {
        let mut out: Vec<_> = m
            .edges
            .values()
            .map(|e| {
                let (a, b) = (q(m.vertices[e.v[0]].pos), q(m.vertices[e.v[1]].pos));
                if a < b { [a, b] } else { [b, a] }
            })
            .collect();
        out.sort();
        out
    }

    /// The k-th edge in geometric order (same choice in both meshes).
    fn kth_edge(m: &Mesh, k: usize) -> Option<crate::EdgeId> {
        let mut es: Vec<_> = m
            .edges
            .iter()
            .map(|(id, e)| {
                let (a, b) = (q(m.vertices[e.v[0]].pos), q(m.vertices[e.v[1]].pos));
                (if a < b { [a, b] } else { [b, a] }, id)
            })
            .collect();
        es.sort_by_key(|x| x.0);
        es.get(k % es.len().max(1)).map(|x| x.1)
    }

    fn kth_face(m: &Mesh, k: usize) -> Option<crate::FaceId> {
        let mut fs: Vec<_> = m
            .faces
            .iter()
            .map(|(id, f)| (canon(f.outer.iter().map(|v| q(m.vertices[*v].pos)).collect()), id))
            .collect();
        fs.sort_by(|x, y| x.0.cmp(&y.0));
        fs.get(k % fs.len().max(1)).map(|x| x.1)
    }

    fn same<T: Ord + std::fmt::Debug + Clone>(a: Vec<T>, b: Vec<T>, what: &str, ctx: impl Fn() -> String) {
        if a != b {
            let only_a: Vec<_> = a.iter().filter(|x| !b.contains(x)).cloned().collect();
            let only_b: Vec<_> = b.iter().filter(|x| !a.contains(x)).cloned().collect();
            panic!("{what} differ\n  localized only: {only_a:?}\n  reference only: {only_b:?}\n  {}", ctx());
        }
    }

    #[derive(Debug, Clone, Copy)]
    enum Op {
        Line(DVec3, DVec3),
        Rect(DVec3, DVec3),
        EraseEdge(usize),
        EraseFace(usize),
        PushPull(usize, f64),
        /// Line between two lattice points of the bounding box of the k-th face (in its plane).
        LineOnFace(usize, [u64; 4]),
        /// Closed box [lo, hi] built from an elevated rectangle (crosses other solids).
        FloatBox(DVec3, DVec3),
        /// Intersect every face with every other one.
        Intersect,
        /// Copy the connected geometry of the k-th face, shifted.
        Duplicate(usize, DVec3),
    }

    fn apply(m: &mut Mesh, op: Op, full: bool) {
        FULL_GATHER.with(|c| c.set(full));
        match op {
            Op::Line(a, b) => {
                m.add_line(a, b);
            }
            Op::Rect(a, b) => {
                let pts = [a, DVec3::new(b.x, a.y, a.z), b, DVec3::new(a.x, b.y, a.z)];
                m.add_face(&pts);
            }
            Op::EraseEdge(k) => {
                if let Some(e) = kth_edge(m, k) {
                    m.erase(&[EntityRef::Edge(e)]);
                }
            }
            Op::EraseFace(k) => {
                if let Some(f) = kth_face(m, k) {
                    m.erase(&[EntityRef::Face(f)]);
                }
            }
            Op::PushPull(k, d) => {
                if let Some(f) = kth_face(m, k) {
                    let _ = m.push_pull(f, d, false);
                }
            }
            Op::LineOnFace(k, t) => {
                if let Some(f) = kth_face(m, k) {
                    let b = m.face_aabb(f).unwrap();
                    let n = m.faces[f].normal.abs();
                    let at = |u: u64, w: u64| {
                        let (fu, fw) = (u as f64 / 4.0, w as f64 / 4.0);
                        let s = b.size();
                        if n.z > 0.9 {
                            DVec3::new(b.min.x + s.x * fu, b.min.y + s.y * fw, b.min.z)
                        } else if n.x > 0.9 {
                            DVec3::new(b.min.x, b.min.y + s.y * fu, b.min.z + s.z * fw)
                        } else {
                            DVec3::new(b.min.x + s.x * fu, b.min.y, b.min.z + s.z * fw)
                        }
                    };
                    if n.x > 0.9 || n.y > 0.9 || n.z > 0.9 {
                        m.add_line(at(t[0], t[1]), at(t[2], t[3]));
                    }
                }
            }
            Op::FloatBox(lo, hi) => {
                let pts = [lo, DVec3::new(lo.x, hi.y, lo.z), DVec3::new(hi.x, hi.y, lo.z), DVec3::new(hi.x, lo.y, lo.z)];
                if m.add_face_exact(&pts).is_some() {
                    // The piece at a fixed corner (the returned face is the largest piece,
                    // ties depend on id order, which differs between gather modes).
                    let p = lo + DVec3::new(10.0, 10.0, 0.0);
                    let mut fs: Vec<_> = m.faces.keys().collect();
                    fs.sort_by_key(|f| canon(m.faces[*f].outer.iter().map(|v| q(m.vertices[*v].pos)).collect()));
                    let f = fs.into_iter().find(|f| {
                        m.faces[*f].normal.z.abs() > 0.999
                            && m.face_plane(*f).distance(p).abs() < TOL
                            && m.face_point_loc(*f, p) == crate::geom::PointLoc::Inside
                    });
                    if let Some(f) = f {
                        let d = if m.faces[f].normal.z < 0.0 { lo.z - hi.z } else { hi.z - lo.z };
                        let _ = m.push_pull(f, d, false);
                    }
                }
            }
            Op::Intersect => {
                m.intersect_all();
            }
            Op::Duplicate(k, off) => {
                if let Some(f) = kth_face(m, k) {
                    let mut ents: Vec<EntityRef> = m.connected(EntityRef::Face(f)).into_iter().collect();
                    ents.sort_by_key(|e| format!("{e:?}"));
                    m.duplicate(&ents, &glam::DMat4::from_translation(off));
                }
            }
        }
        FULL_GATHER.with(|c| c.set(false));
    }

    fn random_op(r: &mut Rng, size: u64, step: f64, planar: bool) -> Op {
        let pt = |r: &mut Rng| DVec3::new(r.below(size + 1) as f64 * step, r.below(size + 1) as f64 * step, 0.0);
        let kind = r.below(if planar { 12 } else { 14 });
        match kind {
            0..=6 => {
                let a = pt(r);
                let mut b = pt(r);
                if r.below(3) == 0 {
                    // axis-aligned lines are the common case
                    b = if r.below(2) == 0 { DVec3::new(a.x, b.y, 0.0) } else { DVec3::new(b.x, a.y, 0.0) };
                }
                Op::Line(a, b)
            }
            7..=9 => {
                let (a, b) = (pt(r), pt(r));
                Op::Rect(a.min(b), a.max(b))
            }
            10 => Op::EraseEdge(r.below(1000) as usize),
            11 => Op::EraseFace(r.below(1000) as usize),
            _ => Op::PushPull(r.below(1000) as usize, if r.below(2) == 0 { step } else { -step * 0.5 }),
        }
    }

    /// Structured 3D scenario: rectangles on the ground, push/pull of horizontal faces, lines
    /// drawn on faces (top faces and walls), erasing edges/faces.
    fn random_op_3d(r: &mut Rng) -> Op {
        match r.below(10) {
            0..=2 => {
                let (x0, y0) = (r.below(6) as f64 * 100.0, r.below(6) as f64 * 100.0);
                let (w, h) = ((1 + r.below(3)) as f64 * 100.0, (1 + r.below(3)) as f64 * 100.0);
                Op::Rect(DVec3::new(x0, y0, 0.0), DVec3::new(x0 + w, y0 + h, 0.0))
            }
            3..=4 => Op::PushPull(r.below(1000) as usize, if r.below(3) == 0 { -50.0 } else { 100.0 * (1 + r.below(2)) as f64 }),
            5..=7 => Op::LineOnFace(r.below(1000) as usize, [r.below(5), r.below(5), r.below(5), r.below(5)]),
            8 => Op::EraseEdge(r.below(1000) as usize),
            _ => Op::EraseFace(r.below(1000) as usize),
        }
    }

    /// [`random_op_3d`] plus floating boxes crossing other solids, Intersect Faces and copies.
    fn random_op_3d_ix(r: &mut Rng) -> Op {
        match r.below(16) {
            0..=9 => random_op_3d(r),
            10..=12 => {
                let c = |r: &mut Rng| r.below(8) as f64 * 50.0;
                let lo = DVec3::new(c(r), c(r), 25.0 + r.below(4) as f64 * 50.0);
                let s = |r: &mut Rng| (1 + r.below(4)) as f64 * 50.0;
                let hi = lo + DVec3::new(s(r), s(r), s(r));
                Op::FloatBox(lo, hi)
            }
            13..=14 => Op::Intersect,
            _ => {
                let o = |r: &mut Rng| (r.below(5) as f64 - 2.0) * 50.0;
                Op::Duplicate(r.below(1000) as usize, DVec3::new(o(r), o(r), 0.0))
            }
        }
    }

    /// Independent geometric sanity: holes lie inside their outer loop and coplanar faces do
    /// not overlap (interior sample of one face is not strictly inside another).
    fn insanity(m: &Mesh) -> Option<&'static str> {
        use crate::geom::{self, PointLoc, Projector};
        let ids: Vec<_> = m.faces.keys().collect();
        for &f in &ids {
            let face = &m.faces[f];
            let pr = Projector::new(m.pos(face.outer[0]), face.normal);
            let o: Vec<_> = face.outer.iter().map(|v| pr.to2(m.pos(*v))).collect();
            for h in &face.holes {
                if h.iter().any(|v| geom::point_in_polygon(pr.to2(m.pos(*v)), &o, TOL) == PointLoc::Outside) {
                    return Some("hole outside face");
                }
            }
            let hs: Vec<Vec<_>> = face.holes.iter().map(|h| h.iter().map(|v| pr.to2(m.pos(*v))).collect()).collect();
            let sample = pr.to3(super::interior_sample(&o, &hs));
            for &g in &ids {
                if g == f || m.faces[g].normal.dot(face.normal).abs() < 1.0 - 1e-6 {
                    continue;
                }
                if m.face_plane(g).distance(sample).abs() > TOL {
                    continue;
                }
                if m.face_point_loc(g, sample) == PointLoc::Inside {
                    return Some("overlapping coplanar faces");
                }
                // Partial overlaps: a boundary vertex or edge midpoint strictly inside.
                for l in face.loops() {
                    let k = l.len();
                    for i in 0..k {
                        let (a, b) = (m.pos(l[i]), m.pos(l[(i + 1) % k]));
                        if m.face_point_loc(g, a) == PointLoc::Inside || m.face_point_loc(g, (a + b) * 0.5) == PointLoc::Inside {
                            return Some("partially overlapping coplanar faces");
                        }
                    }
                }
            }
        }
        None
    }

    /// Random 3D editing session: the localized path must satisfy every invariant after every
    /// step and match the full-gather reference exactly.
    fn run_3d(seed: u64, steps: usize) {
        let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let mut local = Mesh::new();
        let mut reference = Mesh::new();
        let mut ops = Vec::new();
        for i in 0..steps {
            let op = random_op_3d(&mut r);
            ops.push(op);
            apply(&mut local, op, false);
            apply(&mut reference, op, true);
            let ctx = || format!("seed {seed}, step {i}, ops {ops:?}");
            if let Some(v) = violation(&local) {
                panic!("localized mesh broken ({v}): {}", ctx());
            }
            if let Some(v) = violation(&reference) {
                panic!("reference mesh broken ({v}): {}", ctx());
            }
            same(edge_keys(&local), edge_keys(&reference), "edges", ctx);
            same(face_keys(&local), face_keys(&reference), "faces", ctx);
        }
    }

    /// Do two edges touch anywhere except at a shared endpoint (crossing without a vertex,
    /// T-junction, collinear overlap), within the kernel tolerance?
    fn touching_edges(m: &Mesh) -> Option<([DVec3; 2], [DVec3; 2])> {
        let segs: Vec<[DVec3; 2]> = m.edges.values().map(|e| [m.pos(e.v[0]), m.pos(e.v[1])]).collect();
        let tol = TOL * 0.99;
        let on = |p: DVec3, s: &[DVec3; 2]| crate::geom::point_seg_dist(p, s[0], s[1]).0 < tol;
        let interior = |p: DVec3, s: &[DVec3; 2]| on(p, s) && p.distance(s[0]) > tol && p.distance(s[1]) > tol;
        for (i, s) in segs.iter().enumerate() {
            for t in &segs[i + 1..] {
                // T-junctions and overlaps: an endpoint inside the other edge.
                if interior(t[0], s) || interior(t[1], s) || interior(s[0], t) || interior(s[1], t) {
                    return Some((*s, *t));
                }
                // Proper crossing.
                let (d1, d2, r) = (s[1] - s[0], t[1] - t[0], s[0] - t[0]);
                let (a, e, b, c, ff) = (d1.dot(d1), d2.dot(d2), d1.dot(d2), d1.dot(r), d2.dot(r));
                let den = a * e - b * b;
                if den <= 1e-9 * a * e {
                    continue;
                }
                let sp = ((b * ff - c * e) / den).clamp(0.0, 1.0);
                let tp = ((b * sp + ff) / e).clamp(0.0, 1.0);
                let (p, q) = (s[0] + d1 * sp, t[0] + d2 * tp);
                if p.distance(q) < tol && interior(p, s) && interior(q, t) {
                    return Some((*s, *t));
                }
            }
        }
        None
    }

    fn run(seed: u64, steps: usize, planar: bool) {
        let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let mut local = Mesh::new();
        let mut reference = Mesh::new();
        let mut ops = Vec::new();
        for i in 0..steps {
            let op = random_op(&mut r, 8, 100.0, planar);
            ops.push(op);
            apply(&mut local, op, false);
            apply(&mut reference, op, true);
            let ctx = || format!("seed {seed}, step {i}, ops {ops:?}");
            if let Err(e) = local.validate() {
                panic!("localized mesh invalid ({e}): {}", ctx());
            }
            same(edge_keys(&local), edge_keys(&reference), "edges", ctx);
            same(face_keys(&local), face_keys(&reference), "faces", ctx);
        }
    }


    #[test]
    fn localized_refresh_matches_full_gather_planar() {
        for seed in 1..=60 {
            run(seed, 40, true);
        }
    }

    /// First invariant violation of a mesh, by category.
    fn violation(m: &Mesh) -> Option<String> {
        if let Err(e) = m.validate() {
            let e = if e.starts_with("face") { e.split_once(' ').map(|x| x.1.split_once(' ').map(|y| y.1.to_string()).unwrap_or_default()).unwrap_or(e) } else { e };
            return Some(format!("invalid: {e}"));
        }
        if touching_edges(m).is_some() {
            return Some("edges touch without a shared vertex".into());
        }
        for face in m.faces.values() {
            let mut seen = std::collections::HashSet::new();
            for l in face.loops() {
                for i in 0..l.len() {
                    let (a, b) = (l[i], l[(i + 1) % l.len()]);
                    if !seen.insert(if a < b { (a, b) } else { (b, a) }) {
                        return Some("face loops run along an edge twice".into());
                    }
                }
            }
        }
        if let Some(s) = insanity(m) {
            return Some(s.to_string());
        }
        // A chord: an edge inside a face joining two vertices of one of its loops must split it
        // (a bridge between a hole and the outer loop does not).
        for (f, face) in &m.faces {
            for lp in face.loops() {
                let vs: std::collections::HashSet<_> = lp.iter().copied().collect();
                for &v in &vs {
                    for e in m.vertex_edges(v) {
                        let o = m.edges[e].other(v);
                        // Edges of faces in other planes (faces intersecting without Intersect
                        // Faces) are not required to split.
                        if !vs.contains(&o)
                            || m.edge_faces(e).iter().any(|g| *g == f || m.faces[*g].normal.dot(face.normal).abs() < 1.0 - 1e-6)
                        {
                            continue;
                        }
                        let mid = (m.pos(v) + m.pos(o)) * 0.5;
                        if m.face_plane(f).distance(mid).abs() < TOL && m.face_point_loc(f, mid) == crate::geom::PointLoc::Inside {
                            return Some("unsplit chord inside a face".into());
                        }
                    }
                }
            }
        }
        None
    }

    /// Statistics of invariant violations of the (default, localized) kernel over random 3D
    /// editing sessions. `cargo test -p varstotas_kernel --lib invariants_survey -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn invariants_survey() {
        let mut cats: std::collections::BTreeMap<String, Vec<(u64, usize, String)>> = Default::default();
        let mut ok = 0;
        let n: u64 = std::env::var("SURVEY_N").map(|s| s.parse().unwrap()).unwrap_or(600);
        let steps: usize = std::env::var("SURVEY_STEPS").map(|s| s.parse().unwrap()).unwrap_or(30);
        let from: u64 = std::env::var("SURVEY_FROM").map(|s| s.parse().unwrap()).unwrap_or(1000);
        for seed in from..from + n {
            let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
            let mut m = Mesh::new();
            let mut bad = None;
            for i in 0..steps {
                let op = random_op_3d(&mut r);
                apply(&mut m, op, false);
                if let Some(v) = violation(&m) {
                    let kind = format!("{op:?}");
                    bad = Some((v, i, kind.split('(').next().unwrap().to_string()));
                    break;
                }
            }
            match bad {
                None => ok += 1,
                Some((v, i, k)) => cats.entry(v).or_default().push((seed, i, k)),
            }
        }
        eprintln!("SURVEY ok {ok}");
        for (k, v) in &cats {
            eprintln!("  {k}: {} e.g. {:?}", v.len(), &v[..v.len().min(8)]);
        }
    }

    #[test]
    fn localized_refresh_matches_full_gather_3d() {
        // Random 3D editing (rectangles, lines on faces, push/pull, push-through, erasing).
        for seed in 1000..=1400 {
            run_3d(seed, 30);
        }
    }

    #[test]
    fn grid_raycast_matches_brute_force() {
        // Random 3D models; rays from random points towards vertices, edge midpoints (ties
        // between faces sharing an edge), face samples and random directions, incl. axis
        // aligned and unnormalized ones.
        let mut hits = 0;
        for seed in 9000u64..9040 {
            let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
            let mut m = Mesh::new();
            for _ in 0..30 {
                apply(&mut m, random_op_3d(&mut r), false);
            }
            m.ensure();
            let mut brute = m.clone();
            brute.invalidate_caches();
            let f = |r: &mut Rng, s: f64| (r.below(20001) as f64 / 10000.0 - 1.0) * s;
            let verts: Vec<DVec3> = m.vertices.values().map(|v| v.pos).collect();
            let mids: Vec<DVec3> = m.edges.values().map(|e| (m.pos(e.v[0]) + m.pos(e.v[1])) * 0.5).collect();
            for k in 0..300 {
                let origin = DVec3::new(f(&mut r, 1500.0) + 400.0, f(&mut r, 1500.0) + 400.0, f(&mut r, 800.0));
                let target = match k % 5 {
                    0 if !verts.is_empty() => verts[r.below(verts.len() as u64) as usize],
                    1 if !mids.is_empty() => mids[r.below(mids.len() as u64) as usize],
                    2 => DVec3::new(origin.x, origin.y, origin.z - 1.0 - r.below(3) as f64),
                    3 => DVec3::new(r.below(9) as f64 * 100.0, r.below(9) as f64 * 100.0, r.below(5) as f64 * 50.0 - 100.0),
                    _ => origin + DVec3::new(f(&mut r, 1.0), f(&mut r, 1.0), f(&mut r, 1.0)),
                };
                let dir = (target - origin) * (0.001 + r.below(1000) as f64 / 10.0);
                let a = m.raycast_faces_filtered(origin, dir, |_| true);
                let b = brute.raycast_faces_filtered(origin, dir, |_| true);
                assert_eq!(a, b, "seed {seed} ray {k}: {origin:?} {dir:?}");
                hits += a.is_some() as usize;
            }
        }
        assert!(hits > 3000, "too few hits to be meaningful: {hits}");
        // A dense grid of small faces with a pushed cell; grazing rays cross many cells.
        let mut m = Mesh::new();
        for i in 0..=30 {
            let x = i as f64 * 10.0;
            apply(&mut m, Op::Line(DVec3::new(x, 0.0, 0.0), DVec3::new(x, 300.0, 0.0)), false);
            apply(&mut m, Op::Line(DVec3::new(0.0, x, 0.0), DVec3::new(300.0, x, 0.0)), false);
        }
        let f = m.faces.keys().nth(400).unwrap();
        m.push_pull(f, -15.0, false).unwrap();
        m.ensure();
        let mut brute = m.clone();
        brute.invalidate_caches();
        let mut r = Rng(77);
        let mut hits = 0;
        for k in 0..3000 {
            let origin = DVec3::new(r.below(3001) as f64 / 10.0 - 50.0, r.below(3001) as f64 / 10.0 - 50.0, 1.0 + r.below(400) as f64 / 10.0);
            let target = DVec3::new(r.below(31) as f64 * 10.0, r.below(31) as f64 * 10.0 + if k % 2 == 0 { 0.0 } else { 5.0 }, 0.0);
            let dir = target - origin;
            let a = m.raycast_faces_filtered(origin, dir, |_| true);
            assert_eq!(a, brute.raycast_faces_filtered(origin, dir, |_| true), "grid ray {k}");
            hits += a.is_some() as usize;
        }
        assert!(hits > 2500);
    }

    #[test]
    fn obj_round_trip_of_random_models_keeps_area_and_validity() {
        let dir = std::env::temp_dir().join(format!("varstotas_objrt_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let area = |m: &Mesh| m.faces.keys().map(|f| m.face_area(f)).sum::<f64>();
        for seed in 6000u64..6040 {
            let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
            let mut model = crate::Model::new();
            for _ in 0..30 {
                apply(model.root_mesh_mut(), random_op_3d(&mut r), false);
            }
            let p = dir.join(format!("{seed}.obj"));
            crate::io::export_obj(&model, &p).unwrap();
            let mut back = crate::Model::new();
            crate::io::import_obj(&mut back, &p).unwrap();
            let (a, b) = (model.root_mesh(), back.root_mesh());
            b.validate().unwrap_or_else(|e| panic!("seed {seed}: {e}"));
            assert!((area(a) - area(b)).abs() < 1e-6 * area(a).max(1.0), "seed {seed}: area {} -> {}", area(a), area(b));
            // Merging never makes more faces than the original had.
            assert!(b.faces.len() <= a.faces.len(), "seed {seed}: {} -> {} faces", a.faces.len(), b.faces.len());
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn regression_seeds_keep_invariants() {
        // Random sessions that broke the kernel (seed, failing step): crossing diagonal after
        // push/pull (1002), duplicate walls from push-through (1150), T-junction in
        // push-through (1204), side top edge overlapping an edge (1458), missed crossing of a
        // split piece (1069), partial coplanar overlap (1507), stretched face overlapping
        // (1688), re-split pieces (2426), slid vertex merged before its faces were collected
        // (3758), new side not split by a dangling line (1065), faceless side pieces after
        // merging (1332), chord after a split at a vertex (3522), order-dependent leftover
        // edge (1386), stretched face folded onto itself (2201), overlay of a neighbouring
        // pending face (7476).
        let cases = [
            (1002, 12), (1150, 11), (1204, 13), (1458, 18), (1069, 25), (1507, 12), (1688, 3), (2426, 13),
            (3758, 34), (1065, 10), (1332, 16), (3522, 29), (1386, 29), (2201, 16), (7476, 32),
        ];
        for (seed, step) in cases {
            let mut r = Rng((seed as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
            let mut m = Mesh::new();
            for i in 0..=step {
                let op = random_op_3d(&mut r);
                apply(&mut m, op, false);
                if let Some(v) = violation(&m) {
                    panic!("{v}: seed {seed}, step {i}");
                }
            }
        }
    }

    #[test]
    fn regression_seeds_with_intersect_keep_invariants() {
        // Sessions with intersect/duplicate that broke the kernel: a push/pull stretching a
        // hole of a perpendicular neighbour out through its outer loop (30431, 30585, 32381;
        // the first and last also fail without intersect/duplicate ops).
        for seed in [30_431u64, 30_585, 32_381] {
            for noix in [false, true] {
                let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
                let mut local = Mesh::new();
                let mut reference = Mesh::new();
                for i in 0..30 {
                    let op = random_op_3d_ix(&mut r);
                    if noix && matches!(op, Op::Intersect | Op::Duplicate(..)) {
                        continue;
                    }
                    apply(&mut local, op, false);
                    apply(&mut reference, op, true);
                    if let Some(v) = violation(&local) {
                        panic!("{v}: seed {seed}, step {i}, noix {noix}");
                    }
                    same(edge_keys(&local), edge_keys(&reference), "edges", || format!("seed {seed}, step {i}"));
                    same(face_keys(&local), face_keys(&reference), "faces", || format!("seed {seed}, step {i}"));
                }
            }
        }
    }

    #[test]
    fn random_3d_sessions_keep_invariants() {
        // Valid topology, no edges touching without a shared vertex, no overlapping coplanar
        // faces, no holes outside faces, no unsplit chords -- after every step.
        for seed in 5000u64..5300 {
            let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
            let mut m = Mesh::new();
            let mut ops = Vec::new();
            for i in 0..40 {
                let op = random_op_3d(&mut r);
                ops.push(op);
                apply(&mut m, op, false);
                if let Some(v) = violation(&m) {
                    panic!("{v}: seed {seed}, step {i}, ops {ops:?}");
                }
            }
        }
    }

    #[test]
    fn random_sessions_with_intersect_keep_invariants() {
        // Crossing solids, Intersect Faces and copies welded into existing geometry: every
        // invariant holds after every step, and the localized refresh matches the reference.
        for seed in std::env::var("IXF").map(|s| s.parse().unwrap()).unwrap_or(20_000u64)..std::env::var("IXN").map(|s| s.parse().unwrap()).unwrap_or(20_200) {
            let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
            let mut local = Mesh::new();
            let mut reference = Mesh::new();
            let mut ops = Vec::new();
            for i in 0..30 {
                let op = random_op_3d_ix(&mut r);
                ops.push(op);
                apply(&mut local, op, false);
                apply(&mut reference, op, true);
                let ctx = || format!("seed {seed}, step {i}, ops {ops:?}");
                if let Some(v) = violation(&local) {
                    panic!("{v}: {}", ctx());
                }
                same(edge_keys(&local), edge_keys(&reference), "edges", ctx);
                same(face_keys(&local), face_keys(&reference), "faces", ctx);
            }
        }
    }

    /// `IXS_FROM=.. IXS_N=.. cargo test --release -p varstotas_kernel --lib ix_survey -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn ix_survey() {
        let from: u64 = std::env::var("IXS_FROM").map(|s| s.parse().unwrap()).unwrap_or(30_000);
        let n: u64 = std::env::var("IXS_N").map(|s| s.parse().unwrap()).unwrap_or(1000);
        let mut cats: std::collections::BTreeMap<String, Vec<(u64, usize)>> = Default::default();
        for seed in from..from + n {
            let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
            let mut local = Mesh::new();
            let mut reference = Mesh::new();
            let noix = std::env::var("NOIX").is_ok();
            for i in 0..30 {
                let op = random_op_3d_ix(&mut r);
                if noix && matches!(op, Op::Intersect | Op::Duplicate(..)) {
                    continue;
                }
                apply(&mut local, op, false);
                apply(&mut reference, op, true);
                let kind = format!("{op:?}").split('(').next().unwrap().to_string();
                let bad = if let Some(v) = violation(&local) {
                    Some(format!("local {v}"))
                } else if let Some(v) = violation(&reference) {
                    Some(format!("ref {v}"))
                } else if edge_keys(&local) != edge_keys(&reference) || face_keys(&local) != face_keys(&reference) {
                    Some("mismatch".to_string())
                } else {
                    None
                };
                if let Some(b) = bad {
                    cats.entry(format!("{b} @ {kind}")).or_default().push((seed, i));
                    break;
                }
            }
        }
        let bad: usize = cats.values().map(|v| v.len()).sum();
        eprintln!("IX SURVEY {from}..{}: ok {} of {n}", from + n, n as usize - bad);
        for (k, v) in &cats {
            eprintln!("{k}: {} e.g. {:?}", v.len(), &v[..v.len().min(6)]);
        }
    }

    #[test]
    fn intersect_twice_changes_nothing() {
        for seed in 21_000u64..21_060 {
            let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
            let mut m = Mesh::new();
            for _ in 0..20 {
                apply(&mut m, random_op_3d_ix(&mut r), false);
            }
            m.intersect_all();
            let (e, f) = (edge_keys(&m), face_keys(&m));
            let again = m.intersect_all();
            assert!(again.is_empty(), "seed {seed}: second intersect created {} edges", again.len());
            assert_eq!(edge_keys(&m), e, "seed {seed}");
            assert_eq!(face_keys(&m), f, "seed {seed}");
        }
    }

    #[test]
    fn localized_refresh_matches_full_gather_on_dense_grid() {
        // Lines over an existing 12x12 grid of faces (lots of two-sided edges).
        for seed in 200..=215 {
            let mut r = Rng(seed * 7919 + 1);
            let mut local = Mesh::new();
            let mut reference = Mesh::new();
            let mut ops = Vec::new();
            for i in 0..=12 {
                let x = i as f64 * 50.0;
                for op in [Op::Line(DVec3::new(x, 0.0, 0.0), DVec3::new(x, 600.0, 0.0)), Op::Line(DVec3::new(0.0, x, 0.0), DVec3::new(600.0, x, 0.0))] {
                    apply(&mut local, op, false);
                    apply(&mut reference, op, true);
                }
            }
            assert_eq!(local.faces.len(), 144);
            for i in 0..25 {
                let op = random_op(&mut r, 24, 25.0, true);
                ops.push(op);
                apply(&mut local, op, false);
                apply(&mut reference, op, true);
                let ctx = || format!("seed {seed}, step {i}, ops {ops:?}");
                local.validate().unwrap_or_else(|e| panic!("invalid ({e}): {}", ctx()));
                same(edge_keys(&local), edge_keys(&reference), "edges", ctx);
                same(face_keys(&local), face_keys(&reference), "faces", ctx);
            }
        }
    }
}
