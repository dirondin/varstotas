//! Small geometric helpers shared by the kernel (public for reuse by the app).

use glam::{DVec2, DVec3};

/// Newell normal of a polygon (not normalized; length = 2 * area).
pub fn newell(pts: &[DVec3]) -> DVec3 {
    let n = pts.len();
    let mut acc = DVec3::ZERO;
    if n < 3 {
        return acc;
    }
    // Translate to the first point to reduce cancellation for far-away polygons.
    let o = pts[0];
    for i in 0..n {
        let a = pts[i] - o;
        let b = pts[(i + 1) % n] - o;
        acc.x += (a.y - b.y) * (a.z + b.z);
        acc.y += (a.z - b.z) * (a.x + b.x);
        acc.z += (a.x - b.x) * (a.y + b.y);
    }
    acc
}

/// Orthonormal basis `(u, v)` of the plane with normal `n`, such that `u × v = n`.
/// Horizontal planes get `u = ±X`, which keeps 2D projections intuitive.
pub fn plane_basis(n: DVec3) -> (DVec3, DVec3) {
    let n = n.normalize_or(DVec3::Z);
    let helper = if n.x.abs() < 0.9 { DVec3::X } else { DVec3::Y };
    let u = (helper - n * helper.dot(n)).normalize();
    let v = n.cross(u);
    (u, v)
}

/// 2D projection helper for one plane.
#[derive(Clone, Copy, Debug)]
pub struct Projector {
    pub origin: DVec3,
    pub u: DVec3,
    pub v: DVec3,
    pub n: DVec3,
}

impl Projector {
    pub fn new(origin: DVec3, normal: DVec3) -> Self {
        let n = normal.normalize_or(DVec3::Z);
        let (u, v) = plane_basis(n);
        Self { origin, u, v, n }
    }
    pub fn to2(&self, p: DVec3) -> DVec2 {
        let d = p - self.origin;
        DVec2::new(d.dot(self.u), d.dot(self.v))
    }
    pub fn to3(&self, p: DVec2) -> DVec3 {
        self.origin + self.u * p.x + self.v * p.y
    }
}

/// Signed area of a 2D polygon (positive = counter-clockwise).
pub fn signed_area(poly: &[DVec2]) -> f64 {
    let n = poly.len();
    if n < 3 {
        return 0.0;
    }
    let o = poly[0];
    let mut a = 0.0;
    for i in 1..n - 1 {
        a += (poly[i] - o).perp_dot(poly[i + 1] - o);
    }
    a * 0.5
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointLoc {
    Inside,
    On,
    Outside,
}

/// Distance from `p` to segment `ab` and the clamped parameter along it.
pub fn point_seg_dist2(p: DVec2, a: DVec2, b: DVec2) -> (f64, f64) {
    let ab = b - a;
    let l2 = ab.length_squared();
    let t = if l2 > 0.0 { ((p - a).dot(ab) / l2).clamp(0.0, 1.0) } else { 0.0 };
    ((a + ab * t - p).length(), t)
}

/// 3D distance from `p` to segment `ab` and the clamped parameter along it.
pub fn point_seg_dist(p: DVec3, a: DVec3, b: DVec3) -> (f64, f64) {
    let ab = b - a;
    let l2 = ab.length_squared();
    let t = if l2 > 0.0 { ((p - a).dot(ab) / l2).clamp(0.0, 1.0) } else { 0.0 };
    ((a + ab * t - p).length(), t)
}

/// Point-in-polygon (any orientation) with a boundary tolerance.
pub fn point_in_polygon(p: DVec2, poly: &[DVec2], tol: f64) -> PointLoc {
    let n = poly.len();
    if n < 3 {
        return PointLoc::Outside;
    }
    for i in 0..n {
        let (d, _) = point_seg_dist2(p, poly[i], poly[(i + 1) % n]);
        if d <= tol {
            return PointLoc::On;
        }
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (a, b) = (poly[i], poly[j]);
        if (a.y > p.y) != (b.y > p.y) {
            let x = a.x + (p.y - a.y) * (b.x - a.x) / (b.y - a.y);
            if p.x < x {
                inside = !inside;
            }
        }
        j = i;
    }
    if inside { PointLoc::Inside } else { PointLoc::Outside }
}

/// Point location relative to a polygon with holes ("On" for any boundary).
pub fn point_in_region(p: DVec2, outer: &[DVec2], holes: &[Vec<DVec2>], tol: f64) -> PointLoc {
    match point_in_polygon(p, outer, tol) {
        PointLoc::Outside => PointLoc::Outside,
        PointLoc::On => PointLoc::On,
        PointLoc::Inside => {
            for h in holes {
                match point_in_polygon(p, h, tol) {
                    PointLoc::Inside => return PointLoc::Outside,
                    PointLoc::On => return PointLoc::On,
                    PointLoc::Outside => {}
                }
            }
            PointLoc::Inside
        }
    }
}

/// Closest points between segments `p1q1` and `p2q2`. Returns `(s, t, c1, c2)` where
/// `c1 = p1 + s (q1 - p1)` and `c2 = p2 + t (q2 - p2)`; `None` if the segments are parallel.
pub fn seg_seg_closest(p1: DVec3, q1: DVec3, p2: DVec3, q2: DVec3) -> Option<(f64, f64, DVec3, DVec3)> {
    let d1 = q1 - p1;
    let d2 = q2 - p2;
    let r = p1 - p2;
    let a = d1.dot(d1);
    let e = d2.dot(d2);
    let f = d2.dot(r);
    if a < 1e-24 || e < 1e-24 {
        return None;
    }
    let c = d1.dot(r);
    let b = d1.dot(d2);
    let denom = a * e - b * b;
    if denom <= 1e-12 * a * e {
        return None;
    }
    let mut s = ((b * f - c * e) / denom).clamp(0.0, 1.0);
    let mut t = (b * s + f) / e;
    if t < 0.0 {
        t = 0.0;
        s = (-c / a).clamp(0.0, 1.0);
    } else if t > 1.0 {
        t = 1.0;
        s = ((b - c) / a).clamp(0.0, 1.0);
    }
    Some((s, t, p1 + d1 * s, p2 + d2 * t))
}

/// Do 2D segments `ab` and `cd` intersect (including touching)?
pub fn segs_intersect_2d(a: DVec2, b: DVec2, c: DVec2, d: DVec2, eps: f64) -> bool {
    let o1 = (b - a).perp_dot(c - a);
    let o2 = (b - a).perp_dot(d - a);
    let o3 = (d - c).perp_dot(a - c);
    let o4 = (d - c).perp_dot(b - c);
    let sgn = |x: f64| if x > eps { 1 } else if x < -eps { -1 } else { 0 };
    let (s1, s2, s3, s4) = (sgn(o1), sgn(o2), sgn(o3), sgn(o4));
    if s1 * s2 < 0 && s3 * s4 < 0 {
        return true;
    }
    let on = |p: DVec2, q: DVec2, r: DVec2| {
        r.x >= p.x.min(q.x) - eps && r.x <= p.x.max(q.x) + eps && r.y >= p.y.min(q.y) - eps && r.y <= p.y.max(q.y) + eps
    };
    (s1 == 0 && on(a, b, c)) || (s2 == 0 && on(a, b, d)) || (s3 == 0 && on(c, d, a)) || (s4 == 0 && on(c, d, b))
}

/// Intersection of two 2D lines `p + t d` and `q + s e`; `None` if parallel.
pub fn line_line_2d(p: DVec2, d: DVec2, q: DVec2, e: DVec2) -> Option<DVec2> {
    let den = d.perp_dot(e);
    if den.abs() < 1e-12 * d.length() * e.length() {
        return None;
    }
    let t = (q - p).perp_dot(e) / den;
    Some(p + d * t)
}

/// True if the two unit-ish directions are parallel (same or opposite).
pub fn parallel(a: DVec3, b: DVec3) -> bool {
    let (a, b) = (a.normalize_or_zero(), b.normalize_or_zero());
    a.cross(b).length() < 1e-9
}
