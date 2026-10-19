//! The common frame of the Components and Materials cards: the top row (search, Refresh, the
//! plus menu, the online library, the … menu), section headings that fold (In Model, the
//! library folders, Examples) and tiles in three sizes (`TileSize`, shared by both cards).

use super::icons::{Icon, paint_icon};
use egui::{Color32, Rect, Response, RichText, Sense, Stroke, Ui, pos2, vec2};
use serde::{Deserialize, Serialize};

/// Tile size of both library cards (… menu).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileSize {
    /// Pictures only, the name in the hover text.
    Small,
    /// Pictures with the name below.
    #[default]
    Large,
    /// One row per item: a small picture and the name.
    List,
}

pub const TILE_SIZES: [(TileSize, &str); 3] = [(TileSize::Small, "Small Tiles"), (TileSize::Large, "Large Tiles"), (TileSize::List, "List")];

/// Side of a large tile's picture (a small one: `SMALL`, a list row's: `LIST`).
pub const LARGE: f32 = 78.0;
const SMALL: f32 = 36.0;
const LIST: f32 = 24.0;
/// Room under a large tile's picture for the name.
const NAME_H: f32 = 16.0;
const GAP: f32 = 6.0;

/// What the top row's buttons asked for.
#[derive(Default)]
pub struct TopRow {
    pub refresh: bool,
    pub online: bool,
}

/// An icon button opening a menu (a click inside closes it, as usual).
fn icon_menu(ui: &mut Ui, icon: Icon, tip: &str, content: impl FnOnce(&mut Ui)) -> Response {
    let id = ui.id().with(("tray_menu", tip));
    let b = egui::Button::new(egui::Atom::custom(id, vec2(15.0, 15.0))).frame(false).min_size(vec2(22.0, 22.0));
    let (r, _) = egui::containers::menu::MenuButton::from_button(b).ui(ui, content);
    if r.hovered() {
        ui.painter().rect_filled(r.rect, 7.0, Color32::from_rgb(0xDF, 0xE4, 0xEB));
    }
    paint_icon(ui.painter(), Rect::from_center_size(r.rect.center(), vec2(15.0, 15.0)), icon, false);
    r.on_hover_text(tip)
}

/// Search, Refresh, + (`plus`), the online library (`online_tip`), … (`more`, ends with the
/// tile size). `tag` prefixes the harness spots ("" Components, "m:" Materials).
#[allow(clippy::too_many_arguments)]
pub fn top_row(ui: &mut Ui, tag: &str, filter: &mut String, tiles: &mut TileSize, plus_tip: &str, online_tip: &str, plus: impl FnOnce(&mut Ui), more: impl FnOnce(&mut Ui)) -> TopRow {
    let mut out = TopRow::default();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        ui.add(egui::TextEdit::singleline(filter).hint_text("Search").desired_width(ui.available_width() - 4.0 * 24.0));
        let r = super::small_icon_button(ui, Icon::Refresh, "Refresh: read the library folder again");
        spot(format!("{tag}refresh"), r.rect.center());
        if r.clicked() {
            out.refresh = true;
        }
        let r = icon_menu(ui, Icon::Plus, plus_tip, plus);
        spot(format!("{tag}plus"), r.rect.center());
        let r = super::small_icon_button(ui, Icon::Cloud, online_tip);
        spot(format!("{tag}online"), r.rect.center());
        if r.clicked() {
            out.online = true;
        }
        let r = icon_menu(ui, Icon::More, "Library folder, what to show, tile size", |ui| {
            more(ui);
            ui.separator();
            for (s, name) in TILE_SIZES {
                if ui.radio(*tiles == s, name).clicked() {
                    *tiles = s;
                    ui.close();
                }
            }
        });
        spot(format!("{tag}more"), r.rect.center());
    });
    out
}

/// A section heading: fold arrow, `title` (a link when `link` — the caller shows the folder),
/// the number of items; `extra` adds buttons after it. Folded sections are listed in
/// `collapsed` by `key` — or, for a section folded by default (`fold_default`), the unfolded
/// ones. Returns (folded, title link clicked).
#[allow(clippy::too_many_arguments)]
pub fn section(ui: &mut Ui, collapsed: &mut Vec<String>, key: &str, title: &str, tip: &str, link: bool, fold_default: bool, count: Option<usize>, extra: impl FnOnce(&mut Ui)) -> (bool, bool) {
    let listed = collapsed.iter().any(|f| f == key);
    let folded = listed != fold_default;
    let mut clicked = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let arrow = ui
            .add_sized([14.0, 18.0], egui::Button::new(if folded { "⏵" } else { "⏷" }).frame(false))
            .on_hover_text(if folded { "Show the section" } else { "Fold the section" });
        spot(format!("▸{key}"), arrow.rect.center());
        if arrow.clicked() {
            if listed {
                collapsed.retain(|f| f != key);
            } else {
                collapsed.push(key.to_string());
            }
        }
        let r = if link { ui.link(RichText::new(title).strong()) } else { ui.label(RichText::new(title).strong()) };
        spot(format!("§{key}"), r.rect.center());
        let r = if tip.is_empty() { r } else { r.on_hover_text(tip) };
        clicked = link && r.clicked();
        if let Some(n) = count {
            ui.label(RichText::new(n.to_string()).small().weak());
        }
        extra(ui);
    });
    (folded, clicked)
}

/// Lay out `n` tiles of `size` (rows of as many as fit; a list: one per row); `tile(ui, k)`
/// draws the k-th.
pub fn tiles(ui: &mut Ui, size: TileSize, n: usize, mut tile: impl FnMut(&mut Ui, usize)) {
    if size == TileSize::List {
        for k in 0..n {
            tile(ui, k);
        }
        return;
    }
    let w = if size == TileSize::Small { SMALL } else { LARGE };
    let cols = ((ui.available_width() + GAP) / (w + GAP)).floor().max(1.0) as usize;
    let mut k = 0;
    while k < n {
        let end = (k + cols).min(n);
        ui.horizontal(|ui| {
            for j in k..end {
                tile(ui, j);
            }
        });
        k = end;
    }
}

/// One tile: its picture painted by `paint` in the given rect, the name (below a large one,
/// beside it in a list), a frame (accent when `selected`). Click and context menu as usual.
pub fn tile(ui: &mut Ui, size: TileSize, name: &str, selected: bool, paint: impl FnOnce(&egui::Painter, Rect)) -> Response {
    let (rect, pic) = match size {
        TileSize::Small => {
            let (r, _) = ui.allocate_exact_size(vec2(SMALL, SMALL), Sense::hover());
            (r, r)
        }
        TileSize::Large => {
            let (r, _) = ui.allocate_exact_size(vec2(LARGE, LARGE + NAME_H), Sense::hover());
            (r, Rect::from_min_size(r.min, vec2(LARGE, LARGE)))
        }
        TileSize::List => {
            let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), LIST + 2.0), Sense::hover());
            (r, Rect::from_min_size(r.min + vec2(2.0, 1.0), vec2(LIST, LIST)))
        }
    };
    let resp = ui.interact(rect, ui.id().with(("tile", name, rect.min.x as i32, rect.min.y as i32)), Sense::click());
    spot(name, pic.center());
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let hover = resp.hovered();
        let round = if size == TileSize::Large { 6.0 } else { 4.0 };
        if size == TileSize::List && (hover || selected) {
            p.rect_filled(rect, 4.0, if selected { super::theme::ACCENT_SOFT } else { Color32::from_rgb(0xE6, 0xEE, 0xFA) });
        }
        let fill = if hover && size != TileSize::List { Color32::from_rgb(0xE6, 0xEE, 0xFA) } else { Color32::from_gray(248) };
        p.rect_filled(pic, round, fill);
        paint(p, pic);
        let border = if selected { Stroke::new(2.0, super::theme::ACCENT) } else { Stroke::new(1.0, Color32::from_gray(215)) };
        p.rect_stroke(pic, round, border, egui::StrokeKind::Inside);
        let font = egui::FontId::proportional(11.0);
        let color = ui.visuals().text_color();
        match size {
            TileSize::Large => {
                let label = if name.chars().count() > 12 { format!("{}…", name.chars().take(11).collect::<String>()) } else { name.to_string() };
                p.text(pos2(rect.center().x, rect.max.y - 7.0), egui::Align2::CENTER_CENTER, label, font, color);
            }
            TileSize::List => {
                p.text(pos2(pic.max.x + 6.0, rect.center().y), egui::Align2::LEFT_CENTER, name, egui::FontId::proportional(12.0), color);
            }
            TileSize::Small => {}
        }
    }
    resp
}

/// A picture `id` fitted into `r` (the tiles' pictures).
pub fn paint_picture(ui_ctx: &egui::Context, p: &egui::Painter, r: Rect, id: egui::TextureId) {
    let [w, h] = ui_ctx.tex_manager().read().meta(id).map_or([1, 1], |m| m.size);
    let k = (r.width() - 2.0) / w.max(h).max(1) as f32;
    let fit = Rect::from_center_size(r.center(), vec2(w as f32 * k, h as f32 * k));
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    p.image(id, fit, uv, Color32::WHITE);
}

/// Name matches the search `filter` (lower case; empty = all).
pub fn found(filter: &str, name: &str) -> bool {
    filter.is_empty() || name.to_lowercase().contains(filter)
}

thread_local! {
    /// Where the tiles, headings and top-row buttons were drawn this frame (screen points),
    /// for the harness: `lib_spots=` in the dump.
    static SPOTS: std::cell::RefCell<Vec<(String, egui::Pos2)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Forget last frame's spots (start of the toolbar).
pub fn clear_spots() {
    SPOTS.with(|s| s.borrow_mut().clear());
}

pub fn spot(name: impl Into<String>, p: egui::Pos2) {
    SPOTS.with(|s| s.borrow_mut().push((name.into(), p)));
}

/// The spots relative to `origin` (the harness raw origin), rounded.
pub fn spots(origin: egui::Pos2) -> Vec<(String, i32, i32)> {
    SPOTS.with(|s| s.borrow().iter().map(|(n, p)| (n.clone(), (p.x - origin.x).round() as i32, (p.y - origin.y).round() as i32)).collect())
}
