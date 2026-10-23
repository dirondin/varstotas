//! Camera navigation: middle-drag orbit / shift+middle pan / wheel zoom toward cursor,
//! macOS trackpad gestures, smooth camera transitions and zoom extents.

use crate::document::guarded;
use egui::{Event, MouseWheelUnit, PointerButton, Pos2, Rect, Response, Vec2};
use varstotas_kernel::glam::{DQuat, DVec2, DVec3};
use varstotas_kernel::{Aabb, Model, Ray};
use varstotas_render::{Camera, StandardView};
use serde::{Deserialize, Serialize};

/// What a gesture does (the other one comes with Shift).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavGesture {
    #[default]
    Orbit,
    Pan,
    /// Trackpad only: two fingers zoom (Shift = pan).
    Zoom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TwistGesture {
    Disabled,
    #[default]
    Orbit,
    Roll,
}

/// Mouse and trackpad navigation (Preferences → Navigation).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NavPrefs {
    /// Orbit speed, × the standard one.
    pub orbit_speed: f64,
    /// Zoom speed of the wheel and ⌘-scroll, × the standard one.
    pub zoom_speed: f64,
    /// Wheel away from you zooms out (else in).
    pub invert_zoom: bool,
    /// Zoom toward the point under the cursor (else toward the middle of the view).
    pub zoom_to_cursor: bool,
    /// Middle-button drag: Orbit or Pan (Shift gives the other).
    pub middle: NavGesture,
    /// Two-finger scroll on a trackpad: Orbit, Pan or Zoom (Shift: Pan, or Orbit when it pans).
    pub two_finger: NavGesture,
    pub pan_speed: f64,
    pub trackpad_enabled: bool,
    pub trackpad_speed: f64,
    pub invert_swipe: bool,
    pub pinch_enabled: bool,
    pub pinch_speed: f64,
    pub invert_pinch: bool,
    pub twist: TwistGesture,
    pub twist_speed: f64,
    pub invert_twist: bool,
}

impl Default for NavPrefs {
    fn default() -> Self {
        Self { orbit_speed: 1.0, zoom_speed: 1.0, invert_zoom: false, zoom_to_cursor: true, middle: NavGesture::Orbit, two_finger: NavGesture::Orbit, pan_speed: 1.0,
            trackpad_enabled: true, trackpad_speed: 1.0, invert_swipe: false,
            pinch_enabled: true, pinch_speed: 1.0, invert_pinch: false,
            twist: TwistGesture::Orbit, twist_speed: 1.0, invert_twist: false }
    }
}

thread_local! {
    static NAV: std::cell::RefCell<NavPrefs> = std::cell::RefCell::new(NavPrefs::default());
}

/// Make `p` the settings the navigation tools (Orbit, Zoom, Look Around) read this frame.
pub fn publish(p: &NavPrefs) {
    NAV.with(|n| {
        if *n.borrow() != *p {
            *n.borrow_mut() = p.clone();
        }
    });
}

/// The published navigation settings.
pub fn prefs() -> NavPrefs {
    NAV.with(|n| n.borrow().clone())
}

impl NavPrefs {
    /// A zoom factor `f` (> 1 = in) with the speed and the inversion applied.
    pub fn zoom_factor(&self, f: f64) -> f64 {
        if !f.is_finite() || f <= 0.0 { return 1.0; }
        let k = f.ln() * self.zoom_speed.clamp(0.1, 10.0);
        (if self.invert_zoom { -k } else { k }).exp()
    }

    pub fn pinch_factor(&self, factor: f64) -> f64 {
        if !factor.is_finite() || factor <= 0.0 { return 1.0; }
        let k = factor.ln() * self.pinch_speed.clamp(0.1, 10.0);
        (if self.invert_pinch { -k } else { k }).exp()
    }

    pub fn swipe_delta(&self, delta: DVec2) -> DVec2 {
        delta * self.trackpad_speed.clamp(0.1, 10.0) * if self.invert_swipe { -1.0 } else { 1.0 }
    }

    pub fn apply_twist(&self, camera: &mut Camera, angle: f64, pivot: DVec3) -> bool {
        if !self.trackpad_enabled || self.twist == TwistGesture::Disabled || !angle.is_finite() || angle == 0.0 {
            return false;
        }
        let angle = angle * self.twist_speed.clamp(0.1, 10.0) * if self.invert_twist { 1.0 } else { -1.0 };
        let axis = if self.twist == TwistGesture::Orbit { DVec3::Z } else { camera.basis().forward };
        let rotation = DQuat::from_axis_angle(axis, angle);
        if self.twist == TwistGesture::Orbit {
            camera.eye = pivot + rotation * (camera.eye - pivot);
            camera.target = pivot + rotation * (camera.target - pivot);
        }
        camera.up = rotation * camera.up;
        true
    }

    pub fn orbit_delta(&self, d: DVec2) -> DVec2 {
        d * self.orbit_speed.clamp(0.1, 10.0)
    }
}

pub fn dvec2(v: Vec2) -> DVec2 {
    DVec2::new(v.x as f64, v.y as f64)
}

/// World ray through a viewport-local point.
pub fn ray_at(camera: &Camera, local: Pos2, viewport: Vec2) -> Ray {
    screen_ray(camera, DVec2::new(local.x as f64, local.y as f64), dvec2(viewport))
}

/// How far behind the eye plane parallel-projection pick rays start. The ortho renderer also
/// shows geometry behind the eye plane (zooming in moves the eye toward the model), so rays
/// must start well behind it or such geometry could not be picked / inferred.
pub fn ortho_ray_back(camera: &Camera) -> f64 {
    (camera.distance() + camera.ortho_height) * 4.0 + camera.eye.length() + camera.target.length() + 1.0e6
}

/// Pick ray through a screen point (logical points). For parallel projection the origin is
/// moved back along the view direction by [`ortho_ray_back`].
pub fn screen_ray(camera: &Camera, px: DVec2, viewport: DVec2) -> Ray {
    let (mut origin, dir) = camera.screen_ray(px, viewport);
    if camera.ortho {
        origin -= dir * ortho_ray_back(camera);
    }
    Ray { origin, dir }
}

/// Model point under the cursor; falls back to the ground plane, then to the target depth.
pub fn point_under(model: &Model, camera: &Camera, local: Pos2, viewport: Vec2) -> DVec3 {
    let ray = ray_at(camera, local, viewport);
    if let Ok(Some(hit)) = guarded(|| model.raycast(&ray)) {
        return hit.point;
    }
    let dist = (camera.eye - camera.target).length().max(1.0);
    let max_t = if camera.ortho { ortho_ray_back(camera) + dist * 20.0 } else { dist * 20.0 };
    if ray.dir.z.abs() > 1e-6 {
        let t = -ray.origin.z / ray.dir.z;
        if t > 0.0 && t < max_t {
            return ray.origin + ray.dir * t;
        }
    }
    if camera.ortho {
        // Project the target onto the ray.
        let t = (camera.target - ray.origin).dot(ray.dir);
        return ray.origin + ray.dir * t;
    }
    ray.origin + ray.dir * dist
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NavDrag {
    Orbit,
    Pan,
}

#[derive(Default)]
pub struct NavController {
    drag: Option<NavDrag>,
    pivot: DVec3,
    pan_point: DVec3,
    /// Pivot for trackpad orbit/pan gestures and the time it was last used.
    scroll_anchor: Option<(DVec3, f64)>,
}

impl NavController {
    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// Handle middle-button drags and wheel/trackpad gestures over the viewport.
    /// Returns true if the camera changed.
    pub fn handle(&mut self, response: &Response, rect: Rect, model: &Model, camera: &mut Camera, prefs: &NavPrefs) -> bool {
        let ctx = response.ctx.clone();
        let viewport = rect.size();
        let mut changed = false;
        let (shift, pointer) = ctx.input(|i| (i.modifiers.shift, i.pointer.hover_pos()));
        let local = pointer.map(|p| (p - rect.min).to_pos2());
        // the point a zoom goes toward
        let zoom_at = |l: Pos2, camera: &Camera| {
            let l = if prefs.zoom_to_cursor { l } else { (viewport * 0.5).to_pos2() };
            point_under(model, camera, l, viewport)
        };
        let middle = |shift: bool| match (prefs.middle == NavGesture::Pan) != shift {
            true => NavDrag::Pan,
            false => NavDrag::Orbit,
        };

        // ---- middle-button drag ------------------------------------------------
        if response.drag_started_by(PointerButton::Middle) {
            let press = ctx
                .input(|i| i.pointer.press_origin())
                .map(|p| (p - rect.min).to_pos2())
                .or(local)
                .unwrap_or(rect.size().to_pos2() * 0.5);
            let p = point_under(model, camera, press, viewport);
            self.pivot = p;
            self.pan_point = p;
            self.drag = Some(middle(shift));
        }
        if self.drag.is_some() && response.dragged_by(PointerButton::Middle) {
            // Shift can be pressed/released mid-drag.
            let want = middle(shift);
            if Some(want) != self.drag {
                if let Some(l) = local {
                    let p = point_under(model, camera, l, viewport);
                    self.pivot = p;
                    self.pan_point = p;
                }
                self.drag = Some(want);
            }
            let d = response.drag_delta();
            if d != Vec2::ZERO {
                match self.drag {
                    Some(NavDrag::Orbit) => camera.orbit(prefs.orbit_delta(dvec2(d)), self.pivot),
                    Some(NavDrag::Pan) => camera.pan(dvec2(d) * prefs.pan_speed.clamp(0.1, 10.0), self.pan_point, dvec2(viewport)),
                    None => {}
                }
                changed = true;
            }
        }
        if self.drag.is_some() && !ctx.input(|i| i.pointer.button_down(PointerButton::Middle)) {
            self.drag = None;
        }

        // ---- wheel / trackpad ----------------------------------------------------
        if response.hovered() || response.contains_pointer() {
            let (events, now) = ctx.input(|i| (i.events.clone(), i.time));
            for ev in events {
                match ev {
                    Event::MouseWheel { unit, delta, modifiers, phase } => {
                        let Some(l) = local else { continue };
                        match unit {
                            MouseWheelUnit::Line | MouseWheelUnit::Page => {
                                // Classic mouse wheel: zoom toward cursor.
                                let steps = if unit == MouseWheelUnit::Page { delta.y * 3.0 } else { delta.y };
                                if steps != 0.0 {
                                    let target = zoom_at(l, camera);
                                    camera.zoom(prefs.zoom_factor(1.18f64.powf(steps as f64)), target);
                                    changed = true;
                                }
                            }
                            MouseWheelUnit::Point => {
                                if !prefs.trackpad_enabled { continue; }
                                if matches!(phase, egui::TouchPhase::Start) { self.scroll_anchor = None; }
                                if matches!(phase, egui::TouchPhase::End | egui::TouchPhase::Cancel) && delta == Vec2::ZERO {
                                    self.scroll_anchor = None;
                                    continue;
                                }
                                let delta = if modifiers.command || modifiers.ctrl { dvec2(delta) } else { prefs.swipe_delta(dvec2(delta)) };
                                if !delta.is_finite() || delta == DVec2::ZERO { continue; }
                                let zoom = prefs.two_finger == NavGesture::Zoom && !modifiers.shift;
                                if modifiers.command || modifiers.ctrl || zoom {
                                    if (modifiers.command || modifiers.ctrl) && !prefs.pinch_enabled { continue; }
                                    let target = zoom_at(l, camera);
                                    let factor = (delta.y * 0.01).exp();
                                    camera.zoom(if modifiers.command || modifiers.ctrl { prefs.pinch_factor(factor) } else { factor }, target);
                                    changed = true;
                                    continue;
                                }
                                let anchor = match self.scroll_anchor {
                                    Some((p, t)) if now - t < 0.25 => p,
                                    _ => point_under(model, camera, l, viewport),
                                };
                                self.scroll_anchor = Some((anchor, now));
                                // Orbit (Shift: Pan), Pan (Shift: Orbit), Zoom (Shift: Pan)
                                let pan = match prefs.two_finger {
                                    NavGesture::Pan => !modifiers.shift,
                                    _ => modifiers.shift,
                                };
                                if pan {
                                    camera.pan(delta * prefs.pan_speed.clamp(0.1, 10.0), anchor, dvec2(viewport));
                                } else {
                                    camera.orbit(prefs.orbit_delta(delta), anchor);
                                }
                                changed = true;
                            }
                        }
                    }
                    Event::Zoom(f) if prefs.trackpad_enabled && prefs.pinch_enabled && f.is_finite() && f > 0.0 && f != 1.0 => {
                        if let Some(l) = local {
                            let target = zoom_at(l, camera);
                            camera.zoom(prefs.pinch_factor(f as f64), target);
                            changed = true;
                        }
                    }
                    Event::Rotate(angle) => {
                        if let Some(l) = local {
                            let anchor = match self.scroll_anchor {
                                Some((p, t)) if now - t < 0.25 => p,
                                _ => point_under(model, camera, l, viewport),
                            };
                            if prefs.apply_twist(camera, angle as f64, anchor) {
                                self.scroll_anchor = Some((anchor, now));
                                changed = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        changed
    }
}

/// Smooth camera transition (scenes, standard views).
pub struct CameraAnimation {
    from: Camera,
    to: Camera,
    t: f32,
    duration: f32,
}

impl CameraAnimation {
    pub fn new(from: Camera, to: Camera, duration: f32) -> Self {
        Self { from, to, t: 0.0, duration: duration.max(0.01) }
    }

    /// Advance and write the interpolated camera; returns false when finished.
    pub fn step(&mut self, dt: f32, camera: &mut Camera) -> bool {
        self.t = (self.t + dt / self.duration).min(1.0);
        let s = self.t as f64;
        let e = s * s * (3.0 - 2.0 * s); // smoothstep
        let (a, b) = (&self.from, &self.to);
        let target = a.target.lerp(b.target, e);
        let da = a.eye - a.target;
        let db = b.eye - b.target;
        let dist = da.length() + (db.length() - da.length()) * e;
        let dir = da.normalize_or_zero().lerp(db.normalize_or_zero(), e).normalize_or(db.normalize_or_zero());
        camera.target = target;
        camera.eye = target + dir * dist;
        camera.up = a.up.lerp(b.up, e).normalize_or(b.up);
        camera.fov_y_deg = a.fov_y_deg + (b.fov_y_deg - a.fov_y_deg) * e;
        camera.ortho_height = a.ortho_height + (b.ortho_height - a.ortho_height) * e;
        camera.ortho = if self.t >= 1.0 { b.ortho } else { a.ortho || b.ortho };
        if self.t >= 1.0 {
            *camera = b.clone();
            return false;
        }
        true
    }
}

/// Bounds used for zoom extents: model geometry plus guide points (all contexts) and dimensions. Guide lines are infinite and
/// therefore deliberately ignored.
pub fn extents_bounds(model: &Model) -> Option<Aabb> {
    let geom = guarded(|| model.bounds()).ok().flatten();
    // Guide points of every context (nested groups too) and dimensions.
    let mut extra: Vec<DVec3> = if model.guides_hidden {
        Vec::new()
    } else {
        guarded(|| model.guide_points_world()).unwrap_or_default()
    };
    extra.extend(guarded(|| model.dimension_points_world()).unwrap_or_default());
    let pts = Aabb::from_points(extra.into_iter().filter(|p| p.is_finite()));
    match (geom, pts) {
        (Some(a), Some(b)) => Some(a.union(&b)),
        (a, b) => a.or(b),
    }
}

/// Target camera for zoom extents (unchanged camera if the model is empty).
pub fn zoom_extents_camera(model: &Model, camera: &Camera, aspect: f64) -> Camera {
    let mut cam = camera.clone();
    if let Some(b) = extents_bounds(model) {
        let _ = guarded(|| cam.zoom_extents(b.min, b.max, aspect));
    }
    cam
}

/// classic startup view: perspective, looking slightly down across a ~10 m area in front
/// of the origin, with the horizon (sky) in the upper part of the view.
pub fn startup_camera() -> Camera {
    let mut cam = guarded(Camera::default).unwrap_or(Camera {
        eye: DVec3::ONE,
        target: DVec3::ZERO,
        up: DVec3::Z,
        fov_y_deg: 35.0,
        ortho: false,
        ortho_height: 5000.0,
    });
    // The standard Iso view (as View → Iso shows it), a few metres around the origin.
    cam.target = DVec3::new(2000.0, 2000.0, 400.0);
    cam.eye = cam.target + DVec3::new(0.0, -12000.0, 0.0);
    cam.ortho = false;
    cam.ortho_height = 7000.0;
    let _ = guarded(|| cam.set_standard_view(StandardView::Iso));
    cam
}

/// Target camera for a standard view, keeping the model framed.
pub fn standard_view_camera(model: &Model, camera: &Camera, v: StandardView, aspect: f64) -> Camera {
    let mut cam = camera.clone();
    cam.set_standard_view(v);
    zoom_extents_camera(model, &cam, aspect)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_prefs_scale_zoom_and_orbit() {
        let mut p = NavPrefs::default();
        assert!((p.zoom_factor(1.5) - 1.5).abs() < 1e-12);
        p.invert_zoom = true;
        assert!((p.zoom_factor(1.5) - 1.0 / 1.5).abs() < 1e-12);
        p.invert_zoom = false;
        p.zoom_speed = 2.0;
        assert!((p.zoom_factor(1.5) - 2.25).abs() < 1e-12);
        p.orbit_speed = 0.5;
        assert_eq!(p.orbit_delta(DVec2::new(10.0, -4.0)), DVec2::new(5.0, -2.0));
    }

    #[test]
    fn trackpad_defaults_migrate_and_pinch_is_independent_of_the_wheel() {
        let old: NavPrefs = serde_json::from_str(r#"{"invert_zoom":true,"two_finger":"Pan"}"#).unwrap();
        assert!(old.trackpad_enabled && old.pinch_enabled);
        assert_eq!(old.two_finger, NavGesture::Pan);
        assert_eq!(old.twist, TwistGesture::Orbit);
        assert_eq!(old.pinch_factor(2.0), 2.0);
        let mut p = old.clone();
        p.pinch_speed = 2.0;
        assert_eq!(p.pinch_factor(2.0), 4.0);
        p.invert_pinch = true;
        assert_eq!(p.pinch_factor(2.0), 0.25);
        assert_eq!(p.pinch_factor(f64::NAN), 1.0);
        assert_eq!(p.pinch_factor(0.0), 1.0);
        p.trackpad_speed = 2.0;
        p.invert_swipe = true;
        assert_eq!(p.swipe_delta(DVec2::new(10.0, -4.0)), DVec2::new(-20.0, 8.0));
        let saved = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<NavPrefs>(&saved).unwrap(), p);
    }

    #[test]
    fn twist_orbit_roll_inversion_and_disable_preserve_camera_geometry() {
        let mut camera = Camera::default();
        let original = camera.clone();
        let pivot = DVec3::new(100.0, 200.0, 300.0);
        let mut prefs = NavPrefs::default();
        assert!(prefs.apply_twist(&mut camera, 0.4, pivot));
        assert!((camera.eye.distance(pivot) - original.eye.distance(pivot)).abs() < 1e-6);
        assert!((camera.distance() - original.distance()).abs() < 1e-6);
        prefs.invert_twist = true;
        prefs.apply_twist(&mut camera, 0.4, pivot);
        assert!(camera.eye.distance(original.eye) < 1e-6);
        assert!(camera.up.distance(original.up) < 1e-6);
        prefs.twist = TwistGesture::Roll;
        prefs.apply_twist(&mut camera, 0.4, pivot);
        assert!(camera.eye.distance(original.eye) < 1e-6);
        assert!(camera.target.distance(original.target) < 1e-6);
        assert!(camera.up.distance(original.up) > 0.1);
        prefs.trackpad_enabled = false;
        let before = camera.clone();
        assert!(!prefs.apply_twist(&mut camera, 0.4, pivot));
        assert_eq!(camera.eye, before.eye);
        assert_eq!(camera.up, before.up);
    }

    fn model_with_face(z: f64) -> Model {
        let mut m = Model::new();
        m.root_mesh_mut().add_face(&[
            DVec3::new(-1000.0, -1000.0, z),
            DVec3::new(1000.0, -1000.0, z),
            DVec3::new(1000.0, 1000.0, z),
            DVec3::new(-1000.0, 1000.0, z),
        ]);
        m
    }

    #[test]
    fn ortho_pick_reaches_geometry_behind_the_eye_plane() {
        // Parallel top view whose eye sits *below* a face (possible after zooming in).
        let model = model_with_face(5000.0);
        let mut cam = Camera { target: DVec3::ZERO, ..Camera::default() };
        cam.set_standard_view(StandardView::Top);
        cam.set_ortho(true);
        cam.eye = DVec3::new(0.0, 0.0, 100.0);
        let vp = Vec2::new(800.0, 600.0);
        let ray = ray_at(&cam, Pos2::new(400.0, 300.0), vp);
        let hit = model.raycast(&ray).expect("face above the eye plane is pickable");
        assert!((hit.point.z - 5000.0).abs() < 1e-6);
        // Same screen position as the plain camera ray.
        let s = cam.world_to_screen(hit.point, dvec2(vp)).unwrap();
        assert!((s - DVec2::new(400.0, 300.0)).length() < 1e-6);
        // Empty space falls back to the ground plane.
        let p = point_under(&Model::new(), &cam, Pos2::new(100.0, 100.0), vp);
        assert!(p.z.abs() < 1e-6);
    }

    #[test]
    fn standard_views_keep_projection_mode() {
        let model = model_with_face(0.0);
        for ortho in [false, true] {
            let mut cam = startup_camera();
            cam.set_ortho(ortho);
            assert_eq!(cam.standard_view(), Some(StandardView::Iso));
            for v in [StandardView::Top, StandardView::Front, StandardView::Right, StandardView::Iso] {
                let c = standard_view_camera(&model, &cam, v, 1.5);
                assert_eq!(c.ortho, ortho);
                let s = c.world_to_screen(DVec3::ZERO, DVec2::new(1500.0, 1000.0)).unwrap();
                assert!(s.x > 0.0 && s.x < 1500.0 && s.y > 0.0 && s.y < 1000.0, "{v:?} ortho={ortho} {s:?}");
            }
        }
    }
}
