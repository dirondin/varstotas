//! Modal dialogs: "save changes?" confirmation, autosave recovery, Tape Measure resize,
//! PNG export, template chooser, Preferences and About.

use super::theme;
use crate::app::VarstotasApp;
use crate::templates::Template;
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
                        if ui.add(egui::Button::new(RichText::new("Save").strong())).clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
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

        if self.confirm.is_none()
            && self.autosave.recoveries.is_empty()
            && let Some(f) = self.resize_confirm
        {
            let mut choice: Option<bool> = None;
            let what = if self.doc.active_path.is_empty() { "the model" } else { "the active group or component" };
            let resp = Modal::new(Id::new("confirm_resize")).show(ctx, |ui| {
                ui.set_width(360.0);
                ui.heading("Resize?");
                ui.add_space(4.0);
                ui.label(format!("Do you want to resize {what}?"));
                ui.label(
                    RichText::new(format!("Everything in it is scaled by {:.4} about its origin.", f))
                        .color(theme::TEXT_DIM),
                );
                ui.label(RichText::new("Enter = Yes · Esc = No").small().color(theme::TEXT_DIM));
                ui.add_space(10.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let yes = ui.add(egui::Button::new(RichText::new("Yes").strong()));
                    if yes.clicked() {
                        choice = Some(true);
                    }
                    if ui.button("No").clicked() {
                        choice = Some(false);
                    }
                });
                // Yes is the default button.
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    choice = Some(true);
                }
            });
            if resp.should_close() && choice.is_none() {
                choice = Some(false);
            }
            if let Some(c) = choice {
                self.resize_confirm = None;
                if c {
                    self.apply_resize(f);
                }
            }
        }

        if let Some(mut p) = self.text_prompt.clone() {
            let mut choice: Option<bool> = None;
            let resp = Modal::new(Id::new("text_prompt")).show(ctx, |ui| {
                ui.set_width(380.0);
                ui.heading(&p.title);
                ui.add_space(4.0);
                let mut out = egui::TextEdit::singleline(&mut p.value).desired_width(f32::INFINITY).show(ui);
                if !p.focused {
                    // Focus with the whole value selected, so typing replaces it.
                    out.response.request_focus();
                    let n = p.value.chars().count();
                    out.state.cursor.set_char_range(Some(egui::text::CCursorRange::two(
                        egui::text::CCursor::new(0),
                        egui::text::CCursor::new(n),
                    )));
                    out.state.store(ui.ctx(), out.response.id);
                    p.focused = true;
                }
                ui.label(RichText::new(&p.hint).small().color(theme::TEXT_DIM));
                ui.add_space(10.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new("OK").strong())).clicked() {
                        choice = Some(true);
                    }
                    if ui.button("Cancel").clicked() {
                        choice = Some(false);
                    }
                });
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    choice = Some(true);
                }
            });
            if resp.should_close() && choice.is_none() {
                choice = Some(false);
            }
            match choice {
                Some(true) => {
                    if self.apply_prompt(&p) {
                        self.text_prompt = None;
                    } else {
                        self.flash_error("Invalid value");
                        self.text_prompt = Some(p);
                    }
                }
                Some(false) => self.text_prompt = None,
                None => self.text_prompt = Some(p),
            }
        }

        if let Some(mut d) = self.export_dialog.clone() {
            let vp = self.viewport.size();
            let ppp = ctx.pixels_per_point();
            let aspect = (vp.x / vp.y.max(1.0)).max(1e-3) as f64;
            let mut choice: Option<bool> = None;
            let resp = Modal::new(Id::new("export_png")).show(ctx, |ui| {
                ui.set_width(340.0);
                ui.heading("Export 2D Graphic");
                ui.add_space(6.0);
                let max = varstotas_render::MAX_EXPORT_DIM;
                egui::Grid::new("export_grid").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                    ui.label("Width");
                    let w = ui.add(egui::DragValue::new(&mut d.width).range(1..=max).suffix(" px"));
                    ui.end_row();
                    ui.label("Height");
                    let h = ui.add(egui::DragValue::new(&mut d.height).range(1..=max).suffix(" px"));
                    ui.end_row();
                    if d.lock_aspect {
                        if w.changed() {
                            d.height = ((d.width as f64 / aspect).round() as u32).clamp(1, max);
                        } else if h.changed() {
                            d.width = ((d.height as f64 * aspect).round() as u32).clamp(1, max);
                        }
                    }
                });
                ui.checkbox(&mut d.lock_aspect, "Use view aspect ratio");
                ui.horizontal(|ui| {
                    let px = vp * ppp;
                    if ui.button("Viewport size").on_hover_text(format!("{} × {} px (screen pixels)", px.x.round(), px.y.round())).clicked() {
                        d.width = px.x.round().max(1.0) as u32;
                        d.height = px.y.round().max(1.0) as u32;
                    }
                    if ui.button("×2").on_hover_text("Double the image size").clicked() {
                        d.width = (d.width * 2).min(max);
                        d.height = (d.height * 2).min(max);
                    }
                });
                ui.checkbox(&mut d.transparent, "Transparent background");
                ui.label(
                    RichText::new("Guides are included; the selection and tool previews are not.")
                        .small()
                        .color(theme::TEXT_DIM),
                );
                ui.add_space(10.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new("Export…").strong())).clicked() {
                        choice = Some(true);
                    }
                    if ui.button("Cancel").clicked() {
                        choice = Some(false);
                    }
                });
            });
            if resp.should_close() && choice.is_none() {
                choice = Some(false);
            }
            match choice {
                Some(true) => {
                    self.export_dialog = None;
                    if let Some(mut path) = rfd::FileDialog::new()
                        .add_filter("PNG image", &["png"])
                        .set_file_name(format!("{}.png", self.doc.display_name()))
                        .save_file()
                    {
                        if path.extension().is_none() {
                            path.set_extension("png");
                        }
                        self.export_png_to(path, [d.width, d.height], d.transparent);
                    }
                }
                Some(false) => self.export_dialog = None,
                None => self.export_dialog = Some(d),
            }
        }

        if self.show_templates && self.confirm.is_none() && self.autosave.recoveries.is_empty() {
            let mut picked: Option<Template> = None;
            let resp = Modal::new(Id::new("choose_template")).show(ctx, |ui| {
                ui.set_width(380.0);
                ui.heading("Choose a template");
                ui.label(
                    RichText::new("New models start with these units. You can change the default in Preferences.")
                        .color(theme::TEXT_DIM),
                );
                ui.add_space(8.0);
                for t in Template::ALL {
                    let b = egui::Button::new(RichText::new(t.name()).strong()).min_size(egui::vec2(360.0, 28.0));
                    let r = ui.add(b);
                    ui.label(RichText::new(t.description()).small().color(theme::TEXT_DIM));
                    ui.add_space(4.0);
                    if r.clicked() {
                        picked = Some(t);
                    }
                }
            });
            if resp.should_close() && picked.is_none() {
                picked = Some(Template::ArchitecturalMm);
            }
            if let Some(t) = picked {
                self.show_templates = false;
                self.prefs.template = Some(t);
                // Only the empty startup model is replaced.
                if !self.doc.dirty && self.doc.history.undo_label().is_none() {
                    self.actions.push(crate::app::Action::NewFromTemplate(t));
                }
            }
        }

        if self.show_prefs {
            // Modal (centred over the window, below nothing): blocks the model behind it.
            let resp = Modal::new(Id::new("preferences")).show(ctx, |ui| {
                    ui.set_width(360.0);
                    ui.heading("Preferences");
                    ui.add_space(6.0);
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
                    ui.add_space(8.0);
                    ui.label(RichText::new("Template").strong());
                    let cur = self.prefs.template.unwrap_or(Template::ArchitecturalMm);
                    let mut t = cur;
                    egui::ComboBox::from_id_salt("default_template").selected_text(t.name()).width(260.0).show_ui(
                        ui,
                        |ui| {
                            for x in Template::ALL {
                                ui.selectable_value(&mut t, x, x.name());
                            }
                        },
                    );
                    if t != cur || self.prefs.template.is_none() {
                        self.prefs.template = Some(t);
                    }
                    ui.label(RichText::new("Used by File → New.").small().color(theme::TEXT_DIM));
                    ui.add_space(10.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(egui::Button::new(RichText::new("Done").strong())).clicked()
                    })
                    .inner
                });
            if resp.inner || resp.should_close() {
                self.show_prefs = false;
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
