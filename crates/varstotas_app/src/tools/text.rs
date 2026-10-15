//! Text tool: leader text notes (stored as [`varstotas_kernel::Dimension`]s with `leader`).
//!
//! Click a point, move to where the text goes and click again. The default text is what was
//! clicked: a point's coordinates, an edge's length or a face's area. Clicking empty space
//! places a text without a leader. Edit the text in Entity Info.

use super::common::*;
use super::draw::Clicker;
use super::{OverlayView, Tool, ToolContext, ToolId};
use crate::inference::{self, InferenceEngine, InferenceKind, InferenceResult};
use crate::overlay::Marker2d;
use egui::{CursorIcon, Key};
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::{Model, units};
use varstotas_render::Overlay3d;

#[derive(Clone, Debug)]
enum Stage {
    Idle,
    Leader { at: DVec3, text: String },
}

pub struct TextTool {
    engine: InferenceEngine,
    stage: Stage,
    cur: Option<InferenceResult>,
    /// Text position while placing the leader.
    pos: DVec3,
    clicker: Clicker,
}

impl Default for TextTool {
    fn default() -> Self {
        Self { engine: InferenceEngine::default(), stage: Stage::Idle, cur: None, pos: DVec3::ZERO, clicker: Clicker::default() }
    }
}

/// World area of a face (its loops transformed by `xf`).
fn world_face_area(model: &Model, path: &[varstotas_kernel::InstanceId], f: varstotas_kernel::FaceId) -> Option<f64> {
    let mesh = model.mesh_at(path)?;
    let face = mesh.faces.get(f)?;
    let xf: DMat4 = model.transform_at(path);
    let loop_area = |l: &[varstotas_kernel::VertexId]| {
        let pts: Vec<DVec3> = l.iter().filter_map(|v| mesh.vertices.get(*v)).map(|v| xf.transform_point3(v.pos)).collect();
        let mut n = DVec3::ZERO;
        for i in 0..pts.len() {
            n += pts[i].cross(pts[(i + 1) % pts.len()]);
        }
        n.length() * 0.5
    };
    Some(loop_area(&face.outer) - face.holes.iter().map(|h| loop_area(h)).sum::<f64>())
}

/// Default text for a click: coordinates of a point, length of an edge, area of a face.
fn default_text(ctx: &ToolContext, r: &InferenceResult) -> Option<String> {
    let model = &ctx.doc.model;
    let unit = model.unit;
    let len = |v: f64| units::format_length(v, unit);
    match r.kind {
        InferenceKind::Endpoint
        | InferenceKind::Intersection
        | InferenceKind::Center
        | InferenceKind::Origin
        | InferenceKind::GuidePoint => {
            let ax = crate::axes::current();
            let p = ax.local_vec(r.point - ax.origin);
            Some(format!("({}, {}, {})", len(p.x + 0.0), len(p.y + 0.0), len(p.z + 0.0)))
        }
        InferenceKind::OnEdge | InferenceKind::Midpoint if r.edge.is_some() => {
            let (a, b) = r.edge?;
            Some(len(a.distance(b)))
        }
        _ => {
            let f = r.face.as_ref()?;
            let area = world_face_area(model, &f.path, f.face)?;
            Some(crate::ui::fmt_area(area, unit))
        }
    }
}

impl TextTool {
    fn update(&mut self, ctx: &ToolContext) {
        match &self.stage {
            Stage::Idle => {
                self.cur = Some(infer(&mut self.engine, ctx, None, None, None));
            }
            Stage::Leader { at, .. } => {
                // Text position in the plane through the point facing the camera.
                let ray = ctx.pointer.ray;
                let n = ctx.camera.basis().forward;
                let plane = varstotas_kernel::Plane::new(*at, n);
                self.pos = match plane.intersect_ray(&ray) {
                    Some(t) if t > 0.0 && ray.at(t).is_finite() => ray.at(t),
                    _ => *at,
                };
                self.cur = None;
            }
        }
    }

    fn place(ctx: &mut ToolContext, at: DVec3, offset: DVec3, text: String) {
        let path = ctx.doc.active_path.clone();
        if let Some(Some(id)) = ctx.doc.commit("Text", |m| m.add_text_note_in(&path, at, offset, text)) {
            ctx.doc.set_selection_full(Vec::new(), Vec::new(), vec![id]);
        }
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        match std::mem::replace(&mut self.stage, Stage::Idle) {
            Stage::Idle => {
                let Some(r) = self.cur.clone() else { return };
                match default_text(ctx, &r) {
                    Some(text) if r.kind != InferenceKind::Free || r.face.is_some() => {
                        self.stage = Stage::Leader { at: r.point, text };
                    }
                    _ => Self::place(ctx, r.point, DVec3::ZERO, "Text".into()),
                }
            }
            Stage::Leader { at, text } => {
                let off = if ctx.pointer.pos.distance(ctx.to_screen(at).unwrap_or(ctx.pointer.pos)) < 3.0 {
                    DVec3::ZERO
                } else {
                    self.pos - at
                };
                Self::place(ctx, at, off, text);
            }
        }
        self.update(ctx);
    }
}

impl Tool for TextTool {
    fn id(&self) -> ToolId {
        ToolId::Text
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
        if pressed && key == Key::Escape && !matches!(self.stage, Stage::Idle) {
            self.stage = Stage::Idle;
            self.update(ctx);
            return true;
        }
        false
    }
    fn status_text(&self) -> String {
        match self.stage {
            Stage::Idle => "Click a point, edge or face to attach a text, or empty space for a text without a leader.".into(),
            Stage::Leader { .. } => "Move to place the text and click. Edit the text in Entity Info.".into(),
        }
    }
    fn overlay(&self, _view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let Stage::Leader { at, text } = &self.stage {
            out3d.add_leader_text(*at, self.pos, text.clone(), &note_style(false));
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
