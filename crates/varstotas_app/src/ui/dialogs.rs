//! Modal dialogs: "save changes?" confirmation, autosave recovery, Tape Measure resize,
//! PNG export, template chooser, Preferences and About.

use super::theme;
use crate::app::VarstotasApp;
use crate::templates::Template;
use egui::{Context, Id, Modal, RichText};
use varstotas_i18n::tr;

impl VarstotasApp {
    pub(crate) fn dialogs(&mut self, ctx: &Context) {
        self.library_delete_dialog(ctx);
        self.def_delete_dialog(ctx);
        self.component_rename_dialog(ctx);
        self.material_delete_dialog(ctx);
        self.folder_dialog(ctx);
        self.folder_delete_dialog(ctx);
        if let Some(pending) = self.confirm.clone() {
            let mut choice: Option<u8> = None;
            let resp = Modal::new(Id::new("confirm_save")).show(ctx, |ui| {
                ui.set_width(360.0);
                ui.heading(tr!("dialog-save-title"));
                ui.add_space(4.0);
                ui.label(tr!("dialog-save-question", name = self.doc.display_name()));
                ui.label(RichText::new(tr!("dialog-save-warning")).color(theme::text_dim()));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(tr!("dialog-dont-save")).clicked() {
                        choice = Some(1);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(RichText::new(tr!("dialog-save")).strong())).clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            choice = Some(2);
                        }
                        if ui.button(tr!("cancel")).clicked() {
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
                ui.heading(tr!("dialog-recover-title"));
                ui.add_space(4.0);
                ui.label(match &r.original {
                    Some(_) => tr!("dialog-recover-copy", name = r.display_name(), age = r.age_text()),
                    None => tr!("dialog-recover-untitled", age = r.age_text()),
                });
                ui.label(
                    RichText::new(tr!("dialog-recover-note"))
                        .color(theme::text_dim()),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(tr!("dialog-discard")).on_hover_text(tr!("dialog-discard-hint")).clicked() {
                        choice = Some(Some(false));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(RichText::new(tr!("dialog-recover")).strong())).clicked() {
                            choice = Some(Some(true));
                        }
                        if ui.button(tr!("dialog-not-now")).on_hover_text(tr!("dialog-not-now-hint")).clicked() {
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
            let question = tr!(if self.doc.active_path.is_empty() { "dialog-resize-model" } else { "dialog-resize-group" });
            let resp = Modal::new(Id::new("confirm_resize")).show(ctx, |ui| {
                ui.set_width(360.0);
                ui.heading(tr!("dialog-resize-title"));
                ui.add_space(4.0);
                ui.label(question.clone());
                ui.label(
                    RichText::new(tr!("dialog-resize-scaled", factor = format!("{:.4}", f)))
                        .color(theme::text_dim()),
                );
                ui.label(RichText::new(tr!("dialog-resize-keys")).small().color(theme::text_dim()));
                ui.add_space(10.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let yes = ui.add(egui::Button::new(RichText::new(tr!("dialog-yes")).strong()));
                    if yes.clicked() {
                        choice = Some(true);
                    }
                    if ui.button(tr!("dialog-no")).clicked() {
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
                ui.label(RichText::new(&p.hint).small().color(theme::text_dim()));
                ui.add_space(10.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new(tr!("ok")).strong())).clicked() {
                        choice = Some(true);
                    }
                    if ui.button(tr!("cancel")).clicked() {
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
                        self.flash_error(tr!("dialog-invalid-value"));
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
                ui.heading(tr!("dialog-export-title"));
                ui.add_space(6.0);
                let max = varstotas_render::MAX_EXPORT_DIM;
                egui::Grid::new("export_grid").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                    ui.label(tr!("dialog-width"));
                    let w = ui.add(egui::DragValue::new(&mut d.width).range(1..=max).suffix(" px"));
                    ui.end_row();
                    ui.label(tr!("dialog-height"));
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
                ui.checkbox(&mut d.lock_aspect, tr!("dialog-export-aspect"));
                ui.horizontal(|ui| {
                    let px = vp * ppp;
                    if ui.button(tr!("dialog-export-viewport")).on_hover_text(tr!("dialog-export-viewport-hint", w = px.x.round().to_string(), h = px.y.round().to_string())).clicked() {
                        d.width = px.x.round().max(1.0) as u32;
                        d.height = px.y.round().max(1.0) as u32;
                    }
                    if ui.button("×2").on_hover_text(tr!("dialog-export-double")).clicked() {
                        d.width = (d.width * 2).min(max);
                        d.height = (d.height * 2).min(max);
                    }
                });
                ui.checkbox(&mut d.content.transparent, tr!("dialog-export-transparent"));
                ui.horizontal(|ui| {
                    ui.checkbox(&mut d.content.axes, tr!("dialog-export-axes"));
                    ui.checkbox(&mut d.content.guides, tr!("dialog-export-guides"));
                    ui.checkbox(&mut d.content.dims, tr!("dialog-export-dims"));
                });
                ui.label(
                    RichText::new(tr!("dialog-export-note"))
                        .small()
                        .color(theme::text_dim()),
                );
                ui.add_space(10.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new(tr!("dialog-export")).strong())).clicked() {
                        choice = Some(true);
                    }
                    if ui.button(tr!("cancel")).clicked() {
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
                        .add_filter(tr!("dialog-filter-png"), &["png"])
                        .set_file_name(format!("{}.png", self.doc.display_name()))
                        .save_file()
                    {
                        if path.extension().is_none() {
                            path.set_extension("png");
                        }
                        self.export_png_to(path, [d.width, d.height], d.content);
                    }
                }
                Some(false) => self.export_dialog = None,
                None => self.export_dialog = Some(d),
            }
        }

        if let Some(mut o) = self.vector_dialog {
            let ortho = self.camera.ortho;
            let mut choice: Option<&str> = None;
            let resp = Modal::new(Id::new("export_drawing")).show(ctx, |ui| {
                ui.set_width(340.0);
                ui.heading(tr!("dialog-drawing-title"));
                ui.add_space(6.0);
                ui.label(RichText::new(tr!("dialog-drawing-intro")).color(theme::text_dim()));
                ui.add_space(4.0);
                ui.add_enabled_ui(ortho, |ui| {
                    ui.horizontal(|ui| {
                        let mut to_scale = o.scale.is_some() && ortho;
                        if ui.checkbox(&mut to_scale, tr!("dialog-drawing-scale")).changed() {
                            o.scale = to_scale.then_some(o.scale.unwrap_or(50.0));
                        }
                        let mut n = o.scale.unwrap_or(50.0);
                        if ui.add_enabled(to_scale, egui::DragValue::new(&mut n).range(1.0..=10000.0).speed(1.0)).changed() {
                            o.scale = Some(n);
                        }
                    });
                });
                if !ortho {
                    ui.label(
                        RichText::new(tr!("dialog-drawing-perspective"))
                            .small()
                            .color(theme::text_dim()),
                    );
                } else if o.scale.is_none() {
                    ui.label(RichText::new(tr!("dialog-drawing-fitted")).small().color(theme::text_dim()));
                }
                ui.checkbox(&mut o.profiles, tr!("dialog-drawing-profiles"));
                ui.checkbox(&mut o.hidden, tr!("dialog-drawing-hidden"));
                ui.checkbox(&mut o.dims, tr!("dialog-drawing-dims"));
                ui.checkbox(&mut o.sections, tr!("dialog-drawing-sections"));
                ui.add_space(10.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new(tr!("dialog-drawing-pdf")).strong())).clicked() {
                        choice = Some("pdf");
                    }
                    if ui.add(egui::Button::new(RichText::new(tr!("dialog-drawing-svg")).strong())).clicked() {
                        choice = Some("svg");
                    }
                    if ui.button(tr!("cancel")).clicked() {
                        choice = Some("");
                    }
                });
            });
            if resp.should_close() && choice.is_none() {
                choice = Some("");
            }
            match choice {
                Some("") => self.vector_dialog = None,
                Some(ext) => {
                    self.vector_dialog = None;
                    self.vector_opts = o;
                    if let Some(mut path) = rfd::FileDialog::new()
                        .add_filter(ext.to_uppercase(), &[ext])
                        .set_file_name(format!("{}.{ext}", self.doc.display_name()))
                        .save_file()
                    {
                        if path.extension().is_none() {
                            path.set_extension(ext);
                        }
                        self.export_to(&path);
                    }
                }
                None => self.vector_dialog = Some(o),
            }
        }

        if let Some(mut p) = self.text3d_dialog.clone() {
            let unit = self.doc.model.unit;
            let mut choice: Option<bool> = None;
            let resp = Modal::new(Id::new("text3d")).show(ctx, |ui| {
                ui.set_width(360.0);
                ui.heading(tr!("dialog-text3d-title"));
                ui.add_space(6.0);
                let te = ui.add(egui::TextEdit::multiline(&mut p.text).desired_rows(3).desired_width(f32::INFINITY));
                if !te.has_focus() && ui.memory(|m| m.focused().is_none()) {
                    te.request_focus();
                }
                ui.add_space(4.0);
                egui::Grid::new("text3d_grid").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                    ui.label(tr!("dialog-text3d-font"));
                    egui::ComboBox::from_id_salt("text3d_font").selected_text(p.font.name()).show_ui(ui, |ui| {
                        for f in crate::text3d::Text3dFont::ALL {
                            ui.selectable_value(&mut p.font, f, f.name());
                        }
                    });
                    ui.end_row();
                    ui.label(tr!("dialog-height"));
                    ui.add(super::len_drag(&mut p.height, unit).range(0.1..=1.0e6));
                    ui.end_row();
                    ui.checkbox(&mut p.filled, tr!("dialog-text3d-filled"));
                    ui.label("");
                    ui.end_row();
                    ui.add_enabled_ui(p.filled, |ui| ui.checkbox(&mut p.extruded, tr!("dialog-text3d-extruded")));
                    ui.add_enabled_ui(p.filled && p.extruded, |ui| ui.add(super::len_drag(&mut p.depth, unit).range(0.1..=1.0e6)));
                    ui.end_row();
                });
                ui.label(RichText::new(tr!("dialog-text3d-hint")).small().color(theme::text_dim()));
                ui.add_space(10.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let enter = ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command);
                    if ui.add(egui::Button::new(RichText::new(tr!("dialog-text3d-place")).strong())).clicked() || enter {
                        choice = Some(true);
                    }
                    if ui.button(tr!("cancel")).clicked() {
                        choice = Some(false);
                    }
                });
            });
            if resp.should_close() && choice.is_none() {
                choice = Some(false);
            }
            match choice {
                Some(true) => {
                    self.text3d_dialog = None;
                    self.actions.push(crate::app::Action::PlaceText3d(p));
                }
                Some(false) => self.text3d_dialog = None,
                None => self.text3d_dialog = Some(p),
            }
        }

        if self.show_templates && self.confirm.is_none() && self.autosave.recoveries.is_empty() {
            let mut picked: Option<Template> = None;
            let resp = Modal::new(Id::new("choose_template")).show(ctx, |ui| {
                ui.set_width(380.0);
                ui.heading(tr!("dialog-template-title"));
                ui.label(
                    RichText::new(tr!("dialog-template-intro"))
                        .color(theme::text_dim()),
                );
                ui.add_space(8.0);
                for t in Template::ALL {
                    let b = egui::Button::new(RichText::new(t.name()).strong()).min_size(egui::vec2(360.0, 28.0));
                    let r = ui.add(b);
                    ui.label(RichText::new(t.description()).small().color(theme::text_dim()));
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

        if self.show_about {
            if self.about_icon.is_none()
                && let Some(image) = crate::textures::thumbnail(include_bytes!("../../assets/icon.png"), 320)
            {
                self.about_icon = Some(ctx.load_texture("about_app_icon", image, egui::TextureOptions::LINEAR));
            }
            let resp = Modal::new(Id::new("about")).show(ctx, |ui| {
                ui.set_width(540.0);
                ui.add_space(8.0);
                ui.horizontal_top(|ui| {
                    if let Some(icon) = &self.about_icon {
                        ui.image((icon.id(), egui::vec2(156.0, 156.0)));
                        ui.add_space(20.0);
                    }
                    ui.vertical(|ui| {
                        ui.set_max_width(340.0);
                        ui.label(RichText::new(crate::APP_NAME).size(28.0).strong());
                        ui.add_space(4.0);
                        ui.label(RichText::new(tr!("dialog-about-version", version = env!("CARGO_PKG_VERSION"))).color(theme::text_dim()));
                        ui.add_space(16.0);
                        ui.label(tr!("dialog-about-tagline"));
                        ui.add_space(14.0);
                        ui.label(RichText::new(tr!("dialog-about-licence")).color(theme::text_dim()));
                    });
                });
                ui.add_space(18.0);
                ui.separator();
                ui.add_space(6.0);
                let mut close = false;
                ui.horizontal(|ui| {
                    let r = ui.button(tr!("dialog-about-licences"));
                    super::tray_lib::spot("about:licences", r.rect.center());
                    if r.clicked() {
                        self.open_licences(crate::ui::licences::Tab::Varstotas);
                        close = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let r = ui.button(tr!("close"));
                        super::tray_lib::spot("about:close", r.rect.center());
                        close |= r.clicked();
                    });
                });
                close
            });
            if resp.inner || resp.should_close() {
                self.show_about = false;
            }
        }
    }
}
