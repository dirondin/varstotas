//! The viewport's own controls in its top-right corner: the view (a standard view or a free
//! one, perspective / parallel) and the display style (Shaded … Realistic, X-Ray, edges,
//! profiles) — each a small drop-down. Drawn in the viewport's layer (windows stay above it);
//! clicks on it do not reach the tools (`hud_rect`). One per viewport: a split view will draw
//! one in each pane.

use super::icons::{Icon, paint_icon};
use crate::app::{Action, FaceStylePref, VarstotasApp};
use egui::{Align, Color32, Layout, Rect, Response, Stroke, Ui, UiBuilder, vec2};
use varstotas_render::StandardView;

/// The display styles in the order of the View menu: a short label for the button, the
/// menu's name and the toolbar's icon.
pub const STYLES: [(FaceStylePref, &str, &str, Icon); 6] = [
    (FaceStylePref::ShadedColor, "Shaded", "Shaded", Icon::StyleShaded),
    (FaceStylePref::Shaded, "Textured", "Shaded with Textures", Icon::StyleTextured),
    (FaceStylePref::Monochrome, "Monochrome", "Monochrome", Icon::StyleMonochrome),
    (FaceStylePref::HiddenLine, "Hidden Line", "Hidden Line", Icon::StyleHiddenLine),
    (FaceStylePref::Wireframe, "Wireframe", "Wireframe", Icon::StyleWireframe),
    (FaceStylePref::Realistic, "Realistic", "Realistic", Icon::StyleRealistic),
];

pub fn view_icon(v: StandardView) -> Icon {
    match v {
        StandardView::Iso => Icon::ViewIso,
        StandardView::Top => Icon::ViewTop,
        StandardView::Bottom => Icon::ViewBottom,
        StandardView::Front => Icon::ViewFront,
        StandardView::Back => Icon::ViewBack,
        StandardView::Left => Icon::ViewLeft,
        StandardView::Right => Icon::ViewRight,
    }
}

const ICON: f32 = 18.0;

/// A menu row: the icon, the text (highlighted when `on`), an optional shortcut.
fn item(ui: &mut Ui, icon: Icon, text: &str, on: bool, shortcut: Option<&str>) -> Response {
    let id = ui.id().with(("hud_icon", text));
    let mut b = egui::Button::selectable(on, (egui::Atom::custom(id, vec2(ICON, ICON)), text));
    if let Some(k) = shortcut {
        b = b.shortcut_text(k);
    }
    let r = b.atom_ui(ui);
    if let Some(rect) = r.rect(id) {
        paint_icon(ui.painter(), rect, icon, false);
    }
    r.response
}

/// A drop-down button of the bar: its icon, then the label.
fn pill(ui: &mut Ui, icon: Icon, label: &str, content: impl FnOnce(&mut Ui)) -> Response {
    let id = ui.id().with(("hud_pill", label));
    // the icon's room is a little wider than the icon: a gap before the label
    let b = egui::Button::new((egui::Atom::custom(id, vec2(ICON + 4.0, ICON)), format!("{label} ⏷"))).frame(false);
    let (r, _) = egui::containers::menu::MenuButton::from_button(b).ui(ui, content);
    // the custom atom comes first: right after the button's padding
    let pad = ui.spacing().button_padding.x;
    let rect = Rect::from_center_size(egui::pos2(r.rect.left() + pad + ICON / 2.0, r.rect.center().y), vec2(ICON, ICON));
    paint_icon(ui.painter(), rect, icon, false);
    r
}

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
        let ortho = self.camera.ortho;
        let icon = match view {
            Some(v) => view_icon(v),
            None if ortho => Icon::Parallel,
            None => Icon::Perspective,
        };
        pill(ui, icon, &view_label(view, ortho), |ui| {
            for v in StandardView::ALL {
                if item(ui, view_icon(v), v.name(), view == Some(v), None).clicked() {
                    self.actions.push(Action::View(v));
                    ui.close();
                }
            }
            ui.separator();
            if item(ui, Icon::Perspective, "Perspective", !ortho, None).clicked() {
                self.actions.push(Action::SetParallel(false));
                ui.close();
            }
            if item(ui, Icon::Parallel, "Parallel Projection", ortho, None).clicked() {
                self.actions.push(Action::SetParallel(true));
                ui.close();
            }
            ui.separator();
            if item(ui, Icon::ZoomExtents, "Zoom Extents", false, Some("Shift+Z")).clicked() {
                self.actions.push(Action::ZoomExtents);
                ui.close();
            }
        })
        .on_hover_text("The view: standard views, perspective or parallel projection");
    }

    fn hud_style_menu(&mut self, ui: &mut Ui) {
        let style = self.prefs.face_style;
        let mut label = style_label(style).to_string();
        if self.prefs.xray {
            label.push_str(" · X-Ray");
        }
        let icon = STYLES.iter().find(|x| x.0 == style).map(|x| x.3).unwrap_or(Icon::StyleShaded);
        pill(ui, icon, &label, |ui| {
            for (s, _, name, icon) in STYLES {
                if item(ui, icon, name, style == s, None).clicked() {
                    self.prefs.face_style = s;
                    ui.close();
                }
            }
            ui.separator();
            // switches: a click toggles, the menu stays open
            if item(ui, Icon::StyleXRay, "X-Ray", self.prefs.xray, None).clicked() {
                self.prefs.xray = !self.prefs.xray;
            }
            let edges = *self.prefs.edges_mut();
            if item(ui, Icon::StyleEdges, "Edges", edges, None).clicked() {
                *self.prefs.edges_mut() = !edges;
            }
            if item(ui, Icon::StyleProfiles, "Profiles", self.prefs.profiles, None).clicked() {
                self.prefs.profiles = !self.prefs.profiles;
            }
        })
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
