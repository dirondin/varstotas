//! Minimal Select tool: click / double / triple click, window & crossing selection,
//! modifier semantics, entering/leaving group contexts.

use super::{OverlayView, Tool, ToolContext, ToolId};
use crate::document::guarded;
use crate::overlay::Marker2d;
use egui::{Color32, CursorIcon, Key, PointerButton, Pos2, Rect};
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{ContextPath, EntityRef};
use varstotas_render::Overlay3d;

const EDGE_PICK_PX: f32 = 6.0;
const DRAG_THRESHOLD_PX: f32 = 4.0;

#[derive(Default)]
pub struct SelectTool {
    press: Option<Pos2>,
    box_end: Option<Pos2>,
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
        let m = ctx.pointer.modifiers;
        let add = m.command || m.ctrl;
        let picked = self.pick(ctx);
        let Some(p) = picked else {
            if !add && !m.shift {
                ctx.doc.clear_selection();
            }
            return;
        };
        if p.context != ctx.doc.active_path {
            ctx.doc.set_active_path(p.context);
        }
        match (add, m.shift) {
            (true, true) => ctx.doc.remove_from_selection(&[p.entity]),
            (true, false) => ctx.doc.add_to_selection([p.entity]),
            (false, true) => ctx.doc.toggle_selected(p.entity),
            (false, false) => ctx.doc.set_selection(vec![p.entity]),
        }
    }

    fn box_select(&mut self, ctx: &mut ToolContext, a: Pos2, b: Pos2) {
        let rect = Rect::from_two_pos(a, b);
        let crossing = b.x < a.x;
        let active = ctx.doc.active_path.clone();
        let found = guarded(|| {
            let model = &ctx.doc.model;
            let mut out = Vec::new();
            let Some(mesh) = model.mesh_at(&active) else {
                return out;
            };
            let xf = model.transform_at(&active);
            let proj = |p: DVec3| ctx.to_screen(xf.transform_point3(p));
            let poly_test = |pts: &[Option<Pos2>], closed: bool| -> bool {
                if pts.iter().any(|p| p.is_none()) {
                    return false;
                }
                let pts: Vec<Pos2> = pts.iter().map(|p| p.unwrap_or_default()).collect();
                if crossing {
                    let n = pts.len();
                    let segs = if closed { n } else { n.saturating_sub(1) };
                    pts.iter().any(|p| rect.contains(*p))
                        || (0..segs).any(|i| segment_hits_rect(pts[i], pts[(i + 1) % n], rect))
                } else {
                    pts.iter().all(|p| rect.contains(*p))
                }
            };
            for (id, e) in mesh.edges.iter() {
                if e.hidden {
                    continue;
                }
                let pts = [proj(mesh.vertices[e.v[0]].pos), proj(mesh.vertices[e.v[1]].pos)];
                if poly_test(&pts, false) {
                    out.push(EntityRef::Edge(id));
                }
            }
            for (id, f) in mesh.faces.iter() {
                if f.hidden {
                    continue;
                }
                let pts: Vec<Option<Pos2>> = f.outer.iter().map(|v| proj(mesh.vertices[*v].pos)).collect();
                if poly_test(&pts, true) {
                    out.push(EntityRef::Face(id));
                }
            }
            for (id, inst) in mesh.instances.iter() {
                if inst.hidden {
                    continue;
                }
                let Some(def) = model.defs.get(inst.def) else { continue };
                let Some(bb) = def.mesh.bounds() else { continue };
                let m = xf * inst.transform;
                let pts: Vec<Option<Pos2>> = (0..8)
                    .map(|i| {
                        let c = DVec3::new(
                            if i & 1 == 0 { bb.min.x } else { bb.max.x },
                            if i & 2 == 0 { bb.min.y } else { bb.max.y },
                            if i & 4 == 0 { bb.min.z } else { bb.max.z },
                        );
                        ctx.to_screen(m.transform_point3(c))
                    })
                    .collect();
                let hit = if crossing {
                    pts.iter().all(|p| p.is_some()) && {
                        let r = Rect::from_points(&pts.iter().map(|p| p.unwrap_or_default()).collect::<Vec<_>>());
                        r.intersects(rect)
                    }
                } else {
                    poly_test(&pts, false)
                };
                if hit {
                    out.push(EntityRef::Instance(id));
                }
            }
            out
        })
        .unwrap_or_default();

        let m = ctx.pointer.modifiers;
        let add = m.command || m.ctrl;
        match (add, m.shift) {
            (true, true) => ctx.doc.remove_from_selection(&found),
            (true, false) => ctx.doc.add_to_selection(found),
            (false, true) => {
                for e in found {
                    ctx.doc.toggle_selected(e);
                }
            }
            (false, false) => ctx.doc.set_selection(found),
        }
    }
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
            "Drag left→right for window selection, right→left for crossing selection.".into()
        } else {
            "Select objects. Ctrl/Cmd = add, Shift = toggle. Drag to select multiple. Double-click a group to edit it."
                .into()
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
