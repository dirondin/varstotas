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
    pub errors: Vec<String>,
    pub scanned: Option<PathBuf>,
    /// The bundled examples, built on first use.
    examples: Option<Vec<MatItem>>,
    /// Library file to delete once confirmed.
    pub delete_ask: Option<PathBuf>,
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
        default_dir()
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
        let (Some(dir), Some(mat)) = (self.material_library_dir(), self.doc.model.materials.get(id).cloned()) else { return };
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(format!("{}.{MATERIAL_EXT}", safe_name(&mat.name)));
        match varstotas_kernel::io::save_material(&mat, &path) {
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
    pub(crate) fn import_materials(&mut self) {
        let Some(paths) = rfd::FileDialog::new().add_filter("Varstotas material", &[MATERIAL_EXT]).pick_files() else { return };
        for p in paths {
            self.import_material_from(&p);
        }
    }

    pub(crate) fn import_material_from(&mut self, path: &Path) {
        match varstotas_kernel::io::load_material(path) {
            Ok(mat) => {
                self.use_material(&mat);
                self.flash(format!("Imported material {}", mat.name));
            }
            Err(e) => self.flash_error(format!("Can't import {}: {e}", path.display())),
        }
    }

    /// The context menu of a model material's swatch.
    pub(crate) fn model_material_menu(&mut self, ui: &mut Ui, id: MaterialId) {
        if ui.button("Save to Library").on_hover_text("Write it to ~/.varstotas/materials, for any document").clicked() {
            self.save_material_to_library(id);
            ui.close();
        }
        if ui.button("Export…").clicked() {
            self.export_material(id);
            ui.close();
        }
    }

    /// The Library part of the Materials card: folders of swatches.
    pub(crate) fn material_library_ui(&mut self, ui: &mut Ui) {
        if self.mat_library.scanned != self.material_library_dir() {
            self.refresh_materials();
        }
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Library").small().color(super::theme::TEXT_DIM));
            if let Some(dir) = self.material_library_dir()
                && ui.small_button("Folder").on_hover_text(format!("{}\nShow in the Finder", dir.display())).clicked()
            {
                crate::ui::extensions::reveal(&dir);
            }
            if ui.small_button("Import…").on_hover_text("Add material files (.vrstmat) to the document").clicked() {
                self.import_materials();
            }
            if ui.small_button("Online…").on_hover_text("Download free PBR materials (Poly Haven, ambientCG)").clicked() {
                self.online.open = true;
            }
            if ui.small_button("⟳").on_hover_text("Rescan the library folder").clicked() {
                self.refresh_materials();
            }
        });
        let mut folders: Vec<String> = self.mat_library.items.iter().map(|i| i.folder.clone()).collect();
        folders.dedup();
        folders.sort();
        folders.dedup();
        if self.mat_library.items.is_empty() {
            ui.label(RichText::new("Right-click a model material → Save to Library").small().weak());
        }
        folders.push(EXAMPLES.into());
        let ctx = ui.ctx().clone();
        let cur = self.doc.current_material.as_ref().map(|c| c.name.clone());
        let mut pick: Option<Material> = None;
        let mut delete: Option<PathBuf> = None;
        for folder in folders {
            let title = if folder.is_empty() { "My Materials".to_string() } else { folder.clone() };
            let examples = folder == EXAMPLES;
            let resp = egui::CollapsingHeader::new(RichText::new(title).small()).id_salt(("matlib", &folder)).default_open(!examples).show(ui, |ui| {
                if examples {
                    self.ensure_examples();
                }
                let list: Vec<(Option<PathBuf>, Material)> = if examples {
                    self.mat_library.examples.iter().flatten().map(|i| (None, i.mat.clone())).collect()
                } else {
                    self.mat_library.items.iter().filter(|i| i.folder == folder).map(|i| (i.path.clone(), i.mat.clone())).collect()
                };
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                    for (path, m) in &list {
                        let r = self.material_swatch(ui, &ctx, m, 22.0, cur.as_deref() == Some(m.name.as_str())).on_hover_text(describe(m));
                        if r.clicked() {
                            pick = Some(m.clone());
                        }
                        r.context_menu(|ui| {
                            if ui.button("Add to Model").clicked() {
                                pick = Some(m.clone());
                                ui.close();
                            }
                            if let Some(p) = path {
                                if ui.button("Show in Finder").clicked() {
                                    crate::ui::extensions::reveal(p);
                                    ui.close();
                                }
                                if ui.button("Delete…").clicked() {
                                    delete = Some(p.clone());
                                    ui.close();
                                }
                            }
                        });
                    }
                });
            });
            if examples {
                resp.header_response.on_hover_text("Materials bundled with Varstotas (the Material Gallery demo)");
            }
        }
        if let Some(m) = pick {
            self.use_material(&m);
        }
        if delete.is_some() {
            self.mat_library.delete_ask = delete;
        }
    }

    /// A material swatch: its colour, the texture's thumbnail on top.
    pub(crate) fn material_swatch(&mut self, ui: &mut Ui, ctx: &egui::Context, m: &Material, size: f32, selected: bool) -> egui::Response {
        let r = super::swatch(ui, super::color32(m.color), size, selected);
        if let Some(img) = &m.image
            && let Some(t) = self.thumbnail(ctx, img)
        {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(t, r.rect.shrink(3.0), uv, Color32::WHITE);
        }
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
        if let Some(c) = mark {
            let a = r.rect.right_top();
            let tri = vec![a + egui::vec2(-9.0, 2.0), a + egui::vec2(-2.0, 2.0), a + egui::vec2(-2.0, 9.0)];
            ui.painter().add(egui::Shape::convex_polygon(tri, c, egui::Stroke::new(0.5, Color32::from_black_alpha(90))));
        }
        r
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
