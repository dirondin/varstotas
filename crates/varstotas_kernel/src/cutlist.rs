//! Cut list (track W, `docs/WORKSHOP-PLAN.md`): the parts of the model laid out on the stock
//! they are cut from.
//!
//! A material can carry [`Stock`]: sheets (plywood, MDF: length × width × thickness, grain along
//! the length or none), boards (a section width × thickness, the lengths on sale) or linear stock
//! (a named profile cut to length, its section only a limit). Parts are the
//! leaf groups / components of the visible model (no visible instance inside), sized like the
//! parts list (`report.rs`: the bounding box in their own axes, length ≥ width ≥ thickness);
//! equal parts share a row with a count. A part's material is its instance's (inherited), else
//! the material covering most of its faces; its thickness picks that material's stock.
//!
//! Sheets are packed with guillotine cuts (every cut runs right through the piece, as on a panel
//! or track saw): free rectangles split after each placement, several part orders and placement
//! and split rules tried, the plan with the fewest sheets (then the largest offcut) kept. Boards
//! are 1D cutting stock: first fit decreasing over the longest length, then each board shrunk
//! to the cheapest length on sale that holds its pieces. The kerf is left after every piece
//! except at the stock's edge. Everything is in millimetres.
//!
//! A material can carry several stock items. A part takes the nearest thickness; among boards of
//! that thickness the narrowest that holds the part's width; among sheets of that thickness the
//! optimizer picks the size: the cheapest that holds the parts (by price, by area while some size
//! has no price), then the last sheet is cut from a cheaper size when its pieces fit there.
//! Linear stock comes first when its section holds the part (the smallest such profile).
//!
//! Offcuts kept from earlier projects ([`Offcut`], [`plan_with`]) are cut from before anything
//! is bought: the smallest first, as many of each as there are. After cutting,
//! [`CutPlan::keep_offcuts`] takes the used ones out and puts the plan's leftovers in.
//!
//! Parts can be edge-banded on any of their four edges ([`PartOptions::bands`]): the banding is
//! summed per part thickness into [`CutPlan::banding`], and with
//! [`CutSettings::band_subtract`] the parts are cut smaller by the band.

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
    /// Sawn timber: a section [`Stock::width`] × [`Stock::thickness`], cut to length from
    /// [`Stock::lengths`].
    Board,
    /// Mouldings, rods, profiles: cut to length like boards. The section (width × thickness,
    /// 0 = any) only has to hold the part's, and [`Stock::name`] says which profile it is.
    Linear,
}

/// One stock item of a material (a material can have several thicknesses).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stock {
    #[serde(default)]
    pub kind: StockKind,
    /// The profile of linear stock ("Ø 20 rod", "Skirting 70"), shown instead of its section.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
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
    /// Boards priced per piece instead of per metre: the price of each of [`Stock::lengths`]
    /// (a single value is the price of every length).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub piece_prices: Vec<f64>,
}

impl Stock {
    pub fn sheet(length: f64, width: f64, thickness: f64, grain: bool) -> Self {
        Self { kind: StockKind::Sheet, name: String::new(), thickness, length, width, lengths: Vec::new(), grain, price: 0.0, piece_prices: Vec::new() }
    }

    pub fn board(width: f64, thickness: f64, lengths: &[f64]) -> Self {
        let length = lengths.iter().copied().fold(0.0, f64::max);
        Self { kind: StockKind::Board, name: String::new(), thickness, length, width, lengths: lengths.to_vec(), grain: true, price: 0.0, piece_prices: Vec::new() }
    }

    /// Linear stock `name` with a section `width` × `thickness` (0 = any).
    pub fn linear(name: &str, width: f64, thickness: f64, lengths: &[f64]) -> Self {
        Self { kind: StockKind::Linear, name: name.to_string(), ..Self::board(width, thickness, lengths) }
    }

    /// Cut to length (boards and linear stock), not packed on a sheet.
    pub fn is_lengths(&self) -> bool {
        self.kind != StockKind::Sheet
    }

    /// Linear stock: whether its section holds a part's `w` × `t`, either way round (a side of
    /// 0 holds anything).
    pub fn holds_section(&self, w: f64, t: f64, tolerance: f64) -> bool {
        let le = |a: f64, s: f64| s <= 0.0 || a <= s + tolerance + EPS;
        (le(w, self.width) && le(t, self.thickness)) || (le(t, self.width) && le(w, self.thickness))
    }

    /// The same stock, size aside: kind, thickness and, cut to length, section and profile.
    fn same_kind(&self, o: &Stock) -> bool {
        self.kind == o.kind && (self.thickness - o.thickness).abs() < EPS && (!self.is_lengths() || ((self.width - o.width).abs() < EPS && self.name == o.name))
    }

    /// The price of one sheet, or of one board of `length`.
    pub fn cost_of(&self, length: f64) -> f64 {
        match self.kind {
            StockKind::Sheet => self.price,
            StockKind::Board | StockKind::Linear if self.piece_prices.is_empty() => length / 1000.0 * self.price,
            StockKind::Board | StockKind::Linear => {
                let i = self.lengths.iter().position(|l| (l - length).abs() < EPS);
                i.and_then(|i| self.piece_prices.get(i)).or(self.piece_prices.last()).copied().unwrap_or(0.0)
            }
        }
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

/// Leftover stock kept for later projects: `count` pieces of `stock` (unpriced) of the material
/// named `material`. Planned first, before any stock is bought (see [`plan_with`]).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Offcut {
    pub material: String,
    pub stock: Stock,
    pub count: usize,
    /// Where it was left over (the project's file), for the user.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source: String,
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
    /// Edge-banded edges: the two long edges (along the length), then the two short ones.
    #[serde(default, skip_serializing_if = "no_bands")]
    pub bands: [bool; 4],
}

fn no_bands(b: &[bool; 4]) -> bool {
    !b.iter().any(|&x| x)
}

impl PartOptions {
    /// The banded edges' length of one part (mm).
    pub fn band_length(&self, size: [f64; 3]) -> f64 {
        self.bands.iter().enumerate().filter(|(_, b)| **b).map(|(i, _)| if i < 2 { size[0] } else { size[1] }).sum()
    }
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
    /// Edge band thickness (mm).
    #[serde(default = "default_band")]
    pub band: f64,
    /// Parts are cut smaller by the band on their banded edges.
    #[serde(default)]
    pub band_subtract: bool,
    /// Edge band price per metre.
    #[serde(default)]
    pub band_price: f64,
}

fn default_band() -> f64 {
    1.0
}

impl Default for CutSettings {
    fn default() -> Self {
        Self { kerf: 3.0, trim: 0.0, min_offcut: 100.0, tolerance: 0.5, parts: BTreeMap::new(), band: default_band(), band_subtract: false, band_price: 0.0 }
    }
}

impl CutSettings {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    pub fn options(&self, part: &str) -> PartOptions {
        self.parts.get(part).cloned().unwrap_or_default()
    }

    /// The size a part is cut at: less the band on its banded edges when that is subtracted.
    pub fn cut_size(&self, name: &str, size: [f64; 3]) -> [f64; 3] {
        if !self.band_subtract {
            return size;
        }
        let b = self.options(name).bands.map(|x| if x { self.band } else { 0.0 });
        [size[0] - b[2] - b[3], size[1] - b[0] - b[1], size[2]]
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
    /// Size along x and y (the cut size).
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
    /// Cut from the offcut of this index (in the list given to [`plan_with`]), not bought.
    pub offcut: Option<usize>,
}

impl StockPlan {
    /// Sheets: the count × price; boards: the metres × price, or the price of each board.
    pub fn cost(&self) -> f64 {
        self.layouts.iter().map(|l| self.stock.cost_of(l.length)).sum()
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

/// Edge banding to buy for the parts of one thickness (the band's width).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Banding {
    /// The parts' thickness (mm).
    pub width: f64,
    /// The band's thickness (mm).
    pub thickness: f64,
    /// Total length (mm).
    pub length: f64,
    /// Price per metre.
    pub price: f64,
}

impl Banding {
    pub fn cost(&self) -> f64 {
        self.length / 1000.0 * self.price
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CutPlan {
    /// Every part row, excluded ones too (see [`CutSettings::parts`]).
    pub parts: Vec<CutPart>,
    pub plans: Vec<StockPlan>,
    pub issues: Vec<Issue>,
    /// Edge banding by part thickness, the thinnest first.
    pub banding: Vec<Banding>,
}

impl CutPlan {
    /// Stock and edge banding.
    pub fn cost(&self) -> f64 {
        self.plans.iter().map(StockPlan::cost).sum::<f64>() + self.banding.iter().map(Banding::cost).sum::<f64>()
    }

    /// What is left over after cutting: the offcuts of every sheet and board, equal ones
    /// counted together.
    pub fn leftovers(&self) -> Vec<Offcut> {
        let mut out: Vec<Offcut> = Vec::new();
        for sp in &self.plans {
            for o in sp.layouts.iter().flat_map(|l| &l.offcuts) {
                let mut st = Stock { price: 0.0, piece_prices: Vec::new(), ..sp.stock.clone() };
                if st.is_lengths() {
                    st.length = o[2];
                    st.lengths = vec![o[2]];
                } else {
                    (st.length, st.width) = (o[2], o[3]);
                }
                match out.iter_mut().find(|c| c.material == sp.material_name && c.stock == st) {
                    Some(c) => c.count += 1,
                    None => out.push(Offcut { material: sp.material_name.clone(), stock: st, count: 1, source: String::new() }),
                }
            }
        }
        out
    }

    /// After cutting: the offcuts this plan used are taken out of `inventory` (the list it was
    /// planned with) and its leftovers put in, from `source`.
    pub fn keep_offcuts(&self, inventory: &mut Vec<Offcut>, source: &str) {
        for sp in &self.plans {
            if let Some(o) = sp.offcut.and_then(|i| inventory.get_mut(i)) {
                o.count = o.count.saturating_sub(sp.layouts.len());
            }
        }
        inventory.retain(|o| o.count > 0);
        for n in self.leftovers() {
            match inventory.iter_mut().find(|o| o.material == n.material && o.stock == n.stock) {
                Some(o) => o.count += n.count,
                None => inventory.push(Offcut { source: source.to_string(), ..n }),
            }
        }
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
    pack_boards_priced(pieces, lengths, width, kerf, min_offcut, &|l| l)
}

/// [`pack_boards`] with each board bought at the cheapest length (by `cost`) that holds it.
pub fn pack_boards_priced(pieces: &[BoardPiece], lengths: &[f64], width: f64, kerf: f64, min_offcut: f64, cost: &dyn Fn(f64) -> f64) -> Vec<Layout> {
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
            let length = lengths.iter().copied().filter(|l| *l + EPS >= used).min_by(|a, b| cost(*a).total_cmp(&cost(*b)).then(a.total_cmp(b))).unwrap_or(longest);
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
    plan_with(model, &[])
}

/// [`plan`] with offcuts on hand, cut from before any stock (the smallest first that holds a part).
pub fn plan_with(model: &Model, offcuts: &[Offcut]) -> CutPlan {
    plan_parts_with(model, cut_parts(model), &model.cut_list, offcuts)
}

/// The stock a part is cut from: the indices of `stocks` (one board, or the sheet sizes of one
/// thickness for the optimizer to choose from), or `None` without stock.
fn stock_choice(stocks: &[Stock], p: &CutPart, tolerance: f64) -> Option<Vec<usize>> {
    // linear stock whose section holds the part's: the smallest section
    let section = |s: &Stock| if s.width > 0.0 && s.thickness > 0.0 { s.width * s.thickness } else { f64::MAX };
    let linear = || stocks.iter().enumerate().filter(|(_, s)| s.kind == StockKind::Linear);
    if let Some((i, _)) = linear().filter(|(_, s)| s.holds_section(p.size[1], p.size[2], tolerance)).min_by(|a, b| section(a.1).total_cmp(&section(b.1))) {
        return Some(vec![i]);
    }
    let Some(t) = stocks.iter().filter(|s| s.kind != StockKind::Linear).map(|s| s.thickness).min_by(|a, b| (a - p.size[2]).abs().total_cmp(&(b - p.size[2]).abs())) else {
        // only linear stock and none holds it: the largest, reported as too big
        return linear().max_by(|a, b| section(a.1).total_cmp(&section(b.1))).map(|(i, _)| vec![i]);
    };
    let same = |kind: StockKind| stocks.iter().enumerate().filter(move |(_, s)| s.kind == kind && (s.thickness - t).abs() < EPS);
    let narrowest = same(StockKind::Board).filter(|(_, s)| p.size[1] <= s.width + tolerance + EPS).min_by(|a, b| a.1.width.total_cmp(&b.1.width));
    if let Some((i, _)) = narrowest {
        return Some(vec![i]);
    }
    let sheets: Vec<usize> = same(StockKind::Sheet).map(|(i, _)| i).collect();
    if !sheets.is_empty() {
        return Some(sheets);
    }
    // too wide for every board of that thickness: the widest, reported as too big
    same(StockKind::Board).max_by(|a, b| a.1.width.total_cmp(&b.1.width)).map(|(i, _)| vec![i])
}

/// Sheets for `members` from the offcuts `offcuts` (index into the offcut list, stock, count),
/// then from the sheet sizes `stocks` (index into the material's stock, size): the layouts per
/// offcut index and per stock index.
#[allow(clippy::type_complexity)]
fn plan_sheets(
    stocks: &[(usize, &Stock)],
    offcuts: &[(usize, &Stock, usize)],
    members: &[usize],
    parts: &[CutPart],
    settings: &CutSettings,
    issues: &mut Vec<Issue>,
) -> (Vec<(usize, Vec<Layout>)>, Vec<(usize, Vec<Layout>)>) {
    let item = |st: &Stock, i: usize| {
        let p = &parts[i];
        let grain = settings.options(&p.name).grain;
        let (w, h) = match grain {
            PartGrain::Width => (p.size[1], p.size[0]),
            _ => (p.size[0], p.size[1]),
        };
        Item { id: i, w, h, can_rotate: !st.grain || grain == PartGrain::Any }
    };
    let usable = |st: &Stock| (st.length - 2.0 * settings.trim, st.width - 2.0 * settings.trim);
    let fits = |st: &Stock, i: usize| {
        let (l, w) = usable(st);
        !too_big_for_sheet(&item(st, i), l, w)
    };
    // compared by price, or by area while a size has no price
    let priced = stocks.iter().all(|(_, s)| s.price > 0.0);
    let unit = |st: &Stock| if priced { st.price } else { st.length * st.width / 1e6 };
    // an item across the grain is the part turned: `rotated` says how the part lies
    let turn = |layouts: &mut Vec<Layout>| {
        for p in layouts.iter_mut().flat_map(|l| &mut l.pieces) {
            p.rotated ^= settings.options(&parts[p.part].name).grain == PartGrain::Width;
        }
    };
    let pack = |st: &Stock, ids: &[usize]| {
        let (l, w) = usable(st);
        let items: Vec<Item> = ids.iter().map(|&i| item(st, i)).collect();
        let mut layouts = pack_sheets(&items, l, w, settings.kerf, settings.min_offcut);
        turn(&mut layouts);
        // back to the whole sheet's coordinates
        for l in &mut layouts {
            l.length = st.length;
            l.width = st.width;
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
    };
    // one id per piece
    let mut left: Vec<usize> = Vec::new();
    for &i in members {
        if stocks.iter().any(|(_, st)| fits(st, i)) {
            left.extend(std::iter::repeat_n(i, parts[i].count));
        } else {
            issues.push(Issue::TooBig { part: i });
        }
    }
    let take = |left: &mut Vec<usize>, layouts: &[Layout]| {
        for p in layouts.iter().flat_map(|l| &l.pieces) {
            if let Some(at) = left.iter().position(|&x| x == p.part) {
                left.swap_remove(at);
            }
        }
    };
    // offcuts first, the smallest first; their edges are sawn, so no trim
    let mut from_offcuts = Vec::new();
    let mut offs = offcuts.to_vec();
    offs.sort_by(|a, b| (a.1.length * a.1.width).total_cmp(&(b.1.length * b.1.width)));
    for (oi, st, n) in offs {
        let items: Vec<Item> = left.iter().map(|&i| item(st, i)).filter(|it| !too_big_for_sheet(it, st.length, st.width)).collect();
        if items.is_empty() {
            continue;
        }
        let mut layouts = pack_sheets(&items, st.length, st.width, settings.kerf, settings.min_offcut);
        layouts.truncate(n);
        turn(&mut layouts);
        take(&mut left, &layouts);
        from_offcuts.push((oi, layouts));
    }
    let mut out: Vec<(usize, Vec<Layout>)> = Vec::new();
    let mut add = |k: usize, layouts: Vec<Layout>| match out.iter_mut().find(|(j, _)| *j == k) {
        Some((_, l)) => l.extend(layouts),
        None => out.push((k, layouts)),
    };
    // all the pieces it holds on size `k`, the last sheet from a cheaper size when its pieces fit
    // there: the layouts per size and the cost
    let try_size = |k: usize, ids: &[usize]| {
        let mut layouts = pack(stocks[k].1, ids);
        let mut cost = layouts.len() as f64 * unit(stocks[k].1);
        let mut extra = None;
        if let Some(last) = layouts.last() {
            let ids: Vec<usize> = last.pieces.iter().map(|p| p.part).collect();
            let own = unit(stocks[k].1);
            let cheaper = stocks
                .iter()
                .enumerate()
                .filter(|(j, (_, st))| *j != k && ids.iter().all(|&i| fits(st, i)))
                .map(|(j, (_, st))| {
                    let l = pack(st, &ids);
                    let c = l.len() as f64 * unit(st);
                    (j, l, c)
                })
                .filter(|(_, _, c)| *c + EPS < own)
                .min_by(|a, b| a.2.total_cmp(&b.2));
            if let Some((j, l, c)) = cheaper {
                layouts.pop();
                cost += c - own;
                extra = Some((j, l));
            }
        }
        (layouts, extra, cost)
    };
    while !left.is_empty() {
        // the size that holds the most pieces, then the cheapest plan for them
        let best = (0..stocks.len())
            .filter_map(|k| {
                let ids: Vec<usize> = left.iter().copied().filter(|&i| fits(stocks[k].1, i)).collect();
                if ids.is_empty() {
                    return None;
                }
                let (layouts, extra, cost) = try_size(k, &ids);
                Some((k, ids, layouts, extra, cost))
            })
            .max_by(|a, b| a.1.len().cmp(&b.1.len()).then(b.4.total_cmp(&a.4)));
        let Some((k, ids, layouts, extra, _)) = best else { break };
        add(k, layouts);
        if let Some((j, l)) = extra {
            add(j, l);
        }
        for i in ids {
            if let Some(at) = left.iter().position(|&x| x == i) {
                left.swap_remove(at);
            }
        }
    }
    out.sort_by_key(|(k, _)| *k);
    (from_offcuts, out.into_iter().map(|(k, l)| (stocks[k].0, l)).collect())
}

/// The cut list of `parts` (from [`cut_parts`]).
pub fn plan_parts(model: &Model, parts: Vec<CutPart>, settings: &CutSettings) -> CutPlan {
    plan_parts_with(model, parts, settings, &[])
}

/// [`plan_parts`] with offcuts on hand (see [`plan_with`]).
pub fn plan_parts_with(model: &Model, parts: Vec<CutPart>, settings: &CutSettings, offcuts: &[Offcut]) -> CutPlan {
    let mut issues = Vec::new();
    let mut banding: Vec<Banding> = Vec::new();
    for p in parts.iter().filter(|p| !settings.options(&p.name).exclude) {
        let length = settings.options(&p.name).band_length(p.size) * p.count as f64;
        if length <= 0.0 {
            continue;
        }
        match banding.iter_mut().find(|b| (b.width - p.size[2]).abs() < EPS) {
            Some(b) => b.length += length,
            None => banding.push(Banding { width: p.size[2], thickness: settings.band, length, price: settings.band_price }),
        }
    }
    banding.sort_by(|a, b| a.width.total_cmp(&b.width));
    // packed at their cut size; the plan keeps the finished sizes
    let finished = parts;
    let parts: Vec<CutPart> = finished.iter().map(|p| CutPart { size: settings.cut_size(&p.name, p.size), ..p.clone() }).collect();
    // parts per (material, the stock indices to cut them from)
    let mut groups: BTreeMap<(Option<MaterialId>, Vec<usize>), Vec<usize>> = BTreeMap::new();
    for (i, p) in parts.iter().enumerate() {
        if settings.options(&p.name).exclude {
            continue;
        }
        let stocks = p.material.and_then(|m| model.materials.get(m)).map(|m| m.stock.as_slice()).unwrap_or_default();
        let Some(choice) = stock_choice(stocks, p, settings.tolerance) else {
            issues.push(Issue::NoStock { part: i });
            continue;
        };
        let t = stocks[choice[0]].thickness;
        if stocks[choice[0]].kind != StockKind::Linear && (t - p.size[2]).abs() > settings.tolerance + EPS {
            issues.push(Issue::Thickness { part: i, stock: t });
        }
        groups.entry((p.material, choice)).or_default().push(i);
    }
    let mut plans = Vec::new();
    for ((material, choice), members) in groups {
        let Some(mat) = material.and_then(|m| model.materials.get(m)) else { continue };
        let plan = |si: usize, layouts: Vec<Layout>| StockPlan { material, material_name: mat.name.clone(), stock: mat.stock[si].clone(), layouts, offcut: None };
        let offcut_plan = |oi: usize, layouts: Vec<Layout>| StockPlan {
            material,
            material_name: mat.name.clone(),
            stock: offcuts[oi].stock.clone(),
            layouts,
            offcut: Some(oi),
        };
        let stock = &mat.stock[choice[0]];
        let offs: Vec<(usize, &Stock, usize)> = offcuts
            .iter()
            .enumerate()
            .filter(|(_, o)| o.count > 0 && o.material == mat.name && o.stock.same_kind(stock))
            .map(|(i, o)| (i, &o.stock, o.count))
            .collect();
        if stock.kind == StockKind::Sheet {
            let sizes: Vec<(usize, &Stock)> = choice.iter().map(|&si| (si, &mat.stock[si])).collect();
            let (from_offcuts, bought) = plan_sheets(&sizes, &offs, &members, &parts, settings, &mut issues);
            plans.extend(from_offcuts.into_iter().map(|(oi, l)| offcut_plan(oi, l)));
            plans.extend(bought.into_iter().filter(|(_, l)| !l.is_empty()).map(|(si, l)| plan(si, l)));
            continue;
        }
        let linear = stock.kind == StockKind::Linear;
        let lengths = stock.board_lengths();
        let longest = lengths.last().copied().unwrap_or(0.0);
        let fits = |p: &CutPart| {
            p.size[0] <= longest + EPS
                && if linear { stock.holds_section(p.size[1], p.size[2], settings.tolerance) } else { p.size[1] <= stock.width + settings.tolerance + EPS }
        };
        // linear stock is drawn as wide as its section (or the widest part when it has none)
        let width = if linear && stock.width <= 0.0 { members.iter().map(|&i| parts[i].size[1]).fold(0.0, f64::max) } else { stock.width };
        let mut pieces = Vec::new();
        for &i in &members {
            let p = &parts[i];
            if !fits(p) {
                issues.push(Issue::TooBig { part: i });
                continue;
            }
            pieces.extend(std::iter::repeat_n((i, p.size[0], if linear { width } else { p.size[1].min(width) }), p.count));
        }
        // offcuts first, the shortest first
        let mut offs = offs;
        offs.sort_by(|a, b| a.1.length.total_cmp(&b.1.length));
        for (oi, st, n) in offs {
            let len = st.board_lengths().last().copied().unwrap_or(0.0);
            let fit: Vec<BoardPiece> = pieces.iter().copied().filter(|p| p.1 <= len + EPS).collect();
            let mut layouts = pack_boards(&fit, &[len], width, settings.kerf, settings.min_offcut);
            layouts.truncate(n);
            for p in layouts.iter().flat_map(|l| &l.pieces) {
                if let Some(at) = pieces.iter().position(|q| q.0 == p.part) {
                    pieces.remove(at);
                }
            }
            if !layouts.is_empty() {
                plans.push(offcut_plan(oi, layouts));
            }
        }
        let layouts = pack_boards_priced(&pieces, &lengths, width, settings.kerf, settings.min_offcut, &|l| stock.cost_of(l));
        if !layouts.is_empty() {
            plans.push(plan(choice[0], layouts));
        }
    }
    CutPlan { parts: finished, plans, issues, banding }
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
    let _ = writeln!(s, "Part,Count,Length ({sx}),Width ({sx}),Thickness ({sx}),Material,Grain,Banding,Note");
    for (i, p) in plan.parts.iter().enumerate() {
        let o = model.cut_list.options(&p.name);
        let cut = model.cut_list.cut_size(&p.name, p.size);
        let note = if o.exclude {
            // i18n-ignore-next
            "excluded".to_string()
        } else {
            // i18n-ignore-next
            let cut = (cut != p.size).then(|| format!("cut {} × {}", num(cut[0] / k), num(cut[1] / k)));
            cut.into_iter()
                .chain(plan.issues.iter()
                .filter_map(|is| match *is {
                    // i18n-ignore-next
                    Issue::NoStock { part } if part == i => Some("no stock".to_string()),
                    // i18n-ignore-next
                    Issue::TooBig { part } if part == i => Some("larger than the stock".to_string()),
                    // i18n-ignore-next
                    Issue::Thickness { part, stock } if part == i => Some(format!("cut from {}", num(stock / k))),
                    _ => None,
                }))
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
        // i18n-ignore-next
        let bands = ["L1", "L2", "W1", "W2"].iter().zip(o.bands).filter(|(_, b)| *b).map(|(n, _)| *n).collect::<Vec<_>>().join(" ");
        let _ = writeln!(
            s,
            "{},{},{},{},{},{},{},{},{}",
            csv_field(&p.name),
            p.count,
            num(p.size[0] / k),
            num(p.size[1] / k),
            num(p.size[2] / k),
            csv_field(&p.material_name),
            grain,
            bands,
            csv_field(&note)
        );
    }
    // i18n-ignore-next
    let _ = writeln!(s, "\nMaterial,Stock,Length ({sx}),Count,Waste (%),Cost");
    for sp in &plan.plans {
        let st = &sp.stock;
        let mut what = match st.kind {
            StockKind::Sheet => format!("{} × {} × {}", num(st.length / k), num(st.width / k), num(st.thickness / k)),
            StockKind::Linear if !st.name.is_empty() => st.name.clone(),
            _ => format!("{} × {}", num(st.width / k), num(st.thickness / k)),
        };
        if sp.offcut.is_some() {
            // i18n-ignore-next
            what.push_str(" (offcut)");
        }
        let waste = num((sp.waste() * 1000.0).round() / 10.0);
        for (len, n) in sp.to_buy() {
            let cost = n as f64 * st.cost_of(len);
            let _ = writeln!(s, "{},{},{},{},{},{}", csv_field(&sp.material_name), csv_field(&what), num(len / k), n, waste, num(cost));
        }
    }
    for b in &plan.banding {
        // i18n-ignore-next
        let _ = writeln!(s, "Edge banding,{} × {},{},1,,{}", num(b.width / k), num(b.thickness / k), num(b.length / k), num(b.cost()));
    }
    s
}
