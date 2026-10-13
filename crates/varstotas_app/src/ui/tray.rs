//! The "Default Tray": Entity Info, Materials, Tags, Outliner, Scenes, Instructor.

use super::icons::{Icon, paint_icon};
use super::{color_f32, color32, fmt_area, fmt_len, small_icon_button, swatch, theme};
use crate::app::{Action, VarstotasApp};
use crate::document::{PaletteMaterial, guarded};
use egui::{Color32, Frame, Margin, RichText, ScrollArea, Sense, Stroke, Ui, vec2};
use varstotas_kernel::{ContextPath, EntityRef, InstanceId, LengthUnit, Mesh, Model, TagId};

#[derive(Default)]
pub struct TrayState {
    pub selected_scene: Option<usize>,
    pub close_requested: bool,
    tag_rename: Option<(TagId, String)>,
    scene_rename: Option<(usize, String)>,
    inst_name: Option<(InstanceId, String)>,
    custom_color: Option<Color32>,
}

/// Default palette (name, sRGB).
const PALETTE: [(&str, [u8; 3]); 24] = [
    ("White", [0xFF, 0xFF, 0xFF]),
    ("Light Grey", [0xD4, 0xD6, 0xDA]),
    ("Grey", [0x9A, 0x9E, 0xA6]),
    ("Charcoal", [0x4A, 0x4E, 0x56]),
    ("Black", [0x1C, 0x1D, 0x20]),
    ("Concrete", [0xB8, 0xB4, 0xAA]),
    ("Brick", [0xB4, 0x54, 0x3C]),
    ("Terracotta", [0xD8, 0x7A, 0x52]),
    ("Orange", [0xF0, 0x9A, 0x36]),
    ("Sand", [0xE8, 0xD3, 0xA2]),
    ("Yellow", [0xF6, 0xD8, 0x4A]),
    ("Olive", [0x8E, 0x98, 0x48]),
    ("Grass", [0x6E, 0xB0, 0x4C]),
    ("Forest", [0x2F, 0x6E, 0x3E]),
    ("Teal", [0x2E, 0x9A, 0x96]),
    ("Sky", [0x8C, 0xC4, 0xEC]),
    ("Water", [0x3A, 0x84, 0xC8]),
    ("Navy", [0x24, 0x3C, 0x74]),
    ("Lavender", [0xB4, 0xA8, 0xE0]),
    ("Purple", [0x7A, 0x4E, 0xA8]),
    ("Pink", [0xF0, 0xA0, 0xB8]),
    ("Light Wood", [0xE2, 0xC2, 0x8E]),
    ("Wood", [0xB8, 0x86, 0x52]),
    ("Dark Wood", [0x6E, 0x48, 0x2C]),
];

fn rgb(c: [u8; 3]) -> [f32; 4] {
    [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, 1.0]
}

fn card<R>(ui: &mut Ui, title: &str, icon: Icon, default_open: bool, body: impl FnOnce(&mut Ui) -> R) {
    Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(8.0)
        .inner_margin(Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let id = ui.make_persistent_id(("tray", title));
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, default_open)
                .show_header(ui, |ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                    paint_icon(ui.painter(), rect, icon, false);
                    ui.label(RichText::new(title).strong());
                })
                .body(|ui| {
                    ui.add_space(2.0);
                    body(ui);
                });
        });
    ui.add_space(6.0);
}

fn tag_name(model: &Model, t: Option<TagId>) -> String {
    t.and_then(|t| model.tags.get(t)).map(|t| t.name.clone()).unwrap_or_else(|| "Untagged".into())
}

/// Tag picker; returns Some(new) when changed.
fn tag_combo(ui: &mut Ui, model: &Model, current: Option<TagId>, salt: &str) -> Option<Option<TagId>> {
    let mut out = None;
    egui::ComboBox::from_id_salt(("tag_combo", salt))
        .selected_text(tag_name(model, current))
        .width(150.0)
        .show_ui(ui, |ui| {
            if ui.selectable_label(current.is_none(), "Untagged").clicked() {
                out = Some(None);
            }
            for (id, t) in model.tags.iter() {
                if ui.selectable_label(current == Some(id), &t.name).clicked() {
                    out = Some(Some(id));
                }
            }
        });
    out
}

fn material_label(ui: &mut Ui, model: &Model, m: Option<varstotas_kernel::MaterialId>) {
    ui.horizontal(|ui| {
        let (color, name) = match m.and_then(|id| model.materials.get(id)) {
            Some(mat) => (color32(mat.color), mat.name.clone()),
            None => (Color32::WHITE, "Default".to_string()),
        };
        let (rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
        ui.painter().rect(rect, 3.0, color, Stroke::new(1.0, theme::BORDER), egui::StrokeKind::Inside);
        ui.label(name);
    });
}

fn instance_label(model: &Model, mesh: &Mesh, id: InstanceId) -> (String, bool) {
    let Some(inst) = mesh.instances.get(id) else {
        return ("?".into(), true);
    };
    let def = model.defs.get(inst.def);
    let is_group = def.is_none_or(|d| d.is_group);
    let def_name = def.map(|d| d.name.clone()).unwrap_or_default();
    let label = match (inst.name.is_empty(), is_group) {
        (false, true) => inst.name.clone(),
        (false, false) => format!("{} <{def_name}>", inst.name),
        (true, true) => "Group".to_string(),
        (true, false) => format!("<{def_name}>"),
    };
    (label, is_group)
}

impl VarstotasApp {
    pub(crate) fn tray_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Default Tray").strong().size(14.0));
        });
        ui.add_space(4.0);
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            card(ui, "Entity Info", Icon::Info, true, |ui| self.entity_info(ui));
            card(ui, "Materials", Icon::Paint, true, |ui| self.materials_panel(ui));
            card(ui, "Tags", Icon::Eye, false, |ui| self.tags_panel(ui));
            card(ui, "Outliner", Icon::Group, true, |ui| self.outliner(ui));
            card(ui, "Scenes", Icon::ViewIso, false, |ui| self.scenes_panel(ui));
            card(ui, "Instructor", Icon::Select, false, |ui| self.instructor(ui));
        });
    }

    // ---- Entity Info -------------------------------------------------------

    fn entity_info(&mut self, ui: &mut Ui) {
        let unit = self.doc.model.unit;
        let sel = self.doc.selection.clone();
        let path = self.doc.active_path.clone();
        let mut set_tag: Option<Option<TagId>> = None;
        let mut rename: Option<(InstanceId, String)> = None;
        let mut set_unit: Option<LengthUnit> = None;
        let mut toggle_edge_soft: Option<bool> = None;

        let r = guarded(|| {
            let model = &self.doc.model;
            let Some(mesh) = model.mesh_at(&path) else {
                ui.label("Invalid context");
                return;
            };
            egui::Grid::new("entity_info").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                match sel.as_slice() {
                    [] => {
                        ui.label(RichText::new("No Selection").strong());
                        ui.end_row();
                        ui.label("Units");
                        let mut u = unit;
                        egui::ComboBox::from_id_salt("units")
                            .selected_text(super::unit_suffix(u))
                            .show_ui(ui, |ui| {
                                for (v, n) in [
                                    (LengthUnit::Millimeter, "Millimeters"),
                                    (LengthUnit::Centimeter, "Centimeters"),
                                    (LengthUnit::Meter, "Meters"),
                                    (LengthUnit::Inch, "Inches"),
                                    (LengthUnit::Foot, "Feet"),
                                ] {
                                    ui.selectable_value(&mut u, v, n);
                                }
                            });
                        if u != unit {
                            set_unit = Some(u);
                        }
                        ui.end_row();
                        ui.label("Faces");
                        ui.label(mesh.faces.len().to_string());
                        ui.end_row();
                        ui.label("Edges");
                        ui.label(mesh.edges.len().to_string());
                        ui.end_row();
                        ui.label("Groups");
                        ui.label(mesh.instances.len().to_string());
                        ui.end_row();
                    }
                    [EntityRef::Face(id)] => {
                        let Some(f) = mesh.faces.get(*id) else { return };
                        ui.label(RichText::new("Face").strong());
                        ui.end_row();
                        ui.label("Area");
                        ui.label(fmt_area(mesh.face_area(*id), unit));
                        ui.end_row();
                        ui.label("Front");
                        material_label(ui, model, f.front);
                        ui.end_row();
                        ui.label("Back");
                        material_label(ui, model, f.back);
                        ui.end_row();
                        ui.label("Tag");
                        set_tag = tag_combo(ui, model, f.tag, "face");
                        ui.end_row();
                    }
                    [EntityRef::Edge(id)] => {
                        let Some(e) = mesh.edges.get(*id) else { return };
                        ui.label(RichText::new("Edge").strong());
                        ui.end_row();
                        ui.label("Length");
                        ui.label(fmt_len(mesh.edge_length(*id), unit));
                        ui.end_row();
                        ui.label("Soft");
                        let mut soft = e.soft;
                        if ui.checkbox(&mut soft, "").changed() {
                            toggle_edge_soft = Some(soft);
                        }
                        ui.end_row();
                        ui.label("Tag");
                        set_tag = tag_combo(ui, model, e.tag, "edge");
                        ui.end_row();
                    }
                    [EntityRef::Instance(id)] => {
                        let Some(inst) = mesh.instances.get(*id) else { return };
                        let (_, is_group) = instance_label(model, mesh, *id);
                        ui.label(RichText::new(if is_group { "Group" } else { "Component" }).strong());
                        ui.end_row();
                        ui.label("Instance");
                        if self.tray.inst_name.as_ref().map(|(i, _)| *i) != Some(*id) {
                            self.tray.inst_name = Some((*id, inst.name.clone()));
                        }
                        if let Some((_, buf)) = self.tray.inst_name.as_mut() {
                            let resp = ui.add(egui::TextEdit::singleline(buf).hint_text("name").desired_width(150.0));
                            if !resp.has_focus() && !resp.lost_focus() && *buf != inst.name {
                                // Model changed underneath (undo): resync.
                                *buf = inst.name.clone();
                            } else if resp.lost_focus() && *buf != inst.name {
                                rename = Some((*id, buf.clone()));
                            }
                        }
                        ui.end_row();
                        if !is_group {
                            ui.label("Definition");
                            ui.label(model.defs.get(inst.def).map(|d| d.name.clone()).unwrap_or_default());
                            ui.end_row();
                        }
                        ui.label("Material");
                        material_label(ui, model, inst.material);
                        ui.end_row();
                        ui.label("Tag");
                        set_tag = tag_combo(ui, model, inst.tag, "inst");
                        ui.end_row();
                    }
                    many => {
                        let faces = many.iter().filter(|e| matches!(e, EntityRef::Face(_))).count();
                        let edges = many.iter().filter(|e| matches!(e, EntityRef::Edge(_))).count();
                        let insts = many.iter().filter(|e| matches!(e, EntityRef::Instance(_))).count();
                        ui.label(RichText::new(format!("{} Entities", many.len())).strong());
                        ui.end_row();
                        let mut parts = Vec::new();
                        if faces > 0 {
                            parts.push(format!("{faces} Face{}", if faces == 1 { "" } else { "s" }));
                        }
                        if edges > 0 {
                            parts.push(format!("{edges} Edge{}", if edges == 1 { "" } else { "s" }));
                        }
                        if insts > 0 {
                            parts.push(format!("{insts} Group{}", if insts == 1 { "" } else { "s" }));
                        }
                        ui.label("Contains");
                        ui.label(parts.join(", "));
                        ui.end_row();
                        if faces > 0 {
                            let area: f64 = many
                                .iter()
                                .filter_map(|e| if let EntityRef::Face(f) = e { Some(mesh.face_area(*f)) } else { None })
                                .sum();
                            ui.label("Area");
                            ui.label(fmt_area(area, unit));
                            ui.end_row();
                        }
                        ui.label("Tag");
                        set_tag = tag_combo(ui, model, None, "many");
                        ui.end_row();
                    }
                }
            });
        });
        if let Err(e) = r {
            ui.label(RichText::new(format!("(kernel not ready: {e})")).color(theme::TEXT_DIM));
        }

        if let Some(u) = set_unit {
            self.doc.mutate_untracked(|m| m.unit = u);
        }
        if let Some((id, name)) = rename {
            let p = path.clone();
            self.doc.commit("Rename", |m| {
                if let Some(i) = m.mesh_at_mut(&p).and_then(|mesh| mesh.instances.get_mut(id)) {
                    i.name = name;
                }
            });
        }
        if let Some(soft) = toggle_edge_soft {
            let p = path.clone();
            let sel2 = sel.clone();
            self.doc.commit(if soft { "Soften Edge" } else { "Unsoften Edge" }, |m| {
                if let Some(mesh) = m.mesh_at_mut(&p) {
                    for e in &sel2 {
                        if let EntityRef::Edge(id) = e
                            && let Some(e) = mesh.edges.get_mut(*id) {
                                e.soft = soft;
                                e.smooth = soft;
                            }
                    }
                }
            });
        }
        if let Some(tag) = set_tag {
            self.doc.commit("Set Tag", |m| {
                if let Some(mesh) = m.mesh_at_mut(&path) {
                    for e in &sel {
                        match *e {
                            EntityRef::Face(id) => {
                                if let Some(f) = mesh.faces.get_mut(id) {
                                    f.tag = tag;
                                }
                            }
                            EntityRef::Edge(id) => {
                                if let Some(x) = mesh.edges.get_mut(id) {
                                    x.tag = tag;
                                }
                            }
                            EntityRef::Instance(id) => {
                                if let Some(x) = mesh.instances.get_mut(id) {
                                    x.tag = tag;
                                }
                            }
                            EntityRef::Vertex(_) => {}
                        }
                    }
                }
            });
        }
    }

    // ---- Materials ---------------------------------------------------------

    fn materials_panel(&mut self, ui: &mut Ui) {
        // Current material
        ui.horizontal(|ui| {
            let (color, name) = match &self.doc.current_material {
                Some(m) => (color32(m.color), m.name.clone()),
                None => (Color32::WHITE, "Default".to_string()),
            };
            let (rect, _) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::hover());
            ui.painter().rect(rect, 6.0, color, Stroke::new(1.0, theme::BORDER), egui::StrokeKind::Inside);
            if self.doc.current_material.is_none() {
                // Default material: white front / blue-grey back split.
                let tri = vec![rect.right_top(), rect.right_bottom(), rect.left_bottom()];
                ui.painter().add(egui::Shape::convex_polygon(
                    tri,
                    color32(crate::scene_builder::BACK_DEFAULT),
                    Stroke::NONE,
                ));
            }
            ui.vertical(|ui| {
                ui.label(RichText::new(name).strong());
                ui.label(RichText::new("Current paint material (B)").small().color(theme::TEXT_DIM));
            });
        });
        ui.add_space(4.0);

        // Palette grid
        let cur = self.doc.current_material.clone();
        let size = 26.0;
        let mut picked: Option<Option<PaletteMaterial>> = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
            // "Default" swatch first.
            let r = swatch(ui, Color32::WHITE, size, cur.is_none()).on_hover_text("Default");
            ui.painter().add(egui::Shape::convex_polygon(
                vec![r.rect.right_top() + vec2(-2.0, 2.0), r.rect.right_bottom() - vec2(2.0, 2.0), r.rect.left_bottom() + vec2(2.0, -2.0)],
                color32(crate::scene_builder::BACK_DEFAULT),
                Stroke::NONE,
            ));
            if r.clicked() {
                picked = Some(None);
            }
            for (name, c) in PALETTE.iter() {
                let sel = cur.as_ref().is_some_and(|m| m.name == *name);
                if swatch(ui, Color32::from_rgb(c[0], c[1], c[2]), size, sel).on_hover_text(*name).clicked() {
                    picked = Some(Some(PaletteMaterial { name: name.to_string(), color: rgb(*c) }));
                }
            }
        });

        // Custom colour
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let mut c = self.tray.custom_color.unwrap_or(Color32::from_rgb(0x80, 0xA0, 0xC0));
            ui.label("Custom");
            if egui::color_picker::color_edit_button_srgba(ui, &mut c, egui::color_picker::Alpha::OnlyBlend).changed() {
                self.tray.custom_color = Some(c);
            }
            if ui.button("Use").clicked() {
                let [r, g, b, _] = c.to_srgba_unmultiplied();
                picked = Some(Some(PaletteMaterial {
                    name: format!("Color #{r:02X}{g:02X}{b:02X}"),
                    color: color_f32(c),
                }));
            }
        });

        // Materials in model
        if !self.doc.model.materials.is_empty() {
            ui.add_space(4.0);
            ui.label(RichText::new("In Model").small().color(theme::TEXT_DIM));
            ui.horizontal_wrapped(|ui| {
                for (_, m) in self.doc.model.materials.iter() {
                    let sel = cur.as_ref().is_some_and(|c| c.name == m.name);
                    if swatch(ui, color32(m.color), 22.0, sel).on_hover_text(&m.name).clicked() {
                        picked = Some(Some(PaletteMaterial { name: m.name.clone(), color: m.color }));
                    }
                }
            });
        }
        if let Some(p) = picked {
            self.doc.current_material = p;
            if self.tool_id() != crate::tools::ToolId::Paint {
                self.actions.push(Action::SetTool(crate::tools::ToolId::Paint));
            }
        }
    }

    // ---- Tags ----------------------------------------------------------------

    fn tags_panel(&mut self, ui: &mut Ui) {
        let mut toggle: Option<TagId> = None;
        let mut rename_commit: Option<(TagId, String)> = None;
        let mut add = false;
        ui.horizontal(|ui| {
            if small_icon_button(ui, Icon::Plus, "Add tag").clicked() {
                add = true;
            }
            ui.label(RichText::new("Visible · Name").small().color(theme::TEXT_DIM));
        });
        ui.horizontal(|ui| {
            ui.add_space(26.0);
            ui.label("Untagged");
        });
        let tags: Vec<(TagId, String, bool, [f32; 4])> = self
            .doc
            .model
            .tags
            .iter()
            .map(|(id, t)| (id, t.name.clone(), t.visible, t.color))
            .collect();
        for (id, name, visible, color) in tags {
            ui.horizontal(|ui| {
                if small_icon_button(ui, if visible { Icon::Eye } else { Icon::EyeOff }, "Toggle visibility").clicked() {
                    toggle = Some(id);
                }
                let (rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                ui.painter().circle_filled(rect.center(), 4.5, color32(color));
                match &mut self.tray.tag_rename {
                    Some((rid, buf)) if *rid == id => {
                        let r = ui.add(egui::TextEdit::singleline(buf).desired_width(140.0));
                        if r.lost_focus() {
                            rename_commit = Some((id, buf.clone()));
                        } else {
                            r.request_focus();
                        }
                    }
                    _ => {
                        let text = if visible { RichText::new(&name) } else { RichText::new(&name).color(theme::TEXT_DIM) };
                        let r = ui.add(egui::Label::new(text).sense(Sense::click())).on_hover_text("Double-click to rename");
                        if r.double_clicked() {
                            self.tray.tag_rename = Some((id, name.clone()));
                        }
                    }
                }
            });
        }
        if let Some(id) = toggle {
            self.doc.mutate_untracked(|m| {
                if let Some(t) = m.tags.get_mut(id) {
                    t.visible = !t.visible;
                }
            });
        }
        if let Some((id, name)) = rename_commit {
            self.tray.tag_rename = None;
            if !name.trim().is_empty() {
                self.doc.commit("Rename Tag", |m| {
                    if let Some(t) = m.tags.get_mut(id) {
                        t.name = name.trim().to_string();
                    }
                });
            }
        }
        if add {
            let n = self.doc.model.tags.len() + 1;
            let name = format!("Tag {n}");
            if let Some(id) = self.doc.commit("Add Tag", |m| m.add_tag(&name)) {
                self.tray.tag_rename = Some((id, name));
            }
        }
    }

    // ---- Outliner ----------------------------------------------------------

    fn outliner(&mut self, ui: &mut Ui) {
        let mut actions = Vec::new();
        let r = guarded(|| {
            let model = &self.doc.model;
            let root = model.root_mesh();
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                paint_icon(ui.painter(), rect, Icon::ViewIso, false);
                ui.label(RichText::new(self.doc.display_name()).strong());
            });
            if root.instances.is_empty() {
                ui.label(RichText::new("No groups or components yet.").color(theme::TEXT_DIM));
            }
            outliner_level(ui, model, root, &Vec::new(), &self.doc.active_path, &self.doc.selection, &mut actions);
        });
        if let Err(e) = r {
            ui.label(RichText::new(format!("(kernel not ready: {e})")).color(theme::TEXT_DIM));
        }
        self.actions.extend(actions);
    }

    // ---- Scenes --------------------------------------------------------------

    fn scenes_panel(&mut self, ui: &mut Ui) {
        let sel = self.tray.selected_scene;
        ui.horizontal(|ui| {
            if small_icon_button(ui, Icon::Plus, "Add scene from current view").clicked() {
                self.actions.push(Action::AddScene);
            }
            ui.add_enabled_ui(sel.is_some(), |ui| {
                if small_icon_button(ui, Icon::Refresh, "Update selected scene to current view").clicked()
                    && let Some(i) = sel {
                        self.actions.push(Action::UpdateScene(i));
                    }
                if small_icon_button(ui, Icon::Trash, "Delete selected scene").clicked()
                    && let Some(i) = sel {
                        self.actions.push(Action::DeleteScene(i));
                    }
            });
        });
        if self.doc.model.scenes.is_empty() {
            ui.label(RichText::new("No scenes. Click + to save the current view.").color(theme::TEXT_DIM));
        }
        let names: Vec<String> = self.doc.model.scenes.iter().map(|s| s.name.clone()).collect();
        let mut rename_commit = None;
        for (i, name) in names.into_iter().enumerate() {
            match &mut self.tray.scene_rename {
                Some((ri, buf)) if *ri == i => {
                    let r = ui.add(egui::TextEdit::singleline(buf).desired_width(f32::INFINITY));
                    if r.lost_focus() {
                        rename_commit = Some((i, buf.clone()));
                    } else {
                        r.request_focus();
                    }
                }
                _ => {
                    let r = ui
                        .add(egui::Button::selectable(sel == Some(i), &name).min_size(vec2(ui.available_width(), 22.0)))
                        .on_hover_text("Click to go to this scene, double-click to rename");
                    if r.double_clicked() {
                        self.tray.scene_rename = Some((i, name.clone()));
                    } else if r.clicked() {
                        self.actions.push(Action::ActivateScene(i));
                    }
                }
            }
        }
        if let Some((i, name)) = rename_commit {
            self.tray.scene_rename = None;
            if !name.trim().is_empty() {
                self.doc.commit("Rename Scene", |m| {
                    if let Some(s) = m.scenes.get_mut(i) {
                        s.name = name.trim().to_string();
                    }
                });
            }
        }
    }

    // ---- Instructor ------------------------------------------------------------

    fn instructor(&mut self, ui: &mut Ui) {
        let id = self.tool_id();
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::hover());
            paint_icon(ui.painter(), rect, id.icon(), false);
            ui.vertical(|ui| {
                ui.label(RichText::new(id.name()).strong().size(14.0));
                if let Some(s) = id.shortcut_label() {
                    ui.label(RichText::new(format!("Shortcut: {s}")).small().color(theme::TEXT_DIM));
                }
            });
        });
        ui.label(RichText::new(id.description()).italics());
        ui.add_space(2.0);
        ui.label(id.instructor());
        if !id.implemented() {
            ui.add_space(4.0);
            ui.label(RichText::new("This tool is not implemented yet.").color(Color32::from_rgb(0xC0, 0x60, 0x20)));
        }
    }
}

fn outliner_level(
    ui: &mut Ui,
    model: &Model,
    mesh: &Mesh,
    path: &ContextPath,
    active: &ContextPath,
    selection: &[EntityRef],
    actions: &mut Vec<Action>,
) {
    for (id, inst) in mesh.instances.iter() {
        let (label, is_group) = instance_label(model, mesh, id);
        let selected = path == active && selection.contains(&EntityRef::Instance(id));
        let child_mesh = model.defs.get(inst.def).map(|d| &d.mesh);
        let has_children = child_mesh.is_some_and(|m| !m.instances.is_empty());
        let mut child_path = path.clone();
        child_path.push(id);
        let row = |ui: &mut Ui, actions: &mut Vec<Action>| {
            if small_icon_button(ui, if inst.hidden { Icon::EyeOff } else { Icon::Eye }, "Hide / show").clicked() {
                actions.push(Action::ToggleInstanceHidden(path.clone(), id));
            }
            let (rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
            paint_icon(ui.painter(), rect, if is_group { Icon::Group } else { Icon::ViewIso }, false);
            let mut text = RichText::new(label.clone());
            if inst.hidden {
                text = text.color(theme::TEXT_DIM).italics();
            }
            if ui.selectable_label(selected, text).clicked() {
                actions.push(Action::SelectInstance(path.clone(), id));
            }
        };
        if has_children {
            let cid = ui.make_persistent_id(("outliner", &child_path));
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), cid, true)
                .show_header(ui, |ui| row(ui, actions))
                .body(|ui| {
                    if let Some(cm) = child_mesh {
                        outliner_level(ui, model, cm, &child_path, active, selection, actions);
                    }
                });
        } else {
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                row(ui, actions)
            });
        }
    }
}
