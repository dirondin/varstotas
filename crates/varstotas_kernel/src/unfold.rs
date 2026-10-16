//! Unfold: faces → flat patterns (paper models, sheet metal, plywood skins).
//!
//! Faces are joined into pieces along shared edges (longest edges first) as long as the new
//! face does not overlap the piece; a face that would overlap starts or joins another piece.
//! Folds are mountain (convex, the crease toward the printed side, which is the faces' front)
//! or valley (concave). Edges cut apart get a number on both sides and, optionally, a glue
//! tab on one side. Pieces are turned to their smallest bounding box and packed in rows on
//! sheets. Everything on the result is in paper millimetres, y up.

use crate::geom::{Projector, point_in_region, PointLoc};
use crate::model::Model;
use crate::{EntityRef, InstanceId};
use glam::{DMat4, DVec2, DVec3};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt::Write as _;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnfoldOptions {
    /// Paper mm per model mm (1:N → 1/N).
    pub scale: f64,
    pub tabs: bool,
    /// Glue tab width, paper mm.
    pub tab_width: f64,
    /// Matching numbers on edges cut apart.
    pub numbers: bool,
    /// Draw folds at soft edges (curved surfaces); off: the paper is just bent there.
    pub soft_folds: bool,
    /// Text height, paper mm.
    pub text: f64,
    /// Space between pieces, paper mm.
    pub gap: f64,
    /// Usable sheet area, paper mm (`None`: one sheet fitting all pieces).
    pub sheet: Option<DVec2>,
}

impl Default for UnfoldOptions {
    fn default() -> Self {
        Self { scale: 1.0, tabs: true, tab_width: 8.0, numbers: true, soft_folds: false, text: 3.0, gap: 6.0, sheet: None }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlatLine {
    Cut,
    Mountain,
    Valley,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlatLabel {
    /// Centre of the text.
    pub at: DVec2,
    /// Direction of the text, radians counter-clockwise (y up).
    pub angle: f64,
    pub size: f64,
    pub text: String,
}

#[derive(Clone, Debug, Default)]
pub struct FlatPiece {
    /// Face outlines (outer loop, holes), for previews and fills.
    pub faces: Vec<(Vec<DVec2>, Vec<Vec<DVec2>>)>,
    /// Glue tab outlines.
    pub tabs: Vec<Vec<DVec2>>,
    pub lines: Vec<(DVec2, DVec2, FlatLine)>,
    pub labels: Vec<FlatLabel>,
}

impl FlatPiece {
    fn points(&self) -> impl Iterator<Item = DVec2> + '_ {
        self.faces.iter().flat_map(|(o, h)| o.iter().chain(h.iter().flatten())).chain(self.tabs.iter().flatten()).copied()
    }

    /// Bounding box (min, max).
    pub fn bounds(&self) -> (DVec2, DVec2) {
        self.points().fold((DVec2::splat(f64::MAX), DVec2::splat(f64::MIN)), |(lo, hi), p| (lo.min(p), hi.max(p)))
    }

    fn map(&mut self, f: impl Fn(DVec2) -> DVec2, turn: f64) {
        for (o, hs) in &mut self.faces {
            for p in o.iter_mut().chain(hs.iter_mut().flatten()) {
                *p = f(*p);
            }
        }
        for p in self.tabs.iter_mut().flatten() {
            *p = f(*p);
        }
        for (a, b, _) in &mut self.lines {
            *a = f(*a);
            *b = f(*b);
        }
        for l in &mut self.labels {
            l.at = f(l.at);
            l.angle = readable(l.angle + turn);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Sheet {
    /// Size of the used area, paper mm (the requested sheet size, or the pieces' extent).
    pub size: DVec2,
    pub pieces: Vec<FlatPiece>,
}

#[derive(Clone, Debug, Default)]
pub struct Unfolding {
    pub sheets: Vec<Sheet>,
    pub faces: usize,
    pub pieces: usize,
    /// Pieces larger than the sheet (placed alone, overflowing).
    pub oversize: usize,
}

/// One face to unfold, world coordinates.
#[derive(Clone, Debug)]
pub struct UnfoldFace {
    pub outer: Vec<DVec3>,
    pub holes: Vec<Vec<DVec3>>,
    pub normal: DVec3,
}

/// Faces to unfold plus the soft edges among them (by endpoint keys, see [`point_key`]).
#[derive(Clone, Debug, Default)]
pub struct UnfoldInput {
    pub faces: Vec<UnfoldFace>,
    pub soft: HashSet<([i64; 3], [i64; 3])>,
}

/// Position key: coordinates rounded to 0.001 mm.
pub fn point_key(p: DVec3) -> [i64; 3] {
    let q = (p * 1000.0).round();
    [q.x as i64, q.y as i64, q.z as i64]
}

fn edge_key(a: [i64; 3], b: [i64; 3]) -> ([i64; 3], [i64; 3]) {
    if a <= b { (a, b) } else { (b, a) }
}

impl UnfoldInput {
    /// Faces of `ents` in the context `path` (instances: all their visible faces); empty
    /// `ents`: everything visible in the context.
    pub fn from_model(model: &Model, path: &[InstanceId], ents: &[EntityRef]) -> Self {
        let mut input = Self::default();
        let Some(def) = model.def_at(path) else { return input };
        let t = model.transform_at(path);
        let mesh = &model.defs[def].mesh;
        let all: Vec<EntityRef>;
        let ents = if ents.is_empty() {
            all = mesh
                .faces
                .keys()
                .map(EntityRef::Face)
                .chain(mesh.instances.keys().map(EntityRef::Instance))
                .collect();
            &all
        } else {
            ents
        };
        let mut seen = HashSet::new();
        for e in ents {
            match *e {
                EntityRef::Face(f) => input.add_face(model, def, f, &t, &mut seen),
                EntityRef::Instance(i) => {
                    if let Some(inst) = mesh.instances.get(i).filter(|i| !i.hidden && model.tag_visible(def, i.tag)) {
                        input.add_def(model, inst.def, &(t * inst.transform), 0);
                    }
                }
                _ => {}
            }
        }
        input
    }

    fn add_def(&mut self, model: &Model, def: crate::DefId, t: &DMat4, depth: usize) {
        let Some(d) = model.defs.get(def) else { return };
        if depth > 32 {
            return;
        }
        let mut seen = HashSet::new();
        for f in d.mesh.faces.keys() {
            self.add_face(model, def, f, t, &mut seen);
        }
        for inst in d.mesh.instances.values().filter(|i| !i.hidden && model.tag_visible(def, i.tag)) {
            self.add_def(model, inst.def, &(*t * inst.transform), depth + 1);
        }
    }

    fn add_face(&mut self, model: &Model, def: crate::DefId, f: crate::FaceId, t: &DMat4, seen: &mut HashSet<crate::FaceId>) {
        let mesh = &model.defs[def].mesh;
        let Some(face) = mesh.faces.get(f) else { return };
        if face.hidden || !model.tag_visible(def, face.tag) || !seen.insert(f) {
            return;
        }
        let w = |l: &[crate::VertexId]| -> Vec<DVec3> { l.iter().map(|v| t.transform_point3(mesh.pos(*v))).collect() };
        let outer = w(&face.outer);
        let holes: Vec<Vec<DVec3>> = face.holes.iter().map(|h| w(h)).collect();
        // Reflections reverse the loops' sense; the Newell normal follows the actual order.
        let normal = crate::geom::newell(&outer).normalize_or(t.transform_vector3(face.normal).normalize_or(DVec3::Z));
        for l in face.loops() {
            let n = l.len();
            for i in 0..n {
                let (a, b) = (l[i], l[(i + 1) % n]);
                if let Some(e) = mesh.edge_between(a, b)
                    && (mesh.edges[e].soft || mesh.edges[e].hidden)
                {
                    let (pa, pb) = (t.transform_point3(mesh.pos(a)), t.transform_point3(mesh.pos(b)));
                    self.soft.insert(edge_key(point_key(pa), point_key(pb)));
                }
            }
        }
        self.faces.push(UnfoldFace { outer, holes, normal });
    }
}

/// Keep text upright-ish: angles in (-90°, 90°].
fn readable(a: f64) -> f64 {
    let mut a = a.rem_euclid(std::f64::consts::TAU);
    if a > std::f64::consts::FRAC_PI_2 + 1e-9 && a <= 1.5 * std::f64::consts::PI + 1e-9 {
        a -= std::f64::consts::PI;
    }
    if a > std::f64::consts::PI {
        a -= std::f64::consts::TAU;
    }
    a
}

/// Optional mirror (a face oriented against its neighbours: its back faces the paper's
/// printed side), then rotation (cos, sin) and translation.
#[derive(Clone, Copy)]
struct Place {
    c: DVec2,
    t: DVec2,
    m: bool,
}

impl Place {
    const ID: Place = Place { c: DVec2::X, t: DVec2::ZERO, m: false };
    fn apply(&self, p: DVec2) -> DVec2 {
        let p = if self.m { DVec2::new(p.x, -p.y) } else { p };
        DVec2::new(self.c.x * p.x - self.c.y * p.y, self.c.y * p.x + self.c.x * p.y) + self.t
    }
}

struct Face {
    /// Loop 0 = outer, then holes: vertex indices and local (scaled) 2D points.
    loops: Vec<Vec<usize>>,
    local: Vec<Vec<DVec2>>,
    normal: DVec3,
    area: f64,
    /// Interior sample points (triangle centroids), local.
    samples: Vec<DVec2>,
}

impl Face {
    fn local_of(&self, v: usize) -> DVec2 {
        for (l, pts) in self.loops.iter().zip(&self.local) {
            if let Some(i) = l.iter().position(|x| *x == v) {
                return pts[i];
            }
        }
        DVec2::ZERO
    }
}

/// A placed face (or tab) for overlap tests.
struct Shape {
    loops: Vec<Vec<DVec2>>,
    samples: Vec<DVec2>,
    lo: DVec2,
    hi: DVec2,
}

impl Shape {
    fn new(loops: Vec<Vec<DVec2>>, samples: Vec<DVec2>) -> Self {
        let (lo, hi) = loops.iter().flatten().fold((DVec2::splat(f64::MAX), DVec2::splat(f64::MIN)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
        Self { loops, samples, lo, hi }
    }

    fn contains(&self, p: DVec2, tol: f64) -> bool {
        point_in_region(p, &self.loops[0], &self.loops[1..], tol) == PointLoc::Inside
    }

    fn overlaps(&self, o: &Shape, tol: f64) -> bool {
        if self.lo.x >= o.hi.x - tol || o.lo.x >= self.hi.x - tol || self.lo.y >= o.hi.y - tol || o.lo.y >= self.hi.y - tol {
            return false;
        }
        for l in &self.loops {
            for i in 0..l.len() {
                let (a, b) = (l[i], l[(i + 1) % l.len()]);
                for m in &o.loops {
                    for j in 0..m.len() {
                        if crosses(a, b, m[j], m[(j + 1) % m.len()], tol) {
                            return true;
                        }
                    }
                }
            }
        }
        self.samples.iter().any(|p| o.contains(*p, tol)) || o.samples.iter().any(|p| self.contains(*p, tol))
    }
}

/// Segments cross at a point inside both (touching and collinear runs do not count).
fn crosses(a: DVec2, b: DVec2, c: DVec2, d: DVec2, tol: f64) -> bool {
    let side = |p: DVec2, q: DVec2, r: DVec2| {
        let len = (q - p).length().max(1e-12);
        (q - p).perp_dot(r - p) / len
    };
    let (d1, d2) = (side(a, b, c), side(a, b, d));
    let (d3, d4) = (side(c, d, a), side(c, d, b));
    ((d1 > tol && d2 < -tol) || (d1 < -tol && d2 > tol)) && ((d3 > tol && d4 < -tol) || (d3 < -tol && d4 > tol))
}

fn samples_of(loops: &[Vec<DVec2>]) -> Vec<DVec2> {
    let holes: Vec<&[DVec2]> = loops[1..].iter().map(|h| h.as_slice()).collect();
    let all: Vec<DVec2> = loops.iter().flatten().copied().collect();
    let tris = crate::triangulate::triangulate(&loops[0], &holes);
    let mut s: Vec<DVec2> = tris.iter().map(|t| (all[t[0]] + all[t[1]] + all[t[2]]) / 3.0).collect();
    // Thin triangles' centroids hug the boundary: add mid points of the longest sides too.
    for t in &tris {
        let (p, q, r) = (all[t[0]], all[t[1]], all[t[2]]);
        s.push((p + q + r * 4.0) / 6.0);
    }
    s
}

/// An edge use: (face, loop, index of the start vertex in the loop).
type Use = (usize, usize, usize);

/// Unfold `input` into flat pieces on sheets.
pub fn unfold(input: &UnfoldInput, opts: &UnfoldOptions) -> Unfolding {
    let scale = if opts.scale.is_finite() && opts.scale > 0.0 { opts.scale } else { 1.0 };
    // Shared vertices by position.
    let mut keys: HashMap<[i64; 3], usize> = HashMap::new();
    let mut world: Vec<DVec3> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();
    for uf in &input.faces {
        let mut loops = Vec::new();
        let mut ok = true;
        for l in std::iter::once(&uf.outer).chain(&uf.holes) {
            let mut idx: Vec<usize> = Vec::new();
            for p in l {
                let k = point_key(*p);
                let i = *keys.entry(k).or_insert_with(|| {
                    world.push(*p);
                    world.len() - 1
                });
                if idx.last() != Some(&i) {
                    idx.push(i);
                }
            }
            while idx.len() > 1 && idx.first() == idx.last() {
                idx.pop();
            }
            if idx.len() < 3 {
                ok &= !loops.is_empty(); // a degenerate hole is dropped, a degenerate outer drops the face
                continue;
            }
            loops.push(idx);
        }
        if !ok || loops.is_empty() {
            continue;
        }
        let proj = Projector::new(world[loops[0][0]], uf.normal);
        let local: Vec<Vec<DVec2>> = loops.iter().map(|l| l.iter().map(|v| proj.to2(world[*v]) * scale).collect()).collect();
        let area = crate::geom::signed_area(&local[0]).abs() - local[1..].iter().map(|h| crate::geom::signed_area(h).abs()).sum::<f64>();
        if area <= 1e-9 {
            continue;
        }
        let samples = samples_of(&local);
        faces.push(Face { loops, local, normal: proj.n, area, samples });
    }
    let mut uses: HashMap<(usize, usize), Vec<Use>> = HashMap::new();
    for (fi, f) in faces.iter().enumerate() {
        for (li, l) in f.loops.iter().enumerate() {
            for i in 0..l.len() {
                let (a, b) = (l[i], l[(i + 1) % l.len()]);
                uses.entry((a.min(b), a.max(b))).or_default().push((fi, li, i));
            }
        }
    }
    let ends = |u: &Use| {
        let l = &faces[u.0].loops[u.1];
        (l[u.2], l[(u.2 + 1) % l.len()])
    };
    // Foldable: two faces share the edge (same direction: one of them gets mirrored).
    let foldable = |k: &(usize, usize)| -> Option<[Use; 2]> {
        let u = uses.get(k)?;
        (u.len() == 2 && u[0].0 != u[1].0).then(|| [u[0], u[1]])
    };
    let tol = 1e-6 * (1.0 + world.iter().map(|p| p.abs().max_element()).fold(0.0, f64::max) * scale);
    let mut place: Vec<Option<(Place, usize)>> = vec![None; faces.len()];
    let mut shapes: Vec<Vec<(usize, Shape)>> = Vec::new();
    let mut folds: HashSet<(usize, usize)> = HashSet::new();
    let mut order: Vec<usize> = (0..faces.len()).collect();
    order.sort_by(|a, b| faces[*b].area.total_cmp(&faces[*a].area));
    let placed_loops = |f: &Face, p: &Place| -> Vec<Vec<DVec2>> { f.local.iter().map(|l| l.iter().map(|q| p.apply(*q)).collect()).collect() };
    for &root in &order {
        if place[root].is_some() {
            continue;
        }
        let piece = shapes.len();
        place[root] = Some((Place::ID, piece));
        let f = &faces[root];
        shapes.push(vec![(root, Shape::new(f.local.clone(), f.samples.clone()))]);
        let mut heap: BinaryHeap<(u64, usize, usize)> = BinaryHeap::new();
        let push_edges = |heap: &mut BinaryHeap<(u64, usize, usize)>, fi: usize| {
            for l in &faces[fi].loops {
                for i in 0..l.len() {
                    let (a, b) = (l[i], l[(i + 1) % l.len()]);
                    let len = world[a].distance(world[b]);
                    heap.push((len.to_bits(), a.min(b), a.max(b)));
                }
            }
        };
        push_edges(&mut heap, root);
        while let Some((_, a, b)) = heap.pop() {
            let Some([u, w]) = foldable(&(a, b)) else { continue };
            let (from, to) = match (place[u.0], place[w.0]) {
                (Some((_, p)), None) if p == piece => (u, w),
                (None, Some((_, p))) if p == piece => (w, u),
                _ => continue,
            };
            let (pa, pb) = ends(&from);
            let parent = place[from.0].unwrap().0;
            let (fa, fb) = (parent.apply(faces[from.0].local_of(pa)), parent.apply(faces[from.0].local_of(pb)));
            let child = &faces[to.0];
            // Opposite directions keep the parent's side up; the same direction mirrors.
            let m = parent.m ^ (ends(&to) == (pa, pb));
            let pre = Place { m, ..Place::ID };
            let (la, lb) = (pre.apply(child.local_of(pa)), pre.apply(child.local_of(pb)));
            let ang = (fb - fa).y.atan2((fb - fa).x) - (lb - la).y.atan2((lb - la).x);
            let c = DVec2::new(ang.cos(), ang.sin());
            let mut p = Place { c, t: DVec2::ZERO, m };
            p.t = fa - p.apply(child.local_of(pa));
            let shape = Shape::new(placed_loops(child, &p), child.samples.iter().map(|q| p.apply(*q)).collect());
            if shapes[piece].iter().any(|(_, s)| s.overlaps(&shape, tol)) {
                continue;
            }
            place[to.0] = Some((p, piece));
            shapes[piece].push((to.0, shape));
            folds.insert((a, b));
            push_edges(&mut heap, to.0);
        }
    }
    // Lines, numbers and tabs per piece.
    let mut pieces: Vec<FlatPiece> = shapes
        .iter()
        .map(|s| FlatPiece {
            faces: s.iter().map(|(_, sh)| (sh.loops[0].clone(), sh.loops[1..].to_vec())).collect(),
            ..Default::default()
        })
        .collect();
    let placed_seg = |u: &Use| {
        let (a, b) = ends(u);
        let (p, _) = place[u.0].unwrap();
        (p.apply(faces[u.0].local_of(a)), p.apply(faces[u.0].local_of(b)))
    };
    let mut edge_keys: Vec<&(usize, usize)> = uses.keys().collect();
    edge_keys.sort();
    let mut number = 0;
    let mut tab_shapes: Vec<Vec<Shape>> = (0..pieces.len()).map(|_| Vec::new()).collect();
    let fold_line = |u: &Use, w: &Use| -> Option<FlatLine> {
        let (a, b) = ends(u);
        let side = |f: usize| if place[f].unwrap().0.m { -1.0 } else { 1.0 };
        // Normals of the printed sides; `u`'s direction as that side sees it.
        let (nf, ng) = (faces[u.0].normal * side(u.0), faces[w.0].normal * side(w.0));
        let along = (world[b] - world[a]) * side(u.0);
        if nf.dot(ng) > (0.5f64).to_radians().cos() {
            return None; // coplanar: no crease
        }
        let soft = input.soft.contains(&edge_key(point_key(world[a]), point_key(world[b])));
        if soft && !opts.soft_folds {
            return None;
        }
        Some(if nf.cross(ng).dot(along) > 0.0 { FlatLine::Mountain } else { FlatLine::Valley })
    };
    for k in edge_keys {
        let u = &uses[k];
        if let Some([x, y]) = foldable(k) {
            let (px, py) = (place[x.0].unwrap().1, place[y.0].unwrap().1);
            let (sx, sy) = (placed_seg(&x), placed_seg(&y));
            let near = |p: DVec2, q: DVec2| p.distance(q) < tol * 10.0;
            let same = (near(sx.0, sy.1) && near(sx.1, sy.0)) || (near(sx.0, sy.0) && near(sx.1, sy.1));
            let joined = folds.contains(k) || (px == py && same);
            if joined {
                if let Some(kind) = fold_line(&x, &y) {
                    pieces[px].lines.push((sx.0, sx.1, kind));
                }
                continue;
            }
            // Cut apart: lines on both sides, a shared number, one tab.
            pieces[px].lines.push((sx.0, sx.1, FlatLine::Cut));
            pieces[py].lines.push((sy.0, sy.1, FlatLine::Cut));
            number += 1;
            let mut tabbed = !opts.tabs;
            for (side, seg, pc) in [(x, sx, px), (y, sy, py)] {
                let (a, b) = seg;
                let len = a.distance(b);
                if len < 1e-9 {
                    continue;
                }
                let d = (b - a) / len;
                // The face lies left of its loops' direction, right when mirrored.
                let inward = if place[side.0].unwrap().0.m { -d.perp() } else { d.perp() };
                if opts.numbers {
                    let off = opts.text * 0.9;
                    if len > opts.text * 0.5 {
                        pieces[pc].labels.push(FlatLabel {
                            at: (a + b) * 0.5 + inward * off,
                            angle: readable(d.y.atan2(d.x)),
                            size: opts.text,
                            text: number.to_string(),
                        });
                    }
                }
                if tabbed {
                    continue;
                }
                let out = -inward;
                for w in [opts.tab_width, opts.tab_width * 0.5] {
                    let inset = w.min(len * 0.3);
                    let poly = vec![a, b, b + out * w - d * inset, a + out * w + d * inset];
                    let shape = Shape::new(vec![poly.clone()], samples_of(std::slice::from_ref(&poly)));
                    let clash = shapes[pc].iter().any(|(_, s)| s.overlaps(&shape, tol)) || tab_shapes[pc].iter().any(|s| s.overlaps(&shape, tol));
                    if !clash {
                        let fold = fold_line(&side, if side == x { &y } else { &x }).unwrap_or(FlatLine::Mountain);
                        let p = &mut pieces[pc];
                        // The cut line on this side becomes the tab's fold.
                        if let Some(l) = p.lines.iter_mut().rev().find(|l| l.0 == a && l.1 == b) {
                            l.2 = fold;
                        }
                        for i in 1..4 {
                            p.lines.push((poly[i], poly[(i + 1) % 4], FlatLine::Cut));
                        }
                        p.tabs.push(poly);
                        tab_shapes[pc].push(shape);
                        tabbed = true;
                        break;
                    }
                }
            }
        } else {
            // Open boundary or more than two faces: cut.
            for w in u {
                let s = placed_seg(w);
                pieces[place[w.0].unwrap().1].lines.push((s.0, s.1, FlatLine::Cut));
            }
        }
    }
    let n_pieces = pieces.len();
    let (sheets, oversize) = layout(pieces, opts);
    Unfolding { sheets, faces: faces.len(), pieces: n_pieces, oversize }
}

/// Turn each piece to its smallest bounding box, then pack rows on sheets.
fn layout(mut pieces: Vec<FlatPiece>, opts: &UnfoldOptions) -> (Vec<Sheet>, usize) {
    for p in &mut pieces {
        let pts: Vec<DVec2> = p.points().collect();
        let mut dirs: Vec<(f64, DVec2)> = p.lines.iter().map(|(a, b, _)| (a.distance(*b), *b - *a)).filter(|(l, _)| *l > 1e-9).collect();
        dirs.sort_by(|a, b| b.0.total_cmp(&a.0));
        dirs.truncate(64);
        let area = |d: DVec2| {
            let d = d.normalize();
            let (mut lo, mut hi) = (DVec2::splat(f64::MAX), DVec2::splat(f64::MIN));
            for q in &pts {
                let r = DVec2::new(q.dot(d), d.perp_dot(*q));
                lo = lo.min(r);
                hi = hi.max(r);
            }
            let s = hi - lo;
            (s.x * s.y, s)
        };
        let min = dirs.iter().map(|(_, d)| area(*d).0).fold(area(DVec2::X).0, f64::min);
        // Along the longest edge that is nearly as compact as the best (no odd diagonal turns
        // to save a corner of a tab).
        let mut d = dirs.iter().map(|(_, d)| d.normalize()).find(|d| area(*d).0 <= min * 1.15).unwrap_or(DVec2::X);
        // Landscape pieces (wider than tall), or whatever fits the sheet.
        let (_, s) = area(d);
        let fits = |s: DVec2| opts.sheet.is_none_or(|sh| s.x <= sh.x && s.y <= sh.y);
        if (s.y > s.x && fits(DVec2::new(s.y, s.x))) || !fits(s) && fits(DVec2::new(s.y, s.x)) {
            d = d.perp();
        }
        let turn = -d.y.atan2(d.x);
        let (c, sn) = (turn.cos(), turn.sin());
        p.map(|q| DVec2::new(c * q.x - sn * q.y, sn * q.x + c * q.y), turn);
        let (lo, _) = p.bounds();
        p.map(|q| q - lo, 0.0);
    }
    let sizes: Vec<DVec2> = pieces.iter().map(|p| p.bounds().1).collect();
    let mut idx: Vec<usize> = (0..pieces.len()).collect();
    idx.sort_by(|a, b| sizes[*b].y.total_cmp(&sizes[*a].y).then(sizes[*b].x.total_cmp(&sizes[*a].x)));
    let gap = opts.gap.max(0.0);
    let width = match opts.sheet {
        Some(s) => s.x,
        None => {
            let total: f64 = sizes.iter().map(|s| (s.x + gap) * (s.y + gap)).sum();
            let widest = sizes.iter().map(|s| s.x).fold(0.0, f64::max);
            widest.max(total.sqrt() * 1.4)
        }
    };
    let mut sheets: Vec<Sheet> = Vec::new();
    let mut oversize = 0;
    let (mut x, mut y, mut row_h) = (0.0, 0.0, 0.0f64);
    let mut cur: Vec<(usize, DVec2)> = Vec::new();
    let flush = |cur: &mut Vec<(usize, DVec2)>, sheets: &mut Vec<Sheet>, pieces: &mut Vec<Option<FlatPiece>>| {
        if cur.is_empty() {
            return;
        }
        let mut sheet = Sheet::default();
        for (i, at) in cur.drain(..) {
            let mut p = pieces[i].take().unwrap();
            p.map(|q| q + at, 0.0);
            sheet.pieces.push(p);
        }
        sheets.push(sheet);
    };
    let mut slots: Vec<Option<FlatPiece>> = pieces.into_iter().map(Some).collect();
    // Rows are filled top-down: a piece's y here is its distance from the top (flipped below).
    for &i in &idx {
        let s = sizes[i];
        if let Some(sh) = opts.sheet
            && (s.x > sh.x + 1e-9 || s.y > sh.y + 1e-9)
        {
            oversize += 1;
            flush(&mut cur, &mut sheets, &mut slots);
            cur.push((i, DVec2::ZERO));
            flush(&mut cur, &mut sheets, &mut slots);
            (x, y, row_h) = (0.0, 0.0, 0.0);
            continue;
        }
        if x > 0.0 && x + s.x > width + 1e-9 {
            x = 0.0;
            y += row_h + gap;
            row_h = 0.0;
        }
        if let Some(sh) = opts.sheet
            && y + s.y > sh.y + 1e-9
        {
            flush(&mut cur, &mut sheets, &mut slots);
            (x, y, row_h) = (0.0, 0.0, 0.0);
        }
        cur.push((i, DVec2::new(x, y)));
        x += s.x + gap;
        row_h = row_h.max(s.y);
    }
    flush(&mut cur, &mut sheets, &mut slots);
    // Flip rows to y up and size the sheets.
    for sh in &mut sheets {
        let hi = sh.pieces.iter().map(|p| p.bounds().1).fold(DVec2::ZERO, DVec2::max);
        let size = opts.sheet.map_or(hi, |s| s.max(hi));
        for p in &mut sh.pieces {
            // Its bottom is at the distance from the top: move it to size.y - (that + height).
            let (lo, top) = p.bounds();
            let dy = size.y - top.y - lo.y;
            p.map(|q| q + DVec2::new(0.0, dy), 0.0);
        }
        sh.size = size;
    }
    (sheets, oversize)
}

/// DXF (R12, mm) of the unfolding: layers CUT, MOUNTAIN, VALLEY, LABELS; sheets side by side.
pub fn to_dxf(u: &Unfolding) -> String {
    let mut s = String::from("0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n0\nSECTION\n2\nTABLES\n0\nTABLE\n2\nLAYER\n70\n4\n");
    for (name, color) in [("CUT", 7), ("MOUNTAIN", 1), ("VALLEY", 5), ("LABELS", 3)] {
        let _ = write!(s, "0\nLAYER\n2\n{name}\n70\n0\n62\n{color}\n6\nCONTINUOUS\n");
    }
    s.push_str("0\nENDTAB\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n");
    let mut ox = 0.0;
    for sheet in &u.sheets {
        for p in &sheet.pieces {
            for (a, b, kind) in &p.lines {
                let layer = match kind {
                    FlatLine::Cut => "CUT",
                    FlatLine::Mountain => "MOUNTAIN",
                    FlatLine::Valley => "VALLEY",
                };
                let _ = write!(s, "0\nLINE\n8\n{layer}\n10\n{}\n20\n{}\n30\n0\n11\n{}\n21\n{}\n31\n0\n", num(a.x + ox), num(a.y), num(b.x + ox), num(b.y));
            }
            for l in &p.labels {
                // Centre-aligned text (72 = 1, 73 = 2) anchored at the label centre.
                let _ = write!(
                    s,
                    "0\nTEXT\n8\nLABELS\n10\n{x}\n20\n{y}\n30\n0\n40\n{}\n1\n{}\n50\n{}\n72\n1\n73\n2\n11\n{x}\n21\n{y}\n31\n0\n",
                    num(l.size),
                    l.text,
                    num(l.angle.to_degrees()),
                    x = num(l.at.x + ox),
                    y = num(l.at.y)
                );
            }
        }
        ox += sheet.size.x + 20.0;
    }
    s.push_str("0\nENDSEC\n0\nEOF\n");
    s
}

fn num(v: f64) -> String {
    let s = format!("{:.4}", (v * 1e4).round() / 1e4);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}
