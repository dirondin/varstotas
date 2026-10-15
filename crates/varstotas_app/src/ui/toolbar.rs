//! Top toolbar with large vector-icon buttons.

use super::icons::Icon;
use super::{icon_button, theme};
use crate::app::{Action, FaceStylePref, VarstotasApp};
use crate::tools::ToolId;
use egui::{RichText, Ui};
use varstotas_render::StandardView;

const BTN: f32 = 36.0;

fn group_sep(ui: &mut Ui) {
    ui.add_space(4.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, BTN - 8.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::BORDER);
    ui.add_space(4.0);
}

fn tip(resp: egui::Response, title: &str, shortcut: Option<&str>, desc: &str) -> egui::Response {
    resp.on_hover_ui(|ui| {
        ui.set_max_width(260.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(title).strong());
            if let Some(s) = shortcut {
                ui.label(RichText::new(format!("({s})")).color(theme::TEXT_DIM));
            }
        });
        if !desc.is_empty() {
            ui.label(RichText::new(desc).color(theme::TEXT_DIM));
        }
    })
}

impl VarstotasApp {
    fn tool_button(&mut self, ui: &mut Ui, id: ToolId) {
        let active = self.tool_id() == id;
        let resp = icon_button(ui, id.icon(), active, BTN);
        let mut desc = id.description().to_string();
        if !id.implemented() {
            desc.push_str("\n(not implemented yet)");
        }
        if tip(resp, id.name(), id.shortcut_label(), &desc).clicked() {
            self.actions.push(Action::SetTool(id));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn action_button(&mut self, ui: &mut Ui, icon: Icon, selected: bool, title: &str, sc: Option<&str>, desc: &str, action: Action) {
        let resp = icon_button(ui, icon, selected, BTN);
        if tip(resp, title, sc, desc).clicked() {
            self.actions.push(action);
        }
    }

    fn style_button(&mut self, ui: &mut Ui, icon: Icon, style: FaceStylePref, title: &str) {
        let resp = icon_button(ui, icon, self.prefs.face_style == style, BTN);
        if tip(resp, title, None, "Face style").clicked() {
            self.prefs.face_style = style;
        }
    }

    pub(crate) fn toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for id in [ToolId::Select, ToolId::Eraser, ToolId::Paint] {
                self.tool_button(ui, id);
            }
            group_sep(ui);
            // 3 Point Arc, Pie and Rotated Rectangle live in the Draw menu (keeps the toolbar
            // on one row at the default window width).
            for id in [ToolId::Line, ToolId::Arc, ToolId::ArcCenter, ToolId::Rectangle, ToolId::Circle, ToolId::Polygon] {
                self.tool_button(ui, id);
            }
            group_sep(ui);
            for id in [
                ToolId::Move,
                ToolId::PushPull,
                ToolId::Rotate,
                ToolId::Scale,
                ToolId::Offset,
                ToolId::FollowMe,
            ] {
                self.tool_button(ui, id);
            }
            group_sep(ui);
            // Protractor lives in the Tools menu (keeps the toolbar on one row).
            for id in [ToolId::Tape, ToolId::Dimension, ToolId::Axes, ToolId::Section] {
                self.tool_button(ui, id);
            }
            group_sep(ui);
            for id in [ToolId::Orbit, ToolId::Pan, ToolId::Zoom] {
                self.tool_button(ui, id);
            }
            self.action_button(
                ui,
                Icon::ZoomExtents,
                false,
                "Zoom Extents",
                Some("Shift+Z"),
                "Fit the whole model in the view.",
                Action::ZoomExtents,
            );
            group_sep(ui);
            for (icon, v, name) in [
                (Icon::ViewIso, StandardView::Iso, "Iso"),
                (Icon::ViewTop, StandardView::Top, "Top"),
                (Icon::ViewFront, StandardView::Front, "Front"),
                (Icon::ViewRight, StandardView::Right, "Right"),
            ] {
                self.action_button(ui, icon, false, name, None, "Standard view", Action::View(v));
            }
            let ortho = self.camera.ortho;
            self.action_button(
                ui,
                Icon::Perspective,
                !ortho,
                if ortho { "Parallel Projection" } else { "Perspective" },
                None,
                "Toggle perspective / parallel projection.",
                Action::SetParallel(!ortho),
            );
            group_sep(ui);
            let xray = self.prefs.xray;
            let resp = icon_button(ui, Icon::StyleXRay, xray, BTN);
            if tip(resp, "X-Ray", None, "See through faces.").clicked() {
                self.prefs.xray = !xray;
            }
            self.style_button(ui, Icon::StyleWireframe, FaceStylePref::Wireframe, "Wireframe");
            self.style_button(ui, Icon::StyleHiddenLine, FaceStylePref::HiddenLine, "Hidden Line");
            self.style_button(ui, Icon::StyleShaded, FaceStylePref::ShadedColor, "Shaded");
            self.style_button(ui, Icon::StyleTextured, FaceStylePref::Shaded, "Shaded with Textures");
            self.style_button(ui, Icon::StyleMonochrome, FaceStylePref::Monochrome, "Monochrome");
        });
    }
}
