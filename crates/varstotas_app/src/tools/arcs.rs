//! Arc (centre + two points), Pie, 3 Point Arc and Rotated Rectangle tools.
//! The 2 Point Arc lives in `draw.rs`.

use varstotas_i18n::tr;
use super::common::*;
use super::draw::{Clicker, plane_point, shape_normal};
use super::transform::draw_protractor;
use super::{OverlayView, Tool, ToolContext, ToolId};
use crate::inference::{self, Constraint, InferenceEngine, InferenceKind, InferenceResult, axis_kind_of};
use crate::overlay::Marker2d;
use egui::{Color32, CursorIcon, Key};
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{CurveInfo, Plane, TOL};
use varstotas_render::Overlay3d;
use std::f64::consts::TAU;

/// Angle snap step and tolerance (degrees) for protractor-driven tools.
const SNAP_STEP: f64 = 15.0;
const SNAP_TOL: f64 = 1.5;

/// Default segment count for arcs (12).
const ARC_SEGS: u32 = 12;

/// Split `a,b` / `a;b` VCB input into its two parts (second optional).
fn split2(s: &str) -> (&str, Option<&str>) {
    match s.split_once([',', ';']) {
        Some((a, b)) => (a.trim(), Some(b.trim())),
        None => (s.trim(), None),
    }
}

/// Bring `a` (radians, in (-π, π]) to the turn closest to `prev` so a sweep can pass ±180°.
fn unwrap_angle(a: f64, prev: f64) -> f64 {
    let k = ((prev - a) / TAU).round();
    (a + k * TAU).clamp(-TAU + 1e-6, TAU - 1e-6)
}

// ===========================================================================
// Arc (centre, start, end) and Pie
// ===========================================================================

#[derive(Clone, Copy, Debug)]
struct CenterArcSpec {
    c: DVec3,
    n: DVec3,
    /// Start point (radius = distance to `c`).
    start: DVec3,
    /// Signed sweep in radians about `n`.
    sweep: f64,
    segs: u32,
}

impl CenterArcSpec {
    fn info(&self) -> Option<CurveInfo> {
        CurveInfo::arc_center(self.c, self.n, self.start, self.sweep, self.segs)
    }
    fn radius(&self) -> f64 {
        self.start.distance(self.c)
    }
}

enum CStage {
    Idle,
    Center { c: DVec3, n: DVec3 },
    Start { c: DVec3, n: DVec3, start: DVec3 },
}

pub struct ArcCenterTool {
    pie: bool,
    segs: u32,
    engine: InferenceEngine,
    stage: CStage,
    cur: Option<InferenceResult>,
    /// Preview of the radius (stage Center) / the arc (stage Start).
    spec: Option<CenterArcSpec>,
    snapped: bool,
    clicker: Clicker,
    last: Option<CenterArcSpec>,
}

impl ArcCenterTool {
    pub fn new(pie: bool) -> Self {
        Self {
            pie,
            segs: ARC_SEGS,
            engine: InferenceEngine::default(),
            stage: CStage::Idle,
            cur: None,
            spec: None,
            snapped: false,
            clicker: Clicker::default(),
            last: None,
        }
    }

    fn label(&self) -> String {
        tr!(if self.pie { "tool-undo-pie" } else { "tool-undo-arc" })
    }

    fn update(&mut self, ctx: &ToolContext) {
        let (anchor, plane) = match self.stage {
            CStage::Idle => (None, None),
            CStage::Center { c, n } | CStage::Start { c, n, .. } => (Some(c), Some(Plane::new(c, n))),
        };
        // An arrow lock along the normal would collapse the radius: constrain to the plane.
        let constraint = match (plane, self.engine.arrow_dir()) {
            (Some(p), Some((d, _))) if d.dot(p.normal).abs() > 0.5 => Some(Constraint::Plane(p)),
            _ => None,
        };
        let r = infer(&mut self.engine, ctx, anchor, constraint, plane);
        let prev = self.spec;
        self.spec = None;
        self.snapped = false;
        match self.stage {
            CStage::Idle => {}
            CStage::Center { c, n } => {
                let q = plane_point(ctx, &r, &Plane::new(c, n));
                self.spec = Some(CenterArcSpec { c, n, start: q, sweep: 0.0, segs: self.segs });
            }
            CStage::Start { c, n, start } => {
                let q = plane_point(ctx, &r, &Plane::new(c, n));
                let (u, v) = (start - c, q - c);
                if v.length() > 1e-9 {
                    let a = signed_angle(u, v, n).to_radians();
                    let prev_sweep = prev.map_or(0.0, |s| s.sweep);
                    let mut sw = unwrap_angle(a, prev_sweep);
                    if !r.is_geometry_snap() {
                        let (d, hit) = snap_angle(sw.to_degrees(), SNAP_STEP, SNAP_TOL);
                        if hit {
                            sw = d.to_radians();
                            self.snapped = true;
                        }
                    }
                    self.spec = Some(CenterArcSpec { c, n, start, sweep: sw, segs: self.segs });
                } else if let Some(p) = prev {
                    self.spec = Some(p);
                }
            }
        }
        self.cur = Some(r);
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        let Some(r) = self.cur.clone() else { return };
        match self.stage {
            CStage::Idle => {
                let (n, _) = shape_normal(&self.engine, &r);
                self.stage = CStage::Center { c: r.point, n };
                self.engine.arrow_lock = None;
                self.update(ctx);
            }
            CStage::Center { c, n } => {
                if let Some(s) = self.spec.filter(|s| s.radius() > TOL) {
                    self.stage = CStage::Start { c, n, start: s.start };
                    self.engine.arrow_lock = None;
                    self.update(ctx);
                }
            }
            CStage::Start { .. } => {
                if let Some(s) = self.spec.filter(|s| s.sweep.abs() > 1e-6) {
                    self.commit(ctx, s);
                }
            }
        }
    }

    fn commit(&mut self, ctx: &mut ToolContext, s: CenterArcSpec) {
        let label = self.label();
        if let Some(info) = s.info()
            && commit_curve(ctx, &label, &info, self.pie)
        {
            self.last = Some(s);
        } else if ctx.doc.last_error.is_none() {
            ctx.doc.last_error = Some(tr!("tool-degenerate", label = label));
        }
        self.reset();
    }

    fn reset(&mut self) {
        self.stage = CStage::Idle;
        self.spec = None;
        self.snapped = false;
    }
}

impl Tool for ArcCenterTool {
    fn id(&self) -> ToolId {
        if self.pie { ToolId::Pie } else { ToolId::ArcCenter }
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
        if self.clicker.up_is_drag(ctx) && !matches!(self.stage, CStage::Idle) {
            self.click(ctx);
        }
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
            // Before the start point, an arrow re-orients the arc plane.
            if let (CStage::Center { c, .. }, Some((d, _))) = (&self.stage, self.engine.arrow_dir()) {
                self.stage = CStage::Center { c: *c, n: d };
                self.engine.arrow_lock = None;
            }
            self.update(ctx);
        }
        r
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        let unit = ctx.doc.model.unit;
        if let Some(n) = parse_segments(input) {
            self.segs = n;
            if matches!(self.stage, CStage::Idle)
                && let Some(s) = self.last
            {
                if !undo_last(ctx) {
                    return false;
                }
                self.commit(ctx, CenterArcSpec { segs: n, ..s });
            }
            self.update(ctx);
            return true;
        }
        match self.stage {
            CStage::Idle => {
                // Edit the last arc: a plain number is the angle, `500r` the radius.
                let Some(s) = self.last else { return false };
                let new = if let Some(r) = parse_radius(unit, input) {
                    CenterArcSpec { start: s.c + (s.start - s.c).normalize_or(plane_axes(s.n).0) * r, ..s }
                } else {
                    let Some(deg) = parse_angle(input).filter(|d| d.abs() > 1e-6 && d.abs() < 360.0) else {
                        return false;
                    };
                    CenterArcSpec { sweep: deg.to_radians(), ..s }
                };
                if !undo_last(ctx) {
                    return false;
                }
                self.commit(ctx, new);
                true
            }
            CStage::Center { c, n } => {
                let r = parse_radius(unit, input).or_else(|| parse_len(unit, input).map(f64::abs));
                let Some(r) = r.filter(|r| *r > TOL) else { return false };
                let dir = self.spec.map(|s| s.start - c).filter(|d| d.length() > 1e-9).unwrap_or(plane_axes(n).0);
                self.stage = CStage::Start { c, n, start: c + dir.normalize() * r };
                self.update(ctx);
                true
            }
            CStage::Start { c, n, start } => {
                let Some(deg) = parse_angle(input).filter(|d| d.abs() > 1e-6 && d.abs() < 360.0) else { return false };
                // Typed angles follow the side the preview sweeps to.
                let sign = self.spec.map_or(1.0, |s| if s.sweep < 0.0 { -1.0 } else { 1.0 });
                let deg = if deg > 0.0 { deg * sign } else { deg };
                self.commit(ctx, CenterArcSpec { c, n, start, sweep: deg.to_radians(), segs: self.segs });
                true
            }
        }
    }
    fn vcb_label(&self) -> String {
        match self.stage {
            CStage::Center { .. } => tr!("tool-vcb-radius"),
            CStage::Start { .. } => tr!("tool-vcb-angle"),
            CStage::Idle if self.last.is_some() => tr!("tool-vcb-angle"),
            CStage::Idle => tr!("tool-vcb-segments"),
        }
    }
    fn vcb_value(&self) -> String {
        match (&self.stage, self.spec) {
            (CStage::Idle, _) if self.last.is_none() => format!("{}s", self.segs),
            (CStage::Center { .. }, Some(s)) => format!("{:.1}", s.radius()),
            (CStage::Start { .. }, Some(s)) => fmt_angle(s.sweep.to_degrees()),
            _ => String::new(),
        }
    }
    fn status_text(&self) -> String {
        let what = tr!(if self.pie { "tool-arc-what-pie" } else { "tool-arc-what-arc" });
        match self.stage {
            CStage::Idle => tr!("tool-arc-center-first", what = what, segs = self.segs),
            CStage::Center { .. } => tr!("tool-arc-center-start"),
            CStage::Start { .. } => tr!("tool-arc-center-end", what = what),
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        let model = &view.doc.model;
        match self.stage {
            CStage::Idle => {
                if let Some(r) = &self.cur {
                    let n = self.engine.arrow_dir().map(|d| d.0).or(r.face.as_ref().map(|f| f.plane.normal));
                    draw_protractor(view, r.point, n.unwrap_or(crate::axes::current().z), None, out3d);
                }
            }
            CStage::Center { c, n } => {
                draw_protractor(view, c, n, self.spec.map(|s| s.start - c).filter(|d| d.length() > 1e-9), out3d);
                if let Some(s) = self.spec.filter(|s| s.radius() > TOL) {
                    dashed(out3d, c, s.start, normal_color(n), 1.5);
                    label(out2d, (c + s.start) * 0.5, fmt_len(model, s.radius()));
                }
            }
            CStage::Start { c, n, start } => {
                draw_protractor(view, c, n, Some(start - c), out3d);
                if let Some(s) = self.spec {
                    if let Some(info) = s.info() {
                        let mut pts = info.points();
                        if self.pie {
                            pts.insert(0, c);
                            pts.push(c);
                        }
                        polyline(out3d, &pts, false, PREVIEW, 2.0);
                        label(out2d, info.mid_point(), format!("{}°", fmt_angle(s.sweep.to_degrees())));
                    }
                    line(out3d, c, start, Color32::from_gray(90), 1.0);
                }
                if self.snapped {
                    out2d.push(Marker2d::CursorTooltip { text: tr!("tool-cursor-angle") });
                }
            }
        }
        if let Some(r) = &self.cur
            && !self.snapped
        {
            inference::draw_result(r, out3d, out2d);
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        !matches!(self.stage, CStage::Idle)
    }
}

// ===========================================================================
// 3 Point Arc
// ===========================================================================

#[derive(Default)]
pub struct Arc3Tool {
    engine: InferenceEngine,
    segs: Option<u32>,
    p1: Option<(DVec3, Option<DVec3>)>,
    p2: Option<DVec3>,
    cur: Option<InferenceResult>,
    /// Preview arc (after the second point).
    preview: Option<CurveInfo>,
    clicker: Clicker,
    last: Option<CurveInfo>,
}

impl Arc3Tool {
    fn segs(&self) -> u32 {
        self.segs.unwrap_or(ARC_SEGS)
    }

    fn update(&mut self, ctx: &ToolContext) {
        let anchor = self.p2.or(self.p1.map(|p| p.0));
        let fallback = match (self.p1, self.p2) {
            (Some((a, n)), Some(b)) => {
                let n = n.filter(|n| n.dot((b - a).normalize_or_zero()).abs() < 1e-6);
                Some(Plane::new(a, n.unwrap_or_else(|| super::draw::arc_default_normal(a, b))))
            }
            (Some((a, n)), None) => Some(Plane::new(a, n.unwrap_or_else(|| crate::axes::current().z))),
            _ => None,
        };
        let r = infer(&mut self.engine, ctx, anchor, None, fallback);
        self.preview = match (self.p1, self.p2) {
            (Some((a, _)), Some(b)) => CurveInfo::arc_3pt(a, b, r.point, self.segs()),
            _ => None,
        };
        self.cur = Some(r);
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        let Some(r) = self.cur.clone() else { return };
        match (self.p1, self.p2) {
            (None, _) => {
                let (n, fixed) = shape_normal(&self.engine, &r);
                self.p1 = Some((r.point, fixed.then_some(n)));
            }
            (Some((a, _)), None) => {
                if a.distance(r.point) > TOL {
                    self.p2 = Some(r.point);
                    self.engine.arrow_lock = None;
                    self.update(ctx);
                }
            }
            (Some(_), Some(_)) => {
                if let Some(info) = self.preview {
                    self.commit(ctx, info);
                }
            }
        }
    }

    fn commit(&mut self, ctx: &mut ToolContext, info: CurveInfo) {
        if commit_curve(ctx, &tr!("tool-undo-arc3"), &info, false) {
            self.last = Some(info);
        }
        self.reset();
    }

    fn reset(&mut self) {
        self.p1 = None;
        self.p2 = None;
        self.preview = None;
    }
}

impl Tool for Arc3Tool {
    fn id(&self) -> ToolId {
        ToolId::Arc3
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
        if self.clicker.up_is_drag(ctx) && self.p1.is_some() {
            self.click(ctx);
        }
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
            self.segs = Some(n);
            if self.p1.is_none()
                && let Some(info) = self.last
            {
                if !undo_last(ctx) {
                    return false;
                }
                self.commit(ctx, CurveInfo { segments: n, ..info });
            }
            self.update(ctx);
            return true;
        }
        match (self.p1, self.p2) {
            (Some((a, _)), None) => {
                let Some(v) = parse_len(unit, input).filter(|v| v.abs() > TOL) else { return false };
                let Some(dir) = self.cur.as_ref().map(|r| r.point - a).filter(|d| d.length() > 1e-9) else {
                    return false;
                };
                self.p2 = Some(a + dir.normalize() * v);
                self.update(ctx);
                true
            }
            (Some(_), Some(_)) => {
                // Angle of the arc on the current preview circle.
                let Some(info) = self.preview else { return false };
                let Some(deg) = parse_angle(input).filter(|d| *d > 1e-6 && *d < 360.0) else { return false };
                self.commit(ctx, CurveInfo { end_angle: info.start_angle + deg.to_radians(), ..info });
                true
            }
            (None, _) => {
                let Some(info) = self.last else { return false };
                let Some(deg) = parse_angle(input).filter(|d| *d > 1e-6 && *d < 360.0) else { return false };
                if !undo_last(ctx) {
                    return false;
                }
                self.commit(ctx, CurveInfo { end_angle: info.start_angle + deg.to_radians(), ..info });
                true
            }
        }
    }
    fn vcb_label(&self) -> String {
        match (self.p1, self.p2) {
            (Some(_), None) => tr!("tool-vcb-length"),
            (Some(_), Some(_)) => tr!("tool-vcb-angle"),
            (None, _) if self.last.is_some() => tr!("tool-vcb-angle"),
            _ => tr!("tool-vcb-segments"),
        }
    }
    fn vcb_value(&self) -> String {
        match (self.p1, self.p2) {
            (None, _) if self.last.is_none() => format!("{}s", self.segs()),
            (Some((a, _)), None) => self.cur.as_ref().map(|r| format!("{:.1}", a.distance(r.point))).unwrap_or_default(),
            (Some(_), Some(_)) => self.preview.map(|i| fmt_angle(i.sweep().to_degrees())).unwrap_or_default(),
            _ => String::new(),
        }
    }
    fn status_text(&self) -> String {
        match (self.p1, self.p2) {
            (None, _) => tr!("tool-arc3-first"),
            (Some(_), None) => tr!("tool-arc3-second"),
            _ => tr!("tool-arc3-third"),
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        match (self.p1, self.p2, &self.cur) {
            (Some((a, _)), None, Some(r)) => {
                line(out3d, a, r.point, PREVIEW, 1.5);
                label(out2d, (a + r.point) * 0.5, fmt_len(&view.doc.model, a.distance(r.point)));
            }
            (Some((a, _)), Some(b), Some(r)) => {
                match self.preview {
                    Some(info) => {
                        polyline(out3d, &info.points(), false, PREVIEW, 2.0);
                        label(out2d, info.center, format!("{}°", fmt_angle(info.sweep().to_degrees())));
                    }
                    None => polyline(out3d, &[a, b, r.point], false, PREVIEW, 1.5),
                }
                out2d.push(Marker2d::Point { at: b, shape: crate::overlay::MarkerShape::Dot, color: Color32::from_gray(60) });
            }
            _ => {}
        }
        if let Some(r) = &self.cur {
            inference::draw_result(r, out3d, out2d);
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        self.p1.is_some()
    }
}

// ===========================================================================
// Rotated Rectangle
// ===========================================================================

#[derive(Clone, Copy, Debug)]
struct RotRectSpec {
    p1: DVec3,
    p2: DVec3,
    /// Width vector (perpendicular to p1→p2).
    w: DVec3,
}

impl RotRectSpec {
    fn corners(&self) -> [DVec3; 4] {
        [self.p1, self.p2, self.p2 + self.w, self.p1 + self.w]
    }
}

/// Zero direction for widths around edge direction `e`: horizontal and perpendicular to the
/// edge (red axis for a vertical edge).
fn width_zero(e: DVec3) -> DVec3 {
    let ax = crate::axes::current();
    let z = ax.z.cross(e);
    if z.length() > 1e-6 { z.normalize() } else { ax.x }
}

/// Width direction at `deg` degrees from `width_zero(e)` about `e`.
fn width_dir(e: DVec3, deg: f64) -> DVec3 {
    let z = width_zero(e);
    let r = deg.to_radians();
    (z * r.cos() + e.cross(z) * r.sin()).normalize()
}

#[derive(Default)]
pub struct RotatedRectTool {
    engine: InferenceEngine,
    p1: Option<DVec3>,
    p2: Option<DVec3>,
    cur: Option<InferenceResult>,
    spec: Option<RotRectSpec>,
    /// Width axis colour / angle snap shown at the cursor.
    snap: Option<InferenceKind>,
    snapped_angle: bool,
    clicker: Clicker,
    last: Option<RotRectSpec>,
}

impl RotatedRectTool {
    fn update(&mut self, ctx: &ToolContext) {
        let anchor = self.p2.or(self.p1);
        let fallback = match (self.p1, self.p2) {
            // Width point moves around the edge: plane perpendicular to it through p2.
            (Some(a), Some(b)) => Some(Plane::new(b, (b - a).normalize())),
            (Some(a), None) => Some(Plane::new(a, crate::axes::current().z)),
            _ => None,
        };
        let r = infer(&mut self.engine, ctx, anchor, None, fallback);
        self.spec = None;
        self.snap = None;
        self.snapped_angle = false;
        if let (Some(p1), Some(p2)) = (self.p1, self.p2) {
            let e = (p2 - p1).normalize();
            let q = r.point - p2;
            let mut w = q - e * q.dot(e);
            if w.length() > 1e-9 && !r.is_geometry_snap() {
                let ang = signed_angle(width_zero(e), w, e);
                let (d, hit) = snap_angle(ang, SNAP_STEP, SNAP_TOL);
                if hit {
                    w = width_dir(e, d) * w.length();
                    self.snapped_angle = true;
                }
            }
            self.snap = axis_kind_of(w);
            self.spec = Some(RotRectSpec { p1, p2, w });
        }
        self.cur = Some(r);
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        let Some(r) = self.cur.clone() else { return };
        match (self.p1, self.p2) {
            (None, _) => self.p1 = Some(r.point),
            (Some(a), None) => {
                if a.distance(r.point) > TOL {
                    self.p2 = Some(r.point);
                    self.engine.arrow_lock = None;
                    self.update(ctx);
                }
            }
            (Some(_), Some(_)) => {
                if let Some(s) = self.spec.filter(|s| s.w.length() > TOL) {
                    self.commit(ctx, s);
                }
            }
        }
    }

    fn commit(&mut self, ctx: &mut ToolContext, s: RotRectSpec) {
        let pts: Vec<DVec3> = s.corners().iter().map(|p| to_local(ctx.doc, *p)).collect();
        if commit_mesh(ctx, &tr!("tool-undo-rotated-rectangle"), |m| m.add_face(&pts)).flatten().is_some() {
            self.last = Some(s);
        } else if ctx.doc.last_error.is_none() {
            ctx.doc.last_error = Some(tr!("tool-degenerate", label = tr!("tool-undo-rotated-rectangle")));
        }
        self.reset();
    }

    fn reset(&mut self) {
        self.p1 = None;
        self.p2 = None;
        self.spec = None;
        self.snap = None;
        self.snapped_angle = false;
    }

    /// Spec with width `w` (and optional angle in degrees) on edge p1→p2, defaulting the
    /// direction to the current preview's.
    fn with_width(&self, p1: DVec3, p2: DVec3, w: f64, deg: Option<f64>) -> RotRectSpec {
        let e = (p2 - p1).normalize();
        let dir = match deg {
            Some(d) => width_dir(e, d),
            None => self.spec.map(|s| s.w).filter(|w| w.length() > 1e-9).map_or(width_zero(e), |w| w.normalize()),
        };
        RotRectSpec { p1, p2, w: dir * w }
    }
}

impl Tool for RotatedRectTool {
    fn id(&self) -> ToolId {
        ToolId::RotatedRectangle
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
        if self.clicker.up_is_drag(ctx) && self.p1.is_some() {
            self.click(ctx);
        }
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
        let (s1, s2) = split2(input);
        match (self.p1, self.p2) {
            (Some(a), None) => {
                // `l` or `l,w`.
                let Some(l) = parse_len(unit, s1).filter(|v| v.abs() > TOL) else { return false };
                let Some(dir) = self.cur.as_ref().map(|r| r.point - a).filter(|d| d.length() > 1e-9) else {
                    return false;
                };
                let b = a + dir.normalize() * l;
                match s2 {
                    Some(t) => {
                        let Some(w) = parse_len(unit, t).filter(|v| v.abs() > TOL) else { return false };
                        let s = self.with_width(a, b, w, Some(0.0));
                        self.commit(ctx, s);
                    }
                    None => {
                        self.p2 = Some(b);
                        self.update(ctx);
                    }
                }
                true
            }
            (Some(a), Some(b)) => {
                // `w` or `w,angle`.
                let Some(w) = parse_len(unit, s1).filter(|v| v.abs() > TOL) else { return false };
                let deg = match s2 {
                    Some(t) => Some(match parse_angle(t) {
                        Some(d) => d,
                        None => return false,
                    }),
                    None => None,
                };
                let s = self.with_width(a, b, w, deg);
                self.commit(ctx, s);
                true
            }
            (None, _) => {
                let Some(last) = self.last else { return false };
                let Some(w) = parse_len(unit, s1).filter(|v| v.abs() > TOL) else { return false };
                let e = (last.p2 - last.p1).normalize();
                let deg = match s2 {
                    Some(t) => match parse_angle(t) {
                        Some(d) => d,
                        None => return false,
                    },
                    None => signed_angle(width_zero(e), last.w, e),
                };
                if !undo_last(ctx) {
                    return false;
                }
                self.commit(ctx, RotRectSpec { w: width_dir(e, deg) * w, ..last });
                true
            }
        }
    }
    fn vcb_label(&self) -> String {
        match (self.p1, self.p2) {
            (Some(_), None) => tr!("tool-vcb-length"),
            (Some(_), Some(_)) => tr!("tool-vcb-width-angle"),
            (None, _) if self.last.is_some() => tr!("tool-vcb-width-angle"),
            _ => tr!("tool-vcb-measurements"),
        }
    }
    fn vcb_value(&self) -> String {
        match (self.p1, self.p2, &self.cur, self.spec) {
            (Some(a), None, Some(r), _) => format!("{:.1}", a.distance(r.point)),
            (Some(_), Some(_), _, Some(s)) => {
                let e = (s.p2 - s.p1).normalize();
                format!("{:.1}, {}", s.w.length(), fmt_angle(signed_angle(width_zero(e), s.w, e)))
            }
            _ => String::new(),
        }
    }
    fn status_text(&self) -> String {
        match (self.p1, self.p2) {
            (None, _) => tr!("tool-rotrect-first"),
            (Some(_), None) => tr!("tool-rotrect-second"),
            _ => tr!("tool-rotrect-third"),
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        let model = &view.doc.model;
        match (self.p1, self.p2, &self.cur) {
            (Some(a), None, Some(r)) => {
                line(out3d, a, r.point, PREVIEW, 1.5);
                label(out2d, (a + r.point) * 0.5, fmt_len(model, a.distance(r.point)));
            }
            (Some(a), Some(b), _) => {
                if let Some(s) = self.spec {
                    let c = s.corners();
                    polyline(out3d, &c, true, PREVIEW, 2.0);
                    fill_polygon(out3d, &c, Color32::from_rgba_unmultiplied(0x40, 0x60, 0xA0, 0x30));
                    let wc = self.snap.map(InferenceKind::color).unwrap_or(Color32::from_gray(90));
                    line(out3d, b, b + s.w, wc, if self.snap.is_some() { 2.5 } else { 1.5 });
                    label(out2d, b + s.w * 0.5, fmt_len(model, s.w.length()));
                    // Protractor around the edge at p2.
                    let e = (b - a).normalize();
                    draw_protractor(view, b, e, Some(width_zero(e)), out3d);
                }
                label(out2d, (a + b) * 0.5, fmt_len(model, a.distance(b)));
            }
            _ => {}
        }
        if let Some(k) = self.snap.filter(|_| self.p2.is_some()) {
            out2d.push(Marker2d::CursorTooltip { text: k.label().into() });
        } else if self.snapped_angle {
            out2d.push(Marker2d::CursorTooltip { text: tr!("tool-cursor-angle") });
        } else if let Some(r) = &self.cur {
            inference::draw_result(r, out3d, out2d);
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        self.p1.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn unwrap_passes_half_turn() {
        let a = unwrap_angle(-170f64.to_radians(), 175f64.to_radians());
        assert!((a.to_degrees() - 190.0).abs() < 1e-9);
        assert!((unwrap_angle(0.5, 0.0) - 0.5).abs() < 1e-12);
        assert!(unwrap_angle(PI, TAU * 3.0) < TAU);
    }

    #[test]
    fn width_dirs() {
        let e = DVec3::X;
        assert!((width_zero(e) - DVec3::Y).length() < 1e-12 || (width_zero(e) + DVec3::Y).length() < 1e-12);
        let w = width_dir(e, 90.0);
        assert!(w.dot(e).abs() < 1e-12 && w.dot(width_zero(e)).abs() < 1e-12);
        assert_eq!(split2("10, 20"), ("10", Some("20")));
        assert_eq!(split2("10"), ("10", None));
    }
}
