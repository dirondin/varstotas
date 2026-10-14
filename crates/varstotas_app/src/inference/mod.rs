//! Inference (snapping) engine. Owner: `tools` agent.
//!
//! Collects candidates in screen space — endpoints, midpoints, curve centers, edge/edge
//! intersections, points on edges and guides, faces under the cursor, and direction
//! inferences relative to an anchor (red/green/blue axes, parallel/perpendicular to recently
//! hovered edges, "from point" alignment with remembered hover points) — and ranks them per
//! docs/05-UX-SPEC.md: Endpoint > Midpoint/Center > Intersection > On Edge > directions >
//! On Face > free point.
//!
//! Shift locks the current direction/plane inference; arrow keys lock an axis
//! (→ red, ← green, ↑ blue, ↓ parallel to the last hovered edge).

pub mod geo;

use crate::document::guarded;
use crate::overlay::{Marker2d, MarkerShape};
use egui::Color32;
use geo::{CamKey, GeoCache, ScreenCache, closest_on_line_to_ray, closest_on_seg_to_ray, dist_point_seg2};
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::{ContextPath, EntityRef, FaceId, InstanceId, Model, Plane, Ray};
use varstotas_render::{Camera, Overlay3d, OverlayLine};
use std::cell::RefCell;
use std::rc::Rc;

/// Pick radius for points (px).
pub const POINT_PICK_PX: f64 = 12.0;
/// Pick radius for edges / guides (px).
pub const EDGE_PICK_PX: f64 = 8.0;
/// Pick radius for direction lines (px).
pub const AXIS_PICK_PX: f64 = 9.0;
/// Pick radius for "from point" alignment lines (px).
pub const FROM_POINT_PX: f64 = 6.0;
/// Hover time after which a point becomes a "from point" reference (s).
pub const HOVER_REMEMBER_S: f64 = 0.5;

/// Kind of snap, in priority order (see the inference table in docs/05-UX-SPEC.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InferenceKind {
    Endpoint,
    Midpoint,
    Center,
    Intersection,
    OnEdge,
    OnRedAxis,
    OnGreenAxis,
    OnBlueAxis,
    ParallelToEdge,
    PerpendicularToEdge,
    FromPoint,
    OnFace,
    GuidePoint,
    OnGuide,
    /// The model (world) origin.
    Origin,
    /// Tool-level shape inferences (tooltip only).
    Square,
    GoldenSection,
    HalfCircle,
    /// No snap: point on the ground plane / at a default depth.
    Free,
}

impl InferenceKind {
    /// Tooltip text shown near the cursor.
    pub fn label(self) -> &'static str {
        match self {
            Self::Endpoint => "Endpoint",
            Self::Midpoint => "Midpoint",
            Self::Center => "Center",
            Self::Intersection => "Intersection",
            Self::OnEdge => "On Edge",
            Self::OnRedAxis => "On Red Axis",
            Self::OnGreenAxis => "On Green Axis",
            Self::OnBlueAxis => "On Blue Axis",
            Self::ParallelToEdge => "Parallel to Edge",
            Self::PerpendicularToEdge => "Perpendicular to Edge",
            Self::FromPoint => "From Point",
            Self::OnFace => "On Face",
            Self::GuidePoint => "Guide Point",
            Self::OnGuide => "On Line",
            Self::Origin => "Origin",
            Self::Square => "Square",
            Self::GoldenSection => "Golden Section",
            Self::HalfCircle => "Half Circle",
            Self::Free => "",
        }
    }

    /// Marker colour per the UX spec.
    pub fn color(self) -> Color32 {
        match self {
            Self::Endpoint | Self::Center => Color32::from_rgb(0x00, 0xC0, 0x00),
            Self::Origin => Color32::from_rgb(0xE0, 0xA0, 0x00),
            Self::Midpoint => Color32::from_rgb(0x00, 0xC0, 0xFF),
            Self::OnEdge | Self::Intersection | Self::OnRedAxis => Color32::from_rgb(0xFF, 0x00, 0x00),
            Self::OnGreenAxis => Color32::from_rgb(0x00, 0xB0, 0x00),
            Self::OnBlueAxis | Self::OnFace => Color32::from_rgb(0x00, 0x00, 0xFF),
            Self::ParallelToEdge | Self::PerpendicularToEdge => Color32::from_rgb(0xFF, 0x00, 0xFF),
            Self::FromPoint | Self::GuidePoint | Self::OnGuide => Color32::from_rgb(0x30, 0x30, 0x30),
            Self::Square | Self::GoldenSection | Self::HalfCircle => Color32::from_rgb(0xFF, 0x00, 0xFF),
            Self::Free => Color32::BLACK,
        }
    }

    pub fn marker(self) -> Option<MarkerShape> {
        Some(match self {
            Self::Endpoint | Self::Midpoint | Self::GuidePoint | Self::Origin => MarkerShape::Dot,
            Self::Center => MarkerShape::Ring,
            Self::Intersection => MarkerShape::Cross,
            Self::OnEdge | Self::OnGuide => MarkerShape::Square,
            Self::OnFace => MarkerShape::Diamond,
            _ => return None,
        })
    }

    /// Point-type inferences (ranked above edges/directions).
    pub fn is_point(self) -> bool {
        matches!(
            self,
            Self::Endpoint | Self::Midpoint | Self::Center | Self::Intersection | Self::GuidePoint | Self::Origin
        )
    }

    pub fn is_direction(self) -> bool {
        matches!(
            self,
            Self::OnRedAxis | Self::OnGreenAxis | Self::OnBlueAxis | Self::ParallelToEdge | Self::PerpendicularToEdge
        )
    }

    fn point_rank(self) -> u8 {
        match self {
            Self::Endpoint => 0,
            Self::Origin => 1,
            Self::Midpoint | Self::Center => 1,
            Self::GuidePoint => 1,
            Self::Intersection => 2,
            _ => 3,
        }
    }
}

/// Axis kind for a world direction (if aligned with X/Y/Z within ~0.5°).
pub fn axis_kind_of(d: DVec3) -> Option<InferenceKind> {
    let d = d.normalize_or_zero();
    if d.x.abs() > 0.99996 {
        Some(InferenceKind::OnRedAxis)
    } else if d.y.abs() > 0.99996 {
        Some(InferenceKind::OnGreenAxis)
    } else if d.z.abs() > 0.99996 {
        Some(InferenceKind::OnBlueAxis)
    } else {
        None
    }
}

/// Entity the inference snapped to (in the context reached by `path`).
#[derive(Clone, Debug, PartialEq)]
pub struct InferenceRef {
    pub path: ContextPath,
    pub entity: EntityRef,
}

/// Face under the cursor (world plane).
#[derive(Clone, Debug, PartialEq)]
pub struct FaceInfo {
    pub path: ContextPath,
    pub face: FaceId,
    pub plane: Plane,
    pub point: DVec3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InferenceResult {
    /// Snapped world-space point.
    pub point: DVec3,
    pub kind: InferenceKind,
    /// Direction constraint (axis / parallel / perpendicular), unit vector.
    pub constraint_dir: Option<DVec3>,
    pub ref_entity: Option<InferenceRef>,
    /// Face under the cursor, even if the point snapped to something else.
    pub face: Option<FaceInfo>,
    /// Edge segment (world) the point lies on (OnEdge / Midpoint).
    pub edge: Option<(DVec3, DVec3)>,
    /// Origin of the direction line (the anchor) for drawing.
    pub line_origin: Option<DVec3>,
    /// Reference edge for parallel/perpendicular inferences (drawn magenta).
    pub ref_edge: Option<(DVec3, DVec3)>,
    /// "From point" reference: source point and the colour kind of the dotted line.
    pub from_point: Option<(DVec3, InferenceKind)>,
    /// The result came from a Shift/arrow/tool lock.
    pub locked: bool,
}

impl InferenceResult {
    pub fn free(point: DVec3) -> Self {
        Self {
            point,
            kind: InferenceKind::Free,
            constraint_dir: None,
            ref_entity: None,
            face: None,
            edge: None,
            line_origin: None,
            ref_edge: None,
            from_point: None,
            locked: false,
        }
    }

    fn with_kind(point: DVec3, kind: InferenceKind) -> Self {
        Self { kind, ..Self::free(point) }
    }

    /// Whether the point snapped to model geometry (not free / face / direction only).
    pub fn is_geometry_snap(&self) -> bool {
        self.kind.is_point() || matches!(self.kind, InferenceKind::OnEdge | InferenceKind::OnGuide)
    }
}

/// A constraint the result must satisfy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Constraint {
    /// Result lies on the infinite line through `origin` along unit `dir`; `kind` colours it.
    Line { origin: DVec3, dir: DVec3, kind: InferenceKind },
    /// Result lies on the plane.
    Plane(Plane),
}

/// Arrow-key axis lock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArrowLock {
    Red,
    Green,
    Blue,
    /// Parallel to the last hovered edge.
    Edge,
}

pub struct Query<'a> {
    pub model: &'a Model,
    pub model_version: u64,
    pub camera: &'a Camera,
    pub viewport: DVec2,
    /// Viewport-local cursor position (logical points).
    pub cursor: DVec2,
    #[allow(dead_code)] // reserved: geometry outside the active context is inferred too
    pub active_path: &'a [InstanceId],
    /// The tool's previous click (enables direction inferences).
    pub anchor: Option<DVec3>,
    /// Constraint imposed by the tool (e.g. Push/Pull normal line). Overrides locks.
    pub constraint: Option<Constraint>,
    /// Plane for free points (default: horizontal through the anchor, else z = 0).
    pub fallback_plane: Option<Plane>,
    pub time: f64,
    /// Shift held: lock the current inference.
    pub shift: bool,
}

/// Snapping engine with per-frame caches and hover memory. One per tool instance.
#[derive(Default)]
struct SharedCaches {
    /// ((model address, model version), candidates).
    geo: Option<((usize, u64), Rc<GeoCache>)>,
    screen: Option<(CamKey, Rc<ScreenCache>)>,
}

thread_local! {
    static SHARED: RefCell<SharedCaches> = RefCell::new(SharedCaches::default());
}

#[derive(Default)]
pub struct InferenceEngine {
    /// Recently hovered edges (world), most recent last.
    pub recent_edges: Vec<(DVec3, DVec3)>,
    hover: Option<(DVec3, f64)>,
    /// Remembered points for "From Point" alignment, most recent last.
    pub from_points: Vec<DVec3>,
    pub arrow_lock: Option<ArrowLock>,
    shift_lock: Option<Constraint>,
    last: Option<InferenceResult>,
}

const MAX_RECENT_EDGES: usize = 3;
const MAX_FROM_POINTS: usize = 3;

fn same_seg(a: (DVec3, DVec3), b: (DVec3, DVec3)) -> bool {
    let e = 1e-6;
    (a.0.distance_squared(b.0) < e && a.1.distance_squared(b.1) < e)
        || (a.0.distance_squared(b.1) < e && a.1.distance_squared(b.0) < e)
}

impl InferenceEngine {
    #[allow(dead_code)]
    pub fn last(&self) -> Option<&InferenceResult> {
        self.last.as_ref()
    }

    /// Forget locks (e.g. when a tool resets).
    pub fn clear_locks(&mut self) {
        self.arrow_lock = None;
        self.shift_lock = None;
    }

    /// Handle an arrow key; returns true if consumed. Pressing the same arrow again unlocks.
    pub fn handle_arrow(&mut self, key: egui::Key) -> bool {
        let lock = match key {
            egui::Key::ArrowRight => ArrowLock::Red,
            egui::Key::ArrowLeft => ArrowLock::Green,
            egui::Key::ArrowUp => ArrowLock::Blue,
            egui::Key::ArrowDown => ArrowLock::Edge,
            _ => return false,
        };
        if self.arrow_lock == Some(lock) {
            self.arrow_lock = None;
        } else if lock != ArrowLock::Edge || !self.recent_edges.is_empty() {
            self.arrow_lock = Some(lock);
        }
        true
    }

    /// The locked arrow direction (unit) and its colour kind.
    pub fn arrow_dir(&self) -> Option<(DVec3, InferenceKind)> {
        match self.arrow_lock? {
            ArrowLock::Red => Some((DVec3::X, InferenceKind::OnRedAxis)),
            ArrowLock::Green => Some((DVec3::Y, InferenceKind::OnGreenAxis)),
            ArrowLock::Blue => Some((DVec3::Z, InferenceKind::OnBlueAxis)),
            ArrowLock::Edge => {
                let (a, b) = *self.recent_edges.last()?;
                let d = (b - a).normalize_or_zero();
                (d != DVec3::ZERO).then_some((d, InferenceKind::ParallelToEdge))
            }
        }
    }

    fn remember_edge(&mut self, seg: (DVec3, DVec3)) {
        if let Some(i) = self.recent_edges.iter().position(|s| same_seg(*s, seg)) {
            self.recent_edges.remove(i);
        }
        self.recent_edges.push(seg);
        if self.recent_edges.len() > MAX_RECENT_EDGES {
            self.recent_edges.remove(0);
        }
    }

    fn remember_hover(&mut self, p: Option<DVec3>, time: f64) {
        match (p, self.hover) {
            (Some(p), Some((h, t0))) if p.distance_squared(h) < 1e-6 => {
                if time - t0 >= HOVER_REMEMBER_S && !self.from_points.iter().any(|q| q.distance_squared(p) < 1e-6) {
                    self.from_points.push(p);
                    if self.from_points.len() > MAX_FROM_POINTS {
                        self.from_points.remove(0);
                    }
                }
            }
            (Some(p), _) => self.hover = Some((p, time)),
            (None, _) => self.hover = None,
        }
    }

    /// Run inference for one cursor position.
    pub fn infer(&mut self, q: &Query) -> InferenceResult {
        let r = guarded(|| self.infer_inner(q)).unwrap_or_else(|_| InferenceResult::free(safe_fallback(q)));
        let r = if r.point.is_finite() { r } else { InferenceResult::free(safe_fallback(q)) };
        self.last = Some(r.clone());
        r
    }

    fn effective_constraint(&mut self, q: &Query) -> Option<Constraint> {
        if q.constraint.is_some() {
            // Tool constraint: Shift/arrows do not apply.
            return q.constraint;
        }
        if let (Some(anchor), Some((dir, kind))) = (q.anchor, self.arrow_dir()) {
            return Some(Constraint::Line { origin: anchor, dir, kind });
        }
        if !q.shift {
            self.shift_lock = None;
            return None;
        }
        if self.shift_lock.is_none()
            && let Some(last) = &self.last
        {
            self.shift_lock = if let (Some(dir), Some(o)) = (last.constraint_dir, last.line_origin) {
                Some(Constraint::Line { origin: o, dir, kind: last.kind })
            } else if last.kind == InferenceKind::OnFace {
                last.face.as_ref().map(|f| Constraint::Plane(f.plane))
            } else if let (InferenceKind::OnEdge, Some((a, b))) = (last.kind, last.edge) {
                let d = (b - a).normalize_or_zero();
                (d != DVec3::ZERO).then_some(Constraint::Line { origin: a, dir: d, kind: InferenceKind::OnEdge })
            } else {
                None
            };
        }
        self.shift_lock
    }

    fn infer_inner(&mut self, q: &Query) -> InferenceResult {
        let model_key = (q.model as *const Model as usize, q.model_version);
        // World candidates and their screen projections are shared by all engines (every tool
        // has one), so switching tools does not rebuild them on big models.
        let (geo, screen) = SHARED.with_borrow_mut(|sh| {
            if sh.geo.as_ref().is_none_or(|(k, _)| *k != model_key) {
                sh.geo = Some((model_key, Rc::new(GeoCache::build(q.model, q.model_version))));
                sh.screen = None;
            }
            let geo = sh.geo.as_ref().map(|(_, g)| g.clone()).unwrap_or_default();
            let cam = CamKey::of(q.camera, q.viewport);
            if sh.screen.as_ref().is_none_or(|(k, _)| *k != cam) {
                let mut sc = ScreenCache::default();
                sc.update(&geo, q.camera, q.viewport);
                sh.screen = Some((cam, Rc::new(sc)));
            }
            let screen = sh.screen.as_ref().map(|(_, s)| s.clone()).unwrap_or_default();
            (geo, screen)
        });
        self.infer_with(q, &geo, &screen)
    }

    fn infer_with(&mut self, q: &Query, geo: &GeoCache, screen: &ScreenCache) -> InferenceResult {
        let cam = q.camera;
        let vp = q.viewport;
        let ray = crate::nav::screen_ray(cam, q.cursor, vp);
        let (ro, rd) = (ray.origin, ray.dir);
        let to_screen = |p: DVec3| cam.world_to_screen(p, vp);
        let px_dist = |p: DVec3| to_screen(p).map(|s| s.distance(q.cursor)).unwrap_or(f64::INFINITY);

        // Face under the cursor.
        let hit = crate::document::cached_raycast(q.model, q.model_version, &ray);
        let face = hit.as_ref().and_then(|h| {
            let m = q.model.transform_at(&h.path);
            let mesh = q.model.mesh_at(&h.path)?;
            let f = mesh.faces.get(h.face)?;
            let n = m.inverse().transpose().transform_vector3(f.normal).normalize_or(f.normal);
            Some(FaceInfo { path: h.path.clone(), face: h.face, plane: Plane::new(h.point, n), point: h.point })
        });
        let hit_t = hit.as_ref().map(|h| h.t);
        let visible = |p: DVec3| -> bool {
            let Some(t) = hit_t else { return true };
            let depth = (p - ro).dot(rd);
            let tol = (cam.pixel_size_at(p, vp) * 3.0).max(depth.abs() * 1e-4).max(1e-3);
            t >= depth - tol
        };

        // ---- point candidates -------------------------------------------------
        let mut pts: Vec<(u8, f64, usize)> = Vec::new();
        for (i, s) in screen.points.iter().enumerate() {
            if let Some(s) = s {
                let d = s.distance(q.cursor);
                if d <= POINT_PICK_PX {
                    pts.push((geo.points[i].kind.point_rank(), d, i));
                }
            }
        }
        pts.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
        let best_point = pts.iter().take(8).find(|c| visible(geo.points[c.2].pos)).map(|c| &geo.points[c.2]);

        // ---- edge candidates ----------------------------------------------------
        let mut near_edges: Vec<(f64, usize)> = Vec::new();
        for (i, s) in screen.edges.iter().enumerate() {
            if let Some((a, b)) = s {
                let (d, _) = dist_point_seg2(q.cursor, *a, *b);
                if d <= EDGE_PICK_PX * 1.5 {
                    near_edges.push((d, i));
                }
            }
        }
        near_edges.sort_by(|a, b| a.0.total_cmp(&b.0));
        near_edges.truncate(16);
        let best_edge = near_edges.iter().filter(|(d, _)| *d <= EDGE_PICK_PX).find_map(|(_, i)| {
            let e = &geo.edges[*i];
            let p = closest_on_seg_to_ray(e.a, e.b, ro, rd);
            visible(p).then_some((e, p))
        });

        // Intersections between near edges / guides.
        let mut best_ix: Option<(f64, DVec3)> = None;
        for (k, (_, i)) in near_edges.iter().enumerate() {
            let e1 = &geo.edges[*i];
            for (_, j) in near_edges.iter().skip(k + 1) {
                let e2 = &geo.edges[*j];
                if let Some((s, t, c1, c2)) = varstotas_kernel::geom::seg_seg_closest(e1.a, e1.b, e2.a, e2.b) {
                    let interior = |x: f64| x > 1e-6 && x < 1.0 - 1e-6;
                    if !(interior(s) || interior(t)) {
                        continue; // shared endpoint: that's an Endpoint inference
                    }
                    let tol = cam.pixel_size_at(c1, vp) * 2.0;
                    if c1.distance(c2) <= tol.max(1e-3) {
                        let d = px_dist(c1);
                        if d <= POINT_PICK_PX && visible(c1) && best_ix.is_none_or(|(bd, _)| d < bd) {
                            best_ix = Some((d, (c1 + c2) * 0.5));
                        }
                    }
                }
            }
        }

        // Guides.
        let mut best_guide: Option<(f64, DVec3, DVec3)> = None;
        for (o, d) in &geo.guides {
            if let Some(p) = closest_on_line_to_ray(*o, *d, ro, rd)
                && (p - ro).dot(rd) > 0.0
            {
                let dist = px_dist(p);
                if dist <= EDGE_PICK_PX && best_guide.is_none_or(|(bd, _, _)| dist < bd) {
                    best_guide = Some((dist, p, *d));
                }
            }
        }

        // Hover memory.
        self.remember_hover(best_point.map(|p| p.pos), q.time);
        if let Some((e, _)) = best_edge {
            self.remember_edge((e.a, e.b));
        }

        let finish = |mut r: InferenceResult| {
            r.face = face.clone();
            r
        };

        // ---- constrained -----------------------------------------------------------
        if let Some(c) = self.effective_constraint(q) {
            let locked = q.constraint.is_none();
            match c {
                Constraint::Line { origin, dir, kind } => {
                    let mut r = InferenceResult::with_kind(origin, kind);
                    r.constraint_dir = Some(dir);
                    r.line_origin = Some(origin);
                    r.locked = locked;
                    let proj = |p: DVec3| origin + dir * (p - origin).dot(dir);
                    // Natural position: closest point between the pick ray and the line.
                    // When the line is (nearly) parallel to the view ray it is ill-defined;
                    // fall back to the anchor rather than jumping far away.
                    let natural = constrained_line_point(cam, vp, q.cursor, origin, dir, ro, rd);
                    // Only point-like snaps (endpoints, midpoints, intersections) are projected
                    // onto the line; an edge snap would project an arbitrary point.
                    let snap_src = best_point.map(|p| p.pos).or(best_ix.map(|x| x.1));
                    let from = self.from_points.iter().rev().copied().find(|fp| px_dist(proj(*fp)) <= FROM_POINT_PX);
                    if let Some(src) = snap_src {
                        r.point = proj(src);
                        if src.distance(r.point) > 1e-6 {
                            r.from_point = Some((src, axis_kind_of(src - r.point).unwrap_or(InferenceKind::FromPoint)));
                        }
                    } else if let Some(fp) = from {
                        r.point = proj(fp);
                        r.from_point = Some((fp, axis_kind_of(fp - r.point).unwrap_or(InferenceKind::FromPoint)));
                    } else {
                        r.point = natural;
                    }
                    return finish(r);
                }
                Constraint::Plane(pl) => {
                    if let Some(p) = best_point.filter(|p| pl.distance(p.pos).abs() < 1e-3) {
                        let mut r = InferenceResult::with_kind(p.pos, p.kind);
                        r.ref_entity = p.r.clone();
                        r.locked = locked;
                        return finish(r);
                    }
                    let point = pl
                        .intersect_ray(&ray)
                        .filter(|t| *t > 0.0)
                        .map(|t| ray.at(t))
                        .unwrap_or_else(|| pl.project(safe_fallback(q)));
                    let mut r = InferenceResult::with_kind(point, InferenceKind::OnFace);
                    r.face = Some(FaceInfo {
                        path: face.as_ref().map(|f| f.path.clone()).unwrap_or_default(),
                        face: face.as_ref().map(|f| f.face).unwrap_or_default(),
                        plane: pl,
                        point,
                    });
                    r.locked = locked;
                    return r;
                }
            }
        }

        // ---- unconstrained ranking ---------------------------------------------------
        if let Some(p) = best_point {
            let mut r = InferenceResult::with_kind(p.pos, p.kind);
            r.ref_entity = p.r.clone();
            if p.kind == InferenceKind::Midpoint
                && let Some(InferenceRef { entity: EntityRef::Edge(_), .. }) = &p.r
            {
                // find the segment for highlighting
                if let Some(e) = geo.edges.iter().find(|e| e.r == p.r) {
                    r.edge = Some((e.a, e.b));
                }
            }
            return finish(r);
        }
        if let Some((_, p)) = best_ix {
            return finish(InferenceResult::with_kind(p, InferenceKind::Intersection));
        }
        if let Some((e, p)) = best_edge {
            let mut r = InferenceResult::with_kind(p, InferenceKind::OnEdge);
            r.ref_entity = e.r.clone();
            r.edge = Some((e.a, e.b));
            return finish(r);
        }
        if let Some((_, p, _)) = best_guide {
            return finish(InferenceResult::with_kind(p, InferenceKind::OnGuide));
        }

        // Directions from the anchor.
        if let Some(anchor) = q.anchor {
            let mut dirs: Vec<DirCand> = vec![
                (DVec3::X, InferenceKind::OnRedAxis, None),
                (DVec3::Y, InferenceKind::OnGreenAxis, None),
                (DVec3::Z, InferenceKind::OnBlueAxis, None),
            ];
            for seg in self.recent_edges.iter().rev() {
                let d = (seg.1 - seg.0).normalize_or_zero();
                if d == DVec3::ZERO || axis_kind_of(d).is_some() {
                    continue;
                }
                dirs.push((d, InferenceKind::ParallelToEdge, Some(*seg)));
            }
            for seg in self.recent_edges.iter().rev() {
                let d = (seg.1 - seg.0).normalize_or_zero();
                if d == DVec3::ZERO {
                    continue;
                }
                // Perpendicular within the plane through the edge whose normal is the face/up.
                let n = face.as_ref().map(|f| f.plane.normal).filter(|n| n.dot(d).abs() < 1e-6).unwrap_or(DVec3::Z);
                let p = d.cross(n).normalize_or_zero();
                if p != DVec3::ZERO && axis_kind_of(p).is_none() {
                    dirs.push((p, InferenceKind::PerpendicularToEdge, Some(*seg)));
                }
            }
            let a_s = to_screen(anchor);
            let px = cam.pixel_size_at(anchor, vp);
            let mut best_dir: Option<BestDir> = None;
            for (d, kind, seg) in dirs {
                // Skip lines that are (nearly) end-on in screen space: tiny cursor offsets
                // would map to huge distances along them.
                if let (Some(sa), Some(sb)) = (a_s, to_screen(anchor + d * px * 40.0))
                    && sa.distance(sb) < 10.0
                {
                    continue;
                }
                let Some(p) = closest_on_line_to_ray(anchor, d, ro, rd) else { continue };
                if (p - ro).dot(rd) <= 0.0 {
                    continue;
                }
                let bonus = if kind.is_direction() && matches!(kind, InferenceKind::OnRedAxis | InferenceKind::OnGreenAxis | InferenceKind::OnBlueAxis) { 1.0 } else { 0.0 };
                let dist = px_dist(p) - bonus;
                if dist <= AXIS_PICK_PX && best_dir.as_ref().is_none_or(|b| dist < b.0) {
                    best_dir = Some((dist, p, d, kind, seg));
                }
            }
            if let Some((_, mut p, d, kind, seg)) = best_dir {
                let mut r = InferenceResult::with_kind(p, kind);
                r.constraint_dir = Some(d);
                r.line_origin = Some(anchor);
                r.ref_edge = seg;
                // From-point alignment along this direction.
                let proj = |x: DVec3| anchor + d * (x - anchor).dot(d);
                if let Some(fp) = self
                    .from_points
                    .iter()
                    .rev()
                    .copied()
                    .filter(|fp| fp.distance_squared(anchor) > 1e-6)
                    .find(|fp| px_dist(proj(*fp)) <= FROM_POINT_PX)
                {
                    p = proj(fp);
                    r.point = p;
                    if fp.distance(p) > 1e-6 {
                        r.from_point = Some((fp, axis_kind_of(fp - p).unwrap_or(InferenceKind::FromPoint)));
                    }
                }
                return finish(r);
            }
        }

        // From point (axis lines through remembered points).
        let mut best_from: Option<(f64, DVec3, DVec3, InferenceKind)> = None;
        for fp in self.from_points.iter().rev() {
            if q.anchor.is_some_and(|a| a.distance_squared(*fp) < 1e-6) {
                continue;
            }
            for (d, kind) in [
                (DVec3::X, InferenceKind::OnRedAxis),
                (DVec3::Y, InferenceKind::OnGreenAxis),
                (DVec3::Z, InferenceKind::OnBlueAxis),
            ] {
                if end_on(cam, vp, *fp, d) {
                    continue;
                }
                let Some(p) = closest_on_line_to_ray(*fp, d, ro, rd) else { continue };
                if (p - ro).dot(rd) <= 0.0 || p.distance(*fp) < 1e-6 {
                    continue;
                }
                let dist = px_dist(p);
                if dist <= FROM_POINT_PX && best_from.is_none_or(|b| dist < b.0) {
                    best_from = Some((dist, p, *fp, kind));
                }
            }
        }
        if let Some((_, p, fp, kind)) = best_from {
            let mut r = InferenceResult::with_kind(p, InferenceKind::FromPoint);
            r.from_point = Some((fp, kind));
            return finish(r);
        }

        if let Some(f) = &face {
            let mut r = InferenceResult::with_kind(f.point, InferenceKind::OnFace);
            r.ref_entity = Some(InferenceRef { path: f.path.clone(), entity: EntityRef::Face(f.face) });
            return finish(r);
        }

        // Free point on the fallback plane.
        let plane = q.fallback_plane.unwrap_or_else(|| Plane::new(DVec3::new(0.0, 0.0, q.anchor.map_or(0.0, |a| a.z)), DVec3::Z));
        finish(InferenceResult::free(free_point(q, &ray, plane)))
    }
}

/// Point on `plane` under the ray, or on the view plane through the anchor/target if the
/// plane is (nearly) edge-on or behind the camera.
pub fn free_point(q: &Query, ray: &Ray, plane: Plane) -> DVec3 {
    let dist = (q.camera.eye - q.camera.target).length().max(1.0);
    if let Some(t) = plane.intersect_ray(ray) {
        let grazing = ray.dir.dot(plane.normal).abs() < 0.02;
        // Parallel projection has no horizon: any non-grazing hit in front is fine.
        if t > 0.0 && (q.camera.ortho || t < dist * 50.0) && !grazing {
            let p = ray.at(t);
            if p.is_finite() {
                return p;
            }
        }
    }
    let center = q.anchor.unwrap_or(q.camera.target);
    let view = Plane::new(center, q.camera.basis().forward);
    view.intersect_ray(ray).map(|t| ray.at(t)).filter(|p| p.is_finite()).unwrap_or(center)
}

fn safe_fallback(q: &Query) -> DVec3 {
    q.anchor.unwrap_or(q.camera.target)
}

// ---------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------

pub fn rgba(c: Color32) -> [f32; 4] {
    [c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, c.a() as f32 / 255.0]
}

pub fn push_line(out: &mut Overlay3d, a: DVec3, b: DVec3, color: Color32, width: f32, dashed: bool, on_top: bool) {
    if !a.is_finite() || !b.is_finite() {
        return;
    }
    out.lines.push(OverlayLine {
        a: a.as_vec3().to_array(),
        b: b.as_vec3().to_array(),
        color: rgba(color),
        width_px: width,
        dashed,
        on_top,
    });
}

/// True when the line through `p` along `d` is (nearly) end-on in screen space.
fn end_on(cam: &Camera, vp: DVec2, p: DVec3, d: DVec3) -> bool {
    let px = cam.pixel_size_at(p, vp);
    match (cam.world_to_screen(p, vp), cam.world_to_screen(p + d * px * 40.0, vp)) {
        (Some(a), Some(b)) => a.distance(b) < 10.0,
        _ => false,
    }
}

/// Closest point on the line `origin + t*dir` to the pick ray, robust at grazing angles:
/// the result is rejected (→ `origin`) when the line is nearly end-on, and it is limited
/// to points whose projection stays near the cursor.
fn constrained_line_point(cam: &Camera, vp: DVec2, cursor: DVec2, origin: DVec3, dir: DVec3, ro: DVec3, rd: DVec3) -> DVec3 {
    let Some(p) = closest_on_line_to_ray(origin, dir, ro, rd) else { return origin };
    if !p.is_finite() {
        return origin;
    }
    // End-on in screen space: the cursor cannot meaningfully drive the distance.
    if end_on(cam, vp, origin, dir) {
        let px = cam.pixel_size_at(origin, vp);
        if let (Some(a), Some(b)) = (cam.world_to_screen(origin, vp), cam.world_to_screen(origin + dir * px * 40.0, vp))
            && a.distance(b) < 1.0
        {
            return origin;
        }
    }
    // Behind the camera or projecting far from the cursor: clamp to the farthest point that
    // still projects within the viewport neighbourhood of the cursor.
    let ok = |x: DVec3| (x - ro).dot(rd) > 0.0 && cam.world_to_screen(x, vp).is_some_and(|s| s.distance(cursor) < vp.length());
    if ok(p) {
        return p;
    }
    // Bisect between the origin and p.
    let (mut lo, mut hi) = (origin, p);
    if !ok(lo) {
        return origin;
    }
    for _ in 0..40 {
        let m = (lo + hi) * 0.5;
        if ok(m) { lo = m } else { hi = m }
    }
    lo
}

type DirCand = (DVec3, InferenceKind, Option<(DVec3, DVec3)>);
type BestDir = (f64, DVec3, DVec3, InferenceKind, Option<(DVec3, DVec3)>);

/// Draw an inference result: marker, tooltip, direction line from the anchor,
/// dotted from-point line, highlighted reference edge.
pub fn draw_result(r: &InferenceResult, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
    if (r.kind.is_direction() || matches!(r.kind, InferenceKind::OnEdge))
        && let Some(o) = r.line_origin
    {
        let w = if r.locked { 3.0 } else { 2.0 };
        push_line(out3d, o, r.point, r.kind.color(), w, false, true);
    }
    if let Some(seg) = r.ref_edge {
        push_line(out3d, seg.0, seg.1, InferenceKind::ParallelToEdge.color(), 3.0, false, true);
    }
    if let Some((fp, kind)) = r.from_point {
        let c = if kind == InferenceKind::FromPoint { Color32::from_gray(60) } else { kind.color() };
        push_line(out3d, fp, r.point, c, 1.5, true, true);
        out2d.push(Marker2d::Point { at: fp, shape: MarkerShape::Dot, color: Color32::from_gray(80) });
    }
    if let (InferenceKind::OnEdge | InferenceKind::Midpoint, Some((a, b))) = (r.kind, r.edge) {
        push_line(out3d, a, b, r.kind.color().gamma_multiply(0.8), 2.0, false, true);
    }
    if let Some(shape) = r.kind.marker() {
        out2d.push(Marker2d::Point { at: r.point, shape, color: r.kind.color() });
    }
    let mut label = r.kind.label().to_string();
    if r.kind.is_direction() && r.from_point.is_some() {
        label = "From Point".into();
    }
    if !label.is_empty() {
        out2d.push(Marker2d::CursorTooltip { text: label });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use varstotas_kernel::glam::DVec2;

    fn camera_top() -> Camera {
        Camera { eye: DVec3::new(0.0, -0.001, 10_000.0), target: DVec3::ZERO, up: DVec3::Y, ortho: false, ..Camera::default() }
    }

    fn query<'a>(model: &'a Model, cam: &'a Camera, cursor: DVec2, anchor: Option<DVec3>) -> Query<'a> {
        Query {
            model,
            model_version: 1,
            camera: cam,
            viewport: DVec2::new(800.0, 600.0),
            cursor,
            active_path: &[],
            anchor,
            constraint: None,
            fallback_plane: None,
            time: 0.0,
            shift: false,
        }
    }

    fn square_model() -> Model {
        let mut m = Model::new();
        m.root_mesh_mut().add_face(&[
            DVec3::new(0.0, 0.0, 0.0),
            DVec3::new(2000.0, 0.0, 0.0),
            DVec3::new(2000.0, 2000.0, 0.0),
            DVec3::new(0.0, 2000.0, 0.0),
        ]);
        m
    }

    fn screen(cam: &Camera, p: DVec3) -> DVec2 {
        cam.world_to_screen(p, DVec2::new(800.0, 600.0)).unwrap()
    }

    #[test]
    fn origin_snaps_exactly() {
        let m = Model::new();
        let cam = camera_top();
        let mut e = InferenceEngine::default();
        let s = screen(&cam, DVec3::ZERO) + DVec2::new(2.5, -1.5);
        let r = e.infer(&query(&m, &cam, s, None));
        assert_eq!(r.kind, InferenceKind::Origin);
        assert_eq!(r.point, DVec3::ZERO);
    }

    #[test]
    fn endpoint_beats_midpoint_and_edge() {
        let m = square_model();
        let cam = camera_top();
        let mut e = InferenceEngine::default();
        let s = screen(&cam, DVec3::new(2000.0, 0.0, 0.0)) + DVec2::new(3.0, 2.0);
        let r = e.infer(&query(&m, &cam, s, None));
        assert_eq!(r.kind, InferenceKind::Endpoint);
        assert!(r.point.distance(DVec3::new(2000.0, 0.0, 0.0)) < 1e-6);
    }

    #[test]
    fn midpoint_then_on_edge_then_face_then_free() {
        let m = square_model();
        let cam = camera_top();
        let mut e = InferenceEngine::default();
        let r = e.infer(&query(&m, &cam, screen(&cam, DVec3::new(1000.0, 0.0, 0.0)) + DVec2::new(2.0, 0.0), None));
        assert_eq!(r.kind, InferenceKind::Midpoint);
        let r = e.infer(&query(&m, &cam, screen(&cam, DVec3::new(500.0, 0.0, 0.0)) + DVec2::new(0.0, 3.0), None));
        assert_eq!(r.kind, InferenceKind::OnEdge);
        assert!(r.point.y.abs() < 1e-6 && (r.point.x - 500.0).abs() < 50.0);
        let r = e.infer(&query(&m, &cam, screen(&cam, DVec3::new(700.0, 900.0, 0.0)), None));
        assert_eq!(r.kind, InferenceKind::OnFace);
        let r = e.infer(&query(&m, &cam, screen(&cam, DVec3::new(-1500.0, -900.0, 0.0)), None));
        assert_eq!(r.kind, InferenceKind::Free);
        assert!(r.point.z.abs() < 1e-6 && r.point.is_finite());
    }

    #[test]
    fn axis_inference_from_anchor() {
        let m = Model::new();
        let cam = camera_top();
        let mut e = InferenceEngine::default();
        let anchor = DVec3::new(-1000.0, -1000.0, 0.0);
        // cursor slightly off the red axis
        let s = screen(&cam, DVec3::new(500.0, -1000.0, 0.0)) + DVec2::new(0.0, 4.0);
        let r = e.infer(&query(&m, &cam, s, Some(anchor)));
        assert_eq!(r.kind, InferenceKind::OnRedAxis);
        assert!((r.point.y + 1000.0).abs() < 1e-6);
        let s = screen(&cam, DVec3::new(-1000.0, 300.0, 0.0)) + DVec2::new(4.0, 0.0);
        let r = e.infer(&query(&m, &cam, s, Some(anchor)));
        assert_eq!(r.kind, InferenceKind::OnGreenAxis);
        // far from any axis → free
        let s = screen(&cam, DVec3::new(500.0, 500.0, 0.0));
        let r = e.infer(&query(&m, &cam, s, Some(anchor)));
        assert_eq!(r.kind, InferenceKind::Free);
    }

    #[test]
    fn arrow_lock_projects_onto_axis() {
        let m = Model::new();
        let cam = camera_top();
        let mut e = InferenceEngine::default();
        assert!(e.handle_arrow(egui::Key::ArrowRight));
        let anchor = DVec3::ZERO;
        let s = screen(&cam, DVec3::new(700.0, 900.0, 0.0));
        let r = e.infer(&query(&m, &cam, s, Some(anchor)));
        assert_eq!(r.kind, InferenceKind::OnRedAxis);
        assert!(r.locked);
        assert!(r.point.y.abs() < 1e-6 && (r.point.x - 700.0).abs() < 100.0);
        e.handle_arrow(egui::Key::ArrowRight);
        assert!(e.arrow_lock.is_none());
    }

    #[test]
    fn shift_locks_previous_direction() {
        let m = Model::new();
        let cam = camera_top();
        let mut e = InferenceEngine::default();
        let anchor = DVec3::ZERO;
        let s = screen(&cam, DVec3::new(600.0, 0.0, 0.0)) + DVec2::new(0.0, 3.0);
        assert_eq!(e.infer(&query(&m, &cam, s, Some(anchor))).kind, InferenceKind::OnRedAxis);
        let mut qq = query(&m, &cam, screen(&cam, DVec3::new(900.0, 800.0, 0.0)), Some(anchor));
        qq.shift = true;
        let r = e.infer(&qq);
        assert_eq!(r.kind, InferenceKind::OnRedAxis);
        assert!(r.locked && r.point.y.abs() < 1e-6);
    }

    #[test]
    fn intersection_of_crossing_edges() {
        let mut m = Model::new();
        m.root_mesh_mut().add_line(DVec3::new(-1000.0, 0.0, 0.0), DVec3::new(1000.0, 0.0, 0.0));
        // second line in a different context height so the kernel does not split them
        m.root_mesh_mut().add_line(DVec3::new(0.0, -1000.0, 500.0), DVec3::new(0.0, 1000.0, 500.0));
        let cam = camera_top();
        let mut e = InferenceEngine::default();
        let r = e.infer(&query(&m, &cam, screen(&cam, DVec3::ZERO) + DVec2::new(2.0, 2.0), None));
        // Not intersecting in 3D (different heights) → not an intersection.
        assert_ne!(r.kind, InferenceKind::Intersection);
    }

    #[test]
    fn parallel_to_recent_edge() {
        let mut m = Model::new();
        let a = DVec3::new(0.0, 0.0, 0.0);
        let b = DVec3::new(1000.0, 1000.0, 0.0);
        m.root_mesh_mut().add_line(a, b);
        let cam = camera_top();
        let mut e = InferenceEngine::default();
        // hover the edge
        let r = e.infer(&query(&m, &cam, screen(&cam, DVec3::new(300.0, 300.0, 0.0)), None));
        assert_eq!(r.kind, InferenceKind::OnEdge);
        let anchor = DVec3::new(-2000.0, 0.0, 0.0);
        let target = anchor + DVec3::new(700.0, 700.0, 0.0);
        let r = e.infer(&query(&m, &cam, screen(&cam, target) + DVec2::new(3.0, 0.0), Some(anchor)));
        assert_eq!(r.kind, InferenceKind::ParallelToEdge);
        let d = (r.point - anchor).normalize();
        assert!((d.x - d.y).abs() < 1e-6);
    }

    #[test]
    fn from_point_alignment_after_hover() {
        let m = square_model();
        let cam = camera_top();
        let mut e = InferenceEngine::default();
        let corner = DVec3::new(2000.0, 2000.0, 0.0);
        let mut q = query(&m, &cam, screen(&cam, corner), None);
        e.infer(&q);
        q.time = 1.0;
        e.infer(&q);
        assert_eq!(e.from_points.len(), 1);
        // Draw from an anchor along red; cursor near x = 2000 should align with the corner.
        let anchor = DVec3::new(-3000.0, -1500.0, 0.0);
        let s = screen(&cam, DVec3::new(1995.0, -1500.0, 0.0)) + DVec2::new(0.0, 2.0);
        let r = e.infer(&query(&m, &cam, s, Some(anchor)));
        assert_eq!(r.kind, InferenceKind::OnRedAxis);
        assert!(r.from_point.is_some());
        assert!((r.point.x - 2000.0).abs() < 1e-6);
    }

    #[test]
    fn line_constraint_snaps_to_points() {
        let m = square_model();
        let cam = camera_top();
        let mut e = InferenceEngine::default();
        let mut q = query(&m, &cam, screen(&cam, DVec3::new(2000.0, 2000.0, 0.0)), None);
        q.constraint = Some(Constraint::Line { origin: DVec3::ZERO, dir: DVec3::X, kind: InferenceKind::OnRedAxis });
        let r = e.infer(&q);
        assert!((r.point - DVec3::new(2000.0, 0.0, 0.0)).length() < 1e-6);
    }

    #[test]
    fn free_point_never_nan_looking_at_horizon() {
        let m = Model::new();
        let cam = Camera { eye: DVec3::new(0.0, -5000.0, 0.0), target: DVec3::ZERO, up: DVec3::Z, ..Camera::default() };
        let mut e = InferenceEngine::default();
        for y in [0.0, 299.0, 300.0, 301.0, 600.0] {
            let r = e.infer(&query(&m, &cam, DVec2::new(400.0, y), None));
            assert!(r.point.is_finite());
        }
    }

    #[test]
    fn circumcenter_works() {
        let c = geo::circumcenter(DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, 1.0, 0.0), DVec3::new(-1.0, 0.0, 0.0)).unwrap();
        assert!(c.length() < 1e-9);
    }
}
