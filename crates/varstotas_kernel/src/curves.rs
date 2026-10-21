//! Curves: arcs, circles and polygons drawn as chains of edges sharing an `Edge::curve` id.
//!
//! A curve created with [`Mesh::add_curve`] / [`Mesh::add_pie`] remembers how it was made
//! ([`CurveInfo`]: kind, centre, normal, radius, start/end angle, segment count) in
//! [`Mesh::curves`]. [`Mesh::curve_info`] returns that metadata only while the edges still form
//! the curve it describes (an arc/circle that was cut, stretched or partly erased has none).
//! [`Mesh::set_curve`] regenerates a curve with a new radius and/or segment count in place
//! (Entity Info), keeping its faces and the edges attached at arc ends.
//!
//! Angles are in radians about `normal`, measured from `xaxis` (`yaxis = normal × xaxis`);
//! arcs always run counter-clockwise about `normal` from `start_angle` to `end_angle`.

use crate::mesh::{Mesh, PTOL};
use crate::{EdgeId, FaceId, OpError, OpResult, Plane, TOL, VertexId};
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::f64::consts::TAU;
use varstotas_i18n::tr;

/// Upper bound for the segment count of one curve.
pub const MAX_CURVE_SEGMENTS: u32 = 999;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurveKind {
    /// Open arc (`segments` edges from start to end).
    Arc,
    /// Closed circle (`segments` edges).
    Circle,
    /// Closed regular polygon (`segments` sides); like a circle but not smoothed in classic modellers.
    Polygon,
}

/// How a curve was made (mesh-local coordinates).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CurveInfo {
    pub kind: CurveKind,
    pub center: DVec3,
    /// Unit normal of the curve plane.
    pub normal: DVec3,
    /// Unit in-plane direction of angle 0.
    pub xaxis: DVec3,
    pub radius: f64,
    /// Radians about `normal` from `xaxis`. For circles/polygons `end = start + TAU`.
    pub start_angle: f64,
    /// Radians; always `> start_angle` (arcs run counter-clockwise about `normal`).
    pub end_angle: f64,
    pub segments: u32,
}

impl CurveInfo {
    /// Unit in-plane direction of angle 90°.
    pub fn yaxis(&self) -> DVec3 {
        self.normal.cross(self.xaxis)
    }

    pub fn is_closed(&self) -> bool {
        self.kind != CurveKind::Arc
    }

    /// Swept angle (radians, > 0).
    pub fn sweep(&self) -> f64 {
        self.end_angle - self.start_angle
    }

    /// Point on the circle at angle `a`.
    pub fn point_at(&self, a: f64) -> DVec3 {
        self.center + (self.xaxis * a.cos() + self.yaxis() * a.sin()) * self.radius
    }

    /// Unit tangent at angle `a` in the direction of increasing angle.
    pub fn tangent_at(&self, a: f64) -> DVec3 {
        (self.yaxis() * a.cos() - self.xaxis * a.sin()).normalize_or(self.yaxis())
    }

    pub fn start_point(&self) -> DVec3 {
        self.point_at(self.start_angle)
    }

    pub fn end_point(&self) -> DVec3 {
        self.point_at(self.end_angle)
    }

    /// Unit tangent leaving the arc at its start (pointing away from the arc).
    pub fn start_tangent_out(&self) -> DVec3 {
        -self.tangent_at(self.start_angle)
    }

    /// Unit tangent leaving the arc at its end (pointing away from the arc).
    pub fn end_tangent_out(&self) -> DVec3 {
        self.tangent_at(self.end_angle)
    }

    /// Point halfway along the arc (the "bulge" apex of a 2-point arc).
    pub fn mid_point(&self) -> DVec3 {
        self.point_at((self.start_angle + self.end_angle) * 0.5)
    }

    /// Arc length of the true arc / circumference.
    pub fn length(&self) -> f64 {
        self.radius * self.sweep()
    }

    /// Sagitta (distance from the chord midpoint to the arc apex; signed positive).
    pub fn bulge(&self) -> f64 {
        self.radius * (1.0 - (self.sweep() * 0.5).cos())
    }

    /// The polyline: `segments + 1` points for an arc (start .. end), `segments` points for a
    /// closed curve (first point at `start_angle`).
    pub fn points(&self) -> Vec<DVec3> {
        let n = self.segments.max(1);
        if self.is_closed() {
            (0..n).map(|i| self.point_at(self.start_angle + TAU * i as f64 / n as f64)).collect()
        } else {
            let sw = self.sweep();
            let mut pts: Vec<DVec3> = (0..=n).map(|i| self.point_at(self.start_angle + sw * i as f64 / n as f64)).collect();
            // Exact end points (no trig noise at the ends that other geometry snaps to).
            pts[0] = self.start_point();
            pts[n as usize] = self.end_point();
            pts
        }
    }

    /// Circle through the centre `center` in the plane with normal `normal`, angle 0 towards
    /// `start` (projected into the plane; any vector works for direction only).
    pub fn circle(center: DVec3, normal: DVec3, start: DVec3, radius: f64, segments: u32) -> Option<CurveInfo> {
        Self::closed(CurveKind::Circle, center, normal, start, radius, segments)
    }

    /// Regular polygon with `segments` sides, first corner towards `start`.
    pub fn polygon(center: DVec3, normal: DVec3, start: DVec3, radius: f64, segments: u32) -> Option<CurveInfo> {
        Self::closed(CurveKind::Polygon, center, normal, start, radius, segments)
    }

    fn closed(kind: CurveKind, center: DVec3, normal: DVec3, start: DVec3, radius: f64, segments: u32) -> Option<CurveInfo> {
        let n = normal.try_normalize()?;
        let x = (start - n * start.dot(n)).normalize_or(crate::geom::plane_basis(n).0);
        if !radius.is_finite() || radius <= TOL || segments < 3 {
            return None;
        }
        Some(CurveInfo { kind, center, normal: n, xaxis: x, radius, start_angle: 0.0, end_angle: TAU, segments })
    }

    /// Arc around `center` starting at the point `start` (radius = its distance from the centre
    /// in the plane) sweeping `sweep` radians about `normal` (negative = clockwise; stored as a
    /// counter-clockwise arc about `-normal`). `|sweep|` is clamped to (0, TAU).
    /// "Arc" tool (centre, start point, end angle) and the arc of a Pie.
    pub fn arc_center(center: DVec3, normal: DVec3, start: DVec3, sweep: f64, segments: u32) -> Option<CurveInfo> {
        let mut n = normal.try_normalize()?;
        let d = start - center;
        let d = d - n * d.dot(n);
        let r = d.length();
        if r <= TOL || !sweep.is_finite() || sweep.abs() < 1e-9 || segments < 1 {
            return None;
        }
        if sweep < 0.0 {
            n = -n;
        }
        let sw = sweep.abs().min(TAU - 1e-9);
        Some(CurveInfo {
            kind: CurveKind::Arc,
            center,
            normal: n,
            xaxis: d / r,
            radius: r,
            start_angle: 0.0,
            end_angle: sw,
            segments,
        })
    }

    /// Arc through three points, from `a` via `through` to `b` ("3 Point Arc").
    /// `None` when the points are (nearly) collinear or coincident.
    pub fn arc_3pt(a: DVec3, through: DVec3, b: DVec3, segments: u32) -> Option<CurveInfo> {
        let n = (through - a).cross(b - through);
        if n.length() < 1e-9 * (through - a).length().max(1.0) * (b - through).length().max(1.0) || segments < 1 {
            return None;
        }
        let n = n.normalize();
        let center = circumcenter(a, through, b)?;
        let r = (a - center).length();
        if r <= TOL || (b - a).length() <= TOL {
            return None;
        }
        let x = (a - center) / r;
        let y = n.cross(x);
        let ang = |p: DVec3| {
            let d = p - center;
            d.dot(y).atan2(d.dot(x)).rem_euclid(TAU)
        };
        let end = ang(b);
        if end <= 1e-12 {
            return None;
        }
        Some(CurveInfo { kind: CurveKind::Arc, center, normal: n, xaxis: x, radius: r, start_angle: 0.0, end_angle: end, segments })
    }

    /// 2-point arc ("2 Point Arc"): from `a` to `b` with sagitta `bulge` measured along
    /// the in-plane direction `side` (perpendicular to the chord; it is projected). A negative
    /// bulge bends to `-side`. `None` for a zero bulge or chord.
    pub fn arc_2pt(a: DVec3, b: DVec3, side: DVec3, bulge: f64, segments: u32) -> Option<CurveInfo> {
        let chord = b - a;
        let c2 = chord.length_squared();
        if c2 < TOL * TOL || bulge.abs() < 1e-9 {
            return None;
        }
        let s = (side - chord * side.dot(chord) / c2).try_normalize()?;
        let apex = (a + b) * 0.5 + s * bulge;
        Self::arc_3pt(a, apex, b, segments)
    }

    /// Arc from `a` to `b` leaving `a` along `tangent` (tangent arc / "tangent at vertex").
    /// `None` if `tangent` is parallel to the chord.
    pub fn arc_tangent(a: DVec3, tangent: DVec3, b: DVec3, segments: u32) -> Option<CurveInfo> {
        let t = tangent.try_normalize()?;
        let chord = b - a;
        if chord.length() <= TOL || t.cross(chord / chord.length()).length() < 1e-6 || segments < 1 {
            return None;
        }
        let n = t.cross(chord).normalize();
        let u = n.cross(t);
        let s = chord.length_squared() / (2.0 * u.dot(chord));
        let center = a + u * s;
        let r = s.abs();
        let x = (a - center) / r;
        let y = n.cross(x);
        let d = b - center;
        let end = d.dot(y).atan2(d.dot(x)).rem_euclid(TAU);
        if end <= 1e-12 {
            return None;
        }
        Some(CurveInfo { kind: CurveKind::Arc, center, normal: n, xaxis: x, radius: r, start_angle: 0.0, end_angle: end, segments })
    }

    /// The curve after an affine transform, if it stays circular (a similarity in its plane:
    /// moves, rotations, uniform scales, mirrors). `None` for non-uniform scales.
    pub fn transformed(&self, m: &DMat4) -> Option<CurveInfo> {
        let x = m.transform_vector3(self.xaxis);
        let y = m.transform_vector3(self.yaxis());
        let (lx, ly) = (x.length(), y.length());
        if lx < 1e-12 || (lx - ly).abs() > 1e-9 * lx.max(1.0) || x.dot(y).abs() > 1e-9 * lx * ly {
            return None;
        }
        let (xn, yn) = (x / lx, y / ly);
        Some(CurveInfo {
            center: m.transform_point3(self.center),
            normal: xn.cross(yn).normalize(),
            xaxis: xn,
            radius: self.radius * lx,
            ..*self
        })
    }
}

/// Centre of the circle through three points (None if collinear).
pub fn circumcenter(a: DVec3, b: DVec3, c: DVec3) -> Option<DVec3> {
    let (ab, ac) = (b - a, c - a);
    let n = ab.cross(ac);
    let d = 2.0 * n.length_squared();
    if d < 1e-18 {
        return None;
    }
    Some(a + (n.cross(ab) * ac.length_squared() + ac.cross(n) * ab.length_squared()) / d)
}

impl Mesh {
    /// Edges of curve `cid`.
    pub fn curve_edges(&self, cid: u32) -> Vec<EdgeId> {
        self.edges.iter().filter(|(_, e)| e.curve == Some(cid)).map(|(id, _)| id).collect()
    }

    /// Vertices of curve `cid` in chain order and whether the chain is closed; `None` if the
    /// curve's edges do not form one simple chain (branching / disconnected / no edges).
    pub fn curve_chain(&self, cid: u32) -> Option<(Vec<VertexId>, bool)> {
        let mut adj: HashMap<VertexId, Vec<VertexId>> = HashMap::new();
        let mut n_edges = 0usize;
        for e in self.edges.values().filter(|e| e.curve == Some(cid)) {
            adj.entry(e.v[0]).or_default().push(e.v[1]);
            adj.entry(e.v[1]).or_default().push(e.v[0]);
            n_edges += 1;
        }
        if n_edges == 0 || adj.values().any(|l| l.len() > 2) {
            return None;
        }
        let mut ends: Vec<VertexId> = adj.iter().filter(|(_, l)| l.len() == 1).map(|(v, _)| *v).collect();
        ends.sort();
        let closed = ends.is_empty();
        if !closed && ends.len() != 2 {
            return None;
        }
        let start = if closed { *adj.keys().min()? } else { ends[0] };
        let mut chain = vec![start];
        let mut prev: Option<VertexId> = None;
        let mut cur = start;
        loop {
            let next = adj[&cur].iter().copied().find(|x| Some(*x) != prev && (*x != start || closed));
            let Some(nx) = next else { break };
            if nx == start {
                break;
            }
            if chain.len() > adj.len() {
                return None;
            }
            chain.push(nx);
            prev = Some(cur);
            cur = nx;
        }
        if chain.len() != adj.len() || (closed && chain.len() < 3) {
            return None;
        }
        Some((chain, closed))
    }

    /// Metadata of curve `cid` if its edges still form that arc / circle / polygon (on the
    /// circle, in the plane, not cut open; vertices added on its edges by splits are allowed).
    pub fn curve_info(&self, cid: u32) -> Option<CurveInfo> {
        let info = *self.curves.get(&cid)?;
        let (chain, closed) = self.curve_chain(cid)?;
        if closed != info.is_closed() {
            return None;
        }
        let tol = (TOL * 2.0).max(info.radius * 1e-9);
        let plane = Plane::new(info.center, info.normal);
        let k = chain.len();
        // Polygons: corners on the circle, split points on the sides (also covered by the
        // collinearity test below).
        for i in 0..k {
            let p = self.pos(chain[i]);
            if plane.distance(p).abs() > tol {
                return None;
            }
            let on_circle = ((plane.project(p) - info.center).length() - info.radius).abs() <= tol;
            if on_circle {
                continue;
            }
            let inner = closed || (i > 0 && i + 1 < k);
            if !inner {
                return None;
            }
            let (a, b) = (self.pos(chain[(i + k - 1) % k]), self.pos(chain[(i + 1) % k]));
            if crate::geom::point_seg_dist(p, a, b).0 > tol {
                return None;
            }
        }
        if !closed {
            let (s, e) = (self.pos(chain[0]), self.pos(chain[k - 1]));
            let (ps, pe) = (info.start_point(), info.end_point());
            let fwd = s.distance(ps) <= tol && e.distance(pe) <= tol;
            let bwd = s.distance(pe) <= tol && e.distance(ps) <= tol;
            if !fwd && !bwd {
                return None;
            }
        }
        Some(info)
    }

    /// Curve id and valid metadata of the curve that edge `e` belongs to.
    pub fn curve_of_edge(&self, e: EdgeId) -> Option<(u32, CurveInfo)> {
        let cid = self.edges.get(e)?.curve?;
        Some((cid, self.curve_info(cid)?))
    }

    /// Insert a curve (arc / circle / polygon) as sticky edges grouped into one curve, and
    /// remember its metadata. Faces are detected as for any closed planar loop. Returns the
    /// edge pieces (like `add_polyline`). The new curve id is `edges[e].curve` of any of them.
    pub fn add_curve(&mut self, info: &CurveInfo) -> Vec<EdgeId> {
        self.ensure();
        let cid = self.next_curve_id();
        let pts = info.points();
        let out = self.add_polyline_curve(&pts, info.is_closed(), Some(cid));
        if self.edges.values().any(|e| e.curve == Some(cid)) {
            self.curves.insert(cid, *info);
        }
        out
    }

    /// Pie ("Pie" tool): the arc `arc` plus two straight radii from its ends to its
    /// centre; the closed planar loop gets a face. Returns all edge pieces (arc first).
    pub fn add_pie(&mut self, arc: &CurveInfo) -> Vec<EdgeId> {
        let mut out = self.add_curve(arc);
        out.extend(self.add_line(arc.end_point(), arc.center));
        out.extend(self.add_line(arc.center, arc.start_point()));
        out.retain(|e| self.edges.contains_key(*e));
        // add_line may have split arc pieces.
        out
    }

    /// Unit direction for "tangent at vertex" inference at `v`: if `v` is an end of an arc
    /// (with metadata) the tangent leaving the arc there; otherwise, if exactly one edge meets
    /// `v`, the continuation of that edge (pointing away from it). Mesh-local coordinates.
    pub fn tangent_at_vertex(&self, v: VertexId) -> Option<DVec3> {
        let p = self.vertices.get(v)?.pos;
        let es = self.vertex_edges(v);
        for e in &es {
            let Some((cid, info)) = self.curve_of_edge(*e) else { continue };
            if info.kind != CurveKind::Arc {
                continue;
            }
            let Some((chain, _)) = self.curve_chain(cid) else { continue };
            if chain.first() != Some(&v) && chain.last() != Some(&v) {
                continue;
            }
            let tol = (TOL * 2.0).max(info.radius * 1e-9);
            if p.distance(info.start_point()) <= tol {
                return Some(info.start_tangent_out());
            }
            if p.distance(info.end_point()) <= tol {
                return Some(info.end_tangent_out());
            }
        }
        if es.len() == 1 {
            let o = self.pos(self.edges[es[0]].other(v));
            return (p - o).try_normalize();
        }
        None
    }

    /// Regenerate curve `cid` with a new `radius` and/or `segments` (Entity Info). The centre,
    /// plane and angles stay; arc ends move with the radius (edges attached there stretch).
    /// Faces bounded by the curve keep their ids and attributes. Returns the curve's edges.
    ///
    /// Errors (`Invalid`): no valid metadata; other edges attached to the curve's inner
    /// vertices; the curve bounds faces outside its plane (e.g. a push/pulled cylinder base);
    /// radius <= TOL; segments < 3 (closed) / < 1 (arc) or > [`MAX_CURVE_SEGMENTS`].
    pub fn set_curve(&mut self, cid: u32, radius: Option<f64>, segments: Option<u32>) -> OpResult<Vec<EdgeId>> {
        self.ensure();
        let old = self.curve_info(cid).ok_or(OpError::Invalid(tr!("kernel-err-not-an-arc-circle-or-polygon").into()))?;
        let mut new = old;
        if let Some(r) = radius {
            if !r.is_finite() || r <= TOL {
                return Err(OpError::Invalid(tr!("kernel-err-radius-must-be-positive").into()));
            }
            new.radius = r;
        }
        if let Some(s) = segments {
            let min = if old.is_closed() { 3 } else { 1 };
            if s < min || s > MAX_CURVE_SEGMENTS {
                return Err(OpError::Invalid(tr!("kernel-err-segments-must-be-min-max-curve", min = min.to_string(), MAX_CURVE_SEGMENTS = MAX_CURVE_SEGMENTS.to_string())));
            }
            new.segments = s;
        }
        let (mut chain, closed) = self.curve_chain(cid).ok_or(OpError::NotFound)?;
        if closed {
            let pts: Vec<DVec3> = chain.iter().map(|v| self.pos(*v)).collect();
            if crate::geom::newell(&pts).dot(old.normal) < 0.0 {
                chain.reverse();
            }
        } else {
            let (s, e) = (self.pos(chain[0]), self.pos(*chain.last().unwrap()));
            if s.distance(old.start_point()) > e.distance(old.start_point()) {
                chain.reverse();
            }
        }
        self.rebuild_curve(cid, chain, closed, &Plane::new(old.center, old.normal), &new.points())?;
        self.curves.insert(cid, new);
        Ok(self.curve_edges(cid))
    }

    /// Faces bounded by the edges of curve `cid`.
    pub(crate) fn curve_faces(&self, cid: u32) -> Vec<FaceId> {
        let mut faces: Vec<FaceId> = Vec::new();
        for e in self.curve_edges(cid) {
            for f in self.edge_faces(e) {
                if !faces.contains(&f) {
                    faces.push(f);
                }
            }
        }
        faces
    }

    /// Replace the vertices of curve `cid` (`chain`, in the order of `pts`; open: the ends
    /// are kept and moved to the ends of `pts`) by new ones at `pts`. Faces bounded by the
    /// curve keep their ids; faces attached at moved ends are made planar again.
    ///
    /// Errors (`Invalid`): other edges attached to the inner vertices; the curve bounds faces
    /// outside `plane`.
    pub(crate) fn rebuild_curve(&mut self, cid: u32, chain: Vec<VertexId>, closed: bool, plane: &Plane, pts: &[DVec3]) -> OpResult<()> {
        let edges = self.curve_edges(cid);
        let eset: HashSet<EdgeId> = edges.iter().copied().collect();
        let k = chain.len();
        if pts.len() < 2 || (closed && pts.len() < 3) {
            return Err(OpError::Invalid(tr!("kernel-err-too-few-points").into()));
        }
        let interior: Vec<VertexId> = if closed { chain.clone() } else { chain[1..k - 1].to_vec() };
        for v in &interior {
            let ves = self.vertex_edges(*v);
            if ves.len() != 2 || ves.iter().any(|e| !eset.contains(e)) {
                return Err(OpError::Invalid(tr!("kernel-err-other-edges-are-attached-to-the").into()));
            }
        }
        let faces = self.curve_faces(cid);
        for f in &faces {
            if self.faces[*f].loops().flatten().any(|v| plane.distance(self.pos(*v)).abs() > PTOL * 10.0) {
                return Err(OpError::Invalid(tr!("kernel-err-the-curve-bounds-faces-outside-its").into()));
            }
        }
        let tmpl = self.edges[edges[0]].clone();
        for f in &faces {
            self.f_unregister(*f);
        }
        for e in &edges {
            self.e_remove(*e);
        }
        for v in &interior {
            self.v_remove(*v);
        }
        let (first, last) = (chain[0], chain[k - 1]);
        let ends_moved = !closed && (self.pos(first).distance(pts[0]) > 1e-9 || self.pos(last).distance(*pts.last().unwrap()) > 1e-9);
        let new_chain: Vec<VertexId> = if closed {
            pts.iter().map(|p| self.v_new(*p)).collect()
        } else {
            if ends_moved {
                self.v_set_pos(first, pts[0]);
                self.v_set_pos(last, *pts.last().unwrap());
            }
            let mut c = vec![first];
            c.extend(pts[1..pts.len() - 1].iter().map(|p| self.v_new(*p)));
            c.push(last);
            c
        };
        for f in &faces {
            let face = &mut self.faces[*f];
            let loops = std::iter::once(&mut face.outer).chain(face.holes.iter_mut());
            for l in loops {
                replace_chain(l, &chain, &new_chain, closed);
            }
        }
        let n = new_chain.len();
        let pairs = if closed { n } else { n - 1 };
        for i in 0..pairs {
            let (a, b) = (new_chain[i], new_chain[(i + 1) % n]);
            match self.edge_between(a, b) {
                Some(e) => self.edges[e].curve = Some(cid),
                None => {
                    self.e_new(a, b, Some(&tmpl));
                }
            }
        }
        for f in &faces {
            self.f_register(*f);
        }
        if ends_moved {
            // Faces attached at the moved ends (outside the curve's own faces).
            let mut fix = Vec::new();
            for v in [first, last] {
                for f in self.faces_of_vertex(v) {
                    if !faces.contains(&f) && !fix.contains(&f) {
                        fix.push(f);
                    }
                }
            }
            let mut new_edges = Vec::new();
            for f in fix {
                if self.faces.contains_key(f) {
                    new_edges.extend(self.fix_face_planarity(f));
                }
            }
            self.merge_coplanar_at(&new_edges);
        }
        let mut weld: Vec<VertexId> = new_chain.into_iter().filter(|v| self.vertices.contains_key(*v)).collect();
        weld.sort();
        // The curve's faces too: a grown curve may now enclose (or overlap) other coplanar faces.
        self.absorb(&weld, &faces);
        Ok(())
    }

    /// Drop metadata of curves that have no edges left.
    pub fn prune_curves(&mut self) {
        if self.curves.is_empty() && self.splines.is_empty() {
            return;
        }
        let used: HashSet<u32> = self.edges.values().filter_map(|e| e.curve).collect();
        self.curves.retain(|k, _| used.contains(k));
        self.splines.retain(|k, _| used.contains(k));
    }

    /// After moving vertices `vs` by `m`: transform the metadata of curves that moved as a whole
    /// (drop it when the transform is not a similarity).
    pub(crate) fn transform_curve_infos(&mut self, vs: &HashSet<VertexId>, m: &DMat4) {
        if self.curves.is_empty() && self.splines.is_empty() {
            return;
        }
        let mut all_in: HashMap<u32, bool> = HashMap::new();
        for e in self.edges.values() {
            if let Some(c) = e.curve
                && (self.curves.contains_key(&c) || self.splines.contains_key(&c))
            {
                let inside = vs.contains(&e.v[0]) && vs.contains(&e.v[1]);
                let x = all_in.entry(c).or_insert(true);
                *x &= inside;
            }
        }
        for (c, inside) in all_in {
            if let Some(sp) = self.splines.get(&c).filter(|_| inside) {
                let t = sp.transformed(m);
                self.splines.insert(c, t);
                continue;
            }
            if inside {
                match self.curves[&c].transformed(m) {
                    Some(t) => {
                        self.curves.insert(c, t);
                    }
                    None => {
                        self.curves.remove(&c);
                    }
                }
            }
        }
    }

    /// After push/pull: the loop that did not keep a curve's edges (original or offset copy,
    /// `map`: loop vertex → its copy) gets a fresh curve id with the metadata translated, so
    /// both the base and the cap circle stay editable.
    pub(crate) fn copy_loop_curves(&mut self, loops: &[Vec<VertexId>], map: &HashMap<VertexId, VertexId>) {
        let mut groups: std::collections::BTreeMap<u32, Vec<(EdgeId, EdgeId, DVec3, DVec3)>> = Default::default();
        for l in loops {
            let k = l.len();
            for i in 0..k {
                let (a, b) = (l[i], l[(i + 1) % k]);
                let (Some(&a2), Some(&b2)) = (map.get(&a), map.get(&b)) else { continue };
                if a2 == a || b2 == b {
                    continue;
                }
                let (Some(e), Some(e2)) = (self.edge_between(a, b), self.edge_between(a2, b2)) else { continue };
                if let Some(cid) = self.edges[e].curve.or(self.edges[e2].curve) {
                    groups.entry(cid).or_default().push((e, e2, self.pos(a), self.pos(a2)));
                }
            }
        }
        for (cid, list) in groups {
            let info = self.curves.get(&cid).copied();
            let spline = self.splines.get(&cid).cloned();
            let (origin, normal) = match (&info, &spline) {
                (Some(i), _) => (i.center, i.normal),
                (None, Some(s)) => (s.plane().origin, s.normal),
                _ => continue,
            };
            let (_, _, p, p2) = list[0];
            // Which loop holds the stored metadata: the one in the curve's plane.
            let on_orig = (p - origin).dot(normal).abs() <= TOL;
            let (delta, targets): (DVec3, Vec<EdgeId>) = if on_orig {
                (p2 - p, list.iter().map(|x| x.1).collect())
            } else {
                (p - p2, list.iter().map(|x| x.0).collect())
            };
            let shift = DMat4::from_translation(delta);
            let next = self.next_curve_id();
            if let Some(info) = info {
                let Some(moved) = info.transformed(&shift) else { continue };
                for e in &targets {
                    self.edges[*e].curve = Some(next);
                }
                self.curves.insert(next, moved);
                if self.curve_info(next).is_none() {
                    self.curves.remove(&next);
                }
            } else if let Some(s) = spline {
                for e in &targets {
                    self.edges[*e].curve = Some(next);
                }
                self.splines.insert(next, s.transformed(&shift));
                if self.spline(next).is_none() {
                    self.splines.remove(&next);
                }
            }
            // The kept side must still be one valid curve under its old id.
            let kept: Vec<EdgeId> = list.iter().map(|x| if on_orig { x.0 } else { x.1 }).collect();
            for e in kept {
                self.edges[e].curve = Some(cid);
            }
        }
    }

    /// Curve ids used by `edges` mapped to fresh ids of this mesh, with metadata from `src`
    /// transformed by `m` (for copies: duplicate / explode); splines from `src_splines` likewise.
    pub(crate) fn remap_curves(
        &mut self,
        src: &std::collections::BTreeMap<u32, CurveInfo>,
        src_splines: &std::collections::BTreeMap<u32, crate::spline::Spline>,
        ids: impl IntoIterator<Item = u32>,
        m: &DMat4,
    ) -> HashMap<u32, u32> {
        let mut map = HashMap::new();
        let mut ids: Vec<u32> = ids.into_iter().collect();
        ids.sort();
        ids.dedup();
        for (next, c) in (self.next_curve_id()..).zip(ids) {
            map.insert(c, next);
            if let Some(t) = src.get(&c).and_then(|i| i.transformed(m)) {
                self.curves.insert(next, t);
            }
            if let Some(s) = src_splines.get(&c) {
                self.splines.insert(next, s.transformed(m));
            }
        }
        map
    }
}

/// Replace the run `old` (chain order) in loop `l` by `new` (same orientation as `old`).
/// Closed chains replace the whole loop.
fn replace_chain(l: &mut Vec<VertexId>, old: &[VertexId], new: &[VertexId], closed: bool) {
    let n = l.len();
    let k = old.len();
    let Some(i) = l.iter().position(|v| *v == old[0]) else { return };
    if closed {
        if n != k {
            return;
        }
        if l[(i + 1) % n] == old[1] {
            *l = new.to_vec();
        } else {
            *l = new.iter().rev().copied().collect();
        }
        return;
    }
    let forward = l[(i + 1) % n] == old[1];
    let backward = l[(i + n - 1) % n] == old[1];
    if !forward && !backward {
        return;
    }
    if backward {
        l.reverse();
        replace_chain(l, old, new, false);
        l.reverse();
        return;
    }
    // l[i] = old[0], l[i+1 ..= i+k-1] = old[1..], keep the rest.
    let mut out = Vec::with_capacity(n - k + new.len());
    out.extend_from_slice(new);
    for j in 0..n - k {
        out.push(l[(i + k + j) % n]);
    }
    *l = out;
}
