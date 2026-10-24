//! UV editing on top of the per-face texture placements: islands (faces joined by shared
//! edges that are not seams), unfolding an island flat, projections (planar, box, cylinder,
//! sphere) and island transforms (move, rotate, flip, fit, pack, align, texel density).
//!
//! Texture coordinates here are millimetres of the texture plane ([`face_uv_mm`]); one tile
//! of a material spans its image size. Every function returns new placements
//! `(face, placement)` (`None` = the default projection) and leaves the mesh alone.

use crate::glam::{DVec2, DVec3};
use crate::texture::{TexPlacement, continue_placement, face_uv_mm, texture_axes};
use crate::{EdgeId, FaceId, Mesh};
use std::collections::{HashSet, VecDeque};

pub type Placements = Vec<(FaceId, Option<TexPlacement>)>;

/// Corner points of a face's outer loop.
pub fn corners(mesh: &Mesh, f: FaceId) -> Vec<DVec3> {
    mesh.faces.get(f).map_or_else(Vec::new, |face| face.outer.iter().filter_map(|v| mesh.vertices.get(*v).map(|x| x.pos)).collect())
}

/// Texture coordinates (mm) of a face's outer loop.
pub fn face_tex(mesh: &Mesh, f: FaceId) -> Vec<DVec2> {
    let Some(face) = mesh.faces.get(f) else { return Vec::new() };
    corners(mesh, f).into_iter().map(|p| face_uv_mm(p, face.normal, face.uv.as_ref())).collect()
}

/// Edges of the faces in `faces` (each once).
pub fn edges_of(mesh: &Mesh, faces: &[FaceId]) -> Vec<EdgeId> {
    let mut seen = HashSet::new();
    faces.iter().flat_map(|f| mesh.face_edges(*f)).filter(|e| seen.insert(*e)).collect()
}

/// Shared edges of `faces` that join two of them and are not seams.
fn joins(mesh: &Mesh, e: EdgeId) -> bool {
    mesh.edges.get(e).is_some_and(|ed| !ed.seam)
}

/// `faces` split into islands: faces joined by shared edges that are not seams.
pub fn islands(mesh: &Mesh, faces: &[FaceId]) -> Vec<Vec<FaceId>> {
    let set: HashSet<FaceId> = faces.iter().copied().collect();
    let mut done: HashSet<FaceId> = HashSet::new();
    let mut out = Vec::new();
    for &f in faces {
        if !done.insert(f) {
            continue;
        }
        let mut island = vec![f];
        let mut queue = VecDeque::from([f]);
        while let Some(g) = queue.pop_front() {
            for e in mesh.face_edges(g) {
                if !joins(mesh, e) {
                    continue;
                }
                for h in mesh.edge_faces(e) {
                    if set.contains(&h) && done.insert(h) {
                        island.push(h);
                        queue.push_back(h);
                    }
                }
            }
        }
        out.push(island);
    }
    out
}

/// The island unfolded flat from `root` (its placement kept): each face continues its
/// neighbour's texture across their shared edge, so the island is laid out with true lengths
/// and no seams inside it. Faces that can't be reached across non-seam edges keep theirs.
pub fn unfold(mesh: &Mesh, island: &[FaceId], root: FaceId) -> Placements {
    let set: HashSet<FaceId> = island.iter().copied().collect();
    let root_uv = mesh.faces.get(root).and_then(|f| f.uv);
    let mut placed: Placements = vec![(root, root_uv)];
    let mut queue = VecDeque::from([root]);
    while let Some(f) = queue.pop_front() {
        let uv = placed.iter().find(|(g, _)| *g == f).and_then(|x| x.1);
        for e in mesh.face_edges(f) {
            if !joins(mesh, e) {
                continue;
            }
            for g in mesh.edge_faces(e) {
                if !set.contains(&g) || placed.iter().any(|(h, _)| *h == g) {
                    continue;
                }
                if let Some(p) = continue_placement(mesh, f, uv.as_ref(), g) {
                    placed.push((g, p));
                    queue.push_back(g);
                }
            }
        }
    }
    placed
}

/// The face of `island` with the largest area (a stable root for unfolding).
pub fn largest_face(mesh: &Mesh, island: &[FaceId]) -> Option<FaceId> {
    island.iter().copied().max_by(|a, b| area(mesh, *a).total_cmp(&area(mesh, *b)))
}

/// World area of a face's outer loop.
pub fn area(mesh: &Mesh, f: FaceId) -> f64 {
    let p = corners(mesh, f);
    let mut s = DVec3::ZERO;
    for i in 0..p.len() {
        s += p[i].cross(p[(i + 1) % p.len()]);
    }
    s.length() / 2.0
}

/// Area of a polygon in the plane.
fn area2(q: &[DVec2]) -> f64 {
    let mut s = 0.0;
    for i in 0..q.len() {
        let (a, b) = (q[i], q[(i + 1) % q.len()]);
        s += a.x * b.y - a.y * b.x;
    }
    s.abs() / 2.0
}

/// How the texture is projected onto faces.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Projection {
    /// Straight along a direction onto the plane across it.
    Planar(DVec3),
    /// Each face along the world axis nearest its normal.
    Box,
    /// Wrapped round an axis through `origin`: u runs round it (mm at `radius`), v along it;
    /// faces across the axis (the ends) are projected along it.
    Cylinder { origin: DVec3, axis: DVec3, radius: f64 },
    /// Longitude / latitude round `center` (mm at `radius`).
    Sphere { center: DVec3, radius: f64 },
}

impl Projection {
    /// A planar projection along the faces' mean normal.
    pub fn planar_for(mesh: &Mesh, faces: &[FaceId]) -> Projection {
        let n: DVec3 = faces.iter().filter_map(|f| mesh.faces.get(*f).map(|x| x.normal * area(mesh, *f))).sum();
        Projection::Planar(if n.length() < 1e-9 { DVec3::Z } else { n.normalize() })
    }

    /// A cylinder round the world axis the faces' normals are most nearly across, through the
    /// centre of their bounds; the radius is the mean distance of the corners.
    pub fn cylinder_for(mesh: &Mesh, faces: &[FaceId]) -> Projection {
        let along = |a: DVec3| -> f64 { faces.iter().filter_map(|f| mesh.faces.get(*f).map(|x| x.normal.dot(a).abs() * area(mesh, *f))).sum() };
        let axis = [DVec3::X, DVec3::Y, DVec3::Z].into_iter().min_by(|a, b| along(*a).total_cmp(&along(*b))).unwrap_or(DVec3::Z);
        let (lo, hi) = bounds3(mesh, faces);
        let origin = (lo + hi) / 2.0;
        let pts: Vec<DVec3> = faces.iter().flat_map(|f| corners(mesh, *f)).collect();
        let radius = mean(pts.iter().map(|p| {
            let r = *p - origin;
            (r - axis * r.dot(axis)).length()
        }));
        Projection::Cylinder { origin, axis, radius: radius.max(1.0) }
    }

    /// A sphere round the centre of the faces' bounds.
    pub fn sphere_for(mesh: &Mesh, faces: &[FaceId]) -> Projection {
        let (lo, hi) = bounds3(mesh, faces);
        let center = (lo + hi) / 2.0;
        let radius = mean(faces.iter().flat_map(|f| corners(mesh, *f)).map(|p| (p - center).length()));
        Projection::Sphere { center, radius: radius.max(1.0) }
    }

    /// Texture coordinates (mm) of the corners `pts` of one face with normal `n`.
    fn coords(&self, pts: &[DVec3], n: DVec3) -> Vec<DVec2> {
        match *self {
            Projection::Planar(d) => {
                let (u, v) = texture_axes(d);
                pts.iter().map(|p| DVec2::new(p.dot(u), p.dot(v))).collect()
            }
            Projection::Box => {
                let a = n.abs();
                let axis = if a.z >= a.x && a.z >= a.y {
                    DVec3::Z * n.z.signum()
                } else if a.x >= a.y {
                    DVec3::X * n.x.signum()
                } else {
                    DVec3::Y * n.y.signum()
                };
                let (u, v) = texture_axes(axis);
                pts.iter().map(|p| DVec2::new(p.dot(u), p.dot(v))).collect()
            }
            Projection::Cylinder { origin, axis, radius } => {
                // the ends (faces across the axis) are projected along it, like caps
                if n.dot(axis).abs() > 0.7 {
                    return Projection::Planar(axis * n.dot(axis).signum()).coords(pts, n);
                }
                let (e1, e2) = across(axis);
                let raw: Vec<(f64, f64)> = pts
                    .iter()
                    .map(|p| {
                        let r = *p - origin;
                        (r.dot(e2).atan2(r.dot(e1)), r.dot(axis))
                    })
                    .collect();
                unwrap_angles(raw).into_iter().map(|(a, h)| DVec2::new(a * radius, h)).collect()
            }
            Projection::Sphere { center, radius } => {
                let raw: Vec<(f64, f64)> = pts
                    .iter()
                    .map(|p| {
                        let r = (*p - center).normalize_or_zero();
                        (r.y.atan2(r.x), r.z.clamp(-1.0, 1.0).asin())
                    })
                    .collect();
                unwrap_angles(raw).into_iter().map(|(a, b)| DVec2::new(a * radius, b * radius)).collect()
            }
        }
    }
}

fn mean(it: impl Iterator<Item = f64>) -> f64 {
    let (s, n) = it.fold((0.0, 0usize), |(s, n), x| (s + x, n + 1));
    if n == 0 { 0.0 } else { s / n as f64 }
}

/// Two unit vectors across `axis` (right-handed with it).
fn across(axis: DVec3) -> (DVec3, DVec3) {
    let a = axis.normalize_or_zero();
    let e1 = if a.z.abs() > 0.9 { DVec3::X } else { DVec3::Z.cross(a).normalize_or_zero() };
    let e1 = (e1 - a * e1.dot(a)).normalize_or_zero();
    (e1, a.cross(e1))
}

/// Angles of one face's corners brought within half a turn of the first (a face across the
/// wrap-around keeps one piece).
fn unwrap_angles(mut v: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    use std::f64::consts::{PI, TAU};
    if let Some(&(a0, _)) = v.first() {
        for (a, _) in v.iter_mut() {
            while *a - a0 > PI {
                *a -= TAU;
            }
            while *a - a0 < -PI {
                *a += TAU;
            }
        }
    }
    v
}

/// World bounds of the faces' corners.
fn bounds3(mesh: &Mesh, faces: &[FaceId]) -> (DVec3, DVec3) {
    let pts: Vec<DVec3> = faces.iter().flat_map(|f| corners(mesh, *f)).collect();
    let lo = pts.iter().copied().reduce(DVec3::min).unwrap_or_default();
    let hi = pts.iter().copied().reduce(DVec3::max).unwrap_or_default();
    (lo, hi)
}

/// The placement giving face `f`'s corners the texture coordinates `want`.
fn fit_face(mesh: &Mesh, f: FaceId, want: &[DVec2]) -> Option<Option<TexPlacement>> {
    let face = mesh.faces.get(f)?;
    let p = TexPlacement::fit(&corners(mesh, f), want, face.normal, [1.0, 1.0])?;
    Some((!(p.is_identity() || p.approx_eq(&TexPlacement::default()))).then_some(p))
}

/// Placements projecting the texture onto `faces` (faces the projection can't cover, e.g.
/// edge-on to a planar one, keep theirs).
pub fn project(mesh: &Mesh, faces: &[FaceId], proj: Projection) -> Placements {
    faces
        .iter()
        .filter_map(|f| {
            let n = mesh.faces.get(*f)?.normal;
            let want = proj.coords(&corners(mesh, *f), n);
            fit_face(mesh, *f, &want).map(|p| (*f, p))
        })
        .collect()
}

/// Placements moving the texture coordinates of `faces` through `map` (an affine map of
/// the texture plane: move, rotate, scale, flip).
pub fn map_faces(mesh: &Mesh, faces: &[FaceId], map: impl Fn(DVec2) -> DVec2) -> Placements {
    faces
        .iter()
        .filter_map(|f| {
            let want: Vec<DVec2> = face_tex(mesh, *f).into_iter().map(&map).collect();
            fit_face(mesh, *f, &want).map(|p| (*f, p))
        })
        .collect()
}

/// Texture-plane bounds of `faces`.
pub fn bounds(mesh: &Mesh, faces: &[FaceId]) -> Option<(DVec2, DVec2)> {
    let q: Vec<DVec2> = faces.iter().flat_map(|f| face_tex(mesh, *f)).collect();
    Some((q.iter().copied().reduce(DVec2::min)?, q.iter().copied().reduce(DVec2::max)?))
}

/// Rotation of the texture plane by `deg` about `c`.
pub fn rotation(c: DVec2, deg: f64) -> impl Fn(DVec2) -> DVec2 {
    let (s, k) = deg.to_radians().sin_cos();
    move |q| {
        let d = q - c;
        c + DVec2::new(d.x * k - d.y * s, d.x * s + d.y * k)
    }
}

/// Island transforms about the island's centre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IslandOp {
    Rotate(f64),
    FlipU,
    FlipV,
    Scale(f64),
    Move(DVec2),
}

/// Placements after `op` on the faces (one island or several, about their common centre).
pub fn transform(mesh: &Mesh, faces: &[FaceId], op: IslandOp) -> Placements {
    let Some((lo, hi)) = bounds(mesh, faces) else { return Vec::new() };
    let c = (lo + hi) / 2.0;
    match op {
        IslandOp::Rotate(deg) => map_faces(mesh, faces, rotation(c, deg)),
        IslandOp::FlipU => map_faces(mesh, faces, |q| DVec2::new(2.0 * c.x - q.x, q.y)),
        IslandOp::FlipV => map_faces(mesh, faces, |q| DVec2::new(q.x, 2.0 * c.y - q.y)),
        IslandOp::Scale(k) => map_faces(mesh, faces, |q| c + (q - c) * k),
        IslandOp::Move(d) => map_faces(mesh, faces, |q| q + d),
    }
}

/// What [`randomize`] varies besides the shift.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Jitter {
    /// Any angle (else a multiple of 90°).
    pub any_angle: bool,
    /// Uniform scale picked in `[min, max]` (`[1, 1]`: kept).
    pub scale: [f64; 2],
}

impl Default for Jitter {
    fn default() -> Self {
        Self { any_angle: false, scale: [1.0, 1.0] }
    }
}

/// The island turned by a random angle (a multiple of 90°, or any), scaled by a random
/// factor in `jitter.scale` and moved by a random part of the tile (`seed` picks the
/// variant; the same seed gives the same result), so repeated textures on neighbouring
/// islands don't line up.
pub fn randomize(mesh: &Mesh, faces: &[FaceId], tile: [f64; 2], seed: u64, jitter: Jitter) -> Placements {
    let Some((lo, hi)) = bounds(mesh, faces) else { return Vec::new() };
    let mut s = seed;
    let mut next = move || {
        // splitmix64
        s = s.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = s;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    };
    let unit = |v: u64| (v >> 11) as f64 / (1u64 << 53) as f64;
    let turn_pick = next();
    let deg = if jitter.any_angle { unit(turn_pick) * 360.0 } else { (turn_pick % 4) as f64 * 90.0 };
    let d = DVec2::new(unit(next()) * tile[0], unit(next()) * tile[1]);
    let [a, b] = jitter.scale;
    let k = (a + (b - a) * unit(next())).max(1e-3);
    let c = (lo + hi) / 2.0;
    let turn = rotation(c, deg);
    map_faces(mesh, faces, |q| c + (turn(q) - c) * k + d)
}

/// The island scaled uniformly and moved so it fills one tile (`tile` mm), from its origin.
pub fn fit_to_tile(mesh: &Mesh, faces: &[FaceId], tile: [f64; 2]) -> Placements {
    let Some((lo, hi)) = bounds(mesh, faces) else { return Vec::new() };
    let size = hi - lo;
    if size.x < 1e-9 || size.y < 1e-9 {
        return Vec::new();
    }
    let k = (tile[0] / size.x).min(tile[1] / size.y);
    map_faces(mesh, faces, |q| (q - lo) * k)
}

/// The island turned (by less than 45°) so its longest edge in the texture runs straight
/// across or up.
pub fn align(mesh: &Mesh, faces: &[FaceId]) -> Placements {
    let mut best: Option<(f64, DVec2)> = None;
    for f in faces {
        let q = face_tex(mesh, *f);
        for i in 0..q.len() {
            let d = q[(i + 1) % q.len()] - q[i];
            if best.is_none_or(|b| d.length() > b.0) {
                best = Some((d.length(), d));
            }
        }
    }
    let Some((_, d)) = best else { return Vec::new() };
    let a = d.y.atan2(d.x).to_degrees();
    let turn = -(a - (a / 90.0).round() * 90.0);
    if turn.abs() < 1e-9 {
        return Vec::new();
    }
    transform(mesh, faces, IslandOp::Rotate(turn))
}

/// Texture millimetres per world millimetre over `faces` (from their areas).
pub fn tex_per_world(mesh: &Mesh, faces: &[FaceId]) -> f64 {
    let (mut t, mut w) = (0.0, 0.0);
    for f in faces {
        t += area2(&face_tex(mesh, *f));
        w += area(mesh, *f);
    }
    if w < 1e-12 { 1.0 } else { (t / w).sqrt() }
}

/// Islands scaled by one common factor and laid out in rows inside one tile, `pad`
/// (a fraction of the tile width) apart; the scale is the largest that fits.
pub fn pack(mesh: &Mesh, islands: &[Vec<FaceId>], tile: [f64; 2], pad: f64) -> Placements {
    let boxes: Vec<(DVec2, DVec2)> = islands.iter().filter_map(|i| bounds(mesh, i)).collect();
    if boxes.len() != islands.len() || boxes.is_empty() {
        return Vec::new();
    }
    let gap = pad * tile[0];
    // islands tallest first, rows left to right
    let mut order: Vec<usize> = (0..boxes.len()).collect();
    order.sort_by(|a, b| (boxes[*b].1.y - boxes[*b].0.y).total_cmp(&(boxes[*a].1.y - boxes[*a].0.y)));
    let layout = |k: f64| -> Option<Vec<DVec2>> {
        let mut at = vec![DVec2::ZERO; boxes.len()];
        let (mut x, mut y, mut row_h) = (gap, gap, 0.0f64);
        for &i in &order {
            let s = (boxes[i].1 - boxes[i].0) * k;
            if x + s.x + gap > tile[0] && x > gap {
                y += row_h + gap;
                x = gap;
                row_h = 0.0;
            }
            if x + s.x + gap > tile[0] + 1e-9 {
                return None;
            }
            at[i] = DVec2::new(x, y);
            x += s.x + gap;
            row_h = row_h.max(s.y);
        }
        (y + row_h + gap <= tile[1] + 1e-9).then_some(at)
    };
    let max_side = boxes.iter().map(|(lo, hi)| (hi.x - lo.x).max(hi.y - lo.y)).fold(0.0, f64::max);
    if max_side < 1e-9 {
        return Vec::new();
    }
    let (mut lo_k, mut hi_k) = (0.0, tile[0].max(tile[1]) / max_side);
    for _ in 0..40 {
        let mid = (lo_k + hi_k) / 2.0;
        if layout(mid).is_some() {
            lo_k = mid;
        } else {
            hi_k = mid;
        }
    }
    let Some(at) = layout(lo_k) else { return Vec::new() };
    let mut out = Vec::new();
    for (i, island) in islands.iter().enumerate() {
        let (lo, slot) = (boxes[i].0, at[i]);
        out.extend(map_faces(mesh, island, |q| slot + (q - lo) * lo_k));
    }
    out
}

/// Shared edges between two of `faces` that are not soft or smooth: where a box-like object
/// folds (the usual seams for unfolding it).
pub fn hard_edges(mesh: &Mesh, faces: &[FaceId]) -> Vec<EdgeId> {
    let set: HashSet<FaceId> = faces.iter().copied().collect();
    edges_of(mesh, faces)
        .into_iter()
        .filter(|e| {
            let ed = &mesh.edges[*e];
            let fs = mesh.edge_faces(*e);
            !ed.soft && !ed.smooth && fs.len() == 2 && fs.iter().all(|f| set.contains(f)) && {
                let (a, b) = (mesh.faces[fs[0]].normal, mesh.faces[fs[1]].normal);
                a.dot(b) < 1.0 - 1e-6
            }
        })
        .collect()
}
