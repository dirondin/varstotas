//! Startup hub; it never replaces a document without the usual unsaved-work guard.
use crate::{
    app::{Action, VarstotasApp},
    templates::Template,
};
use egui::{Context, Id, Modal, RichText};
use varstotas_i18n::tr;

impl VarstotasApp {
    pub(crate) fn start_screen(&mut self, ctx: &Context) {
        if !self.show_start || self.confirm.is_some() || !self.autosave.recoveries.is_empty() {
            return;
        }
        let mut action = None;
        let mut close = false;
        let mut guide = false;
        let mut template = self.prefs.template.unwrap_or(Template::ArchitecturalMm);
        let recent = self.prefs.recent.clone();
        let response = Modal::new(Id::new("start_screen")).show(ctx, |ui| {
            ui.set_width(580.0);
            ui.heading(RichText::new(crate::APP_NAME).size(28.0));
            ui.label(tr!("start-subtitle"));
            ui.add_space(12.0);
            ui.columns(2, |cols| {
                cols[0].heading(tr!("start-create"));
                egui::ComboBox::from_id_salt("start_template")
                    .selected_text(template.name())
                    .show_ui(&mut cols[0], |ui| {
                        for t in Template::ALL {
                            ui.selectable_value(&mut template, t, t.name());
                        }
                    });
                cols[0].add_space(6.0);
                let r = cols[0].button(tr!("start-new"));
                super::tray_lib::spot("start:new", r.rect.center());
                if r.clicked() {
                    action = Some(Action::NewFromTemplate(template));
                }
                let r = cols[0].button(tr!("start-open"));
                super::tray_lib::spot("start:open", r.rect.center());
                if r.clicked() {
                    action = Some(Action::Open);
                }
                cols[0].add_space(8.0);
                if cols[0].button(tr!("menu-examples")).clicked() {
                    super::online_mats::open_url(crate::EXAMPLES_URL);
                }
                let r = cols[0].button(tr!("tutorial-title"));
                super::tray_lib::spot("start:tutorial", r.rect.center());
                if r.clicked() { action = Some(Action::Tutorial); }
                let r = cols[0].button(tr!("start-guide"));
                super::tray_lib::spot("start:guide", r.rect.center());
                if r.clicked() {
                    guide = true;
                    close = true;
                }
                cols[1].heading(tr!("start-recent"));
                if recent.is_empty() {
                    cols[1].label(tr!("start-no-recent"));
                }
                for (i, path) in recent.iter().take(6).enumerate() {
                    let name = path
                        .file_name()
                        .unwrap_or(path.as_os_str())
                        .to_string_lossy();
                    let r = cols[1]
                        .add_enabled(path.is_file(), egui::Button::new(name).truncate())
                        .on_hover_text(path.display().to_string());
                    super::tray_lib::spot(format!("start:recent:{i}"), r.rect.center());
                    if r.clicked() {
                        action = Some(Action::OpenPath(path.clone()));
                    }
                }
            });
            ui.add_space(12.0);
            ui.separator();
            ui.label(RichText::new(tr!("start-basics-title")).strong());
            ui.label(tr!("start-basics"));
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let r = ui.checkbox(&mut self.prefs.show_start_screen, tr!("start-show"));
                super::tray_lib::spot("start:show", r.rect.center());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let r = ui.button(tr!("start-continue"));
                    super::tray_lib::spot("start:continue", r.rect.center());
                    if r.clicked() {
                        close = true;
                    }
                });
            });
        });
        self.prefs.template = Some(template);
        if guide {
            self.open_app_help(super::app_help::Topic::Navigation);
        }
        if response.should_close() || close || action.is_some() {
            self.show_start = false;
        }
        if let Some(action) = action {
            self.actions.push(action);
        }
    }
}
