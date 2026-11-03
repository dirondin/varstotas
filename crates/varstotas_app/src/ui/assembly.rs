//! Window → Assembly (track W): the exploded view, the assembly steps
//! (`varstotas_kernel::assembly`) and the instructions booklet. The steps are suggested from
//! the contact graph and edited here: moved up or down, joined, split, given a note. Clicking a
//! step shows it (later parts hidden, its own selected); "Scenes from steps" keeps one scene per
//! step; the booklet (PDF, A4 portrait) has a cover, the parts and hardware, then a page per
//! step drawn from the current view with the earlier parts grey. A fastener step (no parts, the
//! joints whose dowels, shelf pins or cam bolts go in before the next part) is suggested or split
//! out here ("Fasteners first"); each step's page outlines its fasteners with arrows into their
//! holes.

use crate::app::VarstotasApp;
use crate::ui::theme;
use crate::vector_export::{Drawing, Pen, Text2, VectorOptions, view_lines};
use egui::{Context, RichText, Ui, vec2};
use std::path::Path;
use varstotas_i18n::tr;
use varstotas_kernel::assembly::{SceneAssembly, Step, Unit, fasteners_first, suggest_all, suggest_steps};
use varstotas_kernel::cutlist::CutPlan;
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::{ContextPath, DefId, EntityRef, Model};
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

/// The parts a step adds: "2 × Side (1)", numbered as the cut list numbers them, and its
/// sub-assemblies: "3 × Drawer (A)", lettered as `subs` orders them.
pub(crate) fn step_parts(model: &Model, plan: &CutPlan, step: &Step, subs: &[(DefId, usize)]) -> Vec<String> {
    let units = varstotas_kernel::assembly::units_within(model, &model.assembly.within);
    let cut = varstotas_kernel::cutlist::cut_parts(model);
    let mut by: Vec<(Unit, usize)> = Vec::new();
    // a group's every unit (a group the steps add whole that is no sub-assembly)
    for (_, u) in units.iter().filter(|(q, _)| step.parts.iter().any(|p| q.starts_with(p))) {
        match by.iter_mut().find(|(k, _)| k == u) {
            Some(e) => e.1 += 1,
            None => by.push((*u, 1)),
        }
    }
    by.into_iter()
        .map(|(u, n)| {
            let i = match u {
                Unit::Part(i) => i,
                Unit::Sub(d) => {
                    let name = model.defs.get(d).map(|x| x.name.clone()).unwrap_or_default();
                    let k = subs.iter().position(|s| s.0 == d).map(|k| format!(" ({})", letter(k))).unwrap_or_default();
                    return format!("{n} × {name}{k}");
                }
            };
            let part = &cut[i];
            // its row: the same name, material and size (equal names can differ in size)
            let same = |p: &&varstotas_kernel::cutlist::CutPart| p.name == part.name && p.material == part.material;
            let row = plan.parts.iter().position(|p| same(&p) && p.size == part.size).or_else(|| plan.parts.iter().position(|p| same(&p)));
            let no = row.map(|k| format!(" ({})", k + 1)).unwrap_or_default();
            format!("{n} × {}{no}", part.name)
        })
        .collect()
}

/// The hardware a step uses: the joints given to it, and those whose later part it adds.
pub(crate) fn step_hardware(model: &Model, step: usize) -> Vec<String> {
    let mut by: Vec<(String, usize)> = Vec::new();
    for (i, j) in model.placed_joints.iter().enumerate() {
        if model.fastener_step(i) != Some(step) {
            continue;
        }
        let c = varstotas_kernel::joinery::joint_contact(model, j);
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

/// A step's fasteners on its picture (screen points): each body's outline, and an arrow along
/// each round one, from outside its outer end towards the part it goes into.
type Outline = Vec<DVec2>;
type Arrow = (DVec2, DVec2);
fn fastener_lines(m: &Model, step: usize, cam: &Camera, px: DVec2) -> (Vec<Outline>, Vec<Arrow>) {
    let (mut outlines, mut arrows) = (Vec::new(), Vec::new());
    for f in &m.fasteners {
        if m.fastener_step(f.joint) != Some(step) {
            continue;
        }
        let Some(t) = m.fastener_transform(f) else { continue };
        for s in &f.solids {
            let pts: Vec<DVec2> = s.points().into_iter().filter_map(|p| cam.world_to_screen(t.transform_point3(p), px)).collect();
            outlines.push(hull(pts));
        }
        let Some(&varstotas_kernel::fasteners::Solid::Round { at, dir, len, .. }) = f.solids.first() else { continue };
        let centre = m.fastener_anchor(f).and_then(|(path, _)| path.split_last().and_then(|(&id, parent)| m.instance_world_bounds(parent, id))).map(|b| (b.min + b.max) * 0.5);
        let (e0, e1) = (t.transform_point3(at), t.transform_point3(at + dir * len));
        let (inner, outer) = match centre {
            Some(c) if e1.distance(c) < e0.distance(c) => (e1, e0),
            _ => (e0, e1),
        };
        let u = (outer - inner).normalize_or_zero();
        // at least a few percent of the picture long, however short the axis looks from here
        let reach = (len * 2.0).max(80.0);
        if let (Some(tail), Some(head)) = (cam.world_to_screen(outer + u * reach, px), cam.world_to_screen(outer + u * reach * 0.15, px)) {
            let along = tail - head;
            let min = px.x * 0.05;
            if along.length() > 1e-3 {
                arrows.push((head + along.normalize() * along.length().max(min), head));
            }
        }
    }
    (outlines, arrows)
}

/// The convex hull of `pts`, counter-clockwise.
fn hull(mut pts: Vec<DVec2>) -> Vec<DVec2> {
    pts.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    pts.dedup();
    if pts.len() < 3 {
        return pts;
    }
    let cross = |o: DVec2, a: DVec2, b: DVec2| (a - o).perp_dot(b - o);
    let mut h: Vec<DVec2> = Vec::new();
    for pass in 0..2 {
        let start = h.len();
        let it: Box<dyn Iterator<Item = &DVec2>> = if pass == 0 { Box::new(pts.iter()) } else { Box::new(pts.iter().rev()) };
        for &p in it {
            while h.len() >= start + 2 && cross(h[h.len() - 2], h[h.len() - 1], p) <= 0.0 {
                h.pop();
            }
            h.push(p);
        }
        h.pop();
    }
    h
}

/// The side of a step's fastener close-up (mm).
const CLOSE_UP: f64 = 60.0;

/// Segment `p`–`q` within the box `lo`–`hi` (Liang–Barsky), if any of it is.
fn clip(p: DVec2, q: DVec2, lo: DVec2, hi: DVec2) -> Option<(DVec2, DVec2)> {
    let d = q - p;
    let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
    for (dp, a, b) in [(d.x, p.x - lo.x, hi.x - p.x), (d.y, p.y - lo.y, hi.y - p.y)] {
        for (num, den) in [(a, -dp), (b, dp)] {
            if den.abs() < 1e-12 {
                if num < 0.0 {
                    return None;
                }
            } else {
                let t = num / den;
                if den < 0.0 {
                    t0 = t0.max(t);
                } else {
                    t1 = t1.min(t);
                }
            }
        }
    }
    (t0 < t1).then(|| (p + d * t0, p + d * t1))
}

/// A document's name as a title: a file name's underscores as spaces, its first letter capital
/// ("chest_of_drawers" → "Chest of drawers").
fn title(doc: &str) -> String {
    let t = doc.replace('_', " ");
    let mut c = t.chars();
    c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
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

/// The sub-assemblies the steps add (each with steps of its own), in the order they are put
/// together (one inside another first), each with how many are made.
pub(crate) fn sub_assemblies(model: &Model) -> Vec<(DefId, usize)> {
    fn walk(m: &Model, prefix: &[varstotas_kernel::InstanceId], steps: &[Step], out: &mut Vec<(DefId, usize)>) {
        for p in steps.iter().flat_map(|s| &s.parts) {
            let full: ContextPath = prefix.iter().chain(p).copied().collect();
            let Some(d) = m.def_at(&full).filter(|&d| m.defs.get(d).is_some_and(|x| !x.steps.is_empty())) else { continue };
            walk(m, &full, &m.defs[d].steps, out);
            match out.iter_mut().find(|e| e.0 == d) {
                Some(e) => e.1 += 1,
                None => out.push((d, 1)),
            }
        }
    }
    let mut out = Vec::new();
    walk(model, &[], &model.assembly.steps, &mut out);
    out
}

/// A sub-assembly's letter in the booklet: A, B, … (then AA, AB, …).
fn letter(k: usize) -> String {
    let a = |i: usize| char::from(b'A' + (i % 26) as u8);
    if k < 26 { a(k).to_string() } else { format!("{}{}", a(k / 26 - 1), a(k)) }
}

/// The booklet: a cover, the parts and hardware, the pages of each sub-assembly (put together
/// first, a drawer), then a page per step. Pictures are drawn with `cam`, every step framed
/// alike (by the whole model, or the sub-assembly, exploded as now).
pub(crate) fn assembly_pages(model: &Model, plan: &CutPlan, cam: &Camera, doc: &str) -> Vec<Drawing> {
    let view = model.assembly.view();
    let mut model = model.clone();
    model.set_assembly_within(Vec::new());
    model.set_assembly_view(view);
    let model = &model;
    let a = &model.assembly;
    let rect = (DVec2::new(MARGIN, 32.0), DVec2::new(PAGE.x - 2.0 * MARGIN, 175.0));
    let px = rect.1 * 4.0;
    let opts = VectorOptions { profiles: true, hidden: false, dims: false, sections: false, ..Default::default() };
    let mut all = model.clone();
    all.set_assembly_view(SceneAssembly { explode: view.explode, step: None });
    let whole = view_lines(&all, cam, px, &opts);
    let blank = || Drawing { size: PAGE, ..Default::default() };
    let mut pages = Vec::new();

    // cover: the model as now (exploded or not), larger
    let mut d = blank();
    text(&mut d, DVec2::new(PAGE.x * 0.5, 28.0), 9.0, fit(&title(doc), 9.0, PAGE.x - 2.0 * MARGIN), false);
    text(&mut d, DVec2::new(PAGE.x * 0.5, 40.0), 5.0, tr!("assembly-booklet-title"), false);
    let cover = (DVec2::new(MARGIN, 55.0), DVec2::new(PAGE.x - 2.0 * MARGIN, 190.0));
    let mut d = match fit_map(&whole, cover, &[]) {
        Some((off, k)) => whole.on_paper(d, move |p| off + p * k),
        None => d,
    };
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

    // each sub-assembly alone, seen the same way, framed by itself
    let subs = sub_assemblies(model);
    for (k, &(def, made)) in subs.iter().enumerate() {
        let mut sub = model.sub_model(def);
        sub.set_assembly_view(SceneAssembly { explode: view.explode, step: None });
        let Some(b) = sub.bounds() else { continue };
        let mut c = cam.clone();
        c.zoom_extents(b.min, b.max, px.x / px.y);
        let name = model.defs[def].name.clone();
        let head = |n: usize, total: usize| tr!("assembly-booklet-sub-step", sub = letter(k), name = name.clone(), made = made, n = n, total = total);
        pages.extend(step_pages(&sub, plan, &c, view.explode, &subs, &head));
    }
    pages.extend(step_pages(model, plan, cam, view.explode, &subs, &|n, total| tr!("assembly-booklet-step", n = n, total = total)));
    let total = pages.len();
    for (i, d) in pages.iter_mut().enumerate() {
        text(d, DVec2::new(PAGE.x * 0.5, PAGE.y - 8.0), 2.6, tr!("assembly-booklet-footer", doc = doc.to_string(), n = i + 1, total = total), false);
    }
    pages
}

/// Screen → paper, fitted to `lines` (and the points `extra`) in the box `min`, `size`.
fn fit_map(lines: &crate::vector_export::ViewLines, (min, size): (DVec2, DVec2), extra: &[DVec2]) -> Option<(DVec2, f64)> {
    let (lo, hi) = lines.bounds()?;
    let (lo, hi) = extra.iter().fold((lo, hi), |(l, h), p| (l.min(*p), h.max(*p)));
    let ext = (hi - lo).max(DVec2::splat(1e-6));
    let k = (size.x / ext.x).min(size.y / ext.y);
    Some((min + size * 0.5 - (lo + hi) * 0.5 * k, k))
}

/// A page per step of `model`'s steps, headed `head(n, total)`, arrows showing where its parts
/// go; `subs` letters the sub-assemblies a step adds.
fn step_pages(model: &Model, plan: &CutPlan, cam: &Camera, explode: f64, subs: &[(DefId, usize)], head: &dyn Fn(usize, usize) -> String) -> Vec<Drawing> {
    let a = &model.assembly;
    let rect = (DVec2::new(MARGIN, 32.0), DVec2::new(PAGE.x - 2.0 * MARGIN, 175.0));
    let px = rect.1 * 4.0;
    let opts = VectorOptions { profiles: true, hidden: false, dims: false, sections: false, ..Default::default() };
    let mut all = model.clone();
    all.set_assembly_view(SceneAssembly { explode, step: None });
    let whole = view_lines(&all, cam, px, &opts);
    // the steps' arrows (the parts' only with the model exploded) and fasteners, framed with
    // the model
    let dirs = varstotas_kernel::assembly::explode_dirs(model);
    let (step_arrows, step_fasteners): (Vec<Vec<Arrow>>, Vec<Vec<Outline>>) = (0..a.steps.len())
        .map(|k| {
            let mut m = model.clone();
            m.set_assembly_view(SceneAssembly { explode, step: Some(k) });
            let mut out = if explode > 0.0 { arrows(&m, &a.steps[k], &dirs, explode, cam, px) } else { Vec::new() };
            let (outlines, more) = fastener_lines(&m, k, cam, px);
            out.extend(more);
            (out, outlines)
        })
        .unzip();
    let ends: Vec<DVec2> = step_arrows.iter().flatten().flat_map(|&(t, h)| [t, h]).collect();
    let map = fit_map(&whole, rect, &ends);
    let picture = |d: Drawing, lines: &crate::vector_export::ViewLines, map: Option<(DVec2, f64)>| match map {
        Some((off, k)) => lines.on_paper(d, move |p| off + p * k),
        None => d,
    };
    let blank = || Drawing { size: PAGE, ..Default::default() };
    let mut pages = Vec::new();
    let n = a.steps.len();
    for (k, step) in a.steps.iter().enumerate() {
        let mut m = model.clone();
        m.set_assembly_view(SceneAssembly { explode, step: Some(k) });
        let faint: Vec<ContextPath> = a.steps[..k].iter().flat_map(|s| s.parts.iter().cloned()).collect();
        let lines = view_lines(&m, cam, px, &VectorOptions { faint, ..opts.clone() });
        let mut d = blank();
        text(&mut d, DVec2::new(MARGIN, 22.0), 7.0, fit(&head(k + 1, n), 7.0, PAGE.x - 2.0 * MARGIN), true);
        let mut d = picture(d, &lines, map);
        if let Some((off, scale)) = map {
            for o in &step_fasteners[k] {
                for (i, &p) in o.iter().enumerate() {
                    let q = o[(i + 1) % o.len()];
                    d.lines.push((off + p * scale, off + q * scale, Pen::Profile));
                }
            }
            for &(t, h) in &step_arrows[k] {
                draw_arrow(&mut d, off + t * scale, off + h * scale);
            }
        }
        let mut y = rect.0.y + rect.1.y + 12.0;
        // a close-up of the step's first fastener, at the lower right, the text beside it; none
        // for hardware large enough to see (legs, slides)
        let extent = |o: &Outline| o.iter().fold((DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY)), |(l, h), p| (l.min(*p), h.max(*p)));
        let small = step_fasteners[k].first().filter(|o| {
            let (lo, hi) = extent(o);
            (hi - lo).max_element() < px.x * 0.04
        });
        let close = small.zip(map).map(|(o, (off, scale))| {
            let (lo, hi) = extent(o);
            let c = (lo + hi) * 0.5;
            let half = ((hi - lo).max_element() * 2.5).max(px.x * 0.03);
            let at = DVec2::new(PAGE.x - MARGIN - CLOSE_UP, y - 4.0);
            let to = |p: DVec2| at + (p - c + DVec2::splat(half)) * (CLOSE_UP / (2.0 * half));
            let (w0, w1) = (c - DVec2::splat(half), c + DVec2::splat(half));
            for &(p, q, pen) in &lines.strokes {
                if let Some((p, q)) = clip(p, q, w0, w1) {
                    d.lines.push((to(p), to(q), pen));
                }
            }
            for o in &step_fasteners[k] {
                for (i, &p) in o.iter().enumerate() {
                    if let Some((p, q)) = clip(p, o[(i + 1) % o.len()], w0, w1) {
                        d.lines.push((to(p), to(q), Pen::Profile));
                    }
                }
            }
            // its frame, and where it is on the picture
            let frame = |d: &mut Drawing, a: DVec2, b: DVec2| {
                let corners = [a, DVec2::new(b.x, a.y), b, DVec2::new(a.x, b.y)];
                for i in 0..4 {
                    d.lines.push((corners[i], corners[(i + 1) % 4], Pen::Edge));
                }
            };
            frame(&mut d, at, at + DVec2::splat(CLOSE_UP));
            frame(&mut d, off + w0 * scale, off + w1 * scale);
        });
        let width = PAGE.x - 2.0 * MARGIN - if close.is_some() { CLOSE_UP + 6.0 } else { 0.0 };
        let block = |d: &mut Drawing, head: String, items: Vec<String>, y: &mut f64| {
            if items.is_empty() {
                return;
            }
            text(d, DVec2::new(MARGIN, *y), 4.0, head, true);
            *y += 6.0;
            for it in items {
                text(d, DVec2::new(MARGIN + 4.0, *y), 3.6, fit(&it, 3.6, width - 4.0), true);
                *y += 5.0;
            }
            *y += 2.0;
        };
        block(&mut d, tr!("assembly-booklet-add"), step_parts(model, plan, step, subs), &mut y);
        block(&mut d, tr!("assembly-booklet-hardware"), step_hardware(model, k), &mut y);
        for l in wrap(&step.note, if close.is_some() { 60 } else { 90 }) {
            if y > PAGE.y - 18.0 {
                break;
            }
            text(&mut d, DVec2::new(MARGIN, y), 3.6, l, true);
            y += 5.0;
        }
        pages.push(d);
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
        if let Some(s) = v.step.and_then(|s| self.doc.model.shown_steps().get(s).cloned()).filter(|s| !s.parts.is_empty()) {
            let parent = s.parts.first().map(|p| p[..p.len() - 1].to_vec()).unwrap_or_default();
            let sel: Vec<EntityRef> = s.parts.iter().filter(|p| p[..p.len() - 1] == parent[..]).filter_map(|p| p.last().map(|&id| EntityRef::Instance(id))).collect();
            self.doc.set_active_path(parent);
            self.doc.set_selection(sel);
        }
        self.doc.prune_selection();
    }

    /// Replace the steps shown with those the contact graph suggests; the model's give every
    /// group or component with joints inside steps of its own when it has none.
    pub(crate) fn suggest_assembly(&mut self) {
        self.set_assembly_view(SceneAssembly::default());
        let within = self.doc.model.assembly.within.clone();
        match self.doc.model.def_at(&within).filter(|_| !within.is_empty()) {
            None => {
                let (steps, subs) = suggest_all(&self.doc.model);
                self.doc.commit(&tr!("assembly-undo-steps"), move |m| {
                    m.assembly.steps = steps;
                    for (d, s) in subs {
                        if let Some(def) = m.defs.get_mut(d) {
                            def.steps = s;
                        }
                    }
                });
            }
            Some(d) => self.make_sub_assembly(d),
        }
    }

    /// Give definition `d` the steps its parts suggest: a sub-assembly.
    fn make_sub_assembly(&mut self, d: DefId) {
        let steps = suggest_steps(&self.doc.model.sub_model(d));
        self.doc.commit(&tr!("assembly-undo-steps"), move |m| {
            if let Some(def) = m.defs.get_mut(d) {
                def.steps = steps;
            }
        });
    }

    /// The sub-assembly whose steps the window shows: the innermost open group or component
    /// with steps of its own; the model's when none is open.
    fn assembly_scope(&self) -> ContextPath {
        let ap = &self.doc.active_path;
        let m = &self.doc.model;
        (1..=ap.len()).rev().map(|k| &ap[..k]).find(|p| m.def_at(p).is_some_and(|d| !m.defs[d].steps.is_empty())).map(|p| p.to_vec()).unwrap_or_default()
    }

    /// Replace the steps shown with `steps` (their paths from the root, joints of
    /// [`Model::placed_joints`]): the model's, or the sub-assembly's.
    fn commit_steps(&mut self, label: &str, steps: Vec<Step>) {
        let m = &self.doc.model;
        let within = m.assembly.within.clone();
        let Some(d) = m.def_at(&within).filter(|_| !within.is_empty()) else {
            self.doc.commit(label, move |m| m.assembly.steps = steps);
            return;
        };
        let own = |i: usize| m.joint_places.get(i).map(|p| p.index);
        let steps: Vec<Step> = steps
            .into_iter()
            .map(|s| Step {
                parts: s.parts.iter().filter_map(|p| p.strip_prefix(&within[..]).map(|r| r.to_vec())).collect(),
                joints: s.joints.iter().filter_map(|&i| own(i)).collect(),
                note: s.note,
            })
            .collect();
        self.doc.commit(label, move |m| {
            if let Some(def) = m.defs.get_mut(d) {
                def.steps = steps;
            }
        });
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
        let m = &self.doc.model;
        let a = &m.assembly;
        let plan = crate::ui::cutlist::model_plan(m);
        let subs = sub_assemblies(m);
        let steps = m.shown_steps();
        // a fastener step by its hardware
        let what = |(k, s): (usize, &Step)| if s.parts.is_empty() { step_hardware(m, k) } else { step_parts(m, &plan, s, &subs) };
        (steps.len(), a.explode.round() as i64, a.step, steps.iter().enumerate().map(what).collect())
    }

    /// The sub-assembly shown (its name; "" the model's) and every sub-assembly the model's
    /// steps add with how many are made.
    pub(crate) fn sub_assembly_dump(&self) -> (String, Vec<(String, usize)>) {
        let m = &self.doc.model;
        let within = &m.assembly.within;
        let name = if within.is_empty() { String::new() } else { m.def_at(within).map(|d| m.defs[d].name.clone()).unwrap_or_default() };
        (name, sub_assemblies(m).into_iter().map(|(d, n)| (m.defs[d].name.clone(), n)).collect())
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
        let Some(mut path) = crate::file_dialog::FileDialog::new()
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
        // the steps of the open sub-assembly, or the model's
        let scope = self.assembly_scope();
        if scope != self.doc.model.assembly.within {
            self.doc.mutate_untracked(|m| m.set_assembly_within(scope));
            self.doc.prune_selection();
        }
        let mut open = true;
        let win = self.placed(egui::Window::new(tr!("assembly-title")).id(egui::Id::new("assembly_window")).open(&mut open).collapsible(false).resizable(true).default_size([480.0, 420.0]).min_width(420.0).pivot(egui::Align2::RIGHT_TOP), pos);
        win.show(ctx, |ui| {
            self.assembly_view_ui(ui);
            ui.separator();
            self.assembly_steps_ui(ui);
        });
        if !open {
            self.assembly.open = false;
            self.set_assembly_view(SceneAssembly::default());
            self.doc.mutate_untracked(|m| m.set_assembly_within(Vec::new()));
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

    /// Whose steps these are: a sub-assembly's (and a way back to a part of the whole), or the
    /// model's, with a way to make the open group or component a sub-assembly.
    fn assembly_scope_ui(&mut self, ui: &mut Ui) {
        let m = &self.doc.model;
        let within = m.assembly.within.clone();
        let open = self.doc.active_path.clone();
        if let Some(d) = m.def_at(&within).filter(|_| !within.is_empty()) {
            let name = m.defs[d].name.clone();
            let mut clear = false;
            ui.horizontal(|ui| {
                ui.label(RichText::new(tr!("assembly-sub-of", name = name.clone())).strong());
                let r = ui.small_button(tr!("assembly-sub-clear"));
                super::tray_lib::spot("assembly:sub-clear", r.rect.center());
                clear = r.on_hover_text(tr!("assembly-sub-clear-tip")).clicked();
            });
            ui.label(RichText::new(tr!("assembly-sub-note")).small().color(theme::text_dim()));
            if clear {
                self.set_assembly_view(SceneAssembly::default());
                self.doc.commit(&tr!("assembly-undo-steps"), move |m| {
                    if let Some(def) = m.defs.get_mut(d) {
                        def.steps.clear();
                    }
                });
            }
        } else if let Some(d) = m.def_at(&open).filter(|_| !open.is_empty()) {
            let name = m.defs[d].name.clone();
            let r = ui.button(tr!("assembly-sub-make", name = name));
            super::tray_lib::spot("assembly:sub-make", r.rect.center());
            if r.on_hover_text(tr!("assembly-sub-make-tip")).clicked() {
                self.set_assembly_view(SceneAssembly::default());
                self.make_sub_assembly(d);
            }
        }
    }

    fn assembly_steps_ui(&mut self, ui: &mut Ui) {
        self.assembly_scope_ui(ui);
        let mut a = self.doc.model.assembly.clone();
        a.steps = self.doc.model.shown_steps().into_owned();
        let sub = !a.within.is_empty();
        ui.horizontal(|ui| {
            let r = ui.button(RichText::new(tr!(if a.steps.is_empty() { "assembly-suggest" } else { "assembly-suggest-again" })).strong());
            super::tray_lib::spot("assembly:suggest", r.rect.center());
            if r.on_hover_text(tr!("assembly-suggest-tip")).clicked() {
                self.suggest_assembly();
            }
            ui.add_enabled_ui(!a.steps.is_empty() && !sub, |ui| {
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
        let subs = sub_assemblies(&self.doc.model);
        let mut show: Option<Option<usize>> = None;
        let mut edit: Option<(String, Vec<Step>)> = None;
        // the rest of the window, as large as it is made
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
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
                        // a fastener step by its hardware, in italics
                        let what = if step.parts.is_empty() { step_hardware(&self.doc.model, k) } else { step_parts(&self.doc.model, &plan, step, &subs) };
                        let what = RichText::new(fit(&what.join(", "), 9.0, 220.0));
                        ui.label(if step.parts.is_empty() { what.italics() } else { what });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let mut steps = a.steps.clone();
                            let first = fasteners_first(&self.doc.model, &a.steps, k);
                            if !first.is_empty() && !step.parts.is_empty() {
                                let r = ui.small_button("⊕");
                                super::tray_lib::spot(format!("assembly:fasteners:{k}"), r.rect.center());
                                if r.on_hover_text(tr!("assembly-fasteners-first-tip")).clicked() {
                                    steps.insert(k, Step { joints: first, ..Default::default() });
                                    edit = Some((tr!("assembly-undo-steps"), steps.clone()));
                                }
                            }
                            if step.parts.len() > 1 && ui.small_button("÷").on_hover_text(tr!("assembly-split-tip")).clicked() {
                                let parts = steps[k].parts.clone();
                                steps.splice(k..=k, parts.into_iter().map(|p| Step { parts: vec![p], ..Default::default() }));
                                if let Some(s) = steps.get_mut(k) {
                                    s.note = step.note.clone();
                                    s.joints = step.joints.clone();
                                }
                                edit = Some((tr!("assembly-undo-steps"), steps));
                            } else if k > 0 && ui.small_button("⤒").on_hover_text(tr!("assembly-join-tip")).clicked() {
                                let s = steps.remove(k);
                                steps[k - 1].parts.extend(s.parts);
                                steps[k - 1].joints.extend(s.joints);
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
            self.commit_steps(&label, steps);
        }
    }
}
