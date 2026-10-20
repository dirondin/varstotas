//! Window → Model Info…: statistics of the document — what the scene shows (every placed copy
//! counted) and what the file stores (definitions, materials, textures …), the libraries — and
//! Purge Unused: remove the components and materials nothing in the scene uses. Also File →
//! Preview Image → Show…: the document's preview picture in a window of its own.

use crate::app::VarstotasApp;
use egui::{Color32, Context, RichText, vec2};
use varstotas_kernel::stats::ModelStats;

pub struct ModelInfoWindow {
    pub open: bool,
    /// Statistics for a model version.
    cache: Option<(u64, ModelStats)>,
    /// What Purge Unused removes.
    pub purge_defs: bool,
    pub purge_mats: bool,
    /// The last purge's report (for the window and the harness).
    pub last: Option<String>,
    /// File → Preview Image → Show… window.
    pub preview_open: bool,
    /// The preview picture as a texture, for its bytes (pointer) — (key, texture, pixel size).
    preview_tex: Option<(usize, egui::TextureHandle, [usize; 2])>,
}

impl Default for ModelInfoWindow {
    fn default() -> Self {
        Self { open: false, cache: None, purge_defs: true, purge_mats: true, last: None, preview_open: false, preview_tex: None }
    }
}

/// `1234567` → `1 234 567`.
fn n(v: usize) -> String {
    let s = v.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('\u{2009}');
        }
        out.push(c);
    }
    out
}

fn bytes(b: u64) -> String {
    match b {
        0..1024 => format!("{b} B"),
        1024..1_048_576 => format!("{:.0} KB", b as f64 / 1024.0),
        _ => format!("{:.1} MB", b as f64 / 1_048_576.0),
    }
}

/// "used of all" with the unused part marked.
fn used_of(used: usize, all: usize) -> RichText {
    if used == all { RichText::new(n(all)) } else { RichText::new(format!("{} of {} ({} unused)", n(used), n(all), n(all - used))).color(Color32::from_rgb(190, 110, 20)) }
}

impl VarstotasApp {
    /// The document's statistics (recomputed when the model changes).
    pub(crate) fn model_stats(&mut self) -> &ModelStats {
        let v = self.doc.model_version;
        if self.model_info.cache.as_ref().is_none_or(|c| c.0 != v) {
            self.model_info.cache = Some((v, self.doc.model.statistics()));
        }
        &self.model_info.cache.as_ref().unwrap().1
    }

    /// Component and group definitions nothing in the model uses.
    pub(crate) fn unused_defs(&mut self) -> usize {
        let s = self.model_stats();
        (s.component_defs - s.component_defs_used) + (s.group_defs - s.group_defs_used)
    }

    /// Purge Unused (one undo step): the unused components / materials the window's switches
    /// choose.
    pub(crate) fn purge_unused(&mut self, defs: bool, mats: bool) {
        let s = self.model_stats();
        let has = (defs && (s.component_defs_used < s.component_defs || s.group_defs_used < s.group_defs)) || (mats && s.materials_used < s.materials);
        let msg = if !has {
            "Nothing unused to purge".to_string()
        } else {
            match self.doc.commit("Purge Unused", |m| m.purge_unused(defs, mats)) {
                Some(p) => {
                    self.library.def_thumbs.clear();
                    let mut parts = Vec::new();
                    if p.components > 0 {
                        parts.push(format!("{} component(s)", p.components));
                    }
                    if p.groups > 0 {
                        parts.push(format!("{} group(s)", p.groups));
                    }
                    if p.materials > 0 {
                        parts.push(format!("{} material(s)", p.materials));
                    }
                    format!("Purged {}", parts.join(", "))
                }
                None => "Purge failed".to_string(),
            }
        };
        self.flash(msg.clone());
        self.model_info.last = Some(msg);
    }

    /// The document's preview picture as a texture (decoded again when it changes) and its
    /// pixel size; None without one or when it can't be read.
    fn doc_preview_texture(&mut self, ctx: &Context) -> Option<(egui::TextureId, [usize; 2])> {
        let Some(img) = self.doc.model.preview.as_ref() else {
            self.model_info.preview_tex = None;
            return None;
        };
        let key = std::sync::Arc::as_ptr(&img.bytes) as *const u8 as usize;
        if self.model_info.preview_tex.as_ref().is_none_or(|t| t.0 != key) {
            let ci = crate::textures::thumbnail(&img.bytes, crate::textures::PREVIEW_SIDE)?;
            let size = ci.size;
            let tex = ctx.load_texture("doc_preview", ci, egui::TextureOptions::LINEAR);
            self.model_info.preview_tex = Some((key, tex, size));
        }
        self.model_info.preview_tex.as_ref().map(|t| (t.1.id(), t.2))
    }

    /// "512 × 384 JPEG, 41 KB" for the document's preview picture.
    fn doc_preview_label(&self, size: [usize; 2]) -> String {
        let b = self.doc.model.preview.as_ref().map(|i| &i.bytes[..]).unwrap_or_default();
        let kind = if b.starts_with(b"\x89PNG") { "PNG" } else { "JPEG" };
        format!("{} × {} {kind}, {}", size[0], size[1], bytes(b.len() as u64))
    }

    /// File → Preview Image → Show…: the picture the Components library shows for this file,
    /// with the buttons that set or remove it.
    pub(crate) fn doc_preview_window(&mut self, ctx: &Context) {
        if !self.model_info.preview_open {
            return;
        }
        let tex = self.doc_preview_texture(ctx);
        let label = tex.map(|(_, s)| self.doc_preview_label(s));
        let mut open = true;
        let mut action = None;
        let pos = self.viewport.view_rect.left_top() + vec2(60.0, 60.0);
        let mut win = egui::Window::new("Preview Image").id(egui::Id::new("doc_preview_window")).open(&mut open).collapsible(false).resizable(false).default_pos(pos);
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        win.show(ctx, |ui| {
            ui.set_width(360.0);
            match tex {
                Some((id, size)) => {
                    let k = (360.0 / size[0].max(size[1]) as f32).min(1.0);
                    ui.vertical_centered(|ui| ui.image((id, vec2(size[0] as f32 * k, size[1] as f32 * k))));
                    ui.label(RichText::new(label.unwrap_or_default()).weak());
                }
                None => {
                    let (r, _) = ui.allocate_exact_size(vec2(360.0, 160.0), egui::Sense::hover());
                    ui.painter().rect_filled(r, 4.0, ui.visuals().faint_bg_color);
                    ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, "No preview image — the Components library draws the model", egui::FontId::proportional(12.0), ui.visuals().weak_text_color());
                }
            }
            ui.label(RichText::new("Saved with the file; the Components library shows it for this file.").weak().small());
            ui.horizontal(|ui| {
                if ui.button("From Image File…").clicked() {
                    action = Some(crate::app::Action::PreviewFromFile);
                }
                if ui.button("From Current View").clicked() {
                    action = Some(crate::app::Action::PreviewFromView);
                }
                if ui.add_enabled(tex.is_some(), egui::Button::new("Remove")).clicked() {
                    action = Some(crate::app::Action::PreviewRemove);
                }
            });
        });
        self.actions.extend(action);
        if !open {
            self.model_info.preview_open = false;
        }
    }

    pub(crate) fn model_info_window(&mut self, ctx: &Context) {
        if !self.model_info.open {
            return;
        }
        // The libraries are counted from their scans.
        if self.library.scanned.is_none() {
            self.refresh_library();
        }
        if self.mat_library.scanned.is_none() {
            self.refresh_materials();
        }
        let s = self.model_stats().clone();
        let preview = self.doc_preview_texture(ctx);
        let preview_label = preview.map(|(_, sz)| self.doc_preview_label(sz));
        let mut show_preview = false;
        let file_size = self.doc.path.as_ref().and_then(|p| std::fs::metadata(p).ok()).map(|m| m.len());
        let lib_components = self.library.items.len();
        let lib_materials = self.mat_library.items.iter().filter(|i| i.path.is_some()).count();
        let mut open = true;
        let mut purge = false;
        let pos = self.viewport.view_rect.left_top() + vec2(40.0, 40.0);
        let mut win = egui::Window::new("Model Info").id(egui::Id::new("model_info_window")).open(&mut open).collapsible(false).resizable(false).default_pos(pos);
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        win.show(ctx, |ui| {
            ui.set_width(380.0);
            let row = |ui: &mut egui::Ui, k: &str, v: RichText| {
                ui.label(k);
                ui.label(v);
                ui.end_row();
            };
            ui.label(RichText::new("In the scene").strong()).on_hover_text("What is drawn: the content of every placed copy counted");
            egui::Grid::new("mi_scene").num_columns(2).spacing([16.0, 3.0]).show(ui, |ui| {
                row(ui, "Polygons (faces)", RichText::new(n(s.scene.faces)));
                row(ui, "Triangles", RichText::new(n(s.scene.triangles)));
                row(ui, "Edges", RichText::new(n(s.scene.edges)));
                row(ui, "Groups placed", RichText::new(n(s.scene.groups)));
                row(ui, "Components placed", RichText::new(n(s.scene.components)));
            });
            ui.add_space(6.0);
            ui.label(RichText::new("In the file").strong()).on_hover_text("What the document stores: each definition once, used or not");
            egui::Grid::new("mi_file").num_columns(2).spacing([16.0, 3.0]).show(ui, |ui| {
                row(ui, "Polygons (faces)", RichText::new(n(s.stored.faces)));
                row(ui, "Triangles", RichText::new(n(s.stored.triangles)));
                row(ui, "Components", used_of(s.component_defs_used, s.component_defs));
                row(ui, "Groups", used_of(s.group_defs_used, s.group_defs));
                row(ui, "Materials", used_of(s.materials_used, s.materials));
                if s.textures > 0 {
                    row(ui, "Textures", RichText::new(format!("{} ({})", n(s.textures), bytes(s.texture_bytes as u64))));
                }
                row(ui, "Tags", RichText::new(n(s.tags)));
                row(ui, "Scenes", RichText::new(n(s.scenes)));
                if s.sheets > 0 {
                    row(ui, "Layout sheets", RichText::new(n(s.sheets)));
                }
                if s.generators > 0 {
                    row(ui, "Generators", RichText::new(n(s.generators)));
                }
                if s.lights > 0 {
                    row(ui, "Lights", RichText::new(n(s.lights)));
                }
                if s.guides + s.dims > 0 {
                    row(ui, "Guides / dimensions", RichText::new(format!("{} / {}", n(s.guides), n(s.dims))));
                }
                if let Some(b) = file_size {
                    row(ui, "File size (saved)", RichText::new(bytes(b)));
                }
                ui.label("Preview image");
                ui.horizontal(|ui| {
                    if let Some((id, size)) = preview {
                        let k = 48.0 / size[0].max(size[1]) as f32;
                        ui.image((id, vec2(size[0] as f32 * k, size[1] as f32 * k)));
                        ui.label(RichText::new(preview_label.unwrap_or_default()).weak());
                    } else {
                        ui.label(RichText::new("none").weak());
                    }
                    show_preview = ui.link("Show…").on_hover_text("File → Preview Image → Show…").clicked();
                });
                ui.end_row();
            });
            ui.add_space(6.0);
            ui.label(RichText::new("Libraries").strong());
            egui::Grid::new("mi_libs").num_columns(2).spacing([16.0, 3.0]).show(ui, |ui| {
                row(ui, "Components library", RichText::new(format!("{} file(s)", n(lib_components))));
                row(ui, "Materials library", RichText::new(format!("{} file(s)", n(lib_materials))));
            });
            ui.separator();
            ui.label(RichText::new("Purge Unused").strong());
            ui.label(RichText::new("Removes what nothing in the scene uses (Edit → Undo brings it back).").weak());
            let unused_defs = s.component_defs - s.component_defs_used + s.group_defs - s.group_defs_used;
            let unused_mats = s.materials - s.materials_used;
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.model_info.purge_defs, format!("Components and groups ({unused_defs})"));
                ui.checkbox(&mut self.model_info.purge_mats, format!("Materials ({unused_mats})"));
            });
            ui.horizontal(|ui| {
                let can = (self.model_info.purge_defs && unused_defs > 0) || (self.model_info.purge_mats && unused_mats > 0);
                let r = ui.add_enabled(can, egui::Button::new("Purge Unused"));
                crate::ui::tray_lib::spot("purge", r.rect.center());
                if r.clicked() {
                    purge = true;
                }
                if let Some(l) = &self.model_info.last {
                    ui.label(RichText::new(l).weak());
                }
            });
        });
        if show_preview {
            self.model_info.preview_open = true;
        }
        if purge {
            self.purge_unused(self.model_info.purge_defs, self.model_info.purge_mats);
        }
        if !open {
            self.model_info.open = false;
        }
    }
}
