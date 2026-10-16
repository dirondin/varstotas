//! Generators (track К) in the UI: the Generators list of the Components card (the document's
//! generators and `~/.varstotas/generators`), placing one, and the parameters of selected
//! generated components in Entity Info. See `docs/GENERATORS.md`.

use super::{fmt_len, theme};
use crate::app::{Action, VarstotasApp};
use egui::{RichText, Ui};
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::{EntityRef, GenKind, GenValue, GenValues, Generator, InstanceId, Model};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const EXAMPLES: &[(&str, &str)] = &[
    ("shelf.rhai", include_str!("../../../../examples/generators/shelf.rhai")),
    ("drawer_box.rhai", include_str!("../../../../examples/generators/drawer_box.rhai")),
];

/// Source "document" id of generator placements (never a real document).
const GENERATOR_UID: u64 = u64::MAX - 7;

#[derive(Clone)]
pub struct GenItem {
    pub name: String,
    pub description: String,
    pub source: String,
    /// The library file (None = only in the document).
    pub file: Option<PathBuf>,
}

#[derive(Default)]
pub struct GenLibrary {
    /// Library generators (files), sorted by name.
    pub items: Vec<GenItem>,
    pub errors: Vec<String>,
    pub scanned: bool,
    /// Thumbnails with the default values by source: (edges in the unit square, size), or the
    /// build error.
    thumbs: std::collections::HashMap<String, Result<(Thumb, DVec3), String>>,
}

type Thumb = Vec<(egui::Pos2, egui::Pos2)>;

/// Build `src` with its defaults in a model with the materials of `m`.
fn build_preview(m: &Model, src: &str) -> Result<(String, crate::clipboard::Clipboard), String> {
    let mut tmp = Model::new();
    tmp.unit = m.unit;
    tmp.materials = m.materials.clone();
    tmp.tags = m.tags.clone();
    tmp.generators = m.generators.clone();
    crate::document::guarded(|| {
        let name = varstotas_script::generator::add(&mut tmp, src)?;
        let i = varstotas_script::generator::place(&mut tmp, &[], &name, &GenValues::new(), DMat4::IDENTITY)?;
        let clip = crate::clipboard::Clipboard::from_parts(&tmp, GENERATOR_UID, &[], &[EntityRef::Instance(i)], &[]).map_err(|e| e.to_string())?;
        Ok::<_, String>((name, clip))
    })
    .and_then(|r| r)
}

/// `~/.varstotas/generators`.
pub fn generators_dir() -> Option<PathBuf> {
    crate::bridge::home_dir().map(|d| d.join("generators"))
}

/// Write the example generators to `<dir>/examples` (replacing older copies).
fn install_examples(dir: &Path) -> std::io::Result<()> {
    let ex = dir.join("examples");
    std::fs::create_dir_all(&ex)?;
    for (name, src) in EXAMPLES {
        let p = ex.join(name);
        if std::fs::read_to_string(&p).ok().as_deref() != Some(*src) {
            std::fs::write(p, src)?;
        }
    }
    Ok(())
}

/// `.rhai` generators under `dir` (one folder level deep).
fn scan(dir: &Path) -> (Vec<GenItem>, Vec<String>) {
    let mut files = Vec::new();
    for d in std::iter::once(dir.to_path_buf()).chain(std::fs::read_dir(dir).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.is_dir())) {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.extension().is_some_and(|x| x == "rhai") {
                files.push(p);
            }
        }
    }
    let (mut items, mut errors) = (Vec::new(), Vec::new());
    for f in files {
        match std::fs::read_to_string(&f).map_err(|e| e.to_string()).and_then(|s| Generator::parse(&s).map(|g| (g, s))) {
            Ok((g, source)) => items.push(GenItem { name: g.name, description: g.description, source, file: Some(f) }),
            Err(e) => errors.push(format!("{}: {e}", f.display())),
        }
    }
    items.sort_by_key(|a| a.name.to_lowercase());
    items.dedup_by(|a, b| a.name == b.name);
    (items, errors)
}

/// Text of a value in Entity Info.
fn show_value(v: &GenValue, kind: &GenKind, unit: varstotas_kernel::LengthUnit) -> String {
    match (v, kind) {
        (GenValue::Num(x), GenKind::Length) => fmt_len(*x, unit),
        (GenValue::Num(x), _) => format!("{x}"),
        (GenValue::Bool(b), _) => b.to_string(),
        (GenValue::Text(s), _) => s.clone(),
    }
}

impl VarstotasApp {
    pub(crate) fn refresh_generators(&mut self) {
        let lib = &mut self.library.gens;
        lib.scanned = true;
        lib.thumbs.clear();
        let Some(dir) = generators_dir() else { return };
        if let Err(e) = install_examples(&dir) {
            log::warn!("generators: cannot install the examples: {e}");
        }
        (lib.items, lib.errors) = scan(&dir);
        for e in &lib.errors {
            log::warn!("generators: {e}");
        }
    }

    /// The document's generators, then the library's (by name, once).
    pub(crate) fn generator_items(&self) -> Vec<GenItem> {
        let mut out: Vec<GenItem> = self
            .library
            .gens
            .items
            .iter()
            .map(|it| match self.doc.model.generators.get(&it.name) {
                // The document's copy is what its components were built with.
                Some(g) if g.source != it.source => GenItem { source: g.source.clone(), file: None, ..it.clone() },
                _ => it.clone(),
            })
            .collect();
        for g in self.doc.model.generators.values() {
            if !out.iter().any(|x| x.name == g.name) {
                out.push(GenItem { name: g.name.clone(), description: g.description.clone(), source: g.source.clone(), file: None });
            }
        }
        out.sort_by_key(|a| a.name.to_lowercase());
        out
    }

    /// Start placing a component of generator `src` with its default values (the generator is
    /// added to the document with the placement).
    pub(crate) fn place_generator(&mut self, src: &str) {
        match build_preview(&self.doc.model, src) {
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

    /// The Generators part of the Components card.
    pub(crate) fn generators_panel(&mut self, ui: &mut Ui) {
        if !self.library.gens.scanned {
            self.refresh_generators();
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Generators").strong().small());
            if let Some(d) = generators_dir()
                && ui.small_button("…").on_hover_text(format!("{}\nClick to show in the Finder", d.display())).clicked()
            {
                crate::ui::extensions::reveal(&d);
            }
        });
        let filter = self.library.filter.trim().to_lowercase();
        let unit = self.doc.model.unit;
        let items: Vec<GenItem> =
            self.generator_items().into_iter().filter(|it| filter.is_empty() || it.name.to_lowercase().contains(&filter)).collect();
        for it in &items {
            if !self.library.gens.thumbs.contains_key(&it.source) {
                let t = build_preview(&self.doc.model, &it.source).map(|(_, c)| {
                    let edges: Vec<_> = c.preview.iter().take(crate::ui::library::THUMB_EDGES).copied().collect();
                    (crate::ui::library::thumbnail(&edges), c.bounds.max - c.bounds.min)
                });
                self.library.gens.thumbs.insert(it.source.clone(), t);
            }
        }
        let mut place = None;
        ui.horizontal_wrapped(|ui| {
            for it in &items {
                let mut note = it.description.clone();
                if it.file.is_none() {
                    note += if note.is_empty() { "In this document" } else { "\nIn this document" };
                }
                let (thumb, size) = match &self.library.gens.thumbs[&it.source] {
                    Ok((t, s)) => (t.as_slice(), *s),
                    Err(e) => {
                        note += &format!("\nError: {e}");
                        (&[][..], DVec3::ZERO)
                    }
                };
                if crate::ui::library::tile_ex(ui, &it.name, thumb, size, unit, true, &note) {
                    place = Some(it.source.clone());
                }
            }
        });
        if !self.library.gens.errors.is_empty() {
            ui.label(RichText::new(format!("{} generator(s) could not be read", self.library.gens.errors.len())).color(egui::Color32::from_rgb(200, 50, 40)))
                .on_hover_text(self.library.gens.errors.join("\n"));
        }
        if let Some(src) = place {
            self.actions.push(Action::PlaceGenerator(src));
        }
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
        egui::Grid::new("entity_info_generator").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            for p in &g.params {
                let v = cur.get(&p.key).unwrap_or(&p.default).clone();
                let mixed = values.iter().any(|x| x.get(&p.key) != Some(&v));
                let label = ui.label(&p.label);
                if mixed {
                    label.on_hover_text("The selected components differ; a change sets them all");
                }
                let key = p.key.clone();
                match &p.kind {
                    GenKind::Length => {
                        let Some((_, bufs)) = self.tray.gen_text.as_mut() else { continue };
                        let buf = bufs.entry(key.clone()).or_default();
                        let resp = ui.add(egui::TextEdit::singleline(buf).desired_width(120.0));
                        let text = &texts[&key];
                        if super::tray::committed(ui, &resp) && buf != text {
                            match varstotas_kernel::units::parse_length(buf, unit) {
                                Some(x) => {
                                    change.insert(key, GenValue::Num(x));
                                }
                                None => bad = Some(format!("{}: not a length", p.label)),
                            }
                        } else if !resp.has_focus() && buf != text {
                            *buf = text.clone();
                        }
                    }
                    GenKind::Bool => {
                        let mut b = v.flag().unwrap_or(false);
                        if ui.checkbox(&mut b, "").changed() {
                            change.insert(key, GenValue::Bool(b));
                        }
                    }
                    GenKind::Choice(opts) => {
                        let sel = v.text().unwrap_or_default().to_string();
                        egui::ComboBox::from_id_salt(("gen_choice", &key)).selected_text(&sel).width(120.0).show_ui(ui, |ui| {
                            for o in opts {
                                if ui.selectable_label(*o == sel, o).clicked() && *o != sel {
                                    change.insert(key.clone(), GenValue::Text(o.clone()));
                                }
                            }
                        });
                    }
                    GenKind::Material => {
                        let sel = v.text().unwrap_or_default().to_string();
                        let shown = if sel.is_empty() { "(default)".to_string() } else { sel.clone() };
                        egui::ComboBox::from_id_salt(("gen_mat", &key)).selected_text(shown).width(120.0).show_ui(ui, |ui| {
                            if ui.selectable_label(sel.is_empty(), "(default)").clicked() && !sel.is_empty() {
                                change.insert(key.clone(), GenValue::Text(String::new()));
                            }
                            for m in &materials {
                                if ui.selectable_label(*m == sel, m).clicked() && *m != sel {
                                    change.insert(key.clone(), GenValue::Text(m.clone()));
                                }
                            }
                        });
                    }
                    kind => {
                        // Numbers: a slider for angles and percentages, else a drag value; the
                        // value is applied when the drag ends (one rebuild, one undo step).
                        let mut x = match &self.tray.gen_drag {
                            Some((k, x)) if *k == key => *x,
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
                            self.tray.gen_drag = Some((key.clone(), x));
                        }
                        if (resp.drag_stopped() || (resp.changed() && !resp.dragged())) && Some(x) != v.num() {
                            self.tray.gen_drag = None;
                            change.insert(key, GenValue::Num(x));
                        } else if resp.drag_stopped() {
                            self.tray.gen_drag = None;
                        }
                    }
                }
                ui.end_row();
            }
        });
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
