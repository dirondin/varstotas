//! Lights in the viewport: markers (a sun-like dot, the spot's cone, the area's rectangle),
//! picking with the Select tool and the aim handle of the selected spot / area light (drag it
//! onto what the light should shine at). Move / Rotate move and turn a selected light
//! (`transform::Targets::light`); Entity Info edits it.

use super::OverlayView;
use super::common::px_at;
use egui::Pos2;
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::{Light, LightKind};
use varstotas_render::{Camera, MarkShape, Overlay3d, OverlayLine};

/// Pick radius of a light's marker and of the aim handle (px).
pub const PICK_PX: f32 = 10.0;
/// Length of the aim line (px on screen).
const AIM_PX: f64 = 70.0;
const LIGHT_ON: [f32; 4] = [0.95, 0.7, 0.1, 1.0];
const LIGHT_OFF: [f32; 4] = [0.6, 0.6, 0.6, 1.0];

fn screen(cam: &Camera, viewport: egui::Vec2, p: DVec3) -> Option<Pos2> {
    cam.world_to_screen(p, DVec2::new(viewport.x as f64, viewport.y as f64)).map(|s| Pos2::new(s.x as f32, s.y as f32))
}

/// Where the aim handle of a spot or an area light is (None for a point light).
pub fn aim_handle(l: &Light, cam: &Camera, viewport: egui::Vec2) -> Option<DVec3> {
    (l.kind != LightKind::Point).then(|| l.position + l.direction * AIM_PX * px_at(cam, viewport, l.position))
}

/// The light whose marker is under `pos` (the closest within `PICK_PX`).
pub fn pick_light(lights: &[Light], cam: &Camera, viewport: egui::Vec2, pos: Pos2) -> Option<usize> {
    lights
        .iter()
        .enumerate()
        .filter_map(|(i, l)| Some((i, screen(cam, viewport, l.position)?.distance(pos))))
        .filter(|(_, d)| *d <= PICK_PX)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// `pos` is on the aim handle of light `i`.
pub fn on_aim_handle(l: &Light, cam: &Camera, viewport: egui::Vec2, pos: Pos2) -> bool {
    aim_handle(l, cam, viewport).and_then(|h| screen(cam, viewport, h)).is_some_and(|s| s.distance(pos) <= PICK_PX)
}

type ScreenPt = Option<(i32, i32)>;

/// The light's marker and handles in viewport points (harness): marker, aim handle.
pub fn light_screen(l: &Light, cam: &Camera, viewport: egui::Vec2) -> (ScreenPt, ScreenPt) {
    let r = |p: Option<Pos2>| p.map(|p| (p.x.round() as i32, p.y.round() as i32));
    (r(screen(cam, viewport, l.position)), r(aim_handle(l, cam, viewport).and_then(|h| screen(cam, viewport, h))))
}

fn line(out: &mut Overlay3d, a: DVec3, b: DVec3, color: [f32; 4], width_px: f32, dashed: bool) {
    out.lines.push(OverlayLine { a: a.as_vec3().to_array(), b: b.as_vec3().to_array(), color, width_px, dashed, on_top: true });
}

/// Two unit vectors across `d`.
fn across(d: DVec3) -> (DVec3, DVec3) {
    let u = if d.z.abs() < 0.9 { d.cross(DVec3::Z) } else { d.cross(DVec3::X) }.normalize_or(DVec3::X);
    (u, d.cross(u).normalize_or(DVec3::Y))
}

/// One light: its marker, and the shape of its beam; the selected one with its aim handle.
pub fn draw_light(l: &Light, selected: bool, cam: &Camera, viewport: egui::Vec2, out: &mut Overlay3d) {
    let c = if selected {
        varstotas_render::HIGHLIGHT_SELECTED
    } else if l.enabled {
        LIGHT_ON
    } else {
        LIGHT_OFF
    };
    out.add_marker(l.position, MarkShape::Dot, if selected { 14.0 } else { 12.0 }, c, true);
    let px = px_at(cam, viewport, l.position);
    match l.kind {
        LightKind::Point => {
            // Short rays around the bulb.
            let r = 9.0 * px;
            for k in 0..8 {
                let a = k as f64 * std::f64::consts::FRAC_PI_4;
                let (u, v) = (cam.basis().right, cam.basis().up);
                let d = u * a.cos() + v * a.sin();
                line(out, l.position + d * r, l.position + d * r * 1.6, c, 1.2, false);
            }
        }
        LightKind::Spot => {
            // The cone: four lines and its rim at the aim length.
            let len = AIM_PX * 0.8 * px;
            let half = (l.spot_angle.clamp(1.0, 179.0) as f64 * 0.5).to_radians().min(1.4);
            let (u, v) = across(l.direction);
            let centre = l.position + l.direction * len;
            let rad = len * half.tan();
            let rim: Vec<DVec3> = (0..=24).map(|k| {
                let a = k as f64 / 24.0 * std::f64::consts::TAU;
                centre + (u * a.cos() + v * a.sin()) * rad
            }).collect();
            for w in rim.windows(2) {
                line(out, w[0], w[1], c, 1.0, false);
            }
            for k in 0..4 {
                line(out, l.position, rim[k * 6], c, 1.0, false);
            }
        }
        LightKind::Area => {
            // The emitting rectangle, facing `direction`.
            let (u, v) = across(l.direction);
            let (hw, hh) = (l.size[0] as f64 * 0.5, l.size[1] as f64 * 0.5);
            let p = |a: f64, b: f64| l.position + u * a + v * b;
            let pts = [p(-hw, -hh), p(hw, -hh), p(hw, hh), p(-hw, hh), p(-hw, -hh)];
            for w in pts.windows(2) {
                line(out, w[0], w[1], c, 1.5, false);
            }
        }
    }
    if let Some(h) = aim_handle(l, cam, viewport) {
        line(out, l.position, h, c, 1.5, !selected);
        if selected {
            out.add_marker(h, MarkShape::Square, 12.0, varstotas_render::HIGHLIGHT_SELECTED, true);
        }
    }
}

/// Every light of the model (selected one highlighted).
pub fn draw_lights(view: &OverlayView, out: &mut Overlay3d) {
    for (i, l) in view.doc.model.lights.iter().enumerate() {
        draw_light(l, view.doc.light_sel == Some(i), view.camera, view.viewport, out);
    }
}
