//! Accelerated intersection and sampled normal-ray thickness analysis, in world mm.
use super::{Bvh, PrintSelection, tolerance};
use crate::{
    Aabb, Model,
    glam::{DVec2, DVec3},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AnalysisOptions {
    pub minimum_wall_mm: f64,
    pub max_samples: usize,
    pub max_pairs: usize,
    pub overhang_degrees: f64,
}
impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            minimum_wall_mm: 0.8,
            max_samples: 20_000,
            max_pairs: 2_000_000,
            overhang_degrees: 45.,
        }
    }
}
#[derive(Clone, Debug)]
pub struct HeatTriangle {
    pub points: [DVec3; 3],
    pub value: Option<f64>,
}
#[derive(Clone, Debug, Default)]
pub struct AdvancedReport {
    pub intersection_pairs: usize,
    pub candidate_pairs: usize,
    pub intersections_complete: bool,
    pub intersections: Vec<[DVec3; 2]>,
    pub triangles: usize,
    pub sampled_triangles: usize,
    pub samples: usize,
    pub possible_samples: usize,
    pub unknown_samples: usize,
    pub thin_samples: usize,
    pub minimum_mm: Option<f64>,
    pub thickness: Vec<HeatTriangle>,
    pub overhang_area_mm2: f64,
    pub overhangs: Vec<HeatTriangle>,
    pub reliable_surface: bool,
}
impl Model {
    pub fn print_analyze(
        &self,
        selection: Option<&PrintSelection>,
        tol: f64,
        options: &AnalysisOptions,
    ) -> AdvancedReport {
        let tol = tolerance(tol);
        let mut triangles = Vec::new();
        for f in crate::tessellate(self).faces {
            if selection.is_some_and(|s| {
                f.face
                    .is_none_or(|id| !s.includes(&f.owner_path, crate::EntityRef::Face(id)))
            }) {
                continue;
            }
            for p in f.positions.as_chunks::<3>().0 {
                if (p[1] - p[0]).cross(p[2] - p[0]).length() > tol * tol {
                    triangles.push(*p);
                }
            }
        }
        let mut out = AdvancedReport {
            triangles: triangles.len(),
            possible_samples: triangles.len().saturating_mul(4),
            intersections_complete: true,
            ..Default::default()
        };
        if triangles.is_empty() {
            return out;
        }
        let bvh = Bvh::new(&triangles);
        let bounds: Vec<_> = triangles
            .iter()
            .map(|p| Aabb::from_points(*p).unwrap())
            .collect();
        let pair_limit = options.max_pairs.clamp(1, 10_000_000);
        'all: for i in 0..triangles.len() {
            let mut stack = vec![0];
            while let Some(n) = stack.pop() {
                let node = &bvh.nodes[n];
                if !overlaps(bounds[i], node.bounds, tol) {
                    continue;
                }
                if let Some((a, b)) = node.children {
                    stack.extend([a, b]);
                    continue;
                }
                for &j in &bvh.order[node.range.clone()] {
                    if j <= i || !overlaps(bounds[i], bounds[j], tol) {
                        continue;
                    }
                    if out.candidate_pairs == pair_limit {
                        out.intersections_complete = false;
                        break 'all;
                    }
                    out.candidate_pairs += 1;
                    if let Some(segment) = triangle_intersection(triangles[i], triangles[j], tol) {
                        out.intersection_pairs += 1;
                        out.intersections.push(segment);
                    }
                }
            }
        }
        let normal = |p: [DVec3; 3]| (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        let weights = [
            [1. / 3.; 3],
            [0.8, 0.1, 0.1],
            [0.1, 0.8, 0.1],
            [0.1, 0.1, 0.8],
        ];
        let samples = out
            .possible_samples
            .min(options.max_samples.clamp(1, 500_000));
        let mut minimums = vec![None::<f64>; triangles.len()];
        let mut measured = HashSet::new();
        let epsilon = tol.max(1e-7);
        for sample in 0..samples {
            let index = sample * out.possible_samples / samples;
            let i = index / 4;
            let w = weights[index % 4];
            let p = triangles[i];
            let n = normal(p);
            let center = p[0] * w[0] + p[1] * w[1] + p[2] * w[2];
            out.samples += 1;
            measured.insert(i);
            // Start on the surface, not a fixed 0.5-mm offset which skips thin walls. Ignore only
            // numerical self hits; the first actual surface must be an exit, not a later distant part.
            let mut hits = bvh.hits(&triangles, center, -n);
            hits.retain(|(t, j)| *j != i && *t > epsilon);
            let hit = hits.into_iter().min_by(|a, b| a.0.total_cmp(&b.0));
            if let Some((distance, j)) = hit
                && normal(triangles[j]).dot(-n) > 1e-6
            {
                minimums[i] = Some(minimums[i].map_or(distance, |m| m.min(distance)));
                out.minimum_mm = Some(out.minimum_mm.map_or(distance, |m| m.min(distance)));
                if distance < options.minimum_wall_mm {
                    out.thin_samples += 1;
                }
            } else {
                out.unknown_samples += 1;
            }
        }
        out.sampled_triangles = measured.len();
        let mut measured: Vec<_> = measured.into_iter().collect();
        measured.sort_unstable();
        out.thickness = measured
            .into_iter()
            .map(|i| HeatTriangle {
                points: triangles[i],
                value: minimums[i],
            })
            .collect();
        let threshold = options.overhang_degrees.clamp(0., 89.9);
        for p in &triangles {
            let n = normal(*p);
            if n.z >= 0. || p.iter().all(|p| p.z.abs() <= tol) {
                continue;
            }
            let degrees = (-n.z).clamp(-1., 1.).acos().to_degrees();
            if degrees < threshold {
                out.overhang_area_mm2 += (p[1] - p[0]).cross(p[2] - p[0]).length() * 0.5;
                out.overhangs.push(HeatTriangle {
                    points: *p,
                    value: Some(degrees),
                });
            }
        }
        let basic = self.print_check(selection, tol);
        out.reliable_surface = !basic.has_surface_errors()
            && out.intersections_complete
            && out.intersection_pairs == 0
            && !basic.shells.is_empty()
            && basic.shells.iter().all(|s| s.closed && s.consistent);
        out
    }
}
fn overlaps(a: Aabb, b: Aabb, e: f64) -> bool {
    (a.min - DVec3::splat(e)).cmple(b.max).all() && (b.min - DVec3::splat(e)).cmple(a.max).all()
}
fn inside(p: DVec3, t: [DVec3; 3], eps: f64) -> bool {
    let (a, b, q) = (t[1] - t[0], t[2] - t[0], p - t[0]);
    let (aa, ab, bb, qa, qb) = (a.dot(a), a.dot(b), b.dot(b), q.dot(a), q.dot(b));
    let det = aa * bb - ab * ab;
    if det <= 1e-30 {
        return false;
    }
    let (u, v) = ((qa * bb - qb * ab) / det, (qb * aa - qa * ab) / det);
    let e = eps / a.length().min(b.length()).max(eps);
    u >= -e && v >= -e && u + v <= 1. + e
}
/// Positive-area coplanar overlap and non-boundary surface crossings. Legitimate shared
/// triangle edges and vertices are excluded, but a shared vertex does not mask another crossing.
fn triangle_intersection(a: [DVec3; 3], b: [DVec3; 3], e: f64) -> Option<[DVec3; 2]> {
    let na = (a[1] - a[0]).cross(a[2] - a[0]).normalize_or_zero();
    let nb = (b[1] - b[0]).cross(b[2] - b[0]).normalize_or_zero();
    let da = a.map(|p| (p - b[0]).dot(nb));
    let db = b.map(|p| (p - a[0]).dot(na));
    let separate = |d: [f64; 3]| d.iter().all(|d| *d > e) || d.iter().all(|d| *d < -e);
    if separate(da) || separate(db) {
        return None;
    }
    if na.cross(nb).length() < 1e-8 && da.iter().all(|d| d.abs() <= e) {
        let pr = crate::geom::Projector::new(a[0], na);
        let mut polygon: Vec<DVec2> = a.map(|p| pr.to2(p)).to_vec();
        let mut clip = b.map(|p| pr.to2(p));
        if crate::geom::signed_area(&clip) < 0. {
            clip.reverse();
        }
        for k in 0..3 {
            let (x, y) = (clip[k], clip[(k + 1) % 3]);
            let side = |p: DVec2| (y - x).perp_dot(p - x);
            let mut next = Vec::new();
            if polygon.is_empty() {
                break;
            }
            for i in 0..polygon.len() {
                let (p, q) = (polygon[i], polygon[(i + 1) % polygon.len()]);
                let (dp, dq) = (side(p), side(q));
                if dp >= 0. {
                    next.push(p);
                }
                if (dp >= 0.) != (dq >= 0.) {
                    next.push(p + (q - p) * (dp / (dp - dq)));
                }
            }
            polygon = next;
        }
        if polygon.len() >= 3 && crate::geom::signed_area(&polygon).abs() > e * e {
            return Some([pr.to3(polygon[0]), pr.to3(polygon[polygon.len() / 2])]);
        }
        return None;
    }
    let mut hits = Vec::new();
    for (t, d, other) in [(a, da, b), (b, db, a)] {
        for k in 0..3 {
            let l = (k + 1) % 3;
            if d[k].abs() <= e && inside(t[k], other, e) {
                hits.push(t[k]);
            }
            if d[k] * d[l] < 0. {
                let p = t[k] + (t[l] - t[k]) * (d[k] / (d[k] - d[l]));
                if inside(p, other, e) {
                    hits.push(p);
                }
            }
        }
    }
    if hits.is_empty() {
        return None;
    }
    let common: Vec<_> = a
        .iter()
        .filter(|p| b.iter().any(|q| p.distance(*q) <= e))
        .copied()
        .collect();
    let on_boundary = |p: DVec3| match common.as_slice() {
        [a] => p.distance(*a) <= e,
        [a, b, ..] => {
            let d = *b - *a;
            let u = ((p - *a).dot(d) / d.length_squared()).clamp(0., 1.);
            p.distance(*a + d * u) <= e
        }
        _ => false,
    };
    if hits.iter().all(|p| on_boundary(*p)) {
        return None;
    }
    let mut pair = [hits[0], hits[0]];
    let mut length = 0.;
    for &p in &hits {
        for &q in &hits {
            if p.distance_squared(q) > length {
                length = p.distance_squared(q);
                pair = [p, q];
            }
        }
    }
    Some(pair)
}
