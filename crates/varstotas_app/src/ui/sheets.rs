//! Window → Sheets…: paper sheets with viewports of the model (the current view or a scene,
//! fitted or to scale), texts and a title block; a live preview where items are picked,
//! moved and resized with the mouse; export to PDF (all sheets) or SVG (the current one).
//! Edits go to a working copy that becomes one undo step when the mouse and keyboard let go.

use crate::app::VarstotasApp;
use crate::layout::{BORDER, TITLE_ROOM, camera_scene, sheet_drawing, sheets_pdf, standard_views, table_size, viewport_scale};
use crate::ui::theme;
use crate::vector_export::{Drawing, Pen};
use egui::{Color32, Context, Pos2, Rect, RichText, Sense, Stroke, pos2, vec2};
use varstotas_kernel::glam::DVec2;
use varstotas_kernel::Model;
use varstotas_kernel::layout::{LayoutSheet, PAPERS, SheetTable, SheetText, SheetViewport, TitleBlock};
use std::path::Path;
use std::rc::Rc;
use varstotas_i18n::tr;

/// Preview size, points.
const PREVIEW: egui::Vec2 = vec2(620.0, 440.0);
/// Grab radius of the resize handle, points.
const HANDLE: f32 = 7.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SheetItem {
    Viewport(usize),
    Text(usize),
    Table(usize),
}

#[derive(Clone, Copy, Debug)]
struct Drag {
    item: SheetItem,
    resize: bool,
    min: DVec2,
    size: DVec2,
}

#[derive(Default)]
pub struct SheetsState {
    pub open: bool,
    pub current: usize,
    pub selected: Option<SheetItem>,
    drag: Option<Drag>,
    /// Edited copy of the current sheet, committed when no field or drag is active.
    work: Option<(usize, LayoutSheet)>,
    cache: Option<(u64, usize, usize, LayoutSheet, Rc<Drawing>)>,
    /// Screen position of the paper's corner and points per mm in the preview (`sheetp`).
    pub preview: Option<(Pos2, f32)>,
}

/// Paper name of a size, either way round.
fn paper_name(size: DVec2) -> String {
    PAPERS.iter().find(|(_, s)| s.distance(size) < 0.5 || DVec2::new(s.y, s.x).distance(size) < 0.5).map_or_else(|| tr!("sheets-custom"), |p| p.0.to_string())
}

/// Rough box of a text on paper (top-left, size).
fn text_box(t: &SheetText) -> (DVec2, DVec2) {
    let lines: Vec<&str> = t.text.lines().collect();
    let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1) as f64 * t.size * 0.6;
    let h = lines.len().max(1) as f64 * t.size * 1.3;
    (DVec2::new(t.at.x, t.at.y - t.size), DVec2::new(w, h))
}

/// The topmost item at `p` (mm): texts, tables, then viewports; later above earlier.
fn hit(model: &Model, sheet: &LayoutSheet, p: DVec2) -> Option<SheetItem> {
    let within = |a: DVec2, s: DVec2| p.x >= a.x && p.y >= a.y && p.x <= a.x + s.x && p.y <= a.y + s.y;
    for (i, t) in sheet.texts.iter().enumerate().rev() {
        let (a, s) = text_box(t);
        if within(a, s) {
            return Some(SheetItem::Text(i));
        }
    }
    for (i, t) in sheet.tables.iter().enumerate().rev() {
        if within(t.at, table_size(model, t)) {
            return Some(SheetItem::Table(i));
        }
    }
    sheet.viewports.iter().rposition(|v| v.contains(p)).map(SheetItem::Viewport)
}

fn snap(v: DVec2) -> DVec2 {
    (v * 2.0).round() / 2.0
}

fn pen_stroke(pen: Pen, k: f32) -> Stroke {
    let [r, g, b] = pen.rgb().map(|c| (c * 255.0) as u8);
    Stroke::new((pen.width() as f32 * k).clamp(0.6, 2.5), Color32::from_rgb(r, g, b))
}

/// Paint a drawing (paper mm, y down) at `o` with `k` points per mm.
fn paint_drawing(painter: &egui::Painter, d: &Drawing, o: Pos2, k: f32) {
    let sp = |p: DVec2| pos2(o.x + p.x as f32 * k, o.y + p.y as f32 * k);
    let mut mesh = egui::Mesh::default();
    let tint = Color32::from_gray(200);
    for f in &d.fills {
        for t in varstotas_kernel::triangulate::triangulate(f, &[]) {
            let base = mesh.vertices.len() as u32;
            for i in t {
                mesh.colored_vertex(sp(f[i]), tint);
            }
            mesh.add_triangle(base, base + 1, base + 2);
        }
    }
    painter.add(egui::Shape::mesh(mesh));
    for (a, b, pen) in &d.lines {
        let (a, b) = (sp(*a), sp(*b));
        let stroke = pen_stroke(*pen, k);
        let dash = pen.dash();
        if dash.is_empty() {
            painter.line_segment([a, b], stroke);
        } else {
            let (on, off) = ((dash[0] as f32 * k).max(2.0), (dash[1] as f32 * k).max(1.5));
            painter.extend(egui::Shape::dashed_line(&[a, b], stroke, on, off));
        }
    }
    for t in &d.texts {
        let px = t.size as f32 * k;
        if px < 3.0 {
            continue;
        }
        let font = egui::FontId::proportional(px);
        for (i, line) in t.lines.iter().enumerate() {
            let galley = painter.layout_no_wrap(line.clone(), font.clone(), Color32::from_gray(30));
            let ang = t.angle as f32;
            let (dir, down) = (vec2(ang.cos(), ang.sin()), vec2(-ang.sin(), ang.cos()));
            let along = if t.left { 0.0 } else { galley.size().x * 0.5 };
            let base = sp(t.at) + down * (i as f32 * px * 1.3);
            let top_left = base - dir * along - down * (galley.size().y * 0.8);
            painter.add(egui::epaint::TextShape::new(top_left, galley, Color32::from_gray(30)).with_angle(ang));
        }
    }
}

impl VarstotasApp {
    /// The current view as a viewport source (its eye distance keeps the parallel size).
    fn current_view_scene(&self, name: &str) -> varstotas_kernel::Scene {
        let mut s = self.scene_from_camera(name.to_string());
        s.eye = camera_scene(name, &self.camera).eye;
        s
    }

    /// A viewport of the current view in the free part of `sheet`.
    pub(crate) fn new_viewport(&self, sheet: &LayoutSheet) -> SheetViewport {
        let lo = DVec2::splat(BORDER + 5.0);
        let room = sheet.size - lo * 2.0 - DVec2::new(0.0, if sheet.title.is_some() { TITLE_ROOM } else { 0.0 });
        let n = sheet.viewports.len();
        let (min, size) = if n == 0 {
            (lo, room)
        } else {
            let off = DVec2::splat(8.0 * ((n - 1) % 6) as f64);
            (lo + off, room * 0.5)
        };
        SheetViewport::new(min, size.max(DVec2::splat(20.0)), self.current_view_scene(&format!("Viewport {}", n + 1))) // i18n-ignore
    }

    /// A new landscape sheet (A3 unless `paper` names another).
    pub(crate) fn add_sheet(&mut self, paper: &str, portrait: bool) {
        let size = PAPERS.iter().find(|(n, _)| n.eq_ignore_ascii_case(paper)).map_or(PAPERS[1].1, |p| p.1);
        let size = if portrait { size } else { DVec2::new(size.y, size.x) };
        let n = self.doc.model.sheets.len();
        let sheet = LayoutSheet::new(format!("Sheet {}", n + 1), size); // i18n-ignore
        self.doc.commit(&tr!("sheets-undo-add"), |m| m.sheets.push(sheet));
        self.sheets.current = n;
        self.sheets.selected = None;
        self.sheets.work = None;
    }

    /// Change the current sheet in one undo step (from the harness and buttons).
    pub(crate) fn edit_sheet(&mut self, label: &str, f: impl FnOnce(&mut LayoutSheet)) {
        self.sheets.work = None;
        let cur = self.sheets.current;
        if cur >= self.doc.model.sheets.len() {
            self.flash(tr!("sheets-none-first"));
            return;
        }
        self.doc.commit(label, |m| f(&mut m.sheets[cur]));
    }

    /// Sheet `i` on paper, cached until the model or the sheet changes.
    fn sheet_page(&mut self, i: usize, sheet: &LayoutSheet) -> Rc<Drawing> {
        let n = self.doc.model.sheets.len();
        let v = self.doc.model_version;
        if let Some((cv, ci, cn, cs, d)) = &self.sheets.cache
            && (*cv, *ci, *cn) == (v, i, n)
            && cs == sheet
        {
            return d.clone();
        }
        let name = self.doc.display_name();
        let d = Rc::new(crate::document::guarded(|| sheet_drawing(&self.doc.model, sheet, i, n, &name)).unwrap_or_default());
        self.sheets.cache = Some((v, i, n, sheet.clone(), d.clone()));
        d
    }

    /// Sheets, current (1-based), its viewports, texts and lines on paper (`dump`).
    pub(crate) fn sheets_dump(&self) -> Option<(usize, usize, usize, usize, usize)> {
        let m = &self.doc.model;
        let i = self.sheets.current.min(m.sheets.len().checked_sub(1)?);
        let s = &m.sheets[i];
        let lines = sheet_drawing(m, s, i, m.sheets.len(), "").lines.len();
        Some((m.sheets.len(), i + 1, s.viewports.len(), s.texts.len(), lines))
    }

    /// Texts on the current sheet (lines joined with `|`), for `expect`.
    pub(crate) fn sheet_texts_dump(&self) -> Vec<String> {
        let m = &self.doc.model;
        let Some(s) = m.sheets.get(self.sheets.current) else { return Vec::new() };
        sheet_drawing(m, s, self.sheets.current, m.sheets.len(), "").texts.iter().map(|t| t.lines.join("|")).collect()
    }

    /// Position and size of the selected viewport or table, whole mm (`dump`).
    pub(crate) fn sheet_vp_dump(&self) -> Option<(i64, i64, i64, i64)> {
        let m = &self.doc.model;
        let sheet = m.sheets.get(self.sheets.current)?;
        let (min, size) = match self.sheets.selected? {
            SheetItem::Viewport(v) => sheet.viewports.get(v).map(|vp| (vp.min, vp.size))?,
            SheetItem::Table(t) => sheet.tables.get(t).map(|t| (t.at, table_size(m, t)))?,
            SheetItem::Text(_) => return None,
        };
        let r = |x: f64| x.round() as i64;
        Some((r(min.x), r(min.y), r(size.x), r(size.y)))
    }

    /// PDF of all sheets, or SVG of the current one (by extension).
    pub(crate) fn export_sheets(&mut self, path: &Path) {
        let n = self.doc.model.sheets.len();
        if n == 0 {
            self.flash(tr!("sheets-nothing-to-export"));
            return;
        }
        let name = self.doc.display_name();
        let svg = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("svg"));
        let cur = self.sheets.current.min(n - 1);
        let model = &self.doc.model;
        let bytes = crate::document::guarded(|| {
            if svg {
                crate::vector_export::to_svg(&sheet_drawing(model, &model.sheets[cur], cur, n, &name)).into_bytes()
            } else {
                sheets_pdf(model, &name)
            }
        });
        match bytes.map_err(|e| e.to_string()).and_then(|b| std::fs::write(path, b).map_err(|e| e.to_string())) {
            Ok(()) if svg => self.flash(tr!("sheets-exported-one", i = cur + 1, n = n, path = path.display().to_string())),
            Ok(()) => self.flash(tr!("sheets-exported-all", n = n, path = path.display().to_string())),
            Err(e) => self.flash(tr!("sheets-export-failed", error = e)),
        }
    }

    fn sheets_save(&mut self, ext: &str) {
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter(ext.to_uppercase(), &[ext])
            .set_file_name(format!("{} sheets.{ext}", self.doc.display_name()))
            .save_file()
        else {
            return;
        };
        if path.extension().is_none() {
            path.set_extension(ext);
        }
        self.export_sheets(&path);
    }

    pub(crate) fn sheets_window(&mut self, ctx: &Context) {
        if !self.sheets.open {
            self.sheets.preview = None;
            return;
        }
        let n = self.doc.model.sheets.len();
        self.sheets.current = self.sheets.current.min(n.saturating_sub(1));
        let cur = self.sheets.current;
        let orig = self.doc.model.sheets.get(cur).cloned();
        let mut sheet = match (&self.sheets.work, &orig) {
            (Some((i, w)), Some(_)) if *i == cur => Some(w.clone()),
            _ => orig.clone(),
        };
        let page = sheet.as_ref().map(|s| self.sheet_page(cur, s));
        let names: Vec<String> = self.doc.model.sheets.iter().map(|s| s.name.clone()).collect();
        let scenes: Vec<(String, bool)> = self.doc.model.scenes.iter().map(|s| (s.name.clone(), s.ortho)).collect();
        let current_view = self.current_view_scene("View"); // i18n-ignore
        let new_vp = sheet.as_ref().map(|s| self.new_viewport(s));
        let pos = self.viewport.view_rect.min + vec2(20.0, 20.0);
        let mut open = true;
        let (mut add, mut delete, mut duplicate, mut goto, mut save) = (false, false, false, None, None);
        let mut win = egui::Window::new(tr!("sheets-title")).id(egui::Id::new("sheets_window")).open(&mut open).collapsible(false).resizable(false).default_pos(pos);
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        let model = &self.doc.model;
        let st = &mut self.sheets;
        win.show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                // Left column: sheets, paper, title block, the selected item.
                ui.vertical(|ui| {
                    ui.set_width(270.0);
                    ui.label(RichText::new(tr!("sheets-title")).strong());
                    egui::ScrollArea::vertical().id_salt("sheet_list").max_height(110.0).show(ui, |ui| {
                        for (i, name) in names.iter().enumerate() {
                            if ui.selectable_label(i == cur, name).clicked() {
                                goto = Some(i);
                            }
                        }
                        if names.is_empty() {
                            ui.label(RichText::new(tr!("sheets-none-yet")).color(theme::text_dim()));
                        }
                    });
                    ui.horizontal(|ui| {
                        add = ui.button(tr!("sheets-add")).clicked();
                        duplicate = ui.add_enabled(n > 0, egui::Button::new(tr!("sheets-duplicate"))).clicked();
                        delete = ui.add_enabled(n > 0, egui::Button::new(tr!("delete"))).clicked();
                    });
                    let Some(sheet) = sheet.as_mut() else { return };
                    ui.separator();
                    egui::Grid::new("sheet_props").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                        ui.label(tr!("sheets-name"));
                        ui.text_edit_singleline(&mut sheet.name);
                        ui.end_row();
                        ui.label(tr!("sheets-paper"));
                        ui.horizontal(|ui| {
                            let landscape = sheet.size.x > sheet.size.y;
                            egui::ComboBox::from_id_salt("sheet_paper").selected_text(paper_name(sheet.size)).show_ui(ui, |ui| {
                                for (name, s) in PAPERS {
                                    if ui.selectable_label(paper_name(sheet.size) == name, name).clicked() {
                                        sheet.size = if landscape { DVec2::new(s.y, s.x) } else { s };
                                    }
                                }
                            });
                            let mut l = landscape;
                            if ui.checkbox(&mut l, tr!("sheets-landscape")).changed() && l != landscape {
                                sheet.size = DVec2::new(sheet.size.y, sheet.size.x);
                            }
                        });
                        ui.end_row();
                        let mut has_title = sheet.title.is_some();
                        ui.label(tr!("sheets-title-block"));
                        if ui.checkbox(&mut has_title, "").changed() {
                            sheet.title = has_title.then(TitleBlock::default);
                        }
                        ui.end_row();
                        if let Some(tb) = sheet.title.as_mut() {
                            for (label, v) in [(tr!("sheets-tb-title"), &mut tb.title), (tr!("sheets-tb-author"), &mut tb.author), (tr!("sheets-tb-date"), &mut tb.date)] {
                                ui.label(label);
                                ui.text_edit_singleline(v);
                                ui.end_row();
                            }
                        }
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(tr!("sheets-add-viewport")).on_hover_text(tr!("sheets-add-viewport-hint")).clicked()
                            && let Some(vp) = new_vp.clone()
                        {
                            sheet.viewports.push(vp);
                            st.selected = Some(SheetItem::Viewport(sheet.viewports.len() - 1));
                        }
                        if ui.button(tr!("sheets-add-text")).clicked() {
                            let at = DVec2::new(BORDER + 10.0, sheet.size.y - BORDER - 12.0);
                            sheet.texts.push(SheetText { at, size: 4.0, text: "Text".into() /* i18n-ignore */ });
                            st.selected = Some(SheetItem::Text(sheet.texts.len() - 1));
                        }
                        if ui.button(tr!("sheets-add-credits")).on_hover_text(tr!("sheets-add-credits-hint")).clicked() {
                            let at = DVec2::new(BORDER + 6.0, sheet.size.y - BORDER - 30.0);
                            sheet.texts.push(SheetText { at, size: 2.5, text: crate::layout::CREDITS.into() });
                            st.selected = Some(SheetItem::Text(sheet.texts.len() - 1));
                        }
                        if ui.add_enabled(st.selected.is_some(), egui::Button::new(tr!("sheets-remove"))).clicked() {
                            match st.selected.take() {
                                Some(SheetItem::Viewport(i)) if i < sheet.viewports.len() => {
                                    sheet.viewports.remove(i);
                                }
                                Some(SheetItem::Text(i)) if i < sheet.texts.len() => {
                                    sheet.texts.remove(i);
                                }
                                Some(SheetItem::Table(i)) if i < sheet.tables.len() => {
                                    sheet.tables.remove(i);
                                }
                                _ => {}
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        let views = ui.add_enabled(model.bounds().is_some(), egui::Button::new(tr!("sheets-add-views")));
                        if views.on_hover_text(tr!("sheets-add-views-hint")).clicked() {
                            sheet.viewports.extend(standard_views(model, sheet, None));
                            st.selected = None;
                        }
                        if ui.button(tr!("sheets-add-table")).on_hover_text(tr!("sheets-add-table-hint")).clicked() {
                            let mut t = SheetTable { at: DVec2::ZERO, size: 3.0 };
                            let h = table_size(model, &t).y;
                            t.at = DVec2::new(BORDER + 5.0, sheet.size.y - BORDER - 5.0 - h);
                            sheet.tables.push(t);
                            st.selected = Some(SheetItem::Table(sheet.tables.len() - 1));
                        }
                    });
                    match st.selected {
                        Some(SheetItem::Viewport(i)) if i < sheet.viewports.len() => {
                            let vp = &mut sheet.viewports[i];
                            ui.label(RichText::new(tr!("sheets-viewport-n", n = i + 1)).strong());
                            egui::Grid::new("vp_props").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                                ui.label(tr!("sheets-shows"));
                                let shown = vp.scene.clone().unwrap_or_else(|| tr!("sheets-captured"));
                                egui::ComboBox::from_id_salt("vp_scene").selected_text(shown).show_ui(ui, |ui| {
                                    if ui.selectable_label(vp.scene.is_none(), tr!("sheets-captured")).clicked() {
                                        vp.scene = None;
                                    }
                                    for (name, _) in &scenes {
                                        if ui.selectable_label(vp.scene.as_ref() == Some(name), name).clicked() {
                                            vp.scene = Some(name.clone());
                                        }
                                    }
                                });
                                ui.end_row();
                                ui.label("");
                                if ui.button(tr!("sheets-update-view")).clicked() {
                                    let name = vp.view.name.clone();
                                    vp.view = current_view.clone();
                                    vp.view.name = name;
                                    vp.scene = None;
                                }
                                ui.end_row();
                                let ortho = model.viewport_scene(vp).ortho;
                                ui.label(tr!("sheets-scale"));
                                ui.horizontal(|ui| {
                                    let mut to_scale = vp.scale.is_some();
                                    if ui.add_enabled(ortho, egui::Checkbox::new(&mut to_scale, "1:")).changed() {
                                        vp.scale = to_scale.then_some(50.0);
                                    }
                                    if let Some(n) = vp.scale.as_mut() {
                                        ui.add_enabled(ortho, egui::DragValue::new(n).range(0.01..=100000.0).speed(0.5));
                                    }
                                })
                                .response
                                .on_hover_text(tr!("sheets-scale-hint"));
                                ui.end_row();
                                ui.label(tr!("sheets-position"));
                                ui.horizontal(|ui| {
                                    ui.add(egui::DragValue::new(&mut vp.min.x).speed(0.5).suffix(" mm"));
                                    ui.add(egui::DragValue::new(&mut vp.min.y).speed(0.5).suffix(" mm"));
                                });
                                ui.end_row();
                                ui.label(tr!("sheets-size"));
                                ui.horizontal(|ui| {
                                    ui.add(egui::DragValue::new(&mut vp.size.x).range(5.0..=2000.0).speed(0.5).suffix(" mm"));
                                    ui.add(egui::DragValue::new(&mut vp.size.y).range(5.0..=2000.0).speed(0.5).suffix(" mm"));
                                });
                                ui.end_row();
                            });
                            ui.horizontal_wrapped(|ui| {
                                ui.checkbox(&mut vp.profiles, tr!("sheets-profiles"));
                                ui.checkbox(&mut vp.hidden, tr!("sheets-hidden"));
                                ui.checkbox(&mut vp.dims, tr!("sheets-dims"));
                                ui.checkbox(&mut vp.sections, tr!("sheets-sections"));
                                ui.checkbox(&mut vp.frame, tr!("sheets-frame"));
                                ui.checkbox(&mut vp.caption, tr!("sheets-caption"));
                                ui.add_enabled(model.viewport_scene(vp).ortho, egui::Checkbox::new(&mut vp.auto_dims, tr!("sheets-overall")));
                            });
                        }
                        Some(SheetItem::Text(i)) if i < sheet.texts.len() => {
                            let t = &mut sheet.texts[i];
                            ui.label(RichText::new(tr!("sheets-text-n", n = i + 1)).strong());
                            ui.add(egui::TextEdit::multiline(&mut t.text).desired_rows(3).desired_width(260.0));
                            ui.horizontal(|ui| {
                                ui.label(tr!("sheets-height"));
                                ui.add(egui::DragValue::new(&mut t.size).range(1.0..=100.0).speed(0.1).suffix(" mm"));
                            });
                            ui.horizontal(|ui| {
                                ui.label(tr!("sheets-position"));
                                ui.add(egui::DragValue::new(&mut t.at.x).speed(0.5).suffix(" mm"));
                                ui.add(egui::DragValue::new(&mut t.at.y).speed(0.5).suffix(" mm"));
                            });
                        }
                        Some(SheetItem::Table(i)) if i < sheet.tables.len() => {
                            let t = &mut sheet.tables[i];
                            ui.label(RichText::new(tr!("sheets-table-n", n = i + 1)).strong());
                            ui.horizontal(|ui| {
                                ui.label(tr!("sheets-text-height"));
                                ui.add(egui::DragValue::new(&mut t.size).range(1.0..=20.0).speed(0.1).suffix(" mm"));
                            });
                            ui.horizontal(|ui| {
                                ui.label(tr!("sheets-position"));
                                ui.add(egui::DragValue::new(&mut t.at.x).speed(0.5).suffix(" mm"));
                                ui.add(egui::DragValue::new(&mut t.at.y).speed(0.5).suffix(" mm"));
                            });
                        }
                        _ => {
                            ui.label(RichText::new(tr!("sheets-hint")).color(theme::text_dim()));
                        }
                    }
                });
                // Right: the sheet.
                ui.vertical(|ui| {
                    let (rect, resp) = ui.allocate_exact_size(PREVIEW, Sense::click_and_drag());
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 4.0, theme::border());
                    let (Some(sheet), Some(page)) = (sheet.as_mut(), page.as_ref()) else {
                        st.preview = None;
                        painter.text(rect.center(), egui::Align2::CENTER_CENTER, &tr!("sheets-add-first"), egui::FontId::proportional(15.0), theme::text_dim());
                        return;
                    };
                    let k = ((rect.width() - 24.0) / sheet.size.x as f32).min((rect.height() - 24.0) / sheet.size.y as f32);
                    let o = rect.center() - vec2(sheet.size.x as f32, sheet.size.y as f32) * k * 0.5;
                    st.preview = Some((o, k));
                    let sp = |p: DVec2| pos2(o.x + p.x as f32 * k, o.y + p.y as f32 * k);
                    let mm = |p: Pos2| DVec2::new(((p.x - o.x) / k) as f64, ((p.y - o.y) / k) as f64);
                    painter.rect_filled(Rect::from_min_max(sp(DVec2::ZERO), sp(sheet.size)), 0.0, Color32::WHITE);
                    paint_drawing(&painter, page, o, k);
                    // Pick, move, resize.
                    let press = ctx.input(|i| i.pointer.press_origin());
                    let at = if resp.drag_started() { press } else { resp.clicked().then(|| resp.interact_pointer_pos()).flatten() };
                    if let Some(pp) = at {
                        let handle = match st.selected {
                            Some(SheetItem::Viewport(i)) => sheet.viewports.get(i).filter(|v| sp(v.min + v.size).distance(pp) <= HANDLE).map(|_| i),
                            _ => None,
                        };
                        let item = if handle.is_some() { st.selected } else { hit(model, sheet, mm(pp)) };
                        st.selected = item;
                        st.drag = None;
                        if resp.drag_started() {
                            st.drag = match item {
                                Some(SheetItem::Viewport(i)) => {
                                    let v = &sheet.viewports[i];
                                    Some(Drag { item: SheetItem::Viewport(i), resize: handle.is_some(), min: v.min, size: v.size })
                                }
                                Some(SheetItem::Text(i)) => Some(Drag { item: SheetItem::Text(i), resize: false, min: sheet.texts[i].at, size: DVec2::ZERO }),
                                Some(SheetItem::Table(i)) => Some(Drag { item: SheetItem::Table(i), resize: false, min: sheet.tables[i].at, size: DVec2::ZERO }),
                                None => None,
                            };
                        }
                    }
                    if let (Some(d), Some(pp), Some(now)) = (st.drag, press, ctx.input(|i| i.pointer.interact_pos()))
                        && resp.dragged()
                    {
                        let delta = DVec2::new(((now.x - pp.x) / k) as f64, ((now.y - pp.y) / k) as f64);
                        match d.item {
                            SheetItem::Viewport(i) if i < sheet.viewports.len() => {
                                let v = &mut sheet.viewports[i];
                                if d.resize {
                                    v.size = snap(d.size + delta).max(DVec2::splat(5.0));
                                } else {
                                    v.min = snap(d.min + delta);
                                }
                            }
                            SheetItem::Text(i) if i < sheet.texts.len() => sheet.texts[i].at = snap(d.min + delta),
                            SheetItem::Table(i) if i < sheet.tables.len() => sheet.tables[i].at = snap(d.min + delta),
                            _ => {}
                        }
                    }
                    if resp.drag_stopped() {
                        st.drag = None;
                    }
                    // Selection outline (the item as edited, not yet redrawn).
                    let sel = match st.selected {
                        Some(SheetItem::Viewport(i)) => sheet.viewports.get(i).map(|v| (v.min, v.size, true)),
                        Some(SheetItem::Text(i)) => sheet.texts.get(i).map(|t| {
                            let (a, s) = text_box(t);
                            (a, s, false)
                        }),
                        Some(SheetItem::Table(i)) => sheet.tables.get(i).map(|t| (t.at, table_size(model, t), false)),
                        None => None,
                    };
                    if let Some((a, s, handle)) = sel {
                        let r = Rect::from_min_max(sp(a), sp(a + s));
                        painter.rect_stroke(r, 0.0, Stroke::new(1.5, theme::ACCENT), egui::StrokeKind::Outside);
                        if handle {
                            painter.rect_filled(Rect::from_center_size(r.max, vec2(8.0, 8.0)), 1.0, theme::ACCENT);
                        }
                    }
                    ui.horizontal(|ui| {
                        let scale = sheet.viewports.iter().filter(|v| viewport_scale(model, v).is_some()).count();
                        ui.label(RichText::new(tr!("sheets-summary", paper = paper_name(sheet.size), viewports = sheet.viewports.len(), scaled = scale, texts = sheet.texts.len())).color(theme::text_dim()));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(RichText::new(tr!("dialog-drawing-svg")).strong()).on_hover_text(tr!("sheets-svg-hint")).clicked() {
                                save = Some("svg");
                            }
                            if ui.button(RichText::new(tr!("dialog-drawing-pdf")).strong()).on_hover_text(tr!("sheets-pdf-hint")).clicked() {
                                save = Some("pdf");
                            }
                            ui.label(RichText::new(tr!("sheets-export")).color(theme::text_dim()));
                        });
                    });
                });
            });
        });
        // The working copy: kept while a field has focus or the mouse is down, then committed.
        if let (Some(s), Some(o)) = (sheet, orig) {
            if s != o {
                self.sheets.work = Some((cur, s));
            } else {
                self.sheets.work = None;
            }
        }
        let busy = ctx.text_edit_focused() || ctx.input(|i| i.pointer.any_down());
        if !busy && let Some((i, s)) = self.sheets.work.take() {
            self.doc.commit(&tr!("sheets-undo-edit"), |m| {
                if let Some(slot) = m.sheets.get_mut(i) {
                    *slot = s;
                }
            });
        }
        if !open {
            self.sheets.open = false;
        }
        if add {
            self.add_sheet("A3", false);
        }
        if duplicate && cur < n {
            let mut s = self.doc.model.sheets[cur].clone();
            s.name = format!("{} copy", s.name); // i18n-ignore
            self.sheets.work = None;
            self.doc.commit(&tr!("sheets-undo-duplicate"), |m| m.sheets.insert(cur + 1, s));
            self.sheets.current = cur + 1;
        }
        if delete && cur < n {
            self.sheets.work = None;
            self.doc.commit(&tr!("sheets-undo-delete"), |m| {
                m.sheets.remove(cur);
            });
            self.sheets.selected = None;
        }
        if let Some(i) = goto
            && i != cur
        {
            self.sheets.current = i;
            self.sheets.selected = None;
        }
        if let Some(ext) = save {
            self.sheets_save(ext);
        }
    }
}
