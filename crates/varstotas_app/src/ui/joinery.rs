//! Window → Joinery (track W): joints between parts that touch (`varstotas_kernel::joinery`).
//! Select two groups or components, pick a kind and add a joint; the list below edits its
//! sizes, swaps its parts or deletes it. A click on a joint's name selects its two parts.
//! While the window is open the joints are outlined on the model with their numbers, the one
//! picked in the list highlighted with its fastener positions.

use crate::app::VarstotasApp;
use crate::ui::theme;
use egui::{Context, RichText, Ui, vec2};
use varstotas_i18n::tr;
use varstotas_kernel::carcass::{BackSpec, PlinthSpec};
use varstotas_kernel::drawer::{DoorSpec, DrawerFront, DrawerSpec};
use varstotas_kernel::joinery::{Contact, SlideKind, WoodJoint, WoodJointKind};
use varstotas_kernel::{EntityRef, InstanceId, LengthUnit, Model};

#[derive(Default)]
pub struct JoineryState {
    pub open: bool,
    /// The kind a new joint gets.
    pub kind: WoodJointKind,
    /// A joint being dragged (committed when the pointer is released).
    draft: Option<(usize, WoodJoint)>,
    /// The joint picked in the list (highlighted on the model).
    pub selected: Option<usize>,
    /// A new drawer's height, front and slides; its height from the floor (`None`: the
    /// opening's bottom, a little up); how many (0 or 1: one).
    drawer: Option<DrawerSpec>,
    drawer_z: Option<f64>,
    drawer_count: usize,
    /// New doors' count, front and latches.
    door: Option<DoorSpec>,
    /// A new back's and plinth's settings.
    back: Option<BackSpec>,
    plinth: Option<PlinthSpec>,
    /// A new fitting: the part it is for and its kind and sizes (suiting that part until changed).
    fitting: Option<(Vec<InstanceId>, WoodJoint)>,
}

const SLIDES: [(SlideKind, &str); 3] = [(SlideKind::BallBearing, "joinery-slide-ball"), (SlideKind::Roller, "joinery-slide-roller"), (SlideKind::Undermount, "joinery-slide-undermount")];

/// A combo box for a slide's kind.
fn slide_combo(ui: &mut Ui, id: &str, kind: &mut SlideKind) -> egui::Response {
    let name = |k: SlideKind| SLIDES.iter().find(|x| x.0 == k).map(|x| varstotas_i18n::tr(x.1)).unwrap_or_default();
    ui.label(tr!("joinery-slide")).on_hover_text(tr!("joinery-slide-tip"));
    let mut changed = false;
    let mut r = egui::ComboBox::from_id_salt(id)
        .selected_text(name(*kind))
        .show_ui(ui, |ui| {
            for (i, (k, _)) in SLIDES.into_iter().enumerate() {
                let r = ui.selectable_value(kind, k, name(k));
                super::tray_lib::spot(format!("{id}:{i}"), r.rect.center());
                changed |= r.changed();
            }
        })
        .response;
    super::tray_lib::spot(id, r.rect.center());
    if changed {
        r.mark_changed();
    }
    r
}

const JOINT_LINE: egui::Color32 = egui::Color32::from_rgb(0xE0, 0x7A, 0x10);
const JOINT_PICKED: egui::Color32 = egui::Color32::from_rgb(0x00, 0x50, 0xFF);

pub(crate) fn kind_name(k: WoodJointKind) -> String {
    varstotas_i18n::tr(match k {
        WoodJointKind::Dowels => "joinery-kind-dowels",
        WoodJointKind::Screws => "joinery-kind-screws",
        WoodJointKind::Housing => "joinery-kind-housing",
        WoodJointKind::MortiseTenon => "joinery-kind-mortise-tenon",
        WoodJointKind::BoxJoint => "joinery-kind-box",
        WoodJointKind::Dovetail => "joinery-kind-dovetail",
        WoodJointKind::PocketHole => "joinery-kind-pocket",
        WoodJointKind::Confirmat => "joinery-kind-confirmat",
        WoodJointKind::CamLock => "joinery-kind-cam-lock",
        WoodJointKind::ShelfPins => "joinery-kind-shelf-pins",
        WoodJointKind::Hinge => "joinery-kind-hinge",
        WoodJointKind::GasStrut => "joinery-kind-gas-strut",
        WoodJointKind::Slide => "joinery-kind-slide",
        WoodJointKind::Handle => "joinery-kind-handle",
        WoodJointKind::Knob => "joinery-kind-knob",
        WoodJointKind::Leg => "joinery-kind-leg",
        WoodJointKind::Caster => "joinery-kind-caster",
        WoodJointKind::TableTop => "joinery-kind-table-top",
        WoodJointKind::Nails => "joinery-kind-nails",
    })
}

/// The kinds in a combo box, wood joints then fittings under their headings; `only` keeps
/// the kinds on one part or on two.
fn kind_combo(ui: &mut Ui, id: &str, k: &mut WoodJointKind, only: Option<bool>) {
    let r = egui::ComboBox::from_id_salt(id).selected_text(kind_name(*k)).height(520.0).show_ui(ui, |ui| {
        for fitting in [false, true] {
            let kinds: Vec<WoodJointKind> = WoodJointKind::ALL.into_iter().filter(|x| x.fitting() == fitting && only.is_none_or(|o| x.single() == o)).collect();
            if kinds.is_empty() || only == Some(true) && !fitting {
                continue;
            }
            if only != Some(true) {
                ui.label(RichText::new(tr!(if fitting { "joinery-group-fittings" } else { "joinery-group-joints" })).small().color(theme::text_dim()));
            }
            for x in kinds {
                let r = ui.selectable_value(k, x, kind_name(x));
                super::tray_lib::spot(format!("{id}:{x:?}"), r.rect.center());
            }
        }
    });
    super::tray_lib::spot(id, r.response.rect.center());
}

/// A part's name: its instance name, else its definition's.
fn part_name(model: &Model, path: &[InstanceId]) -> String {
    let Some((&id, parent)) = path.split_last() else { return "?".into() };
    let Some(inst) = model.mesh_at(parent).and_then(|m| m.instances.get(id)) else { return "?".into() };
    if !inst.name.is_empty() {
        return inst.name.clone();
    }
    model.defs.get(inst.def).map(|d| d.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| tr!("tray-info-group"))
}

/// A length field in the model's units (`min..=max` mm).
fn len(ui: &mut Ui, label: &str, v: &mut f64, unit: LengthUnit, min: f64, max: f64) -> egui::Response {
    ui.label(label);
    ui.add(super::len_drag(v, unit).range(min..=max).speed(0.2))
}

fn mm(ui: &mut Ui, label: &str, v: &mut f64, unit: LengthUnit, max: f64) -> bool {
    len(ui, label, v, unit, 0.5, max).changed()
}

/// A fitting's sizes, by its kind.
fn fitting_sizes(ui: &mut Ui, j: &mut WoodJoint, unit: LengthUnit) {
    match j.kind {
        WoodJointKind::Handle => {
            mm(ui, &tr!("joinery-centres"), &mut j.size, unit, 1000.0);
            ui.checkbox(&mut j.flip, tr!("joinery-flip-back"));
        }
        WoodJointKind::Knob => {
            mm(ui, &tr!("joinery-diameter"), &mut j.diameter, unit, 100.0);
            ui.checkbox(&mut j.flip, tr!("joinery-flip-back"));
        }
        WoodJointKind::Leg | WoodJointKind::Caster => {
            mm(ui, &tr!("joinery-diameter"), &mut j.diameter, unit, 300.0);
            mm(ui, &tr!("joinery-height"), &mut j.size, unit, 2000.0);
            len(ui, &tr!("joinery-margin"), &mut j.margin, unit, 0.0, 1000.0);
        }
        _ => {}
    }
}

/// How a joint's fasteners are laid out: evenly, from the ends with a pitch, at distances.
fn layout_ui(ui: &mut Ui, j: &mut WoodJoint, c: Option<Contact>, unit: LengthUnit) {
    let len_max = c.map_or(10_000.0, |c| c.length());
    ui.horizontal_wrapped(|ui| {
        ui.label(tr!("joinery-layout"));
        let mode = if !j.at.is_empty() { 2 } else if j.pitch > 0.0 { 1 } else { 0 };
        let mut m = mode;
        let names = [tr!("joinery-layout-even"), tr!("joinery-layout-pitch"), tr!("joinery-layout-at")];
        egui::ComboBox::from_id_salt("layout").selected_text(names[m].clone()).show_ui(ui, |ui| {
            for (k, n) in names.iter().enumerate() {
                ui.selectable_value(&mut m, k, n.clone());
            }
        });
        if m != mode {
            match m {
                0 => {
                    j.at.clear();
                    j.pitch = 0.0;
                }
                1 => {
                    j.at.clear();
                    j.pitch = 32.0;
                    if j.margin <= 0.0 {
                        j.margin = 37.0;
                    }
                }
                _ => j.at = c.map(|c| j.positions(&c)).unwrap_or_default(),
            }
        }
        match m {
            2 => {
                let mut remove = None;
                for k in 0..j.at.len() {
                    ui.add(super::len_drag(&mut j.at[k], unit).range(0.0..=len_max).speed(0.5));
                    if j.at.len() > 1 && ui.small_button("−").on_hover_text(tr!("joinery-point-remove")).clicked() {
                        remove = Some(k);
                    }
                }
                if let Some(k) = remove {
                    j.at.remove(k);
                }
                if ui.small_button("+").on_hover_text(tr!("joinery-point-add")).clicked() {
                    let last = j.at.last().copied().unwrap_or(0.0);
                    j.at.push((last + 32.0).min(len_max));
                }
                j.count = j.at.len().max(1);
            }
            _ => {
                ui.label(tr!("joinery-count"));
                ui.add(egui::DragValue::new(&mut j.count).range(1..=32).speed(0.05));
                len(ui, &tr!("joinery-margin"), &mut j.margin, unit, 0.0, len_max).on_hover_text(tr!("joinery-margin-tip"));
                if m == 1 {
                    len(ui, &tr!("joinery-pitch"), &mut j.pitch, unit, 1.0, len_max);
                }
            }
        }
        if j.kind != WoodJointKind::PocketHole {
            len(ui, &tr!("joinery-offset"), &mut j.offset, unit, -500.0, 500.0).on_hover_text(tr!("joinery-offset-tip"));
        }
    });
}

impl VarstotasApp {
    /// The selected groups or components as paths from the root: two, or one for a fitting on
    /// one part (the second path empty).
    fn joint_selection(&self) -> Option<(Vec<InstanceId>, Vec<InstanceId>)> {
        let ids: Vec<InstanceId> = self.doc.selection.iter().filter_map(|e| if let EntityRef::Instance(id) = e { Some(*id) } else { None }).collect();
        let want = if self.joinery.kind.single() { 1 } else { 2 };
        if ids.len() != want || self.doc.selection.len() != want {
            return None;
        }
        let path = |id| {
            let mut p = self.doc.active_path.clone();
            p.push(id);
            p
        };
        Some((path(ids[0]), ids.get(1).map(|&id| path(id)).unwrap_or_default()))
    }

    /// Join the selected parts with a joint of the chosen kind.
    pub(crate) fn add_joint(&mut self) {
        let Some((p, q)) = self.joint_selection() else {
            self.flash_error(tr!(if self.joinery.kind.single() { "joint-err-one-part" } else { "joint-err-two-parts" }));
            return;
        };
        let kind = self.joinery.kind;
        self.doc.try_commit(&tr!("joinery-undo-add"), move |m| m.add_wood_joint(kind, p, q));
    }

    /// The joints listed in the window: those in the open group or component (or the model),
    /// its own and those of the groups and components in it, each once (a component's joint
    /// once for all its copies).
    fn listed_joints(&self) -> Vec<usize> {
        let m = &self.doc.model;
        let mut seen = std::collections::HashSet::new();
        m.joint_places.iter().enumerate().filter(|(_, p)| p.prefix.starts_with(&self.doc.active_path) && seen.insert((p.def, p.index))).map(|(i, _)| i).collect()
    }

    /// The joints for the debug dump: (kind, b, a, hardware).
    pub(crate) fn joints_dump(&self) -> Vec<(String, String, String, String)> {
        let m = &self.doc.model;
        let name = |p: &[InstanceId]| if p.is_empty() { String::new() } else { part_name(m, p) };
        m.placed_joints
            .iter()
            .enumerate()
            .map(|(i, j)| (format!("{:?}", j.kind), name(&j.b), name(&j.a), j.hardware(m.joint_contacts.get(i).copied().flatten().as_ref()).map(|(n, c)| format!("{c} × {n}")).unwrap_or_default()))
            .collect()
    }

    /// The joints on the model while the window is open: each contact outlined with its number,
    /// the picked one in blue with its fastener positions.
    pub(crate) fn joints_overlay(&self, o3: &mut varstotas_render::Overlay3d, o2: &mut Vec<crate::overlay::Marker2d>) {
        if !self.joinery.open {
            return;
        }
        let m = &self.doc.model;
        for (i, (j, c)) in m.placed_joints.iter().zip(&m.joint_contacts).enumerate() {
            let Some(c) = c else { continue };
            // with part a, wherever the exploded view put it
            let off = m.assembly_offset(&j.a);
            let picked = self.joinery.selected == Some(i);
            let (color, width) = if picked { (JOINT_PICKED, 3.0) } else { (JOINT_LINE, 1.5) };
            let corner = |u: bool, v: bool| {
                let mut p = c.lo;
                p[c.along] = if u { c.hi[c.along] } else { c.lo[c.along] };
                p[c.across] = if v { c.hi[c.across] } else { c.lo[c.across] };
                p + off
            };
            let ring = [corner(false, false), corner(true, false), corner(true, true), corner(false, true)];
            for k in 0..4 {
                crate::inference::push_line(o3, ring[k], ring[(k + 1) % 4], color, width, !picked, true);
            }
            o2.push(crate::overlay::Marker2d::Badge { at: (c.lo + c.hi) * 0.5 + off, text: (i + 1).to_string() });
            if picked {
                for p in j.points(c) {
                    o2.push(crate::overlay::Marker2d::Point { at: p + off, shape: crate::overlay::MarkerShape::Dot, color: JOINT_PICKED });
                }
            }
        }
    }

    /// The picked joint's fasteners drawn through its parts (Window → Joinery's checkbox),
    /// shaded by a light from the eye; only the sides facing the eye.
    pub(crate) fn joint_hardware_overlay(&self, o3: &mut varstotas_render::Overlay3d) {
        let (true, true, Some(i)) = (self.joinery.open, self.prefs.joint_hardware, self.joinery.selected) else { return };
        let eye = self.camera.eye;
        let fwd = self.camera.basis().forward;
        for (t, color) in self.doc.model.joint_fastener_triangles(i) {
            let n = (t[1] - t[0]).cross(t[2] - t[0]).normalize_or_zero();
            let view = if self.camera.ortho { -fwd } else { (eye - t[0]).normalize_or_zero() };
            let lit = n.dot(view);
            if lit <= 0.0 {
                continue;
            }
            let k = (0.45 + 0.55 * lit) as f32;
            o3.polygons.push(varstotas_render::OverlayPolygon {
                tris: t.map(|p| p.as_vec3().to_array()).to_vec(),
                color: [color[0] * k, color[1] * k, color[2] * k, 1.0],
                on_top: true,
            });
        }
    }

    /// The fasteners for the debug dump: (bodies, bodies shown, their names with counts, the
    /// picked joint's fastener triangles shown through its parts).
    pub(crate) fn fasteners_dump(&self) -> (usize, usize, Vec<String>, usize) {
        let m = &self.doc.model;
        let shown: Vec<&varstotas_kernel::fasteners::Fastener> = m.fasteners.iter().filter(|f| m.fastener_transform(f).is_some()).collect();
        let mut names: Vec<(String, usize)> = Vec::new();
        for f in &shown {
            match names.iter_mut().find(|n| n.0 == f.name) {
                Some(n) => n.1 += 1,
                None => names.push((f.name.clone(), 1)),
            }
        }
        let through = match (self.joinery.open, self.prefs.joint_hardware, self.joinery.selected) {
            (true, true, Some(i)) => m.joint_fastener_triangles(i).len(),
            _ => 0,
        };
        (m.fasteners.len(), shown.len(), names.into_iter().map(|(n, c)| format!("{c} × {n}")).collect(), through)
    }

    /// Where each joint's fasteners are, from its start (whole mm), for the debug dump.
    pub(crate) fn joint_points_dump(&self) -> Vec<Vec<i64>> {
        let m = &self.doc.model;
        m.placed_joints.iter().zip(&m.joint_contacts).map(|(j, c)| c.map(|c| j.positions(&c).iter().map(|p| p.round() as i64).collect()).unwrap_or_default()).collect()
    }

    fn select_joint_parts(&mut self, j: &WoodJoint) {
        let Some((&a, pa)) = j.a.split_last() else { return };
        let Some((&b, pb)) = j.b.split_last() else {
            self.doc.set_active_path(pa.to_vec());
            self.doc.set_selection(vec![EntityRef::Instance(a)]);
            return;
        };
        self.doc.set_active_path(pa.to_vec());
        self.doc.set_selection(if pa == pb { vec![EntityRef::Instance(a), EntityRef::Instance(b)] } else { vec![EntityRef::Instance(a)] });
    }

    pub(crate) fn joinery_window(&mut self, ctx: &Context) {
        if !self.joinery.open {
            return;
        }
        let pos = self.viewport.view_rect.min + vec2(60.0, 60.0);
        let mut open = true;
        let win = self.placed(egui::Window::new(tr!("joinery-title")).id(egui::Id::new("joinery_window")).open(&mut open).collapsible(false).resizable(true).default_size([460.0, 300.0]).min_width(420.0), pos);
        win.show(ctx, |ui| {
            self.joinery_add_ui(ui);
            ui.separator();
            self.joinery_list_ui(ui);
        });
        if !open {
            self.joinery.open = false;
        }
    }

    fn joinery_add_ui(&mut self, ui: &mut Ui) {
        let sel = self.joint_selection();
        ui.horizontal(|ui| {
            ui.label(tr!("joinery-kind"));
            if self.joinery.kind.single() {
                self.joinery.kind = WoodJointKind::default();
            }
            kind_combo(ui, "joinery_new_kind", &mut self.joinery.kind, Some(false));
            let single = self.joinery.kind.single();
            let need = if single { tr!("joinery-pick-one") } else { tr!("joint-err-two-parts") };
            let r = ui.add_enabled(sel.is_some(), egui::Button::new(RichText::new(tr!("joinery-add")).strong()));
            super::tray_lib::spot("joinery:add", r.rect.center());
            if r.on_hover_text(tr!("joinery-add-tip")).on_disabled_hover_text(need.clone()).clicked() {
                self.add_joint();
            }
            let r = ui.checkbox(&mut self.prefs.joint_hardware, tr!("joinery-show-hardware"));
            super::tray_lib::spot("joinery:hardware", r.rect.center());
            r.on_hover_text(tr!("joinery-show-hardware-tip"));
        });
        let hint = match &sel {
            Some((p, q)) if q.is_empty() => tr!("joinery-selected-one", a = part_name(&self.doc.model, p)),
            Some((p, q)) => tr!("joinery-selected", a = part_name(&self.doc.model, p), b = part_name(&self.doc.model, q)),
            None if self.joinery.kind.single() => tr!("joinery-pick-one"),
            None => tr!("joint-err-two-parts"),
        };
        ui.label(RichText::new(hint).small().color(theme::text_dim()));
        self.fitting_ui(ui);
        self.drawer_ui(ui);
        self.door_ui(ui);
        self.back_ui(ui);
        self.plinth_ui(ui);
    }

    /// A drawer for the opening between the two selected parts (shown only for such a pair).
    fn drawer_ui(&mut self, ui: &mut Ui) {
        let unit = self.doc.model.unit;
        let ids: Vec<InstanceId> = self.doc.selection.iter().filter_map(|e| if let EntityRef::Instance(id) = e { Some(*id) } else { None }).collect();
        let pair = (ids.len() == 2).then(|| {
            let path = |id| {
                let mut p = self.doc.active_path.clone();
                p.push(id);
                p
            };
            (path(ids[0]), path(ids[1]))
        });
        // only for two parts with an opening between them
        let Some(auto) = pair.as_ref().and_then(|(l, r)| self.doc.model.drawer_z(l, r)) else { return };
        let mut spec = self.joinery.drawer.unwrap_or_default();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(tr!("joinery-drawer")).strong());
            let count = &mut self.joinery.drawer_count;
            *count = (*count).max(1);
            ui.label(tr!("joinery-drawer-count")).on_hover_text(tr!("joinery-drawer-count-tip"));
            let c = ui.add(egui::DragValue::new(count).range(1..=12).speed(0.05));
            super::tray_lib::spot("joinery:drawer_count", c.rect.center());
            let count = *count;
            let mut z = self.joinery.drawer_z.unwrap_or(auto);
            if count == 1 {
                len(ui, &tr!("joinery-height"), &mut spec.height, unit, 40.0, 2000.0);
                if len(ui, &tr!("joinery-drawer-z"), &mut z, unit, 0.0, 10_000.0).changed() {
                    self.joinery.drawer_z = Some(z);
                }
            }
            ui.label(tr!("joinery-drawer-front"));
            let fronts = [(DrawerFront::Inset, "joinery-drawer-front-inset"), (DrawerFront::Overlay, "joinery-drawer-front-overlay"), (DrawerFront::None, "joinery-drawer-front-none")];
            let name = |f: DrawerFront| fronts.iter().find(|x| x.0 == f).map(|x| varstotas_i18n::tr(x.1)).unwrap_or_default();
            let c = egui::ComboBox::from_id_salt("drawer_front").selected_text(name(spec.front)).show_ui(ui, |ui| {
                for (i, (f, _)) in fronts.into_iter().enumerate() {
                    let r = ui.selectable_value(&mut spec.front, f, name(f));
                    super::tray_lib::spot(format!("joinery:drawer_front:{i}"), r.rect.center());
                }
            });
            super::tray_lib::spot("joinery:drawer_front", c.response.rect.center());
            slide_combo(ui, "drawer_slide", &mut spec.slide);
            let c = ui.checkbox(&mut spec.push, tr!("joinery-push"));
            super::tray_lib::spot("joinery:drawer_push", c.rect.center());
            c.on_hover_text(tr!("joinery-push-tip"));
            let r = ui.button(tr!("joinery-drawer-add"));
            super::tray_lib::spot("joinery:drawer", r.rect.center());
            if r.on_hover_text(tr!("joinery-drawer-tip")).clicked()
                && let Some((l, r)) = pair.clone()
            {
                let spec = DrawerSpec { z: z.max(0.0), ..spec };
                let space = self.doc.model.drawer_space(&l, &r);
                let add = move |m: &mut Model| match space {
                    Some(space) if count > 1 => m.add_drawers(&l, &r, count, space, spec).map(|_| ()),
                    _ => m.add_drawer(&l, &r, spec).map(|_| ()),
                };
                if self.doc.try_commit(&tr!("joinery-drawer-undo"), add).is_some() {
                    self.joinery.drawer_z = None;
                }
            }
        });
        self.joinery.drawer = Some(spec);
    }

    /// Doors for the opening between the two selected parts (shown only for such a pair).
    fn door_ui(&mut self, ui: &mut Ui) {
        let ids: Vec<InstanceId> = self.doc.selection.iter().filter_map(|e| if let EntityRef::Instance(id) = e { Some(*id) } else { None }).collect();
        if ids.len() != 2 {
            return;
        }
        let path = |id| {
            let mut p = self.doc.active_path.clone();
            p.push(id);
            p
        };
        let (l, r) = (path(ids[0]), path(ids[1]));
        let Some(space) = self.doc.model.door_space(&l, &r) else { return };
        let mut spec = self.joinery.door.unwrap_or_default();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(tr!("joinery-doors")).strong());
            let c = ui.add(egui::DragValue::new(&mut spec.count).range(1..=2).speed(0.05));
            super::tray_lib::spot("joinery:doors_count", c.rect.center());
            c.on_hover_text(tr!("joinery-doors-count-tip"));
            ui.label(tr!("joinery-drawer-front"));
            let name = |inset: bool| tr!(if inset { "joinery-drawer-front-inset" } else { "joinery-drawer-front-overlay" });
            let c = egui::ComboBox::from_id_salt("door_front").selected_text(name(spec.inset)).show_ui(ui, |ui| {
                for (i, inset) in [false, true].into_iter().enumerate() {
                    let r = ui.selectable_value(&mut spec.inset, inset, name(inset));
                    super::tray_lib::spot(format!("joinery:door_front:{i}"), r.rect.center());
                }
            });
            super::tray_lib::spot("joinery:door_front", c.response.rect.center());
            let c = ui.checkbox(&mut spec.push, tr!("joinery-door-push"));
            super::tray_lib::spot("joinery:door_push", c.rect.center());
            c.on_hover_text(tr!("joinery-door-push-tip"));
            let b = ui.button(tr!("joinery-doors-add"));
            super::tray_lib::spot("joinery:doors", b.rect.center());
            if b.on_hover_text(tr!("joinery-doors-tip")).clicked() {
                self.doc.try_commit(&tr!("joinery-doors-undo"), move |m: &mut Model| m.add_doors(&l, &r, space, spec).map(|_| ()));
            }
        });
        self.joinery.door = Some(spec);
    }

    /// A fitting (a handle, a knob, legs, casters) for the one selected part: its kind and
    /// sizes, put on as objects mounted on it (`varstotas_kernel::fitting`).
    fn fitting_ui(&mut self, ui: &mut Ui) {
        let ids: Vec<InstanceId> = self.doc.selection.iter().filter_map(|e| if let EntityRef::Instance(id) = e { Some(*id) } else { None }).collect();
        if ids.len() != 1 || self.doc.selection.len() != 1 {
            return;
        }
        let mut part = self.doc.active_path.clone();
        part.push(ids[0]);
        if self.doc.model.def_at(&part).is_some_and(|d| self.doc.model.fitting_of(d).is_some()) {
            return;
        }
        let kind = self.joinery.fitting.as_ref().map_or(WoodJointKind::Handle, |f| f.1.kind);
        let mut j = match self.joinery.fitting.take() {
            Some((p, j)) if p == part => j,
            _ => match self.doc.model.fitting_joint(kind, part.clone()) {
                Ok(j) => j,
                Err(_) => return,
            },
        };
        let unit = self.doc.model.unit;
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(tr!("joinery-fitting")).strong()).on_hover_text(tr!("joinery-fitting-tip"));
            let mut k = j.kind;
            kind_combo(ui, "joinery_fitting_kind", &mut k, Some(true));
            if k != j.kind
                && let Ok(n) = self.doc.model.fitting_joint(k, part.clone())
            {
                j = n;
            }
            fitting_sizes(ui, &mut j, unit);
            let b = ui.button(tr!("joinery-fitting-add"));
            super::tray_lib::spot("joinery:fitting", b.rect.center());
            if b.on_hover_text(tr!("joinery-fitting-add-tip")).clicked() {
                let f = j.clone();
                self.doc.try_commit(&tr!("joinery-fitting-undo"), move |m: &mut Model| m.put_fitting_joint(&f).map(|_| ()));
            }
        });
        self.joinery.fitting = Some((part, j));
    }

    /// A back for the opening between the two selected parts (shown only for such a pair).
    fn back_ui(&mut self, ui: &mut Ui) {
        let ids: Vec<InstanceId> = self.doc.selection.iter().filter_map(|e| if let EntityRef::Instance(id) = e { Some(*id) } else { None }).collect();
        if ids.len() != 2 {
            return;
        }
        let path = |id| {
            let mut p = self.doc.active_path.clone();
            p.push(id);
            p
        };
        let (l, r) = (path(ids[0]), path(ids[1]));
        if self.doc.model.drawer_z(&l, &r).is_none() {
            return;
        }
        let mut spec = self.joinery.back.unwrap_or_default();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(tr!("joinery-back")).strong());
            let c = ui.checkbox(&mut spec.groove, tr!("joinery-back-groove"));
            super::tray_lib::spot("joinery:back_groove", c.rect.center());
            c.on_hover_text(tr!("joinery-back-groove-tip"));
            let b = ui.button(tr!("joinery-back-add"));
            super::tray_lib::spot("joinery:back", b.rect.center());
            if b.on_hover_text(tr!("joinery-back-tip")).clicked() {
                self.doc.try_commit(&tr!("joinery-back-undo"), move |m: &mut Model| m.add_back(&l, &r, spec).map(|_| ()));
            }
        });
        self.joinery.back = Some(spec);
    }

    /// A plinth frame under the selected parts (a carcass's).
    fn plinth_ui(&mut self, ui: &mut Ui) {
        let unit = self.doc.model.unit;
        let parts: Vec<Vec<InstanceId>> = self
            .doc
            .selection
            .iter()
            .filter_map(|e| if let EntityRef::Instance(id) = e { Some(*id) } else { None })
            .map(|id| {
                let mut p = self.doc.active_path.clone();
                p.push(id);
                p
            })
            .collect();
        if parts.is_empty() {
            return;
        }
        let mut spec = self.joinery.plinth.unwrap_or_default();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(tr!("joinery-plinth")).strong());
            len(ui, &tr!("joinery-height"), &mut spec.height, unit, 30.0, 1000.0);
            len(ui, &tr!("joinery-plinth-setback"), &mut spec.setback, unit, 0.0, 300.0);
            let b = ui.button(tr!("joinery-plinth-add"));
            super::tray_lib::spot("joinery:plinth", b.rect.center());
            if b.on_hover_text(tr!("joinery-plinth-tip")).clicked() {
                self.doc.try_commit(&tr!("joinery-plinth-undo"), move |m: &mut Model| m.add_plinth(&parts, spec).map(|_| ()));
            }
        });
        self.joinery.plinth = Some(spec);
    }

    fn joinery_list_ui(&mut self, ui: &mut Ui) {
        let listed = self.listed_joints();
        if listed.is_empty() {
            ui.label(RichText::new(tr!("joinery-none")).color(theme::text_dim()));
            return;
        }
        let mut remove = None;
        let mut swap = None;
        let mut kind = None;
        let mut select = None;
        let mut changed = None;
        let mut points = None;
        let warnings = self.doc.model.joint_warnings();
        let flagged = warnings.iter().filter(|w| !w.is_empty()).count();
        if flagged > 0 {
            ui.label(RichText::new(format!("⚠ {}", tr!("joinery-warnings", count = flagged.to_string()))).color(theme::warning())).on_hover_text(tr!("joinery-warnings-tip"));
        }
        // the rest of the window, as large as it is made
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for &i in &listed {
                let mut j = match &self.joinery.draft {
                    Some((k, d)) if *k == i => d.clone(),
                    _ => self.doc.model.placed_joints[i].clone(),
                };
                let before = j.clone();
                let picked = self.joinery.selected == Some(i);
                ui.push_id(("joint", i), |ui| {
                    ui.horizontal(|ui| {
                        let name = if j.kind.single() {
                            format!("{}. {}", i + 1, part_name(&self.doc.model, &j.a))
                        } else {
                            format!("{}. {} → {}", i + 1, part_name(&self.doc.model, &j.b), part_name(&self.doc.model, &j.a))
                        };
                        let text = if picked { RichText::new(name).strong().color(JOINT_PICKED) } else { RichText::new(name).strong() };
                        let r = ui.link(text);
                        super::tray_lib::spot(format!("joinery:pick:{i}"), r.rect.center());
                        if r.on_hover_text(tr!("joinery-select-tip")).clicked() {
                            select = Some((i, j.clone()));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let r = super::small_icon_button(ui, super::icons::Icon::Trash, &tr!("joinery-remove"));
                            super::tray_lib::spot(format!("joinery:remove:{i}"), r.rect.center());
                            if r.clicked() {
                                remove = Some(i);
                            }
                            if !j.kind.single() {
                                let r = ui.small_button("⇄").on_hover_text(tr!("joinery-swap-tip"));
                                super::tray_lib::spot(format!("joinery:swap:{i}"), r.rect.center());
                                if r.clicked() {
                                    swap = Some(i);
                                }
                            }
                            let mut k = j.kind;
                            kind_combo(ui, "kind", &mut k, Some(j.kind.single()));
                            if k != j.kind {
                                kind = Some((i, k));
                            }
                        });
                    });
                    let (na, nb) = (part_name(&self.doc.model, &j.a), part_name(&self.doc.model, &j.b));
                    let unit = self.doc.model.unit;
                    ui.horizontal_wrapped(|ui| match j.kind {
                        WoodJointKind::Dowels | WoodJointKind::Screws => {
                            mm(ui, &tr!("joinery-diameter"), &mut j.diameter, unit, 30.0);
                            if j.kind == WoodJointKind::Dowels {
                                mm(ui, &tr!("joinery-depth-in", part = na.clone()), &mut j.depth, unit, 200.0);
                            }
                            mm(ui, &tr!("joinery-depth-in", part = nb.clone()), &mut j.depth_b, unit, 200.0);
                        }
                        WoodJointKind::Housing => {
                            mm(ui, &tr!("joinery-depth"), &mut j.depth, unit, 200.0);
                            mm(ui, &tr!("joinery-clearance"), &mut j.clearance, unit, 5.0);
                        }
                        WoodJointKind::MortiseTenon => {
                            mm(ui, &tr!("joinery-thickness"), &mut j.thickness, unit, 200.0);
                            mm(ui, &tr!("joinery-shoulder"), &mut j.shoulder, unit, 500.0);
                            ui.add_enabled_ui(!j.through, |ui| mm(ui, &tr!("joinery-depth"), &mut j.depth, unit, 200.0));
                            ui.checkbox(&mut j.through, tr!("joinery-through"));
                            mm(ui, &tr!("joinery-clearance"), &mut j.clearance, unit, 5.0);
                            len(ui, &tr!("joinery-offset"), &mut j.offset, unit, -500.0, 500.0).on_hover_text(tr!("joinery-offset-tip"));
                        }
                        WoodJointKind::BoxJoint => {
                            ui.label(tr!("joinery-fingers"));
                            ui.add(egui::DragValue::new(&mut j.count).range(1..=64).speed(0.05));
                            mm(ui, &tr!("joinery-clearance"), &mut j.clearance, unit, 5.0);
                        }
                        WoodJointKind::Dovetail => {
                            ui.label(tr!("joinery-tails"));
                            ui.add(egui::DragValue::new(&mut j.count).range(1..=32).speed(0.05));
                            mm(ui, &tr!("joinery-pin"), &mut j.pin, unit, 200.0);
                            ui.label(tr!("joinery-angle"));
                            ui.add(egui::DragValue::new(&mut j.angle).range(0.5..=30.0).speed(0.1).max_decimals(1).suffix("°"));
                            mm(ui, &tr!("joinery-clearance"), &mut j.clearance, unit, 5.0);
                        }
                        WoodJointKind::PocketHole => {
                            mm(ui, &tr!("joinery-diameter"), &mut j.diameter, unit, 10.0);
                            mm(ui, &tr!("joinery-depth-in", part = na.clone()), &mut j.depth, unit, 200.0);
                            ui.checkbox(&mut j.flip, tr!("joinery-flip")).on_hover_text(tr!("joinery-flip-tip"));
                        }
                        WoodJointKind::Confirmat | WoodJointKind::Nails => {
                            mm(ui, &tr!("joinery-diameter"), &mut j.diameter, unit, 10.0);
                            mm(ui, &tr!("joinery-length"), &mut j.size, unit, 200.0);
                        }
                        WoodJointKind::CamLock => {
                            mm(ui, &tr!("joinery-diameter"), &mut j.diameter, unit, 40.0);
                            mm(ui, &tr!("joinery-to-cam"), &mut j.size, unit, 100.0);
                            ui.checkbox(&mut j.flip, tr!("joinery-flip-cam"));
                        }
                        WoodJointKind::ShelfPins => {
                            mm(ui, &tr!("joinery-diameter"), &mut j.diameter, unit, 10.0);
                            mm(ui, &tr!("joinery-depth"), &mut j.depth, unit, 50.0);
                            ui.label(tr!("joinery-rows")).on_hover_text(tr!("joinery-rows-tip"));
                            ui.add(egui::DragValue::new(&mut j.rows).range(0..=40).speed(0.05));
                            ui.checkbox(&mut j.flip, tr!("joinery-flip-pins"));
                        }
                        WoodJointKind::Hinge => {
                            mm(ui, &tr!("joinery-diameter"), &mut j.diameter, unit, 60.0);
                            mm(ui, &tr!("joinery-depth"), &mut j.depth, unit, 30.0);
                            ui.checkbox(&mut j.inset, tr!("joinery-inset")).on_hover_text(tr!("joinery-inset-tip"));
                            let mut a = j.hinge_angle();
                            ui.label(tr!("joinery-opens")).on_hover_text(tr!("joinery-opens-tip"));
                            if ui.add(egui::DragValue::new(&mut a).range(90.0..=180.0).speed(0.5).max_decimals(0).suffix("°")).changed() {
                                j.angle = a;
                            }
                        }
                        WoodJointKind::GasStrut => {
                            ui.label(tr!("joinery-force")).on_hover_text(tr!("joinery-force-tip"));
                            ui.add(egui::DragValue::new(&mut j.size).range(10.0..=1000.0).speed(1.0).max_decimals(0).suffix(" N"));
                        }
                        WoodJointKind::Slide => {
                            len(ui, &tr!("joinery-length"), &mut j.size, unit, 0.0, 2000.0).on_hover_text(tr!("joinery-length-auto-tip"));
                            len(ui, &tr!("joinery-offset"), &mut j.offset, unit, -500.0, 500.0).on_hover_text(tr!("joinery-offset-tip"));
                            ui.checkbox(&mut j.flip, tr!("joinery-flip-front"));
                            slide_combo(ui, "joint_slide", &mut j.slide);
                            ui.checkbox(&mut j.push, tr!("joinery-push")).on_hover_text(tr!("joinery-push-tip"));
                        }
                        // an old file's fitting that could not become one
                        WoodJointKind::Handle | WoodJointKind::Knob | WoodJointKind::Leg | WoodJointKind::Caster => fitting_sizes(ui, &mut j, unit),
                        WoodJointKind::TableTop => {
                            ui.checkbox(&mut j.flip, tr!("joinery-flip-side"));
                        }
                    });
                    let c = self.doc.model.joint_contacts.get(i).copied().flatten();
                    if j.kind.spaced() {
                        layout_ui(ui, &mut j, c, unit);
                        let r = ui.small_button(tr!("joinery-points")).on_hover_text(tr!("joinery-points-tip"));
                        super::tray_lib::spot(format!("joinery:points:{i}"), r.rect.center());
                        if r.clicked() {
                            points = Some(i);
                        }
                    }
                    if let Some((name, n)) = j.hardware(c.as_ref()) {
                        ui.label(RichText::new(format!("{n} × {name}")).small().color(theme::text_dim()));
                    }
                    for w in warnings.get(i).into_iter().flatten() {
                        ui.label(RichText::new(format!("⚠ {w}")).small().color(theme::warning()));
                    }
                });
                if j != before {
                    changed = Some((i, j));
                }
                ui.separator();
            }
        });
        if let Some(d) = changed {
            self.joinery.draft = Some(d);
        }
        let released = !ui.ctx().input(|i| i.pointer.any_down()) && !ui.ctx().memory(|m| m.focused().is_some());
        if released
            && let Some((i, j)) = self.joinery.draft.take()
            && self.doc.model.placed_joints.get(i).is_some_and(|o| *o != j)
        {
            self.doc.try_commit(&tr!("joinery-undo-edit"), move |m| m.set_wood_joint(i, j));
        }
        if let Some((i, j)) = select {
            self.joinery.selected = Some(i);
            self.select_joint_parts(&j);
        }
        if let Some(i) = points {
            self.joinery.selected = Some(i);
            self.install_tool(Box::new(crate::tools::joint_points::JointPointsTool::new(i)));
        }
        if let Some((i, k)) = kind {
            self.joinery.draft = None;
            self.doc.try_commit(&tr!("joinery-undo-edit"), move |m| m.set_wood_joint_kind(i, k));
        }
        if let Some(i) = swap {
            self.joinery.draft = None;
            let mut j = self.doc.model.placed_joints[i].clone();
            std::mem::swap(&mut j.a, &mut j.b);
            self.doc.try_commit(&tr!("joinery-undo-edit"), move |m| m.set_wood_joint(i, j));
        }
        if let Some(i) = remove {
            self.joinery.draft = None;
            self.joinery.selected = None;
            self.doc.commit(&tr!("joinery-undo-remove"), move |m| m.remove_wood_joint(i));
        }
    }
}
