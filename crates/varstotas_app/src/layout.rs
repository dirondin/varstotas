//! Layout sheets on paper (a light page layout): each viewport is the line drawing of a scene
//! (`vector_export::view_lines`) fitted into its rectangle or drawn to scale 1:N, clipped to
//! the rectangle; plus free texts, a border and a title block. Exported as PDF (a page per
//! sheet) or SVG (one sheet).

use crate::vector_export::{Drawing, Pen, Text2, VectorOptions, view_lines};
use varstotas_kernel::Model;
use varstotas_kernel::Scene;
use varstotas_kernel::glam::DVec2;
use varstotas_kernel::layout::{LayoutSheet, SheetViewport};
use crate::ui::unfold::scale_label;
use varstotas_render::Camera;

/// Border inset from the paper edge, mm.
pub const BORDER: f64 = 10.0;
/// Screen points per paper mm when a viewport's view is projected (precision only).
const PX_PER_MM: f64 = 4.0;
/// Title block: width and row heights, mm.
const TB_W: f64 = 180.0;
const TB_ROWS: [f64; 3] = [10.0, 7.0, 7.0];

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
    let placed = match (viewport_scale(model, vp), lines.bounds()) {
        (Some(n), _) => {
            let k = cam.pixel_size_at(cam.target, px) / n;
            Some(lines.on_paper(Drawing::default(), move |p| centre + (p - px * 0.5) * k))
        }
        (None, Some((a, b))) => {
            let pad = 3.0f64.min(vp.size.min_element() * 0.1);
            let ext = (b - a).max(DVec2::splat(1e-6));
            let k = ((vp.size.x - 2.0 * pad) / ext.x).min((vp.size.y - 2.0 * pad) / ext.y).max(1e-9);
            let mid = (a + b) * 0.5;
            Some(lines.on_paper(Drawing::default(), move |p| centre + (p - mid) * k))
        }
        (None, None) => None,
    };
    if let Some(d) = placed {
        out.lines.extend(d.lines.into_iter().filter_map(|(a, b, pen)| clip(a, b, lo, hi).map(|(a, b)| (a, b, pen))));
        let inside = |p: DVec2| p.x >= lo.x && p.y >= lo.y && p.x <= hi.x && p.y <= hi.y;
        out.fills.extend(d.fills.into_iter().filter(|f| f.iter().all(|p| inside(*p))));
        out.texts.extend(d.texts.into_iter().filter(|t| inside(t.at)));
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

/// The scale for the title block: one common scale, "As shown", or "-".
fn sheet_scale(model: &Model, sheet: &LayoutSheet) -> String {
    let mut scales: Vec<f64> = sheet.viewports.iter().filter_map(|v| viewport_scale(model, v)).collect();
    scales.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    match scales.as_slice() {
        [] => "-".into(),
        [n] if sheet.viewports.iter().all(|v| viewport_scale(model, v).is_some()) => scale_label(*n),
        _ => "As shown".into(),
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
    for t in &sheet.texts {
        d.texts.push(Text2 { at: t.at, angle: 0.0, size: t.size, lines: t.text.lines().map(str::to_string).collect(), left: true });
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
        d.texts.push(small(x0, yb, "Author", &tb.author));
        d.texts.push(small(xm, yb, "Date", &tb.date));
        d.texts.push(small(x0, y1, "Scale", &sheet_scale(model, sheet)));
        d.texts.push(small(xm, y1, "Sheet", &format!("{} of {total}", index + 1)));
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
    use varstotas_render::StandardView;

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
        sheet.title.as_mut().unwrap().author = "Me".into();
        model.sheets.push(sheet.clone());
        let d = sheet_drawing(&model, &sheet, 0, 1, "box");
        assert_eq!(d.size, DVec2::new(420., 297.));
        // Fitted inside the viewport (3 mm padding).
        let vp_lines: Vec<_> = d.lines.iter().filter(|l| l.2 == Pen::Edge || l.2 == Pen::Profile).filter(|l| l.0.x < 230.0 && l.0.y < 180.0 && l.0.x > 15.0 && l.0.y > 15.0).collect();
        assert!(vp_lines.len() >= 9, "{}", vp_lines.len());
        let texts: Vec<String> = d.texts.iter().map(|t| t.lines.join("|")).collect();
        for want in ["box", "Author: Me", "Sheet: 1 of 1", "Scale: -", "Note one|Note two", "Iso"] {
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
    fn clip_segments() {
        let (lo, hi) = (DVec2::ZERO, DVec2::splat(10.0));
        assert_eq!(clip(DVec2::new(-5., 5.), DVec2::new(15., 5.), lo, hi), Some((DVec2::new(0., 5.), DVec2::new(10., 5.))));
        assert_eq!(clip(DVec2::new(-5., -5.), DVec2::new(-1., 20.), lo, hi), None);
        assert_eq!(clip(DVec2::new(2., 2.), DVec2::new(3., 3.), lo, hi), Some((DVec2::new(2., 2.), DVec2::new(3., 3.))));
    }
}
