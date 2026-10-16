//! Construction tools: Tape Measure (measure + guides), Protractor (angled guides), Axes.

use super::common::*;
use super::draw::Clicker;
use super::transform::draw_protractor;
use super::{OverlayView, Tool, ToolContext, ToolId};
use crate::inference::{self, InferenceEngine, InferenceKind, InferenceResult};
use crate::overlay::{Marker2d, MarkerShape};
use egui::{Color32, CursorIcon, Key};
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::{GuideGeom, GuideLine, Plane, TOL};
use varstotas_render::Overlay3d;

const GUIDE: Color32 = Color32::from_rgb(0x50, 0x50, 0x60);

// ===========================================================================
// Tape Measure
// ===========================================================================

#[derive(Clone, Copy, Debug)]
struct TapeStart {
    a: DVec3,
    /// Edge direction when the measurement started on an edge (parallel guide mode).
    edge_dir: Option<DVec3>,
}

/// Perpendicular offset of `q` from the line through `a` with unit direction `d`.
pub fn perp_offset(a: DVec3, d: DVec3, q: DVec3) -> DVec3 {
    let v = q - a;
    v - d * v.dot(d)
}

pub struct TapeTool {
    engine: InferenceEngine,
    start: Option<TapeStart>,
    cur: Option<InferenceResult>,
    /// Create guides (Ctrl toggles; on by default).
    guides: bool,
    toggle: ModToggle,
    clicker: Clicker,
    last_measure: Option<f64>,
    /// Last completed point-to-point measurement (typing a value then offers a resize).
    resize_base: Option<f64>,
    unit: Option<varstotas_kernel::LengthUnit>,
}

/// Uniformly scale everything in the active context (including its guides) about the
/// context origin. Inside a component this resizes its definition (all instances).
pub(crate) fn resize_active(doc: &mut crate::document::Document, factor: f64) -> bool {
    let path = doc.active_path.clone();
    let root = path.is_empty();
    let m = DMat4::from_scale(DVec3::splat(factor));
    let label = if root { "Resize Model" } else { "Resize" };
    doc
        .commit(label, |model| {
            let mesh = model.mesh_at_mut(&path)?;
            let vs: std::collections::HashSet<_> = mesh.vertices.keys().collect();
            if !vs.is_empty() {
                mesh.transform_vertices(&vs, &m);
            }
            for (_, inst) in mesh.instances.iter_mut() {
                inst.transform = m * inst.transform;
            }
            // The context's own guides (local coordinates) scale with it.
            let d = model.def_at(&path)?;
            model.transform_def_guides(d, &m);
            Some(())
        })
        .flatten()
        .is_some()
}

impl Default for TapeTool {
    fn default() -> Self {
        Self {
            engine: InferenceEngine::default(),
            start: None,
            cur: None,
            guides: true,
            toggle: ModToggle::default(),
            clicker: Clicker::default(),
            last_measure: None,
            resize_base: None,
            unit: None,
        }
    }
}

impl TapeTool {
    fn update(&mut self, ctx: &ToolContext) {
        self.unit = Some(ctx.doc.model.unit);
        let held = ctx.pointer.modifiers.ctrl || ctx.pointer.modifiers.alt;
        let before = self.toggle.on;
        self.toggle.update(held);
        if self.toggle.on != before {
            self.guides = !self.guides;
        }
        self.cur = Some(infer(&mut self.engine, ctx, self.start.map(|s| s.a), None, None));
    }

    /// Current measured value and the guide that would be created.
    fn measure(&self) -> Option<(f64, Option<GuideLine>, Option<DVec3>)> {
        let s = self.start?;
        let q = self.cur.as_ref()?.point;
        match s.edge_dir.filter(|_| self.guides) {
            Some(d) => {
                let off = perp_offset(s.a, d, q);
                Some((off.length(), Some(GuideLine { origin: s.a + off, dir: d }), None))
            }
            None => Some((s.a.distance(q), None, self.guides.then_some(q))),
        }
    }

    fn finish(&mut self, ctx: &mut ToolContext, m: (f64, Option<GuideLine>, Option<DVec3>)) {
        self.last_measure = Some(m.0);
        self.resize_base = (m.1.is_none() && m.0 > TOL).then_some(m.0);
        self.start = None;
        let (_, gl, gp) = m;
        if gl.is_none() && gp.is_none() {
            return;
        }
        if m.0 < TOL {
            return;
        }
        // Guides belong to the open group / component (stored in its local coordinates).
        let path = ctx.doc.active_path.clone();
        ctx.doc.commit("Create Guide", |model| {
            if let Some(g) = gl {
                model.add_guide_in(&path, GuideGeom::Line(g));
            }
            if let Some(p) = gp {
                model.add_guide_in(&path, GuideGeom::Point(p));
            }
        });
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        let Some(r) = self.cur.clone() else { return };
        match self.start {
            None => {
                let edge_dir = match (r.kind, r.edge) {
                    (InferenceKind::OnEdge, Some((a, b))) => Some((b - a).normalize_or(DVec3::X)),
                    _ => None,
                };
                self.start = Some(TapeStart { a: r.point, edge_dir });
                self.resize_base = None;
            }
            Some(_) => {
                if let Some(m) = self.measure() {
                    self.finish(ctx, m);
                }
            }
        }
    }
}

impl Tool for TapeTool {
    fn id(&self) -> ToolId {
        ToolId::Tape
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if self.clicker.down(ctx) {
            self.click(ctx);
        }
    }
    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        if self.clicker.up_is_drag(ctx) && self.start.is_some() {
            self.click(ctx);
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        if key == Key::Escape {
            self.start = None;
            self.engine.clear_locks();
            return true;
        }
        let r = self.engine.handle_arrow(key);
        if r {
            self.update(ctx);
        }
        r
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        let Some(v) = parse_len(ctx.doc.model.unit, input) else { return false };
        // After a completed point-to-point measurement: offer to resize the model.
        if self.start.is_none() {
            let Some(base) = self.resize_base else { return false };
            if v <= TOL {
                return false;
            }
            // The app asks for confirmation in a dialog, then resizes.
            // The measurement is kept: after "No" another value can be typed (the app resets
            // the tool after a confirmed resize).
            ctx.requests.push(super::ToolRequest::ConfirmResize(v / base));
            return true;
        }
        // While measuring: the value sets the guide distance.
        let (Some(s), Some(r)) = (self.start, self.cur.clone()) else { return false };
        match s.edge_dir.filter(|_| self.guides) {
            Some(d) => {
                let off = perp_offset(s.a, d, r.point);
                let dir = off.normalize_or(d.any_orthonormal_vector());
                let g = GuideLine { origin: s.a + dir * v, dir: d };
                self.finish(ctx, (v.abs(), Some(g), None));
            }
            None => {
                let dir = (r.point - s.a).normalize_or(DVec3::X);
                let p = s.a + dir * v;
                self.finish(ctx, (v.abs(), None, self.guides.then_some(p)));
            }
        }
        true
    }
    fn vcb_label(&self) -> &str {
        "Length"
    }
    fn vcb_value(&self) -> String {
        match (self.unit, self.measure().map(|m| m.0).or(self.last_measure)) {
            (Some(u), Some(d)) => varstotas_kernel::units::format_length(d, u),
            _ => String::new(),
        }
    }
    fn status_text(&self) -> String {
        let g = if self.guides { "guides on" } else { "measure only" };
        if self.start.is_none() && self.resize_base.is_some() {
            return format!("Type a new length to resize the model, or select a start point ({g}).");
        }
        if self.start.is_none() {
            format!("Select a start point (an edge for a parallel guide). Ctrl/Option toggles guides ({g}).")
        } else {
            format!("Select the end point or type a distance ({g}).")
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let (Some(s), Some(r), Some((d, gl, _))) = (self.start, &self.cur, self.measure()) {
            let model = &view.doc.model;
            match gl {
                Some(g) => {
                    let reach = (view.camera.eye - view.camera.target).length() * 10.0;
                    dashed(out3d, g.origin - g.dir * reach, g.origin + g.dir * reach, GUIDE, 1.5);
                    dashed(out3d, s.a, g.origin, Color32::from_gray(40), 1.5);
                }
                None => dashed(out3d, s.a, r.point, Color32::from_gray(40), 1.5),
            }
            label(out2d, r.point, fmt_len(model, d));
        }
        if let Some(r) = &self.cur {
            inference::draw_result(r, out3d, out2d);
            if self.start.is_none() && self.guides {
                out2d.push(Marker2d::Point { at: r.point, shape: MarkerShape::Cross, color: GUIDE });
            }
        }
    }
    fn cursor_plus(&self) -> bool {
        self.guides
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        self.start.is_some()
    }
}

// ===========================================================================
// Protractor
// ===========================================================================

#[derive(Clone, Copy, Debug)]
enum ProStage {
    Center { c: DVec3, n: DVec3 },
    Angle { c: DVec3, n: DVec3, zero: DVec3 },
}

#[derive(Default)]
pub struct ProtractorTool {
    engine: InferenceEngine,
    stage: Option<ProStage>,
    cur: Option<InferenceResult>,
    angle: f64,
    clicker: Clicker,
}

impl ProtractorTool {
    fn update(&mut self, ctx: &ToolContext) {
        let (anchor, plane) = match self.stage {
            Some(ProStage::Center { c, n }) | Some(ProStage::Angle { c, n, .. }) => (Some(c), Some(Plane::new(c, n))),
            None => (None, None),
        };
        let r = infer(&mut self.engine, ctx, anchor, None, plane);
        if let Some(ProStage::Angle { c, n, zero }) = self.stage {
            let pl = Plane::new(c, n);
            let q = if r.is_geometry_snap() {
                pl.project(r.point)
            } else {
                pl.intersect_ray(&ctx.pointer.ray).filter(|t| *t > 0.0).map(|t| ctx.pointer.ray.at(t)).unwrap_or(pl.project(r.point))
            };
            let d = q - c;
            if d.length() > 1e-9 {
                let a = signed_angle(zero, d.normalize(), n);
                self.angle = if r.is_geometry_snap() { a } else { snap_angle(a, 15.0, 3.0).0 };
            }
        }
        self.cur = Some(r);
    }

    fn finish(&mut self, ctx: &mut ToolContext, c: DVec3, n: DVec3, zero: DVec3, deg: f64) {
        self.stage = None;
        let dir = DMat4::from_axis_angle(n, deg.to_radians()).transform_vector3(zero);
        let path = ctx.doc.active_path.clone();
        ctx.doc.commit("Create Guide", |m| m.add_guide_in(&path, GuideGeom::Line(GuideLine { origin: c, dir })));
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        let Some(r) = self.cur.clone() else { return };
        match self.stage {
            None => {
                let n = self
                    .engine
                    .arrow_dir()
                    .map(|d| d.0)
                    .or(r.face.as_ref().map(|f| f.plane.normal))
                    .unwrap_or(crate::axes::current().z);
                self.engine.arrow_lock = None;
                self.stage = Some(ProStage::Center { c: r.point, n });
            }
            Some(ProStage::Center { c, n }) => {
                let d = Plane::new(c, n).project(r.point) - c;
                if d.length() > TOL {
                    self.stage = Some(ProStage::Angle { c, n, zero: d.normalize() });
                }
            }
            Some(ProStage::Angle { c, n, zero }) => {
                let a = self.angle;
                self.finish(ctx, c, n, zero, a);
            }
        }
    }
}

impl Tool for ProtractorTool {
    fn id(&self) -> ToolId {
        ToolId::Protractor
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if self.clicker.down(ctx) {
            self.click(ctx);
        }
    }
    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        if self.clicker.up_is_drag(ctx) && self.stage.is_some() {
            self.click(ctx);
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        if key == Key::Escape {
            self.stage = None;
            return true;
        }
        let r = self.engine.handle_arrow(key);
        if r {
            if let (Some(ProStage::Center { c, .. }), Some((d, _))) = (self.stage, self.engine.arrow_dir()) {
                self.stage = Some(ProStage::Center { c, n: d });
                self.engine.arrow_lock = None;
            }
            self.update(ctx);
        }
        r
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        let Some(a) = parse_angle(input) else { return false };
        if let Some(ProStage::Angle { c, n, zero }) = self.stage {
            self.finish(ctx, c, n, zero, a);
            return true;
        }
        false
    }
    fn vcb_label(&self) -> &str {
        "Angle"
    }
    fn vcb_value(&self) -> String {
        if matches!(self.stage, Some(ProStage::Angle { .. })) { fmt_angle(self.angle) } else { String::new() }
    }
    fn status_text(&self) -> String {
        match self.stage {
            None => "Select the protractor center (arrows lock its axis).".into(),
            Some(ProStage::Center { .. }) => "Select the base direction.".into(),
            Some(ProStage::Angle { .. }) => "Select the angle (snaps every 15°) or type degrees.".into(),
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        let r = self.cur.as_ref();
        match self.stage {
            None => {
                if let Some(r) = r {
                    let n = self.engine.arrow_dir().map(|d| d.0).or(r.face.as_ref().map(|f| f.plane.normal)).unwrap_or(crate::axes::current().z);
                    draw_protractor(view, r.point, n, None, out3d);
                    inference::draw_result(r, out3d, out2d);
                }
            }
            Some(ProStage::Center { c, n }) => {
                draw_protractor(view, c, n, None, out3d);
                if let Some(r) = r {
                    dashed(out3d, c, r.point, Color32::from_gray(60), 1.5);
                }
            }
            Some(ProStage::Angle { c, n, zero }) => {
                draw_protractor(view, c, n, Some(zero), out3d);
                let reach = (view.camera.eye - view.camera.target).length() * 10.0;
                let dir = DMat4::from_axis_angle(n, self.angle.to_radians()).transform_vector3(zero);
                dashed(out3d, c - dir * reach, c + dir * reach, GUIDE, 1.5);
                dashed(out3d, c, c + zero * reach * 0.01, Color32::from_gray(40), 1.5);
                out2d.push(Marker2d::CursorTooltip { text: fmt_angle(self.angle) });
            }
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        self.stage.is_some()
    }
}

// ===========================================================================
// Axes
// ===========================================================================

#[derive(Clone, Copy, Debug)]
enum AxStage {
    Idle,
    Origin { o: DVec3 },
    Red { o: DVec3, x: DVec3 },
}

pub struct AxesTool {
    engine: InferenceEngine,
    stage: AxStage,
    cur: Option<InferenceResult>,
    clicker: Clicker,
}

impl Default for AxesTool {
    fn default() -> Self {
        Self { engine: InferenceEngine::default(), stage: AxStage::Idle, cur: None, clicker: Clicker::default() }
    }
}

/// Length of the tentative axes drawn by the tool (screen px).
const AXES_PREVIEW_PX: f64 = 120.0;

impl AxesTool {
    fn update(&mut self, ctx: &ToolContext) {
        let (anchor, fallback) = match self.stage {
            AxStage::Idle => (None, None),
            AxStage::Origin { o } => (Some(o), Some(Plane::new(o, crate::axes::current().z))),
            AxStage::Red { o, x } => {
                // Green point: on the plane through the red axis facing the current blue.
                let cz = crate::axes::current().z;
                let n = if x.cross(cz).length() > 1e-6 { cz - x * cz.dot(x) } else { crate::axes::current().y };
                (Some(o), Some(Plane::new(o, n.normalize())))
            }
        };
        let r = infer(&mut self.engine, ctx, anchor, None, fallback);
        self.cur = Some(r);
    }

    /// The axes the tool would set now (preview).
    fn preview(&self) -> Option<crate::axes::Axes> {
        let p = self.cur.as_ref()?.point;
        let cur = crate::axes::current();
        match self.stage {
            AxStage::Idle => Some(crate::axes::Axes { origin: p, ..cur }),
            AxStage::Origin { o } => {
                let x = (p - o).normalize_or_zero();
                if x == DVec3::ZERO {
                    return Some(crate::axes::Axes { origin: o, ..cur });
                }
                // Keep blue as close to the current blue as possible.
                let y_hint = cur.z.cross(x);
                let y_hint = if y_hint.length() > 1e-6 { y_hint } else { cur.y };
                crate::axes::Axes::from_dirs(o, x, y_hint)
            }
            AxStage::Red { o, x } => crate::axes::Axes::from_dirs(o, x, p - o),
        }
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        let Some(r) = self.cur.clone() else { return };
        match self.stage {
            AxStage::Idle => self.stage = AxStage::Origin { o: r.point },
            AxStage::Origin { o } => {
                if o.distance(r.point) > TOL {
                    self.stage = AxStage::Red { o, x: (r.point - o).normalize() };
                }
            }
            AxStage::Red { .. } => {
                if let Some(a) = self.preview() {
                    self.commit(ctx, a);
                }
            }
        }
        self.engine.arrow_lock = None;
    }

    fn commit(&mut self, ctx: &mut ToolContext, a: crate::axes::Axes) {
        if ctx.doc.try_commit("Place Axes", |m| m.set_drawing_frame(a.origin, a.x, a.y)).is_some() {
            crate::axes::set_current(a);
            ctx.requests.push(super::ToolRequest::SwitchTool(ToolId::Select));
        }
        self.stage = AxStage::Idle;
    }
}

impl Tool for AxesTool {
    fn id(&self) -> ToolId {
        ToolId::Axes
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if self.clicker.down(ctx) {
            self.click(ctx);
        }
    }
    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        if self.clicker.up_is_drag(ctx) && !matches!(self.stage, AxStage::Idle) {
            self.click(ctx);
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        if key == Key::Escape {
            self.stage = AxStage::Idle;
            self.engine.clear_locks();
            return true;
        }
        let r = self.engine.handle_arrow(key);
        if r {
            self.update(ctx);
        }
        r
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        // `x,y,z`: origin in the current axes' coordinates (before the first click).
        if !matches!(self.stage, AxStage::Idle) {
            return false;
        }
        let Some(v) = parse_lens(ctx.doc.model.unit, input).filter(|v| v.len() == 3) else { return false };
        let cur = crate::axes::current();
        let o = cur.origin + cur.world_vec(DVec3::new(v[0], v[1], v[2]));
        self.stage = AxStage::Origin { o };
        self.update(ctx);
        true
    }
    fn vcb_label(&self) -> &str {
        match self.stage {
            AxStage::Idle => "Origin",
            _ => "Measurements",
        }
    }
    fn status_text(&self) -> String {
        match self.stage {
            AxStage::Idle => "Select the new origin, or type x,y,z.".into(),
            AxStage::Origin { .. } => "Select a point along the new red axis.".into(),
            AxStage::Red { .. } => "Select a point toward the new green axis.".into(),
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let Some(a) = self.preview() {
            let vp = varstotas_kernel::glam::DVec2::new(view.viewport.x as f64, view.viewport.y as f64);
            let len = view.camera.pixel_size_at(a.origin, vp) * AXES_PREVIEW_PX;
            for (d, k) in [
                (a.x, InferenceKind::OnRedAxis),
                (a.y, InferenceKind::OnGreenAxis),
                (a.z, InferenceKind::OnBlueAxis),
            ] {
                line(out3d, a.origin, a.origin + d * len, k.color(), 2.5);
                dashed(out3d, a.origin, a.origin - d * len * 0.5, k.color(), 1.5);
            }
        }
        if let Some(r) = &self.cur {
            inference::draw_result(r, out3d, out2d);
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        !matches!(self.stage, AxStage::Idle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perp_offset_from_edge() {
        let off = perp_offset(DVec3::ZERO, DVec3::X, DVec3::new(500.0, 300.0, 0.0));
        assert!((off - DVec3::new(0.0, 300.0, 0.0)).length() < 1e-12);
    }
}
