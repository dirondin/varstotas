//! Minimal Select tool: click / double / triple click, window & crossing selection,
//! modifier semantics, entering/leaving group contexts.

use varstotas_i18n::tr;
use super::{OverlayView, Tool, ToolContext, ToolId};
use super::common::DimPart;
use crate::document::guarded;
use crate::overlay::Marker2d;
use egui::{Color32, CursorIcon, Key, PointerButton, Pos2, Rect};
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::{ContextPath, EntityRef, GuideRef};
use varstotas_render::Overlay3d;

const EDGE_PICK_PX: f32 = 6.0;
const DRAG_THRESHOLD_PX: f32 = 4.0;

#[derive(Default)]
pub struct SelectTool {
    press: Option<Pos2>,
    box_end: Option<Pos2>,
    /// Aiming a light by its handle: the light and how it was before the drag.
    aim: Option<(usize, varstotas_kernel::Light)>,
}

/// How a click / box combines with the current selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelOp {
    Replace,
    Add,
    Toggle,
    Remove,
}

/// Selection modifiers: "add" is Option on macOS and Ctrl on Windows; Cmd and Ctrl
/// are accepted everywhere too. Shift toggles; Shift + add removes.
pub fn sel_op(m: &egui::Modifiers) -> SelOp {
    let add = m.alt || m.ctrl || m.command || m.mac_cmd;
    match (add, m.shift) {
        (true, true) => SelOp::Remove,
        (true, false) => SelOp::Add,
        (false, true) => SelOp::Toggle,
        (false, false) => SelOp::Replace,
    }
}

/// Modifier help for the status bar / Instructor (platform wording).
pub fn sel_mods_help() -> String {
    tr!("tool-select-mods")
}

/// What was clicked, relative to some context.
struct Picked {
    /// Context the entity lives in (may differ from the active one when clicking outside it).
    context: ContextPath,
    entity: EntityRef,
}

fn starts_with(path: &[varstotas_kernel::InstanceId], prefix: &[varstotas_kernel::InstanceId]) -> bool {
    path.len() >= prefix.len() && path[..prefix.len()] == *prefix
}

fn dist_to_segment(p: Pos2, a: Pos2, b: Pos2) -> (f32, f32) {
    let ab = b - a;
    let len2 = ab.length_sq();
    let t = if len2 > 0.0 { ((p - a).dot(ab) / len2).clamp(0.0, 1.0) } else { 0.0 };
    ((a + ab * t - p).length(), t)
}

/// Liang–Barsky segment/rect overlap test.
fn segment_hits_rect(a: Pos2, b: Pos2, r: Rect) -> bool {
    if r.contains(a) || r.contains(b) {
        return true;
    }
    let d = b - a;
    let (mut t0, mut t1) = (0.0f32, 1.0f32);
    for (p, q) in [
        (-d.x, a.x - r.min.x),
        (d.x, r.max.x - a.x),
        (-d.y, a.y - r.min.y),
        (d.y, r.max.y - a.y),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return false;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
            if t0 > t1 {
                return false;
            }
        }
    }
    true
}

impl SelectTool {
    /// Put the light back as it was before an aim drag.
    fn cancel_aim(&mut self, ctx: &mut ToolContext) {
        if let Some((i, orig)) = self.aim.take()
            && let Some(l) = ctx.doc.model.lights.get_mut(i)
        {
            *l = orig;
        }
    }

    fn pick(&self, ctx: &ToolContext) -> Option<Picked> {
        let active = ctx.doc.active_path.clone();
        let hit = ctx.pick();
        // Edges of the active context take priority when close to the cursor and not occluded
        // (perspective-correct, silhouette/boundary edges win over their own faces).
        if let Some(super::common::Hover::Edge { edge, .. }) =
            super::common::pick_edge_at(ctx, ctx.pointer.pos, EDGE_PICK_PX)
        {
            return Some(Picked { context: active, entity: EntityRef::Edge(edge) });
        }
        // Child groups without faces (edges, guides or dimensions only) are picked by their
        // wire when it is in front of the face under the cursor.
        if let Some(super::common::Hover::Instance { id, point }) =
            super::common::pick_instance_wire(ctx, EDGE_PICK_PX as f64)
        {
            let eye = ctx.pointer.ray.origin;
            if hit.as_ref().is_none_or(|h| point.distance(eye) <= h.point.distance(eye) + varstotas_kernel::TOL) {
                return Some(Picked { context: active, entity: EntityRef::Instance(id) });
            }
        }
        let hit = hit?;
        if starts_with(&hit.path, &active) {
            let entity = if hit.path.len() > active.len() {
                EntityRef::Instance(hit.path[active.len()])
            } else {
                EntityRef::Face(hit.face)
            };
            if ctx.doc.model.entity_locked(&active, entity) { return None; }
            Some(Picked { context: active, entity })
        } else {
            // Clicked outside the open group: resolve in the closest common context.
            let n = hit.path.iter().zip(active.iter()).take_while(|(a, b)| a == b).count();
            let context: ContextPath = hit.path[..n].to_vec();
            let entity = if hit.path.len() > n {
                EntityRef::Instance(hit.path[n])
            } else {
                EntityRef::Face(hit.face)
            };
            if ctx.doc.model.entity_locked(&context, entity) { return None; }
            Some(Picked { context, entity })
        }
    }

    /// A guide under the cursor wins over faces/instances (not over edges of the context).
    fn pick_guide(&self, ctx: &ToolContext, picked: &Option<Picked>) -> Option<GuideRef> {
        if matches!(picked, Some(Picked { entity: EntityRef::Edge(_), .. })) {
            return None;
        }
        super::common::pick_guide(ctx)
    }

    /// A dimension under the cursor: its text always wins; its lines like guides.
    fn pick_dim(&self, ctx: &ToolContext, picked: &Option<Picked>) -> Option<(varstotas_kernel::DimId, DimPart)> {
        let d = super::common::pick_dim(ctx)?;
        if d.1 == DimPart::Line && matches!(picked, Some(Picked { entity: EntityRef::Edge(_), .. })) {
            return None;
        }
        Some(d)
    }

    /// A section plane whose border is under the cursor (edges of the context win).
    fn pick_section(&self, ctx: &ToolContext, picked: &Option<Picked>) -> Option<usize> {
        if matches!(picked, Some(Picked { entity: EntityRef::Edge(_), .. })) {
            return None;
        }
        super::section::pick_section_at(ctx.doc, ctx.camera, ctx.viewport, ctx.pointer.pos)
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        let op = sel_op(&ctx.pointer.modifiers);
        // Light markers are drawn on top: they win.
        if let Some(i) = super::light::pick_light(&ctx.doc.model.lights, ctx.camera, ctx.viewport, ctx.pointer.pos) {
            match op {
                SelOp::Remove => ctx.doc.clear_selection(),
                SelOp::Toggle if ctx.doc.light_sel == Some(i) => ctx.doc.clear_selection(),
                _ => ctx.doc.select_light(i),
            }
            return;
        }
        let picked = self.pick(ctx);
        if let Some(i) = self.pick_section(ctx, &picked) {
            match op {
                SelOp::Remove => ctx.doc.clear_selection(),
                SelOp::Toggle if ctx.doc.section_sel == Some(i) => ctx.doc.clear_selection(),
                _ => ctx.doc.select_section(i),
            }
            return;
        }
        if let Some((d, _)) = self.pick_dim(ctx, &picked) {
            match op {
                SelOp::Remove => ctx.doc.remove_dims_from_selection(&[d]),
                SelOp::Add => ctx.doc.add_dims(&[d]),
                SelOp::Toggle => ctx.doc.toggle_dims(&[d]),
                SelOp::Replace => ctx.doc.set_selection_full(Vec::new(), Vec::new(), vec![d]),
            }
            return;
        }
        if let Some(g) = self.pick_guide(ctx, &picked) {
            match op {
                SelOp::Remove => ctx.doc.remove_guides_from_selection(&[g]),
                SelOp::Add => ctx.doc.add_guides(&[g]),
                SelOp::Toggle => ctx.doc.toggle_guides(&[g]),
                SelOp::Replace => ctx.doc.set_selection_with_guides(Vec::new(), vec![g]),
            }
            return;
        }
        let Some(p) = picked else {
            if op == SelOp::Replace {
                ctx.doc.clear_selection();
            }
            return;
        };
        if p.context != ctx.doc.active_path {
            ctx.doc.set_active_path(p.context);
        }
        // An edge of an arc / circle selects the whole curve.
        let ents = guarded(|| ctx.doc.model.mesh_at(&ctx.doc.active_path).map(|m| curve_unit(m, p.entity)))
            .ok()
            .flatten()
            .unwrap_or_else(|| vec![p.entity]);
        match op {
            SelOp::Remove => ctx.doc.remove_from_selection(&ents),
            SelOp::Add => ctx.doc.add_to_selection(ents),
            SelOp::Toggle => ctx.doc.toggle_many(&ents),
            SelOp::Replace => ctx.doc.set_selection(ents),
        }
    }

    fn box_select(&mut self, ctx: &mut ToolContext, a: Pos2, b: Pos2) {
        let rect = Rect::from_two_pos(a, b);
        let mode = if b.x < a.x { BoxMode::Crossing } else { BoxMode::Window };
        let active = ctx.doc.active_path.clone();
        let found = guarded(|| {
            let model = &ctx.doc.model;
            let mut out = Vec::new();
            let Some(mesh) = model.mesh_at(&active) else {
                return out;
            };
            let proj = BoxProjector { camera: ctx.camera, viewport: ctx.viewport_d(), rect, mode };
            let xf = model.transform_at(&active);
            let world = |v: varstotas_kernel::VertexId| xf.transform_point3(mesh.vertices[v].pos);
            for (id, e) in mesh.edges.iter() {
                if !e.hidden && proj.segment(world(e.v[0]), world(e.v[1])) {
                    out.push(EntityRef::Edge(id));
                }
            }
            for (id, f) in mesh.faces.iter() {
                if f.hidden {
                    continue;
                }
                let pts: Vec<DVec3> = f.outer.iter().map(|v| world(*v)).collect();
                if proj.polygon(&pts) {
                    out.push(EntityRef::Face(id));
                }
            }
            for (id, inst) in mesh.instances.iter() {
                if inst.hidden {
                    continue;
                }
                let mut geo = InstanceGeo::default();
                collect_instance_geo(model, inst.def, xf * inst.transform, &mut geo, 0);
                if proj.instance(&geo) {
                    out.push(EntityRef::Instance(id));
                }
            }
            out
        })
        .unwrap_or_default();
        let guides = guarded(|| {
            let proj = BoxProjector { camera: ctx.camera, viewport: ctx.viewport_d(), rect, mode };
            proj.guides(&ctx.doc.model, &ctx.doc.active_path)
        })
        .unwrap_or_default();

        let dims = guarded(|| {
            let proj = BoxProjector { camera: ctx.camera, viewport: ctx.viewport_d(), rect, mode };
            proj.dims(&ctx.doc.model, &ctx.doc.active_path)
        })
        .unwrap_or_default();

        match sel_op(&ctx.pointer.modifiers) {
            SelOp::Remove => {
                ctx.doc.remove_from_selection(&found);
                ctx.doc.remove_guides_from_selection(&guides);
                ctx.doc.remove_dims_from_selection(&dims);
            }
            SelOp::Add => {
                ctx.doc.add_to_selection(found);
                ctx.doc.add_guides(&guides);
                ctx.doc.add_dims(&dims);
            }
            SelOp::Toggle => {
                ctx.doc.toggle_many(&found);
                ctx.doc.toggle_guides(&guides);
                ctx.doc.toggle_dims(&dims);
            }
            SelOp::Replace => ctx.doc.set_selection_full(found, guides, dims),
        }
    }
}

/// `e` and, for an edge of a curve, all edges of that curve.
pub fn curve_unit(mesh: &varstotas_kernel::Mesh, e: EntityRef) -> Vec<EntityRef> {
    if let EntityRef::Edge(id) = e
        && let Some(cid) = mesh.edges.get(id).and_then(|x| x.curve)
    {
        let mut v: Vec<EntityRef> = vec![e];
        v.extend(mesh.curve_edges(cid).into_iter().filter(|x| *x != id).map(EntityRef::Edge));
        return v;
    }
    vec![e]
}

/// Window (left→right: entirely inside) or crossing (right→left: touching) selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BoxMode {
    Window,
    Crossing,
}

/// Tests world-space primitives against a screen selection rectangle.
struct BoxProjector<'a> {
    camera: &'a varstotas_render::Camera,
    viewport: DVec2,
    rect: Rect,
    mode: BoxMode,
}

impl BoxProjector<'_> {
    fn screen(&self, p: DVec3) -> Option<Pos2> {
        self.camera.world_to_screen(p, self.viewport).map(|s| Pos2::new(s.x as f32, s.y as f32))
    }

    /// Visible (near-clipped) part of a segment on screen.
    fn clipped(&self, a: DVec3, b: DVec3) -> Option<(Pos2, Pos2)> {
        let (sa, sb) = crate::inference::geo::project_segment(self.camera, self.viewport, a, b)?;
        Some((Pos2::new(sa.x as f32, sa.y as f32), Pos2::new(sb.x as f32, sb.y as f32)))
    }

    fn inside(&self, p: DVec3) -> bool {
        self.screen(p).is_some_and(|s| self.rect.contains(s))
    }

    fn segment(&self, a: DVec3, b: DVec3) -> bool {
        match self.mode {
            BoxMode::Window => self.inside(a) && self.inside(b),
            BoxMode::Crossing => self.clipped(a, b).is_some_and(|(sa, sb)| segment_hits_rect(sa, sb, self.rect)),
        }
    }

    /// Closed planar polygon (face outer loop): window = all corners inside; crossing = any
    /// boundary segment touches the box, or the box lies inside the polygon.
    fn polygon(&self, pts: &[DVec3]) -> bool {
        if pts.is_empty() {
            return false;
        }
        match self.mode {
            BoxMode::Window => pts.iter().all(|p| self.inside(*p)),
            BoxMode::Crossing => {
                let n = pts.len();
                if (0..n).any(|i| self.segment(pts[i], pts[(i + 1) % n])) {
                    return true;
                }
                // The box may be entirely inside the face (no boundary touches it).
                let screen: Option<Vec<Pos2>> = pts.iter().map(|p| self.screen(*p)).collect();
                screen.is_some_and(|poly| point_in_polygon(self.rect.center(), &poly))
            }
        }
    }

    /// Guides in the box: guide points inside it (both modes); guide lines are infinite, so
    /// only a crossing box (touching the visible line) selects them.
    /// Only guides of the active context `path` are considered.
    fn guides(&self, model: &varstotas_kernel::Model, path: &[varstotas_kernel::InstanceId]) -> Vec<GuideRef> {
        if model.guides_hidden {
            return Vec::new();
        }
        let mut out = Vec::new();
        for (g, geom) in model.guides_in(path) {
            let hit = match geom {
                varstotas_kernel::GuideGeom::Point(p) => self.inside(p),
                varstotas_kernel::GuideGeom::Line(l) => {
                    self.mode == BoxMode::Crossing
                        && super::common::guide_segment(self.camera, &l).is_some_and(|(a, b)| self.segment(a, b))
                }
            };
            if hit {
                out.push(g);
            }
        }
        out
    }

    /// Dimensions of the context `path` in the box: window = anchors and dimension line inside;
    /// crossing = any of its lines touches the box.
    fn dims(&self, model: &varstotas_kernel::Model, path: &[varstotas_kernel::InstanceId]) -> Vec<varstotas_kernel::DimId> {
        let mut out = Vec::new();
        for (id, g) in model.dimensions_in(path) {
            let (p, q) = g.line();
            let hit = match self.mode {
                BoxMode::Window => [g.a, g.b, p, q].iter().all(|x| self.inside(*x)),
                BoxMode::Crossing => [(p, q), (g.a, p), (g.b, q)].iter().any(|(a, b)| self.segment(*a, *b)),
            };
            if hit {
                out.push(id);
            }
        }
        out
    }

    /// Group/component: window = all its geometry inside; crossing = any of it touches.
    fn instance(&self, g: &InstanceGeo) -> bool {
        if g.segments.is_empty() && g.faces.is_empty() {
            return false;
        }
        match self.mode {
            BoxMode::Window => {
                g.segments.iter().all(|(a, b)| self.segment(*a, *b)) && g.faces.iter().all(|f| self.polygon(f))
            }
            BoxMode::Crossing => {
                g.segments.iter().any(|(a, b)| self.segment(*a, *b)) || g.faces.iter().any(|f| self.polygon(f))
            }
        }
    }
}

/// World-space geometry of an instance (recursively through nested instances).
#[derive(Default)]
struct InstanceGeo {
    segments: Vec<(DVec3, DVec3)>,
    faces: Vec<Vec<DVec3>>,
}

fn collect_instance_geo(
    model: &varstotas_kernel::Model,
    def: varstotas_kernel::DefId,
    m: varstotas_kernel::glam::DMat4,
    out: &mut InstanceGeo,
    depth: usize,
) {
    if depth > 16 {
        return;
    }
    let Some(d) = model.defs.get(def) else { return };
    let mesh = &d.mesh;
    for (_, e) in mesh.edges.iter() {
        if !e.hidden {
            out.segments.push((
                m.transform_point3(mesh.vertices[e.v[0]].pos),
                m.transform_point3(mesh.vertices[e.v[1]].pos),
            ));
        }
    }
    for (_, f) in mesh.faces.iter() {
        if !f.hidden {
            out.faces.push(f.outer.iter().map(|v| m.transform_point3(mesh.vertices[*v].pos)).collect());
        }
    }
    for id in mesh.dims.keys() {
        if let Some(g) = mesh.dim_geom(id) {
            let (p, q) = g.transformed(&m).line();
            out.segments.push((p, q));
        }
    }
    if !model.guides_hidden {
        out.segments.extend(d.guides.points.values().map(|p| (m.transform_point3(*p), m.transform_point3(*p))));
    }
    for inst in mesh.instances.values() {
        if !inst.hidden {
            collect_instance_geo(model, inst.def, m * inst.transform, out, depth + 1);
        }
    }
}

/// Even-odd point-in-polygon test (screen space).
fn point_in_polygon(p: Pos2, poly: &[Pos2]) -> bool {
    let mut inside = false;
    let n = poly.len();
    let mut j = n.wrapping_sub(1);
    for i in 0..n {
        let (a, b) = (poly[i], poly[j]);
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
        j = i;
    }
    inside
}

impl Tool for SelectTool {
    fn id(&self) -> ToolId {
        ToolId::Select
    }

    fn deactivate(&mut self, ctx: &mut ToolContext) {
        self.press = None;
        self.box_end = None;
        self.cancel_aim(ctx);
    }

    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        // The selected light's aim handle: drag it onto what the light should shine at.
        if ctx.pointer.button == Some(PointerButton::Primary)
            && let Some(i) = ctx.doc.light_sel
            && let Some(l) = ctx.doc.model.lights.get(i)
            && super::light::on_aim_handle(l, ctx.camera, ctx.viewport, ctx.pointer.pos)
        {
            self.aim = Some((i, l.clone()));
            return;
        }
        if ctx.pointer.button == Some(PointerButton::Primary) {
            self.press = Some(ctx.pointer.pos);
            self.box_end = None;
        }
    }

    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        if let Some((i, orig)) = &self.aim {
            let target = ctx.point_under_pointer();
            if let Some(d) = (target - orig.position).try_normalize()
                && let Some(l) = ctx.doc.model.lights.get_mut(*i)
            {
                l.direction = d;
            }
            return;
        }
        if let Some(p) = self.press {
            if !ctx.pointer.primary_down {
                self.press = None;
                self.box_end = None;
            } else if self.box_end.is_some() || (ctx.pointer.pos - p).length() > DRAG_THRESHOLD_PX {
                self.box_end = Some(ctx.pointer.pos);
            }
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button != Some(PointerButton::Primary) {
            return;
        }
        if let Some((i, orig)) = self.aim.take() {
            // One undo step from the light as it was before the drag.
            let Some(now) = ctx.doc.model.lights.get_mut(i).map(|l| std::mem::replace(l, orig.clone())) else { return };
            if now != orig {
                ctx.commit(&tr!("tool-undo-aim-light"), |m| m.lights[i] = now);
            }
            return;
        }
        match (self.press.take(), self.box_end.take()) {
            (Some(a), Some(b)) => self.box_select(ctx, a, b),
            (Some(_), None) => self.click(ctx),
            _ => {}
        }
    }

    fn on_double_click(&mut self, ctx: &mut ToolContext) {
        let picked = self.pick(ctx);
        if let Some(i) = self.pick_section(ctx, &picked) {
            // Double-click a section plane: toggle its cut.
            if ctx.pointer.click_count == 2 {
                let on = ctx.doc.model.sections.get(i).is_some_and(|s| s.active);
                ctx.commit(&tr!("tool-undo-active-cut"), |m| m.set_active_section((!on).then_some(i)));
                ctx.doc.select_section(i);
            }
            return;
        }
        if let Some((d, _)) = self.pick_dim(ctx, &picked) {
            // Double-click a dimension (its text or lines): edit the text.
            if ctx.pointer.click_count == 2 {
                ctx.requests.push(super::ToolRequest::EditDimensionText(d));
            }
            return;
        }
        if self.pick_guide(ctx, &picked).is_some() {
            return;
        }
        let Some(p) = picked else { return };
        if p.context != ctx.doc.active_path {
            return;
        }
        let path = p.context.clone();
        match (ctx.pointer.click_count, p.entity) {
            (2, EntityRef::Instance(id)) => {
                let mut new_path = path;
                new_path.push(id);
                ctx.doc.set_active_path(new_path);
            }
            (2, EntityRef::Face(f)) => {
                let edges = guarded(|| ctx.doc.model.mesh_at(&path).map(|m| m.face_edges(f)))
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                let mut sel = vec![EntityRef::Face(f)];
                sel.extend(edges.into_iter().map(EntityRef::Edge));
                ctx.doc.add_to_selection(sel);
            }
            (2, EntityRef::Edge(e))
                if ctx.doc.model.mesh_at(&path).is_some_and(|m| m.spline_of_edge(e).is_some()) =>
            {
                // a Bezier / smooth / freehand curve: reshape it
                ctx.doc.set_selection(vec![EntityRef::Edge(e)]);
                ctx.requests.push(super::ToolRequest::SwitchTool(super::ToolId::EditCurve));
            }
            (2, EntityRef::Edge(e)) => {
                let faces = guarded(|| ctx.doc.model.mesh_at(&path).map(|m| m.edge_faces(e)))
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                ctx.doc.add_to_selection(faces.into_iter().map(EntityRef::Face));
            }
            (n, ent) if n >= 3 && !matches!(ent, EntityRef::Instance(_)) => {
                let conn = guarded(|| ctx.doc.model.mesh_at(&path).map(|m| m.connected(ent)))
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                ctx.doc.add_to_selection(conn.into_iter().filter(|e| !matches!(e, EntityRef::Vertex(_))));
            }
            _ => {}
        }
    }

    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed || key != Key::Escape {
            return false;
        }
        if self.aim.is_some() {
            self.cancel_aim(ctx);
        } else if self.press.is_some() {
            self.press = None;
            self.box_end = None;
        } else if ctx.doc.has_selection() {
            ctx.doc.clear_selection();
        } else {
            ctx.doc.exit_context();
        }
        true
    }

    fn status_text(&self) -> String {
        if self.aim.is_some() {
            tr!("tool-select-aim")
        } else if self.box_end.is_some() {
            tr!("tool-select-box", mods = sel_mods_help())
        } else {
            tr!("tool-select-idle", mods = sel_mods_help())
        }
    }

    fn overlay(&self, _view: &OverlayView, _out3d: &mut Overlay3d, out2d: &mut Vec<crate::overlay::Marker2d>) {
        if let (Some(a), Some(b)) = (self.press, self.box_end) {
            let crossing = b.x < a.x;
            out2d.push(Marker2d::ScreenRect {
                min: a,
                max: b,
                dashed: crossing,
                color: if crossing { Color32::from_rgb(0x20, 0x90, 0x40) } else { Color32::from_rgb(0x00, 0x50, 0xFF) },
            });
        }
    }

    fn cursor(&self) -> CursorIcon {
        CursorIcon::Default
    }

    fn is_busy(&self) -> bool {
        self.press.is_some() || self.aim.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_modifiers_follow_platform() {
        let m = |f: fn(&mut egui::Modifiers)| {
            let mut m = egui::Modifiers::NONE;
            f(&mut m);
            m
        };
        assert_eq!(sel_op(&egui::Modifiers::NONE), SelOp::Replace);
        assert_eq!(sel_op(&m(|m| m.shift = true)), SelOp::Toggle);
        // macOS: Option adds, Shift+Option removes.
        assert_eq!(sel_op(&m(|m| m.alt = true)), SelOp::Add);
        assert_eq!(sel_op(&m(|m| (m.alt, m.shift) = (true, true))), SelOp::Remove);
        // Windows (and Cmd on macOS) parity.
        assert_eq!(sel_op(&m(|m| m.ctrl = true)), SelOp::Add);
        assert_eq!(sel_op(&m(|m| (m.command, m.mac_cmd) = (true, true))), SelOp::Add);
        assert_eq!(sel_op(&m(|m| (m.ctrl, m.shift) = (true, true))), SelOp::Remove);
    }
    use varstotas_render::{Camera, StandardView};

    const VP: DVec2 = DVec2::new(1000.0, 800.0);

    /// Parallel top view: world (x, y) at the origin is the viewport centre, 10 mm per point.
    fn top_camera() -> Camera {
        let mut c = Camera { target: DVec3::ZERO, ..Camera::default() };
        c.set_standard_view(StandardView::Top);
        c.set_ortho(true);
        c.ortho_height = 8000.0;
        c
    }

    fn proj(c: &Camera, a: Pos2, b: Pos2) -> BoxProjector<'_> {
        BoxProjector {
            camera: c,
            viewport: VP,
            rect: Rect::from_two_pos(a, b),
            mode: if b.x < a.x { BoxMode::Crossing } else { BoxMode::Window },
        }
    }

    fn square(x: f64, y: f64, s: f64) -> Vec<DVec3> {
        vec![
            DVec3::new(x, y, 0.0),
            DVec3::new(x + s, y, 0.0),
            DVec3::new(x + s, y + s, 0.0),
            DVec3::new(x, y + s, 0.0),
        ]
    }

    #[test]
    fn polygon_helpers() {
        let sq = [Pos2::new(0.0, 0.0), Pos2::new(10.0, 0.0), Pos2::new(10.0, 10.0), Pos2::new(0.0, 10.0)];
        assert!(point_in_polygon(Pos2::new(5.0, 5.0), &sq));
        assert!(!point_in_polygon(Pos2::new(15.0, 5.0), &sq));
        let r = Rect::from_min_max(Pos2::new(2.0, 2.0), Pos2::new(4.0, 4.0));
        assert!(segment_hits_rect(Pos2::new(0.0, 3.0), Pos2::new(10.0, 3.0), r));
        assert!(!segment_hits_rect(Pos2::new(0.0, 5.0), Pos2::new(10.0, 5.0), r));
    }

    #[test]
    fn window_needs_fully_inside_crossing_needs_touch() {
        let c = top_camera();
        // 1000 mm square at the origin = screen (500,400)-(600,300).
        let face = square(0.0, 0.0, 1000.0);
        let s0 = c.world_to_screen(DVec3::ZERO, VP).unwrap();
        assert!((s0 - DVec2::new(500.0, 400.0)).length() < 1e-6);

        // Window around it: selected; window covering half: not selected.
        assert!(proj(&c, Pos2::new(450.0, 250.0), Pos2::new(650.0, 450.0)).polygon(&face));
        assert!(!proj(&c, Pos2::new(450.0, 250.0), Pos2::new(550.0, 450.0)).polygon(&face));
        // Crossing (right→left) touching half: selected.
        assert!(proj(&c, Pos2::new(550.0, 450.0), Pos2::new(450.0, 250.0)).polygon(&face));
        // Crossing box entirely inside the face (touches no edge): selected.
        assert!(proj(&c, Pos2::new(560.0, 360.0), Pos2::new(540.0, 340.0)).polygon(&face));
        // Crossing box away from it: not selected.
        assert!(!proj(&c, Pos2::new(800.0, 700.0), Pos2::new(700.0, 600.0)).polygon(&face));
        // Edges.
        let (a, b) = (DVec3::ZERO, DVec3::new(1000.0, 0.0, 0.0));
        assert!(!proj(&c, Pos2::new(540.0, 450.0), Pos2::new(560.0, 350.0)).segment(a, b));
        assert!(proj(&c, Pos2::new(560.0, 450.0), Pos2::new(540.0, 350.0)).segment(a, b));
    }

    #[test]
    fn instance_uses_geometry_not_bounding_box() {
        let c = top_camera();
        // L-shaped group: two squares; the bbox corner (1000..2000, 1000..2000) is empty.
        let g = InstanceGeo {
            segments: Vec::new(),
            faces: vec![square(0.0, 0.0, 1000.0), square(1000.0, 0.0, 1000.0), square(0.0, 1000.0, 1000.0)],
        };
        // Crossing box in the empty corner: not touching the group.
        assert!(!proj(&c, Pos2::new(690.0, 210.0), Pos2::new(610.0, 290.0)).instance(&g));
        // Crossing box over one leg: touching.
        assert!(proj(&c, Pos2::new(560.0, 210.0), Pos2::new(520.0, 290.0)).instance(&g));
        // Window around all of it.
        assert!(proj(&c, Pos2::new(490.0, 190.0), Pos2::new(710.0, 410.0)).instance(&g));
        assert!(!proj(&c, Pos2::new(490.0, 190.0), Pos2::new(650.0, 410.0)).instance(&g));
    }
}
