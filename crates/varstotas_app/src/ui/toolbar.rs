//! Top toolbar with large vector-icon buttons (standard views, projection and face styles are
//! in the viewport's own controls, `view_hud.rs`).

use super::commands::Cmd;
use super::icons::Icon;
use super::{icon_button, theme};
use crate::app::{Action, VarstotasApp};
use crate::tools::ToolId;
use egui::{RichText, Ui};
use varstotas_i18n::tr;

/// Side of a toolbar button (Preferences → Interface → Toolbar icons).
static BTN_SIDE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(36);

#[allow(non_snake_case)]
fn BTN() -> f32 {
    BTN_SIDE.load(std::sync::atomic::Ordering::Relaxed) as f32
}

/// Toolbar icon sizes: id and button side.
pub const ICON_SIZES: [(&str, u32); 3] = [("Small", 28), ("Medium", 36), ("Large", 44)]; // i18n-ignore

/// The shown name of an icon size id (`ICON_SIZES`).
pub fn icon_size_label(id: &str) -> String {
    tr(match id {
        "Small" => "toolbar-size-small", // i18n-ignore
        "Medium" => "toolbar-size-medium", // i18n-ignore
        _ => "toolbar-size-large",
    })
}

/// The shown heading of a reserve group id (`GROUPS`).
pub fn group_label(id: &str) -> String {
    tr(match id {
        "Draw" => "toolbar-group-draw", // i18n-ignore
        "Tools" => "toolbar-group-tools", // i18n-ignore
        "Camera" => "toolbar-group-camera", // i18n-ignore
        "File" => "toolbar-group-file", // i18n-ignore
        "Edit" => "toolbar-group-edit", // i18n-ignore
        "Snapping" => "toolbar-group-snapping", // i18n-ignore
        _ => "toolbar-group-window",
    })
}

pub fn set_button_side(side: u32) {
    BTN_SIDE.store(side.clamp(20, 64), std::sync::atomic::Ordering::Relaxed);
}

/// One button of the toolbar (Preferences → Toolbar orders them; the rest wait in reserve).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Tool(ToolId),
    /// A menu command (File, Edit, Solid Tools, a snap …).
    Cmd(Cmd),
    /// The magnet: opens the Snapping menu.
    Snapping,
    /// A line between groups.
    Sep,
}

impl Item {
    /// The id kept in the preferences.
    pub fn key(self) -> String {
        match self {
            Item::Tool(id) => format!("{id:?}"),
            Item::Cmd(c) => c.key(),
            Item::Snapping => "Snapping".into(), // i18n-ignore
            Item::Sep => "|".into(),
        }
    }

    pub fn from_key(k: &str) -> Option<Item> {
        match k {
            "Snapping" => Some(Item::Snapping), // i18n-ignore
            "|" => Some(Item::Sep),
            _ => ToolId::ALL.iter().copied().find(|t| format!("{t:?}") == k).map(Item::Tool).or_else(|| Cmd::from_key(k).map(Item::Cmd)),
        }
    }

    pub fn name(self) -> String {
        match self {
            Item::Tool(id) => id.name().into(),
            Item::Cmd(c) => c.name(),
            Item::Snapping => tr!("toolbar-snapping-menu"),
            Item::Sep => tr!("toolbar-separator"),
        }
    }

    pub fn icon(self) -> Option<Icon> {
        match self {
            Item::Tool(id) => Some(id.icon()),
            Item::Cmd(c) => Some(c.icon()),
            Item::Snapping => Some(Icon::Magnet),
            Item::Sep => None,
        }
    }

    /// The heading it comes under in the reserve: the menu it lives in.
    pub fn group(self) -> Option<&'static str> {
        use ToolId as T;
        match self {
            Item::Tool(T::Orbit | T::Pan | T::Zoom | T::PositionCamera | T::LookAround | T::Walk) => Some("Camera"), // i18n-ignore
            Item::Tool(
                T::Line | T::Arc | T::ArcCenter | T::Arc3 | T::Pie | T::Bezier | T::SmoothCurve | T::Freehand | T::EditCurve | T::Rectangle | T::RotatedRectangle | T::Circle | T::Polygon,
            ) => Some("Draw"), // i18n-ignore
            Item::Tool(id) if crate::tools::SHAPES.contains(&id) => Some("Draw"),
            Item::Tool(_) => Some("Tools"), // i18n-ignore
            Item::Cmd(Cmd::ZoomExtents) => Some("Camera"), // i18n-ignore
            Item::Cmd(c) => Some(c.group()),
            Item::Snapping => Some("Snapping"), // i18n-ignore
            Item::Sep => None,
        }
    }

    /// Every button there is (separators aside), by menu (`GROUPS`), in each the menu's order.
    pub fn all() -> Vec<Item> {
        let mut v: Vec<Item> = ToolId::ALL.iter().copied().filter(|t| t.implemented()).map(Item::Tool).collect();
        v.push(Item::Snapping);
        v.extend(Cmd::all().into_iter().map(Item::Cmd));
        // stable: the order inside a group stays
        v.sort_by_key(|i| GROUPS.iter().position(|g| Some(*g) == i.group()).unwrap_or(GROUPS.len()));
        v
    }
}

/// The reserve's headings, in order.
pub const GROUPS: [&str; 7] = ["Draw", "Tools", "Camera", "File", "Edit", "Snapping", "Window"]; // i18n-ignore

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
    v.extend([Tool(T::Tape), Tool(T::Dimension), Tool(T::Text), Item::Cmd(Cmd::Text3d), Tool(T::Axes), Tool(T::Section), Tool(T::GuidePlane), Sep]);
    v.extend([Tool(T::Orbit), Tool(T::Pan), Tool(T::Zoom), Item::Cmd(Cmd::ZoomExtents), Sep, Item::Snapping]);
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
        if tip(resp, &id.name(), id.shortcut_label().as_deref(), &desc).clicked() {
            self.actions.push(Action::SetTool(id));
        }
    }

    /// A menu command: greyed out where it does not apply, pressed when a switch is on.
    fn cmd_button(&mut self, ui: &mut Ui, c: Cmd) {
        let enabled = c.enabled(self);
        let on = c.checked(self).unwrap_or(false);
        // no child ui here: a scope would not wrap to the next row
        let r = icon_button(ui, c.icon(), on, BTN());
        if !enabled {
            // icon_button paints at full strength: veil it
            ui.painter().rect_filled(r.rect, 7.0, theme::toolbar_bg().gamma_multiply(0.6));
        }
        super::tray_lib::spot(format!("tb:{}", c.key()), r.rect.center());
        if tip(r, &c.name(), c.shortcut().as_deref(), &c.description()).clicked() && enabled {
            self.actions.push(c.action());
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
            (false, _) => "off",
            (true, true) => "some",
            _ => "on",
        };
        let resp = tip(resp, &tr!("toolbar-snapping"), Some("Shift+S"), &tr!("toolbar-snapping-tip", state = state)); // i18n-ignore
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
                    Item::Cmd(c) => self.cmd_button(ui, c),
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
        assert!(!r.contains(&Item::Tool(ToolId::Line)) && !r.contains(&Item::Sep) && r.contains(&Item::Cmd(Cmd::ZoomExtents)));
        // every command can go on the toolbar, and comes back from its id
        assert!(r.contains(&Item::Cmd(Cmd::Solid(varstotas_kernel::SolidOp::Union))));
        for i in Item::all() {
            assert_eq!(Item::from_key(&i.key()), Some(i));
        }
        assert_eq!(r.len() + 2, Item::all().len());
    }
}
