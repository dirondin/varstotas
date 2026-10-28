//! Cut list (track W, `docs/WORKSHOP-PLAN.md`): the parts of the model laid out on the stock
//! they are cut from.
//!
//! A material can carry [`Stock`]: sheets (plywood, MDF: length × width × thickness, grain along
//! the length or none) or boards (a section width × thickness, the lengths on sale). Parts are the
//! leaf groups / components of the visible model (no visible instance inside), sized like the
//! parts list (`report.rs`: the bounding box in their own axes, length ≥ width ≥ thickness);
//! equal parts share a row with a count. A part's material is its instance's (inherited), else
//! the material covering most of its faces; its thickness picks that material's stock.
//!
//! Sheets are packed with guillotine cuts (every cut runs right through the piece, as on a panel
//! or track saw): free rectangles split after each placement, several part orders and placement
//! and split rules tried, the plan with the fewest sheets (then the largest offcut) kept. Boards
//! are 1D cutting stock: first fit decreasing over the longest length, then each board shrunk
//! to the shortest length on sale that holds its pieces. The kerf is left after every piece
//! except at the stock's edge. Everything is in millimetres.

use crate::model::{ContextPath, Model};
use crate::tessellate::triangulate_face;
use crate::{DefId, MaterialId};
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// What a material is bought as.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StockKind {
    /// Plywood, MDF, chipboard…: [`Stock::length`] × [`Stock::width`].
    #[default]
    Sheet,
    /// Sawn timber or any linear stock: a section [`Stock::width`] × [`Stock::thickness`],
    /// cut to length from [`Stock::lengths`].
    Board,
}

/// One stock item of a material (a material can have several thicknesses).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stock {
    #[serde(default)]
    pub kind: StockKind,
    pub thickness: f64,
    /// Sheet: the size along the grain. Boards use [`Stock::lengths`].
    #[serde(default)]
    pub length: f64,
    /// Sheet width, or the board's section width.
    pub width: f64,
    /// Boards: the lengths on sale.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lengths: Vec<f64>,
    /// Sheet: grain along [`Stock::length`], so parts keep their grain that way (boards always
    /// have it along their length).
    #[serde(default)]
    pub grain: bool,
    /// Per sheet, or per metre of board.
    #[serde(default)]
    pub price: f64,
}

impl Stock {
    pub fn sheet(length: f64, width: f64, thickness: f64, grain: bool) -> Self {
        Self { kind: StockKind::Sheet, thickness, length, width, lengths: Vec::new(), grain, price: 0.0 }
    }

    pub fn board(width: f64, thickness: f64, lengths: &[f64]) -> Self {
        let length = lengths.iter().copied().fold(0.0, f64::max);
        Self { kind: StockKind::Board, thickness, length, width, lengths: lengths.to_vec(), grain: true, price: 0.0 }
    }

    /// Board lengths on sale, shortest first (never empty for a board with a length).
    pub fn board_lengths(&self) -> Vec<f64> {
        let mut l: Vec<f64> = self.lengths.iter().copied().filter(|v| *v > 0.0).collect();
        if l.is_empty() && self.length > 0.0 {
            l.push(self.length);
        }
        l.sort_by(f64::total_cmp);
        l.dedup();
        l
    }
}

/// Which part direction must follow the stock's grain.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartGrain {
    #[default]
    Length,
    Width,
    /// No grain: the part can turn either way.
    Any,
}

/// Per-part options, by the part's name.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PartOptions {
    #[serde(default)]
    pub grain: PartGrain,
    /// Left out of the cut list (bought ready-made, hardware…).
    #[serde(default)]
    pub exclude: bool,
}

/// The document's cut-list settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CutSettings {
    /// Saw kerf (mm).
    pub kerf: f64,
    /// Trimmed off every sheet edge (factory edges are often damaged).
    pub trim: f64,
    /// Leftovers at least this size on both sides are listed as offcuts.
    pub min_offcut: f64,
    /// Thickness difference between a part and its stock that is accepted silently.
    pub tolerance: f64,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub parts: BTreeMap<String, PartOptions>,
}

impl Default for CutSettings {
    fn default() -> Self {
        Self { kerf: 3.0, trim: 0.0, min_offcut: 100.0, tolerance: 0.5, parts: BTreeMap::new() }
    }
}

impl CutSettings {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    pub fn options(&self, part: &str) -> PartOptions {
        self.parts.get(part).cloned().unwrap_or_default()
    }
}

/// A part row: equal parts (name, definition, size, material) counted together.
#[derive(Clone, Debug, PartialEq)]
pub struct CutPart {
    /// Instance name, else the definition name.
    pub name: String,
    pub material: Option<MaterialId>,
    /// "" = no material.
    pub material_name: String,
    /// Length ≥ width ≥ thickness (mm).
    pub size: [f64; 3],
    pub count: usize,
    /// Instance chain of every copy, for selecting them in the model.
    pub paths: Vec<ContextPath>,
}

/// A piece on a sheet or board, `x` along the stock's length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    /// Index into [`CutPlan::parts`].
    pub part: usize,
    pub x: f64,
    pub y: f64,
    /// Size along x and y.
    pub w: f64,
    pub h: f64,
    /// The part's width lies along the stock's length.
    pub rotated: bool,
}

/// One sheet or board with what is cut from it.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub length: f64,
    pub width: f64,
    pub pieces: Vec<Placement>,
    /// Leftovers `[x, y, w, h]` of at least the minimum offcut size.
    pub offcuts: Vec<[f64; 4]>,
}

impl Layout {
    pub fn used_area(&self) -> f64 {
        self.pieces.iter().map(|p| p.w * p.h).sum()
    }

    pub fn used_length(&self) -> f64 {
        self.pieces.iter().map(|p| p.x + p.w).fold(0.0, f64::max)
    }
}

/// All the sheets or boards of one stock item.
#[derive(Clone, Debug, PartialEq)]
pub struct StockPlan {
    pub material: Option<MaterialId>,
    pub material_name: String,
    pub stock: Stock,
    pub layouts: Vec<Layout>,
}

impl StockPlan {
    /// Sheets: the count × price; boards: the metres × price.
    pub fn cost(&self) -> f64 {
        match self.stock.kind {
            StockKind::Sheet => self.layouts.len() as f64 * self.stock.price,
            StockKind::Board => self.layouts.iter().map(|l| l.length).sum::<f64>() / 1000.0 * self.stock.price,
        }
    }

    /// Share of the bought stock that is not in a part (0..1).
    pub fn waste(&self) -> f64 {
        let total: f64 = self.layouts.iter().map(|l| l.length * l.width).sum();
        let used: f64 = self.layouts.iter().map(Layout::used_area).sum();
        if total > 0.0 { (1.0 - used / total).max(0.0) } else { 0.0 }
    }

    /// What to buy: (length, count), longest first (one entry for sheets).
    pub fn to_buy(&self) -> Vec<(f64, usize)> {
        let mut by: BTreeMap<u64, usize> = BTreeMap::new();
        for l in &self.layouts {
            *by.entry((l.length * 1000.0).round() as u64).or_default() += 1;
        }
        by.into_iter().rev().map(|(k, n)| (k as f64 / 1000.0, n)).collect()
    }
}

/// Why a part is not on any stock.
#[derive(Clone, Debug, PartialEq)]
pub enum Issue {
    /// Its material has no stock (or it has no material).
    NoStock { part: usize },
    /// Larger than the stock (or wider than the board).
    TooBig { part: usize },
    /// Cut from the nearest thickness, which differs by more than the tolerance.
    Thickness { part: usize, stock: f64 },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CutPlan {
    /// Every part row, excluded ones too (see [`CutSettings::parts`]).
    pub parts: Vec<CutPart>,
    pub plans: Vec<StockPlan>,
    pub issues: Vec<Issue>,
}

impl CutPlan {
    pub fn cost(&self) -> f64 {
        self.plans.iter().map(StockPlan::cost).sum()
    }
}

fn axis_scales(m: &DMat4) -> DVec3 {
    DVec3::new(m.x_axis.truncate().length(), m.y_axis.truncate().length(), m.z_axis.truncate().length())
}

/// Bounding box of a definition (nested instances too) under `xf`.
fn bounds(model: &Model, def: DefId, xf: DMat4) -> (DVec3, DVec3) {
    let mut lo = DVec3::splat(f64::INFINITY);
    let mut hi = DVec3::splat(f64::NEG_INFINITY);
    if let Some(d) = model.defs.get(def) {
        for v in d.mesh.vertices.values() {
            let p = xf.transform_point3(v.pos);
            lo = lo.min(p);
            hi = hi.max(p);
        }
        for i in d.mesh.instances.values() {
            let (a, b) = bounds(model, i.def, xf * i.transform);
            lo = lo.min(a);
            hi = hi.max(b);
        }
    }
    (lo, hi)
}

/// The material covering most of a definition's own faces.
fn face_material(model: &Model, def: DefId) -> Option<MaterialId> {
    let d = model.defs.get(def)?;
    let mut by: Vec<(MaterialId, f64)> = Vec::new();
    for (f, face) in &d.mesh.faces {
        let Some(m) = face.front else { continue };
        let a: f64 = triangulate_face(&d.mesh, f).iter().map(|t| (t[1] - t[0]).cross(t[2] - t[0]).length() * 0.5).sum();
        match by.iter_mut().find(|(id, _)| *id == m) {
            Some(e) => e.1 += a,
            None => by.push((m, a)),
        }
    }
    by.into_iter().max_by(|a, b| a.1.total_cmp(&b.1)).map(|(m, _)| m)
}

/// The parts of the visible model: leaf groups and components, equal ones counted together.
pub fn cut_parts(model: &Model) -> Vec<CutPart> {
    fn walk(model: &Model, def: DefId, path: &mut ContextPath, inherited: Option<MaterialId>, out: &mut Vec<CutPart>) {
        let Some(d) = model.defs.get(def) else { return };
        for (id, inst) in d.mesh.instances.iter().filter(|(_, i)| !i.hidden) {
            let Some(cd) = model.defs.get(inst.def) else { continue };
            let mat = inst.material.or(inherited);
            path.push(id);
            if cd.mesh.instances.values().any(|i| !i.hidden) {
                walk(model, inst.def, path, mat, out);
            } else if !cd.mesh.faces.is_empty() {
                let (lo, hi) = bounds(model, inst.def, DMat4::IDENTITY);
                let s = axis_scales(&inst.transform) * (hi - lo).max(DVec3::ZERO);
                let mut size = [s.x, s.y, s.z].map(|v| (v * 1e3).round() / 1e3);
                size.sort_by(|a, b| b.total_cmp(a));
                let material = mat.or_else(|| face_material(model, inst.def));
                let part = CutPart {
                    name: if inst.name.is_empty() { cd.name.clone() } else { inst.name.clone() },
                    material,
                    material_name: material.and_then(|m| model.materials.get(m)).map(|m| m.name.clone()).unwrap_or_default(),
                    size,
                    count: 1,
                    paths: vec![path.clone()],
                };
                match out.iter_mut().find(|p| p.name == part.name && p.material == part.material && p.size == part.size) {
                    Some(p) => {
                        p.count += 1;
                        p.paths.push(path.clone());
                    }
                    None => out.push(part),
                }
            }
            path.pop();
        }
    }
    let mut out = Vec::new();
    walk(model, model.root, &mut Vec::new(), None, &mut out);
    out
}

/// A rectangle to pack: `w` along the sheet's length when it cannot turn.
#[derive(Clone, Copy, Debug)]
pub struct Item {
    pub id: usize,
    pub w: f64,
    pub h: f64,
    pub can_rotate: bool,
}

#[derive(Clone, Copy, Debug)]
struct Free {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

const EPS: f64 = 1e-6;

#[derive(Clone, Copy)]
enum Order {
    Area,
    LongSide,
    Perimeter,
    Width,
    Height,
}

#[derive(Clone, Copy)]
enum Fit {
    Area,
    ShortSide,
    LongSide,
}

#[derive(Clone, Copy)]
enum Split {
    ShorterLeftover,
    LongerLeftover,
    MinArea,
    MaxArea,
}

struct Sheet {
    free: Vec<Free>,
    placed: Vec<(usize, f64, f64, f64, f64, bool)>,
}

/// One packing attempt; items larger than an empty sheet are skipped.
fn pack_once(items: &[Item], len: f64, wid: f64, kerf: f64, order: Order, fit: Fit, split: Split) -> Vec<Sheet> {
    let mut sorted: Vec<Item> = items.to_vec();
    let key = |i: &Item| match order {
        Order::Area => i.w * i.h,
        Order::LongSide => i.w.max(i.h),
        Order::Perimeter => i.w + i.h,
        Order::Width => i.w,
        Order::Height => i.h,
    };
    sorted.sort_by(|a, b| key(b).total_cmp(&key(a)).then(b.w.max(b.h).total_cmp(&a.w.max(a.h))));
    let mut sheets: Vec<Sheet> = Vec::new();
    for it in sorted {
        let score = |f: &Free, w: f64, h: f64| -> Option<(f64, f64)> {
            if w > f.w + EPS || h > f.h + EPS {
                return None;
            }
            let (dw, dh) = (f.w - w, f.h - h);
            Some(match fit {
                Fit::Area => (f.w * f.h - w * h, dw.min(dh)),
                Fit::ShortSide => (dw.min(dh), dw.max(dh)),
                Fit::LongSide => (dw.max(dh), dw.min(dh)),
            })
        };
        let mut best: Option<((f64, f64), usize, usize, bool)> = None;
        for (si, s) in sheets.iter().enumerate() {
            for (fi, f) in s.free.iter().enumerate() {
                for rot in [false, true] {
                    if rot && !it.can_rotate {
                        continue;
                    }
                    let (w, h) = if rot { (it.h, it.w) } else { (it.w, it.h) };
                    if let Some(sc) = score(f, w, h)
                        && best.is_none_or(|(b, ..)| sc.0 < b.0 - EPS || (sc.0 < b.0 + EPS && sc.1 < b.1 - EPS))
                    {
                        best = Some((sc, si, fi, rot));
                    }
                }
            }
        }
        let (si, fi, rot) = match best {
            Some((_, si, fi, rot)) => (si, fi, rot),
            None => {
                let fits = |w: f64, h: f64| w <= len + EPS && h <= wid + EPS;
                let rot = !fits(it.w, it.h) && it.can_rotate && fits(it.h, it.w);
                if !fits(it.w, it.h) && !rot {
                    continue;
                }
                sheets.push(Sheet { free: vec![Free { x: 0.0, y: 0.0, w: len, h: wid }], placed: Vec::new() });
                (sheets.len() - 1, 0, rot)
            }
        };
        let s = &mut sheets[si];
        let f = s.free.swap_remove(fi);
        let (w, h) = if rot { (it.h, it.w) } else { (it.w, it.h) };
        s.placed.push((it.id, f.x, f.y, w, h, rot));
        // the kerf goes with the piece, except at the free rectangle's far edge
        let cw = (w + kerf).min(f.w);
        let ch = (h + kerf).min(f.h);
        let (rw, th) = (f.w - cw, f.h - ch);
        let horizontal = match split {
            Split::ShorterLeftover => rw < th,
            Split::LongerLeftover => rw >= th,
            Split::MinArea => rw * h < cw * th,
            Split::MaxArea => rw * h >= cw * th,
        };
        // horizontal: the strip above spans the whole width; otherwise the strip to the right
        // spans the whole height
        let (right, top) = if horizontal {
            (Free { x: f.x + cw, y: f.y, w: rw, h: ch }, Free { x: f.x, y: f.y + ch, w: f.w, h: th })
        } else {
            (Free { x: f.x + cw, y: f.y, w: rw, h: f.h }, Free { x: f.x, y: f.y + ch, w: cw, h: th })
        };
        for r in [right, top] {
            if r.w > EPS && r.h > EPS {
                s.free.push(r);
            }
        }
    }
    sheets
}

/// Items packed onto sheets `len` × `wid` (already trimmed); the best of all attempts. Items
/// larger than a sheet are left out (see [`too_big_for_sheet`]).
pub fn pack_sheets(items: &[Item], len: f64, wid: f64, kerf: f64, min_offcut: f64) -> Vec<Layout> {
    let mut best: Option<(usize, f64, Vec<Sheet>)> = None;
    for order in [Order::Area, Order::LongSide, Order::Perimeter, Order::Width, Order::Height] {
        for fit in [Fit::Area, Fit::ShortSide, Fit::LongSide] {
            for split in [Split::ShorterLeftover, Split::LongerLeftover, Split::MinArea, Split::MaxArea] {
                let sheets = pack_once(items, len, wid, kerf, order, fit, split);
                let largest = sheets.iter().flat_map(|s| s.free.iter()).map(|f| f.w * f.h).fold(0.0, f64::max);
                if best.as_ref().is_none_or(|(n, l, _)| sheets.len() < *n || (sheets.len() == *n && largest > *l + EPS)) {
                    best = Some((sheets.len(), largest, sheets));
                }
            }
        }
    }
    let sheets = best.map(|b| b.2).unwrap_or_default();
    sheets
        .into_iter()
        .map(|s| {
            let mut pieces: Vec<Placement> =
                s.placed.iter().map(|&(part, x, y, w, h, rotated)| Placement { part, x, y, w, h, rotated }).collect();
            pieces.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
            let mut offcuts: Vec<[f64; 4]> =
                s.free.iter().filter(|f| f.w.min(f.h) >= min_offcut - EPS).map(|f| [f.x, f.y, f.w, f.h]).collect();
            offcuts.sort_by(|a, b| (b[2] * b[3]).total_cmp(&(a[2] * a[3])));
            Layout { length: len, width: wid, pieces, offcuts }
        })
        .collect()
}

/// Whether an item fits on an empty sheet at all.
pub fn too_big_for_sheet(it: &Item, len: f64, wid: f64) -> bool {
    let fits = |w: f64, h: f64| w <= len + EPS && h <= wid + EPS;
    !(fits(it.w, it.h) || (it.can_rotate && fits(it.h, it.w)))
}

/// A piece of board: (id, length, width).
pub type BoardPiece = (usize, f64, f64);

/// Pieces `(id, length, width)` cut from boards of `lengths` (shortest first); pieces longer than the
/// longest board are left out. Each layout is `width` wide, its pieces at y = 0 with `h` = width.
pub fn pack_boards(pieces: &[BoardPiece], lengths: &[f64], width: f64, kerf: f64, min_offcut: f64) -> Vec<Layout> {
    let Some(&longest) = lengths.last() else { return Vec::new() };
    let mut sorted: Vec<BoardPiece> = pieces.iter().copied().filter(|p| p.1 <= longest + EPS).collect();
    sorted.sort_by(|a, b| b.1.total_cmp(&a.1));
    // per board: the used length (pieces and the kerfs between them) and its pieces
    let mut boards: Vec<(f64, Vec<BoardPiece>)> = Vec::new();
    for p in sorted {
        let at = |used: f64| if used > 0.0 { used + kerf } else { 0.0 };
        let slot = boards
            .iter_mut()
            .filter(|(used, _)| at(*used) + p.1 <= longest + EPS)
            .min_by(|a, b| (longest - at(a.0) - p.1).total_cmp(&(longest - at(b.0) - p.1)));
        match slot {
            Some((used, list)) => {
                *used = at(*used) + p.1;
                list.push(p);
            }
            None => boards.push((p.1, vec![p])),
        }
    }
    boards
        .into_iter()
        .map(|(used, list)| {
            let length = lengths.iter().copied().find(|l| *l + EPS >= used).unwrap_or(longest);
            let mut x = 0.0;
            let pieces: Vec<Placement> = list
                .iter()
                .map(|&(part, len, h)| {
                    let p = Placement { part, x, y: 0.0, w: len, h, rotated: false };
                    x += len + kerf;
                    p
                })
                .collect();
            let rest = length - used - kerf;
            let offcuts = if rest >= min_offcut - EPS { vec![[length - rest, 0.0, rest, width]] } else { Vec::new() };
            Layout { length, width, pieces, offcuts }
        })
        .collect()
}

/// The cut list of the model with its settings.
pub fn plan(model: &Model) -> CutPlan {
    plan_parts(model, cut_parts(model), &model.cut_list)
}

/// The cut list of `parts` (from [`cut_parts`]).
pub fn plan_parts(model: &Model, parts: Vec<CutPart>, settings: &CutSettings) -> CutPlan {
    let mut issues = Vec::new();
    // parts per (material, stock index)
    let mut groups: BTreeMap<(Option<MaterialId>, usize), Vec<usize>> = BTreeMap::new();
    for (i, p) in parts.iter().enumerate() {
        if settings.options(&p.name).exclude {
            continue;
        }
        let stocks = p.material.and_then(|m| model.materials.get(m)).map(|m| m.stock.as_slice()).unwrap_or_default();
        let Some((si, s)) = stocks.iter().enumerate().min_by(|a, b| (a.1.thickness - p.size[2]).abs().total_cmp(&(b.1.thickness - p.size[2]).abs()))
        else {
            issues.push(Issue::NoStock { part: i });
            continue;
        };
        if (s.thickness - p.size[2]).abs() > settings.tolerance + EPS {
            issues.push(Issue::Thickness { part: i, stock: s.thickness });
        }
        groups.entry((p.material, si)).or_default().push(i);
    }
    let mut plans = Vec::new();
    for ((material, si), members) in groups {
        let mat = material.and_then(|m| model.materials.get(m));
        let Some(stock) = mat.and_then(|m| m.stock.get(si)).cloned() else { continue };
        let layouts = match stock.kind {
            StockKind::Sheet => {
                let len = stock.length - 2.0 * settings.trim;
                let wid = stock.width - 2.0 * settings.trim;
                let mut items = Vec::new();
                for &i in &members {
                    let p = &parts[i];
                    let grain = settings.options(&p.name).grain;
                    let (w, h) = match grain {
                        PartGrain::Width => (p.size[1], p.size[0]),
                        _ => (p.size[0], p.size[1]),
                    };
                    let item = Item { id: i, w, h, can_rotate: !stock.grain || grain == PartGrain::Any };
                    if too_big_for_sheet(&item, len, wid) {
                        issues.push(Issue::TooBig { part: i });
                        continue;
                    }
                    items.extend(std::iter::repeat_n(item, p.count));
                }
                let mut layouts = pack_sheets(&items, len, wid, settings.kerf, settings.min_offcut);
                // back to the whole sheet's coordinates
                for l in &mut layouts {
                    l.length = stock.length;
                    l.width = stock.width;
                    for p in &mut l.pieces {
                        p.x += settings.trim;
                        p.y += settings.trim;
                    }
                    for o in &mut l.offcuts {
                        o[0] += settings.trim;
                        o[1] += settings.trim;
                    }
                }
                layouts
            }
            StockKind::Board => {
                let lengths = stock.board_lengths();
                let longest = lengths.last().copied().unwrap_or(0.0);
                let mut pieces = Vec::new();
                for &i in &members {
                    let p = &parts[i];
                    if p.size[0] > longest + EPS || p.size[1] > stock.width + settings.tolerance + EPS {
                        issues.push(Issue::TooBig { part: i });
                        continue;
                    }
                    pieces.extend(std::iter::repeat_n((i, p.size[0], p.size[1].min(stock.width)), p.count));
                }
                pack_boards(&pieces, &lengths, stock.width, settings.kerf, settings.min_offcut)
            }
        };
        if !layouts.is_empty() {
            plans.push(StockPlan { material, material_name: mat.map(|m| m.name.clone()).unwrap_or_default(), stock, layouts });
        }
    }
    CutPlan { parts, plans, issues }
}

fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n', ';']) { format!("\"{}\"", s.replace('"', "\"\"")) } else { s.to_string() }
}

fn num(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}

/// The cut list as CSV in the model's units: the parts, a blank line, the stock to buy.
pub fn plan_csv(model: &Model, plan: &CutPlan) -> String {
    let u = model.unit;
    let (k, sx) = (u.mm(), u.suffix());
    let mut s = String::new();
    // i18n-ignore-next
    let _ = writeln!(s, "Part,Count,Length ({sx}),Width ({sx}),Thickness ({sx}),Material,Grain,Note");
    for (i, p) in plan.parts.iter().enumerate() {
        let o = model.cut_list.options(&p.name);
        let note = if o.exclude {
            // i18n-ignore-next
            "excluded".to_string()
        } else {
            plan.issues
                .iter()
                .filter_map(|is| match *is {
                    // i18n-ignore-next
                    Issue::NoStock { part } if part == i => Some("no stock".to_string()),
                    // i18n-ignore-next
                    Issue::TooBig { part } if part == i => Some("larger than the stock".to_string()),
                    // i18n-ignore-next
                    Issue::Thickness { part, stock } if part == i => Some(format!("cut from {}", num(stock / k))),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("; ")
        };
        let grain = match o.grain {
            // i18n-ignore-next
            PartGrain::Length => "length",
            // i18n-ignore-next
            PartGrain::Width => "width",
            // i18n-ignore-next
            PartGrain::Any => "any",
        };
        let _ = writeln!(
            s,
            "{},{},{},{},{},{},{},{}",
            csv_field(&p.name),
            p.count,
            num(p.size[0] / k),
            num(p.size[1] / k),
            num(p.size[2] / k),
            csv_field(&p.material_name),
            grain,
            csv_field(&note)
        );
    }
    // i18n-ignore-next
    let _ = writeln!(s, "\nMaterial,Stock,Length ({sx}),Count,Waste (%),Cost");
    for sp in &plan.plans {
        let st = &sp.stock;
        let what = match st.kind {
            StockKind::Sheet => format!("{} × {} × {}", num(st.length / k), num(st.width / k), num(st.thickness / k)),
            StockKind::Board => format!("{} × {}", num(st.width / k), num(st.thickness / k)),
        };
        let waste = num((sp.waste() * 1000.0).round() / 10.0);
        for (len, n) in sp.to_buy() {
            let cost = match st.kind {
                StockKind::Sheet => n as f64 * st.price,
                StockKind::Board => n as f64 * len / 1000.0 * st.price,
            };
            let _ = writeln!(s, "{},{},{},{},{},{}", csv_field(&sp.material_name), csv_field(&what), num(len / k), n, waste, num(cost));
        }
    }
    s
}
