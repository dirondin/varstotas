//! Orbit / Pan / Zoom tools (left-drag camera navigation).

use super::{Tool, ToolContext, ToolId};
use crate::nav::dvec2;
use egui::{CursorIcon, PointerButton};
use varstotas_kernel::glam::DVec3;

#[derive(Default)]
pub struct OrbitTool {
    pivot: Option<DVec3>,
}

impl Tool for OrbitTool {
    fn id(&self) -> ToolId {
        ToolId::Orbit
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button == Some(PointerButton::Primary) {
            self.pivot = Some(ctx.point_under_pointer());
        }
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        if let (Some(p), true) = (self.pivot, ctx.pointer.primary_down) {
            if ctx.pointer.modifiers.shift {
                ctx.camera.pan(dvec2(ctx.pointer.delta), p, ctx.viewport_d());
            } else {
                ctx.camera.orbit(crate::nav::prefs().orbit_delta(dvec2(ctx.pointer.delta)), p);
            }
        }
    }
    fn on_pointer_up(&mut self, _ctx: &mut ToolContext) {
        self.pivot = None;
    }
    fn status_text(&self) -> String {
        "Drag to orbit. Shift = pan.".into()
    }
    fn cursor(&self) -> CursorIcon {
        if self.pivot.is_some() { CursorIcon::Grabbing } else { CursorIcon::Grab }
    }
}

#[derive(Default)]
pub struct PanTool {
    anchor: Option<DVec3>,
}

impl Tool for PanTool {
    fn id(&self) -> ToolId {
        ToolId::Pan
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button == Some(PointerButton::Primary) {
            self.anchor = Some(ctx.point_under_pointer());
        }
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        if let (Some(p), true) = (self.anchor, ctx.pointer.primary_down) {
            ctx.camera.pan(dvec2(ctx.pointer.delta), p, ctx.viewport_d());
        }
    }
    fn on_pointer_up(&mut self, _ctx: &mut ToolContext) {
        self.anchor = None;
    }
    fn status_text(&self) -> String {
        "Drag to pan.".into()
    }
    fn cursor(&self) -> CursorIcon {
        if self.anchor.is_some() { CursorIcon::Grabbing } else { CursorIcon::Grab }
    }
}

#[derive(Default)]
pub struct ZoomTool {
    toward: Option<DVec3>,
    fov: f64,
}

impl Tool for ZoomTool {
    fn id(&self) -> ToolId {
        ToolId::Zoom
    }
    fn activate(&mut self, ctx: &mut ToolContext) {
        self.fov = ctx.camera.fov_y_deg;
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button == Some(PointerButton::Primary) {
            self.toward = Some(ctx.point_under_pointer());
        }
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        if let (Some(p), true) = (self.toward, ctx.pointer.primary_down) {
            let dy = ctx.pointer.delta.y as f64;
            if dy != 0.0 {
                ctx.camera.zoom(crate::nav::prefs().zoom_factor((-dy * 0.01).exp()), p);
            }
        }
        self.fov = ctx.camera.fov_y_deg;
    }
    fn on_pointer_up(&mut self, _ctx: &mut ToolContext) {
        self.toward = None;
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        let s = input.trim().trim_end_matches("deg").trim_end_matches('°').trim();
        match s.parse::<f64>() {
            Ok(v) if (1.0..=120.0).contains(&v) => {
                ctx.camera.fov_y_deg = v;
                self.fov = v;
                true
            }
            _ => false,
        }
    }
    fn vcb_label(&self) -> &str {
        "Field of View"
    }
    fn vcb_value(&self) -> String {
        format!("{:.1} deg", self.fov)
    }
    fn status_text(&self) -> String {
        "Drag up/down to zoom. Type a field of view and press Enter.".into()
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::ZoomIn
    }
}
