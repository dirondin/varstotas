//! Edit Curve: reshape a curve drawn with Bezier Curve / Smooth Curve / Freehand (its anchors
//! are kept in [`varstotas_kernel::Mesh::splines`]). Drag an anchor (square) or a handle
//! (ring); a smooth anchor keeps its handles in line. A click on the curve between anchors
//! adds one (the shape stays), Delete removes the selected anchor, a double-click on an
//! anchor turns it corner ↔ smooth, `12s` sets the segments per span. Each change is one
//! undo step ([`varstotas_kernel::Mesh::set_spline`]); the drag shows a preview until release.

use super::common::*;
use super::{OverlayView, Tool, ToolContext, ToolId, ToolRequest};
use crate::inference::{self, Constraint, InferenceEngine, InferenceResult};
use crate::overlay::{Marker2d, MarkerShape};
use egui::{Color32, CursorIcon, Key, PointerButton, Pos2};
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::spline::{Anchor, smooth};
use varstotas_kernel::{EntityRef, Plane, Spline, TOL};
use varstotas_render::Overlay3d;

/// Screen distance (points) for grabbing an anchor or a handle.
const GRIP_PX: f32 = 8.0;
/// Screen distance (points) for a click on the curve itself.
const CURVE_PX: f32 = 6.0;
const HANDLE: Color32 = Color32::from_rgb(0x20, 0x70, 0xE0);
const HOT: Color32 = Color32::from_rgb(0xE0, 0x60, 0x10);

/// What can be dragged: an anchor or one of its handles (by anchor index).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Grip {
    Anchor(usize),
    In(usize),
    Out(usize),
}

impl Grip {
    fn anchor(self) -> usize {
        match self {
            Grip::Anchor(i) | Grip::In(i) | Grip::Out(i) => i,
        }
    }
}

struct Drag {
    grip: Grip,
    /// The curve being reshaped (context-local); committed on release.
    spline: Spline,
    press: Pos2,
    /// Where the grabbed point was at the press (context-local): inferences start there.
    from: DVec3,
    moved: bool,
    /// The anchor's handles were in line at the press: keep them so.
    smooth: bool,
}

#[derive(Default)]
pub struct EditCurveTool {
    engine: InferenceEngine,
    /// The curve (`Edge::curve` id in the active context).
    cid: Option<u32>,
    /// The selected anchor (Delete removes it).
    sel: Option<usize>,
    hover: Option<Grip>,
    drag: Option<Drag>,
    cur: Option<InferenceResult>,
}

/// Handles of anchor `a` point in opposite directions.
fn in_line(a: &Anchor) -> bool {
    match (a.h_in, a.h_out) {
        (Some(i), Some(o)) => {
            let (u, v) = (i - a.p, o - a.p);
            u.length() > TOL && v.length() > TOL && u.normalize().dot(v.normalize()) < -0.999
        }
        _ => false,
    }
}

/// Split span `i` (anchor `i` to the next) at `t`: a new anchor that keeps the curve's shape
/// (de Casteljau). Returns the new anchor's index.
fn split_span(sp: &mut Spline, i: usize, t: f64) -> usize {
    let n = sp.anchors.len();
    let (a, b) = (sp.anchors[i], sp.anchors[(i + 1) % n]);
    let mid = if a.h_out.is_none() && b.h_in.is_none() {
        Anchor::corner(a.p.lerp(b.p, t))
    } else {
        let (p0, p1, p2, p3) = (a.p, a.h_out.unwrap_or(a.p), b.h_in.unwrap_or(b.p), b.p);
        let (q0, q1, q2) = (p0.lerp(p1, t), p1.lerp(p2, t), p2.lerp(p3, t));
        let (r0, r1) = (q0.lerp(q1, t), q1.lerp(q2, t));
        let keep = |h: DVec3, at: DVec3| (h.distance(at) > TOL).then_some(h);
        sp.anchors[i].h_out = keep(q0, p0);
        sp.anchors[(i + 1) % n].h_in = keep(q2, p3);
        Anchor { p: r0.lerp(r1, t), h_in: Some(r0), h_out: Some(r1) }
    };
    sp.anchors.insert(i + 1, mid);
    i + 1
}

impl EditCurveTool {
    /// The curve as stored (context-local), while it still is one.
    fn spline(&self, doc: &crate::document::Document) -> Option<Spline> {
        doc.model.mesh_at(&doc.active_path)?.spline(self.cid?)
    }

    /// The curve shown: the one being dragged, else the stored one.
    fn shown(&self, doc: &crate::document::Document) -> Option<Spline> {
        match &self.drag {
            Some(d) => Some(d.spline.clone()),
            None => self.spline(doc),
        }
    }

    /// Start editing curve `cid` (selects its edges).
    fn take(&mut self, ctx: &mut ToolContext, cid: u32) {
        self.cid = Some(cid);
        self.sel = None;
        self.drag = None;
        if let Some(m) = ctx.doc.model.mesh_at(&ctx.doc.active_path) {
            let es = m.curve_edges(cid);
            ctx.doc.set_selection(es.into_iter().map(EntityRef::Edge).collect());
        }
    }

    fn grip_under(&self, ctx: &ToolContext, sp: &Spline) -> Option<Grip> {
        let xf = context_xf(ctx.doc);
        let at = ctx.pointer.pos;
        let near = |p: DVec3| ctx.to_screen(xf.transform_point3(p)).map(|s| s.distance(at)).filter(|d| *d <= GRIP_PX);
        let mut best: Option<(f32, Grip)> = None;
        let mut offer = |d: Option<f32>, g: Grip, bias: f32| {
            if let Some(d) = d.map(|d| d - bias)
                && best.is_none_or(|b| d < b.0)
            {
                best = Some((d, g));
            }
        };
        for (i, a) in sp.anchors.iter().enumerate() {
            // handles win a tie: they sit on top of a short-handled anchor
            offer(a.h_in.and_then(near), Grip::In(i), 0.5);
            offer(a.h_out.and_then(near), Grip::Out(i), 0.5);
            offer(near(a.p), Grip::Anchor(i), 0.0);
        }
        best.map(|b| b.1)
    }

    /// The span and parameter of the curve point nearest the pointer on screen.
    fn on_curve(&self, ctx: &ToolContext, sp: &Spline) -> Option<(usize, f64)> {
        let xf = context_xf(ctx.doc);
        let n = sp.anchors.len();
        let spans = if sp.closed { n } else { n - 1 };
        let mut best: Option<(f32, usize, f64)> = None;
        for i in 0..spans {
            let one = Spline { anchors: vec![sp.anchors[i], sp.anchors[(i + 1) % n]], closed: false, segs: 64, ..sp.clone() };
            let pts = one.points();
            let k = (pts.len() - 1).max(1) as f64;
            for (j, p) in pts.iter().enumerate() {
                let Some(s) = ctx.to_screen(xf.transform_point3(*p)) else { continue };
                let d = s.distance(ctx.pointer.pos);
                if best.is_none_or(|b| d < b.0) {
                    best = Some((d, i, j as f64 / k));
                }
            }
        }
        best.filter(|b| b.0 <= CURVE_PX && b.2 > 0.0 && b.2 < 1.0).map(|b| (b.1, b.2))
    }

    /// The pointer on the curve's plane (world), with snapping.
    fn drag_point(&mut self, ctx: &ToolContext, sp: &Spline, from: DVec3) -> Option<DVec3> {
        let xf = context_xf(ctx.doc);
        let n = xf.inverse().transpose().transform_vector3(sp.normal).try_normalize()?;
        let plane = Plane::new(xf.transform_point3(sp.anchors[0].p), n);
        let r = infer(&mut self.engine, ctx, Some(xf.transform_point3(from)), Some(Constraint::Plane(plane)), Some(plane));
        let p = plane.project(r.point);
        self.cur = Some(r);
        p.is_finite().then(|| xf.inverse().transform_point3(p))
    }

    fn apply_drag(&mut self, ctx: &ToolContext) {
        let Some(d) = &self.drag else { return };
        let (grip, smooth_at_press, mut sp, from) = (d.grip, d.smooth, d.spline.clone(), d.from);
        let i = grip.anchor();
        let Some(q) = self.drag_point(ctx, &sp, from) else { return };
        let a = &mut sp.anchors[i];
        match grip {
            Grip::Anchor(_) => {
                let dv = q - a.p;
                a.p = q;
                a.h_in = a.h_in.map(|h| h + dv);
                a.h_out = a.h_out.map(|h| h + dv);
            }
            Grip::In(_) | Grip::Out(_) => {
                let (this, other) = if matches!(grip, Grip::In(_)) { (&mut a.h_in, &mut a.h_out) } else { (&mut a.h_out, &mut a.h_in) };
                if q.distance(a.p) <= TOL {
                    *this = None;
                } else {
                    *this = Some(q);
                    if smooth_at_press && let Some(o) = *other {
                        let len = o.distance(a.p);
                        *other = Some(a.p - (q - a.p).normalize() * len);
                    }
                }
            }
        }
        if let Some(d) = self.drag.as_mut() {
            d.spline = sp;
        }
    }

    /// Commit a reshaped curve (one undo step); keeps it selected.
    fn commit(&mut self, ctx: &mut ToolContext, sp: &Spline, label: &str) -> bool {
        let Some(cid) = self.cid else { return false };
        match try_commit_mesh(ctx, label, |m| m.set_spline(cid, sp)) {
            Some(es) => {
                ctx.doc.set_selection(es.into_iter().map(EntityRef::Edge).collect());
                true
            }
            None => false,
        }
    }

    fn finish(&mut self, ctx: &mut ToolContext) {
        self.drag = None;
        ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select));
    }

    fn min_anchors(sp: &Spline) -> usize {
        if sp.closed { 3 } else { 2 }
    }
}

impl Tool for EditCurveTool {
    fn id(&self) -> ToolId {
        ToolId::EditCurve
    }

    fn activate(&mut self, ctx: &mut ToolContext) {
        let Some(m) = ctx.doc.model.mesh_at(&ctx.doc.active_path) else { return };
        let cid = ctx.doc.selection.iter().find_map(|e| match e {
            EntityRef::Edge(e) => m.spline_of_edge(*e).map(|x| x.0),
            _ => None,
        });
        if let Some(c) = cid {
            self.take(ctx, c);
        }
    }

    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        if let Some(d) = self.drag.as_mut() {
            if !ctx.pointer.primary_down {
                return;
            }
            if !d.moved && (ctx.pointer.pos - d.press).length() <= DRAG_CLICK_PX {
                return;
            }
            d.moved = true;
            self.apply_drag(ctx);
            return;
        }
        self.cur = None;
        self.hover = self.spline(ctx.doc).and_then(|sp| self.grip_under(ctx, &sp));
    }

    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button != Some(PointerButton::Primary) {
            return;
        }
        if let Some(sp) = self.spline(ctx.doc) {
            if let Some(g) = self.grip_under(ctx, &sp) {
                self.sel = Some(g.anchor());
                let a = sp.anchors[g.anchor()];
                let from = match g {
                    Grip::Anchor(_) => a.p,
                    Grip::In(_) => a.h_in.unwrap_or(a.p),
                    Grip::Out(_) => a.h_out.unwrap_or(a.p),
                };
                self.drag = Some(Drag { grip: g, spline: sp, press: ctx.pointer.pos, from, moved: false, smooth: in_line(&a) });
                return;
            }
            if let Some((i, t)) = self.on_curve(ctx, &sp) {
                let mut sp = sp;
                let k = split_span(&mut sp, i, t);
                if self.commit(ctx, &sp, "Add Curve Point") {
                    self.sel = Some(k);
                }
                return;
            }
        }
        // Another curve: edit that one; empty space: done.
        let other = match pick_edge_at(ctx, ctx.pointer.pos, CURVE_PX) {
            Some(Hover::Edge { edge, .. }) => ctx.doc.model.mesh_at(&ctx.doc.active_path).and_then(|m| m.spline_of_edge(edge)).map(|x| x.0),
            _ => None,
        };
        match other {
            Some(c) => self.take(ctx, c),
            None => self.finish(ctx),
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button != Some(PointerButton::Primary) {
            return;
        }
        if self.drag.as_ref().is_some_and(|d| d.moved) {
            self.apply_drag(ctx);
        }
        let Some(d) = self.drag.take() else { return };
        self.cur = None;
        if d.moved {
            self.commit(ctx, &d.spline, "Edit Curve");
        }
    }

    fn on_double_click(&mut self, ctx: &mut ToolContext) {
        let Some(mut sp) = self.spline(ctx.doc) else { return };
        let Some(Grip::Anchor(i)) = self.grip_under(ctx, &sp) else { return };
        let n = sp.anchors.len();
        let a = sp.anchors[i];
        if a.is_corner() {
            // smooth: handles along the neighbours (an open end: toward its neighbour)
            let s = smooth(&sp.anchors, sp.closed)[i];
            sp.anchors[i] = if s.is_corner() {
                let nb = if i == 0 { sp.anchors[1].p } else { sp.anchors[n - 2].p };
                let h = a.p + (nb - a.p) / 3.0;
                if i == 0 { Anchor { h_out: Some(h), ..a } } else { Anchor { h_in: Some(h), ..a } }
            } else {
                s
            };
        } else {
            sp.anchors[i] = Anchor::corner(a.p);
        }
        self.drag = None;
        self.sel = Some(i);
        self.commit(ctx, &sp, "Edit Curve");
    }

    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        match key {
            Key::Escape if self.drag.is_some() => {
                self.drag = None;
                self.cur = None;
                true
            }
            Key::Escape | Key::Enter => {
                self.finish(ctx);
                true
            }
            Key::Delete | Key::Backspace => {
                // never the Edit → Delete of the selected curve
                if let (Some(i), Some(mut sp)) = (self.sel, self.spline(ctx.doc))
                    && self.drag.is_none()
                    && i < sp.anchors.len()
                {
                    if sp.anchors.len() <= Self::min_anchors(&sp) {
                        ctx.doc.last_error = Some("Edit Curve: too few points to remove one".into());
                    } else {
                        sp.anchors.remove(i);
                        if self.commit(ctx, &sp, "Remove Curve Point") {
                            self.sel = None;
                        }
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        let (Some(n), Some(mut sp)) = (parse_segments(input), self.spline(ctx.doc)) else { return false };
        sp.segs = n;
        self.commit(ctx, &sp, "Edit Segments")
    }

    fn vcb_label(&self) -> &str {
        "Segments"
    }

    fn vcb_value(&self) -> String {
        self.drag.as_ref().map_or(String::new(), |d| format!("{}s", d.spline.segs))
    }

    fn status_text(&self) -> String {
        if self.cid.is_none() {
            return "Click a curve drawn with Bezier Curve, Smooth Curve or Freehand.".into();
        }
        if self.drag.is_some() {
            return "Drag to reshape; release to apply, Esc cancels.".into();
        }
        "Drag points and handles; click the curve to add a point, Delete removes the selected point; \
         double-click a point: corner / smooth. Enter finishes."
            .into()
    }

    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        let Some(sp) = self.shown(view.doc) else { return };
        let xf: DMat4 = context_xf(view.doc);
        let w = |p: DVec3| xf.transform_point3(p);
        if self.drag.as_ref().is_some_and(|d| d.moved) {
            let pts: Vec<DVec3> = sp.points().into_iter().map(w).collect();
            polyline(out3d, &pts, sp.closed, PREVIEW, 2.0);
        }
        for (i, a) in sp.anchors.iter().enumerate() {
            for (h, g) in [(a.h_in, Grip::In(i)), (a.h_out, Grip::Out(i))] {
                let Some(h) = h else { continue };
                line(out3d, w(a.p), w(h), HANDLE, 1.0);
                let color = if self.hover == Some(g) { HOT } else { HANDLE };
                out2d.push(Marker2d::Point { at: w(h), shape: MarkerShape::Ring, color });
            }
            let color = if self.hover == Some(Grip::Anchor(i)) {
                HOT
            } else if self.sel == Some(i) {
                SELECT_BLUE
            } else {
                Color32::from_gray(60)
            };
            out2d.push(Marker2d::Point { at: w(a.p), shape: MarkerShape::Square, color });
        }
        if let Some(r) = &self.cur {
            inference::draw_result(r, out3d, out2d);
        }
    }

    fn cursor(&self) -> CursorIcon {
        match (&self.drag, self.hover) {
            (Some(_), _) => CursorIcon::Grabbing,
            (None, Some(_)) => CursorIcon::Grab,
            _ => CursorIcon::Default,
        }
    }

    fn is_busy(&self) -> bool {
        self.drag.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_keeps_the_shape() {
        let sp = Spline {
            anchors: vec![
                Anchor { p: DVec3::ZERO, h_in: None, h_out: Some(DVec3::new(0.0, 1.0, 0.0)) },
                Anchor { p: DVec3::new(2.0, 0.0, 0.0), h_in: Some(DVec3::new(2.0, 1.0, 0.0)), h_out: None },
            ],
            closed: false,
            segs: 8,
            normal: DVec3::Z,
        };
        let mut s2 = sp.clone();
        assert_eq!(split_span(&mut s2, 0, 0.5), 1);
        assert_eq!(s2.anchors.len(), 3);
        // the new anchor is the curve's middle, and the halves trace the same curve
        assert!(s2.anchors[1].p.distance(DVec3::new(1.0, 0.75, 0.0)) < 1e-12);
        let before = Spline { segs: 16, ..sp }.points();
        let after = s2.points();
        assert!(after.iter().all(|p| before.iter().any(|q| q.distance(*p) < 1e-9)));
        // a straight span splits into two corners
        let mut l = Spline { anchors: vec![Anchor::corner(DVec3::ZERO), Anchor::corner(DVec3::X)], closed: false, segs: 4, normal: DVec3::Z };
        split_span(&mut l, 0, 0.25);
        assert_eq!(l.points(), vec![DVec3::ZERO, DVec3::new(0.25, 0.0, 0.0), DVec3::X]);
    }

    #[test]
    fn handles_in_line() {
        let p = DVec3::ZERO;
        assert!(in_line(&Anchor { p, h_in: Some(-DVec3::X), h_out: Some(DVec3::X * 3.0) }));
        assert!(!in_line(&Anchor { p, h_in: Some(DVec3::Y), h_out: Some(DVec3::X) }));
        assert!(!in_line(&Anchor::corner(p)));
    }
}
