//! Solid Tools (Tools → Solid Tools): Outer Shell, Intersect, Union, Subtract, Trim, Split of
//! solid groups / components of the active context.
//!
//! With solids selected the operation runs at once (Outer Shell / Union: all of them, others:
//! the first two in selection order). Otherwise click the first solid, then the second; Outer
//! Shell / Union keep adding the next clicked solid to the result.

use varstotas_i18n::tr;
use super::common::*;
use super::draw::Clicker;
use super::{OverlayView, Tool, ToolContext, ToolId, ToolRequest};
use crate::document::Document;
use crate::overlay::Marker2d;
use egui::{CursorIcon, Key};
use varstotas_kernel::glam::DMat4;
use varstotas_kernel::{EntityRef, InstanceId, SolidOp};
use varstotas_render::Overlay3d;

pub struct SolidTool {
    op: SolidOp,
    first: Option<InstanceId>,
    /// Solid under the cursor (`Err`: an instance that is not a solid).
    hover: Option<Result<InstanceId, InstanceId>>,
    clicker: Clicker,
}

impl SolidTool {
    pub fn new(op: SolidOp) -> Self {
        Self { op, first: None, hover: None, clicker: Clicker::default() }
    }

    fn accumulates(&self) -> bool {
        matches!(self.op, SolidOp::Union | SolidOp::OuterShell)
    }

    fn update(&mut self, ctx: &ToolContext) {
        self.hover = match pick_face(ctx).or_else(|| pick_instance_wire(ctx, crate::inference::EDGE_PICK_PX)) {
            Some(Hover::Instance { id, .. }) => Some(if is_solid(ctx.doc, id) { Ok(id) } else { Err(id) }),
            _ => None,
        };
    }
}

/// Child instance `id` of the active context is a solid.
fn is_solid(doc: &Document, id: InstanceId) -> bool {
    let m = &doc.model;
    m.mesh_at(&doc.active_path)
        .and_then(|mesh| mesh.instances.get(id))
        .and_then(|i| m.defs.get(i.def))
        .is_some_and(|d| d.mesh.is_solid())
}

/// Run `op` on the selected solids. `None` when fewer than two instances are selected.
pub fn apply_to_selection(doc: &mut Document, op: SolidOp) -> Option<bool> {
    let insts: Vec<InstanceId> =
        doc.selection.iter().filter_map(|e| if let EntityRef::Instance(i) = e { Some(*i) } else { None }).collect();
    if insts.len() < 2 {
        return None;
    }
    let path = doc.active_path.clone();
    let out = if matches!(op, SolidOp::Union | SolidOp::OuterShell) {
        doc.try_commit(&op.name(), |m| m.solid_union_all_in(&path, &insts).map(|i| vec![i]))
    } else {
        doc.try_commit(&op.name(), |m| m.solid_op_in(&path, op, insts[0], insts[1]))
    };
    let done = out.is_some();
    if let Some(out) = out {
        doc.set_selection_full(out.into_iter().map(EntityRef::Instance).collect(), Vec::new(), Vec::new());
    }
    Some(done)
}

impl Tool for SolidTool {
    fn id(&self) -> ToolId {
        ToolId::SolidTools
    }
    fn activate(&mut self, ctx: &mut ToolContext) {
        if apply_to_selection(ctx.doc, self.op).is_some() {
            ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select));
            return;
        }
        ctx.doc.set_selection_full(Vec::new(), Vec::new(), Vec::new());
        self.update(ctx);
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if !self.clicker.down(ctx) {
            return;
        }
        self.update(ctx);
        let Some(Ok(id)) = self.hover else { return };
        let Some(first) = self.first else {
            self.first = Some(id);
            ctx.doc.set_selection_full(vec![EntityRef::Instance(id)], Vec::new(), Vec::new());
            return;
        };
        if first == id {
            return;
        }
        let path = ctx.doc.active_path.clone();
        let op = self.op;
        if let Some(out) = ctx.doc.try_commit(&op.name(), |m| m.solid_op_in(&path, op, first, id)) {
            self.first = if self.accumulates() { out.first().copied() } else { None };
            let sel = if self.accumulates() { out } else { Vec::new() };
            ctx.doc.set_selection_full(sel.into_iter().map(EntityRef::Instance).collect(), Vec::new(), Vec::new());
        }
        self.update(ctx);
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if pressed && key == Key::Escape && self.first.is_some() {
            self.first = None;
            ctx.doc.set_selection_full(Vec::new(), Vec::new(), Vec::new());
            return true;
        }
        false
    }
    fn status_text(&self) -> String {
        let name = self.op.name();
        if let Some(Err(_)) = self.hover {
            return tr!("tool-solid-not-solid", name = name);
        }
        match (self.first.is_some(), self.op) {
            (false, _) => tr!("tool-solid-first", name = name),
            (true, SolidOp::Union | SolidOp::OuterShell) => tr!("tool-solid-next", name = name),
            (true, SolidOp::Subtract | SolidOp::Trim) => tr!("tool-solid-cut", name = name),
            (true, _) => tr!("tool-solid-second", name = name),
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, _out2d: &mut Vec<Marker2d>) {
        if let Some(h) = self.hover {
            let (id, color) = match h {
                Ok(id) => (id, SELECT_BLUE),
                Err(id) => (id, egui::Color32::from_rgb(0xD0, 0x30, 0x30)),
            };
            for (a, b) in transformed_preview(view.doc, &[EntityRef::Instance(id)], &DMat4::IDENTITY, 4000) {
                line(out3d, a, b, color, 2.0);
            }
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::PointingHand
    }
    fn is_busy(&self) -> bool {
        self.first.is_some()
    }
}
