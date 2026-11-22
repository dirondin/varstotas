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
/// A phone's sheet: its side margins.
const SHEET_PAD: f32 = 16.0;
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
    /// The hint opened in full by a tap (while it is the one shown).
    pub hint_open: Option<String>,
    /// The one side bar card the sheet shows (None: all of them).
    pub card: Option<&'static str>,
    /// A tap the app makes itself (back on a window: its ✕), pressed and then let go.
    tap: Option<(Pos2, bool)>,
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
    /// A long press on the dock: on its tool in this place (move, remove, replace), or on all
    /// tools (add one).
    Dock(Option<usize>),
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
            Menu::Dock(_) => "compact-dock",
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
            Menu::Dock(_) => Icon::Select,
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

thread_local! {
    /// This frame: the interface is compact, and the page is a phone's (narrow).
    static FIT: std::cell::Cell<(bool, bool)> = const { std::cell::Cell::new((false, false)) };
    static INSETS: std::cell::Cell<Insets> = const { std::cell::Cell::new(Insets { top: 0.0, bottom: 0.0, left: 0.0, right: 0.0, keyboard: 0.0 }) };
    static TEST_KEYBOARD: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    /// The QA scripts' phone: its status bar and home bar (top, bottom).
    static TEST_SAFE: std::cell::Cell<(f32, f32)> = const { std::cell::Cell::new((0.0, 0.0)) };
    /// The sheet's handle (a QA spot, given once the frame's spots are cleared).
    static HANDLE: std::cell::Cell<Option<Pos2>> = const { std::cell::Cell::new(None) };
    /// The dialog shown as a sheet the last frame.
    static SHEET: std::cell::Cell<Option<egui::LayerId>> = const { std::cell::Cell::new(None) };
    /// The sheet's handle is held (where it was pressed), and how far it is pulled down.
    static SWIPE: std::cell::Cell<(Option<Pos2>, f32)> = const { std::cell::Cell::new((None, 0.0)) };
    /// Dialogs shown as sheets the last frame.
    static SHOWN: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// A window as the compact interface shows it (call last on the builder): on a phone the whole
/// page, borderless, not moved or resized; on a tablet kept inside the page.
pub fn fit_window<'a>(ctx: &egui::Context, win: egui::Window<'a>) -> egui::Window<'a> {
    let (on, phone) = FIT.get();
    if !on {
        return win;
    }
    let screen = ctx.content_rect();
    if phone {
        // the whole page, no border, corners or shadow; the status bar's room above the title,
        // the home bar's below the content (egui gives a window's margins to its title bar too)
        let i = safe_area();
        let m = |v: f32| (12.0 + v).min(100.0) as i8;
        let frame = egui::Frame::window(&ctx.global_style()).corner_radius(0).stroke(Stroke::NONE).shadow(egui::epaint::Shadow::NONE);
        let title = frame.inner_margin(egui::Margin { left: m(i.left), right: m(i.right), top: m(i.top), bottom: m(0.0) });
        let frame = frame.inner_margin(egui::Margin { left: m(i.left), right: m(i.right), top: m(0.0), bottom: m(i.bottom) });
        // above the keyboard
        let sheet = Rect::from_min_max(screen.min, screen.max - vec2(0.0, i.keyboard));
        // what a window lays out wider or taller than a phone scrolls
        win.fixed_rect(sheet).collapsible(false).frame(frame).title_frame(title).scroll(true)
    } else {
        let inside = screen.shrink(MARGIN);
        win.constrain_to(inside).max_size(inside.size())
    }
}

/// This frame the interface is compact on a phone's page: windows are sheets, side by side
/// layouts go one above the other.
pub fn phone() -> bool {
    FIT.get().1
}

/// Where the view's top-left notes (the group being edited) go in the view `rect`: under the
/// floating buttons in the compact interface, `None` in the classic one.
pub fn below_buttons(rect: Rect) -> Option<Pos2> {
    FIT.get().0.then(|| {
        let r = safe_area().shrink(rect);
        r.min + vec2(MARGIN, MARGIN + BUTTON + 8.0)
    })
}

/// A dialog's width `w`, narrower on a page that has no room for it; on a phone the page's.
pub fn fit_width(ctx: &egui::Context, w: f32) -> f32 {
    let page = ctx.content_rect().width();
    if FIT.get().1 { page - 2.0 * SHEET_PAD } else { w.min(page - 56.0).max(160.0) }
}

/// A dialog as the compact interface shows it: on a phone a sheet across the bottom of the
/// page (where the thumb is), above the safe area.
pub fn fit_modal(ctx: &egui::Context, modal: egui::Modal) -> egui::Modal {
    if !FIT.get().1 {
        return modal;
    }
    let inset = safe_area();
    let frame = egui::Frame::window(&ctx.global_style())
        .corner_radius(egui::CornerRadius { nw: 18, ne: 18, sw: 0, se: 0 })
        .stroke(Stroke::NONE)
        .inner_margin(egui::Margin { left: SHEET_PAD as i8, right: SHEET_PAD as i8, top: 18, bottom: (SHEET_PAD + inset.bottom).min(100.0) as i8 });
    let mut modal = modal;
    // above the keyboard; pulled down by its handle
    modal.area = modal.area.anchor(Align2::CENTER_BOTTOM, vec2(0.0, SWIPE.get().1 - inset.keyboard));
    SHOWN.set(SHOWN.get() + 1);
    SHEET.set(Some(modal.area.layer()));
    modal.frame(frame)
}

/// A row of a window's controls (in place of `ui.horizontal`): on a phone it wraps, the page
/// being narrower than the window was laid out for.
pub fn row<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    if phone() { ui.horizontal_wrapped(add).inner } else { ui.horizontal(add).inner }
}

/// A window's panes side by side (`None`: lay them out as ever); on a phone one at a time, by
/// tabs drawn here over them: the one to show. `id`: the window's, `names`: the panes'.
pub fn pane_tabs(ui: &mut Ui, id: &str, names: &[String]) -> Option<usize> {
    if !phone() {
        return None;
    }
    let key = Id::new(("compact_panes", id));
    let mut at = ui.data(|d| d.get_temp::<usize>(key)).unwrap_or(0).min(names.len().saturating_sub(1));
    ui.horizontal_wrapped(|ui| {
        for (i, name) in names.iter().enumerate() {
            let r = ui.selectable_value(&mut at, i, name);
            spot(format!("pane:{id}:{i}"), r.rect.center());
        }
    });
    ui.separator();
    ui.data_mut(|d| d.insert_temp(key, at));
    Some(at)
}

/// Shows pane `at` of the window `id`'s [`pane_tabs`] (an entry picked in a list shows its page).
pub fn set_pane(ctx: &egui::Context, id: &str, at: usize) {
    ctx.data_mut(|d| d.insert_temp(Id::new(("compact_panes", id)), at));
}

/// A table on a phone: each row a card, its first cell the title and the others under it,
/// each after its column's name. `rows`: the cells, as the columns of `head`.
pub fn cards(ui: &mut Ui, head: &[String], rows: &[Vec<String>]) {
    for row in rows {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            if let Some(title) = row.first() {
                ui.label(egui::RichText::new(title).strong());
            }
            for (name, cell) in head.iter().zip(row).skip(1) {
                if cell.trim().is_empty() {
                    continue;
                }
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.label(egui::RichText::new(format!("{name}:")).small().color(theme::text_dim()));
                    ui.add(egui::Label::new(egui::RichText::new(cell).small()).wrap());
                });
            }
        });
    }
}

/// A row of a dialog's buttons (in place of `ui.horizontal`): on a phone one above another,
/// the sheet wide and a finger tall.
pub fn buttons<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    if !phone() {
        return ui.horizontal(add).inner;
    }
    stacked(ui, add)
}

/// The buttons at the end of a dialog's row (in place of a right to left layout): on a phone
/// one above another, the first (the default) on top.
pub fn buttons_end<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    if !phone() {
        return ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add).inner;
    }
    stacked(ui, add)
}

fn stacked<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        ui.spacing_mut().interact_size.y = 44.0;
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), add).inner
    })
    .inner
}

/// Where a window's ✕ is on a phone (`rect`: the window, the whole page).
pub fn close_at(ctx: &egui::Context, rect: Rect) -> Pos2 {
    let font = egui::TextStyle::Heading.resolve(&ctx.global_style());
    let h = ctx.fonts_mut(|f| f.row_height(&font));
    let i = safe_area();
    pos2(rect.max.x - 12.0 - i.right - h / 2.0, rect.min.y + 12.0 + i.top + h / 2.0)
}

/// The page's safe area: how far in from each side the notch, the rounded corners and the home
/// bar reach (points; zero outside a browser).
#[derive(Clone, Copy, Default)]
pub struct Insets {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
    /// The on-screen keyboard's height (it covers the page's bottom).
    pub keyboard: f32,
}

impl Insets {
    pub fn shrink(self, r: Rect) -> Rect {
        Rect::from_min_max(r.min + vec2(self.left, self.top), r.max - vec2(self.right, self.bottom.max(self.keyboard)))
    }
}

/// The QA spots found before the frame's were cleared.
pub fn frame_spots() {
    if let Some(p) = HANDLE.get() {
        spot("sheet:handle", p);
    }
}

/// The QA scripts' keyboard (`action keyboard N`).
pub fn set_test_keyboard(h: f32) {
    TEST_KEYBOARD.set(h);
}

/// The QA scripts' safe area (`action safe_area TOP BOTTOM`): a phone's status and home bars.
pub fn set_test_safe_area(top: f32, bottom: f32) {
    TEST_SAFE.set((top, bottom));
}

pub fn safe_area() -> Insets {
    INSETS.get()
}

/// Read the page's safe area (`env(safe-area-inset-*)`, as `--safe-*` on the page's root).
fn read_safe_area(ctx: &egui::Context) -> Insets {
    #[cfg(target_arch = "wasm32")]
    {
        let get = || -> Option<Insets> {
            let w = web_sys::window()?;
            let root = w.document()?.document_element()?;
            let style = w.get_computed_style(&root).ok()??;
            let zoom = ctx.zoom_factor();
            let px = |name: &str| style.get_property_value(name).ok().and_then(|v| v.trim().trim_end_matches("px").parse::<f32>().ok()).unwrap_or(0.0) / zoom;
            // the keyboard: what of the page the visual viewport leaves out at the bottom
            let keyboard = w.visual_viewport().map_or(0.0, |v| {
                let page = w.inner_height().ok().and_then(|h| h.as_f64()).unwrap_or(0.0);
                ((page - v.height() - v.offset_top()).max(0.0) as f32) / zoom
            });
            Some(Insets { top: px("--safe-top"), bottom: px("--safe-bottom"), left: px("--safe-left"), right: px("--safe-right"), keyboard })
        };
        get().unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ctx;
        let (top, bottom) = TEST_SAFE.get();
        Insets { top, bottom, keyboard: TEST_KEYBOARD.get(), ..Default::default() }
    }
}

/// Finger-sized buttons and rows everywhere (`on`), or the theme's own.
fn touch_style(ctx: &egui::Context, on: bool) {
    const ROW: f32 = 34.0;
    const HEADING: f32 = 1.3;
    const LONG: f32 = 0.6;
    let is = ctx.global_style().spacing.interact_size.y == ROW;
    if on == is {
        return;
    }
    ctx.options_mut(|o| o.input_options.max_click_duration = if on { LONG as f64 } else { egui::InputOptions::default().max_click_duration });
    if on {
        ctx.all_styles_mut(|s| {
            s.spacing.interact_size.y = ROW;
            s.spacing.button_padding = vec2(12.0, 7.0);
            s.spacing.item_spacing = vec2(8.0, 8.0);
            s.spacing.icon_width = 20.0;
            s.spacing.icon_spacing = 8.0;
            // a finger is less precise than a mouse: a press this near a widget is its (the
            // touch targets grow to about 44 points without the rows growing)
            s.interaction.interact_radius = 12.0;
            // a press held this long shows the tooltip and is no click
            s.interaction.tooltip_delay = LONG;
            // a taller title bar: a bigger ✕ to hit
            if let Some(h) = s.text_styles.get_mut(&egui::TextStyle::Heading) {
                h.size *= HEADING;
            }
        });
    } else {
        theme::apply(ctx);
        let d = egui::Style::default();
        ctx.all_styles_mut(|s| {
            s.spacing.icon_width = d.spacing.icon_width;
            s.spacing.icon_spacing = d.spacing.icon_spacing;
            s.interaction.interact_radius = d.interaction.interact_radius;
            s.interaction.tooltip_delay = d.interaction.tooltip_delay;
            if let Some(h) = s.text_styles.get_mut(&egui::TextStyle::Heading) {
                h.size /= HEADING;
            }
        });
    }
}

/// An Escape this frame (closes a dialog).
fn escape(ctx: &egui::Context) {
    ctx.input_mut(|i| i.events.push(egui::Event::Key { key: Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() }));
}

/// The page's history while the compact interface has something open (see `compact_back`).
mod back {
    use std::cell::Cell;

    thread_local! {
        /// An entry is pushed for what is open.
        static PUSHED: Cell<bool> = const { Cell::new(false) };
        /// The browser went back off it.
        static WENT: Cell<bool> = const { Cell::new(false) };
        /// The app took the entry off itself (its popstate is not a back).
        static OWN: Cell<bool> = const { Cell::new(false) };
    }

    /// The browser went back since the last call (scripts: `action back`).
    pub fn take() -> bool {
        WENT.replace(false)
    }

    pub fn press() {
        WENT.set(true);
        PUSHED.set(false);
    }

    /// Keep an entry in the history while something is `open`.
    pub fn hold(open: bool) {
        if open == PUSHED.get() {
            return;
        }
        PUSHED.set(open);
        #[cfg(target_arch = "wasm32")]
        if let Some(h) = web_sys::window().and_then(|w| w.history().ok()) {
            if open {
                let _ = h.push_state(&wasm_bindgen::JsValue::NULL, "");
            } else {
                OWN.set(true);
                let _ = h.back();
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn install(ctx: egui::Context) {
        use wasm_bindgen::JsCast;
        let Some(window) = web_sys::window() else { return };
        let on_pop = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
            if OWN.replace(false) {
                return;
            }
            press();
            ctx.request_repaint();
        });
        let _ = window.add_event_listener_with_callback("popstate", on_pop.as_ref().unchecked_ref());
        on_pop.forget();
    }
}

#[cfg(target_arch = "wasm32")]
pub fn install_back(ctx: egui::Context) {
    back::install(ctx);
}

/// A back as the browser's (the QA scripts).
pub fn press_back() {
    back::press();
}

/// The page's main pointer is a finger (`pointer: coarse`).
pub(crate) fn touch_screen() -> bool {
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
        let narrow = ctx.content_rect().width() < NARROW;
        FIT.set((self.compact.on, self.compact.on && narrow));
        INSETS.set(if self.compact.on { read_safe_area(ctx) } else { Insets::default() });
        let dialogs = SHOWN.replace(0);
        let sheet = SHEET.replace(None).filter(|_| dialogs > 0);
        self.sheet_swipe(ctx, sheet);
        self.compact_back(ctx, dialogs > 0);
        touch_style(ctx, self.compact.on && (narrow || touch_screen()));
        if !self.compact.on {
            self.compact.panels = false;
            self.compact.numpad = false;
        }
    }

    /// The browser's back (Android's back button and gesture) closes what the compact interface
    /// has open on top: the menus a level at a time, the number pad, the panels, a dialog. While
    /// something is open the page's history has an entry for it, so back does not leave the page.
    fn compact_back(&mut self, ctx: &egui::Context, dialog: bool) {
        // a window on a phone: the topmost of those taking the whole page
        let screen = ctx.content_rect();
        let window = (FIT.get().1 && !dialog)
            .then(|| {
                ctx.memory(|m| {
                    m.layer_ids()
                        .filter(|l| l.order == egui::Order::Middle && m.areas().is_visible(l))
                        .filter_map(|l| m.area_rect(l.id))
                        .filter(|r| r.width() >= screen.width() - 1.0 && r.top() <= screen.top() + 1.0)
                        .last()
                })
            })
            .flatten();
        let open = self.compact.on && (self.compact.radial.is_some() || self.compact.numpad || self.compact.panels || dialog || window.is_some());
        if back::take() {
            if let Some(r) = self.compact.radial.as_mut() {
                if r.sub.pop().is_none() {
                    self.radial_back(false);
                }
            } else if self.compact.numpad {
                self.compact.numpad = false;
            } else if self.compact.panels {
                self.compact.panels = false;
            } else if dialog {
                escape(ctx);
            } else if let Some(r) = window {
                // a tap on its ✕
                self.compact.tap = Some((close_at(ctx, r), false));
            }
            ctx.request_repaint();
            return;
        }
        back::hold(open);
    }

    /// A dialog's sheet is pulled down by its handle (a bar at its top): far enough, it closes.
    fn sheet_swipe(&mut self, ctx: &egui::Context, sheet: Option<egui::LayerId>) {
        const GRAB: f32 = 28.0;
        HANDLE.set(None);
        let Some(rect) = sheet.and_then(|l| ctx.memory(|m| m.area_rect(l.id))) else {
            SWIPE.set((None, 0.0));
            return;
        };
        let (mut from, before) = SWIPE.get();
        let rect = rect.translate(vec2(0.0, -before));
        let grab = Rect::from_min_size(rect.min, vec2(rect.width(), GRAB));
        HANDLE.set(Some(grab.center()));
        let (pressed, down, pos, origin) = ctx.input(|i| (i.pointer.primary_pressed(), i.pointer.primary_down(), i.pointer.latest_pos(), i.pointer.press_origin()));
        if pressed && origin.is_some_and(|o| grab.contains(o)) {
            from = origin;
        }
        let mut pull = 0.0;
        if let Some(o) = from {
            let d = pos.map_or(before, |p| (p.y - o.y).max(0.0));
            if down {
                pull = d;
            } else {
                from = None;
                if d > (rect.height() * 0.25).min(120.0) {
                    escape(ctx);
                }
            }
            ctx.request_repaint();
        }
        SWIPE.set((from, pull));
        // the handle
        let bar = Rect::from_center_size(pos2(rect.center().x, rect.top() + pull + 9.0), vec2(36.0, 5.0));
        ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, Id::new("sheet_handle"))).rect_filled(bar, 2.5, theme::text_dim().gamma_multiply(0.5));
    }

    /// The app's own tap (see `Compact::tap`), into the frame's input.
    pub(crate) fn compact_raw_input(&mut self, raw: &mut egui::RawInput) {
        let Some((at, pressed)) = self.compact.tap else { return };
        let button = |pressed| egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
        if pressed {
            raw.events.extend([button(false), egui::Event::PointerGone]);
            self.compact.tap = None;
        } else {
            raw.events.extend([egui::Event::PointerMoved(at), button(true)]);
            self.compact.tap = Some((at, true));
        }
    }

    /// For the QA scripts: (on, the ring's open level, panels, number pad).
    pub(crate) fn compact_dump(&self) -> String {
        let level = self.compact.radial.as_ref().and_then(|r| r.levels.last().copied());
        let sub = self.compact.radial.as_ref().map(|r| r.sub.clone()).unwrap_or_default();
        format!("({} {level:?} {} {}) compact_sub={sub:?} compact_card={:?} compact_dock={:?} compact_windows=(prefs {} about {}) tap_marks={} hint_open={}", self.compact.on, self.compact.panels, self.compact.numpad, self.compact.card, self.dock_tools(), self.prefs_win.open, self.show_about, self.touch.marks.len(), self.compact.hint_open.is_some())
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
        // clear of the notch, the rounded corners and the home bar
        let rect = safe_area().shrink(rect);
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
        let all = self.dock_tools();
        let tools = &all[..fit.min(all.len())];
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
            if self.dock_button(ui, r, c.icon(), false, enabled, &c.menu_label(), &format!("dock:{}", c.key())).clicked() {
                self.actions.push(c.action());
            }
        }
        sep(ui, &mut x);
        let current = self.tool_id();
        // a long press (a right click) on a tool or on all tools: the dock's own menu
        let edit = |r: &Response| r.long_touched() || r.secondary_clicked();
        for (i, &t) in tools.iter().enumerate() {
            let r = slot(&mut x);
            let resp = self.dock_button(ui, r, t.icon(), current == t, true, &t.name(), &format!("dock:{t:?}"));
            if edit(&resp) {
                self.open_dock_list(r.center(), Some(i));
            } else if resp.clicked() {
                self.actions.push(Action::SetTool(t));
            }
        }
        sep(ui, &mut x);
        let r = slot(&mut x);
        let other = !tools.contains(&current);
        let resp = self.dock_button(ui, r, Icon::More, other, true, &tr!("compact-all-tools"), "dock:all");
        if edit(&resp) {
            self.open_dock_list(r.center(), None);
        } else if resp.clicked() {
            let center = rect.center();
            self.open_radial(Some(center), false);
            if let Some(radial) = self.compact.radial.as_mut() {
                radial.levels.push(Level::Ring(Ring::Tools));
            }
        }
        dock
    }

    pub(crate) fn default_dock(&self) -> Vec<ToolId> {
        DOCK_TOOLS.to_vec()
    }

    /// The dock's tools: the user's or the default.
    pub(crate) fn dock_tools(&self) -> Vec<ToolId> {
        if self.prefs.dock.is_empty() { DOCK_TOOLS.to_vec() } else { self.prefs.dock.clone() }
    }

    fn open_dock_list(&mut self, at: Pos2, slot: Option<usize>) {
        self.compact.radial = Some(Radial { center: at, levels: vec![Level::List(Menu::Dock(slot))], held: false, resting: None, sub: Vec::new() });
    }

    /// The dock's menu: for its tool at `slot` move it, take it off or put another tool in its
    /// place; for all tools (`None`) put one more on the dock.
    fn dock_menu(&mut self, ui: &mut Ui, slot: Option<usize>) {
        use super::menu::icon_row;
        let mut dock = self.dock_tools();
        let mut changed = false;
        if let Some(i) = slot.filter(|&i| i < dock.len()) {
            let n = dock.len();
            for (label, key, ok, to) in [(tr!("compact-dock-left"), "left", i > 0, i.wrapping_sub(1)), (tr!("compact-dock-right"), "right", i + 1 < n, i + 1)] {
                let r = icon_row(ui, None, &label, None, ok, false);
                spot(format!("dockmenu:{key}"), r.rect.center());
                if r.clicked() {
                    dock.swap(i, to);
                    changed = true;
                }
            }
            let r = icon_row(ui, Some(Icon::Eraser), &tr!("compact-dock-remove"), None, n > 1, false);
            spot("dockmenu:remove", r.rect.center());
            if r.clicked() {
                dock.remove(i);
                changed = true;
            }
        }
        if !self.prefs.dock.is_empty() {
            let r = icon_row(ui, None, &tr!("compact-dock-reset"), None, true, false);
            spot("dockmenu:reset", r.rect.center());
            if r.clicked() {
                dock = DOCK_TOOLS.to_vec();
                changed = true;
            }
        }
        if slot.is_some() || !self.prefs.dock.is_empty() {
            ui.separator();
        }
        let head = if slot.is_some() { tr!("compact-dock-replace") } else { tr!("compact-dock-add") };
        ui.label(egui::RichText::new(head).color(theme::text_dim()));
        for t in ToolId::ALL {
            let here = slot.and_then(|i| dock.get(i)) == Some(&t);
            if dock.contains(&t) && !here {
                continue;
            }
            let r = icon_row(ui, Some(t.icon()), &t.name(), None, true, here);
            spot(format!("dockmenu:{t:?}"), r.rect.center());
            if r.clicked() && !here {
                match slot.filter(|&i| i < dock.len()) {
                    Some(i) => dock[i] = t,
                    None => dock.push(t),
                }
                changed = true;
            }
        }
        if changed {
            self.prefs.dock = if dock == DOCK_TOOLS { Vec::new() } else { dock };
            ui.close();
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn dock_button(&mut self, ui: &mut Ui, r: Rect, icon: Icon, on: bool, enabled: bool, tip: &str, name: &str) -> Response {
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
        resp.on_hover_text(tip)
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
        if text.is_empty() {
            self.compact.hint_open = None;
            return;
        }
        // two lines beside the chip, growing up; what does not fit opens in full by a tap,
        // across the page above the chip
        let open = self.compact.hint_open.as_deref() == Some(text.as_str());
        if !open {
            self.compact.hint_open = None;
        }
        let room = if open { right - left - 24.0 } else { chip_left - left - 24.0 };
        if room < 60.0 {
            return;
        }
        let mut job = egui::text::LayoutJob::simple(text.clone(), font, fg, room);
        job.wrap.max_rows = if open { 12 } else { 2 };
        job.wrap.overflow_character = Some('…');
        let g = ui.painter().layout_job(job);
        let h = (g.size().y + 16.0).max(H);
        let bottom = if open && chip_left < right { y - 8.0 } else { y + H };
        let hint = Rect::from_min_size(pos2(left, bottom - h), vec2(g.size().x + 24.0, h));
        if g.elided || open {
            if ui.interact(hint, Id::new("compact_hint"), Sense::click()).clicked() {
                self.compact.hint_open = (!open).then_some(text);
            }
            self.nav_button_rects.push(hint);
        }
        spot("compact:hint", hint.center());
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
                        Menu::Tools => {
                            self.menu_tools(ui);
                            ui.separator();
                            self.menu_nav_tools(ui);
                        }
                        Menu::Snapping => self.snapping_menu(ui, true),
                        Menu::Window => self.menu_window(ui),
                        Menu::Help => self.menu_help(ui),
                        Menu::Context => self.viewport_context_menu(ui),
                        Menu::Dock(slot) => self.dock_menu(ui, slot),
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
            screen
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
            let corner = if narrow { egui::CornerRadius::ZERO } else { egui::CornerRadius::same(18) };
            if !narrow {
                shadow(ui, sheet, 18.0);
            }
            ui.painter().rect_filled(sheet, corner, theme::tray_bg());
            let inner = if narrow { safe_area().shrink(sheet) } else { sheet }.shrink2(vec2(10.0, 8.0));
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
                self.card_chips(ui);
                ui.add_space(6.0);
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

    /// A chip per side bar card over the sheet: one shows that card alone, open.
    fn card_chips(&mut self, ui: &mut Ui) {
        let hidden = self.prefs.tray_hidden.clone();
        let cards = super::tray::TRAY_CARDS.into_iter().filter(|c| !hidden.iter().any(|h| h == c));
        egui::ScrollArea::horizontal().id_salt("compact_chips").scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                for card in std::iter::once(None).chain(cards.map(Some)) {
                    let label = card.map_or_else(|| tr!("compact-cards-all"), super::tray::card_label);
                    let on = self.compact.card == card;
                    let b = ui.add(egui::Button::new(egui::RichText::new(label).size(14.0)).selected(on).min_size(vec2(0.0, 34.0)).corner_radius(17.0));
                    spot(format!("chip:{}", card.unwrap_or("All")), b.rect.center()); // i18n-ignore
                    if b.clicked() {
                        self.compact.card = card;
                    }
                }
            });
        });
    }

    /// The Measurements box's number pad at the bottom: digits, the separators of the
    /// Measurements syntax, Enter.
    fn numpad_sheet(&mut self, ctx: &egui::Context) {
        let screen = ctx.content_rect();
        const KEY: f32 = 50.0;
        const GAP: f32 = 8.0;
        let w = (screen.width() - 2.0 * MARGIN).min(420.0);
        let h = 4.0 * KEY + 3.0 * GAP + 96.0;
        let sheet = Rect::from_min_size(pos2(screen.center().x - w / 2.0, screen.bottom() - MARGIN - safe_area().bottom - h), vec2(w, h));
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
                        "⌫" => {
                            // a key with its point to the left and a small cross in it
                            let c = r.center() + vec2(1.0, 0.0);
                            let (w, h) = (13.0, 9.0);
                            let shape = vec![c + vec2(-w - 6.0, 0.0), c + vec2(-w + 2.0, -h), c + vec2(w, -h), c + vec2(w, h), c + vec2(-w + 2.0, h)];
                            p.add(egui::Shape::closed_line(shape, Stroke::new(2.0, fg)));
                            paint_cross(p, c + vec2(1.5, 0.0), 4.0, fg);
                        }
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
