//! Helpers shared by the drawing/editing tools: inference wrapper, context transforms,
//! picking in the active context, VCB parsing, overlay drawing, geometry previews.

use super::{OverlayView, ToolContext};
use crate::document::{Document, guarded};
use crate::inference::{self, Constraint, InferenceEngine, InferenceResult, Query, push_line};
use crate::overlay::Marker2d;
use egui::{Color32, Pos2};
use varstotas_kernel::glam::{DMat4, DVec2, DVec3};
use varstotas_kernel::{EdgeId, EntityRef, FaceId, InstanceId, LengthUnit, Mesh, Model, Plane, units};
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
    ctx.doc.undo().is_some()
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
        return Some(Hover::Instance { id: hit.path[active.len()], point: hit.point });
    }
    let mesh = ctx.doc.model.mesh_at(active)?;
    let f = mesh.faces.get(hit.face)?;
    let xf = context_xf(ctx.doc);
    let n = xf.inverse().transpose().transform_vector3(f.normal).normalize_or(f.normal);
    Some(Hover::Face { face: hit.face, point: hit.point, plane: Plane::new(hit.point, n) })
}

/// Edge of the active context within `radius` px of the pointer (not occluded).
pub fn pick_edge_at(ctx: &ToolContext, pos: Pos2, radius: f32) -> Option<Hover> {
    let active = ctx.doc.active_path.clone();
    let mesh = ctx.doc.model.mesh_at(&active)?;
    let xf = context_xf(ctx.doc);
    let vp = ctx.viewport_d();
    let c = DVec2::new(pos.x as f64, pos.y as f64);
    let mut best: Option<(f64, EdgeId, DVec3, DVec3, DVec3)> = None;
    for (id, e) in mesh.edges.iter() {
        if e.hidden {
            continue;
        }
        let (a, b) = mesh.edge_segment(id);
        let (a, b) = (xf.transform_point3(a), xf.transform_point3(b));
        let Some((sa, sb)) = inference::geo::project_segment(ctx.camera, vp, a, b) else { continue };
        let (d, _) = inference::geo::dist_point_seg2(c, sa, sb);
        if d <= radius as f64 && best.as_ref().is_none_or(|x| d < x.0) {
            let ray = ctx.ray_at(pos);
            let p = inference::geo::closest_on_seg_to_ray(a, b, ray.origin, ray.dir);
            best = Some((d, id, a, b, p));
        }
    }
    let (_, edge, a, b, p) = best?;
    let ray = ctx.ray_at(pos);
    if let Some(hit) = guarded(|| ctx.doc.model.raycast(&ray)).ok().flatten() {
        let depth = (p - ray.origin).dot(ray.dir);
        let tol = (ctx.camera.pixel_size_at(p, vp) * 3.0).max(depth.abs() * 1e-4);
        if hit.t < depth - tol {
            return None;
        }
    }
    Some(Hover::Edge { edge, a, b })
}

/// Edge first, then face/instance.
pub fn pick_entity(ctx: &ToolContext) -> Option<Hover> {
    pick_edge_at(ctx, ctx.pointer.pos, inference::EDGE_PICK_PX as f32).or_else(|| pick_face(ctx))
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

/// Angle in degrees: `45`, `-30`, `22.5`, `22,5`, `45deg`, `45°`.
pub fn parse_angle(s: &str) -> Option<f64> {
    let s = s.trim().to_ascii_lowercase();
    let s = s.trim_end_matches('°').trim_end_matches("deg").trim().replace(',', ".");
    let v = s.parse::<f64>().ok()?;
    v.is_finite().then_some(v)
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
    let n = n.normalize_or(DVec3::Z);
    let mut u = DVec3::X - n * DVec3::X.dot(n);
    if u.length_squared() < 1e-6 {
        u = DVec3::Y - n * DVec3::Y.dot(n);
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
/// (in the active context). Instances contribute their definition's edges (capped).
pub fn transformed_preview(doc: &Document, ents: &[EntityRef], t_world: &DMat4, limit: usize) -> Vec<(DVec3, DVec3)> {
    guarded(|| {
        let mut out = Vec::new();
        let Some(mesh) = doc.active_mesh() else { return out };
        let xf = context_xf(doc);
        let vs = mesh.vertices_of(ents);
        let tl = xf.inverse() * *t_world * xf;
        let mut seen: HashSet<EdgeId> = HashSet::new();
        for v in &vs {
            for e in mesh.vertex_edges(*v) {
                if !seen.insert(e) || out.len() >= limit {
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
                instance_edges(&doc.model, inst.def, m, &mut out, limit, 0);
            }
        }
        out
    })
    .unwrap_or_default()
}

fn instance_edges(model: &Model, def: varstotas_kernel::DefId, m: DMat4, out: &mut Vec<(DVec3, DVec3)>, limit: usize, depth: usize) {
    if depth > 16 {
        return;
    }
    let Some(d) = model.defs.get(def) else { return };
    for (id, e) in d.mesh.edges.iter() {
        if out.len() >= limit {
            return;
        }
        if e.hidden {
            continue;
        }
        let (a, b) = d.mesh.edge_segment(id);
        out.push((m.transform_point3(a), m.transform_point3(b)));
    }
    for inst in d.mesh.instances.values() {
        instance_edges(model, inst.def, m * inst.transform, out, limit, depth + 1);
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
            for (a, b) in transformed_preview(view.doc, &[EntityRef::Instance(*id)], &DMat4::IDENTITY, 4000) {
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

/// Draw guide lines and guide points of the model (dashed, always visible). Called by the app.
pub fn draw_guides(view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
    let model = &view.doc.model;
    if model.guides.is_empty() && model.guide_points.is_empty() {
        return;
    }
    let cam = view.camera;
    let reach = (cam.eye - cam.target).length().max(1000.0) * 20.0;
    let c = Color32::from_rgb(0x50, 0x50, 0x60);
    for g in &model.guides {
        let d = g.dir.normalize_or_zero();
        if d == DVec3::ZERO {
            continue;
        }
        // Center the drawn segment on the point of the guide closest to the camera target.
        let mid = g.origin + d * (cam.target - g.origin).dot(d);
        push_line(out3d, mid - d * reach, mid + d * reach, c, 1.0, true, false);
    }
    for p in &model.guide_points {
        out2d.push(Marker2d::Point { at: *p, shape: crate::overlay::MarkerShape::Cross, color: c });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
