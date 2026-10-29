//! Window → Joinery (track W): joints between parts that touch (`varstotas_kernel::joinery`).
//! Select two groups or components, pick a kind and add a joint; the list below edits its
//! sizes, swaps its parts or deletes it. A click on a joint's name selects its two parts.

use crate::app::VarstotasApp;
use crate::ui::theme;
use egui::{Context, RichText, Ui, vec2};
use varstotas_i18n::tr;
use varstotas_kernel::joinery::{WoodJoint, WoodJointKind};
use varstotas_kernel::{EntityRef, InstanceId, Model};

#[derive(Default)]
pub struct JoineryState {
    pub open: bool,
    /// The kind a new joint gets.
    pub kind: WoodJointKind,
    /// A joint being dragged (committed when the pointer is released).
    draft: Option<(usize, WoodJoint)>,
}

pub(crate) fn kind_name(k: WoodJointKind) -> String {
    varstotas_i18n::tr(match k {
        WoodJointKind::Dowels => "joinery-kind-dowels",
        WoodJointKind::Screws => "joinery-kind-screws",
        WoodJointKind::Housing => "joinery-kind-housing",
        WoodJointKind::MortiseTenon => "joinery-kind-mortise-tenon",
        WoodJointKind::BoxJoint => "joinery-kind-box",
        WoodJointKind::Dovetail => "joinery-kind-dovetail",
        WoodJointKind::PocketHole => "joinery-kind-pocket",
    })
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

fn mm(ui: &mut Ui, label: &str, v: &mut f64, max: f64) -> bool {
    ui.label(label);
    ui.add(egui::DragValue::new(v).range(0.5..=max).speed(0.2).max_decimals(1).suffix(" mm")).changed()
}

impl VarstotasApp {
    /// The two selected groups or components, as paths from the root.
    fn joint_selection(&self) -> Option<(Vec<InstanceId>, Vec<InstanceId>)> {
        let ids: Vec<InstanceId> = self.doc.selection.iter().filter_map(|e| if let EntityRef::Instance(id) = e { Some(*id) } else { None }).collect();
        if ids.len() != 2 || self.doc.selection.len() != 2 {
            return None;
        }
        let path = |id| {
            let mut p = self.doc.active_path.clone();
            p.push(id);
            p
        };
        Some((path(ids[0]), path(ids[1])))
    }

    /// Join the two selected parts with a joint of the chosen kind.
    pub(crate) fn add_joint(&mut self) {
        let Some((p, q)) = self.joint_selection() else {
            self.flash_error(tr!("joint-err-two-parts"));
            return;
        };
        let kind = self.joinery.kind;
        self.doc.try_commit(&tr!("joinery-undo-add"), move |m| m.add_wood_joint(kind, p, q));
    }

    /// The joints for the debug dump: (kind, b, a, hardware).
    pub(crate) fn joints_dump(&self) -> Vec<(String, String, String, String)> {
        let m = &self.doc.model;
        m.joints.iter().map(|j| (format!("{:?}", j.kind), part_name(m, &j.b), part_name(m, &j.a), j.hardware(None).map(|(n, c)| format!("{c} × {n}")).unwrap_or_default())).collect()
    }

    fn select_joint_parts(&mut self, j: &WoodJoint) {
        let (Some((&a, pa)), Some((&b, pb))) = (j.a.split_last(), j.b.split_last()) else { return };
        self.doc.set_active_path(pa.to_vec());
        self.doc.set_selection(if pa == pb { vec![EntityRef::Instance(a), EntityRef::Instance(b)] } else { vec![EntityRef::Instance(a)] });
    }

    pub(crate) fn joinery_window(&mut self, ctx: &Context) {
        if !self.joinery.open {
            return;
        }
        let pos = self.viewport.view_rect.min + vec2(60.0, 60.0);
        let mut open = true;
        let win = self.placed(egui::Window::new(tr!("joinery-title")).id(egui::Id::new("joinery_window")).open(&mut open).collapsible(false).resizable(false), pos);
        win.show(ctx, |ui| {
            ui.set_width(460.0);
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
            egui::ComboBox::from_id_salt("joinery_new_kind").selected_text(kind_name(self.joinery.kind)).show_ui(ui, |ui| {
                for k in WoodJointKind::ALL {
                    ui.selectable_value(&mut self.joinery.kind, k, kind_name(k));
                }
            });
            let r = ui.add_enabled(sel.is_some(), egui::Button::new(RichText::new(tr!("joinery-add")).strong()));
            super::tray_lib::spot("joinery:add", r.rect.center());
            if r.on_hover_text(tr!("joinery-add-tip")).on_disabled_hover_text(tr!("joint-err-two-parts")).clicked() {
                self.add_joint();
            }
        });
        let hint = match &sel {
            Some((p, q)) => tr!("joinery-selected", a = part_name(&self.doc.model, p), b = part_name(&self.doc.model, q)),
            None => tr!("joint-err-two-parts"),
        };
        ui.label(RichText::new(hint).small().color(theme::text_dim()));
    }

    fn joinery_list_ui(&mut self, ui: &mut Ui) {
        if self.doc.model.joints.is_empty() {
            ui.label(RichText::new(tr!("joinery-none")).color(theme::text_dim()));
            return;
        }
        let mut remove = None;
        let mut swap = None;
        let mut kind = None;
        let mut select = None;
        let mut changed = None;
        egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
            for i in 0..self.doc.model.joints.len() {
                let mut j = match &self.joinery.draft {
                    Some((k, d)) if *k == i => d.clone(),
                    _ => self.doc.model.joints[i].clone(),
                };
                let before = j.clone();
                ui.push_id(("joint", i), |ui| {
                    ui.horizontal(|ui| {
                        let name = format!("{}. {} → {}", i + 1, part_name(&self.doc.model, &j.b), part_name(&self.doc.model, &j.a));
                        if ui.link(RichText::new(name).strong()).on_hover_text(tr!("joinery-select-tip")).clicked() {
                            select = Some(j.clone());
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let r = super::small_icon_button(ui, super::icons::Icon::Trash, &tr!("joinery-remove"));
                            super::tray_lib::spot(format!("joinery:remove:{i}"), r.rect.center());
                            if r.clicked() {
                                remove = Some(i);
                            }
                            let r = ui.small_button("⇄").on_hover_text(tr!("joinery-swap-tip"));
                            super::tray_lib::spot(format!("joinery:swap:{i}"), r.rect.center());
                            if r.clicked() {
                                swap = Some(i);
                            }
                            let mut k = j.kind;
                            egui::ComboBox::from_id_salt("kind").selected_text(kind_name(k)).show_ui(ui, |ui| {
                                for x in WoodJointKind::ALL {
                                    ui.selectable_value(&mut k, x, kind_name(x));
                                }
                            });
                            if k != j.kind {
                                kind = Some((i, k));
                            }
                        });
                    });
                    let (na, nb) = (part_name(&self.doc.model, &j.a), part_name(&self.doc.model, &j.b));
                    ui.horizontal_wrapped(|ui| match j.kind {
                        WoodJointKind::Dowels | WoodJointKind::Screws => {
                            ui.label(tr!("joinery-count"));
                            ui.add(egui::DragValue::new(&mut j.count).range(1..=32).speed(0.05));
                            mm(ui, &tr!("joinery-diameter"), &mut j.diameter, 30.0);
                            if j.kind == WoodJointKind::Dowels {
                                mm(ui, &tr!("joinery-depth-in", part = na.clone()), &mut j.depth, 200.0);
                            }
                            mm(ui, &tr!("joinery-depth-in", part = nb.clone()), &mut j.depth_b, 200.0);
                        }
                        WoodJointKind::Housing => {
                            mm(ui, &tr!("joinery-depth"), &mut j.depth, 200.0);
                            mm(ui, &tr!("joinery-clearance"), &mut j.clearance, 5.0);
                        }
                        WoodJointKind::MortiseTenon => {
                            mm(ui, &tr!("joinery-thickness"), &mut j.thickness, 200.0);
                            mm(ui, &tr!("joinery-shoulder"), &mut j.shoulder, 500.0);
                            ui.add_enabled_ui(!j.through, |ui| mm(ui, &tr!("joinery-depth"), &mut j.depth, 200.0));
                            ui.checkbox(&mut j.through, tr!("joinery-through"));
                            mm(ui, &tr!("joinery-clearance"), &mut j.clearance, 5.0);
                        }
                        WoodJointKind::BoxJoint => {
                            ui.label(tr!("joinery-fingers"));
                            ui.add(egui::DragValue::new(&mut j.count).range(1..=64).speed(0.05));
                            mm(ui, &tr!("joinery-clearance"), &mut j.clearance, 5.0);
                        }
                        WoodJointKind::Dovetail => {
                            ui.label(tr!("joinery-tails"));
                            ui.add(egui::DragValue::new(&mut j.count).range(1..=32).speed(0.05));
                            mm(ui, &tr!("joinery-pin"), &mut j.pin, 200.0);
                            ui.label(tr!("joinery-angle"));
                            ui.add(egui::DragValue::new(&mut j.angle).range(0.5..=30.0).speed(0.1).max_decimals(1).suffix("°"));
                            mm(ui, &tr!("joinery-clearance"), &mut j.clearance, 5.0);
                        }
                        WoodJointKind::PocketHole => {
                            ui.label(tr!("joinery-count"));
                            ui.add(egui::DragValue::new(&mut j.count).range(1..=32).speed(0.05));
                            mm(ui, &tr!("joinery-diameter"), &mut j.diameter, 10.0);
                            mm(ui, &tr!("joinery-depth-in", part = na.clone()), &mut j.depth, 200.0);
                            ui.checkbox(&mut j.flip, tr!("joinery-flip")).on_hover_text(tr!("joinery-flip-tip"));
                        }
                    });
                    if let Some((name, n)) = j.hardware(None).filter(|_| j.kind != WoodJointKind::Screws) {
                        ui.label(RichText::new(format!("{n} × {name}")).small().color(theme::text_dim()));
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
            && self.doc.model.joints.get(i).is_some_and(|o| *o != j)
        {
            self.doc.try_commit(&tr!("joinery-undo-edit"), move |m| m.set_wood_joint(i, j));
        }
        if let Some(j) = select {
            self.select_joint_parts(&j);
        }
        if let Some((i, k)) = kind {
            self.joinery.draft = None;
            self.doc.try_commit(&tr!("joinery-undo-edit"), move |m| m.set_wood_joint_kind(i, k));
        }
        if let Some(i) = swap {
            self.joinery.draft = None;
            let mut j = self.doc.model.joints[i].clone();
            std::mem::swap(&mut j.a, &mut j.b);
            self.doc.try_commit(&tr!("joinery-undo-edit"), move |m| m.set_wood_joint(i, j));
        }
        if let Some(i) = remove {
            self.joinery.draft = None;
            self.doc.commit(&tr!("joinery-undo-remove"), move |m| m.remove_wood_joint(i));
        }
    }
}
