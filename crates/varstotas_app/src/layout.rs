//! Layout sheets on paper (light page layout): each viewport is the line drawing of a scene
//! (`vector_export::view_lines`) fitted into its rectangle or drawn to scale 1:N, clipped to
//! the rectangle, with optional overall dimensions; plus free texts, parts tables, a border
//! and a title block. `standard_views` lays out front / top / left / iso views at a common
//! standard scale. Exported as PDF (a page per sheet) or SVG (one sheet).

use crate::vector_export::{Drawing, Pen, Text2, VectorOptions, view_lines};
use crate::ui::unfold::scale_label;
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::layout::{LayoutSheet, SheetTable, SheetViewport};
use varstotas_kernel::{LengthUnit, Model, Scene};
use varstotas_i18n::tr;
use varstotas_render::{Camera, StandardView};

/// Border inset from the paper edge, mm.
pub const BORDER: f64 = 10.0;
/// Screen points per paper mm when a viewport's view is projected (precision only).
const PX_PER_MM: f64 = 4.0;
/// Title block: width and row heights, mm.
const TB_W: f64 = 180.0;
const TB_ROWS: [f64; 3] = [10.0, 7.0, 7.0];
/// Height kept free above the title block for new viewports, mm.
pub const TITLE_ROOM: f64 = 30.0;
/// Margin around a standard view's content (dimensions, caption), mm.
const VIEW_MARGIN: f64 = 14.0;
/// Overall dimensions: distance from the content and text height, mm.
const DIM_OFFSET: f64 = 7.0;
const DIM_TEXT: f64 = 2.5;
/// Standard reduction scales (GOST 2.302), 1:N.
const SCALES: [f64; 18] = [1.0, 2.0, 2.5, 4.0, 5.0, 10.0, 15.0, 20.0, 25.0, 40.0, 50.0, 75.0, 100.0, 200.0, 400.0, 500.0, 800.0, 1000.0];

/// The camera a scene stores (parallel projection: visible height from the eye distance).
pub fn scene_camera(s: &Scene) -> Camera {
    let d = (s.eye - s.target).length();
    Camera {
        eye: s.eye,
        target: s.target,
        up: s.up,
        fov_y_deg: s.fov_y_deg,
        ortho: s.ortho,
        ortho_height: 2.0 * d * (s.fov_y_deg.to_radians() * 0.5).tan(),
    }
}

/// A scene of `cam` whose stored eye distance gives back its parallel height.
pub fn camera_scene(name: &str, cam: &Camera) -> Scene {
    let mut eye = cam.eye;
    if cam.ortho {
        let t = (cam.fov_y_deg.to_radians() * 0.5).tan();
        let d = (cam.ortho_height * 0.5 / t).max(1e-3);
        eye = cam.target - cam.basis().forward * d;
    }
    Scene::camera(name, eye, cam.target, cam.up, cam.fov_y_deg, cam.ortho)
}

/// The viewport's scale when it applies (parallel projection).
pub fn viewport_scale(model: &Model, vp: &SheetViewport) -> Option<f64> {
    vp.scale.filter(|n| *n > 0.0 && model.viewport_scene(vp).ortho)
}

/// A sheet text's field for the credits of the downloaded models (File → Export → Attributions).
pub const CREDITS: &str = "{credits}";

/// "Credits:" and a line per downloaded model the model uses.
pub fn credits_text(model: &Model) -> String {
    let lines: Vec<String> = model.attributions().iter().map(|a| a.line()).collect();
    if lines.is_empty() { tr!("layout-credits-none") } else { tr!("layout-credits", lines = lines.join("\n")) }
}

/// Clip segment `a`–`b` to the box (Liang–Barsky).
fn clip(a: DVec2, b: DVec2, lo: DVec2, hi: DVec2) -> Option<(DVec2, DVec2)> {
    let d = b - a;
    let (mut t0, mut t1) = (0.0f64, 1.0f64);
    for (p, q) in [(-d.x, a.x - lo.x), (d.x, hi.x - a.x), (-d.y, a.y - lo.y), (d.y, hi.y - a.y)] {
        if p.abs() < 1e-12 {
            if q < 0.0 {
                return None;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                t0 = t0.max(r);
            } else {
                t1 = t1.min(r);
            }
        }
    }
    (t0 <= t1).then(|| (a + d * t0, a + d * t1))
}

fn rect_lines(d: &mut Drawing, lo: DVec2, hi: DVec2, pen: Pen) {
    let c = [lo, DVec2::new(hi.x, lo.y), hi, DVec2::new(lo.x, hi.y)];
    for i in 0..4 {
        d.lines.push((c[i], c[(i + 1) % 4], pen));
    }
}

fn text(at: DVec2, size: f64, s: impl Into<String>) -> Text2 {
    Text2 { at, angle: 0.0, size, lines: vec![s.into()], left: true }
}

/// The viewport's lines on paper (clipped to its rectangle), with frame and caption.
pub fn viewport_drawing(model: &Model, vp: &SheetViewport) -> Drawing {
    let scene = model.viewport_scene(vp);
    let m = model.with_scene_state(scene);
    let cam = scene_camera(scene);
    let px = (vp.size * PX_PER_MM).max(DVec2::splat(8.0));
    let opts = VectorOptions { scale: None, profiles: vp.profiles, hidden: vp.hidden, dims: vp.dims, sections: vp.sections };
    let lines = view_lines(&m, &cam, px, &opts);
    let (lo, hi) = (vp.min, vp.min + vp.size);
    let centre = vp.min + vp.size * 0.5;
    let mut out = Drawing::default();
    // Screen point p → paper: off + p * k.
    let map = match (viewport_scale(model, vp), lines.bounds()) {
        (Some(n), _) => {
            let k = cam.pixel_size_at(cam.target, px) / n;
            Some((centre - px * 0.5 * k, k))
        }
        (None, Some((a, b))) => {
            let pad = if vp.auto_dims { VIEW_MARGIN } else { 3.0 }.min(vp.size.min_element() * 0.2);
            let ext = (b - a).max(DVec2::splat(1e-6));
            let k = ((vp.size.x - 2.0 * pad) / ext.x).min((vp.size.y - 2.0 * pad) / ext.y).max(1e-9);
            Some((centre - (a + b) * 0.5 * k, k))
        }
        (None, None) => None,
    };
    if let Some((off, k)) = map {
        let d = lines.on_paper(Drawing::default(), move |p| off + p * k);
        out.lines.extend(d.lines.into_iter().filter_map(|(a, b, pen)| clip(a, b, lo, hi).map(|(a, b)| (a, b, pen))));
        let inside = |p: DVec2| p.x >= lo.x && p.y >= lo.y && p.x <= hi.x && p.y <= hi.y;
        out.fills.extend(d.fills.into_iter().filter(|f| f.iter().all(|p| inside(*p))));
        out.texts.extend(d.texts.into_iter().filter(|t| inside(t.at)));
        // Overall dimensions of the model's lines (world size = screen size × point size).
        let mut b = (DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY));
        for (p, q, _) in &lines.strokes {
            b = (b.0.min(p.min(*q)), b.1.max(p.max(*q)));
        }
        if vp.auto_dims && cam.ortho && b.0.is_finite() {
            let world = (b.1 - b.0) * cam.pixel_size_at(cam.target, px);
            overall_dims(&mut out, off + b.0 * k, off + b.1 * k, world, model.unit);
        }
    }
    if vp.frame {
        rect_lines(&mut out, lo, hi, Pen::Dim);
    }
    if vp.caption {
        let name = vp.scene.clone().unwrap_or_else(|| scene.name.clone());
        let caption = match viewport_scale(model, vp) {
            Some(n) => format!("{name}  {}", scale_label(n)),
            None => name,
        };
        if !caption.trim().is_empty() {
            out.texts.push(text(DVec2::new(lo.x, hi.y + 4.5), 3.0, caption));
        }
    }
    out
}

/// A dimension value: plain millimetres on drawings, else with the unit.
fn dim_text(v: f64, unit: LengthUnit) -> String {
    if unit == LengthUnit::Millimeter {
        let s = format!("{v:.1}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        varstotas_kernel::units::format_length(v, unit)
    }
}

/// An arrowhead at `tip` pointing along `dir` (unit).
fn arrow(d: &mut Drawing, tip: DVec2, dir: DVec2) {
    let (c, s) = (15f64.to_radians().cos(), 15f64.to_radians().sin());
    for side in [-1.0, 1.0] {
        let r = DVec2::new(dir.x * c - dir.y * s * side, dir.x * s * side + dir.y * c);
        d.lines.push((tip, tip - r * 2.5, Pen::Dim));
    }
}

/// Width below and height right of the paper box `lo`–`hi` (world sizes `world`).
fn overall_dims(d: &mut Drawing, lo: DVec2, hi: DVec2, world: DVec2, unit: LengthUnit) {
    if hi.x - lo.x > 1.0 {
        let y = hi.y + DIM_OFFSET;
        for x in [lo.x, hi.x] {
            d.lines.push((DVec2::new(x, hi.y + 1.0), DVec2::new(x, y + 1.5), Pen::Dim));
        }
        d.lines.push((DVec2::new(lo.x, y), DVec2::new(hi.x, y), Pen::Dim));
        arrow(d, DVec2::new(lo.x, y), DVec2::NEG_X);
        arrow(d, DVec2::new(hi.x, y), DVec2::X);
        let at = DVec2::new(f64::midpoint(lo.x, hi.x), y - 1.0);
        d.texts.push(Text2 { at, angle: 0.0, size: DIM_TEXT, lines: vec![dim_text(world.x, unit)], left: false });
    }
    if hi.y - lo.y > 1.0 {
        let x = hi.x + DIM_OFFSET;
        for y in [lo.y, hi.y] {
            d.lines.push((DVec2::new(hi.x + 1.0, y), DVec2::new(x + 1.5, y), Pen::Dim));
        }
        d.lines.push((DVec2::new(x, lo.y), DVec2::new(x, hi.y), Pen::Dim));
        arrow(d, DVec2::new(x, lo.y), DVec2::NEG_Y);
        arrow(d, DVec2::new(x, hi.y), DVec2::Y);
        let at = DVec2::new(x - 1.0, f64::midpoint(lo.y, hi.y));
        d.texts.push(Text2 { at, angle: -std::f64::consts::FRAC_PI_2, size: DIM_TEXT, lines: vec![dim_text(world.y, unit)], left: false });
    }
}

/// Rows of a parts table (header first) from the model's groups and components.
pub fn table_rows(model: &Model) -> Vec<[String; 5]> {
    let (parts, _) = varstotas_kernel::report::model_report(model);
    let mut rows = vec![["#".to_string(), tr!("layout-col-name"), tr!("layout-col-qty"), tr!("layout-col-size"), tr!("layout-col-material")]]; // i18n-ignore
    for (i, p) in parts.iter().enumerate() {
        let size = p.size.map(|v| dim_text(v, model.unit)).join(" × ");
        let name = format!("{}{}", "  ".repeat(p.depth), p.name);
        let material = if p.material.is_empty() { "-".to_string() } else { p.material.clone() };
        rows.push([(i + 1).to_string(), name, p.count.to_string(), size, material]);
    }
    if parts.is_empty() {
        rows.push([String::new(), tr!("layout-no-parts"), String::new(), String::new(), String::new()]);
    }
    rows
}

/// Column widths and the row height of a table, mm.
fn table_layout(rows: &[[String; 5]], size: f64) -> (Vec<f64>, f64) {
    let widths = (0..5).map(|c| rows.iter().map(|r| r[c].chars().count()).max().unwrap_or(1) as f64 * size * 0.62 + size * 1.2).collect();
    (widths, size * 2.0)
}

/// Size of a table on paper, mm.
pub fn table_size(model: &Model, t: &SheetTable) -> DVec2 {
    let rows = table_rows(model);
    let (w, h) = table_layout(&rows, t.size);
    DVec2::new(w.iter().sum(), h * rows.len() as f64)
}

fn table_drawing(d: &mut Drawing, model: &Model, t: &SheetTable) {
    let rows = table_rows(model);
    let (widths, h) = table_layout(&rows, t.size);
    let total = DVec2::new(widths.iter().sum(), h * rows.len() as f64);
    rect_lines(d, t.at, t.at + total, Pen::Profile);
    for r in 1..rows.len() {
        let y = t.at.y + h * r as f64;
        d.lines.push((DVec2::new(t.at.x, y), DVec2::new(t.at.x + total.x, y), if r == 1 { Pen::Profile } else { Pen::Edge }));
    }
    let mut x = t.at.x;
    for (c, w) in widths.iter().enumerate() {
        if c > 0 {
            d.lines.push((DVec2::new(x, t.at.y), DVec2::new(x, t.at.y + total.y), Pen::Edge));
        }
        for (r, row) in rows.iter().enumerate() {
            d.texts.push(text(DVec2::new(x + t.size * 0.6, t.at.y + h * r as f64 + h * 0.5 + t.size * 0.4), t.size, row[c].clone()));
        }
        x += w;
    }
}

/// Front, top and left views at a common standard scale (first-angle projection: top under
/// the front, left to its right) with overall dimensions, and an iso view in the free corner.
/// `scale` 1:N, or the largest standard scale that fits.
pub fn standard_views(model: &Model, sheet: &LayoutSheet, scale: Option<f64>) -> Vec<SheetViewport> {
    let Some(b) = model.bounds() else { return Vec::new() };
    let ext = (b.max - b.min).max(DVec3::splat(1.0));
    let centre = (b.min + b.max) * 0.5;
    let lo = DVec2::splat(BORDER + 5.0);
    let room = sheet.size - lo * 2.0 - DVec2::new(0.0, if sheet.title.is_some() { TITLE_ROOM } else { 0.0 });
    let m2 = 4.0 * VIEW_MARGIN;
    let fits = |n: f64| (ext.x + ext.y) / n + m2 <= room.x && (ext.z + ext.y) / n + m2 <= room.y;
    let n = scale.filter(|n| *n > 0.0).unwrap_or_else(|| SCALES.into_iter().find(|n| fits(*n)).unwrap_or_else(|| {
        let need = ((ext.x + ext.y) / (room.x - m2).max(10.0)).max((ext.z + ext.y) / (room.y - m2).max(10.0));
        (need / 1000.0).ceil() * 1000.0
    }));
    let view = |name: &str, v: StandardView, ortho: bool| {
        let mut c = Camera { target: centre, eye: centre + DVec3::Z, ..Default::default() };
        c.set_standard_view(v);
        c.ortho = ortho;
        // Well outside the model: the eye distance sets the parallel size.
        let d = ext.length() * 1.5 + 1000.0;
        c.eye = centre + (c.eye - centre).normalize() * d;
        let t = (c.fov_y_deg.to_radians() * 0.5).tan();
        c.ortho_height = 2.0 * d * t;
        camera_scene(name, &c)
    };
    let cell = |w: f64, h: f64| DVec2::new(w / n, h / n) + DVec2::splat(2.0 * VIEW_MARGIN);
    let (front, top, left) = (cell(ext.x, ext.z), cell(ext.x, ext.y), cell(ext.y, ext.z));
    let make = |min: DVec2, size: DVec2, s: Scene| {
        let mut vp = SheetViewport::new(min, size, s);
        vp.scale = Some(n);
        vp.auto_dims = true;
        vp
    };
    let mut out = vec![
        make(lo, front, view("Front", StandardView::Front, true)), // i18n-ignore
        make(lo + DVec2::new(0.0, front.y), top, view("Top", StandardView::Top, true)), // i18n-ignore
        make(lo + DVec2::new(front.x, 0.0), left, view("Left", StandardView::Left, true)), // i18n-ignore
    ];
    let iso_min = lo + front;
    let iso_size = DVec2::new(lo.x + room.x - iso_min.x, top.y.min(lo.y + room.y - iso_min.y));
    if iso_size.min_element() >= 30.0 {
        out.push(SheetViewport::new(iso_min, iso_size, view("Iso", StandardView::Iso, true))); // i18n-ignore
    }
    out
}

/// The scale for the title block: the one scale of the scaled viewports (fitted ones, like
/// an iso view, do not count), "As shown" when they differ, or "-".
fn sheet_scale(model: &Model, sheet: &LayoutSheet) -> String {
    let mut scales: Vec<f64> = sheet.viewports.iter().filter_map(|v| viewport_scale(model, v)).collect();
    scales.sort_by(f64::total_cmp);
    scales.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    match scales.as_slice() {
        [] => "-".into(),
        [n] => scale_label(*n),
        _ => tr!("layout-scale-as-shown"),
    }
}

/// Sheet `index` (of `total`) on paper; `file` names the document in an empty title.
pub fn sheet_drawing(model: &Model, sheet: &LayoutSheet, index: usize, total: usize, file: &str) -> Drawing {
    let mut d = Drawing { size: sheet.size, ..Default::default() };
    let b = DVec2::splat(BORDER);
    rect_lines(&mut d, b, sheet.size - b, Pen::Profile);
    for vp in &sheet.viewports {
        let v = viewport_drawing(model, vp);
        d.lines.extend(v.lines);
        d.fills.extend(v.fills);
        d.texts.extend(v.texts);
    }
    for t in &sheet.tables {
        table_drawing(&mut d, model, t);
    }
    for t in &sheet.texts {
        let text = if t.text.contains(CREDITS) { t.text.replace(CREDITS, &credits_text(model)) } else { t.text.clone() };
        d.texts.push(Text2 { at: t.at, angle: 0.0, size: t.size, lines: text.lines().map(str::to_string).collect(), left: true });
    }
    if let Some(tb) = &sheet.title {
        let w = TB_W.min(sheet.size.x - 2.0 * BORDER);
        let h: f64 = TB_ROWS.iter().sum();
        let (x0, y0) = (sheet.size.x - BORDER - w, sheet.size.y - BORDER - h);
        let (x1, y1) = (x0 + w, y0 + h);
        rect_lines(&mut d, DVec2::new(x0, y0), DVec2::new(x1, y1), Pen::Profile);
        let (ya, yb) = (y0 + TB_ROWS[0], y0 + TB_ROWS[0] + TB_ROWS[1]);
        let xm = x0 + w * 0.5;
        d.lines.push((DVec2::new(x0, ya), DVec2::new(x1, ya), Pen::Edge));
        d.lines.push((DVec2::new(x0, yb), DVec2::new(x1, yb), Pen::Edge));
        d.lines.push((DVec2::new(xm, ya), DVec2::new(xm, y1), Pen::Edge));
        let title = if tb.title.trim().is_empty() { file.to_string() } else { tb.title.clone() };
        d.texts.push(text(DVec2::new(x0 + 3.0, y0 + 7.0), 5.0, title));
        let small = |x: f64, y: f64, label: &str, v: &str| text(DVec2::new(x + 3.0, y - 2.2), 2.8, format!("{label}: {v}"));
        d.texts.push(small(x0, yb, &tr!("layout-tb-author"), &tb.author));
        d.texts.push(small(xm, yb, &tr!("layout-tb-date"), &tb.date));
        d.texts.push(small(x0, y1, &tr!("layout-tb-scale"), &sheet_scale(model, sheet)));
        d.texts.push(small(xm, y1, &tr!("layout-tb-sheet"), &tr!("layout-tb-sheet-of", n = index + 1, total = total)));
    }
    d
}

/// All sheets of the model as a PDF (a page per sheet).
pub fn sheets_pdf(model: &Model, file: &str) -> Vec<u8> {
    let n = model.sheets.len();
    let pages: Vec<Drawing> = model.sheets.iter().enumerate().map(|(i, s)| sheet_drawing(model, s, i, n, file)).collect();
    crate::vector_export::to_pdf_pages(&pages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use varstotas_kernel::glam::DVec3;
    use varstotas_kernel::layout::SheetText;

    fn boxed() -> Model {
        let mut m = Model::new();
        let r = m.root_mesh_mut();
        let f = r
            .add_face(&[DVec3::new(0., 0., 0.), DVec3::new(1000., 0., 0.), DVec3::new(1000., 500., 0.), DVec3::new(0., 500., 0.)])
            .unwrap();
        r.push_pull(f, -400.0, false).unwrap();
        m
    }

    fn top_view(target: DVec3) -> Camera {
        let mut c = Camera { target, eye: target + DVec3::Z * 5000.0, ..Default::default() };
        c.set_standard_view(StandardView::Top);
        c.set_ortho(true);
        c
    }

    #[test]
    fn scaled_viewport_draws_to_scale_and_clips() {
        let model = boxed();
        let cam = top_view(DVec3::new(500., 250., 0.));
        let mut vp = SheetViewport::new(DVec2::new(20., 20.), DVec2::new(100., 80.), camera_scene("Top", &cam));
        vp.scale = Some(20.0);
        let d = viewport_drawing(&model, &vp);
        // The 1000 mm side is 50 mm on paper at 1:20.
        let longest = d.lines.iter().map(|(a, b, _)| a.distance(*b)).fold(0.0, f64::max);
        assert!((longest - 50.0).abs() < 0.5, "{longest}");
        assert!(d.texts.iter().any(|t| t.lines[0] == "Top  1:20"));
        // At 1:8 the box (125 mm) overflows the 100 mm viewport: its long sides are clipped.
        vp.scale = Some(8.0);
        let d = viewport_drawing(&model, &vp);
        assert!(!d.lines.is_empty());
        for (a, b, _) in &d.lines {
            for p in [a, b] {
                assert!(p.x >= 20.0 - 1e-6 && p.x <= 120.0 + 1e-6 && p.y >= 20.0 - 1e-6 && p.y <= 100.0 + 1e-6, "{p}");
            }
        }
    }

    #[test]
    fn fitted_viewport_and_sheet_parts() {
        let mut model = boxed();
        let cam = Camera { target: DVec3::new(500., 250., 200.), ..Default::default() };
        let mut sheet = LayoutSheet::new("S1", DVec2::new(420., 297.));
        sheet.viewports.push(SheetViewport::new(DVec2::new(20., 20.), DVec2::new(200., 150.), camera_scene("Iso", &cam)));
        sheet.texts.push(SheetText { at: DVec2::new(30., 200.), size: 4.0, text: "Note one\nNote two".into() });
        sheet.texts.push(SheetText { at: DVec2::new(30., 250.), size: 2.5, text: CREDITS.into() });
        sheet.title.as_mut().unwrap().author = "Me".into();
        model.sheets.push(sheet.clone());
        let d = sheet_drawing(&model, &sheet, 0, 1, "box");
        assert_eq!(d.size, DVec2::new(420., 297.));
        // Fitted inside the viewport (3 mm padding).
        let vp_lines: Vec<_> = d.lines.iter().filter(|l| l.2 == Pen::Edge || l.2 == Pen::Profile).filter(|l| l.0.x < 230.0 && l.0.y < 180.0 && l.0.x > 15.0 && l.0.y > 15.0).collect();
        assert!(vp_lines.len() >= 9, "{}", vp_lines.len());
        let texts: Vec<String> = d.texts.iter().map(|t| t.lines.join("|")).collect();
        for want in ["box", "Author: Me", "Sheet: 1 of 1", "Scale: -", "Note one|Note two", "Iso", "Credits: no downloaded models"] {
            assert!(texts.iter().any(|t| t == want), "{want} in {texts:?}");
        }
        // Following a scene: the scene's camera wins.
        let mut s = camera_scene("Plan", &top_view(DVec3::new(500., 250., 0.)));
        s.name = "Plan".into();
        model.scenes.push(s);
        let mut vp = sheet.viewports[0].clone();
        vp.scene = Some("Plan".into());
        vp.scale = Some(10.0);
        assert!(viewport_scale(&model, &vp).is_some());
        let d = viewport_drawing(&model, &vp);
        let longest = d.lines.iter().map(|(a, b, _)| a.distance(*b)).fold(0.0, f64::max);
        assert!((longest - 100.0).abs() < 0.5, "{longest}");
        let pdf = sheets_pdf(&model, "box");
        assert!(pdf.starts_with(b"%PDF") && pdf.windows(10).filter(|w| w == b"/Type /Pag").count() >= 2);
    }

    #[test]
    fn standard_views_scale_dimensions_and_table() {
        let mut model = boxed();
        let sheet = LayoutSheet::new("S", DVec2::new(420., 297.));
        let views = standard_views(&model, &sheet, None);
        // 1000 + 500 wide and 400 + 500 tall with margins: 1:5 on A3 landscape.
        assert_eq!(views.len(), 4);
        assert!(views[..3].iter().all(|v| v.scale == Some(5.0) && v.auto_dims));
        assert_eq!(views[0].view.name, "Front");
        // Aligned: top under front (same column), left beside it (same row).
        assert_eq!(views[1].min.x, views[0].min.x);
        assert_eq!(views[1].size.x, views[0].size.x);
        assert_eq!(views[2].min.y, views[0].min.y);
        assert_eq!(views[2].size.y, views[0].size.y);
        let dims = |vp: &SheetViewport| -> Vec<String> {
            viewport_drawing(&model, vp).texts.iter().filter(|t| t.size == DIM_TEXT).map(|t| t.lines[0].clone()).collect()
        };
        assert_eq!(dims(&views[0]), ["1000", "400"]);
        assert_eq!(dims(&views[1]), ["1000", "500"]);
        assert_eq!(dims(&views[2]), ["500", "400"]);
        // The front view is 1000 / 5 = 200 mm wide on paper.
        let d = viewport_drawing(&model, &views[0]);
        let xs: Vec<f64> = d.lines.iter().filter(|l| l.2 != Pen::Dim).flat_map(|l| [l.0.x, l.1.x]).collect();
        let w = xs.iter().copied().fold(f64::MIN, f64::max) - xs.iter().copied().fold(f64::MAX, f64::min);
        assert!((w - 200.0).abs() < 0.5, "{w}");
        // A fixed scale is kept; a small sheet gets a smaller scale.
        assert!(standard_views(&model, &sheet, Some(10.0)).iter().take(3).all(|v| v.scale == Some(10.0)));
        let a4 = LayoutSheet::new("S", DVec2::new(210., 297.));
        assert_eq!(standard_views(&model, &a4, None)[0].scale, Some(15.0));
        // Parts table: the box made a group, twice the same part.
        let rows = table_rows(&model);
        assert_eq!(rows[1][1], "No groups or components");
        let mesh = model.root_mesh_mut();
        let ents: Vec<_> = mesh.faces.keys().map(varstotas_kernel::EntityRef::Face).chain(mesh.edges.keys().map(varstotas_kernel::EntityRef::Edge)).collect();
        let g = model.make_component(&[], &ents, "Crate").unwrap();
        let mut copy = model.root_mesh().instances[g].clone();
        copy.transform = varstotas_kernel::glam::DMat4::from_translation(DVec3::new(3000., 0., 0.));
        model.root_mesh_mut().instances.insert(copy);
        let rows = table_rows(&model);
        assert_eq!(rows.len(), 2, "{rows:?}");
        assert_eq!(rows[1], ["1", "Crate", "2", "1000 × 500 × 400", "-"].map(String::from));
        let mut sheet = sheet;
        let t = SheetTable { at: DVec2::new(20., 200.), size: 3.0 };
        let size = table_size(&model, &t);
        assert!((size.y - 12.0).abs() < 1e-9 && size.x > 40.0, "{size}");
        sheet.tables.push(t);
        let d = sheet_drawing(&model, &sheet, 0, 1, "x");
        assert!(d.texts.iter().any(|t| t.lines[0] == "1000 × 500 × 400"));
    }

    #[test]
    fn clip_segments() {
        let (lo, hi) = (DVec2::ZERO, DVec2::splat(10.0));
        assert_eq!(clip(DVec2::new(-5., 5.), DVec2::new(15., 5.), lo, hi), Some((DVec2::new(0., 5.), DVec2::new(10., 5.))));
        assert_eq!(clip(DVec2::new(-5., -5.), DVec2::new(-1., 20.), lo, hi), None);
        assert_eq!(clip(DVec2::new(2., 2.), DVec2::new(3., 3.), lo, hi), Some((DVec2::new(2., 2.), DVec2::new(3., 3.))));
    }
}
