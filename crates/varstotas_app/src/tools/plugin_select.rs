//! Object selection for scripts and plugins with `@select`: clicks toggle faces, edges and
//! groups of the open context in the selection, Enter hands the selection to the script as
//! `objects`, Esc cancels.

use super::common::*;
use super::{OverlayView, Tool, ToolContext, ToolId, ToolRequest};
use crate::overlay::Marker2d;
use egui::{CursorIcon, Key};
use varstotas_kernel::EntityRef;
use varstotas_kernel::glam::DMat4;
use varstotas_render::Overlay3d;

pub struct PluginSelectTool {
    prompt: String,
    hover: Option<EntityRef>,
}

impl PluginSelectTool {
    pub fn new(prompt: String) -> Self {
        Self { prompt, hover: None }
    }

    /// The entity of the open context under the cursor: an edge near it, a child group's wire,
    /// else the face or the child group hit.
    fn pick(ctx: &ToolContext) -> Option<EntityRef> {
        if let Some(h @ Hover::Edge { .. }) = pick_edge_at(ctx, ctx.pointer.pos, crate::inference::EDGE_PICK_PX as f32) {
            return Some(h.entity());
        }
        let hit = ctx.pick();
        if let Some(Hover::Instance { id, point }) = pick_instance_wire(ctx, crate::inference::EDGE_PICK_PX) {
            let eye = ctx.pointer.ray.origin;
            if hit.as_ref().is_none_or(|h| point.distance(eye) <= h.point.distance(eye) + varstotas_kernel::TOL) {
                return Some(EntityRef::Instance(id));
            }
        }
        let hit = hit?;
        let active = &ctx.doc.active_path;
        if !hit.path.starts_with(active) {
            return None;
        }
        Some(match hit.path.get(active.len()) {
            Some(id) => EntityRef::Instance(*id),
            None => EntityRef::Face(hit.face),
        })
    }
}

impl Tool for PluginSelectTool {
    fn id(&self) -> ToolId {
        ToolId::Select
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        self.hover = Self::pick(ctx);
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if let Some(e) = Self::pick(ctx) {
            ctx.doc.toggle_selected(e);
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        match key {
            Key::Enter => ctx.requests.push(ToolRequest::PluginObjects),
            Key::Escape => {}
            _ => return false,
        }
        ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select));
        true
    }
    fn status_text(&self) -> String {
        format!("{} — click to add / remove, Enter to continue, Esc cancels.", self.prompt)
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, _out2d: &mut Vec<Marker2d>) {
        match self.hover {
            Some(EntityRef::Face(f)) => highlight_face(view, f, HOVER_FACE, out3d),
            Some(e) => {
                for (a, b) in transformed_preview(view.doc, &[e], &DMat4::IDENTITY, 4000) {
                    line(out3d, a, b, SELECT_BLUE, 3.0);
                }
            }
            None => {}
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::PointingHand
    }
    fn is_busy(&self) -> bool {
        true
    }
}
