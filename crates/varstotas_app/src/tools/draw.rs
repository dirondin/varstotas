//! Drawing tools: Line, Rectangle, Circle / Polygon, Arc (2-point).
//! Arc (centre), 3-point arc, Pie and Rotated Rectangle live in `arcs.rs`.

use super::common::*;
use super::{OverlayView, Tool, ToolContext, ToolId};
use crate::inference::{self, Constraint, InferenceEngine, InferenceKind, InferenceResult};
use crate::overlay::Marker2d;
use egui::{CursorIcon, Key, PointerButton, Pos2};
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::{CurveInfo, Plane, TOL};
use varstotas_render::Overlay3d;

/// Shared click / drag-release bookkeeping.
#[derive(Default)]
pub struct Clicker {
    press: Option<Pos2>,
}

impl Clicker {
    /// Returns true if this is a primary press.
    pub fn down(&mut self, ctx: &ToolContext) -> bool {
        if ctx.pointer.button == Some(PointerButton::Primary) {
            self.press = Some(ctx.pointer.pos);
            true
        } else {
            false
        }
    }
    /// Returns true if the release ends a press-drag (counts as a second click).
    pub fn up_is_drag(&mut self, ctx: &ToolContext) -> bool {
        if ctx.pointer.button != Some(PointerButton::Primary) {
            return false;
        }
        self.press.take().is_some_and(|p| (ctx.pointer.pos - p).length() > DRAG_CLICK_PX)
    }
}

pub(super) fn plane_point(ctx: &ToolContext, res: &InferenceResult, plane: &Plane) -> DVec3 {
    if res.is_geometry_snap() || res.kind.is_direction() || res.kind == InferenceKind::FromPoint {
        return plane.project(res.point);
    }
    let ray = ctx.pointer.ray;
    match plane.intersect_ray(&ray) {
        Some(t) if t > 0.0 && ray.at(t).is_finite() => ray.at(t),
        _ => plane.project(res.point),
    }
}

/// Plane normal for shapes started at `res`: arrow lock > face under cursor > horizontal.
pub(super) fn shape_normal(engine: &InferenceEngine, res: &InferenceResult) -> (DVec3, bool) {
    if let Some((d, _)) = engine.arrow_dir() {
        return (d, true);
    }
    if let Some(f) = &res.face
        && f.plane.distance(res.point).abs() < 1e-3 * (1.0 + res.point.length() * 1e-6)
    {
        return (f.plane.normal, true);
    }
    (crate::axes::current().z, false)
}

// ===========================================================================
// Line
// ===========================================================================

#[derive(Default)]
pub struct LineTool {
    engine: InferenceEngine,
    start: Option<DVec3>,
    chain_first: Option<DVec3>,
    cur: Option<InferenceResult>,
    clicker: Clicker,
    last_seg: Option<(DVec3, DVec3)>,
    unit: Option<varstotas_kernel::LengthUnit>,
}

impl LineTool {
    fn update(&mut self, ctx: &ToolContext) {
        self.unit = Some(ctx.doc.model.unit);
        self.cur = Some(infer(&mut self.engine, ctx, self.start, None, None));
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        let Some(p) = self.cur.as_ref().map(|r| r.point) else { return };
        match self.start {
            None => {
                self.start = Some(p);
                self.chain_first = Some(p);
            }
            Some(a) if a.distance(p) > TOL => self.commit_segment(ctx, a, p),
            _ => {}
        }
    }

    fn commit_segment(&mut self, ctx: &mut ToolContext, a: DVec3, b: DVec3) {
        let faces_before = ctx.doc.active_mesh().map(|m| m.faces.len()).unwrap_or(0);
        let (la, lb) = (to_local(ctx.doc, a), to_local(ctx.doc, b));
        if commit_mesh(ctx, "Line", |m| m.add_line(la, lb)).is_none() {
            return;
        }
        self.last_seg = Some((a, b));
        let faces_after = ctx.doc.active_mesh().map(|m| m.faces.len()).unwrap_or(0);
        let closed = self.chain_first.is_some_and(|f| f.distance(b) <= TOL);
        if closed || faces_after > faces_before {
            self.start = None;
            self.chain_first = None;
        } else {
            self.start = Some(b);
        }
        self.engine.arrow_lock = None;
    }

    fn cur_len(&self) -> Option<f64> {
        Some(self.start?.distance(self.cur.as_ref()?.point))
    }
}

impl Tool for LineTool {
    fn id(&self) -> ToolId {
        ToolId::Line
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
    fn on_double_click(&mut self, _ctx: &mut ToolContext) {
        self.start = None;
        self.chain_first = None;
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        if key == Key::Escape {
            self.start = None;
            self.chain_first = None;
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
        let Some(len) = parse_len(ctx.doc.model.unit, input) else { return false };
        if len.abs() < TOL {
            return false;
        }
        if let Some(a) = self.start {
            let dir = self
                .cur
                .as_ref()
                .map(|r| r.point - a)
                .filter(|d| d.length() > 1e-9)
                .map(|d| d.normalize())
                .or_else(|| self.cur.as_ref().and_then(|r| r.constraint_dir))
                .unwrap_or(DVec3::X);
            self.commit_segment(ctx, a, a + dir * len);
            return true;
        }
        if let Some((a, b)) = self.last_seg {
            // Edit the last segment.
            let dir = (b - a).normalize_or(DVec3::X);
            if !undo_last(ctx) {
                return false;
            }
            self.commit_segment(ctx, a, a + dir * len);
            self.start = None;
            self.chain_first = None;
            return true;
        }
        false
    }
    fn vcb_label(&self) -> &str {
        "Length"
    }
    fn vcb_value(&self) -> String {
        match (self.unit, self.cur_len()) {
            (Some(u), Some(l)) => varstotas_kernel::units::format_length(l, u),
            _ => String::new(),
        }
    }
    fn status_text(&self) -> String {
        if self.start.is_none() {
            "Select start point. Arrow keys lock an axis.".into()
        } else {
            "Select end point or type a length. Shift = lock inference, arrows = lock axis, Esc/double-click = finish."
                .into()
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        let Some(r) = &self.cur else { return };
        if let Some(a) = self.start {
            let color = if r.kind.is_direction() { r.kind.color() } else { PREVIEW };
            line(out3d, a, r.point, color, 2.0);
            label(out2d, (a + r.point) * 0.5, fmt_len(&view.doc.model, a.distance(r.point)));
        }
        inference::draw_result(r, out3d, out2d);
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        self.start.is_some()
    }
}

// ===========================================================================
// Rectangle
// ===========================================================================

/// Drawing frame of a rectangle: (normal, u, v).
pub fn rect_frame(c1: DVec3, fixed_normal: Option<DVec3>, p: DVec3) -> (DVec3, DVec3, DVec3) {
    // Work in drawing-axes coordinates (red / green / blue), then map back to world.
    let ax = crate::axes::current();
    let n = match fixed_normal {
        Some(n) => ax.local_vec(n.normalize_or(ax.z)),
        None => {
            let d = ax.local_vec(p - c1);
            let horiz = DVec2::new(d.x, d.y).length();
            if d.z.abs() > 1e-6 && d.z.abs() > horiz * 0.02 {
                // Vertical plane along the dominant horizontal axis.
                if d.x.abs() >= d.y.abs() { DVec3::Y } else { DVec3::X }
            } else {
                DVec3::Z
            }
        }
    };
    let (u, v) = plane_axes(ax.world_vec(n));
    let (u, v) = (ax.local_vec(u), ax.local_vec(v));
    // Prefer the vertical direction as v for vertical planes.
    let (u, v) = if n.z.abs() < 1e-9 && u.z.abs() > v.z.abs() { (v, u) } else { (u, v) };
    let v = if n.z.abs() < 1e-9 && v.z < 0.0 { -v } else { v };
    (ax.world_vec(n), ax.world_vec(u), ax.world_vec(v))
}

pub const GOLDEN: f64 = std::f64::consts::GOLDEN_RATIO;

#[derive(Clone, Copy, Debug)]
struct RectSpec {
    c1: DVec3,
    u: DVec3,
    v: DVec3,
    w: f64,
    h: f64,
}

impl RectSpec {
    fn corners(&self) -> [DVec3; 4] {
        let (c1, u, v) = (self.c1, self.u * self.w, self.v * self.h);
        [c1, c1 + u, c1 + u + v, c1 + v]
    }
}

#[derive(Default)]
pub struct RectangleTool {
    engine: InferenceEngine,
    c1: Option<(DVec3, Option<DVec3>)>,
    cur: Option<InferenceResult>,
    spec: Option<RectSpec>,
    shape_kind: Option<InferenceKind>,
    clicker: Clicker,
    last: Option<RectSpec>,
    unit: Option<varstotas_kernel::LengthUnit>,
}

impl RectangleTool {
    fn update(&mut self, ctx: &ToolContext) {
        self.unit = Some(ctx.doc.model.unit);
        // An arrow key pressed after the first corner locks the rectangle's plane normal
        // (not a line), so the second corner moves freely on that plane.
        let lock = self.c1.and_then(|_| self.engine.arrow_dir().map(|(d, _)| d));
        let fallback = self.c1.map(|(c, n)| Plane::new(c, lock.or(n).unwrap_or(crate::axes::current().z)));
        let constraint = match (self.c1, lock) {
            (Some((c, _)), Some(d)) => Some(Constraint::Plane(Plane::new(c, d))),
            _ => None,
        };
        let r = infer(&mut self.engine, ctx, self.c1.map(|c| c.0), constraint, fallback);
        self.spec = None;
        self.shape_kind = None;
        if let Some((c1, fixed)) = self.c1 {
            let fixed = lock.or(fixed);
            let (n, u, v) = rect_frame(c1, fixed, r.point);
            let plane = Plane::new(c1, n);
            let q = plane_point(ctx, &r, &plane);
            let (mut w, mut h) = ((q - c1).dot(u), (q - c1).dot(v));
            if !r.is_geometry_snap() && w.abs() > TOL && h.abs() > TOL {
                let cands = [
                    (w.abs(), InferenceKind::Square),
                    (w.abs() / GOLDEN, InferenceKind::GoldenSection),
                    (w.abs() * GOLDEN, InferenceKind::GoldenSection),
                ];
                // Square / golden section inference (within ~10 px).
                let px = ctx.camera.pixel_size_at(q, ctx.viewport_d()) * 10.0;
                if let Some((hh, k)) = cands
                    .iter()
                    .filter(|(_, k)| crate::inference::allowed(*k))
                    .map(|(hh, k)| (*hh, *k, (hh - h.abs()).abs()))
                    .filter(|c| c.2 <= px)
                    .min_by(|a, b| a.2.total_cmp(&b.2))
                    .map(|c| (c.0, c.1))
                {
                    h = hh * h.signum();
                    self.shape_kind = Some(k);
                }
            }
            if w.abs() < 1e-12 {
                w = 0.0;
            }
            self.spec = Some(RectSpec { c1, u, v, w, h });
        }
        self.cur = Some(r);
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        let Some(r) = self.cur.clone() else { return };
        match self.c1 {
            None => {
                let (n, fixed) = shape_normal(&self.engine, &r);
                self.c1 = Some((r.point, fixed.then_some(n)));
            }
            Some(_) => {
                if let Some(s) = self.spec
                    && s.w.abs() > TOL
                    && s.h.abs() > TOL
                {
                    self.commit(ctx, s);
                }
            }
        }
    }

    fn commit(&mut self, ctx: &mut ToolContext, s: RectSpec) {
        let pts: Vec<DVec3> = s.corners().iter().map(|p| to_local(ctx.doc, *p)).collect();
        if commit_mesh(ctx, "Rectangle", |m| m.add_face(&pts)).flatten().is_some() {
            self.last = Some(s);
        } else if ctx.doc.last_error.is_none() {
            ctx.doc.last_error = Some("Rectangle: degenerate input".into());
        }
        self.c1 = None;
        self.spec = None;
        self.engine.arrow_lock = None;
    }
}

impl Tool for RectangleTool {
    fn id(&self) -> ToolId {
        ToolId::Rectangle
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
        if self.clicker.up_is_drag(ctx) && self.c1.is_some() {
            self.click(ctx);
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        if key == Key::Escape {
            self.c1 = None;
            self.spec = None;
            self.engine.clear_locks();
            return true;
        }
        let r = self.engine.handle_arrow(key);
        if r {
            // Arrow keys pick the drawing plane: its normal is the locked axis.
            if let Some((c, _)) = self.c1 {
                self.c1 = Some((c, self.engine.arrow_dir().map(|d| d.0)));
                self.engine.arrow_lock = None;
            }
            self.update(ctx);
        }
        r
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        let Some(vals) = parse_lens(ctx.doc.model.unit, input) else { return false };
        let (w, h) = match vals.as_slice() {
            [w, h] => (*w, *h),
            [w] => (*w, *w),
            _ => return false,
        };
        if w.abs() < TOL || h.abs() < TOL {
            return false;
        }
        // First corner placed but the mouse has not moved yet: no preview spec exists, so
        // build one on the default plane (the corner's face plane, else ground).
        let spec = self.spec.or_else(|| {
            self.c1.map(|(c1, fixed)| {
                let (_, u, v) = rect_frame(c1, fixed, c1);
                RectSpec { c1, u, v, w: 1.0, h: 1.0 }
            })
        });
        if let Some(s) = spec {
            let sw = if s.w < 0.0 { -1.0 } else { 1.0 };
            let sh = if s.h < 0.0 { -1.0 } else { 1.0 };
            self.commit(ctx, RectSpec { w: w * sw, h: h * sh, ..s });
            return true;
        }
        if let Some(s) = self.last {
            if !undo_last(ctx) {
                return false;
            }
            self.commit(ctx, RectSpec { w: w * s.w.signum(), h: h * s.h.signum(), ..s });
            return true;
        }
        false
    }
    fn vcb_label(&self) -> &str {
        "Dimensions"
    }
    fn vcb_value(&self) -> String {
        match (self.unit, self.spec.or(self.last)) {
            (Some(u), Some(s)) => format!(
                "{}, {}",
                varstotas_kernel::units::format_length(s.w.abs(), u),
                varstotas_kernel::units::format_length(s.h.abs(), u)
            ),
            _ => String::new(),
        }
    }
    fn status_text(&self) -> String {
        if self.c1.is_none() {
            "Select first corner. Arrow keys pick the plane (→ red, ← green, ↑ blue normal).".into()
        } else {
            "Select opposite corner or type width,height. Esc = cancel.".into()
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let Some(s) = self.spec {
            let c = s.corners();
            fill_polygon(out3d, &c, egui::Color32::from_rgba_unmultiplied(0x40, 0x60, 0xA0, 0x30));
            let color = if self.shape_kind.is_some() { InferenceKind::Square.color() } else { PREVIEW };
            polyline(out3d, &c, true, color, 2.0);
            if let Some(k) = self.shape_kind {
                dashed(out3d, c[0], c[2], k.color(), 1.5);
                out2d.push(Marker2d::CursorTooltip { text: k.label().into() });
            }
            let m = &view.doc.model;
            label(out2d, c[2], format!("{} × {}", fmt_len(m, s.w.abs()), fmt_len(m, s.h.abs())));
            if self.shape_kind.is_some() {
                return;
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
        self.c1.is_some()
    }
}

// ===========================================================================
// Circle / Polygon
// ===========================================================================

#[derive(Clone, Copy, Debug)]
struct CircleSpec {
    c: DVec3,
    n: DVec3,
    start: DVec3,
    r: f64,
    segs: u32,
}

pub struct CircleTool {
    polygon: bool,
    segs: u32,
    engine: InferenceEngine,
    center: Option<(DVec3, DVec3)>,
    cur: Option<InferenceResult>,
    spec: Option<CircleSpec>,
    clicker: Clicker,
    last: Option<CircleSpec>,
}

impl CircleTool {
    pub fn new(polygon: bool) -> Self {
        Self {
            polygon,
            segs: if polygon { 6 } else { 24 },
            engine: InferenceEngine::default(),
            center: None,
            cur: None,
            spec: None,
            clicker: Clicker::default(),
            last: None,
        }
    }

    fn update(&mut self, ctx: &ToolContext) {
        let fallback = self.center.map(|(c, n)| Plane::new(c, n));
        // An arrow lock along the circle's normal would collapse the radius: use the plane.
        let constraint = match (self.center, self.engine.arrow_dir()) {
            (Some((c, n)), Some((d, _))) if d.dot(n).abs() > 0.5 => Some(Constraint::Plane(Plane::new(c, n))),
            _ => None,
        };
        let r = infer(&mut self.engine, ctx, self.center.map(|c| c.0), constraint, fallback);
        self.spec = self.center.map(|(c, n)| {
            let q = plane_point(ctx, &r, &Plane::new(c, n));
            let d = q - c;
            CircleSpec { c, n, start: d, r: d.length(), segs: self.segs }
        });
        self.cur = Some(r);
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        let Some(r) = self.cur.clone() else { return };
        match self.center {
            None => {
                let (n, _) = shape_normal(&self.engine, &r);
                self.center = Some((r.point, n));
                self.engine.arrow_lock = None;
            }
            Some(_) => {
                if let Some(s) = self.spec.filter(|s| s.r > TOL) {
                    self.commit(ctx, s);
                }
            }
        }
    }

    fn commit(&mut self, ctx: &mut ToolContext, s: CircleSpec) {
        let label = if self.polygon { "Polygon" } else { "Circle" };
        let info = if self.polygon {
            CurveInfo::polygon(s.c, s.n, s.start, s.r, s.segs)
        } else {
            CurveInfo::circle(s.c, s.n, s.start, s.r, s.segs)
        };
        if let Some(info) = info
            && commit_curve(ctx, label, &info, false)
        {
            self.last = Some(s);
        }
        self.center = None;
        self.spec = None;
    }
}

impl Tool for CircleTool {
    fn id(&self) -> ToolId {
        if self.polygon { ToolId::Polygon } else { ToolId::Circle }
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
        if self.clicker.up_is_drag(ctx) && self.center.is_some() {
            self.click(ctx);
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        if key == Key::Escape {
            self.center = None;
            self.spec = None;
            self.engine.clear_locks();
            return true;
        }
        let r = self.engine.handle_arrow(key);
        if r {
            if let (Some((c, _)), Some((d, _))) = (self.center, self.engine.arrow_dir()) {
                self.center = Some((c, d));
                self.engine.arrow_lock = None;
            }
            self.update(ctx);
        }
        r
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        if let Some(n) = parse_segments(input).or_else(|| {
            // Polygon: a bare integer before the first click sets the number of sides.
            (self.polygon && self.center.is_none() && self.last.is_none())
                .then(|| input.trim().parse::<u32>().ok().filter(|n| (3..=360).contains(n)))
                .flatten()
        }) {
            self.segs = n;
            if self.center.is_none()
                && let Some(s) = self.last
            {
                if !undo_last(ctx) {
                    return false;
                }
                self.commit(ctx, CircleSpec { segs: n, ..s });
            }
            self.update(ctx);
            return true;
        }
        let Some(r) = parse_len(ctx.doc.model.unit, input).map(f64::abs).filter(|r| *r > TOL) else {
            return false;
        };
        if let Some(s) = self.spec {
            let start = if s.start.length() > 1e-9 { s.start } else { plane_axes(s.n).0 };
            self.commit(ctx, CircleSpec { r, start, ..s });
            return true;
        }
        if let Some(s) = self.last {
            if !undo_last(ctx) {
                return false;
            }
            self.commit(ctx, CircleSpec { r, ..s });
            return true;
        }
        false
    }
    fn vcb_label(&self) -> &str {
        if self.center.is_some() || self.last.is_some() {
            "Radius"
        } else if self.polygon {
            "Sides"
        } else {
            "Segments"
        }
    }
    fn vcb_value(&self) -> String {
        if self.center.is_none() && self.last.is_none() {
            return format!("{}s", self.segs);
        }
        self.spec.map(|s| format!("{:.1}", s.r)).unwrap_or_default()
    }
    fn status_text(&self) -> String {
        let what = if self.polygon { "polygon" } else { "circle" };
        if self.center.is_none() {
            format!("Select the {what} center (face under cursor orients it; arrows lock the normal). Type `{}s` for {}.", self.segs, if self.polygon { "sides" } else { "segments" })
        } else {
            "Select a point on the radius or type a radius.".into()
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let Some(s) = self.spec.filter(|s| s.r > TOL) {
            let pts = circle_points(s.c, s.n, s.start, s.r, s.segs);
            polyline(out3d, &pts, true, PREVIEW, 2.0);
            line(out3d, s.c, s.c + s.start, normal_color(s.n), 1.5);
            label(out2d, s.c + s.start * 0.5, fmt_len(&view.doc.model, s.r));
        }
        if let Some(r) = &self.cur {
            inference::draw_result(r, out3d, out2d);
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        self.center.is_some()
    }
}

// ===========================================================================
// Arc (2 point)
// ===========================================================================

#[derive(Clone, Copy, Debug)]
struct ArcSpec {
    a: DVec3,
    b: DVec3,
    side: DVec3,
    bulge: f64,
    segs: u32,
}

/// Plane normal for an arc with chord a→b (no other hints).
pub fn arc_default_normal(a: DVec3, b: DVec3) -> DVec3 {
    let ax = crate::axes::current();
    let d = (b - a).normalize_or(ax.x);
    if d.dot(ax.z).abs() < 1e-6 {
        ax.z
    } else {
        let n = d.cross(ax.z);
        if n.length_squared() > 1e-12 { n.normalize() } else { ax.y }
    }
}

pub struct ArcTool {
    engine: InferenceEngine,
    segs: u32,
    a: Option<(DVec3, Option<DVec3>)>,
    /// "Tangent at vertex" direction at the start point (edge / arc ending there).
    tangent: Option<DVec3>,
    b: Option<DVec3>,
    cur: Option<InferenceResult>,
    spec: Option<ArcSpec>,
    half: bool,
    /// The bulge snapped to the arc tangent to `tangent` (cyan).
    tangent_snap: bool,
    clicker: Clicker,
    last: Option<ArcSpec>,
}

impl Default for ArcTool {
    fn default() -> Self {
        Self {
            engine: InferenceEngine::default(),
            segs: 12,
            a: None,
            tangent: None,
            b: None,
            cur: None,
            spec: None,
            half: false,
            tangent_snap: false,
            clicker: Clicker::default(),
            last: None,
        }
    }
}

/// Cyan used for tangent-arc previews.
pub fn tangent_color() -> egui::Color32 {
    InferenceKind::Tangent.color()
}

impl ArcSpec {
    /// The arc as curve metadata (None for a zero bulge / chord).
    fn info(&self) -> Option<CurveInfo> {
        CurveInfo::arc_2pt(self.a, self.b, self.side, self.bulge, self.segs)
    }
}

impl ArcTool {
    fn normal(&self, a: DVec3, b: DVec3, face_n: Option<DVec3>, r: &InferenceResult) -> DVec3 {
        let chord = b - a;
        if let Some((d, _)) = self.engine.arrow_dir()
            && d.dot(chord.normalize_or_zero()).abs() < 1e-6
        {
            return d;
        }
        if let Some(n) = face_n
            && n.dot(chord.normalize_or_zero()).abs() < 1e-6
        {
            return n;
        }
        // Plane of the tangent at the start vertex and the chord.
        if let Some(t) = self.tangent {
            let n = t.cross(chord);
            if n.length() > 1e-6 * chord.length() {
                let n = n.normalize();
                // Keep the sign of the default normal so `side` stays stable.
                return if n.dot(arc_default_normal(a, b)) < 0.0 { -n } else { n };
            }
        }
        if r.is_geometry_snap() {
            let n = chord.cross(r.point - a);
            if n.length() > 1e-6 * chord.length_squared() {
                return n.normalize();
            }
        }
        arc_default_normal(a, b)
    }

    /// The arc from the start point to `b` tangent to the start vertex's edge.
    fn tangent_arc(&self, b: DVec3) -> Option<CurveInfo> {
        let (a, _) = self.a?;
        CurveInfo::arc_tangent(a, self.tangent?, b, self.segs)
    }

    fn update(&mut self, ctx: &ToolContext) {
        let anchor = self.b.or(self.a.map(|a| a.0));
        let fallback = match (self.a, self.b) {
            (Some((a, _)), Some(b)) => Some(Plane::new(a, arc_default_normal(a, b))),
            (Some((a, n)), None) => Some(Plane::new(a, n.unwrap_or_else(|| crate::axes::current().z))),
            _ => None,
        };
        let constraint = None::<Constraint>;
        let r = infer(&mut self.engine, ctx, anchor, constraint, fallback);
        self.spec = None;
        self.half = false;
        self.tangent_snap = false;
        if let (Some((a, fnorm)), Some(b)) = (self.a, self.b) {
            let n = self.normal(a, b, fnorm, &r);
            let plane = Plane::new(a, n);
            let q = plane_point(ctx, &r, &plane);
            let chord = b - a;
            let side = n.cross(chord).normalize_or(DVec3::Y);
            let m = (a + b) * 0.5;
            let mut h = (q - m).dot(side);
            let half = chord.length() * 0.5;
            let px = ctx.camera.pixel_size_at(q, ctx.viewport_d()) * 10.0;
            let tan = self.tangent.and_then(|t| {
                let (tb, ts) = tangent_bulge(a, b, t)?;
                (ts.dot(n).abs() < 1e-6).then(|| tb * ts.dot(side).signum())
            });
            if let Some(ht) = tan
                && (h - ht).abs() <= px
            {
                h = ht;
                self.tangent_snap = true;
            } else if crate::inference::allowed(InferenceKind::HalfCircle) && (h.abs() - half).abs() <= px {
                h = half * h.signum();
                self.half = true;
            }
            self.spec = Some(ArcSpec { a, b, side, bulge: h, segs: self.segs });
        } else if let (Some((a, _)), None) = (self.a, self.b) {
            let b = r.point;
            self.spec = Some(ArcSpec { a, b, side: DVec3::Y, bulge: 0.0, segs: 1 });
        }
        self.cur = Some(r);
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        let Some(r) = self.cur.clone() else { return };
        match (self.a, self.b) {
            (None, _) => {
                let (n, fixed) = shape_normal(&self.engine, &r);
                self.a = Some((r.point, fixed.then_some(n)));
                self.tangent =
                    if r.kind == InferenceKind::Endpoint && crate::inference::allowed(InferenceKind::Tangent) { tangent_at_point(ctx.doc, r.point) } else { None };
            }
            (Some((a, _)), None) => {
                if a.distance(r.point) > TOL {
                    self.b = Some(r.point);
                    self.engine.arrow_lock = None;
                    self.update(ctx);
                }
            }
            (Some(_), Some(_)) => {
                // A click with no bulge (e.g. the 2nd click of a double-click on the end point)
                // does nothing; the double-click handler draws the tangent arc.
                if let Some(s) = self.spec.filter(|s| s.bulge.abs() > TOL) {
                    self.commit(ctx, s);
                }
            }
        }
    }

    fn commit(&mut self, ctx: &mut ToolContext, s: ArcSpec) {
        if let Some(info) = s.info()
            && commit_curve(ctx, "Arc", &info, false)
        {
            self.last = Some(s);
        }
        self.reset();
    }

    fn reset(&mut self) {
        self.a = None;
        self.b = None;
        self.tangent = None;
        self.spec = None;
        self.half = false;
        self.tangent_snap = false;
    }
}

impl Tool for ArcTool {
    fn id(&self) -> ToolId {
        ToolId::Arc
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
        if self.clicker.up_is_drag(ctx) && self.a.is_some() {
            self.click(ctx);
        }
    }
    fn on_double_click(&mut self, ctx: &mut ToolContext) {
        // Double-click on the end point: the arc tangent to the edge at the start point.
        // (A fast click on the start point may make egui count this as a triple click.)
        let (Some((a, _)), Some(b)) = (self.a, self.b) else { return };
        if ctx.pointer.click_count < 2 {
            return;
        }
        let Some(t) = self.tangent else { return };
        let Some((h, side)) = tangent_bulge(a, b, t) else { return };
        self.commit(ctx, ArcSpec { a, b, side, bulge: h, segs: self.segs });
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        if key == Key::Escape {
            self.reset();
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
        let unit = ctx.doc.model.unit;
        if let Some(n) = parse_segments(input) {
            self.segs = n;
            if self.a.is_none()
                && let Some(s) = self.last
            {
                if !undo_last(ctx) {
                    return false;
                }
                self.commit(ctx, ArcSpec { segs: n, ..s });
            }
            self.update(ctx);
            return true;
        }
        // Bulge from `500r` (radius) or a plain distance, keeping the side the arc bends to.
        let bulge_of = |s: &ArcSpec| -> Option<f64> {
            let sign = if s.bulge < 0.0 { -1.0 } else { 1.0 };
            if let Some(r) = parse_radius(unit, input) {
                return bulge_for_radius(s.a.distance(s.b), r).map(|h| h * sign);
            }
            parse_len(unit, input).filter(|v| v.abs() > TOL).map(|v| v.abs() * sign)
        };
        match (self.a, self.b) {
            (Some((a, _)), None) => {
                let Some(v) = parse_len(unit, input).filter(|v| v.abs() > TOL) else { return false };
                let dir = self.cur.as_ref().map(|r| r.point - a).filter(|d| d.length() > 1e-9);
                let Some(dir) = dir else { return false };
                self.b = Some(a + dir.normalize() * v);
                self.update(ctx);
                true
            }
            (Some(_), Some(_)) => {
                let Some(s) = self.spec else { return false };
                let Some(h) = bulge_of(&s) else { return false };
                self.commit(ctx, ArcSpec { bulge: h, ..s });
                true
            }
            (None, _) => {
                let Some(s) = self.last else { return false };
                let Some(h) = bulge_of(&s) else { return false };
                if !undo_last(ctx) {
                    return false;
                }
                self.commit(ctx, ArcSpec { bulge: h, ..s });
                true
            }
        }
    }
    fn vcb_label(&self) -> &str {
        if self.b.is_some() || (self.a.is_none() && self.last.is_some()) { "Bulge" } else { "Length" }
    }
    fn status_text(&self) -> String {
        match (self.a, self.b) {
            (None, _) => "Select the start point of the arc. Start at the end of an edge to draw tangent to it.".into(),
            (Some(_), None) if self.tangent.is_some() => {
                "Select the end point (double-click it for the cyan tangent arc) or type the chord length.".into()
            }
            (Some(_), None) => "Select the end point or type the chord length.".into(),
            _ => "Move to set the bulge and click, or type a bulge (`500r` for a radius). Type `12s` for segments.".into(),
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let Some(s) = self.spec {
            if self.b.is_some() {
                let color = if self.tangent_snap { tangent_color() } else { PREVIEW };
                match s.info() {
                    Some(info) => polyline(out3d, &info.points(), false, color, 2.0),
                    None => line(out3d, s.a, s.b, color, 2.0),
                }
                dashed(out3d, s.a, s.b, egui::Color32::from_gray(90), 1.0);
                let apex = (s.a + s.b) * 0.5 + s.side * s.bulge;
                label(out2d, apex, fmt_len(&view.doc.model, s.bulge.abs()));
                if self.tangent_snap {
                    out2d.push(Marker2d::CursorTooltip { text: InferenceKind::Tangent.label().into() });
                    return;
                }
                if self.half {
                    out2d.push(Marker2d::CursorTooltip { text: InferenceKind::HalfCircle.label().into() });
                    return;
                }
            } else if let Some(info) = self.tangent_arc(s.b) {
                // Starting at an edge end: preview the tangent arc in cyan.
                polyline(out3d, &info.points(), false, tangent_color(), 2.0);
                line(out3d, s.a, s.b, egui::Color32::from_gray(120), 1.0);
            } else {
                line(out3d, s.a, s.b, PREVIEW, 1.5);
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
        self.a.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_plane_horizontal_by_default() {
        let (n, u, v) = rect_frame(DVec3::ZERO, None, DVec3::new(3000.0, 2000.0, 0.0));
        assert_eq!(n, DVec3::Z);
        assert_eq!((u, v), (DVec3::X, DVec3::Y));
    }

    #[test]
    fn rect_plane_vertical_when_point_above() {
        let (n, u, v) = rect_frame(DVec3::ZERO, None, DVec3::new(3000.0, 10.0, 2000.0));
        assert_eq!(n, DVec3::Y);
        assert!((u - DVec3::X).length() < 1e-12 || (u + DVec3::X).length() < 1e-12);
        assert!((v - DVec3::Z).length() < 1e-12);
        let (n, _, v) = rect_frame(DVec3::ZERO, None, DVec3::new(5.0, 1000.0, -800.0));
        assert_eq!(n, DVec3::X);
        assert!((v - DVec3::Z).length() < 1e-12);
    }

    #[test]
    fn rect_plane_from_face() {
        let (n, u, v) = rect_frame(DVec3::ZERO, Some(DVec3::new(1.0, 0.0, 0.0)), DVec3::new(0.0, 5.0, 5.0));
        assert_eq!(n, DVec3::X);
        assert!(u.dot(n).abs() < 1e-12 && v.dot(n).abs() < 1e-12);
        assert!((v - DVec3::Z).length() < 1e-12);
    }

    #[test]
    fn arc_normal_defaults() {
        assert_eq!(arc_default_normal(DVec3::ZERO, DVec3::new(1000.0, 0.0, 0.0)), DVec3::Z);
        let n = arc_default_normal(DVec3::ZERO, DVec3::new(1000.0, 0.0, 500.0));
        assert!(n.dot(DVec3::new(1000.0, 0.0, 500.0)).abs() < 1e-9);
    }
}
