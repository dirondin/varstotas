//! Modal dialogs: "save changes?" confirmation, autosave recovery, Preferences and About.

use super::theme;
use crate::app::VarstotasApp;
use egui::{Context, Id, Modal, RichText};

impl VarstotasApp {
    pub(crate) fn dialogs(&mut self, ctx: &Context) {
        if let Some(pending) = self.confirm.clone() {
            let mut choice: Option<u8> = None;
            let resp = Modal::new(Id::new("confirm_save")).show(ctx, |ui| {
                ui.set_width(360.0);
                ui.heading("Save changes?");
                ui.add_space(4.0);
                ui.label(format!(
                    "Do you want to save the changes you made to “{}”?",
                    self.doc.display_name()
                ));
                ui.label(RichText::new("Your changes will be lost if you don't save them.").color(theme::TEXT_DIM));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Don't Save").clicked() {
                        choice = Some(1);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(RichText::new("Save").strong())).clicked() {
                            choice = Some(2);
                        }
                        if ui.button("Cancel").clicked() {
                            choice = Some(0);
                        }
                    });
                });
            });
            if resp.should_close() && choice.is_none() {
                choice = Some(0);
            }
            match choice {
                Some(0) => self.confirm = None,
                Some(1) => {
                    self.confirm = None;
                    self.doc.dirty = false;
                    self.proceed(pending);
                }
                Some(2) => {
                    self.confirm = None;
                    if self.save() {
                        self.proceed(pending);
                    }
                }
                _ => {}
            }
        }

        if self.confirm.is_none()
            && let Some(r) = self.autosave.recoveries.first().cloned()
        {
            let mut choice: Option<Option<bool>> = None;
            let resp = Modal::new(Id::new("recover_autosave")).show(ctx, |ui| {
                ui.set_width(400.0);
                ui.heading("Recover autosaved model?");
                ui.add_space(4.0);
                ui.label(match &r.original {
                    Some(_) => format!(
                        "Varstotas found an autosaved copy of “{}” from {} that is newer than the saved file.",
                        r.display_name(),
                        r.age_text()
                    ),
                    None => format!("Varstotas found an autosaved untitled model from {}.", r.age_text()),
                });
                ui.label(
                    RichText::new("It may contain changes that were not saved, e.g. after a crash.")
                        .color(theme::TEXT_DIM),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Discard").on_hover_text("Delete the autosaved copy").clicked() {
                        choice = Some(Some(false));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(RichText::new("Recover").strong())).clicked() {
                            choice = Some(Some(true));
                        }
                        if ui.button("Not Now").on_hover_text("Keep the autosaved copy for later").clicked() {
                            choice = Some(None);
                        }
                    });
                });
            });
            if resp.should_close() && choice.is_none() {
                choice = Some(None);
            }
            if let Some(c) = choice {
                self.resolve_recovery(c);
            }
        }

        if self.show_prefs {
            let mut open = true;
            egui::Window::new("Preferences")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(RichText::new("General").strong());
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.prefs.autosave, "Auto-save every");
                        ui.add_enabled(
                            self.prefs.autosave,
                            egui::DragValue::new(&mut self.prefs.autosave_minutes).range(1..=120).speed(0.2),
                        );
                        ui.label("minutes");
                    });
                    ui.label(
                        RichText::new("Unsaved changes are written to <name>.vrst~autosave next to the file.")
                            .small()
                            .color(theme::TEXT_DIM),
                    );
                });
            self.show_prefs = open;
        }

        if self.show_about {
            let resp = Modal::new(Id::new("about")).show(ctx, |ui| {
                ui.set_width(320.0);
                ui.heading("Varstotas");
                ui.label("A light, native 3D modeler in the spirit of classic modellers.");
                ui.label(RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION"))).color(theme::TEXT_DIM));
                ui.add_space(8.0);
                ui.button("Close").clicked()
            });
            if resp.inner || resp.should_close() {
                self.show_about = false;
            }
        }
    }
}
