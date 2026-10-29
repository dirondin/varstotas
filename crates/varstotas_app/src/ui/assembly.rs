//! Window → Assembly (track W): the exploded view, the assembly steps
//! (`varstotas_kernel::assembly`) and the instructions booklet. The steps are suggested from
//! the contact graph and edited here: moved up or down, joined, split, given a note. Clicking a
//! step shows it (later parts hidden, its own selected); "Scenes from steps" keeps one scene per
//! step; the booklet (PDF, A4 portrait) has a cover, the parts and hardware, then a page per
//! step drawn from the current view with the earlier parts grey.

use crate::app::VarstotasApp;
use crate::ui::theme;
use crate::vector_export::{Drawing, Pen, Text2, VectorOptions, view_lines};
use egui::{Context, RichText, Ui, vec2};
use std::path::Path;
use varstotas_i18n::tr;
use varstotas_kernel::assembly::{SceneAssembly, Step, suggest_steps};
use varstotas_kernel::cutlist::CutPlan;
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::{ContextPath, EntityRef, Model};
use varstotas_render::Camera;

/// A4 portrait, mm.
const PAGE: DVec2 = DVec2::new(210.0, 297.0);
const MARGIN: f64 = 15.0;

#[derive(Default)]
pub struct AssemblyState {
    pub open: bool,
    /// A step's note being typed (committed when the field loses focus).
    note: Option<(usize, String)>,
}

fn text(d: &mut Drawing, at: DVec2, size: f64, s: impl Into<String>, left: bool) {
    d.texts.push(Text2 { at, angle: 0.0, size, lines: vec![s.into()], left });
}

/// `s` cut to about `width` mm at `size` (with dots: the PDF font has no ellipsis).
fn fit(s: &str, size: f64, width: f64) -> String {
    let n = (width / (size * 0.5)).max(4.0) as usize;
    if s.chars().count() <= n { s.to_string() } else { format!("{}...", s.chars().take(n - 3).collect::<String>()) }
}

/// `s` broken into lines of at most `n` characters, at spaces.
fn wrap(s: &str, n: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in s.split_whitespace() {
        match out.last_mut() {
            Some(l) if l.chars().count() + 1 + word.chars().count() <= n => {
                l.push(' ');
                l.push_str(word);
            }
            _ => out.push(word.to_string()),
        }
    }
    out
}

/// The parts a step adds: "2 × Side (1)", numbered as the cut list numbers them.
pub(crate) fn step_parts(model: &Model, plan: &CutPlan, step: &Step) -> Vec<String> {
    let parts = varstotas_kernel::assembly::parts(model);
    let cut = varstotas_kernel::cutlist::cut_parts(model);
    let mut by: Vec<(usize, usize)> = Vec::new();
    for p in &step.parts {
        let Some((_, i)) = parts.iter().find(|(q, _)| q == p) else { continue };
        match by.iter_mut().find(|(k, _)| k == i) {
            Some(e) => e.1 += 1,
            None => by.push((*i, 1)),
        }
    }
    by.into_iter()
        .map(|(i, n)| {
            let part = &cut[i];
            let no = plan.parts.iter().position(|p| p.name == part.name && p.material == part.material).map(|k| format!(" ({})", k + 1)).unwrap_or_default();
            format!("{n} × {}{no}", part.name)
        })
        .collect()
}

/// The hardware a step uses: the joints whose later part it adds.
pub(crate) fn step_hardware(model: &Model, step: usize) -> Vec<String> {
    let mut by: Vec<(String, usize)> = Vec::new();
    for (i, j) in model.joints.iter().enumerate() {
        if model.joint_step(i) != Some(step) {
            continue;
        }
        let c = varstotas_kernel::joinery::contact(model, &j.a, &j.b);
        if let Some((name, n)) = j.hardware(c.as_ref()) {
            match by.iter_mut().find(|(k, _)| *k == name) {
                Some(e) => e.1 += n,
                None => by.push((name, n)),
            }
        }
    }
    by.into_iter().map(|(name, n)| format!("{n} × {name}")).collect()
}

/// Arrows for a step's picture (screen points, tail and head): for each part it adds, from
/// outside it, on the side it moved out to in the exploded view, to its face, the way it goes in.
fn arrows(m: &Model, step: &Step, dirs: &[(ContextPath, DVec3)], gap: f64, cam: &Camera, px: DVec2) -> Vec<(DVec2, DVec2)> {
    let mut out = Vec::new();
    for path in &step.parts {
        let Some((_, dir)) = dirs.iter().find(|(p, _)| p == path) else { continue };
        let Some((&id, parent)) = path.split_last() else { continue };
        let Some(b) = m.instance_world_bounds(parent, id) else { continue };
        let face = (b.max - b.min).abs().dot(dir.abs()) * 0.5;
        let c = (b.min + b.max) * 0.5;
        let at = |k: f64| cam.world_to_screen(c + *dir * (face + gap * k), px);
        if let (Some(tail), Some(head)) = (at(0.6), at(0.1)) {
            out.push((tail, head));
        }
    }
    out
}

fn draw_arrow(d: &mut Drawing, tail: DVec2, head: DVec2) {
    let len = (head - tail).length();
    if len < 6.0 {
        return;
    }
    let u = (head - tail) / len;
    let side = DVec2::new(-u.y, u.x);
    d.lines.push((tail, head, Pen::Profile));
    d.lines.push((head, head - u * 3.0 + side * 1.3, Pen::Profile));
    d.lines.push((head, head - u * 3.0 - side * 1.3, Pen::Profile));
}

/// The booklet: a cover, the parts and hardware, a page per step. Pictures are drawn with
/// `cam`, every step framed alike (by the whole model exploded as now).
pub(crate) fn assembly_pages(model: &Model, plan: &CutPlan, cam: &Camera, doc: &str) -> Vec<Drawing> {
    let a = &model.assembly;
    let view = a.view();
    let rect = (DVec2::new(MARGIN, 32.0), DVec2::new(PAGE.x - 2.0 * MARGIN, 175.0));
    let px = rect.1 * 4.0;
    let opts = VectorOptions { profiles: true, hidden: false, dims: false, sections: false, ..Default::default() };
    let mut all = model.clone();
    all.set_assembly_view(SceneAssembly { explode: view.explode, step: None });
    let whole = view_lines(&all, cam, px, &opts);
    // screen → paper, fitted to the whole model in `rect`
    let fit_map = |lines: &crate::vector_export::ViewLines, (min, size): (DVec2, DVec2), extra: &[DVec2]| -> Option<(DVec2, f64)> {
        let (lo, hi) = lines.bounds()?;
        let (lo, hi) = extra.iter().fold((lo, hi), |(l, h), p| (l.min(*p), h.max(*p)));
        let ext = (hi - lo).max(DVec2::splat(1e-6));
        let k = (size.x / ext.x).min(size.y / ext.y);
        Some((min + size * 0.5 - (lo + hi) * 0.5 * k, k))
    };
    // the steps' arrows (only with the model exploded), framed with the model
    let dirs = varstotas_kernel::assembly::explode_dirs(model);
    let step_arrows: Vec<Vec<(DVec2, DVec2)>> = (0..a.steps.len())
        .map(|k| {
            if view.explode <= 0.0 {
                return Vec::new();
            }
            let mut m = model.clone();
            m.set_assembly_view(SceneAssembly { explode: view.explode, step: Some(k) });
            arrows(&m, &a.steps[k], &dirs, view.explode, cam, px)
        })
        .collect();
    let ends: Vec<DVec2> = step_arrows.iter().flatten().flat_map(|&(t, h)| [t, h]).collect();
    let map = fit_map(&whole, rect, &ends);
    let picture = |d: Drawing, lines: &crate::vector_export::ViewLines, map: Option<(DVec2, f64)>| match map {
        Some((off, k)) => lines.on_paper(d, move |p| off + p * k),
        None => d,
    };
    let blank = || Drawing { size: PAGE, ..Default::default() };
    let mut pages = Vec::new();

    // cover: the model as now (exploded or not), larger
    let mut d = blank();
    text(&mut d, DVec2::new(PAGE.x * 0.5, 28.0), 9.0, fit(doc, 9.0, PAGE.x - 2.0 * MARGIN), false);
    text(&mut d, DVec2::new(PAGE.x * 0.5, 40.0), 5.0, tr!("assembly-booklet-title"), false);
    let cover = (DVec2::new(MARGIN, 55.0), DVec2::new(PAGE.x - 2.0 * MARGIN, 190.0));
    let mut d = picture(d, &whole, fit_map(&whole, cover, &[]));
    let count: usize = plan.parts.iter().map(|p| p.count).sum();
    text(&mut d, DVec2::new(PAGE.x * 0.5, 262.0), 4.0, tr!("assembly-booklet-summary", steps = a.steps.len(), parts = count), false);
    pages.push(d);

    // parts and hardware, as many pages as they take
    let mut d = blank();
    let mut y = 25.0;
    text(&mut d, DVec2::new(MARGIN, y), 6.0, tr!("assembly-booklet-parts"), true);
    y += 10.0;
    let cols = [MARGIN, MARGIN + 12.0, MARGIN + 85.0, MARGIN + 103.0, MARGIN + 155.0];
    let row = |d: &mut Drawing, y: f64, cells: [String; 5]| {
        let widths = [11.0, 72.0, 17.0, 51.0, 25.0];
        for (k, c) in cells.into_iter().enumerate() {
            text(d, DVec2::new(cols[k], y), 3.4, fit(&c, 3.4, widths[k]), true);
        }
    };
    row(&mut d, y, ["#".into(), tr!("cutlist-col-part"), tr!("cutlist-col-count"), tr!("cutlist-col-size"), tr!("cutlist-col-material")]);
    d.lines.push((DVec2::new(MARGIN, y + 1.5), DVec2::new(PAGE.x - MARGIN, y + 1.5), Pen::Edge));
    y += 6.5;
    let unit = model.unit;
    let size = |s: [f64; 3]| format!("{} × {} × {}", crate::ui::cutlist::dim(s[0], unit), crate::ui::cutlist::dim(s[1], unit), crate::ui::cutlist::dim(s[2], unit));
    let mut lines: Vec<[String; 5]> = plan.parts.iter().enumerate().map(|(i, p)| [(i + 1).to_string(), p.name.clone(), p.count.to_string(), size(p.size), p.material_name.clone()]).collect();
    if !plan.hardware.is_empty() {
        lines.push(Default::default());
        lines.push(["".into(), tr!("assembly-booklet-hardware"), "".into(), "".into(), "".into()]);
        lines.extend(plan.hardware.iter().map(|h| ["".into(), h.name.clone(), h.count.to_string(), "".into(), "".into()]));
    }
    for cells in lines {
        if y > PAGE.y - 25.0 {
            pages.push(std::mem::replace(&mut d, blank()));
            y = 25.0;
        }
        row(&mut d, y, cells);
        y += 5.5;
    }
    pages.push(d);

    // a page per step, arrows showing where its parts go
    let n = a.steps.len();
    for (k, step) in a.steps.iter().enumerate() {
        let mut m = model.clone();
        m.set_assembly_view(SceneAssembly { explode: view.explode, step: Some(k) });
        let faint: Vec<ContextPath> = a.steps[..k].iter().flat_map(|s| s.parts.iter().cloned()).collect();
        let lines = view_lines(&m, cam, px, &VectorOptions { faint, ..opts.clone() });
        let mut d = blank();
        text(&mut d, DVec2::new(MARGIN, 22.0), 7.0, tr!("assembly-booklet-step", n = k + 1, total = n), true);
        let mut d = picture(d, &lines, map);
        if let Some((off, scale)) = map {
            for &(t, h) in &step_arrows[k] {
                draw_arrow(&mut d, off + t * scale, off + h * scale);
            }
        }
        let mut y = rect.0.y + rect.1.y + 12.0;
        let block = |d: &mut Drawing, head: String, items: Vec<String>, y: &mut f64| {
            if items.is_empty() {
                return;
            }
            text(d, DVec2::new(MARGIN, *y), 4.0, head, true);
            *y += 6.0;
            for it in items {
                text(d, DVec2::new(MARGIN + 4.0, *y), 3.6, fit(&it, 3.6, PAGE.x - 2.0 * MARGIN - 4.0), true);
                *y += 5.0;
            }
            *y += 2.0;
        };
        block(&mut d, tr!("assembly-booklet-add"), step_parts(model, plan, step), &mut y);
        block(&mut d, tr!("assembly-booklet-hardware"), step_hardware(model, k), &mut y);
        for l in wrap(&step.note, 90) {
            if y > PAGE.y - 18.0 {
                break;
            }
            text(&mut d, DVec2::new(MARGIN, y), 3.6, l, true);
            y += 5.0;
        }
        pages.push(d);
    }
    let total = pages.len();
    for (i, d) in pages.iter_mut().enumerate() {
        text(d, DVec2::new(PAGE.x * 0.5, PAGE.y - 8.0), 2.6, tr!("assembly-booklet-footer", doc = doc.to_string(), n = i + 1, total = total), false);
    }
    pages
}

impl VarstotasApp {
    /// Show the model exploded by `v.explode` and up to step `v.step` (not an undo step,
    /// like hiding tags); a step shown gets its parts selected.
    pub(crate) fn set_assembly_view(&mut self, v: SceneAssembly) {
        if v == self.doc.model.assembly.view() {
            return;
        }
        self.doc.mutate_untracked(|m| m.set_assembly_view(v));
        if let Some(s) = v.step.and_then(|s| self.doc.model.assembly.steps.get(s)) {
            let parent = s.parts.first().map(|p| p[..p.len() - 1].to_vec()).unwrap_or_default();
            let sel: Vec<EntityRef> = s.parts.iter().filter(|p| p[..p.len() - 1] == parent[..]).filter_map(|p| p.last().map(|&id| EntityRef::Instance(id))).collect();
            self.doc.set_active_path(parent);
            self.doc.set_selection(sel);
        }
        self.doc.prune_selection();
    }

    /// Replace the steps with those the contact graph suggests.
    pub(crate) fn suggest_assembly(&mut self) {
        self.set_assembly_view(SceneAssembly::default());
        let steps = suggest_steps(&self.doc.model);
        self.doc.commit(&tr!("assembly-undo-steps"), move |m| m.assembly.steps = steps);
    }

    /// One scene per step ("Step N"), seen from the current view as exploded now; scenes of
    /// those names are replaced.
    pub(crate) fn assembly_scenes(&mut self) {
        let explode = self.doc.model.assembly.explode;
        let scenes: Vec<varstotas_kernel::Scene> = (0..self.doc.model.assembly.steps.len())
            .map(|k| {
                let mut s = self.scene_from_camera(tr!("assembly-scene-name", n = k + 1));
                s.assembly = Some(SceneAssembly { explode, step: Some(k) });
                s
            })
            .collect();
        self.doc.commit(&tr!("assembly-undo-scenes"), move |m| {
            for s in scenes {
                match m.scenes.iter_mut().find(|x| x.name == s.name) {
                    Some(x) => *x = s,
                    None => m.scenes.push(s),
                }
            }
        });
    }

    pub(crate) fn assembly_dump(&self) -> (usize, i64, Option<usize>, Vec<Vec<String>>) {
        let a = &self.doc.model.assembly;
        let plan = crate::ui::cutlist::model_plan(&self.doc.model);
        (a.steps.len(), a.explode.round() as i64, a.step, a.steps.iter().map(|s| step_parts(&self.doc.model, &plan, s)).collect())
    }

    /// Write the instructions booklet to `path`.
    pub(crate) fn export_assembly(&mut self, path: &Path) {
        let plan = crate::ui::cutlist::model_plan(&self.doc.model);
        let pages = assembly_pages(&self.doc.model, &plan, &self.camera, &self.doc.display_name());
        match varstotas_kernel::io::atomic_write(path, crate::vector_export::to_pdf_pages(&pages)) {
            Ok(()) => self.flash(tr!("cutlist-exported", path = path.display().to_string())),
            Err(e) => self.flash_error(tr!("cutlist-export-failed", error = e.to_string())),
        }
    }

    fn assembly_save(&mut self) {
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            // i18n-ignore-next
            .set_file_name(format!("{} assembly.pdf", self.doc.display_name()))
            .save_file()
        else {
            return;
        };
        if path.extension().is_none() {
            path.set_extension("pdf");
        }
        self.export_assembly(&path);
    }

    pub(crate) fn assembly_window(&mut self, ctx: &Context) {
        if !self.assembly.open {
            return;
        }
        // at the view's right edge, clear of the model in the middle
        let pos = self.viewport.view_rect.right_top() + vec2(-20.0, 60.0);
        let mut open = true;
        let win = self.placed(egui::Window::new(tr!("assembly-title")).id(egui::Id::new("assembly_window")).open(&mut open).collapsible(false).resizable(false).pivot(egui::Align2::RIGHT_TOP), pos);
        win.show(ctx, |ui| {
            ui.set_width(480.0);
            self.assembly_view_ui(ui);
            ui.separator();
            self.assembly_steps_ui(ui);
        });
        if !open {
            self.assembly.open = false;
            self.set_assembly_view(SceneAssembly::default());
        }
    }

    fn assembly_view_ui(&mut self, ui: &mut Ui) {
        let mut v = self.doc.model.assembly.view();
        let size = self.doc.model.bounds().map_or(1000.0, |b| (b.max - b.min).max_element());
        ui.horizontal(|ui| {
            ui.label(tr!("assembly-explode"));
            let r = ui.add(egui::Slider::new(&mut v.explode, 0.0..=(size * 0.6).max(50.0).round()).suffix(" mm").max_decimals(0));
            super::tray_lib::spot("assembly:explode", r.rect.center());
            if ui.button(tr!("assembly-assembled")).on_hover_text(tr!("assembly-assembled-tip")).clicked() {
                v = SceneAssembly::default();
            }
        });
        self.set_assembly_view(v);
    }

    fn assembly_steps_ui(&mut self, ui: &mut Ui) {
        let a = self.doc.model.assembly.clone();
        ui.horizontal(|ui| {
            let r = ui.button(RichText::new(tr!(if a.steps.is_empty() { "assembly-suggest" } else { "assembly-suggest-again" })).strong());
            super::tray_lib::spot("assembly:suggest", r.rect.center());
            if r.on_hover_text(tr!("assembly-suggest-tip")).clicked() {
                self.suggest_assembly();
            }
            ui.add_enabled_ui(!a.steps.is_empty(), |ui| {
                if ui.button(tr!("assembly-scenes")).on_hover_text(tr!("assembly-scenes-tip")).clicked() {
                    self.assembly_scenes();
                }
                let r = ui.button(RichText::new(tr!("assembly-booklet")).strong());
                if r.on_hover_text(tr!("assembly-booklet-tip")).clicked() {
                    self.assembly_save();
                }
            });
        });
        if a.steps.is_empty() {
            ui.label(RichText::new(tr!("assembly-none")).color(theme::text_dim()));
            return;
        }
        let plan = crate::ui::cutlist::model_plan(&self.doc.model);
        let mut show: Option<Option<usize>> = None;
        let mut edit: Option<(String, Vec<Step>)> = None;
        egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
            let r = ui.selectable_label(a.step.is_none(), tr!("assembly-all-steps"));
            if r.clicked() {
                show = Some(None);
            }
            for (k, step) in a.steps.iter().enumerate() {
                ui.push_id(("step", k), |ui| {
                    ui.horizontal(|ui| {
                        let r = ui.selectable_label(a.step == Some(k), RichText::new(tr!("assembly-step", n = k + 1)).strong());
                        super::tray_lib::spot(format!("assembly:step:{k}"), r.rect.center());
                        if r.on_hover_text(tr!("assembly-step-tip")).clicked() {
                            show = Some(Some(k));
                        }
                        ui.label(fit(&step_parts(&self.doc.model, &plan, step).join(", "), 9.0, 220.0));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let mut steps = a.steps.clone();
                            if step.parts.len() > 1 && ui.small_button("÷").on_hover_text(tr!("assembly-split-tip")).clicked() {
                                let parts = steps[k].parts.clone();
                                steps.splice(k..=k, parts.into_iter().map(|p| Step { parts: vec![p], note: String::new() }));
                                if let Some(s) = steps.get_mut(k) {
                                    s.note = step.note.clone();
                                }
                                edit = Some((tr!("assembly-undo-steps"), steps));
                            } else if k > 0 && ui.small_button("⤒").on_hover_text(tr!("assembly-join-tip")).clicked() {
                                let s = steps.remove(k);
                                steps[k - 1].parts.extend(s.parts);
                                if !s.note.is_empty() {
                                    let n = &mut steps[k - 1].note;
                                    *n = if n.is_empty() { s.note } else { format!("{n} {}", s.note) };
                                }
                                edit = Some((tr!("assembly-undo-steps"), steps));
                            } else if k + 1 < a.steps.len() && ui.small_button("▼").on_hover_text(tr!("assembly-down-tip")).clicked() {
                                steps.swap(k, k + 1);
                                edit = Some((tr!("assembly-undo-steps"), steps));
                            } else if k > 0 && ui.small_button("▲").on_hover_text(tr!("assembly-up-tip")).clicked() {
                                steps.swap(k, k - 1);
                                edit = Some((tr!("assembly-undo-steps"), steps));
                            }
                        });
                    });
                    let mut note = match &self.assembly.note {
                        Some((i, n)) if *i == k => n.clone(),
                        _ => step.note.clone(),
                    };
                    let r = ui.add(egui::TextEdit::singleline(&mut note).hint_text(tr!("assembly-note-hint")).desired_width(f32::INFINITY));
                    if r.changed() {
                        self.assembly.note = Some((k, note.clone()));
                    }
                    if r.lost_focus() {
                        self.assembly.note = None;
                        if note != step.note {
                            let mut steps = a.steps.clone();
                            steps[k].note = note;
                            edit = Some((tr!("assembly-undo-note"), steps));
                        }
                    }
                });
            }
        });
        if let Some(s) = show {
            self.set_assembly_view(SceneAssembly { step: s, ..self.doc.model.assembly.view() });
        }
        if let Some((label, steps)) = edit {
            let view = self.doc.model.assembly.view();
            self.set_assembly_view(SceneAssembly { step: None, ..view });
            self.doc.commit(&label, move |m| m.assembly.steps = steps);
        }
    }
}
