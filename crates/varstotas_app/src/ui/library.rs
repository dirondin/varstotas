//! Components: a local library (the side bar card) of `.vrst` files and generators
//! (`.rhai`, components with parameters — see `ui/generators.rs`), in one list. The library is
//! `~/.varstotas/components` (or a folder chosen by the user); subfolders are categories, the
//! bundled example generators show under "Examples". A click on an item places it as a
//! component (one definition per document, however often it is placed); "Save Selection"
//! writes the selection to the library.

use super::tray_lib::{DragItem, FolderAct, FolderAsk, LibMenuAct, TrayDrag, file_key};
use crate::app::{Action, VarstotasApp};
use crate::clipboard::Clipboard;
use crate::files::{COMPONENT, safe_name};
use varstotas_i18n::tr;
use egui::{Color32, RichText, Stroke, Ui, pos2, vec2};
use varstotas_kernel::glam::DVec3;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

/// Edges kept for a thumbnail.
pub(crate) const THUMB_EDGES: usize = 4000;
const TILE: f32 = super::tray_lib::LARGE;
/// Section ids of the favourites and recents (not folder names: a folder can't start with \u{1}).
pub const FAVOURITES: &str = "\u{1}Favourites";
pub const RECENT: &str = "\u{1}Recent";

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
    /// Subfolders of the library folder (empty ones too).
    pub dirs: Vec<String>,
    /// The New Folder / Rename Folder dialog (either library card).
    pub folder_ask: Option<FolderAsk>,
    /// Delete Folder, waiting for the user's yes (either library card).
    pub folder_delete: Option<FolderDelete>,
}

/// A library subfolder to delete once confirmed, and what is in it.
#[derive(Clone, Debug, PartialEq)]
pub struct FolderDelete {
    pub materials: bool,
    pub dir: PathBuf,
    /// Files and subfolders inside (all levels).
    pub files: usize,
    pub dirs: usize,
    /// The first few files' names (relative to the folder).
    pub sample: Vec<String>,
}

impl FolderDelete {
    pub fn new(materials: bool, dir: PathBuf) -> Self {
        let (mut files, mut dirs, mut sample) = (0, 0, Vec::new());
        let mut stack = vec![dir.clone()];
        while let Some(d) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&d) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    dirs += 1;
                    stack.push(p);
                } else {
                    files += 1;
                    if sample.len() < 6 {
                        sample.push(p.strip_prefix(&dir).unwrap_or(&p).display().to_string());
                    }
                }
            }
        }
        sample.sort();
        FolderDelete { materials, dir, files, dirs, sample }
    }

    pub fn is_empty(&self) -> bool {
        self.files + self.dirs == 0
    }
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
    let edges: Vec<_> = crate::clipboard::sample_edges(&clip.preview, THUMB_EDGES);
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

/// Rename the library file `path` to `<name>.vrst` in its folder; a file holding one
/// component (or a component with parameters) named after the file gets the new name too.
/// Returns the new path.
fn rename_library_file(path: &Path, name: &str) -> Result<PathBuf, String> {
    let to = path.with_file_name(format!("{}.vrst", safe_name(name, COMPONENT)));
    if to == path {
        return Ok(to);
    }
    if to.exists() && !to.to_string_lossy().eq_ignore_ascii_case(&path.to_string_lossy()) {
        return Err(tr!("library-exists", path = to.display().to_string()));
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
        self.library.dirs = super::tray_lib::subfolders(&dir);
        self.library.scanned = Some(dir);
        self.refresh_generators();
    }

    pub(crate) fn library_index(&self, name: &str) -> Option<usize> {
        self.library.items.iter().position(|i| i.name.eq_ignore_ascii_case(name.trim()))
    }

    /// Start placing library item `i`.
    pub(crate) fn place_component(&mut self, i: usize) {
        let Some(item) = self.library.items.get(i) else { return };
        let warning = crate::downloads::licence_warning(&item.clip.credits());
        let tool = crate::tools::transform::PasteTool::named(item.clip.clone(), item.name.clone());
        self.install_tool(Box::new(tool));
        // A model whose licence limits its use: say so when it is placed.
        if let Some(w) = warning {
            self.flash(w);
        }
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
        "Component".into() // i18n-ignore
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
            Err(e) => return self.flash_error(tr!("library-save-error", error = e.to_string())),
        };
        let base = safe_name(name, COMPONENT);
        let mut path = dir.join(format!("{base}.vrst"));
        let mut n = 2;
        while path.exists() {
            path = dir.join(format!("{base} {n}.vrst"));
            n += 1;
        }
        let _ = std::fs::create_dir_all(&dir);
        match varstotas_kernel::io::save(&model, &path) {
            Ok(()) => {
                self.flash(tr!("library-saved", path = path.display().to_string()));
                self.refresh_library();
            }
            Err(e) => self.flash_error(tr!("library-save-failed", error = e.to_string())),
        }
    }

    /// A section heading of the Components list: In Model (the document's components and
    /// generators; Hide), Favourites, Recent, the library folder ("" — My Library) and its
    /// subfolders (the title shows the folder in the Finder; New / Rename / Delete Folder in
    /// its context menu), Examples (bundled; Hide). True when folded.
    fn folder_header(&mut self, ui: &mut Ui, folder: &str, root: Option<&Path>, count: usize, act: &mut Option<FolderAct>) -> bool {
        use super::generators::{DOCUMENT_FOLDER, EXAMPLES_FOLDER};
        let special = matches!(folder, DOCUMENT_FOLDER | EXAMPLES_FOLDER | FAVOURITES | RECENT);
        let dir = root.filter(|_| !special).map(|r| if folder.is_empty() { r.to_path_buf() } else { r.join(folder) });
        let shown = |d: &Option<PathBuf>| d.as_ref().map(|d| tr!("library-folder-reveal-hover", path = d.display().to_string())).unwrap_or_default();
        let (title, tip, link) = match folder {
            EXAMPLES_FOLDER => (tr!("library-examples"), tr!("library-examples-hover"), false),
            DOCUMENT_FOLDER => (tr!("library-in-model"), tr!("library-in-model-hover"), false),
            FAVOURITES => (tr!("library-favourites"), tr!("library-favourites-hover"), false),
            RECENT => (tr!("library-recent"), tr!("library-recent-hover"), false),
            "" => (tr!("tray-my-library"), shown(&dir), dir.is_some()),
            f => (f.to_string(), shown(&dir), dir.is_some()),
        };
        let mut hide = None;
        let unused = if folder == DOCUMENT_FOLDER { self.unused_defs() } else { 0 };
        let mut purge = false;
        let (folded, clicked, title) = super::tray_lib::section(ui, &mut self.prefs.library_collapsed, folder, &title, &tip, link, false, Some(count), |ui| {
            if folder == DOCUMENT_FOLDER {
                purge = super::tray_lib::purge_button(ui, "doc", "components", unused);
            }
            if folder == EXAMPLES_FOLDER || folder == DOCUMENT_FOLDER {
                let tip = tr!(if folder == EXAMPLES_FOLDER { "library-hide-examples" } else { "library-hide-in-model" });
                let r = super::icon_button(ui, super::icons::Icon::EyeOff, false, 18.0).on_hover_text(&tip);
                super::tray_lib::spot(format!("hide:{folder}"), r.rect.center());
                if r.clicked() {
                    hide = Some(folder == EXAMPLES_FOLDER);
                }
            }
        });
        super::tray_lib::folder_menu(&title, folder, dir.as_deref(), act);
        if purge {
            self.purge_unused(true, false);
        }
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

    /// A folder heading's menu: New Folder… / Rename Folder… open the folder dialog, Delete
    /// Folder asks first ([`Self::folder_delete_dialog`]; `materials`: of the materials library).
    pub(crate) fn folder_act(&mut self, materials: bool, act: FolderAct) {
        match act {
            FolderAct::New => self.library.folder_ask = Some(FolderAsk { materials, rename: None, name: String::new() }),
            FolderAct::Rename(f) => {
                let name = f.rsplit('/').next().unwrap_or(&f).to_string();
                self.library.folder_ask = Some(FolderAsk { materials, rename: Some(f), name });
            }
            FolderAct::Delete(dir) => self.library.folder_delete = Some(FolderDelete::new(materials, dir)),
        }
    }

    fn refresh_library_of(&mut self, materials: bool) {
        if materials {
            self.refresh_materials();
        } else {
            self.refresh_library();
        }
    }

    /// "Delete Folder?" (`LibraryState::folder_delete`): where it is and, in a warning, what is
    /// in it when it is not empty; Delete removes the folder with everything inside.
    pub(crate) fn folder_delete_dialog(&mut self, ctx: &egui::Context) {
        let Some(del) = self.library.folder_delete.clone() else { return };
        let name = del.dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let mut choice = None;
        let resp = egui::Modal::new(egui::Id::new("library_folder_delete")).show(ctx, |ui| {
            ui.set_width(380.0);
            ui.heading(tr!("library-folder-delete-title"));
            ui.add_space(4.0);
            ui.label(tr!(if del.materials { "library-folder-delete-text-materials" } else { "library-folder-delete-text-components" }, name = name.as_str()));
            ui.label(RichText::new(del.dir.display().to_string()).small().weak());
            ui.add_space(6.0);
            if del.is_empty() {
                ui.label(tr!("library-folder-empty"));
            } else {
                let warn = Color32::from_rgb(200, 50, 40);
                egui::Frame::new()
                    .fill(warn.gamma_multiply(0.12))
                    .stroke(egui::Stroke::new(1.0, warn))
                    .corner_radius(6.0)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let what = if del.dirs > 0 { tr!("library-folder-what-dirs", files = del.files, dirs = del.dirs) } else { tr!("library-folder-what", files = del.files) };
                        ui.label(RichText::new(tr!("library-folder-not-empty", what = what)).strong().color(warn));
                        for f in &del.sample {
                            ui.label(RichText::new(format!("• {f}")).small());
                        }
                        if del.files > del.sample.len() {
                            ui.label(RichText::new(tr!("library-folder-more", n = del.files - del.sample.len())).small().weak());
                        }
                    });
            }
            ui.add_space(4.0);
            ui.label(RichText::new(tr!("library-undone-copies-stay")).weak());
            ui.add_space(10.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = if del.is_empty() { tr!("delete") } else { tr!("library-folder-delete-with", files = del.files) };
                let r = ui.add(egui::Button::new(RichText::new(label).strong().color(Color32::from_rgb(200, 50, 40))));
                super::tray_lib::spot("folder_delete:ok", r.rect.center());
                if r.clicked() {
                    choice = Some(true);
                }
                let r = ui.button(tr!("cancel"));
                super::tray_lib::spot("folder_delete:cancel", r.rect.center());
                if r.clicked() {
                    choice = Some(false);
                }
            });
        });
        if resp.should_close() && choice.is_none() {
            choice = Some(false);
        }
        let Some(ok) = choice else { return };
        self.library.folder_delete = None;
        if !ok {
            return;
        }
        match std::fs::remove_dir_all(&del.dir) {
            Ok(()) => {
                // favourites and recents of what was inside go too
                let (prefix, lists) = if del.materials {
                    ("m:", [&mut self.prefs.mat_favourites, &mut self.prefs.mat_recent])
                } else {
                    ("c:", [&mut self.prefs.lib_favourites, &mut self.prefs.lib_recent])
                };
                let key = super::tray_lib::file_key(prefix, &del.dir);
                for list in lists {
                    list.retain(|k| !k.strip_prefix(&key).is_some_and(|r| r.starts_with(std::path::MAIN_SEPARATOR)));
                }
                self.flash(if del.is_empty() { tr!("library-folder-deleted", name = name.as_str()) } else { tr!("library-folder-deleted-files", name = name.as_str(), files = del.files) });
            }
            Err(e) => self.flash_error(tr!("library-folder-delete-failed", error = e.to_string())),
        }
        self.refresh_library_of(del.materials);
    }

    /// The New Folder / Rename Folder dialog (`LibraryState::folder_ask`).
    pub(crate) fn folder_dialog(&mut self, ctx: &egui::Context) {
        let Some(ask) = self.library.folder_ask.as_mut() else { return };
        let mut choice = None;
        let heading = tr!(if ask.rename.is_some() { "library-folder-rename-title" } else { "library-folder-new-title" });
        let resp = egui::Modal::new(egui::Id::new("library_folder")).show(ctx, |ui| {
            ui.set_width(320.0);
            ui.heading(&heading);
            ui.add_space(4.0);
            let r = ui.add(egui::TextEdit::singleline(&mut ask.name).hint_text(tr!("library-folder-name")).desired_width(f32::INFINITY));
            super::tray_lib::spot("folder_name", r.rect.center());
            if !r.has_focus() && !r.lost_focus() {
                r.request_focus();
            }
            let ok = super::tray_lib::folder_name_ok(&ask.name);
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && ok {
                choice = Some(true);
            }
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add_enabled(ok, egui::Button::new(tr!(if ask.rename.is_some() { "library-rename" } else { "library-create" }))).clicked() {
                    choice = Some(true);
                }
                if ui.button(tr!("cancel")).clicked() {
                    choice = Some(false);
                }
            });
        });
        if resp.should_close() && choice.is_none() {
            choice = Some(false);
        }
        let Some(ok) = choice else { return };
        let ask = self.library.folder_ask.take().expect("checked above");
        if !ok {
            return;
        }
        let root = if ask.materials { self.material_library_dir() } else { self.library_dir() };
        let Some(root) = root else { return };
        let name = ask.name.trim().to_string();
        let r = match &ask.rename {
            None => std::fs::create_dir_all(root.join(&name)).map_err(|e| e.to_string()),
            Some(old) => {
                let from = root.join(old);
                let to = from.parent().unwrap_or(&root).join(&name);
                if to.exists() {
                    Err(tr!("library-exists", path = to.display().to_string()))
                } else {
                    std::fs::rename(&from, &to).map_err(|e| e.to_string()).map(|()| {
                        let (prefix, lists) = if ask.materials {
                            ("m:", [&mut self.prefs.mat_favourites, &mut self.prefs.mat_recent])
                        } else {
                            ("c:", [&mut self.prefs.lib_favourites, &mut self.prefs.lib_recent])
                        };
                        super::tray_lib::rekey(lists, prefix, &from, &to);
                        // the fold state follows
                        let new_rel = to.strip_prefix(&root).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or(name.clone());
                        let tag = if ask.materials { "mat:" } else { "" };
                        for k in self.prefs.library_collapsed.iter_mut() {
                            if *k == format!("{tag}{old}") {
                                *k = format!("{tag}{new_rel}");
                            }
                        }
                    })
                }
            }
        };
        match r {
            Ok(()) => self.flash(tr!("library-folder-done", heading = heading.as_str(), name = name.as_str())),
            Err(e) => self.flash_error(tr!("library-folder-failed", heading = heading.as_str(), error = e)),
        }
        self.refresh_library_of(ask.materials);
    }

    /// "Delete from the library?" for the file chosen in a tile's context menu.
    pub(crate) fn library_delete_dialog(&mut self, ctx: &egui::Context) {
        let Some((path, name)) = self.library.delete_ask.clone() else { return };
        let mut choice = None;
        let resp = egui::Modal::new(egui::Id::new("library_delete")).show(ctx, |ui| {
            ui.set_width(360.0);
            match &path {
                Some(path) => {
                    ui.heading(tr!("library-delete-title"));
                    ui.add_space(4.0);
                    ui.label(tr!("library-delete-text", name = name.as_str()));
                    ui.label(RichText::new(path.display().to_string()).small().weak());
                    ui.label(RichText::new(tr!("library-undone-copies-stay")).weak());
                }
                None => {
                    ui.heading(tr!("library-delete-doc-title"));
                    ui.add_space(4.0);
                    ui.label(tr!("library-delete-gen-text", name = name.as_str()));
                    let uses = self.doc.model.generator_uses(&name);
                    if uses > 0 {
                        ui.label(RichText::new(tr!("library-delete-gen-uses", uses = uses)).weak());
                    }
                    ui.label(RichText::new(tr!("library-undo-brings-back")).weak());
                }
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new(tr!("delete")).strong().color(Color32::from_rgb(200, 50, 40)))).clicked() {
                        choice = Some(true);
                    }
                    if ui.button(tr!("cancel")).clicked() {
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
                if let Some(Some(n)) = self.doc.commit(&tr!("library-undo-delete-generator", name = name.as_str()), |m| m.remove_generator(&name)) {
                    self.flash(if n > 0 { tr!("library-generator-removed-uses", name = name.as_str(), n = n) } else { tr!("library-generator-removed", name = name.as_str()) });
                }
                self.library.gens.thumbs.clear();
            }
            Some(true) => {
                self.library.delete_ask = None;
                let path = path.unwrap_or_default();
                match std::fs::remove_file(&path) {
                    Ok(()) => {
                        self.flash(tr!("library-deleted", name = name.as_str()));
                        self.refresh_library();
                    }
                    Err(e) => self.flash_error(tr!("library-delete-failed", path = path.display().to_string(), error = e.to_string())),
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
        if let Some(n) = self.doc.try_commit(&tr!("library-undo-delete-component", name = name.as_str()), |m| m.delete_definition(d)) {
            self.library.def_thumbs.remove(&d);
            self.flash(if n > 0 { tr!("library-component-deleted-copies", name = name.as_str(), n = n) } else { tr!("library-component-deleted", name = name.as_str()) });
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
            ui.heading(tr!("library-delete-doc-title"));
            ui.add_space(4.0);
            ui.label(tr!("library-delete-def-text", name = name.as_str(), uses = uses));
            ui.label(RichText::new(tr!("library-undo-brings-them-back")).weak());
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new(tr!("delete")).strong().color(Color32::from_rgb(200, 50, 40)))).clicked() {
                        choice = Some(true);
                    }
                    if ui.button(tr!("cancel")).clicked() {
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
            ui.heading(tr!(if matches!(target, RenameTarget::Mat(_) | RenameTarget::MatFile(_)) { "library-rename-material" } else { "library-rename-component" }));
            ui.add_space(4.0);
            if let RenameTarget::File(path) | RenameTarget::MatFile(path) = &target {
                let file = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                ui.label(RichText::new(tr!("library-rename-file-too", file = file.as_str())).small().weak()).on_hover_text(path.display().to_string());
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
                    if ui.add_enabled(!text.trim().is_empty(), egui::Button::new(RichText::new(tr!("library-rename")).strong())).clicked() {
                        choice = Some(true);
                    }
                    if ui.button(tr!("cancel")).clicked() {
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
                if self.doc.try_commit(&tr!("library-undo-rename-component", name = name), |m| m.rename_definition(d, name)).is_some() {
                    self.flash(tr!("library-renamed", name = name));
                }
            }
            RenameTarget::File(path) => match rename_library_file(path, name) {
                Ok(to) => {
                    self.flash(tr!("library-renamed", name = to.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()));
                    self.refresh_library();
                }
                Err(e) => self.flash_error(tr!("library-rename-failed", path = path.display().to_string(), error = e)),
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
            .map(|(id, d)| (id, if d.name.is_empty() { "Component".to_string() } else { d.name.clone() })) // i18n-ignore
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
        let note = tr!("library-in-document", uses = uses);
        tile_ex(ui, tiles, &name, &t.thumb, own.or(t.pic), t.size, m.unit, false, &note)
    }

    /// Set or remove the open document's preview picture (undoable; saved with the file).
    pub(crate) fn set_doc_preview(&mut self, bytes: Option<Vec<u8>>) {
        let set = bytes.is_some();
        self.doc.commit(&tr!(if set { "library-undo-set-preview" } else { "library-undo-remove-preview" }), |m| m.set_preview(bytes));
        self.flash(tr!(if set { "library-preview-set-file" } else { "library-preview-removed" }));
    }

    /// The document's preview picture from an image file.
    pub(crate) fn preview_from_file(&mut self, f: &Path) {
        match std::fs::read(f).map_err(|e| e.to_string()).and_then(|b| crate::textures::preview_bytes(&b)) {
            Ok(b) => self.set_doc_preview(Some(b)),
            Err(e) => self.flash_error(tr!("library-preview-error-file", path = f.display().to_string(), error = e)),
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
                    Err(e) => self.flash_error(tr!("library-preview-error", error = e)),
                }
                true
            }
            Ok(_) => {
                self.preview_shot = Some((path, target));
                false
            }
            Err(e) => {
                self.flash_error(tr!("library-preview-error", error = e));
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
                self.doc.commit(&tr!(if set { "library-undo-set-def-preview" } else { "library-undo-remove-def-preview" }), |m| {
                    if let Some(def) = m.defs.get_mut(d) {
                        def.preview = bytes.map(|b| varstotas_kernel::texture::TextureImage::new(b, "preview", 0.0));
                    }
                });
                self.flash(if set { tr!("library-preview-set-def", name = name.as_str()) } else { tr!("library-preview-removed-def", name = name.as_str()) });
            }
        }
    }

    /// Set (from the image file `image`) or remove the preview picture of `target`.
    pub(crate) fn preview_of_from_file(&mut self, target: &PreviewTarget, image: Option<&Path>) {
        let bytes = match image {
            Some(f) => match std::fs::read(f).map_err(|e| format!("{}: {e}", f.display())).and_then(|b| crate::textures::preview_bytes(&b)) {
                Ok(b) => Some(b),
                Err(e) => return self.flash_error(tr!("library-preview-error", error = e)),
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
                self.flash_error(tr!("library-preview-error", error = e));
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
            Err(e) => self.flash_error(tr!("library-place-failed", name = name.as_str(), error = e.to_string())),
        }
    }

    /// Select the instances of `d` placed at the top level of the model.
    fn select_definition(&mut self, d: varstotas_kernel::DefId) {
        let m = &self.doc.model;
        let ids: Vec<_> = m.root_mesh().instances.iter().filter(|(_, i)| i.def == d || m.rig_master(i.def) == Some(d)).map(|(id, _)| varstotas_kernel::EntityRef::Instance(id)).collect();
        if ids.is_empty() {
            self.flash(tr!("library-no-copies"));
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
        let unused = self.unused_defs();
        let mut purge = false;
        let top = super::tray_lib::top_row(
            ui,
            "",
            &mut self.library.filter,
            &mut self.prefs.lib_tiles,
            &tr!("library-plus-hover"),
            &tr!("library-online-hover"),
            |ui| {
                if ui.add_enabled(can_save, egui::Button::new(tr!("library-save-selection"))).on_disabled_hover_text(tr!("library-select-first")).clicked() {
                    save = true;
                    ui.close();
                }
                if ui.button(tr!("library-new-generator")).on_hover_text(tr!("library-new-generator-hover")).clicked() {
                    new_gen = true;
                    ui.close();
                }
            },
            |ui| {
                if ui.button(tr!("library-choose-folder")).clicked() {
                    choose = true;
                    ui.close();
                }
                if custom && ui.button(tr!("library-default-folder")).on_hover_text(tr!("library-default-folder-hover")).clicked() {
                    default_dir = true;
                    ui.close();
                }
                if ui.button(tr!("library-show-in-finder")).clicked() {
                    reveal_root = true;
                    ui.close();
                }
                if ui.add_enabled(unused > 0, egui::Button::new(tr!("library-purge"))).on_hover_text(tr!("library-purge-hover")).clicked() {
                    purge = true;
                    ui.close();
                }
                ui.separator();
                if crate::debug_capture::qa_examples_dir().is_some() {
                    ui.checkbox(&mut show_ex, tr!("library-show-examples")).on_hover_text(tr!("library-show-examples-hover"));
                }
                ui.checkbox(&mut show_doc, tr!("library-show-in-model")).on_hover_text(tr!("library-show-in-model-hover"));
            },
        );
        self.prefs.hide_examples = !show_ex;
        self.prefs.hide_document = !show_doc;
        if purge {
            self.purge_unused(true, false);
        }
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
        use super::generators::{DOCUMENT_FOLDER, EXAMPLES_FOLDER};
        // Components and generators in one list: by folder, then name.
        let gens = self.generator_items();
        let doc_defs = if self.prefs.hide_document { Vec::new() } else { self.document_components() };
        let is_doc = |g: &super::generators::GenItem| g.file.is_none() && !g.bundled && g.folder == DOCUMENT_FOLDER;
        // The Draw → Shapes generators (box, cylinder …) are tools, not components to list.
        let shape = |g: &super::generators::GenItem| is_doc(g) && crate::tools::primitives::SOURCES.iter().any(|(_, _, s)| *s == g.source);
        let mut all: Vec<(String, String, Tile)> = self
            .library
            .items
            .iter()
            .enumerate()
            .filter(|(_, it)| found(&it.name))
            .map(|(i, it)| (it.folder.clone(), it.name.to_lowercase(), Tile::Comp(i)))
            .chain(gens.iter().enumerate().filter(|(_, g)| found(&g.name) && !(is_doc(g) && self.prefs.hide_document) && !shape(g)).map(|(i, g)| (g.folder.clone(), g.name.to_lowercase(), Tile::Gen(i))))
            .chain(doc_defs.iter().filter(|(_, n)| found(n)).map(|(d, n)| (DOCUMENT_FOLDER.to_string(), n.to_lowercase(), Tile::Def(*d))))
            .collect();
        all.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
        // Favourites / recents key: a library file, or a bundled example generator by name.
        let key_of = |t: Tile| -> Option<String> {
            match t {
                Tile::Comp(i) => Some(file_key("c:", &self.library.items[i].path)),
                Tile::Gen(i) => match (&gens[i].file, gens[i].bundled) {
                    (Some(f), _) => Some(file_key("c:", f)),
                    (None, true) => Some(format!("g:{}", gens[i].name)),
                    _ => None,
                },
                Tile::Def(_) => None,
            }
        };
        let keyed: Vec<(Option<String>, Tile)> = all.iter().map(|t| (key_of(t.2), t.2)).collect();
        let by_keys = |keys: &[String]| -> Vec<Tile> { keys.iter().filter_map(|k| keyed.iter().find(|(x, _)| x.as_deref() == Some(k.as_str())).map(|t| t.1)).collect() };
        let in_folder = |f: &str| -> Vec<Tile> { all.iter().filter(|t| t.0 == f).map(|t| t.2).collect() };
        let mut dirs = self.library.dirs.clone();
        dirs.extend(all.iter().map(|t| t.0.clone()).filter(|f| !f.is_empty() && f != DOCUMENT_FOLDER && f != EXAMPLES_FOLDER));
        dirs.sort_by_key(|a| a.to_lowercase());
        dirs.dedup();
        // In Model, Favourites, Recent, My Library, its subfolders, Examples: (folder, tiles,
        // listed for the harness)
        let mut secs: Vec<(String, Vec<Tile>, bool)> = vec![
            (DOCUMENT_FOLDER.to_string(), in_folder(DOCUMENT_FOLDER), true),
            (FAVOURITES.to_string(), by_keys(&self.prefs.lib_favourites), false),
            (RECENT.to_string(), by_keys(&self.prefs.lib_recent), false),
            (String::new(), in_folder(""), true),
        ];
        secs.extend(dirs.iter().map(|f| (f.clone(), in_folder(f), true)));
        if !self.prefs.hide_examples {
            secs.push((EXAMPLES_FOLDER.to_string(), in_folder(EXAMPLES_FOLDER), true));
        }
        if all.is_empty() && filter.is_empty() {
            ui.label(RichText::new(tr!("library-empty")).weak());
        } else if all.is_empty() {
            ui.label(RichText::new(tr!("library-nothing-found")).weak());
        }
        let (mut place, mut place_gen, mut edit, mut delete, mut place_def, mut select_def) = (None, None, None, None, None, None);
        let mut delete_def = None;
        let mut rig_def = None;
        let mut save_def = None;
        let mut rename = None;
        let mut recent = None;
        let mut drag: Option<TrayDrag> = None;
        let mut lib_act: Option<(String, Option<PathBuf>, LibMenuAct)> = None;
        let mut folder_act: Option<FolderAct> = None;
        let lib_dir = self.library_dir();
        let mut set_preview: Option<(PreviewTarget, bool)> = None;
        let mut preview_view: Option<PreviewTarget> = None;
        let size = self.prefs.lib_tiles;
        let favs = self.prefs.lib_favourites.clone();
        self.library.shown_folders.clear();
        // In Model stays while it has something to purge (unused group definitions show no tiles)
        let purgeable = !self.prefs.hide_document && filter.is_empty() && self.unused_defs() > 0;
        for (folder, group, listed) in secs {
            let fixed = folder.is_empty() || dirs.contains(&folder) || (purgeable && folder == DOCUMENT_FOLDER);
            if group.is_empty() && !(fixed && (filter.is_empty() || folder.is_empty())) {
                continue;
            }
            let folded = self.folder_header(ui, &folder, lib_dir.as_deref(), group.len(), &mut folder_act);
            if listed && !group.is_empty() {
                self.library.shown_folders.push((folder.clone(), group.len(), folded));
            }
            if folded {
                continue;
            }
            super::tray_lib::tiles(ui, size, group.len(), |ui, j| {
                let key = keyed.iter().find(|t| matches!((t.1, group[j]), (Tile::Comp(a), Tile::Comp(b)) | (Tile::Gen(a), Tile::Gen(b)) if a == b)).and_then(|t| t.0.clone());
                let fav = key.as_ref().is_some_and(|k| favs.contains(k));
                match group[j] {
                    Tile::Comp(i) => {
                        let it = &self.library.items[i];
                        // The file's picture, else a rendered thumbnail.
                        let pic = it.picture(ui.ctx()).or_else(|| {
                            let key = format!("lib:{}|{:?}", it.path.display(), it.mtime);
                            self.thumbs.get(ui.ctx(), &key, || crate::thumbs::disk_path(&it.path), || it.clip.to_model(unit).ok())
                        });
                        let resp = tile_ex(ui, size, &it.name, &it.thumb, pic, it.size, unit, false, "");
                        if fav {
                            super::tray_lib::mark_favourite(ui, size, &resp);
                        }
                        if resp.clicked() {
                            place = Some(i);
                            recent = key.clone();
                        }
                        if resp.drag_started() {
                            drag = Some(TrayDrag { item: DragItem::Comp(it.path.clone()), name: it.name.clone(), recent: key.clone().map(|k| (false, k)), entered: false });
                        }
                        resp.context_menu(|ui| {
                            if ui.button(tr!("library-place")).clicked() {
                                place = Some(i);
                                recent = key.clone();
                                ui.close();
                            }
                            if ui.button(tr!("library-preview-from-view")).on_hover_text(tr!("library-preview-from-view-file-hover")).clicked() {
                                preview_view = Some(PreviewTarget::File(it.path.clone()));
                                ui.close();
                            }
                            if ui.button(tr!("library-preview-image")).on_hover_text(tr!("library-preview-image-file-hover")).clicked() {
                                set_preview = Some((PreviewTarget::File(it.path.clone()), true));
                                ui.close();
                            }
                            if it.preview.is_some() && ui.button(tr!("library-undo-remove-preview")).clicked() {
                                set_preview = Some((PreviewTarget::File(it.path.clone()), false));
                                ui.close();
                            }
                            if ui.button(tr!("library-rename-ellipsis")).clicked() {
                                rename = Some((RenameTarget::File(it.path.clone()), it.name.clone()));
                                ui.close();
                            }
                            if ui.button(tr!("library-show-in-finder")).clicked() {
                                crate::ui::extensions::reveal(&it.path);
                                ui.close();
                            }
                            let mut a = None;
                            super::tray_lib::lib_menu(ui, Some(fav), Some((dirs.as_slice(), it.folder.as_str())), &mut a);
                            if let (Some(a), Some(k)) = (a, &key) {
                                lib_act = Some((k.clone(), Some(it.path.clone()), a));
                            }
                            if ui.button(tr!("library-delete-ellipsis")).clicked() {
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
                        if resp.drag_started() {
                            let name = self.doc.model.defs.get(d).map(|x| x.name.clone()).unwrap_or_default();
                            drag = Some(TrayDrag { item: DragItem::Def(d), name, recent: None, entered: false });
                        }
                        resp.context_menu(|ui| {
                            if ui.button(tr!("library-place")).clicked() {
                                place_def = Some(d);
                                ui.close();
                            }
                            if ui.button(tr!("library-select-instances")).clicked() {
                                select_def = Some(d);
                                ui.close();
                            }
                            if ui.button(tr!("library-preview-from-view")).on_hover_text(tr!("library-preview-from-view-def-hover")).clicked() {
                                preview_view = Some(PreviewTarget::Def(d));
                                ui.close();
                            }
                            if ui.button(tr!("library-preview-image")).on_hover_text(tr!("library-preview-image-def-hover")).clicked() {
                                set_preview = Some((PreviewTarget::Def(d), true));
                                ui.close();
                            }
                            if self.doc.model.defs.get(d).is_some_and(|x| x.preview.is_some()) && ui.button(tr!("library-undo-remove-preview")).clicked() {
                                set_preview = Some((PreviewTarget::Def(d), false));
                                ui.close();
                            }
                            if ui.button(tr!("library-rename-ellipsis")).clicked() {
                                let name = self.doc.model.defs.get(d).map(|x| x.name.clone()).unwrap_or_default();
                                rename = Some((RenameTarget::Def(d), name));
                                ui.close();
                            }
                            if ui.button(tr!("library-save-to-library")).on_hover_text(tr!("library-save-to-library-hover")).clicked() {
                                save_def = Some(d);
                                ui.close();
                            }
                            if ui.button(tr!("library-parameters")).on_hover_text(tr!("library-parameters-hover")).clicked() {
                                rig_def = Some(d);
                                ui.close();
                            }
                            if ui.button(tr!("library-delete-from-document")).on_hover_text(tr!("library-delete-from-document-hover")).clicked() {
                                delete_def = Some(d);
                                ui.close();
                            }
                        });
                    }
                    Tile::Gen(i) => {
                        let g = &gens[i];
                        let folders = g.file.as_ref().map(|_| (dirs.as_slice(), g.folder.as_str()));
                        let t = self.generator_tile(ui, size, g, key.as_ref().map(|_| fav), folders);
                        if fav {
                            super::tray_lib::mark_favourite(ui, size, &t.resp);
                        }
                        if t.delete {
                            delete = Some((g.file.clone(), g.name.clone()));
                        }
                        if t.place {
                            place_gen = Some(g.source.clone());
                            recent = key.clone();
                        }
                        if t.edit {
                            edit = Some(i);
                        }
                        if t.resp.drag_started() {
                            drag = Some(TrayDrag { item: DragItem::Gen(g.source.clone()), name: g.name.clone(), recent: key.clone().map(|k| (false, k)), entered: false });
                        }
                        if let (Some(a), Some(k)) = (t.lib, &key) {
                            lib_act = Some((k.clone(), g.file.clone(), a));
                        }
                    }
                }
            });
        }
        if let Some(k) = recent {
            super::tray_lib::note_recent(&mut self.prefs.lib_recent, &k);
        }
        if drag.is_some() {
            self.tray_drag = drag;
        }
        if let Some(a) = folder_act {
            self.folder_act(false, a);
        }
        match lib_act {
            Some((k, _, LibMenuAct::Favourite)) => super::tray_lib::toggle(&mut self.prefs.lib_favourites, &k),
            Some((_, Some(file), LibMenuAct::Move(f))) => {
                if let Some(root) = self.library_dir() {
                    match super::tray_lib::move_file(&file, &root, &f) {
                        Ok(to) => {
                            super::tray_lib::rekey([&mut self.prefs.lib_favourites, &mut self.prefs.lib_recent], "c:", &file, &to);
                            self.refresh_library();
                        }
                        Err(e) => self.flash_error(tr!("library-move-failed", error = e)),
                    }
                }
            }
            _ => {}
        }
        if delete.is_some() {
            self.library.delete_ask = delete;
        }
        if rename.is_some() {
            self.library.rename = rename;
        }
        if let Some((target, set)) = set_preview {
            let image = if set { rfd::FileDialog::new().add_filter(tr!("library-image-filter"), &["png", "jpg", "jpeg", "tga", "bmp"]).pick_file() } else { None };
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
            match gens[i].folder == super::generators::DOCUMENT_FOLDER && gens[i].file.is_none() && !gens[i].bundled {
                true => self.gen_editor.edit_doc(&gens[i].source, &gens[i].name),
                false => self.gen_editor.edit(&gens[i].source, gens[i].file.clone()),
            }
        }
        if !self.library.errors.is_empty() {
            ui.label(RichText::new(tr!("library-unreadable-files", n = self.library.errors.len())).color(Color32::from_rgb(200, 50, 40)))
                .on_hover_text(self.library.errors.join("\n"));
        }
        if !self.library.gens.errors.is_empty() {
            ui.label(RichText::new(tr!("library-unreadable-generators", n = self.library.gens.errors.len())).color(Color32::from_rgb(200, 50, 40)))
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
            let r = ui.add(egui::TextEdit::singleline(name).hint_text(tr!("library-name")).desired_width(ui.available_width() - 110.0));
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                done = Some(true);
            }
            if ui.button(tr!("library-save")).clicked() {
                done = Some(true);
            }
            if ui.button(tr!("cancel")).clicked() {
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
            let stroke = Stroke::new(if large { 0.8 } else { 0.6 }, super::theme::pick(Color32::from_gray(60), Color32::from_gray(190)));
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
    resp.on_hover_text(tr!("library-tile-hover", name = name, note = note, x = f(size.x), y = f(size.y), z = f(size.z)))
}

impl VarstotasApp {
    /// A tile dragged from a library card (`tray_drag`, after the viewport's input). Over the
    /// viewport its tool takes over — placing the component (it follows the cursor) or Paint
    /// with the material — and the release clicks there: the component is put down, the face
    /// or object under the cursor painted (a library material joins the model with that
    /// paint). Released elsewhere nothing happens (a placement the drag started is cancelled).
    pub(crate) fn tray_drag_frame(&mut self, ctx: &egui::Context, view: &egui::Response) {
        use crate::tools::ToolId;
        let Some(mut d) = self.tray_drag.take() else { return };
        let pos = ctx.input(|i| i.pointer.latest_pos());
        let hud = self.hud_rect.expand(2.0);
        let over = view.contains_pointer() && pos.is_some_and(|p| view.rect.contains(p) && !hud.contains(p));
        if over && !d.entered {
            d.entered = true;
            match &d.item {
                DragItem::Comp(p) => {
                    if let Some(i) = self.library.items.iter().position(|x| &x.path == p) {
                        self.place_component(i);
                    }
                }
                DragItem::Def(def) => self.place_definition(*def),
                DragItem::Gen(src) => self.place_generator(src, &Default::default()),
                DragItem::Mat(m, library) => {
                    self.doc.current_material = Some(if *library { self.palette_for(m) } else { crate::document::PaletteMaterial::of(m) });
                    if self.tool_id() != ToolId::Paint {
                        self.set_tool(ToolId::Paint);
                    }
                }
            }
        }
        let released = ctx.input(|i| i.pointer.primary_released() || !i.pointer.primary_down());
        if !released {
            if !over && let Some(p) = pos {
                // the name at the cursor until the viewport takes over
                egui::Area::new(egui::Id::new("tray_drag_label"))
                    .order(egui::Order::Tooltip)
                    .interactable(false)
                    .fixed_pos(p + vec2(14.0, 10.0))
                    .show(ctx, |ui| {
                        egui::Frame::popup(ui.style()).show(ui, |ui| ui.label(&d.name));
                    });
                ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
            }
            self.tray_drag = Some(d);
            return;
        }
        let is_mat = matches!(d.item, DragItem::Mat(..));
        if over && d.entered && let Some(p) = pos {
            let local = (p - self.viewport.rect.min).to_pos2();
            self.last_pointer = Some(local);
            let pi = self.pointer_info(local, egui::Vec2::ZERO, None, false);
            self.with_tool(pi, |t, c| t.on_pointer_move(c));
            let pi = self.pointer_info(local, egui::Vec2::ZERO, Some(egui::PointerButton::Primary), true);
            self.with_tool(pi, |t, c| t.on_pointer_down(c));
            let pi = self.pointer_info(local, egui::Vec2::ZERO, Some(egui::PointerButton::Primary), false);
            self.with_tool(pi, |t, c| t.on_pointer_up(c));
            match d.recent {
                Some((true, k)) => super::tray_lib::note_recent(&mut self.prefs.mat_recent, &k),
                Some((false, k)) => super::tray_lib::note_recent(&mut self.prefs.lib_recent, &k),
                None => {}
            }
        } else if d.entered && !is_mat {
            self.set_tool(ToolId::Select);
        }
    }
}
