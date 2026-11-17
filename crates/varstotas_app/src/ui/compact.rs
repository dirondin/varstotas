//! The compact interface (Preferences → Interface, F8): for phones and tablets in the web
//! build, and for anyone who wants the model on the whole window. The menu bar, toolbar, status
//! bar and side bar are hidden; floating buttons over the view stand in for them:
//! - top left: the menu button (the menus on a ring, [`Radial`]) and the panels button (the side
//!   bar's cards in a sheet);
//! - top right: the view's own controls (`view_hud.rs`), as in the classic interface;
//! - bottom: a dock with undo / redo and the common tools, the tool's hint and the Measurements
//!   value above it (a tap opens a number pad: no on-screen keyboard needed).
//!
//! The ring works with a finger: a tap opens a ring or a menu, the middle goes back; a press on
//! the menu button dragged onto an item and let go picks it (resting on a ring opens it). The
//! ring also opens at the pointer with its key (`, in either interface).

use super::commands::Cmd;
use super::icons::{Icon, paint_icon};
use super::theme;
use super::tray_lib::spot;
use crate::app::{Action, VarstotasApp};
use crate::tools::ToolId;
use egui::{Align2, Color32, FontId, Id, Key, Pos2, Rect, Response, Sense, Stroke, Ui, UiBuilder, pos2, vec2};
use serde::{Deserialize, Serialize};
use varstotas_i18n::tr;
use varstotas_render::StandardView;

/// Which interface (Preferences → Interface).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiMode {
    /// Compact on a touch screen or a narrow page (the web build), classic elsewhere.
    #[default]
    Auto,
    Classic,
    Compact,
}

/// Side of the round buttons: a fingertip.
const BUTTON: f32 = 48.0;
/// Side of the dock's buttons.
const DOCK: f32 = 44.0;
/// Gap to the window's edges.
const MARGIN: f32 = 12.0;
/// Side of a ring's item.
const ITEM: f32 = 60.0;
/// A held pointer resting this long on a ring or a menu opens it.
const REST: f64 = 0.35;
/// Narrower than this the panels fill the window.
const NARROW: f32 = 700.0;
/// The dock's tools, as many as fit, in this order.
const DOCK_TOOLS: [ToolId; 6] = [ToolId::Select, ToolId::Line, ToolId::Rectangle, ToolId::PushPull, ToolId::Move, ToolId::Tape];

/// The compact interface's state.
#[derive(Default)]
pub struct Compact {
    /// The interface is compact this frame.
    pub on: bool,
    pub radial: Option<Radial>,
    /// The side bar's cards in a sheet.
    pub panels: bool,
    /// The Measurements box's number pad.
    pub numpad: bool,
}

/// An open ring menu.
pub struct Radial {
    center: Pos2,
    /// The levels opened, the shown one last.
    levels: Vec<Level>,
    /// Opened by a press that is still held: letting go over an item picks it.
    held: bool,
    /// The item the held pointer rests on, and since when.
    resting: Option<(usize, f64)>,
    /// The submenus entered in the shown list (their labels).
    sub: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Level {
    Ring(Ring),
    List(Menu),
}

/// The rings: the menus, and the tools by group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ring {
    Root,
    Draw,
    Arcs,
    Shapes,
    Tools,
    Modify,
    Measure,
    Camera,
    Views,
}

/// The menus of the menu bar, shown as a list with finger-sized rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Menu {
    File,
    Edit,
    View,
    Camera,
    Draw,
    Tools,
    Snapping,
    Window,
    Help,
    /// The view's context menu (a right click, a long press).
    Context,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Item {
    Tool(ToolId),
    Cmd(Cmd),
    View(StandardView),
    /// Parallel projection on / off.
    Projection,
    Sub(Ring),
    List(Menu),
}

impl Ring {
    fn items(self) -> Vec<Item> {
        use Item::{List, Sub, Tool};
        use ToolId as T;
        match self {
            Ring::Root => vec![
                List(Menu::File),
                List(Menu::Edit),
                List(Menu::View),
                Sub(Ring::Camera),
                Sub(Ring::Draw),
                Sub(Ring::Tools),
                List(Menu::Snapping),
                List(Menu::Window),
                List(Menu::Help),
            ],
            Ring::Draw => vec![
                Tool(T::Line),
                Sub(Ring::Arcs),
                Tool(T::Rectangle),
                Tool(T::RotatedRectangle),
                Tool(T::Circle),
                Tool(T::Polygon),
                Sub(Ring::Shapes),
                List(Menu::Draw),
            ],
            Ring::Arcs => crate::tools::DRAW_CURVES.into_iter().map(Tool).collect(),
            Ring::Shapes => crate::tools::SHAPES.into_iter().map(Tool).chain([Item::Cmd(Cmd::Text3d)]).collect(),
            Ring::Tools => vec![
                Tool(T::Select),
                Tool(T::Eraser),
                Tool(T::Paint),
                Tool(T::PushPull),
                Tool(T::Move),
                Tool(T::Rotate),
                Tool(T::Scale),
                Sub(Ring::Modify),
                Sub(Ring::Measure),
                List(Menu::Tools),
            ],
            Ring::Modify => [T::FollowMe, T::Offset, T::Chamfer, T::Fillet, T::Section, T::Slice].into_iter().map(Tool).collect(),
            Ring::Measure => crate::tools::TOOLS_MEASURE.into_iter().map(Tool).collect(),
            Ring::Camera => vec![
                Tool(T::Orbit),
                Tool(T::Pan),
                Tool(T::Zoom),
                Item::Cmd(Cmd::ZoomExtents),
                Sub(Ring::Views),
                Item::Projection,
                Tool(T::PositionCamera),
                Tool(T::LookAround),
                Tool(T::Walk),
                List(Menu::Camera),
            ],
            Ring::Views => [
                StandardView::Iso,
                StandardView::Top,
                StandardView::Front,
                StandardView::Right,
                StandardView::Back,
                StandardView::Left,
                StandardView::Bottom,
            ]
            .into_iter()
            .map(Item::View)
            .collect(),
        }
    }

    fn label(self) -> String {
        tr(match self {
            Ring::Root => "compact-menu",
            Ring::Draw => "menu-draw",
            Ring::Arcs => "compact-arcs",
            Ring::Shapes => "menu-shapes",
            Ring::Tools => "menu-tools",
            Ring::Modify => "compact-modify",
            Ring::Measure => "compact-measure",
            Ring::Camera => "menu-camera",
            Ring::Views => "menu-standard-views",
        })
    }

    fn icon(self) -> Icon {
        match self {
            Ring::Root => Icon::More,
            Ring::Draw => Icon::Line,
            Ring::Arcs => Icon::ArcCenter,
            Ring::Shapes => Icon::Box3d,
            Ring::Tools => Icon::Move,
            Ring::Modify => Icon::FollowMe,
            Ring::Measure => Icon::Tape,
            Ring::Camera => Icon::Orbit,
            Ring::Views => Icon::ViewIso,
        }
    }

    /// The ring holds `tool`, or a ring under it does.
    fn holds(self, tool: ToolId) -> bool {
        self.items().into_iter().any(|i| match i {
            Item::Tool(t) => t == tool,
            Item::Sub(r) => r.holds(tool),
            _ => false,
        })
    }
}

impl Menu {
    fn label(self) -> String {
        tr(match self {
            Menu::File => "menu-file",
            Menu::Edit => "menu-edit",
            Menu::View => "menu-view",
            Menu::Camera => "menu-camera",
            Menu::Draw => "menu-draw",
            Menu::Tools => "menu-tools",
            Menu::Snapping => "menu-snapping",
            Menu::Window => "menu-window",
            Menu::Help => "menu-help",
            Menu::Context => "compact-context",
        })
    }

    fn icon(self) -> Icon {
        match self {
            Menu::File => Icon::FileOpen,
            Menu::Edit => Icon::Cut,
            Menu::View => Icon::StyleShaded,
            Menu::Camera => Icon::Orbit,
            Menu::Draw => Icon::Line,
            Menu::Tools => Icon::Move,
            Menu::Snapping => Icon::Magnet,
            Menu::Window => Icon::SideBar,
            Menu::Help => Icon::Info,
            Menu::Context => Icon::More,
        }
    }

    /// Its switches are toggled several at a time: a click does not close it.
    fn stays_open(self) -> bool {
        matches!(self, Menu::View | Menu::Snapping)
    }
}

impl Item {
    /// A name for the QA scripts' spots.
    fn key(self) -> String {
        match self {
            Item::Tool(t) => format!("{t:?}"),
            Item::Cmd(c) => c.key(),
            Item::View(v) => format!("{v:?}"),
            Item::Projection => "Projection".into(), // i18n-ignore
            Item::Sub(r) => format!("{r:?}"),
            Item::List(m) => format!("menu:{m:?}"),
        }
    }
}

/// The compact interface is wanted in `mode` on this device.
fn wanted(mode: UiMode, ctx: &egui::Context) -> bool {
    match mode {
        UiMode::Classic => false,
        UiMode::Compact => true,
        UiMode::Auto => cfg!(target_arch = "wasm32") && (touch_screen() || ctx.content_rect().width() < NARROW),
    }
}

/// The page's main pointer is a finger (`pointer: coarse`).
fn touch_screen() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        thread_local! {
            static COARSE: bool = web_sys::window()
                .and_then(|w| w.match_media("(pointer: coarse)").ok().flatten())
                .is_some_and(|m| m.matches());
        }
        COARSE.with(|c| *c)
    }
    #[cfg(not(target_arch = "wasm32"))]
    false
}

/// A soft shadow under a floating control.
fn shadow(ui: &Ui, rect: Rect, radius: f32) {
    let s = egui::epaint::Shadow { offset: [0, 2], blur: 10, spread: 0, color: Color32::from_black_alpha(if theme::dark() { 90 } else { 45 }) };
    ui.painter().add(s.as_shape(rect, radius));
}

/// A round floating button with an icon.
fn round_button(ui: &mut Ui, rect: Rect, icon: Icon, accent: bool, on: bool, id: Id, tip: &str) -> Response {
    let r = ui.interact(rect, id, Sense::click_and_drag());
    let radius = rect.width() / 2.0;
    shadow(ui, rect, radius);
    let fill = if accent {
        theme::ACCENT
    } else if on {
        theme::accent_soft()
    } else if r.is_pointer_button_down_on() {
        theme::hover_bg()
    } else {
        theme::card_bg()
    };
    ui.painter().circle(rect.center(), radius, fill, Stroke::new(1.0, Color32::from_black_alpha(25)));
    let side = (rect.width() * 0.5).round();
    let icon_rect = Rect::from_center_size(rect.center(), vec2(side, side));
    if accent {
        // white on the accent: the icon drawn in the light ink of the dark look
        let painter = ui.painter().clone();
        paint_white_icon(&painter, icon_rect, icon);
    } else {
        paint_icon(ui.painter(), icon_rect, icon, on);
    }
    r.on_hover_text(tip)
}

/// Three dots on a ring, drawn by hand: white on the accent button.
fn paint_white_icon(painter: &egui::Painter, rect: Rect, icon: Icon) {
    if icon == Icon::More {
        let c = rect.center();
        let d = rect.width() * 0.32;
        for (dx, dy) in [(0.0, -d), (d, 0.0), (0.0, d), (-d, 0.0)] {
            painter.circle_filled(c + vec2(dx, dy), rect.width() * 0.09, Color32::WHITE);
        }
        painter.circle_stroke(c, rect.width() * 0.16, Stroke::new(1.8, Color32::WHITE));
    } else {
        paint_icon(painter, rect, icon, true);
    }
}

impl VarstotasApp {
    /// Decide the interface for this frame (Preferences → Interface; scripted runs keep the
    /// classic one unless they ask).
    pub(crate) fn compact_frame(&mut self, ctx: &egui::Context) {
        let mode = if self.debug.script_active() && self.prefs.ui_mode == UiMode::Auto { UiMode::Classic } else { self.prefs.ui_mode };
        self.compact.on = wanted(mode, ctx);
        if !self.compact.on {
            self.compact.panels = false;
            self.compact.numpad = false;
        }
    }

    /// For the QA scripts: (on, the ring's open level, panels, number pad).
    pub(crate) fn compact_dump(&self) -> String {
        let level = self.compact.radial.as_ref().and_then(|r| r.levels.last().copied());
        let sub = self.compact.radial.as_ref().map(|r| r.sub.clone()).unwrap_or_default();
        format!("({} {level:?} {} {}) compact_sub={sub:?}", self.compact.on, self.compact.panels, self.compact.numpad)
    }

    /// F8: the other interface.
    pub(crate) fn toggle_compact(&mut self) {
        self.prefs.ui_mode = if self.compact.on { UiMode::Classic } else { UiMode::Compact };
    }

    /// Open the ring menu at `at` (None: the middle of the view), on its first ring.
    pub(crate) fn open_radial(&mut self, at: Option<Pos2>, held: bool) {
        let center = at.unwrap_or_else(|| self.viewport.view_rect.center());
        self.compact.radial = Some(Radial { center, levels: vec![Level::Ring(Ring::Root)], held, resting: None, sub: Vec::new() });
    }

    /// The view's context menu as a list, for what a right click or a long press at `at` picked.
    pub(crate) fn open_context_list(&mut self, at: Pos2) {
        self.compact.radial = Some(Radial { center: at, levels: vec![Level::List(Menu::Context)], held: false, resting: None, sub: Vec::new() });
    }

    /// The floating buttons over the view `rect` (the central panel's `ui`).
    pub(crate) fn compact_chrome(&mut self, ui: &mut Ui, rect: Rect) {
        // top left: the menu and the panels
        let menu_rect = Rect::from_min_size(rect.min + vec2(MARGIN, MARGIN), vec2(BUTTON, BUTTON));
        let menu = round_button(ui, menu_rect, Icon::More, true, false, Id::new("compact_menu"), &tr!("compact-menu"));
        spot("compact:menu", menu_rect.center());
        if menu.drag_started() || (menu.clicked() && self.compact.radial.is_none()) {
            // a press that is dragged on picks from the ring as it lets go
            let held = menu.drag_started();
            let center = rect.center();
            self.open_radial(Some(center), held);
        }
        let panels_rect = menu_rect.translate(vec2(BUTTON + 8.0, 0.0));
        let panels = round_button(ui, panels_rect, Icon::SideBar, false, self.compact.panels, Id::new("compact_panels"), &tr!("compact-panels"));
        spot("compact:panels", panels_rect.center());
        if panels.clicked() {
            self.compact.panels = !self.compact.panels;
        }
        self.nav_button_rects.extend([menu_rect, panels_rect]);

        let dock = self.compact_dock(ui, rect);
        self.nav_button_rects.push(dock);
        self.compact_status(ui, rect, dock);
    }

    /// The dock at the bottom: undo, redo, the common tools as many as fit, all tools (the
    /// Tools ring). Returns its rect.
    fn compact_dock(&mut self, ui: &mut Ui, rect: Rect) -> Rect {
        const PAD: f32 = 6.0;
        const SEP: f32 = 13.0;
        let room = rect.width() - 2.0 * MARGIN - 2.0 * PAD;
        // undo, redo and all tools stay; the tools take what is left
        let fit = ((room - 3.0 * DOCK - 2.0 * SEP) / DOCK).floor().max(0.0) as usize;
        let tools = &DOCK_TOOLS[..fit.min(DOCK_TOOLS.len())];
        let width = (3 + tools.len()) as f32 * DOCK + 2.0 * SEP + 2.0 * PAD;
        let dock = Rect::from_center_size(pos2(rect.center().x, rect.bottom() - MARGIN - (DOCK + 2.0 * PAD) / 2.0), vec2(width, DOCK + 2.0 * PAD));
        shadow(ui, dock, 16.0);
        ui.painter().rect(dock, 16.0, theme::card_bg(), Stroke::new(1.0, Color32::from_black_alpha(25)), egui::StrokeKind::Inside);
        let top = dock.top() + PAD;
        let mut x = dock.left() + PAD;
        let slot = |x: &mut f32| {
            let r = Rect::from_min_size(pos2(*x, top), vec2(DOCK, DOCK));
            *x += DOCK;
            r
        };
        let sep = |ui: &Ui, x: &mut f32| {
            ui.painter().vline(*x + SEP / 2.0, dock.shrink(14.0).y_range(), Stroke::new(1.0, theme::border()));
            *x += SEP;
        };
        for c in [Cmd::Undo, Cmd::Redo] {
            let r = slot(&mut x);
            let enabled = c.enabled(self);
            if self.dock_button(ui, r, c.icon(), false, enabled, &c.menu_label(), &format!("dock:{}", c.key())) {
                self.actions.push(c.action());
            }
        }
        sep(ui, &mut x);
        let current = self.tool_id();
        for &t in tools {
            let r = slot(&mut x);
            if self.dock_button(ui, r, t.icon(), current == t, true, &t.name(), &format!("dock:{t:?}")) {
                self.actions.push(Action::SetTool(t));
            }
        }
        sep(ui, &mut x);
        let r = slot(&mut x);
        let other = !tools.contains(&current);
        if self.dock_button(ui, r, Icon::More, other, true, &tr!("compact-all-tools"), "dock:all") {
            let center = rect.center();
            self.open_radial(Some(center), false);
            if let Some(radial) = self.compact.radial.as_mut() {
                radial.levels.push(Level::Ring(Ring::Tools));
            }
        }
        dock
    }

    fn dock_button(&mut self, ui: &mut Ui, r: Rect, icon: Icon, on: bool, enabled: bool, tip: &str, name: &str) -> bool {
        let resp = ui.interact(r, Id::new(name), if enabled { Sense::click() } else { Sense::hover() });
        spot(name.to_string(), r.center());
        let p = ui.painter();
        if on {
            p.rect(r, 10.0, theme::accent_soft(), Stroke::new(1.0, theme::ACCENT), egui::StrokeKind::Inside);
        } else if resp.is_pointer_button_down_on() {
            p.rect_filled(r, 10.0, theme::hover_bg());
        }
        let mut painter = p.clone();
        if !enabled {
            painter.multiply_opacity(0.35);
        }
        paint_icon(&painter, Rect::from_center_size(r.center(), vec2(26.0, 26.0)), icon, on);
        resp.on_hover_text(tip).clicked()
    }

    /// Above the dock: the tool's hint (or a message) on the left, the Measurements value on
    /// the right (a tap opens the number pad).
    fn compact_status(&mut self, ui: &mut Ui, rect: Rect, dock: Rect) {
        const H: f32 = 34.0;
        let y = dock.top() - 8.0 - H;
        let left = rect.left() + MARGIN;
        let right = rect.right() - MARGIN;
        // the Measurements chip
        let typing = !self.vcb.is_empty();
        let label = if self.grid_vcb() { tr!("status-grid-step") } else { self.tool.vcb_label() };
        let value = if typing {
            self.vcb.clone()
        } else if self.grid_vcb() {
            crate::tools::common::fmt_len(&self.doc.model, self.prefs.grid.step)
        } else {
            self.tool.vcb_value()
        };
        let font = FontId::proportional(14.0);
        let mut chip_left = right;
        if !label.is_empty() || !value.is_empty() {
            let p = ui.painter();
            let lg = p.layout_no_wrap(label.clone(), font.clone(), theme::text_dim());
            let vg = p.layout_no_wrap(if value.is_empty() { "—".into() } else { value }, FontId::monospace(14.0), theme::text());
            let w = 14.0 + lg.size().x + 8.0 + vg.size().x + 14.0;
            let chip = Rect::from_min_size(pos2(right - w, y), vec2(w, H));
            let resp = ui.interact(chip, Id::new("compact_vcb"), Sense::click());
            spot("compact:vcb", chip.center());
            shadow(ui, chip, H / 2.0);
            let border = if self.vcb_error {
                Color32::from_rgb(0xD0, 0x40, 0x30)
            } else if typing || self.compact.numpad {
                theme::ACCENT
            } else {
                Color32::from_black_alpha(25)
            };
            let p = ui.painter();
            p.rect(chip, H / 2.0, theme::card_bg(), Stroke::new(if typing { 1.5 } else { 1.0 }, border), egui::StrokeKind::Inside);
            p.galley(pos2(chip.left() + 14.0, chip.center().y - lg.size().y / 2.0), lg.clone(), theme::text_dim());
            p.galley(pos2(chip.left() + 22.0 + lg.size().x, chip.center().y - vg.size().y / 2.0), vg, theme::text());
            if resp.clicked() {
                self.compact.numpad = !self.compact.numpad;
            }
            self.nav_button_rects.push(chip);
            chip_left = chip.left() - 8.0;
        }
        // the hint, or a message
        let (text, flash) = self.status_text();
        let (text, fg, bg) = match flash {
            Some((msg, true)) => (msg, Color32::WHITE, Color32::from_rgb(0xB0, 0x30, 0x22)),
            Some((msg, false)) => (msg, Color32::WHITE, theme::ACCENT),
            None => (text, Color32::WHITE, Color32::from_rgba_unmultiplied(0x1F, 0x23, 0x28, 210)),
        };
        let room = chip_left - left - 24.0;
        if room < 60.0 || text.is_empty() {
            return;
        }
        let mut job = egui::text::LayoutJob::simple_singleline(text, font, fg);
        job.wrap = egui::text::TextWrapping::truncate_at_width(room);
        let g = ui.painter().layout_job(job);
        let hint = Rect::from_min_size(pos2(left, y), vec2(g.size().x + 24.0, H));
        let p = ui.painter();
        p.rect_filled(hint, H / 2.0, bg);
        p.galley(pos2(hint.left() + 12.0, hint.center().y - g.size().y / 2.0), g, fg);
    }

    /// The ring menu, the panels and the number pad (over everything but dialogs).
    pub(crate) fn compact_overlays(&mut self, ctx: &egui::Context) {
        self.radial_ui(ctx);
        if self.compact.on && self.compact.panels {
            self.panels_sheet(ctx);
        }
        if self.compact.on && self.compact.numpad {
            self.numpad_sheet(ctx);
        }
    }

    fn radial_ui(&mut self, ctx: &egui::Context) {
        let Some(level) = self.compact.radial.as_ref().and_then(|r| r.levels.last().copied()) else { return };
        let screen = ctx.content_rect();
        egui::Area::new(Id::new("compact_radial")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            let backdrop = ui.allocate_rect(screen, Sense::click());
            ui.painter().rect_filled(screen, 0.0, Color32::from_black_alpha(130));
            let close = match level {
                Level::Ring(ring) => self.ring_ui(ui, screen, ring, &backdrop),
                Level::List(menu) => self.list_ui(ui, screen, menu, &backdrop),
            };
            if close {
                self.radial_back(true);
            } else if ui.input(|i| i.key_pressed(Key::Escape)) {
                // out of a submenu first
                match self.compact.radial.as_mut() {
                    Some(r) if !r.sub.is_empty() => {
                        r.sub.pop();
                    }
                    _ => self.radial_back(false),
                }
            }
        });
    }

    /// One level back (`all`: close the ring).
    fn radial_back(&mut self, all: bool) {
        if let Some(r) = self.compact.radial.as_mut() {
            r.levels.pop();
            r.sub.clear();
            r.resting = None;
            if all || r.levels.is_empty() {
                self.compact.radial = None;
            }
        }
    }

    /// Run an item; true when the ring is done.
    fn radial_pick(&mut self, item: Item) -> bool {
        let open = |app: &mut Self, level: Level| {
            if let Some(r) = app.compact.radial.as_mut() {
                r.levels.push(level);
                r.sub.clear();
                r.resting = None;
            }
            false
        };
        match item {
            Item::Tool(t) => self.actions.push(Action::SetTool(t)),
            Item::Cmd(c) => {
                if !c.enabled(self) {
                    return false;
                }
                self.actions.push(c.action());
            }
            Item::View(v) => self.actions.push(Action::View(v)),
            Item::Projection => self.actions.push(Action::SetParallel(!self.camera.ortho)),
            Item::Sub(r) => return open(self, Level::Ring(r)),
            Item::List(m) => return open(self, Level::List(m)),
        }
        true
    }

    /// The items of `ring` round the middle; returns true to close the menu.
    fn ring_ui(&mut self, ui: &mut Ui, screen: Rect, ring: Ring, backdrop: &Response) -> bool {
        let Some(radial) = self.compact.radial.as_ref() else { return true };
        let items = ring.items();
        let n = items.len();
        let radius = (n as f32 * (ITEM + 30.0) / std::f32::consts::TAU).max(120.0);
        let reach = radius + ITEM / 2.0 + 30.0;
        let inner = screen.shrink2(vec2(reach.min(screen.width() / 2.0), reach.min(screen.height() / 2.0)));
        let c = inner.clamp(radial.center);
        let (held, resting) = (radial.held, radial.resting);
        let now = ui.input(|i| i.time);
        let pointer = ui.input(|i| i.pointer.latest_pos());
        let released = ui.input(|i| i.pointer.primary_released());
        let current = self.tool_id();

        // the middle: back, or close on the first ring
        let mid = Rect::from_center_size(c, vec2(84.0, 84.0));
        let mid_resp = ui.interact(mid, Id::new("radial_mid"), Sense::click());
        spot("radial:back", c);
        {
            let p = ui.painter();
            p.circle(c, 42.0, Color32::from_rgba_unmultiplied(0x1F, 0x23, 0x28, 235), Stroke::new(2.0, Color32::from_white_alpha(140)));
            let (title, sub) = if ring == Ring::Root { (tr!("compact-close"), String::new()) } else { (ring.label(), tr!("compact-back")) };
            let g = p.layout(title, FontId::proportional(13.0), Color32::WHITE, 74.0);
            let total = g.size().y + if sub.is_empty() { 0.0 } else { 16.0 };
            let top = c.y - total / 2.0;
            p.galley(pos2(c.x - g.size().x / 2.0, top), g.clone(), Color32::WHITE);
            if !sub.is_empty() {
                p.text(pos2(c.x, top + g.size().y + 2.0), Align2::CENTER_TOP, sub, FontId::proportional(11.0), Color32::from_white_alpha(200));
            }
        }

        let mut picked = None;
        let mut over = None;
        for (i, &item) in items.iter().enumerate() {
            let a = (-90.0 + i as f32 * 360.0 / n as f32).to_radians();
            let at = c + vec2(a.cos(), a.sin()) * radius;
            let r = Rect::from_center_size(at, vec2(ITEM, ITEM));
            let resp = ui.interact(r, Id::new(("radial_item", i)), Sense::click());
            spot(format!("radial:{}", item.key()), at);
            let hot = resp.hovered() || (held && pointer.is_some_and(|p| r.contains(p)));
            if hot {
                over = Some(i);
            }
            let (label, icon, on, enabled) = match item {
                Item::Tool(t) => (t.name(), t.icon(), t == current, true),
                Item::Cmd(c) => (c.menu_label().trim_end_matches('…').to_string(), c.icon(), false, c.enabled(self)),
                Item::View(v) => (v.name(), super::view_hud::view_icon(v), false, true),
                Item::Projection => (tr!("view-parallel"), Icon::Parallel, self.camera.ortho, true),
                Item::Sub(s) => (s.label(), s.icon(), s.holds(current), true),
                Item::List(m) => (if ring == Ring::Root { m.label() } else { tr!("compact-all") }, if ring == Ring::Root { m.icon() } else { Icon::More }, false, true),
            };
            shadow(ui, r, ITEM / 2.0);
            let fill = if on {
                theme::accent_soft()
            } else if hot {
                theme::hover_bg()
            } else {
                theme::card_bg()
            };
            let stroke = if on { Stroke::new(2.0, theme::ACCENT) } else { Stroke::new(1.0, Color32::from_black_alpha(30)) };
            let mut p = ui.painter().clone();
            p.circle(at, ITEM / 2.0, fill, stroke);
            if !enabled {
                p.multiply_opacity(0.4);
            }
            paint_icon(&p, Rect::from_center_size(at, vec2(28.0, 28.0)), icon, on);
            // (on the first ring every item opens one)
            if ring != Ring::Root && matches!(item, Item::Sub(_) | Item::List(_)) {
                // more behind it: a dot on the rim, away from the middle
                let dot = at + vec2(a.cos(), a.sin()) * (ITEM / 2.0 - 2.0);
                p.circle(dot, 6.0, theme::ACCENT, Stroke::new(1.5, Color32::WHITE));
            }
            let g = p.layout(label, FontId::proportional(12.0), Color32::WHITE, ITEM + 34.0);
            let top = at.y + ITEM / 2.0 + 3.0;
            let back = Rect::from_min_size(pos2(at.x - g.size().x / 2.0 - 5.0, top), g.size() + vec2(10.0, 2.0));
            p.rect_filled(back, 5.0, Color32::from_black_alpha(120));
            p.galley(pos2(at.x - g.size().x / 2.0, top + 1.0), g, Color32::WHITE);
            if resp.clicked() {
                picked = Some(item);
            }
        }

        if held {
            let radial = self.compact.radial.as_mut().expect("open");
            if released {
                // let go over an item: that one; elsewhere: the ring stays for taps
                radial.held = false;
                picked = over.map(|i| items[i]);
            } else if let Some(i) = over.filter(|&i| matches!(items[i], Item::Sub(_) | Item::List(_))) {
                // resting on a ring or a menu opens it, the press still held
                match resting {
                    Some((j, since)) if j == i && now - since > REST => {
                        picked = Some(items[i]);
                    }
                    Some((j, _)) if j == i => ui.ctx().request_repaint(),
                    _ => {
                        radial.resting = Some((i, now));
                        ui.ctx().request_repaint();
                    }
                }
            } else {
                radial.resting = None;
            }
        }
        if let Some(item) = picked {
            return self.radial_pick(item);
        }
        if mid_resp.clicked() {
            self.radial_back(false);
            return false;
        }
        // a tap beside the ring closes it
        backdrop.clicked() && over.is_none()
    }

    /// A menu of the menu bar with finger-sized rows; returns true to close.
    fn list_ui(&mut self, ui: &mut Ui, screen: Rect, menu: Menu, backdrop: &Response) -> bool {
        let w = (screen.width() - 2.0 * MARGIN).min(440.0);
        let h = (screen.height() - 2.0 * MARGIN).min(640.0);
        let panel = Rect::from_center_size(screen.center(), vec2(w, h));
        let before = self.actions.len();
        let mut back = false;
        let mut close = false;
        let mut should_close = false;
        let path = self.compact.radial.as_ref().map(|r| r.sub.clone()).unwrap_or_default();
        let title = path.last().cloned().unwrap_or_else(|| menu.label());
        let mut drilled = None;
        ui.scope_builder(UiBuilder::new().max_rect(panel).closable(), |ui| {
            egui::Frame::new().fill(theme::card_bg()).corner_radius(18.0).inner_margin(egui::Margin::same(12)).shadow(egui::epaint::Shadow {
                offset: [0, 4],
                blur: 18,
                spread: 0,
                color: Color32::from_black_alpha(70),
            }).show(ui, |ui| {
                ui.set_min_size(panel.size() - vec2(24.0, 24.0));
                ui.set_max_size(panel.size() - vec2(24.0, 24.0));
                ui.horizontal(|ui| {
                    let b = ui.add(egui::Button::new(egui::RichText::new(format!("‹  {title}")).size(18.0).strong()).frame(false).min_size(vec2(0.0, 40.0)));
                    spot("radial:back", b.rect.center());
                    back = b.clicked();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let x = ui.add(egui::Button::new("").min_size(vec2(40.0, 40.0)).corner_radius(20.0));
                        paint_cross(ui.painter(), x.rect.center(), 7.0, theme::text());
                        spot("radial:close", x.rect.center());
                        close = x.clicked();
                    });
                });
                ui.separator();
                // a submenu opens in the panel, its rows in place of the menu's
                drilled = Some(super::menu::drilled(ui, &path, list_rows, |ui| match menu {
                        Menu::File => self.menu_file(ui),
                        Menu::Edit => self.menu_edit(ui),
                        Menu::View => self.menu_view(ui),
                        Menu::Camera => self.menu_camera(ui),
                        Menu::Draw => self.menu_draw(ui),
                        Menu::Tools => self.menu_tools(ui),
                        Menu::Snapping => self.snapping_menu(ui, true),
                        Menu::Window => self.menu_window(ui),
                        Menu::Help => self.menu_help(ui),
                        Menu::Context => self.viewport_context_menu(ui),
                }));
            });
            should_close = ui.should_close();
        });
        let (opened, sub_closed) = drilled.map_or((None, false), |d| (d.opened, d.closed));
        if let Some(label) = opened
            && let Some(r) = self.compact.radial.as_mut()
        {
            r.sub.push(label);
            return false;
        }
        if back {
            match self.compact.radial.as_mut() {
                Some(r) if !r.sub.is_empty() => {
                    r.sub.pop();
                }
                _ => self.radial_back(false),
            }
            return false;
        }
        let acted = self.actions.len() > before && !menu.stays_open();
        // a tap beside the panel closes the menu too
        let outside = backdrop.clicked() && ui.input(|i| i.pointer.interact_pos()).is_some_and(|p| !panel.contains(p));
        close || should_close || sub_closed || acted || outside
    }

    /// The side bar's cards: the whole window on a phone, a card on the right on wider screens.
    fn panels_sheet(&mut self, ctx: &egui::Context) {
        let screen = ctx.content_rect();
        let narrow = screen.width() < NARROW;
        let sheet = if narrow {
            Rect::from_min_max(screen.min + vec2(0.0, 24.0), screen.max)
        } else {
            Rect::from_min_max(pos2(screen.right() - MARGIN - 380.0, screen.top() + MARGIN), screen.max - vec2(MARGIN, MARGIN))
        };
        let mut done = false;
        egui::Area::new(Id::new("compact_panels_sheet")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            if narrow {
                ui.painter().rect_filled(screen, 0.0, Color32::from_black_alpha(100));
            }
            // taps beside the sheet close it (on a phone: only the strip above it)
            let backdrop = ui.allocate_rect(screen, Sense::click());
            let corner = if narrow { egui::CornerRadius { nw: 18, ne: 18, sw: 0, se: 0 } } else { egui::CornerRadius::same(18) };
            shadow(ui, sheet, 18.0);
            ui.painter().rect_filled(sheet, corner, theme::tray_bg());
            let inner = sheet.shrink2(vec2(10.0, 8.0));
            ui.scope_builder(UiBuilder::new().max_rect(inner).id_salt("compact_panels"), |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(tr!("compact-panels")).size(18.0).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let b = ui.add(egui::Button::new(egui::RichText::new(tr!("compact-done")).size(15.0).strong()).min_size(vec2(72.0, 40.0)).corner_radius(20.0));
                        spot("compact:done", b.rect.center());
                        done = b.clicked();
                    });
                });
                ui.add_space(4.0);
                self.tray_ui(ui);
            });
            if backdrop.clicked() && ui.input(|i| i.pointer.interact_pos()).is_some_and(|p| !sheet.contains(p)) {
                done = true;
            }
        });
        if done || ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.compact.panels = false;
        }
    }

    /// The Measurements box's number pad at the bottom: digits, the separators of the
    /// Measurements syntax, Enter.
    fn numpad_sheet(&mut self, ctx: &egui::Context) {
        let screen = ctx.content_rect();
        const KEY: f32 = 50.0;
        const GAP: f32 = 8.0;
        let w = (screen.width() - 2.0 * MARGIN).min(420.0);
        let h = 4.0 * KEY + 3.0 * GAP + 96.0;
        let sheet = Rect::from_min_size(pos2(screen.center().x - w / 2.0, screen.bottom() - MARGIN - h), vec2(w, h));
        let mut close = false;
        egui::Area::new(Id::new("compact_numpad")).order(egui::Order::Foreground).fixed_pos(sheet.min).show(ctx, |ui| {
            shadow(ui, sheet, 18.0);
            ui.painter().rect_filled(sheet, 18.0, theme::card_bg());
            let inner = sheet.shrink(12.0);
            // the line typed, or the tool's value
            let label = if self.grid_vcb() { tr!("status-grid-step") } else { self.tool.vcb_label() };
            let field = Rect::from_min_size(inner.min, vec2(inner.width(), 48.0));
            let border = if self.vcb_error { Color32::from_rgb(0xD0, 0x40, 0x30) } else { theme::ACCENT };
            let p = ui.painter();
            p.rect(field, 12.0, theme::field_bg(), Stroke::new(2.0, border), egui::StrokeKind::Inside);
            p.text(field.left_center() + vec2(12.0, 0.0), Align2::LEFT_CENTER, &label, FontId::proportional(13.0), theme::text_dim());
            let shown = if self.vcb.is_empty() { self.tool.vcb_value() } else { self.vcb.clone() };
            let color = if self.vcb.is_empty() { theme::text_dim() } else { theme::text() };
            p.text(field.right_center() - vec2(12.0, 0.0), Align2::RIGHT_CENTER, shown, FontId::monospace(22.0), color);
            let keys: [[&str; 4]; 4] = [["7", "8", "9", "⌫"], ["4", "5", "6", ";"], ["1", "2", "3", "-"], [".", "0", "✕", "⏎"]];
            let kw = (inner.width() - 3.0 * GAP) / 4.0;
            let top = field.bottom() + 12.0;
            for (row, line) in keys.iter().enumerate() {
                for (col, &k) in line.iter().enumerate() {
                    let r = Rect::from_min_size(pos2(inner.left() + col as f32 * (kw + GAP), top + row as f32 * (KEY + GAP)), vec2(kw, KEY));
                    let resp = ui.interact(r, Id::new(("numpad", k)), Sense::click());
                    spot(format!("numpad:{k}"), r.center());
                    let enter = k == "⏎";
                    let fill = if enter {
                        theme::ACCENT
                    } else if resp.is_pointer_button_down_on() {
                        theme::hover_bg()
                    } else if k.chars().all(|c| c.is_ascii_digit()) {
                        theme::field_bg()
                    } else {
                        theme::toolbar_bg()
                    };
                    let p = ui.painter();
                    p.rect(r, 12.0, fill, Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
                    let fg = if enter { Color32::WHITE } else { theme::text() };
                    match k {
                        // drawn: not every font has these signs
                        "✕" => paint_cross(p, r.center(), 8.0, fg),
                        "⏎" => {
                            let c = r.center();
                            p.line(vec![c + vec2(-11.0, 0.0), c + vec2(-3.0, 8.0), c + vec2(12.0, -8.0)], Stroke::new(2.5, fg));
                        }
                        _ => {
                            p.text(r.center(), Align2::CENTER_CENTER, k, FontId::proportional(22.0), fg);
                        }
                    }
                    if resp.clicked() {
                        match k {
                            "⌫" => {
                                self.vcb.pop();
                            }
                            "✕" => {
                                self.vcb.clear();
                                self.vcb_error = false;
                                close = true;
                            }
                            "⏎" => {
                                if !self.vcb.is_empty() {
                                    self.submit_vcb();
                                }
                                close = !self.vcb_error;
                            }
                            _ => {
                                if self.vcb_error {
                                    self.vcb.clear();
                                    self.vcb_error = false;
                                }
                                self.vcb.push_str(k);
                            }
                        }
                    }
                }
            }
        });
        if close {
            self.compact.numpad = false;
        }
    }
}

/// A list's rows: finger-sized, as wide as the panel, scrolled.
fn list_rows(ui: &mut Ui, add: &mut dyn FnMut(&mut Ui)) {
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        touch_spacing(ui);
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| add(ui));
    });
}

/// An × of half-side `r` at `c`.
fn paint_cross(p: &egui::Painter, c: Pos2, r: f32, color: Color32) {
    let s = Stroke::new(2.0, color);
    p.line_segment([c + vec2(-r, -r), c + vec2(r, r)], s);
    p.line_segment([c + vec2(-r, r), c + vec2(r, -r)], s);
}

/// Rows a finger can hit: taller buttons, larger text.
fn touch_spacing(ui: &mut Ui) {
    let s = ui.style_mut();
    // flat rows, as in a menu
    s.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    s.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    s.spacing.interact_size.y = 40.0;
    s.spacing.button_padding = vec2(10.0, 9.0);
    s.spacing.item_spacing.y = 4.0;
    for (style, font) in s.text_styles.iter_mut() {
        if matches!(style, egui::TextStyle::Body | egui::TextStyle::Button) {
            font.size = 16.0;
        }
    }
}
