//! Top toolbar with large vector-icon buttons (standard views, projection and face styles are
//! in the viewport's own controls, `view_hud.rs`).

use super::icons::Icon;
use super::{icon_button, theme};
use crate::app::{Action, VarstotasApp};
use crate::tools::ToolId;
use egui::{RichText, Ui};

/// Side of a toolbar button (Preferences → Interface → Toolbar icons).
static BTN_SIDE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(36);

#[allow(non_snake_case)]
fn BTN() -> f32 {
    BTN_SIDE.load(std::sync::atomic::Ordering::Relaxed) as f32
}

/// Toolbar icon sizes: name and button side.
pub const ICON_SIZES: [(&str, u32); 3] = [("Small", 28), ("Medium", 36), ("Large", 44)];

pub fn set_button_side(side: u32) {
    BTN_SIDE.store(side.clamp(20, 64), std::sync::atomic::Ordering::Relaxed);
}

fn group_sep(ui: &mut Ui) {
    ui.add_space(4.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, BTN() - 8.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::border());
    ui.add_space(4.0);
}

fn tip(resp: egui::Response, title: &str, shortcut: Option<&str>, desc: &str) -> egui::Response {
    resp.on_hover_ui(|ui| {
        ui.set_max_width(260.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(title).strong());
            if let Some(s) = shortcut {
                ui.label(RichText::new(format!("({s})")).color(theme::text_dim()));
            }
        });
        if !desc.is_empty() {
            ui.label(RichText::new(desc).color(theme::text_dim()));
        }
    })
}

impl VarstotasApp {
    fn tool_button(&mut self, ui: &mut Ui, id: ToolId) {
        let active = self.tool_id() == id;
        let resp = icon_button(ui, id.icon(), active, BTN());
        let mut desc = id.description().to_string();
        if !id.implemented() {
            desc.push_str("\n(not implemented yet)");
        }
        if tip(resp, id.name(), id.shortcut_label().as_deref(), &desc).clicked() {
            self.actions.push(Action::SetTool(id));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn action_button(&mut self, ui: &mut Ui, icon: Icon, selected: bool, title: &str, sc: Option<&str>, desc: &str, action: Action) {
        let resp = icon_button(ui, icon, selected, BTN());
        if tip(resp, title, sc, desc).clicked() {
            self.actions.push(action);
        }
    }

    /// The magnet: a click opens the Snapping menu. Crossed out when snapping is off, a dot
    /// when some inferences are off.
    fn snap_button(&mut self, ui: &mut Ui) {
        let snap = &self.prefs.snapping;
        let (on, partial) = (snap.enabled, snap.partial());
        let resp = icon_button(ui, Icon::Magnet, false, BTN());
        super::tray_lib::spot("magnet", resp.rect.center());
        let p = ui.painter();
        let r = resp.rect.shrink(7.0);
        if !on {
            p.line_segment([r.left_bottom(), r.right_top()], egui::Stroke::new(2.5, theme::text_dim()));
        } else if partial {
            p.circle_filled(r.right_top() + egui::vec2(2.0, -2.0), 3.5, egui::Color32::from_rgb(0xE8, 0x7A, 0x1E));
        }
        let state = match (on, partial) {
            (false, _) => "Snapping is off.",
            (true, true) => "Some snaps are off.",
            _ => "All snaps are on.",
        };
        let resp = tip(resp, "Snapping", Some("Shift+S"), &format!("Choose which points and directions the cursor snaps to. {state}"));
        egui::Popup::menu(&resp)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| self.snapping_menu(ui));
    }

    pub(crate) fn toolbar(&mut self, ui: &mut Ui) {
        // the harness spots are collected from here on each frame (toolbar, then the side bar)
        super::tray_lib::clear_spots();
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for id in [ToolId::Select, ToolId::Eraser, ToolId::Paint] {
                self.tool_button(ui, id);
            }
            group_sep(ui);
            // 3 Point Arc, Pie and Rotated Rectangle live in the Draw menu (keeps the toolbar
            // on one row at the default window width).
            for id in [ToolId::Line, ToolId::Arc, ToolId::ArcCenter, ToolId::Bezier, ToolId::Rectangle, ToolId::Circle, ToolId::Polygon] {
                self.tool_button(ui, id);
            }
            group_sep(ui);
            // Shapes: solids in one operation
            for id in crate::tools::SHAPES {
                self.tool_button(ui, id);
            }
            group_sep(ui);
            for id in [
                ToolId::Move,
                ToolId::Rotate,
                ToolId::Scale,
                ToolId::PushPull,
                ToolId::Offset,
                ToolId::FollowMe,
                ToolId::Chamfer,
                ToolId::Fillet,
            ] {
                self.tool_button(ui, id);
            }
            group_sep(ui);
            // Protractor lives in the Tools menu (keeps the toolbar on one row).
            for id in [ToolId::Tape, ToolId::Dimension, ToolId::Text] {
                self.tool_button(ui, id);
            }
            self.action_button(ui, Icon::Text3d, false, "3D Text", None, "Add text as 3D geometry (a group).", Action::Text3d);
            for id in [ToolId::Axes, ToolId::Section, ToolId::GuidePlane] {
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
            self.snap_button(ui);
        });
    }
}
