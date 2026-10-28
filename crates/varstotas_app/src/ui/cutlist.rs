//! Window → Cut List (track W): the parts of the model laid out on the stock of their
//! materials (`varstotas_kernel::cutlist`), with the settings, the shopping list, the parts
//! table and one cutting diagram per sheet (boards share a page). A click on a piece selects
//! that part in the model. Export: PDF (a page per diagram), SVG, CSV. The stock itself is set
//! in the Material Editor (see [`VarstotasApp::material_stock`]).

use crate::app::VarstotasApp;
use crate::document::guarded;
use crate::ui::theme;
use crate::vector_export::{Drawing, Pen, Text2};
use egui::{Color32, Context, RichText, Sense, Stroke, Ui, pos2, vec2};
use std::path::Path;
use std::rc::Rc;
use varstotas_i18n::tr;
use varstotas_kernel::cutlist::{CutPlan, CutSettings, Issue, PartGrain, PartOptions, Stock, StockKind, StockPlan, plan, plan_csv};
use varstotas_kernel::glam::DVec2;
use varstotas_kernel::{EntityRef, LengthUnit, MaterialId};

/// A4 landscape, mm.
const PAGE: DVec2 = DVec2::new(297.0, 210.0);
const MARGIN: f64 = 10.0;
/// Room for the title above the drawing, mm.
const HEADER: f64 = 12.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Diagrams,
    Parts,
}

#[derive(Default)]
pub struct CutListState {
    pub open: bool,
    pub tab: Tab,
    /// Page shown in the preview.
    pub page: usize,
    cache: Option<(u64, Rc<CutPlan>)>,
    /// Settings being dragged (committed when the pointer is released).
    draft: Option<CutSettings>,
    /// Stock being edited in the Material Editor (committed when the pointer is released or a
    /// text field loses focus).
    pub(crate) stock_draft: Option<(MaterialId, Vec<Stock>)>,
}

/// A diagram page: one sheet, or several boards of one stock item.
struct Page {
    plan: usize,
    layouts: std::ops::Range<usize>,
}

/// A piece on a page (paper mm, y down) and its part.
struct Hit {
    min: DVec2,
    max: DVec2,
    part: usize,
}

/// A length for labels: the bare number in millimetres, else with the unit.
pub(crate) fn dim(mm: f64, unit: LengthUnit) -> String {
    match unit {
        LengthUnit::Millimeter => {
            let s = format!("{mm:.1}");
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        }
        _ => varstotas_kernel::units::format_length(mm, unit),
    }
}

fn stock_title(sp: &StockPlan, unit: LengthUnit) -> String {
    let st = &sp.stock;
    let name = if sp.material_name.is_empty() { tr!("cutlist-no-material") } else { sp.material_name.clone() };
    match st.kind {
        StockKind::Sheet => tr!("cutlist-stock-sheet", material = name, length = dim(st.length, unit), width = dim(st.width, unit), thickness = dim(st.thickness, unit)),
        StockKind::Board => tr!("cutlist-stock-board", material = name, width = dim(st.width, unit), thickness = dim(st.thickness, unit)),
    }
}

fn pages(plan: &CutPlan) -> Vec<Page> {
    let mut out = Vec::new();
    for (pi, sp) in plan.plans.iter().enumerate() {
        match sp.stock.kind {
            StockKind::Sheet => out.extend((0..sp.layouts.len()).map(|i| Page { plan: pi, layouts: i..i + 1 })),
            StockKind::Board => {
                // boards one under another at a common scale, as many as fit the page
                let (_, row) = board_scale(sp);
                let per = (((PAGE.y - 2.0 * MARGIN - HEADER) / row).floor() as usize).max(1);
                let mut i = 0;
                while i < sp.layouts.len() {
                    let end = (i + per).min(sp.layouts.len());
                    out.push(Page { plan: pi, layouts: i..end });
                    i = end;
                }
            }
        }
    }
    out
}

/// Boards: paper mm per stock mm and the height of one board row on paper.
fn board_scale(sp: &StockPlan) -> (f64, f64) {
    let longest = sp.layouts.iter().map(|l| l.length).fold(1.0, f64::max);
    let k = (PAGE.x - 2.0 * MARGIN) / longest;
    (k, sp.stock.width * k + 9.0)
}

fn rect_lines(d: &mut Drawing, a: DVec2, b: DVec2, pen: Pen) {
    let (c, e) = (DVec2::new(b.x, a.y), DVec2::new(a.x, b.y));
    d.lines.extend([(a, c, pen), (c, b, pen), (b, e, pen), (e, a, pen)]);
}

fn text(d: &mut Drawing, at: DVec2, size: f64, s: String, left: bool) {
    d.texts.push(Text2 { at, angle: 0.0, size, lines: vec![s], left });
}

/// The diagram of a page and where its pieces are.
fn page_drawing(plan: &CutPlan, page: &Page, total: usize, n: usize, unit: LengthUnit, footer: &str) -> (Drawing, Vec<Hit>) {
    let sp = &plan.plans[page.plan];
    let mut d = Drawing { size: PAGE, ..Default::default() };
    let mut hits = Vec::new();
    let title = match sp.stock.kind {
        StockKind::Sheet => tr!("cutlist-page-sheet", stock = stock_title(sp, unit), n = page.layouts.start + 1, total = sp.layouts.len()),
        StockKind::Board => tr!("cutlist-page-boards", stock = stock_title(sp, unit), from = page.layouts.start + 1, to = page.layouts.end, total = sp.layouts.len()),
    };
    text(&mut d, DVec2::new(MARGIN, MARGIN + 4.0), 4.0, title, true);
    // the legend: what the numbers on this page are
    let mut used: Vec<usize> = page.layouts.clone().flat_map(|li| sp.layouts[li].pieces.iter().map(|p| p.part)).collect();
    used.sort_unstable();
    used.dedup();
    let legend = used.iter().filter_map(|&i| plan.parts.get(i).map(|p| format!("{} {}", i + 1, p.name))).collect::<Vec<_>>().join(" · ");
    text(&mut d, DVec2::new(MARGIN, MARGIN + 9.0), 2.8, legend, true);
    text(&mut d, DVec2::new(MARGIN, PAGE.y - MARGIN * 0.4), 2.5, tr!("cutlist-footer", doc = footer.to_string(), n = n, total = total), true);
    let area_h = PAGE.y - 2.0 * MARGIN - HEADER;
    let (k, row) = match sp.stock.kind {
        StockKind::Sheet => {
            let l = &sp.layouts[page.layouts.start];
            (((PAGE.x - 2.0 * MARGIN) / l.length).min(area_h / l.width), 0.0)
        }
        StockKind::Board => board_scale(sp),
    };
    for (row_i, li) in page.layouts.clone().enumerate() {
        let l = &sp.layouts[li];
        let o = DVec2::new(MARGIN, MARGIN + HEADER + row_i as f64 * row);
        let o = if sp.stock.kind == StockKind::Board {
            text(&mut d, o + DVec2::new(0.0, 3.0), 2.5, tr!("cutlist-board-label", n = li + 1, length = dim(l.length, unit)), true);
            o + DVec2::new(0.0, 4.5)
        } else {
            o
        };
        // stock x → right, stock y → up on paper
        let at = |x: f64, y: f64| DVec2::new(o.x + x * k, o.y + (l.width - y) * k);
        for p in &l.pieces {
            let (a, b) = (at(p.x, p.y + p.h), at(p.x + p.w, p.y));
            d.fills.push(vec![a, DVec2::new(b.x, a.y), b, DVec2::new(a.x, b.y)]);
            rect_lines(&mut d, a, b, Pen::Edge);
            hits.push(Hit { min: a, max: b, part: p.part });
            let (w, h) = (b.x - a.x, b.y - a.y);
            let size = (h * 0.28).min(3.2).min(w / 6.0);
            if size >= 1.0 {
                let c = (a + b) * 0.5;
                let (len, wid) = if p.rotated { (p.h, p.w) } else { (p.w, p.h) };
                let label = format!("{}  {} × {}", p.part + 1, dim(len, unit), dim(wid, unit));
                let size = size.min(w / (label.chars().count() as f64 * 0.55));
                text(&mut d, c + DVec2::new(0.0, size * 0.35), size.max(1.0), label, false);
            }
        }
        for off in &l.offcuts {
            let (a, b) = (at(off[0], off[1] + off[3]), at(off[0] + off[2], off[1]));
            rect_lines(&mut d, a, b, Pen::Hidden);
            let size = ((b.y - a.y) * 0.22).min(2.5).min((b.x - a.x) / 8.0);
            if size >= 1.0 {
                text(&mut d, (a + b) * 0.5 + DVec2::new(0.0, size * 0.35), size, format!("{} × {}", dim(off[2], unit), dim(off[3], unit)), false);
            }
        }
        rect_lines(&mut d, at(0.0, l.width), at(l.length, 0.0), Pen::Profile);
    }
    (d, hits)
}

impl VarstotasApp {
    /// The cut list of the model, cached until it changes.
    pub(crate) fn cut_plan(&mut self) -> Rc<CutPlan> {
        if let Some((v, p)) = &self.cutlist.cache
            && *v == self.doc.model_version
        {
            return p.clone();
        }
        let p = Rc::new(guarded(|| plan(&self.doc.model)).unwrap_or_default());
        self.cutlist.cache = Some((self.doc.model_version, p.clone()));
        p
    }

    fn cut_drawings(&mut self) -> Vec<(Drawing, Vec<Hit>)> {
        let plan = self.cut_plan();
        let pages = pages(&plan);
        let unit = self.doc.model.unit;
        let name = self.doc.display_name();
        pages.iter().enumerate().map(|(i, p)| page_drawing(&plan, p, pages.len(), i + 1, unit, &name)).collect()
    }

    /// Write the cut list to `path`: PDF (a page per diagram), SVG (the pages one under
    /// another) or CSV (parts and the stock to buy).
    pub(crate) fn export_cut_list(&mut self, path: &Path) {
        let plan = self.cut_plan();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase();
        let bytes = match ext.as_str() {
            "csv" => plan_csv(&self.doc.model, &plan).into_bytes(),
            "pdf" => crate::vector_export::to_pdf_pages(&self.cut_drawings().into_iter().map(|d| d.0).collect::<Vec<_>>()),
            _ => {
                let mut out = Drawing::default();
                for (d, _) in self.cut_drawings() {
                    let dy = if out.size.y > 0.0 { out.size.y + MARGIN } else { 0.0 };
                    let off = DVec2::new(0.0, dy);
                    out.lines.extend(d.lines.into_iter().map(|(a, b, p)| (a + off, b + off, p)));
                    out.fills.extend(d.fills.into_iter().map(|f| f.into_iter().map(|p| p + off).collect()));
                    out.texts.extend(d.texts.into_iter().map(|mut t| {
                        t.at += off;
                        t
                    }));
                    out.size = DVec2::new(out.size.x.max(d.size.x), dy + d.size.y);
                }
                crate::vector_export::to_svg(&out).into_bytes()
            }
        };
        match varstotas_kernel::io::atomic_write(path, bytes) {
            Ok(()) => self.flash(tr!("cutlist-exported", path = path.display().to_string())),
            Err(e) => self.flash_error(tr!("cutlist-export-failed", error = e.to_string())),
        }
    }

    fn cut_list_save(&mut self, ext: &str) {
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter(ext.to_uppercase(), &[ext])
            // i18n-ignore-next
            .set_file_name(format!("{} cut list.{ext}", self.doc.display_name()))
            .save_file()
        else {
            return;
        };
        if path.extension().is_none() {
            path.set_extension(ext);
        }
        self.export_cut_list(&path);
    }

    /// `(open, parts, stock items, sheets and boards, issues, cost)` for UI test scripts.
    pub(crate) fn cutlist_dump(&self) -> (bool, usize, usize, usize, usize, String) {
        let p = match &self.cutlist.cache {
            Some((v, p)) if *v == self.doc.model_version => p.clone(),
            _ => Rc::new(guarded(|| plan(&self.doc.model)).unwrap_or_default()),
        };
        (self.cutlist.open, p.parts.len(), p.plans.len(), p.plans.iter().map(|s| s.layouts.len()).sum(), p.issues.len(), format!("{:.2}", p.cost()))
    }

    /// Select the copies of part `i` (those in the same group as its first copy).
    pub(crate) fn select_cut_part(&mut self, i: usize) {
        let plan = self.cut_plan();
        let Some(part) = plan.parts.get(i) else { return };
        let Some(first) = part.paths.first() else { return };
        let parent = first[..first.len() - 1].to_vec();
        let copies: Vec<EntityRef> =
            part.paths.iter().filter(|p| p[..p.len() - 1] == parent[..]).filter_map(|p| p.last().map(|&id| EntityRef::Instance(id))).collect();
        self.doc.set_active_path(parent);
        self.doc.set_selection(copies);
    }

    fn set_part_options(&mut self, name: &str, o: PartOptions) {
        let name = name.to_string();
        self.doc.commit(&tr!("cutlist-undo-part"), move |m| {
            if o == PartOptions::default() {
                m.cut_list.parts.remove(&name);
            } else {
                m.cut_list.parts.insert(name, o);
            }
        });
    }

    fn cut_settings_ui(&mut self, ui: &mut Ui) {
        let unit = self.doc.model.unit;
        let k = unit.mm();
        let mut s = self.cutlist.draft.clone().unwrap_or_else(|| self.doc.model.cut_list.clone());
        let mut changed = false;
        ui.horizontal(|ui| {
            for (label, v, max) in [
                (tr!("cutlist-kerf"), &mut s.kerf, 20.0),
                (tr!("cutlist-trim"), &mut s.trim, 100.0),
                (tr!("cutlist-min-offcut"), &mut s.min_offcut, 2000.0),
            ] {
                ui.label(label);
                let mut shown = *v / k;
                let r = ui.add(egui::DragValue::new(&mut shown).range(0.0..=max / k).speed(0.1 / k).max_decimals(3).suffix(format!(" {}", unit.suffix())));
                if r.changed() {
                    *v = shown * k;
                    changed = true;
                }
                ui.add_space(8.0);
            }
        });
        if changed {
            self.cutlist.draft = Some(s);
        }
        let released = !ui.ctx().input(|i| i.pointer.any_down()) && !ui.ctx().memory(|m| m.focused().is_some());
        if released
            && let Some(s) = self.cutlist.draft.take()
            && s != self.doc.model.cut_list
        {
            self.doc.commit(&tr!("cutlist-undo-settings"), |m| m.cut_list = s);
        }
    }

    fn cut_summary_ui(&mut self, ui: &mut Ui, plan: &CutPlan) {
        let unit = self.doc.model.unit;
        if plan.plans.is_empty() {
            ui.label(RichText::new(varstotas_i18n::tr(if plan.parts.is_empty() { "cutlist-no-parts" } else { "cutlist-no-stock-at-all" })).color(theme::text_dim()));
        }
        for sp in &plan.plans {
            let buy: Vec<String> = match sp.stock.kind {
                StockKind::Sheet => vec![tr!("cutlist-buy-sheets", n = sp.layouts.len())],
                StockKind::Board => sp.to_buy().into_iter().map(|(len, n)| tr!("cutlist-buy-boards", n = n, length = dim(len, unit))).collect(),
            };
            let mut line = format!("{}: {}", stock_title(sp, unit), buy.join(", "));
            line.push_str(&tr!("cutlist-waste", percent = format!("{:.0}", sp.waste() * 100.0)));
            if sp.stock.price > 0.0 {
                line.push_str(&tr!("cutlist-cost", cost = format!("{:.2}", sp.cost())));
            }
            ui.label(line);
        }
        if plan.plans.len() > 1 && plan.cost() > 0.0 {
            ui.label(RichText::new(tr!("cutlist-total", cost = format!("{:.2}", plan.cost()))).strong());
        }
        for is in &plan.issues {
            let (part, msg) = match *is {
                Issue::NoStock { part } => {
                    let p = &plan.parts[part];
                    (part, if p.material.is_some() { tr!("cutlist-issue-no-stock", material = p.material_name.clone()) } else { tr!("cutlist-issue-no-material") })
                }
                Issue::TooBig { part } => (part, tr!("cutlist-issue-too-big")),
                Issue::Thickness { part, stock } => {
                    (part, tr!("cutlist-issue-thickness", part_thickness = dim(plan.parts[part].size[2], unit), stock = dim(stock, unit)))
                }
            };
            let p = &plan.parts[part];
            let r = ui.add(egui::Label::new(RichText::new(format!("⚠ {} {}: {msg}", part + 1, p.name)).color(theme::warning())).sense(Sense::click()));
            if r.on_hover_text(tr!("cutlist-click-select")).clicked() {
                self.select_cut_part(part);
            }
        }
    }

    fn cut_parts_ui(&mut self, ui: &mut Ui, plan: &CutPlan) {
        let unit = self.doc.model.unit;
        let mut set: Option<(String, PartOptions)> = None;
        let mut select = None;
        egui::ScrollArea::vertical().max_height(360.0).auto_shrink([false, true]).show(ui, |ui| {
            egui::Grid::new("cutlist_parts").num_columns(7).striped(true).spacing([10.0, 4.0]).show(ui, |ui| {
                for h in ["cutlist-col-part", "cutlist-col-count", "cutlist-col-size", "cutlist-col-material", "cutlist-col-grain", "cutlist-col-exclude"] {
                    ui.label(RichText::new(varstotas_i18n::tr(h)).strong());
                }
                ui.end_row();
                for (i, p) in plan.parts.iter().enumerate() {
                    let o = self.doc.model.cut_list.options(&p.name);
                    if ui.add(egui::Label::new(format!("{}  {}", i + 1, p.name)).sense(Sense::click())).on_hover_text(tr!("cutlist-click-select")).clicked() {
                        select = Some(i);
                    }
                    ui.label(p.count.to_string());
                    ui.label(format!("{} × {} × {}", dim(p.size[0], unit), dim(p.size[1], unit), dim(p.size[2], unit)));
                    ui.label(if p.material_name.is_empty() { tr!("cutlist-no-material") } else { p.material_name.clone() });
                    let mut g = o.grain;
                    egui::ComboBox::from_id_salt(("cutlist_grain", i)).selected_text(grain_name(g)).show_ui(ui, |ui| {
                        for v in [PartGrain::Length, PartGrain::Width, PartGrain::Any] {
                            ui.selectable_value(&mut g, v, grain_name(v));
                        }
                    });
                    let mut ex = o.exclude;
                    ui.checkbox(&mut ex, "");
                    if g != o.grain || ex != o.exclude {
                        set = Some((p.name.clone(), PartOptions { grain: g, exclude: ex }));
                    }
                    ui.end_row();
                }
            });
        });
        ui.label(RichText::new(tr!("cutlist-parts-note")).small().color(theme::text_dim()));
        if let Some((name, o)) = set {
            self.set_part_options(&name, o);
        }
        if let Some(i) = select {
            self.select_cut_part(i);
        }
    }

    fn cut_diagrams_ui(&mut self, ui: &mut Ui) {
        let drawings = self.cut_drawings();
        let s = &mut self.cutlist;
        s.page = s.page.min(drawings.len().saturating_sub(1));
        let (rect, resp) = ui.allocate_exact_size(vec2(640.0, 452.0), Sense::click());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, theme::border());
        let mut clicked = None;
        if let Some((d, hits)) = drawings.get(s.page) {
            let k = ((rect.width() as f64 - 16.0) / d.size.x).min((rect.height() as f64 - 16.0) / d.size.y);
            let o = rect.center() - vec2((d.size.x * k) as f32, (d.size.y * k) as f32) * 0.5;
            let sp = |p: DVec2| pos2(o.x + (p.x * k) as f32, o.y + (p.y * k) as f32);
            painter.rect_filled(egui::Rect::from_min_max(sp(DVec2::ZERO), sp(d.size)), 0.0, Color32::WHITE);
            let hover = resp.hover_pos();
            for h in hits {
                let r = egui::Rect::from_min_max(sp(h.min), sp(h.max));
                let on = hover.is_some_and(|p| r.contains(p));
                painter.rect_filled(r, 0.0, if on { theme::accent_soft() } else { Color32::from_rgb(0xE6, 0xDA, 0xC4) });
                if on && resp.clicked() {
                    clicked = Some(h.part);
                }
            }
            for (a, b, pen) in &d.lines {
                let (a, b) = (sp(*a), sp(*b));
                match pen {
                    Pen::Hidden => painter.extend(egui::Shape::dashed_line(&[a, b], Stroke::new(1.0, Color32::from_gray(140)), 4.0, 3.0)),
                    Pen::Profile => {
                        painter.line_segment([a, b], Stroke::new(1.6, Color32::BLACK));
                    }
                    _ => {
                        painter.line_segment([a, b], Stroke::new(1.0, Color32::from_gray(40)));
                    }
                }
            }
            for t in &d.texts {
                let px = (t.size * k) as f32;
                if px < 6.0 {
                    continue;
                }
                let galley = painter.layout_no_wrap(t.lines.join(" "), egui::FontId::proportional(px), Color32::from_gray(40));
                let size = galley.size();
                let along = if t.left { 0.0 } else { size.x * 0.5 };
                painter.galley(sp(t.at) - vec2(along, size.y * 0.8), galley, Color32::from_gray(40));
            }
            if hits.iter().any(|h| hover.is_some_and(|p| egui::Rect::from_min_max(sp(h.min), sp(h.max)).contains(p))) {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
        }
        if drawings.len() > 1 {
            ui.horizontal(|ui| {
                if ui.add_enabled(s.page > 0, egui::Button::new("◀")).clicked() {
                    s.page -= 1;
                }
                ui.label(tr!("cutlist-page-of", n = s.page + 1, total = drawings.len()));
                if ui.add_enabled(s.page + 1 < drawings.len(), egui::Button::new("▶")).clicked() {
                    s.page += 1;
                }
            });
        }
        if let Some(i) = clicked {
            self.select_cut_part(i);
        }
    }

    /// The Cut List window (not modal: the model can change while it is open).
    pub(crate) fn cut_list_window(&mut self, ctx: &Context) {
        if !self.cutlist.open {
            return;
        }
        let plan = self.cut_plan();
        let pos = self.viewport.view_rect.min + vec2(40.0, 40.0);
        let mut open = true;
        let mut save: Option<&str> = None;
        let win = self.placed(egui::Window::new(tr!("cutlist-title")).id(egui::Id::new("cutlist_window")).open(&mut open).collapsible(false).resizable(false), pos);
        win.show(ctx, |ui| {
            ui.set_width(640.0);
            self.cut_settings_ui(ui);
            ui.add_space(4.0);
            self.cut_summary_ui(ui, &plan);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.cutlist.tab, Tab::Diagrams, tr!("cutlist-tab-diagrams"));
                ui.selectable_value(&mut self.cutlist.tab, Tab::Parts, tr!("cutlist-tab-parts", n = plan.parts.len()));
            });
            ui.separator();
            match self.cutlist.tab {
                Tab::Diagrams => self.cut_diagrams_ui(ui),
                Tab::Parts => self.cut_parts_ui(ui, &plan),
            }
            ui.add_space(6.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_enabled_ui(!plan.parts.is_empty(), |ui| {
                    for (ext, need_plans) in [("csv", false), ("svg", true), ("pdf", true)] {
                        let on = !need_plans || !plan.plans.is_empty();
                        if ui.add_enabled(on, egui::Button::new(RichText::new(format!("{}…", ext.to_uppercase())).strong())).clicked() {
                            save = Some(ext);
                        }
                    }
                });
                ui.label(RichText::new(tr!("cutlist-export")).color(theme::text_dim()));
            });
        });
        if !open {
            self.cutlist.open = false;
        }
        if let Some(ext) = save {
            self.cut_list_save(ext);
        }
    }

    /// The Material Editor's stock section: what the material is bought as.
    pub(crate) fn material_stock(&mut self, ui: &mut Ui) {
        let Some(id) = self.current_material_id() else {
            ui.label(RichText::new(tr!("mated-stock-not-in-model")).small().color(theme::text_dim()));
            return;
        };
        let unit = self.doc.model.unit;
        let k = unit.mm();
        let mut list = match &self.cutlist.stock_draft {
            Some((m, l)) if *m == id => l.clone(),
            _ => self.doc.model.materials[id].stock.clone(),
        };
        let mut changed = false;
        let mut remove = None;
        let len = |ui: &mut Ui, v: &mut f64, changed: &mut bool| {
            let mut shown = *v / k;
            if ui.add(egui::DragValue::new(&mut shown).range(0.0..=100_000.0 / k).speed(1.0 / k).max_decimals(3).suffix(format!(" {}", unit.suffix()))).changed() {
                *v = shown * k;
                *changed = true;
            }
        };
        for (i, st) in list.iter_mut().enumerate() {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                egui::Grid::new(("mated_stock", i)).num_columns(2).spacing([10.0, 4.0]).show(ui, |ui| {
                    ui.label(tr!("mated-stock-kind"));
                    ui.horizontal(|ui| {
                        let before = st.kind;
                        ui.selectable_value(&mut st.kind, StockKind::Sheet, tr!("mated-stock-sheet"));
                        ui.selectable_value(&mut st.kind, StockKind::Board, tr!("mated-stock-board"));
                        if st.kind != before {
                            if st.kind == StockKind::Board && st.lengths.is_empty() {
                                st.lengths = vec![st.length.max(1000.0)];
                            }
                            changed = true;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("✕").on_hover_text(tr!("mated-stock-remove")).clicked() {
                                remove = Some(i);
                            }
                        });
                    });
                    ui.end_row();
                    ui.label(tr!("mated-stock-thickness"));
                    len(ui, &mut st.thickness, &mut changed);
                    ui.end_row();
                    match st.kind {
                        StockKind::Sheet => {
                            ui.label(tr!("mated-stock-size"));
                            ui.horizontal(|ui| {
                                len(ui, &mut st.length, &mut changed);
                                ui.label("×");
                                len(ui, &mut st.width, &mut changed);
                            });
                            ui.end_row();
                            ui.label("");
                            changed |= ui.checkbox(&mut st.grain, tr!("mated-stock-grain")).changed();
                            ui.end_row();
                        }
                        StockKind::Board => {
                            ui.label(tr!("mated-stock-width"));
                            len(ui, &mut st.width, &mut changed);
                            ui.end_row();
                            ui.label(tr!("mated-stock-lengths"));
                            let id = egui::Id::new(("mated_stock_lengths", i));
                            let mut s = ui.data(|d| d.get_temp::<String>(id)).unwrap_or_else(|| {
                                st.board_lengths().iter().map(|v| dim(*v, unit)).collect::<Vec<_>>().join(", ")
                            });
                            let r = ui.add(egui::TextEdit::singleline(&mut s).desired_width(180.0));
                            if r.changed() {
                                ui.data_mut(|d| d.insert_temp(id, s.clone()));
                            }
                            if r.lost_focus() {
                                ui.data_mut(|d| d.remove::<String>(id));
                                if let Some(v) = varstotas_kernel::units::parse_lengths(&s.replace(';', ","), unit) {
                                    let v: Vec<f64> = v.into_iter().filter(|x| *x > 0.0).collect();
                                    if !v.is_empty() && v != st.lengths {
                                        st.length = v.iter().copied().fold(0.0, f64::max);
                                        st.lengths = v;
                                        changed = true;
                                    }
                                }
                            }
                            ui.end_row();
                        }
                    }
                    ui.label(varstotas_i18n::tr(if st.kind == StockKind::Sheet { "mated-stock-price-sheet" } else { "mated-stock-price-metre" }));
                    if ui.add(egui::DragValue::new(&mut st.price).range(0.0..=1e7).speed(0.1).max_decimals(2)).changed() {
                        changed = true;
                    }
                    ui.end_row();
                });
            });
        }
        if let Some(i) = remove {
            list.remove(i);
            changed = true;
        }
        ui.menu_button(tr!("mated-stock-add"), |ui| {
            for (key, st) in stock_presets() {
                if ui.button(varstotas_i18n::tr(key)).clicked() {
                    list.push(st);
                    changed = true;
                    ui.close();
                }
            }
        });
        ui.label(RichText::new(tr!("mated-stock-note")).small().color(theme::text_dim()));
        if changed {
            self.cutlist.stock_draft = Some((id, list));
        }
        let released = !ui.ctx().input(|i| i.pointer.any_down()) && !ui.ctx().memory(|m| m.focused().is_some());
        if released
            && let Some((m, l)) = self.cutlist.stock_draft.take()
            && self.doc.model.materials.get(m).is_some_and(|x| x.stock != l)
        {
            self.doc.commit(&tr!("mated-undo-stock"), move |model| model.materials[m].stock = l);
        }
    }
}

fn grain_name(g: PartGrain) -> String {
    varstotas_i18n::tr(match g {
        PartGrain::Length => "cutlist-grain-length",
        PartGrain::Width => "cutlist-grain-width",
        PartGrain::Any => "cutlist-grain-any",
    })
}

/// Common stock, metric sizes.
fn stock_presets() -> Vec<(&'static str, Stock)> {
    vec![
        ("mated-stock-preset-ply-18", Stock::sheet(2440.0, 1220.0, 18.0, true)),
        ("mated-stock-preset-ply-12", Stock::sheet(2440.0, 1220.0, 12.0, true)),
        ("mated-stock-preset-birch-18", Stock::sheet(1525.0, 1525.0, 18.0, true)),
        ("mated-stock-preset-mdf-16", Stock::sheet(2800.0, 2070.0, 16.0, false)),
        ("mated-stock-preset-chipboard-16", Stock::sheet(2750.0, 1830.0, 16.0, false)),
        ("mated-stock-preset-hdf-3", Stock::sheet(2440.0, 1220.0, 3.0, false)),
        ("mated-stock-preset-pine-95x45", Stock::board(95.0, 45.0, &[2400.0, 3000.0, 3600.0, 4200.0, 4800.0, 6000.0])),
        ("mated-stock-preset-board-145x20", Stock::board(145.0, 20.0, &[2000.0, 2500.0, 3000.0])),
    ]
}
