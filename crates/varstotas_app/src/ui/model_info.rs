//! Window → Model Info…: statistics of the document — what the scene shows (every placed copy
//! counted) and what the file stores (definitions, materials, textures …), the libraries — and
//! Purge Unused: remove the components and materials nothing in the scene uses.

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
}

impl Default for ModelInfoWindow {
    fn default() -> Self {
        Self { open: false, cache: None, purge_defs: true, purge_mats: true, last: None }
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
        let file_size = self.doc.path.as_ref().and_then(|p| std::fs::metadata(p).ok()).map(|m| m.len());
        let lib_components = self.library.items.len();
        let lib_materials = self.mat_library.items.iter().filter(|i| i.path.is_some()).count();
        let mut open = true;
        let mut purge = false;
        let pos = self.viewport.rect.left_top() + vec2(40.0, 40.0);
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
                if ui.add_enabled(can, egui::Button::new("Purge Unused")).clicked() {
                    purge = true;
                }
                if let Some(l) = &self.model_info.last {
                    ui.label(RichText::new(l).weak());
                }
            });
        });
        if purge {
            self.purge_unused(self.model_info.purge_defs, self.model_info.purge_mats);
        }
        if !open {
            self.model_info.open = false;
        }
    }
}
