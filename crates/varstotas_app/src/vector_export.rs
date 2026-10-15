//! File → Export → 2D Drawing (SVG / PDF): the current view as a line drawing on paper.
//!
//! Edges are projected with the camera; hidden parts are found by casting a ray at sample
//! points along each edge (`Model::raycast`, which honours the active section cut). Profiles
//! (outline edges: one face, or one face toward the viewer and one away) are thicker; soft
//! edges only show as profiles. The active cut adds thick section lines (and a grey fill).
//! Dimensions and text notes are redrawn on paper (ticks, text along the line).
//! In parallel projection the drawing can be to scale (1:N); otherwise it fits an A4 page.

use crate::document::guarded;
use std::collections::HashSet;
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::{Model, Ray, tessellate};
use varstotas_render::Camera;
use std::fmt::Write as _;

/// What goes into the drawing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VectorOptions {
    /// `Some(n)`: scale 1:n (parallel projection only); `None`: fit to an A4 page.
    pub scale: Option<f64>,
    pub profiles: bool,
    /// Hidden edges as thin dashed lines.
    pub hidden: bool,
    pub dims: bool,
    pub sections: bool,
}

impl Default for VectorOptions {
    fn default() -> Self {
        Self { scale: None, profiles: true, hidden: false, dims: true, sections: true }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pen {
    Edge,
    Profile,
    Hidden,
    Section,
    Dim,
}

impl Pen {
    /// Line width in mm.
    fn width(self) -> f64 {
        match self {
            Pen::Edge => 0.25,
            Pen::Profile => 0.5,
            Pen::Hidden => 0.18,
            Pen::Section => 0.7,
            Pen::Dim => 0.18,
        }
    }
    fn gray(self) -> f64 {
        if self == Pen::Hidden { 0.45 } else { 0.0 }
    }
}

#[derive(Clone, Debug)]
pub struct Text2 {
    /// Anchor (mm, y down): the centre of the first line's baseline, or its start (`left`).
    pub at: DVec2,
    /// Rotation, radians, clockwise on paper (y down).
    pub angle: f64,
    pub size: f64,
    pub lines: Vec<String>,
    pub left: bool,
}

/// A drawing on paper: millimetres, origin top-left, y down.
#[derive(Clone, Debug, Default)]
pub struct Drawing {
    pub size: DVec2,
    pub lines: Vec<(DVec2, DVec2, Pen)>,
    pub fills: Vec<Vec<DVec2>>,
    pub texts: Vec<Text2>,
}

const MARGIN: f64 = 10.0;
const TEXT_MM: f64 = 2.5;
const A4: DVec2 = DVec2::new(297.0, 210.0);

/// Visible runs of the segment `a`–`b` (world) as parameter intervals, plus hidden ones.
fn visibility(model: &Model, cam: &Camera, vp: DVec2, a: DVec3, b: DVec3, px_len: f64) -> Vec<(f64, f64, bool)> {
    let n = ((px_len / 6.0).ceil() as usize).clamp(1, 96);
    let fwd = cam.basis().forward;
    let section = model.active_section().cloned();
    let mut runs: Vec<(f64, f64, bool)> = Vec::new();
    for i in 0..n {
        let t = (i as f64 + 0.5) / n as f64;
        let p = a.lerp(b, t);
        if section.as_ref().is_some_and(|s| s.dist(p) < -1e-6) {
            continue;
        }
        let (origin, dir, tp) = if cam.ortho {
            let d = (p - cam.eye).dot(fwd);
            (p - fwd * d, fwd, d)
        } else {
            let v = p - cam.eye;
            (cam.eye, v.normalize_or_zero(), v.length())
        };
        if tp <= 0.0 {
            continue;
        }
        let tol = (cam.pixel_size_at(p, vp) * 1.5).max(tp * 1e-6).max(0.01);
        let hit = guarded(|| model.raycast(&Ray { origin, dir })).ok().flatten();
        let visible = hit.is_none_or(|h| h.t >= tp - tol);
        let (t0, t1) = (i as f64 / n as f64, (i + 1) as f64 / n as f64);
        match runs.last_mut() {
            Some(r) if r.2 == visible && (r.1 - t0).abs() < 1e-12 => r.1 = t1,
            _ => runs.push((t0, t1, visible)),
        }
    }
    runs
}

/// Is the edge a profile (outline) seen from the camera?
fn is_profile(model: &Model, cam: &Camera, path: &[varstotas_kernel::InstanceId], edge: varstotas_kernel::EdgeId, mid: DVec3) -> bool {
    let Some(mesh) = model.mesh_at(path) else { return false };
    let faces = mesh.edge_faces(edge);
    if faces.len() == 1 {
        return true;
    }
    if faces.len() != 2 {
        return false;
    }
    let nx = model.transform_at(path).inverse().transpose();
    let view = if cam.ortho { cam.basis().forward } else { mid - cam.eye };
    let side = |f| mesh.faces.get(f).map(|f| nx.transform_vector3(f.normal).dot(view) < 0.0);
    side(faces[0]) != side(faces[1])
}

/// Build the drawing of the current view.
pub fn build(model: &Model, cam: &Camera, vp: DVec2, opts: &VectorOptions) -> Drawing {
    // 1. Screen-space strokes (logical points, y down).
    let mut strokes: Vec<(DVec2, DVec2, Pen)> = Vec::new();
    let project = |p: DVec3| cam.world_to_screen(p, vp);
    let add = |a: DVec3, b: DVec3, pen: Pen, strokes: &mut Vec<(DVec2, DVec2, Pen)>| {
        let (Some(sa), Some(sb)) = (project(a), project(b)) else { return };
        let len = sa.distance(sb);
        if len < 1e-3 {
            return;
        }
        for (t0, t1, vis) in visibility(model, cam, vp, a, b, len) {
            let pen = if vis { pen } else { Pen::Hidden };
            if !vis && !opts.hidden {
                continue;
            }
            let (Some(p), Some(q)) = (project(a.lerp(b, t0)), project(a.lerp(b, t1))) else { continue };
            strokes.push((p, q, pen));
        }
    };
    let tess = tessellate(model);
    let mut seen = HashSet::new();
    for e in &tess.edges {
        // Same edge twice (e.g. two instances in the same place): once.
        let key = [e.a, e.b].map(|p| [p.x, p.y, p.z].map(|c| (c * 100.0).round() as i64));
        let key = if key[0] <= key[1] { key } else { [key[1], key[0]] };
        if !seen.insert(key) {
            continue;
        }
        let profile = opts.profiles && is_profile(model, cam, &e.owner_path, e.edge, (e.a + e.b) * 0.5);
        if e.soft && !profile {
            continue;
        }
        add(e.a, e.b, if profile { Pen::Profile } else { Pen::Edge }, &mut strokes);
    }
    let mut fills_scr: Vec<Vec<DVec2>> = Vec::new();
    if opts.sections
        && model.section_display.cuts
        && let Some(plane) = model.active_section().cloned()
    {
        let segs = model.section_cut(&plane);
        if model.section_display.fill {
            for t in varstotas_kernel::section::section_fill(&plane, &segs) {
                if let (Some(a), Some(b), Some(c)) = (project(t[0]), project(t[1]), project(t[2])) {
                    fills_scr.push(vec![a, b, c]);
                }
            }
        }
        for s in &segs {
            add(s[0], s[1], Pen::Section, &mut strokes);
        }
    }
    // Dimensions: (a, b, line a, line b, label) / notes: (anchor, text, label) on screen.
    let unit = model.unit;
    let mut dims: Vec<[DVec2; 4]> = Vec::new();
    let mut dim_labels: Vec<String> = Vec::new();
    let mut notes: Vec<(DVec2, DVec2, String)> = Vec::new();
    if opts.dims {
        for (_, _, g) in model.all_dimensions_world() {
            let label = g.label(unit);
            if g.leader {
                if let (Some(a), Some(t)) = (project(g.a), project(g.text_pos())) {
                    notes.push((a, t, label));
                }
                continue;
            }
            let (p, q) = g.line();
            if let (Some(a), Some(b), Some(p), Some(q)) = (project(g.a), project(g.b), project(p), project(q)) {
                dims.push([a, b, p, q]);
                dim_labels.push(label);
            }
        }
    }
    // 2. Paper: millimetres per screen point and the content box.
    let mut lo = DVec2::splat(f64::INFINITY);
    let mut hi = DVec2::splat(f64::NEG_INFINITY);
    let mut grow = |p: DVec2| {
        lo = lo.min(p);
        hi = hi.max(p);
    };
    strokes.iter().for_each(|(a, b, _)| {
        grow(*a);
        grow(*b);
    });
    fills_scr.iter().flatten().for_each(|p| grow(*p));
    dims.iter().flatten().for_each(|p| grow(*p));
    notes.iter().for_each(|(a, t, _)| {
        grow(*a);
        grow(*t);
    });
    if !lo.is_finite() {
        return Drawing { size: A4, ..Default::default() };
    }
    let ext = (hi - lo).max(DVec2::splat(1e-6));
    let scale = opts.scale.filter(|n| cam.ortho && *n > 0.0);
    let k = match scale {
        // Model mm per point / N.
        Some(n) => cam.pixel_size_at(cam.target, vp) / n,
        None => {
            let page = if ext.x >= ext.y { A4 } else { DVec2::new(A4.y, A4.x) };
            // Room for dimension text around the drawing.
            ((page.x - 2.0 * MARGIN - 10.0) / ext.x).min((page.y - 2.0 * MARGIN - 10.0) / ext.y)
        }
    };
    let pad = DVec2::splat(MARGIN + 5.0);
    let to_paper = |p: DVec2| (p - lo) * k + pad;
    let mut d = Drawing { size: ext * k + pad * 2.0, ..Default::default() };
    if scale.is_none() {
        // Centre on the A4 page.
        let page = if ext.x >= ext.y { A4 } else { DVec2::new(A4.y, A4.x) };
        let shift = ((page - d.size) * 0.5).max(DVec2::ZERO);
        d.size = d.size.max(page);
        let tp = to_paper;
        return finish(d, &strokes, &fills_scr, &dims, &dim_labels, &notes, move |p| tp(p) + shift, None);
    }
    let label = scale.map(|n| format!("Scale 1:{}", trim(n)));
    d.size.y += if label.is_some() { 6.0 } else { 0.0 };
    finish(d, &strokes, &fills_scr, &dims, &dim_labels, &notes, to_paper, label)
}

fn trim(n: f64) -> String {
    let s = format!("{n:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[allow(clippy::too_many_arguments)]
fn finish(
    mut d: Drawing,
    strokes: &[(DVec2, DVec2, Pen)],
    fills: &[Vec<DVec2>],
    dims: &[[DVec2; 4]],
    dim_labels: &[String],
    notes: &[(DVec2, DVec2, String)],
    to_paper: impl Fn(DVec2) -> DVec2,
    scale_label: Option<String>,
) -> Drawing {
    d.fills = fills.iter().map(|f| f.iter().map(|p| to_paper(*p)).collect()).collect();
    d.lines = strokes.iter().map(|(a, b, pen)| (to_paper(*a), to_paper(*b), *pen)).collect();
    for (pts, label) in dims.iter().zip(dim_labels) {
        let [a, b, p, q] = pts.map(&to_paper);
        // Extension lines: a small gap at the model, past the dimension line.
        for (from, to) in [(a, p), (b, q)] {
            let v = to - from;
            let len = v.length();
            if len > 1.0 {
                let u = v / len;
                d.lines.push((from + u * 1.0, to + u * 1.5, Pen::Dim));
            }
        }
        let along = (q - p).normalize_or(DVec2::X);
        d.lines.push((p, q, Pen::Dim));
        // Architectural ticks.
        let tick = DVec2::new(along.x + along.y, along.y - along.x).normalize() * 1.2;
        for e in [p, q] {
            d.lines.push((e - tick, e + tick, Pen::Dim));
        }
        // Readable text above the line.
        let mut dir = along;
        if dir.x < -1e-9 || (dir.x.abs() <= 1e-9 && dir.y > 0.0) {
            dir = -dir;
        }
        let up = DVec2::new(dir.y, -dir.x);
        d.texts.push(Text2 {
            at: (p + q) * 0.5 + up * 1.0,
            angle: dir.y.atan2(dir.x),
            size: TEXT_MM,
            lines: vec![label.clone()],
            left: false,
        });
    }
    for (a, t, label) in notes {
        let (a, t) = (to_paper(*a), to_paper(*t));
        if a.distance(t) > 0.5 {
            d.lines.push((a, t, Pen::Dim));
            d.fills.push((0..8).map(|i| a + DVec2::from_angle(i as f64 * std::f64::consts::TAU / 8.0) * 0.5).collect());
        }
        let lines: Vec<String> = label.lines().map(str::to_string).collect();
        let n = lines.len().max(1) as f64;
        let left = t.x >= a.x;
        d.texts.push(Text2 {
            at: t + DVec2::new(if left { 1.0 } else { -1.0 }, TEXT_MM * 0.35 - (n - 1.0) * TEXT_MM * 0.6),
            angle: 0.0,
            size: TEXT_MM,
            lines,
            left,
        });
    }
    if let Some(label) = scale_label {
        d.texts.push(Text2 { at: DVec2::new(MARGIN, d.size.y - MARGIN * 0.5), angle: 0.0, size: TEXT_MM, lines: vec![label], left: true });
    }
    d
}

fn xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn n(v: f64) -> String {
    trim((v * 1000.0).round() / 1000.0)
}

/// SVG text (user units = mm).
pub fn to_svg(d: &Drawing) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}mm\" height=\"{h}mm\" viewBox=\"0 0 {w} {h}\">",
        w = n(d.size.x),
        h = n(d.size.y)
    );
    let _ = writeln!(s, "<rect width=\"100%\" height=\"100%\" fill=\"white\"/>");
    if !d.fills.is_empty() {
        let _ = writeln!(s, "<g fill=\"#d4d4d4\" stroke=\"#d4d4d4\" stroke-width=\"0.05\">");
        for f in &d.fills {
            let pts: Vec<String> = f.iter().map(|p| format!("{},{}", n(p.x), n(p.y))).collect();
            let _ = writeln!(s, "<polygon points=\"{}\"/>", pts.join(" "));
        }
        s.push_str("</g>\n");
    }
    for pen in [Pen::Hidden, Pen::Edge, Pen::Section, Pen::Profile, Pen::Dim] {
        let lines: Vec<_> = d.lines.iter().filter(|l| l.2 == pen).collect();
        if lines.is_empty() {
            continue;
        }
        let g = (pen.gray() * 255.0) as u8;
        let dash = if pen == Pen::Hidden { " stroke-dasharray=\"1.5 1\"" } else { "" };
        let _ = writeln!(
            s,
            "<g fill=\"none\" stroke=\"#{g:02x}{g:02x}{g:02x}\" stroke-width=\"{}\" stroke-linecap=\"round\"{dash}>",
            n(pen.width())
        );
        let mut path = String::new();
        for (a, b, _) in lines {
            let _ = write!(path, "M{} {}L{} {}", n(a.x), n(a.y), n(b.x), n(b.y));
        }
        let _ = writeln!(s, "<path d=\"{path}\"/>\n</g>");
    }
    for t in &d.texts {
        let anchor = if t.left { "start" } else { "middle" };
        let _ = write!(
            s,
            "<text x=\"{x}\" y=\"{y}\" font-family=\"Helvetica, Arial, sans-serif\" font-size=\"{}\" text-anchor=\"{anchor}\"",
            n(t.size),
            x = n(t.at.x),
            y = n(t.at.y)
        );
        if t.angle.abs() > 1e-9 {
            let _ = write!(s, " transform=\"rotate({} {} {})\"", n(t.angle.to_degrees()), n(t.at.x), n(t.at.y));
        }
        s.push('>');
        for (i, l) in t.lines.iter().enumerate() {
            if i == 0 {
                s.push_str(&xml(l));
            } else {
                let _ = write!(s, "<tspan x=\"{}\" dy=\"{}\">{}</tspan>", n(t.at.x), n(t.size * 1.2), xml(l));
            }
        }
        s.push_str("</text>\n");
    }
    s.push_str("</svg>\n");
    s
}

/// A text string for the standard Helvetica font (WinAnsi): Latin-1 kept, others `?`.
fn pdf_string(s: &str) -> Vec<u8> {
    let mut out = vec![b'('];
    for c in s.chars() {
        let b = match c as u32 {
            0x20..=0x7E => c as u8,
            0xA0..=0xFF => c as u32 as u8,
            _ => b'?',
        };
        if matches!(b, b'(' | b')' | b'\\') {
            out.push(b'\\');
        }
        out.push(b);
    }
    out.push(b')');
    out
}

/// PDF bytes (one page, vector lines, Helvetica text).
pub fn to_pdf(d: &Drawing) -> Vec<u8> {
    let pt = 72.0 / 25.4;
    let h = d.size.y;
    // Paper mm (y down) → PDF points (y up).
    let p = |v: DVec2| (v.x * pt, (h - v.y) * pt);
    let mut c: Vec<u8> = Vec::new();
    fn w(c: &mut Vec<u8>, s: String) {
        c.extend_from_slice(s.as_bytes());
    }
    w(&mut c, "1 J 1 j\n".into());
    if !d.fills.is_empty() {
        w(&mut c, "0.83 g 0.83 G 0.15 w\n".into());
        for f in &d.fills {
            for (i, v) in f.iter().enumerate() {
                let (x, y) = p(*v);
                w(&mut c, format!("{} {} {}\n", n(x), n(y), if i == 0 { "m" } else { "l" }));
            }
            w(&mut c, "b\n".into());
        }
    }
    for pen in [Pen::Hidden, Pen::Edge, Pen::Section, Pen::Profile, Pen::Dim] {
        let lines: Vec<_> = d.lines.iter().filter(|l| l.2 == pen).collect();
        if lines.is_empty() {
            continue;
        }
        let dash = if pen == Pen::Hidden { format!("[{} {}] 0 d", n(1.5 * pt), n(pt)) } else { "[] 0 d".into() };
        w(&mut c, format!("{} G {} w {dash}\n", n(pen.gray()), n(pen.width() * pt)));
        for (a, b, _) in lines {
            let ((x0, y0), (x1, y1)) = (p(*a), p(*b));
            w(&mut c, format!("{} {} m {} {} l\n", n(x0), n(y0), n(x1), n(y1)));
        }
        w(&mut c, "S\n".into());
    }
    w(&mut c, "0 g\n".into());
    for t in &d.texts {
        let size = t.size * pt;
        let (cos, sin) = (t.angle.cos(), -t.angle.sin());
        for (i, line) in t.lines.iter().enumerate() {
            // Helvetica averages ~0.52 em per character: centre approximately.
            let width = if t.left { 0.0 } else { line.chars().count() as f64 * 0.52 * size };
            let at = t.at + DVec2::new(-t.angle.sin(), t.angle.cos()) * (i as f64 * t.size * 1.2);
            let (x, y) = p(at);
            let (x, y) = (x - cos * width * 0.5, y - sin * width * 0.5);
            w(&mut c, format!("BT /F1 {} Tf {} {} {} {} {} {} Tm ", n(size), n(cos), n(sin), n(-sin), n(cos), n(x), n(y)));
            c.extend(pdf_string(line));
            c.extend_from_slice(b" Tj ET\n");
        }
    }
    let content = c;
    let mut out: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::new();
    let mut obj = |out: &mut Vec<u8>, body: &[u8]| {
        offsets.push(out.len());
        out.extend(format!("{} 0 obj\n", offsets.len()).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    };
    obj(&mut out, b"<< /Type /Catalog /Pages 2 0 R >>");
    obj(&mut out, b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>");
    let page = format!(
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>",
        n(d.size.x * pt),
        n(d.size.y * pt)
    );
    obj(&mut out, page.as_bytes());
    let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
    stream.extend_from_slice(&content);
    stream.extend_from_slice(b"\nendstream");
    obj(&mut out, &stream);
    obj(&mut out, b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>");
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1).as_bytes());
    for o in &offsets {
        out.extend(format!("{o:010} 00000 n \n").as_bytes());
    }
    out.extend(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", offsets.len() + 1).as_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use varstotas_render::StandardView;

    #[test]
    fn top_view_of_a_square_to_scale() {
        let mut model = Model::new();
        model.root_mesh_mut().add_face(&[
            DVec3::new(-1000.0, -1000.0, 0.0),
            DVec3::new(1000.0, -1000.0, 0.0),
            DVec3::new(1000.0, 1000.0, 0.0),
            DVec3::new(-1000.0, 1000.0, 0.0),
        ]);
        let vp = DVec2::new(800.0, 600.0);
        let mut cam = Camera { target: DVec3::ZERO, ..Camera::default() };
        cam.set_standard_view(StandardView::Top);
        cam.set_ortho(true);
        let cam = crate::nav::zoom_extents_camera(&model, &cam, vp.x / vp.y);
        let d = build(&model, &cam, vp, &VectorOptions { scale: Some(50.0), ..Default::default() });
        // Four outline edges, each 2000 mm → 40 mm on paper.
        let outline: Vec<_> = d.lines.iter().filter(|l| l.2 == Pen::Profile).collect();
        assert_eq!(outline.len(), 4);
        for (a, b, _) in outline {
            assert!(((*a - *b).length() - 40.0).abs() < 0.01, "{a} {b}");
        }
        assert!(d.texts.iter().any(|t| t.lines.iter().any(|l| l == "Scale 1:50")));
        let svg = to_svg(&d);
        assert!(svg.starts_with("<?xml") && svg.contains("<svg") && svg.trim_end().ends_with("</svg>"));
        let pdf = to_pdf(&d);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF\n"));
    }
}
