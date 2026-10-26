//! Window → Script Console: a Rhai editor over the model (see `docs/SCRIPTING.md`), and
//! File → Run Script… (`.rhai`, or `.py` through the bridge). A run is one undo step
//! ("Script"); an error changes nothing.

use super::theme;
use crate::app::VarstotasApp;
use egui::{Color32, Context, Key, Modifiers, RichText, ScrollArea};
use varstotas_i18n::tr;

/// One line of the console log.
#[derive(Clone, Debug, PartialEq)]
pub enum LogLine {
    /// The code that was run (first line).
    Input(String),
    Output(String),
    Error(String),
}

#[derive(Default)]
pub struct ConsoleState {
    pub open: bool,
    pub log: Vec<LogLine>,
    /// Earlier runs, for Cmd+↑ / Cmd+↓.
    history: Vec<String>,
    hist_pos: Option<usize>,
    /// Last run for the script harness dump: (ok, output or error).
    pub last: Option<(bool, String)>,
    editor: super::code_editor::ScriptEditor,
    /// Height of the code editor (the splitter above it drags it); 0 = default.
    pub code_h: f32,
}

impl ConsoleState {
    /// Add a line to the output; errors go to Window → Log too.
    pub fn push(&mut self, line: LogLine) {
        if let LogLine::Error(e) = &line {
            crate::applog::error("Script Console", e); // i18n-ignore
        }
        self.log.push(line);
    }
}

impl VarstotasApp {
    /// Run Rhai source against the document (one undo step when it changed the model).
    pub(crate) fn run_script_source(&mut self, src: &str, label: &str) {
        let ctx = varstotas_script::ScriptCtx {
            path: self.doc.active_path.clone(),
            selection: self.doc.selection.clone(),
            time_limit: None,
            trusted_io: true,
        };
        let first = src.lines().find(|l| !l.trim().is_empty()).unwrap_or_default().trim().to_string();
        let more = src.lines().filter(|l| !l.trim().is_empty()).count() > 1;
        self.console.push(LogLine::Input(if more { format!("{first} …") } else { first }));
        let run = self.doc.try_commit_if(
            label,
            |m| Ok::<_, String>(varstotas_script::run(m, &ctx, src)),
            |r: &varstotas_script::ScriptRun| r.changed,
        );
        let Some(run) = run else {
            let e = self.doc.last_error.take().unwrap_or_else(|| tr!("console-script-failed"));
            self.console.push(LogLine::Error(e.clone()));
            self.console.last = Some((false, e));
            return;
        };
        for l in run.output.lines() {
            self.console.push(LogLine::Output(l.to_string()));
        }
        if let Some(sel) = run.selection {
            self.doc.set_selection(sel);
        }
        match run.error {
            Some(e) => {
                self.console.push(LogLine::Error(e.clone()));
                self.flash_error_quiet(tr!("console-script-error", error = e.clone()));
                self.console.last = Some((false, e));
            }
            None => self.console.last = Some((true, run.output.trim_end().to_string())),
        }
        if self.console.log.len() > 2000 {
            self.console.log.drain(..self.console.log.len() - 2000);
        }
    }

    /// File → Run Script…: pick a `.rhai` file and run it.
    pub(crate) fn run_script_file(&mut self, path: &std::path::Path) {
        if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("py")) {
            self.run_python_file(path);
            return;
        }
        match std::fs::read_to_string(path) {
            Ok(src) => {
                let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                self.run_script_source(&src, &tr!("console-script-undo", name = name));
            }
            Err(e) => self.flash_error(tr!("console-cannot-read", path = path.display().to_string(), error = e.to_string())),
        }
    }

    pub(crate) fn console_window(&mut self, ctx: &Context) {
        if !self.console.open {
            return;
        }
        let mut open = true;
        let mut run: Option<String> = None;
        // The reference to open (and the word to look up).
        let mut help: Option<Option<String>> = None;
        let pos = self.viewport.view_rect.min + egui::vec2(16.0, 16.0);
        let frame = egui::Frame::window(&ctx.global_style()).fill(theme::card_bg());
        let win = self.placed(egui::Window::new(tr!("console-title")).id(egui::Id::new("Script Console")).open(&mut open), pos);
        win
            .default_size([560.0, 460.0])
            .frame(frame)
            .resizable(true)
            .show(ctx, |ui| {
            let mono = egui::TextStyle::Monospace;
            // Output | splitter | code | buttons: the splitter trades output for code height.
            let avail = ui.available_height();
            let max_code = (avail - 130.0).max(60.0);
            let code_h = if self.console.code_h > 0.0 { self.console.code_h } else { 150.0 }.clamp(60.0, max_code);
            let out_h = (avail - code_h - 70.0).max(50.0);
            egui::Frame::new().fill(theme::tray_bg()).inner_margin(6.0).show(ui, |ui| {
                ScrollArea::vertical().max_height(out_h).min_scrolled_height(out_h).stick_to_bottom(true).auto_shrink([false, false]).show(
                    ui,
                    |ui| {
                        if self.console.log.is_empty() {
                            ui.label(
                                RichText::new(tr!("console-hint"))
                                    .color(theme::text_dim()),
                            );
                        }
                        for l in &self.console.log {
                            let (t, c) = match l {
                                LogLine::Input(s) => (format!("› {s}"), theme::text_dim()),
                                LogLine::Output(s) => (s.clone(), ui.visuals().text_color()),
                                LogLine::Error(s) => (s.clone(), Color32::from_rgb(200, 50, 40)),
                            };
                            ui.label(RichText::new(t).text_style(mono.clone()).color(c));
                        }
                    },
                );
            });
            let (split, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 8.0), egui::Sense::drag());
            let resp = resp.on_hover_cursor(egui::CursorIcon::ResizeVertical);
            let c = if resp.hovered() || resp.dragged() { theme::text_dim() } else { Color32::from_gray(200) };
            ui.painter().hline(split.center().x - 20.0..=split.center().x + 20.0, split.center().y - 1.5, egui::Stroke::new(1.0, c));
            ui.painter().hline(split.center().x - 20.0..=split.center().x + 20.0, split.center().y + 1.5, egui::Stroke::new(1.0, c));
            self.console.code_h = (code_h - resp.drag_delta().y).clamp(60.0, max_code);
            let focused = self.console.editor.has_focus(ui);
            if focused && ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Enter)) {
                run = Some(self.prefs.console_code.clone());
            }
            if focused && !self.console.history.is_empty() {
                let up = ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::ArrowUp));
                let down = ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::ArrowDown));
                if up || down {
                    let n = self.console.history.len();
                    let pos = match (self.console.hist_pos, up) {
                        (None, true) => n - 1,
                        (Some(p), true) => p.saturating_sub(1),
                        (Some(p), false) => (p + 1).min(n - 1),
                        (None, false) => n - 1,
                    };
                    self.console.hist_pos = Some(pos);
                    self.prefs.console_code = self.console.history[pos].clone();
                }
            }
            if let Some(w) = self.console.editor.f1(ui, &self.prefs.console_code) {
                help = Some(Some(w));
            }
            self.console.editor.show(ui, &mut self.prefs.console_code, code_h);
            ui.horizontal(|ui| {
                if ui.button(tr!("console-run")).on_hover_text(tr!("console-run-hint")).clicked() {
                    run = Some(self.prefs.console_code.clone());
                }
                if ui.button(tr!("console-open")).clicked()
                    && let Some(p) = rfd::FileDialog::new().add_filter(tr!("console-filter-rhai"), &["rhai"]).pick_file()
                {
                    match std::fs::read_to_string(&p) {
                        Ok(s) => self.prefs.console_code = s,
                        Err(e) => self.console.push(LogLine::Error(format!("{}: {e}", p.display()))),
                    }
                }
                if ui.button(tr!("console-save")).clicked()
                    && let Some(p) = rfd::FileDialog::new().add_filter(tr!("console-filter-rhai"), &["rhai"]).set_file_name("script.rhai").save_file()
                    && let Err(e) = varstotas_kernel::io::atomic_write(&p, &self.prefs.console_code)
                {
                    self.console.push(LogLine::Error(format!("{}: {e}", p.display())));
                }
                if self.bridge.python.is_some() && ui.button(tr!("console-stop-python")).clicked() {
                    self.stop_python();
                }
                if ui.button(tr!("console-clear")).clicked() {
                    self.console.log.clear();
                }
                if super::script_help::help_button(ui) {
                    help = Some(None);
                }
                if let Some(b) = &self.bridge.server {
                    ui.label(RichText::new(tr!("console-bridge", port = b.port)).color(theme::text_dim()))
                        .on_hover_text(tr!("console-bridge-hint"));
                }
            });
        });
        if let Some(w) = help {
            self.open_script_help(super::script_help::Topic::Console, w);
        }
        if let Some(src) = run
            && !src.trim().is_empty()
        {
            if self.console.history.last() != Some(&src) {
                self.console.history.push(src.clone());
            }
            self.console.hist_pos = None;
            self.run_console_code(&src);
        }
        self.console.open = open;
    }
}
