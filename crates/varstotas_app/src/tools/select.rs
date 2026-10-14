//! Minimal Select tool: click / double / triple click, window & crossing selection,
//! modifier semantics, entering/leaving group contexts.

use super::{OverlayView, Tool, ToolContext, ToolId};
use crate::document::guarded;
use crate::overlay::Marker2d;
use egui::{Color32, CursorIcon, Key, PointerButton, Pos2, Rect};
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::{ContextPath, EntityRef};
use varstotas_render::Overlay3d;

const EDGE_PICK_PX: f32 = 6.0;
const DRAG_THRESHOLD_PX: f32 = 4.0;

#[derive(Default)]
pub struct SelectTool {
    press: Option<Pos2>,
    box_end: Option<Pos2>,
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
pub const SEL_MODS_HELP: &str = if cfg!(target_os = "macos") {
    "Option (or Cmd) = add, Shift = toggle, Shift+Option = remove"
} else {
    "Ctrl = add, Shift = toggle, Ctrl+Shift = remove"
};

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
        let hit = hit?;
        if starts_with(&hit.path, &active) {
            let entity = if hit.path.len() > active.len() {
                EntityRef::Instance(hit.path[active.len()])
            } else {
                EntityRef::Face(hit.face)
            };
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
            Some(Picked { context, entity })
        }
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        let op = sel_op(&ctx.pointer.modifiers);
        let picked = self.pick(ctx);
        let Some(p) = picked else {
            if op == SelOp::Replace {
                ctx.doc.clear_selection();
            }
            return;
        };
        if p.context != ctx.doc.active_path {
            ctx.doc.set_active_path(p.context);
        }
        match op {
            SelOp::Remove => ctx.doc.remove_from_selection(&[p.entity]),
            SelOp::Add => ctx.doc.add_to_selection([p.entity]),
            SelOp::Toggle => ctx.doc.toggle_selected(p.entity),
            SelOp::Replace => ctx.doc.set_selection(vec![p.entity]),
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

        match sel_op(&ctx.pointer.modifiers) {
            SelOp::Remove => ctx.doc.remove_from_selection(&found),
            SelOp::Add => ctx.doc.add_to_selection(found),
            SelOp::Toggle => ctx.doc.toggle_many(&found),
            SelOp::Replace => ctx.doc.set_selection(found),
        }
    }
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

    fn deactivate(&mut self, _ctx: &mut ToolContext) {
        self.press = None;
        self.box_end = None;
    }

    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button == Some(PointerButton::Primary) {
            self.press = Some(ctx.pointer.pos);
            self.box_end = None;
        }
    }

    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
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
        match (self.press.take(), self.box_end.take()) {
            (Some(a), Some(b)) => self.box_select(ctx, a, b),
            (Some(_), None) => self.click(ctx),
            _ => {}
        }
    }

    fn on_double_click(&mut self, ctx: &mut ToolContext) {
        let Some(p) = self.pick(ctx) else { return };
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
        if self.press.is_some() {
            self.press = None;
            self.box_end = None;
        } else if !ctx.doc.selection.is_empty() {
            ctx.doc.clear_selection();
        } else {
            ctx.doc.exit_context();
        }
        true
    }

    fn status_text(&self) -> String {
        if self.box_end.is_some() {
            format!("Drag left→right for window selection, right→left for crossing selection. {SEL_MODS_HELP}.")
        } else {
            format!("Select objects. {SEL_MODS_HELP}. Drag to select multiple. Double-click a group to edit it.")
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
        self.press.is_some()
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
