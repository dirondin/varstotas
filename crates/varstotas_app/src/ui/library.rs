//! Components: a local library (the Default Tray card) of `.vrst` files and generators
//! (`.rhai`, components with parameters — see `ui/generators.rs`), in one list. The library is
//! `~/.varstotas/components` (or a folder chosen by the user); subfolders are categories, the
//! bundled example generators show under "Examples". A click on an item places it as a
//! component (one definition per document, however often it is placed); "Save Selection"
//! writes the selection to the library.

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
    /// The file's preview picture (shown instead of the edges), decoded on first draw.
    pub preview: Option<std::sync::Arc<[u8]>>,
    picture: std::cell::OnceCell<Option<egui::TextureHandle>>,
}

impl LibItem {
    /// The preview picture as a texture (None without one or when it can't be read).
    fn picture(&self, ctx: &egui::Context) -> Option<egui::TextureId> {
        let bytes = self.preview.as_ref()?;
        self.picture
            .get_or_init(|| {
                let ci = crate::textures::thumbnail(bytes, 2 * TILE as u32)?;
                Some(ctx.load_texture(format!("libpreview:{}", self.path.display()), ci, egui::TextureOptions::LINEAR))
            })
            .as_ref()
            .map(|t| t.id())
    }
}

#[derive(Default)]
pub struct LibraryState {
    pub items: Vec<LibItem>,
    pub errors: Vec<String>,
    pub filter: String,
    /// Name being typed for Save Selection.
    pub save_name: Option<String>,
    pub scanned: Option<PathBuf>,
    /// Open the tray card on the next frame (the `components` harness command).
    pub open_card: bool,
    /// Folders of the list drawn last: (folder, tiles, folded), for the harness.
    pub shown_folders: Vec<(String, usize, bool)>,
    /// Generators of the same folder.
    pub gens: super::generators::GenLibrary,
    /// To delete once confirmed: (library file, or None for a generator of the document; name).
    pub delete_ask: Option<(Option<PathBuf>, String)>,
    /// A document's component to delete once confirmed (it has placed copies).
    pub def_delete: Option<varstotas_kernel::DefId>,
    /// The Rename dialog: the component (document's or a library file) and the name typed.
    pub rename: Option<(RenameTarget, String)>,
    /// Thumbnails of the document's components (Document folder).
    pub def_thumbs: std::collections::HashMap<varstotas_kernel::DefId, DefThumb>,
}

/// What the Rename dialog renames.
#[derive(Clone, Debug, PartialEq)]
pub enum RenameTarget {
    Def(varstotas_kernel::DefId),
    File(PathBuf),
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

/// A tile of the Components list.
#[derive(Clone, Copy)]
enum Tile {
    Comp(usize),
    Gen(usize),
    /// A component definition of the open document.
    Def(varstotas_kernel::DefId),
}

/// Thumbnail and size of a document's component definition, for one model version.
pub struct DefThumb {
    version: u64,
    thumb: Vec<(egui::Pos2, egui::Pos2)>,
    size: DVec3,
    /// Rendered picture ([`crate::thumbs`]) and its key.
    pic: Option<egui::TextureId>,
    pic_key: String,
}

fn load_item(path: &Path, root: &Path) -> Result<LibItem, String> {
    let mut model = crate::document::guarded(|| varstotas_kernel::io::load(path)).and_then(|r| r.map_err(|e| e.to_string()))?;
    let preview = model.preview.take().map(|p| p.bytes);
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
        preview,
        picture: Default::default(),
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

/// Rename the library file `path` to `<name>.vrst` in its folder; a file holding one
/// component (or a component with parameters) named after the file gets the new name too.
/// Returns the new path.
fn rename_library_file(path: &Path, name: &str) -> Result<PathBuf, String> {
    let to = path.with_file_name(format!("{}.vrst", safe_name(name)));
    if to == path {
        return Ok(to);
    }
    if to.exists() && !to.to_string_lossy().eq_ignore_ascii_case(&path.to_string_lossy()) {
        return Err(format!("{} already exists", to.display()));
    }
    let old = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let mut model = crate::document::guarded(|| varstotas_kernel::io::load(path)).and_then(|r| r.map_err(|e| e.to_string()))?;
    let root = model.root_mesh();
    if root.faces.is_empty() && root.edges.is_empty() && root.instances.len() == 1 {
        let d = root.instances.values().next().expect("one instance").def;
        let rigged = model.defs.get(d).is_some_and(|x| x.rigged.is_some());
        if rigged || model.defs.get(d).is_some_and(|x| !x.is_group && x.name == old) {
            model.rename_definition(d, name).map_err(|e| e.to_string())?;
        }
    }
    varstotas_kernel::io::save(&model, &to).map_err(|e| e.to_string())?;
    if !to.to_string_lossy().eq_ignore_ascii_case(&path.to_string_lossy()) {
        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(to)
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
        self.refresh_generators();
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

    /// A folder heading of the Components list ("" = the library folder itself): a fold arrow
    /// and the name, a link showing the folder in the Finder (the bundled Examples and the Document folder: Hide instead). True
    /// when folded.
    fn folder_header(&mut self, ui: &mut Ui, folder: &str, root: Option<&Path>) -> bool {
        let folded = self.prefs.library_collapsed.iter().any(|f| f == folder);
        let examples = folder == super::generators::EXAMPLES_FOLDER;
        let document = folder == super::generators::DOCUMENT_FOLDER;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let arrow = ui
                .add_sized([14.0, 18.0], egui::Button::new(if folded { "⏵" } else { "⏷" }).frame(false))
                .on_hover_text(if folded { "Show the folder" } else { "Fold the folder" });
            if arrow.clicked() {
                if folded {
                    self.prefs.library_collapsed.retain(|f| f != folder);
                } else {
                    self.prefs.library_collapsed.push(folder.to_string());
                }
            }
            if examples {
                ui.label(folder).on_hover_text("Generators bundled with Varstotas (not files)");
                if ui.small_button("Hide").on_hover_text("Hide the bundled examples (… → Show Examples brings them back)").clicked() {
                    self.prefs.hide_examples = true;
                }
            } else if document {
                ui.label(folder).on_hover_text("Generators found only in the open document (Edit → Save to Library makes a file)");
                if ui.small_button("Hide").on_hover_text("Hide the document's generators (… → Show Document brings them back)").clicked() {
                    self.prefs.hide_document = true;
                }
            } else if folder.is_empty() {
                // The library folder: its name, a link; Default when the user chose another one.
                if let Some(dir) = root {
                    let name = dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                    if ui.link(name).on_hover_text(format!("{}\nClick to show in the Finder", dir.display())).clicked() {
                        crate::ui::extensions::reveal(dir);
                    }
                }
                if self.prefs.library_dir.is_some() && ui.small_button("Default").on_hover_text("Back to ~/.varstotas/components").clicked() {
                    self.prefs.library_dir = None;
                }
            } else if let Some(dir) = root.map(|r| r.join(folder))
                && ui.link(folder).on_hover_text(format!("{}\nClick to show in the Finder", dir.display())).clicked()
            {
                crate::ui::extensions::reveal(&dir);
            }
        });
        folded
    }

    /// "Delete from the library?" for the file chosen in a tile's context menu.
    pub(crate) fn library_delete_dialog(&mut self, ctx: &egui::Context) {
        let Some((path, name)) = self.library.delete_ask.clone() else { return };
        let mut choice = None;
        let resp = egui::Modal::new(egui::Id::new("library_delete")).show(ctx, |ui| {
            ui.set_width(360.0);
            match &path {
                Some(path) => {
                    ui.heading("Delete from the library?");
                    ui.add_space(4.0);
                    ui.label(format!("“{name}” will be deleted from the library folder:"));
                    ui.label(RichText::new(path.display().to_string()).small().weak());
                    ui.label(RichText::new("This cannot be undone. Copies already placed in documents stay.").weak());
                }
                None => {
                    ui.heading("Delete from the document?");
                    ui.add_space(4.0);
                    ui.label(format!("The generator “{name}” will be removed from this document."));
                    let uses = self.doc.model.generator_uses(&name);
                    if uses > 0 {
                        ui.label(RichText::new(format!("{uses} placed component(s) stay as plain components, without parameters.")).weak());
                    }
                    ui.label(RichText::new("Edit → Undo brings it back.").weak());
                }
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new("Delete").strong().color(Color32::from_rgb(200, 50, 40)))).clicked() {
                        choice = Some(true);
                    }
                    if ui.button("Cancel").clicked() {
                        choice = Some(false);
                    }
                });
            });
        });
        if resp.should_close() && choice.is_none() {
            choice = Some(false);
        }
        match choice {
            Some(true) if path.is_none() => {
                self.library.delete_ask = None;
                if let Some(Some(n)) = self.doc.commit(&format!("Delete Generator {name}"), |m| m.remove_generator(&name)) {
                    self.flash(if n > 0 { format!("Removed the generator {name}: {n} component(s) stay without parameters") } else { format!("Removed the generator {name}") });
                }
                self.library.gens.thumbs.clear();
            }
            Some(true) => {
                self.library.delete_ask = None;
                let path = path.unwrap_or_default();
                match std::fs::remove_file(&path) {
                    Ok(()) => {
                        self.flash(format!("Deleted from the library: {name}"));
                        self.refresh_library();
                    }
                    Err(e) => self.flash_error(format!("Delete {}: {e}", path.display())),
                }
            }
            Some(false) => self.library.delete_ask = None,
            None => {}
        }
    }

    /// Delete the document's component `d`: at once when nothing places it, else after the
    /// "Delete from the document?" question ([`Self::def_delete_dialog`]).
    pub(crate) fn ask_delete_definition(&mut self, d: varstotas_kernel::DefId) {
        if self.doc.model.placed_count(d) == 0 {
            self.delete_definition(d);
        } else {
            self.library.def_delete = Some(d);
        }
    }

    /// Remove component `d` and its placed copies (one undo step).
    pub(crate) fn delete_definition(&mut self, d: varstotas_kernel::DefId) {
        let name = self.doc.model.defs.get(d).map(|x| x.name.clone()).unwrap_or_default();
        if let Some(n) = self.doc.try_commit(&format!("Delete Component {name}"), |m| m.delete_definition(d)) {
            self.library.def_thumbs.remove(&d);
            self.flash(if n > 0 { format!("Deleted {name} and {n} placed copy(ies)") } else { format!("Deleted {name} from the document") });
        }
    }

    /// "Delete from the document?" for a component with placed copies.
    pub(crate) fn def_delete_dialog(&mut self, ctx: &egui::Context) {
        let Some(d) = self.library.def_delete else { return };
        let Some(def) = self.doc.model.defs.get(d) else {
            self.library.def_delete = None;
            return;
        };
        let (name, uses) = (def.name.clone(), self.doc.model.placed_count(d));
        let mut choice = None;
        let resp = egui::Modal::new(egui::Id::new("def_delete")).show(ctx, |ui| {
            ui.set_width(360.0);
            ui.heading("Delete from the document?");
            ui.add_space(4.0);
            ui.label(format!("“{name}” is placed {uses} time(s). The component and all its copies will be removed."));
            ui.label(RichText::new("Edit → Undo brings them back.").weak());
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new("Delete").strong().color(Color32::from_rgb(200, 50, 40)))).clicked() {
                        choice = Some(true);
                    }
                    if ui.button("Cancel").clicked() {
                        choice = Some(false);
                    }
                });
            });
        });
        if resp.should_close() && choice.is_none() {
            choice = Some(false);
        }
        if let Some(ok) = choice {
            self.library.def_delete = None;
            if ok {
                self.delete_definition(d);
            }
        }
    }

    /// The Rename dialog of a component tile ([`LibraryState::rename`]).
    pub(crate) fn component_rename_dialog(&mut self, ctx: &egui::Context) {
        let Some((target, mut text)) = self.library.rename.clone() else { return };
        let mut choice = None;
        let resp = egui::Modal::new(egui::Id::new("component_rename")).show(ctx, |ui| {
            ui.set_width(340.0);
            ui.heading("Rename Component");
            ui.add_space(4.0);
            if let RenameTarget::File(path) = &target {
                let file = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                ui.label(RichText::new(format!("The library file {file} is renamed too.")).small().weak()).on_hover_text(path.display().to_string());
            }
            let r = ui.add(egui::TextEdit::singleline(&mut text).desired_width(f32::INFINITY));
            if !r.has_focus() && !r.lost_focus() {
                r.request_focus();
            }
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                choice = Some(true);
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add_enabled(!text.trim().is_empty(), egui::Button::new(RichText::new("Rename").strong())).clicked() {
                        choice = Some(true);
                    }
                    if ui.button("Cancel").clicked() {
                        choice = Some(false);
                    }
                });
            });
        });
        if resp.should_close() && choice.is_none() {
            choice = Some(false);
        }
        match choice {
            Some(true) if !text.trim().is_empty() => {
                self.library.rename = None;
                self.rename_component(&target, text.trim());
            }
            Some(_) => self.library.rename = None,
            None => self.library.rename = Some((target, text)),
        }
    }

    /// Rename a document's component (with its variants, one undo step) or a library file
    /// (the file and the component inside it).
    pub(crate) fn rename_component(&mut self, target: &RenameTarget, name: &str) {
        match target {
            RenameTarget::Def(d) => {
                let d = *d;
                if self.doc.try_commit(&format!("Rename Component {name}"), |m| m.rename_definition(d, name)).is_some() {
                    self.flash(format!("Renamed to {name}"));
                }
            }
            RenameTarget::File(path) => match rename_library_file(path, name) {
                Ok(to) => {
                    self.flash(format!("Renamed to {}", to.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()));
                    self.refresh_library();
                }
                Err(e) => self.flash_error(format!("Rename {}: {e}", path.display())),
            },
        }
    }

    pub(crate) fn library_open_requested(&mut self) -> bool {
        std::mem::take(&mut self.library.open_card)
    }

    /// Component definitions of the open document (not groups, generated components or the
    /// internal copies of parametric groups), with their names, by name.
    pub(crate) fn document_components(&self) -> Vec<(varstotas_kernel::DefId, String)> {
        let m = &self.doc.model;
        let internal: std::collections::HashSet<_> =
            m.defs.values().filter_map(|d| d.params.as_ref()).flat_map(|p| std::iter::once(p.base).chain(p.variant)).collect();
        let mut out: Vec<_> = m
            .defs
            .iter()
            .filter(|(id, d)| *id != m.root && !d.is_group && d.generated.is_none() && d.rigged.is_none() && !internal.contains(id))
            .map(|(id, d)| (id, if d.name.is_empty() { "Component".to_string() } else { d.name.clone() }))
            .collect();
        out.sort_by_key(|a| a.1.to_lowercase());
        out
    }

    /// A tile of a document's component (thumbnail rebuilt when the model changes).
    fn def_tile(&mut self, ui: &mut Ui, d: varstotas_kernel::DefId) -> egui::Response {
        let version = self.doc.model_version;
        let m = &self.doc.model;
        let fresh = self.library.def_thumbs.get(&d).is_some_and(|t| t.version == version);
        if !fresh {
            let edges = crate::clipboard::def_edges(m, d, THUMB_EDGES);
            let pts = edges.iter().flat_map(|(a, b)| [*a, *b]);
            let size = varstotas_kernel::Aabb::from_points(pts).map_or(DVec3::ZERO, |b| b.max - b.min);
            // The rendered picture of the previous version stays until the new one is there.
            let (pic, pic_key) = self.library.def_thumbs.remove(&d).map(|t| (t.pic, t.pic_key)).unwrap_or_default();
            self.library.def_thumbs.insert(d, DefThumb { version, thumb: thumbnail(&edges), size, pic, pic_key });
        }
        let key = format!("def:{}:{d:?}:{version}", self.doc.uid);
        let (uid, unit) = (self.doc.uid, m.unit);
        if let Some(id) = self.thumbs.get(ui.ctx(), &key, || None, || Clipboard::of_definition(m, uid, d).and_then(|c| c.to_model(unit)).ok()) {
            let t = self.library.def_thumbs.get_mut(&d).expect("thumb");
            if t.pic_key != key {
                if !t.pic_key.is_empty() {
                    self.thumbs.remove(&t.pic_key);
                }
                t.pic = Some(id);
                t.pic_key = key;
            }
        }
        let name = m.defs.get(d).map(|x| x.name.clone()).unwrap_or_default();
        let uses = m.defs.values().flat_map(|x| x.mesh.instances.values()).filter(|i| i.def == d).count();
        let t = &self.library.def_thumbs[&d];
        let note = format!("In this document: {uses} placed");
        tile_ex(ui, &name, &t.thumb, t.pic, t.size, m.unit, false, &note)
    }

    /// Set or remove the open document's preview picture (undoable; saved with the file).
    pub(crate) fn set_doc_preview(&mut self, bytes: Option<Vec<u8>>) {
        let set = bytes.is_some();
        self.doc.commit(if set { "Set Preview Image" } else { "Remove Preview Image" }, |m| m.set_preview(bytes));
        self.flash(if set { "Preview image set (saved with the file)" } else { "Preview image removed" });
    }

    /// The document's preview picture from an image file.
    pub(crate) fn preview_from_file(&mut self, f: &Path) {
        match std::fs::read(f).map_err(|e| e.to_string()).and_then(|b| crate::textures::preview_bytes(&b)) {
            Ok(b) => self.set_doc_preview(Some(b)),
            Err(e) => self.flash_error(format!("Preview image: {}: {e}", f.display())),
        }
    }

    /// The document's preview picture from the current view (rendered like File → Export →
    /// 2D Graphic, at most [`crate::textures::PREVIEW_SIDE`] pixels; finished by
    /// [`Self::preview_shot_done`]).
    pub(crate) fn preview_from_view(&mut self) {
        let vp = self.viewport.size();
        let side = crate::textures::PREVIEW_SIDE as f32;
        let k = side / vp.x.max(vp.y).max(1.0);
        let size = [(vp.x * k).round().max(1.0) as u32, (vp.y * k).round().max(1.0) as u32];
        let path = std::env::temp_dir().join(format!("varstotas_preview_{}.png", std::process::id()));
        let content = crate::app::ExportContent { transparent: false, axes: false, guides: false, dims: false };
        self.preview_shot = Some(path.clone());
        self.export_png_to(path, size, content);
    }

    /// True when `r` is the render of [`Self::preview_from_view`] (then it becomes the preview).
    pub(crate) fn preview_shot_done(&mut self, r: &Result<PathBuf, String>) -> bool {
        let Some(path) = self.preview_shot.take() else { return false };
        match r {
            Ok(p) if *p == path => {
                let res = std::fs::read(p).map_err(|e| e.to_string()).and_then(|b| crate::textures::preview_bytes(&b));
                let _ = std::fs::remove_file(p);
                match res {
                    Ok(b) => self.set_doc_preview(Some(b)),
                    Err(e) => self.flash_error(format!("Preview image: {e}")),
                }
                true
            }
            Ok(_) => {
                self.preview_shot = Some(path);
                false
            }
            Err(e) => {
                self.flash_error(format!("Preview image: {e}"));
                true
            }
        }
    }

    /// Set (from the image file `image`) or remove the preview picture of library file `path`.
    pub(crate) fn set_library_preview(&mut self, path: &Path, image: Option<&Path>) {
        let res = (|| -> Result<(), String> {
            let mut m = varstotas_kernel::io::load(path).map_err(|e| e.to_string())?;
            let bytes = match image {
                Some(f) => Some(crate::textures::preview_bytes(&std::fs::read(f).map_err(|e| format!("{}: {e}", f.display()))?)?),
                None => None,
            };
            m.set_preview(bytes);
            varstotas_kernel::io::save(&m, path).map_err(|e| e.to_string())
        })();
        match res {
            Ok(()) => self.refresh_library(),
            Err(e) => {
                log::warn!("preview image of {}: {e}", path.display());
                self.flash_error(format!("Preview image: {e}"));
            }
        }
    }

    /// Place one more instance of the document's component `d`.
    pub(crate) fn place_definition(&mut self, d: varstotas_kernel::DefId) {
        let name = self.doc.model.defs.get(d).map(|x| x.name.clone()).unwrap_or_default();
        match Clipboard::of_definition(&self.doc.model, self.doc.uid, d) {
            Ok(clip) => {
                let tool = crate::tools::transform::PasteTool::named(Rc::new(clip), name);
                self.install_tool(Box::new(tool));
            }
            Err(e) => self.flash_error(format!("Place {name}: {e}")),
        }
    }

    /// Select the instances of `d` placed at the top level of the model.
    fn select_definition(&mut self, d: varstotas_kernel::DefId) {
        let m = &self.doc.model;
        let ids: Vec<_> = m.root_mesh().instances.iter().filter(|(_, i)| i.def == d || m.rig_master(i.def) == Some(d)).map(|(id, _)| varstotas_kernel::EntityRef::Instance(id)).collect();
        if ids.is_empty() {
            self.flash("No copies of this component at the top level of the model".to_string());
            return;
        }
        while self.doc.exit_context() {}
        self.doc.set_selection(ids);
    }

    /// The Components card.
    pub(crate) fn library_panel(&mut self, ui: &mut Ui) {
        if self.library.scanned != self.library_dir() {
            self.refresh_library();
        }
        let unit = self.doc.model.unit;
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.library.filter).hint_text("Search").desired_width(ui.available_width() - 100.0));
            if ui.button("⟳").on_hover_text("Refresh").clicked() {
                self.refresh_library();
            }
            if ui.button("+").on_hover_text("New generator: a component built by a script with parameters (Window → Generator Editor)").clicked() {
                self.gen_editor.new_generator();
            }
            ui.menu_button("…", |ui| {
                if ui.button("Choose Folder…").clicked() {
                    ui.close();
                    if let Some(d) = rfd::FileDialog::new().pick_folder() {
                        self.prefs.library_dir = Some(d);
                    }
                }
                let mut show = !self.prefs.hide_examples;
                if ui.checkbox(&mut show, "Show Examples").on_hover_text("The bundled example generators").changed() {
                    self.prefs.hide_examples = !show;
                }
                let mut show = !self.prefs.hide_document;
                if ui.checkbox(&mut show, "Show Document").on_hover_text("Generators found only in the open document").changed() {
                    self.prefs.hide_document = !show;
                }
            })
            .response
            .on_hover_text("Library folder, examples");
        });
        let filter = self.library.filter.trim().to_lowercase();
        let found = |name: &str| filter.is_empty() || name.to_lowercase().contains(&filter);
        // Components and generators in one list: by folder, then name.
        let gens = self.generator_items();
        let doc_defs = if self.prefs.hide_document { Vec::new() } else { self.document_components() };
        let is_doc = |g: &super::generators::GenItem| g.file.is_none() && !g.bundled && g.folder == super::generators::DOCUMENT_FOLDER;
        // The Draw → Shapes generators (box, cylinder …) are tools, not components to list.
        let shape = |g: &super::generators::GenItem| is_doc(g) && crate::tools::primitives::SOURCES.iter().any(|(_, _, s)| *s == g.source);
        let mut shown: Vec<(String, String, Tile)> = self
            .library
            .items
            .iter()
            .enumerate()
            .filter(|(_, it)| found(&it.name))
            .map(|(i, it)| (it.folder.clone(), it.name.to_lowercase(), Tile::Comp(i)))
            .chain(gens.iter().enumerate().filter(|(_, g)| found(&g.name) && !(g.bundled && self.prefs.hide_examples) && !(is_doc(g) && self.prefs.hide_document) && !shape(g)).map(|(i, g)| (g.folder.clone(), g.name.to_lowercase(), Tile::Gen(i))))
            .chain(doc_defs.iter().filter(|(_, n)| found(n)).map(|(d, n)| (super::generators::DOCUMENT_FOLDER.to_string(), n.to_lowercase(), Tile::Def(*d))))
            .collect();
        // Library folders first, then Document, then Examples.
        let rank = |f: &str| match f {
            super::generators::DOCUMENT_FOLDER => 1,
            super::generators::EXAMPLES_FOLDER => 2,
            _ => 0,
        };
        shown.sort_by(|a, b| rank(&a.0).cmp(&rank(&b.0)).then_with(|| (&a.0, &a.1).cmp(&(&b.0, &b.1))));
        if self.library.items.is_empty() && gens.is_empty() && doc_defs.is_empty() {
            ui.label(RichText::new("No components yet: select something and use Save Selection.").weak());
        } else if shown.is_empty() {
            ui.label(RichText::new("Nothing found").weak());
        }
        let (mut place, mut place_gen, mut edit, mut delete, mut place_def, mut select_def) = (None, None, None, None, None, None);
        let mut delete_def = None;
        let mut rig_def = None;
        let mut rename = None;
        let cols = ((ui.available_width() + 6.0) / (TILE + 6.0)).floor().max(1.0) as usize;
        let lib_dir = self.library_dir();
        let mut set_preview: Option<(PathBuf, bool)> = None;
        self.library.shown_folders.clear();
        // The library folder itself: always a heading (its own items fold under it).
        let root_folded = self.folder_header(ui, "", lib_dir.as_deref());
        let mut k = 0;
        while k < shown.len() {
            let folder = shown[k].0.clone();
            let end = shown[k..].iter().position(|t| t.0 != folder).map_or(shown.len(), |n| k + n);
            let folded = if folder.is_empty() { root_folded } else { self.folder_header(ui, &folder, lib_dir.as_deref()) };
            self.library.shown_folders.push((folder.clone(), end - k, folded));
            if folded {
                k = end;
                continue;
            }
            for row in shown[k..end].chunks(cols) {
                ui.horizontal(|ui| {
                    for (_, _, t) in row {
                        match *t {
                            Tile::Comp(i) => {
                                let it = &self.library.items[i];
                                // The file's picture, else a rendered thumbnail.
                                let pic = it.picture(ui.ctx()).or_else(|| {
                                    let key = format!("lib:{}|{:?}", it.path.display(), it.mtime);
                                    self.thumbs.get(ui.ctx(), &key, || crate::thumbs::disk_path(&it.path), || it.clip.to_model(unit).ok())
                                });
                                let resp = tile_ex(ui, &it.name, &it.thumb, pic, it.size, unit, false, "");
                                if resp.clicked() {
                                    place = Some(i);
                                }
                                resp.context_menu(|ui| {
                                    if ui.button("Place").clicked() {
                                        place = Some(i);
                                        ui.close();
                                    }
                                    if ui.button("Set Preview Image…").on_hover_text("A picture shown here instead of the drawn model (saved in the file)").clicked() {
                                        set_preview = Some((it.path.clone(), true));
                                        ui.close();
                                    }
                                    if it.preview.is_some() && ui.button("Remove Preview Image").clicked() {
                                        set_preview = Some((it.path.clone(), false));
                                        ui.close();
                                    }
                                    if ui.button("Rename…").clicked() {
                                        rename = Some((RenameTarget::File(it.path.clone()), it.name.clone()));
                                        ui.close();
                                    }
                                    if ui.button("Delete…").clicked() {
                                        delete = Some((Some(it.path.clone()), it.name.clone()));
                                        ui.close();
                                    }
                                });
                            }
                            Tile::Def(d) => {
                                let resp = self.def_tile(ui, d);
                                if resp.clicked() {
                                    place_def = Some(d);
                                }
                                resp.context_menu(|ui| {
                                    if ui.button("Place").clicked() {
                                        place_def = Some(d);
                                        ui.close();
                                    }
                                    if ui.button("Select Instances").clicked() {
                                        select_def = Some(d);
                                        ui.close();
                                    }
                                    if ui.button("Rename…").clicked() {
                                        let name = self.doc.model.defs.get(d).map(|x| x.name.clone()).unwrap_or_default();
                                        rename = Some((RenameTarget::Def(d), name));
                                        ui.close();
                                    }
                                    if ui.button("Delete from Document…").on_hover_text("The component and all its placed copies").clicked() {
                                        delete_def = Some(d);
                                        ui.close();
                                    }
                                    if ui.button("Parameters…").on_hover_text("Window → Component Parameters: values that move, show or paint its parts").clicked() {
                                        rig_def = Some(d);
                                        ui.close();
                                    }
                                });
                            }
                            Tile::Gen(i) => {
                                let (p, e, d) = self.generator_tile(ui, &gens[i]);
                                if d {
                                    delete = Some((gens[i].file.clone(), gens[i].name.clone()));
                                }
                                if p {
                                    place_gen = Some(gens[i].source.clone());
                                }
                                if e {
                                    edit = Some(i);
                                }
                            }
                        }
                    }
                });
            }
            k = end;
        }
        if delete.is_some() {
            self.library.delete_ask = delete;
        }
        if rename.is_some() {
            self.library.rename = rename;
        }
        if let Some((path, set)) = set_preview {
            let image = if set { rfd::FileDialog::new().add_filter("Image", &["png", "jpg", "jpeg", "tga", "bmp"]).pick_file() } else { None };
            if !set || image.is_some() {
                self.set_library_preview(&path, image.as_deref());
            }
        }
        if let Some(d) = place_def {
            self.place_definition(d);
        }
        if let Some(d) = select_def {
            self.select_definition(d);
        }
        if let Some(d) = rig_def {
            self.open_rig_editor_for(d, None);
        }
        if let Some(d) = delete_def {
            self.ask_delete_definition(d);
        }
        if let Some(src) = place_gen {
            self.actions.push(Action::PlaceGenerator(src, Default::default()));
        }
        if let Some(i) = edit {
            self.gen_editor.edit(&gens[i].source, gens[i].file.clone());
        }
        if !self.library.errors.is_empty() {
            ui.label(RichText::new(format!("{} file(s) could not be read", self.library.errors.len())).color(Color32::from_rgb(200, 50, 40)))
                .on_hover_text(self.library.errors.join("\n"));
        }
        if !self.library.gens.errors.is_empty() {
            ui.label(RichText::new(format!("{} generator(s) could not be read", self.library.gens.errors.len())).color(Color32::from_rgb(200, 50, 40)))
                .on_hover_text(self.library.gens.errors.join("\n"));
        }
        ui.add_space(4.0);
        match self.library.save_name.as_mut() {
            None => {
                let can = !self.doc.selection.is_empty();
                if ui.add_enabled(can, egui::Button::new("Save Selection to Library…")).clicked() {
                    self.library.save_name = Some(self.selection_name());
                }
                if ui.button("3D Library…").on_hover_text("Free models from the web (Poly Haven, Smithsonian, NASA, plugins)").clicked() {
                    self.models.open = true;
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

/// A library tile: the picture `image` (fitted) or else the thumbnail edges in the unit
/// square, the name below; `gear` marks a generator (a gear in the top right corner).
#[allow(clippy::too_many_arguments)]
pub(crate) fn tile_ex(
    ui: &mut Ui,
    name: &str,
    thumb: &[(egui::Pos2, egui::Pos2)],
    image: Option<egui::TextureId>,
    size: DVec3,
    unit: varstotas_kernel::LengthUnit,
    gear: bool,
    note: &str,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(TILE, TILE + 16.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let pic = Rect::from_min_size(rect.min, vec2(TILE, TILE));
        let fill = if resp.hovered() { Color32::from_rgb(0xE6, 0xEE, 0xFA) } else { Color32::from_gray(248) };
        p.rect_filled(pic, 6.0, fill);
        p.rect_stroke(pic, 6.0, Stroke::new(1.0, Color32::from_gray(215)), egui::StrokeKind::Inside);
        if let Some(id) = image {
            let [w, h] = ui.ctx().tex_manager().read().meta(id).map_or([1, 1], |m| m.size);
            let k = (pic.width() - 2.0) / w.max(h).max(1) as f32;
            let r = Rect::from_center_size(pic.center(), vec2(w as f32 * k, h as f32 * k));
            egui::Image::new((id, r.size())).corner_radius(5.0).paint_at(ui, r);
        } else {
            let inner = pic.shrink(8.0);
            let to = |q: egui::Pos2| inner.min + vec2(q.x * inner.width(), q.y * inner.height());
            let stroke = Stroke::new(0.8, Color32::from_gray(60));
            for (a, b) in thumb {
                p.line_segment([to(*a), to(*b)], stroke);
            }
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
    resp.on_hover_text(format!("{name}\n{note}{} × {} × {}\nClick to place", f(size.x), f(size.y), f(size.z)))
}
