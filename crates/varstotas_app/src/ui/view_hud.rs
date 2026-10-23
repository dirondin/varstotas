//! The viewport's own controls in its top-right corner: the view (a standard view or a free
//! one, perspective / parallel, field of view, 1 / 2 / 4 viewports), the display style
//! (Shaded … Realistic, X-Ray, edges, profiles) and what is shown (axes, ground, grid; the
//! model's guides, shadows, sections) — each a small drop-down. Drawn in the viewport's layer
//! (windows stay above it); clicks on it do not reach the tools (`hud_rect`). One per pane of
//! a split view (`panes.rs`).

use super::icons::{Icon, paint_icon};
use crate::app::{Action, FaceStylePref, VarstotasApp};
use egui::{Align, Color32, Layout, Rect, Response, Stroke, Ui, UiBuilder, vec2};
use varstotas_i18n::tr;
use varstotas_render::StandardView;

/// The display styles in the order of the View menu: keys of a short label for the button and
/// of the menu's name, and the toolbar's icon.
pub const STYLES: [(FaceStylePref, &str, &str, Icon); 6] = [
    (FaceStylePref::ShadedColor, "view-style-shaded", "view-style-shaded", Icon::StyleShaded),
    (FaceStylePref::Shaded, "view-style-textured", "view-style-textured-menu", Icon::StyleTextured),
    (FaceStylePref::Monochrome, "view-style-monochrome", "view-style-monochrome", Icon::StyleMonochrome),
    (FaceStylePref::HiddenLine, "view-style-hidden-line", "view-style-hidden-line", Icon::StyleHiddenLine),
    (FaceStylePref::Wireframe, "view-style-wireframe", "view-style-wireframe", Icon::StyleWireframe),
    (FaceStylePref::Realistic, "view-style-realistic", "view-style-realistic", Icon::StyleRealistic),
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

/// A small selectable button with only an icon.
fn icon_button(ui: &mut Ui, icon: Icon, on: bool, tip: &str) -> Response {
    let id = ui.id().with(("hud_icon_button", tip));
    let r = egui::Button::selectable(on, egui::Atom::custom(id, vec2(ICON, ICON))).atom_ui(ui);
    if let Some(rect) = r.rect(id) {
        paint_icon(ui.painter(), rect, icon, false);
    }
    r.response.on_hover_text(tip)
}

/// A drop-down button of the bar: only its icon and the arrow; `name` identifies it, the
/// caller's hover text says what it shows.
fn pill(ui: &mut Ui, icon: Icon, name: &str, content: impl FnOnce(&mut Ui)) -> Response {
    let id = ui.id().with(("hud_pill", name));
    // the icon's room is a little wider than the icon: a gap before the arrow
    let b = egui::Button::new((egui::Atom::custom(id, vec2(ICON + 3.0, ICON)), "⏷")).frame(false);
    // a click inside does not close it: the switches and the field of view stay in the open
    // menu, the choices close it themselves (`ui.close()`)
    let config = egui::containers::menu::MenuConfig::new().close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
    let (r, _) = egui::containers::menu::MenuButton::from_button(b).config(config).ui(ui, content);
    // the custom atom comes first: right after the button's padding
    let pad = ui.spacing().button_padding.x;
    let rect = Rect::from_center_size(egui::pos2(r.rect.left() + pad + ICON / 2.0, r.rect.center().y), vec2(ICON, ICON));
    paint_icon(ui.painter(), rect, icon, false);
    r
}

/// The short name of a style.
pub fn style_label(s: FaceStylePref) -> String {
    tr(STYLES.iter().find(|x| x.0 == s).map(|x| x.1).unwrap_or("view-style-shaded"))
}

/// "Top · Parallel", "Perspective" (a free view) …
pub fn view_label(view: Option<StandardView>, ortho: bool) -> String {
    let proj = tr(if ortho { "view-parallel" } else { "view-perspective" });
    match view {
        Some(v) => tr!("view-label", view = v.name(), projection = proj),
        None => proj,
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
                        // (right to left: the view ends up on the left, Show on the right)
                        let sep = |ui: &mut Ui| {
                            let (r, _) = ui.allocate_exact_size(vec2(9.0, 16.0), egui::Sense::hover());
                            ui.painter().vline(r.center().x, r.y_range(), Stroke::new(1.0, Color32::from_black_alpha(40)));
                        };
                        self.hud_show_menu(ui);
                        sep(ui);
                        self.hud_style_menu(ui);
                        sep(ui);
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
        pill(ui, icon, "view", |ui| {
            self.hud_open = Some((self.active_pane, "view"));
            for v in StandardView::ALL {
                if item(ui, view_icon(v), &v.name(), view == Some(v), None).clicked() {
                    self.actions.push(Action::View(v));
                    ui.close();
                }
            }
            ui.separator();
            if item(ui, Icon::Perspective, &tr!("view-perspective"), !ortho, None).clicked() {
                self.actions.push(Action::SetParallel(false));
                ui.close();
            }
            if item(ui, Icon::Parallel, &tr!("view-parallel-projection"), ortho, None).clicked() {
                self.actions.push(Action::SetParallel(true));
                ui.close();
            }
            ui.add_enabled_ui(!ortho, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(ICON + 8.0);
                    ui.label(tr!("view-fov"));
                    ui.add(egui::DragValue::new(&mut self.camera.fov_y_deg).range(10.0..=120.0).speed(0.5).suffix("°").max_decimals(0));
                })
                .response
                .on_hover_text(tr!("view-fov-tip"));
            });
            ui.separator();
            if item(ui, Icon::ZoomExtents, &tr!("view-zoom-extents"), false, Some("Shift+Z")) // i18n-ignore
            .clicked() {
                self.actions.push(Action::ZoomExtents);
                ui.close();
            }
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(tr!("view-viewports"));
                for (n, icon, tip) in [(1, Icon::Panes1, tr!("view-panes-1")), (2, Icon::Panes2, tr!("view-panes-2")), (4, Icon::Panes4, tr!("view-panes-4"))] {
                    if icon_button(ui, icon, self.pane_count == n, &tip).clicked() {
                        self.actions.push(Action::SetPanes(n));
                        ui.close();
                    }
                }
            });
            ui.menu_button(tr!("view-presets"), |ui| {
                for (i, p) in crate::panes::PRESETS.iter().enumerate() {
                    let icon = if p.panes.len() == 4 { Icon::Panes4 } else { Icon::Panes2 };
                    if item(ui, icon, &p.name(), false, None).clicked() {
                        self.actions.push(Action::ViewPreset(i));
                        ui.close();
                    }
                }
            });
        })
        .on_hover_text(tr!("view-hud-view-tip", label = view_label(view, ortho)));
    }

    fn hud_style_menu(&mut self, ui: &mut Ui) {
        let style = self.prefs.face_style;
        let mut label = style_label(style);
        if self.prefs.xray {
            label = tr!("view-style-xray-suffix", label = label);
        }
        let icon = STYLES.iter().find(|x| x.0 == style).map(|x| x.3).unwrap_or(Icon::StyleShaded);
        pill(ui, icon, "style", |ui| {
            self.hud_open = Some((self.active_pane, "style"));
            for (s, _, name, icon) in STYLES {
                if item(ui, icon, &tr(name), style == s, None).clicked() {
                    self.prefs.face_style = s;
                    ui.close();
                }
            }
            ui.separator();
            // switches: a click toggles, the menu stays open
            if item(ui, Icon::StyleXRay, &tr!("view-xray"), self.prefs.xray, None).clicked() {
                self.prefs.xray = !self.prefs.xray;
            }
            let edges = *self.prefs.edges_mut();
            if item(ui, Icon::StyleEdges, &tr!("view-edges"), edges, None).clicked() {
                *self.prefs.edges_mut() = !edges;
            }
            let r = item(ui, Icon::StyleWireframe, &tr!("view-triangles"), self.prefs.show_triangles, None);
            super::tray_lib::spot("hud:triangles", r.rect.center());
            if r.on_hover_text(tr!("view-triangles-hint")).clicked() {
                self.prefs.show_triangles = !self.prefs.show_triangles;
            }
            if item(ui, Icon::StyleProfiles, &tr!("view-profiles"), self.prefs.profiles, None).clicked() {
                self.prefs.profiles = !self.prefs.profiles;
            }
        })
        .on_hover_text(tr!("view-hud-style-tip", label = label));
    }

    /// What the Show drop-down switches: icon, name, on. The first four are the view's own,
    /// the rest the model's (every view).
    pub(crate) fn hud_show_items(&self) -> [(Icon, String, bool); 9] {
        let m = &self.doc.model;
        let d = m.section_display;
        [
            (Icon::Axes, tr!("view-show-axes"), self.prefs.show_axes),
            (Icon::Ground, tr!("view-show-ground"), self.prefs.show_ground),
            (Icon::Sky, tr!("view-show-sky"), self.prefs.show_sky),
            (Icon::Grid, tr!("view-show-grid"), self.prefs.grid.show),
            (Icon::Guides, tr!("view-show-guides"), !m.guides_hidden),
            (Icon::Shadows, tr!("view-show-shadows"), m.shadows.enabled),
            (Icon::Section, tr!("view-show-section-planes"), d.planes),
            (Icon::SectionCut, tr!("view-show-section-cuts"), d.cuts),
            (Icon::SectionFill, tr!("view-show-section-fill"), d.fill),
        ]
    }

    fn hud_show_menu(&mut self, ui: &mut Ui) {
        let items = self.hud_show_items();
        pill(ui, Icon::Eye, "show", |ui| {
            self.hud_open = Some((self.active_pane, "show"));
            // switches: a click toggles, the menu stays open
            for (k, (icon, name, on)) in items.into_iter().enumerate() {
                if k == 4 {
                    ui.separator();
                    ui.label(egui::RichText::new(tr!("view-show-model-header")).small().weak());
                }
                if k == 6 {
                    ui.separator();
                }
                if item(ui, icon, &name, on, None).clicked() {
                    match k {
                        0 => self.prefs.show_axes = !on,
                        1 => self.prefs.show_ground = !on,
                        2 => self.prefs.show_sky = !on,
                        3 => self.actions.push(Action::ToggleGrid),
                        4 => self.actions.push(Action::ToggleGuides),
                        5 => self.actions.push(Action::ToggleShadows),
                        _ => self.actions.push(Action::ToggleSectionDisplay(k as u8 - 6)),
                    }
                }
            }
        })
        .on_hover_text(tr!("view-hud-show-tip"));
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
