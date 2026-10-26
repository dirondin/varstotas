//! Guide Plane tool (work planes), plus drawing of the model's work planes.
//!
//! Click three points for a plane through them (origin, red toward the second point), or click
//! one point and then: double-click / Enter = the plane of the face under it (else the current
//! red-green plane), arrow keys = normal along red / green / blue / the last hovered edge. The
//! new plane becomes active: drawing tools put points in the air on it, and the inference axes
//! and the grid follow its frame. With the tool idle the active plane shows a gizmo: drag the
//! origin, the normal arrow tip (tilt) or the red handle (spin); each drag is one undo step.

use varstotas_i18n::tr;
use super::common::*;
use super::draw::Clicker;
use super::{OverlayView, Tool, ToolContext, ToolId, ToolRequest};
use crate::inference::{self, InferenceEngine, InferenceKind, InferenceResult};
use crate::overlay::{Marker2d, MarkerShape};
use egui::{Color32, CursorIcon, Key, Pos2};
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::{Plane, WorkPlane, TOL};
use varstotas_render::{HAlign, Overlay3d, OverlayPolygon, OverlayText, VAlign};

const ACTIVE: Color32 = Color32::from_rgb(0xE8, 0x7A, 0x1E);
const INACTIVE: Color32 = Color32::from_rgb(0x8C, 0x8C, 0x96);
/// Half size of a drawn plane, in screen px at its origin.
const PLANE_PX: f64 = 150.0;
/// Length of the gizmo's normal arrow and spin handle, in screen px.
const GIZMO_PX: f64 = 90.0;
/// Pick radius of the gizmo handles (px).
const HANDLE_PX: f32 = 9.0;

fn rgba(c: Color32, a: f32) -> [f32; 4] {
    [c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, a]
}

/// One plane: a translucent square around the origin (sized on screen), its border and short
/// red / green ticks; the active one in orange with its name.
pub fn draw_plane(p: &WorkPlane, active: bool, half: f64, out: &mut Overlay3d) {
    let (o, x, y) = (p.origin, p.x * half, p.y * half);
    let c = [o - x - y, o + x - y, o + x + y, o - x + y];
    let color = if active { ACTIVE } else { INACTIVE };
    let w = if active { 1.5 } else { 1.0 };
    for i in 0..4 {
        line(out, c[i], c[(i + 1) % 4], color, w);
    }
    out.polygons.push(OverlayPolygon {
        tris: [c[0], c[1], c[2], c[0], c[2], c[3]].map(|p| p.as_vec3().to_array()).to_vec(),
        color: rgba(color, if active { 0.10 } else { 0.06 }),
        on_top: false,
    });
    line(out, o, o + p.x * half * 0.25, InferenceKind::OnRedAxis.color(), w);
    line(out, o, o + p.y * half * 0.25, InferenceKind::OnGreenAxis.color(), w);
    let mut t = OverlayText::new(c[3], p.name.clone());
    t.color = rgba(color, 1.0);
    t.h_align = HAlign::Left;
    t.v_align = VAlign::Bottom;
    t.offset_pt = [4.0, -2.0];
    out.texts.push(t);
}

/// All visible work planes.
pub fn draw_work_planes(view: &OverlayView, out: &mut Overlay3d) {
    let m = &view.doc.model;
    for (i, p) in m.work_planes.iter().enumerate() {
        if p.hidden || !p.is_valid() {
            continue;
        }
        let half = px_at(view.camera, view.viewport, p.origin) * PLANE_PX;
        draw_plane(p, m.active_plane == Some(i), half, out);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Handle {
    Origin,
    Normal,
    Spin,
}

#[derive(Clone, Debug)]
enum Stage {
    Idle,
    A { a: DVec3, face_n: Option<DVec3> },
    B { a: DVec3, b: DVec3 },
    /// Dragging a gizmo handle of the active plane (`orig` = the plane before the drag).
    Drag { h: Handle, orig: WorkPlane },
}

pub struct GuidePlaneTool {
    engine: InferenceEngine,
    stage: Stage,
    cur: Option<InferenceResult>,
    clicker: Clicker,
    hover: Option<Handle>,
}

impl Default for GuidePlaneTool {
    fn default() -> Self {
        Self { engine: InferenceEngine::default(), stage: Stage::Idle, cur: None, clicker: Clicker::default(), hover: None }
    }
}

/// Handle positions of the active plane's gizmo.
fn handles(p: &WorkPlane, len: f64) -> [(Handle, DVec3); 3] {
    [(Handle::Origin, p.origin), (Handle::Normal, p.origin + p.normal() * len), (Handle::Spin, p.origin + p.x * len)]
}

/// The active plane's gizmo handles (origin, normal tip, spin) in viewport points (harness).
pub fn gizmo_screen(doc: &crate::document::Document, cam: &varstotas_render::Camera, viewport: egui::Vec2) -> Vec<(i32, i32)> {
    let Some(p) = doc.model.active_work_plane().filter(|p| !p.hidden) else { return Vec::new() };
    let len = px_at(cam, viewport, p.origin) * GIZMO_PX;
    let vp = DVec2::new(viewport.x as f64, viewport.y as f64);
    handles(p, len)
        .iter()
        .filter_map(|(_, at)| cam.world_to_screen(*at, vp).map(|s| (s.x.round() as i32, s.y.round() as i32)))
        .collect()
}

impl GuidePlaneTool {
    fn gizmo_len(ctx_cam: &varstotas_render::Camera, viewport: egui::Vec2, p: &WorkPlane) -> f64 {
        px_at(ctx_cam, viewport, p.origin) * GIZMO_PX
    }

    fn handle_at(&self, ctx: &ToolContext, pos: Pos2) -> Option<Handle> {
        let p = ctx.doc.model.active_work_plane().filter(|p| !p.hidden)?;
        let len = Self::gizmo_len(ctx.camera, ctx.viewport, p);
        let mut best: Option<(f32, Handle)> = None;
        for (h, at) in handles(p, len) {
            let Some(s) = ctx.to_screen(at) else { continue };
            let d = (s - pos).length();
            if d <= HANDLE_PX && best.is_none_or(|b| d < b.0) {
                best = Some((d, h));
            }
        }
        best.map(|b| b.1)
    }

    fn update(&mut self, ctx: &ToolContext) {
        let ax = crate::axes::current();
        let (anchor, fallback) = match &self.stage {
            Stage::Idle => (None, None),
            Stage::A { a, .. } => (Some(*a), Some(Plane::new(*a, ax.z))),
            Stage::B { a, b } => {
                // Third point: on the plane through a-b facing the current blue (or green).
                let d = (*b - *a).normalize_or_zero();
                let n = if d.cross(ax.z).length() > 1e-6 { ax.z - d * ax.z.dot(d) } else { ax.y };
                (Some(*a), Some(Plane::new(*a, n.normalize())))
            }
            Stage::Drag { h, orig } => match h {
                Handle::Origin => (Some(orig.origin), Some(Plane::new(orig.origin, orig.normal()))),
                Handle::Spin => (Some(orig.origin), Some(Plane::new(orig.origin, orig.normal()))),
                Handle::Normal => {
                    // The tip moves on a plane through the origin facing the camera.
                    let f = (ctx.camera.target - ctx.camera.eye).normalize_or(DVec3::Y);
                    (Some(orig.origin), Some(Plane::new(orig.origin, f)))
                }
            },
        };
        self.cur = Some(infer(&mut self.engine, ctx, anchor, None, fallback));
    }

    /// The plane the tool would create now (preview).
    fn preview(&self) -> Option<WorkPlane> {
        let p = self.cur.as_ref()?.point;
        match &self.stage {
            Stage::A { a, .. } => {
                let ax = crate::axes::current();
                let d = p - *a;
                if d.length() <= TOL || d.cross(ax.z).length() <= 1e-6 * d.length() {
                    return None;
                }
                // Through a-p, as upright as possible: contains the current blue.
                WorkPlane::from_normal("", *a, d.cross(ax.z), Some(d))
            }
            Stage::B { a, b } => WorkPlane::from_points("", *a, *b, p),
            _ => None,
        }
    }

    fn create(&mut self, ctx: &mut ToolContext, p: WorkPlane) {
        if ctx.commit(&tr!("tool-undo-add-guide-plane"), |m| m.add_work_plane(p)).is_some() {
            crate::axes::sync(&ctx.doc.model);
        }
        self.stage = Stage::Idle;
        self.engine.clear_locks();
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        let Some(r) = self.cur.clone() else { return };
        match self.stage.clone() {
            Stage::Idle => self.stage = Stage::A { a: r.point, face_n: r.face_normal() },
            Stage::A { a, .. } => {
                if a.distance(r.point) > TOL {
                    self.stage = Stage::B { a, b: r.point };
                }
            }
            Stage::B { a, b } => {
                if let Some(p) = WorkPlane::from_points("", a, b, r.point) {
                    self.create(ctx, p);
                }
            }
            Stage::Drag { .. } => {}
        }
        self.engine.arrow_lock = None;
    }

    /// Plane through the first point with normal `n` (red kept near the current red).
    fn create_with_normal(&mut self, ctx: &mut ToolContext, n: DVec3) {
        let Stage::A { a, .. } = self.stage else { return };
        if let Some(p) = WorkPlane::from_normal("", a, n, Some(crate::axes::current().x)) {
            self.create(ctx, p);
        }
    }

    /// Apply the dragged handle to the model's active plane (live; committed on release).
    fn drag_to(&mut self, ctx: &mut ToolContext) {
        let Stage::Drag { h, orig } = self.stage.clone() else { return };
        self.update(ctx);
        let Some(r) = &self.cur else { return };
        let Some(i) = ctx.doc.model.active_plane else { return };
        let mut p = orig.clone();
        match h {
            Handle::Origin => p.origin = r.point,
            Handle::Normal => {
                let _ = p.set_normal(r.point - orig.origin);
            }
            Handle::Spin => {
                let n = orig.normal();
                let d = r.point - orig.origin;
                let d = d - n * d.dot(n);
                if d.length() > TOL {
                    let ang = orig.x.cross(d).dot(n).atan2(orig.x.dot(d));
                    p.spin_by(ang.to_degrees());
                }
            }
        }
        if let Some(q) = ctx.doc.model.work_planes.get_mut(i) {
            *q = p;
        }
        crate::axes::sync(&ctx.doc.model);
    }

    fn finish_drag(&mut self, ctx: &mut ToolContext) {
        let Stage::Drag { orig, .. } = std::mem::replace(&mut self.stage, Stage::Idle) else { return };
        let Some(i) = ctx.doc.model.active_plane else { return };
        let Some(now) = ctx.doc.model.work_planes.get(i).cloned() else { return };
        ctx.doc.model.work_planes[i] = orig.clone();
        if now != orig {
            ctx.commit(&tr!("tool-undo-edit-guide-plane"), |m| m.work_planes[i] = now);
        }
        crate::axes::sync(&ctx.doc.model);
    }

    fn cancel(&mut self, ctx: &mut ToolContext) {
        if let Stage::Drag { orig, .. } = std::mem::replace(&mut self.stage, Stage::Idle) {
            if let Some(q) = ctx.doc.model.active_plane.and_then(|i| ctx.doc.model.work_planes.get_mut(i)) {
                *q = orig;
            }
            crate::axes::sync(&ctx.doc.model);
        }
        self.engine.clear_locks();
    }
}

impl Tool for GuidePlaneTool {
    fn id(&self) -> ToolId {
        ToolId::GuidePlane
    }
    fn deactivate(&mut self, ctx: &mut ToolContext) {
        self.cancel(ctx);
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        if matches!(self.stage, Stage::Drag { .. }) {
            self.drag_to(ctx);
            return;
        }
        self.hover = if matches!(self.stage, Stage::Idle) { self.handle_at(ctx, ctx.pointer.pos) } else { None };
        self.update(ctx);
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if !self.clicker.down(ctx) {
            return;
        }
        if matches!(self.stage, Stage::Idle)
            && let Some(h) = self.handle_at(ctx, ctx.pointer.pos)
            && let Some(orig) = ctx.doc.model.active_work_plane().cloned()
        {
            self.stage = Stage::Drag { h, orig };
            self.hover = Some(h);
            return;
        }
        self.click(ctx);
    }
    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        if matches!(self.stage, Stage::Drag { .. }) {
            self.clicker.up_is_drag(ctx);
            self.finish_drag(ctx);
            return;
        }
        if self.clicker.up_is_drag(ctx) && !matches!(self.stage, Stage::Idle) {
            self.click(ctx);
        }
    }
    fn on_double_click(&mut self, ctx: &mut ToolContext) {
        // The first click of the double-click set point A: plane of the face there.
        if let Stage::A { face_n: Some(n), .. } = self.stage {
            self.create_with_normal(ctx, n);
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        if key == Key::Escape {
            if matches!(self.stage, Stage::Idle) {
                ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select));
            }
            self.cancel(ctx);
            return true;
        }
        if let Stage::A { face_n, .. } = self.stage {
            if key == Key::Enter {
                self.create_with_normal(ctx, face_n.unwrap_or(crate::axes::current().z));
                return true;
            }
            // Arrows: normal along red / green / blue / the last hovered edge.
            if self.engine.handle_arrow(key) {
                let dir = self.engine.arrow_dir().map(|d| d.0);
                self.engine.arrow_lock = None;
                if let Some(n) = dir {
                    self.create_with_normal(ctx, n);
                }
                return true;
            }
        }
        let r = self.engine.handle_arrow(key);
        if r {
            self.update(ctx);
        }
        r
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        // `x,y,z`: the first point in the current axes' coordinates.
        if !matches!(self.stage, Stage::Idle) {
            return false;
        }
        let Some(v) = parse_lens(ctx.doc.model.unit, input).filter(|v| v.len() == 3) else { return false };
        let cur = crate::axes::current();
        self.stage = Stage::A { a: cur.origin + cur.world_vec(DVec3::new(v[0], v[1], v[2])), face_n: None };
        self.update(ctx);
        true
    }
    fn vcb_label(&self) -> String {
        match self.stage {
            Stage::Idle => tr!("tool-vcb-point"),
            _ => tr!("tool-vcb-measurements"),
        }
    }
    fn status_text(&self) -> String {
        match &self.stage {
            Stage::Idle => tr!("tool-workplane-origin"),
            Stage::A { .. } => tr!("tool-workplane-red"),
            Stage::B { .. } => tr!("tool-workplane-third"),
            Stage::Drag { h: Handle::Origin, .. } => tr!("tool-workplane-drag-origin"),
            Stage::Drag { h: Handle::Normal, .. } => tr!("tool-workplane-drag-normal"),
            Stage::Drag { h: Handle::Spin, .. } => tr!("tool-workplane-drag-spin"),
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let Some(p) = self.preview() {
            let half = px_at(view.camera, view.viewport, p.origin) * PLANE_PX;
            draw_plane(&p, true, half, out3d);
        }
        if let Stage::A { a, .. } | Stage::B { a, .. } = self.stage {
            out2d.push(Marker2d::Point { at: a, shape: MarkerShape::Dot, color: ACTIVE });
        }
        if let Stage::B { a, b } = self.stage {
            line(out3d, a, b, ACTIVE, 1.5);
        }
        // Gizmo of the active plane.
        if matches!(self.stage, Stage::Idle | Stage::Drag { .. })
            && let Some(p) = view.doc.model.active_work_plane().filter(|p| !p.hidden)
        {
            let len = Self::gizmo_len(view.camera, view.viewport, p);
            let n = p.normal();
            let tip = p.origin + n * len;
            line(out3d, p.origin, tip, InferenceKind::OnBlueAxis.color(), 2.5);
            line(out3d, p.origin, p.origin + p.x * len, InferenceKind::OnRedAxis.color(), 2.5);
            // Spin arc from red toward green.
            let steps = 12;
            for k in 0..steps {
                let (t0, t1) = (k as f64 / steps as f64, (k + 1) as f64 / steps as f64);
                let pt = |t: f64| {
                    let a = t * std::f64::consts::FRAC_PI_2;
                    p.origin + (p.x * a.cos() + p.y * a.sin()) * len * 0.6
                };
                line(out3d, pt(t0), pt(t1), ACTIVE, 1.5);
            }
            let active = match &self.stage {
                Stage::Drag { h, .. } => Some(*h),
                _ => self.hover,
            };
            for (h, at) in handles(p, len) {
                let base = match h {
                    Handle::Origin => ACTIVE,
                    Handle::Normal => InferenceKind::OnBlueAxis.color(),
                    Handle::Spin => InferenceKind::OnRedAxis.color(),
                };
                let color = if active == Some(h) { SELECT_BLUE } else { base };
                let shape = if h == Handle::Origin { MarkerShape::Ring } else { MarkerShape::Square };
                out2d.push(Marker2d::Point { at, shape, color });
            }
        }
        if (!matches!(self.stage, Stage::Idle) || self.hover.is_none())
            && let Some(r) = &self.cur
        {
            inference::draw_result(r, out3d, out2d);
        }
    }
    fn cursor(&self) -> CursorIcon {
        if self.hover.is_some() && matches!(self.stage, Stage::Idle) { CursorIcon::Grab } else { CursorIcon::None }
    }
    fn is_busy(&self) -> bool {
        !matches!(self.stage, Stage::Idle)
    }
}
