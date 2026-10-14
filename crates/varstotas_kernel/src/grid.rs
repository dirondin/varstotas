//! Uniform hash grid used as a spatial index for vertices, edges and faces.
//!
//! Items are registered in every cell their geometry touches (conservatively):
//! - points: the cell containing them;
//! - segments: the cells overlapped by the bounding boxes of consecutive pieces of at most
//!   half a cell (a piece's box spans at most two cells per axis, so the cover is tight);
//! - boxes: all cells overlapped by the box.
//!
//! Items that would cover too many cells go to a "big" list that is returned by every query.
//! Queries visit exactly the cells overlapped by the query geometry (segment queries are padded
//! by [`QUERY_PAD`]), so every item whose geometry is within that pad of the query is found.
//! Results are deduplicated supersets: callers do their own exact tests.

use glam::DVec3;
use slotmap::{Key, SecondaryMap};
use std::collections::HashMap;

pub(crate) type Cell = (i64, i64, i64);

/// Max cells an item may occupy before it is stored in the "big" list.
const MAX_ITEM_CELLS: usize = 256;
/// Max cells a query may visit before it degrades to "return everything".
const MAX_QUERY_CELLS: usize = 20_000;
/// Padding of segment queries: comfortably larger than the merge tolerance.
pub(crate) const QUERY_PAD: f64 = crate::TOL * 4.0;

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
    /// Box enclosing all geometry ever inserted (conservative: never shrinks on removal).
    extent: Option<(DVec3, DVec3)>,
}

impl<K: Key + Ord> Default for Grid<K> {
    fn default() -> Self {
        Self::new(100.0)
    }
}

pub(crate) fn cell_of(p: DVec3, cell: f64) -> Cell {
    ((p.x / cell).floor() as i64, (p.y / cell).floor() as i64, (p.z / cell).floor() as i64)
}

/// Push all cells overlapped by the box `[min, max]`; `false` if that would exceed `max`.
fn push_box_cells(out: &mut Vec<Cell>, min: DVec3, max: DVec3, cell: f64, max_cells: usize) -> bool {
    let lo = cell_of(min, cell);
    let hi = cell_of(max, cell);
    let count = ((hi.0 - lo.0 + 1) as f64) * ((hi.1 - lo.1 + 1) as f64) * ((hi.2 - lo.2 + 1) as f64);
    if !count.is_finite() || out.len() as f64 + count > max_cells as f64 {
        return false;
    }
    for x in lo.0..=hi.0 {
        for y in lo.1..=hi.1 {
            for z in lo.2..=hi.2 {
                out.push((x, y, z));
            }
        }
    }
    true
}

/// Cells overlapped by the box `[min, max]`, or `None` if more than `max_cells`.
pub(crate) fn box_cells(min: DVec3, max: DVec3, cell: f64, max_cells: usize) -> Option<Vec<Cell>> {
    let mut out = Vec::new();
    push_box_cells(&mut out, min, max, cell, max_cells).then_some(out)
}

/// Cells overlapped by the segment `a..b` swept by a cube of half-size `pad`, or `None` if
/// more than `max_cells`. Sorted and deduplicated.
pub(crate) fn segment_cells(a: DVec3, b: DVec3, pad: f64, cell: f64, max_cells: usize) -> Option<Vec<Cell>> {
    let len = (b - a).length();
    if !len.is_finite() {
        return None;
    }
    let n = ((len / (cell * 0.5)).ceil() as usize).max(1);
    if n > max_cells {
        return None;
    }
    let d = DVec3::splat(pad);
    let mut out = Vec::with_capacity(n * 2);
    let mut prev = a;
    for i in 1..=n {
        let q = if i == n { b } else { a.lerp(b, i as f64 / n as f64) };
        // Generous cap during collection (duplicates between neighbouring pieces).
        if !push_box_cells(&mut out, prev.min(q) - d, prev.max(q) + d, cell, max_cells * 8) {
            return None;
        }
        prev = q;
    }
    out.sort_unstable();
    out.dedup();
    (out.len() <= max_cells).then_some(out)
}

impl<K: Key + Ord> Grid<K> {
    pub fn new(cell: f64) -> Self {
        Self { cell, cells: HashMap::new(), big: Vec::new(), entries: SecondaryMap::new(), extent: None }
    }

    fn insert_cells(&mut self, k: K, cells: Option<Vec<Cell>>) {
        self.remove(k);
        match cells {
            Some(cs) if cs.len() <= MAX_ITEM_CELLS => {
                for c in &cs {
                    self.cells.entry(*c).or_default().push(k);
                }
                self.entries.insert(k, Entry::Cells(cs));
            }
            _ => {
                self.big.push(k);
                self.entries.insert(k, Entry::Big);
            }
        }
    }

    fn grow(&mut self, min: DVec3, max: DVec3) {
        if !(min.is_finite() && max.is_finite()) {
            return;
        }
        self.extent = Some(match self.extent {
            Some((lo, hi)) => (lo.min(min), hi.max(max)),
            None => (min, max),
        });
    }

    /// A box containing every item's geometry (possibly larger), `None` if nothing (finite)
    /// was ever inserted.
    pub fn extent(&self) -> Option<(DVec3, DVec3)> {
        self.extent
    }

    pub fn insert_point(&mut self, k: K, p: DVec3) {
        self.grow(p, p);
        let c = p.is_finite().then(|| vec![cell_of(p, self.cell)]);
        self.insert_cells(k, c);
    }

    pub fn insert_segment(&mut self, k: K, a: DVec3, b: DVec3) {
        self.grow(a.min(b), a.max(b));
        let cs = segment_cells(a, b, 0.0, self.cell, MAX_ITEM_CELLS);
        self.insert_cells(k, cs);
    }

    pub fn insert_box(&mut self, k: K, min: DVec3, max: DVec3) {
        self.grow(min, max);
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

    fn collect(&self, cells: Option<Vec<Cell>>) -> Vec<K> {
        match cells {
            None => self.entries.keys().collect(),
            Some(cs) => {
                let mut out: Vec<K> = self.big.clone();
                for c in &cs {
                    if let Some(v) = self.cells.get(c) {
                        out.extend_from_slice(v);
                    }
                }
                // Only items registered in several visited cells can repeat.
                if cs.len() > 1 || !self.big.is_empty() {
                    out.sort_unstable();
                    out.dedup();
                }
                out
            }
        }
    }

    /// Items possibly within [`QUERY_PAD`] of the segment `a..b`.
    pub fn query_segment(&self, a: DVec3, b: DVec3) -> Vec<K> {
        self.collect(segment_cells(a, b, QUERY_PAD, self.cell, MAX_QUERY_CELLS))
    }

    /// Items possibly overlapping the box.
    pub fn query_box(&self, min: DVec3, max: DVec3) -> Vec<K> {
        self.collect(box_cells(min, max, self.cell, MAX_QUERY_CELLS))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Aabb, VertexId};
    use slotmap::SlotMap;

    /// Small deterministic PRNG (xorshift64*).
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }
        fn f(&mut self, lo: f64, hi: f64) -> f64 {
            lo + (hi - lo) * (self.next() >> 11) as f64 / (1u64 << 53) as f64
        }
        fn p(&mut self, s: f64) -> DVec3 {
            DVec3::new(self.f(-s, s), self.f(-s, s), self.f(-s, s))
        }
    }

    fn seg_seg_dist(a: DVec3, b: DVec3, c: DVec3, d: DVec3) -> f64 {
        crate::geom::seg_seg_closest(a, b, c, d)
            .map(|(_, _, p, q)| (p - q).length())
            .unwrap_or_else(|| {
                // parallel: min of endpoint distances
                [
                    crate::geom::point_seg_dist(a, c, d).0,
                    crate::geom::point_seg_dist(b, c, d).0,
                    crate::geom::point_seg_dist(c, a, b).0,
                    crate::geom::point_seg_dist(d, a, b).0,
                ]
                .into_iter()
                .fold(f64::MAX, f64::min)
            })
    }

    fn seg_box_overlap(a: DVec3, b: DVec3, bx: &Aabb) -> bool {
        // Sampled check is enough for the test: dense samples along the segment.
        (0..=400).any(|i| bx.contains(a.lerp(b, i as f64 / 400.0)))
    }

    #[test]
    fn randomized_equivalence_with_brute_force() {
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        for &cell in &[3.0, 10.0, 37.5] {
            let mut keys: SlotMap<VertexId, ()> = SlotMap::with_key();
            let mut pts: SecondaryMap<VertexId, DVec3> = SecondaryMap::new();
            let mut segs: SecondaryMap<VertexId, (DVec3, DVec3)> = SecondaryMap::new();
            let mut boxes: SecondaryMap<VertexId, Aabb> = SecondaryMap::new();
            let mut gp: Grid<VertexId> = Grid::new(cell);
            let mut gs: Grid<VertexId> = Grid::new(cell);
            let mut gb: Grid<VertexId> = Grid::new(cell);
            for i in 0..1500 {
                let k = keys.insert(());
                let p = rng.p(100.0);
                pts.insert(k, p);
                gp.insert_point(k, p);
                // mostly short segments, some long ones (big list)
                let q = if i % 50 == 0 { rng.p(100.0) } else { p + rng.p(8.0) };
                segs.insert(k, (p, q));
                gs.insert_segment(k, p, q);
                let bx = Aabb::from_points([p, p + rng.p(6.0)]).unwrap();
                boxes.insert(k, bx);
                gb.insert_box(k, bx.min, bx.max);
            }
            // remove / re-insert some items
            let ks: Vec<VertexId> = keys.keys().collect();
            for (i, k) in ks.iter().enumerate() {
                if i % 7 == 0 {
                    gp.remove(*k);
                    gs.remove(*k);
                    gb.remove(*k);
                    keys.remove(*k);
                    pts.remove(*k);
                    segs.remove(*k);
                    boxes.remove(*k);
                } else if i % 11 == 0 {
                    let p = rng.p(100.0);
                    let q = p + rng.p(5.0);
                    pts.insert(*k, p);
                    gp.insert_point(*k, p);
                    segs.insert(*k, (p, q));
                    gs.insert_segment(*k, p, q);
                }
            }
            for _ in 0..300 {
                // box queries
                let c = rng.p(100.0);
                let qb = Aabb::from_points([c, c + rng.p(10.0)]).unwrap();
                let got_p: std::collections::HashSet<_> = gp.query_box(qb.min, qb.max).into_iter().collect();
                for (k, p) in &pts {
                    if qb.contains(*p) {
                        assert!(got_p.contains(&k), "point missed by box query");
                    }
                }
                let got_b: std::collections::HashSet<_> = gb.query_box(qb.min, qb.max).into_iter().collect();
                for (k, bx) in &boxes {
                    if bx.intersects(&qb) {
                        assert!(got_b.contains(&k), "box missed by box query");
                    }
                }
                let got_s: std::collections::HashSet<_> = gs.query_box(qb.min, qb.max).into_iter().collect();
                for (k, (a, b)) in &segs {
                    if seg_box_overlap(*a, *b, &qb) {
                        assert!(got_s.contains(&k), "segment missed by box query");
                    }
                }
                // segment queries (including degenerate point queries)
                let a = rng.p(100.0);
                let b = if rng.next().is_multiple_of(3) { a } else { a + rng.p(30.0) };
                let got_s: std::collections::HashSet<_> = gs.query_segment(a, b).into_iter().collect();
                for (k, (c, d)) in &segs {
                    if seg_seg_dist(a, b, *c, *d) <= QUERY_PAD * 0.99 {
                        assert!(got_s.contains(&k), "segment missed by segment query");
                    }
                }
                let got_p: std::collections::HashSet<_> = gp.query_segment(a, b).into_iter().collect();
                for (k, p) in &pts {
                    if crate::geom::point_seg_dist(*p, a, b).0 <= QUERY_PAD * 0.99 {
                        assert!(got_p.contains(&k), "point missed by segment query");
                    }
                }
                // results contain no duplicates and no removed keys
                let v = gs.query_segment(a, b);
                let mut s = v.clone();
                s.sort_unstable();
                s.dedup();
                assert_eq!(s.len(), v.len());
                assert!(v.iter().all(|k| keys.contains_key(*k)));
            }
        }
    }

    #[test]
    fn exact_hits_near_cell_boundaries() {
        // Points within the pad of a segment that runs exactly along cell boundaries.
        let mut keys: SlotMap<VertexId, ()> = SlotMap::with_key();
        let mut g: Grid<VertexId> = Grid::new(10.0);
        let mut ps = Vec::new();
        for i in 0..=100 {
            let k = keys.insert(());
            let p = DVec3::new(i as f64 * 1.0, 10.0 + if i % 2 == 0 { 1e-4 } else { -1e-4 }, 0.0);
            g.insert_point(k, p);
            ps.push(k);
        }
        let got = g.query_segment(DVec3::new(0.0, 10.0, 0.0), DVec3::new(100.0, 10.0, 0.0));
        for k in ps {
            assert!(got.contains(&k));
        }
    }
}
