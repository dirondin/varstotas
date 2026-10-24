//! Materials library: `.vrstmat` files (one material each — colour, texture, PBR parameters,
//! maps, shader) in `~/.varstotas/materials`, shared between documents and sessions like the
//! components library; subfolders are categories, the demo gallery's materials show under
//! "Examples". In the Materials card: a click on a library swatch paints with the material (it
//! is added to the document by the first paint, `pick_material`); a model material's context
//! menu saves it to the library, exports it to a file or deletes it from the document;
//! Import… adds a material file to the document. Favourites, Recent and folders as in
//! `tray_lib`.

use crate::app::{Action, VarstotasApp};
use super::tray_lib::{DragItem, FolderAct, LibMenuAct, TrayDrag, file_key};
use crate::document::PaletteMaterial;
use varstotas_i18n::tr;
use egui::{Color32, RichText, Ui};
use varstotas_kernel::io::MATERIAL_EXT;
use varstotas_kernel::{Material, MaterialId};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The folder of the bundled examples (not files).
pub const EXAMPLES: &str = "Examples"; // i18n-ignore

pub struct MatItem {
    /// `None` for a bundled example.
    pub path: Option<PathBuf>,
    /// Subfolder ("" = the library folder itself).
    pub folder: String,
    pub mat: Material,
    mtime: Option<SystemTime>,
}

#[derive(Default)]
pub struct MatLibraryState {
    pub items: Vec<MatItem>,
    /// The search field.
    pub filter: String,
    /// Names of the tiles drawn last (In Model, library, examples unless folded), for the harness.
    pub shown: Vec<String>,
    pub errors: Vec<String>,
    pub scanned: Option<PathBuf>,
    /// The bundled examples, built on first use.
    examples: Option<Vec<MatItem>>,
    /// Library file to delete once confirmed.
    pub delete_ask: Option<PathBuf>,
    /// Subfolders of the library folder (empty ones too).
    pub dirs: Vec<String>,
}

/// What a click or a tile's menu asked for (applied after the list is drawn).
enum MatAct {
    /// A model material: paint with it.
    Paint(Material),
    /// A library or example material (its favourites / recents key): paint with it (added to
    /// the document by the first paint).
    Use(Material, String),
    Rename(super::library::RenameTarget, String),
    Save(Material),
    Export(MaterialId),
    Delete(PathBuf),
    /// A model material: remove it from the document.
    DeleteModel(MaterialId),
    /// A model material: repaint its uses with the current material and remove it.
    Replace(MaterialId),
    /// Toggle the favourite of this key.
    Favourite(String),
    /// Move a library file to a subfolder.
    Move(PathBuf, String),
}

/// A material tile's look (`material_look`), painted into any rect.
pub(crate) struct MatLook {
    color: Color32,
    tex: Option<egui::TextureId>,
    mark: Option<Color32>,
}

impl MatLook {
    pub(crate) fn paint(&self, p: &egui::Painter, r: egui::Rect) {
        p.rect_filled(r.shrink(1.0), 4.0, self.color);
        if let Some(t) = self.tex {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            p.image(t, r.shrink(3.0), uv, Color32::WHITE);
        }
        if let Some(c) = self.mark {
            let a = r.right_top();
            let tri = vec![a + egui::vec2(-9.0, 2.0), a + egui::vec2(-2.0, 2.0), a + egui::vec2(-2.0, 9.0)];
            p.add(egui::Shape::convex_polygon(tri, c, egui::Stroke::new(0.5, Color32::from_black_alpha(90))));
        }
    }
}

/// `~/.varstotas/materials`.
pub fn default_dir() -> Option<PathBuf> {
    crate::bridge::home_dir().map(|d| d.join("materials"))
}

fn scan(root: &Path, old: Vec<MatItem>) -> (Vec<MatItem>, Vec<String>) {
    fn walk(d: &Path, depth: usize, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                if depth < 2 {
                    walk(&p, depth + 1, out);
                }
            } else if p.extension().is_some_and(|e| e.eq_ignore_ascii_case(MATERIAL_EXT)) {
                out.push(p);
            }
        }
    }
    let mut files = Vec::new();
    walk(root, 0, &mut files);
    let mut old: Vec<MatItem> = old;
    let (mut items, mut errors) = (Vec::new(), Vec::new());
    for path in files {
        let mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        // Unchanged files keep their parsed material.
        if let Some(i) = old.iter().position(|it| it.path.as_deref() == Some(path.as_path()) && it.mtime == mtime) {
            items.push(old.swap_remove(i));
            continue;
        }
        let folder = path.parent().and_then(|p| p.strip_prefix(root).ok()).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_default();
        match varstotas_kernel::io::load_material(&path) {
            Ok(mat) => items.push(MatItem { path: Some(path), folder, mat, mtime }),
            Err(e) => errors.push(format!("{}: {e}", path.display())),
        }
    }
    (items, errors)
}

pub(crate) fn safe_name(s: &str) -> String {
    let n: String = s.chars().map(|c| if c.is_alphanumeric() || " -_.()".contains(c) { c } else { '_' }).collect();
    let n = n.trim().trim_matches('.').to_string();
    if n.is_empty() { "Material".into() } else { n } // i18n-ignore
}

/// The materials of the Material Gallery demo (its floor and backdrop left out).
fn examples() -> Vec<MatItem> {
    let Some(dir) = crate::debug_capture::qa_examples_dir() else { return Vec::new() };
    let Ok(m) = varstotas_kernel::io::load(&dir.join("demos/material_gallery.vrst")) else { return Vec::new() };
    m.materials
        .values()
        .filter(|x| !matches!(x.name.as_str(), "Showroom Floor" | "Backdrop")) // i18n-ignore
        .map(|x| MatItem { path: None, folder: EXAMPLES.into(), mat: x.clone(), mtime: None })
        .collect()
}

/// Same look: colour, texture and PBR (names aside).
fn same_material(a: &Material, b: &Material) -> bool {
    a.color == b.color && a.image == b.image && a.pbr == b.pbr
}

/// One-line description for a swatch's hover text.
pub(crate) fn describe(m: &Material) -> String {
    let p = m.pbr_or_default();
    let mut kind = Vec::new();
    if p.metallic > 0.5 {
        kind.push(tr!("mat-kind-metal"));
    }
    if p.transmission > 0.5 {
        kind.push(tr!("mat-kind-glass"));
    }
    if p.emission_strength > 0.0 && p.emission.iter().any(|c| *c > 0.0) {
        kind.push(tr!("mat-kind-emissive"));
    }
    if m.image.is_some() {
        kind.push(tr!("mat-kind-texture"));
    }
    if let Some(sh) = &p.shader {
        kind.push(tr!("mat-kind-shader", name = crate::shaders::preset_name(&sh.source).map(str::to_string).unwrap_or_else(|| tr!("mat-kind-shader-custom"))));
    }
    kind.push(tr!("mat-kind-roughness", value = format!("{:.2}", p.roughness)));
    format!("{}\n{}", m.name, kind.join(", "))
}

impl VarstotasApp {
    pub(crate) fn material_library_dir(&self) -> Option<PathBuf> {
        self.prefs.mat_library_dir.clone().or_else(default_dir)
    }

    pub(crate) fn refresh_materials(&mut self) {
        let Some(dir) = self.material_library_dir() else { return };
        let _ = std::fs::create_dir_all(&dir);
        let (items, errors) = scan(&dir, std::mem::take(&mut self.mat_library.items));
        for e in &errors {
            log::warn!("materials: {e}");
        }
        self.mat_library.items = items;
        self.mat_library.errors = errors;
        self.mat_library.dirs = super::tray_lib::subfolders(&dir);
        self.mat_library.scanned = Some(dir);
    }

    fn ensure_examples(&mut self) {
        if self.mat_library.examples.is_none() {
            self.mat_library.examples = Some(examples());
        }
    }

    /// Library materials and the examples (built when `with_examples`), as (folder, material).
    pub(crate) fn library_materials(&mut self, with_examples: bool) -> Vec<(String, Material)> {
        if self.mat_library.scanned != self.material_library_dir() {
            self.refresh_materials();
        }
        if with_examples {
            self.ensure_examples();
        }
        let ex = self.mat_library.examples.iter().flatten().filter(|_| with_examples);
        self.mat_library.items.iter().chain(ex).map(|i| (i.folder.clone(), i.mat.clone())).collect()
    }

    /// Add `mat` to the document (one undo step; an identical material of the same name is
    /// reused, another one of that name gets a new name) and paint with it.
    pub(crate) fn use_material(&mut self, mat: &Material) -> MaterialId {
        let found = self.doc.model.materials.iter().find(|(_, m)| m.name == mat.name).map(|(id, m)| (id, same_material(m, mat)));
        let id = match found {
            Some((id, true)) => id,
            _ => {
                let mut mat = mat.clone();
                if found.is_some() {
                    let base = mat.name.clone();
                    let mut k = 2;
                    while self.doc.model.materials.values().any(|m| m.name == mat.name) {
                        mat.name = format!("{base} {k}");
                        k += 1;
                    }
                }
                let name = mat.name.clone();
                self.doc
                    .commit(&tr!("mat-undo-add", name = name.as_str()), |m| {
                        let id = m.add_material(&mat.name, mat.color);
                        m.materials[id] = mat;
                        id
                    })
                    .unwrap_or_default()
            }
        };
        if let Some(m) = self.doc.model.materials.get(id) {
            self.doc.current_material = Some(PaletteMaterial::of(m));
        }
        if self.tool_id() != crate::tools::ToolId::Paint {
            self.actions.push(Action::SetTool(crate::tools::ToolId::Paint));
        }
        id
    }

    /// Write model material `id` to `<library>/<name>.vrstmat` (replacing a file of that
    /// name: saving again updates it).
    pub(crate) fn save_material_to_library(&mut self, id: MaterialId) {
        if let Some(mat) = self.doc.model.materials.get(id).cloned() {
            self.save_material_value(&mat);
        }
    }

    /// Write `mat` to `<library>/<name>.vrstmat` (a model material or an example).
    pub(crate) fn save_material_value(&mut self, mat: &Material) {
        let Some(dir) = self.material_library_dir() else { return };
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(format!("{}.{MATERIAL_EXT}", safe_name(&mat.name)));
        match varstotas_kernel::io::save_material(mat, &path) {
            Ok(()) => {
                self.flash(tr!("mat-saved", name = mat.name.as_str()));
                self.refresh_materials();
            }
            Err(e) => self.flash_error(tr!("mat-save-failed", error = e.to_string())),
        }
    }

    /// Export model material `id` to a file chosen by the user.
    pub(crate) fn export_material(&mut self, id: MaterialId) {
        let Some(mat) = self.doc.model.materials.get(id).cloned() else { return };
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter(tr!("mat-file-filter"), &[MATERIAL_EXT])
            .set_file_name(format!("{}.{MATERIAL_EXT}", safe_name(&mat.name)))
            .save_file()
        else {
            return;
        };
        if path.extension().is_none() {
            path.set_extension(MATERIAL_EXT);
        }
        self.export_material_to(id, &path);
    }

    pub(crate) fn export_material_to(&mut self, id: MaterialId, path: &Path) {
        let Some(mat) = self.doc.model.materials.get(id) else { return };
        match varstotas_kernel::io::save_material(mat, path) {
            Ok(()) => self.flash(tr!("mat-exported", path = path.display().to_string())),
            Err(e) => self.flash_error(tr!("mat-export-failed", error = e.to_string())),
        }
    }

    /// Import… : add material files chosen by the user to the document.
    /// Materials → Import…: Varstotas materials and whatever the `@material_import` plugins read.
    pub(crate) fn import_materials(&mut self) {
        let importers: Vec<_> = self.plugins.list.iter().filter(|p| !p.material_import.is_empty()).cloned().collect();
        let mut all: Vec<&str> = vec![MATERIAL_EXT];
        all.extend(importers.iter().flat_map(|p| p.material_import.iter().map(String::as_str)));
        let mut d = rfd::FileDialog::new();
        if !importers.is_empty() {
            d = d.add_filter(tr!("mat-all-filter"), &all);
        }
        d = d.add_filter(tr!("mat-file-filter"), &[MATERIAL_EXT]);
        for p in &importers {
            let exts: Vec<&str> = p.material_import.iter().map(String::as_str).collect();
            d = d.add_filter(p.name.clone(), &exts);
        }
        let Some(paths) = d.pick_files() else { return };
        for p in paths {
            self.import_material_from(&p);
        }
    }

    /// A `.vrstmat`, or a file of a material importer plugin (its extension).
    pub(crate) fn import_material_from(&mut self, path: &Path) {
        let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        let plugin = self.plugins.list.iter().find(|p| p.material_import.contains(&ext)).cloned();
        if let Some(p) = plugin.filter(|_| ext != MATERIAL_EXT) {
            let res = crate::plugins::param_values(&p, &[], self.doc.model.unit).and_then(|params| crate::mat_plugins::import(&p, path, params));
            match res {
                Ok(mats) if !mats.is_empty() => {
                    for m in &mats {
                        self.use_material(m);
                    }
                    let names: Vec<&str> = mats.iter().map(|m| m.name.as_str()).collect();
                    self.flash(tr!("mat-imported-with", names = names.join(", "), plugin = p.name.as_str()));
                }
                Ok(_) => self.flash_error(tr!("mat-import-empty", plugin = p.name.as_str(), path = path.display().to_string())),
                Err(e) => self.flash_error(tr!("mat-import-failed-with", path = path.display().to_string(), plugin = p.name.as_str(), error = e)),
            }
            return;
        }
        match varstotas_kernel::io::load_material(path) {
            Ok(mat) => {
                self.use_material(&mat);
                self.flash(tr!("mat-imported", name = mat.name.as_str()));
            }
            Err(e) => self.flash_error(tr!("mat-import-failed", path = path.display().to_string(), error = e.to_string())),
        }
    }

    /// The library part of the Materials card, framed like the Components list: the top row
    /// (search, Refresh, + New from Texture / Import, Online Materials, …), then the sections
    /// In Model, My Library (and its subfolders), Examples. A click on a model material paints
    /// with it, on a library one adds it to the document first.
    pub(crate) fn material_library_ui(&mut self, ui: &mut Ui) {
        use super::tray_lib::{TopRow, section, tiles, top_row};
        if self.mat_library.scanned != self.material_library_dir() {
            self.refresh_materials();
        }
        ui.add_space(4.0);
        let custom = self.prefs.mat_library_dir.is_some();
        let (mut texture, mut import, mut choose, mut default_dir, mut reveal_root) = (false, false, false, false, false);
        let mut show_ex = !self.prefs.hide_mat_examples;
        let unused = {
            let s = self.model_stats();
            s.materials - s.materials_used
        };
        let mut purge = false;
        let top: TopRow = top_row(
            ui,
            "m:",
            &mut self.mat_library.filter,
            &mut self.prefs.lib_tiles,
            &tr!("mat-plus-hover"),
            &tr!("mat-online-hover"),
            |ui| {
                if ui.button(tr!("mat-new-from-texture")).on_hover_text(tr!("mat-new-from-texture-hover")).clicked() {
                    texture = true;
                    ui.close();
                }
                if ui.button(tr!("mat-import")).on_hover_text(tr!("mat-import-hover")).clicked() {
                    import = true;
                    ui.close();
                }
            },
            |ui| {
                if ui.button(tr!("mat-choose-folder")).clicked() {
                    choose = true;
                    ui.close();
                }
                if custom && ui.button(tr!("mat-default-folder")).on_hover_text(tr!("mat-default-folder-hover")).clicked() {
                    default_dir = true;
                    ui.close();
                }
                if ui.button(tr!("library-show-in-finder")).clicked() {
                    reveal_root = true;
                    ui.close();
                }
                if ui.add_enabled(unused > 0, egui::Button::new(tr!("mat-purge"))).on_hover_text(tr!("mat-purge-hover")).clicked() {
                    purge = true;
                    ui.close();
                }
                if crate::debug_capture::qa_examples_dir().is_some() {
                    ui.separator();
                    ui.checkbox(&mut show_ex, tr!("library-show-examples")).on_hover_text(tr!("mat-show-examples-hover"));
                }
            },
        );
        self.prefs.hide_mat_examples = !show_ex;
        if top.refresh {
            self.refresh_materials();
        }
        if top.online {
            self.online.open = true;
        }
        if texture {
            self.actions.push(Action::ImportTexture);
        }
        if import {
            self.import_materials();
        }
        if choose && let Some(d) = rfd::FileDialog::new().pick_folder() {
            self.prefs.mat_library_dir = Some(d);
        }
        if default_dir {
            self.prefs.mat_library_dir = None;
        }
        if reveal_root && let Some(d) = self.material_library_dir() {
            crate::ui::extensions::reveal(&d);
        }
        let filter = self.mat_library.filter.trim().to_lowercase();
        let found = |m: &Material| super::tray_lib::found(&filter, &m.name);
        let size = self.prefs.lib_tiles;
        let ctx = ui.ctx().clone();
        let cur = self.doc.current_material.as_ref().map(|c| c.name.clone());
        let mut act: Option<MatAct> = None;
        let mut drag: Option<TrayDrag> = None;
        let mut folder_act: Option<FolderAct> = None;
        let root = self.material_library_dir();

        self.mat_library.shown.clear();
        // In Model
        let model: Vec<(MaterialId, Material)> = self.doc.model.materials.iter().filter(|(_, m)| found(m)).map(|(id, m)| (id, m.clone())).collect();
        let (folded, _, _) = section(ui, &mut self.prefs.library_collapsed, "mat:In Model", &tr!("library-in-model"), &tr!("mat-in-model-hover"), false, false, Some(model.len()), |ui| { // i18n-ignore
            purge |= super::tray_lib::purge_button(ui, "mat", "materials", unused);
        });
        if purge {
            self.purge_unused(false, true);
        }
        if !folded {
            self.mat_library.shown.extend(model.iter().map(|(_, m)| m.name.clone()));
            tiles(ui, size, model.len(), |ui, k| {
                let (id, m) = &model[k];
                let look = self.material_look(&ctx, m);
                let r = super::tray_lib::tile(ui, size, &m.name, cur.as_deref() == Some(m.name.as_str()), |p, r| look.paint(p, r)).on_hover_text(describe(m));
                if r.clicked() {
                    act = Some(MatAct::Paint(m.clone()));
                }
                if r.drag_started() {
                    drag = Some(TrayDrag { item: DragItem::Mat(Box::new(m.clone()), false), name: m.name.clone(), recent: None, entered: false });
                }
                r.context_menu(|ui| {
                    if ui.button(tr!("mat-use")).clicked() {
                        act = Some(MatAct::Paint(m.clone()));
                        ui.close();
                    }
                    if ui.button(tr!("library-rename-ellipsis")).clicked() {
                        act = Some(MatAct::Rename(super::library::RenameTarget::Mat(*id), m.name.clone()));
                        ui.close();
                    }
                    if ui.button(tr!("library-save-to-library")).on_hover_text(tr!("mat-save-hover")).clicked() {
                        act = Some(MatAct::Save(m.clone()));
                        ui.close();
                    }
                    if ui.button(tr!("mat-export")).clicked() {
                        act = Some(MatAct::Export(*id));
                        ui.close();
                    }
                    let other = cur.as_deref().is_some_and(|c| c != m.name);
                    let r = ui.add_enabled(other, egui::Button::new(tr!("mat-replace"))).on_hover_text(tr!("mat-replace-hover"));
                    if r.clicked() {
                        act = Some(MatAct::Replace(*id));
                        ui.close();
                    }
                    if ui.button(tr!("delete")).on_hover_text(tr!("mat-delete-model-hover")).clicked() {
                        act = Some(MatAct::DeleteModel(*id));
                        ui.close();
                    }
                });
            });
        }

        // Favourites, Recent, My Library, its subfolders, Examples
        if !self.prefs.hide_mat_examples || self.prefs.mat_favourites.iter().chain(&self.prefs.mat_recent).any(|k| k.starts_with("x:")) {
            self.ensure_examples();
        }
        let examples: Vec<&MatItem> = self.mat_library.examples.iter().flatten().collect();
        let entry = |i: &MatItem| (i.path.clone(), i.folder.clone(), i.mat.clone());
        let by_key = |key: &str| -> Option<(Option<PathBuf>, String, Material)> {
            if let Some(name) = key.strip_prefix("x:") {
                examples.iter().find(|i| i.mat.name == name).map(|i| entry(i))
            } else {
                self.mat_library.items.iter().find(|i| i.path.as_ref().is_some_and(|p| file_key("m:", p) == key)).map(entry)
            }
        };
        let mut dirs = self.mat_library.dirs.clone();
        dirs.extend(self.mat_library.items.iter().map(|i| i.folder.clone()).filter(|f| !f.is_empty()));
        dirs.sort_by_key(|a| a.to_lowercase());
        dirs.dedup();
        struct Sec {
            key: String,
            title: String,
            tip: String,
            dir: Option<PathBuf>,
            /// "" root, a subfolder, None for the others
            folder: Option<String>,
            fold_default: bool,
            examples: bool,
            list: Vec<(Option<PathBuf>, String, Material)>,
        }
        let shown_in = |d: &Option<PathBuf>| d.as_ref().map(|d| tr!("library-folder-reveal-hover", path = d.display().to_string())).unwrap_or_default();
        let mut secs = vec![
            Sec {
                key: "mat:\u{1}Favourites".into(),
                title: tr!("library-favourites"),
                tip: tr!("mat-favourites-hover"),
                dir: None,
                folder: None,
                fold_default: false,
                examples: false,
                list: self.prefs.mat_favourites.iter().filter_map(|k| by_key(k)).filter(|e| found(&e.2)).collect(),
            },
            Sec {
                key: "mat:\u{1}Recent".into(),
                title: tr!("library-recent"),
                tip: tr!("mat-recent-hover"),
                dir: None,
                folder: None,
                fold_default: false,
                examples: false,
                list: self.prefs.mat_recent.iter().filter_map(|k| by_key(k)).filter(|e| found(&e.2)).collect(),
            },
        ];
        for f in std::iter::once(String::new()).chain(dirs.iter().cloned()) {
            let dir = root.as_ref().map(|r| if f.is_empty() { r.clone() } else { r.join(&f) });
            secs.push(Sec {
                key: format!("mat:{f}"),
                title: if f.is_empty() { tr!("tray-my-library") } else { f.clone() },
                tip: shown_in(&dir),
                dir,
                folder: Some(f.clone()),
                fold_default: false,
                examples: false,
                list: self.mat_library.items.iter().filter(|i| i.folder == f && found(&i.mat)).map(entry).collect(),
            });
        }
        if !self.prefs.hide_mat_examples && !examples.is_empty() {
            secs.push(Sec {
                key: format!("mat:{EXAMPLES}"),
                title: tr!("library-examples"),
                tip: tr!("mat-examples-hover"),
                dir: None,
                folder: None,
                // the examples start folded (many large tiles)
                fold_default: true,
                examples: true,
                list: examples.iter().filter(|i| found(&i.mat)).map(|i| entry(i)).collect(),
            });
        }
        let favs = self.prefs.mat_favourites.clone();
        for sec in secs {
            let special = sec.folder.is_none() && !sec.examples;
            if special && sec.list.is_empty() {
                continue;
            }
            let mut hide = false;
            let (folded, clicked, title) = section(ui, &mut self.prefs.library_collapsed, &sec.key, &sec.title, &sec.tip, sec.dir.is_some(), sec.fold_default, Some(sec.list.len()), |ui| {
                if sec.examples {
                    let r = super::icon_button(ui, super::icons::Icon::EyeOff, false, 18.0).on_hover_text(tr!("library-hide-examples"));
                    super::tray_lib::spot("hide:mat:Examples", r.rect.center());
                    hide = r.clicked();
                }
            });
            if let Some(f) = &sec.folder {
                super::tray_lib::folder_menu(&title, f, sec.dir.as_deref(), &mut folder_act);
            }
            if hide {
                self.prefs.hide_mat_examples = true;
            }
            if clicked && let Some(d) = &sec.dir {
                crate::ui::extensions::reveal(d);
            }
            if sec.folder.as_deref() == Some("") && self.mat_library.items.is_empty() && !folded {
                ui.label(RichText::new(tr!("mat-empty")).small().weak());
            }
            if folded {
                continue;
            }
            self.mat_library.shown.extend(sec.list.iter().map(|(_, _, m)| m.name.clone()));
            let list = &sec.list;
            tiles(ui, size, list.len(), |ui, k| {
                let (path, folder, m) = &list[k];
                let key = match path {
                    Some(p) => file_key("m:", p),
                    None => format!("x:{}", m.name),
                };
                let fav = favs.contains(&key);
                let look = self.material_look(&ctx, m);
                let r = super::tray_lib::tile(ui, size, &m.name, cur.as_deref() == Some(m.name.as_str()), |p, r| look.paint(p, r)).on_hover_text(describe(m));
                if fav {
                    super::tray_lib::mark_favourite(ui, size, &r);
                }
                if r.clicked() {
                    act = Some(MatAct::Use(m.clone(), key.clone()));
                }
                if r.drag_started() {
                    drag = Some(TrayDrag { item: DragItem::Mat(Box::new(m.clone()), true), name: m.name.clone(), recent: Some((true, key.clone())), entered: false });
                }
                r.context_menu(|ui| {
                    if ui.button(tr!("mat-use")).on_hover_text(tr!("mat-use-hover")).clicked() {
                        act = Some(MatAct::Use(m.clone(), key.clone()));
                        ui.close();
                    }
                    match path {
                        Some(p) => {
                            if ui.button(tr!("library-rename-ellipsis")).clicked() {
                                act = Some(MatAct::Rename(super::library::RenameTarget::MatFile(p.clone()), m.name.clone()));
                                ui.close();
                            }
                            if ui.button(tr!("library-show-in-finder")).clicked() {
                                crate::ui::extensions::reveal(p);
                                ui.close();
                            }
                        }
                        None => {
                            if ui.button(tr!("library-save-to-library")).on_hover_text(tr!("mat-copy-hover")).clicked() {
                                act = Some(MatAct::Save(m.clone()));
                                ui.close();
                            }
                        }
                    }
                    let mut lib = None;
                    super::tray_lib::lib_menu(ui, Some(fav), path.as_ref().map(|_| (dirs.as_slice(), folder.as_str())), &mut lib);
                    match lib {
                        Some(LibMenuAct::Favourite) => act = Some(MatAct::Favourite(key.clone())),
                        Some(LibMenuAct::Move(f)) => act = path.clone().map(|p| MatAct::Move(p, f)),
                        None => {}
                    }
                    if let Some(p) = path
                        && ui.button(tr!("library-delete-ellipsis")).clicked()
                    {
                        act = Some(MatAct::Delete(p.clone()));
                        ui.close();
                    }
                });
            });
        }
        if !self.mat_library.errors.is_empty() {
            ui.label(RichText::new(tr!("library-unreadable-files", n = self.mat_library.errors.len())).color(Color32::from_rgb(200, 50, 40)))
                .on_hover_text(self.mat_library.errors.join("\n"));
        }
        if drag.is_some() {
            self.tray_drag = drag;
        }
        if let Some(a) = folder_act {
            self.folder_act(true, a);
        }
        match act {
            Some(MatAct::Paint(m)) => {
                self.doc.current_material = Some(PaletteMaterial::of(&m));
                if self.tool_id() != crate::tools::ToolId::Paint {
                    self.actions.push(Action::SetTool(crate::tools::ToolId::Paint));
                }
            }
            Some(MatAct::Use(m, key)) => {
                self.pick_material(&m);
                super::tray_lib::note_recent(&mut self.prefs.mat_recent, &key);
            }
            Some(MatAct::Rename(t, name)) => self.library.rename = Some((t, name)),
            Some(MatAct::Save(m)) => self.save_material_value(&m),
            Some(MatAct::Export(id)) => self.export_material(id),
            Some(MatAct::Delete(p)) => self.mat_library.delete_ask = Some(p),
            Some(MatAct::DeleteModel(id)) => self.delete_model_material(id),
            Some(MatAct::Replace(id)) => self.replace_with_current_material(id),
            Some(MatAct::Favourite(key)) => super::tray_lib::toggle(&mut self.prefs.mat_favourites, &key),
            Some(MatAct::Move(p, folder)) => {
                if let Some(root) = self.material_library_dir() {
                    match super::tray_lib::move_file(&p, &root, &folder) {
                        Ok(to) => {
                            super::tray_lib::rekey([&mut self.prefs.mat_favourites, &mut self.prefs.mat_recent], "m:", &p, &to);
                            self.refresh_materials();
                        }
                        Err(e) => self.flash_error(tr!("library-move-failed", error = e)),
                    }
                }
            }
            None => {}
        }
    }

    /// Take `mat` (a library, example or downloaded material) to paint with. It is added to
    /// the document only when something is painted with it (`PaletteMaterial::ensure_in`, in
    /// the Paint step); the same material in the model is used as it is, another one of that
    /// name makes this one get a new name.
    pub(crate) fn pick_material(&mut self, mat: &Material) {
        self.doc.current_material = Some(self.palette_for(mat));
        if self.tool_id() != crate::tools::ToolId::Paint {
            self.actions.push(Action::SetTool(crate::tools::ToolId::Paint));
        }
    }

    /// The paint entry of a library material (see [`Self::pick_material`]).
    pub(crate) fn palette_for(&self, mat: &Material) -> PaletteMaterial {
        let m = &self.doc.model;
        let mut pal = PaletteMaterial::of(mat);
        if m.materials.values().any(|x| x.name == mat.name && !same_material(x, mat)) {
            let mut k = 2;
            while m.materials.values().any(|x| x.name == pal.name) {
                pal.name = format!("{} {k}", mat.name);
                k += 1;
            }
        }
        pal
    }

    /// In Model → Delete: remove model material `id` (one undo step); what it painted gets the
    /// default material.
    /// Repaint what material `id` paints with the current paint material (added to the model
    /// if needed) and remove `id`: one undo step.
    pub(crate) fn replace_with_current_material(&mut self, id: MaterialId) {
        let Some(name) = self.doc.model.materials.get(id).map(|m| m.name.clone()) else { return };
        let Some(cur) = self.doc.current_material.clone().filter(|c| c.name != name) else { return };
        let label = tr!("mat-undo-replace", name = name.as_str(), with = cur.name.as_str());
        match self.doc.commit(&label, |m| {
            let new = cur.ensure_in(m);
            m.replace_material(id, new)
        }) {
            Some(Ok(n)) => self.flash(tr!("mat-replaced", name = name.as_str(), with = cur.name.as_str(), n = n)),
            _ => self.flash_error(tr!("mat-delete-failed", name = name.as_str())),
        }
    }

    pub(crate) fn delete_model_material(&mut self, id: MaterialId) {
        let Some(name) = self.doc.model.materials.get(id).map(|m| m.name.clone()) else { return };
        match self.doc.commit(&tr!("mat-undo-delete", name = name.as_str()), |m| m.delete_material(id)) {
            Some(Ok(n)) => {
                if self.doc.current_material.as_ref().is_some_and(|c| c.name == name) {
                    self.doc.current_material = None;
                }
                self.flash(if n > 0 { tr!("mat-deleted-uses", name = name.as_str(), n = n) } else { tr!("mat-deleted", name = name.as_str()) });
            }
            _ => self.flash_error(tr!("mat-delete-failed", name = name.as_str())),
        }
    }

    /// How a material tile looks: its colour, the texture's thumbnail, a corner mark.
    pub(crate) fn material_look(&mut self, ctx: &egui::Context, m: &Material) -> MatLook {
        let tex = m.image.as_ref().and_then(|img| self.thumbnail(ctx, img));
        let p = m.pbr_or_default();
        // A corner mark: metal (light), glass (blue), emissive (warm).
        let mark = if p.emission_strength > 0.0 && p.emission.iter().any(|c| *c > 0.0) {
            Some(Color32::from_rgb(255, 200, 80))
        } else if p.transmission > 0.5 {
            Some(Color32::from_rgb(120, 190, 255))
        } else if p.metallic > 0.5 {
            Some(Color32::from_gray(245))
        } else {
            None
        };
        MatLook { color: super::color32(m.color), tex, mark }
    }

    /// Rename model material `id` (one undo step; the paint material follows).
    pub(crate) fn rename_material(&mut self, id: MaterialId, name: &str) {
        let Some(old) = self.doc.model.materials.get(id).map(|m| m.name.clone()) else { return };
        if self.doc.model.materials.iter().any(|(i, m)| i != id && m.name == name) {
            return self.flash_error(tr!("mat-rename-exists", name = name));
        }
        self.doc.commit(&tr!("mat-undo-rename", name = name), |m| m.materials[id].name = name.to_string());
        if let Some(c) = self.doc.current_material.as_mut().filter(|c| c.name == old) {
            c.name = name.to_string();
        }
        self.flash(tr!("library-renamed", name = name));
    }

    /// Rename a library material: the material in the file and the file itself.
    pub(crate) fn rename_material_file(&mut self, path: &Path, name: &str) {
        let r = (|| -> Result<PathBuf, String> {
            let mut mat = varstotas_kernel::io::load_material(path).map_err(|e| e.to_string())?;
            mat.name = name.to_string();
            let to = path.with_file_name(format!("{}.{MATERIAL_EXT}", safe_name(name)));
            if to != path && to.exists() {
                return Err(tr!("mat-file-exists", path = to.display().to_string()));
            }
            varstotas_kernel::io::save_material(&mat, &to).map_err(|e| e.to_string())?;
            if to != path {
                std::fs::remove_file(path).map_err(|e| e.to_string())?;
            }
            Ok(to)
        })();
        match r {
            Ok(_) => {
                self.flash(tr!("library-renamed", name = name));
                self.refresh_materials();
            }
            Err(e) => self.flash_error(tr!("library-rename-failed", path = path.display().to_string(), error = e)),
        }
    }

    /// "Delete from the materials library?"
    pub(crate) fn material_delete_dialog(&mut self, ctx: &egui::Context) {
        let Some(path) = self.mat_library.delete_ask.clone() else { return };
        let name = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let mut choice = None;
        let resp = egui::Modal::new(egui::Id::new("material_delete")).show(ctx, |ui| {
            ui.set_width(360.0);
            ui.heading(tr!("mat-delete-title"));
            ui.add_space(4.0);
            ui.label(tr!("library-delete-text", name = name.as_str()));
            ui.label(RichText::new(path.display().to_string()).small().weak());
            ui.label(RichText::new(tr!("mat-delete-undone")).weak());
            ui.add_space(10.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new(RichText::new(tr!("delete")).strong().color(Color32::from_rgb(200, 50, 40)))).clicked() {
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
        match choice {
            Some(true) => {
                self.mat_library.delete_ask = None;
                match std::fs::remove_file(&path) {
                    Ok(()) => {
                        self.flash(tr!("mat-deleted-from-library", name = name.as_str()));
                        self.refresh_materials();
                    }
                    Err(e) => self.flash_error(tr!("library-delete-failed", path = path.display().to_string(), error = e.to_string())),
                }
            }
            Some(false) => self.mat_library.delete_ask = None,
            None => {}
        }
    }
}
