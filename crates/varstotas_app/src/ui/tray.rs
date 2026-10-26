//! The side bar: Entity Info, Materials, Tags, Outliner, Scenes, Instructor.

use super::icons::{Icon, paint_icon};
use varstotas_i18n::tr;
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
    /// The shader editor window (open while `Some`).
    pub(crate) shader_edit: Option<crate::ui::realistic::ShaderEdit>,
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
    ("White", [0xFF, 0xFF, 0xFF]), // i18n-ignore
    ("Light Grey", [0xD4, 0xD6, 0xDA]), // i18n-ignore
    ("Grey", [0x9A, 0x9E, 0xA6]), // i18n-ignore
    ("Charcoal", [0x4A, 0x4E, 0x56]), // i18n-ignore
    ("Black", [0x1C, 0x1D, 0x20]), // i18n-ignore
    ("Concrete", [0xB8, 0xB4, 0xAA]), // i18n-ignore
    ("Brick", [0xB4, 0x54, 0x3C]), // i18n-ignore
    ("Terracotta", [0xD8, 0x7A, 0x52]), // i18n-ignore
    ("Orange", [0xF0, 0x9A, 0x36]), // i18n-ignore
    ("Sand", [0xE8, 0xD3, 0xA2]), // i18n-ignore
    ("Yellow", [0xF6, 0xD8, 0x4A]), // i18n-ignore
    ("Olive", [0x8E, 0x98, 0x48]), // i18n-ignore
    ("Grass", [0x6E, 0xB0, 0x4C]), // i18n-ignore
    ("Forest", [0x2F, 0x6E, 0x3E]), // i18n-ignore
    ("Teal", [0x2E, 0x9A, 0x96]), // i18n-ignore
    ("Sky", [0x8C, 0xC4, 0xEC]), // i18n-ignore
    ("Water", [0x3A, 0x84, 0xC8]), // i18n-ignore
    ("Navy", [0x24, 0x3C, 0x74]), // i18n-ignore
    ("Lavender", [0xB4, 0xA8, 0xE0]), // i18n-ignore
    ("Purple", [0x7A, 0x4E, 0xA8]), // i18n-ignore
    ("Pink", [0xF0, 0xA0, 0xB8]), // i18n-ignore
    ("Light Wood", [0xE2, 0xC2, 0x8E]), // i18n-ignore
    ("Wood", [0xB8, 0x86, 0x52]), // i18n-ignore
    ("Dark Wood", [0x6E, 0x48, 0x2C]), // i18n-ignore
];

pub(crate) fn rgb(c: [u8; 3]) -> [f32; 4] {
    [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, 1.0]
}

/// Side bar cards in their order (Window menu checkboxes, `Prefs::tray_hidden`).
pub(crate) const TRAY_CARDS: [&str; 11] =
    ["Entity Info", "Materials", "Material Editor", "Components", "Tags", "Outliner", "Scenes", "Shadows", "Environment", "Soften Edges", "Instructor"]; // i18n-ignore

/// The side bar card `title` (an id: preference values, spots) as shown to the user.
pub(crate) fn card_label(title: &str) -> String {
    match title {
        "Entity Info" => tr!("tray-card-entity-info"), // i18n-ignore
        "Materials" => tr!("tray-card-materials"), // i18n-ignore
        "Material Editor" => tr!("tray-card-material-editor"), // i18n-ignore
        "Components" => tr!("tray-card-components"), // i18n-ignore
        "Tags" => tr!("tray-card-tags"), // i18n-ignore
        "Outliner" => tr!("tray-card-outliner"), // i18n-ignore
        "Scenes" => tr!("tray-card-scenes"), // i18n-ignore
        "Shadows" => tr!("tray-card-shadows"), // i18n-ignore
        "Environment" => tr!("tray-card-environment"), // i18n-ignore
        "Soften Edges" => tr!("tray-card-soften-edges"), // i18n-ignore
        "Instructor" => tr!("tray-card-instructor"), // i18n-ignore
        other => other.to_string(),
    }
}

fn card<R>(ui: &mut Ui, title: &str, icon: Icon, default_open: bool, body: impl FnOnce(&mut Ui) -> R) {
    card_ex(ui, title, icon, default_open, false, None, body);
}

/// [`card`] that can be forced open and have an icon button at the right of its header
/// (`button`: icon, tooltip). True when that button was clicked.
fn card_ex<R>(ui: &mut Ui, title: &str, icon: Icon, default_open: bool, force_open: bool, button: Option<(Icon, &str)>, body: impl FnOnce(&mut Ui) -> R) -> bool {
    let mut clicked = false;
    let r = Frame::new()
        .fill(theme::card_bg())
        .stroke(Stroke::new(1.0, theme::border()))
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
                    super::tray_lib::spot(format!("card:{title}"), rect.center());
                    super::tray_lib::spot(format!("card_toggle:{title}"), rect.center() - vec2(17.0, 0.0));
                    ui.label(RichText::new(card_label(title)).strong());
                    if let Some((icon, tip)) = button {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let r = super::icon_button(ui, icon, false, 22.0).on_hover_text(tip);
                            super::tray_lib::spot(format!("card_btn:{title}"), r.rect.center());
                            clicked = r.clicked();
                        });
                    }
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
    clicked
}

fn tag_name(model: &Model, t: Option<TagId>) -> String {
    t.and_then(|t| model.tags.get(t)).map(|t| t.name.clone()).unwrap_or_else(|| tr!("tray-untagged"))
}

/// Tag picker; returns Some(new) when changed.
fn tag_combo(ui: &mut Ui, model: &Model, current: Option<TagId>, salt: &str) -> Option<Option<TagId>> {
    let mut out = None;
    egui::ComboBox::from_id_salt(("tag_combo", salt))
        .selected_text(tag_name(model, current))
        .width(150.0)
        .show_ui(ui, |ui| {
            if ui.selectable_label(current.is_none(), tr!("tray-untagged")).clicked() {
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
            None => (Color32::WHITE, tr!("tray-default")),
        };
        let (rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
        ui.painter().rect(rect, 3.0, color, Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
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
        (true, true) => tr!("tray-info-group"),
        (true, false) => format!("<{def_name}>"),
    };
    (label, is_group)
}

impl VarstotasApp {
    pub(crate) fn tray_ui(&mut self, ui: &mut Ui) {
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
            if on("Entity Info") { // i18n-ignore
                let tip = tr!("tray-card-model-info-tip");
                if card_ex(ui, "Entity Info", Icon::Info, true, false, Some((Icon::ModelInfo, tip.as_str())), |ui| self.entity_info(ui)) { // i18n-ignore
                    self.actions.push(Action::ModelInfo);
                }
            }
            if on("Materials") { // i18n-ignore
                card(ui, "Materials", Icon::Paint, true, |ui| self.materials_panel(ui)); // i18n-ignore
            }
            if on("Material Editor") { // i18n-ignore
                card(ui, "Material Editor", Icon::Gear, true, |ui| self.material_editor_card(ui)); // i18n-ignore
            }
            if on("Components") { // i18n-ignore
                let open = self.library_open_requested();
                card_ex(ui, "Components", Icon::Group, false, open, None, |ui| self.library_panel(ui)); // i18n-ignore
            }
            if on("Tags") { // i18n-ignore
                card(ui, "Tags", Icon::Eye, false, |ui| self.tags_panel(ui)); // i18n-ignore
            }
            if on("Outliner") { // i18n-ignore
                card(ui, "Outliner", Icon::Group, true, |ui| self.outliner(ui)); // i18n-ignore
            }
            if on("Scenes") { // i18n-ignore
                card(ui, "Scenes", Icon::ViewIso, false, |ui| self.scenes_panel(ui)); // i18n-ignore
            }
            if on("Shadows") { // i18n-ignore
                card(ui, "Shadows", Icon::Shadows, false, |ui| self.shadows_panel(ui)); // i18n-ignore
            }
            if on("Environment") { // i18n-ignore
                card(ui, "Environment", Icon::StyleRealistic, false, |ui| self.environment_panel(ui)); // i18n-ignore
            }
            if on("Soften Edges") { // i18n-ignore
                card(ui, "Soften Edges", Icon::Arc, false, |ui| self.soften_panel(ui)); // i18n-ignore
            }
            if on("Instructor") { // i18n-ignore
                card(ui, "Instructor", Icon::Select, false, |ui| self.instructor(ui)); // i18n-ignore
            }
            if hidden.len() >= TRAY_CARDS.len() {
                ui.label(RichText::new(tr!("tray-cards-all-off")).color(theme::text_dim()));
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
        ui.checkbox(&mut sh.enabled, tr!("tray-shadows-show"));
        egui::Grid::new("shadows").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
            ui.label(tr!("tray-shadows-time"));
            ui.add(egui::Slider::new(&mut sh.minutes, 0..=24 * 60 - 1).step_by(5.0).custom_formatter(|v, _| {
                let m = v as u32;
                format!("{:02}:{:02}", m / 60, m % 60)
            }).custom_parser(|t| {
                let (h, m) = t.trim().split_once(':')?;
                Some((h.trim().parse::<u32>().ok()? * 60 + m.trim().parse::<u32>().ok()?) as f64)
            }));
            ui.end_row();
            ui.label(tr!("tray-shadows-date"));
            ui.add(egui::Slider::new(&mut sh.day, 1..=365).custom_formatter(|v, _| {
                let (m, d) = varstotas_kernel::shadows::month_day(v as u32);
                format!("{d} {}", varstotas_kernel::shadows::month_name(m))
            }));
            ui.end_row();
            ui.label(tr!("tray-shadows-light"));
            ui.add(egui::Slider::new(&mut sh.light, 0.0..=1.0).custom_formatter(|v, _| format!("{:.0}", v * 100.0)));
            ui.end_row();
            ui.label(tr!("tray-shadows-dark"));
            ui.add(egui::Slider::new(&mut sh.dark, 0.0..=1.0).custom_formatter(|v, _| format!("{:.0}", v * 100.0)));
            ui.end_row();
            ui.label(tr!("tray-shadows-display"));
            ui.horizontal(|ui| {
                ui.checkbox(&mut sh.on_faces, tr!("tray-shadows-on-faces"));
                ui.checkbox(&mut sh.on_ground, tr!("tray-shadows-on-ground"));
            });
            ui.end_row();
        });
        egui::CollapsingHeader::new(tr!("tray-shadows-location")).id_salt("shadow_location").show(ui, |ui| {
            egui::Grid::new("shadow_loc").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                ui.label(tr!("tray-shadows-latitude"));
                ui.add(egui::DragValue::new(&mut sh.latitude).range(-90.0..=90.0).speed(0.1).suffix("°"));
                ui.end_row();
                ui.label(tr!("tray-shadows-longitude"));
                ui.add(egui::DragValue::new(&mut sh.longitude).range(-180.0..=180.0).speed(0.1).suffix("°"));
                ui.end_row();
                ui.label(tr!("tray-shadows-utc-offset"));
                ui.add(egui::DragValue::new(&mut sh.utc_offset).range(-12.0..=14.0).speed(0.05).fixed_decimals(1).suffix(" h"));
                ui.end_row();
                ui.label(tr!("tray-shadows-north-angle"));
                ui.add(egui::DragValue::new(&mut sh.north_deg).range(0.0..=360.0).speed(0.5).suffix("°"));
                ui.end_row();
            });
        });
        let alt = sh.sun_altitude_deg();
        ui.label(RichText::new(if alt > 0.0 { tr!("tray-sun-above", alt = format!("{alt:.0}")) } else { tr!("tray-sun-below") }).color(theme::text_dim()));
        if sh != self.doc.model.shadows {
            self.actions.push(Action::SetShadows(sh));
        }
    }

    fn soften_panel(&mut self, ui: &mut Ui) {
        let n = self.soften_targets().len();
        let st = &mut self.tray.soften;
        ui.label(tr!("tray-soften-allow-normals"));
        let resp = ui.add_enabled(n > 0, egui::Slider::new(&mut st.angle, 0.0..=180.0).suffix("°").fixed_decimals(0));
        let smooth = ui.add_enabled(n > 0, egui::Checkbox::new(&mut st.smooth, tr!("tray-soften-smooth-normals")));
        let coplanar = ui.add_enabled(n > 0, egui::Checkbox::new(&mut st.coplanar, tr!("tray-soften-coplanar")));
        if n == 0 {
            ui.label(RichText::new(tr!("tray-soften-select")).color(theme::text_dim()));
        } else {
            ui.label(RichText::new(tr!("tray-soften-edges-count", n = n)).color(theme::text_dim()));
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
        let sel: Vec<EntityRef> = self.doc.selection.iter().take(2).copied().collect();
        let guides: Vec<GuideRef> = self.doc.guide_sel.iter().take(2).copied().collect();
        if let Some(i) = self.doc.section_sel {
            self.section_info(ui, i);
            return;
        }
        if let Some(i) = self.doc.light_sel {
            self.light_info(ui, i);
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
        if guides.is_empty() && self.doc.dim_sel.is_empty() && let Some(cid) = self.selected_spline() {
            self.spline_info(ui, cid);
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
                ui.label(tr!("tray-info-invalid-context"));
                return;
            };
            egui::Grid::new("entity_info").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                match sel.as_slice() {
                    [] => {
                        ui.label(RichText::new(tr!("tray-info-no-selection")).strong());
                        ui.end_row();
                        ui.label(tr!("tray-info-units"));
                        let mut u = unit;
                        egui::ComboBox::from_id_salt("units")
                            .selected_text(super::unit_suffix(u))
                            .show_ui(ui, |ui| {
                                for (v, n) in [
                                    (LengthUnit::Millimeter, tr!("tray-unit-millimeters")),
                                    (LengthUnit::Centimeter, tr!("tray-unit-centimeters")),
                                    (LengthUnit::Meter, tr!("tray-unit-meters")),
                                    (LengthUnit::Inch, tr!("tray-unit-inches")),
                                    (LengthUnit::Foot, tr!("tray-unit-feet")),
                                ] {
                                    ui.selectable_value(&mut u, v, n);
                                }
                            });
                        if u != unit {
                            set_unit = Some(u);
                        }
                        ui.end_row();
                        ui.label(tr!("tray-info-faces"));
                        ui.label(mesh.faces.len().to_string());
                        ui.end_row();
                        ui.label(tr!("tray-info-edges"));
                        ui.label(mesh.edges.len().to_string());
                        ui.end_row();
                        let groups = mesh
                            .instances
                            .iter()
                            .filter(|(_, i)| model.defs.get(i.def).is_none_or(|d| d.is_group))
                            .count();
                        ui.label(tr!("tray-info-groups"));
                        ui.label(groups.to_string());
                        ui.end_row();
                        ui.label(tr!("tray-info-components"));
                        ui.label((mesh.instances.len() - groups).to_string());
                        ui.end_row();
                    }
                    [EntityRef::Face(id)] => {
                        let Some(f) = mesh.faces.get(*id) else { return };
                        ui.label(RichText::new(tr!("tray-info-face")).strong());
                        ui.end_row();
                        ui.label(tr!("tray-info-area"));
                        ui.label(fmt_area(mesh.face_area(*id), unit));
                        ui.end_row();
                        ui.label(tr!("tray-info-front"));
                        material_label(ui, model, f.front);
                        ui.end_row();
                        ui.label(tr!("tray-info-back"));
                        material_label(ui, model, f.back);
                        ui.end_row();
                        ui.label(tr!("tray-info-tag"));
                        set_tag = tag_combo(ui, model, f.tag, "face");
                        ui.end_row();
                    }
                    [EntityRef::Edge(id)] => {
                        let Some(e) = mesh.edges.get(*id) else { return };
                        ui.label(RichText::new(tr!("tray-info-edge")).strong());
                        ui.end_row();
                        ui.label(tr!("tray-info-length"));
                        ui.label(fmt_len(mesh.edge_length(*id), unit));
                        ui.end_row();
                        ui.label(tr!("tray-info-soft"));
                        let mut soft = e.soft;
                        if ui.checkbox(&mut soft, "").changed() {
                            toggle_edge_soft = Some((soft, soft));
                        }
                        ui.end_row();
                        ui.label(tr!("tray-info-smooth"));
                        let mut smooth = e.smooth;
                        if ui.checkbox(&mut smooth, "").changed() {
                            toggle_edge_soft = Some((e.soft, smooth));
                        }
                        ui.end_row();
                        ui.label(tr!("tray-info-tag"));
                        set_tag = tag_combo(ui, model, e.tag, "edge");
                        ui.end_row();
                    }
                    [EntityRef::Instance(id)] => {
                        let Some(inst) = mesh.instances.get(*id) else { return };
                        let (_, is_group) = instance_label(model, mesh, *id);
                        let solid = model.defs.get(inst.def).filter(|d| d.mesh.is_solid());
                        let parametric = model.defs.get(inst.def).is_some_and(|d| d.params.is_some());
                        let kind = if parametric { tr!("tray-info-parametric-group") } else if is_group { tr!("tray-info-group") } else { tr!("tray-info-component") };
                        ui.label(RichText::new(if solid.is_some() { tr!("tray-info-solid", kind = kind) } else { kind }).strong());
                        ui.end_row();
                        if let Some(d) = solid {
                            // Signed volume of the definition, scaled by the instance transform.
                            let v = d.mesh.volume() * inst.transform.determinant().abs();
                            ui.label(tr!("tray-info-volume"));
                            ui.label(fmt_volume(v.abs(), unit));
                            ui.end_row();
                        }
                        ui.label(tr!("tray-info-instance"));
                        if self.tray.inst_name.as_ref().map(|(i, _)| *i) != Some(*id) {
                            self.tray.inst_name = Some((*id, inst.name.clone()));
                        }
                        if let Some((_, buf)) = self.tray.inst_name.as_mut() {
                            let resp = ui.add(egui::TextEdit::singleline(buf).hint_text(tr!("tray-info-name-hint")).desired_width(150.0));
                            if !resp.has_focus() && !resp.lost_focus() && *buf != inst.name {
                                // Model changed underneath (undo): resync.
                                *buf = inst.name.clone();
                            } else if committed(ui, &resp) && *buf != inst.name {
                                rename = Some((*id, buf.clone()));
                            }
                        }
                        ui.end_row();
                        if !is_group {
                            ui.label(tr!("tray-info-definition"));
                            ui.add(egui::Label::new(model.defs.get(inst.def).map(|d| d.name.clone()).unwrap_or_default()).truncate());
                            ui.end_row();
                        }
                        ui.label(tr!("tray-info-material"));
                        material_label(ui, model, inst.material);
                        ui.end_row();
                        ui.label(tr!("tray-info-tag"));
                        set_tag = tag_combo(ui, model, inst.tag, "inst");
                        ui.end_row();
                        // A downloaded model's credit (3D Library).
                        if let Some(c) = model.defs.get(inst.def).and_then(|d| d.source.as_ref()) {
                            ui.label(tr!("tray-info-source"));
                            let lic = if c.license.is_empty() { tr!("tray-info-licence-unknown") } else { c.license.clone() };
                            let who = if c.author.is_empty() { c.source.clone() } else { format!("{} · {}", c.author, c.source) };
                            ui.add(egui::Label::new(format!("{who} · {lic}")).truncate()).on_hover_text(tr!("tray-info-source-hover", line = c.line()));
                            ui.end_row();
                            // What the licence asks (an ND badge, NC / SA / unknown reminders).
                            let kind = c.license_kind();
                            if !kind.note().is_empty() {
                                ui.label("");
                                let col = if matches!(kind, varstotas_kernel::LicenseKind::CcByNd | varstotas_kernel::LicenseKind::CcByNc | varstotas_kernel::LicenseKind::Unknown) { egui::Color32::from_rgb(220, 150, 50) } else { ui.visuals().weak_text_color() };
                                ui.add(egui::Label::new(RichText::new(format!("{} — {}", kind.label(), kind.note())).small().color(col)).truncate());
                                ui.end_row();
                            }
                        }
                    }
                    _ => {
                        let SelSummary { count, faces, edges, groups, components, guides, dims, area } = summary;
                        let only_guides = guides + dims == count;
                        let title = if guides == count {
                            tr!("tray-info-n-guides", count = count)
                        } else if dims == count {
                            tr!("tray-info-n-dimensions", count = count)
                        } else {
                            tr!("tray-info-n-entities", count = count)
                        };
                        ui.label(RichText::new(title).strong());
                        ui.end_row();
                                        let mut parts = Vec::new();
                        if faces > 0 {
                            parts.push(tr!("tray-count-faces", n = faces));
                        }
                        if edges > 0 {
                            parts.push(tr!("tray-count-edges", n = edges));
                        }
                        if groups > 0 {
                            parts.push(tr!("tray-count-groups", n = groups));
                        }
                        if components > 0 {
                            parts.push(tr!("tray-count-components", n = components));
                        }
                        if guides > 0 {
                            parts.push(tr!("tray-count-guides", n = guides));
                        }
                        if dims > 0 {
                            parts.push(tr!("tray-count-dimensions", n = dims));
                        }
                        ui.label(tr!("tray-info-contains"));
                        ui.add(egui::Label::new(parts.join(", ")).wrap());
                        ui.end_row();
                        if faces > 0 {
                            ui.label(tr!("tray-info-area"));
                            ui.label(fmt_area(area, unit));
                            ui.end_row();
                        }
                        if !only_guides {
                            ui.label(tr!("tray-info-tag"));
                            set_tag = tag_combo(ui, model, None, "many");
                            ui.end_row();
                        }
                    }
                }
            });
        });
        if let Err(e) = r {
            ui.label(RichText::new(tr!("tray-kernel-not-ready", error = e)).color(theme::text_dim()));
        }

        if let Some(u) = set_unit {
            self.doc.mutate_untracked(|m| m.unit = u);
        }
        if let Some((id, name)) = rename {
            let p = path.clone();
            self.doc.commit(&tr!("tray-undo-rename"), |m| {
                if let Some(i) = m.mesh_at_mut(&p).and_then(|mesh| mesh.instances.get_mut(id)) {
                    i.name = name;
                }
            });
        }
        if let Some((soft, smooth)) = toggle_edge_soft {
            let p = path.clone();
            self.doc.commit(&tr!(if soft { "tray-undo-soften-edge" } else { "tray-undo-unsoften-edge" }), |m| {
                if let Some(mesh) = m.mesh_at_mut(&p) {
                    for e in &sel {
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
            self.doc.commit(&tr!("tray-undo-set-tag"), |m| {
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
        if !self.generator_info(ui)
            && !self.rig_info(ui)
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
                if ui.button(tr!("tray-params-make")).on_hover_text(tr!("tray-params-make-hover")).clicked() {
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
            ui.label(RichText::new(tr!("tray-params-title")).strong());
            ui.end_row();
            let Some((_, bufs)) = self.tray.params_edit.as_mut() else { return };
            for k in [0, 1, 2, 4, 3] {
                ui.label(&[tr!("tray-params-length"), tr!("tray-params-width"), tr!("tray-params-height"), tr!("tray-params-gap"), tr!("tray-params-copies")][k]);
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
                    ui.label(tr!("tray-params-along"));
                    ui.horizontal(|ui| {
                        for (a, n) in ["X", "Y", "Z"].into_iter().enumerate() {
                            ui.selectable_value(&mut next.axis, a as u8, n);
                        }
                    });
                    ui.end_row();
                }
            }
            ui.label(tr!("tray-params-resize"));
            ui.horizontal(|ui| {
                ui.selectable_value(&mut next.stretch, true, tr!("tray-params-stretch")).on_hover_text(tr!("tray-params-stretch-hover"));
                ui.selectable_value(&mut next.stretch, false, tr!("tray-params-scale"));
            });
            ui.end_row();
        });
        if bad {
            self.tray.params_edit = None;
            self.flash_error(tr!("tray-invalid-value"));
        } else if next != p {
            self.tray.params_edit = None;
            self.actions.push(Action::SetParams(next));
        }
    }

    /// Thumbnail of a texture for the material swatches (decoded once).
    pub(crate) fn thumbnail(&mut self, ctx: &egui::Context, img: &varstotas_kernel::TextureImage) -> Option<egui::TextureId> {
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
    pub(crate) fn texture_size_editor(&mut self, ui: &mut Ui) {
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
            for (i, (label, buf, text)) in [(tr!("tray-tex-width"), wb, &wt), (tr!("tray-tex-height"), hb, &ht)].into_iter().enumerate() {
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
        if ui.checkbox(&mut keep, tr!("tray-tex-keep-aspect")).changed() {
            self.tray.tex_free_aspect = !keep;
        }
        if bad {
            self.tray.tex_size = None;
            self.flash_error(tr!("tray-invalid-value"));
        }
        if let Some(sz) = new {
            self.tray.tex_size = None;
            self.doc.commit(&tr!("tray-undo-edit-material"), |m| {
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
        let cid = self.selected_curve_edges()?;
        self.doc.model.mesh_at(&self.doc.active_path)?.curve_info(cid).map(|_| cid)
    }

    /// The Bezier / smooth / freehand curve when the selection is exactly its edges.
    fn selected_spline(&self) -> Option<u32> {
        let cid = self.selected_curve_edges()?;
        self.doc.model.mesh_at(&self.doc.active_path)?.spline(cid).map(|_| cid)
    }

    /// The curve id when the selection is exactly the edges of one curve.
    fn selected_curve_edges(&self) -> Option<u32> {
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
        (n == mesh.curve_edges(cid).len()).then_some(cid)
    }

    /// Entity Info for a Bezier / smooth / freehand curve: segments per span are editable,
    /// Edit Curve reshapes it.
    fn spline_info(&mut self, ui: &mut Ui, cid: u32) {
        let path = self.doc.active_path.clone();
        let unit = self.doc.model.unit;
        let Some(sp) = self.doc.model.mesh_at(&path).and_then(|m| m.spline(cid)) else { return };
        let s_text = sp.segs.to_string();
        if self.tray.curve_edit.as_ref().map(|c| c.0) != Some(cid) {
            self.tray.curve_edit = Some((cid, String::new(), s_text.clone()));
        }
        let mut apply: Option<u32> = None;
        let mut bad = false;
        egui::Grid::new("entity_info_spline").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(RichText::new(tr!(if sp.closed { "tray-curve-closed" } else { "tray-curve" })).strong());
            ui.end_row();
            ui.label(tr!("tray-curve-points"));
            let smooth = sp.anchors.iter().filter(|a| !a.is_corner()).count();
            ui.label(tr!("tray-curve-points-value", count = sp.anchors.len(), smooth = smooth));
            ui.end_row();
            let Some((_, _, sb)) = self.tray.curve_edit.as_mut() else { return };
            ui.label(tr!("tray-curve-segments")).on_hover_text(tr!("tray-curve-segments-hover"));
            let resp = ui.add(egui::TextEdit::singleline(sb).desired_width(120.0));
            if committed(ui, &resp) && *sb != s_text {
                match sb.trim().trim_end_matches(['s', 'S']).trim().parse::<u32>() {
                    Ok(n) => apply = Some(n),
                    Err(_) => bad = true,
                }
            } else if !resp.has_focus() && *sb != s_text {
                *sb = s_text.clone();
            }
            ui.end_row();
            ui.label(tr!("tray-info-length"));
            ui.label(fmt_len(sp.length(), unit));
            ui.end_row();
        });
        let edit = ui.button(tr!("tray-curve-edit")).on_hover_text(tr!("tray-curve-edit-hover"));
        super::tray_lib::spot("entity_info:edit_curve", edit.rect.center());
        if edit.clicked() {
            self.actions.push(Action::SetTool(crate::tools::ToolId::EditCurve));
        }
        if bad {
            self.tray.curve_edit = None;
            self.flash_error(tr!("tray-invalid-value"));
        }
        if let Some(n) = apply {
            let new = varstotas_kernel::Spline { segs: n, ..sp };
            let edges = self.doc.try_commit(&tr!("tray-undo-edit-segments"), |m| match m.mesh_at_mut(&path) {
                Some(mesh) => mesh.set_spline(cid, &new),
                None => Err(varstotas_kernel::OpError::NotFound),
            });
            self.tray.curve_edit = None;
            if let Some(es) = edges {
                self.doc.set_selection(es.into_iter().map(EntityRef::Edge).collect());
            } else if let Some(e) = self.doc.last_error.clone() {
                self.flash_error(e);
            }
        }
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
                varstotas_kernel::CurveKind::Arc => tr!("tray-curve-arc"),
                varstotas_kernel::CurveKind::Circle => tr!("tray-curve-circle"),
                varstotas_kernel::CurveKind::Polygon => tr!("tray-curve-polygon"),
            };
            ui.label(RichText::new(title).strong());
            ui.end_row();
            let Some((_, rb, sb)) = self.tray.curve_edit.as_mut() else { return };
            ui.label(tr!("tray-curve-radius"));
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
            ui.label(if info.kind == varstotas_kernel::CurveKind::Polygon { tr!("tray-curve-sides") } else { tr!("tray-curve-segments") });
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
            ui.label(if info.is_closed() { tr!("tray-curve-circumference") } else { tr!("tray-curve-arc-length") });
            ui.label(fmt_len(info.length(), unit));
            ui.end_row();
            if info.kind == varstotas_kernel::CurveKind::Arc {
                ui.label(tr!("tray-curve-angle"));
                ui.label(crate::tools::common::fmt_angle(info.sweep().to_degrees()));
                ui.end_row();
            }
        });
        if bad {
            self.tray.curve_edit = None;
            self.flash_error(tr!("tray-invalid-value"));
        }
        if let Some((r, n)) = apply {
            let label = tr!(if r.is_some() { "tray-undo-edit-radius" } else { "tray-undo-edit-segments" });
            let edges = self.doc.try_commit(&label, |m| match m.mesh_at_mut(&path) {
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
            ui.label(RichText::new(tr!(if g.leader { "tray-dim-text" } else { "tray-dim-linear" })).strong());
            ui.end_row();
            if !g.leader {
                ui.label(tr!("tray-info-length"));
                ui.label(fmt_len(g.length(), unit));
                ui.end_row();
            }
            let Some((_, buf)) = self.tray.dim_text.as_mut() else { return };
            ui.label(tr!("tray-dim-text"));
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
            let label = tr!(if g.leader { "tray-undo-edit-text" } else { "tray-undo-edit-dimension-text" });
            self.doc.commit(&label, |m| m.set_dimension_text_in(&path, id, t));
            self.tray.dim_text = None;
        }
    }

    /// Entity Info for a single selected guide line / guide point (world coordinates).
    /// A selected light: its properties (one undo step per edit, as in the Render window).
    fn light_info(&mut self, ui: &mut Ui, i: usize) {
        let Some(mut l) = self.doc.model.lights.get(i).cloned() else { return };
        // Compared sanitized: re-normalizing the direction the aim handle wrote may change its
        // last bits, which must not count as an edit (it would take over the drag as "Edit Lights").
        let before = l.clone().sanitized();
        ui.label(RichText::new(tr!("tray-light-title")).strong());
        crate::ui::render::light_editor(ui, "entity_info_light", &mut l, self.doc.model.unit, &self.camera);
        ui.horizontal(|ui| {
            if ui.button(tr!("tray-light-render")).on_hover_text(tr!("tray-light-render-hover")).clicked() {
                self.render_window.open = true;
            }
            if ui.button(tr!("delete")).clicked() {
                self.actions.push(Action::Delete);
            }
        });
        ui.label(RichText::new(tr!("tray-light-hint")).small().weak());
        let l = l.sanitized();
        if l != before {
            self.edit_light(i, l);
        }
    }

    fn section_info(&mut self, ui: &mut Ui, i: usize) {
        let Some(s) = self.doc.model.sections.get(i).cloned() else { return };
        let unit = self.doc.model.unit;
        if self.tray.section_name.as_ref().is_none_or(|n| n.0 != i) {
            self.tray.section_name = Some((i, s.name.clone()));
        }
        let mut rename = None;
        let mut active = s.active;
        egui::Grid::new("entity_info_section").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(RichText::new(tr!("tray-section-title")).strong());
            ui.end_row();
            ui.label(tr!("tray-section-name"));
            if let Some((_, buf)) = self.tray.section_name.as_mut() {
                let resp = ui.add(egui::TextEdit::singleline(buf).desired_width(150.0));
                if committed(ui, &resp) && buf.trim() != s.name && !buf.trim().is_empty() {
                    rename = Some(buf.trim().to_string());
                } else if !resp.has_focus() && *buf != s.name {
                    *buf = s.name.clone();
                }
            }
            ui.end_row();
            ui.label(tr!("tray-section-active-cut"));
            ui.checkbox(&mut active, "");
            ui.end_row();
            let p = s.origin;
            ui.label(tr!("tray-through"));
            ui.add(egui::Label::new(format!("{}, {}, {}", fmt_len(p.x, unit), fmt_len(p.y, unit), fmt_len(p.z, unit))).wrap());
            ui.end_row();
            ui.label(tr!("tray-section-normal"));
            ui.label(format!("{:.3}, {:.3}, {:.3}", s.normal.x, s.normal.y, s.normal.z));
            ui.end_row();
        });
        ui.horizontal(|ui| {
            if ui.button(tr!("tray-section-reverse")).clicked() {
                self.actions.push(Action::ReverseSection);
            }
            if ui.button(tr!("tray-section-align-view")).clicked() {
                self.actions.push(Action::AlignSectionView);
            }
        });
        if active != s.active {
            self.actions.push(Action::ActiveCut);
        }
        if let Some(name) = rename {
            self.tray.section_name = None;
            self.doc.commit(&tr!("tray-undo-rename-section"), |m| {
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
                ui.label(RichText::new(tr!("tray-guide-line")).strong());
                ui.end_row();
                ui.label(tr!("tray-through"));
                ui.add(egui::Label::new(pt(l.origin)).wrap());
                ui.end_row();
                let d = l.dir.normalize_or_zero();
                ui.label(tr!("tray-guide-direction"));
                ui.label(format!("{:.3}, {:.3}, {:.3}", d.x, d.y, d.z));
                ui.end_row();
            }
            Some(varstotas_kernel::GuideGeom::Point(p)) => {
                ui.label(RichText::new(tr!("tray-guide-point")).strong());
                ui.end_row();
                ui.label(tr!("tray-guide-position"));
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
                None => (Color32::WHITE, tr!("tray-default")),
            };
            let (rect, _) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::hover());
            ui.painter().rect(rect, 6.0, color, Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
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
                ui.label(RichText::new(tr!("tray-mat-current")).small().color(theme::text_dim()));
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
            let r = swatch(ui, Color32::WHITE, size, cur.is_none()).on_hover_text(tr!("tray-default"));
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
            ui.label(tr!("tray-mat-custom"));
            if egui::color_picker::color_edit_button_srgba(ui, &mut c, egui::color_picker::Alpha::OnlyBlend).changed() {
                self.tray.custom_color = Some(c);
            }
            if ui.button(tr!("tray-mat-use")).clicked() {
                let [r, g, b, _] = c.to_srgba_unmultiplied();
                picked = Some(Some(PaletteMaterial::color(format!("Color #{r:02X}{g:02X}{b:02X}"), color_f32(c)))); // i18n-ignore
            }
        });

        self.material_library_ui(ui);
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
        self.doc.commit(&tr!("tray-undo-delete-tag"), |m| m.delete_tag(id));
        self.flash(if uses > 0 { tr!("tray-tag-deleted-uses", name = name.as_str(), uses = uses) } else { tr!("tray-tag-deleted", name = name.as_str()) });
    }

    fn tags_panel(&mut self, ui: &mut Ui) {
        let mut toggle: Option<TagId> = None;
        let mut lock_tag: Option<TagId> = None;
        let mut rename_commit: Option<(TagId, String)> = None;
        let mut delete: Option<TagId> = None;
        let mut add = false;
        ui.horizontal(|ui| {
            if small_icon_button(ui, Icon::Plus, &tr!("tray-tags-add")).clicked() {
                add = true;
            }
            ui.label(RichText::new(tr!("tray-tags-header")).small().color(theme::text_dim()));
        });
        let un_shown = !self.doc.model.untagged_hidden;
        ui.horizontal(|ui| {
            let r = small_icon_button(ui, if un_shown { Icon::Eye } else { Icon::EyeOff }, &tr!("tray-tags-untagged-hover"));
            super::tray_lib::spot("tag_eye:Untagged", r.rect.center());
            if r.clicked() {
                self.doc.mutate_untracked(|m| m.untagged_hidden = !m.untagged_hidden);
            }
            let locked = self.doc.model.untagged_locked;
            let r = small_icon_button(ui, if locked { Icon::Lock } else { Icon::Unlock }, &tr!(if locked { "cmd-unlock" } else { "cmd-lock" }));
            super::tray_lib::spot("tag_lock:Untagged", r.rect.center());
            if r.clicked() {
                self.doc.change_locks(&tr!("cmd-lock"), |m| m.untagged_locked = !m.untagged_locked);
            }
            ui.label(if un_shown { RichText::new(tr!("tray-untagged")) } else { RichText::new(tr!("tray-untagged")).color(theme::text_dim()) });
        });
        let tags: Vec<(TagId, String, bool, [f32; 4])> = self
            .doc
            .model
            .tags
            .iter()
            .map(|(id, t)| (id, t.name.clone(), t.visible, t.color))
            .collect();
        for (id, name, visible, color) in tags {
            let locked = self.doc.model.tags[id].locked;
            ui.horizontal(|ui| {
                if small_icon_button(ui, if visible { Icon::Eye } else { Icon::EyeOff }, &tr!("tray-tags-toggle")).clicked() {
                    toggle = Some(id);
                }
                let r = small_icon_button(ui, if locked { Icon::Lock } else { Icon::Unlock }, &tr!(if locked { "cmd-unlock" } else { "cmd-lock" }));
                super::tray_lib::spot(format!("tag_lock:{name}"), r.rect.center());
                if r.clicked() { lock_tag = Some(id); }
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
                        let text = if visible { RichText::new(&name) } else { RichText::new(&name).color(theme::text_dim()) };
                        let r = ui.add(egui::Label::new(text).sense(Sense::click())).on_hover_text(tr!("tray-tags-rename-hover"));
                        if r.double_clicked() {
                            self.tray.tag_rename = Some((id, name.clone()));
                        }
                        r.context_menu(|ui| {
                            if ui.button(tr!("tray-rename")).clicked() {
                                self.tray.tag_rename = Some((id, name.clone()));
                                ui.close();
                            }
                            if ui.button(tr!("tray-undo-delete-tag")).clicked() {
                                delete = Some(id);
                                ui.close();
                            }
                        });
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let r = small_icon_button(ui, Icon::Trash, &tr!("tray-tags-delete-hover"));
                    super::tray_lib::spot("tag_delete", r.rect.center());
                    if r.clicked() {
                        delete = Some(id);
                    }
                });
            });
        }
        if let Some(id) = lock_tag {
            self.doc.change_locks(&tr!("cmd-lock"), |m| m.tags[id].locked = !m.tags[id].locked);
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
                self.doc.commit(&tr!("tray-undo-rename-tag"), |m| {
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
            let name = tr!("tray-tag-new-name", n = n);
            if let Some(id) = self.doc.commit(&tr!("tray-undo-add-tag"), |m| m.add_tag(&name)) {
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
            let hdr = ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                paint_icon(ui.painter(), rect, Icon::ViewIso, false);
                ui.label(RichText::new(self.doc.display_name()).strong());
            });
            // Drop on the document: move to the top level.
            drop_target(ui, model, &hdr.response, &Vec::new(), &mut actions);
            if root.instances.is_empty() {
                ui.label(RichText::new(tr!("tray-outliner-empty")).color(theme::text_dim()));
            }
            outliner_level(ui, model, root, &Vec::new(), &self.doc.active_path, &self.doc.selection, &mut actions);
        });
        outliner_drag_label(ui.ctx());
        if let Err(e) = r {
            ui.label(RichText::new(tr!("tray-kernel-not-ready", error = e)).color(theme::text_dim()));
        }
        self.actions.extend(actions);
    }

    // ---- Scenes --------------------------------------------------------------

    fn scenes_panel(&mut self, ui: &mut Ui) {
        let sel = self.tray.selected_scene;
        ui.horizontal(|ui| {
            if small_icon_button(ui, Icon::Plus, &tr!("tray-scenes-add")).clicked() {
                self.actions.push(Action::AddScene);
            }
            ui.add_enabled_ui(sel.is_some(), |ui| {
                if small_icon_button(ui, Icon::Refresh, &tr!("tray-scenes-update")).clicked()
                    && let Some(i) = sel {
                        self.actions.push(Action::UpdateScene(i));
                    }
                if small_icon_button(ui, Icon::Trash, &tr!("tray-scenes-delete")).clicked()
                    && let Some(i) = sel {
                        self.actions.push(Action::DeleteScene(i));
                    }
                if ui.small_button("↑").on_hover_text(tr!("tray-scenes-up")).clicked()
                    && let Some(i) = sel {
                        self.actions.push(Action::MoveScene(i, false));
                    }
                if ui.small_button("↓").on_hover_text(tr!("tray-scenes-down")).clicked()
                    && let Some(i) = sel {
                        self.actions.push(Action::MoveScene(i, true));
                    }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = tr!(if self.scene_player.is_some() { "tray-scenes-pause" } else { "tray-scenes-play" });
                if ui
                    .add_enabled(!self.doc.model.scenes.is_empty(), egui::Button::new(label).small())
                    .on_hover_text(tr!("tray-scenes-play-hover"))
                    .clicked()
                {
                    self.actions.push(Action::ToggleAnimation);
                }
            });
        });
        if self.doc.model.scenes.is_empty() {
            ui.label(RichText::new(tr!("tray-scenes-empty")).color(theme::text_dim()));
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
                        .on_hover_text(tr!("tray-scenes-click-hover"));
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
            if ui.checkbox(&mut inc, tr!("tray-scenes-include")).changed() {
                self.doc.commit(&tr!("tray-undo-scene-properties"), |m| m.scenes[i].in_animation = inc);
            }
        }
        egui::CollapsingHeader::new(tr!("tray-scenes-animation")).id_salt("scene_anim_settings").show(ui, |ui| {
            let mut set = self.doc.model.scene_settings;
            ui.checkbox(&mut set.transitions, tr!("tray-scenes-transitions"));
            ui.add_enabled_ui(set.transitions, |ui| {
                ui.horizontal(|ui| {
                    ui.label(tr!("tray-scenes-transition"));
                    ui.add(egui::DragValue::new(&mut set.transition_secs).range(0.0..=60.0).speed(0.05).suffix(" s"));
                });
            });
            ui.horizontal(|ui| {
                ui.label(tr!("tray-scenes-delay"));
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
                self.doc.commit(&tr!("tray-undo-rename-scene"), |m| {
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
                    ui.label(RichText::new(tr!("tray-instructor-shortcut", key = s)).small().color(theme::text_dim()));
                }
            });
        });
        ui.label(RichText::new(id.description()).italics());
        ui.add_space(2.0);
        ui.label(id.instructor());
        if ui.link(tr!("tray-instructor-more")).clicked() {
            self.app_help_for_tool();
        }
        if !id.implemented() {
            ui.add_space(4.0);
            ui.label(RichText::new(tr!("tray-instructor-not-implemented")).color(Color32::from_rgb(0xC0, 0x60, 0x20)));
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
            let lock = small_icon_button(ui, if model.entity_locked(path, EntityRef::Instance(id)) { Icon::Lock } else { Icon::Unlock }, &tr!(if inst.locked { "cmd-unlock" } else { "cmd-lock" }));
            super::tray_lib::spot(format!("outliner_lock:{label}"), lock.rect.center());
            if lock.clicked() { actions.push(Action::ToggleInstanceLocked(path.clone(), id)); }
            if small_icon_button(ui, if inst.hidden { Icon::EyeOff } else { Icon::Eye }, &tr!("tray-outliner-hide-show")).clicked() {
                actions.push(Action::ToggleInstanceHidden(path.clone(), id));
            }
            let (rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
            paint_icon(ui.painter(), rect, if is_group { Icon::Group } else { Icon::ViewIso }, false);
            let mut text = RichText::new(label.clone());
            if inst.hidden {
                text = text.color(theme::text_dim()).italics();
            }
            let resp = ui.selectable_label(selected, text).interact(Sense::drag());
            super::tray_lib::spot(format!("outliner_row:{label}"), resp.rect.center());
            if resp.double_clicked() {
                actions.push(Action::FocusInstance(path.clone(), id));
            } else if resp.clicked() {
                actions.push(Action::SelectInstance(path.clone(), id));
            }
            if resp.drag_started() {
                egui::DragAndDrop::set_payload(ui.ctx(), OutlinerDrag { path: path.clone(), id, label: label.clone() });
            }
            drop_target(ui, model, &resp, &child_path, actions);
            resp.context_menu(|ui| {
                if ui.button(tr!("tray-outliner-zoom")).on_hover_text(tr!("tray-outliner-zoom-hover")).clicked() {
                    actions.push(Action::FocusInstance(path.clone(), id));
                    ui.close();
                }
                if ui.button(tr!(if is_group { "tray-outliner-edit-group" } else { "tray-outliner-edit-component" })).clicked() {
                    actions.push(Action::EditInstance(path.clone(), id));
                    ui.close();
                }
                if ui.button(tr!(if inst.hidden { "tray-outliner-unhide" } else { "tray-outliner-hide" })).clicked() {
                    actions.push(Action::ToggleInstanceHidden(path.clone(), id));
                    ui.close();
                }
                if ui.button(tr!(if inst.locked { "cmd-unlock" } else { "cmd-lock" })).clicked() {
                    actions.push(Action::ToggleInstanceLocked(path.clone(), id));
                    ui.close();
                }
                if ui.button(tr!("tray-outliner-explode")).clicked() {
                    actions.push(Action::SelectInstance(path.clone(), id));
                    actions.push(Action::Explode);
                    ui.close();
                }
                if !is_group
                    && model.can_make_unique(path, id)
                    && ui.button(tr!("tray-outliner-make-unique")).on_hover_text(tr!("tray-outliner-make-unique-hover")).clicked()
                {
                    actions.push(Action::SelectInstance(path.clone(), id));
                    actions.push(Action::MakeUnique);
                    ui.close();
                }
                if let Some((_, parent)) = path.split_last()
                    && ui.button(tr!("tray-outliner-move-out")).on_hover_text(tr!("tray-outliner-move-out-hover")).clicked()
                {
                    actions.push(Action::MoveInstance(path.clone(), id, parent.to_vec()));
                    ui.close();
                }
                ui.separator();
                if ui.button(tr!("delete")).clicked() {
                    actions.push(Action::DeleteInstance(path.clone(), id));
                    ui.close();
                }
            });
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

/// An outliner row being dragged (to another group, or onto the document for the top level).
#[derive(Clone)]
struct OutlinerDrag {
    path: ContextPath,
    id: InstanceId,
    label: String,
}

/// `resp` (a row, or the document heading) takes a dragged row into the context `to`: an
/// outline while one hovers that can go there, a Move on release.
fn drop_target(ui: &Ui, model: &Model, resp: &egui::Response, to: &ContextPath, actions: &mut Vec<Action>) {
    if let Some(d) = resp.dnd_hover_payload::<OutlinerDrag>()
        && model.can_move_instance(&d.path, d.id, to)
    {
        ui.painter().rect_stroke(resp.rect.expand(1.0), 4.0, Stroke::new(1.5, theme::ACCENT), egui::StrokeKind::Outside);
    }
    if let Some(d) = resp.dnd_release_payload::<OutlinerDrag>()
        && model.can_move_instance(&d.path, d.id, to)
    {
        actions.push(Action::MoveInstance(d.path.clone(), d.id, to.clone()));
    }
}

/// While a row is dragged: its name at the pointer.
fn outliner_drag_label(ctx: &egui::Context) {
    let (Some(d), Some(p)) = (egui::DragAndDrop::payload::<OutlinerDrag>(ctx), ctx.pointer_latest_pos()) else { return };
    ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
    egui::Area::new(egui::Id::new("outliner_drag")).order(egui::Order::Tooltip).fixed_pos(p + vec2(12.0, 6.0)).interactable(false).show(ctx, |ui| {
        egui::Frame::popup(ui.style()).show(ui, |ui| ui.label(&d.label));
    });
}

/// A text field was left with its value accepted (Enter, Tab or a click elsewhere); Escape
/// leaves it too but means "revert".
pub(crate) fn committed(ui: &egui::Ui, resp: &egui::Response) -> bool {
    resp.lost_focus() && !ui.input(|i| i.key_pressed(egui::Key::Escape))
}
