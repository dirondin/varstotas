//! Plugins in the app (see `plugins.rs`): the Extensions menu, File → Import / Export entries,
//! the parameter dialog, point picking, object selection and running. Console code with a
//! plugin header (`@param`, `@pick`, `@select`) goes the same way as the pseudo plugin
//! [`CONSOLE`]. A Rhai plugin run is one undo step named
//! after the plugin; a Python plugin runs like File → Run Script… with its values in
//! `VARSTOTAS_PARAMS`.

use crate::app::{Action, VarstotasApp};
use crate::plugins::{Lang, ParamKind, Plugin};
use crate::ui::console::LogLine;
use crate::ui::icons::Icon;
use egui::{Context, Key, RichText, Ui};
use varstotas_kernel::glam::DVec3;
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::path::PathBuf;
use varstotas_i18n::tr;

/// Plugin index of the Script Console's code (see [`PluginState::console`]).
pub const CONSOLE: usize = usize::MAX;

/// The values being edited for a plugin run.
pub struct ParamDialog {
    pub idx: usize,
    pub values: Vec<String>,
    /// File for an importer / exporter.
    pub path: Option<PathBuf>,
    pub error: Option<String>,
    focus: bool,
}

#[derive(Default)]
pub struct PluginState {
    pub list: Vec<Plugin>,
    pub errors: Vec<String>,
    pub dialog: Option<ParamDialog>,
    /// Values waiting for the `@pick` points / `@select` objects.
    pending: Option<(usize, Map<String, Value>)>,
    /// Console code run with its header: the parsed header and the source.
    pub console: Option<(Plugin, String)>,
    /// Last values per plugin file (the dialog starts from them).
    last: HashMap<PathBuf, Vec<String>>,
}

fn vec_json(p: DVec3) -> Value {
    json!({ "$vec": [p.x, p.y, p.z] })
}

impl VarstotasApp {
    /// Scan user-installed plugins in `~/.varstotas/plugins`.
    pub(crate) fn load_plugins(&mut self) {
        let Some(dir) = crate::plugins::plugins_dir() else { return };
        let (list, errors) = crate::plugins::scan(&dir);
        for e in &errors {
            log::warn!("plugin: {e}");
        }
        log::info!("plugins: {} loaded from {}", list.len(), dir.display());
        self.plugins.list = list;
        self.plugins.errors = errors;
    }

    /// A plugin by index ([`CONSOLE`] = the console code being run).
    pub(crate) fn plugin_at(&self, idx: usize) -> Option<&Plugin> {
        if idx == CONSOLE { self.plugins.console.as_ref().map(|(p, _)| p) } else { self.plugins.list.get(idx) }
    }

    /// Console code: with a header asking for parameters, points or objects it runs like a
    /// plugin (dialog, picking, then the run); else directly.
    pub(crate) fn run_console_code(&mut self, src: &str) {
        match crate::plugins::parse(src, std::path::Path::new("Script.rhai") /* i18n-ignore */) {
            Ok(p) if !p.params.is_empty() || p.pick.is_some() || p.select.is_some() => {
                self.plugins.console = Some((p, src.to_string()));
                self.start_plugin(CONSOLE, None);
            }
            Ok(_) => self.run_script_source(src, &tr!("ext-script-undo")),
            Err(e) => {
                self.console.push(LogLine::Error(tr!("ext-header-error", error = e.to_string())));
                self.console.last = Some((false, e));
            }
        }
    }

    pub(crate) fn plugin_by_name(&self, name: &str) -> Option<usize> {
        self.plugins.list.iter().position(|p| p.name.eq_ignore_ascii_case(name.trim()))
    }

    /// Extensions menu: plugins by their menu paths, then the folder commands.
    pub(crate) fn extensions_menu(&mut self, ui: &mut Ui) {
        let entries: Vec<(usize, Vec<String>)> = self
            .plugins
            .list
            .iter()
            .enumerate()
            .filter(|(_, p)| p.in_menu())
            .map(|(i, p)| (i, p.menu.clone()))
            .collect();
        if entries.is_empty() {
            ui.label(RichText::new(tr!("ext-no-plugins")).weak());
        }
        self.menu_level(ui, &entries, 0);
        ui.separator();
        if !self.plugins.errors.is_empty() {
            let n = self.plugins.errors.len();
            if ui.button(tr!("ext-failed-to-load", n = n)).on_hover_text(self.plugins.errors.join("\n")).clicked() {
                for e in self.plugins.errors.clone() {
                    self.console.push(LogLine::Error(e));
                }
                self.console.open = true;
                ui.close();
            }
        }
        if ui.button(tr!("ext-reload")).clicked() {
            self.actions.push(Action::ReloadPlugins);
            ui.close();
        }
        if ui.button(tr!("ext-open-folder")).clicked() {
            if let Some(d) = crate::plugins::plugins_dir() {
                reveal(&d);
            }
            ui.close();
        }
    }

    fn menu_level(&mut self, ui: &mut Ui, entries: &[(usize, Vec<String>)], depth: usize) {
        let mut i = 0;
        while i < entries.len() {
            let (idx, path) = &entries[i];
            if path.len() == depth + 1 {
                let p = &self.plugins.list[*idx];
                let label = path[depth].clone();
                let r = ui.button(label);
                let r = if p.description.is_empty() { r } else { r.on_hover_text(p.description.clone()) };
                if r.clicked() {
                    self.actions.push(Action::Plugin(*idx, None));
                    ui.close();
                }
                i += 1;
                continue;
            }
            // A submenu: all following entries with the same prefix.
            let head = path[depth].clone();
            let n = entries[i..].iter().take_while(|(_, p)| p.len() > depth + 1 && p[depth] == head).count().max(1);
            let group = entries[i..i + n].to_vec();
            ui.menu_button(head, |ui| self.menu_level(ui, &group, depth + 1));
            i += n;
        }
    }

    /// File → Import / Export entries of plugins.
    pub(crate) fn plugin_file_items(&mut self, ui: &mut Ui, export: bool) {
        let items: Vec<(usize, String)> = self
            .plugins
            .list
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                let exts = if export { &p.export } else { &p.import };
                (!exts.is_empty()).then(|| (i, format!("{} ({})…", p.name, exts.iter().map(|e| format!(".{e}")).collect::<Vec<_>>().join(", "))))
            })
            .collect();
        if items.is_empty() {
            return;
        }
        ui.separator();
        let icon = if export { Icon::Export("") } else { Icon::Import("") };
        for (i, label) in items {
            if super::menu::icon_row(ui, Some(icon), &label, None, true, false).clicked() {
                self.actions.push(if export { Action::PluginExport(i) } else { Action::PluginImport(i) });
                ui.close();
            }
        }
    }

    /// File → Import items of the `@model_import` plugins.
    pub(crate) fn model_import_items(&mut self, ui: &mut Ui) {
        let items: Vec<(usize, String)> = self
            .plugins
            .list
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.model_import.is_empty())
            .map(|(i, p)| (i, format!("{} ({})…", p.name, p.model_import.iter().map(|e| format!(".{e}")).collect::<Vec<_>>().join(", "))))
            .collect();
        if items.is_empty() {
            return;
        }
        ui.separator();
        for (i, label) in items {
            if super::menu::icon_row(ui, Some(Icon::Import("")), &label, None, true, false).clicked() {
                self.actions.push(Action::ImportModelPlugin(i));
                ui.close();
            }
        }
    }

    /// File dialog for an importer / exporter plugin, then [`Self::start_plugin`].
    pub(crate) fn plugin_file(&mut self, idx: usize, export: bool) {
        let Some(p) = self.plugins.list.get(idx) else { return };
        let exts: Vec<&str> = if export { p.export.iter().map(String::as_str).collect() } else { p.import.iter().map(String::as_str).collect() };
        let d = rfd::FileDialog::new().add_filter(p.name.clone(), &exts);
        let path = if export {
            d.set_file_name(format!("{}.{}", self.doc.display_name(), exts.first().copied().unwrap_or("txt"))).save_file()
        } else {
            d.pick_file()
        };
        if let Some(path) = path {
            self.start_plugin(idx, Some(path));
        }
    }

    /// Menu click: the parameter dialog when the plugin has parameters, else run (or pick).
    pub(crate) fn start_plugin(&mut self, idx: usize, path: Option<PathBuf>) {
        // Plugins run with their own working directory: pass an absolute path.
        let path = path.map(|p| std::path::absolute(&p).unwrap_or(p));
        let Some(p) = self.plugin_at(idx) else { return };
        if p.params.is_empty() {
            let mut vars = Map::new();
            if let Some(f) = path {
                vars.insert("path".into(), json!(f.to_string_lossy()));
            }
            self.plugin_go(idx, vars);
            return;
        }
        let values = match self.plugins.last.get(&p.path) {
            Some(v) if v.len() == p.params.len() => v.clone(),
            _ => p.params.iter().map(|q| q.default.clone()).collect(),
        };
        self.plugins.dialog = Some(ParamDialog { idx, values, path, error: None, focus: true });
    }

    /// Values of the dialog as script variables.
    fn plugin_vars(&self, d: &ParamDialog) -> Result<Map<String, Value>, String> {
        let Some(p) = self.plugin_at(d.idx) else { return Err(tr!("ext-script-gone")) };
        let mut vars = Map::new();
        for (q, v) in p.params.iter().zip(&d.values) {
            vars.insert(q.key.clone(), crate::plugins::param_value(q, v, self.doc.model.unit)?);
        }
        if let Some(f) = &d.path {
            vars.insert("path".into(), json!(f.to_string_lossy()));
        }
        Ok(vars)
    }

    /// The next step: pick the points, select the objects, then run.
    fn plugin_go(&mut self, idx: usize, vars: Map<String, Value>) {
        let Some(p) = self.plugin_at(idx) else { return };
        if let Some((n, prompt)) = p.pick.clone().filter(|_| !vars.contains_key("points")) {
            self.plugins.pending = Some((idx, vars));
            self.install_tool(Box::new(crate::tools::plugin_pick::PluginPickTool::new(n, prompt)));
            return;
        }
        if let Some(prompt) = p.select.clone().filter(|_| !vars.contains_key("objects")) {
            self.plugins.pending = Some((idx, vars));
            self.install_tool(Box::new(crate::tools::plugin_select::PluginSelectTool::new(prompt)));
            return;
        }
        self.run_plugin(idx, vars);
    }

    /// The `@pick` points were clicked.
    pub(crate) fn plugin_points(&mut self, pts: Vec<DVec3>) {
        if self.rig_pick_done(&pts) {
            return;
        }
        if let Some((idx, mut vars)) = self.plugins.pending.take() {
            vars.insert("points".into(), Value::Array(pts.into_iter().map(vec_json).collect()));
            self.plugin_go(idx, vars);
        }
    }

    /// The `@select` objects were confirmed: the selection (of the open context) as `objects`.
    pub(crate) fn plugin_objects(&mut self) {
        if let Some((idx, mut vars)) = self.plugins.pending.take() {
            let path = &self.doc.active_path;
            let objs = self.doc.selection.iter().map(|e| json!({ "$entity": varstotas_script::entity_id(path, *e) })).collect();
            vars.insert("objects".into(), Value::Array(objs));
            self.plugin_go(idx, vars);
        }
    }

    pub(crate) fn run_plugin(&mut self, idx: usize, vars: Map<String, Value>) {
        let Some(p) = self.plugin_at(idx).cloned() else { return };
        if p.lang == Lang::Python {
            self.run_python_with(&p.path, Some(Value::Object(vars).to_string()));
            return;
        }
        let src = if idx == CONSOLE {
            self.plugins.console.as_ref().map(|(_, s)| s.clone()).unwrap_or_default()
        } else {
            match std::fs::read_to_string(&p.path) {
                Ok(s) => s,
                Err(e) => return self.flash_error(tr!("console-cannot-read", path = p.path.display().to_string(), error = e.to_string())),
            }
        };
        let ctx = varstotas_script::ScriptCtx { path: self.doc.active_path.clone(), selection: self.doc.selection.clone(), time_limit: None };
        self.console.push(LogLine::Input(if idx == CONSOLE { tr!("ext-run-script", name = p.name.clone()) } else { tr!("ext-run-plugin", name = p.name.clone()) }));
        let r = self.doc.try_commit_if(&p.name, |m| Ok::<_, String>(varstotas_script::eval_json(m, &ctx, &src, &vars)), |r| r.run.changed);
        let Some(r) = r else {
            let e = self.doc.last_error.take().unwrap_or_else(|| tr!("ext-plugin-failed"));
            crate::applog::error(&p.name, &e);
            self.console.log.push(LogLine::Error(e.clone()));
            self.console.last = Some((false, e.clone()));
            return self.flash_error_quiet(e);
        };
        for l in r.run.output.lines() {
            self.console.push(LogLine::Output(l.to_string()));
        }
        if let Some(sel) = r.run.selection {
            self.doc.set_selection(sel);
        }
        match r.run.error {
            Some(e) => {
                crate::applog::error(&p.name, &e);
                self.console.log.push(LogLine::Error(e.clone()));
                self.console.last = Some((false, e.clone()));
                self.flash_error_quiet(format!("{}: {e}", p.name));
            }
            None => {
                let out = r.run.output.trim_end().to_string();
                self.flash(out.lines().last().map_or_else(|| tr!("ext-done", name = p.name.clone()), str::to_string));
                self.console.last = Some((true, out));
                if !p.import.is_empty() {
                    self.actions.push(Action::ZoomExtents);
                }
            }
        }
    }

    pub(crate) fn plugin_dialog(&mut self, ctx: &Context) {
        let Some(mut d) = self.plugins.dialog.take() else { return };
        let Some(p) = self.plugin_at(d.idx).cloned() else { return };
        let pos = self.viewport.view_rect.min + egui::vec2(40.0, 40.0);
        let mut open = true;
        let (mut run, mut cancel) = (false, false);
        let mut win = egui::Window::new(&p.name).id(egui::Id::new("plugin_dialog")).open(&mut open).collapsible(false).resizable(false).default_pos(pos);
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        win.show(ctx, |ui| {
            ui.set_min_width(300.0);
            if !p.description.is_empty() {
                ui.label(RichText::new(&p.description).weak());
                ui.add_space(4.0);
            }
            if let Some(f) = &d.path {
                ui.label(RichText::new(f.display().to_string()).small());
            }
            egui::Grid::new("plugin_params").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                for (k, q) in p.params.iter().enumerate() {
                    ui.label(&q.label);
                    let v = &mut d.values[k];
                    if q.kind == ParamKind::Bool {
                        let mut b = v == "true";
                        if ui.checkbox(&mut b, "").changed() {
                            *v = b.to_string();
                        }
                    } else {
                        let r = ui.add(egui::TextEdit::singleline(v).desired_width(140.0));
                        if d.focus && k == 0 {
                            r.request_focus();
                        }
                        if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                            run = true;
                        }
                    }
                    ui.end_row();
                }
            });
            d.focus = false;
            if let Some(e) = &d.error {
                ui.colored_label(egui::Color32::from_rgb(200, 50, 40), e);
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let label = if p.pick.is_some() {
                    tr!("ext-pick-points")
                } else if p.select.is_some() {
                    tr!("ext-select-objects")
                } else {
                    tr!("ext-run")
                };
                run |= ui.button(label).clicked();
                cancel = ui.button(tr!("cancel")).clicked();
            });
        });
        if cancel || !open {
            return;
        }
        if run {
            match self.plugin_vars(&d) {
                Ok(vars) => {
                    self.plugins.last.insert(p.path.clone(), d.values.clone());
                    self.plugin_go(d.idx, vars);
                    return;
                }
                Err(e) => d.error = Some(e),
            }
        }
        self.plugins.dialog = Some(d);
    }
}

/// Show a folder in Finder / Explorer.
/// The folder [`reveal`] showed last (scripted runs only record it).
pub(crate) static LAST_REVEALED: std::sync::Mutex<Option<std::path::PathBuf>> = std::sync::Mutex::new(None);

pub(crate) fn reveal(dir: &std::path::Path) {
    let _ = std::fs::create_dir_all(dir);
    if let Ok(mut l) = LAST_REVEALED.lock() {
        *l = Some(dir.to_path_buf());
    }
    // A Finder window would cover a scripted run (it stops drawing when covered).
    if std::env::var_os("VARSTOTAS_SCRIPT").is_some() {
        log::info!("reveal {}", dir.display());
        return;
    }
    let cmd = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(windows) {
        "explorer"
    } else {
        "xdg-open"
    };
    let _ = std::process::Command::new(cmd).arg(dir).spawn();
}
