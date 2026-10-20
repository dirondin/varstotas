//! Texture Position: move, rotate and scale a face's texture with pins, like the usual
//! Texture → Position. Red pin (or dragging the face) moves the texture, green rotates and
//! scales it uniformly about the red pin, blue scales its height.

use super::common::{Hover, context_xf, pick_face};
use super::{OverlayView, Tool, ToolContext, ToolId, ToolRequest};
use crate::overlay::{Marker2d, MarkerShape};
use egui::{Color32, CursorIcon, Key};
use varstotas_kernel::glam::{DMat4, DVec2, DVec3};
use varstotas_kernel::texture::texture_axes;
use varstotas_kernel::{EntityRef, FaceId, Model, TexPlacement};
use varstotas_render::Overlay3d;

/// Pick radius of a pin, points.
const PIN_PX: f32 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Grab {
    Move,
    Green,
    Blue,
}

struct Drag {
    grab: Grab,
    /// Placement when the drag started (`None` = default projection).
    orig: Option<TexPlacement>,
    /// Planar point (mm) where the drag started.
    start: DVec2,
}

#[derive(Default)]
pub struct TexturePositionTool {
    face: Option<FaceId>,
    drag: Option<Drag>,
    hover_pin: Option<Grab>,
}

/// Face geometry for the tool: context transform, planar axes, plane offset, tile size.
struct Frame {
    xf: DMat4,
    u: DVec3,
    v: DVec3,
    n: DVec3,
    /// `n · p` for points of the face (local coordinates).
    d: f64,
    tile: [f64; 2],
    placement: TexPlacement,
}

impl Frame {
    fn of(model: &Model, doc: &crate::document::Document, face: FaceId) -> Option<Frame> {
        let mesh = model.mesh_at(&doc.active_path)?;
        let f = mesh.faces.get(face)?;
        let tile = [f.front, f.back]
            .into_iter()
            .flatten()
            .find_map(|id| model.materials.get(id)?.image.as_ref().map(|i| i.size))?;
        let (u, v) = texture_axes(f.normal);
        let p0 = mesh.vertices.get(*f.outer.first()?)?.pos;
        Some(Frame {
            xf: context_xf(doc),
            u,
            v,
            n: f.normal,
            d: f.normal.dot(p0),
            tile,
            placement: f.uv.unwrap_or_default(),
        })
    }

    /// World point of planar coordinates `q`.
    fn world(&self, q: DVec2) -> DVec3 {
        self.xf.transform_point3(self.u * q.x + self.v * q.y + self.n * self.d)
    }

    /// Planar coordinates where `ray` meets the face plane.
    fn planar_at(&self, ray: &varstotas_kernel::Ray) -> Option<DVec2> {
        let inv = self.xf.inverse();
        let o = inv.transform_point3(ray.origin);
        let dir = inv.transform_vector3(ray.dir);
        let den = self.n.dot(dir);
        if den.abs() < 1e-12 {
            return None;
        }
        let t = (self.d - self.n.dot(o)) / den;
        let p = o + dir * t;
        Some(DVec2::new(p.dot(self.u), p.dot(self.v)))
    }

    /// Planar positions of the red, green and blue pins (tile origin and its u / v corners).
    fn pins(&self, pl: &TexPlacement) -> [DVec2; 3] {
        [
            pl.to_planar(DVec2::ZERO),
            pl.to_planar(DVec2::new(self.tile[0], 0.0)),
            pl.to_planar(DVec2::new(0.0, self.tile[1])),
        ]
    }
}

/// New placement for dragging `grab` from `start` to `q` (planar mm), starting at `o`.
fn dragged(grab: Grab, o: TexPlacement, tile: [f64; 2], start: DVec2, q: DVec2) -> TexPlacement {
    let mut p = o;
    match grab {
        Grab::Move => {
            let off = DVec2::from(o.offset) + (q - start);
            p.offset = off.to_array();
        }
        Grab::Green => {
            let d = q - DVec2::from(o.offset);
            if d.length() < 1e-6 {
                return o;
            }
            let flip = if o.scale[0] < 0.0 { 180.0 } else { 0.0 };
            p.angle_deg = d.y.atan2(d.x).to_degrees() - flip;
            let sx = d.length() / tile[0] * o.scale[0].signum();
            p.scale = [sx, o.scale[1] * (sx / o.scale[0]).abs()];
        }
        Grab::Blue => {
            let (s, c) = o.angle_deg.to_radians().sin_cos();
            let vdir = DVec2::new(-s, c);
            let h = (q - DVec2::from(o.offset)).dot(vdir) / tile[1];
            if h.abs() < 1e-3 {
                return o;
            }
            p.scale[1] = h;
        }
    }
    // Pointer rays are f32-precise: keep the stored numbers tidy (0.01 mm, 0.01°, 1e-4 scale).
    let r = |x: f64, k: f64| (x * k).round() / k;
    p.offset = [r(p.offset[0], 100.0), r(p.offset[1], 100.0)];
    p.angle_deg = r(p.angle_deg, 100.0);
    p.scale = [r(p.scale[0], 1e4), r(p.scale[1], 1e4)];
    p
}

impl TexturePositionTool {
    fn frame(&self, ctx: &ToolContext) -> Option<Frame> {
        Frame::of(&ctx.doc.model, ctx.doc, self.face?)
    }

    /// The textured face under the pointer (active context).
    fn textured_face_under(ctx: &ToolContext) -> Option<FaceId> {
        match pick_face(ctx)? {
            Hover::Face { face, .. } => Frame::of(&ctx.doc.model, ctx.doc, face).map(|_| face),
            _ => None,
        }
    }

    fn pin_under(&self, ctx: &ToolContext, fr: &Frame) -> Option<Grab> {
        let pins = fr.pins(&fr.placement);
        let at = ctx.pointer.pos;
        [(Grab::Green, pins[1]), (Grab::Blue, pins[2]), (Grab::Move, pins[0])]
            .into_iter()
            .find(|(_, q)| ctx.to_screen(fr.world(*q)).is_some_and(|s| s.distance(at) <= PIN_PX))
            .map(|(g, _)| g)
    }

    fn set_untracked(ctx: &mut ToolContext, face: FaceId, p: Option<TexPlacement>) {
        let path = ctx.doc.active_path.clone();
        ctx.doc.mutate_untracked(|m| {
            if let Some(f) = m.mesh_at_mut(&path).and_then(|mesh| mesh.faces.get_mut(face)) {
                f.uv = p.filter(|p| !p.is_identity());
            }
        });
    }

    fn cancel_drag(&mut self, ctx: &mut ToolContext) {
        if let (Some(d), Some(face)) = (self.drag.take(), self.face) {
            Self::set_untracked(ctx, face, d.orig);
        }
    }
}

impl Tool for TexturePositionTool {
    fn id(&self) -> ToolId {
        ToolId::TexturePosition
    }

    fn activate(&mut self, ctx: &mut ToolContext) {
        if let [EntityRef::Face(f)] = ctx.doc.selection.as_slice()
            && Frame::of(&ctx.doc.model, ctx.doc, *f).is_some()
        {
            self.face = Some(*f);
        }
    }

    fn deactivate(&mut self, ctx: &mut ToolContext) {
        self.cancel_drag(ctx);
    }

    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        let Some(fr) = self.frame(ctx) else {
            self.face = None;
            self.hover_pin = None;
            return;
        };
        if let (Some(d), Some(face)) = (self.drag.as_ref(), self.face) {
            if let Some(q) = fr.planar_at(&ctx.pointer.ray) {
                let o = d.orig.unwrap_or_default();
                let p = dragged(d.grab, o, fr.tile, d.start, q);
                Self::set_untracked(ctx, face, Some(p));
            }
        } else {
            self.hover_pin = self.pin_under(ctx, &fr);
        }
    }

    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button != Some(egui::PointerButton::Primary) {
            return;
        }
        let under = Self::textured_face_under(ctx);
        let fr = self.frame(ctx);
        let grab = fr.as_ref().and_then(|fr| self.pin_under(ctx, fr)).or_else(|| (under.is_some() && under == self.face).then_some(Grab::Move));
        match (grab, fr) {
            (Some(grab), Some(fr)) => {
                if let Some(start) = fr.planar_at(&ctx.pointer.ray) {
                    let orig = ctx.doc.model.mesh_at(&ctx.doc.active_path).and_then(|m| m.faces.get(self.face?)).and_then(|f| f.uv);
                    self.drag = Some(Drag { grab, orig, start });
                }
            }
            _ => match under {
                Some(f) => {
                    self.face = Some(f);
                    ctx.doc.set_selection(vec![EntityRef::Face(f)]);
                }
                // Click outside a textured face: done.
                None => ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select)),
            },
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        let (Some(d), Some(face)) = (self.drag.take(), self.face) else { return };
        let path = ctx.doc.active_path.clone();
        let now = ctx.doc.model.mesh_at(&path).and_then(|m| m.faces.get(face)).and_then(|f| f.uv);
        if now == d.orig {
            return;
        }
        // One undo step: back to the original, then commit the result.
        Self::set_untracked(ctx, face, d.orig);
        ctx.commit("Texture Position", |m| {
            if let Some(f) = m.mesh_at_mut(&path).and_then(|mesh| mesh.faces.get_mut(face)) {
                f.uv = now;
            }
        });
    }

    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        match key {
            Key::Escape if self.drag.is_some() => {
                self.cancel_drag(ctx);
                true
            }
            Key::Escape | Key::Enter => {
                ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select));
                true
            }
            _ => false,
        }
    }

    fn status_text(&self) -> String {
        match (self.face, self.drag.as_ref().map(|d| d.grab)) {
            (None, _) => "Click a textured face to position its texture.".into(),
            (_, Some(Grab::Move)) => "Moving the texture. Esc cancels.".into(),
            (_, Some(Grab::Green)) => "Rotating and scaling the texture about the red pin.".into(),
            (_, Some(Grab::Blue)) => "Scaling the texture height.".into(),
            _ => "Drag the face or the red pin to move, the green pin to rotate/scale, the blue pin to stretch. Enter or click outside when done.".into(),
        }
    }

    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        let Some(fr) = self.face.and_then(|f| Frame::of(&view.doc.model, view.doc, f)) else { return };
        let pl = fr.placement;
        let corner = |a: f64, b: f64| fr.world(pl.to_planar(DVec2::new(a * fr.tile[0], b * fr.tile[1])));
        let c = [corner(0.0, 0.0), corner(1.0, 0.0), corner(1.0, 1.0), corner(0.0, 1.0)];
        let line = |a: DVec3, b: DVec3, color: [f32; 4], width_px: f32, dashed: bool| varstotas_render::OverlayLine {
            a: a.as_vec3().to_array(),
            b: b.as_vec3().to_array(),
            color,
            width_px,
            dashed,
            on_top: true,
        };
        let yellow = [1.0, 0.85, 0.1, 1.0];
        for i in 0..4 {
            out3d.lines.push(line(c[i], c[(i + 1) % 4], yellow, 2.0, false));
        }
        // Neighbouring tiles, dashed, to show the repeat.
        for (a, b) in [((-1.0, 0.0), (2.0, 0.0)), ((-1.0, 1.0), (2.0, 1.0)), ((0.0, -1.0), (0.0, 2.0)), ((1.0, -1.0), (1.0, 2.0))] {
            out3d.lines.push(line(corner(a.0, a.1), corner(b.0, b.1), [1.0, 0.85, 0.1, 0.6], 1.0, true));
        }
        let pins = fr.pins(&pl);
        for (q, color, grab) in [
            (pins[0], Color32::from_rgb(220, 40, 40), Grab::Move),
            (pins[1], Color32::from_rgb(40, 170, 60), Grab::Green),
            (pins[2], Color32::from_rgb(40, 90, 220), Grab::Blue),
        ] {
            let active = self.hover_pin == Some(grab) || self.drag.as_ref().is_some_and(|d| d.grab == grab);
            out2d.push(Marker2d::Point { at: fr.world(q), shape: if active { MarkerShape::Square } else { MarkerShape::Dot }, color });
        }
    }

    fn cursor(&self) -> CursorIcon {
        match (self.drag.as_ref().map(|d| d.grab), self.hover_pin) {
            (Some(_), _) => CursorIcon::Grabbing,
            (None, Some(_)) => CursorIcon::Grab,
            _ => CursorIcon::Default,
        }
    }

    fn cursor_glyph(&self) -> Option<Marker2d> {
        None
    }

    fn is_busy(&self) -> bool {
        self.drag.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn green_pin_rotates_and_scales_uniformly() {
        let o = TexPlacement { offset: [100.0, 0.0], angle_deg: 0.0, scale: [1.0, 2.0] };
        // Green pin (tile 400 wide) dragged straight up by 800 mm: 90°, scale ×2 on both axes.
        let p = dragged(Grab::Green, o, [400.0, 200.0], DVec2::new(500.0, 0.0), DVec2::new(100.0, 800.0));
        assert!((p.angle_deg - 90.0).abs() < 1e-9 && (p.scale[0] - 2.0).abs() < 1e-9 && (p.scale[1] - 4.0).abs() < 1e-9, "{p:?}");
        assert!((p.to_planar(DVec2::new(400.0, 0.0)) - DVec2::new(100.0, 800.0)).length() < 1e-9);
    }

    #[test]
    fn blue_pin_and_move() {
        let o = TexPlacement { offset: [0.0, 0.0], angle_deg: 90.0, scale: [1.0, 1.0] };
        // v direction is -x when rotated 90°.
        let p = dragged(Grab::Blue, o, [400.0, 200.0], DVec2::ZERO, DVec2::new(-300.0, 50.0));
        assert!((p.scale[1] - 1.5).abs() < 1e-9 && p.scale[0] == 1.0, "{p:?}");
        let m = dragged(Grab::Move, o, [400.0, 200.0], DVec2::new(10.0, 10.0), DVec2::new(15.0, 30.0));
        assert_eq!(m.offset, [5.0, 20.0]);
    }
}
