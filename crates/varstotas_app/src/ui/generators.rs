//! Generators (track К) in the UI: the Generators list of the Components card (the document's
//! generators and `~/.varstotas/generators`), placing one, and the parameters of selected
//! generated components in Entity Info. See `docs/GENERATORS.md`.

use super::{fmt_len, theme};
use crate::app::{Action, VarstotasApp};
use egui::{RichText, Ui};
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::{EntityRef, GenKind, GenParam, GenValue, GenValues, Generator, InstanceId, Model};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const EXAMPLES: &[(&str, &str)] = &[
    ("shelf.rhai", include_str!("../../../../examples/generators/shelf.rhai")),
    ("drawer_box.rhai", include_str!("../../../../examples/generators/drawer_box.rhai")),
    ("stair.rhai", include_str!("../../../../examples/generators/stair.rhai")),
    ("cabinet.rhai", include_str!("../../../../examples/generators/cabinet.rhai")),
    ("railing.rhai", include_str!("../../../../examples/generators/railing.rhai")),
    ("table.rhai", include_str!("../../../../examples/generators/table.rhai")),
];

/// Source "document" id of generator placements (never a real document).
const GENERATOR_UID: u64 = u64::MAX - 7;

#[derive(Clone)]
pub struct GenItem {
    pub name: String,
    pub description: String,
    pub source: String,
    /// The library file (None = bundled or only in the document).
    pub file: Option<PathBuf>,
    /// Subfolder of the components folder ("" = the folder itself).
    pub folder: String,
    /// A bundled example.
    pub bundled: bool,
}

#[derive(Default)]
pub struct GenLibrary {
    /// Generators of the components folder (`.rhai` files), sorted by name.
    pub items: Vec<GenItem>,
    pub errors: Vec<String>,
    pub scanned: bool,
    /// Thumbnails with the default values by source: (edges in the unit square, size), or the
    /// build error.
    pub(crate) thumbs: std::collections::HashMap<String, Result<(Thumb, DVec3), String>>,
}

type Thumb = Vec<(egui::Pos2, egui::Pos2)>;

/// Build `src` with `values` (over its defaults) in a model with the materials of `m`.
pub(crate) fn build_preview(m: &Model, src: &str, values: &GenValues) -> Result<(String, crate::clipboard::Clipboard), String> {
    let mut tmp = Model::new();
    tmp.unit = m.unit;
    tmp.materials = m.materials.clone();
    tmp.tags = m.tags.clone();
    tmp.generators = m.generators.clone();
    crate::document::guarded(|| {
        let name = varstotas_script::generator::add(&mut tmp, src)?;
        let i = varstotas_script::generator::place(&mut tmp, &[], &name, values, DMat4::IDENTITY)?;
        let clip = crate::clipboard::Clipboard::from_parts(&tmp, GENERATOR_UID, &[], &[EntityRef::Instance(i)], &[]).map_err(|e| e.to_string())?;
        Ok::<_, String>((name, clip))
    })
    .and_then(|r| r)
}

/// Values of `g` kept from `values`: known keys with acceptable values (after the header
/// changed, the rest fall back to the defaults).
pub(crate) fn kept_values(g: &Generator, values: &GenValues) -> GenValues {
    values.iter().filter(|(k, v)| g.param(k).is_some_and(|p| p.normalize(v).is_ok())).map(|(k, v)| (k.clone(), v.clone())).collect()
}

/// The folder of the bundled examples in the Components list.
pub const EXAMPLES_FOLDER: &str = "Examples";
/// The Components list folder of generators found only in the open document.
pub const DOCUMENT_FOLDER: &str = "Document";

/// `~/.varstotas/generators`, where generators lived before they joined the components.
fn legacy_dir() -> Option<PathBuf> {
    crate::bridge::home_dir().map(|d| d.join("generators"))
}

/// Move the user's generators from `old` into the components folder `lib` (a free name when one
/// is taken) and drop the installed example copies; `old` goes when empty.
fn migrate_legacy(old: &Path, lib: &Path) {
    let _ = std::fs::remove_dir_all(old.join("examples"));
    let mut files = Vec::new();
    for d in std::iter::once(old.to_path_buf()).chain(std::fs::read_dir(old).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.is_dir())) {
        files.extend(std::fs::read_dir(&d).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "rhai")));
    }
    for f in files {
        let rel = f.parent().and_then(|p| p.strip_prefix(old).ok()).map(Path::to_path_buf).unwrap_or_default();
        let to_dir = lib.join(rel);
        let stem = f.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let to = (1..).map(|n| to_dir.join(if n == 1 { format!("{stem}.rhai") } else { format!("{stem} {n}.rhai") })).find(|p| !p.exists()).expect("a free name");
        match std::fs::create_dir_all(&to_dir).and_then(|_| std::fs::rename(&f, &to)) {
            Ok(()) => log::info!("generators: moved {} to {}", f.display(), to.display()),
            Err(e) => log::warn!("generators: cannot move {}: {e}", f.display()),
        }
    }
    // Only empty folders go.
    for d in std::fs::read_dir(old).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.is_dir()) {
        let _ = std::fs::remove_dir(d);
    }
    let _ = std::fs::remove_dir(old);
}

/// `.rhai` generators of the components folder `root` (two folder levels, like components).
fn scan(root: &Path) -> (Vec<GenItem>, Vec<String>) {
    let mut files = Vec::new();
    fn walk(d: &Path, depth: usize, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() && depth < 2 {
                walk(&p, depth + 1, out);
            } else if p.extension().is_some_and(|x| x == "rhai") {
                out.push(p);
            }
        }
    }
    walk(root, 0, &mut files);
    files.sort();
    let (mut items, mut errors) = (Vec::new(), Vec::new());
    for f in files {
        match std::fs::read_to_string(&f).map_err(|e| e.to_string()).and_then(|s| Generator::parse(&s).map(|g| (g, s))) {
            Ok((g, source)) => {
                let folder = f.parent().and_then(|p| p.strip_prefix(root).ok()).map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                items.push(GenItem { name: g.name, description: g.description, source, file: Some(f), folder, bundled: false })
            }
            Err(e) => errors.push(format!("{}: {e}", f.display())),
        }
    }
    items.sort_by_key(|a| a.name.to_lowercase());
    items.dedup_by(|a, b| a.name == b.name);
    (items, errors)
}

/// The bundled examples (not files; Save to Library in the editor makes an own copy).
fn bundled() -> Vec<GenItem> {
    EXAMPLES
        .iter()
        .filter_map(|(_, src)| Generator::parse(src).ok().map(|g| (g, src)))
        .map(|(g, src)| GenItem {
            name: g.name,
            description: g.description,
            source: src.to_string(),
            file: None,
            folder: EXAMPLES_FOLDER.into(),
            bundled: true,
        })
        .collect()
}

/// Text of a value in Entity Info.
pub(crate) fn show_value(v: &GenValue, kind: &GenKind, unit: varstotas_kernel::LengthUnit) -> String {
    match (v, kind) {
        (GenValue::Num(x), GenKind::Length) => fmt_len(*x, unit),
        (GenValue::Num(x), GenKind::Int) => format!("{}", x.round()),
        (GenValue::Num(x), k) => {
            let t = format!("{:.3}", x);
            let t = t.trim_end_matches('0').trim_end_matches('.');
            match k {
                GenKind::Angle => format!("{t}°"),
                GenKind::Percent => format!("{t}%"),
                _ => t.to_string(),
            }
        }
        (GenValue::Bool(b), _) => if *b { "yes" } else { "no" }.to_string(),
        (GenValue::Text(s), _) => s.clone(),
    }
}

/// The field of parameter `p` showing `v`: a length text (`buf`, `shown` when not edited), a
/// check box, a list, a slider (angles, percentages) or a drag value (applied when the drag
/// ends; `drag` holds the dragged value meanwhile). The new value on a change; `Err` for a
/// length that does not parse.
#[allow(clippy::too_many_arguments)]
pub(crate) fn param_field(
    ui: &mut Ui,
    p: &GenParam,
    v: &GenValue,
    unit: varstotas_kernel::LengthUnit,
    materials: &[String],
    buf: &mut String,
    shown: &str,
    drag: &mut Option<(String, f64)>,
) -> Result<Option<GenValue>, String> {
    let key = &p.key;
    match &p.kind {
        GenKind::Length => {
            let resp = ui.add(egui::TextEdit::singleline(buf).desired_width(120.0));
            if super::tray::committed(ui, &resp) && buf != shown {
                return match varstotas_kernel::units::parse_length(buf, unit) {
                    Some(x) => Ok(Some(GenValue::Num(x))),
                    None => Err(format!("{}: not a length", p.label)),
                };
            } else if !resp.has_focus() && buf != shown {
                *buf = shown.to_string();
            }
            Ok(None)
        }
        GenKind::Bool => {
            let mut b = v.flag().unwrap_or(false);
            Ok(ui.checkbox(&mut b, "").changed().then_some(GenValue::Bool(b)))
        }
        GenKind::Choice(opts) => {
            let sel = v.text().unwrap_or_default().to_string();
            let mut out = None;
            egui::ComboBox::from_id_salt(("gen_choice", key)).selected_text(&sel).width(120.0).show_ui(ui, |ui| {
                for o in opts {
                    if ui.selectable_label(*o == sel, o).clicked() && *o != sel {
                        out = Some(GenValue::Text(o.clone()));
                    }
                }
            });
            Ok(out)
        }
        GenKind::Material => {
            let sel = v.text().unwrap_or_default().to_string();
            let shown = if sel.is_empty() { "(default)".to_string() } else { sel.clone() };
            let mut out = None;
            egui::ComboBox::from_id_salt(("gen_mat", key)).selected_text(shown).width(120.0).show_ui(ui, |ui| {
                if ui.selectable_label(sel.is_empty(), "(default)").clicked() && !sel.is_empty() {
                    out = Some(GenValue::Text(String::new()));
                }
                for m in materials {
                    if ui.selectable_label(*m == sel, m).clicked() && *m != sel {
                        out = Some(GenValue::Text(m.clone()));
                    }
                }
            });
            Ok(out)
        }
        kind => {
            let mut x = match drag {
                Some((k, x)) if k == key => *x,
                _ => v.num().unwrap_or(0.0),
            };
            let step = p.step.unwrap_or(if *kind == GenKind::Int { 1.0 } else { 0.0 });
            let resp = match kind {
                GenKind::Angle | GenKind::Percent => {
                    let (lo, hi) = (p.min.unwrap_or(0.0), p.max.unwrap_or(if *kind == GenKind::Angle { 360.0 } else { 100.0 }));
                    let mut s = egui::Slider::new(&mut x, lo..=hi).suffix(if *kind == GenKind::Angle { "°" } else { "%" });
                    if step > 0.0 {
                        s = s.step_by(step);
                    }
                    ui.add(s)
                }
                _ => {
                    let mut d = egui::DragValue::new(&mut x).speed(if step > 0.0 { step } else { 0.1 });
                    if let (Some(lo), Some(hi)) = (p.min, p.max) {
                        d = d.range(lo..=hi);
                    }
                    if *kind == GenKind::Int {
                        d = d.fixed_decimals(0);
                    }
                    ui.add(d)
                }
            };
            if resp.dragged() {
                *drag = Some((key.clone(), x));
            }
            if (resp.drag_stopped() || (resp.changed() && !resp.dragged())) && Some(x) != v.num() {
                *drag = None;
                Ok(Some(GenValue::Num(x)))
            } else {
                if resp.drag_stopped() {
                    *drag = None;
                }
                Ok(None)
            }
        }
    }
}

/// Read-only rows of the formula values of `g` with `values`, or the error of a formula / check.
pub(crate) fn formula_rows(ui: &mut Ui, g: &Generator, values: &GenValues, unit: varstotas_kernel::LengthUnit) {
    if g.formulas.is_empty() && g.checks.is_empty() {
        return;
    }
    match varstotas_script::generator::formula_values(g, values) {
        Ok(rows) => {
            for (f, v) in rows {
                ui.label(RichText::new(&f.label).color(theme::TEXT_DIM)).on_hover_text(format!("= {}", f.expr));
                ui.label(RichText::new(show_value(&v, &f.kind, unit)).color(theme::TEXT_DIM));
                ui.end_row();
            }
        }
        Err(e) => {
            ui.label(RichText::new("Formulas").color(theme::TEXT_DIM));
            ui.label(RichText::new(e).color(egui::Color32::from_rgb(200, 50, 40)).small());
            ui.end_row();
        }
    }
}

impl VarstotasApp {
    /// Rescan the generators of the components folder (moving old `~/.varstotas/generators`
    /// files there first).
    pub(crate) fn refresh_generators(&mut self) {
        let dir = self.library_dir();
        let lib = &mut self.library.gens;
        lib.scanned = true;
        lib.thumbs.clear();
        let Some(dir) = dir else { return };
        if let Some(old) = legacy_dir().filter(|d| d.is_dir()) {
            migrate_legacy(&old, &crate::ui::library::default_dir().unwrap_or(dir.clone()));
        }
        (lib.items, lib.errors) = scan(&dir);
        for e in &lib.errors {
            log::warn!("generators: {e}");
        }
    }

    /// Every generator of the Components list: the folder's files, the bundled examples (unless
    /// a file has the name) and the document's own (by name, once; the document's source wins,
    /// its components were built with it).
    pub(crate) fn generator_items(&self) -> Vec<GenItem> {
        let mut out: Vec<GenItem> = self.library.gens.items.clone();
        out.extend(bundled().into_iter().filter(|b| !self.library.gens.items.iter().any(|x| x.name == b.name)));
        for it in &mut out {
            if let Some(g) = self.doc.model.generators.get(&it.name)
                && g.source != it.source
            {
                it.source = g.source.clone();
                it.file = None;
                it.bundled = false;
            }
        }
        for g in self.doc.model.generators.values() {
            if !out.iter().any(|x| x.name == g.name) {
                out.push(GenItem {
                    name: g.name.clone(),
                    description: g.description.clone(),
                    source: g.source.clone(),
                    file: None,
                    folder: DOCUMENT_FOLDER.into(),
                    bundled: false,
                });
            }
        }
        out.sort_by(|a, b| a.folder.cmp(&b.folder).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
        out
    }

    /// A generator tile: thumbnail with the defaults (built once per source), a gear; click
    /// places, the context menu edits or deletes a library file. Returns (place, edit, delete).
    pub(crate) fn generator_tile(&mut self, ui: &mut Ui, it: &GenItem) -> (bool, bool, bool) {
        let unit = self.doc.model.unit;
        if !self.library.gens.thumbs.contains_key(&it.source) {
            let t = build_preview(&self.doc.model, &it.source, &GenValues::new()).map(|(_, c)| {
                let edges: Vec<_> = c.preview.iter().take(crate::ui::library::THUMB_EDGES).copied().collect();
                (crate::ui::library::thumbnail(&edges), c.bounds.max - c.bounds.min)
            });
            self.library.gens.thumbs.insert(it.source.clone(), t);
        }
        let mut note = it.description.clone();
        let origin = if it.bundled {
            "Bundled example (Edit → Save to Library makes your copy)"
        } else if it.file.is_none() {
            "In this document"
        } else {
            ""
        };
        if !origin.is_empty() {
            note += &format!("{}{origin}", if note.is_empty() { "" } else { "\n" });
        }
        let (thumb, size) = match &self.library.gens.thumbs[&it.source] {
            Ok((t, s)) => (t.as_slice(), *s),
            Err(e) => {
                note += &format!("\nError: {e}");
                (&[][..], DVec3::ZERO)
            }
        };
        let resp = crate::ui::library::tile_ex(ui, &it.name, thumb, None, size, unit, true, &format!("Generator — parameters in Entity Info\n{note}"));
        let (mut place, mut edit, mut delete) = (resp.clicked(), false, false);
        resp.context_menu(|ui| {
            if ui.button("Edit Generator…").clicked() {
                edit = true;
                ui.close();
            }
            if ui.button("Place").clicked() {
                place = true;
                ui.close();
            }
            let only_doc = it.file.is_none() && !it.bundled && it.folder == DOCUMENT_FOLDER;
            if (it.file.is_some() || only_doc) && ui.button("Delete…").clicked() {
                delete = true;
                ui.close();
            }
        });
        (place, edit, delete)
    }

    /// Start placing a component of generator `src` with its default values (the generator is
    /// added to the document with the placement).
    pub(crate) fn place_generator(&mut self, src: &str, values: &GenValues) {
        match build_preview(&self.doc.model, src, values) {
            Ok((name, clip)) => {
                // A changed source replaces the document's (its components are rebuilt).
                if self.doc.model.generators.get(&name).is_some_and(|g| g.source != src)
                    && self.doc.try_commit(&format!("Update Generator {name}"), |m| varstotas_script::generator::add(m, src)).is_none()
                {
                    let e = self.doc.last_error.take().unwrap_or_default();
                    return self.flash_error(e);
                }
                let tool = crate::tools::transform::PasteTool::named(std::rc::Rc::new(clip), name);
                self.install_tool(Box::new(tool));
            }
            Err(e) => self.flash_error(format!("Generator: {e}")),
        }
    }

    /// Index of the generator named `name` among [`Self::generator_items`].
    pub(crate) fn generator_source(&self, name: &str) -> Option<String> {
        self.generator_items().into_iter().find(|g| g.name.eq_ignore_ascii_case(name.trim())).map(|g| g.source)
    }

    /// Selected generated components of one generator: (instances, generator, values of each).
    pub(crate) fn selected_generated(&self) -> Option<(Vec<InstanceId>, Generator, Vec<GenValues>)> {
        let path = &self.doc.active_path;
        let model = &self.doc.model;
        let mut ids = Vec::new();
        let mut values = Vec::new();
        let mut name: Option<&str> = None;
        for e in &self.doc.selection {
            let EntityRef::Instance(i) = e else { return None };
            let g = model.instance_generated(path, *i)?;
            if name.is_some_and(|n| n != g.generator) {
                return None;
            }
            name = Some(&g.generator);
            ids.push(*i);
            values.push(g.values.clone());
        }
        let g = model.generators.get(name?)?.clone();
        Some((ids, g, values))
    }

    /// Entity Info for selected generated components: a field per parameter (several selected
    /// are edited together), Reset and Make Unique. False when the selection is not that.
    pub(crate) fn generator_info(&mut self, ui: &mut Ui) -> bool {
        let Some((ids, g, values)) = self.selected_generated() else { return false };
        let unit = self.doc.model.unit;
        let cur = &values[0];
        let texts: BTreeMap<String, String> =
            g.params.iter().map(|p| (p.key.clone(), show_value(cur.get(&p.key).unwrap_or(&p.default), &p.kind, unit))).collect();
        if self.tray.gen_text.as_ref().map(|t| &t.0) != Some(&ids) {
            self.tray.gen_text = Some((ids.clone(), texts.clone()));
            self.tray.gen_drag = None;
        }
        let materials: Vec<String> = self.doc.model.materials.values().map(|m| m.name.clone()).collect();
        let mut change = GenValues::new();
        let mut bad = None;
        ui.add_space(6.0);
        let title = if ids.len() > 1 { format!("{} — {} components", g.name, ids.len()) } else { g.name.clone() };
        ui.label(RichText::new(title).strong()).on_hover_text(&g.description);
        let mut bufs = self.tray.gen_text.take().map(|t| t.1).unwrap_or_default();
        let mut drag = self.tray.gen_drag.take();
        egui::Grid::new("entity_info_generator").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            for p in &g.params {
                let v = cur.get(&p.key).unwrap_or(&p.default).clone();
                let mixed = values.iter().any(|x| x.get(&p.key) != Some(&v));
                let label = ui.label(&p.label);
                if mixed {
                    label.on_hover_text("The selected components differ; a change sets them all");
                }
                let buf = bufs.entry(p.key.clone()).or_default();
                match param_field(ui, p, &v, unit, &materials, buf, &texts[&p.key], &mut drag) {
                    Ok(Some(x)) => {
                        change.insert(p.key.clone(), x);
                    }
                    Ok(None) => {}
                    Err(e) => bad = Some(e),
                }
                ui.end_row();
            }
            formula_rows(ui, &g, cur, unit);
        });
        self.tray.gen_text = Some((ids.clone(), bufs));
        self.tray.gen_drag = drag;
        ui.horizontal(|ui| {
            if ui.button("Reset").on_hover_text("Back to the generator's defaults").clicked() {
                self.actions.push(Action::SetGenValues(GenValues::new(), true));
            }
            let shared = ids.len() == 1
                && self.doc.active_mesh().and_then(|m| m.instances.get(ids[0])).is_some_and(|i| self.doc.model.instance_count(i.def) > 1);
            if ui
                .add_enabled(shared, egui::Button::new("Make Unique"))
                .on_hover_text("Give this component its own definition (instances with equal values share one)")
                .clicked()
            {
                self.actions.push(Action::MakeUnique);
            }
        });
        ui.label(RichText::new(format!("Generator: {}", g.name)).small().color(theme::TEXT_DIM));
        if let Some(e) = bad {
            self.tray.gen_text = None;
            self.flash_error(e);
        } else if !change.is_empty() {
            self.tray.gen_text = None;
            self.actions.push(Action::SetGenValues(change, false));
        }
        true
    }

    /// Apply generator values to every selected generated component (one undo step).
    pub(crate) fn set_gen_values(&mut self, values: GenValues, reset: bool) {
        let Some((ids, _, _)) = self.selected_generated() else {
            return self.flash("Select generated components");
        };
        let path = self.doc.active_path.clone();
        let label = if reset { "Reset Parameters" } else { "Component Parameters" };
        let r = self.doc.try_commit(label, |m| {
            for id in &ids {
                varstotas_script::generator::set_values(m, &path, *id, &values, reset)?;
            }
            Ok::<_, String>(())
        });
        if r.is_none() {
            let e = self.doc.last_error.take().unwrap_or_default();
            self.flash_error(e);
        }
    }
}
