//! Slice tool: cut the selection (or everything in the active context) with a plane.
//!
//! The plane is set like a Guide Plane: three points; or one point and then double-click /
//! Enter = the plane of the face under it, arrow keys = normal along red / green / blue / the
//! last hovered edge. The cut adds edges along the plane; with Split on (Option / Ctrl tap,
//! "+" on the cursor) every cut body falls apart into two groups, capped where they were cut.

use super::common::*;
use super::draw::Clicker;
use super::{OverlayView, Tool, ToolContext, ToolId, ToolRequest};
use crate::inference::{InferenceEngine, InferenceResult};
use crate::overlay::{Marker2d, MarkerShape};
use egui::{Color32, CursorIcon, Key};
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{Plane, SectionPlane, TOL, WorkPlane};
use varstotas_i18n::tr;
use varstotas_render::{Overlay3d, OverlayPolygon};

const COLOR: Color32 = Color32::from_rgb(0xD6, 0x3A, 0x2A);

#[derive(Clone, Debug)]
enum Stage {
    Idle,
    A { a: DVec3, face_n: Option<DVec3> },
    B { a: DVec3, b: DVec3 },
}

pub struct SliceTool {
    engine: InferenceEngine,
    stage: Stage,
    cur: Option<InferenceResult>,
    clicker: Clicker,
    split: ModToggle,
}

impl Default for SliceTool {
    fn default() -> Self {
        Self { engine: InferenceEngine::default(), stage: Stage::Idle, cur: None, clicker: Clicker::default(), split: ModToggle::default() }
    }
}

/// World normal of the face under an inference result.
fn face_normal(r: &InferenceResult) -> Option<DVec3> {
    let f = r.face.as_ref()?;
    Some(f.plane.normal).filter(|n| n.length() > 0.5)
}

impl SliceTool {
    fn update(&mut self, ctx: &ToolContext) {
        self.split.update(copy_mod(&ctx.pointer.modifiers));
        let ax = crate::axes::current();
        let (anchor, fallback) = match &self.stage {
            Stage::Idle => (None, None),
            Stage::A { a, .. } => (Some(*a), Some(Plane::new(*a, ax.z))),
            Stage::B { a, b } => {
                let d = (*b - *a).normalize_or_zero();
                let n = if d.cross(ax.z).length() > 1e-6 { ax.z - d * ax.z.dot(d) } else { ax.y };
                (Some(*a), Some(Plane::new(*a, n.normalize())))
            }
        };
        self.cur = Some(infer(&mut self.engine, ctx, anchor, None, fallback));
    }

    /// The cutting plane the tool would use now (world).
    fn preview(&self) -> Option<Plane> {
        let p = self.cur.as_ref()?.point;
        let wp = match &self.stage {
            Stage::Idle => return None,
            Stage::A { a, .. } => {
                let ax = crate::axes::current();
                let d = p - *a;
                if d.length() <= TOL || d.cross(ax.z).length() <= 1e-6 * d.length() {
                    return None;
                }
                WorkPlane::from_normal("", *a, d.cross(ax.z), Some(d))?
            }
            Stage::B { a, b } => WorkPlane::from_points("", *a, *b, p)?,
        };
        Some(Plane::new(wp.origin, wp.normal()))
    }

    /// Cut with the world plane `plane`.
    fn cut(&mut self, ctx: &mut ToolContext, plane: Plane) {
        let xf = context_xf(ctx.doc);
        let local = Plane::new(xf.inverse().transform_point3(plane.origin), xf.transpose().transform_vector3(plane.normal));
        let path = ctx.doc.active_path.clone();
        let sel = ctx.doc.selection.clone();
        let split = self.split.on;
        let r = ctx.doc.try_commit_if(
            &if split { tr!("tool-slice-undo-split") } else { tr!("tool-slice-undo") },
            |m| m.slice(&path, &sel, local, split),
            |r| r.edges > 0 || !r.parts.is_empty(),
        );
        if let Some(r) = r {
            if !r.parts.is_empty() {
                ctx.doc.set_selection(r.parts.iter().map(|i| varstotas_kernel::EntityRef::Instance(*i)).collect());
            } else {
                ctx.doc.prune_selection();
            }
            if r.edges == 0 && r.parts.is_empty() {
                ctx.doc.last_error = Some(tr!("tool-slice-nothing"));
            }
        }
        self.stage = Stage::Idle;
        self.engine.clear_locks();
    }

    fn cut_with_normal(&mut self, ctx: &mut ToolContext, n: DVec3) {
        let Stage::A { a, .. } = self.stage else { return };
        if n.length() > 0.5 {
            self.cut(ctx, Plane::new(a, n));
        }
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        let Some(r) = self.cur.clone() else { return };
        match self.stage.clone() {
            Stage::Idle => self.stage = Stage::A { a: r.point, face_n: face_normal(&r) },
            Stage::A { a, .. } => {
                if a.distance(r.point) > TOL {
                    self.stage = Stage::B { a, b: r.point };
                }
            }
            Stage::B { a, b } => {
                if let Some(p) = WorkPlane::from_points("", a, b, r.point) {
                    self.cut(ctx, Plane::new(p.origin, p.normal()));
                }
            }
        }
        self.engine.arrow_lock = None;
    }

    fn cancel(&mut self) {
        self.stage = Stage::Idle;
        self.engine.clear_locks();
    }
}

impl Tool for SliceTool {
    fn id(&self) -> ToolId {
        ToolId::Slice
    }
    fn deactivate(&mut self, _ctx: &mut ToolContext) {
        self.cancel();
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
        if self.clicker.up_is_drag(ctx) && !matches!(self.stage, Stage::Idle) {
            self.click(ctx);
        }
    }
    fn on_double_click(&mut self, ctx: &mut ToolContext) {
        // The first click of the double-click set point A: the plane of the face there.
        if let Stage::A { face_n: Some(n), .. } = self.stage {
            self.cut_with_normal(ctx, n);
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        if key == Key::Escape {
            if matches!(self.stage, Stage::Idle) {
                ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select));
            }
            self.cancel();
            return true;
        }
        if let Stage::A { face_n, .. } = self.stage {
            if key == Key::Enter {
                self.cut_with_normal(ctx, face_n.unwrap_or(crate::axes::current().z));
                return true;
            }
            // Arrows: normal along red / green / blue / the last hovered edge.
            if self.engine.handle_arrow(key) {
                let dir = self.engine.arrow_dir().map(|d| d.0);
                self.engine.arrow_lock = None;
                if let Some(n) = dir {
                    self.cut_with_normal(ctx, n);
                }
                return true;
            }
        }
        let r = self.engine.handle_arrow(key);
        if r {
            self.update(ctx);
        }
        r
    }
    fn status_text(&self) -> String {
        let step = match self.stage {
            Stage::Idle => "tool-slice-status-first",
            Stage::A { .. } => "tool-slice-status-second",
            Stage::B { .. } => "tool-slice-status-third",
        };
        tr!("tool-slice-status", step = tr!(step), split = if self.split.on { "on" } else { "off" })
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let Some(p) = self.preview() {
            let c = SectionPlane::new(String::new(), p.origin, p.normal).rect(view.doc.model.bounds());
            for i in 0..4 {
                line(out3d, c[i], c[(i + 1) % 4], COLOR, 1.5);
            }
            out3d.polygons.push(OverlayPolygon {
                tris: [c[0], c[1], c[2], c[0], c[2], c[3]].map(|p| p.as_vec3().to_array()).to_vec(),
                color: [0xD6 as f32 / 255.0, 0x3A as f32 / 255.0, 0x2A as f32 / 255.0, 0.12],
                on_top: false,
            });
        }
        if let Stage::A { a, .. } | Stage::B { a, .. } = self.stage {
            out2d.push(Marker2d::Point { at: a, shape: MarkerShape::Dot, color: COLOR });
        }
        if let Stage::B { a, b } = self.stage {
            line(out3d, a, b, COLOR, 1.5);
        }
        if let Some(r) = &self.cur {
            crate::inference::draw_result(r, out3d, out2d);
        }
    }
    fn cursor_plus(&self) -> bool {
        self.split.on
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        !matches!(self.stage, Stage::Idle)
    }
}
