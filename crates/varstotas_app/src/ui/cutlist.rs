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
use varstotas_kernel::cutlist::{CutPlan, CutSettings, Issue, Offcut, PartGrain, PartOptions, Stock, StockKind, StockPlan, plan_csv, plan_with};
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::layout::SheetCutDiagram;
use varstotas_kernel::{EntityRef, LengthUnit, MaterialId, Model};

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
    /// The plan and what it was made from: (model version, offcuts revision).
    cache: Option<((u64, u64), Rc<CutPlan>)>,
    /// Offcuts were kept at this model version: the plan shown stays as it was cut.
    kept: Option<u64>,
    /// The part numbers are not shown on the model.
    pub hide_numbers: bool,
    /// Where the part numbers go on the model, at this model version and plan.
    numbers: Option<(u64, Vec<(DVec3, usize)>)>,
    /// Settings being dragged (committed when the pointer is released).
    draft: Option<CutSettings>,
    /// Stock being edited in the Material Editor (committed when the pointer is released or a
    /// text field loses focus).
    pub(crate) stock_draft: Option<(MaterialId, Vec<Stock>)>,
}

/// The offcuts the cut list plans with: `Prefs::offcuts`, those of other projects while "Use my
/// offcuts" is on (the rest with no count, so the indices stay), and a revision for caches.
static OFFCUTS: std::sync::Mutex<(u64, Vec<Offcut>)> = std::sync::Mutex::new((0, Vec::new()));

fn offcuts_rev() -> u64 {
    OFFCUTS.lock().map(|o| o.0).unwrap_or(0)
}

/// The cut list of `model`, offcuts on hand first.
pub(crate) fn model_plan(model: &Model) -> CutPlan {
    let offcuts = OFFCUTS.lock().map(|o| o.1.clone()).unwrap_or_default();
    guarded(|| plan_with(model, &offcuts)).unwrap_or_default()
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
    let title = match st.kind {
        StockKind::Sheet => tr!("cutlist-stock-sheet", material = name, length = dim(st.length, unit), width = dim(st.width, unit), thickness = dim(st.thickness, unit)),
        StockKind::Board => tr!("cutlist-stock-board", material = name, width = dim(st.width, unit), thickness = dim(st.thickness, unit)),
        StockKind::Linear => tr!("cutlist-stock-linear", material = name, profile = stock_label(st, unit)),
    };
    if sp.offcut.is_some() { tr!("cutlist-stock-offcut", stock = title) } else { title }
}

fn pages(plan: &CutPlan) -> Vec<Page> {
    let mut out = Vec::new();
    for (pi, sp) in plan.plans.iter().enumerate() {
        match sp.stock.kind {
            StockKind::Sheet => out.extend((0..sp.layouts.len()).map(|i| Page { plan: pi, layouts: i..i + 1 })),
            StockKind::Board | StockKind::Linear => {
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
    let width = sp.layouts.iter().map(|l| l.width).fold(0.0, f64::max);
    (k, width * k + 9.0)
}

fn rect_lines(d: &mut Drawing, a: DVec2, b: DVec2, pen: Pen) {
    let (c, e) = (DVec2::new(b.x, a.y), DVec2::new(a.x, b.y));
    d.lines.extend([(a, c, pen), (c, b, pen), (b, e, pen), (e, a, pen)]);
}

fn text(d: &mut Drawing, at: DVec2, size: f64, s: String, left: bool) {
    d.texts.push(Text2 { at, angle: 0.0, size, lines: vec![s], left });
}

fn page_title(plan: &CutPlan, page: &Page, unit: LengthUnit) -> String {
    let sp = &plan.plans[page.plan];
    match sp.stock.kind {
        StockKind::Sheet => tr!("cutlist-page-sheet", stock = stock_title(sp, unit), n = page.layouts.start + 1, total = sp.layouts.len()),
        StockKind::Board => tr!("cutlist-page-boards", stock = stock_title(sp, unit), from = page.layouts.start + 1, to = page.layouts.end, total = sp.layouts.len()),
        StockKind::Linear => tr!("cutlist-page-lengths", stock = stock_title(sp, unit), from = page.layouts.start + 1, to = page.layouts.end, total = sp.layouts.len()),
    }
}

/// The titles of the cut list's pages (for picking one on a sheet).
pub(crate) fn cut_page_titles(plan: &CutPlan, unit: LengthUnit) -> Vec<String> {
    pages(plan).iter().map(|p| page_title(plan, p, unit)).collect()
}

/// Cut-list page `c.page` fitted into its rectangle on a sheet (track D), without the footer;
/// a page that no longer exists leaves a note.
pub(crate) fn cut_diagram_on_sheet(model: &Model, plan: &CutPlan, c: &SheetCutDiagram, d: &mut Drawing) {
    let pages = pages(plan);
    let Some(page) = pages.get(c.page) else {
        d.texts.push(Text2 { at: c.min + DVec2::new(2.0, 5.0), angle: 0.0, size: 3.0, lines: vec![tr!("cutlist-sheet-no-page")], left: true });
        rect_lines(d, c.min, c.min + c.size, Pen::Hidden);
        return;
    };
    let (src, _) = page_drawing(plan, &model.cut_list, page, pages.len(), c.page + 1, model.unit, None);
    // the page's margins are not drawn: fit its content box (A4 less the margins)
    let content = PAGE - DVec2::splat(2.0 * MARGIN);
    let k = (c.size.x / content.x).min(c.size.y / content.y);
    let o = c.min + (c.size - content * k) * 0.5 - DVec2::splat(MARGIN) * k;
    let at = |p: DVec2| o + p * k;
    d.lines.extend(src.lines.iter().map(|(a, b, pen)| (at(*a), at(*b), *pen)));
    d.fills.extend(src.fills.iter().map(|f| f.iter().map(|p| at(*p)).collect()));
    d.texts.extend(src.texts.into_iter().map(|t| Text2 { at: at(t.at), size: t.size * k, ..t }));
}

/// The diagram of a page and where its pieces are.
fn page_drawing(plan: &CutPlan, settings: &CutSettings, page: &Page, total: usize, n: usize, unit: LengthUnit, footer: Option<&str>) -> (Drawing, Vec<Hit>) {
    let sp = &plan.plans[page.plan];
    let mut d = Drawing { size: PAGE, ..Default::default() };
    let mut hits = Vec::new();
    text(&mut d, DVec2::new(MARGIN, MARGIN + 4.0), 4.0, page_title(plan, page, unit), true);
    // the legend: what the numbers on this page are
    let mut used: Vec<usize> = page.layouts.clone().flat_map(|li| sp.layouts[li].pieces.iter().map(|p| p.part)).collect();
    used.sort_unstable();
    used.dedup();
    let legend = used.iter().filter_map(|&i| plan.parts.get(i).map(|p| format!("{} {}", i + 1, p.name))).collect::<Vec<_>>().join(" · ");
    text(&mut d, DVec2::new(MARGIN, MARGIN + 9.0), 2.8, legend, true);
    if let Some(footer) = footer {
        text(&mut d, DVec2::new(MARGIN, PAGE.y - MARGIN * 0.4), 2.5, tr!("cutlist-footer", doc = footer.to_string(), n = n, total = total), true);
    }
    let area_h = PAGE.y - 2.0 * MARGIN - HEADER;
    let (k, row) = match sp.stock.kind {
        StockKind::Sheet => {
            let l = &sp.layouts[page.layouts.start];
            (((PAGE.x - 2.0 * MARGIN) / l.length).min(area_h / l.width), 0.0)
        }
        StockKind::Board | StockKind::Linear => board_scale(sp),
    };
    for (row_i, li) in page.layouts.clone().enumerate() {
        let l = &sp.layouts[li];
        let o = DVec2::new(MARGIN, MARGIN + HEADER + row_i as f64 * row);
        let o = if sp.stock.is_lengths() {
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
            // banded edges: a heavy line just inside the piece; the long edges run along x
            // unless the part is turned
            if let Some(name) = plan.parts.get(p.part).map(|x| &x.name) {
                let bands = settings.options(name).bands;
                let i = 0.6_f64.min(w / 4.0).min(h / 4.0);
                let (lo, hi) = (a + DVec2::splat(i), b - DVec2::splat(i));
                let bottom = (DVec2::new(lo.x, hi.y), hi);
                let top = (lo, DVec2::new(hi.x, lo.y));
                let left = (lo, DVec2::new(lo.x, hi.y));
                let right = (DVec2::new(hi.x, lo.y), hi);
                let sides = if p.rotated { [left, right, bottom, top] } else { [bottom, top, left, right] };
                for (e, on) in sides.into_iter().zip(bands) {
                    if on {
                        d.lines.push((e.0, e.1, Pen::Section));
                    }
                }
            }
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

/// Label sheet: A4 portrait, 3 × 8 labels of 70 × 37 mm (the common 24-up sheet).
const LABEL_PAGE: DVec2 = DVec2::new(210.0, 297.0);
const LABEL: DVec2 = DVec2::new(70.0, 37.0);
const LABEL_COLS: usize = 3;
const LABEL_ROWS: usize = 8;

/// `s` cut to about `width` mm at text `size`.
fn fit_text(s: &str, size: f64, width: f64) -> String {
    let max = (width / (size * 0.55)).floor() as usize;
    if s.chars().count() <= max { s.to_string() } else { format!("{}…", s.chars().take(max.saturating_sub(1)).collect::<String>()) }
}

/// One label per piece to cut (left-out parts have none): its number, name and copy, size,
/// material and, on the right, the part's outline with its banded edges and the grain.
fn label_pages(plan: &CutPlan, settings: &CutSettings, unit: LengthUnit, doc: &str) -> Vec<Drawing> {
    let mut pages: Vec<Drawing> = Vec::new();
    let per = LABEL_COLS * LABEL_ROWS;
    let top = (LABEL_PAGE.y - LABEL.y * LABEL_ROWS as f64) * 0.5;
    let mut n = 0;
    for (i, p) in plan.parts.iter().enumerate() {
        let o = settings.options(&p.name);
        if o.exclude {
            continue;
        }
        for copy in 0..p.count {
            if n % per == 0 {
                pages.push(Drawing { size: LABEL_PAGE, ..Default::default() });
            }
            let Some(d) = pages.last_mut() else { break };
            let slot = n % per;
            n += 1;
            let at = DVec2::new((slot % LABEL_COLS) as f64 * LABEL.x, top + (slot / LABEL_COLS) as f64 * LABEL.y);
            rect_lines(d, at, at + LABEL, Pen::Hidden);
            let x = at.x + 4.0;
            let number = (i + 1).to_string();
            text(d, DVec2::new(x, at.y + 10.0), 7.0, number.clone(), true);
            let nx = x + number.chars().count() as f64 * 4.0 + 2.0;
            let title = if p.count > 1 { tr!("cutlist-label-copy", name = p.name.clone(), n = copy + 1, total = p.count) } else { p.name.clone() };
            text(d, DVec2::new(nx, at.y + 9.0), 3.4, fit_text(&title, 3.4, at.x + 42.0 - nx), true);
            text(d, DVec2::new(x, at.y + 18.0), 3.2, format!("{} × {} × {}", dim(p.size[0], unit), dim(p.size[1], unit), dim(p.size[2], unit)), true);
            let cut = settings.cut_size(&p.name, p.size);
            if cut != p.size {
                text(d, DVec2::new(x, at.y + 23.0), 2.5, tr!("cutlist-label-cut", length = dim(cut[0], unit), width = dim(cut[1], unit)), true);
            }
            let material = if p.material_name.is_empty() { tr!("cutlist-no-material") } else { p.material_name.clone() };
            text(d, DVec2::new(x, at.y + 28.0), 2.6, fit_text(&material, 2.6, 38.0), true);
            text(d, DVec2::new(x, at.y + LABEL.y - 3.5), 2.0, fit_text(doc, 2.0, 38.0), true);
            // the part to scale in a 20 × 14 box, length across
            let k = (20.0 / p.size[0].max(1.0)).min(14.0 / p.size[1].max(1.0));
            let (w, h) = ((p.size[0] * k).max(3.0), (p.size[1] * k).max(3.0));
            let c = DVec2::new(at.x + LABEL.x - 15.0, at.y + LABEL.y * 0.5);
            let (a, b) = (c - DVec2::new(w, h) * 0.5, c + DVec2::new(w, h) * 0.5);
            rect_lines(d, a, b, Pen::Edge);
            let i = 0.7;
            let (lo, hi) = (a + DVec2::splat(i), b - DVec2::splat(i));
            let sides = [(DVec2::new(lo.x, hi.y), hi), (lo, DVec2::new(hi.x, lo.y)), (lo, DVec2::new(lo.x, hi.y)), (DVec2::new(hi.x, lo.y), hi)];
            for (e, on) in sides.into_iter().zip(o.bands) {
                if on {
                    d.lines.push((e.0, e.1, Pen::Section));
                }
            }
            // the grain: an arrow along the length or across it
            let dir = match o.grain {
                PartGrain::Length => Some(DVec2::X * (w * 0.35)),
                PartGrain::Width => Some(DVec2::Y * (h * 0.35)),
                PartGrain::Any => None,
            };
            if let Some(v) = dir {
                let (s0, s1) = (c - v, c + v);
                let head = v.normalize() * 1.5;
                let side = DVec2::new(-head.y, head.x);
                d.lines.extend([(s0, s1, Pen::Edge), (s1, s1 - head + side * 0.6, Pen::Edge), (s1, s1 - head - side * 0.6, Pen::Edge)]);
            }
        }
    }
    pages
}

impl VarstotasApp {
    /// The cut list of the model, cached until it changes.
    pub(crate) fn cut_plan(&mut self) -> Rc<CutPlan> {
        let key = (self.doc.model_version, offcuts_rev());
        if let Some((k, p)) = &self.cutlist.cache
            && (*k == key || (k.0 == key.0 && self.cutlist.kept == Some(key.0)))
        {
            return p.clone();
        }
        self.cutlist.kept = None;
        let p = Rc::new(model_plan(&self.doc.model));
        self.cutlist.cache = Some((key, p.clone()));
        p
    }

    /// Keep the offcuts the cut list plans with in step with the preferences and the document.
    pub(crate) fn sync_offcuts(&self) {
        let own = self.doc.path.as_ref().map(|p| p.display().to_string());
        let want: Vec<Offcut> = self
            .prefs
            .offcuts
            .iter()
            .map(|o| {
                let usable = self.prefs.use_offcuts && own.as_deref() != Some(o.source.as_str());
                Offcut { count: if usable { o.count } else { 0 }, ..o.clone() }
            })
            .collect();
        if let Ok(mut g) = OFFCUTS.lock()
            && g.1 != want
        {
            g.0 += 1;
            g.1 = want;
        }
    }

    /// The cut is done: the offcuts the plan used leave the user's offcuts and its leftovers
    /// join them. The plan shown stays as it was until the model changes.
    pub(crate) fn keep_offcuts(&mut self) {
        self.sync_offcuts();
        let plan = self.cut_plan();
        let used: usize = plan.plans.iter().filter(|s| s.offcut.is_some()).map(|s| s.layouts.len()).sum();
        let kept: usize = plan.leftovers().iter().map(|o| o.count).sum();
        let source = self.doc.path.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
        plan.keep_offcuts(&mut self.prefs.offcuts, &source);
        self.cutlist.kept = Some(self.doc.model_version);
        self.flash(tr!("cutlist-kept", kept = kept, used = used));
    }

    fn cut_drawings(&mut self) -> Vec<(Drawing, Vec<Hit>)> {
        let plan = self.cut_plan();
        let pages = pages(&plan);
        let unit = self.doc.model.unit;
        let name = self.doc.display_name();
        let settings = &self.doc.model.cut_list;
        pages.iter().enumerate().map(|(i, p)| page_drawing(&plan, settings, p, pages.len(), i + 1, unit, Some(&name))).collect()
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

    /// Write a PDF of part labels (A4, 24 per page) to `path`.
    pub(crate) fn export_cut_labels(&mut self, path: &Path) {
        let plan = self.cut_plan();
        let pages = label_pages(&plan, &self.doc.model.cut_list, self.doc.model.unit, &self.doc.display_name());
        match varstotas_kernel::io::atomic_write(path, crate::vector_export::to_pdf_pages(&pages)) {
            Ok(()) => self.flash(tr!("cutlist-exported", path = path.display().to_string())),
            Err(e) => self.flash_error(tr!("cutlist-export-failed", error = e.to_string())),
        }
    }

    fn cut_labels_save(&mut self) {
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            // i18n-ignore-next
            .set_file_name(format!("{} labels.pdf", self.doc.display_name()))
            .save_file()
        else {
            return;
        };
        if path.extension().is_none() {
            path.set_extension("pdf");
        }
        self.export_cut_labels(&path);
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
            Some((k, p)) if k.0 == self.doc.model_version && (k.1 == offcuts_rev() || self.cutlist.kept == Some(k.0)) => p.clone(),
            _ => Rc::new(model_plan(&self.doc.model)),
        };
        (self.cutlist.open, p.parts.len(), p.plans.len(), p.plans.iter().map(|s| s.layouts.len()).sum(), p.issues.len(), format!("{:.2}", p.cost()))
    }

    /// `([(part, banded edges)], banding mm)` for UI test scripts.
    pub(crate) fn cut_bands_dump(&self) -> (Vec<(String, [bool; 4])>, i64) {
        let c = &self.doc.model.cut_list;
        let parts = c.parts.iter().filter(|(_, o)| o.bands.iter().any(|&b| b)).map(|(n, o)| (n.clone(), o.bands)).collect();
        let total = model_plan(&self.doc.model).banding.iter().map(|b| b.length).sum::<f64>();
        (parts, total.round() as i64)
    }

    /// Part numbers on the model while the Cut List is open: one badge at the centre of each
    /// copy of a part (not of left-out ones).
    pub(crate) fn cut_numbers_overlay(&mut self, out: &mut Vec<crate::overlay::Marker2d>) {
        if !self.cutlist.open || self.cutlist.hide_numbers {
            return;
        }
        let v = self.doc.model_version;
        if self.cutlist.numbers.as_ref().is_none_or(|n| n.0 != v) {
            let plan = self.cut_plan();
            let m = &self.doc.model;
            let mut at = Vec::new();
            for (i, p) in plan.parts.iter().enumerate().filter(|(_, p)| !m.cut_list.options(&p.name).exclude) {
                for path in &p.paths {
                    let Some((&id, parent)) = path.split_last() else { continue };
                    if let Some(b) = m.instance_world_bounds(parent, id) {
                        at.push(((b.min + b.max) * 0.5, i));
                    }
                }
            }
            self.cutlist.numbers = Some((v, at));
        }
        if let Some((_, at)) = &self.cutlist.numbers {
            out.extend(at.iter().map(|(p, i)| crate::overlay::Marker2d::Badge { at: *p, text: (i + 1).to_string() }));
        }
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
        ui.horizontal(|ui| {
            ui.label(tr!("cutlist-band"));
            let mut shown = s.band / k;
            let r = ui.add(egui::DragValue::new(&mut shown).range(0.0..=10.0 / k).speed(0.05 / k).max_decimals(3).suffix(format!(" {}", unit.suffix())));
            if r.on_hover_text(tr!("cutlist-band-tip")).changed() {
                s.band = shown * k;
                changed = true;
            }
            ui.add_space(8.0);
            ui.label(tr!("cutlist-band-price"));
            if ui.add(egui::DragValue::new(&mut s.band_price).range(0.0..=1000.0).speed(0.05).max_decimals(2)).changed() {
                changed = true;
            }
            ui.add_space(8.0);
            if ui.checkbox(&mut s.band_subtract, tr!("cutlist-band-subtract")).on_hover_text(tr!("cutlist-band-subtract-tip")).changed() {
                changed = true;
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
                _ if sp.offcut.is_some() => vec![tr!("cutlist-offcuts-used", n = sp.layouts.len())],
                StockKind::Sheet => vec![tr!("cutlist-buy-sheets", n = sp.layouts.len())],
                StockKind::Board | StockKind::Linear => sp.to_buy().into_iter().map(|(len, n)| tr!("cutlist-buy-boards", n = n, length = dim(len, unit))).collect(),
            };
            let mut line = format!("{}: {}", stock_title(sp, unit), buy.join(", "));
            line.push_str(&tr!("cutlist-waste", percent = format!("{:.0}", sp.waste() * 100.0)));
            if sp.cost() > 0.0 {
                line.push_str(&tr!("cutlist-cost", cost = format!("{:.2}", sp.cost())));
            }
            ui.label(line);
        }
        for b in &plan.banding {
            let mut line = tr!("cutlist-banding", thickness = varstotas_kernel::units::format_length(b.thickness, unit), width = varstotas_kernel::units::format_length(b.width, unit), metres = format!("{:.2}", b.length / 1000.0));
            if b.cost() > 0.0 {
                line.push_str(&tr!("cutlist-cost", cost = format!("{:.2}", b.cost())));
            }
            ui.label(line);
        }
        if plan.plans.len() + plan.banding.len() > 1 && plan.cost() > 0.0 {
            ui.label(RichText::new(tr!("cutlist-total", cost = format!("{:.2}", plan.cost()))).strong());
        }
        let mut keep = false;
        ui.horizontal(|ui| {
            let on_hand: usize = self.prefs.offcuts.iter().map(|o| o.count).sum();
            if on_hand > 0 {
                ui.checkbox(&mut self.prefs.use_offcuts, tr!("cutlist-use-offcuts", n = on_hand)).on_hover_text(tr!("cutlist-use-offcuts-tip"));
            }
            let mut show = !self.cutlist.hide_numbers;
            if ui.checkbox(&mut show, tr!("cutlist-numbers")).on_hover_text(tr!("cutlist-numbers-tip")).changed() {
                self.cutlist.hide_numbers = !show;
            }
            let kept = self.cutlist.kept == Some(self.doc.model_version);
            let r = ui.add_enabled(!plan.plans.is_empty() && !kept, egui::Button::new(tr!("cutlist-keep")));
            super::tray_lib::spot("cutlist:keep", r.rect.center());
            if r.on_hover_text(tr!("cutlist-keep-tip")).on_disabled_hover_text(varstotas_i18n::tr(if kept { "cutlist-keep-done" } else { "cutlist-keep-tip" })).clicked() {
                keep = true;
            }
        });
        if keep {
            self.keep_offcuts();
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
                for h in ["cutlist-col-part", "cutlist-col-count", "cutlist-col-size", "cutlist-col-material", "cutlist-col-grain", "cutlist-col-bands", "cutlist-col-exclude"] {
                    ui.label(RichText::new(varstotas_i18n::tr(h)).strong());
                }
                ui.end_row();
                for (i, p) in plan.parts.iter().enumerate() {
                    let o = self.doc.model.cut_list.options(&p.name);
                    if ui.add(egui::Label::new(format!("{}  {}", i + 1, p.name)).sense(Sense::click())).on_hover_text(tr!("cutlist-click-select")).clicked() {
                        select = Some(i);
                    }
                    ui.label(p.count.to_string());
                    let r = ui.label(format!("{} × {} × {}", dim(p.size[0], unit), dim(p.size[1], unit), dim(p.size[2], unit)));
                    let cut = self.doc.model.cut_list.cut_size(&p.name, p.size);
                    if cut != p.size {
                        r.on_hover_text(tr!("cutlist-cut-size", length = dim(cut[0], unit), width = dim(cut[1], unit)));
                    }
                    ui.label(if p.material_name.is_empty() { tr!("cutlist-no-material") } else { p.material_name.clone() });
                    let mut g = o.grain;
                    egui::ComboBox::from_id_salt(("cutlist_grain", i)).selected_text(grain_name(g)).show_ui(ui, |ui| {
                        for v in [PartGrain::Length, PartGrain::Width, PartGrain::Any] {
                            ui.selectable_value(&mut g, v, grain_name(v));
                        }
                    });
                    let mut bands = o.bands;
                    band_edges(ui, &mut bands, i);
                    let mut ex = o.exclude;
                    ui.checkbox(&mut ex, "");
                    if g != o.grain || ex != o.exclude || bands != o.bands {
                        set = Some((p.name.clone(), PartOptions { grain: g, exclude: ex, bands }));
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
        // the page's content (not its empty margins), as wide as the window allows
        let bounds = drawings.get(s.page).map(|(d, _)| {
            let pts = d.lines.iter().flat_map(|(a, b, _)| [*a, *b]).chain(d.texts.iter().map(|t| t.at - DVec2::new(0.0, t.size)));
            pts.fold((DVec2::splat(f64::MAX), DVec2::splat(f64::MIN)), |(lo, hi), p| (lo.min(p), hi.max(p)))
        });
        let (lo, hi) = bounds.filter(|(lo, hi)| hi.x > lo.x && hi.y > lo.y).unwrap_or((DVec2::ZERO, PAGE));
        let (lo, hi) = (lo - DVec2::splat(3.0), hi + DVec2::splat(3.0));
        let k = ((640.0 - 16.0) / (hi.x - lo.x)).min((452.0 - 16.0) / (hi.y - lo.y));
        let height = ((hi.y - lo.y) * k + 16.0).max(200.0) as f32;
        let (rect, resp) = ui.allocate_exact_size(vec2(640.0, height), Sense::click());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, theme::border());
        let mut clicked = None;
        if let Some((d, hits)) = drawings.get(s.page) {
            let o = rect.center() - vec2(((hi.x - lo.x) * k) as f32, ((hi.y - lo.y) * k) as f32) * 0.5;
            let sp = |p: DVec2| pos2(o.x + ((p.x - lo.x) * k) as f32, o.y + ((p.y - lo.y) * k) as f32);
            painter.rect_filled(egui::Rect::from_min_max(sp(lo), sp(hi)), 0.0, Color32::WHITE);
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
                    Pen::Section => {
                        painter.line_segment([a, b], Stroke::new(2.5, BAND_COLOR));
                    }
                    _ => {
                        painter.line_segment([a, b], Stroke::new(1.0, Color32::from_gray(40)));
                    }
                }
            }
            for t in &d.texts {
                // larger than on paper (the preview is a fraction of the page's size); labels centred
                // in pieces and offcuts are sized on paper to fit them, so their floor is lower
                let px = ((t.size * k * 1.5) as f32).max(if t.left { 9.0 } else { 7.0 });
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
        let mut labels = false;
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
                    let r = ui.button(RichText::new(tr!("cutlist-labels")).strong());
                    if r.on_hover_text(tr!("cutlist-labels-tip")).clicked() {
                        labels = true;
                    }
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
        if labels {
            self.cut_labels_save();
        }
    }

    /// The Material Editor's stock section: what the material is bought as.
    pub(crate) fn material_stock(&mut self, ui: &mut Ui) {
        let Some(id) = self.current_material_id() else {
            ui.label(RichText::new(tr!("mated-stock-not-in-model")).small().color(theme::text_dim()));
            return;
        };
        let unit = self.doc.model.unit;
        let mut list = match &self.cutlist.stock_draft {
            Some((m, l)) if *m == id => l.clone(),
            _ => self.doc.model.materials[id].stock.clone(),
        };
        let mut changed = false;
        let mut remove = None;
        let mut save = None;
        for (i, st) in list.iter_mut().enumerate() {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(stock_label(st, unit)).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if super::small_icon_button(ui, super::icons::Icon::Trash, &tr!("mated-stock-remove")).clicked() {
                            remove = Some(i);
                        }
                        if ui.small_button("☆").on_hover_text(tr!("mated-stock-save-preset")).clicked() {
                            save = Some(st.clone());
                        }
                    });
                });
                changed |= stock_editor(ui, st, ("mated_stock", i), unit);
            });
        }
        if let Some(i) = remove {
            list.remove(i);
            changed = true;
        }
        if let Some(st) = save {
            let name = format!("{} {}", self.doc.model.materials[id].name, stock_label(&st, unit));
            self.prefs.stock_presets.push(StockPreset { name: name.clone(), stock: st });
            self.flash(tr!("mated-stock-saved", name = name));
        }
        let mut edit = false;
        ui.menu_button(tr!("mated-stock-add"), |ui| {
            for (key, st) in stock_presets() {
                if ui.button(varstotas_i18n::tr(key)).clicked() {
                    list.push(st);
                    changed = true;
                    ui.close();
                }
            }
            ui.separator();
            ui.label(RichText::new(tr!("mated-stock-my-presets")).small().color(theme::text_dim()));
            for p in &self.prefs.stock_presets {
                if ui.button(&p.name).clicked() {
                    list.push(p.stock.clone());
                    changed = true;
                    ui.close();
                }
            }
            if ui.button(tr!("mated-stock-edit-presets")).clicked() {
                edit = true;
                ui.close();
            }
        });
        if edit {
            self.prefs_win.open = true;
            self.prefs_win.section = crate::ui::preferences::Section::CutList;
        }
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

    /// Preferences → Cut List: the user's stock templates (offered in + Add stock).
    pub(crate) fn prefs_cut_list(&mut self, ui: &mut Ui) {
        let unit = self.doc.model.unit;
        ui.label(RichText::new(tr!("prefs-cut-presets")).strong());
        ui.label(RichText::new(tr!("prefs-cut-presets-note")).small().color(theme::text_dim()));
        ui.add_space(4.0);
        if self.prefs.stock_presets.is_empty() {
            ui.label(RichText::new(tr!("prefs-cut-none")).color(theme::text_dim()));
        }
        let mut remove = None;
        for (i, p) in self.prefs.stock_presets.iter_mut().enumerate() {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut p.name).desired_width(240.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if super::small_icon_button(ui, super::icons::Icon::Trash, &tr!("prefs-cut-remove")).clicked() {
                            remove = Some(i);
                        }
                    });
                });
                stock_editor(ui, &mut p.stock, ("prefs_stock", i), unit);
            });
        }
        if let Some(i) = remove {
            self.prefs.stock_presets.remove(i);
        }
        ui.horizontal(|ui| {
            if ui.button(tr!("prefs-cut-add-sheet")).clicked() {
                let st = Stock::sheet(2440.0, 1220.0, 18.0, true);
                self.prefs.stock_presets.push(StockPreset { name: stock_label(&st, unit), stock: st });
            }
            if ui.button(tr!("prefs-cut-add-board")).clicked() {
                let st = Stock::board(95.0, 45.0, &[3000.0]);
                self.prefs.stock_presets.push(StockPreset { name: stock_label(&st, unit), stock: st });
            }
            if ui.button(tr!("prefs-cut-add-linear")).clicked() {
                let st = Stock::linear("Ø 20", 20.0, 20.0, &[2000.0]);
                self.prefs.stock_presets.push(StockPreset { name: stock_label(&st, unit), stock: st });
            }
        });
        // offcuts kept from earlier projects
        ui.add_space(12.0);
        ui.label(RichText::new(tr!("prefs-cut-offcuts")).strong());
        ui.label(RichText::new(tr!("prefs-cut-offcuts-note")).small().color(theme::text_dim()));
        ui.add_space(4.0);
        if self.prefs.offcuts.is_empty() {
            ui.label(RichText::new(tr!("prefs-cut-offcuts-none")).color(theme::text_dim()));
            return;
        }
        ui.checkbox(&mut self.prefs.use_offcuts, tr!("prefs-cut-use-offcuts"));
        let mut remove = None;
        egui::Grid::new("prefs_offcuts").num_columns(4).spacing([10.0, 4.0]).striped(true).show(ui, |ui| {
            for (i, o) in self.prefs.offcuts.iter_mut().enumerate() {
                ui.label(offcut_label(o, unit));
                ui.add(egui::DragValue::new(&mut o.count).range(1..=999).prefix("× "));
                let from = std::path::Path::new(&o.source).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                ui.label(RichText::new(from).small().color(theme::text_dim())).on_hover_text(&o.source);
                if super::small_icon_button(ui, super::icons::Icon::Trash, &tr!("prefs-cut-offcut-remove")).clicked() {
                    remove = Some(i);
                }
                ui.end_row();
            }
        });
        if let Some(i) = remove {
            self.prefs.offcuts.remove(i);
        }
        if ui.button(tr!("prefs-cut-offcuts-clear")).clicked() {
            self.prefs.offcuts.clear();
        }
    }
}

/// A stock template of the user's (Preferences → Cut List).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StockPreset {
    pub name: String,
    pub stock: Stock,
}

/// "2440 × 1220 × 18" for a sheet, "95 × 45" for a board, the profile of linear stock.
fn stock_label(st: &Stock, unit: LengthUnit) -> String {
    match st.kind {
        StockKind::Sheet => format!("{} × {} × {}", dim(st.length, unit), dim(st.width, unit), dim(st.thickness, unit)),
        StockKind::Linear if !st.name.is_empty() => st.name.clone(),
        StockKind::Linear if st.width <= 0.0 || st.thickness <= 0.0 => tr!("mated-stock-linear"),
        _ => format!("{} × {}", dim(st.width, unit), dim(st.thickness, unit)),
    }
}

/// An offcut in a list: "Plywood 800 × 400 × 18", or a board's length × section.
fn offcut_label(o: &Offcut, unit: LengthUnit) -> String {
    let st = &o.stock;
    let what = match st.kind {
        StockKind::Sheet => stock_label(st, unit),
        _ => format!("{} · {}", dim(st.length, unit), stock_label(st, unit)),
    };
    format!("{} {what}", o.material)
}

/// A comma-separated list edited as text (kept in temp data while typing), parsed on Enter or
/// when it loses focus.
fn list_edit(ui: &mut Ui, id: egui::Id, shown: String, parse: impl FnOnce(&str)) {
    let mut s = ui.data(|d| d.get_temp::<String>(id)).unwrap_or(shown);
    let r = ui.add(egui::TextEdit::singleline(&mut s).desired_width(180.0));
    if r.changed() {
        ui.data_mut(|d| d.insert_temp(id, s.clone()));
    }
    if r.lost_focus() {
        ui.data_mut(|d| d.remove::<String>(id));
        parse(&s);
    }
}

/// The fields of one stock item; true when it changed.
fn stock_editor(ui: &mut Ui, st: &mut Stock, salt: (&'static str, usize), unit: LengthUnit) -> bool {
    let k = unit.mm();
    let mut changed = false;
    let len = |ui: &mut Ui, v: &mut f64, changed: &mut bool| {
        let mut shown = *v / k;
        if ui.add(egui::DragValue::new(&mut shown).range(0.0..=100_000.0 / k).speed(1.0 / k).max_decimals(3).suffix(format!(" {}", unit.suffix()))).changed() {
            *v = shown * k;
            *changed = true;
        }
    };
    egui::Grid::new(salt).num_columns(2).spacing([10.0, 4.0]).show(ui, |ui| {
        ui.label(tr!("mated-stock-kind"));
        ui.horizontal(|ui| {
            let before = st.kind;
            ui.selectable_value(&mut st.kind, StockKind::Sheet, tr!("mated-stock-sheet"));
            ui.selectable_value(&mut st.kind, StockKind::Board, tr!("mated-stock-board"));
            ui.selectable_value(&mut st.kind, StockKind::Linear, tr!("mated-stock-linear"));
            if st.kind != before {
                if st.is_lengths() && st.lengths.is_empty() {
                    st.lengths = vec![st.length.max(1000.0)];
                }
                changed = true;
            }
        });
        ui.end_row();
        if st.kind == StockKind::Linear {
            ui.label(tr!("mated-stock-profile"));
            changed |= ui.add(egui::TextEdit::singleline(&mut st.name).desired_width(180.0).hint_text(tr!("mated-stock-profile-hint"))).changed();
            ui.end_row();
            ui.label(tr!("mated-stock-section"));
            ui.horizontal(|ui| {
                len(ui, &mut st.width, &mut changed);
                ui.label("×");
                len(ui, &mut st.thickness, &mut changed);
                ui.label(RichText::new("ⓘ").color(theme::text_dim())).on_hover_text(tr!("mated-stock-section-tip"));
            });
            ui.end_row();
        } else {
            ui.label(tr!("mated-stock-thickness"));
            len(ui, &mut st.thickness, &mut changed);
            ui.end_row();
        }
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
                ui.label(tr!("mated-stock-price-sheet"));
                changed |= ui.add(egui::DragValue::new(&mut st.price).range(0.0..=1e7).speed(0.1).max_decimals(2)).changed();
                ui.end_row();
            }
            StockKind::Board | StockKind::Linear => {
                if st.kind == StockKind::Board {
                    ui.label(tr!("mated-stock-width"));
                    len(ui, &mut st.width, &mut changed);
                    ui.end_row();
                }
                ui.label(tr!("mated-stock-lengths"));
                let shown = st.board_lengths().iter().map(|v| dim(*v, unit)).collect::<Vec<_>>().join(", ");
                list_edit(ui, egui::Id::new((salt, "lengths")), shown, |s| {
                    if let Some(v) = varstotas_kernel::units::parse_lengths(&s.replace(';', ","), unit) {
                        let v: Vec<f64> = v.into_iter().filter(|x| *x > 0.0).collect();
                        if !v.is_empty() && v != st.lengths {
                            st.length = v.iter().copied().fold(0.0, f64::max);
                            st.lengths = v;
                            changed = true;
                        }
                    }
                });
                ui.end_row();
                ui.label(tr!("mated-stock-price"));
                ui.horizontal(|ui| {
                    let per_piece = !st.piece_prices.is_empty();
                    if ui.selectable_label(!per_piece, tr!("mated-stock-per-metre")).clicked() && per_piece {
                        st.piece_prices.clear();
                        changed = true;
                    }
                    if ui.selectable_label(per_piece, tr!("mated-stock-per-piece")).clicked() && !per_piece {
                        // a first guess from the price per metre
                        st.piece_prices = st.lengths.iter().map(|l| (st.price * l / 1000.0 * 100.0).round() / 100.0).collect();
                        changed = true;
                    }
                });
                ui.end_row();
                ui.label("");
                if st.piece_prices.is_empty() {
                    changed |= ui.add(egui::DragValue::new(&mut st.price).range(0.0..=1e7).speed(0.1).max_decimals(2)).changed();
                } else {
                    let shown = st.piece_prices.iter().map(|v| format!("{v}")).collect::<Vec<_>>().join(", ");
                    ui.horizontal(|ui| {
                        list_edit(ui, egui::Id::new((salt, "prices")), shown, |s| {
                            let v: Option<Vec<f64>> = s.split([',', ';']).map(str::trim).filter(|x| !x.is_empty()).map(|x| x.parse::<f64>().ok().filter(|v| *v >= 0.0)).collect();
                            if let Some(v) = v.filter(|v| !v.is_empty())
                                && v != st.piece_prices
                            {
                                st.piece_prices = v;
                                changed = true;
                            }
                        });
                        ui.label(RichText::new("ⓘ").color(theme::text_dim())).on_hover_text(tr!("mated-stock-prices-tip"));
                    });
                }
                ui.end_row();
            }
        }
    });
    changed
}

/// Banded edges in the preview and the parts table.
const BAND_COLOR: Color32 = Color32::from_rgb(0xB0, 0x4A, 0x1E);

/// The banded edges of part row `row`: a small rectangle (the part, its length across) whose
/// sides toggle on a click; the long edges are the bottom and top, the short ones the sides.
fn band_edges(ui: &mut Ui, bands: &mut [bool; 4], row: usize) {
    let (rect, resp) = ui.allocate_exact_size(vec2(44.0, 22.0), Sense::click());
    let r = rect.shrink2(vec2(6.0, 5.0));
    let sides = [
        [r.left_bottom(), r.right_bottom()],
        [r.left_top(), r.right_top()],
        [r.left_top(), r.left_bottom()],
        [r.right_top(), r.right_bottom()],
    ];
    for (e, s) in sides.iter().enumerate() {
        super::tray_lib::spot(format!("cutlist:band:{row}:{e}"), s[0].lerp(s[1], 0.5));
    }
    // the side nearest the pointer
    let near = resp.hover_pos().map(|p| {
        let d = p - r.center();
        if d.x.abs() / r.width() > d.y.abs() / r.height() { if d.x < 0.0 { 2 } else { 3 } } else if d.y > 0.0 { 0 } else { 1 }
    });
    let painter = ui.painter();
    painter.rect_filled(r, 0.0, theme::border());
    for (e, s) in sides.iter().enumerate() {
        let stroke = if bands[e] {
            Stroke::new(3.0, BAND_COLOR)
        } else if near == Some(e) {
            Stroke::new(2.0, theme::ACCENT)
        } else {
            Stroke::new(1.0, theme::text_dim())
        };
        painter.line_segment(*s, stroke);
    }
    if let Some(e) = near
        && resp.clicked()
    {
        bands[e] = !bands[e];
    }
    resp.on_hover_text(tr!("cutlist-bands-tip"));
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
        ("mated-stock-preset-rod-20", Stock::linear("Ø 20", 20.0, 20.0, &[1000.0, 2000.0])),
        ("mated-stock-preset-skirting-70", Stock::linear("70 × 15", 70.0, 15.0, &[2500.0])),
    ]
}
