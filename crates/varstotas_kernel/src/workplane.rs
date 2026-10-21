//! Work planes (guide planes): named planes in the document, at most one active. Drawing
//! puts points "in the air" on the active plane, and the inference axes and the grid follow
//! its frame (red = `x`, green = `y`, blue = the normal).

use crate::Model;
use glam::{DAffine3, DMat3, DVec3};
use serde::{Deserialize, Serialize};

/// A plane through `origin` with orthonormal in-plane axes `x`, `y` (world space).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkPlane {
    pub name: String,
    pub origin: DVec3,
    pub x: DVec3,
    pub y: DVec3,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub hidden: bool,
}

/// Snap components within ~0.006° of an axis so planes built from model geometry stay
/// exactly axis-aligned.
fn clean(v: DVec3) -> DVec3 {
    let a = v.abs();
    for (i, axis) in [DVec3::X, DVec3::Y, DVec3::Z].into_iter().enumerate() {
        if a[i] > 1.0 - 1e-9 {
            return axis * v[i].signum();
        }
    }
    v
}

impl WorkPlane {
    /// Plane through `origin` with `normal`; red is `x_hint` projected onto the plane (or the
    /// world red / green axis projected when the hint is along the normal).
    pub fn from_normal(name: impl Into<String>, origin: DVec3, normal: DVec3, x_hint: Option<DVec3>) -> Option<Self> {
        let n = clean(normal.try_normalize()?);
        let proj = |v: DVec3| (v - n * v.dot(n)).try_normalize();
        let x = x_hint
            .and_then(proj)
            .or_else(|| proj(DVec3::X))
            .or_else(|| proj(DVec3::Y))?;
        let x = clean(x);
        let y = clean(n.cross(x).normalize());
        Some(Self { name: name.into(), origin, x, y, hidden: false })
    }

    /// Plane through three points: origin `a`, red toward `b`, normal by the right-hand rule.
    pub fn from_points(name: impl Into<String>, a: DVec3, b: DVec3, c: DVec3) -> Option<Self> {
        let n = (b - a).cross(c - a);
        if n.length() <= 1e-9 * (b - a).length().max(1.0) * (c - a).length().max(1.0) {
            return None;
        }
        Self::from_normal(name, a, n, Some(b - a))
    }

    pub fn normal(&self) -> DVec3 {
        self.x.cross(self.y)
    }

    /// Plane coordinates → world.
    pub fn frame(&self) -> DAffine3 {
        DAffine3::from_mat3_translation(DMat3::from_cols(self.x, self.y, self.normal()), self.origin)
    }

    /// (tilt, azimuth, spin) in degrees: the normal leans `tilt` from blue toward the
    /// direction `azimuth` (from red, counter-clockwise), then the plane turns by `spin`
    /// around its normal. `(0, 0, 0)` is the ground; with no tilt the azimuth is 0 and the
    /// turn is all spin.
    pub fn angles(&self) -> (f64, f64, f64) {
        let n = self.normal();
        let tilt = n.z.clamp(-1.0, 1.0).acos();
        let az = if tilt.sin().abs() < 1e-9 { 0.0 } else { n.y.atan2(n.x) };
        let (x0, y0, _) = Self::base(tilt, az);
        let spin = self.x.dot(y0).atan2(self.x.dot(x0));
        let deg = |a: f64| {
            let d = a.to_degrees();
            let r = d.round();
            let d = if (d - r).abs() < 1e-7 { r + 0.0 } else { d };
            // (-180, 180]
            if d <= -180.0 { d + 360.0 } else { d }
        };
        (deg(tilt), deg(az), deg(spin))
    }

    /// The ground tipped by `tilt` about the horizontal axis across the `az` direction (red
    /// stays level for walls: the front wall is `(90, -90, 0)`).
    fn base(tilt: f64, az: f64) -> (DVec3, DVec3, DVec3) {
        let r = DMat3::from_rotation_z(az) * DMat3::from_rotation_y(tilt) * DMat3::from_rotation_z(-az);
        (r * DVec3::X, r * DVec3::Y, r * DVec3::Z)
    }

    /// Set the orientation from angles in degrees (see [`WorkPlane::angles`]).
    pub fn set_angles(&mut self, tilt: f64, azimuth: f64, spin: f64) {
        let (x0, y0, _) = Self::base(tilt.to_radians(), azimuth.to_radians());
        let (s, c) = spin.to_radians().sin_cos();
        self.x = clean(x0 * c + y0 * s);
        self.y = clean(y0 * c - x0 * s);
    }

    /// Turn the plane about its normal by `deg` degrees.
    pub fn spin_by(&mut self, deg: f64) {
        let n = self.normal();
        let r = DMat3::from_axis_angle(n, deg.to_radians());
        self.x = clean(r * self.x);
        self.y = clean(n.cross(self.x).normalize());
    }

    /// Point the normal along `n`, keeping red as close to the old red as possible.
    pub fn set_normal(&mut self, n: DVec3) -> bool {
        match Self::from_normal("", self.origin, n, Some(self.x)) {
            Some(p) => {
                self.x = p.x;
                self.y = p.y;
                true
            }
            None => false,
        }
    }

    pub fn is_valid(&self) -> bool {
        let ok = |v: DVec3| v.is_finite() && (v.length() - 1.0).abs() < 1e-6;
        self.origin.is_finite() && ok(self.x) && ok(self.y) && self.x.dot(self.y).abs() < 1e-6
    }
}

impl Model {
    /// The active work plane, if any (and valid).
    pub fn active_work_plane(&self) -> Option<&WorkPlane> {
        self.work_planes.get(self.active_plane?).filter(|p| p.is_valid())
    }

    /// Add a plane (named "Plane N" when `name` is empty) and make it active; returns its index.
    pub fn add_work_plane(&mut self, mut p: WorkPlane) -> usize {
        if p.name.trim().is_empty() {
            let mut k = self.work_planes.len() + 1;
            // i18n-ignore-next
            while self.work_planes.iter().any(|q| q.name == format!("Plane {k}")) {
                k += 1;
            }
            // i18n-ignore-next
            p.name = format!("Plane {k}");
        }
        self.work_planes.push(p);
        self.active_plane = Some(self.work_planes.len() - 1);
        self.work_planes.len() - 1
    }

    pub fn remove_work_plane(&mut self, i: usize) {
        if i >= self.work_planes.len() {
            return;
        }
        self.work_planes.remove(i);
        self.active_plane = match self.active_plane {
            Some(a) if a == i => None,
            Some(a) if a > i => Some(a - 1),
            a => a,
        };
    }

    /// Move the drawing frame (Axes tool, Move Axes): the active work plane when there is one,
    /// else the drawing axes.
    pub fn set_drawing_frame(&mut self, origin: DVec3, x: DVec3, y: DVec3) -> crate::OpResult<()> {
        let Some(i) = self.active_plane.filter(|&i| i < self.work_planes.len()) else {
            return self.set_axes(origin, x, y);
        };
        let xa = x.try_normalize().ok_or(crate::OpError::Degenerate)?;
        let ya = (y - xa * y.dot(xa)).try_normalize().ok_or(crate::OpError::Degenerate)?;
        let p = &mut self.work_planes[i];
        p.origin = origin;
        p.x = clean(xa);
        p.y = clean(ya);
        Ok(())
    }

    /// The frame drawing tools use: the active work plane, else the drawing axes.
    pub fn drawing_frame(&self) -> DAffine3 {
        self.active_work_plane().map(|p| p.frame()).unwrap_or_else(|| self.axes_frame())
    }
}
