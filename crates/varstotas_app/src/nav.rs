//! Camera navigation: middle-drag orbit / shift+middle pan / wheel zoom toward cursor,
//! macOS trackpad gestures, smooth camera transitions and zoom extents.

use crate::document::guarded;
use egui::{Event, MouseWheelUnit, PointerButton, Pos2, Rect, Response, Vec2};
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::{Model, Ray};
use varstotas_render::{Camera, StandardView};

pub fn dvec2(v: Vec2) -> DVec2 {
    DVec2::new(v.x as f64, v.y as f64)
}

/// World ray through a viewport-local point.
pub fn ray_at(camera: &Camera, local: Pos2, viewport: Vec2) -> Ray {
    let (origin, dir) = camera.screen_ray(DVec2::new(local.x as f64, local.y as f64), dvec2(viewport));
    Ray { origin, dir }
}

/// Model point under the cursor; falls back to the ground plane, then to the target depth.
pub fn point_under(model: &Model, camera: &Camera, local: Pos2, viewport: Vec2) -> DVec3 {
    let ray = ray_at(camera, local, viewport);
    if let Ok(Some(hit)) = guarded(|| model.raycast(&ray)) {
        return hit.point;
    }
    let dist = (camera.eye - camera.target).length().max(1.0);
    if ray.dir.z.abs() > 1e-6 {
        let t = -ray.origin.z / ray.dir.z;
        if t > 0.0 && t < dist * 20.0 {
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
    pub fn handle(&mut self, response: &Response, rect: Rect, model: &Model, camera: &mut Camera) -> bool {
        let ctx = response.ctx.clone();
        let viewport = rect.size();
        let mut changed = false;
        let (shift, pointer) = ctx.input(|i| (i.modifiers.shift, i.pointer.hover_pos()));
        let local = pointer.map(|p| (p - rect.min).to_pos2());

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
            self.drag = Some(if shift { NavDrag::Pan } else { NavDrag::Orbit });
        }
        if self.drag.is_some() && response.dragged_by(PointerButton::Middle) {
            // Shift can be pressed/released mid-drag.
            let want = if shift { NavDrag::Pan } else { NavDrag::Orbit };
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
                    Some(NavDrag::Orbit) => camera.orbit(dvec2(d), self.pivot),
                    Some(NavDrag::Pan) => camera.pan(dvec2(d), self.pan_point, dvec2(viewport)),
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
                    Event::MouseWheel { unit, delta, modifiers, .. } => {
                        let Some(l) = local else { continue };
                        match unit {
                            MouseWheelUnit::Line | MouseWheelUnit::Page => {
                                // Classic mouse wheel: zoom toward cursor.
                                let steps = if unit == MouseWheelUnit::Page { delta.y * 3.0 } else { delta.y };
                                if steps != 0.0 {
                                    let target = point_under(model, camera, l, viewport);
                                    camera.zoom(1.18f64.powf(steps as f64), target);
                                    changed = true;
                                }
                            }
                            MouseWheelUnit::Point => {
                                if modifiers.command || modifiers.ctrl {
                                    let target = point_under(model, camera, l, viewport);
                                    camera.zoom((delta.y as f64 * 0.01).exp(), target);
                                    changed = true;
                                    continue;
                                }
                                let anchor = match self.scroll_anchor {
                                    Some((p, t)) if now - t < 0.25 => p,
                                    _ => point_under(model, camera, l, viewport),
                                };
                                self.scroll_anchor = Some((anchor, now));
                                if modifiers.shift {
                                    camera.pan(dvec2(delta), anchor, dvec2(viewport));
                                } else {
                                    camera.orbit(dvec2(delta), anchor);
                                }
                                changed = true;
                            }
                        }
                    }
                    Event::Zoom(f) if f != 1.0 => {
                        if let Some(l) = local {
                            let target = point_under(model, camera, l, viewport);
                            camera.zoom(f as f64, target);
                            changed = true;
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

/// Target camera for zoom extents (unchanged camera if the model is empty).
pub fn zoom_extents_camera(model: &Model, camera: &Camera, aspect: f64) -> Camera {
    let mut cam = camera.clone();
    if let Ok(Some(b)) = guarded(|| model.bounds()) {
        let _ = guarded(|| cam.zoom_extents(b.min, b.max, aspect));
    }
    cam
}

/// Target camera for a standard view, keeping the model framed.
pub fn standard_view_camera(model: &Model, camera: &Camera, v: StandardView, aspect: f64) -> Camera {
    let mut cam = camera.clone();
    cam.set_standard_view(v);
    zoom_extents_camera(model, &cam, aspect)
}
