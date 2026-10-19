//! Tools → Unfold…: flat patterns of the selection (or the whole context) with a live
//! preview; export to SVG (sheets one under another), PDF (a page per sheet) or DXF.

use crate::app::VarstotasApp;
use crate::document::guarded;
use crate::ui::theme;
use crate::vector_export::{Drawing, Pen, Text2};
use egui::{Color32, Context, RichText, Sense, Stroke, pos2, vec2};
use varstotas_kernel::glam::DVec2;
use varstotas_kernel::unfold::{FlatLine, Sheet, UnfoldInput, UnfoldOptions, Unfolding, to_dxf, unfold};
use std::path::Path;
use std::rc::Rc;

/// Paper margin, mm.
const MARGIN: f64 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Paper {
    /// One sheet as large as the pieces need.
    Fit,
    A4,
    A3,
    A2,
}

impl Paper {
    pub const ALL: [Paper; 4] = [Paper::Fit, Paper::A4, Paper::A3, Paper::A2];

    pub fn name(self) -> &'static str {
        match self {
            Paper::Fit => "Fit to pieces",
            Paper::A4 => "A4",
            Paper::A3 => "A3",
            Paper::A2 => "A2",
        }
    }

    /// Portrait size, mm.
    fn size(self) -> Option<DVec2> {
        match self {
            Paper::Fit => None,
            Paper::A4 => Some(DVec2::new(210.0, 297.0)),
            Paper::A3 => Some(DVec2::new(297.0, 420.0)),
            Paper::A2 => Some(DVec2::new(420.0, 594.0)),
        }
    }
}

pub struct UnfoldState {
    pub open: bool,
    /// Tabs, numbers, soft folds, text (scale and sheet come from the fields below).
    pub opts: UnfoldOptions,
    /// Scale 1:N.
    pub scale_n: f64,
    pub paper: Paper,
    pub landscape: bool,
    /// Sheet shown in the preview.
    pub page: usize,
    cache: Option<(u64, u64, UnfoldOptions, Rc<Unfolding>)>,
    /// Faces, pieces, sheets, lines of the last unfolding (`dump`).
    pub last: Option<(usize, usize, usize, usize)>,
}

impl Default for UnfoldState {
    fn default() -> Self {
        Self { open: false, opts: UnfoldOptions::default(), scale_n: 1.0, paper: Paper::A4, landscape: false, page: 0, cache: None, last: None }
    }
}

impl UnfoldState {
    /// Page size, mm (`None`: fitted to the sheet).
    fn page_size(&self) -> Option<DVec2> {
        self.paper.size().map(|s| if self.landscape { DVec2::new(s.y, s.x) } else { s })
    }

    pub fn options(&self) -> UnfoldOptions {
        UnfoldOptions {
            scale: 1.0 / self.scale_n.max(1e-6),
            sheet: self.page_size().map(|s| s - DVec2::splat(2.0 * MARGIN)),
            ..self.opts
        }
    }
}

/// A sheet on paper (mm, y down), with a footer line.
pub(crate) fn sheet_drawing(sheet: &Sheet, page: Option<DVec2>, footer: &str) -> Drawing {
    // A piece larger than the paper enlarges its page (the window warns about it).
    let need = sheet.size + DVec2::splat(2.0 * MARGIN);
    let size = page.map_or(need, |p| p.max(need));
    let to_paper = |p: DVec2| DVec2::new(MARGIN + p.x, MARGIN + sheet.size.y - p.y);
    let mut d = Drawing { size, ..Default::default() };
    for p in &sheet.pieces {
        for (a, b, kind) in &p.lines {
            let pen = match kind {
                FlatLine::Cut => Pen::Edge,
                FlatLine::Mountain => Pen::Mountain,
                FlatLine::Valley => Pen::Valley,
            };
            d.lines.push((to_paper(*a), to_paper(*b), pen));
        }
        for l in &p.labels {
            // y flips: the text turns the other way; its baseline sits below the centre.
            let angle = -l.angle;
            let dir = DVec2::new(angle.cos(), angle.sin());
            let up = DVec2::new(dir.y, -dir.x);
            d.texts.push(Text2 { at: to_paper(l.at) - up * l.size * 0.35, angle, size: l.size, lines: vec![l.text.clone()], left: false });
        }
    }
    if !footer.is_empty() {
        d.texts.push(Text2 { at: DVec2::new(MARGIN, size.y - MARGIN * 0.4), angle: 0.0, size: 2.5, lines: vec![footer.to_string()], left: true });
    }
    d
}

/// Drawings one under another (for SVG, which has no pages).
fn stack(pages: Vec<Drawing>) -> Drawing {
    let mut out = Drawing::default();
    for d in pages {
        let dy = if out.size.y > 0.0 { out.size.y + MARGIN } else { 0.0 };
        let off = DVec2::new(0.0, dy);
        out.lines.extend(d.lines.into_iter().map(|(a, b, p)| (a + off, b + off, p)));
        out.texts.extend(d.texts.into_iter().map(|mut t| {
            t.at += off;
            t
        }));
        out.size = DVec2::new(out.size.x.max(d.size.x), dy + d.size.y);
    }
    out
}

impl VarstotasApp {
    /// The unfolding of the selection (everything in the context when nothing is selected),
    /// cached until the model, the selection or the options change.
    pub(crate) fn unfolding(&mut self) -> Rc<Unfolding> {
        let opts = self.unfold.options();
        let key = (self.doc.model_version, self.doc.selection_version);
        if let Some((m, s, o, u)) = &self.unfold.cache
            && (*m, *s) == key
            && *o == opts
        {
            return u.clone();
        }
        let input = UnfoldInput::from_model(&self.doc.model, &self.doc.active_path, &self.doc.selection);
        let u = Rc::new(guarded(|| unfold(&input, &opts)).unwrap_or_default());
        let lines = u.sheets.iter().flat_map(|s| &s.pieces).map(|p| p.lines.len()).sum();
        self.unfold.last = Some((u.faces, u.pieces, u.sheets.len(), lines));
        self.unfold.cache = Some((key.0, key.1, opts, u.clone()));
        u
    }

    fn unfold_drawings(&mut self) -> Vec<Drawing> {
        let u = self.unfolding();
        let page = self.unfold.page_size();
        let n = u.sheets.len();
        let name = self.doc.display_name();
        let scale = crate::ui::unfold::scale_label(self.unfold.scale_n);
        u.sheets
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let footer = if n > 1 { format!("{name} - unfold {scale} - sheet {} of {n}", i + 1) } else { format!("{name} - unfold {scale}") };
                sheet_drawing(s, page, &footer)
            })
            .collect()
    }

    /// Write the unfolding to `path` (SVG, PDF or DXF by extension).
    pub(crate) fn export_unfold(&mut self, path: &Path) {
        let u = self.unfolding();
        if u.faces == 0 {
            self.flash("Unfold: nothing to unfold (select faces or groups)".to_string());
            return;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase();
        let bytes = match ext.as_str() {
            "dxf" => to_dxf(&u).into_bytes(),
            "pdf" => crate::vector_export::to_pdf_pages(&self.unfold_drawings()),
            _ => crate::vector_export::to_svg(&stack(self.unfold_drawings())).into_bytes(),
        };
        match std::fs::write(path, bytes) {
            Ok(()) => {
                let more = if u.oversize > 0 { format!(" ({} larger than the sheet)", u.oversize) } else { String::new() };
                self.flash(format!("Unfolded {} faces into {} pieces on {} sheets{more}: {}", u.faces, u.pieces, u.sheets.len(), path.display()));
            }
            Err(e) => self.flash(format!("Export failed: {e}")),
        }
    }

    fn unfold_save(&mut self, ext: &str) {
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter(ext.to_uppercase(), &[ext])
            .set_file_name(format!("{} unfold.{ext}", self.doc.display_name()))
            .save_file()
        else {
            return;
        };
        if path.extension().is_none() {
            path.set_extension(ext);
        }
        self.export_unfold(&path);
    }

    /// The Unfold window (not modal: the selection can change while it is open).
    pub(crate) fn unfold_window(&mut self, ctx: &Context) {
        if !self.unfold.open {
            return;
        }
        let u = self.unfolding();
        let pages = self.unfold_drawings();
        self.unfold.page = self.unfold.page.min(pages.len().saturating_sub(1));
        let pos = self.viewport.view_rect.min + vec2(40.0, 40.0);
        let mut open = true;
        let mut save: Option<&str> = None;
        let mut win = egui::Window::new("Unfold").id(egui::Id::new("unfold_window")).open(&mut open).collapsible(false).resizable(false).default_pos(pos);
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        let selection = !self.doc.selection.is_empty();
        win.show(ctx, |ui| {
            ui.set_width(560.0);
            let what = if selection { "Selection" } else { "Everything in the context" };
            let summary = if u.faces == 0 {
                format!("{what}: no faces. Select faces, groups or components.")
            } else {
                format!("{what}: {} faces → {} pieces on {} sheet(s)", u.faces, u.pieces, u.sheets.len())
            };
            ui.label(RichText::new(summary).strong());
            if u.oversize > 0 {
                ui.label(RichText::new(format!("{} piece(s) larger than the sheet: choose a larger paper or scale.", u.oversize)).color(Color32::from_rgb(0xC0, 0x60, 0x00)));
            }
            ui.add_space(4.0);
            let s = &mut self.unfold;
            egui::Grid::new("unfold_opts").num_columns(4).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label("Scale 1:");
                ui.add(egui::DragValue::new(&mut s.scale_n).range(0.01..=10000.0).speed(0.1));
                ui.label("Paper");
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("unfold_paper").selected_text(s.paper.name()).show_ui(ui, |ui| {
                        for p in Paper::ALL {
                            ui.selectable_value(&mut s.paper, p, p.name());
                        }
                    });
                    ui.add_enabled(s.paper != Paper::Fit, egui::Checkbox::new(&mut s.landscape, "Landscape"));
                });
                ui.end_row();
                ui.checkbox(&mut s.opts.tabs, "Glue tabs");
                ui.add_enabled(s.opts.tabs, egui::DragValue::new(&mut s.opts.tab_width).range(1.0..=50.0).speed(0.2).suffix(" mm"));
                ui.checkbox(&mut s.opts.numbers, "Edge numbers");
                ui.checkbox(&mut s.opts.soft_folds, "Folds on soft edges");
                ui.end_row();
            });
            ui.add_space(6.0);
            // Preview of the current sheet.
            let (rect, _) = ui.allocate_exact_size(vec2(560.0, 340.0), Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 4.0, theme::BORDER);
            if let Some(d) = pages.get(s.page) {
                let k = ((rect.width() as f64 - 20.0) / d.size.x).min((rect.height() as f64 - 20.0) / d.size.y);
                let o = rect.center() - vec2((d.size.x * k) as f32, (d.size.y * k) as f32) * 0.5;
                let sp = |p: DVec2| pos2(o.x + (p.x * k) as f32, o.y + (p.y * k) as f32);
                painter.rect_filled(egui::Rect::from_min_max(sp(DVec2::ZERO), sp(d.size)), 0.0, Color32::WHITE);
                if let Some(sheet) = u.sheets.get(s.page) {
                    // Faces lightly tinted (triangulated: they may be concave).
                    let to_paper = |p: DVec2| DVec2::new(MARGIN + p.x, MARGIN + sheet.size.y - p.y);
                    let mut mesh = egui::Mesh::default();
                    let tint = Color32::from_rgb(232, 238, 246);
                    for piece in &sheet.pieces {
                        for (o, h) in piece.faces.iter().map(|(o, h)| (o.clone(), h.clone())).chain(piece.tabs.iter().map(|t| (t.clone(), Vec::new()))) {
                            let holes: Vec<&[DVec2]> = h.iter().map(|x| x.as_slice()).collect();
                            let all: Vec<DVec2> = o.iter().chain(h.iter().flatten()).copied().collect();
                            for t in varstotas_kernel::triangulate::triangulate(&o, &holes) {
                                let base = mesh.vertices.len() as u32;
                                for i in t {
                                    mesh.colored_vertex(sp(to_paper(all[i])), tint);
                                }
                                mesh.add_triangle(base, base + 1, base + 2);
                            }
                        }
                    }
                    painter.add(egui::Shape::mesh(mesh));
                }
                for (a, b, pen) in &d.lines {
                    let (a, b) = (sp(*a), sp(*b));
                    match pen {
                        Pen::Mountain => painter.extend(egui::Shape::dashed_line(&[a, b], Stroke::new(1.0, Color32::from_rgb(200, 0, 0)), 5.0, 2.0)),
                        Pen::Valley => painter.extend(egui::Shape::dashed_line(&[a, b], Stroke::new(1.0, Color32::from_rgb(0, 80, 190)), 3.0, 2.0)),
                        _ => {
                            painter.line_segment([a, b], Stroke::new(1.0, Color32::BLACK));
                        }
                    }
                }
                for t in &d.texts {
                    let px = (t.size * k) as f32;
                    if px < 5.0 {
                        continue;
                    }
                    let galley = painter.layout_no_wrap(t.lines.join(" "), egui::FontId::proportional(px), Color32::from_gray(60));
                    let size = galley.size();
                    let ang = t.angle as f32;
                    // Baseline anchor → top-left corner of the rotated galley.
                    let (dir, down) = (vec2(ang.cos(), ang.sin()), vec2(-ang.sin(), ang.cos()));
                    let along = if t.left { 0.0 } else { size.x * 0.5 };
                    let top_left = sp(t.at) - dir * along - down * (size.y * 0.8);
                    painter.add(egui::epaint::TextShape::new(top_left, galley, Color32::from_gray(60)).with_angle(ang));
                }
            }
            ui.horizontal(|ui| {
                if pages.len() > 1 {
                    if ui.add_enabled(s.page > 0, egui::Button::new("◀")).clicked() {
                        s.page -= 1;
                    }
                    ui.label(format!("Sheet {} of {}", s.page + 1, pages.len()));
                    if ui.add_enabled(s.page + 1 < pages.len(), egui::Button::new("▶")).clicked() {
                        s.page += 1;
                    }
                    ui.separator();
                }
                ui.label(RichText::new("━ cut").color(Color32::GRAY));
                ui.label(RichText::new("╍ valley").color(Color32::from_rgb(70, 130, 230)));
                ui.label(RichText::new("━·━ mountain").color(Color32::from_rgb(220, 60, 60)));
            });
            ui.add_space(6.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_enabled_ui(u.faces > 0, |ui| {
                    for ext in ["dxf", "pdf", "svg"] {
                        if ui.add(egui::Button::new(RichText::new(format!("{}…", ext.to_uppercase())).strong())).clicked() {
                            save = Some(ext);
                        }
                    }
                });
                ui.label(RichText::new("Export:").color(theme::TEXT_DIM));
            });
        });
        if !open {
            self.unfold.open = false;
        }
        if let Some(ext) = save {
            self.unfold_save(ext);
        }
    }
}

/// "1:N" with N trimmed.
pub(crate) fn scale_label(n: f64) -> String {
    let s = format!("{n:.3}");
    format!("1:{}", s.trim_end_matches('0').trim_end_matches('.'))
}
