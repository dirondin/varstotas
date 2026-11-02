//! Entity Info for fittings (track W, `varstotas_kernel::fitting`): a selected fitting's item,
//! holes and where it is mounted (the offset along the face, off it, turned; Detach), a group
//! or component made a fitting or not, and Mount on for a fitting selected with its part.

use crate::app::VarstotasApp;
use crate::ui::theme;
use egui::{RichText, Ui};
use varstotas_i18n::tr;
use varstotas_kernel::fitting::{Fitting, FittingHole, Mount};
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{DefId, EntityRef, InstanceId, LengthUnit, Model};

const HOLE: egui::Color32 = egui::Color32::from_rgb(0xD0, 0x20, 0x60);

/// A part's name: its instance name, else its definition's.
fn name_of(m: &Model, path: &[InstanceId]) -> String {
    let Some((&id, parent)) = path.split_last() else { return "?".into() };
    let Some(inst) = m.mesh_at(parent).and_then(|x| x.instances.get(id)) else { return "?".into() };
    if !inst.name.is_empty() {
        return inst.name.clone();
    }
    m.defs.get(inst.def).map(|d| d.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| tr!("tray-info-group"))
}

/// A fitting's holes as a table (x, y on the face, diameter, depth; 0: through), edited in a
/// draft kept while a field is dragged or typed in; the new holes once one is let go.
fn holes_ui(ui: &mut Ui, d: DefId, holes: &[FittingHole], unit: LengthUnit) -> Option<Vec<FittingHole>> {
    let key = egui::Id::new(("fitting_holes", d));
    let mut draft: Vec<FittingHole> = ui.data(|x| x.get_temp(key)).unwrap_or_else(|| holes.to_vec());
    let mut done = false;
    let mut remove = None;
    ui.horizontal(|ui| {
        ui.label(RichText::new(tr!("fitting-holes")).strong()).on_hover_text(tr!("fitting-holes-tip"));
        let r = ui.small_button(tr!("fitting-hole-add"));
        super::tray_lib::spot("fitting:hole_add", r.rect.center());
        if r.on_hover_text(tr!("fitting-hole-add-tip")).clicked() {
            let next = draft.last().map_or(FittingHole { at: DVec3::ZERO, diameter: 5.0, depth: 10.0 }, |h| FittingHole { at: h.at + DVec3::X * 32.0, ..*h });
            draft.push(next);
            done = true;
        }
    });
    if draft.is_empty() {
        ui.label(RichText::new(tr!("fitting-no-holes")).color(theme::text_dim()));
    } else {
        egui::Grid::new(("fitting_holes_grid", d)).num_columns(5).spacing([4.0, 4.0]).show(ui, |ui| {
            ui.label("");
            for t in ["X", "Y", "Ø"] {
                ui.label(RichText::new(t).small().color(theme::text_dim()));
            }
            ui.label(RichText::new(tr!("fitting-hole-depth")).small().color(theme::text_dim())).on_hover_text(tr!("fitting-hole-depth-tip"));
            ui.end_row();
            for (i, h) in draft.iter_mut().enumerate() {
                let r = super::small_icon_button(ui, super::icons::Icon::Trash, &tr!("fitting-hole-remove"));
                super::tray_lib::spot(format!("fitting:hole:{i}:remove"), r.rect.center());
                if r.clicked() {
                    remove = Some(i);
                }
                let mut field = |ui: &mut Ui, v: &mut f64, name: &str, range: std::ops::RangeInclusive<f64>| {
                    let r = ui.add(super::len_drag(v, unit).range(range).speed(0.2));
                    super::tray_lib::spot(format!("fitting:hole:{i}:{name}"), r.rect.center());
                    done |= r.drag_stopped() || r.lost_focus();
                };
                field(ui, &mut h.at.x, "x", -2000.0..=2000.0);
                field(ui, &mut h.at.y, "y", -2000.0..=2000.0);
                field(ui, &mut h.diameter, "d", 0.5..=200.0);
                field(ui, &mut h.depth, "depth", 0.0..=500.0);
                ui.end_row();
            }
        });
    }
    if let Some(i) = remove {
        draft.remove(i);
        done = true;
    }
    if done {
        ui.data_mut(|x| x.remove::<Vec<FittingHole>>(key));
        return (draft != holes).then_some(draft);
    }
    if draft != holes {
        ui.data_mut(|x| x.insert_temp(key, draft));
    } else {
        ui.data_mut(|x| x.remove::<Vec<FittingHole>>(key));
    }
    None
}

impl VarstotasApp {
    /// The holes of the selected fittings on the model: a ring where each starts and a line
    /// as deep as it goes (a through hole: a short dashed one).
    pub(crate) fn fitting_holes_overlay(&self, o3: &mut varstotas_render::Overlay3d) {
        let m = &self.doc.model;
        for e in self.doc.selection.iter().take(64) {
            let EntityRef::Instance(id) = e else { continue };
            let mut p = self.doc.active_path.clone();
            p.push(*id);
            let Some(f) = m.def_at(&p).and_then(|d| m.fitting_of(d)) else { continue };
            let xf = m.transform_at(&p);
            for h in &f.holes {
                let c = xf.transform_point3(h.at);
                let (u, v) = (xf.transform_vector3(DVec3::X).normalize_or_zero(), xf.transform_vector3(DVec3::Y).normalize_or_zero());
                let down = -xf.transform_vector3(DVec3::Z).normalize_or_zero();
                let r = h.diameter * 0.5;
                let ring: Vec<DVec3> = (0..=24).map(|k| {
                    let a = k as f64 / 24.0 * std::f64::consts::TAU;
                    c + (u * a.cos() + v * a.sin()) * r
                }).collect();
                for w in ring.windows(2) {
                    crate::inference::push_line(o3, w[0], w[1], HOLE, 2.0, false, true);
                }
                let depth = if h.depth > 0.0 { h.depth } else { 30.0 };
                crate::inference::push_line(o3, c, c + down * depth, HOLE, 1.5, h.depth <= 0.0, true);
            }
        }
    }

    /// The first selected fitting's holes (x, y, diameter, depth, whole mm), for the debug dump.
    pub(crate) fn fitting_holes_dump(&self) -> Vec<(i64, i64, i64, i64)> {
        let m = &self.doc.model;
        let f = self.doc.selection.iter().find_map(|e| {
            let EntityRef::Instance(id) = e else { return None };
            let mut p = self.doc.active_path.clone();
            p.push(*id);
            m.def_at(&p).and_then(|d| m.fitting_of(d))
        });
        f.map(|f| f.holes.iter().map(|h| (h.at.x.round() as i64, h.at.y.round() as i64, h.diameter.round() as i64, h.depth.round() as i64)).collect()).unwrap_or_default()
    }

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
                            // on the shopping list by the name it was given, else its definition's
                            let item = model.mesh_at(&p[..p.len() - 1]).and_then(|x| x.instances.get(*id)).map(|i| i.name.clone()).unwrap_or_default();
                            self.doc.commit(&tr!("fitting-make"), |m| {
                                if let Some(def) = m.defs.get_mut(d) {
                                    def.fitting = Some(Fitting { item, ..Fitting::default() });
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
    fn fitting_def_ui(&mut self, ui: &mut Ui, p: &[InstanceId], d: DefId, f: &Fitting) {
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
        if let Some(holes) = holes_ui(ui, d, &f.holes, unit) {
            new_fitting = Some(Some(Fitting { holes, ..f.clone() }));
        }
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
