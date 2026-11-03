//! Drilling maps: a PDF page for each part drilled (its copies drilled alike together), every
//! side with holes drawn to scale from outside, each hole's place measured from the corner
//! marked on the part, and what to drill on each side.

use super::cutlist::{dim, fit_text, hole, rect_lines, text};
use crate::app::VarstotasApp;
use crate::vector_export::{Drawing, Pen, Text2};
use std::path::Path;
use varstotas_i18n::tr;
use varstotas_kernel::LengthUnit;
use varstotas_kernel::cutlist::{CutPart, CutPlan, CutSettings, DrillFor, DrillHole, Drilling};
use varstotas_kernel::glam::DVec2;

/// A4 landscape.
const PAGE: DVec2 = DVec2::new(297.0, 210.0);
const MARGIN: f64 = 10.0;
/// The column of what to drill on the right.
const LIST_W: f64 = 64.0;
/// Room above a view for its title and below and left of it for the measures.
const TITLE_H: f64 = 7.0;
const BELOW_H: f64 = 14.0;
const LEFT_W: f64 = 14.0;
const TEXT: f64 = 2.4;

/// The sides in the order drawn: face A first (always, the part's outline), face B, the edges,
/// the ends ([`DrillHole::side`]).
const SIDES: [usize; 6] = [5, 4, 2, 3, 0, 1];

fn side_name(side: usize) -> String {
    tr!(["drill-side-end-1", "drill-side-end-2", "drill-side-edge-1", "drill-side-edge-2", "drill-side-face-b", "drill-side-face-a"][side])
}

/// A side as seen from outside: its width and height on the part and where a hole on it lands,
/// (across, up) from the marked corner. The frame turns as a right hand does, so each view is
/// the side looked at, not its mirror: face A has the length across and the width up, face B
/// is the part turned over along its length, an edge or an end has the thickness up.
fn view(side: usize, size: [f64; 3], h: &DrillHole) -> (f64, f64) {
    let [l, w, _] = size;
    let a = h.at;
    match side {
        5 => (a[0], a[1]),
        4 => (a[0], w - a[1]),
        2 => (a[0], a[2]),
        3 => (l - a[0], a[2]),
        0 => (w - a[1], a[2]),
        _ => (a[1], a[2]),
    }
}

fn view_size(side: usize, size: [f64; 3]) -> (f64, f64) {
    match side / 2 {
        2 => (size[0], size[1]),
        1 => (size[0], size[2]),
        _ => (size[1], size[2]),
    }
}

/// Where a side's corner (0, 0) is on the part: which end, edge and face it touches.
fn corner_note(side: usize) -> String {
    tr!(["drill-corner-end-1", "drill-corner-end-2", "drill-corner-edge-1", "drill-corner-edge-2", "drill-corner-face-b", "drill-corner-face-a"][side])
}

/// The values `v` once each (to 0.1 mm), in order.
fn marks(mut v: Vec<f64>) -> Vec<f64> {
    v.iter_mut().for_each(|x| *x = (*x * 10.0).round() / 10.0);
    v.sort_by(f64::total_cmp);
    v.dedup();
    v
}

/// Label places for ticks at `at` (ascending), each at least `gap` past the one before: labels
/// that would touch go in runs `gap` apart, each run centred on its ticks as far as the run
/// before lets it.
fn spread(at: &[f64], gap: f64) -> Vec<f64> {
    // runs: (members, sum of each member's tick less its place in the run)
    let mut runs: Vec<(usize, f64)> = Vec::new();
    let start = |r: &(usize, f64)| r.1 / r.0 as f64;
    for &x in at {
        runs.push((1, x));
        while runs.len() > 1 {
            let (b, a) = (runs[runs.len() - 1], runs[runs.len() - 2]);
            if start(&a) + a.0 as f64 * gap <= start(&b) {
                break;
            }
            runs.pop();
            let last = runs.len() - 1;
            runs[last] = (a.0 + b.0, a.1 + b.1 - gap * (a.0 * b.0) as f64);
        }
    }
    runs.iter().flat_map(|r| (0..r.0).map(move |j| start(r) + j as f64 * gap)).collect()
}

/// Runs of at least four values `v` (ascending) the same distance apart, as (first, last)
/// indices: a row drilled at one pitch (shelf pins on the 32 mm system).
fn chains(v: &[f64]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < v.len() {
        let p = v[i + 1] - v[i];
        let mut j = i + 1;
        // marks are to 0.1: a pitch of 128.67 reads 128.7, 128.6, 128.7
        while j + 1 < v.len() && (v[j + 1] - v[j] - p).abs() < 0.15 {
            j += 1;
        }
        if j - i >= 3 {
            out.push((i, j));
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

/// The pitch of holes (as placed on their side) in rows: each line along one direction of the
/// side holding four or more at one distance apart, the same in every line.
fn pitch(at: &[(f64, f64)]) -> Option<f64> {
    let along = |swap: bool| -> Option<f64> {
        let mut lines: Vec<(f64, Vec<f64>)> = Vec::new();
        for &(u, v) in at {
            let (u, v) = if swap { (v, u) } else { (u, v) };
            match lines.iter_mut().find(|l| (l.0 - v).abs() < 0.05) {
                Some(l) => l.1.push(u),
                None => lines.push((v, vec![u])),
            }
        }
        let mut p = None;
        for (_, mut l) in lines {
            l.sort_by(f64::total_cmp);
            let d = l.get(1)? - l[0];
            if l.len() < 4 || d < 0.05 || l.windows(2).any(|w| (w[1] - w[0] - d).abs() >= 0.05) || p.is_some_and(|p: f64| (p - d).abs() >= 0.05) {
                return None;
            }
            p = Some(d);
        }
        p
    };
    along(false).or_else(|| along(true))
}

/// What a hole is for, as the list says it.
fn what_name(w: &DrillFor) -> String {
    match w {
        DrillFor::Joint(k) => super::joinery::kind_name(*k),
        DrillFor::Fitting(item) => item.clone(),
    }
}

/// What to drill on side `side`: a row for each kind of hole, the largest first — "count × Ø d,
/// depth deep" (or right through), and under it what for, and the pitch of holes in rows.
fn rows(side: usize, size: [f64; 3], holes: &[&DrillHole], unit: LengthUnit) -> Vec<(String, String)> {
    let mut groups: Vec<(&DrillHole, Vec<&DrillHole>)> = Vec::new();
    for &h in holes {
        match groups.iter_mut().find(|g| g.0.diameter == h.diameter && g.0.depth == h.depth && g.0.through == h.through && g.0.what == h.what) {
            Some(g) => g.1.push(h),
            None => groups.push((h, vec![h])),
        }
    }
    groups.sort_by(|a, b| b.0.diameter.total_cmp(&a.0.diameter).then(b.0.depth.total_cmp(&a.0.depth)));
    groups
        .into_iter()
        .map(|(h, all)| {
            let (n, d) = (all.len(), dim(h.diameter, unit));
            let row = match (h.through, h.depth > 0.0) {
                (true, _) => tr!("drill-row-through", n = n, d = d),
                (false, true) => tr!("drill-row", n = n, d = d, depth = dim(h.depth, unit)),
                (false, false) => tr!("drill-row-open", n = n, d = d),
            };
            let at: Vec<(f64, f64)> = all.iter().map(|x| view(side, size, x)).collect();
            let what = what_name(&h.what);
            let note = match pitch(&at) {
                Some(p) => tr!("drill-row-pitch", what = what, pitch = dim(p, unit)),
                None => what,
            };
            (row, note)
        })
        .collect()
}

/// Ordinate labels for marks `v` (ascending) at paper places `at`: (paper place, text) — a run
/// at one pitch labelled at its ends and once between them "n × pitch".
fn labels(v: &[f64], at: &[f64], unit: LengthUnit) -> Vec<(f64, String)> {
    let runs = chains(v);
    let mut out = Vec::new();
    let mut i = 0;
    while i < v.len() {
        out.push((at[i], dim(v[i], unit)));
        if let Some(&(_, j)) = runs.iter().find(|r| r.0 == i) {
            out.push(((at[i] + at[j]) / 2.0, tr!("drill-chain", n = j - i, pitch = dim(v[i + 1] - v[i], unit))));
            out.push((at[j], dim(v[j], unit)));
            i = j;
        }
        i += 1;
    }
    out
}

/// One page: the part's number, name, how many copies, size and material at the top, its
/// sides with holes stacked down the left, what to drill on each down the right.
fn drilling_page(number: usize, p: &CutPart, dr: &Drilling, unit: LengthUnit, doc: &str) -> Drawing {
    let mut d = Drawing { size: PAGE, ..Default::default() };
    let size = p.size;
    let title = format!("{number}  {}", p.name);
    text(&mut d, DVec2::new(MARGIN, MARGIN + 5.0), 5.0, fit_text(&title, 5.0, PAGE.x - 2.0 * MARGIN - LIST_W), true);
    let copies = if dr.count == p.count { tr!("drill-copies-all", n = dr.count) } else { tr!("drill-copies-some", n = dr.count, total = p.count) };
    let material = if p.material_name.is_empty() { tr!("cutlist-no-material") } else { p.material_name.clone() };
    let line = format!("{} × {} × {}  ·  {material}  ·  {copies}", dim(size[0], unit), dim(size[1], unit), dim(size[2], unit));
    text(&mut d, DVec2::new(MARGIN, MARGIN + 11.0), 3.0, line, true);
    text(&mut d, DVec2::new(PAGE.x - MARGIN - LIST_W, PAGE.y - MARGIN * 0.5), 2.0, fit_text(doc, 2.0, LIST_W), true);

    // the sides drawn: face A always, the others with holes
    let sides: Vec<usize> = SIDES.into_iter().filter(|&s| s == 5 || dr.holes.iter().any(|h| h.side == s)).collect();
    let top = MARGIN + 18.0;
    let (area_w, area_h) = (PAGE.x - 2.0 * MARGIN - LIST_W - LEFT_W - 4.0, PAGE.y - top - MARGIN);
    let widest = sides.iter().map(|&s| view_size(s, size).0).fold(1.0, f64::max);
    let tall: f64 = sides.iter().map(|&s| view_size(s, size).1).sum();
    let room = area_h - sides.len() as f64 * (TITLE_H + BELOW_H);
    let k = (area_w / widest).min(room.max(10.0) / tall.max(1.0)).min(1.0);
    let x0 = MARGIN + LEFT_W;
    let mut y = top;
    for &s in &sides {
        let (w, h) = view_size(s, size);
        let on: Vec<&DrillHole> = dr.holes.iter().filter(|x| x.side == s).collect();
        text(&mut d, DVec2::new(x0, y + 4.5), 3.0, format!("{}  {} × {}", side_name(s), dim(w, unit), dim(h, unit)), true);
        y += TITLE_H;
        // paper: y down, the side's corner at bottom left
        let at = |u: f64, v: f64| DVec2::new(x0 + u * k, y + (h - v) * k);
        let (lo, hi) = (at(0.0, h), at(w, 0.0));
        rect_lines(&mut d, lo, hi, Pen::Profile);
        // the marked corner: a filled triangle in it
        let c = at(0.0, 0.0);
        d.fills.push(vec![c, c + DVec2::new(4.0, 0.0), c + DVec2::new(0.0, -4.0)]);
        let places: Vec<(f64, f64)> = on.iter().map(|x| view(s, size, x)).collect();
        for (x, (u, v)) in on.iter().zip(&places) {
            let (c, r) = (at(*u, *v), (x.diameter / 2.0 * k).max(0.5));
            hole(&mut d, c, r);
            // right through: crossed out past its rim
            if x.through {
                let (a, b) = (DVec2::new(1.0, 1.0) * r * 1.6, DVec2::new(1.0, -1.0) * r * 1.6);
                d.lines.extend([(c - a, c + a, Pen::Edge), (c - b, c + b, Pen::Edge)]);
            }
        }
        // ordinate measures from the corner: across under the side, up on its left, labels
        // that would touch moved apart with a slanted leader to their tick
        if !on.is_empty() {
            let us = marks(places.iter().map(|p| p.0).chain([0.0]).collect());
            let base = at(0.0, 0.0).y;
            let xs: Vec<f64> = us.iter().map(|&u| at(u, 0.0).x).collect();
            for x in &xs {
                d.lines.push((DVec2::new(*x, base + 1.0), DVec2::new(*x, base + 3.0), Pen::Dim));
            }
            let ls = labels(&us, &xs, unit);
            let anchors: Vec<f64> = ls.iter().map(|l| l.0).collect();
            for ((x, s), q) in ls.into_iter().zip(spread(&anchors, TEXT * 1.1)) {
                d.lines.push((DVec2::new(x, base + 3.0), DVec2::new(q, base + 4.5), Pen::Dim));
                let len = s.chars().count() as f64 * TEXT * 0.55;
                d.texts.push(Text2 { at: DVec2::new(q + TEXT * 0.35, base + 5.5 + len), angle: -std::f64::consts::FRAC_PI_2, size: TEXT, lines: vec![s], left: true });
            }
            // up: from the top down on paper
            let vs = marks(places.iter().map(|p| p.1).chain([0.0]).collect());
            let left = at(0.0, 0.0).x;
            let ys: Vec<f64> = vs.iter().map(|&v| at(0.0, v).y).collect();
            for y in &ys {
                d.lines.push((DVec2::new(left - 1.0, *y), DVec2::new(left - 3.0, *y), Pen::Dim));
            }
            let mut ls = labels(&vs, &ys, unit);
            ls.reverse();
            let anchors: Vec<f64> = ls.iter().map(|l| l.0).collect();
            for ((y, s), q) in ls.into_iter().zip(spread(&anchors, TEXT * 1.2)) {
                d.lines.push((DVec2::new(left - 3.0, y), DVec2::new(left - 4.5, q), Pen::Dim));
                let len = s.chars().count() as f64 * TEXT * 0.55;
                text(&mut d, DVec2::new(left - 5.5 - len, q + TEXT * 0.35), TEXT, s, true);
            }
        }
        y += h * k + BELOW_H;
    }

    // what to drill, side by side, and where the corner is
    let lx = PAGE.x - MARGIN - LIST_W;
    let mut ly = top + 4.5;
    text(&mut d, DVec2::new(lx, ly), 3.4, tr!("drill-list", n = dr.holes.len()), true);
    ly += 7.0;
    for &s in &sides {
        let on: Vec<&DrillHole> = dr.holes.iter().filter(|x| x.side == s).collect();
        if on.is_empty() {
            continue;
        }
        text(&mut d, DVec2::new(lx, ly), 3.0, side_name(s), true);
        ly += 4.5;
        for (r, note) in rows(s, size, &on, unit) {
            text(&mut d, DVec2::new(lx + 3.0, ly), 2.7, fit_text(&r, 2.7, LIST_W - 3.0), true);
            ly += 3.2;
            text(&mut d, DVec2::new(lx + 5.0, ly), 2.2, fit_text(&note, 2.2, LIST_W - 5.0), true);
            ly += 4.0;
        }
        text(&mut d, DVec2::new(lx + 3.0, ly), 2.2, fit_text(&corner_note(s), 2.2, LIST_W - 3.0), true);
        ly += 6.0;
    }
    d.texts.push(Text2 { at: DVec2::new(lx, ly + 2.0), angle: 0.0, size: 2.2, lines: wrap(&tr!("drill-note"), 2.2, LIST_W), left: true });
    d
}

/// `s` in lines of about `width` mm at text `size`.
fn wrap(s: &str, size: f64, width: f64) -> Vec<String> {
    let max = (width / (size * 0.55)).floor() as usize;
    let mut out: Vec<String> = Vec::new();
    for word in s.split_whitespace() {
        match out.last_mut() {
            Some(l) if l.chars().count() + 1 + word.chars().count() <= max => {
                l.push(' ');
                l.push_str(word);
            }
            _ => out.push(word.to_string()),
        }
    }
    out
}

/// A page per drilled part, its copies drilled alike together (bought and left-out parts have
/// none), numbered as on the cut diagrams.
pub(crate) fn drilling_pages(plan: &CutPlan, settings: &CutSettings, unit: LengthUnit, doc: &str) -> Vec<Drawing> {
    let mut pages = Vec::new();
    for (i, p) in plan.parts.iter().enumerate() {
        if p.bought(settings) || settings.options(&p.name).exclude {
            continue;
        }
        for dr in &p.drilling {
            pages.push(drilling_page(i + 1, p, dr, unit, doc));
        }
    }
    let n = pages.len();
    for (i, d) in pages.iter_mut().enumerate() {
        text(d, DVec2::new(PAGE.x - MARGIN, MARGIN + 5.0), 2.6, tr!("drill-page", n = i + 1, total = n), false);
    }
    pages
}

/// Whether any part of `plan` has holes to drill.
pub(crate) fn any_drilling(plan: &CutPlan, settings: &CutSettings) -> bool {
    plan.parts.iter().any(|p| !p.drilling.is_empty() && !p.bought(settings) && !settings.options(&p.name).exclude)
}

impl VarstotasApp {
    /// A page each: (part, copies drilled alike, holes, the sides drilled) for UI test scripts.
    pub(crate) fn drilling_dump(&self) -> Vec<(String, usize, usize, Vec<usize>)> {
        let settings = &self.doc.model.cut_list;
        let plan = super::cutlist::model_plan(&self.doc.model);
        let mut out = Vec::new();
        for p in plan.parts.iter().filter(|p| !p.bought(settings) && !settings.options(&p.name).exclude) {
            for d in &p.drilling {
                let mut sides: Vec<usize> = d.holes.iter().map(|h| h.side).collect();
                sides.dedup();
                out.push((p.name.clone(), d.count, d.holes.len(), sides));
            }
        }
        out
    }

    /// Write a PDF of drilling maps (A4, a page per part drilled) to `path`.
    pub(crate) fn export_drilling(&mut self, path: &Path) {
        let plan = self.cut_plan();
        let pages = drilling_pages(&plan, &self.doc.model.cut_list, self.doc.model.unit, &self.doc.display_name());
        if pages.is_empty() {
            return self.flash_error(tr!("drill-none"));
        }
        match varstotas_kernel::io::atomic_write(path, crate::vector_export::to_pdf_pages(&pages)) {
            Ok(()) => self.flash(tr!("cutlist-exported", path = path.display().to_string())),
            Err(e) => self.flash_error(tr!("cutlist-export-failed", error = e.to_string())),
        }
    }

    pub(crate) fn drilling_save(&mut self) {
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            // i18n-ignore-next
            .set_file_name(format!("{} drilling.pdf", self.doc.display_name()))
            .save_file()
        else {
            return;
        };
        if path.extension().is_none() {
            path.set_extension("pdf");
        }
        self.export_drilling(&path);
    }
}

#[cfg(test)]
mod tests {
    use super::{chains, pitch, spread};

    #[test]
    fn rows_at_one_pitch_are_found() {
        // 0 and a row of five 32 apart: one chain from the second mark
        assert_eq!(chains(&[0.0, 37.0, 69.0, 101.0, 133.0, 165.0, 300.0]), vec![(1, 5)]);
        // three marks are not a chain
        assert!(chains(&[0.0, 32.0, 64.0]).is_empty());
        // a third of 386 to 0.1
        assert_eq!(chains(&[0.0, 32.0, 160.7, 289.3, 418.0]), vec![(1, 4)]);
        // shelf pins: two lines of four 32 apart (three are not a row)
        let pins: Vec<(f64, f64)> = [37.0, 260.0].iter().flat_map(|&v| (0..4).map(move |i| (100.0 + 32.0 * i as f64, v))).collect();
        assert_eq!(pitch(&pins), Some(32.0));
        // the same row up the side
        let up: Vec<(f64, f64)> = pins.iter().map(|&(u, v)| (v, u)).collect();
        assert_eq!(pitch(&up), Some(32.0));
        // lines at two pitches, or a pair: none
        assert_eq!(pitch(&[(0.0, 0.0), (32.0, 0.0), (64.0, 0.0), (96.0, 0.0), (0.0, 50.0), (20.0, 50.0), (40.0, 50.0), (60.0, 50.0)]), None);
        assert_eq!(pitch(&[(0.0, 0.0), (32.0, 0.0), (64.0, 0.0)]), None);
    }

    #[test]
    fn labels_that_would_touch_move_apart_round_their_ticks() {
        // apart already: where they are
        assert_eq!(spread(&[0.0, 10.0, 20.0], 3.0), vec![0.0, 10.0, 20.0]);
        // two touching: centred on their ticks
        assert_eq!(spread(&[10.0, 11.0], 3.0), vec![9.0, 12.0]);
        // a run pushed into the next: one run, still in order and apart
        let s = spread(&[0.0, 1.0, 2.0, 5.0, 30.0], 3.0);
        assert!(s.windows(2).all(|w| w[1] - w[0] >= 3.0 - 1e-9), "{s:?}");
        assert_eq!(s[4], 30.0);
    }
}
