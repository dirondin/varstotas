//! Materials library: `.vrstmat` files (one material each — colour, texture, PBR parameters,
//! maps, shader) in `~/.varstotas/materials`, shared between documents and sessions like the
//! components library; subfolders are categories, the demo gallery's materials show under
//! "Examples". In the Materials card: a click on a library swatch adds the material to the
//! document and paints with it; a model material's context menu saves it to the library or
//! exports it to a file; Import… adds a material file to the document.

use crate::app::{Action, VarstotasApp};
use crate::document::PaletteMaterial;
use egui::{Color32, RichText, Ui};
use varstotas_kernel::io::MATERIAL_EXT;
use varstotas_kernel::{Material, MaterialId};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The folder of the bundled examples (not files).
pub const EXAMPLES: &str = "Examples";

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
}

/// What a click or a tile's menu asked for (applied after the list is drawn).
enum MatAct {
    /// A model material: paint with it.
    Paint(Material),
    /// A library or example material: add it to the document, then paint.
    Use(Material),
    Rename(super::library::RenameTarget, String),
    Save(Material),
    Export(MaterialId),
    Delete(PathBuf),
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
    if n.is_empty() { "Material".into() } else { n }
}

/// The materials of the Material Gallery demo (its floor and backdrop left out).
fn examples() -> Vec<MatItem> {
    let m = crate::demo::material_gallery();
    m.materials
        .values()
        .filter(|x| !matches!(x.name.as_str(), "Showroom Floor" | "Backdrop"))
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
        kind.push("metal".to_string());
    }
    if p.transmission > 0.5 {
        kind.push("glass".into());
    }
    if p.emission_strength > 0.0 && p.emission.iter().any(|c| *c > 0.0) {
        kind.push("emissive".into());
    }
    if m.image.is_some() {
        kind.push("texture".into());
    }
    if let Some(sh) = &p.shader {
        kind.push(format!("shader {}", crate::shaders::preset_name(&sh.source).unwrap_or("custom")));
    }
    kind.push(format!("roughness {:.2}", p.roughness));
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
                    .commit(&format!("Add Material {name}"), |m| {
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
                self.flash(format!("Saved to the materials library: {}", mat.name));
                self.refresh_materials();
            }
            Err(e) => self.flash_error(format!("Save material failed: {e}")),
        }
    }

    /// Export model material `id` to a file chosen by the user.
    pub(crate) fn export_material(&mut self, id: MaterialId) {
        let Some(mat) = self.doc.model.materials.get(id).cloned() else { return };
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter("Varstotas material", &[MATERIAL_EXT])
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
            Ok(()) => self.flash(format!("Exported {}", path.display())),
            Err(e) => self.flash_error(format!("Export material failed: {e}")),
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
            d = d.add_filter("All materials", &all);
        }
        d = d.add_filter("Varstotas material", &[MATERIAL_EXT]);
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
                    self.flash(format!("Imported {} with {}", names.join(", "), p.name));
                }
                Ok(_) => self.flash_error(format!("{}: no materials in {}", p.name, path.display())),
                Err(e) => self.flash_error(format!("Can't import {} with {}: {e}", path.display(), p.name)),
            }
            return;
        }
        match varstotas_kernel::io::load_material(path) {
            Ok(mat) => {
                self.use_material(&mat);
                self.flash(format!("Imported material {}", mat.name));
            }
            Err(e) => self.flash_error(format!("Can't import {}: {e}", path.display())),
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
        let top: TopRow = top_row(
            ui,
            "m:",
            &mut self.mat_library.filter,
            &mut self.prefs.lib_tiles,
            "New material from an image, import material files",
            "Online Materials: free PBR materials (Poly Haven, ambientCG, plugins)",
            |ui| {
                if ui.button("New from Texture…").on_hover_text("A material from an image file (PNG, JPEG)").clicked() {
                    texture = true;
                    ui.close();
                }
                if ui.button("Import…").on_hover_text("Add material files (.vrstmat, plugin formats) to the document").clicked() {
                    import = true;
                    ui.close();
                }
            },
            |ui| {
                if ui.button("Choose Folder…").clicked() {
                    choose = true;
                    ui.close();
                }
                if custom && ui.button("Default Folder").on_hover_text("Back to ~/.varstotas/materials").clicked() {
                    default_dir = true;
                    ui.close();
                }
                if ui.button("Show in Finder").clicked() {
                    reveal_root = true;
                    ui.close();
                }
                ui.separator();
                ui.checkbox(&mut show_ex, "Show Examples").on_hover_text("The materials bundled with Varstotas (the Material Gallery demo)");
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
        let root = self.material_library_dir();

        self.mat_library.shown.clear();
        // In Model
        let model: Vec<(MaterialId, Material)> = self.doc.model.materials.iter().filter(|(_, m)| found(m)).map(|(id, m)| (id, m.clone())).collect();
        let (folded, _) = section(ui, &mut self.prefs.library_collapsed, "mat:In Model", "In Model", "The materials of the open document", false, false, Some(model.len()), |_| {});
        if !folded {
            self.mat_library.shown.extend(model.iter().map(|(_, m)| m.name.clone()));
            tiles(ui, size, model.len(), |ui, k| {
                let (id, m) = &model[k];
                let look = self.material_look(&ctx, m);
                let r = super::tray_lib::tile(ui, size, &m.name, cur.as_deref() == Some(m.name.as_str()), |p, r| look.paint(p, r)).on_hover_text(describe(m));
                if r.clicked() {
                    act = Some(MatAct::Paint(m.clone()));
                }
                r.context_menu(|ui| {
                    if ui.button("Use").clicked() {
                        act = Some(MatAct::Paint(m.clone()));
                        ui.close();
                    }
                    if ui.button("Rename…").clicked() {
                        act = Some(MatAct::Rename(super::library::RenameTarget::Mat(*id), m.name.clone()));
                        ui.close();
                    }
                    if ui.button("Save to Library").on_hover_text("Write it to the materials library folder, for any document").clicked() {
                        act = Some(MatAct::Save(m.clone()));
                        ui.close();
                    }
                    if ui.button("Export…").clicked() {
                        act = Some(MatAct::Export(*id));
                        ui.close();
                    }
                });
            });
        }

        // My Library, its subfolders, Examples
        let mut folders: Vec<String> = self.mat_library.items.iter().map(|i| i.folder.clone()).collect();
        folders.push(String::new());
        folders.sort();
        folders.dedup();
        if !self.prefs.hide_mat_examples {
            folders.push(EXAMPLES.into());
        }
        for folder in folders {
            let examples = folder == EXAMPLES;
            if examples {
                self.ensure_examples();
            }
            let list: Vec<(Option<PathBuf>, Material)> = if examples {
                self.mat_library.examples.iter().flatten().filter(|i| found(&i.mat)).map(|i| (None, i.mat.clone())).collect()
            } else {
                self.mat_library.items.iter().filter(|i| i.folder == folder && found(&i.mat)).map(|i| (i.path.clone(), i.mat.clone())).collect()
            };
            let dir = root.as_ref().filter(|_| !examples).map(|r| if folder.is_empty() { r.clone() } else { r.join(&folder) });
            let (title, tip) = match (examples, folder.is_empty()) {
                (true, _) => (folder.clone(), "Materials bundled with Varstotas (the Material Gallery demo)".to_string()),
                (_, true) => ("My Library".to_string(), dir.as_ref().map(|d| format!("{}\nClick to show in the Finder", d.display())).unwrap_or_default()),
                _ => (folder.clone(), dir.as_ref().map(|d| format!("{}\nClick to show in the Finder", d.display())).unwrap_or_default()),
            };
            let mut hide = false;
            let key = format!("mat:{folder}");
            // the examples start folded (many large tiles)
            let (folded, clicked) = section(ui, &mut self.prefs.library_collapsed, &key, &title, &tip, dir.is_some(), examples, Some(list.len()), |ui| {
                if examples {
                    let r = ui.small_button("Hide").on_hover_text("Hide the bundled examples (… → Show Examples brings them back)");
                    super::tray_lib::spot("hide:mat:Examples", r.rect.center());
                    hide = r.clicked();
                }
            });
            if hide {
                self.prefs.hide_mat_examples = true;
            }
            if clicked && let Some(d) = &dir {
                crate::ui::extensions::reveal(d);
            }
            if folder.is_empty() && self.mat_library.items.is_empty() && !folded {
                ui.label(RichText::new("Empty: right-click a model material → Save to Library").small().weak());
            }
            if folded {
                continue;
            }
            self.mat_library.shown.extend(list.iter().map(|(_, m)| m.name.clone()));
            tiles(ui, size, list.len(), |ui, k| {
                let (path, m) = &list[k];
                let look = self.material_look(&ctx, m);
                let r = super::tray_lib::tile(ui, size, &m.name, cur.as_deref() == Some(m.name.as_str()), |p, r| look.paint(p, r)).on_hover_text(describe(m));
                if r.clicked() {
                    act = Some(MatAct::Use(m.clone()));
                }
                r.context_menu(|ui| {
                    if ui.button("Use").on_hover_text("Add it to the model and paint with it").clicked() {
                        act = Some(MatAct::Use(m.clone()));
                        ui.close();
                    }
                    match path {
                        Some(p) => {
                            if ui.button("Rename…").clicked() {
                                act = Some(MatAct::Rename(super::library::RenameTarget::MatFile(p.clone()), m.name.clone()));
                                ui.close();
                            }
                            if ui.button("Show in Finder").clicked() {
                                crate::ui::extensions::reveal(p);
                                ui.close();
                            }
                            if ui.button("Delete…").clicked() {
                                act = Some(MatAct::Delete(p.clone()));
                                ui.close();
                            }
                        }
                        None => {
                            if ui.button("Save to Library").on_hover_text("Copy it to your materials library").clicked() {
                                act = Some(MatAct::Save(m.clone()));
                                ui.close();
                            }
                        }
                    }
                });
            });
        }
        if !self.mat_library.errors.is_empty() {
            ui.label(RichText::new(format!("{} file(s) could not be read", self.mat_library.errors.len())).color(Color32::from_rgb(200, 50, 40)))
                .on_hover_text(self.mat_library.errors.join("\n"));
        }
        match act {
            Some(MatAct::Paint(m)) => {
                self.doc.current_material = Some(PaletteMaterial::of(&m));
                if self.tool_id() != crate::tools::ToolId::Paint {
                    self.actions.push(Action::SetTool(crate::tools::ToolId::Paint));
                }
            }
            Some(MatAct::Use(m)) => {
                self.use_material(&m);
            }
            Some(MatAct::Rename(t, name)) => self.library.rename = Some((t, name)),
            Some(MatAct::Save(m)) => self.save_material_value(&m),
            Some(MatAct::Export(id)) => self.export_material(id),
            Some(MatAct::Delete(p)) => self.mat_library.delete_ask = Some(p),
            None => {}
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
            return self.flash_error(format!("There is a material “{name}” already"));
        }
        self.doc.commit(&format!("Rename Material {name}"), |m| m.materials[id].name = name.to_string());
        if let Some(c) = self.doc.current_material.as_mut().filter(|c| c.name == old) {
            c.name = name.to_string();
        }
        self.flash(format!("Renamed to {name}"));
    }

    /// Rename a library material: the material in the file and the file itself.
    pub(crate) fn rename_material_file(&mut self, path: &Path, name: &str) {
        let r = (|| -> Result<PathBuf, String> {
            let mut mat = varstotas_kernel::io::load_material(path).map_err(|e| e.to_string())?;
            mat.name = name.to_string();
            let to = path.with_file_name(format!("{}.{MATERIAL_EXT}", safe_name(name)));
            if to != path && to.exists() {
                return Err(format!("{} exists already", to.display()));
            }
            varstotas_kernel::io::save_material(&mat, &to).map_err(|e| e.to_string())?;
            if to != path {
                std::fs::remove_file(path).map_err(|e| e.to_string())?;
            }
            Ok(to)
        })();
        match r {
            Ok(_) => {
                self.flash(format!("Renamed to {name}"));
                self.refresh_materials();
            }
            Err(e) => self.flash_error(format!("Rename {}: {e}", path.display())),
        }
    }

    /// "Delete from the materials library?"
    pub(crate) fn material_delete_dialog(&mut self, ctx: &egui::Context) {
        let Some(path) = self.mat_library.delete_ask.clone() else { return };
        let name = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let mut choice = None;
        let resp = egui::Modal::new(egui::Id::new("material_delete")).show(ctx, |ui| {
            ui.set_width(360.0);
            ui.heading("Delete from the materials library?");
            ui.add_space(4.0);
            ui.label(format!("“{name}” will be deleted from the library folder:"));
            ui.label(RichText::new(path.display().to_string()).small().weak());
            ui.label(RichText::new("This cannot be undone. Documents using it keep their copy.").weak());
            ui.add_space(10.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new(RichText::new("Delete").strong().color(Color32::from_rgb(200, 50, 40)))).clicked() {
                    choice = Some(true);
                }
                if ui.button("Cancel").clicked() {
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
                        self.flash(format!("Deleted from the materials library: {name}"));
                        self.refresh_materials();
                    }
                    Err(e) => self.flash_error(format!("Delete {}: {e}", path.display())),
                }
            }
            Some(false) => self.mat_library.delete_ask = None,
            None => {}
        }
    }
}
