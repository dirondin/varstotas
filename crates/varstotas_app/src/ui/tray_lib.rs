//! The common frame of the Components and Materials cards: the top row (search, Refresh, the
//! plus menu, the online library, the … menu), section headings that fold (In Model,
//! Favourites, Recent, the library folders, Examples), tiles in three sizes (`TileSize`, shared
//! by both cards), favourites / recents (`Prefs::lib_favourites` …), the folder dialog (New /
//! Rename Folder) and dragging a tile into the viewport (`TrayDrag`).

use super::icons::{Icon, paint_icon};
use egui::{Color32, Rect, Response, RichText, Sense, Stroke, Ui, pos2, vec2};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
/// ones. Returns (folded, title link clicked, the title — for a context menu).
#[allow(clippy::too_many_arguments)]
pub fn section(ui: &mut Ui, collapsed: &mut Vec<String>, key: &str, title: &str, tip: &str, link: bool, fold_default: bool, count: Option<usize>, extra: impl FnOnce(&mut Ui)) -> (bool, bool, Response) {
    let listed = collapsed.iter().any(|f| f == key);
    let folded = listed != fold_default;
    let mut clicked = false;
    let mut title_resp = None;
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
        title_resp = Some(r);
        if let Some(n) = count {
            ui.label(RichText::new(n.to_string()).small().weak());
        }
        extra(ui);
    });
    let title = title_resp.expect("drawn above");
    (folded, clicked, title)
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
    // click to use, drag into the viewport (`TrayDrag`)
    let resp = ui.interact(rect, ui.id().with(("tile", name, rect.min.x as i32, rect.min.y as i32)), Sense::click_and_drag());
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

/// The picture part of a tile drawn by [`tile`] (its response's rect).
fn picture_rect(size: TileSize, r: Rect) -> Rect {
    match size {
        TileSize::Small => r,
        TileSize::Large => Rect::from_min_size(r.min, vec2(LARGE, LARGE)),
        TileSize::List => Rect::from_min_size(r.min + vec2(2.0, 1.0), vec2(LIST, LIST)),
    }
}

/// A five-pointed star of outer radius `r` at `c`.
pub fn star(p: &egui::Painter, c: egui::Pos2, r: f32, fill: Color32) {
    let pts: Vec<egui::Pos2> = (0..10)
        .map(|k| {
            let a = -std::f32::consts::FRAC_PI_2 + k as f32 * std::f32::consts::PI / 5.0;
            let rr = if k % 2 == 0 { r } else { r * 0.45 };
            c + vec2(a.cos(), a.sin()) * rr
        })
        .collect();
    // not convex: a fan from the centre
    for k in 0..10 {
        p.add(egui::Shape::convex_polygon(vec![c, pts[k], pts[(k + 1) % 10]], fill, Stroke::NONE));
    }
}

/// The favourite mark on a tile (a gold star in the picture's top left corner).
pub fn mark_favourite(ui: &Ui, size: TileSize, tile: &Response) {
    let pic = picture_rect(size, tile.rect);
    let r = if size == TileSize::Large { 6.0 } else { 4.5 };
    let c = pic.min + vec2(r + 2.0, r + 2.0);
    let p = ui.painter();
    p.circle_filled(c, r + 1.5, Color32::from_white_alpha(200));
    star(p, c, r, Color32::from_rgb(0xF2, 0xB0, 0x1E));
}

/// Most recent first, at most this many per card.
pub const RECENT_MAX: usize = 8;

/// `key` was just used: to the front of `list`.
pub fn note_recent(list: &mut Vec<String>, key: &str) {
    list.retain(|k| k != key);
    list.insert(0, key.to_string());
    list.truncate(RECENT_MAX);
}

/// Toggle `key` in the favourites `list`.
pub fn toggle(list: &mut Vec<String>, key: &str) {
    if list.iter().any(|k| k == key) {
        list.retain(|k| k != key);
    } else {
        list.push(key.to_string());
    }
}

/// The keys' names for the harness: file stems, example names.
pub fn key_names(list: &[String]) -> Vec<String> {
    list.iter().map(|k| Path::new(k.get(2..).unwrap_or_default()).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()).collect()
}

/// Key of a library file in the favourites / recents (`prefix` + path).
pub fn file_key(prefix: &str, path: &Path) -> String {
    format!("{prefix}{}", path.display())
}

/// After moving or renaming a file or folder `from` → `to`: keys of (or under) it follow.
pub fn rekey(lists: [&mut Vec<String>; 2], prefix: &str, from: &Path, to: &Path) {
    let (f, t) = (file_key(prefix, from), file_key(prefix, to));
    for list in lists {
        for k in list.iter_mut() {
            if *k == f {
                *k = t.clone();
            } else if let Some(rest) = k.strip_prefix(&f).filter(|r| r.starts_with(std::path::MAIN_SEPARATOR)) {
                *k = format!("{t}{rest}");
            }
        }
    }
}

/// What a tile's library entries asked for ([`lib_menu`]).
pub enum LibMenuAct {
    /// Add to or remove from the favourites.
    Favourite,
    /// Move the file to this subfolder ("" = the library folder itself).
    Move(String),
}

/// The library entries of a tile's context menu: Add to / Remove from Favourites (when
/// `favourite` is known) and Move to Folder ▸ (a file in `current` of the `folders`).
pub fn lib_menu(ui: &mut Ui, favourite: Option<bool>, folders: Option<(&[String], &str)>, act: &mut Option<LibMenuAct>) {
    if let Some(fav) = favourite {
        let label = if fav { "Remove from Favourites" } else { "Add to Favourites" };
        let r = ui.button(label);
        spot("menu:favourite", r.rect.center());
        if r.clicked() {
            *act = Some(LibMenuAct::Favourite);
            ui.close();
        }
    }
    if let Some((folders, current)) = folders {
        let r = ui.menu_button("Move to Folder", |ui| {
            for f in std::iter::once("").chain(folders.iter().map(String::as_str)) {
                let name = if f.is_empty() { "My Library" } else { f };
                let r = ui.add_enabled(f != current, egui::Button::new(name));
                spot(format!("move:{name}"), r.rect.center());
                if r.clicked() {
                    *act = Some(LibMenuAct::Move(f.to_string()));
                    ui.close();
                }
            }
        });
        spot("menu:move", r.response.rect.center());
    }
}

/// Move `file` into `root/folder` (same name). Returns the new path.
pub fn move_file(file: &Path, root: &Path, folder: &str) -> Result<PathBuf, String> {
    let dir = if folder.is_empty() { root.to_path_buf() } else { root.join(folder) };
    let to = dir.join(file.file_name().ok_or("no file name")?);
    if to == file {
        return Ok(to);
    }
    if to.exists() {
        return Err(format!("{} already exists", to.display()));
    }
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::rename(file, &to).map_err(|e| e.to_string())?;
    Ok(to)
}

/// Subfolders of `root` (two levels, as the libraries are scanned; "a/b" for nested ones),
/// sorted, hidden ones left out.
pub fn subfolders(root: &Path) -> Vec<String> {
    fn level(dir: &Path, prefix: &str, out: &mut Vec<String>) -> Vec<(PathBuf, String)> {
        let mut next = Vec::new();
        let Ok(rd) = std::fs::read_dir(dir) else { return next };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if p.is_dir() && !name.starts_with('.') {
                let rel = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };
                out.push(rel.clone());
                next.push((p, rel));
            }
        }
        next
    }
    let mut out = Vec::new();
    for (p, rel) in level(root, "", &mut out) {
        level(&p, &rel, &mut out);
    }
    out.sort_by_key(|a| a.to_lowercase());
    out
}

/// A folder name typed by the user: one path component.
pub fn folder_name_ok(name: &str) -> bool {
    let n = name.trim();
    !n.is_empty() && !n.starts_with('.') && !n.contains(['/', '\\', ':'])
}

/// What a folder heading's context menu asked for ([`folder_menu`]).
pub enum FolderAct {
    New,
    Rename(String),
    Delete(PathBuf),
}

/// The context menu of a library folder's heading (`title`): New Folder… on My Library
/// (`folder` ""), Rename Folder… and Delete Folder (when empty) on a subfolder.
pub fn folder_menu(title: &Response, folder: &str, dir: Option<&Path>, act: &mut Option<FolderAct>) {
    let Some(dir) = dir else { return };
    title.context_menu(|ui| {
        let r = ui.button("New Folder…");
        spot("menu:new_folder", r.rect.center());
        if r.clicked() {
            *act = Some(FolderAct::New);
            ui.close();
        }
        if folder.is_empty() {
            return;
        }
        let r = ui.button("Rename Folder…");
        spot("menu:rename_folder", r.rect.center());
        if r.clicked() {
            *act = Some(FolderAct::Rename(folder.to_string()));
            ui.close();
        }
        let empty = std::fs::read_dir(dir).is_ok_and(|mut d| d.next().is_none());
        let r = ui.add_enabled(empty, egui::Button::new("Delete Folder")).on_disabled_hover_text("Only an empty folder (move or delete what is in it first)");
        spot("menu:delete_folder", r.rect.center());
        if r.clicked() {
            *act = Some(FolderAct::Delete(dir.to_path_buf()));
            ui.close();
        }
    });
}

/// The New Folder / Rename Folder dialog of a library card.
#[derive(Clone, Debug, PartialEq)]
pub struct FolderAsk {
    /// The materials library (else the components library).
    pub materials: bool,
    /// Rename this subfolder (None = a new folder in the library folder).
    pub rename: Option<String>,
    pub name: String,
}

/// What is being dragged from a library card into the viewport.
#[derive(Clone)]
pub enum DragItem {
    /// A components library file.
    Comp(PathBuf),
    /// A generator (its source).
    Gen(String),
    /// A component of the open document.
    Def(varstotas_kernel::DefId),
    /// A material; `true` = from the library (added to the document on the drop).
    Mat(Box<varstotas_kernel::Material>, bool),
}

/// A tile being dragged: what, its name (shown at the cursor), its recents key, and whether
/// the viewport was reached (its tool is on: placing, or Paint).
#[derive(Clone)]
pub struct TrayDrag {
    pub item: DragItem,
    pub name: String,
    /// (materials card, key) for the recents.
    pub recent: Option<(bool, String)>,
    pub entered: bool,
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
