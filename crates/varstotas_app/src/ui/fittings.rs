//! Entity Info for fittings (track W, `varstotas_kernel::fitting`): a selected fitting's item,
//! holes and where it is mounted (the offset along the face, off it, turned; Detach), a group
//! or component made a fitting or not, and Mount on for a fitting selected with its part.

use crate::app::VarstotasApp;
use crate::ui::theme;
use egui::{RichText, Ui};
use varstotas_i18n::tr;
use varstotas_kernel::fitting::{Fitting, Mount};
use varstotas_kernel::{EntityRef, InstanceId, Model};

/// A part's name: its instance name, else its definition's.
fn name_of(m: &Model, path: &[InstanceId]) -> String {
    let Some((&id, parent)) = path.split_last() else { return "?".into() };
    let Some(inst) = m.mesh_at(parent).and_then(|x| x.instances.get(id)) else { return "?".into() };
    if !inst.name.is_empty() {
        return inst.name.clone();
    }
    m.defs.get(inst.def).map(|d| d.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| tr!("tray-info-group"))
}

impl VarstotasApp {
    /// The mounted fittings as (item, part), for the debug dump.
    pub(crate) fn fittings_dump(&self) -> Vec<(String, String)> {
        let m = &self.doc.model;
        m.placed_fittings().iter().map(|(f, p)| (m.def_at(f).and_then(|d| m.fitting_item(d)).unwrap_or_default(), name_of(m, p))).collect()
    }

    /// The fitting section under Entity Info; true when it showed something.
    pub(crate) fn fitting_info(&mut self, ui: &mut Ui) -> bool {
        let ids: Vec<InstanceId> = self.doc.selection.iter().take(3).filter_map(|e| if let EntityRef::Instance(id) = e { Some(*id) } else { None }).collect();
        if ids.len() != self.doc.selection.len().min(3) {
            return false;
        }
        let path = |id| {
            let mut p = self.doc.active_path.clone();
            p.push(id);
            p
        };
        let model = &self.doc.model;
        let is_fitting = |p: &[InstanceId]| model.def_at(p).is_some_and(|d| model.fitting_of(d).is_some());
        match ids.as_slice() {
            [id] => {
                let p = path(*id);
                let Some(d) = model.def_at(&p) else { return false };
                match model.fitting_of(d).cloned() {
                    Some(f) => self.fitting_def_ui(ui, &p, d, &f),
                    None => {
                        ui.separator();
                        let b = ui.button(tr!("fitting-make"));
                        super::tray_lib::spot("fitting:make", b.rect.center());
                        if b.on_hover_text(tr!("fitting-make-tip")).clicked() {
                            self.doc.commit(&tr!("fitting-make"), |m| {
                                if let Some(def) = m.defs.get_mut(d) {
                                    def.fitting = Some(Fitting::default());
                                }
                            });
                        }
                    }
                }
                true
            }
            [a, b] => {
                let (pa, pb) = (path(*a), path(*b));
                let (f, part) = match (is_fitting(&pa), is_fitting(&pb)) {
                    (true, false) => (pa, pb),
                    (false, true) => (pb, pa),
                    _ => return false,
                };
                ui.separator();
                let text = tr!("fitting-mount-on", fitting = name_of(model, &f), part = name_of(model, &part));
                let r = ui.button(text);
                super::tray_lib::spot("fitting:mount", r.rect.center());
                if r.on_hover_text(tr!("fitting-mount-tip")).clicked() {
                    self.doc.try_commit(&tr!("fitting-undo-mount"), move |m| m.mount_fitting(&f, &part));
                }
                true
            }
            _ => false,
        }
    }

    /// A selected fitting: its item and holes, its mount.
    fn fitting_def_ui(&mut self, ui: &mut Ui, p: &[InstanceId], d: varstotas_kernel::DefId, f: &Fitting) {
        let unit = self.doc.model.unit;
        ui.separator();
        ui.label(RichText::new(tr!("fitting-title")).strong());
        let mut item = f.item.clone();
        let mut new_fitting: Option<Option<Fitting>> = None;
        let mut new_mount: Option<Option<Mount>> = None;
        let (&id, prefix) = p.split_last().expect("a path to an instance");
        let mount = self.doc.model.mesh_at(prefix).and_then(|x| x.instances.get(id)).and_then(|i| i.mount.clone());
        egui::Grid::new("fitting_info").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(tr!("fitting-item"));
            let name = self.doc.model.defs.get(d).map(|x| x.name.clone()).unwrap_or_default();
            let r = ui.add(egui::TextEdit::singleline(&mut item).hint_text(name).desired_width(170.0));
            super::tray_lib::spot("fitting:item", r.rect.center());
            if r.on_hover_text(tr!("fitting-item-tip")).lost_focus() && item != f.item {
                new_fitting = Some(Some(Fitting { item: item.clone(), ..f.clone() }));
            }
            ui.end_row();
            ui.label(tr!("fitting-holes"));
            let holes = f.holes.iter().map(|h| format!("Ø{}", super::fmt_len(h.diameter, unit))).collect::<Vec<_>>();
            ui.label(if holes.is_empty() { tr!("fitting-no-holes") } else { holes.join(", ") }).on_hover_text(tr!("fitting-holes-tip"));
            ui.end_row();
            match &mount {
                Some(mt) => {
                    let mut on = prefix.to_vec();
                    on.extend(&mt.on);
                    ui.label(tr!("fitting-on"));
                    ui.label(name_of(&self.doc.model, &on));
                    ui.end_row();
                    let mut m = mt.clone();
                    ui.label(tr!("fitting-offset")).on_hover_text(tr!("fitting-offset-tip"));
                    ui.horizontal(|ui| {
                        for (i, o) in m.offset.iter_mut().enumerate() {
                            let r = ui.add(super::len_drag(o, unit).speed(0.5));
                            super::tray_lib::spot(format!("fitting:offset:{i}"), r.rect.center());
                        }
                    });
                    ui.end_row();
                    ui.label(tr!("fitting-lift"));
                    ui.add(super::len_drag(&mut m.lift, unit).range(-500.0..=500.0).speed(0.2));
                    ui.end_row();
                    ui.label(tr!("fitting-turn"));
                    ui.add(egui::DragValue::new(&mut m.turn).range(-360.0..=360.0).speed(1.0).suffix("°"));
                    ui.end_row();
                    if m != *mt {
                        m.seen = None;
                        new_mount = Some(Some(m));
                    }
                }
                None => {
                    ui.label(tr!("fitting-on"));
                    ui.label(RichText::new(tr!("fitting-not-mounted")).color(theme::text_dim())).on_hover_text(tr!("fitting-mount-tip"));
                    ui.end_row();
                }
            }
        });
        ui.horizontal(|ui| {
            if mount.is_some() {
                let r = ui.button(tr!("fitting-detach"));
                super::tray_lib::spot("fitting:detach", r.rect.center());
                if r.on_hover_text(tr!("fitting-detach-tip")).clicked() {
                    new_mount = Some(None);
                }
            }
            let r = ui.button(tr!("fitting-unmake"));
            super::tray_lib::spot("fitting:unmake", r.rect.center());
            if r.on_hover_text(tr!("fitting-unmake-tip")).clicked() {
                new_fitting = Some(None);
            }
        });
        if let Some(fit) = new_fitting {
            self.doc.commit(&tr!("fitting-undo-edit"), |m| {
                if let Some(def) = m.defs.get_mut(d) {
                    def.fitting = fit;
                }
            });
        }
        if let Some(mt) = new_mount {
            let detach = mt.is_none();
            self.doc.commit(&tr!(if detach { "fitting-detach" } else { "fitting-undo-edit" }), |m| {
                if let Some(i) = m.mesh_at_mut(prefix).and_then(|x| x.instances.get_mut(id)) {
                    i.mount = mt;
                }
            });
        }
    }
}
