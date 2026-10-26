//! Window → Work Planes: the document's guide planes (active, visible, name, delete) and the
//! active plane's origin and angles. Edits apply live; one undo step is recorded when the
//! pointer is released and no field has focus.

use crate::app::VarstotasApp;
use egui::{Context, RichText, vec2};
use varstotas_i18n::tr;
use varstotas_kernel::WorkPlane;

type PlanesState = (Vec<WorkPlane>, Option<usize>);

#[derive(Default)]
pub struct WorkPlanesWindow {
    pub open: bool,
    /// Model state before the edit in progress (restored, then committed as one step).
    orig: Option<PlanesState>,
}

fn deg_drag(v: &mut f64) -> egui::DragValue<'_> {
    egui::DragValue::new(v).speed(0.5).max_decimals(2).suffix("°")
}

impl VarstotasApp {
    pub(crate) fn work_planes_window(&mut self, ctx: &Context) {
        if !self.planes_window.open {
            self.finish_planes_edit(ctx);
            return;
        }
        let m = &self.doc.model;
        let (mut planes, mut active) = (m.work_planes.clone(), m.active_plane);
        let before = (planes.clone(), active);
        let unit = m.unit;
        let mut remove: Option<usize> = None;
        let mut open = true;
        let pos = self.viewport.view_rect.right_top() + vec2(-380.0, 40.0);
        let mut win = egui::Window::new(tr!("workplane-title"))
            .id(egui::Id::new("work_planes_window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_pos(pos);
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        win.show(ctx, |ui| {
            ui.set_width(340.0);
            if planes.is_empty() {
                ui.label(tr!("workplane-none"));
            }
            egui::Grid::new("work_planes_list").num_columns(4).spacing([8.0, 4.0]).show(ui, |ui| {
                for (i, p) in planes.iter_mut().enumerate() {
                    if ui.radio(active == Some(i), "").on_hover_text(tr!("workplane-active-tip")).clicked() {
                        active = Some(i);
                    }
                    let mut shown = !p.hidden;
                    if ui.checkbox(&mut shown, "").on_hover_text(tr!("workplane-visible")).changed() {
                        p.hidden = !shown;
                    }
                    ui.add_sized([190.0, 20.0], egui::TextEdit::singleline(&mut p.name));
                    if ui.small_button(tr!("delete")).clicked() {
                        remove = Some(i);
                    }
                    ui.end_row();
                }
            });
            ui.horizontal(|ui| {
                if ui.add_enabled(active.is_some(), egui::Button::new(tr!("workplane-deactivate"))).clicked() {
                    active = None;
                }
                ui.label(RichText::new(tr!("workplane-no-active-hint")).small().weak());
            });
            if let Some(p) = active.and_then(|i| planes.get_mut(i)) {
                ui.separator();
                ui.label(RichText::new(tr!("workplane-world", name = p.name.clone())).strong());
                egui::Grid::new("work_plane_fields").num_columns(4).spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label(tr!("workplane-origin"));
                    ui.add(super::len_drag(&mut p.origin.x, unit));
                    ui.add(super::len_drag(&mut p.origin.y, unit));
                    ui.add(super::len_drag(&mut p.origin.z, unit));
                    ui.end_row();
                    let (mut t, mut a, mut s) = p.angles();
                    let (t0, a0, s0) = (t, a, s);
                    ui.label(tr!("workplane-angles"));
                    ui.add(deg_drag(&mut t)).on_hover_text(tr!("workplane-tilt-tip"));
                    ui.add(deg_drag(&mut a)).on_hover_text(tr!("workplane-azimuth-tip"));
                    ui.add(deg_drag(&mut s)).on_hover_text(tr!("workplane-spin-tip"));
                    ui.end_row();
                    if (t, a, s) != (t0, a0, s0) {
                        p.set_angles(t, a, s);
                    }
                });
            }
        });
        if !open {
            self.planes_window.open = false;
        }
        if let Some(i) = remove {
            // Deleting is a discrete action: commit it at once (after any pending edit).
            self.finish_planes_edit_now();
            self.doc.commit(&tr!("workplane-undo-delete"), |m| m.remove_work_plane(i));
            crate::axes::sync(&self.doc.model);
            return;
        }
        if (planes.clone(), active) != before {
            if self.planes_window.orig.is_none() {
                self.planes_window.orig = Some(before);
            }
            self.doc.model.work_planes = planes;
            self.doc.model.active_plane = active;
            crate::axes::sync(&self.doc.model);
        }
        self.finish_planes_edit(ctx);
    }

    /// Record the edit in progress as one undo step once the pointer is up and nothing has focus.
    fn finish_planes_edit(&mut self, ctx: &Context) {
        if self.planes_window.orig.is_some() && !ctx.egui_is_using_pointer() && ctx.memory(|m| m.focused().is_none()) {
            self.finish_planes_edit_now();
        }
    }

    fn finish_planes_edit_now(&mut self) {
        let Some((planes, active)) = self.planes_window.orig.take() else { return };
        let m = &mut self.doc.model;
        let now = (std::mem::replace(&mut m.work_planes, planes), std::mem::replace(&mut m.active_plane, active));
        if now != (m.work_planes.clone(), m.active_plane) {
            self.doc.commit(&tr!("workplane-undo-edit"), |m| {
                m.work_planes = now.0;
                m.active_plane = now.1;
            });
        }
        crate::axes::sync(&self.doc.model);
    }
}
