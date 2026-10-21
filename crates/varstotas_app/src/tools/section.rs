//! Section Plane tool, plus drawing and picking of the model's section planes.
//!
//! Click a face: a section plane is placed on it, facing into the model (its arrows point to
//! the side that stays visible), and becomes the active cut.

use varstotas_i18n::tr;
use super::{OverlayView, Tool, ToolContext, ToolId, ToolRequest};
use crate::document::Document;
use crate::overlay::Marker2d;
use egui::{Color32, CursorIcon, Key, Pos2};
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::{Aabb, SectionPlane};
use varstotas_render::{Camera, Overlay3d, OverlayLine, OverlayPolygon};

const ACTIVE: Color32 = Color32::from_rgb(0xE8, 0x7A, 0x1E);
const INACTIVE: Color32 = Color32::from_rgb(0x80, 0x80, 0x80);
const CUT: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const FILL: [f32; 4] = [0.25, 0.25, 0.27, 1.0];

fn rgba(c: Color32, a: f32) -> [f32; 4] {
    [c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, a]
}

fn push(out: &mut Overlay3d, a: DVec3, b: DVec3, color: [f32; 4], width_px: f32, on_top: bool) {
    out.lines.push(OverlayLine {
        a: a.as_vec3().to_array(),
        b: b.as_vec3().to_array(),
        color,
        width_px,
        dashed: false,
        on_top,
    });
}

/// Draw one plane: outline, a faint fill and four arrows along its normal.
pub fn draw_plane(s: &SectionPlane, bounds: Option<Aabb>, color: Color32, width: f32, out: &mut Overlay3d) {
    let c = s.rect(bounds);
    let col = rgba(color, 1.0);
    for i in 0..4 {
        push(out, c[i], c[(i + 1) % 4], col, width, false);
    }
    out.polygons.push(OverlayPolygon {
        tris: [c[0], c[1], c[2], c[0], c[2], c[3]].map(|p| p.as_vec3().to_array()).to_vec(),
        color: rgba(color, 0.08),
        on_top: false,
    });
    // Arrows a little inside each corner.
    let size = (c[1] - c[0]).length().min((c[3] - c[0]).length());
    let len = size * 0.08;
    let mid = (c[0] + c[2]) * 0.5;
    for k in c {
        let p = k + (mid - k) * 0.06;
        let tip = p + s.normal * len;
        push(out, p, tip, col, width, false);
        let side = (mid - k).normalize_or_zero() * len * 0.25;
        push(out, tip, tip - s.normal * len * 0.3 + side, col, width, false);
        push(out, tip, tip - s.normal * len * 0.3 - side, col, width, false);
    }
}

/// All section planes (when shown): selected in blue, the active one orange, others grey.
pub fn draw_sections(view: &OverlayView, out: &mut Overlay3d) {
    let m = &view.doc.model;
    if !m.section_display.planes || m.sections.is_empty() {
        return;
    }
    let bounds = m.bounds();
    for (i, s) in m.sections.iter().enumerate() {
        let (color, w) = if view.doc.section_sel == Some(i) {
            (super::common::SELECT_BLUE, 2.0)
        } else if s.active {
            (ACTIVE, 1.5)
        } else {
            (INACTIVE, 1.0)
        };
        draw_plane(s, bounds, color, w, out);
    }
}

/// Cut lines and fill of the active cut (computed by the caller, cached).
pub fn draw_cut(lines: &[[DVec3; 2]], fill: &[[DVec3; 3]], out: &mut Overlay3d) {
    for [a, b] in lines {
        push(out, *a, *b, CUT, 2.5, false);
    }
    if !fill.is_empty() {
        out.polygons.push(OverlayPolygon {
            tris: fill.iter().flat_map(|t| t.map(|p| p.as_vec3().to_array())).collect(),
            color: FILL,
            on_top: false,
        });
    }
}

/// The section plane whose outline is under `pos` (screen points), if planes are shown.
pub fn pick_section_at(doc: &Document, cam: &Camera, viewport: egui::Vec2, pos: Pos2) -> Option<usize> {
    let m = &doc.model;
    if !m.section_display.planes {
        return None;
    }
    let vp = DVec2::new(viewport.x as f64, viewport.y as f64);
    let c = DVec2::new(pos.x as f64, pos.y as f64);
    let bounds = m.bounds();
    let mut best: Option<(f64, usize)> = None;
    for (i, s) in m.sections.iter().enumerate() {
        let r = s.rect(bounds);
        for k in 0..4 {
            let (Some(a), Some(b)) = (cam.world_to_screen(r[k], vp), cam.world_to_screen(r[(k + 1) % 4], vp)) else {
                continue;
            };
            let d = crate::inference::geo::dist_point_seg2(c, a, b).0;
            if d <= crate::inference::EDGE_PICK_PX && best.is_none_or(|b| d < b.0) {
                best = Some((d, i));
            }
        }
    }
    best.map(|b| b.1)
}

/// Plane for a click: on the hovered face (any context), facing into the model.
fn plane_under(ctx: &ToolContext) -> Option<SectionPlane> {
    let hit = ctx.pick()?;
    let m = &ctx.doc.model;
    let f = m.mesh_at(&hit.path)?.faces.get(hit.face)?;
    let xf = m.transform_at(&hit.path);
    let n = xf.inverse().transpose().transform_vector3(f.normal).normalize_or_zero();
    if n == DVec3::ZERO {
        return None;
    }
    // Snap near-axis normals so planes stay exactly axis-aligned.
    let snap = |n: DVec3| {
        let a = n.abs();
        if a.x > 0.9999 {
            DVec3::X * n.x.signum()
        } else if a.y > 0.9999 {
            DVec3::Y * n.y.signum()
        } else if a.z > 0.9999 {
            DVec3::Z * n.z.signum()
        } else {
            n
        }
    };
    let name = format!("Section {}", m.sections.len() + 1); // i18n-ignore
    Some(SectionPlane::new(name, hit.point, -snap(n)))
}

#[derive(Default)]
pub struct SectionTool {
    preview: Option<SectionPlane>,
}

impl Tool for SectionTool {
    fn id(&self) -> ToolId {
        ToolId::Section
    }

    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        self.preview = plane_under(ctx);
    }

    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button != Some(egui::PointerButton::Primary) {
            return;
        }
        let Some(s) = plane_under(ctx) else { return };
        let r = ctx.commit(&tr!("tool-undo-section-plane"), |m| {
            m.sections.push(s);
            let i = m.sections.len() - 1;
            m.set_active_section(Some(i));
            m.section_display.planes = true;
            m.section_display.cuts = true;
            i
        });
        if let Some(i) = r {
            ctx.doc.select_section(i);
        }
        self.preview = None;
    }

    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if pressed && key == Key::Escape {
            ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select));
            return true;
        }
        false
    }

    fn status_text(&self) -> String {
        tr!("tool-section-status")
    }

    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, _out2d: &mut Vec<Marker2d>) {
        if let Some(s) = &self.preview {
            draw_plane(s, view.doc.model.bounds(), ACTIVE, 1.5, out3d);
        }
    }

    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
}
