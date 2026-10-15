//! Point picking for plugins with `@pick N`: N clicks (with inference), then the app runs
//! the plugin with `points`.

use super::common::*;
use super::draw::Clicker;
use super::{OverlayView, Tool, ToolContext, ToolId, ToolRequest};
use crate::inference::{self, InferenceEngine, InferenceResult};
use crate::overlay::Marker2d;
use egui::{CursorIcon, Key};
use varstotas_kernel::glam::DVec3;
use varstotas_render::Overlay3d;

pub struct PluginPickTool {
    n: usize,
    prompt: String,
    points: Vec<DVec3>,
    engine: InferenceEngine,
    cur: Option<InferenceResult>,
    clicker: Clicker,
}

impl PluginPickTool {
    pub fn new(n: usize, prompt: String) -> Self {
        Self { n, prompt, points: Vec::new(), engine: InferenceEngine::default(), cur: None, clicker: Clicker::default() }
    }

    fn update(&mut self, ctx: &ToolContext) {
        self.cur = Some(infer(&mut self.engine, ctx, self.points.last().copied(), None, None));
    }
}

impl Tool for PluginPickTool {
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
        let Some(p) = self.cur.as_ref().map(|r| r.point) else { return };
        self.points.push(p);
        if self.points.len() >= self.n {
            ctx.requests.push(ToolRequest::PluginPoints(std::mem::take(&mut self.points)));
            ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select));
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        if key == Key::Escape {
            ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select));
            return true;
        }
        let r = self.engine.handle_arrow(key);
        if r {
            self.update(ctx);
        }
        r
    }
    fn status_text(&self) -> String {
        format!("{} ({} of {}; Esc cancels).", self.prompt, self.points.len() + 1, self.n)
    }
    fn overlay(&self, _view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        for w in self.points.windows(2) {
            line(out3d, w[0], w[1], SELECT_BLUE, 2.0);
        }
        if let (Some(a), Some(r)) = (self.points.last(), &self.cur) {
            line(out3d, *a, r.point, PREVIEW, 1.0);
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
