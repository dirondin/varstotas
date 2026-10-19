//! Sticky line insertion: vertex merging, edge splitting, and automatic face creation.

use super::{Classify, Orient};
use crate::geom::{self, Projector};
use crate::mesh::{Mesh, PTOL};
use crate::{EdgeId, FaceId, Plane, TOL, VertexId};
use glam::DVec3;
use std::collections::HashSet;

impl Mesh {
    /// Vertex at `p`: an existing one within TOL, or a new one (splitting an edge it lies on).
    pub(crate) fn vertex_at(&mut self, p: DVec3) -> VertexId {
        if let Some(v) = self.v_find(p, TOL) {
            return v;
        }
        let mut best: Option<(f64, EdgeId)> = None;
        for e in self.cache.egrid.query_segment(p, p) {
            let Some(ed) = self.edges.get(e) else { continue };
            let (a, b) = (self.pos(ed.v[0]), self.pos(ed.v[1]));
            let (d, t) = geom::point_seg_dist(p, a, b);
            if d < TOL && t > 0.0 && t < 1.0 && best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, e));
            }
        }
        let v = self.v_new(p);
        if let Some((_, e)) = best {
            self.e_split(e, v);
        }
        v
    }

    /// Insert segment `a..b` without creating faces. Returns `(pieces, newly created edges)`.
    pub(crate) fn insert_segment(&mut self, a: DVec3, b: DVec3) -> (Vec<EdgeId>, Vec<EdgeId>) {
        if (a - b).length() < TOL {
            return (Vec::new(), Vec::new());
        }
        let va = self.vertex_at(a);
        let vb = self.vertex_at(b);
        if va == vb {
            return (Vec::new(), Vec::new());
        }
        let (pa, pb) = (self.pos(va), self.pos(vb));
        let mut splits: Vec<(f64, VertexId)> = vec![(0.0, va), (1.0, vb)];

        // Crossing edges.
        let mut crossings: Vec<(EdgeId, f64, DVec3)> = Vec::new();
        let mut split_at_end: Vec<(EdgeId, VertexId)> = Vec::new();
        for e in self.cache.egrid.query_segment(pa, pb) {
            let Some(ed) = self.edges.get(e) else { continue };
            if ed.has(va) || ed.has(vb) {
                continue;
            }
            let (c, d) = (self.pos(ed.v[0]), self.pos(ed.v[1]));
            let Some((s, t, p1, p2)) = geom::seg_seg_closest(pa, pb, c, d) else { continue };
            if (p1 - p2).length() >= TOL {
                continue;
            }
            if (p2 - c).length() < TOL || (p2 - d).length() < TOL {
                continue; // endpoint of e lies on the new line: handled by the vertex scan
            }
            if !(0.0..=1.0).contains(&t) {
                continue;
            }
            if (p1 - pa).length() < TOL {
                split_at_end.push((e, va));
            } else if (p1 - pb).length() < TOL {
                split_at_end.push((e, vb));
            } else {
                crossings.push((e, s, p1));
            }
        }
        for (e, v) in split_at_end {
            if self.edges.contains_key(e) {
                self.e_split(e, v);
            }
        }
        for (e, s, p) in crossings {
            if !self.edges.contains_key(e) {
                continue;
            }
            let v = match self.v_find(p, TOL) {
                Some(v) => v,
                None => self.v_new(p),
            };
            let ed = &self.edges[e];
            if !ed.has(v) {
                self.e_split(e, v);
            }
            splits.push((s, v));
        }

        // Existing vertices lying on the new segment.
        for v in self.cache.vgrid.query_segment(pa, pb) {
            if v == va || v == vb {
                continue;
            }
            let Some(vd) = self.vertices.get(v) else { continue };
            let (d, t) = geom::point_seg_dist(vd.pos, pa, pb);
            if d < TOL && t > 0.0 && t < 1.0 {
                splits.push((t, v));
            }
        }

        splits.sort_by(|x, y| x.0.total_cmp(&y.0));
        let mut chain: Vec<VertexId> = Vec::with_capacity(splits.len());
        for (_, v) in splits {
            if !chain.contains(&v) {
                chain.push(v);
            }
        }
        let mut pieces = Vec::new();
        let mut created = Vec::new();
        for w in chain.windows(2) {
            let (e, new) = self.e_ensure(w[0], w[1]);
            pieces.push(e);
            if new {
                created.push(e);
            }
        }
        (pieces, created)
    }

    /// Directions of non-collinear edges reachable from `e`'s endpoints through chains of
    /// edges collinear with `e`.
    fn neighbor_dirs(&self, e: EdgeId) -> Vec<DVec3> {
        let (a, b) = self.edge_segment(e);
        let d = (b - a).normalize_or_zero();
        let mut out = Vec::new();
        let mut seen_e: HashSet<EdgeId> = HashSet::new();
        seen_e.insert(e);
        let mut stack: Vec<_> = self.edges[e].v.to_vec();
        let mut seen_v: HashSet<VertexId> = HashSet::new();
        let mut steps = 0;
        while let Some(v) = stack.pop() {
            if !seen_v.insert(v) {
                continue;
            }
            steps += 1;
            if steps > 512 {
                break;
            }
            for e2 in self.vertex_edges(v) {
                if !seen_e.insert(e2) {
                    continue;
                }
                let (p, q) = self.edge_segment(e2);
                let d2 = (q - p).normalize_or_zero();
                if d.cross(d2).length() < 1e-9 {
                    stack.push(self.edges[e2].other(v));
                } else {
                    out.push(d2);
                }
            }
        }
        out
    }

    /// Faces whose plane contains segment `a..b` and that it may touch.
    pub(crate) fn faces_containing_segment(&self, a: DVec3, b: DVec3) -> Vec<FaceId> {
        let d = (b - a).normalize_or_zero();
        let mut out = Vec::new();
        for f in self.cache.fgrid.query_segment(a, b) {
            let Some(face) = self.faces.get(f) else { continue };
            if face.normal.dot(d).abs() > 1e-6 {
                continue;
            }
            let pl = self.face_plane(f);
            if pl.distance(a).abs() > PTOL || pl.distance(b).abs() > PTOL {
                continue;
            }
            out.push(f);
        }
        out
    }

    /// After inserting `new_edges`, create faces on newly closed planar loops and split faces
    /// crossed by the new edges ("sticky" auto-face behaviour).
    pub(crate) fn detect_faces(&mut self, new_edges: &[EdgeId]) {
        let new_edges: Vec<EdgeId> = new_edges.iter().copied().filter(|e| self.edges.contains_key(*e)).collect();
        if new_edges.is_empty() {
            return;
        }
        let set: HashSet<EdgeId> = new_edges.iter().copied().collect();
        let mut planes: Vec<Plane> = Vec::new();
        let add_plane = |planes: &mut Vec<Plane>, p: Plane| {
            if !planes.iter().any(|q| q.normal.dot(p.normal).abs() > 1.0 - 1e-9 && q.distance(p.origin).abs() < PTOL) {
                planes.push(p);
            }
        };
        for &e in &new_edges {
            let (a, b) = self.edge_segment(e);
            for f in self.faces_containing_segment(a, b) {
                add_plane(&mut planes, self.face_plane(f));
            }
            let d = (b - a).normalize_or_zero();
            for d2 in self.neighbor_dirs(e) {
                let n = d.cross(d2);
                if n.length() > 1e-9 {
                    add_plane(&mut planes, Plane::new(a, n));
                }
            }
        }
        for plane in planes {
            let seeds: Vec<VertexId> = new_edges
                .iter()
                .filter(|e| self.edges.contains_key(**e))
                .flat_map(|e| self.edges[*e].v)
                .filter(|v| plane.distance(self.pos(*v)).abs() <= PTOL)
                .collect();
            if seeds.is_empty() {
                continue;
            }
            self.refresh_plane(plane, &seeds, &[], &Classify::NewEdges(&set), Orient::auto());
        }
    }

    // ---- public sticky editing ---------------------------------------------

    /// Add a straight line. Merges with existing vertices within TOL, splits crossed edges,
    /// splits faces it cuts, and creates new faces for newly closed planar loops.
    /// Returns ids of all edge pieces that make up the line.
    pub fn add_line(&mut self, a: DVec3, b: DVec3) -> Vec<EdgeId> {
        self.ensure();
        let (pieces, created) = self.insert_segment(a, b);
        self.detect_faces(&created);
        pieces.into_iter().filter(|e| self.edges.contains_key(*e)).collect()
    }

    /// Add a closed or open polyline (circles/arcs/polygons). `as_curve` groups the edges into
    /// one curve (shared `Edge::curve` id). Faces are detected once all segments are inserted.
    pub fn add_polyline(&mut self, pts: &[DVec3], closed: bool, as_curve: bool) -> Vec<EdgeId> {
        self.ensure();
        let curve = if as_curve { Some(self.next_curve_id()) } else { None };
        self.add_polyline_curve(pts, closed, curve)
    }

    /// `add_polyline` with an explicit curve id for the new edges.
    pub(crate) fn add_polyline_curve(&mut self, pts: &[DVec3], closed: bool, curve: Option<u32>) -> Vec<EdgeId> {
        self.ensure();
        let pts = clean_points(pts, closed);
        if pts.len() < 2 {
            return Vec::new();
        }
        let mut pieces = Vec::new();
        let mut created = Vec::new();
        let n = pts.len();
        let segs = if closed && n >= 3 { n } else { n - 1 };
        for i in 0..segs {
            let (p, c) = self.insert_segment(pts[i], pts[(i + 1) % n]);
            if let Some(cid) = curve {
                for e in &c {
                    self.edges[*e].curve = Some(cid);
                }
            }
            pieces.extend(p);
            created.extend(c);
        }
        self.detect_faces(&created);
        pieces.retain(|e| self.edges.contains_key(*e));
        pieces
    }

    /// Convenience: closed planar loop → edges + face. Returns the face covering the polygon
    /// (the largest piece if existing edges split it). Faces on the ground plane (z = 0) face
    /// down; otherwise the normal follows the point order (CCW = front).
    /// Returns `None` for fewer than 3 distinct points or non-planar input.
    pub fn add_face(&mut self, pts: &[DVec3]) -> Option<FaceId> {
        self.add_face_impl(pts, true)
    }

    /// Like [`Mesh::add_face`] but the normal always follows the point order (used by importers).
    pub fn add_face_exact(&mut self, pts: &[DVec3]) -> Option<FaceId> {
        self.add_face_impl(pts, false)
    }

    fn add_face_impl(&mut self, pts: &[DVec3], ground_rule: bool) -> Option<FaceId> {
        self.ensure();
        let pts = clean_points(pts, true);
        if pts.len() < 3 {
            return None;
        }
        let n = geom::newell(&pts);
        if n.length() < 1e-9 {
            return None;
        }
        let n = n.normalize();
        let plane = Plane::new(pts[0], n);
        if pts.iter().any(|p| plane.distance(*p).abs() > PTOL * 10.0) {
            return None;
        }
        let mut seeds = Vec::new();
        let mut created = Vec::new();
        let m = pts.len();
        for i in 0..m {
            let (p, c) = self.insert_segment(pts[i], pts[(i + 1) % m]);
            for e in &p {
                seeds.extend(self.edges[*e].v);
            }
            created.extend(c);
        }
        // Splits of faces in other planes that contain the new edges.
        let mut other_planes: Vec<Plane> = Vec::new();
        for &e in &created {
            let (a, b) = self.edge_segment(e);
            for f in self.faces_containing_segment(a, b) {
                let p = self.face_plane(f);
                if p.normal.dot(n).abs() > 1.0 - 1e-9 && plane.distance(p.origin).abs() < PTOL {
                    continue;
                }
                if !other_planes.iter().any(|q| q.normal.dot(p.normal).abs() > 1.0 - 1e-9 && q.distance(p.origin).abs() < PTOL) {
                    other_planes.push(p);
                }
            }
        }
        for p in other_planes {
            let s: Vec<VertexId> = seeds.iter().copied().filter(|v| p.distance(self.pos(*v)).abs() <= PTOL).collect();
            self.refresh_plane(p, &s, &[], &Classify::None, Orient::auto());
        }
        let orient = Orient { pref: Some(n), ground_first: ground_rule };
        let cls = Classify::Region { outer: pts.clone(), holes: Vec::new() };
        let out = self.refresh_plane(plane, &seeds, &[], &cls, orient);
        let pr = Projector::new(plane.origin, n);
        let poly: Vec<_> = pts.iter().map(|p| pr.to2(*p)).collect();
        out.into_iter()
            .filter(|r| self.faces.contains_key(r.face))
            .filter(|r| geom::point_in_polygon(pr.to2(r.sample), &poly, TOL) != geom::PointLoc::Outside)
            .max_by(|a, b| a.area.total_cmp(&b.area))
            .map(|r| r.face)
    }
}

/// Drop consecutive duplicate points (and the closing duplicate for closed loops).
pub(crate) fn clean_points(pts: &[DVec3], closed: bool) -> Vec<DVec3> {
    let mut out: Vec<DVec3> = Vec::with_capacity(pts.len());
    for p in pts {
        if out.last().is_none_or(|q| (*q - *p).length() >= TOL) {
            out.push(*p);
        }
    }
    if closed {
        while out.len() > 1 && (out[0] - *out.last().unwrap()).length() < TOL {
            out.pop();
        }
    }
    out
}
