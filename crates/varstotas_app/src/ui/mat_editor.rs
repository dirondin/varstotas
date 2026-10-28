//! Material Editor: the side bar card after Materials (and the same editor in a window). It
//! edits the current paint material: header (swatch, name, usage, eyedropper), Basic
//! (colour, opacity, texture tile), Physical (PBR, maps, shader).

use super::icons::Icon;
use super::{color_f32, color32, small_icon_button, theme};
use crate::app::VarstotasApp;
use crate::document::PaletteMaterial;
use egui::{Color32, RichText, ScrollArea, Sense, Stroke, Ui, vec2};
use varstotas_i18n::tr;
use varstotas_kernel::{EntityRef, MaterialId, Model};

/// Editor state kept between frames.
#[derive(Default)]
pub struct MatEditorState {
    /// The editor window is open.
    pub window: bool,
    /// Name being edited: (material, text).
    name: Option<(MaterialId, String)>,
    /// Colour being dragged in the picker (material name, colour): committed on release.
    color_draft: Option<(String, [f32; 4])>,
    /// Faces and objects painted with a material: (model version, material, count).
    usage: Option<(u64, MaterialId, usize)>,
    /// `model_version` the current material was last refreshed from the model at.
    synced: u64,
    /// The Textures section and the large view.
    pub maps: super::mat_maps::MapsState,
}

/// Faces (front or back) and objects painted with `id`, in every definition.
fn usage_count(model: &Model, id: MaterialId) -> usize {
    model
        .defs
        .values()
        .map(|d| {
            d.mesh.faces.values().filter(|f| f.front == Some(id) || f.back == Some(id)).count()
                + d.mesh.instances.values().filter(|i| i.material == Some(id)).count()
        })
        .sum()
}

impl VarstotasApp {
    /// The model material of the current paint material, if it is in the model.
    pub(crate) fn current_material_id(&self) -> Option<MaterialId> {
        let cur = self.doc.current_material.as_ref()?;
        self.doc.model.materials.iter().find(|(_, m)| m.name == cur.name).map(|(id, _)| id)
    }

    /// Eyedropper: the material of the first selected face (its front, else back) or object
    /// becomes the current paint material.
    pub(crate) fn pick_material_from_selection(&mut self) {
        let model = &self.doc.model;
        let Some(mesh) = model.mesh_at(&self.doc.active_path) else { return };
        let found = self.doc.selection.iter().find_map(|e| match *e {
            EntityRef::Face(f) => mesh.faces.get(f).and_then(|f| f.front.or(f.back)),
            EntityRef::Instance(i) => mesh.instances.get(i).and_then(|i| i.material),
            _ => None,
        });
        match found.and_then(|id| model.materials.get(id)) {
            Some(m) => {
                let name = m.name.clone();
                self.doc.current_material = Some(PaletteMaterial::of(m));
                self.flash(tr!("mated-picked", name = name));
            }
            None => self.flash(tr!("mated-pick-none")),
        }
    }

    /// Select the faces and objects of the active context painted with material `id`.
    fn select_material_faces(&mut self, id: MaterialId) {
        let Some(mesh) = self.doc.model.mesh_at(&self.doc.active_path) else { return };
        let mut sel: Vec<EntityRef> =
            mesh.faces.iter().filter(|(_, f)| f.front == Some(id) || f.back == Some(id)).map(|(k, _)| EntityRef::Face(k)).collect();
        sel.extend(mesh.instances.iter().filter(|(_, i)| i.material == Some(id)).map(|(k, _)| EntityRef::Instance(k)));
        let n = sel.len();
        self.doc.set_selection(sel);
        self.flash(tr!("mated-selected", count = n));
    }

    /// Set the colour (and opacity) of the current material: one undo step. A palette colour
    /// not yet in the model only changes the paint entry.
    fn set_material_color(&mut self, c: [f32; 4]) {
        let Some(cur) = self.doc.current_material.clone() else { return };
        match self.current_material_id() {
            Some(id) => {
                self.doc.commit(&tr!("mated-undo-color"), |m| m.materials[id].color = c);
                if let Some(m) = self.doc.model.materials.get(id) {
                    self.doc.current_material = Some(PaletteMaterial::of(m));
                }
            }
            None => self.doc.current_material = Some(PaletteMaterial { color: c, ..cur }),
        }
    }

    /// The Material Editor card.
    pub(crate) fn material_editor_card(&mut self, ui: &mut Ui) {
        self.material_editor(ui, false);
    }

    /// The Material Editor window (Open in window).
    pub(crate) fn material_editor_window(&mut self, ctx: &egui::Context) {
        if !self.mat_editor.window {
            return;
        }
        let mut open = true;
        egui::Window::new(tr!("mated-window-title"))
            .id(egui::Id::new("material_editor_window"))
            .open(&mut open)
            .default_size([440.0, 640.0])
            .resizable(true)
            .show(ctx, |ui| {
                ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| self.material_editor(ui, true));
            });
        self.mat_editor.window &= open;
    }

    fn material_editor(&mut self, ui: &mut Ui, big: bool) {
        // the paint material is a copy: refresh it after undo / redo and other model edits
        if self.mat_editor.synced != self.doc.model_version {
            self.mat_editor.synced = self.doc.model_version;
            if let Some(id) = self.current_material_id() {
                self.doc.current_material = Some(PaletteMaterial::of(&self.doc.model.materials[id]));
            }
        }
        let header = self.material_editor_header(ui, big);
        if !header {
            return;
        }
        ui.add_space(4.0);
        let basic = egui::CollapsingHeader::new(RichText::new(tr!("mated-basic")).strong())
            .id_salt(("mated_basic", big))
            .default_open(true)
            .show(ui, |ui| self.material_basic(ui));
        super::tray_lib::spot("mated:basic", basic.header_response.rect.center());
        let textures = egui::CollapsingHeader::new(RichText::new(tr!("mated-textures")).strong())
            .id_salt(("mated_textures", big))
            .default_open(true)
            .show(ui, |ui| self.material_textures(ui, big));
        super::tray_lib::spot("mated:textures", textures.header_response.rect.center());
        self.pbr_editor(ui);
        let stock = egui::CollapsingHeader::new(RichText::new(tr!("mated-stock")).strong())
            .id_salt(("mated_stock", big))
            .default_open(false)
            .show(ui, |ui| self.material_stock(ui));
        super::tray_lib::spot("mated:stock", stock.header_response.rect.center());
    }

    /// Swatch, name, usage and actions. False when there is no material to edit.
    fn material_editor_header(&mut self, ui: &mut Ui, big: bool) -> bool {
        let Some(cur) = self.doc.current_material.clone() else {
            ui.label(RichText::new(tr!("mated-none")).color(theme::text_dim()));
            if ui.button(tr!("mated-pick")).on_hover_text(tr!("mated-pick-tip")).clicked() {
                self.pick_material_from_selection();
            }
            return false;
        };
        let mid = self.current_material_id();
        let side = if big { 64.0 } else { 44.0 };
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(side, side), Sense::hover());
            let thumb = cur.image.as_ref().and_then(|img| self.thumbnail(ui.ctx(), img));
            match thumb {
                Some(t) => {
                    ui.painter().image(t, rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                    ui.painter().rect_stroke(rect, 6.0, Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
                }
                None => {
                    ui.painter().rect(rect, 6.0, color32(cur.color), Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
                }
            }
            ui.vertical(|ui| {
                match mid {
                    Some(id) => {
                        let buf = match &mut self.mat_editor.name {
                            Some((m, b)) if *m == id => b,
                            slot => &mut slot.insert((id, cur.name.clone())).1,
                        };
                        let resp = ui.add(egui::TextEdit::singleline(buf).desired_width(if big { 260.0 } else { 150.0 }).font(egui::TextStyle::Body));
                        super::tray_lib::spot("mated:name", resp.rect.center());
                        let text = buf.trim().to_string();
                        if super::tray::committed(ui, &resp) && !text.is_empty() && text != cur.name {
                            self.mat_editor.name = None;
                            self.rename_material(id, &text);
                        } else if !resp.has_focus() && *buf != cur.name {
                            *buf = cur.name.clone();
                        }
                        let version = self.doc.model_version;
                        let n = match self.mat_editor.usage {
                            Some((v, m, n)) if v == version && m == id => n,
                            _ => {
                                let n = usage_count(&self.doc.model, id);
                                self.mat_editor.usage = Some((version, id, n));
                                n
                            }
                        };
                        ui.label(RichText::new(tr!("mated-used", count = n)).small().color(theme::text_dim()));
                    }
                    None => {
                        ui.label(RichText::new(&cur.name).strong());
                        ui.label(RichText::new(tr!("mated-not-in-model")).small().color(theme::text_dim()));
                    }
                }
            });
        });
        ui.horizontal(|ui| {
            let pick = small_icon_button(ui, Icon::Paint, &tr!("mated-pick-tip"));
            super::tray_lib::spot("mated:pick", pick.rect.center());
            if pick.clicked() {
                self.pick_material_from_selection();
            }
            if let Some(id) = mid {
                let sel = small_icon_button(ui, Icon::SelectAll, &tr!("mated-select-tip"));
                super::tray_lib::spot("mated:select", sel.rect.center());
                if sel.clicked() {
                    self.select_material_faces(id);
                }
            }
            if !big {
                let win = small_icon_button(ui, Icon::SideBar, &tr!("mated-window-tip"));
                super::tray_lib::spot("mated:window", win.rect.center());
                if win.clicked() {
                    self.mat_editor.window = true;
                }
            }
        });
        true
    }

    /// Colour, opacity and the texture tile.
    fn material_basic(&mut self, ui: &mut Ui) {
        let Some(cur) = self.doc.current_material.clone() else { return };
        let shown = match &self.mat_editor.color_draft {
            Some((n, c)) if *n == cur.name => *c,
            _ => cur.color,
        };
        let mut c = shown;
        let mut changed = false;
        egui::Grid::new("mated_basic_grid").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(tr!("mated-color"));
            let mut c32 = color32([c[0], c[1], c[2], 1.0]);
            let resp = egui::color_picker::color_edit_button_srgba(ui, &mut c32, egui::color_picker::Alpha::Opaque);
            super::tray_lib::spot("mated:color", resp.rect.center());
            if resp.changed() {
                let f = color_f32(c32);
                c = [f[0], f[1], f[2], c[3]];
                changed = true;
            }
            ui.end_row();
            ui.label(tr!("mated-opacity"));
            let mut a = c[3];
            let resp = ui.add(egui::Slider::new(&mut a, 0.0..=1.0).fixed_decimals(2));
            super::tray_lib::spot("mated:opacity", resp.rect.left_center());
            if resp.changed() {
                c[3] = a;
                changed = true;
            }
            ui.end_row();
        });
        if changed {
            self.mat_editor.color_draft = Some((cur.name.clone(), c));
        }
        // the picker and the slider apply when the pointer is released
        let released = !ui.ctx().input(|i| i.pointer.any_down());
        if released
            && let Some((n, c)) = self.mat_editor.color_draft.take()
            && n == cur.name
            && c != cur.color
        {
            self.set_material_color(c);
        }
        ui.add_space(4.0);
        if cur.image.is_some() {
            ui.label(RichText::new(tr!("mated-texture")).strong());
            self.texture_size_editor(ui);
            ui.add_space(4.0);
            let r = egui::CollapsingHeader::new(tr!("mated-look")).id_salt("mated_look").show(ui, |ui| self.texture_look_editor(ui));
            super::tray_lib::spot("mated:look", r.header_response.rect.left_center() + egui::vec2(8.0, 0.0));
        } else {
            ui.label(RichText::new(tr!("mated-no-texture")).small().color(theme::text_dim()));
        }
    }
}
