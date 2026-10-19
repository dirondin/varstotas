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

/// One button of the toolbar (Preferences → Toolbar orders them; the rest wait in reserve).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Tool(ToolId),
    Text3d,
    ZoomExtents,
    Snapping,
    /// A line between groups.
    Sep,
}

impl Item {
    /// The id kept in the preferences.
    pub fn key(self) -> String {
        match self {
            Item::Tool(id) => format!("{id:?}"),
            Item::Text3d => "Text3d".into(),
            Item::ZoomExtents => "ZoomExtents".into(),
            Item::Snapping => "Snapping".into(),
            Item::Sep => "|".into(),
        }
    }

    pub fn from_key(k: &str) -> Option<Item> {
        match k {
            "Text3d" => Some(Item::Text3d),
            "ZoomExtents" => Some(Item::ZoomExtents),
            "Snapping" => Some(Item::Snapping),
            "|" => Some(Item::Sep),
            _ => ToolId::ALL.iter().copied().find(|t| format!("{t:?}") == k).map(Item::Tool),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Item::Tool(id) => id.name(),
            Item::Text3d => "3D Text",
            Item::ZoomExtents => "Zoom Extents",
            Item::Snapping => "Snapping",
            Item::Sep => "Separator",
        }
    }

    pub fn icon(self) -> Option<Icon> {
        match self {
            Item::Tool(id) => Some(id.icon()),
            Item::Text3d => Some(Icon::Text3d),
            Item::ZoomExtents => Some(Icon::ZoomExtents),
            Item::Snapping => Some(Icon::Magnet),
            Item::Sep => None,
        }
    }

    /// Every button there is (separators aside), in the menus' order.
    pub fn all() -> Vec<Item> {
        let mut v: Vec<Item> = ToolId::ALL.iter().copied().filter(|t| t.implemented()).map(Item::Tool).collect();
        v.extend([Item::Text3d, Item::ZoomExtents, Item::Snapping]);
        v
    }
}

/// The toolbar out of the box. 3 Point Arc, Pie, Rotated Rectangle, Smooth Curve, Freehand,
/// Protractor … wait in reserve (the toolbar stays short at the default window width).
pub fn default_layout() -> Vec<Item> {
    use Item::{Sep, Tool};
    use ToolId as T;
    let mut v = vec![Tool(T::Select), Tool(T::Eraser), Tool(T::Paint), Sep];
    v.extend([T::Line, T::Arc, T::ArcCenter, T::Bezier, T::Rectangle, T::Circle, T::Polygon].map(Tool));
    v.push(Sep);
    v.extend(crate::tools::SHAPES.map(Tool));
    v.push(Sep);
    v.extend([T::Move, T::Rotate, T::Scale, T::PushPull, T::Offset, T::FollowMe, T::Chamfer, T::Fillet].map(Tool));
    v.push(Sep);
    v.extend([Tool(T::Tape), Tool(T::Dimension), Tool(T::Text), Item::Text3d, Tool(T::Axes), Tool(T::Section), Tool(T::GuidePlane), Sep]);
    v.extend([Tool(T::Orbit), Tool(T::Pan), Tool(T::Zoom), Item::ZoomExtents, Sep, Item::Snapping]);
    v
}

/// The toolbar as saved (`Prefs::toolbar`, empty = the default); unknown ids are skipped.
pub fn layout(keys: &[String]) -> Vec<Item> {
    if keys.is_empty() {
        return default_layout();
    }
    keys.iter().filter_map(|k| Item::from_key(k)).collect()
}

/// The buttons not on the toolbar.
pub fn reserve(on: &[Item]) -> Vec<Item> {
    Item::all().into_iter().filter(|i| !on.contains(i)).collect()
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
        let items = layout(&self.prefs.toolbar);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for item in items {
                match item {
                    Item::Tool(id) => self.tool_button(ui, id),
                    Item::Text3d => self.action_button(ui, Icon::Text3d, false, "3D Text", None, "Add text as 3D geometry (a group).", Action::Text3d),
                    Item::ZoomExtents => self.action_button(
                        ui,
                        Icon::ZoomExtents,
                        false,
                        "Zoom Extents",
                        crate::keymap::label(crate::keymap::KeyCmd::ZoomExtents).as_deref(),
                        "Fit the whole model in the view.",
                        Action::ZoomExtents,
                    ),
                    Item::Snapping => self.snap_button(ui),
                    Item::Sep => group_sep(ui),
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_round_trips_and_skips_unknown_ids() {
        let d = default_layout();
        assert_eq!(layout(&[]), d);
        let keys: Vec<String> = d.iter().map(|i| i.key()).collect();
        assert_eq!(layout(&keys), d);
        let custom = layout(&["Line".into(), "|".into(), "Nope".into(), "Snapping".into()]);
        assert_eq!(custom, [Item::Tool(ToolId::Line), Item::Sep, Item::Snapping]);
        // the reserve: everything else, no separators
        let r = reserve(&custom);
        assert!(!r.contains(&Item::Tool(ToolId::Line)) && !r.contains(&Item::Sep) && r.contains(&Item::ZoomExtents));
        assert_eq!(r.len() + 2, Item::all().len());
    }
}
