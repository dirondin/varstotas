//! Editing tools: Eraser, Paint Bucket, Push/Pull, Offset, Follow Me.

use super::common::*;
use super::draw::Clicker;
use super::{OverlayView, Tool, ToolContext, ToolId};
use crate::document::{PaletteMaterial, guarded};
use crate::inference::{self, Constraint, InferenceEngine, InferenceKind, InferenceResult};
use crate::overlay::Marker2d;
use egui::{Color32, CursorIcon, Key, PointerButton};
use varstotas_kernel::geom::{PointLoc, Projector, point_in_region, point_seg_dist2};
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::{EdgeId, EntityRef, FaceId, GuideRef, InstanceId, Plane, TOL};
use varstotas_render::Overlay3d;

// ===========================================================================
// Eraser
// ===========================================================================

#[derive(Clone, Debug, PartialEq)]
enum EraseTarget {
    Edge(EdgeId, DVec3, DVec3),
    Instance(InstanceId),
    Guide(GuideRef),
}

#[derive(Default)]
pub struct EraserTool {
    hover: Option<EraseTarget>,
    marked: Vec<EraseTarget>,
    dragging: bool,
}

impl EraserTool {
    fn pick(ctx: &ToolContext) -> Option<EraseTarget> {
        if let Some(Hover::Edge { edge, a, b }) = pick_edge_at(ctx, ctx.pointer.pos, inference::EDGE_PICK_PX as f32) {
            return Some(EraseTarget::Edge(edge, a, b));
        }
        if let Some(g) = pick_guide(ctx) {
            return Some(EraseTarget::Guide(g));
        }
        if let Some(Hover::Instance { id, .. }) = pick_face(ctx) {
            return Some(EraseTarget::Instance(id));
        }
        None
    }

    fn apply(&mut self, ctx: &mut ToolContext) {
        let marked = std::mem::take(&mut self.marked);
        if marked.is_empty() {
            return;
        }
        let m = ctx.pointer.modifiers;
        let soften = m.ctrl || m.command;
        let hide = m.shift && !soften;
        let unsoften = soften && m.shift;
        let path = ctx.doc.active_path.clone();
        let label = if hide {
            "Hide"
        } else if unsoften {
            "Unsoften"
        } else if soften {
            "Soften"
        } else {
            "Erase"
        };
        ctx.doc.commit(label, |model| {
            let guides: Vec<GuideRef> =
                marked.iter().filter_map(|t| if let EraseTarget::Guide(g) = t { Some(*g) } else { None }).collect();
            if !hide && !soften {
                model.remove_guides_in(&path, &guides);
            }
            let Some(mesh) = model.mesh_at_mut(&path) else { return };
            let ents: Vec<EntityRef> = marked
                .iter()
                .filter_map(|t| match t {
                    EraseTarget::Edge(e, ..) => Some(EntityRef::Edge(*e)),
                    EraseTarget::Instance(i) => Some(EntityRef::Instance(*i)),
                    EraseTarget::Guide(_) => None,
                })
                .collect();
            if hide || soften {
                for e in &ents {
                    match *e {
                        EntityRef::Edge(id) => {
                            if let Some(ed) = mesh.edges.get_mut(id) {
                                if hide {
                                    ed.hidden = true;
                                } else {
                                    ed.soft = !unsoften;
                                    ed.smooth = !unsoften;
                                }
                            }
                        }
                        EntityRef::Instance(id) if hide => {
                            if let Some(i) = mesh.instances.get_mut(id) {
                                i.hidden = true;
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                mesh.erase(&ents);
            }
        });
    }
}

impl Tool for EraserTool {
    fn id(&self) -> ToolId {
        ToolId::Eraser
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        self.hover = Self::pick(ctx);
        if self.dragging && ctx.pointer.primary_down {
            if let Some(h) = &self.hover
                && !self.marked.contains(h)
            {
                self.marked.push(h.clone());
            }
        } else if self.dragging {
            self.dragging = false;
            self.apply(ctx);
        }
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button == Some(PointerButton::Primary) {
            self.dragging = true;
            self.marked.clear();
            if let Some(h) = Self::pick(ctx) {
                self.marked.push(h);
            }
        }
    }
    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button == Some(PointerButton::Primary) && self.dragging {
            self.dragging = false;
            self.apply(ctx);
            self.hover = None;
        }
    }
    fn on_key(&mut self, _ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if pressed && key == Key::Escape {
            self.marked.clear();
            self.dragging = false;
            return true;
        }
        false
    }
    fn status_text(&self) -> String {
        "Click or drag over edges, guides or groups to erase. Shift = hide, Ctrl = soften, Ctrl+Shift = unsoften."
            .into()
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        let draw = |t: &EraseTarget, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>| match t {
            EraseTarget::Edge(_, a, b) => line(out3d, *a, *b, SELECT_BLUE, 3.0),
            EraseTarget::Instance(id) => highlight_hover(view, &Hover::Instance { id: *id, point: DVec3::ZERO }, out3d),
            EraseTarget::Guide(g) => {
                draw_guide(view, *g, &varstotas_kernel::glam::DMat4::IDENTITY, SELECT_BLUE, 2.0, out3d, out2d);
            }
        };
        for t in &self.marked {
            draw(t, out3d, out2d);
        }
        if let Some(h) = &self.hover {
            draw(h, out3d, out2d);
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        self.dragging
    }
}

// ===========================================================================
// Paint Bucket
// ===========================================================================

/// Paint Bucket modifier help, per platform (conventions).
pub const PAINT_MODS_HELP: &str = if cfg!(target_os = "macos") {
    "Cmd = sample, Option = connected faces with the same material, Shift = all matching faces, Option+Shift = matching connected faces"
} else {
    "Alt = sample, Ctrl = connected faces with the same material, Shift = all matching faces, Ctrl+Shift = matching connected faces"
};

/// What a Paint Bucket click does, decoded from the modifiers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PaintMods {
    /// Eyedropper: pick up the clicked material instead of painting.
    sample: bool,
    /// Fill the connected faces that share the clicked face's material.
    connected: bool,
    /// Replace every face in the context that shares the clicked face's material.
    same: bool,
}

impl PaintMods {
    fn from(m: &egui::Modifiers) -> Self {
        Self::decode(m, cfg!(target_os = "macos"))
    }

    /// Mac: Cmd = sample, Option = connected, Shift = all matching,
    /// Option+Shift = connected matching. Windows/Linux: Alt = sample, Ctrl = connected,
    /// Shift = all matching, Ctrl+Shift = connected matching.
    fn decode(m: &egui::Modifiers, mac: bool) -> Self {
        let (sample, connected) = if mac { (m.mac_cmd || m.command, m.alt) } else { (m.alt, m.ctrl) };
        if sample {
            return Self { sample: true, ..Self::default() };
        }
        // Connected wins over Shift: Ctrl/Option+Shift replaces matching faces on the connected object.
        Self { sample: false, connected, same: m.shift && !connected }
    }
}

#[derive(Default)]
pub struct PaintTool {
    hover: Option<Hover>,
}

impl Tool for PaintTool {
    fn id(&self) -> ToolId {
        ToolId::Paint
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        self.hover = pick_face(ctx);
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button != Some(PointerButton::Primary) {
            return;
        }
        let Some(h) = pick_face(ctx) else { return };
        let m = ctx.pointer.modifiers;
        let path = ctx.doc.active_path.clone();
        let ray_dir = ctx.pointer.ray.dir;
        // Which side of the face is visible.
        let front = match &h {
            Hover::Face { plane, .. } => ray_dir.dot(plane.normal) < 0.0,
            _ => true,
        };
        let pm = PaintMods::from(&m);
        if pm.sample {
            // Sample.
            let mat = guarded(|| {
                let mesh = ctx.doc.model.mesh_at(&path)?;
                let id = match &h {
                    Hover::Face { face, .. } => {
                        let f = mesh.faces.get(*face)?;
                        if front { f.front } else { f.back }
                    }
                    Hover::Instance { id, .. } => mesh.instances.get(*id)?.material,
                    _ => None,
                }?;
                let mm = ctx.doc.model.materials.get(id)?;
                Some(PaletteMaterial { name: mm.name.clone(), color: mm.color })
            })
            .ok()
            .flatten();
            ctx.doc.current_material = mat;
            return;
        }
        let palette = ctx.doc.current_material.clone();
        let PaintMods { connected, same, .. } = pm;
        ctx.doc.commit("Paint", |model| {
            let mat = palette.as_ref().map(|p| p.ensure_in(model));
            let Some(mesh) = model.mesh_at_mut(&path) else { return };
            match h {
                Hover::Instance { id, .. } => {
                    if let Some(i) = mesh.instances.get_mut(id) {
                        i.material = mat;
                    }
                }
                Hover::Face { face, .. } => {
                    let Some(f0) = mesh.faces.get(face) else { return };
                    let old = if front { f0.front } else { f0.back };
                    let targets: Vec<FaceId> = if connected {
                        // only connected faces that share the clicked face's material.
                        mesh.connected(EntityRef::Face(face))
                            .into_iter()
                            .filter_map(|e| if let EntityRef::Face(f) = e { Some(f) } else { None })
                            .filter(|f| mesh.faces.get(*f).is_some_and(|f| (if front { f.front } else { f.back }) == old))
                            .collect()
                    } else if same {
                        mesh.faces
                            .iter()
                            .filter(|(_, f)| (if front { f.front } else { f.back }) == old)
                            .map(|(id, _)| id)
                            .collect()
                    } else {
                        vec![face]
                    };
                    for t in targets {
                        if let Some(f) = mesh.faces.get_mut(t) {
                            if front {
                                f.front = mat;
                            } else {
                                f.back = mat;
                            }
                        }
                    }
                }
                Hover::Edge { .. } => {}
            }
        });
    }
    fn status_text(&self) -> String {
        format!("Click a face or group to paint. {PAINT_MODS_HELP}.")
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let Some(h) = &self.hover {
            highlight_hover(view, h, out3d);
        }
        let name = view.doc.current_material.as_ref().map(|m| m.name.clone()).unwrap_or_else(|| "Default".into());
        out2d.push(Marker2d::CursorTooltip { text: name });
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
}

// ===========================================================================
// Push/Pull
// ===========================================================================

#[derive(Clone, Debug)]
struct PullState {
    face: FaceId,
    origin: DVec3,
    n: DVec3,
    dist: f64,
}

#[derive(Default)]
pub struct PushPullTool {
    engine: InferenceEngine,
    hover: Option<Hover>,
    state: Option<PullState>,
    cur: Option<InferenceResult>,
    keep: ModToggle,
    clicker: Clicker,
    /// Last operation (face, world distance, keep) for VCB edit / double-click repeat.
    last: Option<(FaceId, f64, bool)>,
    last_dist: Option<f64>,
    unit: Option<varstotas_kernel::LengthUnit>,
}

impl PushPullTool {
    fn update(&mut self, ctx: &ToolContext) {
        self.unit = Some(ctx.doc.model.unit);
        self.keep.update(copy_mod(&ctx.pointer.modifiers));
        if let Some(mut s) = self.state.take() {
            let r = self.pull_point(ctx, &s);
            s.dist = (r.point - s.origin).dot(s.n);
            self.state = Some(s);
            self.cur = Some(r);
        } else {
            self.hover = pick_face(ctx).filter(|h| matches!(h, Hover::Face { .. }));
        }
    }

    fn apply(&mut self, ctx: &mut ToolContext, face: FaceId, dist_world: f64, n_world: DVec3, keep: bool) -> bool {
        let d = dist_world * local_scale_along(ctx.doc, n_world);
        let ok = try_commit_mesh(ctx, "Push/Pull", |m| m.push_pull(face, d, keep)).is_some();
        if ok {
            self.last = Some((face, dist_world, keep));
            self.last_dist = Some(dist_world);
        }
        ok
    }

    fn face_normal(ctx: &ToolContext, face: FaceId) -> Option<DVec3> {
        let mesh = ctx.doc.active_mesh()?;
        let f = mesh.faces.get(face)?;
        let xf = context_xf(ctx.doc);
        Some(xf.inverse().transpose().transform_vector3(f.normal).normalize_or(f.normal))
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        match self.state.take() {
            None => {
                if let Some(Hover::Face { face, point, plane }) = pick_face(ctx) {
                    self.state = Some(PullState { face, origin: point, n: plane.normal, dist: 0.0 });
                    // Ctrl toggled (or held) before the click carries over.
                    self.keep.update(copy_mod(&ctx.pointer.modifiers));
                }
            }
            Some(s) => {
                // Use exactly the distance shown in the preview/tooltip/VCB.
                self.state = Some(s);
                self.update(ctx);
                let s = self.state.take().expect("state");
                if s.dist.abs() > TOL {
                    self.apply(ctx, s.face, s.dist, s.n, self.keep.on);
                }
                self.keep.reset();
                self.cur = None;
            }
        }
    }

    /// Distance along the face normal for the current pointer: the closest point between
    /// the pick ray and the line through the picked point along the normal; point snaps
    /// (endpoints, midpoints, intersections) are projected onto that line.
    fn pull_point(&mut self, ctx: &ToolContext, s: &PullState) -> InferenceResult {
        let c = Constraint::Line { origin: s.origin, dir: s.n, kind: InferenceKind::Free };
        infer(&mut self.engine, ctx, Some(s.origin), Some(c), None)
    }
}

impl Tool for PushPullTool {
    fn id(&self) -> ToolId {
        ToolId::PushPull
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if self.clicker.down(ctx) {
            self.click(ctx);
        }
    }
    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        if self.clicker.up_is_drag(ctx) && self.state.is_some() {
            self.click(ctx);
        }
    }
    fn on_double_click(&mut self, ctx: &mut ToolContext) {
        if self.state.is_some() {
            return;
        }
        let (Some(d), Some(Hover::Face { face, plane, .. })) = (self.last_dist, pick_face(ctx)) else { return };
        let keep = copy_mod(&ctx.pointer.modifiers);
        self.apply(ctx, face, d, plane.normal, keep);
    }
    fn on_key(&mut self, _ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if pressed && key == Key::Escape {
            self.state = None;
            self.cur = None;
            self.keep.reset();
            return true;
        }
        false
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        let Some(v) = parse_len(ctx.doc.model.unit, input) else { return false };
        if v.abs() < TOL {
            return false;
        }
        if let Some(s) = self.state.take() {
            let sign = if s.dist < 0.0 { -1.0 } else { 1.0 };
            self.cur = None;
            let keep = self.keep.on;
            self.keep.reset();
            return self.apply(ctx, s.face, v * sign, s.n, keep);
        }
        if let Some((face, d, keep)) = self.last {
            if !undo_last(ctx) {
                return false;
            }
            let Some(n) = Self::face_normal(ctx, face) else { return false };
            let sign = if d < 0.0 { -1.0 } else { 1.0 };
            return self.apply(ctx, face, v * sign, n, keep);
        }
        false
    }
    fn vcb_label(&self) -> &str {
        "Distance"
    }
    fn vcb_value(&self) -> String {
        let d = self.state.as_ref().map(|s| s.dist).or(self.last_dist);
        match (self.unit, d) {
            (Some(u), Some(d)) => varstotas_kernel::units::format_length(d.abs(), u),
            _ => String::new(),
        }
    }
    fn status_text(&self) -> String {
        if self.state.is_some() {
            format!(
                "Move to set the distance, click or type a value. Ctrl/Option = keep original face ({}).",
                if self.keep.on { "on" } else { "off" }
            )
        } else {
            "Click a face to push or pull it. Double-click repeats the last distance.".into()
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let Some(s) = &self.state {
            let off = s.n * s.dist;
            highlight_face(view, s.face, HOVER_FACE, out3d);
            for lp in face_loops_world(view.doc, s.face) {
                let moved: Vec<DVec3> = lp.iter().map(|p| *p + off).collect();
                polyline(out3d, &moved, true, PREVIEW, 2.0);
                for (a, b) in lp.iter().zip(moved.iter()) {
                    line(out3d, *a, *b, PREVIEW, 1.5);
                }
                fill_polygon(out3d, &moved, Color32::from_rgba_unmultiplied(0x40, 0x60, 0xA0, 0x40));
            }
            dashed(out3d, s.origin, s.origin + off, InferenceKind::OnBlueAxis.color(), 1.5);
            label(out2d, s.origin + off, fmt_len(&view.doc.model, s.dist.abs()));
            if let Some(r) = &self.cur
                && let Some((fp, _)) = r.from_point
            {
                dashed(out3d, fp, r.point, Color32::from_gray(60), 1.5);
                out2d.push(Marker2d::Point { at: fp, shape: crate::overlay::MarkerShape::Dot, color: Color32::from_rgb(0, 0xC0, 0) });
            }
        } else if let Some(h) = &self.hover {
            highlight_hover(view, h, out3d);
        }
    }
    fn cursor_plus(&self) -> bool {
        self.keep.on
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        self.state.is_some()
    }
}

// ===========================================================================
// Offset
// ===========================================================================

/// Signed depth of `p` inside a face region (positive inside), in the face's 2D frame.
pub fn inside_depth(proj: &Projector, loops: &[Vec<DVec3>], p: DVec3) -> f64 {
    if loops.is_empty() {
        return 0.0;
    }
    let q = proj.to2(p);
    let l2: Vec<Vec<DVec2>> = loops.iter().map(|l| l.iter().map(|x| proj.to2(*x)).collect()).collect();
    let mut dmin = f64::INFINITY;
    for l in &l2 {
        for i in 0..l.len() {
            let (d, _) = point_seg_dist2(q, l[i], l[(i + 1) % l.len()]);
            dmin = dmin.min(d);
        }
    }
    let holes: Vec<Vec<DVec2>> = l2[1..].to_vec();
    let inside = point_in_region(q, &l2[0], &holes, 1e-9) != PointLoc::Outside;
    if inside { dmin } else { -dmin }
}

/// Simple miter offset of a closed 2D loop (positive = to the right of travel = outward for CCW).
pub fn offset_loop_2d(l: &[DVec2], d: f64) -> Vec<DVec2> {
    let n = l.len();
    if n < 3 {
        return l.to_vec();
    }
    (0..n)
        .map(|i| {
            let p = l[i];
            let a = (p - l[(i + n - 1) % n]).normalize_or_zero();
            let b = (l[(i + 1) % n] - p).normalize_or_zero();
            let na = DVec2::new(a.y, -a.x);
            let nb = DVec2::new(b.y, -b.x);
            let m = (na + nb).normalize_or(na);
            let c = m.dot(nb).max(0.2);
            p + m * (d / c)
        })
        .collect()
}

#[derive(Clone, Debug)]
struct OffsetState {
    face: FaceId,
    start_depth: f64,
    plane: Plane,
    dist: f64,
}

#[derive(Default)]
pub struct OffsetTool {
    engine: InferenceEngine,
    hover: Option<Hover>,
    state: Option<OffsetState>,
    cur: Option<InferenceResult>,
    clicker: Clicker,
    last: Option<(FaceId, f64)>,
    last_dist: Option<f64>,
    unit: Option<varstotas_kernel::LengthUnit>,
}

impl OffsetTool {
    fn current_dist(&mut self, ctx: &ToolContext, s: &OffsetState) -> f64 {
        let c = Constraint::Plane(s.plane);
        let r = infer(&mut self.engine, ctx, None, Some(c), None);
        let loops = face_loops_world(ctx.doc, s.face);
        let proj = Projector::new(s.plane.origin, s.plane.normal);
        let depth = inside_depth(&proj, &loops, r.point);
        self.cur = Some(r);
        // Moving inward (deeper) = negative (kernel convention).
        s.start_depth - depth
    }

    fn apply(&mut self, ctx: &mut ToolContext, face: FaceId, d: f64) -> bool {
        let n = self.state.as_ref().map(|s| s.plane.normal).unwrap_or(DVec3::Z);
        let tangent = plane_axes(n).0;
        let dl = d * local_scale_along(ctx.doc, tangent);
        let ok = try_commit_mesh(ctx, "Offset", |m| m.offset_face(face, dl)).is_some();
        if ok {
            self.last = Some((face, d));
            self.last_dist = Some(d);
        }
        ok
    }
}

impl Tool for OffsetTool {
    fn id(&self) -> ToolId {
        ToolId::Offset
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        self.unit = Some(ctx.doc.model.unit);
        if let Some(s) = self.state.clone() {
            let d = self.current_dist(ctx, &s);
            if let Some(st) = &mut self.state {
                st.dist = d;
            }
        } else {
            self.hover = pick_face(ctx).filter(|h| matches!(h, Hover::Face { .. }));
        }
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if !self.clicker.down(ctx) {
            return;
        }
        self.click(ctx);
    }
    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        if self.clicker.up_is_drag(ctx) && self.state.is_some() {
            self.click(ctx);
        }
    }
    fn on_double_click(&mut self, ctx: &mut ToolContext) {
        if self.state.is_some() {
            return;
        }
        if let (Some(d), Some(Hover::Face { face, plane, .. })) = (self.last_dist, pick_face(ctx)) {
            self.state = Some(OffsetState { face, start_depth: 0.0, plane, dist: d });
            self.apply(ctx, face, d);
            self.state = None;
        }
    }
    fn on_key(&mut self, _ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if pressed && key == Key::Escape {
            self.state = None;
            return true;
        }
        false
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        let Some(v) = parse_len(ctx.doc.model.unit, input).map(f64::abs) else { return false };
        if v < TOL {
            return false;
        }
        if let Some(s) = self.state.clone() {
            let sign = if s.dist > 0.0 { 1.0 } else { -1.0 };
            let ok = self.apply(ctx, s.face, v * sign);
            self.state = None;
            return ok;
        }
        if let Some((face, d)) = self.last {
            if !undo_last(ctx) {
                return false;
            }
            return self.apply(ctx, face, v * d.signum());
        }
        false
    }
    fn vcb_label(&self) -> &str {
        "Distance"
    }
    fn vcb_value(&self) -> String {
        let d = self.state.as_ref().map(|s| s.dist).or(self.last_dist);
        match (self.unit, d) {
            (Some(u), Some(d)) => varstotas_kernel::units::format_length(d.abs(), u),
            _ => String::new(),
        }
    }
    fn status_text(&self) -> String {
        if self.state.is_some() {
            "Move inward or outward and click, or type a distance.".into()
        } else {
            "Click a face to offset its edges. Double-click repeats the last offset.".into()
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let Some(s) = &self.state {
            highlight_face(view, s.face, HOVER_FACE, out3d);
            let proj = Projector::new(s.plane.origin, s.plane.normal);
            for lp in face_loops_world(view.doc, s.face) {
                let l2: Vec<DVec2> = lp.iter().map(|p| proj.to2(*p)).collect();
                let off: Vec<DVec3> = offset_loop_2d(&l2, s.dist).into_iter().map(|p| proj.to3(p)).collect();
                polyline(out3d, &off, true, PREVIEW, 2.0);
            }
            if let Some(r) = &self.cur {
                label(out2d, r.point, fmt_len(&view.doc.model, s.dist.abs()));
            }
        } else if let Some(h) = &self.hover {
            highlight_hover(view, h, out3d);
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        self.state.is_some()
    }
}

impl OffsetTool {
    fn click(&mut self, ctx: &mut ToolContext) {
        match self.state.clone() {
            None => {
                if let Some(Hover::Face { face, point, plane }) = pick_face(ctx) {
                    let loops = face_loops_world(ctx.doc, face);
                    let proj = Projector::new(plane.origin, plane.normal);
                    let start_depth = inside_depth(&proj, &loops, point);
                    self.state = Some(OffsetState { face, start_depth, plane, dist: 0.0 });
                }
            }
            Some(s) => {
                let d = self.current_dist(ctx, &s);
                if d.abs() > TOL {
                    self.apply(ctx, s.face, d);
                }
                self.state = None;
            }
        }
    }
}

// ===========================================================================
// Follow Me
// ===========================================================================

#[derive(Default)]
pub struct FollowMeTool {
    hover: Option<Hover>,
    profile: Option<FaceId>,
    path: Vec<(EdgeId, DVec3, DVec3)>,
    dragging: bool,
    press: Option<egui::Pos2>,
}

impl FollowMeTool {
    fn run(&mut self, ctx: &mut ToolContext, profile: FaceId, path: Vec<EdgeId>) {
        if path.is_empty() {
            return;
        }
        try_commit_mesh(ctx, "Follow Me", |m| m.follow_me(profile, &path));
        self.profile = None;
        self.path.clear();
    }

    fn preselected_path(ctx: &ToolContext) -> Vec<EdgeId> {
        ctx.doc
            .selection
            .iter()
            .filter_map(|e| if let EntityRef::Edge(e) = e { Some(*e) } else { None })
            .collect()
    }

    fn add_edge(&mut self, h: &Hover) {
        if let Hover::Edge { edge, a, b } = h
            && !self.path.iter().any(|p| p.0 == *edge)
        {
            self.path.push((*edge, *a, *b));
        }
    }
}

impl Tool for FollowMeTool {
    fn id(&self) -> ToolId {
        ToolId::FollowMe
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        self.hover = if self.profile.is_some() { pick_entity(ctx) } else { pick_face(ctx) };
        if self.dragging
            && ctx.pointer.primary_down
            && let Some(h) = self.hover.clone()
        {
            self.add_edge(&h);
        }
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button != Some(PointerButton::Primary) {
            return;
        }
        self.press = Some(ctx.pointer.pos);
        match self.profile {
            None => {
                if let Some(Hover::Face { face, .. }) = pick_face(ctx) {
                    let pre = Self::preselected_path(ctx);
                    if pre.is_empty() {
                        self.profile = Some(face);
                        self.path.clear();
                        self.dragging = true;
                    } else {
                        self.run(ctx, face, pre);
                    }
                }
            }
            Some(profile) => match pick_entity(ctx) {
                Some(h @ Hover::Edge { .. }) => {
                    self.add_edge(&h);
                    self.dragging = true;
                }
                Some(Hover::Face { face, .. }) if face != profile => {
                    // Sweep around the clicked face's outer loop.
                    let edges = guarded(|| {
                        let m = ctx.doc.active_mesh()?;
                        let f = m.faces.get(face)?;
                        let n = f.outer.len();
                        Some((0..n).filter_map(|i| m.edge_between(f.outer[i], f.outer[(i + 1) % n])).collect::<Vec<_>>())
                    })
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                    self.run(ctx, profile, edges);
                }
                _ => {}
            },
        }
    }
    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button != Some(PointerButton::Primary) {
            return;
        }
        let was_drag = self.dragging
            && self.press.take().is_some_and(|p| (ctx.pointer.pos - p).length() > DRAG_CLICK_PX);
        self.dragging = false;
        // A drag that collected several edges finishes the sweep.
        if was_drag && self.path.len() >= 2 && self.profile.is_some() {
            let p = self.profile.unwrap_or_default();
            let path = self.path.iter().map(|e| e.0).collect();
            self.run(ctx, p, path);
        }
    }
    fn on_double_click(&mut self, ctx: &mut ToolContext) {
        if let Some(p) = self.profile
            && !self.path.is_empty()
        {
            let path = self.path.iter().map(|e| e.0).collect();
            self.run(ctx, p, path);
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        match key {
            Key::Escape => {
                self.profile = None;
                self.path.clear();
                true
            }
            Key::Enter => {
                self.on_double_click(ctx);
                true
            }
            _ => false,
        }
    }
    fn status_text(&self) -> String {
        if self.profile.is_none() {
            "Click the profile face (pre-select path edges to sweep immediately).".into()
        } else {
            "Click or drag along path edges; double-click/Enter to finish. Click a face to sweep around its edge. Esc = cancel."
                .into()
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, _out2d: &mut Vec<Marker2d>) {
        if let Some(p) = self.profile {
            highlight_face(view, p, Color32::from_rgba_unmultiplied(0xFF, 0x80, 0x00, 0x60), out3d);
        }
        for (_, a, b) in &self.path {
            line(out3d, *a, *b, Color32::from_rgb(0xFF, 0x40, 0x00), 3.0);
        }
        if let Some(h) = &self.hover {
            highlight_hover(view, h, out3d);
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        self.profile.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_mods_per_platform() {
        let m = |f: fn(&mut egui::Modifiers)| {
            let mut m = egui::Modifiers::NONE;
            f(&mut m);
            m
        };
        let pm = |sample, connected, same| PaintMods { sample, connected, same };
        let cmd = m(|m| (m.command, m.mac_cmd) = (true, true));
        let alt = m(|m| m.alt = true);
        let shift = m(|m| m.shift = true);
        let alt_shift = m(|m| (m.alt, m.shift) = (true, true));
        // egui on Windows/Linux sets `command` together with `ctrl`.
        let ctrl = m(|m| (m.ctrl, m.command) = (true, true));
        let ctrl_shift = m(|m| (m.ctrl, m.command, m.shift) = (true, true, true));
        // macOS (for Mac).
        assert_eq!(PaintMods::decode(&cmd, true), pm(true, false, false));
        assert_eq!(PaintMods::decode(&alt, true), pm(false, true, false));
        assert_eq!(PaintMods::decode(&shift, true), pm(false, false, true));
        assert_eq!(PaintMods::decode(&alt_shift, true), pm(false, true, false));
        assert_eq!(PaintMods::decode(&egui::Modifiers::NONE, true), pm(false, false, false));
        // Windows/Linux.
        assert_eq!(PaintMods::decode(&alt, false), pm(true, false, false));
        assert_eq!(PaintMods::decode(&ctrl, false), pm(false, true, false));
        assert_eq!(PaintMods::decode(&shift, false), pm(false, false, true));
        assert_eq!(PaintMods::decode(&ctrl_shift, false), pm(false, true, false));
    }

    #[test]
    fn inside_depth_square() {
        let proj = Projector::new(DVec3::ZERO, DVec3::Z);
        let l = vec![vec![
            DVec3::new(0.0, 0.0, 0.0),
            DVec3::new(1000.0, 0.0, 0.0),
            DVec3::new(1000.0, 1000.0, 0.0),
            DVec3::new(0.0, 1000.0, 0.0),
        ]];
        assert!((inside_depth(&proj, &l, DVec3::new(500.0, 100.0, 0.0)) - 100.0).abs() < 1e-9);
        assert!((inside_depth(&proj, &l, DVec3::new(500.0, -50.0, 0.0)) + 50.0).abs() < 1e-9);
    }

    #[test]
    fn miter_offset_square() {
        let sq = [DVec2::new(0.0, 0.0), DVec2::new(10.0, 0.0), DVec2::new(10.0, 10.0), DVec2::new(0.0, 10.0)];
        let o = offset_loop_2d(&sq, 1.0);
        assert!((o[0] - DVec2::new(-1.0, -1.0)).length() < 1e-9);
        let i = offset_loop_2d(&sq, -2.0);
        assert!((i[2] - DVec2::new(8.0, 8.0)).length() < 1e-9);
    }
}
