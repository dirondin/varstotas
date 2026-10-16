//! Section planes (the classic Section Plane tool). Model-level only, world coordinates.
//!
//! A plane keeps the half-space its normal points to: points with `n · (p - origin) >= 0` stay
//! visible, the rest is cut away (drawing, picking). At most one plane is active; View →
//! Section Cuts can switch cutting off without deactivating it ([`Model::active_section`]).

use crate::{Aabb, Model};
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SectionPlane {
    pub name: String,
    pub origin: DVec3,
    /// Unit normal, pointing to the side that stays visible.
    pub normal: DVec3,
    #[serde(default)]
    pub active: bool,
}

/// View → Section Planes / Section Cuts (stored with the model).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SectionDisplay {
    /// Draw the section planes themselves.
    #[serde(default = "yes")]
    pub planes: bool,
    /// Cut the model with the active plane.
    #[serde(default = "yes")]
    pub cuts: bool,
    /// Fill the cut solids.
    #[serde(default = "yes")]
    pub fill: bool,
}

fn yes() -> bool {
    true
}

impl Default for SectionDisplay {
    fn default() -> Self {
        Self { planes: true, cuts: true, fill: true }
    }
}

/// Section state saved in a scene.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneSections {
    /// Index of the active plane (`None` = no active cut).
    pub active: Option<usize>,
    pub display: SectionDisplay,
}

impl SectionPlane {
    pub fn new(name: impl Into<String>, origin: DVec3, normal: DVec3) -> Self {
        // `+ 0.0` turns -0.0 components into 0.0 (stable output).
        Self { name: name.into(), origin, normal: normal.normalize_or(DVec3::Z) + DVec3::ZERO, active: true }
    }

    /// `[n.x, n.y, n.z, d]` with `n · p >= d` on the visible side.
    pub fn plane(&self) -> [f64; 4] {
        let n = self.normal;
        [n.x, n.y, n.z, n.dot(self.origin)]
    }

    /// Signed distance (`< 0` = cut away).
    pub fn dist(&self, p: DVec3) -> f64 {
        self.normal.dot(p - self.origin)
    }

    /// In-plane unit axes `(u, v)` (horizontal `u` where possible).
    pub fn axes(&self) -> (DVec3, DVec3) {
        let n = self.normal;
        let u = if n.z.abs() > 0.999 { DVec3::X } else { DVec3::Z.cross(n).normalize() };
        (u, n.cross(u))
    }

    /// Corners of the rectangle drawn for the plane: the model bounds projected onto it, grown
    /// by 10% (at least 1 m square around the origin for an empty model).
    pub fn rect(&self, bounds: Option<Aabb>) -> [DVec3; 4] {
        let (u, v) = self.axes();
        let o = self.origin;
        let (mut lo, mut hi) = (glam::DVec2::splat(-500.0), glam::DVec2::splat(500.0));
        if let Some(b) = bounds {
            let pts = b.corners().map(|p| glam::DVec2::new((p - o).dot(u), (p - o).dot(v)));
            lo = pts.iter().fold(glam::DVec2::MAX, |a, p| a.min(*p));
            hi = pts.iter().fold(glam::DVec2::MIN, |a, p| a.max(*p));
            let pad = ((hi - lo) * 0.1).max(glam::DVec2::splat(100.0));
            lo -= pad;
            hi += pad;
        }
        let at = |a: f64, b: f64| o + u * a + v * b;
        [at(lo.x, lo.y), at(hi.x, lo.y), at(hi.x, hi.y), at(lo.x, hi.y)]
    }

    /// The same plane facing the other way.
    pub fn reversed(&self) -> SectionPlane {
        SectionPlane { normal: -self.normal + DVec3::ZERO, ..self.clone() }
    }

    /// Plane moved by `d` along its normal.
    pub fn offset(&self, d: f64) -> SectionPlane {
        SectionPlane { origin: self.origin + self.normal * d, ..self.clone() }
    }

    /// Plane after an affine transform (normal re-normalized).
    pub fn transformed(&self, m: &DMat4) -> SectionPlane {
        let n = DMat4::from_mat3(glam::DMat3::from_mat4(*m).inverse().transpose()).transform_vector3(self.normal);
        SectionPlane { origin: m.transform_point3(self.origin), normal: n.normalize_or(self.normal), ..self.clone() }
    }
}

impl Model {
    /// The plane that currently cuts the model (active and cuts enabled).
    pub fn active_section(&self) -> Option<&SectionPlane> {
        if !self.section_display.cuts {
            return None;
        }
        self.sections.iter().find(|s| s.active)
    }

    /// Section state for a scene.
    pub fn scene_sections(&self) -> SceneSections {
        SceneSections { active: self.sections.iter().position(|s| s.active), display: self.section_display }
    }

    /// Restore a scene's section state (an index past the end means no active plane).
    pub fn apply_scene_sections(&mut self, s: &SceneSections) {
        self.set_active_section(s.active);
        self.section_display = s.display;
    }

    /// Make plane `i` the only active one (`None` = no active plane).
    pub fn set_active_section(&mut self, i: Option<usize>) {
        for (k, s) in self.sections.iter_mut().enumerate() {
            s.active = Some(k) == i;
        }
    }

    /// Section cut lines: where `plane` crosses the visible faces of the model (world space).
    pub fn section_cut(&self, plane: &SectionPlane) -> Vec<[DVec3; 2]> {
        let mut out = Vec::new();
        self.cut_rec(self.root, DMat4::IDENTITY, plane, &mut out, 0);
        out
    }

    fn cut_rec(&self, d: crate::DefId, m: DMat4, plane: &SectionPlane, out: &mut Vec<[DVec3; 2]>, depth: usize) {
        if depth > 64 {
            return;
        }
        let Some(def) = self.defs.get(d) else { return };
        if let Some(b) = def.mesh.bounds() {
            let ds = b.transformed(&m).corners().map(|p| plane.dist(p));
            if ds.iter().all(|d| *d > 0.0) || ds.iter().all(|d| *d < 0.0) {
                return;
            }
        }
        let mesh = &def.mesh;
        for f in mesh.faces.values() {
            if f.hidden || !self.tag_visible(d, f.tag) {
                continue;
            }
            // Crossing points of all loops, as parameters along the cut line.
            let line_dir = plane.normal.cross(m.transform_vector3(f.normal)).normalize_or_zero();
            if line_dir == DVec3::ZERO {
                continue; // face parallel to the plane
            }
            let mut ts: Vec<(f64, DVec3)> = Vec::new();
            for lp in std::iter::once(&f.outer).chain(f.holes.iter()) {
                for i in 0..lp.len() {
                    let (Some(a), Some(b)) = (mesh.vertices.get(lp[i]), mesh.vertices.get(lp[(i + 1) % lp.len()])) else {
                        continue;
                    };
                    let (pa, pb) = (m.transform_point3(a.pos), m.transform_point3(b.pos));
                    let (da, db) = (plane.dist(pa), plane.dist(pb));
                    // Half-open rule (>= 0 is "positive") so a vertex on the plane counts once.
                    if (da >= 0.0) != (db >= 0.0) {
                        let p = pa + (pb - pa) * (da / (da - db));
                        ts.push((p.dot(line_dir), p));
                    }
                }
            }
            ts.sort_by(|a, b| a.0.total_cmp(&b.0));
            for pair in ts.as_chunks::<2>().0 {
                if (pair[1].1 - pair[0].1).length() > crate::TOL {
                    out.push([pair[0].1, pair[1].1]);
                }
            }
        }
        for inst in mesh.instances.values() {
            if inst.hidden || !self.tag_visible(d, inst.tag) {
                continue;
            }
            self.cut_rec(inst.def, m * inst.transform, plane, out, depth + 1);
        }
    }
}

/// Section fill: closed loops of the cut lines (`section_cut`), triangulated in the plane.
/// Loops nested an odd number of times are holes. Open chains (non-solid geometry) are skipped.
pub fn section_fill(plane: &SectionPlane, segs: &[[DVec3; 2]]) -> Vec<[DVec3; 3]> {
    use glam::DVec2;
    use std::collections::HashMap;
    let (u, v) = plane.axes();
    let to2 = |p: DVec3| DVec2::new((p - plane.origin).dot(u), (p - plane.origin).dot(v));
    // Weld endpoints (0.01 mm grid) and walk the segment graph into loops.
    let key = |p: DVec2| ((p.x * 100.0).round() as i64, (p.y * 100.0).round() as i64);
    let mut pts: Vec<DVec2> = Vec::new();
    let mut ids: HashMap<(i64, i64), usize> = HashMap::new();
    let mut id_of = |p: DVec2, pts: &mut Vec<DVec2>| {
        *ids.entry(key(p)).or_insert_with(|| {
            pts.push(p);
            pts.len() - 1
        })
    };
    let mut adj: Vec<Vec<(usize, usize)>> = Vec::new(); // vertex -> (segment, other vertex)
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for [a, b] in segs {
        let (ia, ib) = (id_of(to2(*a), &mut pts), id_of(to2(*b), &mut pts));
        if ia == ib {
            continue;
        }
        adj.resize(pts.len(), Vec::new());
        adj[ia].push((edges.len(), ib));
        adj[ib].push((edges.len(), ia));
        edges.push((ia, ib));
    }
    let mut used = vec![false; edges.len()];
    let mut loops: Vec<Vec<DVec2>> = Vec::new();
    for e0 in 0..edges.len() {
        if used[e0] {
            continue;
        }
        used[e0] = true;
        let (start, mut cur) = edges[e0];
        let mut lp = vec![pts[start]];
        let mut closed = false;
        for _ in 0..edges.len() {
            if cur == start {
                closed = true;
                break;
            }
            lp.push(pts[cur]);
            let Some(&(e, next)) = adj[cur].iter().find(|(e, _)| !used[*e]) else { break };
            used[e] = true;
            cur = next;
        }
        if closed && lp.len() >= 3 {
            loops.push(lp);
        }
    }
    // Nesting depth of each loop (by a point of it inside the others).
    let inside = |p: DVec2, poly: &[DVec2]| {
        let mut c = false;
        for i in 0..poly.len() {
            let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
            if (a.y > p.y) != (b.y > p.y) && p.x < a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x) {
                c = !c;
            }
        }
        c
    };
    let probe = |lp: &[DVec2]| (lp[0] + lp[1]) * 0.5;
    let depth: Vec<usize> = (0..loops.len())
        .map(|i| (0..loops.len()).filter(|&j| j != i && inside(probe(&loops[i]), &loops[j])).count())
        .collect();
    let to3 = |p: DVec2| plane.origin + u * p.x + v * p.y;
    let mut out = Vec::new();
    for (i, outer) in loops.iter().enumerate() {
        if depth[i] % 2 == 1 {
            continue;
        }
        // Holes: loops one level deeper whose smallest even container is this loop.
        let holes: Vec<&[DVec2]> = (0..loops.len())
            .filter(|&j| depth[j] == depth[i] + 1 && inside(probe(&loops[j]), outer))
            .map(|j| loops[j].as_slice())
            .collect();
        let all: Vec<DVec2> = outer.iter().chain(holes.iter().flat_map(|h| h.iter())).copied().collect();
        for t in crate::triangulate::triangulate(outer, &holes) {
            let [a, b, c] = t.map(|k| to3(all[k]));
            out.push([a, b, c]);
        }
    }
    out
}
