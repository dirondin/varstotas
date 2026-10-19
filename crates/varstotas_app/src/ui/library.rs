//! Components: a local library (the Default Tray card) of `.vrst` files and generators
//! (`.rhai`, components with parameters — see `ui/generators.rs`), in one list. The library is
//! `~/.varstotas/components` (or a folder chosen by the user); subfolders are categories, the
//! bundled example generators show under "Examples". A click on an item places it as a
//! component (one definition per document, however often it is placed); "Save Selection"
//! writes the selection to the library.

use crate::app::{Action, VarstotasApp};
use crate::clipboard::Clipboard;
use egui::{Color32, RichText, Stroke, Ui, pos2, vec2};
use varstotas_kernel::glam::DVec3;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

/// Edges kept for a thumbnail.
pub(crate) const THUMB_EDGES: usize = 4000;
const TILE: f32 = super::tray_lib::LARGE;

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
    /// Pictures of the document's components that have one (`Definition::preview`): the
    /// bytes' address and the texture.
    pub def_pics: std::collections::HashMap<varstotas_kernel::DefId, (usize, egui::TextureHandle)>,
    /// The Rename dialog: the component (document's or a library file) and the name typed.
    pub rename: Option<(RenameTarget, String)>,
    /// Thumbnails of the document's components (Document folder).
    pub def_thumbs: std::collections::HashMap<varstotas_kernel::DefId, DefThumb>,
}

/// Whose preview picture a view render becomes (File → Preview Image, a tile's menu).
#[derive(Clone, Debug, PartialEq)]
pub enum PreviewTarget {
    Document,
    /// A library `.vrst` file.
    File(PathBuf),
    /// A component of the open document.
    Def(varstotas_kernel::DefId),
}

/// What the Rename dialog renames.
#[derive(Clone, Debug, PartialEq)]
pub enum RenameTarget {
    Def(varstotas_kernel::DefId),
    File(PathBuf),
    /// A material of the document.
    Mat(varstotas_kernel::MaterialId),
    /// A materials library file.
    MatFile(PathBuf),
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
    let model = crate::document::guarded(|| varstotas_kernel::io::load(path)).and_then(|r| r.map_err(|e| e.to_string()))?;
    let preview = model.preview.as_ref().map(|p| p.bytes.clone());
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
        let r = Clipboard::from_selection(&self.doc).and_then(|c| c.to_model(self.doc.model.unit));
        self.write_to_library(name, r);
    }

    /// A document's component → `<library>/<its name>.vrst` (In Model → Save to Library).
    pub(crate) fn save_definition_to_library(&mut self, d: varstotas_kernel::DefId) {
        let m = &self.doc.model;
        let name = m.defs.get(d).map(|x| x.name.clone()).unwrap_or_default();
        let r = Clipboard::of_definition(m, self.doc.uid, d).and_then(|c| c.to_model(m.unit));
        self.write_to_library(&name, r);
    }

    fn write_to_library(&mut self, name: &str, model: varstotas_kernel::OpResult<varstotas_kernel::Model>) {
        let Some(dir) = self.library_dir() else { return };
        let model = match model {
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

    /// A section heading of the Components list: In Model (the document's components and
    /// generators; Hide), the library folder ("" — My Library) and its subfolders (the title
    /// shows the folder in the Finder), Examples (bundled; Hide). True when folded.
    fn folder_header(&mut self, ui: &mut Ui, folder: &str, root: Option<&Path>, count: usize) -> bool {
        use super::generators::{DOCUMENT_FOLDER, EXAMPLES_FOLDER};
        let dir = root.map(|r| if folder.is_empty() { r.to_path_buf() } else { r.join(folder) });
        let shown = |d: &Option<PathBuf>| d.as_ref().map(|d| format!("{}\nClick to show in the Finder", d.display())).unwrap_or_default();
        let (title, tip, link) = match folder {
            EXAMPLES_FOLDER => (folder.to_string(), "Generators bundled with Varstotas (not files)".to_string(), false),
            DOCUMENT_FOLDER => ("In Model".to_string(), "Components and generators of the open document (Save to Library makes a file)".to_string(), false),
            "" => ("My Library".to_string(), shown(&dir), dir.is_some()),
            f => (f.to_string(), shown(&dir), dir.is_some()),
        };
        let mut hide = None;
        let (folded, clicked) = super::tray_lib::section(ui, &mut self.prefs.library_collapsed, folder, &title, &tip, link, false, Some(count), |ui| {
            if folder == EXAMPLES_FOLDER || folder == DOCUMENT_FOLDER {
                let tip = if folder == EXAMPLES_FOLDER { "Hide the bundled examples (… → Show Examples brings them back)" } else { "Hide the document's components (… → Show In Model brings them back)" };
                let r = ui.small_button("Hide").on_hover_text(tip);
                super::tray_lib::spot(format!("hide:{folder}"), r.rect.center());
                if r.clicked() {
                    hide = Some(folder == EXAMPLES_FOLDER);
                }
            }
        });
        match hide {
            Some(true) => self.prefs.hide_examples = true,
            Some(false) => self.prefs.hide_document = true,
            None => {}
        }
        if clicked && let Some(d) = dir {
            crate::ui::extensions::reveal(&d);
        }
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
            ui.heading(if matches!(target, RenameTarget::Mat(_) | RenameTarget::MatFile(_)) { "Rename Material" } else { "Rename Component" });
            ui.add_space(4.0);
            if let RenameTarget::File(path) | RenameTarget::MatFile(path) = &target {
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
            RenameTarget::Mat(id) => self.rename_material(*id, name),
            RenameTarget::MatFile(path) => self.rename_material_file(path, name),
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
    fn def_tile(&mut self, ui: &mut Ui, tiles: super::tray_lib::TileSize, d: varstotas_kernel::DefId) -> egui::Response {
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
        // Its own picture, when it has one.
        let own = match m.defs.get(d).and_then(|x| x.preview.as_ref()) {
            Some(img) => {
                let key = std::sync::Arc::as_ptr(&img.bytes) as *const u8 as usize;
                if self.library.def_pics.get(&d).is_none_or(|p| p.0 != key)
                    && let Some(ci) = crate::textures::thumbnail(&img.bytes, 2 * TILE as u32)
                {
                    let tex = ui.ctx().load_texture(format!("defpreview:{d:?}"), ci, egui::TextureOptions::LINEAR);
                    self.library.def_pics.insert(d, (key, tex));
                }
                self.library.def_pics.get(&d).map(|p| p.1.id())
            }
            None => {
                self.library.def_pics.remove(&d);
                None
            }
        };
        let name = m.defs.get(d).map(|x| x.name.clone()).unwrap_or_default();
        let uses = m.defs.values().flat_map(|x| x.mesh.instances.values()).filter(|i| i.def == d).count();
        let t = &self.library.def_thumbs[&d];
        let note = format!("In this document: {uses} placed");
        tile_ex(ui, tiles, &name, &t.thumb, own.or(t.pic), t.size, m.unit, false, &note)
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
    pub(crate) fn preview_from_view(&mut self, target: PreviewTarget) {
        let vp = self.viewport.size();
        let side = crate::textures::PREVIEW_SIDE as f32;
        let k = side / vp.x.max(vp.y).max(1.0);
        let size = [(vp.x * k).round().max(1.0) as u32, (vp.y * k).round().max(1.0) as u32];
        let path = std::env::temp_dir().join(format!("varstotas_preview_{}.png", std::process::id()));
        let content = crate::app::ExportContent { transparent: false, axes: false, guides: false, dims: false };
        self.preview_shot = Some((path.clone(), target));
        self.export_png_to(path, size, content);
    }

    /// True when `r` is the render of [`Self::preview_from_view`] (then it becomes the preview).
    pub(crate) fn preview_shot_done(&mut self, r: &Result<PathBuf, String>) -> bool {
        let Some((path, target)) = self.preview_shot.take() else { return false };
        match r {
            Ok(p) if *p == path => {
                let res = std::fs::read(p).map_err(|e| e.to_string()).and_then(|b| crate::textures::preview_bytes(&b));
                let _ = std::fs::remove_file(p);
                match res {
                    Ok(b) => self.set_preview_of(&target, Some(b)),
                    Err(e) => self.flash_error(format!("Preview image: {e}")),
                }
                true
            }
            Ok(_) => {
                self.preview_shot = Some((path, target));
                false
            }
            Err(e) => {
                self.flash_error(format!("Preview image: {e}"));
                true
            }
        }
    }

    /// Set (`bytes`: a PNG / JPEG) or remove the preview picture of `target`: the document
    /// (undoable), a library file (written at once) or a component of the document (undoable).
    pub(crate) fn set_preview_of(&mut self, target: &PreviewTarget, bytes: Option<Vec<u8>>) {
        match target {
            PreviewTarget::Document => self.set_doc_preview(bytes),
            PreviewTarget::File(path) => self.set_library_preview(path, bytes),
            PreviewTarget::Def(d) => {
                let (d, set) = (*d, bytes.is_some());
                let name = self.doc.model.defs.get(d).map(|x| x.name.clone()).unwrap_or_default();
                self.doc.commit(if set { "Set Component Preview" } else { "Remove Component Preview" }, |m| {
                    if let Some(def) = m.defs.get_mut(d) {
                        def.preview = bytes.map(|b| varstotas_kernel::texture::TextureImage::new(b, "preview", 0.0));
                    }
                });
                self.flash(if set { format!("Preview image of {name} set (saved with the document)") } else { format!("Preview image of {name} removed") });
            }
        }
    }

    /// Set (from the image file `image`) or remove the preview picture of `target`.
    pub(crate) fn preview_of_from_file(&mut self, target: &PreviewTarget, image: Option<&Path>) {
        let bytes = match image {
            Some(f) => match std::fs::read(f).map_err(|e| format!("{}: {e}", f.display())).and_then(|b| crate::textures::preview_bytes(&b)) {
                Ok(b) => Some(b),
                Err(e) => return self.flash_error(format!("Preview image: {e}")),
            },
            None => None,
        };
        self.set_preview_of(target, bytes);
    }

    /// Set or remove the preview picture of library file `path` (the file is rewritten).
    pub(crate) fn set_library_preview(&mut self, path: &Path, bytes: Option<Vec<u8>>) {
        let res = (|| -> Result<(), String> {
            let mut m = varstotas_kernel::io::load(path).map_err(|e| e.to_string())?;
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
        // The top row: search, Refresh, + (save the selection, new generator), 3D Library, …
        let can_save = !self.doc.selection.is_empty();
        let custom = self.prefs.library_dir.is_some();
        let (mut save, mut new_gen, mut choose, mut default_dir, mut reveal_root) = (false, false, false, false, false);
        let (mut show_ex, mut show_doc) = (!self.prefs.hide_examples, !self.prefs.hide_document);
        let top = super::tray_lib::top_row(
            ui,
            "",
            &mut self.library.filter,
            &mut self.prefs.lib_tiles,
            "Save the selection to the library, new generator",
            "3D Library: free models from the web (Poly Haven, Smithsonian, NASA, Kenney, plugins)",
            |ui| {
                if ui.add_enabled(can_save, egui::Button::new("Save Selection to Library…")).on_disabled_hover_text("Select something first").clicked() {
                    save = true;
                    ui.close();
                }
                if ui.button("New Generator…").on_hover_text("A component built by a script with parameters (Window → Generator Editor)").clicked() {
                    new_gen = true;
                    ui.close();
                }
            },
            |ui| {
                if ui.button("Choose Folder…").clicked() {
                    choose = true;
                    ui.close();
                }
                if custom && ui.button("Default Folder").on_hover_text("Back to ~/.varstotas/components").clicked() {
                    default_dir = true;
                    ui.close();
                }
                if ui.button("Show in Finder").clicked() {
                    reveal_root = true;
                    ui.close();
                }
                ui.separator();
                ui.checkbox(&mut show_ex, "Show Examples").on_hover_text("The bundled example generators");
                ui.checkbox(&mut show_doc, "Show In Model").on_hover_text("The open document's components and generators");
            },
        );
        self.prefs.hide_examples = !show_ex;
        self.prefs.hide_document = !show_doc;
        if top.refresh {
            self.refresh_library();
        }
        if top.online {
            self.models.open = true;
        }
        if save {
            self.library.save_name = Some(self.selection_name());
        }
        if new_gen {
            self.gen_editor.new_generator();
        }
        if choose && let Some(d) = rfd::FileDialog::new().pick_folder() {
            self.prefs.library_dir = Some(d);
        }
        if default_dir {
            self.prefs.library_dir = None;
        }
        if reveal_root && let Some(d) = self.library_dir() {
            crate::ui::extensions::reveal(&d);
        }
        self.save_name_row(ui);
        let filter = self.library.filter.trim().to_lowercase();
        let found = |name: &str| super::tray_lib::found(&filter, name);
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
        // In Model first, then the library folders, then Examples.
        let rank = |f: &str| match f {
            super::generators::DOCUMENT_FOLDER => 0,
            super::generators::EXAMPLES_FOLDER => 2,
            _ => 1,
        };
        shown.sort_by(|a, b| rank(&a.0).cmp(&rank(&b.0)).then_with(|| (&a.0, &a.1).cmp(&(&b.0, &b.1))));
        if self.library.items.is_empty() && gens.is_empty() && doc_defs.is_empty() {
            ui.label(RichText::new("No components yet: select something and use + → Save Selection to Library.").weak());
        } else if shown.is_empty() {
            ui.label(RichText::new("Nothing found").weak());
        }
        let (mut place, mut place_gen, mut edit, mut delete, mut place_def, mut select_def) = (None, None, None, None, None, None);
        let mut delete_def = None;
        let mut rig_def = None;
        let mut save_def = None;
        let mut rename = None;
        let lib_dir = self.library_dir();
        let mut set_preview: Option<(PreviewTarget, bool)> = None;
        let mut preview_view: Option<PreviewTarget> = None;
        let size = self.prefs.lib_tiles;
        self.library.shown_folders.clear();
        let root_count = shown.iter().filter(|t| t.0.is_empty()).count();
        let mut root_folded = None;
        let mut k = 0;
        loop {
            // The library folder's heading: always there, after In Model (its own items fold
            // under it).
            if root_folded.is_none() && (k == shown.len() || rank(&shown[k].0) >= 1) {
                root_folded = Some(self.folder_header(ui, "", lib_dir.as_deref(), root_count));
            }
            if k >= shown.len() {
                break;
            }
            let folder = shown[k].0.clone();
            let end = shown[k..].iter().position(|t| t.0 != folder).map_or(shown.len(), |n| k + n);
            let folded = if folder.is_empty() { root_folded.unwrap_or(false) } else { self.folder_header(ui, &folder, lib_dir.as_deref(), end - k) };
            self.library.shown_folders.push((folder.clone(), end - k, folded));
            if folded {
                k = end;
                continue;
            }
            let group: Vec<Tile> = shown[k..end].iter().map(|t| t.2).collect();
            super::tray_lib::tiles(ui, size, group.len(), |ui, j| match group[j] {
                Tile::Comp(i) => {
                    let it = &self.library.items[i];
                    // The file's picture, else a rendered thumbnail.
                    let pic = it.picture(ui.ctx()).or_else(|| {
                        let key = format!("lib:{}|{:?}", it.path.display(), it.mtime);
                        self.thumbs.get(ui.ctx(), &key, || crate::thumbs::disk_path(&it.path), || it.clip.to_model(unit).ok())
                    });
                    let resp = tile_ex(ui, size, &it.name, &it.thumb, pic, it.size, unit, false, "");
                    if resp.clicked() {
                        place = Some(i);
                    }
                    resp.context_menu(|ui| {
                        if ui.button("Place").clicked() {
                            place = Some(i);
                            ui.close();
                        }
                        if ui.button("Set Preview from View").on_hover_text("The viewport as it is now becomes the picture (saved in the file): frame the model first").clicked() {
                            preview_view = Some(PreviewTarget::File(it.path.clone()));
                            ui.close();
                        }
                        if ui.button("Set Preview Image…").on_hover_text("A picture shown here instead of the drawn model (saved in the file)").clicked() {
                            set_preview = Some((PreviewTarget::File(it.path.clone()), true));
                            ui.close();
                        }
                        if it.preview.is_some() && ui.button("Remove Preview Image").clicked() {
                            set_preview = Some((PreviewTarget::File(it.path.clone()), false));
                            ui.close();
                        }
                        if ui.button("Rename…").clicked() {
                            rename = Some((RenameTarget::File(it.path.clone()), it.name.clone()));
                            ui.close();
                        }
                        if ui.button("Show in Finder").clicked() {
                            crate::ui::extensions::reveal(&it.path);
                            ui.close();
                        }
                        if ui.button("Delete…").clicked() {
                            delete = Some((Some(it.path.clone()), it.name.clone()));
                            ui.close();
                        }
                    });
                }
                Tile::Def(d) => {
                    let resp = self.def_tile(ui, size, d);
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
                        if ui.button("Set Preview from View").on_hover_text("The viewport as it is now becomes the tile's picture (saved with the document): frame the component first").clicked() {
                            preview_view = Some(PreviewTarget::Def(d));
                            ui.close();
                        }
                        if ui.button("Set Preview Image…").on_hover_text("A picture shown on the tile instead of the rendered model").clicked() {
                            set_preview = Some((PreviewTarget::Def(d), true));
                            ui.close();
                        }
                        if self.doc.model.defs.get(d).is_some_and(|x| x.preview.is_some()) && ui.button("Remove Preview Image").clicked() {
                            set_preview = Some((PreviewTarget::Def(d), false));
                            ui.close();
                        }
                        if ui.button("Rename…").clicked() {
                            let name = self.doc.model.defs.get(d).map(|x| x.name.clone()).unwrap_or_default();
                            rename = Some((RenameTarget::Def(d), name));
                            ui.close();
                        }
                        if ui.button("Save to Library").on_hover_text("Write it to the library folder, for any document").clicked() {
                            save_def = Some(d);
                            ui.close();
                        }
                        if ui.button("Parameters…").on_hover_text("Window → Component Parameters: values that move, show or paint its parts").clicked() {
                            rig_def = Some(d);
                            ui.close();
                        }
                        if ui.button("Delete from Document…").on_hover_text("The component and all its placed copies").clicked() {
                            delete_def = Some(d);
                            ui.close();
                        }
                    });
                }
                Tile::Gen(i) => {
                    let (p, e, d) = self.generator_tile(ui, size, &gens[i]);
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
            });
            k = end;
        }
        if delete.is_some() {
            self.library.delete_ask = delete;
        }
        if rename.is_some() {
            self.library.rename = rename;
        }
        if let Some((target, set)) = set_preview {
            let image = if set { rfd::FileDialog::new().add_filter("Image", &["png", "jpg", "jpeg", "tga", "bmp"]).pick_file() } else { None };
            if !set || image.is_some() {
                self.preview_of_from_file(&target, image.as_deref());
            }
        }
        if let Some(target) = preview_view {
            self.preview_from_view(target);
        }
        if let Some(d) = place_def {
            self.place_definition(d);
        }
        if let Some(d) = select_def {
            self.select_definition(d);
        }
        if let Some(d) = save_def {
            self.save_definition_to_library(d);
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
        if let Some(i) = place {
            self.actions.push(Action::PlaceComponent(i));
        }
    }

    /// The name typed for + → Save Selection to Library (Enter or Save writes it).
    fn save_name_row(&mut self, ui: &mut Ui) {
        let Some(name) = self.library.save_name.as_mut() else { return };
        let mut done = None;
        ui.horizontal(|ui| {
            let r = ui.add(egui::TextEdit::singleline(name).hint_text("Name").desired_width(ui.available_width() - 110.0));
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

/// A library tile (`tray_lib::tile`): the picture `image` (fitted) or else the thumbnail edges
/// in the unit square; `gear` marks a generator (a gear in the top right corner).
#[allow(clippy::too_many_arguments)]
pub(crate) fn tile_ex(
    ui: &mut Ui,
    tiles: super::tray_lib::TileSize,
    name: &str,
    thumb: &[(egui::Pos2, egui::Pos2)],
    image: Option<egui::TextureId>,
    size: DVec3,
    unit: varstotas_kernel::LengthUnit,
    gear: bool,
    note: &str,
) -> egui::Response {
    let ctx = ui.ctx().clone();
    let large = tiles == super::tray_lib::TileSize::Large;
    let resp = super::tray_lib::tile(ui, tiles, name, false, |p, pic| {
        if let Some(id) = image {
            super::tray_lib::paint_picture(&ctx, p, pic, id);
        } else {
            let inner = pic.shrink(if large { 8.0 } else { 3.0 });
            let to = |q: egui::Pos2| inner.min + vec2(q.x * inner.width(), q.y * inner.height());
            let stroke = Stroke::new(if large { 0.8 } else { 0.6 }, Color32::from_gray(60));
            for (a, b) in thumb {
                p.line_segment([to(*a), to(*b)], stroke);
            }
        }
        if gear {
            let r = if large { 8.0 } else { 5.0 };
            let c = pos2(pic.max.x - r - 2.0, pic.min.y + r + 2.0);
            p.circle_filled(c, r, Color32::from_rgb(0x3B, 0x6E, 0xC4));
            p.text(c, egui::Align2::CENTER_CENTER, "⚙", egui::FontId::proportional(1.5 * r), Color32::WHITE);
        }
    });
    let f = |v: f64| varstotas_kernel::units::format_length(v, unit);
    let note = if note.is_empty() { String::new() } else { format!("{note}\n") };
    resp.on_hover_text(format!("{name}\n{note}{} × {} × {}\nClick to place", f(size.x), f(size.y), f(size.z)))
}
