//! Modal dialogs: "save changes?" confirmation and About.

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
