//! Inference candidate geometry: world-space points/edges/guides collected from the model
//! (cached per model version) and their screen projections (cached per camera).

use super::{InferenceKind, InferenceRef};
use varstotas_kernel::glam::{DMat4, DVec2, DVec3};
use varstotas_kernel::{DefId, EntityRef, InstanceId, Model};
use varstotas_render::Camera;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct PointCand {
    pub pos: DVec3,
    pub kind: InferenceKind,
    pub r: Option<InferenceRef>,
}

#[derive(Clone, Debug)]
pub struct EdgeCand {
    pub a: DVec3,
    pub b: DVec3,
    pub r: Option<InferenceRef>,
}

/// World-space candidates (rebuilt when the model changes).
#[derive(Default, Debug)]
pub struct GeoCache {
    pub version: u64,
    pub points: Vec<PointCand>,
    pub edges: Vec<EdgeCand>,
    /// Infinite guide lines (origin, unit dir).
    pub guides: Vec<(DVec3, DVec3)>,
}

const MAX_DEPTH: usize = 32;
/// Safety cap for huge models (inference stays interactive).
const MAX_EDGES: usize = 200_000;

fn tag_visible(model: &Model, t: Option<varstotas_kernel::TagId>) -> bool {
    t.and_then(|t| model.tags.get(t)).is_none_or(|t| t.visible)
}

/// Circumcenter of three points (None if collinear).
pub fn circumcenter(a: DVec3, b: DVec3, c: DVec3) -> Option<DVec3> {
    let ab = b - a;
    let ac = c - a;
    let n = ab.cross(ac);
    let n2 = n.length_squared();
    if n2 < 1e-18 * ab.length_squared().max(1e-30) * ac.length_squared().max(1e-30) || n2 < 1e-24 {
        return None;
    }
    let p = (n.cross(ab) * ac.length_squared() + ac.cross(n) * ab.length_squared()) / (2.0 * n2);
    Some(a + p)
}

impl GeoCache {
    pub fn build(model: &Model, version: u64) -> Self {
        let mut g = GeoCache { version, ..Default::default() };
        let mut path = Vec::new();
        g.collect(model, model.root, DMat4::IDENTITY, &mut path, 0);
        for gl in &model.guides {
            let d = gl.dir.normalize_or_zero();
            if d != DVec3::ZERO && gl.origin.is_finite() {
                g.guides.push((gl.origin, d));
            }
        }
        for p in &model.guide_points {
            g.points.push(PointCand { pos: *p, kind: InferenceKind::GuidePoint, r: None });
        }
        // The model origin is always a snap target (the classic "Origin" inference).
        g.points.push(PointCand { pos: DVec3::ZERO, kind: InferenceKind::Origin, r: None });
        g
    }

    fn collect(&mut self, model: &Model, d: DefId, m: DMat4, path: &mut Vec<InstanceId>, depth: usize) {
        if depth > MAX_DEPTH || self.edges.len() > MAX_EDGES {
            return;
        }
        let Some(def) = model.defs.get(d) else { return };
        let mesh = &def.mesh;
        let mut seen_v = HashSet::new();
        let mut curves: HashMap<u32, Vec<DVec3>> = HashMap::new();
        for (eid, e) in mesh.edges.iter() {
            if e.hidden || !tag_visible(model, e.tag) {
                continue;
            }
            let (Some(va), Some(vb)) = (mesh.vertices.get(e.v[0]), mesh.vertices.get(e.v[1])) else { continue };
            let a = m.transform_point3(va.pos);
            let b = m.transform_point3(vb.pos);
            self.edges.push(EdgeCand {
                a,
                b,
                r: Some(InferenceRef { path: path.clone(), entity: EntityRef::Edge(eid) }),
            });
            for (vid, p) in [(e.v[0], a), (e.v[1], b)] {
                if seen_v.insert(vid) {
                    self.points.push(PointCand {
                        pos: p,
                        kind: InferenceKind::Endpoint,
                        r: Some(InferenceRef { path: path.clone(), entity: EntityRef::Vertex(vid) }),
                    });
                }
            }
            match e.curve {
                None => self.points.push(PointCand {
                    pos: (a + b) * 0.5,
                    kind: InferenceKind::Midpoint,
                    r: Some(InferenceRef { path: path.clone(), entity: EntityRef::Edge(eid) }),
                }),
                Some(c) => {
                    let l = curves.entry(c).or_default();
                    for p in [a, b] {
                        if !l.iter().any(|q| q.distance_squared(p) < 1e-8) {
                            l.push(p);
                        }
                    }
                }
            }
        }
        for pts in curves.values() {
            if pts.len() < 3 {
                continue;
            }
            let n = pts.len();
            if let Some(c) = circumcenter(pts[0], pts[n / 3], pts[(2 * n) / 3]) {
                let r = c.distance(pts[0]);
                if r > 0.0 && pts.iter().all(|p| (p.distance(c) - r).abs() <= r * 0.01 + 1e-6) {
                    self.points.push(PointCand { pos: c, kind: InferenceKind::Center, r: None });
                }
            }
        }
        for (iid, inst) in mesh.instances.iter() {
            if inst.hidden || !tag_visible(model, inst.tag) {
                continue;
            }
            path.push(iid);
            self.collect(model, inst.def, m * inst.transform, path, depth + 1);
            path.pop();
        }
    }
}

/// Key identifying a camera + viewport (projection cache invalidation).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CamKey {
    eye: DVec3,
    target: DVec3,
    up: DVec3,
    fov: f64,
    ortho: bool,
    ortho_h: f64,
    vp: DVec2,
}

impl CamKey {
    pub fn of(c: &Camera, vp: DVec2) -> Self {
        Self { eye: c.eye, target: c.target, up: c.up, fov: c.fov_y_deg, ortho: c.ortho, ortho_h: c.ortho_height, vp }
    }
}

/// Screen projections of the cached geometry.
#[derive(Default)]
pub struct ScreenCache {
    pub key: Option<(u64, CamKey)>,
    pub points: Vec<Option<DVec2>>,
    pub edges: Vec<Option<(DVec2, DVec2)>>,
}

/// Clip a segment to the part in front of the camera and project it.
pub fn project_segment(cam: &Camera, vp: DVec2, a: DVec3, b: DVec3) -> Option<(DVec2, DVec2)> {
    if cam.ortho {
        return Some((cam.world_to_screen(a, vp)?, cam.world_to_screen(b, vp)?));
    }
    let f = cam.basis().forward;
    let near = cam.near_plane().max(1e-6);
    let da = (a - cam.eye).dot(f);
    let db = (b - cam.eye).dot(f);
    if da < near && db < near {
        return None;
    }
    let (mut a, mut b) = (a, b);
    if da < near {
        a = a.lerp(b, (near - da) / (db - da));
    } else if db < near {
        b = b.lerp(a, (near - db) / (da - db));
    }
    Some((cam.world_to_screen(a, vp)?, cam.world_to_screen(b, vp)?))
}

impl ScreenCache {
    pub fn update(&mut self, geo: &GeoCache, cam: &Camera, vp: DVec2) {
        let key = (geo.version, CamKey::of(cam, vp));
        if self.key == Some(key) {
            return;
        }
        self.key = Some(key);
        self.points = geo.points.iter().map(|p| cam.world_to_screen(p.pos, vp)).collect();
        self.edges = geo.edges.iter().map(|e| project_segment(cam, vp, e.a, e.b)).collect();
    }
}

/// Distance from `p` to segment `ab` (screen) and the parameter of the closest point.
pub fn dist_point_seg2(p: DVec2, a: DVec2, b: DVec2) -> (f64, f64) {
    let ab = b - a;
    let l2 = ab.length_squared();
    let t = if l2 > 1e-12 { ((p - a).dot(ab) / l2).clamp(0.0, 1.0) } else { 0.0 };
    ((a + ab * t - p).length(), t)
}

/// Closest point on the infinite line (o, d) to the ray (ro, rd). Returns None when parallel.
pub fn closest_on_line_to_ray(o: DVec3, d: DVec3, ro: DVec3, rd: DVec3) -> Option<DVec3> {
    let w = o - ro;
    let a = d.dot(d);
    let b = d.dot(rd);
    let c = rd.dot(rd);
    let dd = d.dot(w);
    let e = rd.dot(w);
    let den = a * c - b * b;
    if den.abs() < 1e-12 * a * c {
        return None;
    }
    let s = (b * e - c * dd) / den;
    let p = o + d * s;
    p.is_finite().then_some(p)
}

/// Closest point on segment ab to the ray.
pub fn closest_on_seg_to_ray(a: DVec3, b: DVec3, ro: DVec3, rd: DVec3) -> DVec3 {
    let d = b - a;
    if d.length_squared() < 1e-18 {
        return a;
    }
    let w = a - ro;
    let (aa, bb, cc) = (d.dot(d), d.dot(rd), rd.dot(rd));
    let (dd, ee) = (d.dot(w), rd.dot(w));
    let den = aa * cc - bb * bb;
    let s = if den.abs() < 1e-12 * aa * cc { 0.0 } else { ((bb * ee - cc * dd) / den).clamp(0.0, 1.0) };
    a + d * s
}
