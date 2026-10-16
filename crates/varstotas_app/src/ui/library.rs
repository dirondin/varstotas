//! Components: a local library of `.vrst` files (the Default Tray card). The library is
//! `~/.varstotas/components` plus folders added by the user; subfolders are categories. A click
//! on an item places it as a component (one definition per document, however often it is
//! placed); "Save Selection" writes the selection to the library.

use crate::app::{Action, VarstotasApp};
use crate::clipboard::Clipboard;
use egui::{Color32, Rect, RichText, Sense, Stroke, Ui, pos2, vec2};
use varstotas_kernel::glam::DVec3;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

/// Edges kept for a thumbnail.
pub(crate) const THUMB_EDGES: usize = 4000;
const TILE: f32 = 78.0;

pub struct LibItem {
    pub path: PathBuf,
    pub name: String,
    /// Subfolder ("" = the folder itself).
    pub folder: String,
    pub size: DVec3,
    /// Thumbnail edges, projected (iso) into the unit square.
    thumb: Vec<(egui::Pos2, egui::Pos2)>,
    pub clip: Rc<Clipboard>,
    mtime: Option<SystemTime>,
}

#[derive(Default)]
pub struct LibraryState {
    pub items: Vec<LibItem>,
    pub errors: Vec<String>,
    pub filter: String,
    /// Name being typed for Save Selection.
    pub save_name: Option<String>,
    pub scanned: Option<PathBuf>,
    /// Open the tray card on the next frame (Window → Components).
    pub open_card: bool,
    /// Generators (`~/.varstotas/generators`).
    pub gens: super::generators::GenLibrary,
}

/// `~/.varstotas/components`.
pub fn default_dir() -> Option<PathBuf> {
    crate::bridge::home_dir().map(|d| d.join("components"))
}

/// Iso projection of the edges into the unit square (y down).
pub(crate) fn thumbnail(edges: &[(DVec3, DVec3)]) -> Vec<(egui::Pos2, egui::Pos2)> {
    let s2 = std::f64::consts::FRAC_1_SQRT_2;
    let s6 = 1.0 / 6f64.sqrt();
    let proj = |p: DVec3| ((p.x + p.y) * s2, -(-p.x + p.y + 2.0 * p.z) * s6);
    let pts: Vec<_> = edges.iter().flat_map(|(a, b)| [proj(*a), proj(*b)]).collect();
    let (mut lo, mut hi) = ((f64::MAX, f64::MAX), (f64::MIN, f64::MIN));
    for (u, v) in &pts {
        lo = (lo.0.min(*u), lo.1.min(*v));
        hi = (hi.0.max(*u), hi.1.max(*v));
    }
    let span = (hi.0 - lo.0).max(hi.1 - lo.1).max(1e-9);
    let off = ((span - (hi.0 - lo.0)) / 2.0, (span - (hi.1 - lo.1)) / 2.0);
    let q = |(u, v): (f64, f64)| pos2(((u - lo.0 + off.0) / span) as f32, ((v - lo.1 + off.1) / span) as f32);
    pts.chunks(2).map(|c| (q(c[0]), q(c[1]))).collect()
}

fn load_item(path: &Path, root: &Path) -> Result<LibItem, String> {
    let model = crate::document::guarded(|| varstotas_kernel::io::load(path)).and_then(|r| r.map_err(|e| e.to_string()))?;
    let name = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let clip = crate::document::guarded(|| Clipboard::from_library(model, &name)).and_then(|r| r.map_err(|e| e.to_string()))?;
    let folder = path
        .parent()
        .and_then(|p| p.strip_prefix(root).ok())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let edges: Vec<_> = clip.preview.iter().take(THUMB_EDGES).copied().collect();
    Ok(LibItem {
        path: path.to_path_buf(),
        name,
        folder,
        size: clip.bounds.max - clip.bounds.min,
        thumb: thumbnail(&edges),
        clip: Rc::new(clip),
        mtime: std::fs::metadata(path).and_then(|m| m.modified()).ok(),
    })
}

/// `.vrst` files of `root` (two folder levels), reusing unchanged items of `old`.
fn scan(root: &Path, old: Vec<LibItem>) -> (Vec<LibItem>, Vec<String>) {
    let mut files = Vec::new();
    fn walk(d: &Path, depth: usize, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() && depth < 2 {
                walk(&p, depth + 1, out);
            } else if p.extension().is_some_and(|x| x == "vrst") {
                out.push(p);
            }
        }
    }
    walk(root, 0, &mut files);
    files.sort();
    let mut old: std::collections::HashMap<PathBuf, LibItem> = old.into_iter().map(|i| (i.path.clone(), i)).collect();
    let (mut items, mut errors) = (Vec::new(), Vec::new());
    for f in files {
        let mtime = std::fs::metadata(&f).and_then(|m| m.modified()).ok();
        match old.remove(&f) {
            Some(i) if i.mtime == mtime => items.push(i),
            _ => match load_item(&f, root) {
                Ok(i) => items.push(i),
                Err(e) => errors.push(format!("{}: {e}", f.display())),
            },
        }
    }
    items.sort_by(|a, b| a.folder.cmp(&b.folder).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    (items, errors)
}

fn safe_name(s: &str) -> String {
    let n: String = s.chars().map(|c| if c.is_alphanumeric() || " -_.()".contains(c) { c } else { '_' }).collect();
    let n = n.trim().trim_matches('.').to_string();
    if n.is_empty() { "Component".into() } else { n }
}

impl VarstotasApp {
    /// The library folder shown (the default one or a folder chosen by the user).
    pub(crate) fn library_dir(&self) -> Option<PathBuf> {
        self.prefs.library_dir.clone().or_else(default_dir)
    }

    pub(crate) fn refresh_library(&mut self) {
        let Some(dir) = self.library_dir() else { return };
        let _ = std::fs::create_dir_all(&dir);
        let (items, errors) = scan(&dir, std::mem::take(&mut self.library.items));
        for e in &errors {
            log::warn!("library: {e}");
        }
        self.library.items = items;
        self.library.errors = errors;
        self.library.scanned = Some(dir);
    }

    pub(crate) fn library_index(&self, name: &str) -> Option<usize> {
        self.library.items.iter().position(|i| i.name.eq_ignore_ascii_case(name.trim()))
    }

    /// Start placing library item `i`.
    pub(crate) fn place_component(&mut self, i: usize) {
        let Some(item) = self.library.items.get(i) else { return };
        let tool = crate::tools::transform::PasteTool::named(item.clip.clone(), item.name.clone());
        self.install_tool(Box::new(tool));
    }

    /// Default name for Save Selection: the selected group / component's name.
    fn selection_name(&self) -> String {
        let m = &self.doc.model;
        if let [varstotas_kernel::EntityRef::Instance(i)] = self.doc.selection.as_slice()
            && let Some(inst) = self.doc.active_mesh().and_then(|x| x.instances.get(*i))
        {
            if !inst.name.is_empty() {
                return inst.name.clone();
            }
            if let Some(d) = m.defs.get(inst.def)
                && !d.is_group
            {
                return d.name.clone();
            }
        }
        "Component".into()
    }

    /// Write the selection to `<library>/<name>.vrst` (a new name when it exists).
    pub(crate) fn save_to_library(&mut self, name: &str) {
        let Some(dir) = self.library_dir() else { return };
        let r = Clipboard::from_selection(&self.doc).and_then(|c| c.to_model(self.doc.model.unit));
        let model = match r {
            Ok(m) => m,
            Err(e) => return self.flash_error(format!("Save to Library: {e}")),
        };
        let base = safe_name(name);
        let mut path = dir.join(format!("{base}.vrst"));
        let mut n = 2;
        while path.exists() {
            path = dir.join(format!("{base} {n}.vrst"));
            n += 1;
        }
        let _ = std::fs::create_dir_all(&dir);
        match varstotas_kernel::io::save(&model, &path) {
            Ok(()) => {
                self.flash(format!("Saved to the library: {}", path.display()));
                self.refresh_library();
            }
            Err(e) => self.flash_error(format!("Save to Library failed: {e}")),
        }
    }

    pub(crate) fn library_open_requested(&mut self) -> bool {
        std::mem::take(&mut self.library.open_card)
    }

    /// The Components card.
    pub(crate) fn library_panel(&mut self, ui: &mut Ui) {
        if self.library.scanned != self.library_dir() {
            self.refresh_library();
        }
        let unit = self.doc.model.unit;
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.library.filter).hint_text("Search").desired_width(ui.available_width() - 70.0));
            if ui.button("⟳").on_hover_text("Refresh").clicked() {
                self.refresh_library();
                self.refresh_generators();
            }
            if ui.button("…").on_hover_text("Choose the library folder").clicked()
                && let Some(d) = rfd::FileDialog::new().pick_folder()
            {
                self.prefs.library_dir = Some(d);
            }
        });
        if let Some(d) = self.library_dir() {
            let custom = self.prefs.library_dir.is_some();
            ui.horizontal(|ui| {
                let shown = d.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if ui.link(RichText::new(shown).small()).on_hover_text(format!("{}\nClick to show in the Finder", d.display())).clicked() {
                    crate::ui::extensions::reveal(&d);
                }
                if custom && ui.small_button("Default").on_hover_text("Back to ~/.varstotas/components").clicked() {
                    self.prefs.library_dir = None;
                }
            });
        }
        let filter = self.library.filter.trim().to_lowercase();
        let shown: Vec<usize> =
            (0..self.library.items.len()).filter(|i| filter.is_empty() || self.library.items[*i].name.to_lowercase().contains(&filter)).collect();
        if self.library.items.is_empty() {
            ui.label(RichText::new("No components yet: select something and use Save Selection.").weak());
        } else if shown.is_empty() {
            ui.label(RichText::new("Nothing found").weak());
        }
        let mut place = None;
        let cols = ((ui.available_width() + 6.0) / (TILE + 6.0)).floor().max(1.0) as usize;
        let mut folder: Option<&str> = None;
        let items = &self.library.items;
        let mut row: Vec<usize> = Vec::new();
        let mut flush = |ui: &mut Ui, row: &mut Vec<usize>| {
            if row.is_empty() {
                return;
            }
            ui.horizontal(|ui| {
                for &i in row.iter() {
                    if tile(ui, &items[i], unit) {
                        place = Some(i);
                    }
                }
            });
            row.clear();
        };
        for &i in &shown {
            let it = &items[i];
            if folder != Some(it.folder.as_str()) {
                flush(ui, &mut row);
                if !it.folder.is_empty() {
                    ui.label(RichText::new(&it.folder).strong().small());
                }
                folder = Some(it.folder.as_str());
            }
            row.push(i);
            if row.len() == cols {
                flush(ui, &mut row);
            }
        }
        flush(ui, &mut row);
        if !self.library.errors.is_empty() {
            ui.label(RichText::new(format!("{} file(s) could not be read", self.library.errors.len())).color(Color32::from_rgb(200, 50, 40)))
                .on_hover_text(self.library.errors.join("\n"));
        }
        self.generators_panel(ui);
        ui.add_space(4.0);
        match self.library.save_name.as_mut() {
            None => {
                let can = !self.doc.selection.is_empty();
                if ui.add_enabled(can, egui::Button::new("Save Selection to Library…")).clicked() {
                    self.library.save_name = Some(self.selection_name());
                }
            }
            Some(name) => {
                let mut done = None;
                ui.horizontal(|ui| {
                    let r = ui.add(egui::TextEdit::singleline(name).desired_width(ui.available_width() - 110.0));
                    if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        done = Some(true);
                    }
                    if ui.button("Save").clicked() {
                        done = Some(true);
                    }
                    if ui.button("Cancel").clicked() {
                        done = Some(false);
                    }
                });
                if let Some(ok) = done {
                    let n = self.library.save_name.take().unwrap_or_default();
                    if ok {
                        self.actions.push(Action::SaveToLibrary(n));
                    }
                }
            }
        }
        if let Some(i) = place {
            self.actions.push(Action::PlaceComponent(i));
        }
    }
}

/// One library item; true when clicked.
fn tile(ui: &mut Ui, it: &LibItem, unit: varstotas_kernel::LengthUnit) -> bool {
    tile_ex(ui, &it.name, &it.thumb, it.size, unit, false, "")
}

/// A library tile: thumbnail edges in the unit square, the name below; `gear` marks a
/// generator (a gear in the top right corner). True when clicked.
pub(crate) fn tile_ex(
    ui: &mut Ui,
    name: &str,
    thumb: &[(egui::Pos2, egui::Pos2)],
    size: DVec3,
    unit: varstotas_kernel::LengthUnit,
    gear: bool,
    note: &str,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(TILE, TILE + 16.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let pic = Rect::from_min_size(rect.min, vec2(TILE, TILE));
        let fill = if resp.hovered() { Color32::from_rgb(0xE6, 0xEE, 0xFA) } else { Color32::from_gray(248) };
        p.rect_filled(pic, 6.0, fill);
        p.rect_stroke(pic, 6.0, Stroke::new(1.0, Color32::from_gray(215)), egui::StrokeKind::Inside);
        let inner = pic.shrink(8.0);
        let to = |q: egui::Pos2| inner.min + vec2(q.x * inner.width(), q.y * inner.height());
        let stroke = Stroke::new(0.8, Color32::from_gray(60));
        for (a, b) in thumb {
            p.line_segment([to(*a), to(*b)], stroke);
        }
        if gear {
            let c = pos2(pic.max.x - 10.0, pic.min.y + 10.0);
            p.circle_filled(c, 8.0, Color32::from_rgb(0x3B, 0x6E, 0xC4));
            p.text(c, egui::Align2::CENTER_CENTER, "⚙", egui::FontId::proportional(12.0), Color32::WHITE);
        }
        let label = if name.chars().count() > 12 { format!("{}…", name.chars().take(11).collect::<String>()) } else { name.to_string() };
        p.text(
            pos2(rect.center().x, rect.max.y - 7.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(11.0),
            ui.visuals().text_color(),
        );
    }
    let f = |v: f64| varstotas_kernel::units::format_length(v, unit);
    let note = if note.is_empty() { String::new() } else { format!("{note}\n") };
    resp.on_hover_text(format!("{name}\n{note}{} × {} × {}\nClick to place", f(size.x), f(size.y), f(size.z))).clicked()
}
