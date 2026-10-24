//! Texture Position: move, rotate and scale a face's texture with pins, like the usual
//! Texture → Position. Red pin (or dragging the face) moves the texture, green rotates and
//! scales it uniformly about the red pin, blue scales its height. Pins snap to the face's
//! vertices and edges. Several faces are edited together: the texture continues from the
//! picked face across shared edges (and onto coplanar faces). Alt+click continues the texture
//! onto another face, Shift+click adds / removes a face, a double-click on an edge aligns the
//! texture to it; the Measurements box sets the tile size on the face or the angle.

use varstotas_i18n::tr;
use super::common::{Hover, context_xf, pick_face};
use super::{OverlayView, Tool, ToolContext, ToolId, ToolRequest};
use crate::overlay::{Marker2d, MarkerShape};
use egui::{Color32, CursorIcon, Key, Pos2};
use std::sync::Arc;
use varstotas_kernel::glam::{DMat4, DVec2, DVec3};
use varstotas_kernel::texture::{align_to_edge, continue_placement, texture_axes, wrap_placements};
use varstotas_kernel::{EntityRef, FaceId, Model, TexPlacement};
use varstotas_render::Overlay3d;

/// Pick radius of a pin, points.
const PIN_PX: f32 = 10.0;
/// Snap radius of a pin to a vertex / an edge, points.
const SNAP_VERTEX_PX: f32 = 12.0;
const SNAP_EDGE_PX: f32 = 7.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Grab {
    Move,
    Green,
    Blue,
}

struct Drag {
    grab: Grab,
    /// Placements of the edited faces when the drag started (the picked face first).
    orig: Vec<(FaceId, Option<TexPlacement>)>,
    /// Planar point (mm) where the drag started (the red pin itself when it was grabbed).
    start: DVec2,
    /// A pin is dragged (it snaps), not the face.
    pin: bool,
}

#[derive(Default)]
pub struct TexturePositionTool {
    face: Option<FaceId>,
    /// Faces edited together with `face` (the texture continues from it).
    others: Vec<FaceId>,
    drag: Option<Drag>,
    hover_pin: Option<Grab>,
    /// Where the dragged pin snapped (world), for the marker.
    snap: Option<DVec3>,
    /// Tile size on the picked face, for the Measurements box.
    vcb: String,
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
    /// The image shown on the face (full-tile overlay).
    image: Arc<[u8]>,
}

impl Frame {
    fn of(model: &Model, doc: &crate::document::Document, face: FaceId) -> Option<Frame> {
        let mesh = model.mesh_at(&doc.active_path)?;
        let f = mesh.faces.get(face)?;
        let img = [f.front, f.back].into_iter().flatten().find_map(|id| model.materials.get(id)?.image.as_ref())?;
        let (u, v) = texture_axes(f.normal);
        let p0 = mesh.vertices.get(*f.outer.first()?)?.pos;
        Some(Frame {
            xf: context_xf(doc),
            u,
            v,
            n: f.normal,
            d: f.normal.dot(p0),
            tile: img.size,
            placement: f.uv.unwrap_or_default(),
            image: img.bytes.clone(),
        })
    }

    /// World point of planar coordinates `q`.
    fn world(&self, q: DVec2) -> DVec3 {
        self.xf.transform_point3(self.u * q.x + self.v * q.y + self.n * self.d)
    }

    /// Planar coordinates of a world point (projected onto the face plane).
    fn planar_of(&self, w: DVec3) -> DVec2 {
        let p = self.xf.inverse().transform_point3(w);
        DVec2::new(p.dot(self.u), p.dot(self.v))
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
            p.angle_deg = d.y.atan2(d.x).to_degrees() + o.shear.atan().to_degrees() - flip;
            let sx = d.length() / (tile[0] * (1.0 + o.shear * o.shear).sqrt()) * o.scale[0].signum();
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
    tidy(p)
}

/// Pointer rays are f32-precise: keep the stored numbers tidy (0.01 mm, 0.01°, 1e-4 scale,
/// no -0).
fn tidy(mut p: TexPlacement) -> TexPlacement {
    let r = |x: f64, k: f64| (x * k).round() / k + 0.0;
    p.offset = [r(p.offset[0], 100.0), r(p.offset[1], 100.0)];
    p.angle_deg = r(p.angle_deg, 100.0);
    p.scale = [r(p.scale[0], 1e4), r(p.scale[1], 1e4)];
    p
}

/// The placement for a Measurements entry: `<width>` (height follows), `<width>;<height>`
/// (also `x` / `×`) as the tile size on the face, or `<angle>°` / `<angle>deg`.
fn vcb_placement(input: &str, p: TexPlacement, tile: [f64; 2], unit: varstotas_kernel::LengthUnit) -> Option<TexPlacement> {
    let t = input.trim();
    if let Some(a) = t.strip_suffix('°').or_else(|| t.strip_suffix("deg")) {
        let a: f64 = a.trim().replace(',', ".").parse().ok()?;
        return Some(TexPlacement { angle_deg: a, ..p });
    }
    let list = t.replace(['x', '×', 'X'], ";");
    let v = varstotas_kernel::units::parse_lengths(&list, unit)?;
    let sign = |x: f64| if x < 0.0 { -1.0 } else { 1.0 };
    let mut q = p;
    match v[..] {
        [w] if w > 0.0 => {
            let s0 = w / tile[0] * sign(p.scale[0]);
            q.scale = [s0, p.scale[1] * (s0 / p.scale[0]).abs()];
        }
        [w, h] if w > 0.0 && h > 0.0 => q.scale = [w / tile[0] * sign(p.scale[0]), h / tile[1] * sign(p.scale[1])],
        _ => return None,
    }
    Some(q)
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

    /// The edited faces, the picked one first.
    fn set(&self) -> Vec<FaceId> {
        self.face.into_iter().chain(self.others.iter().copied()).collect()
    }

    /// Edges (world segments) of the edited faces.
    fn edges(&self, ctx: &ToolContext) -> Vec<(DVec3, DVec3)> {
        let Some(mesh) = ctx.doc.model.mesh_at(&ctx.doc.active_path) else { return Vec::new() };
        let xf = context_xf(ctx.doc);
        let mut out = Vec::new();
        for f in self.set() {
            for e in mesh.face_edges(f) {
                if let Some(ed) = mesh.edges.get(e) {
                    out.push((xf.transform_point3(mesh.vertices[ed.v[0]].pos), xf.transform_point3(mesh.vertices[ed.v[1]].pos)));
                }
            }
        }
        out
    }

    /// The pointer snapped to a vertex, else an edge, of the edited faces (planar on `fr`).
    fn snapped(&self, ctx: &ToolContext, fr: &Frame, q: DVec2) -> (DVec2, Option<DVec3>) {
        let at = ctx.pointer.pos;
        let edges = self.edges(ctx);
        let near = |w: DVec3| ctx.to_screen(w).map(|s| s.distance(at));
        if let Some(w) = edges
            .iter()
            .flat_map(|(a, b)| [*a, *b])
            .filter_map(|w| near(w).filter(|d| *d <= SNAP_VERTEX_PX).map(|d| (w, d)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|x| x.0)
        {
            return (fr.planar_of(w), Some(w));
        }
        let mut best: Option<(DVec3, f32)> = None;
        for (a, b) in &edges {
            let (Some(sa), Some(sb)) = (ctx.to_screen(*a), ctx.to_screen(*b)) else { continue };
            let (t, d) = seg_param(sa, sb, at);
            if d <= SNAP_EDGE_PX && best.is_none_or(|x| d < x.1) {
                best = Some((a.lerp(*b, t as f64), d));
            }
        }
        match best {
            Some((w, _)) => (fr.planar_of(w), Some(w)),
            None => (q, None),
        }
    }

    /// Placements of every edited face for placement `p` of the picked face.
    fn placements(&self, ctx: &ToolContext, p: Option<TexPlacement>) -> Vec<(FaceId, Option<TexPlacement>)> {
        let Some(face) = self.face else { return Vec::new() };
        let p = p.filter(|p| !p.is_identity());
        let mut out = vec![(face, p)];
        if !self.others.is_empty()
            && let Some(mesh) = ctx.doc.model.mesh_at(&ctx.doc.active_path)
        {
            out.extend(wrap_placements(mesh, &self.set(), face, p).into_iter().map(|(f, p)| (f, p.map(tidy))));
        }
        out
    }

    fn set_untracked(ctx: &mut ToolContext, list: &[(FaceId, Option<TexPlacement>)]) {
        let path = ctx.doc.active_path.clone();
        ctx.doc.mutate_untracked(|m| {
            if let Some(mesh) = m.mesh_at_mut(&path) {
                for (f, p) in list {
                    if let Some(face) = mesh.faces.get_mut(*f) {
                        face.uv = p.filter(|p| !p.is_identity());
                    }
                }
            }
        });
    }

    /// One undo step setting the placements in `list`.
    fn commit(ctx: &mut ToolContext, list: Vec<(FaceId, Option<TexPlacement>)>) {
        let path = ctx.doc.active_path.clone();
        ctx.commit(&tr!("tool-undo-texture-position"), |m| {
            if let Some(mesh) = m.mesh_at_mut(&path) {
                for (f, p) in list {
                    if let Some(face) = mesh.faces.get_mut(f) {
                        face.uv = p.filter(|p| !p.is_identity());
                    }
                }
            }
        });
    }

    fn cancel_drag(&mut self, ctx: &mut ToolContext) {
        if let Some(d) = self.drag.take() {
            Self::set_untracked(ctx, &d.orig);
        }
        self.snap = None;
    }

    fn current(ctx: &ToolContext, f: FaceId) -> Option<TexPlacement> {
        ctx.doc.model.mesh_at(&ctx.doc.active_path).and_then(|m| m.faces.get(f)).and_then(|f| f.uv)
    }

    fn update_vcb(&mut self, ctx: &ToolContext) {
        self.vcb = match self.frame(ctx) {
            Some(fr) => {
                let unit = ctx.doc.model.unit;
                let p = fr.placement;
                format!(
                    "{}; {}",
                    varstotas_kernel::units::format_length(fr.tile[0] * p.scale[0].abs(), unit),
                    varstotas_kernel::units::format_length(fr.tile[1] * p.scale[1].abs(), unit)
                )
            }
            None => String::new(),
        };
    }

    /// Select the edited faces (they show which faces move together).
    fn select_set(&self, ctx: &mut ToolContext) {
        ctx.doc.set_selection(self.set().into_iter().map(EntityRef::Face).collect());
    }
}

/// Parameter (0..1) of the point of segment `a`–`b` closest to `p`, and its distance.
fn seg_param(a: Pos2, b: Pos2, p: Pos2) -> (f32, f32) {
    let ab = b - a;
    let l2 = ab.length_sq();
    let t = if l2 < 1e-6 { 0.0 } else { ((p - a).dot(ab) / l2).clamp(0.0, 1.0) };
    (t, (a + ab * t).distance(p))
}

impl Tool for TexturePositionTool {
    fn id(&self) -> ToolId {
        ToolId::TexturePosition
    }

    fn activate(&mut self, ctx: &mut ToolContext) {
        let faces: Vec<FaceId> = ctx
            .doc
            .selection
            .iter()
            .filter_map(|e| if let EntityRef::Face(f) = e { Some(*f) } else { None })
            .filter(|f| Frame::of(&ctx.doc.model, ctx.doc, *f).is_some())
            .collect();
        if let Some((first, rest)) = faces.split_first() {
            self.face = Some(*first);
            self.others = rest.to_vec();
        }
        self.update_vcb(ctx);
    }

    fn deactivate(&mut self, ctx: &mut ToolContext) {
        self.cancel_drag(ctx);
    }

    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        let Some(fr) = self.frame(ctx) else {
            self.face = None;
            self.others.clear();
            self.hover_pin = None;
            return;
        };
        if let Some(d) = self.drag.as_ref() {
            if let Some(q) = fr.planar_at(&ctx.pointer.ray) {
                let (q, snap) = if d.pin { self.snapped(ctx, &fr, q) } else { (q, None) };
                let o = d.orig[0].1.unwrap_or_default();
                let p = dragged(d.grab, o, fr.tile, d.start, q);
                let list = self.placements(ctx, Some(p));
                Self::set_untracked(ctx, &list);
                self.snap = snap;
            }
        } else {
            self.hover_pin = self.pin_under(ctx, &fr);
        }
        self.update_vcb(ctx);
    }

    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if ctx.pointer.button != Some(egui::PointerButton::Primary) {
            return;
        }
        let under = Self::textured_face_under(ctx);
        let fr = self.frame(ctx);
        let mods = ctx.pointer.modifiers;
        // Alt+click: continue the picked face's texture onto another face (it joins the set)
        if mods.alt
            && let (Some(face), Some(g)) = (self.face, under)
            && g != face
        {
            let src = Self::current(ctx, face);
            let p = ctx.doc.model.mesh_at(&ctx.doc.active_path).and_then(|m| continue_placement(m, face, src.as_ref(), g));
            match p {
                Some(p) => {
                    Self::commit(ctx, vec![(g, p.map(tidy))]);
                    if !self.others.contains(&g) {
                        self.others.push(g);
                    }
                    self.select_set(ctx);
                }
                None => ctx.requests.push(ToolRequest::Flash(tr!("tool-texture-cant-continue"))),
            }
            return;
        }
        // Shift+click: add / remove a face
        if mods.shift
            && let Some(g) = under
        {
            if self.face.is_none() {
                self.face = Some(g);
            } else if Some(g) != self.face {
                match self.others.iter().position(|x| *x == g) {
                    Some(i) => {
                        self.others.remove(i);
                    }
                    None => self.others.push(g),
                }
            }
            self.select_set(ctx);
            return;
        }
        let pin = fr.as_ref().and_then(|fr| self.pin_under(ctx, fr));
        let on_set = under.is_some_and(|g| self.set().contains(&g));
        let grab = pin.or(on_set.then_some(Grab::Move));
        match (grab, fr) {
            (Some(grab), Some(fr)) => {
                if let Some(q) = fr.planar_at(&ctx.pointer.ray) {
                    let orig: Vec<(FaceId, Option<TexPlacement>)> = self.set().into_iter().map(|f| (f, Self::current(ctx, f))).collect();
                    // a grabbed red pin follows the pointer exactly (so it can snap)
                    let start = if pin == Some(Grab::Move) { DVec2::from(fr.placement.offset) } else { q };
                    self.drag = Some(Drag { grab, orig, start, pin: pin.is_some() });
                }
            }
            _ => match under {
                Some(f) => {
                    self.face = Some(f);
                    self.others.clear();
                    self.select_set(ctx);
                    self.update_vcb(ctx);
                }
                // Click outside a textured face: done.
                None => ctx.requests.push(ToolRequest::SwitchTool(ToolId::Select)),
            },
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        self.snap = None;
        let Some(d) = self.drag.take() else { return };
        let now: Vec<(FaceId, Option<TexPlacement>)> = d.orig.iter().map(|(f, _)| (*f, Self::current(ctx, *f))).collect();
        if now == d.orig {
            return;
        }
        // One undo step: back to the original, then commit the result.
        Self::set_untracked(ctx, &d.orig);
        Self::commit(ctx, now);
    }

    /// A double-click on an edge of the picked face aligns the texture to it (the red pin at
    /// the edge's nearer end).
    fn on_double_click(&mut self, ctx: &mut ToolContext) {
        let (Some(face), Some(mesh)) = (self.face, ctx.doc.model.mesh_at(&ctx.doc.active_path)) else { return };
        let Some(f) = mesh.faces.get(face) else { return };
        let xf = context_xf(ctx.doc);
        let at = ctx.pointer.pos;
        let mut best: Option<(DVec3, DVec3, f32, f32)> = None;
        for lp in f.loops() {
            for i in 0..lp.len() {
                let (a, b) = (mesh.vertices[lp[i]].pos, mesh.vertices[lp[(i + 1) % lp.len()]].pos);
                let (Some(sa), Some(sb)) = (ctx.to_screen(xf.transform_point3(a)), ctx.to_screen(xf.transform_point3(b))) else { continue };
                let (t, d) = seg_param(sa, sb, at);
                if d <= SNAP_VERTEX_PX && best.is_none_or(|x| d < x.2) {
                    best = Some((a, b, d, t));
                }
            }
        }
        let Some((a, b, _, t)) = best else { return };
        let (a, b) = if t <= 0.5 { (a, b) } else { (b, a) };
        // the red pin at the nearer end; the texture keeps the edge direction closer to its
        // current one (it doesn't turn upside down)
        let mut p = align_to_edge(f.normal, f.uv.as_ref(), a, b);
        let was = f.uv.map_or(0.0, |u| u.angle_deg);
        if ((p.angle_deg - was + 540.0).rem_euclid(360.0) - 180.0).abs() > 90.0 {
            p.angle_deg = align_to_edge(f.normal, f.uv.as_ref(), b, a).angle_deg;
        }
        let p = tidy(p);
        let list = self.placements(ctx, Some(p));
        Self::commit(ctx, list);
        self.update_vcb(ctx);
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

    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        let Some(fr) = self.frame(ctx) else { return false };
        let Some(p) = vcb_placement(input, fr.placement, fr.tile, ctx.doc.model.unit) else { return false };
        let list = self.placements(ctx, Some(tidy(p)));
        Self::commit(ctx, list);
        self.update_vcb(ctx);
        true
    }

    fn vcb_label(&self) -> String {
        tr!("tool-texture-vcb")
    }

    fn vcb_value(&self) -> String {
        self.vcb.clone()
    }

    fn status_text(&self) -> String {
        match (self.face, self.drag.as_ref().map(|d| d.grab)) {
            (None, _) => tr!("tool-texture-pick"),
            (_, Some(Grab::Move)) => tr!("tool-texture-move"),
            (_, Some(Grab::Green)) => tr!("tool-texture-green"),
            (_, Some(Grab::Blue)) => tr!("tool-texture-blue"),
            _ if !self.others.is_empty() => tr!("tool-texture-idle-many", count = self.others.len() + 1),
            _ => tr!("tool-texture-idle"),
        }
    }

    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        let Some(fr) = self.face.and_then(|f| Frame::of(&view.doc.model, view.doc, f)) else { return };
        let pl = fr.placement;
        let corner = |a: f64, b: f64| fr.world(pl.to_planar(DVec2::new(a * fr.tile[0], b * fr.tile[1])));
        let c = [corner(0.0, 0.0), corner(1.0, 0.0), corner(1.0, 1.0), corner(0.0, 1.0)];
        // the whole tile, faint, also where it runs past the face
        out2d.push(Marker2d::TileImage { bytes: fr.image.clone(), corners: c, alpha: 0.45 });
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
        // the other faces of the set: outlined
        if let Some(mesh) = view.doc.model.mesh_at(&view.doc.active_path) {
            for g in &self.others {
                let Some(f) = mesh.faces.get(*g) else { continue };
                for lp in f.loops() {
                    for i in 0..lp.len() {
                        let a = fr.xf.transform_point3(mesh.vertices[lp[i]].pos);
                        let b = fr.xf.transform_point3(mesh.vertices[lp[(i + 1) % lp.len()]].pos);
                        out3d.lines.push(line(a, b, [1.0, 0.85, 0.1, 0.8], 1.5, true));
                    }
                }
            }
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
        if let Some(w) = self.snap {
            out2d.push(Marker2d::Point { at: w, shape: MarkerShape::Ring, color: Color32::from_rgb(0, 160, 160) });
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
    fn green_pin_preserves_imported_shear_when_rotating_and_scaling() {
        let o = TexPlacement { offset: [100.0, 200.0], angle_deg: 20.0, scale: [2.0, 3.0], shear: 0.75 };
        let tile = [500.0, 500.0];
        let green = o.to_planar(DVec2::new(tile[0], 0.0));
        let same = dragged(Grab::Green, o, tile, green, green);
        assert!((same.angle_deg - o.angle_deg).abs() < 1e-9);
        assert!((same.scale[0] - o.scale[0]).abs() < 1e-9);
        let d = green - DVec2::from(o.offset);
        let target = DVec2::from(o.offset) + DVec2::new(-d.y, d.x) * 2.0;
        let changed = dragged(Grab::Green, o, tile, green, target);
        assert!((changed.angle_deg - 110.0).abs() < 1e-9);
        assert_eq!(changed.scale, [4.0, 6.0]);
        assert_eq!(changed.shear, o.shear);
        assert!(changed.to_planar(DVec2::new(tile[0], 0.0)).distance(target) < 1e-6);
    }

    #[test]
    fn green_pin_rotates_and_scales_uniformly() {
        let o = TexPlacement { offset: [100.0, 0.0], angle_deg: 0.0, scale: [1.0, 2.0], shear: 0.0 };
        // Green pin (tile 400 wide) dragged straight up by 800 mm: 90°, scale ×2 on both axes.
        let p = dragged(Grab::Green, o, [400.0, 200.0], DVec2::new(500.0, 0.0), DVec2::new(100.0, 800.0));
        assert!((p.angle_deg - 90.0).abs() < 1e-9 && (p.scale[0] - 2.0).abs() < 1e-9 && (p.scale[1] - 4.0).abs() < 1e-9, "{p:?}");
        assert!((p.to_planar(DVec2::new(400.0, 0.0)) - DVec2::new(100.0, 800.0)).length() < 1e-9);
    }

    #[test]
    fn blue_pin_and_move() {
        let o = TexPlacement { offset: [0.0, 0.0], angle_deg: 90.0, scale: [1.0, 1.0], shear: 0.0 };
        // v direction is -x when rotated 90°.
        let p = dragged(Grab::Blue, o, [400.0, 200.0], DVec2::ZERO, DVec2::new(-300.0, 50.0));
        assert!((p.scale[1] - 1.5).abs() < 1e-9 && p.scale[0] == 1.0, "{p:?}");
        let m = dragged(Grab::Move, o, [400.0, 200.0], DVec2::new(10.0, 10.0), DVec2::new(15.0, 30.0));
        assert_eq!(m.offset, [5.0, 20.0]);
    }

    #[test]
    fn measurements_set_tile_size_and_angle() {
        use varstotas_kernel::LengthUnit;
        let p = TexPlacement { offset: [5.0, 0.0], angle_deg: 10.0, scale: [-1.0, 2.0], shear: 0.0 };
        let tile = [400.0, 200.0];
        let q = vcb_placement("800", p, tile, LengthUnit::Millimeter).unwrap();
        assert_eq!(q.scale, [-2.0, 4.0]);
        let q = vcb_placement("1m x 100", p, tile, LengthUnit::Millimeter).unwrap();
        assert_eq!(q.scale, [-2.5, 0.5]);
        let q = vcb_placement("45°", p, tile, LengthUnit::Millimeter).unwrap();
        assert_eq!((q.angle_deg, q.scale, q.offset), (45.0, p.scale, p.offset));
        assert!(vcb_placement("0", p, tile, LengthUnit::Millimeter).is_none());
    }
}
