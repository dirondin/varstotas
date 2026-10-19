//! Curve tools: anchors with handles (cubic spans), committed as one curve of edges. Three
//! tools, one per way to place the anchors ([`Method`]): Bezier Curve — a click places a
//! corner anchor, press and drag pulls out its handles (symmetric, the outgoing one under the
//! cursor); Smooth Curve — clicks only, the handles follow the neighbours (Catmull-Rom);
//! Freehand — press and drag a stroke, smoothed on release. Enter / double-click finishes an
//! open curve (edges only); a click on the first anchor (a stroke ended at its start) closes
//! it into a face. Backspace takes back the last anchor, Escape drops it all.

use super::common::*;
use super::draw::{plane_point, shape_normal};
use super::{OverlayView, Tool, ToolContext, ToolId};
use crate::inference::{self, InferenceEngine, InferenceResult};
use crate::overlay::{Marker2d, MarkerShape};
use egui::{Color32, CursorIcon, Key, PointerButton, Pos2};
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{Plane, TOL};
use varstotas_render::Overlay3d;

/// Default edges per span between two anchors.
const BEZIER_SEGS: u32 = 12;
/// Screen distance (points) at which a click on the first anchor closes the curve.
const CLOSE_PX: f32 = 10.0;
/// Freehand: screen distance between the kept points of a stroke, and edges per span.
const FREEHAND_PX: f32 = 12.0;
const FREEHAND_SEGS: u32 = 3;
/// Freehand: the stroke ends this close (points) to its start = closed (a face).
const FREEHAND_CLOSE_PX: f32 = 16.0;
const HANDLE: Color32 = Color32::from_rgb(0x20, 0x70, 0xE0);

/// How the anchors are placed (one tool each).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Method {
    #[default]
    Handles,
    Smooth,
    Freehand,
}

/// One anchor of the curve; handles are absolute points (None = a corner on that side).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    pub p: DVec3,
    pub h_in: Option<DVec3>,
    pub h_out: Option<DVec3>,
}

impl Anchor {
    fn corner(p: DVec3) -> Self {
        Self { p, h_in: None, h_out: None }
    }
}

fn cubic(p0: DVec3, p1: DVec3, p2: DVec3, p3: DVec3, t: f64) -> DVec3 {
    let u = 1.0 - t;
    p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
}

/// The anchors with handles that pass smoothly through them (Catmull-Rom: each handle a
/// sixth of the chord between the neighbours); the ends of an open curve stay corners.
pub fn smooth(anchors: &[Anchor], closed: bool) -> Vec<Anchor> {
    let n = anchors.len();
    (0..n)
        .map(|i| {
            let p = anchors[i].p;
            let (prev, next) = if closed && n >= 3 {
                (anchors[(i + n - 1) % n].p, anchors[(i + 1) % n].p)
            } else if i == 0 || i + 1 == n {
                return Anchor::corner(p);
            } else {
                (anchors[i - 1].p, anchors[i + 1].p)
            };
            let t = (next - prev) / 6.0;
            Anchor { p, h_in: Some(p - t), h_out: Some(p + t) }
        })
        .collect()
}

/// Points of the curve through `anchors` (`segs` edges per curved span, one edge for a span
/// without handles); closed curves add the span from the last anchor back to the first and
/// do not repeat the first point.
pub fn bezier_points(anchors: &[Anchor], closed: bool, segs: u32) -> Vec<DVec3> {
    let Some(first) = anchors.first() else { return Vec::new() };
    let mut out = vec![first.p];
    let n = anchors.len();
    let spans = if closed && n >= 2 { n } else { n.saturating_sub(1) };
    for i in 0..spans {
        let (a, b) = (anchors[i], anchors[(i + 1) % n]);
        if a.h_out.is_none() && b.h_in.is_none() {
            out.push(b.p);
            continue;
        }
        let (p1, p2) = (a.h_out.unwrap_or(a.p), b.h_in.unwrap_or(b.p));
        for k in 1..=segs.max(1) {
            out.push(cubic(a.p, p1, p2, b.p, k as f64 / segs.max(1) as f64));
        }
    }
    if closed && out.len() > 2 {
        out.pop();
    }
    out
}

#[derive(Default)]
pub struct BezierTool {
    engine: InferenceEngine,
    segs: Option<u32>,
    anchors: Vec<Anchor>,
    /// How the anchors are placed (which of the three tools this is).
    method: Method,
    /// Normal of the curve's plane (fixed by the first anchor: a face or an arrow lock).
    normal: Option<DVec3>,
    /// Pressed on a new anchor: dragging pulls out its handles.
    drag: Option<Pos2>,
    /// Freehand: the stroke being drawn (points and where they are on screen).
    stroke: Vec<(DVec3, Pos2)>,
    cur: Option<InferenceResult>,
    /// The cursor is on the first anchor (a click closes the curve).
    closing: bool,
}

impl BezierTool {
    pub fn new(method: Method) -> Self {
        Self { method, ..Default::default() }
    }

    fn segs(&self) -> u32 {
        self.segs.unwrap_or(BEZIER_SEGS)
    }

    fn plane_at(&self, p: DVec3) -> Plane {
        Plane::new(p, self.normal.unwrap_or_else(|| crate::axes::current().z))
    }

    fn update(&mut self, ctx: &ToolContext) {
        let last = self.anchors.last().map(|a| a.p);
        let fallback = last.map(|p| self.plane_at(p));
        let r = infer(&mut self.engine, ctx, last, None, fallback);
        self.closing = self.anchors.len() >= 2
            && ctx.to_screen(self.anchors[0].p).is_some_and(|s| (s - ctx.pointer.pos).length() <= CLOSE_PX);
        self.cur = Some(r);
    }

    /// Drag of the handles of the anchor just placed.
    fn drag_handle(&mut self, ctx: &ToolContext) {
        let Some(a) = self.anchors.last().copied() else { return };
        let plane = self.plane_at(a.p);
        let r = infer(&mut self.engine, ctx, Some(a.p), None, Some(plane));
        let h = plane_point(ctx, &r, &plane);
        let last = self.anchors.last_mut().unwrap();
        if h.distance(a.p) > TOL {
            last.h_out = Some(h);
            last.h_in = Some(a.p * 2.0 - h);
        } else {
            last.h_out = None;
            last.h_in = None;
        }
        self.cur = Some(r);
    }

    /// Freehand: the stroke follows the pointer on the plane of its first point.
    fn extend_stroke(&mut self, ctx: &ToolContext) {
        let Some(&(p0, _)) = self.stroke.first() else { return };
        let plane = self.plane_at(p0);
        let ray = ctx.pointer.ray;
        let Some(t) = plane.intersect_ray(&ray).filter(|t| *t > 0.0) else { return };
        let p = ray.at(t);
        let pos = ctx.pointer.pos;
        // back at the start: the stroke closes (into a face)
        self.closing = self.stroke.len() >= 3 && (self.stroke[0].1 - pos).length() <= FREEHAND_CLOSE_PX;
        if p.is_finite() && self.stroke.last().is_some_and(|&(q, s)| (s - pos).length() >= FREEHAND_PX && q.distance(p) > TOL) {
            self.stroke.push((p, pos));
        }
    }

    fn add_anchor(&mut self, p: DVec3) -> bool {
        if self.anchors.last().is_some_and(|a| a.p.distance(p) <= TOL) {
            return false;
        }
        self.anchors.push(Anchor::corner(p));
        true
    }

    /// The anchors as committed: Smooth and Freehand get their handles here.
    fn shaped(&self, anchors: &[Anchor], closed: bool) -> Vec<Anchor> {
        match self.method {
            Method::Handles => anchors.to_vec(),
            Method::Smooth | Method::Freehand => smooth(anchors, closed),
        }
    }

    fn span_segs(&self) -> u32 {
        if self.method == Method::Freehand { FREEHAND_SEGS } else { self.segs() }
    }

    /// Commits the curve; `closed` = the first point was clicked: a loop with a face.
    fn finish(&mut self, ctx: &mut ToolContext, closed: bool) {
        if self.anchors.len() >= 2 {
            let anchors = self.shaped(&self.anchors, closed);
            let curved = anchors.iter().any(|a| a.h_in.is_some() || a.h_out.is_some());
            let inv = context_xf(ctx.doc).inverse();
            let pts: Vec<DVec3> =
                bezier_points(&anchors, closed, self.span_segs()).iter().map(|p| inv.transform_point3(*p)).collect();
            let closed = closed && pts.len() >= 3;
            if pts.len() >= 2 {
                commit_mesh(ctx, self.id().name(), |m| m.add_polyline(&pts, closed, curved));
            }
        }
        self.reset();
    }

    /// Freehand: the stroke becomes anchors and is committed.
    fn finish_stroke(&mut self, ctx: &mut ToolContext) {
        let mut stroke = std::mem::take(&mut self.stroke);
        let closes = self.closing && stroke.len() >= 3;
        if closes && (stroke[0].1 - stroke[stroke.len() - 1].1).length() <= FREEHAND_CLOSE_PX {
            stroke.pop();
        }
        self.anchors = stroke.iter().map(|&(p, _)| Anchor::corner(p)).collect();
        self.finish(ctx, closes);
    }

    fn reset(&mut self) {
        self.anchors.clear();
        self.stroke.clear();
        self.normal = None;
        self.drag = None;
        self.closing = false;
        self.engine.clear_locks();
    }

    /// The curve so far, with the cursor as the next (corner) anchor.
    fn preview(&self) -> Vec<DVec3> {
        if self.method == Method::Freehand {
            return self.stroke.iter().map(|s| s.0).collect();
        }
        let mut a = self.anchors.clone();
        if self.drag.is_none()
            && let Some(r) = &self.cur
            && !self.closing
        {
            a.push(Anchor::corner(r.point));
        }
        let closed = self.closing;
        bezier_points(&self.shaped(&a, closed), closed, self.segs())
    }

    fn busy(&self) -> bool {
        !self.anchors.is_empty() || !self.stroke.is_empty()
    }
}

impl Tool for BezierTool {
    fn id(&self) -> ToolId {
        match self.method {
            Method::Handles => ToolId::Bezier,
            Method::Smooth => ToolId::SmoothCurve,
            Method::Freehand => ToolId::Freehand,
        }
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        if !self.stroke.is_empty() {
            if ctx.pointer.primary_down {
                self.extend_stroke(ctx);
            }
            return;
        }
        match self.drag {
            Some(p0) if ctx.pointer.primary_down && (ctx.pointer.pos - p0).length() > DRAG_CLICK_PX => self.drag_handle(ctx),
            Some(_) if ctx.pointer.primary_down => {}
            _ => self.update(ctx),
        }
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button != Some(PointerButton::Primary) {
            return;
        }
        self.update(ctx);
        let Some(r) = self.cur.clone() else { return };
        if self.method == Method::Freehand {
            let (n, fixed) = shape_normal(&self.engine, &r);
            self.normal = fixed.then_some(n);
            self.stroke = vec![(r.point, ctx.pointer.pos)];
            self.engine.arrow_lock = None;
            return;
        }
        if self.closing {
            self.finish(ctx, true);
            return;
        }
        if self.anchors.is_empty() {
            let (n, fixed) = shape_normal(&self.engine, &r);
            self.normal = fixed.then_some(n);
        }
        if self.add_anchor(r.point) {
            // Smooth: clicks only, no handles to drag out
            if self.method == Method::Handles {
                self.drag = Some(ctx.pointer.pos);
            }
            self.engine.arrow_lock = None;
        }
    }
    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button != Some(PointerButton::Primary) {
            return;
        }
        if !self.stroke.is_empty() {
            self.extend_stroke(ctx);
            self.finish_stroke(ctx);
            self.update(ctx);
        } else if self.drag.take().is_some() {
            self.update(ctx);
        }
    }
    fn on_double_click(&mut self, ctx: &mut ToolContext) {
        if self.anchors.len() >= 2 && self.method != Method::Freehand {
            self.finish(ctx, false);
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        match key {
            Key::Escape => {
                self.reset();
                true
            }
            Key::Enter if !self.anchors.is_empty() => {
                self.finish(ctx, false);
                true
            }
            Key::Backspace if !self.anchors.is_empty() => {
                self.anchors.pop();
                if self.anchors.is_empty() {
                    self.reset();
                }
                self.update(ctx);
                true
            }
            _ => {
                let r = self.engine.handle_arrow(key);
                if r {
                    self.update(ctx);
                }
                r
            }
        }
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        if let Some(n) = parse_segments(input) {
            self.segs = Some(n);
            return true;
        }
        // a length: the next corner anchor that far toward the cursor
        let Some(last) = self.anchors.last().map(|a| a.p) else { return false };
        let Some(v) = parse_len(ctx.doc.model.unit, input).filter(|v| *v > TOL) else { return false };
        let Some(dir) = self.cur.as_ref().map(|r| r.point - last).filter(|d| d.length() > 1e-9) else { return false };
        self.add_anchor(last + dir.normalize() * v);
        self.update(ctx);
        true
    }
    fn vcb_label(&self) -> &str {
        if self.anchors.is_empty() { "Segments" } else { "Length" }
    }
    fn vcb_value(&self) -> String {
        match (self.anchors.last(), &self.cur) {
            (Some(a), Some(r)) if self.drag.is_none() => format!("{:.1}", a.p.distance(r.point)),
            (None, _) => format!("{}s", self.segs()),
            _ => String::new(),
        }
    }
    fn status_text(&self) -> String {
        let end = "Enter or double-click finishes, a click on the first point closes it into a face";
        match self.method {
            Method::Freehand if self.stroke.is_empty() => "Press and drag to draw; end at the start to close it into a face.".into(),
            Method::Freehand => "Drag to draw; release to finish.".into(),
            Method::Smooth if self.anchors.is_empty() => "Click the first point of the curve.".into(),
            Method::Smooth => format!("Click the next point; {end}. Backspace removes the last point."),
            Method::Handles if self.anchors.is_empty() => "Click the first point of the curve; press and drag to pull out handles.".into(),
            Method::Handles if self.drag.is_some() => "Drag to shape the curve through this point; release to place it.".into(),
            Method::Handles => format!("Click the next point (drag for handles); {end}. Backspace removes the last point."),
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        let pts = self.preview();
        if pts.len() >= 2 {
            let closed = self.closing;
            polyline(out3d, &pts, closed, PREVIEW, 2.0);
        }
        for (i, a) in self.anchors.iter().enumerate() {
            for h in [a.h_in, a.h_out].into_iter().flatten() {
                line(out3d, a.p, h, HANDLE, 1.0);
                out2d.push(Marker2d::Point { at: h, shape: MarkerShape::Ring, color: HANDLE });
            }
            let first_hot = i == 0 && self.closing;
            let color = if first_hot { Color32::from_rgb(0x10, 0xA0, 0x40) } else { Color32::from_gray(60) };
            out2d.push(Marker2d::Point { at: a.p, shape: MarkerShape::Square, color });
        }
        if let (Some(a), Some(r)) = (self.anchors.last(), &self.cur)
            && self.drag.is_none()
            && !self.closing
        {
            label(out2d, (a.p + r.point) * 0.5, fmt_len(&view.doc.model, a.p.distance(r.point)));
        }
        if let Some(r) = &self.cur
            && self.stroke.is_empty()
        {
            inference::draw_result(r, out3d, out2d);
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        self.busy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_make_a_polyline() {
        let a = [Anchor::corner(DVec3::ZERO), Anchor::corner(DVec3::X), Anchor::corner(DVec3::Y)];
        assert_eq!(bezier_points(&a, false, 12), vec![DVec3::ZERO, DVec3::X, DVec3::Y]);
        assert_eq!(bezier_points(&a, true, 12).len(), 3);
    }

    #[test]
    fn handles_bend_the_span() {
        let a = [
            Anchor { p: DVec3::ZERO, h_in: None, h_out: Some(DVec3::new(0.0, 1.0, 0.0)) },
            Anchor { p: DVec3::new(2.0, 0.0, 0.0), h_in: Some(DVec3::new(2.0, 1.0, 0.0)), h_out: None },
        ];
        let pts = bezier_points(&a, false, 4);
        assert_eq!(pts.len(), 5);
        assert!(pts[0].distance(DVec3::ZERO) < 1e-12 && pts[4].distance(DVec3::new(2.0, 0.0, 0.0)) < 1e-12);
        // the middle of a symmetric span: x = 1, y = 3/4 of the handle height
        assert!(pts[2].distance(DVec3::new(1.0, 0.75, 0.0)) < 1e-12);
    }

    #[test]
    fn smooth_handles_follow_the_neighbours() {
        let a = [Anchor::corner(DVec3::ZERO), Anchor::corner(DVec3::new(1.0, 1.0, 0.0)), Anchor::corner(DVec3::new(2.0, 0.0, 0.0))];
        let s = smooth(&a, false);
        assert_eq!((s[0].h_out, s[2].h_in), (None, None));
        assert_eq!(s[1].h_out, Some(DVec3::new(1.0 + 1.0 / 3.0, 1.0, 0.0)));
        assert_eq!(s[1].h_in, Some(DVec3::new(1.0 - 1.0 / 3.0, 1.0, 0.0)));
        // closed: every anchor gets handles, and the curve passes through the anchors
        let c = smooth(&a, true);
        assert!(c.iter().all(|x| x.h_in.is_some() && x.h_out.is_some()));
        let pts = bezier_points(&c, true, 4);
        assert_eq!(pts.len(), 12);
        assert!(pts[4].distance(a[1].p) < 1e-12);
    }

    #[test]
    fn closed_curve_does_not_repeat_the_start() {
        let a = [
            Anchor { p: DVec3::ZERO, h_in: Some(DVec3::new(-1.0, 0.0, 0.0)), h_out: Some(DVec3::X) },
            Anchor::corner(DVec3::new(2.0, 2.0, 0.0)),
        ];
        let pts = bezier_points(&a, true, 6);
        assert_eq!(pts.len(), 12);
        assert!(pts.iter().skip(1).all(|p| p.distance(DVec3::ZERO) > 1e-9));
    }
}
