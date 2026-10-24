//! Dimension tool (linear dimensions).

use varstotas_i18n::tr;
use super::common::*;
use super::draw::Clicker;
use super::{OverlayView, Tool, ToolContext, ToolId};
use crate::inference::{self, InferenceEngine, InferenceKind, InferenceResult, axis_kind_of};
use crate::overlay::Marker2d;
use egui::{CursorIcon, Key};
use varstotas_kernel::TOL;
use varstotas_kernel::glam::DVec3;
use varstotas_render::Overlay3d;

#[derive(Clone, Copy, Debug)]
enum Stage {
    Idle,
    Start { a: DVec3 },
    Offset { a: DVec3, b: DVec3 },
}

pub struct DimensionTool {
    engine: InferenceEngine,
    stage: Stage,
    cur: Option<InferenceResult>,
    /// Hovered edge (clicking it dimensions its length).
    hover_edge: Option<(DVec3, DVec3)>,
    offset: DVec3,
    offset_axis: Option<InferenceKind>,
    clicker: Clicker,
}

impl Default for DimensionTool {
    fn default() -> Self {
        Self {
            engine: InferenceEngine::default(),
            stage: Stage::Idle,
            cur: None,
            hover_edge: None,
            offset: DVec3::ZERO,
            offset_axis: None,
            clicker: Clicker::default(),
        }
    }
}

/// The plane through the measured line `a`–`b` that faces the camera most.
fn offset_plane(ctx: &ToolContext, a: DVec3, b: DVec3) -> varstotas_kernel::Plane {
    let d = (b - a).normalize_or_zero();
    let view = ctx.pointer.ray.dir;
    let n = d.cross(view.cross(d)).normalize_or_zero();
    varstotas_kernel::Plane::new((a + b) * 0.5, if n == DVec3::ZERO { view } else { n })
}

/// Offset of the dimension line for the cursor ray: in the plane containing the measured line
/// that faces the camera most, perpendicular to the line; snapped to an axis direction when
/// within ~10°.
pub fn dim_offset(ctx: &ToolContext, a: DVec3, b: DVec3) -> (DVec3, Option<InferenceKind>) {
    let d = (b - a).normalize_or_zero();
    if d == DVec3::ZERO {
        return (DVec3::ZERO, None);
    }
    let ray = ctx.pointer.ray;
    let m = (a + b) * 0.5;
    let plane = offset_plane(ctx, a, b);
    let q = match plane.intersect_ray(&ray) {
        Some(t) if t > 0.0 => ray.at(t),
        _ => return (DVec3::ZERO, None),
    };
    let off = (q - m) - d * (q - m).dot(d);
    let len = off.length();
    if len < 1e-9 {
        return (off, None);
    }
    // Axis snap: the axis directions perpendicular to the line, by angle.
    let ax = crate::axes::current();
    let mut best: Option<(f64, DVec3)> = None;
    for e in ax.dirs() {
        if e.dot(d).abs() > 1e-6 {
            continue;
        }
        for e in [e, -e] {
            let cos = e.dot(off / len);
            if cos > 10f64.to_radians().cos() && best.is_none_or(|b| cos > b.0) {
                best = Some((cos, e));
            }
        }
    }
    match best {
        Some((cos, e)) => (e * len * cos, axis_kind_of(e)),
        None => (off, None),
    }
}

impl DimensionTool {
    fn update(&mut self, ctx: &ToolContext) {
        self.hover_edge = None;
        match self.stage {
            Stage::Idle | Stage::Start { .. } => {
                let anchor = match self.stage {
                    Stage::Start { a } => Some(a),
                    _ => None,
                };
                let r = infer(&mut self.engine, ctx, anchor, None, None);
                // Clicking an edge (on it or at its midpoint) dimensions its length; endpoints
                // start a point-to-point dimension.
                if matches!(self.stage, Stage::Idle) && matches!(r.kind, InferenceKind::OnEdge | InferenceKind::Midpoint) {
                    self.hover_edge = r.edge;
                }
                self.cur = Some(r);
            }
            Stage::Offset { a, b } => {
                // Full inference from the line's midpoint (points, edges, faces, guides, axes,
                // Shift / arrow locks); free space falls back to the camera-facing plane.
                let m = (a + b) * 0.5;
                let r = infer(&mut self.engine, ctx, Some(m), None, Some(offset_plane(ctx, a, b)));
                let d = (b - a).normalize_or_zero();
                let v = r.point - m;
                let off = v - d * v.dot(d);
                // A snap along the measured line itself (e.g. its own axis) gives no offset.
                if r.kind == InferenceKind::Free || off.length() < 1e-6 * (1.0 + a.distance(b)) {
                    let (o, k) = dim_offset(ctx, a, b);
                    self.offset = o;
                    self.offset_axis = k;
                    self.cur = None;
                } else {
                    self.offset = off;
                    self.offset_axis = None;
                    self.cur = Some(r);
                }
            }
        }
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        match self.stage {
            Stage::Idle => {
                if let Some((a, b)) = self.hover_edge {
                    self.stage = Stage::Offset { a, b };
                } else if let Some(r) = &self.cur {
                    self.stage = Stage::Start { a: r.point };
                }
            }
            Stage::Start { a } => {
                if let Some(r) = &self.cur
                    && r.point.distance(a) > TOL
                {
                    self.stage = Stage::Offset { a, b: r.point };
                }
            }
            Stage::Offset { a, b } => {
                let path = ctx.doc.active_path.clone();
                let off = self.offset;
                ctx.doc.commit(&tr!("tool-undo-dimension"), |m| m.add_dimension_in(&path, a, b, off));
                self.stage = Stage::Idle;
            }
        }
        self.update(ctx);
    }
}

impl Tool for DimensionTool {
    fn id(&self) -> ToolId {
        ToolId::Dimension
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
        if self.clicker.up_is_drag(ctx) && !matches!(self.stage, Stage::Idle) {
            self.click(ctx);
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        if key == Key::Escape {
            self.stage = Stage::Idle;
            self.engine.clear_locks();
            return true;
        }
        let r = self.engine.handle_arrow(key);
        if r {
            self.update(ctx);
        }
        r
    }
    fn status_text(&self) -> String {
        match self.stage {
            Stage::Idle => tr!("tool-dimension-first"),
            Stage::Start { .. } => tr!("tool-dimension-second"),
            Stage::Offset { .. } => tr!("tool-dimension-third"),
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        let unit = view.doc.model.unit;
        match self.stage {
            Stage::Idle => {
                if let Some((a, b)) = self.hover_edge {
                    line(out3d, a, b, SELECT_BLUE, 3.0);
                }
            }
            Stage::Start { a } => {
                if let Some(r) = &self.cur {
                    dashed(out3d, a, r.point, PREVIEW, 1.0);
                    label(out2d, (a + r.point) * 0.5, fmt_len(&view.doc.model, a.distance(r.point)));
                }
            }
            Stage::Offset { a, b } => {
                let text = varstotas_kernel::dims::dim_label(a.distance(b), None, unit);
                out3d.add_linear_dimension(a, b, self.offset, text, &dim_style(false));
                if let Some(k) = self.offset_axis {
                    out2d.push(Marker2d::CursorTooltip { text: k.label() });
                }
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
        !matches!(self.stage, Stage::Idle)
    }
}
