//! Placing 3D Text (Tools → 3D Text…): the text follows the cursor, lying on the face under
//! it (on the ground otherwise); a click adds it as a group. Shown as the Move tool.

use super::common::*;
use super::draw::Clicker;
use super::{OverlayView, Tool, ToolContext, ToolId, ToolRequest};
use crate::inference::{self, InferenceEngine, InferenceResult};
use crate::overlay::Marker2d;
use crate::text3d::Text3dParams;
use egui::{CursorIcon, Key};
use varstotas_kernel::EntityRef;
use varstotas_kernel::glam::{DMat4, DVec2, DVec3, DVec4};
use varstotas_render::Overlay3d;

pub struct Text3dPlaceTool {
    params: Text3dParams,
    outlines: Vec<Vec<DVec2>>,
    engine: InferenceEngine,
    cur: Option<InferenceResult>,
    clicker: Clicker,
}

impl Text3dPlaceTool {
    pub fn new(params: Text3dParams) -> Self {
        let outlines = crate::text3d::outlines(&params);
        Self { params, outlines, engine: InferenceEngine::default(), cur: None, clicker: Clicker::default() }
    }

    fn update(&mut self, ctx: &ToolContext) {
        self.cur = Some(infer(&mut self.engine, ctx, None, None, None));
    }

    /// Text plane → world: the baseline starts at the point; on a face the text reads
    /// left-to-right seen from the front with "up" toward +Z (or +Y on level faces).
    fn placement(&self) -> Option<DMat4> {
        let r = self.cur.as_ref()?;
        let n = match &r.face {
            Some(f) if f.plane.distance(r.point).abs() < 1e-3 * (1.0 + r.point.length() * 1e-6) => f.plane.normal,
            _ => DVec3::Z,
        };
        let x = DVec3::Z.cross(n);
        let x = if x.length() < 1e-6 { DVec3::X } else { x.normalize() };
        let y = n.cross(x);
        Some(DMat4::from_cols(x.extend(0.0), y.extend(0.0), n.extend(0.0), DVec4::from((r.point, 1.0))))
    }
}

impl Tool for Text3dPlaceTool {
    fn id(&self) -> ToolId {
        ToolId::Move
    }
    fn activate(&mut self, ctx: &mut ToolContext) {
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
        let Some(xf) = self.placement() else { return };
        let path = ctx.doc.active_path.clone();
        let opts = self.params.options();
        let outlines = &self.outlines;
        if let Some(inst) = ctx.doc.try_commit("3D Text", |m| m.add_text3d_in(&path, outlines, opts, &xf)) {
            ctx.doc.set_selection_full(vec![EntityRef::Instance(inst)], Vec::new(), Vec::new());
            ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select));
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if pressed && key == Key::Escape {
            ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select));
            return true;
        }
        false
    }
    fn status_text(&self) -> String {
        "Click to place the 3D text (Esc cancels).".into()
    }
    fn overlay(&self, _view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let Some(xf) = self.placement() {
            for l in &self.outlines {
                let pts: Vec<DVec3> = l.iter().map(|p| xf.transform_point3(p.extend(0.0))).collect();
                polyline(out3d, &pts, true, SELECT_BLUE, 1.5);
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
        true
    }
}
