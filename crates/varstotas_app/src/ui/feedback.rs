//! A user-reviewed GitHub issue draft; no token, model attachment or automatic submission.
use crate::{app::VarstotasApp, applog, downloads::url_encode};
use egui::{Context, RichText};
use varstotas_i18n::tr;

pub struct FeedbackWindow {
    pub open: bool,
    pub title: String,
    pub description: String,
    pub include_errors: bool,
    diagnostics: String,
}

impl FeedbackWindow {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gpu = cc
            .wgpu_render_state
            .as_ref()
            .map(|rs| {
                let i = rs.adapter.get_info();
                format!(
                    "GPU: {}\nBackend: {:?}\nDriver: {} {}", // i18n-ignore: stable diagnostic field names
                    i.name, i.backend, i.driver, i.driver_info
                ) // i18n-ignore: stable diagnostic field names
            })
            .unwrap_or_else(|| "GPU: unavailable".into()); // i18n-ignore
        Self {
            open: false,
            title: String::new(),
            description: String::new(),
            include_errors: false,
            diagnostics: format!(
                "Varstotas: {}\nOS: {}\nArchitecture: {}\n{gpu}", // i18n-ignore: stable diagnostic field names
                crate::version_full(),
                std::env::consts::OS,
                std::env::consts::ARCH
            ),
        }
    }

    pub fn body(&self) -> String {
        let mut body = format!(
            "## Problem\n{}\n\n## Diagnostics\n```text\n{}\n```\n",
            self.description.trim(),
            self.diagnostics
        );
        if self.include_errors {
            let entries = applog::recent(10);
            let home = std::env::home_dir().map(|p| p.to_string_lossy().into_owned());
            let lines: Vec<_> = entries
                .iter()
                .map(|e| {
                    let mut text = format!("{} [{}] {}", e.level.label(), e.source, e.text);
                    if let Some(home) = &home {
                        text = text.replace(home, "~");
                    }
                    text = text.replace("```", "'''");
                    text.truncate(text.floor_char_boundary(400.min(text.len())));
                    text
                })
                .collect();
            body.push_str(&format!(
                "\n## Recent warnings and errors\n```text\n{}\n```\n", // i18n-ignore: stable report format
                lines.join("\n")
            ));
        }
        body
    }
}

pub fn issue_url(title: &str, body: &str) -> Option<String> {
    let url = format!(
        "{}?title={}&body={}",
        crate::ISSUES_URL,
        url_encode(title.trim()),
        url_encode(body)
    );
    (url.len() <= 7000).then_some(url)
}

impl VarstotasApp {
    pub(crate) fn feedback_window(&mut self, ctx: &Context) {
        if !self.feedback.open {
            return;
        }
        let mut open = true;
        let mut copy = false;
        let mut send = false;
        let w = &mut self.feedback;
        egui::Window::new(tr!("report-menu"))
            .id(egui::Id::new("feedback_window"))
            .open(&mut open)
            .default_size([620.0, 540.0])
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!(
                        "{} · {}",
                        tr!("dialog-about-version", version = env!("CARGO_PKG_VERSION")),
                        crate::build_line()
                    ))
                    .small()
                    .weak(),
                );
                ui.label(tr!("report-intro"));
                ui.label(RichText::new(tr!("report-access")).small());
                ui.add_space(6.0);
                ui.label(tr!("report-title"));
                let r = ui.add(
                    egui::TextEdit::singleline(&mut w.title)
                        .desired_width(f32::INFINITY)
                        .char_limit(160),
                );
                super::tray_lib::spot("report:title", r.rect.center());
                ui.label(tr!("report-description"));
                let r = ui.add(
                    egui::TextEdit::multiline(&mut w.description)
                        .desired_rows(5)
                        .desired_width(f32::INFINITY)
                        .hint_text(tr!("report-description-hint")),
                );
                super::tray_lib::spot("report:description", r.rect.center());
                let r = ui.checkbox(&mut w.include_errors, tr!("report-errors"));
                super::tray_lib::spot("report:errors", r.rect.center());
                ui.label(RichText::new(tr!("report-review")).small());
                let mut preview = w.body();
                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut preview)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .interactive(false),
                        );
                    });
                ui.horizontal(|ui| {
                    let r = ui.button(tr!("report-copy"));
                    super::tray_lib::spot("report:copy", r.rect.center());
                    copy = r.clicked();
                    let r = ui.add_enabled(
                        !w.title.trim().is_empty() && !w.description.trim().is_empty(),
                        egui::Button::new(tr!("report-github")),
                    );
                    super::tray_lib::spot("report:github", r.rect.center());
                    send = r.clicked();
                });
            });
        self.feedback.open = open;
        if copy {
            ctx.copy_text(self.feedback.body());
        }
        if send {
            let body = self.feedback.body();
            match issue_url(&self.feedback.title, &body) {
                Some(url) => super::online_mats::open_url(&url),
                None => {
                    ctx.copy_text(body);
                    let url = format!(
                        "{}?title={}",
                        crate::ISSUES_URL,
                        url_encode(self.feedback.title.trim())
                    );
                    super::online_mats::open_url(&url);
                    self.flash(tr!("report-paste"));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn issue_drafts_encode_user_text_and_reject_oversized_urls() {
        let url = issue_url("Ошибка & labels=bad", "a\nб?x=1#test").unwrap();
        assert!(url.starts_with(crate::ISSUES_URL));
        assert!(url.contains("%26%20labels%3Dbad"));
        assert!(url.contains("a%0A%D0%B1%3Fx%3D1%23test"));
        assert!(!url.contains("&labels="));
        assert!(issue_url("Title", &"Ж".repeat(5000)).is_none());
    }
    #[test]
    fn report_contains_only_the_selected_diagnostic_sections() {
        let mut w = FeedbackWindow {
            open: true,
            title: "Problem".into(),
            description: "Steps".into(),
            include_errors: false,
            diagnostics: "OS: test\nGPU: test".into(),
        };
        let body = w.body();
        assert!(body.contains("Steps") && body.contains("GPU: test"));
        assert!(!body.contains("Recent warnings and errors"));
        w.include_errors = true;
        assert!(w.body().contains("Recent warnings and errors"));
    }
}
