//! Bezier curves (Bezier Curve / Smooth Curve / Freehand tools): anchors with handles, cubic
//! spans. Like arcs, a spline is a chain of edges sharing an `Edge::curve` id; [`Mesh::splines`]
//! remembers its anchors so it can be reshaped later ([`Mesh::set_spline`]). The metadata
//! counts only while the edges still are that curve ([`Mesh::spline`]): once the chain is cut,
//! stretched or partly erased it is a plain chain of edges.

use crate::mesh::Mesh;
use crate::{EdgeId, OpError, OpResult, Plane, TOL, VertexId};
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};
use varstotas_i18n::tr;

/// One anchor of a curve; handles are absolute points (None = a corner on that side).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Anchor {
    pub p: DVec3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h_in: Option<DVec3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h_out: Option<DVec3>,
}

impl Anchor {
    pub fn corner(p: DVec3) -> Self {
        Self { p, h_in: None, h_out: None }
    }

    pub fn is_corner(&self) -> bool {
        self.h_in.is_none() && self.h_out.is_none()
    }

    fn transformed(&self, m: &DMat4) -> Self {
        let t = |p: DVec3| m.transform_point3(p);
        Self { p: t(self.p), h_in: self.h_in.map(t), h_out: self.h_out.map(t) }
    }
}

/// A curve through anchors (mesh-local coordinates).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spline {
    pub anchors: Vec<Anchor>,
    pub closed: bool,
    /// Edges per curved span (a span without handles is one edge).
    pub segs: u32,
    /// Unit normal of the plane the curve was drawn on (anchors are dragged in it).
    pub normal: DVec3,
}

impl Spline {
    /// The curve's points (see [`bezier_points`]).
    pub fn points(&self) -> Vec<DVec3> {
        bezier_points(&self.anchors, self.closed, self.segs)
    }

    /// The curve after an affine transform (Bezier curves map exactly).
    pub fn transformed(&self, m: &DMat4) -> Spline {
        let n = m.inverse().transpose().transform_vector3(self.normal);
        Spline {
            anchors: self.anchors.iter().map(|a| a.transformed(m)).collect(),
            normal: n.try_normalize().unwrap_or(self.normal),
            ..self.clone()
        }
    }

    /// Whether any span is curved.
    pub fn is_curved(&self) -> bool {
        self.anchors.iter().any(|a| !a.is_corner())
    }

    /// Length of the edge chain.
    pub fn length(&self) -> f64 {
        let pts = self.points();
        let n = pts.len();
        let k = if self.closed { n } else { n.saturating_sub(1) };
        (0..k).map(|i| pts[i].distance(pts[(i + 1) % n])).sum()
    }

    pub fn plane(&self) -> Plane {
        Plane::new(self.anchors.first().map_or(DVec3::ZERO, |a| a.p), self.normal)
    }
}

fn cubic(p0: DVec3, p1: DVec3, p2: DVec3, p3: DVec3, t: f64) -> DVec3 {
    let u = 1.0 - t;
    p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
}

/// The anchors with handles that pass smoothly through them (Catmull-Rom: each handle a
/// sixth of the chord between the neighbours); the ends of an open curve stay corners.
pub fn smooth(anchors: &[Anchor], closed: bool) -> Vec<Anchor> {
    let n = anchors.len();
    (0..n)
        .map(|i| {
            let p = anchors[i].p;
            let (prev, next) = if closed && n >= 3 {
                (anchors[(i + n - 1) % n].p, anchors[(i + 1) % n].p)
            } else if i == 0 || i + 1 == n {
                return Anchor::corner(p);
            } else {
                (anchors[i - 1].p, anchors[i + 1].p)
            };
            let t = (next - prev) / 6.0;
            Anchor { p, h_in: Some(p - t), h_out: Some(p + t) }
        })
        .collect()
}

/// Points of the curve through `anchors` (`segs` edges per curved span, one edge for a span
/// without handles); closed curves add the span from the last anchor back to the first and
/// do not repeat the first point.
pub fn bezier_points(anchors: &[Anchor], closed: bool, segs: u32) -> Vec<DVec3> {
    let Some(first) = anchors.first() else { return Vec::new() };
    let mut out = vec![first.p];
    let n = anchors.len();
    let spans = if closed && n >= 2 { n } else { n.saturating_sub(1) };
    for i in 0..spans {
        let (a, b) = (anchors[i], anchors[(i + 1) % n]);
        if a.h_out.is_none() && b.h_in.is_none() {
            out.push(b.p);
            continue;
        }
        let (p1, p2) = (a.h_out.unwrap_or(a.p), b.h_in.unwrap_or(b.p));
        for k in 1..=segs.max(1) {
            out.push(cubic(a.p, p1, p2, b.p, k as f64 / segs.max(1) as f64));
        }
    }
    if closed && out.len() > 2 {
        out.pop();
    }
    out
}

/// Whether the vertex positions `pos` (chain order, starting at the curve's first point) are
/// the curve `pts`, allowing extra vertices on its edges (splits).
fn chain_matches(pos: &[DVec3], pts: &[DVec3], closed: bool, tol: f64) -> bool {
    let mut target: Vec<DVec3> = Vec::with_capacity(pts.len() + 1);
    for p in pts.iter().chain(closed.then_some(&pts[0])) {
        if target.last().is_none_or(|q: &DVec3| q.distance(*p) > tol) {
            target.push(*p);
        }
    }
    let mut pos = pos.to_vec();
    if closed {
        pos.push(pos[0]);
    }
    if pos[0].distance(target[0]) > tol {
        return false;
    }
    let mut j = 0;
    for p in &pos[1..] {
        if j + 1 >= target.len() {
            return false;
        }
        if p.distance(target[j + 1]) <= tol {
            j += 1;
        } else if crate::geom::point_seg_dist(*p, target[j], target[j + 1]).0 > tol {
            return false;
        }
    }
    j + 1 == target.len()
}

impl Mesh {
    /// Insert a spline as sticky edges grouped into one curve and remember its anchors.
    /// Returns the edge pieces (like `add_polyline`).
    pub fn add_spline(&mut self, sp: &Spline) -> Vec<EdgeId> {
        self.ensure();
        let cid = self.next_curve_id();
        let out = self.add_polyline_curve(&sp.points(), sp.closed, Some(cid));
        if self.edges.values().any(|e| e.curve == Some(cid)) {
            self.splines.insert(cid, sp.clone());
        }
        out
    }

    /// The spline of curve `cid` and its vertices in the order of its points (the first
    /// vertex at the first anchor), if its edges still are that curve.
    pub fn spline_chain(&self, cid: u32) -> Option<(Vec<VertexId>, Spline)> {
        let sp = self.splines.get(&cid)?;
        let (mut chain, closed) = self.curve_chain(cid)?;
        if closed != sp.closed || sp.anchors.len() < 2 {
            return None;
        }
        let pts = sp.points();
        let tol = TOL * 2.0;
        if closed {
            let i = chain.iter().position(|v| self.pos(*v).distance(pts[0]) <= tol)?;
            chain.rotate_left(i);
        } else if self.pos(chain[0]).distance(pts[0]) > tol {
            chain.reverse();
        }
        let pos = |c: &[VertexId]| c.iter().map(|v| self.pos(*v)).collect::<Vec<_>>();
        if !chain_matches(&pos(&chain), &pts, closed, tol) {
            if !closed {
                return None;
            }
            chain[1..].reverse();
            if !chain_matches(&pos(&chain), &pts, closed, tol) {
                return None;
            }
        }
        Some((chain, sp.clone()))
    }

    /// The spline of curve `cid` while its edges still are that curve.
    pub fn spline(&self, cid: u32) -> Option<Spline> {
        self.spline_chain(cid).map(|x| x.1)
    }

    /// Curve id and valid spline of the curve that edge `e` belongs to.
    pub fn spline_of_edge(&self, e: EdgeId) -> Option<(u32, Spline)> {
        let cid = self.edges.get(e)?.curve?;
        Some((cid, self.spline(cid)?))
    }

    /// Reshape spline `cid` (moved anchors / handles, other segment count, anchors added or
    /// removed). Faces bounded by it keep their ids; edges attached at the ends of an open
    /// curve stretch with them. Returns the curve's edges.
    ///
    /// Errors (`Invalid`): no valid spline; fewer than 2 anchors (3 closed); a different
    /// `closed`; other edges attached to its inner vertices; it bounds faces outside its plane;
    /// a closed curve with a face that would cross itself.
    pub fn set_spline(&mut self, cid: u32, new: &Spline) -> OpResult<Vec<EdgeId>> {
        self.ensure();
        let (chain, old) = self.spline_chain(cid).ok_or(OpError::Invalid(tr!("kernel-err-not-a-curve-that-can-be").into()))?;
        let min = if old.closed { 3 } else { 2 };
        if new.closed != old.closed || new.anchors.len() < min {
            return Err(OpError::Invalid(tr!("kernel-err-too-few-points").into()));
        }
        if new.segs == 0 || new.segs > crate::curves::MAX_CURVE_SEGMENTS {
            return Err(OpError::Invalid(tr!("kernel-err-segments-must-be-max-curve-segments", MAX_CURVE_SEGMENTS = crate::curves::MAX_CURVE_SEGMENTS.to_string())));
        }
        let pts = new.points();
        if new.closed && !self.curve_faces(cid).is_empty() && !simple_loop(&pts, &new.plane()) {
            return Err(OpError::Invalid(tr!("kernel-err-the-curve-would-cross-itself").into()));
        }
        self.rebuild_curve(cid, chain, old.closed, &old.plane(), &pts)?;
        self.splines.insert(cid, new.clone());
        Ok(self.curve_edges(cid))
    }
}

/// Whether the closed polygon `pts` (in `plane`) does not cross itself.
fn simple_loop(pts: &[DVec3], plane: &Plane) -> bool {
    let pr = crate::geom::Projector::new(plane.origin, plane.normal);
    let p2: Vec<glam::DVec2> = pts.iter().map(|p| pr.to2(*p)).collect();
    let n = p2.len();
    for i in 0..n {
        for j in i + 2..n {
            if i == 0 && j == n - 1 {
                continue;
            }
            if crate::geom::segs_intersect_2d(p2[i], p2[(i + 1) % n], p2[j], p2[(j + 1) % n], 1e-9) {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_make_a_polyline() {
        let a = [Anchor::corner(DVec3::ZERO), Anchor::corner(DVec3::X), Anchor::corner(DVec3::Y)];
        assert_eq!(bezier_points(&a, false, 12), vec![DVec3::ZERO, DVec3::X, DVec3::Y]);
        assert_eq!(bezier_points(&a, true, 12).len(), 3);
    }

    #[test]
    fn handles_bend_the_span() {
        let a = [
            Anchor { p: DVec3::ZERO, h_in: None, h_out: Some(DVec3::new(0.0, 1.0, 0.0)) },
            Anchor { p: DVec3::new(2.0, 0.0, 0.0), h_in: Some(DVec3::new(2.0, 1.0, 0.0)), h_out: None },
        ];
        let pts = bezier_points(&a, false, 4);
        assert_eq!(pts.len(), 5);
        assert!(pts[0].distance(DVec3::ZERO) < 1e-12 && pts[4].distance(DVec3::new(2.0, 0.0, 0.0)) < 1e-12);
        // the middle of a symmetric span: x = 1, y = 3/4 of the handle height
        assert!(pts[2].distance(DVec3::new(1.0, 0.75, 0.0)) < 1e-12);
    }

    #[test]
    fn smooth_handles_follow_the_neighbours() {
        let a = [Anchor::corner(DVec3::ZERO), Anchor::corner(DVec3::new(1.0, 1.0, 0.0)), Anchor::corner(DVec3::new(2.0, 0.0, 0.0))];
        let s = smooth(&a, false);
        assert_eq!((s[0].h_out, s[2].h_in), (None, None));
        assert_eq!(s[1].h_out, Some(DVec3::new(1.0 + 1.0 / 3.0, 1.0, 0.0)));
        assert_eq!(s[1].h_in, Some(DVec3::new(1.0 - 1.0 / 3.0, 1.0, 0.0)));
        // closed: every anchor gets handles, and the curve passes through the anchors
        let c = smooth(&a, true);
        assert!(c.iter().all(|x| x.h_in.is_some() && x.h_out.is_some()));
        let pts = bezier_points(&c, true, 4);
        assert_eq!(pts.len(), 12);
        assert!(pts[4].distance(a[1].p) < 1e-12);
    }

    #[test]
    fn closed_curve_does_not_repeat_the_start() {
        let a = [
            Anchor { p: DVec3::ZERO, h_in: Some(DVec3::new(-1.0, 0.0, 0.0)), h_out: Some(DVec3::X) },
            Anchor::corner(DVec3::new(2.0, 2.0, 0.0)),
        ];
        let pts = bezier_points(&a, true, 6);
        assert_eq!(pts.len(), 12);
        assert!(pts.iter().skip(1).all(|p| p.distance(DVec3::ZERO) > 1e-9));
    }
}
