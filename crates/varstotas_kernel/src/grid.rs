//! Uniform hash grid used as a spatial index for vertices, edges and faces.
//!
//! Items are registered in a set of cells; items that would cover too many cells go to a
//! "big" list that is returned by every query. Segment-shaped items are registered by
//! sampling them every half cell; segment queries take the 27-neighbourhood of samples taken
//! the same way, which guarantees that any item within `cell/2` of the query is found.

use glam::DVec3;
use slotmap::{Key, SecondaryMap};
use std::collections::{HashMap, HashSet};

pub(crate) type Cell = (i64, i64, i64);

/// Max cells an item may occupy before it is stored in the "big" list.
const MAX_ITEM_CELLS: usize = 256;
/// Max cells a query may visit before it degrades to "return everything".
const MAX_QUERY_CELLS: usize = 20_000;

#[derive(Clone, Debug)]
enum Entry {
    Big,
    Cells(Vec<Cell>),
}

#[derive(Clone, Debug)]
pub(crate) struct Grid<K: Key> {
    pub cell: f64,
    cells: HashMap<Cell, Vec<K>>,
    big: Vec<K>,
    entries: SecondaryMap<K, Entry>,
}

impl<K: Key + Ord> Default for Grid<K> {
    fn default() -> Self {
        Self::new(100.0)
    }
}

/// Result of a cell computation: either a finite set of cells or "too many".
pub(crate) enum Cells {
    Some(Vec<Cell>),
    TooMany,
}

pub(crate) fn cell_of(p: DVec3, cell: f64) -> Cell {
    ((p.x / cell).floor() as i64, (p.y / cell).floor() as i64, (p.z / cell).floor() as i64)
}

/// Cells of samples taken along `a..b` every `cell/2` (both endpoints included).
pub(crate) fn segment_cells(a: DVec3, b: DVec3, cell: f64, max: usize) -> Cells {
    let len = (b - a).length();
    let n = (len / (cell * 0.5)).ceil() as usize + 1;
    if n > max {
        return Cells::TooMany;
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..=n {
        let t = i as f64 / n as f64;
        let c = cell_of(a.lerp(b, t), cell);
        if out.last() != Some(&c) {
            out.push(c);
        }
    }
    out.sort_unstable();
    out.dedup();
    Cells::Some(out)
}

/// All cells overlapped by the box `[min, max]`.
pub(crate) fn box_cells(min: DVec3, max: DVec3, cell: f64, max_cells: usize) -> Cells {
    let lo = cell_of(min, cell);
    let hi = cell_of(max, cell);
    let count = ((hi.0 - lo.0 + 1) as f64) * ((hi.1 - lo.1 + 1) as f64) * ((hi.2 - lo.2 + 1) as f64);
    if count > max_cells as f64 {
        return Cells::TooMany;
    }
    let mut out = Vec::with_capacity(count as usize);
    for x in lo.0..=hi.0 {
        for y in lo.1..=hi.1 {
            for z in lo.2..=hi.2 {
                out.push((x, y, z));
            }
        }
    }
    Cells::Some(out)
}

/// Cells to visit for a segment query: 27-neighbourhood of the segment samples.
pub(crate) fn segment_query_cells(a: DVec3, b: DVec3, cell: f64) -> Cells {
    match segment_cells(a, b, cell, MAX_QUERY_CELLS / 27) {
        Cells::TooMany => Cells::TooMany,
        Cells::Some(cs) => {
            let mut set = HashSet::with_capacity(cs.len() * 27);
            for c in cs {
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        for dz in -1..=1 {
                            set.insert((c.0 + dx, c.1 + dy, c.2 + dz));
                        }
                    }
                }
            }
            Cells::Some(set.into_iter().collect())
        }
    }
}

impl<K: Key + Ord> Grid<K> {
    pub fn new(cell: f64) -> Self {
        Self { cell, cells: HashMap::new(), big: Vec::new(), entries: SecondaryMap::new() }
    }

    fn insert_cells(&mut self, k: K, cells: Cells) {
        self.remove(k);
        match cells {
            Cells::TooMany => {
                self.big.push(k);
                self.entries.insert(k, Entry::Big);
            }
            Cells::Some(cs) => {
                if cs.len() > MAX_ITEM_CELLS {
                    self.big.push(k);
                    self.entries.insert(k, Entry::Big);
                    return;
                }
                for c in &cs {
                    self.cells.entry(*c).or_default().push(k);
                }
                self.entries.insert(k, Entry::Cells(cs));
            }
        }
    }

    pub fn insert_point(&mut self, k: K, p: DVec3) {
        let c = cell_of(p, self.cell);
        self.insert_cells(k, Cells::Some(vec![c]));
    }

    pub fn insert_segment(&mut self, k: K, a: DVec3, b: DVec3) {
        let cs = segment_cells(a, b, self.cell, MAX_ITEM_CELLS);
        self.insert_cells(k, cs);
    }

    pub fn insert_box(&mut self, k: K, min: DVec3, max: DVec3) {
        let cs = box_cells(min, max, self.cell, MAX_ITEM_CELLS);
        self.insert_cells(k, cs);
    }

    pub fn remove(&mut self, k: K) {
        let Some(e) = self.entries.remove(k) else { return };
        match e {
            Entry::Big => self.big.retain(|x| *x != k),
            Entry::Cells(cs) => {
                for c in cs {
                    if let Some(v) = self.cells.get_mut(&c) {
                        v.retain(|x| *x != k);
                        if v.is_empty() {
                            self.cells.remove(&c);
                        }
                    }
                }
            }
        }
    }

    fn collect(&self, cells: Cells) -> Vec<K> {
        match cells {
            Cells::TooMany => self.entries.keys().collect(),
            Cells::Some(cs) => {
                let mut out: Vec<K> = self.big.clone();
                for c in cs {
                    if let Some(v) = self.cells.get(&c) {
                        out.extend_from_slice(v);
                    }
                }
                out.sort_unstable();
                out.dedup();
                out
            }
        }
    }

    /// Items possibly within `cell/2` of the segment `a..b`.
    pub fn query_segment(&self, a: DVec3, b: DVec3) -> Vec<K> {
        self.collect(segment_query_cells(a, b, self.cell))
    }

    /// Items possibly overlapping the box.
    pub fn query_box(&self, min: DVec3, max: DVec3) -> Vec<K> {
        // Inflate by one cell so items registered by samples are found.
        let d = DVec3::splat(self.cell);
        self.collect(box_cells(min - d, max + d, self.cell, MAX_QUERY_CELLS))
    }
}
