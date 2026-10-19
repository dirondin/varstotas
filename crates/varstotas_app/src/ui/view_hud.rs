//! The viewport's own controls in its top-right corner: the view (a standard view or a free
//! one, perspective / parallel) and the display style (Shaded … Realistic, X-Ray, edges,
//! profiles) — each a small drop-down. Drawn in the viewport's layer (windows stay above it);
//! clicks on it do not reach the tools (`hud_rect`). One per viewport: a split view will draw
//! one in each pane.

use crate::app::{Action, FaceStylePref, VarstotasApp};
use egui::{Align, Color32, Layout, Rect, RichText, Stroke, Ui, UiBuilder, vec2};
use varstotas_render::StandardView;

/// The display styles in the order of the View menu, with a short label for the button.
pub const STYLES: [(FaceStylePref, &str, &str); 6] = [
    (FaceStylePref::ShadedColor, "Shaded", "Shaded"),
    (FaceStylePref::Shaded, "Textured", "Shaded with Textures"),
    (FaceStylePref::Monochrome, "Monochrome", "Monochrome"),
    (FaceStylePref::HiddenLine, "Hidden Line", "Hidden Line"),
    (FaceStylePref::Wireframe, "Wireframe", "Wireframe"),
    (FaceStylePref::Realistic, "Realistic", "Realistic"),
];

/// The short name of a style.
pub fn style_label(s: FaceStylePref) -> &'static str {
    STYLES.iter().find(|x| x.0 == s).map(|x| x.1).unwrap_or("Shaded")
}

/// "Top · Parallel", "Perspective" (a free view) …
pub fn view_label(view: Option<StandardView>, ortho: bool) -> String {
    let proj = if ortho { "Parallel" } else { "Perspective" };
    match view {
        Some(v) => format!("{} · {proj}", v.name()),
        None => proj.to_string(),
    }
}

impl VarstotasApp {
    /// Draw the controls over the viewport `rect`; remembers where they are.
    pub(crate) fn view_hud(&mut self, ui: &mut Ui, rect: Rect) {
        let area = Rect::from_min_max(rect.right_top() + vec2(-520.0, 8.0), rect.right_top() + vec2(-8.0, 44.0));
        let mut hud = Rect::NOTHING;
        ui.scope_builder(UiBuilder::new().max_rect(area).layout(Layout::right_to_left(Align::Min)), |ui| {
            let fill = if ui.visuals().dark_mode { Color32::from_black_alpha(170) } else { Color32::from_white_alpha(215) };
            let r = egui::Frame::new()
                .fill(fill)
                .stroke(Stroke::new(1.0, Color32::from_black_alpha(30)))
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(6, 2))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        // (right to left: the view ends up on the left)
                        self.hud_style_menu(ui);
                        let (r, _) = ui.allocate_exact_size(vec2(9.0, 16.0), egui::Sense::hover());
                        ui.painter().vline(r.center().x, r.y_range(), Stroke::new(1.0, Color32::from_black_alpha(40)));
                        self.hud_view_menu(ui);
                    });
                });
            hud = r.response.rect;
        });
        self.hud_rect = hud;
    }

    fn hud_view_menu(&mut self, ui: &mut Ui) {
        let view = self.camera.standard_view();
        let label = format!("{} ⏷", view_label(view, self.camera.ortho));
        let b = egui::Button::new(RichText::new(label)).frame(false);
        egui::containers::menu::MenuButton::from_button(b).ui(ui, |ui| {
            for v in StandardView::ALL {
                if ui.add(egui::Button::selectable(view == Some(v), v.name())).clicked() {
                    self.actions.push(Action::View(v));
                    ui.close();
                }
            }
            ui.separator();
            let ortho = self.camera.ortho;
            if ui.radio(!ortho, "Perspective").clicked() {
                self.actions.push(Action::SetParallel(false));
                ui.close();
            }
            if ui.radio(ortho, "Parallel Projection").clicked() {
                self.actions.push(Action::SetParallel(true));
                ui.close();
            }
            ui.separator();
            if ui.add(egui::Button::new("Zoom Extents").shortcut_text("Shift+Z")).clicked() {
                self.actions.push(Action::ZoomExtents);
                ui.close();
            }
        })
        .0
        .on_hover_text("The view: standard views, perspective or parallel projection");
    }

    fn hud_style_menu(&mut self, ui: &mut Ui) {
        let mut label = style_label(self.prefs.face_style).to_string();
        if self.prefs.xray {
            label.push_str(" · X-Ray");
        }
        let b = egui::Button::new(RichText::new(format!("{label} ⏷"))).frame(false);
        egui::containers::menu::MenuButton::from_button(b).ui(ui, |ui| {
            for (s, _, name) in STYLES {
                if ui.radio(self.prefs.face_style == s, name).clicked() {
                    self.prefs.face_style = s;
                    ui.close();
                }
            }
            ui.separator();
            ui.checkbox(&mut self.prefs.xray, "X-Ray");
            ui.checkbox(self.prefs.edges_mut(), "Edges");
            ui.checkbox(&mut self.prefs.profiles, "Profiles");
        })
        .0
        .on_hover_text("How faces and edges are drawn (also View → Face Style)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels() {
        assert_eq!(view_label(Some(StandardView::Top), true), "Top · Parallel");
        assert_eq!(view_label(None, false), "Perspective");
        assert_eq!(style_label(FaceStylePref::Shaded), "Textured");
        assert_eq!(STYLES.len(), 6);
    }
}
