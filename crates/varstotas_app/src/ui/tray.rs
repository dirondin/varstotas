//! The "Default Tray": Entity Info, Materials, Tags, Outliner, Scenes, Instructor.

use super::icons::{Icon, paint_icon};
use super::{color_f32, color32, fmt_area, fmt_len, fmt_volume, small_icon_button, swatch, theme};
use crate::app::{Action, VarstotasApp};
use crate::document::{PaletteMaterial, guarded};
use egui::{Color32, Frame, Margin, RichText, ScrollArea, Sense, Stroke, Ui, vec2};
use varstotas_kernel::{ContextPath, EntityRef, GuideRef, InstanceId, LengthUnit, Mesh, Model, TagId};

#[derive(Default)]
pub struct TrayState {
    pub selected_scene: Option<usize>,
    pub close_requested: bool,
    tag_rename: Option<(TagId, String)>,
    scene_rename: Option<(usize, String)>,
    inst_name: Option<(InstanceId, String)>,
    custom_color: Option<Color32>,
    /// Multi-selection summary, cached by (model, selection) version: summing face areas of
    /// a huge selection every frame is too slow.
    sel_summary: Option<((u64, u64), SelSummary)>,
    /// Entity Info edit buffers: curve (id, radius, segments) and dimension (id, text).
    curve_edit: Option<(u32, String, String)>,
    dim_text: Option<(varstotas_kernel::DimId, String)>,
    /// Texture position edit buffers of a face: offset x, offset y, angle, scale x, scale y.
    tex_pos: Option<(varstotas_kernel::FaceId, [String; 5])>,
    /// Parametric group edit buffers: (instance, size X, Y, Z, gap, copies).
    params_edit: Option<(InstanceId, [String; 5])>,
    /// Section plane name being edited in Entity Info.
    section_name: Option<(usize, String)>,
    /// Texture size edit buffers of the current material: (material name, width, height).
    tex_size: Option<(String, String, String)>,
    /// Texture size: keep the image's aspect ratio (default on; stored inverted).
    tex_free_aspect: bool,
    /// Material thumbnails by texture bytes pointer.
    thumbs: std::collections::HashMap<usize, egui::TextureHandle>,
    pub soften: SoftenSettings,
    /// Generator parameters in Entity Info: length text buffers of the selected instances, and
    /// a number being dragged (applied when the drag ends).
    pub gen_text: Option<(Vec<InstanceId>, std::collections::BTreeMap<String, String>)>,
    pub gen_drag: Option<(String, f64)>,
    /// Material editor: physical properties being dragged (material name), and the cached
    /// sphere preview (key, texture).
    pub(crate) pbr_draft: Option<(String, varstotas_kernel::Pbr)>,
    pub(crate) pbr_preview: Option<(u64, egui::TextureHandle)>,
}

/// Soften/Smooth Edges panel settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoftenSettings {
    /// Edges folding by at most this many degrees are softened.
    pub angle: f64,
    pub smooth: bool,
    pub coplanar: bool,
}

impl Default for SoftenSettings {
    fn default() -> Self {
        Self { angle: 20.0, smooth: true, coplanar: false }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct SelSummary {
    count: usize,
    faces: usize,
    edges: usize,
    groups: usize,
    components: usize,
    guides: usize,
    dims: usize,
    area: f64,
}

/// Default palette (name, sRGB).
pub(crate) const PALETTE: [(&str, [u8; 3]); 24] = [
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

pub(crate) fn rgb(c: [u8; 3]) -> [f32; 4] {
    [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, 1.0]
}

/// Default Tray cards in their order (Window menu checkboxes, `Prefs::tray_hidden`).
pub(crate) const TRAY_CARDS: [&str; 10] =
    ["Entity Info", "Materials", "Components", "Tags", "Outliner", "Scenes", "Shadows", "Environment", "Soften Edges", "Instructor"];

fn card<R>(ui: &mut Ui, title: &str, icon: Icon, default_open: bool, body: impl FnOnce(&mut Ui) -> R) {
    card_ex(ui, title, icon, default_open, false, body)
}

/// [`card`] that can be forced open.
fn card_ex<R>(ui: &mut Ui, title: &str, icon: Icon, default_open: bool, force_open: bool, body: impl FnOnce(&mut Ui) -> R) {
    let r = Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(8.0)
        .inner_margin(Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let id = ui.make_persistent_id(("tray", title));
            let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, default_open);
            if force_open {
                state.set_open(true);
            }
            state
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
    if force_open {
        r.response.scroll_to_me(Some(egui::Align::Min));
    }
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
        // The tray's width is the user's (drag the edge); content never widens it, so the
        // viewport doesn't jump with the selection. Content lives in a fixed-width, clipped
        // child (long values wrap) (`allocate_rect` reports exactly the available rect to the panel).
        let rect = ui.available_rect_before_wrap();
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(*ui.layout()));
        child.set_clip_rect(ui.clip_rect());
        self.tray_cards(&mut child);
        ui.allocate_rect(rect, Sense::hover());
    }

    fn tray_cards(&mut self, ui: &mut Ui) {
        let hidden = self.prefs.tray_hidden.clone();
        let on = |n: &str| !hidden.iter().any(|h| h == n);
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if on("Entity Info") {
                card(ui, "Entity Info", Icon::Info, true, |ui| self.entity_info(ui));
            }
            if on("Materials") {
                card(ui, "Materials", Icon::Paint, true, |ui| self.materials_panel(ui));
            }
            if on("Components") {
                let open = self.library_open_requested();
                card_ex(ui, "Components", Icon::Group, false, open, |ui| self.library_panel(ui));
            }
            if on("Tags") {
                card(ui, "Tags", Icon::Eye, false, |ui| self.tags_panel(ui));
            }
            if on("Outliner") {
                card(ui, "Outliner", Icon::Group, true, |ui| self.outliner(ui));
            }
            if on("Scenes") {
                card(ui, "Scenes", Icon::ViewIso, false, |ui| self.scenes_panel(ui));
            }
            if on("Shadows") {
                card(ui, "Shadows", Icon::Shadows, false, |ui| self.shadows_panel(ui));
            }
            if on("Environment") {
                card(ui, "Environment", Icon::StyleRealistic, false, |ui| self.environment_panel(ui));
            }
            if on("Soften Edges") {
                card(ui, "Soften Edges", Icon::Arc, false, |ui| self.soften_panel(ui));
            }
            if on("Instructor") {
                card(ui, "Instructor", Icon::Select, false, |ui| self.instructor(ui));
            }
            if hidden.len() >= TRAY_CARDS.len() {
                ui.label(RichText::new("Every card is off: Window → Tray cards").color(theme::TEXT_DIM));
            }
        });
    }

    /// Edges the Soften Edges panel acts on: selected edges plus the edges of selected faces.
    pub(crate) fn soften_targets(&self) -> Vec<varstotas_kernel::EdgeId> {
        let Some(mesh) = self.doc.active_mesh() else { return Vec::new() };
        let mut out = Vec::new();
        for e in &self.doc.selection {
            match *e {
                EntityRef::Edge(id) if mesh.edges.contains_key(id) => out.push(id),
                EntityRef::Face(f) => out.extend(mesh.face_edges(f)),
                _ => {}
            }
        }
        out.sort();
        out.dedup();
        out
    }

    fn shadows_panel(&mut self, ui: &mut Ui) {
        let mut sh = self.doc.model.shadows;
        ui.checkbox(&mut sh.enabled, "Show shadows");
        egui::Grid::new("shadows").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
            ui.label("Time");
            ui.add(egui::Slider::new(&mut sh.minutes, 0..=24 * 60 - 1).step_by(5.0).custom_formatter(|v, _| {
                let m = v as u32;
                format!("{:02}:{:02}", m / 60, m % 60)
            }).custom_parser(|t| {
                let (h, m) = t.trim().split_once(':')?;
                Some((h.trim().parse::<u32>().ok()? * 60 + m.trim().parse::<u32>().ok()?) as f64)
            }));
            ui.end_row();
            ui.label("Date");
            ui.add(egui::Slider::new(&mut sh.day, 1..=365).custom_formatter(|v, _| {
                let (m, d) = varstotas_kernel::shadows::month_day(v as u32);
                format!("{d} {}", varstotas_kernel::shadows::MONTH_NAMES[m as usize - 1])
            }));
            ui.end_row();
            ui.label("Light");
            ui.add(egui::Slider::new(&mut sh.light, 0.0..=1.0).custom_formatter(|v, _| format!("{:.0}", v * 100.0)));
            ui.end_row();
            ui.label("Dark");
            ui.add(egui::Slider::new(&mut sh.dark, 0.0..=1.0).custom_formatter(|v, _| format!("{:.0}", v * 100.0)));
            ui.end_row();
            ui.label("Display");
            ui.horizontal(|ui| {
                ui.checkbox(&mut sh.on_faces, "On faces");
                ui.checkbox(&mut sh.on_ground, "On ground");
            });
            ui.end_row();
        });
        egui::CollapsingHeader::new("Location").id_salt("shadow_location").show(ui, |ui| {
            egui::Grid::new("shadow_loc").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                ui.label("Latitude");
                ui.add(egui::DragValue::new(&mut sh.latitude).range(-90.0..=90.0).speed(0.1).suffix("°"));
                ui.end_row();
                ui.label("Longitude");
                ui.add(egui::DragValue::new(&mut sh.longitude).range(-180.0..=180.0).speed(0.1).suffix("°"));
                ui.end_row();
                ui.label("UTC offset");
                ui.add(egui::DragValue::new(&mut sh.utc_offset).range(-12.0..=14.0).speed(0.05).fixed_decimals(1).suffix(" h"));
                ui.end_row();
                ui.label("North angle");
                ui.add(egui::DragValue::new(&mut sh.north_deg).range(0.0..=360.0).speed(0.5).suffix("°"));
                ui.end_row();
            });
        });
        let alt = sh.sun_altitude_deg();
        ui.label(RichText::new(if alt > 0.0 { format!("Sun {alt:.0}° above the horizon") } else { "Sun below the horizon".into() }).color(theme::TEXT_DIM));
        if sh != self.doc.model.shadows {
            self.actions.push(Action::SetShadows(sh));
        }
    }

    fn soften_panel(&mut self, ui: &mut Ui) {
        let n = self.soften_targets().len();
        let st = &mut self.tray.soften;
        ui.label("Allow normals between");
        let resp = ui.add_enabled(n > 0, egui::Slider::new(&mut st.angle, 0.0..=180.0).suffix("°").fixed_decimals(0));
        let smooth = ui.add_enabled(n > 0, egui::Checkbox::new(&mut st.smooth, "Smooth normals"));
        let coplanar = ui.add_enabled(n > 0, egui::Checkbox::new(&mut st.coplanar, "Soften coplanar"));
        if n == 0 {
            ui.label(RichText::new("Select edges or faces").color(theme::TEXT_DIM));
        } else {
            ui.label(RichText::new(format!("{n} edges")).color(theme::TEXT_DIM));
        }
        // One undo step per slider drag (applied on release), clicks apply at once.
        let slider_done = resp.drag_stopped() || (resp.changed() && !resp.dragged());
        if n > 0 && (slider_done || smooth.changed() || coplanar.changed()) {
            let s = *st;
            self.actions.push(Action::SoftenEdges(s));
        }
    }

    // ---- Entity Info -------------------------------------------------------

    fn selection_summary(&mut self) -> SelSummary {
        let key = (self.doc.model_version, self.doc.selection_version);
        if let Some((k, s)) = self.tray.sel_summary
            && k == key
        {
            return s;
        }
        let model = &self.doc.model;
        let mut s = SelSummary {
            count: self.doc.selection.len() + self.doc.guide_sel.len() + self.doc.dim_sel.len(),
            guides: self.doc.guide_sel.len(),
            dims: self.doc.dim_sel.len(),
            ..Default::default()
        };
        if let Some(mesh) = model.mesh_at(&self.doc.active_path) {
            for e in &self.doc.selection {
                match e {
                    EntityRef::Face(f) => {
                        s.faces += 1;
                        s.area += guarded(|| mesh.face_area(*f)).unwrap_or(0.0);
                    }
                    EntityRef::Edge(_) => s.edges += 1,
                    EntityRef::Instance(i) => {
                        let def = mesh.instances.get(*i).and_then(|inst| model.defs.get(inst.def));
                        if def.is_none_or(|d| d.is_group) {
                            s.groups += 1;
                        } else {
                            s.components += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
        self.tray.sel_summary = Some((key, s));
        s
    }

    fn entity_info(&mut self, ui: &mut Ui) {
        let unit = self.doc.model.unit;
        let summary = self.selection_summary();
        // Only the first two entities are needed to pick the layout; the multi-selection
        // details come from the cached summary (selections can hold 100k+ entities).
        let sel: Vec<EntityRef> = self.doc.selection.iter().take(2).cloned().collect();
        let guides: Vec<GuideRef> = self.doc.guide_sel.iter().take(2).cloned().collect();
        if let Some(i) = self.doc.section_sel {
            self.section_info(ui, i);
            return;
        }
        if sel.is_empty() && guides.len() == 1 && self.doc.dim_sel.is_empty() {
            self.guide_info(ui, guides[0]);
            return;
        }
        if sel.is_empty() && guides.is_empty() && self.doc.dim_sel.len() == 1 {
            self.dim_info(ui, self.doc.dim_sel[0]);
            return;
        }
        if guides.is_empty() && self.doc.dim_sel.is_empty() && let Some(cid) = self.selected_curve() {
            self.curve_info(ui, cid);
            return;
        }
        // Dimensions mixed with other things: multi-selection layout.
        let sel = if self.doc.dim_sel.is_empty() || sel.len() > 1 {
            sel
        } else {
            vec![sel.first().copied().unwrap_or(EntityRef::Vertex(Default::default())); 2]
        };
        // Mixed entity + guide selections use the multi-selection layout.
        let sel = if guides.is_empty() || sel.len() > 1 { sel } else { vec![sel.first().copied().unwrap_or(EntityRef::Vertex(Default::default())); 2] };
        let path = self.doc.active_path.clone();
        let mut set_tag: Option<Option<TagId>> = None;
        let mut rename: Option<(InstanceId, String)> = None;
        let mut set_unit: Option<LengthUnit> = None;
        let mut toggle_edge_soft: Option<(bool, bool)> = None;

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
                        let groups = mesh
                            .instances
                            .iter()
                            .filter(|(_, i)| model.defs.get(i.def).is_none_or(|d| d.is_group))
                            .count();
                        ui.label("Groups");
                        ui.label(groups.to_string());
                        ui.end_row();
                        ui.label("Components");
                        ui.label((mesh.instances.len() - groups).to_string());
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
                            toggle_edge_soft = Some((soft, soft));
                        }
                        ui.end_row();
                        ui.label("Smooth");
                        let mut smooth = e.smooth;
                        if ui.checkbox(&mut smooth, "").changed() {
                            toggle_edge_soft = Some((e.soft, smooth));
                        }
                        ui.end_row();
                        ui.label("Tag");
                        set_tag = tag_combo(ui, model, e.tag, "edge");
                        ui.end_row();
                    }
                    [EntityRef::Instance(id)] => {
                        let Some(inst) = mesh.instances.get(*id) else { return };
                        let (_, is_group) = instance_label(model, mesh, *id);
                        let solid = model.defs.get(inst.def).filter(|d| d.mesh.is_solid());
                        let parametric = model.defs.get(inst.def).is_some_and(|d| d.params.is_some());
                        let kind = if parametric { "Parametric Group" } else if is_group { "Group" } else { "Component" };
                        ui.label(RichText::new(if solid.is_some() { format!("Solid {kind}") } else { kind.into() }).strong());
                        ui.end_row();
                        if let Some(d) = solid {
                            // Signed volume of the definition, scaled by the instance transform.
                            let v = d.mesh.volume() * inst.transform.determinant().abs();
                            ui.label("Volume");
                            ui.label(fmt_volume(v.abs(), unit));
                            ui.end_row();
                        }
                        ui.label("Instance");
                        if self.tray.inst_name.as_ref().map(|(i, _)| *i) != Some(*id) {
                            self.tray.inst_name = Some((*id, inst.name.clone()));
                        }
                        if let Some((_, buf)) = self.tray.inst_name.as_mut() {
                            let resp = ui.add(egui::TextEdit::singleline(buf).hint_text("name").desired_width(150.0));
                            if !resp.has_focus() && !resp.lost_focus() && *buf != inst.name {
                                // Model changed underneath (undo): resync.
                                *buf = inst.name.clone();
                            } else if committed(ui, &resp) && *buf != inst.name {
                                rename = Some((*id, buf.clone()));
                            }
                        }
                        ui.end_row();
                        if !is_group {
                            ui.label("Definition");
                            ui.add(egui::Label::new(model.defs.get(inst.def).map(|d| d.name.clone()).unwrap_or_default()).truncate());
                            ui.end_row();
                        }
                        ui.label("Material");
                        material_label(ui, model, inst.material);
                        ui.end_row();
                        ui.label("Tag");
                        set_tag = tag_combo(ui, model, inst.tag, "inst");
                        ui.end_row();
                    }
                    _ => {
                        let SelSummary { count, faces, edges, groups, components, guides, dims, area } = summary;
                        let only_guides = guides + dims == count;
                        let title = if guides == count {
                            format!("{count} Guides")
                        } else if dims == count {
                            format!("{count} Dimensions")
                        } else {
                            format!("{count} Entities")
                        };
                        ui.label(RichText::new(title).strong());
                        ui.end_row();
                        let plural = |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
                        let mut parts = Vec::new();
                        if faces > 0 {
                            parts.push(plural(faces, "Face", "Faces"));
                        }
                        if edges > 0 {
                            parts.push(plural(edges, "Edge", "Edges"));
                        }
                        if groups > 0 {
                            parts.push(plural(groups, "Group", "Groups"));
                        }
                        if components > 0 {
                            parts.push(plural(components, "Component", "Components"));
                        }
                        if guides > 0 {
                            parts.push(plural(guides, "Guide", "Guides"));
                        }
                        if dims > 0 {
                            parts.push(plural(dims, "Dimension", "Dimensions"));
                        }
                        ui.label("Contains");
                        ui.add(egui::Label::new(parts.join(", ")).wrap());
                        ui.end_row();
                        if faces > 0 {
                            ui.label("Area");
                            ui.label(fmt_area(area, unit));
                            ui.end_row();
                        }
                        if !only_guides {
                            ui.label("Tag");
                            set_tag = tag_combo(ui, model, None, "many");
                            ui.end_row();
                        }
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
        if let Some((soft, smooth)) = toggle_edge_soft {
            let p = path.clone();
            let sel2 = sel.clone();
            self.doc.commit(if soft { "Soften Edge" } else { "Unsoften Edge" }, |m| {
                if let Some(mesh) = m.mesh_at_mut(&p) {
                    for e in &sel2 {
                        if let EntityRef::Edge(id) = e
                            && let Some(e) = mesh.edges.get_mut(*id) {
                                e.soft = soft;
                                e.smooth = smooth;
                            }
                    }
                }
            });
        }
        if let Some(tag) = set_tag {
            // All selected entities (`sel` only holds the first two, for the layout).
            let sel = self.doc.selection.clone();
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
        if let [EntityRef::Face(fid)] = sel.as_slice() {
            self.face_texture_info(ui, *fid);
        }
        if !self.generator_info(ui)
            && let [EntityRef::Instance(id)] = sel.as_slice()
        {
            self.params_info(ui, *id);
        }
    }

    /// Entity Info for a group / component: "Make Parametric", or the attributes of a
    /// parametric group (size along the base's axes, copies in a row, gap, stretch / scale).
    fn params_info(&mut self, ui: &mut Ui, id: InstanceId) {
        let path = self.doc.active_path.clone();
        let unit = self.doc.model.unit;
        let model = &self.doc.model;
        let Some(p) = model.instance_params(&path, id).cloned() else {
            if model.mesh_at(&path).is_some_and(|m| m.instances.contains_key(id)) {
                ui.add_space(6.0);
                if ui.button("Make Parametric").on_hover_text("Size and copies set in Entity Info").clicked() {
                    self.actions.push(Action::MakeParametric);
                }
            }
            return;
        };
        let Some(bb) = model.def_bounds(p.base) else { return };
        let ext = bb.max - bb.min;
        let texts: [String; 5] = [
            fmt_len(p.size[0].unwrap_or(ext.x), unit),
            fmt_len(p.size[1].unwrap_or(ext.y), unit),
            fmt_len(p.size[2].unwrap_or(ext.z), unit),
            fmt_len(p.gap, unit),
            p.count.to_string(),
        ];
        if self.tray.params_edit.as_ref().map(|e| e.0) != Some(id) {
            self.tray.params_edit = Some((id, texts.clone()));
        }
        let mut next = p.clone();
        let mut bad = false;
        ui.add_space(6.0);
        egui::Grid::new("entity_info_params").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(RichText::new("Parameters").strong());
            ui.end_row();
            let Some((_, bufs)) = self.tray.params_edit.as_mut() else { return };
            for k in [0, 1, 2, 4, 3] {
                ui.label(["Length (X)", "Width (Y)", "Height (Z)", "Gap", "Copies"][k]);
                let resp = ui.add(egui::TextEdit::singleline(&mut bufs[k]).desired_width(120.0));
                if committed(ui, &resp) && bufs[k] != texts[k] {
                    let v = varstotas_kernel::units::parse_length(&bufs[k], unit);
                    match (k, v) {
                        (4, _) => match bufs[k].trim().parse::<u32>() {
                            Ok(n) if (1..=1000).contains(&n) => next.count = n,
                            _ => bad = true,
                        },
                        (3, Some(g)) => next.gap = g,
                        (_, Some(s)) if s > 0.0 => next.size[k] = Some(s),
                        _ => bad = true,
                    }
                } else if !resp.has_focus() && bufs[k] != texts[k] {
                    bufs[k] = texts[k].clone();
                }
                ui.end_row();
                if k == 4 {
                    ui.label("Along");
                    ui.horizontal(|ui| {
                        for (a, n) in ["X", "Y", "Z"].into_iter().enumerate() {
                            ui.selectable_value(&mut next.axis, a as u8, n);
                        }
                    });
                    ui.end_row();
                }
            }
            ui.label("Resize");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut next.stretch, true, "Stretch").on_hover_text("Move the far half: ends keep their size");
                ui.selectable_value(&mut next.stretch, false, "Scale");
            });
            ui.end_row();
        });
        if bad {
            self.tray.params_edit = None;
            self.flash_error("Invalid value");
        } else if next != p {
            self.tray.params_edit = None;
            self.actions.push(Action::SetParams(next));
        }
    }

    /// Texture position of a textured face (the numeric form of the classic Texture Position):
    /// offset of the texture origin, rotation and scale.
    fn face_texture_info(&mut self, ui: &mut Ui, fid: varstotas_kernel::FaceId) {
        let path = self.doc.active_path.clone();
        let unit = self.doc.model.unit;
        let model = &self.doc.model;
        let Some(face) = model.mesh_at(&path).and_then(|m| m.faces.get(fid)) else { return };
        let textured = [face.front, face.back]
            .into_iter()
            .flatten()
            .any(|id| model.materials.get(id).is_some_and(|m| m.image.is_some()));
        if !textured {
            return;
        }
        let pl = face.uv.unwrap_or_default();
        let texts = [
            fmt_len(pl.offset[0], unit),
            fmt_len(pl.offset[1], unit),
            format!("{:.1}°", pl.angle_deg),
            format!("{}", pl.scale[0]),
            format!("{}", pl.scale[1]),
        ];
        if self.tray.tex_pos.as_ref().map(|t| t.0) != Some(fid) {
            self.tray.tex_pos = Some((fid, texts.clone()));
        }
        let mut new = None;
        let mut bad = false;
        ui.add_space(6.0);
        egui::Grid::new("entity_info_texture").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(RichText::new("Texture").strong());
            ui.end_row();
            let Some((_, bufs)) = self.tray.tex_pos.as_mut() else { return };
            for (i, label) in ["Offset X", "Offset Y", "Rotation", "Scale X", "Scale Y"].into_iter().enumerate() {
                ui.label(label);
                let resp = ui.add(egui::TextEdit::singleline(&mut bufs[i]).desired_width(120.0));
                if committed(ui, &resp) && bufs[i] != texts[i] {
                    let t = bufs[i].trim();
                    let mut p = pl;
                    let ok = match i {
                        0 | 1 => varstotas_kernel::units::parse_length(t, unit).map(|v| p.offset[i] = v).is_some(),
                        2 => t.trim_end_matches('°').trim().parse::<f64>().map(|v| p.angle_deg = v).is_ok(),
                        _ => t.parse::<f64>().ok().filter(|v| v.abs() > 1e-6).map(|v| p.scale[i - 3] = v).is_some(),
                    };
                    if ok {
                        new = Some(p);
                    } else {
                        bad = true;
                    }
                } else if !resp.has_focus() && bufs[i] != texts[i] {
                    bufs[i] = texts[i].clone();
                }
                ui.end_row();
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Position…").on_hover_text("Texture Position tool (pins)").clicked() {
                self.actions.push(Action::SetTool(crate::tools::ToolId::TexturePosition));
            }
            if ui.button("Rotate 90°").clicked() {
                new = Some(varstotas_kernel::TexPlacement { angle_deg: (pl.angle_deg + 90.0) % 360.0, ..pl });
            }
            if ui.add_enabled(!pl.is_identity(), egui::Button::new("Reset")).clicked() {
                new = Some(Default::default());
            }
        });
        if bad {
            self.tray.tex_pos = None;
            self.flash_error("Invalid value");
        }
        if let Some(p) = new {
            self.tray.tex_pos = None;
            self.doc.commit("Texture Position", |m| {
                if let Some(f) = m.mesh_at_mut(&path).and_then(|mesh| mesh.faces.get_mut(fid)) {
                    f.uv = (!p.is_identity()).then_some(p);
                }
            });
        }
    }

    /// Thumbnail of a texture for the material swatches (decoded once).
    fn thumbnail(&mut self, ctx: &egui::Context, img: &varstotas_kernel::TextureImage) -> Option<egui::TextureId> {
        let key = std::sync::Arc::as_ptr(&img.bytes) as *const u8 as usize;
        if let Some(t) = self.tray.thumbs.get(&key) {
            return Some(t.id());
        }
        let ci = crate::textures::thumbnail(&img.bytes, 64)?;
        let handle = ctx.load_texture(format!("thumb{key}"), ci, egui::TextureOptions::LINEAR);
        let id = handle.id();
        self.tray.thumbs.insert(key, handle);
        Some(id)
    }

    /// Size of the current textured material's tile (width / height, aspect lock).
    fn texture_size_editor(&mut self, ui: &mut Ui) {
        let unit = self.doc.model.unit;
        let Some(cur) = self.doc.current_material.clone() else { return };
        let Some((mid, mat)) = self.doc.model.materials.iter().find(|(_, m)| m.name == cur.name) else { return };
        let Some(img) = mat.image.clone() else { return };
        let (wt, ht) = (fmt_len(img.size[0], unit), fmt_len(img.size[1], unit));
        if self.tray.tex_size.as_ref().map(|t| &t.0) != Some(&cur.name) {
            self.tray.tex_size = Some((cur.name.clone(), wt.clone(), ht.clone()));
        }
        let mut new: Option<[f64; 2]> = None;
        let mut bad = false;
        let lock = !self.tray.tex_free_aspect;
        ui.add_space(4.0);
        egui::Grid::new("texture_size").num_columns(2).spacing([10.0, 4.0]).show(ui, |ui| {
            let Some((_, wb, hb)) = self.tray.tex_size.as_mut() else { return };
            for (i, (label, buf, text)) in [("Texture width", wb, &wt), ("Texture height", hb, &ht)].into_iter().enumerate() {
                ui.label(label);
                let resp = ui.add(egui::TextEdit::singleline(buf).desired_width(100.0));
                if committed(ui, &resp) && buf != text {
                    match varstotas_kernel::units::parse_length(buf, unit).filter(|v| *v > 0.0) {
                        Some(v) => {
                            let mut sz = img.size;
                            let k = v / sz[i];
                            sz[i] = v;
                            if lock {
                                sz[1 - i] *= k;
                            }
                            new = Some(sz);
                        }
                        None => bad = true,
                    }
                } else if !resp.has_focus() && buf != text {
                    *buf = text.clone();
                }
                ui.end_row();
            }
        });
        let mut keep = lock;
        if ui.checkbox(&mut keep, "Keep aspect ratio").changed() {
            self.tray.tex_free_aspect = !keep;
        }
        if bad {
            self.tray.tex_size = None;
            self.flash_error("Invalid value");
        }
        if let Some(sz) = new {
            self.tray.tex_size = None;
            self.doc.commit("Edit Material", |m| {
                if let Some(i) = m.materials.get_mut(mid).and_then(|m| m.image.as_mut()) {
                    i.size = sz;
                }
            });
            if let Some(m) = self.doc.model.materials.get(mid) {
                self.doc.current_material = Some(PaletteMaterial::of(m));
            }
        }
    }

    /// The curve (arc / circle / polygon with metadata) when the selection is exactly its edges.
    fn selected_curve(&self) -> Option<u32> {
        let mesh = self.doc.model.mesh_at(&self.doc.active_path)?;
        let mut cid = None;
        let mut n = 0;
        for e in &self.doc.selection {
            let EntityRef::Edge(id) = e else { return None };
            let c = mesh.edges.get(*id)?.curve?;
            if cid.is_some_and(|x| x != c) {
                return None;
            }
            cid = Some(c);
            n += 1;
        }
        let cid = cid?;
        (n == mesh.curve_edges(cid).len() && mesh.curve_info(cid).is_some()).then_some(cid)
    }

    /// Entity Info for an arc / circle / polygon: radius and segments are editable.
    fn curve_info(&mut self, ui: &mut Ui, cid: u32) {
        let path = self.doc.active_path.clone();
        let unit = self.doc.model.unit;
        let Some(info) = self.doc.model.mesh_at(&path).and_then(|m| m.curve_info(cid)) else { return };
        let r_text = fmt_len(info.radius, unit);
        let s_text = info.segments.to_string();
        if self.tray.curve_edit.as_ref().map(|c| c.0) != Some(cid) {
            self.tray.curve_edit = Some((cid, r_text.clone(), s_text.clone()));
        }
        let mut apply: Option<(Option<f64>, Option<u32>)> = None;
        let mut bad = false;
        egui::Grid::new("entity_info_curve").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            let title = match info.kind {
                varstotas_kernel::CurveKind::Arc => "Arc",
                varstotas_kernel::CurveKind::Circle => "Circle",
                varstotas_kernel::CurveKind::Polygon => "Polygon",
            };
            ui.label(RichText::new(title).strong());
            ui.end_row();
            let Some((_, rb, sb)) = self.tray.curve_edit.as_mut() else { return };
            ui.label("Radius");
            let resp = ui.add(egui::TextEdit::singleline(rb).desired_width(120.0));
            if committed(ui, &resp) && *rb != r_text {
                match varstotas_kernel::units::parse_length(rb, unit).filter(|r| *r > 0.0) {
                    Some(r) => apply = Some((Some(r), None)),
                    None => bad = true,
                }
            } else if !resp.has_focus() && *rb != r_text {
                *rb = r_text.clone();
            }
            ui.end_row();
            ui.label(if info.kind == varstotas_kernel::CurveKind::Polygon { "Sides" } else { "Segments" });
            let resp = ui.add(egui::TextEdit::singleline(sb).desired_width(120.0));
            if committed(ui, &resp) && *sb != s_text {
                match sb.trim().trim_end_matches(['s', 'S']).trim().parse::<u32>() {
                    Ok(n) => apply = Some((None, Some(n))),
                    Err(_) => bad = true,
                }
            } else if !resp.has_focus() && *sb != s_text {
                *sb = s_text.clone();
            }
            ui.end_row();
            ui.label(if info.is_closed() { "Circumference" } else { "Arc Length" });
            ui.label(fmt_len(info.length(), unit));
            ui.end_row();
            if info.kind == varstotas_kernel::CurveKind::Arc {
                ui.label("Angle");
                ui.label(crate::tools::common::fmt_angle(info.sweep().to_degrees()));
                ui.end_row();
            }
        });
        if bad {
            self.tray.curve_edit = None;
            self.flash_error("Invalid value");
        }
        if let Some((r, n)) = apply {
            let label = if r.is_some() { "Edit Radius" } else { "Edit Segments" };
            let edges = self.doc.try_commit(label, |m| match m.mesh_at_mut(&path) {
                Some(mesh) => mesh.set_curve(cid, r, n),
                None => Err(varstotas_kernel::OpError::NotFound),
            });
            self.tray.curve_edit = None;
            if let Some(es) = edges {
                // Keep the regenerated curve selected.
                self.doc.set_selection(es.into_iter().map(EntityRef::Edge).collect());
            } else if let Some(e) = self.doc.last_error.clone() {
                self.flash_error(e);
            }
        }
    }

    /// Entity Info for a dimension: length and editable text.
    fn dim_info(&mut self, ui: &mut Ui, id: varstotas_kernel::DimId) {
        let path = self.doc.active_path.clone();
        let unit = self.doc.model.unit;
        let Some(g) = self.doc.model.dimension_in(&path, id) else { return };
        let text = g.text.clone().unwrap_or_default();
        if self.tray.dim_text.as_ref().map(|d| d.0) != Some(id) {
            self.tray.dim_text = Some((id, text.clone()));
        }
        let mut apply: Option<String> = None;
        egui::Grid::new("entity_info_dim").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(RichText::new(if g.leader { "Text" } else { "Linear Dimension" }).strong());
            ui.end_row();
            if !g.leader {
                ui.label("Length");
                ui.label(fmt_len(g.length(), unit));
                ui.end_row();
            }
            let Some((_, buf)) = self.tray.dim_text.as_mut() else { return };
            ui.label("Text");
            let hint = if g.leader { "" } else { "<>" };
            let resp = ui.add(egui::TextEdit::singleline(buf).hint_text(hint).desired_width(150.0));
            if committed(ui, &resp) && *buf != text {
                apply = Some(buf.clone());
            } else if !resp.has_focus() && *buf != text {
                *buf = text.clone();
            }
            ui.end_row();
        });
        if let Some(t) = apply {
            let t = Some(t.trim().to_string()).filter(|t| !t.is_empty());
            let label = if g.leader { "Edit Text" } else { "Edit Dimension Text" };
            self.doc.commit(label, |m| m.set_dimension_text_in(&path, id, t));
            self.tray.dim_text = None;
        }
    }

    /// Entity Info for a single selected guide line / guide point (world coordinates).
    fn section_info(&mut self, ui: &mut Ui, i: usize) {
        let Some(s) = self.doc.model.sections.get(i).cloned() else { return };
        let unit = self.doc.model.unit;
        if self.tray.section_name.as_ref().is_none_or(|n| n.0 != i) {
            self.tray.section_name = Some((i, s.name.clone()));
        }
        let mut rename = None;
        let mut active = s.active;
        egui::Grid::new("entity_info_section").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(RichText::new("Section Plane").strong());
            ui.end_row();
            ui.label("Name");
            if let Some((_, buf)) = self.tray.section_name.as_mut() {
                let resp = ui.add(egui::TextEdit::singleline(buf).desired_width(150.0));
                if committed(ui, &resp) && buf.trim() != s.name && !buf.trim().is_empty() {
                    rename = Some(buf.trim().to_string());
                } else if !resp.has_focus() && *buf != s.name {
                    *buf = s.name.clone();
                }
            }
            ui.end_row();
            ui.label("Active Cut");
            ui.checkbox(&mut active, "");
            ui.end_row();
            let p = s.origin;
            ui.label("Through");
            ui.add(egui::Label::new(format!("{}, {}, {}", fmt_len(p.x, unit), fmt_len(p.y, unit), fmt_len(p.z, unit))).wrap());
            ui.end_row();
            ui.label("Normal");
            ui.label(format!("{:.3}, {:.3}, {:.3}", s.normal.x, s.normal.y, s.normal.z));
            ui.end_row();
        });
        ui.horizontal(|ui| {
            if ui.button("Reverse").clicked() {
                self.actions.push(Action::ReverseSection);
            }
            if ui.button("Align View").clicked() {
                self.actions.push(Action::AlignSectionView);
            }
        });
        if active != s.active {
            self.actions.push(Action::ActiveCut);
        }
        if let Some(name) = rename {
            self.tray.section_name = None;
            self.doc.commit("Rename Section", |m| {
                if let Some(s) = m.sections.get_mut(i) {
                    s.name = name;
                }
            });
        }
    }

    fn guide_info(&mut self, ui: &mut Ui, g: GuideRef) {
        let model = &self.doc.model;
        let unit = model.unit;
        let pt = |p: varstotas_kernel::glam::DVec3| {
            format!("{}, {}, {}", fmt_len(p.x, unit), fmt_len(p.y, unit), fmt_len(p.z, unit))
        };
        let geom = model.guide_geom_in(&self.doc.active_path, g);
        egui::Grid::new("entity_info_guide").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| match geom {
            Some(varstotas_kernel::GuideGeom::Line(l)) => {
                ui.label(RichText::new("Guide Line").strong());
                ui.end_row();
                ui.label("Through");
                ui.add(egui::Label::new(pt(l.origin)).wrap());
                ui.end_row();
                let d = l.dir.normalize_or_zero();
                ui.label("Direction");
                ui.label(format!("{:.3}, {:.3}, {:.3}", d.x, d.y, d.z));
                ui.end_row();
            }
            Some(varstotas_kernel::GuideGeom::Point(p)) => {
                ui.label(RichText::new("Guide Point").strong());
                ui.end_row();
                ui.label("Position");
                ui.add(egui::Label::new(pt(p)).wrap());
                ui.end_row();
            }
            None => {}
        });
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
                    picked = Some(Some(PaletteMaterial::color(*name, rgb(*c))));
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
            if ui.button("Texture…").on_hover_text("New material from an image file (PNG, JPEG)").clicked() {
                self.actions.push(Action::ImportTexture);
            }
            if ui.button("Use").clicked() {
                let [r, g, b, _] = c.to_srgba_unmultiplied();
                picked = Some(Some(PaletteMaterial::color(format!("Color #{r:02X}{g:02X}{b:02X}"), color_f32(c))));
            }
        });

        // Materials in model
        if !self.doc.model.materials.is_empty() {
            ui.add_space(4.0);
            ui.label(RichText::new("In Model").small().color(theme::TEXT_DIM));
            let mats: Vec<varstotas_kernel::Material> = self.doc.model.materials.values().cloned().collect();
            let ctx = ui.ctx().clone();
            ui.horizontal_wrapped(|ui| {
                for m in &mats {
                    let sel = cur.as_ref().is_some_and(|c| c.name == m.name);
                    let r = swatch(ui, color32(m.color), 22.0, sel).on_hover_text(&m.name);
                    if let Some(img) = &m.image
                        && let Some(t) = self.thumbnail(&ctx, img)
                    {
                        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                        ui.painter().image(t, r.rect.shrink(3.0), uv, Color32::WHITE);
                    }
                    if r.clicked() {
                        picked = Some(Some(PaletteMaterial::of(m)));
                    }
                }
            });
        }
        self.texture_size_editor(ui);
        self.pbr_editor(ui);
        if let Some(p) = picked {
            self.doc.current_material = p;
            if self.tool_id() != crate::tools::ToolId::Paint {
                self.actions.push(Action::SetTool(crate::tools::ToolId::Paint));
            }
        }
    }

    // ---- Tags ----------------------------------------------------------------

    /// Delete tag `id` (one undo step); what it tagged becomes Untagged.
    pub(crate) fn delete_tag(&mut self, id: TagId) {
        let Some(name) = self.doc.model.tags.get(id).map(|t| t.name.clone()) else { return };
        let uses = self.doc.model.tag_uses(id);
        if self.tray.tag_rename.as_ref().is_some_and(|(r, _)| *r == id) {
            self.tray.tag_rename = None;
        }
        self.doc.commit("Delete Tag", |m| m.delete_tag(id));
        self.flash(if uses > 0 { format!("Deleted tag “{name}”: {uses} item(s) are Untagged now") } else { format!("Deleted tag “{name}”") });
    }

    fn tags_panel(&mut self, ui: &mut Ui) {
        let mut toggle: Option<TagId> = None;
        let mut rename_commit: Option<(TagId, String)> = None;
        let mut delete: Option<TagId> = None;
        let mut add = false;
        ui.horizontal(|ui| {
            if small_icon_button(ui, Icon::Plus, "Add tag").clicked() {
                add = true;
            }
            ui.label(RichText::new("Visible · Name").small().color(theme::TEXT_DIM));
        });
        let un_shown = !self.doc.model.untagged_hidden;
        ui.horizontal(|ui| {
            if small_icon_button(ui, if un_shown { Icon::Eye } else { Icon::EyeOff }, "Toggle visibility of untagged top-level geometry, groups and components (their contents belong to them)")
                .clicked()
            {
                self.doc.mutate_untracked(|m| m.untagged_hidden = !m.untagged_hidden);
            }
            ui.add_space(14.0);
            ui.label(if un_shown { RichText::new("Untagged") } else { RichText::new("Untagged").color(theme::TEXT_DIM) });
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
                        let r = ui.add(egui::Label::new(text).sense(Sense::click())).on_hover_text("Double-click to rename; right-click for more");
                        if r.double_clicked() {
                            self.tray.tag_rename = Some((id, name.clone()));
                        }
                        r.context_menu(|ui| {
                            if ui.button("Rename").clicked() {
                                self.tray.tag_rename = Some((id, name.clone()));
                                ui.close();
                            }
                            if ui.button("Delete Tag").clicked() {
                                delete = Some(id);
                                ui.close();
                            }
                        });
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if small_icon_button(ui, Icon::Trash, "Delete the tag (what it tags becomes Untagged)").clicked() {
                        delete = Some(id);
                    }
                });
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
        if let Some(id) = delete {
            self.delete_tag(id);
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
                if ui.small_button("↑").on_hover_text("Move scene up").clicked()
                    && let Some(i) = sel {
                        self.actions.push(Action::MoveScene(i, false));
                    }
                if ui.small_button("↓").on_hover_text("Move scene down").clicked()
                    && let Some(i) = sel {
                        self.actions.push(Action::MoveScene(i, true));
                    }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = if self.scene_player.is_some() { "Pause" } else { "Play" };
                if ui
                    .add_enabled(!self.doc.model.scenes.is_empty(), egui::Button::new(label).small())
                    .on_hover_text("Play the scene animation (View → Animation)")
                    .clicked()
                {
                    self.actions.push(Action::ToggleAnimation);
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
        if let Some(i) = sel.filter(|i| *i < self.doc.model.scenes.len()) {
            let mut inc = self.doc.model.scenes[i].in_animation;
            if ui.checkbox(&mut inc, "Include in animation").changed() {
                self.doc.commit("Scene Properties", |m| m.scenes[i].in_animation = inc);
            }
        }
        egui::CollapsingHeader::new("Animation").id_salt("scene_anim_settings").show(ui, |ui| {
            let mut set = self.doc.model.scene_settings;
            ui.checkbox(&mut set.transitions, "Scene transitions");
            ui.add_enabled_ui(set.transitions, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Transition");
                    ui.add(egui::DragValue::new(&mut set.transition_secs).range(0.0..=60.0).speed(0.05).suffix(" s"));
                });
            });
            ui.horizontal(|ui| {
                ui.label("Scene delay");
                ui.add(egui::DragValue::new(&mut set.delay_secs).range(0.0..=60.0).speed(0.05).suffix(" s"));
            });
            if set != self.doc.model.scene_settings {
                // Not geometry: no undo step or scene rebuild, just an unsaved change.
                self.doc.model.scene_settings = set;
                self.doc.dirty = true;
            }
        });
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

/// A text field was left with its value accepted (Enter, Tab or a click elsewhere); Escape
/// leaves it too but means "revert".
pub(crate) fn committed(ui: &egui::Ui, resp: &egui::Response) -> bool {
    resp.lost_focus() && !ui.input(|i| i.key_pressed(egui::Key::Escape))
}
