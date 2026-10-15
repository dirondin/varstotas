//! Position Camera / Look Around / Walk (first-person camera tools).
//!
//! The camera keeps a target in front of the eye (`LOOK_DIST` at least) so Orbit and Zoom keep
//! working after these tools.

use super::{Tool, ToolContext, ToolId, ToolRequest};
use crate::nav::dvec2;
use egui::{CursorIcon, Key, PointerButton, Pos2};
use varstotas_kernel::LengthUnit;
use varstotas_kernel::glam::{DQuat, DVec2, DVec3};
use varstotas_render::Camera;

/// Default eye height above the clicked point (5'6").
pub const EYE_HEIGHT: f64 = 1676.4;
const LOOK_DIST: f64 = 1000.0;
/// Walk: millimetres per dragged point, and turn per point.
const WALK_MM_PER_PT: f64 = 10.0;
const TURN_RAD_PER_PT: f64 = 0.004;
/// Walk: arrow key step and turn.
const STEP_MM: f64 = 300.0;
const STEP_TURN_DEG: f64 = 5.0;

fn parse_len(ctx: &ToolContext, input: &str) -> Option<f64> {
    super::common::parse_len(ctx.doc.model.unit, input).filter(|v| v.is_finite())
}

fn fmt_len(unit: LengthUnit, v: f64) -> String {
    varstotas_kernel::units::format_length(v, unit)
}

/// Horizontal view direction (the current one, or +Y when looking straight down/up).
fn level_dir(cam: &Camera) -> DVec3 {
    let f = cam.basis().forward;
    let h = DVec3::new(f.x, f.y, 0.0);
    if h.length_squared() > 1e-12 { h.normalize() } else { cam.basis().up.with_z(0.0).normalize_or(DVec3::Y) }
}

/// Put the eye at `eye` looking along `dir`, perspective, up = Z.
fn place(cam: &mut Camera, eye: DVec3, dir: DVec3) {
    let dir = dir.normalize_or(DVec3::Y);
    let d = cam.distance().clamp(LOOK_DIST, 10.0 * LOOK_DIST);
    cam.eye = eye;
    cam.target = eye + dir * d;
    cam.up = DVec3::Z;
    cam.ortho = false;
    cam.up = cam.basis().up;
}

/// Turn the view about the vertical axis through the eye.
fn turn(cam: &mut Camera, rad: f64) {
    let q = DQuat::from_rotation_z(rad);
    cam.target = cam.eye + q * (cam.target - cam.eye);
    cam.up = q * cam.up;
}

fn translate(cam: &mut Camera, d: DVec3) {
    cam.eye += d;
    cam.target += d;
}

// ---------------------------------------------------------------------------

/// Click a point to stand there (eye height above it), or drag from the standing point toward
/// the point to look at. Then switches to Look Around.
pub struct PositionCameraTool {
    height: f64,
    down: Option<(DVec3, Pos2)>,
    unit: LengthUnit,
}

impl Default for PositionCameraTool {
    fn default() -> Self {
        Self { height: EYE_HEIGHT, down: None, unit: LengthUnit::Millimeter }
    }
}

impl Tool for PositionCameraTool {
    fn id(&self) -> ToolId {
        ToolId::PositionCamera
    }
    fn activate(&mut self, ctx: &mut ToolContext) {
        self.unit = ctx.doc.model.unit;
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button == Some(PointerButton::Primary) {
            self.down = Some((ctx.point_under_pointer(), ctx.pointer.pos));
        }
    }
    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        let Some((p, at)) = self.down.take() else { return };
        let eye = p + DVec3::Z * self.height;
        let dir = if ctx.pointer.pos.distance(at) > 4.0 {
            // Dragged: look at the release point (on the ground level of the start point if the
            // point under the cursor is the eye itself).
            let q = ctx.point_under_pointer();
            if (q - eye).length() > 1.0 { q - eye } else { level_dir(ctx.camera) }
        } else {
            level_dir(ctx.camera)
        };
        place(ctx.camera, eye, dir);
        ctx.requests.push(ToolRequest::SwitchTool(ToolId::LookAround));
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if pressed && key == Key::Escape {
            self.down = None;
            ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select));
            return true;
        }
        false
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        match parse_len(ctx, input) {
            Some(v) => {
                self.height = v;
                true
            }
            None => false,
        }
    }
    fn vcb_label(&self) -> &str {
        "Height Offset"
    }
    fn vcb_value(&self) -> String {
        fmt_len(self.unit, self.height)
    }
    fn status_text(&self) -> String {
        "Click to stand at a point, or drag from where to stand toward where to look. Type the eye height.".into()
    }
    fn is_busy(&self) -> bool {
        self.down.is_some()
    }
}

// ---------------------------------------------------------------------------

/// Drag to turn the view around the eye. The VCB sets the eye height (above Z = 0).
#[derive(Default)]
pub struct LookAroundTool {
    dragging: bool,
    eye_z: f64,
    unit: Option<LengthUnit>,
}

impl Tool for LookAroundTool {
    fn id(&self) -> ToolId {
        ToolId::LookAround
    }
    fn activate(&mut self, ctx: &mut ToolContext) {
        self.eye_z = ctx.camera.eye.z;
        self.unit = Some(ctx.doc.model.unit);
        if ctx.camera.ortho {
            ctx.camera.set_ortho(false);
        }
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        self.dragging = ctx.pointer.button == Some(PointerButton::Primary);
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        if self.dragging && ctx.pointer.primary_down {
            let eye = ctx.camera.eye;
            ctx.camera.orbit(dvec2(ctx.pointer.delta), eye);
            ctx.camera.eye = eye;
        }
        self.eye_z = ctx.camera.eye.z;
    }
    fn on_pointer_up(&mut self, _ctx: &mut ToolContext) {
        self.dragging = false;
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if pressed && key == Key::Escape {
            ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select));
            return true;
        }
        false
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        match parse_len(ctx, input) {
            Some(v) => {
                let dz = v - ctx.camera.eye.z;
                translate(ctx.camera, DVec3::Z * dz);
                self.eye_z = v;
                true
            }
            None => false,
        }
    }
    fn vcb_label(&self) -> &str {
        "Eye Height"
    }
    fn vcb_value(&self) -> String {
        fmt_len(self.unit.unwrap_or(LengthUnit::Millimeter), self.eye_z)
    }
    fn status_text(&self) -> String {
        "Drag to look around. Type an eye height and press Enter.".into()
    }
    fn cursor(&self) -> CursorIcon {
        if self.dragging { CursorIcon::Grabbing } else { CursorIcon::Grab }
    }
    fn cursor_glyph(&self) -> Option<crate::overlay::Marker2d> {
        None
    }
}

// ---------------------------------------------------------------------------

/// Drag up/down to walk forward/back, left/right to turn; Shift: move up/down and sideways
/// instead; Ctrl: run. Arrow keys step. The eye height stays unless Shift is held.
#[derive(Default)]
pub struct WalkTool {
    dragging: bool,
    eye_z: f64,
    unit: Option<LengthUnit>,
}

impl WalkTool {
    fn walk(cam: &mut Camera, delta: DVec2, shift: bool, run: bool) {
        let k = if run { 4.0 } else { 1.0 } * WALK_MM_PER_PT;
        let fwd = level_dir(cam);
        if shift {
            let right = fwd.cross(DVec3::Z);
            translate(cam, right * delta.x * k - DVec3::Z * delta.y * k);
        } else {
            translate(cam, -fwd * delta.y * k);
            turn(cam, -delta.x * TURN_RAD_PER_PT);
        }
    }
}

impl Tool for WalkTool {
    fn id(&self) -> ToolId {
        ToolId::Walk
    }
    fn activate(&mut self, ctx: &mut ToolContext) {
        self.eye_z = ctx.camera.eye.z;
        self.unit = Some(ctx.doc.model.unit);
        if ctx.camera.ortho {
            ctx.camera.set_ortho(false);
        }
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        self.dragging = ctx.pointer.button == Some(PointerButton::Primary);
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        if self.dragging && ctx.pointer.primary_down {
            let m = ctx.pointer.modifiers;
            Self::walk(ctx.camera, dvec2(ctx.pointer.delta), m.shift, m.ctrl || m.command);
        }
        self.eye_z = ctx.camera.eye.z;
    }
    fn on_pointer_up(&mut self, _ctx: &mut ToolContext) {
        self.dragging = false;
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        let fwd = level_dir(ctx.camera);
        match key {
            Key::Escape => ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select)),
            Key::ArrowUp => translate(ctx.camera, fwd * STEP_MM),
            Key::ArrowDown => translate(ctx.camera, -fwd * STEP_MM),
            Key::ArrowLeft => turn(ctx.camera, STEP_TURN_DEG.to_radians()),
            Key::ArrowRight => turn(ctx.camera, -STEP_TURN_DEG.to_radians()),
            _ => return false,
        }
        self.eye_z = ctx.camera.eye.z;
        true
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        match parse_len(ctx, input) {
            Some(v) => {
                translate(ctx.camera, DVec3::Z * (v - ctx.camera.eye.z));
                self.eye_z = v;
                true
            }
            None => false,
        }
    }
    fn vcb_label(&self) -> &str {
        "Eye Height"
    }
    fn vcb_value(&self) -> String {
        fmt_len(self.unit.unwrap_or(LengthUnit::Millimeter), self.eye_z)
    }
    fn status_text(&self) -> String {
        "Drag up/down to walk, left/right to turn. Shift = up/down and sideways, Ctrl = run. Arrow keys step.".into()
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::Move
    }
    fn cursor_glyph(&self) -> Option<crate::overlay::Marker2d> {
        None
    }
}
