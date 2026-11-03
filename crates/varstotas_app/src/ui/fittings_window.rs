//! Window → Fittings (track W, `varstotas_kernel::fitting`): a handle, knob, legs or casters
//! put on the selected part, a plinth under the selected parts, and every fitting on the model by part — its sizes and shape made
//! again, its holes, where each copy is on its face; selected, deleted, its part's joints opened;
//! and the fittings that are joints between two parts (hinges, slides, gas struts, shelf pins,
//! confirmats…) with their sizes as in Joinery.

use super::joinery::{fitting_sizes, kind_combo, len};
use crate::app::VarstotasApp;
use crate::ui::theme;
use egui::{Context, RichText, Ui, vec2};
use varstotas_i18n::tr;
use varstotas_kernel::carcass::PlinthSpec;
use varstotas_kernel::fitting::{Builtin, Style};
use varstotas_kernel::joinery::{WoodJoint, WoodJointKind};
use varstotas_kernel::{ContextPath, DefId, EntityRef, InstanceId, LengthUnit, Model};

#[derive(Default)]
pub struct FittingsState {
    pub open: bool,
    /// A new fitting: the part it is for and its kind and sizes (suiting that part until changed).
    fitting: Option<(Vec<InstanceId>, WoodJoint)>,
    /// The shape a new handle, knob or leg is put on in.
    style: Style,
    /// A fitting of the user's put on instead of a built-in one.
    own: Option<Own>,
    /// A new plinth's settings.
    plinth: Option<PlinthSpec>,
    /// A joint between parts being edited (committed when the pointer is released).
    joint_draft: Option<(usize, WoodJoint)>,
}

/// A fitting of the user's: the model's own (not built in) or a library file's.
#[derive(Clone, Debug, PartialEq)]
enum Own {
    Def(DefId),
    File(std::path::PathBuf),
}

/// The fittings of one definition on one part (or not mounted).
struct Row {
    def: DefId,
    part: Option<ContextPath>,
    copies: Vec<ContextPath>,
}

/// The fittings on the model, by definition and part, in the order they are met; a
/// component's are listed once.
fn rows(m: &Model) -> Vec<Row> {
    fn walk(m: &Model, d: DefId, path: &mut ContextPath, seen: &mut Vec<DefId>, out: &mut Vec<Row>) {
        let Some(def) = m.defs.get(d).filter(|_| path.len() < 64) else { return };
        for (id, inst) in &def.mesh.instances {
            path.push(id);
            if m.fitting_of(inst.def).is_some() {
                let part = inst.mount.as_ref().map(|mt| path[..path.len() - 1].iter().chain(&mt.on).copied().collect::<ContextPath>());
                match out.iter_mut().find(|r| r.def == inst.def && r.part == part) {
                    Some(r) => r.copies.push(path.clone()),
                    None => out.push(Row { def: inst.def, part, copies: vec![path.clone()] }),
                }
            } else if !seen.contains(&inst.def) {
                seen.push(inst.def);
                walk(m, inst.def, path, seen, out);
            }
            path.pop();
        }
    }
    let mut out = Vec::new();
    walk(m, m.root, &mut Vec::new(), &mut Vec::new(), &mut out);
    out
}

/// The name of a handle's, knob's or leg's shape.
fn style_name(kind: WoodJointKind, style: Style) -> String {
    tr!(match (style, kind) {
        (Style::Round, _) => "fitting-style-round",
        (Style::Square, _) => "fitting-style-square",
        (Style::Shaped, WoodJointKind::Handle) => "fitting-style-bow",
        (Style::Shaped, WoodJointKind::Knob) => "fitting-style-mushroom",
        (Style::Shaped, _) => "fitting-style-hairpin",
    })
}

/// The shape of a handle, knob or leg; `id` names the combo and its spots.
fn style_combo(ui: &mut Ui, id: &str, kind: WoodJointKind, style: &mut Style) {
    ui.label(tr!("joinery-fitting-style"));
    let r = egui::ComboBox::from_id_salt(id).selected_text(style_name(kind, *style)).show_ui(ui, |ui| {
        for (i, s) in Style::ALL.into_iter().enumerate() {
            let r = ui.selectable_value(style, s, style_name(kind, s));
            super::tray_lib::spot(format!("{id}:{i}"), r.rect.center());
        }
    });
    super::tray_lib::spot(id, r.response.rect.center());
    r.response.on_hover_text(tr!("joinery-fitting-style-tip"));
}

/// A built-in fitting's sizes and shape; `id` names its fields' spots.
fn builtin_ui(ui: &mut Ui, id: &str, b: &mut Builtin, unit: LengthUnit) {
    let spot = |r: &egui::Response, k: usize| super::tray_lib::spot(format!("{id}:size:{k}"), r.rect.center());
    let kind = match b {
        Builtin::Handle { centres, style } => {
            spot(&len(ui, &tr!("joinery-centres"), centres, unit, 16.0, 1000.0), 0);
            Some((WoodJointKind::Handle, style))
        }
        Builtin::Knob { diameter, style } => {
            spot(&len(ui, &tr!("joinery-diameter"), diameter, unit, 0.5, 100.0), 0);
            Some((WoodJointKind::Knob, style))
        }
        Builtin::Leg { height, diameter, style } => {
            spot(&len(ui, &tr!("joinery-height"), height, unit, 10.5, 2000.0), 0);
            spot(&len(ui, &tr!("joinery-diameter"), diameter, unit, 0.5, 300.0), 1);
            Some((WoodJointKind::Leg, style))
        }
        Builtin::Caster { wheel, height } => {
            spot(&len(ui, &tr!("joinery-diameter"), wheel, unit, 0.5, 300.0), 0);
            spot(&len(ui, &tr!("joinery-height"), height, unit, 0.5, 2000.0), 1);
            None
        }
    };
    if let Some((kind, style)) = kind {
        style_combo(ui, &format!("{id}:style"), kind, style);
    }
}

impl VarstotasApp {
    pub(crate) fn fittings_window(&mut self, ctx: &Context) {
        if !self.fittings.open {
            return;
        }
        // the library's fittings, the Components card shown or not
        if self.library.scanned != self.library_dir() {
            self.refresh_library();
        }
        let pos = self.viewport.view_rect.min + vec2(80.0, 80.0);
        let mut open = true;
        let win = self.placed(egui::Window::new(tr!("fittings-title")).id(egui::Id::new("fittings_window")).open(&mut open).collapsible(false).resizable(true).default_size([520.0, 380.0]).min_width(440.0), pos);
        win.show(ctx, |ui| {
            self.fitting_put_ui(ui);
            self.plinth_ui(ui);
            ui.separator();
            egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
                self.fittings_list_ui(ui);
                ui.separator();
                self.fitting_joints_ui(ui);
            });
        });
        if !open {
            self.fittings.open = false;
        }
    }

    /// A fitting (a handle, a knob, legs, casters) for the one selected part: its kind, sizes
    /// and shape, put on as objects mounted on it.
    fn fitting_put_ui(&mut self, ui: &mut Ui) {
        let ids: Vec<InstanceId> = self.doc.selection.iter().filter_map(|e| if let EntityRef::Instance(id) = e { Some(*id) } else { None }).collect();
        let mut part = self.doc.active_path.clone();
        part.extend(ids.first());
        let one = ids.len() == 1 && self.doc.selection.len() == 1 && self.doc.model.def_at(&part).is_some_and(|d| self.doc.model.fitting_of(d).is_none());
        let kind = self.fittings.fitting.as_ref().map_or(WoodJointKind::Handle, |f| f.1.kind);
        let j = match self.fittings.fitting.take() {
            Some((p, j)) if p == part => Some(j),
            _ if one => self.doc.model.fitting_joint(kind, part.clone()).ok(),
            _ => None,
        };
        let Some(mut j) = j.filter(|_| one) else {
            ui.horizontal(|ui| {
                ui.label(RichText::new(tr!("joinery-fitting")).strong()).on_hover_text(tr!("joinery-fitting-tip"));
                ui.label(RichText::new(tr!("fittings-pick-part")).color(theme::text_dim()));
            });
            return;
        };
        let unit = self.doc.model.unit;
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(tr!("joinery-fitting")).strong()).on_hover_text(tr!("joinery-fitting-tip"));
            ui.label(RichText::new(super::joinery::part_name(&self.doc.model, &part)).color(theme::text_dim()));
            let mut k = j.kind;
            kind_combo(ui, "fittings:kind", &mut k, Some(true));
            if k != j.kind
                && let Ok(n) = self.doc.model.fitting_joint(k, part.clone())
            {
                j = n;
            }
            fitting_sizes(ui, &mut j, unit);
            self.own_combo(ui);
            if j.kind != WoodJointKind::Caster && self.fittings.own.is_none() {
                style_combo(ui, "fittings:style", j.kind, &mut self.fittings.style);
            }
            let style = if j.kind == WoodJointKind::Caster { Style::Round } else { self.fittings.style };
            let b = ui.button(RichText::new(tr!("joinery-fitting-add")).strong());
            super::tray_lib::spot("fittings:put", b.rect.center());
            if b.on_hover_text(tr!("joinery-fitting-add-tip")).clicked() {
                let f = j.clone();
                let label = tr!("joinery-fitting-undo");
                match self.fittings.own.clone() {
                    None => {
                        self.doc.try_commit(&label, move |m: &mut Model| m.put_fitting_joint_styled(&f, style).map(|_| ()));
                    }
                    Some(Own::Def(d)) => {
                        self.doc.try_commit(&label, move |m: &mut Model| m.put_fitting_joint_def(&f, d).map(|_| ()));
                    }
                    Some(Own::File(path)) => {
                        if let Some(clip) = self.library.items.iter().find(|x| x.path == path).map(|x| x.clip.clone()) {
                            let uid = self.doc.uid;
                            self.doc.try_commit(&label, move |m: &mut Model| clip.fitting_into(m, uid).and_then(|d| m.put_fitting_joint_def(&f, d)).map(|_| ()));
                        }
                    }
                }
            }
        });
        self.fittings.fitting = Some((part, j));
    }

    /// The fittings of the user's to put on instead of a built-in one: the model's own (not
    /// built in) and the library's, as (which, name).
    fn own_fittings(&self) -> Vec<(Own, String)> {
        let m = &self.doc.model;
        let mut out: Vec<(Own, String)> = m.defs.iter().filter(|(_, d)| d.fitting.as_ref().is_some_and(|f| f.builtin.is_none())).map(|(id, _)| (Own::Def(id), m.fitting_item(id).unwrap_or_default())).collect();
        out.sort_by(|a, b| a.1.cmp(&b.1));
        let mut lib: Vec<(Own, String)> = self.library.items.iter().filter(|x| x.clip.fitting_item().is_some()).map(|x| (Own::File(x.path.clone()), x.name.clone())).collect();
        lib.sort_by(|a, b| a.1.cmp(&b.1));
        out.extend(lib);
        out
    }

    /// Built in or one of the user's fittings ([`Self::own_fittings`]): the Model combo, spot
    /// `fittings:model` and its items `fittings:model:<n>` (0 built in).
    fn own_combo(&mut self, ui: &mut Ui) {
        let list = self.own_fittings();
        if self.fittings.own.as_ref().is_some_and(|o| !list.iter().any(|x| x.0 == *o)) {
            self.fittings.own = None;
        }
        if list.is_empty() {
            return;
        }
        let name = |o: &Own, n: &str| match o {
            Own::Def(_) => tr!("fittings-model-own", name = n),
            Own::File(_) => tr!("fittings-model-library", name = n),
        };
        let shown = self.fittings.own.as_ref().and_then(|o| list.iter().find(|x| x.0 == *o)).map_or_else(|| tr!("fittings-model-builtin"), |x| name(&x.0, &x.1));
        ui.label(tr!("fittings-model"));
        let own = &mut self.fittings.own;
        let r = egui::ComboBox::from_id_salt("fittings:model").selected_text(shown).show_ui(ui, |ui| {
            let r = ui.selectable_value(own, None, tr!("fittings-model-builtin"));
            super::tray_lib::spot("fittings:model:0", r.rect.center());
            for (i, (o, n)) in list.iter().enumerate() {
                let r = ui.selectable_value(own, Some(o.clone()), name(o, n));
                super::tray_lib::spot(format!("fittings:model:{}", i + 1), r.rect.center());
            }
        });
        super::tray_lib::spot("fittings:model", r.response.rect.center());
        r.response.on_hover_text(tr!("fittings-model-tip"));
    }

    /// The fitting put on as (built in or not, its name), for the debug dump.
    pub(crate) fn fitting_model_dump(&self) -> String {
        let list = self.own_fittings();
        self.fittings.own.as_ref().and_then(|o| list.into_iter().find(|x| x.0 == *o)).map_or_else(|| "builtin".into(), |x| x.1) // i18n-ignore
    }

    /// A plinth frame under the selected parts (a carcass's).
    pub(super) fn plinth_ui(&mut self, ui: &mut Ui) {
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
        let mut spec = self.fittings.plinth.unwrap_or_default();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(tr!("joinery-plinth")).strong());
            len(ui, &tr!("joinery-height"), &mut spec.height, unit, 30.0, 1000.0);
            len(ui, &tr!("joinery-plinth-setback"), &mut spec.setback, unit, 0.0, 300.0);
            let b = ui.button(tr!("joinery-plinth-add"));
            super::tray_lib::spot("fittings:plinth", b.rect.center());
            if b.on_hover_text(tr!("joinery-plinth-tip")).clicked() {
                self.doc.try_commit(&tr!("joinery-plinth-undo"), move |m: &mut Model| m.add_plinth(&parts, spec).map(|_| ()));
            }
        });
        self.fittings.plinth = Some(spec);
    }

    /// Every fitting on the model by part: a line each (how many, what, on which part; select,
    /// its part's joints, delete) opening to its sizes, holes and copies.
    fn fittings_list_ui(&mut self, ui: &mut Ui) {
        let rows = rows(&self.doc.model);
        ui.label(RichText::new(tr!("fittings-on-model", count = rows.iter().map(|r| r.copies.len()).sum::<usize>())).strong());
        if rows.is_empty() {
            ui.label(RichText::new(tr!("fittings-none")).color(theme::text_dim()));
            return;
        }
        let unit = self.doc.model.unit;
        for (i, row) in rows.iter().enumerate() {
            let m = &self.doc.model;
            let item = m.fitting_item(row.def).unwrap_or_default();
            let on = match &row.part {
                Some(p) => tr!("fittings-on", part = super::joinery::part_name(m, p)),
                None => tr!("fitting-not-mounted"),
            };
            let id = ui.make_persistent_id(("fittings_row", row.def, &row.part));
            let state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
            let mut act = None;
            let (toggle, _, _) = state
                .show_header(ui, |ui| {
                    ui.label(RichText::new(format!("{} × {item}", row.copies.len())).strong());
                    ui.label(RichText::new(on).color(theme::text_dim()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let r = super::small_icon_button(ui, super::icons::Icon::Trash, &tr!("fittings-delete"));
                        super::tray_lib::spot(format!("fittings:row:{i}:delete"), r.rect.center());
                        if r.clicked() {
                            act = Some(0);
                        }
                        if row.part.is_some() {
                            let r = super::small_icon_button(ui, super::icons::Icon::Joinery, &tr!("fittings-joints"));
                            super::tray_lib::spot(format!("fittings:row:{i}:joints"), r.rect.center());
                            if r.clicked() {
                                act = Some(1);
                            }
                        }
                        let r = super::small_icon_button(ui, super::icons::Icon::Select, &tr!("fittings-select"));
                        super::tray_lib::spot(format!("fittings:row:{i}:select"), r.rect.center());
                        if r.clicked() {
                            act = Some(2);
                        }
                        let r = super::small_icon_button(ui, super::icons::Icon::FileSave, &tr!("fittings-save"));
                        super::tray_lib::spot(format!("fittings:row:{i}:save"), r.rect.center());
                        if r.clicked() {
                            act = Some(3);
                        }
                    });
                })
                .body(|ui| self.fitting_row_ui(ui, i, row, unit));
            super::tray_lib::spot(format!("fittings:row:{i}"), toggle.rect.center());
            match act {
                Some(0) => {
                    let copies = row.copies.clone();
                    self.doc.set_selection(Vec::new());
                    self.doc.commit(&tr!("fittings-delete"), |m| {
                        for p in &copies {
                            if let Some((&id, prefix)) = p.split_last()
                                && let Some(mesh) = m.mesh_at_mut(prefix)
                            {
                                mesh.erase(&[EntityRef::Instance(id)]);
                            }
                        }
                    });
                    return;
                }
                Some(1) => {
                    if let Some(part) = &row.part {
                        self.select_paths(std::slice::from_ref(part));
                        self.joinery.open = true;
                    }
                }
                Some(2) => self.select_paths(&row.copies),
                Some(3) => self.save_definition_to_library(row.def),
                _ => {}
            }
        }
    }

    /// Select the instances at `paths` (from the root) that are in the same container as the
    /// first, opening it.
    fn select_paths(&mut self, paths: &[ContextPath]) {
        let Some((_, prefix)) = paths.first().and_then(|p| p.split_last()) else { return };
        let prefix = prefix.to_vec();
        let sel = paths.iter().filter_map(|p| p.split_last()).filter(|(_, x)| *x == prefix.as_slice()).map(|(id, _)| EntityRef::Instance(*id)).collect();
        self.doc.set_active_path(prefix);
        self.doc.set_selection(sel);
    }

    /// A line opened: a built-in fitting's sizes and shape (made again on Apply), its holes,
    /// each copy's place on its face.
    fn fitting_row_ui(&mut self, ui: &mut Ui, i: usize, row: &Row, unit: LengthUnit) {
        let Some(f) = self.doc.model.fitting_of(row.def).cloned() else { return };
        match f.builtin {
            Some(b) => {
                let key = egui::Id::new(("fittings_size", row.def, &row.part));
                let mut draft: Builtin = ui.data(|x| x.get_temp(key)).unwrap_or(b);
                ui.horizontal_wrapped(|ui| {
                    builtin_ui(ui, &format!("fittings:row:{i}"), &mut draft, unit);
                    let r = ui.add_enabled(draft != b && draft.is_valid(), egui::Button::new(tr!("fittings-apply")));
                    super::tray_lib::spot(format!("fittings:row:{i}:apply"), r.rect.center());
                    if r.on_hover_text(tr!("fittings-apply-tip")).clicked() {
                        let copies = row.copies.clone();
                        self.doc.try_commit(&tr!("fittings-apply-undo"), move |m: &mut Model| m.refit(&copies, draft));
                    }
                });
                if draft != b {
                    ui.data_mut(|x| x.insert_temp(key, draft));
                } else {
                    ui.data_mut(|x| x.remove::<Builtin>(key));
                }
            }
            None => {
                ui.label(RichText::new(tr!("fittings-own")).small().color(theme::text_dim()));
            }
        }
        if let Some(holes) = super::fittings::holes_ui(ui, row.def, &f.holes, unit, &format!("fittings:row:{i}:")) {
            let d = row.def;
            self.doc.commit(&tr!("fitting-undo-edit"), |m| {
                if let Some(def) = m.defs.get_mut(d) {
                    def.fitting = Some(varstotas_kernel::fitting::Fitting { holes, ..f.clone() });
                }
            });
        }
        if row.part.is_none() {
            return;
        }
        let mut set = None;
        egui::Grid::new(("fittings_copies", row.def, &row.part)).num_columns(6).spacing([6.0, 4.0]).show(ui, |ui| {
            ui.label("");
            ui.label(RichText::new(tr!("fitting-offset")).small().color(theme::text_dim())).on_hover_text(tr!("fitting-offset-tip"));
            ui.label("");
            ui.label(RichText::new(tr!("fitting-lift")).small().color(theme::text_dim()));
            ui.label(RichText::new(tr!("fitting-turn")).small().color(theme::text_dim()));
            ui.end_row();
            for (k, p) in row.copies.iter().enumerate() {
                let Some((&id, prefix)) = p.split_last() else { continue };
                let Some(mt) = self.doc.model.mesh_at(prefix).and_then(|x| x.instances.get(id)).and_then(|x| x.mount.clone()) else { continue };
                let mut m = mt.clone();
                ui.label(RichText::new(format!("{}", k + 1)).color(theme::text_dim()));
                for (n, o) in m.offset.iter_mut().enumerate() {
                    let r = ui.add(super::len_drag(o, unit).speed(0.5));
                    super::tray_lib::spot(format!("fittings:row:{i}:copy:{k}:offset:{n}"), r.rect.center());
                }
                ui.add(super::len_drag(&mut m.lift, unit).range(-500.0..=500.0).speed(0.2));
                ui.add(egui::DragValue::new(&mut m.turn).range(-360.0..=360.0).speed(1.0).suffix("°"));
                ui.end_row();
                if m != mt {
                    m.seen = None;
                    set = Some((p.clone(), m));
                }
            }
        });
        if let Some((p, mt)) = set {
            self.doc.commit(&tr!("fitting-undo-edit"), |m| {
                if let Some((&id, prefix)) = p.split_last()
                    && let Some(i) = m.mesh_at_mut(prefix).and_then(|x| x.instances.get_mut(id))
                {
                    i.mount = Some(mt);
                }
            });
        }
    }

    /// The fittings that are joints between two parts (confirmats, cam locks, shelf pins, hinges,
    /// gas struts, slides, table-top fasteners, nails), once each, as the Joinery window's.
    fn fitting_joint_list(&self) -> Vec<usize> {
        let m = &self.doc.model;
        let mut seen = std::collections::HashSet::new();
        m.joint_places
            .iter()
            .enumerate()
            .filter(|(i, p)| seen.insert((p.def, p.index)) && m.placed_joints.get(*i).is_some_and(|j| j.kind.fitting() && !j.kind.single()))
            .map(|(i, _)| i)
            .collect()
    }

    /// The fittings between parts: a line each (what and how many, on which parts; select the
    /// parts, open in Joinery, delete) opening to its sizes and layout, as in Joinery.
    fn fitting_joints_ui(&mut self, ui: &mut Ui) {
        let list = self.fitting_joint_list();
        ui.label(RichText::new(tr!("fittings-between", count = list.len())).strong()).on_hover_text(tr!("fittings-between-tip"));
        if list.is_empty() {
            ui.label(RichText::new(tr!("fittings-between-none")).color(theme::text_dim()));
            return;
        }
        let unit = self.doc.model.unit;
        let warnings = self.doc.model.joint_warnings();
        let (mut act, mut changed) = (None, None);
        for (row, &i) in list.iter().enumerate() {
            let m = &self.doc.model;
            let mut j = match &self.fittings.joint_draft {
                Some((k, d)) if *k == i => d.clone(),
                _ => m.placed_joints[i].clone(),
            };
            let before = j.clone();
            let c = m.joint_contacts.get(i).copied().flatten();
            let what = j.hardware(c.as_ref()).map_or_else(|| super::joinery::kind_name(j.kind), |(n, k)| format!("{k} × {n}"));
            let (na, nb) = (super::joinery::part_name(m, &j.a), super::joinery::part_name(m, &j.b));
            let warn = warnings.get(i).cloned().unwrap_or_default();
            let id = ui.make_persistent_id(("fittings_joint", i));
            let state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
            let (toggle, _, _) = state
                .show_header(ui, |ui| {
                    ui.label(RichText::new(what).strong());
                    ui.label(RichText::new(tr!("fittings-between-parts", a = nb.clone(), b = na.clone())).color(theme::text_dim()));
                    if !warn.is_empty() {
                        ui.label(RichText::new("⚠").color(theme::warning())).on_hover_text(warn.join("\n"));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let r = super::small_icon_button(ui, super::icons::Icon::Trash, &tr!("joinery-remove"));
                        super::tray_lib::spot(format!("fittings:joint:{row}:delete"), r.rect.center());
                        if r.clicked() {
                            act = Some((0, i));
                        }
                        let r = super::small_icon_button(ui, super::icons::Icon::Joinery, &tr!("fittings-joint-open"));
                        super::tray_lib::spot(format!("fittings:joint:{row}:joinery"), r.rect.center());
                        if r.clicked() {
                            act = Some((1, i));
                        }
                        let r = super::small_icon_button(ui, super::icons::Icon::Select, &tr!("fittings-joint-select"));
                        super::tray_lib::spot(format!("fittings:joint:{row}:select"), r.rect.center());
                        if r.clicked() {
                            act = Some((2, i));
                        }
                    });
                })
                .body(|ui| {
                    ui.push_id(("fittings_joint_body", i), |ui| {
                        super::joinery::joint_params_ui(ui, &mut j, c, unit, &na, &nb, &format!("fittings:joint:{row}"));
                        for w in &warn {
                            ui.label(RichText::new(format!("⚠ {w}")).small().color(theme::warning()));
                        }
                    });
                });
            super::tray_lib::spot(format!("fittings:joint:{row}"), toggle.rect.center());
            if j != before {
                changed = Some((i, j));
            }
        }
        if let Some(d) = changed {
            self.fittings.joint_draft = Some(d);
        }
        let released = !ui.ctx().input(|i| i.pointer.any_down()) && !ui.ctx().memory(|m| m.focused().is_some());
        if released
            && let Some((i, j)) = self.fittings.joint_draft.take()
            && self.doc.model.placed_joints.get(i).is_some_and(|o| *o != j)
        {
            self.doc.try_commit(&tr!("joinery-undo-edit"), move |m| m.set_wood_joint(i, j));
        }
        match act {
            Some((0, i)) => {
                self.fittings.joint_draft = None;
                self.joinery.selected = None;
                self.doc.commit(&tr!("joinery-undo-remove"), move |m| m.remove_wood_joint(i));
            }
            Some((k, i)) => {
                let j = self.doc.model.placed_joints[i].clone();
                self.select_joint_parts(&j);
                if k == 1 {
                    self.joinery.selected = Some(i);
                    self.joinery.open = true;
                }
            }
            None => {}
        }
    }

    /// The fittings between parts as (what, parts), for the debug dump.
    pub(crate) fn fitting_joints_dump(&self) -> Vec<(String, String)> {
        let m = &self.doc.model;
        self.fitting_joint_list()
            .into_iter()
            .map(|i| {
                let j = &m.placed_joints[i];
                let c = m.joint_contacts.get(i).copied().flatten();
                let what = j.hardware(c.as_ref()).map_or_else(|| super::joinery::kind_name(j.kind), |(n, k)| format!("{k} × {n}"));
                (what, format!("{} → {}", super::joinery::part_name(m, &j.b), super::joinery::part_name(m, &j.a)))
            })
            .collect()
    }

    /// The fittings window's lines as (count, item, part), for the debug dump.
    pub(crate) fn fittings_rows_dump(&self) -> Vec<(usize, String, String)> {
        let m = &self.doc.model;
        rows(m).iter().map(|r| (r.copies.len(), m.fitting_item(r.def).unwrap_or_default(), r.part.as_ref().map(|p| super::joinery::part_name(m, p)).unwrap_or_default())).collect()
    }
}

