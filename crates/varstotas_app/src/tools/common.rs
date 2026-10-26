//! Helpers shared by the drawing/editing tools: inference wrapper, context transforms,
//! picking in the active context, VCB parsing, overlay drawing, geometry previews.

use super::{OverlayView, ToolContext};
use crate::document::{Document, guarded};
use crate::inference::{self, Constraint, InferenceEngine, InferenceResult, Query, push_line};
use crate::overlay::Marker2d;
use egui::{Color32, Pos2};
use varstotas_kernel::glam::{DMat4, DVec2, DVec3};
use varstotas_kernel::{
    ContextPath, CurveInfo, EdgeId, EntityRef, FaceId, GuideGeom, GuideLine, GuideRef, InstanceId, LengthUnit, Mesh, Model, Plane, units,
};
use varstotas_render::{Overlay3d, OverlayPolygon};
use std::collections::HashSet;

pub const PREVIEW: Color32 = Color32::from_rgb(0x10, 0x10, 0x10);
pub const SELECT_BLUE: Color32 = Color32::from_rgb(0x00, 0x50, 0xFF);
pub const HOVER_FACE: Color32 = Color32::from_rgba_premultiplied(0x00, 0x28, 0x80, 0x50);
/// Press-drag-release threshold (px) to treat a release as a second click.
pub const DRAG_CLICK_PX: f32 = 8.0;

// ---------------------------------------------------------------------------
// Inference wrapper
// ---------------------------------------------------------------------------

/// Run inference for the current pointer.
pub fn infer(
    engine: &mut InferenceEngine,
    ctx: &ToolContext,
    anchor: Option<DVec3>,
    constraint: Option<Constraint>,
    fallback_plane: Option<Plane>,
) -> InferenceResult {
    let q = Query {
        model: &ctx.doc.model,
        model_version: ctx.doc.model_version,
        camera: ctx.camera,
        viewport: ctx.viewport_d(),
        cursor: DVec2::new(ctx.pointer.pos.x as f64, ctx.pointer.pos.y as f64),
        active_path: &ctx.doc.active_path,
        anchor,
        constraint,
        fallback_plane,
        time: ctx.time,
        shift: ctx.pointer.modifiers.shift,
    };
    engine.infer(&q)
}

/// Copy modifier (Ctrl on Windows/Linux, Option/Alt on macOS; Cmd also accepted).
pub fn copy_mod(m: &egui::Modifiers) -> bool {
    m.ctrl || m.alt || m.command
}

/// Tracks a toggling modifier (Ctrl/Option = toggle copy): flips on each rising edge.
#[derive(Default, Clone, Copy, Debug)]
pub struct ModToggle {
    prev: bool,
    pub on: bool,
}

impl ModToggle {
    pub fn update(&mut self, held: bool) {
        if held && !self.prev {
            self.on = !self.on;
        }
        self.prev = held;
    }
    pub fn reset(&mut self) {
        self.on = false;
    }
}

// ---------------------------------------------------------------------------
// Context transforms
// ---------------------------------------------------------------------------

/// World transform of the active editing context.
pub fn context_xf(doc: &Document) -> DMat4 {
    guarded(|| doc.model.transform_at(&doc.active_path)).unwrap_or(DMat4::IDENTITY)
}

/// World → active-context-local.
pub fn to_local(doc: &Document, p: DVec3) -> DVec3 {
    context_xf(doc).inverse().transform_point3(p)
}

/// World transform `t` expressed in the active context's local frame.
pub fn world_to_local_xf(doc: &Document, t: &DMat4) -> DMat4 {
    let xf = context_xf(doc);
    xf.inverse() * *t * xf
}

/// Scale factor of the active context along a world direction (local length per world length).
pub fn local_scale_along(doc: &Document, world_dir: DVec3) -> f64 {
    let inv = context_xf(doc).inverse();
    let l = inv.transform_vector3(world_dir.normalize_or(DVec3::X)).length();
    if l.is_finite() && l > 1e-12 { l } else { 1.0 }
}

/// Run `f` on the active mesh as one undo step.
pub fn commit_mesh<R>(ctx: &mut ToolContext, label: &str, f: impl FnOnce(&mut Mesh) -> R) -> Option<R> {
    let path = ctx.doc.active_path.clone();
    ctx.doc.commit(label, |m| m.mesh_at_mut(&path).map(f)).flatten()
}

/// Add a curve given in world coordinates to the active context (one undo step).
/// `pie` also adds the two radii closing it into a face.
pub fn commit_curve(ctx: &mut ToolContext, label: &str, info: &CurveInfo, pie: bool) -> bool {
    let inv = context_xf(ctx.doc).inverse();
    let Some(local) = info.transformed(&inv) else {
        // Non-uniformly scaled context: the curve is no longer circular locally, so add it
        // as a plain edge chain (no curve metadata).
        let mut pts: Vec<DVec3> = info.points().iter().map(|p| inv.transform_point3(*p)).collect();
        let closed = info.is_closed() || pie;
        if pie {
            pts.push(inv.transform_point3(info.center));
        }
        return commit_mesh(ctx, label, |m| m.add_polyline(&pts, closed, true)).is_some_and(|es| !es.is_empty());
    };
    commit_mesh(ctx, label, |m| if pie { m.add_pie(&local) } else { m.add_curve(&local) })
        .is_some_and(|es| !es.is_empty())
}

/// "Tangent at vertex" direction (world, unit) at the active-context vertex at world point `p`:
/// leaving an arc ending there, or continuing the single edge that ends there.
pub fn tangent_at_point(doc: &Document, p: DVec3) -> Option<DVec3> {
    guarded(|| {
        let mesh = doc.active_mesh()?;
        let xf = context_xf(doc);
        let lp = xf.inverse().transform_point3(p);
        let tol = 1e-3_f64.max(lp.length() * 1e-9);
        let (v, _) = mesh.vertices.iter().find(|(_, v)| v.pos.distance(lp) <= tol)?;
        let t = mesh.tangent_at_vertex(v)?;
        xf.transform_vector3(t).try_normalize()
    })
    .ok()
    .flatten()
}

/// Run a fallible op on the active mesh as one undo step (restored on error).
pub fn try_commit_mesh<R>(
    ctx: &mut ToolContext,
    label: &str,
    f: impl FnOnce(&mut Mesh) -> varstotas_kernel::OpResult<R>,
) -> Option<R> {
    let path = ctx.doc.active_path.clone();
    ctx.doc.try_commit(label, |m| match m.mesh_at_mut(&path) {
        Some(mesh) => f(mesh),
        None => Err(varstotas_kernel::OpError::NotFound),
    })
}

/// Undo the last operation (for "edit last operation" via the VCB).
pub fn undo_last(ctx: &mut ToolContext) -> bool {
    ctx.doc.undo_last_commit().is_some()
}

// ---------------------------------------------------------------------------
// Picking in the active context
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum Hover {
    Face { face: FaceId, point: DVec3, plane: Plane },
    Edge { edge: EdgeId, a: DVec3, b: DVec3 },
    Instance { id: InstanceId, point: DVec3 },
}

impl Hover {
    pub fn entity(&self) -> EntityRef {
        match self {
            Hover::Face { face, .. } => EntityRef::Face(*face),
            Hover::Edge { edge, .. } => EntityRef::Edge(*edge),
            Hover::Instance { id, .. } => EntityRef::Instance(*id),
        }
    }
}

fn starts_with(path: &[InstanceId], prefix: &[InstanceId]) -> bool {
    path.len() >= prefix.len() && path[..prefix.len()] == *prefix
}

/// Face (or child instance) of the active context under the pointer.
pub fn pick_face(ctx: &ToolContext) -> Option<Hover> {
    let hit = ctx.pick()?;
    let active = &ctx.doc.active_path;
    if !starts_with(&hit.path, active) {
        return None;
    }
    if hit.path.len() > active.len() {
        let id = hit.path[active.len()];
        if ctx.doc.model.entity_locked(active, EntityRef::Instance(id)) { return None; }
        return Some(Hover::Instance { id, point: hit.point });
    }
    let mesh = ctx.doc.model.mesh_at(active)?;
    let f = mesh.faces.get(hit.face)?;
    if ctx.doc.model.entity_locked(active, EntityRef::Face(hit.face)) { return None; }
    let xf = context_xf(ctx.doc);
    let n = xf.inverse().transpose().transform_vector3(f.normal).normalize_or(f.normal);
    Some(Hover::Face { face: hit.face, point: hit.point, plane: Plane::new(hit.point, n) })
}

/// Screen projections of the active context's edges, reused while the model, the camera
/// and the context are unchanged (projecting every edge per pointer event is the dominant
/// hover cost on big models).
struct EdgeProjections {
    key: (u64, inference::geo::CamKey, ContextPath),
    /// (edge, world a, world b, screen a, screen b) for visible edges in front of the camera.
    edges: Vec<(EdgeId, DVec3, DVec3, DVec2, DVec2)>,
}

thread_local! {
    static EDGE_PROJ: std::cell::RefCell<Option<EdgeProjections>> = const { std::cell::RefCell::new(None) };
}

/// Edges of the active context whose screen projection is within `radius` px of `c`,
/// nearest first: (screen distance, edge, world a, world b).
fn edges_near(ctx: &ToolContext, c: DVec2, radius: f64) -> Vec<(f64, EdgeId, DVec3, DVec3)> {
    let vp = ctx.viewport_d();
    let key = (ctx.doc.model_version, inference::geo::CamKey::of(ctx.camera, vp), ctx.doc.active_path.clone());
    EDGE_PROJ.with_borrow_mut(|cache| {
        if cache.as_ref().is_none_or(|p| p.key != key) {
            let mut edges = Vec::new();
            if let Some(mesh) = ctx.doc.model.mesh_at(&key.2) {
                let xf = context_xf(ctx.doc);
                edges.reserve(mesh.edges.len());
                for (id, e) in mesh.edges.iter() {
                    if e.hidden || ctx.doc.model.entity_locked(&key.2, EntityRef::Edge(id)) {
                        continue;
                    }
                    let (a, b) = mesh.edge_segment(id);
                    let (a, b) = (xf.transform_point3(a), xf.transform_point3(b));
                    if let Some((sa, sb)) = inference::geo::project_segment(ctx.camera, vp, a, b) {
                        edges.push((id, a, b, sa, sb));
                    }
                }
            }
            *cache = Some(EdgeProjections { key, edges });
        }
        let proj = cache.as_ref().map(|p| p.edges.as_slice()).unwrap_or_default();
        let mut cands: Vec<(f64, EdgeId, DVec3, DVec3)> = proj
            .iter()
            .filter_map(|(id, a, b, sa, sb)| {
                // Cheap bounding-box reject before the exact distance.
                if c.x < sa.x.min(sb.x) - radius
                    || c.x > sa.x.max(sb.x) + radius
                    || c.y < sa.y.min(sb.y) - radius
                    || c.y > sa.y.max(sb.y) + radius
                {
                    return None;
                }
                let (d, _) = inference::geo::dist_point_seg2(c, *sa, *sb);
                (d <= radius).then_some((d, *id, *a, *b))
            })
            .collect();
        cands.sort_by(|x, y| x.0.total_cmp(&y.0));
        cands
    })
}

/// Edge of the active context within `radius` px of the pointer (not occluded).
///
/// Candidates are tried nearest-first; the 3D point is the perspective-correct closest
/// point on the edge to the pick ray, and occlusion is tested with a ray aimed at that
/// point, ignoring the faces bounding the edge (so silhouette/boundary edges win over
/// their own faces).
pub fn pick_edge_at(ctx: &ToolContext, pos: Pos2, radius: f32) -> Option<Hover> {
    let active = ctx.doc.active_path.clone();
    let mesh = ctx.doc.model.mesh_at(&active)?;
    let c = DVec2::new(pos.x as f64, pos.y as f64);
    let ray = ctx.ray_at(pos);
    let cands = edges_near(ctx, c, radius as f64);
    if cands.is_empty() {
        return None;
    }
    // The face under the cursor answers most occlusion questions without a raycast per
    // candidate (each one is a scan of the whole model on big models).
    let cursor_hit = crate::document::cached_raycast(&ctx.doc.model, ctx.doc.model_version, &ray);
    let mut full_tests = 0;
    for (_, edge, a, b) in cands.into_iter().take(8) {
        let p = inference::geo::closest_on_seg_to_ray(a, b, ray.origin, ray.dir);
        match quick_occlusion(ctx, mesh, &active, edge, p, cursor_hit.as_ref()) {
            Some(false) => return Some(Hover::Edge { edge, a, b }),
            Some(true) => continue,
            None => {}
        }
        // Undecided (near a silhouette): exact test, bounded per event.
        if full_tests >= 3 {
            continue;
        }
        full_tests += 1;
        if !edge_point_occluded(ctx, &active, edge, p) {
            return Some(Hover::Edge { edge, a, b });
        }
    }
    None
}

/// Occlusion of edge point `p` decided from the face under the cursor, when possible:
/// - the cursor is on one of the edge's own faces → visible;
/// - the ray towards `p` crosses the cursor's face in front of `p` → occluded;
/// - nothing under the cursor in front of `p`, and the cursor's face (if any) is not crossed
///   by the ray towards `p` → undecided (`None`): another face might still cover `p`.
fn quick_occlusion(
    ctx: &ToolContext,
    mesh: &Mesh,
    active: &ContextPath,
    edge: EdgeId,
    p: DVec3,
    hit: Option<&varstotas_kernel::ModelHit>,
) -> Option<bool> {
    let hit = hit?;
    if hit.path == *active && mesh.edge_faces(edge).contains(&hit.face) {
        return Some(false);
    }
    let sp = ctx.to_screen(p)?;
    let ray = ctx.ray_at(sp);
    let depth = (p - ray.origin).dot(ray.dir);
    let vp = ctx.viewport_d();
    let tol = (ctx.camera.pixel_size_at(p, vp) * 2.0).max(depth.abs() * 1e-6).max(1e-3);
    // Does the ray towards p cross the cursor's face in front of p?
    let hmesh = ctx.doc.model.mesh_at(&hit.path)?;
    let f = hmesh.faces.get(hit.face)?;
    let inv = ctx.doc.model.transform_at(&hit.path).inverse();
    let (lo, ld) = (inv.transform_point3(ray.origin), inv.transform_vector3(ray.dir));
    let v0 = hmesh.vertices.get(*f.outer.first()?)?.pos;
    let den = ld.dot(f.normal);
    if den.abs() > 1e-12 {
        let t = (v0 - lo).dot(f.normal) / den;
        if t > 0.0 && t < depth - tol && hmesh.face_point_loc(hit.face, lo + ld * t) != varstotas_kernel::geom::PointLoc::Outside {
            return Some(true);
        }
    }
    None
}
/// True when a face other than those bounding `edge` lies in front of world point `p`.
fn edge_point_occluded(ctx: &ToolContext, active: &ContextPath, edge: EdgeId, p: DVec3) -> bool {
    let Some(sp) = ctx.to_screen(p) else { return true };
    let ray = ctx.ray_at(sp);
    let depth = (p - ray.origin).dot(ray.dir);
    let vp = ctx.viewport_d();
    let tol = (ctx.camera.pixel_size_at(p, vp) * 2.0).max(depth.abs() * 1e-6).max(1e-3);
    let Some(hit) = guarded(|| ctx.doc.model.raycast(&ray)).ok().flatten() else { return false };
    if hit.t >= depth - tol {
        return false;
    }
    let own = hit.path == *active
        && ctx.doc.model.mesh_at(active).is_some_and(|m| m.edge_faces(edge).contains(&hit.face));
    !own
}

/// Edge first, then face/instance.
pub fn pick_entity(ctx: &ToolContext) -> Option<Hover> {
    pick_edge_at(ctx, ctx.pointer.pos, inference::EDGE_PICK_PX as f32)
        .or_else(|| pick_face(ctx))
        .or_else(|| pick_instance_wire(ctx, inference::EDGE_PICK_PX))
}

/// Screen projections of the child instances' wire (see `Model::nested_wire_world`), cached
/// like [`EdgeProjections`].
struct WireProjections {
    key: (u64, inference::geo::CamKey, ContextPath),
    segs: Vec<(InstanceId, DVec3, DVec3, DVec2, DVec2)>,
}

thread_local! {
    static WIRE_PROJ: std::cell::RefCell<Option<WireProjections>> = const { std::cell::RefCell::new(None) };
}

/// Child instance of the active context whose edges, dimensions or guide points are within
/// `radius` px of the pointer: groups without faces (edges only, guides / dimensions only)
/// are not hit by the face raycast.
pub fn pick_instance_wire(ctx: &ToolContext, radius: f64) -> Option<Hover> {
    let vp = ctx.viewport_d();
    let c = DVec2::new(ctx.pointer.pos.x as f64, ctx.pointer.pos.y as f64);
    let key = (ctx.doc.model_version, inference::geo::CamKey::of(ctx.camera, vp), ctx.doc.active_path.clone());
    let best = WIRE_PROJ.with_borrow_mut(|cache| {
        if cache.as_ref().is_none_or(|p| p.key != key) {
            let segs = ctx
                .doc
                .model
                .nested_wire_world(&key.2)
                .into_iter()
                .filter_map(|(id, a, b)| {
                    if ctx.doc.model.entity_locked(&key.2, EntityRef::Instance(id)) { return None; }
                    let (sa, sb) = inference::geo::project_segment(ctx.camera, vp, a, b)?;
                    Some((id, a, b, sa, sb))
                })
                .collect();
            *cache = Some(WireProjections { key, segs });
        }
        let segs = cache.as_ref().map(|p| p.segs.as_slice()).unwrap_or_default();
        segs.iter()
            .filter_map(|(id, a, b, sa, sb)| {
                let (d, _) = inference::geo::dist_point_seg2(c, *sa, *sb);
                (d <= radius).then_some((d, *id, *a, *b))
            })
            .min_by(|x, y| x.0.total_cmp(&y.0))
    })?;
    let (_, id, a, b) = best;
    let ray = ctx.pointer.ray;
    let point = inference::geo::closest_on_seg_to_ray(a, b, ray.origin, ray.dir);
    Some(Hover::Instance { id, point })
}

// ---------------------------------------------------------------------------
// VCB parsing
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ArrayInput {
    /// `5x` / `x5` / `*5`: N copies at multiples of the last offset.
    Multiply(u32),
    /// `/5`: N copies dividing the last offset.
    Divide(u32),
}

pub fn parse_array(s: &str) -> Option<ArrayInput> {
    let s = s.trim().to_ascii_lowercase();
    let s = s.replace(' ', "");
    let num = |t: &str| t.parse::<u32>().ok().filter(|n| *n >= 1 && *n <= 1000);
    if let Some(r) = s.strip_prefix('/') {
        return num(r).map(ArrayInput::Divide);
    }
    if let Some(r) = s.strip_suffix('x').or_else(|| s.strip_prefix('x')).or_else(|| s.strip_prefix('*')) {
        return num(r).map(ArrayInput::Multiply);
    }
    None
}

/// `24s` → 24 segments/sides.
pub fn parse_segments(s: &str) -> Option<u32> {
    let s = s.trim().to_ascii_lowercase();
    let n = s.strip_suffix('s')?.trim().parse::<u32>().ok()?;
    (3..=360).contains(&n).then_some(n)
}

/// Format an angle for the VCB/tooltips: `90°`, `22.5°`, `-7.25°` (no trailing zeros).
pub fn fmt_angle(deg: f64) -> String {
    let s = format!("{deg:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    let s = if s == "-0" { "0" } else { s };
    format!("{s}°")
}

/// Angle in degrees: `45`, `-30`, `22.5`, `22,5`, `45deg`, `45°`.
pub fn parse_angle(s: &str) -> Option<f64> {
    let s = s.trim().to_ascii_lowercase();
    let s = s.trim_end_matches('°').trim_end_matches("deg").trim().replace(',', ".");
    let v = s.parse::<f64>().ok()?;
    v.is_finite().then_some(v)
}

/// The world size of a screen pixel at `p`.
pub fn px_at(cam: &varstotas_render::Camera, viewport: egui::Vec2, p: DVec3) -> f64 {
    cam.pixel_size_at(p, DVec2::new(viewport.x as f64, viewport.y as f64))
}

pub fn parse_len(unit: LengthUnit, s: &str) -> Option<f64> {
    units::parse_length(s, unit).filter(|v| v.is_finite())
}

pub fn parse_lens(unit: LengthUnit, s: &str) -> Option<Vec<f64>> {
    units::parse_lengths(s, unit).filter(|v| v.iter().all(|x| x.is_finite()))
}

pub fn fmt_len(model: &Model, mm: f64) -> String {
    units::format_length(mm, model.unit)
}

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------

/// In-plane basis (u, v) with u preferring the red axis (then green).
pub fn plane_axes(n: DVec3) -> (DVec3, DVec3) {
    let ax = crate::axes::current();
    let n = n.normalize_or(ax.z);
    let mut u = ax.x - n * ax.x.dot(n);
    if u.length_squared() < 1e-6 {
        u = ax.y - n * ax.y.dot(n);
    }
    let u = u.normalize();
    (u, n.cross(u))
}

/// Points of a regular polygon/circle around `c` in the plane with normal `n`.
/// `start` is the in-plane direction of the first vertex.
pub fn circle_points(c: DVec3, n: DVec3, start: DVec3, r: f64, segs: u32) -> Vec<DVec3> {
    let n = n.normalize_or(DVec3::Z);
    let u = (start - n * start.dot(n)).normalize_or(plane_axes(n).0);
    let v = n.cross(u);
    (0..segs)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / segs as f64;
            c + (u * a.cos() + v * a.sin()) * r
        })
        .collect()
}

/// Two-point arc from `a` to `b` with sagitta `bulge` measured along the in-plane unit
/// direction `side` (perpendicular to the chord). Returns `segs + 1` points from a to b.
pub fn arc_points(a: DVec3, b: DVec3, side: DVec3, bulge: f64, segs: u32) -> Vec<DVec3> {
    let segs = segs.max(1);
    let chord = b - a;
    let c = chord.length();
    if c < 1e-9 || bulge.abs() < 1e-9 {
        return (0..=segs).map(|i| a.lerp(b, i as f64 / segs as f64)).collect();
    }
    let m = (a + b) * 0.5;
    let side = (side - chord * side.dot(chord) / (c * c)).normalize_or(DVec3::Z);
    let h = bulge;
    // Radius from chord and sagitta; center lies on the perpendicular bisector.
    let r = (c * c / 4.0 + h * h) / (2.0 * h.abs());
    let center = m + side * (h.signum() * (h.abs() - r));
    let u = (a - center) / r;
    let w = (b - center) / r;
    let n = u.cross(w);
    // Rotation axis such that sweeping from a passes the bulge apex.
    let apex = m + side * h;
    let ap = ((apex - center) / r).normalize_or(u);
    let axis_n = if n.length_squared() > 1e-18 { n.normalize() } else { u.cross(side).normalize_or(DVec3::Z) };
    let ang = |x: DVec3| {
        let s = u.cross(x).dot(axis_n);
        s.atan2(u.dot(x))
    };
    let mut total = ang(w);
    let apex_ang = ang(ap);
    // Choose sweep direction that contains the apex.
    let norm = |x: f64| x.rem_euclid(std::f64::consts::TAU);
    let (t, ap_a) = (norm(total), norm(apex_ang));
    if ap_a <= t {
        total = t;
    } else {
        total = t - std::f64::consts::TAU;
    }
    let v = axis_n.cross(u);
    (0..=segs)
        .map(|i| {
            if i == 0 {
                return a;
            }
            if i == segs {
                return b;
            }
            let t = total * i as f64 / segs as f64;
            center + (u * t.cos() + v * t.sin()) * r
        })
        .collect()
}

/// Arc around `c` in the plane with normal `n`: starts at `c + start_dir * r` and sweeps
/// `sweep` radians (positive = counter-clockwise about `n`). Returns `segs + 1` points.
pub fn arc_points_center(c: DVec3, n: DVec3, start_dir: DVec3, r: f64, sweep: f64, segs: u32) -> Vec<DVec3> {
    let segs = segs.max(1);
    let n = n.normalize_or(DVec3::Z);
    let u = (start_dir - n * start_dir.dot(n)).normalize_or(plane_axes(n).0);
    let v = n.cross(u);
    (0..=segs)
        .map(|i| {
            let a = sweep * i as f64 / segs as f64;
            c + (u * a.cos() + v * a.sin()) * r
        })
        .collect()
}

/// Sagitta (bulge) of the minor arc with chord length `chord` and radius `r`
/// (None when the radius is shorter than half the chord).
pub fn bulge_for_radius(chord: f64, r: f64) -> Option<f64> {
    let h2 = r * r - chord * chord / 4.0;
    (h2 >= -1e-9 && r > 0.0).then(|| r - h2.max(0.0).sqrt())
}

/// Bulge of the arc over chord `a → b` that leaves `a` along the unit tangent `t`, and the
/// in-plane side direction it bulges to. None when `t` is parallel to the chord.
pub fn tangent_bulge(a: DVec3, b: DVec3, t: DVec3) -> Option<(f64, DVec3)> {
    let chord = b - a;
    let len = chord.length();
    if len < 1e-9 {
        return None;
    }
    let cd = chord / len;
    let perp = t - cd * t.dot(cd);
    if perp.length() < 1e-6 {
        return None;
    }
    let side = perp.normalize();
    // Angle between the tangent and the chord; the sweep is twice that.
    let theta = t.dot(cd).clamp(-1.0, 1.0).acos();
    Some(((len * 0.5) * (theta * 0.5).tan(), side))
}

/// `500r` → 500 (radius, in model units).
pub fn parse_radius(unit: LengthUnit, s: &str) -> Option<f64> {
    let t = s.trim();
    let t = t.strip_suffix('r').or_else(|| t.strip_suffix('R'))?;
    parse_len(unit, t).filter(|r| *r > 0.0)
}

/// Rotation by `deg` degrees about the axis through `center` with direction `axis`.
pub fn rotation_about(center: DVec3, axis: DVec3, deg: f64) -> DMat4 {
    DMat4::from_translation(center)
        * DMat4::from_axis_angle(axis.normalize_or(DVec3::Z), deg.to_radians())
        * DMat4::from_translation(-center)
}

/// Signed angle (degrees) from `a` to `b` around `n`.
pub fn signed_angle(a: DVec3, b: DVec3, n: DVec3) -> f64 {
    let s = a.cross(b).dot(n.normalize_or(DVec3::Z));
    s.atan2(a.dot(b)).to_degrees()
}

/// Snap to multiples of `step` when within `tol` degrees.
pub fn snap_angle(deg: f64, step: f64, tol: f64) -> (f64, bool) {
    let k = (deg / step).round() * step;
    if (deg - k).abs() <= tol { (k, true) } else { (deg, false) }
}

/// World-space edge segments that move/stretch when `ents` are transformed by `t_world`
/// (in the active context). Instances contribute their definition's complete visible edges.
pub fn transformed_preview(doc: &Document, ents: &[EntityRef], t_world: &DMat4) -> Vec<(DVec3, DVec3)> {
    guarded(|| {
        let mut out = Vec::new();
        let Some(mesh) = doc.active_mesh() else { return out };
        let xf = context_xf(doc);
        let vs = mesh.vertices_of(ents);
        let tl = xf.inverse() * *t_world * xf;
        let mut seen: HashSet<EdgeId> = HashSet::new();
        for v in &vs {
            for e in mesh.vertex_edges(*v) {
                if !seen.insert(e) {
                    continue;
                }
                let ed = &mesh.edges[e];
                let p = |vid| {
                    let pos = mesh.vertices[vid].pos;
                    let pos = if vs.contains(&vid) { tl.transform_point3(pos) } else { pos };
                    xf.transform_point3(pos)
                };
                out.push((p(ed.v[0]), p(ed.v[1])));
            }
        }
        for ent in ents {
            if let EntityRef::Instance(id) = ent
                && let Some(inst) = mesh.instances.get(*id)
            {
                let m = *t_world * xf * inst.transform;
                instance_edges(&doc.model, inst.def, m, &mut out, 0);
            }
        }
        out
    })
    .unwrap_or_default()
}

fn instance_edges(model: &Model, def: varstotas_kernel::DefId, m: DMat4, out: &mut Vec<(DVec3, DVec3)>, depth: usize) {
    if depth > 64 {
        return;
    }
    let Some(d) = model.defs.get(def) else { return };
    for (id, e) in d.mesh.edges.iter() {
        if e.hidden {
            continue;
        }
        let (a, b) = d.mesh.edge_segment(id);
        out.push((m.transform_point3(a), m.transform_point3(b)));
    }
    for inst in d.mesh.instances.values() {
        if !inst.hidden {instance_edges(model, inst.def, m * inst.transform, out, depth + 1);}
    }
}

/// World bounds of entities in the active context.
pub fn entities_bounds(doc: &Document, ents: &[EntityRef]) -> Option<varstotas_kernel::Aabb> {
    guarded(|| {
        let mesh = doc.active_mesh()?;
        let xf = context_xf(doc);
        let mut pts: Vec<DVec3> = mesh.vertices_of(ents).iter().map(|v| xf.transform_point3(mesh.vertices[*v].pos)).collect();
        for e in ents {
            if let EntityRef::Instance(id) = e
                && let Some(inst) = mesh.instances.get(*id)
                && let Some(b) = doc.model.def_bounds(inst.def)
            {
                pts.extend(b.transformed(&(xf * inst.transform)).corners());
            }
        }
        varstotas_kernel::Aabb::from_points(pts)
    })
    .ok()
    .flatten()
}

/// Apply a world transform to entities of the active context (vertices stretch, instances move).
/// With `copy`, duplicates instead and returns the new entities.
pub fn apply_transform(mesh: &mut Mesh, ents: &[EntityRef], t_local: &DMat4, copy: bool) -> Vec<EntityRef> {
    if copy {
        return mesh.duplicate(ents, t_local);
    }
    let vs = mesh.vertices_of(ents);
    if !vs.is_empty() {
        mesh.transform_vertices(&vs, t_local);
    }
    for e in ents {
        if let EntityRef::Instance(id) = e
            && let Some(inst) = mesh.instances.get_mut(*id)
        {
            inst.transform = *t_local * inst.transform;
        }
    }
    ents.to_vec()
}

// ---------------------------------------------------------------------------
// Overlay helpers
// ---------------------------------------------------------------------------

pub fn line(out: &mut Overlay3d, a: DVec3, b: DVec3, color: Color32, width: f32) {
    push_line(out, a, b, color, width, false, true);
}

pub fn dashed(out: &mut Overlay3d, a: DVec3, b: DVec3, color: Color32, width: f32) {
    push_line(out, a, b, color, width, true, true);
}

pub fn polyline(out: &mut Overlay3d, pts: &[DVec3], closed: bool, color: Color32, width: f32) {
    let n = pts.len();
    if n < 2 {
        return;
    }
    let segs = if closed { n } else { n - 1 };
    for i in 0..segs {
        line(out, pts[i], pts[(i + 1) % n], color, width);
    }
}

/// Translucent fill of a convex-ish planar polygon (fan triangulation).
pub fn fill_polygon(out: &mut Overlay3d, pts: &[DVec3], color: Color32) {
    if pts.len() < 3 {
        return;
    }
    let mut tris = Vec::new();
    for i in 1..pts.len() - 1 {
        tris.push(pts[0].as_vec3().to_array());
        tris.push(pts[i].as_vec3().to_array());
        tris.push(pts[i + 1].as_vec3().to_array());
    }
    out.polygons.push(OverlayPolygon { tris, color: inference::rgba(color), on_top: false });
}

/// Highlight a face of the active context (world-space triangles).
pub fn highlight_face(view: &OverlayView, face: FaceId, color: Color32, out: &mut Overlay3d) {
    let tris = guarded(|| {
        let mesh = view.doc.active_mesh()?;
        if !mesh.faces.contains_key(face) {
            return None;
        }
        let xf = context_xf(view.doc);
        let tris = varstotas_kernel::tessellate::triangulate_face(mesh, face);
        let mut out = Vec::with_capacity(tris.len() * 3);
        for t in tris {
            for p in t {
                out.push(xf.transform_point3(p).as_vec3().to_array());
            }
        }
        Some(out)
    })
    .ok()
    .flatten();
    if let Some(tris) = tris {
        out.polygons.push(OverlayPolygon { tris, color: inference::rgba(color), on_top: false });
    }
}

/// World-space outer loop + holes of a face in the active context.
pub fn face_loops_world(doc: &Document, face: FaceId) -> Vec<Vec<DVec3>> {
    guarded(|| {
        let Some(mesh) = doc.active_mesh() else { return Vec::new() };
        let Some(f) = mesh.faces.get(face) else { return Vec::new() };
        let xf = context_xf(doc);
        f.loops().map(|l| l.iter().map(|v| xf.transform_point3(mesh.vertices[*v].pos)).collect()).collect()
    })
    .unwrap_or_default()
}

pub fn highlight_hover(view: &OverlayView, h: &Hover, out3d: &mut Overlay3d) {
    match h {
        Hover::Face { face, .. } => highlight_face(view, *face, HOVER_FACE, out3d),
        Hover::Edge { a, b, .. } => line(out3d, *a, *b, SELECT_BLUE, 3.0),
        Hover::Instance { id, .. } => {
            for (a, b) in transformed_preview(view.doc, &[EntityRef::Instance(*id)], &DMat4::IDENTITY) {
                line(out3d, a, b, SELECT_BLUE, 2.0);
            }
        }
    }
}

/// Label at a world point.
pub fn label(out2d: &mut Vec<Marker2d>, at: DVec3, text: impl Into<String>) {
    out2d.push(Marker2d::Label { at, text: text.into() });
}

/// Axis colour for a plane normal (protractor / circle preview).
pub fn normal_color(n: DVec3) -> Color32 {
    use crate::inference::{InferenceKind, axis_kind_of};
    axis_kind_of(n).map(InferenceKind::color).unwrap_or(Color32::from_rgb(0x80, 0x80, 0x80))
}

/// Pick radius for guide points (px).
pub const GUIDE_POINT_PICK_PX: f64 = 8.0;

/// How far guide lines are drawn either side of the point nearest the camera target.
pub fn guide_reach(cam: &varstotas_render::Camera) -> f64 {
    (cam.eye - cam.target).length().max(1000.0) * 20.0
}

/// Visible segment used to draw / box-select an (infinite) guide line.
pub fn guide_segment(cam: &varstotas_render::Camera, g: &GuideLine) -> Option<(DVec3, DVec3)> {
    let d = g.dir.normalize_or_zero();
    if d == DVec3::ZERO || !g.origin.is_finite() {
        return None;
    }
    let reach = guide_reach(cam);
    let mid = g.origin + d * (cam.target - g.origin).dot(d);
    Some((mid - d * reach, mid + d * reach))
}

/// Guide under screen position `pos` (viewport-local): guide points within
/// [`GUIDE_POINT_PICK_PX`] first, then guide lines within the edge pick radius, nearest first.
/// Guides hidden behind faces are skipped; nothing is picked while guides are hidden.
pub fn pick_guide_at(doc: &Document, cam: &varstotas_render::Camera, viewport: egui::Vec2, pos: Pos2) -> Option<GuideRef> {
    let model = &doc.model;
    if model.guides_hidden {
        return None;
    }
    // Only guides of the active context are picked (like its entities).
    let guides = model.guides_in(&doc.active_path);
    if guides.is_empty() {
        return None;
    }
    let vp = DVec2::new(viewport.x as f64, viewport.y as f64);
    let c = DVec2::new(pos.x as f64, pos.y as f64);
    let ray = crate::nav::ray_at(cam, pos, viewport);
    let hit = crate::document::cached_raycast(model, doc.model_version, &ray);
    let visible = |p: DVec3| {
        let depth = (p - ray.origin).dot(ray.dir);
        let tol = (cam.pixel_size_at(p, vp) * 3.0).max(depth.abs() * 1e-6).max(1e-3);
        hit.as_ref().is_none_or(|h| h.t >= depth - tol)
    };
    let mut best: Option<(f64, GuideRef)> = None;
    for (g, geom) in &guides {
        let GuideGeom::Point(p) = *geom else { continue };
        let Some(s) = cam.world_to_screen(p, vp) else { continue };
        let d = s.distance(c);
        if d <= GUIDE_POINT_PICK_PX && best.is_none_or(|b| d < b.0) && visible(p) {
            best = Some((d, *g));
        }
    }
    if best.is_some() {
        return best.map(|b| b.1);
    }
    for (id, geom) in &guides {
        let GuideGeom::Line(g) = geom else { continue };
        let d = g.dir.normalize_or_zero();
        if d == DVec3::ZERO {
            continue;
        }
        let Some(p) = inference::geo::closest_on_line_to_ray(g.origin, d, ray.origin, ray.dir) else { continue };
        if (p - ray.origin).dot(ray.dir) <= 0.0 {
            continue;
        }
        let Some(s) = cam.world_to_screen(p, vp) else { continue };
        let dist = s.distance(c);
        if dist <= inference::EDGE_PICK_PX && best.is_none_or(|b| dist < b.0) && visible(p) {
            best = Some((dist, *id));
        }
    }
    best.map(|b| b.1)
}

/// [`pick_guide_at`] for the tool pointer.
pub fn pick_guide(ctx: &ToolContext) -> Option<GuideRef> {
    pick_guide_at(ctx.doc, ctx.camera, ctx.viewport, ctx.pointer.pos)
}

/// Draw one guide (line or point) with `t` applied, in `color`.
pub fn draw_guide(
    view: &OverlayView,
    g: GuideRef,
    t: &DMat4,
    color: Color32,
    width: f32,
    out3d: &mut Overlay3d,
    out2d: &mut Vec<Marker2d>,
) {
    match view.doc.model.guide_geom_in(&view.doc.active_path, g).map(|x| x.transformed(t)) {
        Some(GuideGeom::Line(gl)) => {
            if let Some((a, b)) = guide_segment(view.camera, &gl) {
                push_line(out3d, a, b, color, width, true, false);
            }
        }
        Some(GuideGeom::Point(p)) => {
            out2d.push(Marker2d::Point { at: p, shape: crate::overlay::MarkerShape::Cross, color });
        }
        None => {}
    }
}

/// Draw guide lines and guide points of the model (dashed; selected ones blue) as 3D overlay
/// lines, so they also appear in PNG exports. Called by the app.
pub fn draw_guides(view: &OverlayView, out3d: &mut Overlay3d) {
    let model = &view.doc.model;
    if model.guides_hidden {
        return;
    }
    // Guides of the open context and of the contexts around it (guides inside closed groups
    // / components are only shown while editing them).
    let path = &view.doc.active_path;
    let sel = &view.doc.guide_sel;
    let vp = DVec2::new(view.viewport.x as f64, view.viewport.y as f64);
    for (depth, g, geom) in model.visible_guides(path) {
        let selected = depth == path.len() && sel.contains(&g);
        match geom {
            GuideGeom::Line(l) => out3d.add_guide_line(view.camera, l.origin, l.dir, selected),
            GuideGeom::Point(p) => out3d.add_guide_point(view.camera, vp, p, selected),
        }
    }
}

// ---------------------------------------------------------------------------
// Dimensions
// ---------------------------------------------------------------------------

/// Colour of selected dimensions.
pub const DIM_SELECTED: [f32; 4] = [0.0, 0.31, 1.0, 1.0];

/// Look of a dimension (selected ones blue).
pub fn dim_style(selected: bool) -> varstotas_render::DimensionStyle {
    let mut s = varstotas_render::DimensionStyle::default();
    if selected {
        s.color = DIM_SELECTED;
        s.text_color = DIM_SELECTED;
        s.width_px = 2.0;
    }
    s
}

/// Look of a text note (selected ones blue).
pub fn note_style(selected: bool) -> varstotas_render::LeaderStyle {
    let mut s = varstotas_render::LeaderStyle::default();
    if selected {
        s.color = DIM_SELECTED;
        s.text_color = DIM_SELECTED;
        s.border = Some(DIM_SELECTED);
        s.width_px = 2.0;
    }
    s
}

/// Draw a dimension or a text note.
pub fn add_dim_geom(out3d: &mut Overlay3d, g: &varstotas_kernel::DimGeom, unit: varstotas_kernel::LengthUnit, selected: bool) {
    if g.leader {
        out3d.add_leader_text(g.a, g.text_pos(), g.label(unit), &note_style(selected));
    } else {
        out3d.add_linear_dimension(g.a, g.b, g.offset, g.label(unit), &dim_style(selected));
    }
}

/// Which part of a dimension is under the cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DimPart {
    Text,
    Line,
}

/// Approximate screen box of a dimension's text: centre and half extents along / across the
/// line (the renderer puts it centred just above the dimension line).
fn dim_text_box(cam: &varstotas_render::Camera, vp: DVec2, g: &varstotas_kernel::DimGeom, label: &str) -> Option<(DVec2, DVec2, f64)> {
    let (p, q) = g.line();
    let (sp, sq) = (cam.world_to_screen(p, vp)?, cam.world_to_screen(q, vp)?);
    let mut along = (sq - sp).normalize_or(DVec2::X);
    if along.x < 0.0 {
        along = -along;
    }
    // "Up" on screen relative to the (readable) text.
    let mut up = DVec2::new(along.y, -along.x);
    if up.y > 0.0 {
        up = -up;
    }
    let half_w = 3.6 * label.chars().count() as f64 + 4.0;
    let centre = (sp + sq) * 0.5 + up * 10.0;
    Some((centre, along, half_w))
}

/// Dimension of the active context under `pos`: its text (generous box) or its dimension /
/// extension lines (edge pick radius). Nearest wins; text first.
pub fn pick_dim_at(
    doc: &Document,
    cam: &varstotas_render::Camera,
    viewport: egui::Vec2,
    pos: Pos2,
) -> Option<(varstotas_kernel::DimId, DimPart)> {
    let model = &doc.model;
    let dims = model.dimensions_in(&doc.active_path);
    if dims.is_empty() {
        return None;
    }
    let vp = DVec2::new(viewport.x as f64, viewport.y as f64);
    let c = DVec2::new(pos.x as f64, pos.y as f64);
    let mut best: Option<(f64, varstotas_kernel::DimId, DimPart)> = None;
    for (id, g) in &dims {
        let label = g.label(model.unit);
        if g.leader {
            // Text box centred on the text position; the leader as a line.
            if let Some(sc) = cam.world_to_screen(g.text_pos(), vp) {
                let d = c - sc;
                let half_w = 3.6 * label.lines().map(|l| l.chars().count()).max().unwrap_or(0) as f64 + 6.0;
                let half_h = 9.0 * label.lines().count().max(1) as f64;
                if d.x.abs() <= half_w && d.y.abs() <= half_h {
                    let dist = d.length() * 0.5;
                    if best.is_none_or(|b| dist < b.0) {
                        best = Some((dist, *id, DimPart::Text));
                    }
                }
            }
            if let (Some(sa), Some(sb)) = (cam.world_to_screen(g.a, vp), cam.world_to_screen(g.text_pos(), vp)) {
                let dist = inference::geo::dist_point_seg2(c, sa, sb).0;
                if g.offset != DVec3::ZERO && dist <= inference::EDGE_PICK_PX && best.is_none_or(|bb| dist < bb.0) {
                    best = Some((dist, *id, DimPart::Line));
                }
            }
            continue;
        }
        if let Some((centre, along, half_w)) = dim_text_box(cam, vp, g, &label) {
            let d = c - centre;
            let (x, y) = (d.dot(along).abs(), d.perp_dot(along).abs());
            if x <= half_w && y <= 10.0 {
                let dist = d.length() * 0.5;
                if best.is_none_or(|b| dist < b.0) {
                    best = Some((dist, *id, DimPart::Text));
                }
            }
        }
        let (p, q) = g.line();
        for (a, b) in [(p, q), (g.a, p), (g.b, q)] {
            let (Some(sa), Some(sb)) = (cam.world_to_screen(a, vp), cam.world_to_screen(b, vp)) else { continue };
            let dist = inference::geo::dist_point_seg2(c, sa, sb).0;
            if dist <= inference::EDGE_PICK_PX && best.is_none_or(|bb| dist < bb.0) {
                best = Some((dist, *id, DimPart::Line));
            }
        }
    }
    best.map(|b| (b.1, b.2))
}

/// [`pick_dim_at`] for the tool pointer.
pub fn pick_dim(ctx: &ToolContext) -> Option<(varstotas_kernel::DimId, DimPart)> {
    pick_dim_at(ctx.doc, ctx.camera, ctx.viewport, ctx.pointer.pos)
}

/// Draw every dimension of the model (inside groups too); selected ones of the active
/// context in blue. 3D overlay, so they also appear in PNG exports. Called by the app.
pub fn draw_dims(view: &OverlayView, out3d: &mut Overlay3d) {
    let model = &view.doc.model;
    let path = &view.doc.active_path;
    for (p, id, g) in model.all_dimensions_world() {
        let selected = &p == path && view.doc.dim_sel.contains(&id);
        add_dim_geom(out3d, &g, model.unit, selected);
    }
}

/// Draw dimension `id` of the active context with world transform `t` applied (Move preview).
/// World geometry of dimension `id` (context `path`) after the world transform `t` when the
/// entities `moved` are transformed with it: free anchors and anchors on moved vertices follow
/// `t`; anchors on other vertices stay, and then the dimension line goes through where `t`
/// takes the old one (moving a dimension alone drags its offset).
pub fn dim_after_transform(
    model: &varstotas_kernel::Model,
    path: &[varstotas_kernel::InstanceId],
    id: varstotas_kernel::DimId,
    moved: &[EntityRef],
    t: &DMat4,
) -> Option<varstotas_kernel::DimGeom> {
    let g = model.dimension_in(path, id)?;
    let mesh = model.mesh_at(path)?;
    let d = mesh.dims.get(id)?;
    let verts = mesh.vertices_of(moved);
    let follows = |a: &varstotas_kernel::DimAnchor| match a.vertex {
        Some(v) if mesh.vertices.contains_key(v) => verts.contains(&v),
        _ => true,
    };
    let a = if follows(&d.a) { t.transform_point3(g.a) } else { g.a };
    let b = if follows(&d.b) { t.transform_point3(g.b) } else { g.b };
    let offset = if follows(&d.a) && follows(&d.b) {
        t.transform_vector3(g.offset)
    } else {
        let target = t.transform_point3((g.a + g.b) * 0.5 + g.offset);
        let dir = (b - a).normalize_or_zero();
        let v = target - (a + b) * 0.5;
        v - dir * v.dot(dir)
    };
    Some(varstotas_kernel::DimGeom { a, b, offset, text: g.text, leader: g.leader })
}

pub fn draw_dim_moved(
    view: &OverlayView,
    id: varstotas_kernel::DimId,
    moved: &[EntityRef],
    t: &DMat4,
    out3d: &mut Overlay3d,
) {
    let model = &view.doc.model;
    if let Some(g) = dim_after_transform(model, &view.doc.active_path, id, moved, t) {
        add_dim_geom(out3d, &g, model.unit, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dense_nested_models_have_complete_transform_and_clipboard_previews() {
        use varstotas_kernel::{Definition,Edge,Vertex,Instance};
        let mut model=varstotas_kernel::Model::new();
        let empty=||Definition{name:String::new(),mesh:varstotas_kernel::Mesh::new(),is_group:true,guides:Default::default(),params:None,generated:None,source:None,rig:None,rigged:None,preview:None};
        let mut def=empty();
        let count=21_001;
        for i in 0..count {
            let a=def.mesh.vertices.insert(Vertex{pos:DVec3::new(i as f64,0.,0.)});
            let b=def.mesh.vertices.insert(Vertex{pos:DVec3::new(i as f64,1.,0.)});
            def.mesh.edges.insert(Edge::new(a,b));
        }
        let mut id=model.defs.insert(def);
        // More than the old 16-level transform-preview depth limit.
        for _ in 0..18 {
            let mut parent=empty();
            parent.mesh.instances.insert(Instance{def:id,transform:DMat4::IDENTITY,name:String::new(),material:None,tag:None,hidden:false,locked:false});
            id=model.defs.insert(parent);
        }
        let instance=model.root_mesh_mut().instances.insert(Instance{def:id,transform:DMat4::IDENTITY,name:String::new(),material:None,tag:None,hidden:false,locked:false});
        model.rebuild_caches();
        let doc=Document::from_model(model,None);
        let shift=DVec3::new(10.,20.,30.);
        let preview=transformed_preview(&doc,&[EntityRef::Instance(instance)],&DMat4::from_translation(shift));
        assert_eq!(preview.len(),count);
        assert_eq!(preview.first().unwrap().0,shift);
        assert_eq!(preview.last().unwrap().0,shift+DVec3::X*(count-1) as f64);
        let clip=crate::clipboard::Clipboard::from_parts(&doc.model,doc.uid,&[],&[EntityRef::Instance(instance)],&[]).unwrap();
        assert_eq!(clip.preview.len(),count,"paste must show the complete surface too");
        assert_eq!(crate::clipboard::sample_edges(&clip.preview,0).len(),0);
        let thumbnail=crate::clipboard::sample_edges(&clip.preview,10);
        assert!(thumbnail.last().unwrap().0.x>(count as f64)*0.8,"thumbnail sampling must span the whole model");
    }

    #[test]
    fn moving_an_anchored_dimension_drags_its_offset() {
        let mut m = varstotas_kernel::Model::new();
        let (a, b) = (DVec3::ZERO, DVec3::new(1000.0, 0.0, 0.0));
        let es = m.root_mesh_mut().add_line(a, b);
        let id = m.add_dimension_in(&[], a, b, DVec3::new(0.0, -100.0, 0.0)).unwrap();
        let t = DMat4::from_translation(DVec3::new(50.0, -300.0, 0.0));
        // Dimension alone: anchored ends stay, the line follows the move across it.
        let g = dim_after_transform(&m, &[], id, &[], &t).unwrap();
        assert!(g.a.distance(a) < 1e-9 && g.b.distance(b) < 1e-9);
        assert!((g.offset - DVec3::new(0.0, -400.0, 0.0)).length() < 1e-9);
        // With the edge: everything moves rigidly.
        let ents: Vec<EntityRef> = es.into_iter().map(EntityRef::Edge).collect();
        let g = dim_after_transform(&m, &[], id, &ents, &t).unwrap();
        assert!(g.a.distance(t.transform_point3(a)) < 1e-9);
        assert!((g.offset - DVec3::new(0.0, -100.0, 0.0)).length() < 1e-9);
    }

    #[test]
    fn angle_format() {
        assert_eq!(fmt_angle(90.0), "90°");
        assert_eq!(fmt_angle(22.5), "22.5°");
        assert_eq!(fmt_angle(-7.25), "-7.25°");
        assert_eq!(fmt_angle(-0.001), "0°");
    }

    #[test]
    fn array_parsing() {
        assert_eq!(parse_array("5x"), Some(ArrayInput::Multiply(5)));
        assert_eq!(parse_array("x3"), Some(ArrayInput::Multiply(3)));
        assert_eq!(parse_array("*4"), Some(ArrayInput::Multiply(4)));
        assert_eq!(parse_array("/6"), Some(ArrayInput::Divide(6)));
        assert_eq!(parse_array("0x"), None);
        assert_eq!(parse_array("2500"), None);
    }

    #[test]
    fn segments_and_angles() {
        assert_eq!(parse_segments("24s"), Some(24));
        assert_eq!(parse_segments("6 S"), Some(6));
        assert_eq!(parse_segments("2s"), None);
        assert_eq!(parse_segments("24"), None);
        assert_eq!(parse_angle("45"), Some(45.0));
        assert_eq!(parse_angle("-22,5"), Some(-22.5));
        assert_eq!(parse_angle("90deg"), Some(90.0));
        assert_eq!(parse_angle("abc"), None);
    }

    #[test]
    fn lengths() {
        assert_eq!(parse_len(LengthUnit::Millimeter, "2500"), Some(2500.0));
        assert_eq!(parse_len(LengthUnit::Millimeter, "2.5m"), Some(2500.0));
        assert_eq!(parse_lens(LengthUnit::Millimeter, "3000,2000"), Some(vec![3000.0, 2000.0]));
    }

    #[test]
    fn half_circle_arc() {
        let a = DVec3::new(-1000.0, 0.0, 0.0);
        let b = DVec3::new(1000.0, 0.0, 0.0);
        let pts = arc_points(a, b, DVec3::Y, 1000.0, 12);
        assert_eq!(pts.len(), 13);
        assert!(pts[0].distance(a) < 1e-9 && pts[12].distance(b) < 1e-9);
        for p in &pts {
            assert!((p.length() - 1000.0).abs() < 1e-6, "{p}");
            assert!(p.y >= -1e-9);
        }
        assert!(pts[6].distance(DVec3::new(0.0, 1000.0, 0.0)) < 1e-6);
    }

    #[test]
    fn shallow_and_negative_arcs() {
        let a = DVec3::new(0.0, 0.0, 0.0);
        let b = DVec3::new(2000.0, 0.0, 0.0);
        let pts = arc_points(a, b, DVec3::Y, -300.0, 8);
        let apex = pts[4];
        assert!((apex - DVec3::new(1000.0, -300.0, 0.0)).length() < 1e-6, "{apex}");
        // Large bulge (more than half circle).
        let pts = arc_points(a, b, DVec3::Y, 1500.0, 16);
        assert!((pts[8] - DVec3::new(1000.0, 1500.0, 0.0)).length() < 1e-6, "{}", pts[8]);
    }

    #[test]
    fn circle_points_on_radius() {
        let pts = circle_points(DVec3::new(10.0, 20.0, 30.0), DVec3::Z, DVec3::X, 500.0, 24);
        assert_eq!(pts.len(), 24);
        assert!(pts.iter().all(|p| (p.distance(DVec3::new(10.0, 20.0, 30.0)) - 500.0).abs() < 1e-9 && (p.z - 30.0).abs() < 1e-9));
        assert!((pts[0] - DVec3::new(510.0, 20.0, 30.0)).length() < 1e-9);
    }

    #[test]
    fn angles() {
        assert!((signed_angle(DVec3::X, DVec3::Y, DVec3::Z) - 90.0).abs() < 1e-9);
        assert!((signed_angle(DVec3::X, DVec3::Y, -DVec3::Z) + 90.0).abs() < 1e-9);
        assert_eq!(snap_angle(44.0, 15.0, 2.0), (45.0, true));
        assert_eq!(snap_angle(40.0, 15.0, 2.0), (40.0, false));
        let r = rotation_about(DVec3::new(1.0, 0.0, 0.0), DVec3::Z, 90.0);
        assert!((r.transform_point3(DVec3::new(2.0, 0.0, 0.0)) - DVec3::new(1.0, 1.0, 0.0)).length() < 1e-9);
    }

    #[test]
    fn plane_axes_prefer_red() {
        let (u, v) = plane_axes(DVec3::Z);
        assert!((u - DVec3::X).length() < 1e-12 && (v - DVec3::Y).length() < 1e-12);
        let (u, _) = plane_axes(DVec3::X);
        assert!((u - DVec3::Y).length() < 1e-12);
    }
}
