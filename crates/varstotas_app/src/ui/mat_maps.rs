//! Material Editor → Textures: the base texture and every physical map of the current
//! material as a row (thumbnail, file, pixel size, size warning) with Load / Replace, Clear,
//! Export, Edit in an external program (reloaded when the file is saved), a large view, and
//! a map set loaded from a folder by file-name suffixes.

use super::icons::Icon;
use super::{small_icon_button, theme};
use crate::app::{Action, VarstotasApp};
use crate::document::PaletteMaterial;
use crate::textures::{LARGE_SIDE, SLOTS};
use egui::{Color32, RichText, Sense, Stroke, Ui, vec2};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};
use varstotas_i18n::tr;
use varstotas_kernel::{TexLook, TextureImage};

/// A texture opened in an external program: reloaded into its slot when the file changes.
pub struct ExternalEdit {
    pub material: String,
    pub slot: &'static str,
    pub path: PathBuf,
    modified: Option<SystemTime>,
}

/// State of the Textures section and the large view.
#[derive(Default)]
pub struct MapsState {
    /// The large view: (material, slot).
    pub view: Option<(String, &'static str)>,
    /// The large view repeats the tile 3 × 3.
    view_tiled: bool,
    /// Decoded image of the large view, keyed by the bytes it shows.
    view_tex: Option<(usize, egui::TextureHandle)>,
    pub edits: Vec<ExternalEdit>,
    last_poll: Option<Instant>,
    /// Look being dragged (material name, look): committed on release.
    look_draft: Option<(String, TexLook)>,
}

/// Label key of a slot.
pub fn slot_label(slot: &str) -> String {
    match slot {
        "base" => tr!("mated-slot-base"),
        "normal" => tr!("realistic-map-normal"),
        "roughness" => tr!("realistic-map-roughness"),
        "metallic" => tr!("realistic-map-metallic"),
        "ao" => tr!("realistic-map-occlusion"),
        _ => tr!("realistic-map-emission"),
    }
}

fn size_text(bytes: usize) -> String {
    if bytes >= 1 << 20 { format!("{:.1} MB", bytes as f64 / (1 << 20) as f64) } else { format!("{} KB", bytes.div_ceil(1024)) }
}

/// Where a texture opened for external editing is written.
fn edit_dir() -> Option<PathBuf> {
    crate::bridge::home_dir().map(|d| d.join("cache").join("edit"))
}

impl VarstotasApp {
    /// Put images into slots of material `name` (one undo step). A base texture keeps the tile
    /// width of the one it replaces; a new base texture also sets the swatch colour; maps
    /// follow the base tile size. `None` clears a slot.
    pub(crate) fn set_material_slots(&mut self, name: &str, slots: Vec<(&'static str, Option<TextureImage>)>, undo: &str) {
        let pm = match self.doc.model.materials.values().find(|m| m.name == name) {
            Some(m) => PaletteMaterial::of(m),
            None => match self.doc.current_material.clone().filter(|c| c.name == name) {
                Some(c) => c,
                None => return,
            },
        };
        let base_color = slots
            .iter()
            .find(|(s, t)| *s == "base" && t.is_some() && pm.image.is_none())
            .and_then(|(_, t)| crate::textures::decode(&t.as_ref()?.bytes).ok())
            .map(|d| crate::textures::average_color(&d));
        self.doc.commit(undo, |m| {
            let id = pm.ensure_in(m);
            let mat = &mut m.materials[id];
            for (slot, tex) in slots {
                if slot == "base" {
                    let tex = tex.map(|mut t| {
                        if let Some(old) = &mat.image {
                            let w = old.size[0];
                            t.size = [w, w * t.size[1] / t.size[0].max(1e-9)];
                        }
                        t
                    });
                    if let Some(c) = base_color {
                        mat.color = [c[0], c[1], c[2], mat.color[3]];
                    }
                    mat.image = tex;
                    continue;
                }
                let width = mat.image.as_ref().map_or(1000.0, |i| i.size[0]);
                let mut p = mat.pbr_or_default();
                let loaded = tex.is_some();
                if let Some(s) = p.map_mut(slot) {
                    *s = tex.map(|mut t| {
                        t.size = [width, width * t.size[1] / t.size[0].max(1e-9)];
                        t
                    });
                }
                // a loaded map decides: its factor becomes 1
                if loaded {
                    match slot {
                        "roughness" => p.roughness = 1.0,
                        "metallic" => p.metallic = 1.0,
                        "emission" if p.emission == [0.0; 3] => p.emission = [1.0; 3],
                        _ => {}
                    }
                }
                mat.pbr = Some(p.sanitized());
            }
        });
        if let Some(m) = self.doc.model.materials.values().find(|m| m.name == pm.name) {
            self.doc.current_material = Some(PaletteMaterial::of(m));
        }
    }

    /// Load an image file into a slot of material `name`.
    pub(crate) fn load_slot_file(&mut self, name: &str, slot: &'static str, path: &Path) {
        match crate::textures::load_file(path, 1000.0) {
            Ok((tex, _)) => self.set_material_slots(name, vec![(slot, Some(tex))], &tr!("mated-undo-texture")),
            Err(e) => self.flash_error(tr!("realistic-cant-use", path = path.display().to_string(), error = e)),
        }
    }

    /// Load the images of a folder into the slots their names suggest (one undo step).
    pub(crate) fn load_map_folder(&mut self, name: &str, dir: &Path) {
        let mut slots = Vec::new();
        for (slot, path) in crate::textures::map_set_in(dir) {
            match crate::textures::load_file(&path, 1000.0) {
                Ok((tex, _)) => slots.push((slot, Some(tex))),
                Err(e) => log::warn!("map set {}: {e}", path.display()),
            }
        }
        if slots.is_empty() {
            self.flash_error(tr!("mated-no-maps", path = dir.display().to_string()));
            return;
        }
        let n = slots.len();
        self.set_material_slots(name, slots, &tr!("mated-undo-map-set"));
        self.flash(tr!("mated-maps-loaded", count = n));
    }

    /// Write the image of a slot to `path`.
    pub(crate) fn export_slot(&mut self, name: &str, slot: &str, path: &Path) {
        let Some(m) = self.doc.model.materials.values().find(|m| m.name == name) else { return };
        let Some(img) = m.slot_image(slot).cloned() else { return };
        match varstotas_kernel::io::atomic_write(path, &img.bytes[..]) {
            Ok(()) => self.flash(tr!("mated-exported", path = path.display().to_string())),
            Err(e) => self.flash_error(tr!("realistic-cant-read", path = path.display().to_string(), error = e.to_string())),
        }
    }

    /// Re-encode a large image of a slot at [`LARGE_SIDE`] pixels.
    pub(crate) fn downscale_slot(&mut self, name: &str, slot: &'static str) {
        let Some(m) = self.doc.model.materials.values().find(|m| m.name == name) else { return };
        let Some(img) = m.slot_image(slot).cloned() else { return };
        match crate::textures::downscale(&img.bytes, LARGE_SIDE) {
            Ok(bytes) => {
                let mut t = TextureImage::new(bytes, img.file_name.clone(), img.size[0]);
                t.size = img.size;
                self.set_material_slots(name, vec![(slot, Some(t))], &tr!("mated-undo-downscale"));
            }
            Err(e) => self.flash_error(e),
        }
    }

    /// Open the image of a slot in the image editor (Preferences → Render → Programs, else the
    /// system's default); saving it there reloads the slot.
    pub(crate) fn edit_slot_externally(&mut self, name: &str, slot: &'static str) {
        let Some(m) = self.doc.model.materials.values().find(|m| m.name == name) else { return };
        let Some(img) = m.slot_image(slot).cloned() else { return };
        let Some(dir) = edit_dir() else { return };
        let safe: String = name.chars().map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' }).collect();
        let path = dir.join(format!("{safe}-{slot}.{}", img.extension()));
        if let Err(e) = std::fs::create_dir_all(&dir).and_then(|()| varstotas_kernel::io::atomic_write(&path, &img.bytes[..])) {
            self.flash_error(tr!("realistic-cant-read", path = path.display().to_string(), error = e.to_string()));
            return;
        }
        let modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        let st = &mut self.mat_editor.maps;
        st.edits.retain(|e| !(e.material == name && e.slot == slot));
        st.edits.push(ExternalEdit { material: name.to_string(), slot, path: path.clone(), modified });
        // a scripted run only records the file (another window would cover it)
        if self.debug.script_active() {
            log::info!("edit externally {}", path.display());
            return;
        }
        let spawned = match &self.prefs.image_editor_path {
            Some(prog) if cfg!(target_os = "macos") && prog.extension().is_some_and(|e| e == "app") => {
                std::process::Command::new("open").arg("-a").arg(prog).arg(&path).spawn()
            }
            Some(prog) => std::process::Command::new(prog).arg(&path).spawn(),
            None if cfg!(target_os = "macos") => std::process::Command::new("open").arg(&path).spawn(),
            None if cfg!(windows) => std::process::Command::new("cmd").args(["/C", "start", ""]).arg(&path).spawn(),
            None => std::process::Command::new("xdg-open").arg(&path).spawn(),
        };
        match spawned {
            Ok(_) => self.flash(tr!("mated-editing", file = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default())),
            Err(e) => self.flash_error(tr!("mated-cant-open-editor", error = e.to_string())),
        }
    }

    /// Reload textures saved in the external editor (checked twice a second).
    pub(crate) fn poll_external_edits(&mut self) {
        let st = &mut self.mat_editor.maps;
        if st.edits.is_empty() || st.last_poll.is_some_and(|t| t.elapsed().as_millis() < 500) {
            return;
        }
        st.last_poll = Some(Instant::now());
        let mut changed = Vec::new();
        for e in &mut st.edits {
            let now = std::fs::metadata(&e.path).and_then(|m| m.modified()).ok();
            if now.is_some() && now != e.modified {
                e.modified = now;
                changed.push((e.material.clone(), e.slot, e.path.clone()));
            }
        }
        for (name, slot, path) in changed {
            let Ok(bytes) = std::fs::read(&path) else { continue };
            if crate::textures::decode(&bytes).is_err() {
                continue; // half-written: the next poll retries
            }
            let old = self.doc.model.materials.values().find(|m| m.name == name).and_then(|m| m.slot_image(slot).cloned());
            let Some(old) = old else { continue };
            let mut t = TextureImage::new(bytes, old.file_name.clone(), old.size[0]);
            t.size[1] = old.size[0] * t.size[1] / t.size[0].max(1e-9);
            t.size[0] = old.size[0];
            self.set_material_slots(&name, vec![(slot, Some(t))], &tr!("mated-undo-reload"));
            self.flash(tr!("mated-reloaded", slot = slot_label(slot)));
        }
        self.mat_editor.maps.edits.retain(|e| e.path.exists());
    }

    /// Set the base texture look of the current material: one undo step.
    pub(crate) fn set_texture_look(&mut self, look: TexLook) {
        let Some(cur) = self.doc.current_material.clone() else { return };
        match self.current_material_id() {
            Some(id) => {
                self.doc.commit(&tr!("mated-undo-look"), |m| {
                    if let Some(t) = &mut m.materials[id].image {
                        t.look = look;
                    }
                });
                if let Some(m) = self.doc.model.materials.get(id) {
                    self.doc.current_material = Some(PaletteMaterial::of(m));
                }
            }
            None => {
                let mut c = cur;
                if let Some(t) = &mut c.image {
                    t.look = look;
                }
                self.doc.current_material = Some(c);
            }
        }
    }

    /// Colour adjustments, cut-out, decal and world mapping of the base texture.
    pub(crate) fn texture_look_editor(&mut self, ui: &mut Ui) {
        let Some(cur) = self.doc.current_material.clone() else { return };
        let Some(img) = &cur.image else { return };
        let mut l = match &self.mat_editor.maps.look_draft {
            Some((n, l)) if *n == cur.name => *l,
            _ => img.look,
        };
        let before = l;
        let mut toggled = false;
        egui::Grid::new("mated_look_grid").num_columns(2).spacing([10.0, 4.0]).show(ui, |ui| {
            let slider = |ui: &mut Ui, label: String, v: &mut f32, range: std::ops::RangeInclusive<f32>, id: &str| {
                ui.label(label);
                let r = ui.add(egui::Slider::new(v, range).fixed_decimals(if id == "hue" { 0 } else { 2 }));
                super::tray_lib::spot(format!("look:{id}"), r.rect.left_center());
                ui.end_row();
            };
            slider(ui, tr!("mated-hue"), &mut l.hue, -180.0..=180.0, "hue");
            slider(ui, tr!("mated-saturation"), &mut l.saturation, -1.0..=1.0, "saturation");
            slider(ui, tr!("mated-lightness"), &mut l.lightness, -1.0..=1.0, "lightness");
            slider(ui, tr!("mated-cutoff"), &mut l.alpha_cutoff, 0.0..=1.0, "cutoff");
        });
        let mut check = |ui: &mut Ui, v: &mut bool, label: String, tip: String, id: &str| {
            let r = ui.checkbox(v, label).on_hover_text(tip);
            super::tray_lib::spot(format!("look:{id}"), r.rect.left_center());
            toggled |= r.changed();
        };
        check(ui, &mut l.colorize, tr!("mated-colorize"), tr!("mated-colorize-tip"), "colorize");
        check(ui, &mut l.decal, tr!("mated-decal"), tr!("mated-decal-tip"), "decal");
        check(ui, &mut l.world, tr!("mated-world"), tr!("mated-world-tip"), "world");
        if !l.is_default() {
            let r = ui.small_button(tr!("mated-look-reset"));
            super::tray_lib::spot("look:reset", r.rect.center());
            if r.clicked() {
                l = TexLook::default();
                toggled = true;
            }
        }
        if toggled {
            self.mat_editor.maps.look_draft = None;
            self.set_texture_look(l);
            return;
        }
        if l != before {
            self.mat_editor.maps.look_draft = Some((cur.name.clone(), l));
        }
        // sliders apply when the pointer is released
        let released = !ui.ctx().input(|i| i.pointer.any_down());
        if released
            && let Some((n, l)) = self.mat_editor.maps.look_draft.take()
            && n == cur.name
            && l != img.look
        {
            self.set_texture_look(l);
        }
    }

    /// The Textures section of the Material Editor.
    pub(crate) fn material_textures(&mut self, ui: &mut Ui, big: bool) {
        let Some(cur) = self.doc.current_material.clone() else { return };
        let name = cur.name.clone();
        for slot in SLOTS {
            let img = cur.slot_image(slot).cloned();
            ui.horizontal(|ui| {
                let side = if big { 56.0 } else { 40.0 };
                let (rect, resp) = ui.allocate_exact_size(vec2(side, side), Sense::click());
                super::tray_lib::spot(format!("slot:{slot}"), rect.center());
                match img.as_ref().and_then(|i| self.thumbnail(ui.ctx(), i)) {
                    Some(t) => {
                        ui.painter().image(t, rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                        ui.painter().rect_stroke(rect, 4.0, Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
                    }
                    None => {
                        ui.painter().rect_stroke(rect, 4.0, Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
                        ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "—", egui::FontId::proportional(13.0), theme::text_dim()); // i18n-ignore
                    }
                }
                if img.is_some() && resp.on_hover_text(tr!("mated-view-tip")).clicked() {
                    self.mat_editor.maps.view = Some((name.clone(), slot));
                }
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(slot_label(slot)).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            let tip = if img.is_some() { tr!("mated-replace-tip") } else { tr!("mated-load-tip") };
                            let load = small_icon_button(ui, Icon::FileOpen, &tip);
                            super::tray_lib::spot(format!("slot_load:{slot}"), load.rect.center());
                            if load.clicked() {
                                self.actions.push(Action::PickSlotFile(name.clone(), slot));
                            }
                            if img.is_some() {
                                let clear = small_icon_button(ui, Icon::Trash, &tr!("mated-clear-tip"));
                                super::tray_lib::spot(format!("slot_clear:{slot}"), clear.rect.center());
                                if clear.clicked() {
                                    self.set_material_slots(&name, vec![(slot, None)], &tr!("mated-undo-clear"));
                                }
                                let edit = small_icon_button(ui, Icon::Drawing, &tr!("mated-edit-tip"));
                                super::tray_lib::spot(format!("slot_edit:{slot}"), edit.rect.center());
                                if edit.clicked() {
                                    self.edit_slot_externally(&name, slot);
                                }
                                if small_icon_button(ui, Icon::FileSaveAs, &tr!("mated-export-tip")).clicked() {
                                    self.actions.push(Action::ExportSlot(name.clone(), slot));
                                }
                            }
                        });
                    });
                    match &img {
                        Some(i) => {
                            let large = i.pixels[0].max(i.pixels[1]) > LARGE_SIDE;
                            let info = format!("{} · {} × {} px · {}", i.file_name, i.pixels[0], i.pixels[1], size_text(i.bytes.len())); // i18n-ignore
                            ui.add(egui::Label::new(RichText::new(info).small().color(if large { theme::warning() } else { theme::text_dim() })).truncate());
                            if large {
                                let b = ui.small_button(tr!("mated-downscale")).on_hover_text(tr!("mated-downscale-tip", side = LARGE_SIDE));
                                super::tray_lib::spot(format!("slot_down:{slot}"), b.rect.center());
                                if b.clicked() {
                                    self.downscale_slot(&name, slot);
                                }
                            }
                        }
                        None => {
                            ui.label(RichText::new(tr!("mated-slot-empty")).small().color(theme::text_dim()));
                        }
                    }
                });
            });
            ui.add_space(2.0);
        }
        let b = ui.button(tr!("mated-map-set")).on_hover_text(tr!("mated-map-set-tip"));
        super::tray_lib::spot("mated:map_set", b.rect.center());
        if b.clicked() {
            self.actions.push(Action::PickMapFolder(name));
        }
        if !self.mat_editor.maps.edits.is_empty() {
            ui.label(RichText::new(tr!("mated-watching", count = self.mat_editor.maps.edits.len())).small().color(theme::text_dim()));
        }
    }

    /// The large view of one texture: fitted (or 3 × 3 tiles), with its file and sizes.
    pub(crate) fn texture_view_window(&mut self, ctx: &egui::Context) {
        let Some((name, slot)) = self.mat_editor.maps.view.clone() else { return };
        let img = self.doc.model.materials.values().find(|m| m.name == name).and_then(|m| m.slot_image(slot).cloned());
        let Some(img) = img else {
            self.mat_editor.maps.view = None;
            return;
        };
        let key = std::sync::Arc::as_ptr(&img.bytes) as *const u8 as usize;
        if self.mat_editor.maps.view_tex.as_ref().is_none_or(|(k, _)| *k != key) {
            let Ok(d) = crate::textures::decode(&img.bytes) else {
                self.mat_editor.maps.view = None;
                return;
            };
            let ci = egui::ColorImage::from_rgba_unmultiplied([d.0 as usize, d.1 as usize], &d.2);
            self.mat_editor.maps.view_tex = Some((key, ctx.load_texture(format!("texview{key}"), ci, egui::TextureOptions::LINEAR_REPEAT)));
        }
        let Some((_, tex)) = &self.mat_editor.maps.view_tex else { return };
        let tex = tex.id();
        let unit = self.doc.model.unit;
        let mut open = true;
        egui::Window::new(tr!("mated-view-title", material = name, slot = slot_label(slot)))
            .id(egui::Id::new("texture_view_window"))
            .open(&mut open)
            .default_size([520.0, 560.0])
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&img.file_name).strong());
                    ui.label(
                        RichText::new(format!(
                            "{} × {} px · {} · {} × {}", // i18n-ignore
                            img.pixels[0],
                            img.pixels[1],
                            size_text(img.bytes.len()),
                            super::fmt_len(img.size[0], unit),
                            super::fmt_len(img.size[1], unit)
                        ))
                        .small()
                        .color(theme::text_dim()),
                    );
                });
                ui.checkbox(&mut self.mat_editor.maps.view_tiled, tr!("mated-view-tiled"));
                let avail = ui.available_size() - vec2(0.0, 4.0);
                let aspect = img.pixels[1].max(1) as f32 / img.pixels[0].max(1) as f32;
                let w = avail.x.min(avail.y / aspect).max(64.0);
                let (rect, _) = ui.allocate_exact_size(vec2(w, w * aspect), Sense::hover());
                let k = if self.mat_editor.maps.view_tiled { 3.0 } else { 1.0 };
                ui.painter().image(tex, rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(k, k)), Color32::WHITE);
                ui.painter().rect_stroke(rect, 0.0, Stroke::new(1.0, theme::border()), egui::StrokeKind::Outside);
            });
        if !open {
            self.mat_editor.maps.view = None;
            self.mat_editor.maps.view_tex = None;
        }
    }
}
