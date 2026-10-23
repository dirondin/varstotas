//! Window → Log: every error and warning of the session (`crate::applog`) — newest at the
//! bottom, with the time, where it came from and how many times it repeated; filters by level
//! and text, Copy and Clear. The status bar shows a red count of errors not seen yet; a click
//! opens this window.

use super::theme;
use crate::app::VarstotasApp;
use crate::applog::{self, Entry, Level};
use egui::{Color32, Context, RichText, ScrollArea};
use varstotas_i18n::tr;

pub struct LogWindow {
    pub open: bool,
    pub errors: bool,
    pub warnings: bool,
    pub filter: String,
    /// Bring the window to the front on the next frame.
    raise: bool,
    /// The entries as of a log version.
    cache: (u64, Vec<Entry>),
}

impl Default for LogWindow {
    fn default() -> Self {
        Self { open: false, errors: true, warnings: true, filter: String::new(), raise: false, cache: (u64::MAX, Vec::new()) }
    }
}

const ERR: Color32 = Color32::from_rgb(0xB0, 0x2A, 0x1C);
const WARN: Color32 = Color32::from_rgb(0xA0, 0x62, 0x00);

impl LogWindow {
    /// The entries shown with the current filters.
    fn shown(&self) -> Vec<&Entry> {
        let q = self.filter.trim().to_lowercase();
        self.cache
            .1
            .iter()
            .filter(|e| match e.level {
                Level::Error => self.errors,
                Level::Warn => self.warnings,
            })
            .filter(|e| q.is_empty() || e.text.to_lowercase().contains(&q) || e.source.to_lowercase().contains(&q))
            .collect()
    }
}

fn line(e: &Entry) -> String {
    let n = if e.count > 1 { format!(" (×{})", e.count) } else { String::new() };
    format!("{} {} [{}] {}{n}", applog::clock(e.at), e.level.label(), e.source, e.text)
}

impl VarstotasApp {
    /// Window → Log and the status bar button: open (in front) or close.
    pub(crate) fn toggle_log(&mut self, open: bool) {
        self.log_window.open = open;
        self.log_window.raise = open;
    }

    pub(crate) fn log_window(&mut self, ctx: &Context) {
        if !self.log_window.open {
            return;
        }
        applog::mark_seen();
        let v = applog::version();
        if self.log_window.cache.0 != v {
            self.log_window.cache = applog::snapshot();
        }
        let mut open = true;
        let pos = self.viewport.view_rect.left_bottom() + egui::vec2(20.0, -340.0);
        let mut win = egui::Window::new(tr!("log-title")).id(egui::Id::new("log_window")).open(&mut open).default_pos(pos).default_size([640.0, 300.0]).resizable(true);
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        win.show(ctx, |ui| {
            let lw = &mut self.log_window;
            let (n_err, n_warn) = lw.cache.1.iter().fold((0, 0), |(a, b), e| if e.level == Level::Error { (a + 1, b) } else { (a, b + 1) });
            ui.horizontal(|ui| {
                ui.toggle_value(&mut lw.errors, RichText::new(tr!("log-errors", n = n_err)).color(ERR));
                ui.toggle_value(&mut lw.warnings, RichText::new(tr!("log-warnings", n = n_warn)).color(WARN));
                ui.add(egui::TextEdit::singleline(&mut lw.filter).hint_text(tr!("log-filter")).desired_width(160.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(tr!("log-clear")).clicked() {
                        applog::clear();
                    }
                    if ui.button(tr!("log-copy")).on_hover_text(tr!("log-copy-hint")).clicked() {
                        let text: Vec<String> = lw.shown().into_iter().map(line).collect();
                        ui.ctx().copy_text(text.join("\n"));
                    }
                });
            });
            ui.separator();
            let shown = lw.shown();
            egui::Frame::new().fill(theme::tray_bg()).inner_margin(6.0).show(ui, |ui| {
                ScrollArea::vertical().stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
                    if shown.is_empty() {
                        let msg = if lw.cache.1.is_empty() { tr!("log-empty") } else { tr!("log-no-match") };
                        ui.label(RichText::new(msg).color(theme::text_dim()));
                    }
                    for e in shown {
                        ui.horizontal_top(|ui| {
                            ui.label(RichText::new(applog::clock(e.at)).monospace().small().color(theme::text_dim()));
                            let (mark, c) = if e.level == Level::Error { ("●", ERR) } else { ("▲", WARN) };
                            ui.label(RichText::new(mark).small().color(c)).on_hover_text(e.level.label());
                            if !e.source.is_empty() {
                                ui.label(RichText::new(&e.source).small().strong());
                            }
                            let n = if e.count > 1 { format!("  ×{}", e.count) } else { String::new() };
                            let r = ui.add(egui::Label::new(RichText::new(format!("{}{n}", e.text)).color(c)).wrap().sense(egui::Sense::click()));
                            r.context_menu(|ui| {
                                if ui.button(tr!("log-copy")).clicked() {
                                    ui.ctx().copy_text(line(e));
                                    ui.close();
                                }
                            });
                        });
                    }
                });
            });
        });
        self.log_window.open = open;
        if std::mem::take(&mut self.log_window.raise) {
            ctx.move_to_top(egui::LayerId::new(egui::Order::Middle, egui::Id::new("log_window")));
        }
    }

    /// The status bar's log button: a red count of errors not seen yet, else a quiet "Log".
    pub(crate) fn log_badge(&mut self, ui: &mut egui::Ui) {
        let n = applog::unseen();
        let resp = if n > 0 {
            let text = RichText::new(tr!("log-badge-errors", n = n)).color(Color32::WHITE).small().strong();
            ui.add(egui::Button::new(text).fill(ERR).corner_radius(8.0))
        } else {
            ui.add(egui::Button::new(RichText::new(tr!("log-badge")).small().color(theme::text_dim())).frame(false))
        };
        super::tray_lib::spot("status:log", resp.rect.center());
        if resp.on_hover_text(tr!("log-badge-hint")).clicked() {
            self.toggle_log(!self.log_window.open || n > 0);
        }
    }
}
