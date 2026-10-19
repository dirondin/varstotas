//! Camera math (f64, Z-up, right-handed, millimetres).
//!
//! Projection convention: **reverse-Z** (near plane -> depth 1, far -> depth 0),
//! perspective uses an **infinite** far plane, so depth precision is relative and
//! scenes from 1 mm to 10 km render without z-fighting. Depth compare in the renderer
//! is `GreaterEqual`, depth clear value is 0.

use glam::{DMat4, DQuat, DVec2, DVec3, DVec4};

/// Default iso direction (from target toward eye): red X goes right-down,
/// green Y goes up-right into the scene, blue Z up (start view).
const ISO_DIR: DVec3 = DVec3::new(0.55, -1.0, 0.72);
/// Orbit speed in radians per logical point of mouse drag.
const ORBIT_RAD_PER_PT: f64 = 0.006;
/// Max elevation (deg) of the view direction, avoids flipping over the pole.
const MAX_PITCH_DEG: f64 = 89.9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StandardView {
    Iso,
    Top,
    Bottom,
    Front,
    Back,
    Left,
    Right,
}

impl StandardView {
    pub const ALL: [StandardView; 7] =
        [StandardView::Iso, StandardView::Top, StandardView::Bottom, StandardView::Front, StandardView::Back, StandardView::Left, StandardView::Right];

    pub fn name(self) -> &'static str {
        match self {
            StandardView::Iso => "Iso",
            StandardView::Top => "Top",
            StandardView::Bottom => "Bottom",
            StandardView::Front => "Front",
            StandardView::Back => "Back",
            StandardView::Left => "Left",
            StandardView::Right => "Right",
        }
    }

    /// Direction from the target toward the eye, and the up direction.
    fn frame(self) -> (DVec3, DVec3) {
        match self {
            StandardView::Iso => {
                let d = ISO_DIR.normalize();
                let right = (-d).cross(DVec3::Z).normalize();
                (d, right.cross(-d))
            }
            StandardView::Top => (DVec3::Z, DVec3::Y),
            StandardView::Bottom => (-DVec3::Z, -DVec3::Y),
            StandardView::Front => (-DVec3::Y, DVec3::Z),
            StandardView::Back => (DVec3::Y, DVec3::Z),
            StandardView::Left => (-DVec3::X, DVec3::Z),
            StandardView::Right => (DVec3::X, DVec3::Z),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Camera {
    pub eye: DVec3,
    pub target: DVec3,
    /// Up hint (kept orthogonal to the view direction by the camera operations).
    pub up: DVec3,
    pub fov_y_deg: f64,
    /// Parallel projection; `ortho_height` is the visible world height in mm.
    pub ortho: bool,
    pub ortho_height: f64,
}

/// Orthonormal camera frame.
#[derive(Clone, Copy, Debug)]
pub struct CameraBasis {
    pub forward: DVec3,
    pub right: DVec3,
    pub up: DVec3,
}

impl Default for Camera {
    /// classic default: iso-ish view of the origin, ~5 m visible.
    fn default() -> Self {
        let fov_y_deg: f64 = 35.0;
        let visible_h = 5000.0;
        let dist = visible_h * 0.5 / (fov_y_deg.to_radians() * 0.5).tan();
        let target = DVec3::ZERO;
        let eye = target + ISO_DIR.normalize() * dist;
        let mut c = Camera {
            eye,
            target,
            up: DVec3::Z,
            fov_y_deg,
            ortho: false,
            ortho_height: visible_h,
        };
        c.up = c.basis().up;
        c
    }
}

impl Camera {
    /// Orthonormal frame (forward, right, up) consistent with [`Camera::view`].
    pub fn basis(&self) -> CameraBasis {
        let mut forward = self.target - self.eye;
        if forward.length_squared() < 1e-30 {
            forward = -DVec3::Z;
        }
        let forward = forward.normalize();
        let mut right = forward.cross(self.up);
        if right.length_squared() < 1e-20 {
            // up hint parallel to view direction: pick a fallback
            right = forward.cross(DVec3::Y);
            if right.length_squared() < 1e-20 {
                right = forward.cross(DVec3::X);
            }
        }
        let right = right.normalize();
        let up = right.cross(forward);
        CameraBasis { forward, right, up }
    }

    /// Distance eye→target.
    pub fn distance(&self) -> f64 {
        (self.target - self.eye).length()
    }

    fn tan_half_fov(&self) -> f64 {
        (self.fov_y_deg.to_radians() * 0.5).tan()
    }

    /// Near plane distance used for the perspective projection.
    pub fn near_plane(&self) -> f64 {
        (self.distance() * 1e-4).max(1e-4)
    }

    /// Depth range `(near, far)` along the view direction used for ortho projection.
    /// `near` may be negative (geometry behind the eye plane is still shown).
    pub fn ortho_depth_range(&self, scene_radius: f64) -> (f64, f64) {
        let b = self.basis();
        // Depth of the world origin, scene is a ball of `scene_radius` around it.
        let c = (-self.eye).dot(b.forward);
        let d = self.distance();
        let r = scene_radius
            .max(self.ortho_height * 2.0)
            .max(d)
            .max(1.0);
        let r = r + (self.target.length()).min(r * 4.0);
        let pad = r * 0.01 + 1.0;
        let near = (c - r - pad).min(d - r - pad);
        let far = (c + r + pad).max(d + r + pad);
        (near, far)
    }

    pub fn view(&self) -> DMat4 {
        let b = self.basis();
        // Right-handed look-at: camera looks down -Z in view space.
        let (f, r, u) = (b.forward, b.right, b.up);
        DMat4::from_cols(
            DVec4::new(r.x, u.x, -f.x, 0.0),
            DVec4::new(r.y, u.y, -f.y, 0.0),
            DVec4::new(r.z, u.z, -f.z, 0.0),
            DVec4::new(-r.dot(self.eye), -u.dot(self.eye), f.dot(self.eye), 1.0),
        )
    }

    /// Reverse-Z projection. Perspective: infinite far plane, near derived from the
    /// eye→target distance. Ortho: depth range derived from `scene_radius`.
    pub fn proj(&self, aspect: f64, scene_radius: f64) -> DMat4 {
        let aspect = if aspect.is_finite() && aspect > 0.0 { aspect } else { 1.0 };
        if self.ortho {
            let hh = self.ortho_height * 0.5;
            let hw = hh * aspect;
            let (n, f) = self.ortho_depth_range(scene_radius);
            // x' = x/hw, y' = y/hh, depth = (z + f) / (f - n)  (z = -n -> 1, z = -f -> 0)
            let rz = 1.0 / (f - n);
            DMat4::from_cols(
                DVec4::new(1.0 / hw, 0.0, 0.0, 0.0),
                DVec4::new(0.0, 1.0 / hh, 0.0, 0.0),
                DVec4::new(0.0, 0.0, rz, 0.0),
                DVec4::new(0.0, 0.0, f * rz, 1.0),
            )
        } else {
            let f = 1.0 / self.tan_half_fov();
            let near = self.near_plane();
            // clip.z = near, clip.w = -z  =>  depth = near / dist
            DMat4::from_cols(
                DVec4::new(f / aspect, 0.0, 0.0, 0.0),
                DVec4::new(0.0, f, 0.0, 0.0),
                DVec4::new(0.0, 0.0, 0.0, -1.0),
                DVec4::new(0.0, 0.0, near, 0.0),
            )
        }
    }

    /// World ray through a pixel. `px` in logical points, origin top-left.
    /// Returns `(origin, unit direction)`. For ortho the origin lies on the eye plane.
    pub fn screen_ray(&self, px: DVec2, viewport: DVec2) -> (DVec3, DVec3) {
        let b = self.basis();
        let (nx, ny, aspect) = ndc(px, viewport);
        if self.ortho {
            let hh = self.ortho_height * 0.5;
            let o = self.eye + b.right * (nx * hh * aspect) + b.up * (ny * hh);
            (o, b.forward)
        } else {
            let t = self.tan_half_fov();
            let d = b.forward + b.right * (nx * t * aspect) + b.up * (ny * t);
            (self.eye, d.normalize())
        }
    }

    /// Project world point to logical-point screen coords; None if behind camera.
    pub fn world_to_screen(&self, p: DVec3, viewport: DVec2) -> Option<DVec2> {
        let b = self.basis();
        let aspect = aspect_of(viewport);
        let v = p - self.eye;
        let (x, y) = if self.ortho {
            let hh = self.ortho_height * 0.5;
            (v.dot(b.right) / (hh * aspect), v.dot(b.up) / hh)
        } else {
            let z = v.dot(b.forward);
            if z <= self.near_plane() * 1e-3 {
                return None;
            }
            let t = self.tan_half_fov();
            (v.dot(b.right) / (z * t * aspect), v.dot(b.up) / (z * t))
        };
        Some(DVec2::new(
            (x + 1.0) * 0.5 * viewport.x,
            (1.0 - y) * 0.5 * viewport.y,
        ))
    }

    /// World size of one screen point at point `p` (for pick tolerances / constant-size gizmos).
    pub fn pixel_size_at(&self, p: DVec3, viewport: DVec2) -> f64 {
        let vh = viewport.y.max(1.0);
        if self.ortho {
            self.ortho_height / vh
        } else {
            let z = (p - self.eye).dot(self.basis().forward).abs().max(1e-9);
            2.0 * z * self.tan_half_fov() / vh
        }
    }

    /// Orbit around `pivot` by screen delta in points (feel: Z stays up).
    /// Horizontal drag = turntable around world Z through `pivot`; vertical drag tilts,
    /// clamped so the camera never flips over the poles.
    pub fn orbit(&mut self, delta: DVec2, pivot: DVec3) {
        let yaw = -delta.x * ORBIT_RAD_PER_PT;
        let rot_yaw = DQuat::from_rotation_z(yaw);
        self.apply_rotation(rot_yaw, pivot);

        let b = self.basis();
        let el = b.forward.z.clamp(-1.0, 1.0).asin();
        let max = MAX_PITCH_DEG.to_radians();
        // Mouse down (delta.y > 0) raises the camera => view direction tilts downward.
        let el_new = (el - delta.y * ORBIT_RAD_PER_PT).clamp(-max, max);
        let mut d_el = el_new - el;
        // If already beyond the clamp (e.g. exact Top view), only allow moving inward.
        if el.abs() > max {
            // inward = |el| decreases: el < 0 needs d_el > 0 (delta.y < 0) and vice versa
            d_el = if (el > 0.0 && delta.y > 0.0) || (el < 0.0 && delta.y < 0.0) {
                -delta.y * ORBIT_RAD_PER_PT
            } else {
                0.0
            };
        }
        if d_el != 0.0 {
            let rot_pitch = DQuat::from_axis_angle(b.right, d_el);
            self.apply_rotation(rot_pitch, pivot);
        }
        self.up = self.basis().up;
    }

    fn apply_rotation(&mut self, q: DQuat, pivot: DVec3) {
        self.eye = pivot + q * (self.eye - pivot);
        self.target = pivot + q * (self.target - pivot);
        self.up = q * self.up;
    }

    /// Pan so the world point under the cursor follows the cursor; `depth_point` is the
    /// point that was under the cursor when the drag began.
    /// `delta` is the cursor movement in logical points since the last call.
    pub fn pan(&mut self, delta: DVec2, depth_point: DVec3, viewport: DVec2) {
        let b = self.basis();
        let s = self.pixel_size_at(depth_point, viewport);
        let shift = b.right * (-delta.x * s) + b.up * (delta.y * s);
        self.eye += shift;
        self.target += shift;
    }

    /// Zoom toward world point `toward` (factor > 1 zooms in). `toward` stays at the same
    /// screen position (homothety centred at `toward`; ortho also scales `ortho_height`).
    pub fn zoom(&mut self, factor: f64, toward: DVec3) {
        if !(factor.is_finite() && factor > 0.0) {
            return;
        }
        let k = 1.0 / factor;
        let new_eye = toward + (self.eye - toward) * k;
        let new_target = toward + (self.target - toward) * k;
        if (new_target - new_eye).length() < 1e-6 {
            return;
        }
        self.eye = new_eye;
        self.target = new_target;
        if self.ortho {
            self.ortho_height = (self.ortho_height * k).max(1e-6);
        }
    }

    /// Fit the axis-aligned box into view (keeps the view direction).
    pub fn zoom_extents(&mut self, min: DVec3, max: DVec3, aspect: f64) {
        if !(min.cmple(max).all()) || !min.is_finite() || !max.is_finite() {
            return;
        }
        let aspect = if aspect.is_finite() && aspect > 0.0 { aspect } else { 1.0 };
        let margin = 1.1;
        let b = self.basis();
        let center = (min + max) * 0.5;
        let corners = (0..8).map(|i| {
            DVec3::new(
                if i & 1 == 0 { min.x } else { max.x },
                if i & 2 == 0 { min.y } else { max.y },
                if i & 4 == 0 { min.z } else { max.z },
            )
        });
        let ty = self.tan_half_fov();
        let tx = ty * aspect;
        let radius = ((max - min).length() * 0.5).max(1.0);
        if self.ortho {
            let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
            for p in corners {
                let v = p - center;
                let (x, y) = (v.dot(b.right), v.dot(b.up));
                x0 = x0.min(x);
                x1 = x1.max(x);
                y0 = y0.min(y);
                y1 = y1.max(y);
            }
            let h = ((y1 - y0).max((x1 - x0) / aspect) * margin).max(1.0);
            let c = center + b.right * ((x0 + x1) * 0.5) + b.up * ((y0 + y1) * 0.5);
            self.ortho_height = h;
            let dist = (h * 0.5 / ty).max(radius * 2.0);
            self.target = c;
            self.eye = c - b.forward * dist;
        } else {
            let (txm, tym) = (tx / margin, ty / margin);
            let mut d: f64 = 0.0;
            for p in corners {
                let v = p - center;
                let z = v.dot(b.forward);
                d = d.max(v.dot(b.right).abs() / txm - z);
                d = d.max(v.dot(b.up).abs() / tym - z);
            }
            let d = d.max(1.0);
            self.target = center;
            self.eye = center - b.forward * d;
            self.ortho_height = 2.0 * d * ty;
        }
        self.up = b.up;
    }

    /// Switch projection, keeping the visible size at the target equal.
    pub fn set_ortho(&mut self, ortho: bool) {
        if ortho == self.ortho {
            return;
        }
        let t = self.tan_half_fov();
        if ortho {
            self.ortho_height = 2.0 * self.distance() * t;
        } else {
            let b = self.basis();
            let d = (self.ortho_height * 0.5 / t).max(1e-3);
            self.eye = self.target - b.forward * d;
        }
        self.ortho = ortho;
    }

    /// Standard view; keeps target and distance.
    pub fn set_standard_view(&mut self, v: StandardView) {
        let (dir, up) = v.frame();
        let d = self.distance().max(1e-3);
        self.eye = self.target + dir * d;
        self.up = up;
    }

    /// The standard view the camera looks along (within a fraction of a degree), if any.
    pub fn standard_view(&self) -> Option<StandardView> {
        let dir = -self.basis().forward;
        StandardView::ALL.into_iter().find(|v| v.frame().0.normalize().dot(dir) > 0.99995)
    }
}

fn aspect_of(viewport: DVec2) -> f64 {
    if viewport.x > 0.0 && viewport.y > 0.0 {
        viewport.x / viewport.y
    } else {
        1.0
    }
}

fn ndc(px: DVec2, viewport: DVec2) -> (f64, f64, f64) {
    let vw = viewport.x.max(1e-9);
    let vh = viewport.y.max(1e-9);
    (2.0 * px.x / vw - 1.0, 1.0 - 2.0 * px.y / vh, aspect_of(viewport))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VP: DVec2 = DVec2::new(1200.0, 800.0);

    #[test]
    fn standard_view_is_recognised() {
        let mut c = Camera::default();
        assert_eq!(c.standard_view(), Some(StandardView::Iso));
        for v in StandardView::ALL {
            c.set_standard_view(v);
            assert_eq!(c.standard_view(), Some(v));
        }
        c.orbit(DVec2::new(40.0, 0.0), DVec3::ZERO);
        assert_eq!(c.standard_view(), None);
    }

    fn cams() -> Vec<Camera> {
        let mut v = vec![Camera::default()];
        let mut o = Camera::default();
        o.set_ortho(true);
        v.push(o);
        let mut t = Camera::default();
        t.set_standard_view(StandardView::Top);
        v.push(t.clone());
        t.set_ortho(true);
        v.push(t);
        let mut f = Camera::default();
        f.set_standard_view(StandardView::Front);
        f.orbit(DVec2::new(37.0, -12.0), DVec3::new(100.0, 50.0, 0.0));
        v.push(f);
        v
    }

    fn assert_close(a: DVec2, b: DVec2, tol: f64) {
        assert!((a - b).length() < tol, "{a:?} vs {b:?}");
    }

    #[test]
    fn default_view_axes_orientation() {
        let c = Camera::default();
        let o = c.world_to_screen(DVec3::ZERO, VP).unwrap();
        let x = c.world_to_screen(DVec3::X * 1000.0, VP).unwrap();
        let y = c.world_to_screen(DVec3::Y * 1000.0, VP).unwrap();
        let z = c.world_to_screen(DVec3::Z * 1000.0, VP).unwrap();
        // red right-down, green right-up, blue up
        assert!(x.x > o.x && x.y > o.y, "X {x:?} {o:?}");
        assert!(y.x > o.x && y.y < o.y, "Y {y:?} {o:?}");
        assert!(z.y < o.y && (z.x - o.x).abs() < 1.0);
        assert_close(o, VP * 0.5, 1e-6);
    }

    #[test]
    fn roundtrip_screen_ray_world_to_screen() {
        for c in cams() {
            for px in [
                DVec2::new(0.0, 0.0),
                DVec2::new(600.0, 400.0),
                DVec2::new(1100.0, 33.0),
                DVec2::new(17.0, 790.0),
            ] {
                let (o, d) = c.screen_ray(px, VP);
                assert!((d.length() - 1.0).abs() < 1e-9);
                for t in [10.0, 1234.0, 98765.0] {
                    let p = o + d * t;
                    let s = c.world_to_screen(p, VP).unwrap();
                    assert_close(s, px, 1e-6);
                }
            }
        }
    }

    #[test]
    fn matrices_match_world_to_screen() {
        for c in cams() {
            let vp = c.proj(VP.x / VP.y, 10_000.0) * c.view();
            for p in [
                DVec3::new(100.0, 200.0, 300.0),
                DVec3::new(-500.0, 20.0, 0.0),
                DVec3::new(1000.0, -1000.0, 50.0),
            ] {
                let clip = vp * p.extend(1.0);
                let ndc = clip.truncate() / clip.w;
                let s = DVec2::new((ndc.x + 1.0) * 0.5 * VP.x, (1.0 - ndc.y) * 0.5 * VP.y);
                assert_close(s, c.world_to_screen(p, VP).unwrap(), 1e-6);
                assert!(ndc.z > 0.0 && ndc.z < 1.0, "depth {} ortho {}", ndc.z, c.ortho);
            }
        }
    }

    #[test]
    fn reverse_z_ordering() {
        let c = Camera::default();
        let vp = c.proj(1.5, 1e7) * c.view();
        let b = c.basis();
        let mut last = 2.0;
        for d in [1.0, 10.0, 1e3, 1e5, 1e7, 1e9] {
            let clip = vp * (c.eye + b.forward * d).extend(1.0);
            let z = clip.z / clip.w;
            assert!(z < last && z > 0.0);
            last = z;
        }
    }

    #[test]
    fn behind_camera_is_none() {
        let c = Camera::default();
        let p = c.eye + (c.eye - c.target);
        assert!(c.world_to_screen(p, VP).is_none());
    }

    #[test]
    fn pan_keeps_point_under_cursor() {
        for mut c in cams() {
            let start = DVec2::new(400.0, 300.0);
            let (o, d) = c.screen_ray(start, VP);
            let p = o + d * 5000.0;
            let mut cur = start;
            for delta in [DVec2::new(10.0, -5.0), DVec2::new(-120.0, 33.0), DVec2::new(3.5, 70.0)] {
                c.pan(delta, p, VP);
                cur += delta;
                assert_close(c.world_to_screen(p, VP).unwrap(), cur, 1e-6);
            }
        }
    }

    #[test]
    fn zoom_keeps_point_under_cursor() {
        for mut c in cams() {
            let px = DVec2::new(900.0, 150.0);
            let (o, d) = c.screen_ray(px, VP);
            let p = o + d * 3000.0;
            let d0 = c.distance();
            c.zoom(1.25, p);
            assert_close(c.world_to_screen(p, VP).unwrap(), px, 1e-6);
            assert!(c.distance() < d0);
            c.zoom(0.5, p);
            assert_close(c.world_to_screen(p, VP).unwrap(), px, 1e-6);
        }
    }

    #[test]
    fn orbit_keeps_z_up_and_pivot() {
        let mut c = Camera::default();
        let pivot = DVec3::new(300.0, -200.0, 100.0);
        let dp0 = (c.eye - pivot).length();
        for _ in 0..200 {
            c.orbit(DVec2::new(13.0, 25.0), pivot);
            let b = c.basis();
            assert!(b.up.z >= -1e-9, "flipped: {:?}", b.up);
            assert!(b.forward.z.asin().to_degrees().abs() <= MAX_PITCH_DEG + 1e-6);
        }
        assert!(((c.eye - pivot).length() - dp0).abs() < 1e-6);
        // and back up
        for _ in 0..400 {
            c.orbit(DVec2::new(-7.0, -25.0), pivot);
            assert!(c.basis().up.z >= -1e-9);
        }
    }

    #[test]
    fn orbit_from_top_view_works() {
        let mut c = Camera::default();
        c.set_standard_view(StandardView::Top);
        let b = c.basis();
        assert!((b.forward + DVec3::Z).length() < 1e-12);
        assert!((b.up - DVec3::Y).length() < 1e-12);
        c.orbit(DVec2::new(10.0, -10.0), c.target);
        let b = c.basis();
        assert!(b.forward.z > -1.0 && b.forward.z < -0.9);
        assert!(b.up.z > 0.0);
    }

    #[test]
    fn standard_views() {
        let mut c = Camera::default();
        let d = c.distance();
        c.set_standard_view(StandardView::Front);
        let b = c.basis();
        assert!((b.forward - DVec3::Y).length() < 1e-12);
        assert!((b.right - DVec3::X).length() < 1e-12);
        assert!((c.distance() - d).abs() < 1e-6);
        c.set_standard_view(StandardView::Right);
        assert!((c.basis().forward + DVec3::X).length() < 1e-12);
        c.set_standard_view(StandardView::Iso);
        let def = Camera::default(); // same target (origin)
        assert_eq!(def.target, c.target);
        assert!((c.basis().forward - def.basis().forward).length() < 1e-12);
    }

    #[test]
    fn zoom_extents_fits_box() {
        for mut c in cams() {
            let (min, max) = (DVec3::new(1e5, 2e5, 0.0), DVec3::new(1.4e5, 2.1e5, 3e4));
            c.zoom_extents(min, max, VP.x / VP.y);
            let mut touches = false;
            for i in 0..8 {
                let p = DVec3::new(
                    if i & 1 == 0 { min.x } else { max.x },
                    if i & 2 == 0 { min.y } else { max.y },
                    if i & 4 == 0 { min.z } else { max.z },
                );
                let s = c.world_to_screen(p, VP).unwrap();
                assert!(s.x >= -1e-6 && s.x <= VP.x + 1e-6 && s.y >= -1e-6 && s.y <= VP.y + 1e-6);
                let m = s.x.min(VP.x - s.x).min(s.y).min(VP.y - s.y);
                if m < VP.y * 0.1 {
                    touches = true;
                }
            }
            assert!(touches, "fit is too loose");
        }
    }

    #[test]
    fn pixel_size_matches_projection() {
        for c in cams() {
            let p = c.target + DVec3::new(10.0, 20.0, 30.0);
            let s = c.pixel_size_at(p, VP);
            let b = c.basis();
            let a = c.world_to_screen(p, VP).unwrap();
            let q = c.world_to_screen(p + b.right * s * 10.0, VP).unwrap();
            assert!(((q - a).length() - 10.0).abs() < 1e-6);
        }
    }

    #[test]
    fn tiny_and_huge_scenes() {
        let mut c = Camera::default();
        c.zoom_extents(DVec3::ZERO, DVec3::splat(1.0), 1.0);
        assert!(c.near_plane() < 0.1);
        c.zoom_extents(DVec3::splat(-5e6), DVec3::splat(5e6), 1.0);
        let vp = c.proj(1.0, 1e7) * c.view();
        let clip = vp * DVec3::splat(5e6).extend(1.0);
        assert!(clip.w > 0.0 && clip.z / clip.w > 0.0 && clip.z / clip.w < 1.0);
    }
}
