//! Window → Generator Editor: the source of a generator (the console's code editor), a live
//! preview built a moment after each edit (drag to turn it), the parameter fields and formula
//! values for the preview, and Save to Library (a `.rhai` in the components folder) / Apply to
//! Document / Place. See
//! `docs/GENERATORS.md`.

use super::code_editor::ScriptEditor;
use super::generators::{EXAMPLES_FOLDER, build_preview, formula_rows, kept_values, param_field, show_value};
use super::theme;
use crate::app::{Action, VarstotasApp};
use egui::{Color32, Context, RichText, Sense, Stroke, Ui, pos2, vec2};
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{GenValue, GenValues, Generator};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use varstotas_i18n::tr;

/// Seconds after the last edit before the preview is rebuilt.
const SETTLE: f64 = 0.4;

const TEMPLATE: &str = r##"// @generator My Generator
// @description What it makes.
// @param width:length(100..3000/10) = 600 "Width"
// @param depth:length(100..1000/10) = 400 "Depth"
// @param height:length(100..3000/10) = 750 "Height"
// @param board:length(10..40) = 18 "Board"
// @formula inner = width - 2 * board "Inner width"
// @check inner > 0 "Too narrow for the boards"

fn build(p) {
    let t = p.board;
    let parts = [
        box(vec3(0, 0, 0), t, p.depth, p.height),
        box(vec3(p.width - t, 0, 0), t, p.depth, p.height),
        box(vec3(t, 0, p.height - t), p.inner, p.depth, t),
    ];
    paint(parts, material("Birch", "#E8D2A6"));
}
"##;

/// What the preview shows.
pub struct Preview {
    pub edges: Vec<(DVec3, DVec3)>,
    pub size: DVec3,
}

pub struct GenEditor {
    pub open: bool,
    pub source: String,
    /// The library file it came from / was saved to.
    pub file: Option<PathBuf>,
    /// The document's generator it edits (In Model): Apply updates that one, renaming it when
    /// the header names it otherwise.
    pub doc: Option<String>,
    /// Preview values (over the defaults).
    pub values: GenValues,
    editor: ScriptEditor,
    /// The header of `source` (parsed when it changes).
    pub parsed: Result<Generator, String>,
    parsed_src: String,
    /// Source and values the preview was built from.
    built: Option<(String, GenValues)>,
    pub preview: Option<Result<Preview, String>>,
    /// Time of the last source edit.
    edited_at: f64,
    yaw: f32,
    pitch: f32,
    bufs: BTreeMap<String, String>,
    drag: Option<(String, f64)>,
    /// The last message (saved, applied, …).
    pub status: String,
    /// Height of the code editor last frame.
    pub code_h: f32,
}

impl Default for GenEditor {
    fn default() -> Self {
        Self {
            open: false,
            source: String::new(),
            file: None,
            doc: None,
            values: GenValues::new(),
            editor: ScriptEditor::with_id("generator_editor_code"),
            parsed: Err(String::new()),
            parsed_src: "\u{0}".into(),
            built: None,
            preview: None,
            edited_at: f64::NEG_INFINITY,
            yaw: 0.65,
            pitch: 0.45,
            bufs: BTreeMap::new(),
            drag: None,
            status: String::new(),
            code_h: 0.0,
        }
    }
}

/// `My Generator` → `my_generator`.
fn slug(name: &str) -> String {
    let s: String = name.chars().map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' }).collect();
    let s = s.trim_matches('_').to_string();
    if s.is_empty() { "generator".into() } else { s }
}

/// Where Save to Library writes generator `name` in the components folder `dir`: its own file
/// there, else `<dir>/<slug>.rhai` (a free name).
fn save_path(dir: &Path, file: Option<&Path>, name: &str) -> PathBuf {
    if let Some(f) = file.filter(|f| f.starts_with(dir) && !f.starts_with(dir.join(EXAMPLES_FOLDER))) {
        return f.to_path_buf();
    }
    let ours = |p: &Path| std::fs::read_to_string(p).ok().and_then(|s| Generator::parse(&s).ok()).is_some_and(|g| g.name == name);
    let base = slug(name);
    (1..)
        .map(|n| dir.join(if n == 1 { format!("{base}.rhai") } else { format!("{base}_{n}.rhai") }))
        .find(|p| !p.exists() || ours(p))
        .expect("a free file name")
}

impl GenEditor {
    /// Open `source` (from `file`, if a library file).
    pub fn edit(&mut self, source: &str, file: Option<PathBuf>) {
        self.open = true;
        self.source = source.to_string();
        self.file = file;
        self.doc = None;
        self.values.clear();
        self.bufs.clear();
        self.built = None;
        self.preview = None;
        self.edited_at = f64::NEG_INFINITY;
        self.status.clear();
    }

    /// Open the document's generator `name` (In Model).
    pub fn edit_doc(&mut self, source: &str, name: &str) {
        self.edit(source, None);
        self.doc = Some(name.to_string());
    }

    pub fn new_generator(&mut self) {
        self.edit(TEMPLATE, None);
    }

    fn reparse(&mut self) {
        if self.parsed_src != self.source {
            self.parsed = Generator::parse(&self.source);
            self.parsed_src = self.source.clone();
            if let Ok(g) = &self.parsed {
                self.values = kept_values(g, &self.values);
            }
        }
    }
}

/// Turn and fit the preview edges into `rect` (orthographic, Z up).
fn draw_preview(ui: &Ui, rect: egui::Rect, edges: &[(DVec3, DVec3)], yaw: f32, pitch: f32) {
    let (sy, cy) = (yaw as f64).sin_cos();
    let (sp, cp) = (pitch as f64).sin_cos();
    let proj = |p: DVec3| {
        let x = p.x * cy - p.y * sy;
        let y = p.x * sy + p.y * cy;
        (x, -(p.z * cp + y * sp))
    };
    let pts: Vec<((f64, f64), (f64, f64))> = edges.iter().map(|(a, b)| (proj(*a), proj(*b))).collect();
    let (mut lo, mut hi) = ((f64::MAX, f64::MAX), (f64::MIN, f64::MIN));
    for (a, b) in &pts {
        for q in [a, b] {
            lo = (lo.0.min(q.0), lo.1.min(q.1));
            hi = (hi.0.max(q.0), hi.1.max(q.1));
        }
    }
    if pts.is_empty() {
        return;
    }
    let inner = rect.shrink(14.0);
    let s = (inner.width() as f64 / (hi.0 - lo.0).max(1e-9)).min(inner.height() as f64 / (hi.1 - lo.1).max(1e-9));
    let c = ((lo.0 + hi.0) / 2.0, (lo.1 + hi.1) / 2.0);
    let to = |q: (f64, f64)| pos2(rect.center().x + ((q.0 - c.0) * s) as f32, rect.center().y + ((q.1 - c.1) * s) as f32);
    let painter = ui.painter_at(rect);
    let stroke = Stroke::new(1.0, super::theme::pick(Color32::from_gray(50), Color32::from_gray(200)));
    for (a, b) in pts {
        painter.line_segment([to(a), to(b)], stroke);
    }
}

impl VarstotasApp {
    pub(crate) fn gen_editor_window(&mut self, ctx: &Context) {
        if !self.gen_editor.open {
            return;
        }
        self.gen_editor.reparse();
        self.rebuild_gen_preview(ctx);
        let mut open = true;
        let pos = self.viewport.view_rect.min + vec2(40.0, 30.0);
        let frame = egui::Frame::window(&ctx.global_style()).fill(super::theme::card_bg());
        let mut win = egui::Window::new(tr!("gen-editor-title")).id(egui::Id::new("Generator Editor")).open(&mut open).default_pos(pos);
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        win.default_size([920.0, 600.0]).frame(frame).resizable(true).show(ctx, |ui| self.gen_editor_ui(ui));
        if !open {
            self.gen_editor.open = false;
        }
    }

    /// Rebuild the preview once the source has rested for [`SETTLE`] (values: at once).
    fn rebuild_gen_preview(&mut self, ctx: &Context) {
        let ed = &mut self.gen_editor;
        let now = ctx.input(|i| i.time);
        let want = (ed.source.clone(), ed.values.clone());
        if ed.built.as_ref() == Some(&want) {
            return;
        }
        let source_changed = ed.built.as_ref().is_none_or(|b| b.0 != want.0);
        if source_changed && now - ed.edited_at < SETTLE {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(SETTLE));
            return;
        }
        let r = build_preview(&self.doc.model, &want.0, &want.1)
            .map(|(_, c)| Preview { edges: c.preview.clone(), size: c.bounds.max - c.bounds.min });
        let ed = &mut self.gen_editor;
        ed.preview = Some(r);
        ed.built = Some(want);
    }

    fn gen_editor_ui(&mut self, ui: &mut Ui) {
        let unit = self.doc.model.unit;
        let name = self.gen_editor.parsed.as_ref().ok().map(|g| g.name.clone());
        ui.horizontal(|ui| {
            if ui.button(tr!("gen-new")).on_hover_text(tr!("gen-new-hint")).clicked() {
                self.gen_editor.new_generator();
            }
            if ui.button(tr!("gen-open")).clicked()
                && let Some(p) = rfd::FileDialog::new()
                    .add_filter(tr!("gen-filter"), &["rhai"])
                    .set_directory(self.library_dir().unwrap_or_default())
                    .pick_file()
            {
                match std::fs::read_to_string(&p) {
                    Ok(s) => self.gen_editor.edit(&s, Some(p)),
                    Err(e) => self.gen_editor.status = tr!("console-cannot-read", path = p.display().to_string(), error = e.to_string()),
                }
            }
            ui.separator();
            let ok = name.is_some();
            if ui
                .add_enabled(ok, egui::Button::new(tr!("gen-save")))
                .on_hover_text(tr!("gen-save-hint"))
                .clicked()
            {
                self.actions.push(Action::GenEditorSave);
            }
            let (apply, tip) = match &self.gen_editor.doc {
                Some(_) => (tr!("apply"), tr!("gen-apply-hint")),
                None => (tr!("gen-apply-doc"), tr!("gen-apply-doc-hint")),
            };
            if ui.add_enabled(ok, egui::Button::new(apply)).on_hover_text(tip).clicked()
            {
                self.actions.push(Action::GenEditorApply);
            }
            if ui.add_enabled(ok, egui::Button::new(tr!("gen-place"))).on_hover_text(tr!("gen-place-hint")).clicked() {
                self.actions.push(Action::PlaceGenerator(self.gen_editor.source.clone(), self.gen_editor.values.clone()));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if super::script_help::help_button(ui) {
                    self.open_script_help(super::script_help::Topic::Generators, None);
                }
            });
        });
        ui.separator();
        // The editor fills the window (it grows when the window is resized).
        let h = (ui.available_height() - 24.0).max(160.0);
        self.gen_editor.code_h = h;
        let w = ui.available_width();
        let edited = ui
            .horizontal_top(|ui| {
                // (the vertical separator would take any height offered)
                ui.set_max_height(h);
                let mut edited = false;
                ui.vertical(|ui| {
                    ui.set_width((w * 0.56).max(260.0));
                    let ed = &mut self.gen_editor;
                    let f1 = ed.editor.f1(ui, &ed.source);
                    edited = ed.editor.show(ui, &mut ed.source, h).response.changed();
                    if let Some(w) = f1 {
                        self.open_script_help(super::script_help::Topic::Generators, Some(w));
                    }
                });
                ui.separator();
                ui.vertical(|ui| self.gen_editor_side(ui, unit, h));
                edited
            })
            .inner;
        if edited {
            self.gen_editor.edited_at = ui.input(|i| i.time);
            self.gen_editor.status.clear();
        }
        let (msg, colour) = match (&self.gen_editor.parsed, &self.gen_editor.preview) {
            (Err(e), _) => (e.clone(), Color32::from_rgb(200, 50, 40)),
            (_, Some(Err(e))) => (e.clone(), Color32::from_rgb(200, 50, 40)),
            _ => (self.gen_editor.status.clone(), theme::text_dim()),
        };
        let file = match (&self.gen_editor.doc, &self.gen_editor.file) {
            (Some(n), _) => tr!("gen-in-model", name = n.clone()),
            (None, Some(f)) => f.display().to_string(),
            (None, None) => tr!("gen-not-saved"),
        };
        ui.label(RichText::new(if msg.is_empty() { file } else { msg }).small().color(colour));
    }

    /// Preview, parameters and formulas.
    fn gen_editor_side(&mut self, ui: &mut Ui, unit: varstotas_kernel::LengthUnit, h: f32) {
        let pw = ui.available_width();
        let (rect, resp) = ui.allocate_exact_size(vec2(pw, 240.0), Sense::drag());
        ui.painter().rect_filled(rect, 6.0, super::theme::pick(Color32::from_gray(246), Color32::from_rgb(0x36, 0x3A, 0x41)));
        if resp.dragged() {
            let d = resp.drag_delta();
            let ed = &mut self.gen_editor;
            ed.yaw += d.x * 0.01;
            ed.pitch = (ed.pitch + d.y * 0.01).clamp(-1.5, 1.5);
        }
        match &self.gen_editor.preview {
            Some(Ok(p)) => {
                draw_preview(ui, rect, &p.edges, self.gen_editor.yaw, self.gen_editor.pitch);
                let f = |v: f64| varstotas_kernel::units::format_length(v, unit);
                ui.painter().text(
                    rect.left_bottom() + vec2(8.0, -6.0),
                    egui::Align2::LEFT_BOTTOM,
                    format!("{} × {} × {}", f(p.size.x), f(p.size.y), f(p.size.z)),
                    egui::FontId::proportional(11.0),
                    theme::text_dim(),
                );
            }
            Some(Err(_)) => {
                ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, tr!("gen-build-failed"), egui::FontId::proportional(13.0), Color32::from_rgb(200, 50, 40));
            }
            None => {}
        }
        resp.on_hover_text(tr!("gen-drag-turn"));
        let Ok(g) = self.gen_editor.parsed.clone() else { return };
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(&g.name).strong()).on_hover_text(&g.description);
            if ui.small_button(tr!("gen-defaults")).clicked() {
                self.gen_editor.values.clear();
                self.gen_editor.bufs.clear();
            }
        });
        let materials: Vec<String> = self.doc.model.materials.values().map(|m| m.name.clone()).collect();
        let values = g.normalize(&self.gen_editor.values).unwrap_or_else(|_| g.defaults());
        let mut change: Option<(String, GenValue)> = None;
        egui::ScrollArea::vertical().id_salt("gen_editor_params").max_height((h - 290.0).max(80.0)).auto_shrink([false, false]).show(ui, |ui| {
            egui::Grid::new("gen_editor_grid").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                let ed = &mut self.gen_editor;
                for p in &g.params {
                    let v = values.get(&p.key).unwrap_or(&p.default).clone();
                    ui.label(&p.label).on_hover_text(&p.key);
                    let shown = show_value(&v, &p.kind, unit);
                    let buf = ed.bufs.entry(p.key.clone()).or_insert_with(|| shown.clone());
                    match param_field(ui, p, &v, unit, &materials, buf, &shown, &mut ed.drag) {
                        Ok(Some(x)) => change = Some((p.key.clone(), x)),
                        Ok(None) => {}
                        Err(e) => ed.status = e,
                    }
                    ui.end_row();
                }
                formula_rows(ui, &g, &values, unit);
            });
        });
        if let Some((k, v)) = change {
            let ed = &mut self.gen_editor;
            let p = g.param(&k).expect("a parameter of the header");
            match p.normalize(&v) {
                Ok(v) => {
                    ed.values.insert(k.clone(), v);
                    ed.bufs.remove(&k);
                }
                Err(e) => ed.status = e,
            }
        }
    }

    /// Save to Library: write the source and rescan the tray.
    pub(crate) fn gen_editor_save(&mut self) {
        let Ok(g) = &self.gen_editor.parsed else { return };
        let Some(dir) = self.library_dir() else { return self.flash_error(tr!("gen-no-home")) };
        let path = save_path(&dir, self.gen_editor.file.as_deref(), &g.name);
        match std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&path, &self.gen_editor.source)) {
            Ok(()) => {
                self.gen_editor.status = tr!("gen-saved", path = path.display().to_string());
                self.gen_editor.file = Some(path);
                self.refresh_library();
            }
            Err(e) => self.gen_editor.status = tr!("gen-cannot-save", path = path.display().to_string(), error = e.to_string()),
        }
    }

    /// Apply to Document: add or update the generator (its components are rebuilt).
    pub(crate) fn gen_editor_apply(&mut self) {
        let Ok(g) = &self.gen_editor.parsed else { return };
        let name = g.name.clone();
        // The document's generator renamed in its header: it and its components follow.
        if let Some(old) = self.gen_editor.doc.clone().filter(|o| *o != name && self.doc.model.generators.contains_key(o)) {
            if self.doc.model.generators.contains_key(&name) {
                self.gen_editor.status = tr!("gen-name-taken", name = name.clone());
                return;
            }
            let g = g.clone();
            let r = self.doc.try_commit(&tr!("gen-undo-update", name = name.clone()), |m| {
                m.rename_generator(&old, g);
                varstotas_script::generator::rebuild_all(m, &name)
            });
            self.gen_editor.status = match r {
                Some(_) => {
                    self.gen_editor.doc = Some(name.clone());
                    tr!("gen-renamed-applied", old = old.clone(), name = name.clone())
                }
                None => self.doc.last_error.take().unwrap_or_default(),
            };
            self.library.gens.thumbs.clear();
            return;
        }
        if self.doc.model.generators.get(&name).is_some_and(|x| x.source == self.gen_editor.source) {
            self.gen_editor.status = tr!("gen-same-source", name = name.clone());
            return;
        }
        let src = self.gen_editor.source.clone();
        let r = self.doc.try_commit(&tr!("gen-undo-update", name = name.clone()), |m| varstotas_script::generator::add(m, &src));
        self.gen_editor.status = match r {
            Some(_) => tr!("gen-applied", name = name.clone()),
            None => {
                let e = self.doc.last_error.take().unwrap_or_default();
                crate::applog::error("Generator Editor", &e); // i18n-ignore
                e
            }
        };
        self.library.gens.thumbs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_paths() {
        let dir = std::env::temp_dir().join(format!("varstotas_gen_editor_{}", std::process::id()));
        std::fs::create_dir_all(dir.join(EXAMPLES_FOLDER)).unwrap();
        assert_eq!(slug("My Shelf 2"), "my_shelf_2");
        assert_eq!(save_path(&dir, None, "My Shelf"), dir.join("my_shelf.rhai"));
        // A file outside the folder is saved as a copy in it.
        assert_eq!(save_path(&dir, Some(Path::new("/elsewhere/shelf.rhai")), "Shelf"), dir.join("shelf.rhai"));
        // Another generator's file is not overwritten.
        std::fs::write(dir.join("shelf.rhai"), "// @generator Other\nfn build(p) {}").unwrap();
        assert_eq!(save_path(&dir, None, "Shelf"), dir.join("shelf_2.rhai"));
        std::fs::write(dir.join("mine.rhai"), "x").unwrap();
        assert_eq!(save_path(&dir, Some(&dir.join("mine.rhai")), "Shelf"), dir.join("mine.rhai"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
