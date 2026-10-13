//! Transform tools: Move, Rotate, Scale (with Ctrl copy and `Nx` / `/N` arrays).

use super::common::*;
use super::draw::Clicker;
use super::{OverlayView, Tool, ToolContext, ToolId};
use crate::inference::{self, InferenceEngine, InferenceKind, InferenceResult};
use crate::overlay::{Marker2d, MarkerShape};
use egui::{Color32, CursorIcon, Key};
use varstotas_kernel::glam::{DMat4, DVec2, DVec3};
use varstotas_kernel::{Aabb, EntityRef, Plane, TOL};
use varstotas_render::Overlay3d;

/// A parametric transform that can be repeated/divided for arrays.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum XfKind {
    Translate(DVec3),
    Rotate { center: DVec3, axis: DVec3, deg: f64 },
    Scale { anchor: DVec3, factor: f64 },
}

impl XfKind {
    /// The transform scaled by `k` (k = 1 → the operation itself).
    pub fn at(&self, k: f64) -> DMat4 {
        match *self {
            XfKind::Translate(v) => DMat4::from_translation(v * k),
            XfKind::Rotate { center, axis, deg } => rotation_about(center, axis, deg * k),
            XfKind::Scale { anchor, factor } => {
                let f = factor.powf(k);
                DMat4::from_translation(anchor) * DMat4::from_scale(DVec3::splat(f)) * DMat4::from_translation(-anchor)
            }
        }
    }
}

/// World transforms for the copies of an array.
pub fn array_transforms(kind: &XfKind, arr: Option<ArrayInput>) -> Vec<DMat4> {
    match arr {
        None => vec![kind.at(1.0)],
        Some(ArrayInput::Multiply(n)) => (1..=n).map(|k| kind.at(k as f64)).collect(),
        Some(ArrayInput::Divide(n)) => (1..=n).map(|k| kind.at(k as f64 / n as f64)).collect(),
    }
}

#[derive(Clone, Debug)]
struct LastXf {
    ents: Vec<EntityRef>,
    kind: XfKind,
    copy: bool,
}

/// Apply (or copy / array) a transform to entities of the active context as one undo step.
fn run_transform(
    ctx: &mut ToolContext,
    label: &str,
    ents: &[EntityRef],
    kind: XfKind,
    copy: bool,
    arr: Option<ArrayInput>,
) -> Option<Vec<EntityRef>> {
    let mats: Vec<DMat4> = array_transforms(&kind, arr).iter().map(|t| world_to_local_xf(ctx.doc, t)).collect();
    let ents = ents.to_vec();
    let r = commit_mesh(ctx, label, |mesh| {
        let mut out = Vec::new();
        if copy {
            for m in &mats {
                out.extend(mesh.duplicate(&ents, m));
            }
        } else if let Some(m) = mats.first() {
            out = apply_transform(mesh, &ents, m, false);
        }
        out
    })?;
    if copy {
        ctx.doc.set_selection(r.clone());
    }
    Some(r)
}

fn targets(ctx: &ToolContext, hover: &Option<Hover>) -> Vec<EntityRef> {
    if !ctx.doc.selection.is_empty() {
        return ctx.doc.selection.clone();
    }
    hover.as_ref().map(|h| vec![h.entity()]).unwrap_or_default()
}

fn hover_if_no_selection(ctx: &ToolContext) -> Option<Hover> {
    if ctx.doc.selection.is_empty() { pick_entity(ctx) } else { None }
}

fn draw_preview(view: &OverlayView, ents: &[EntityRef], t: &DMat4, out3d: &mut Overlay3d) {
    for (a, b) in transformed_preview(view.doc, ents, t, 6000) {
        line(out3d, a, b, SELECT_BLUE, 1.5);
    }
}

/// Edit-last / array handling shared by Move and Rotate.
fn vcb_after(ctx: &mut ToolContext, last: &mut Option<LastXf>, input: &str, label: &str) -> Option<bool> {
    let arr = parse_array(input)?;
    let Some(l) = last.clone() else { return Some(false) };
    if !l.copy {
        return Some(false);
    }
    if !undo_last(ctx) {
        return Some(false);
    }
    let ok = run_transform(ctx, label, &l.ents, l.kind, true, Some(arr)).is_some();
    Some(ok)
}

// ===========================================================================
// Move
// ===========================================================================

#[derive(Default)]
pub struct MoveTool {
    engine: InferenceEngine,
    hover: Option<Hover>,
    ents: Vec<EntityRef>,
    from: Option<DVec3>,
    cur: Option<InferenceResult>,
    copy: ModToggle,
    clicker: Clicker,
    last: Option<LastXf>,
    unit: Option<varstotas_kernel::LengthUnit>,
}

impl MoveTool {
    fn update(&mut self, ctx: &ToolContext) {
        self.unit = Some(ctx.doc.model.unit);
        self.copy.update(copy_mod(&ctx.pointer.modifiers));
        if self.from.is_none() {
            self.hover = hover_if_no_selection(ctx);
        }
        self.cur = Some(infer(&mut self.engine, ctx, self.from, None, None));
    }

    fn vector(&self) -> Option<DVec3> {
        Some(self.cur.as_ref()?.point - self.from?)
    }

    fn finish(&mut self, ctx: &mut ToolContext, v: DVec3) {
        let ents = std::mem::take(&mut self.ents);
        self.from = None;
        self.engine.arrow_lock = None;
        if v.length() < TOL || ents.is_empty() {
            return;
        }
        let kind = XfKind::Translate(v);
        let copy = self.copy.on;
        if run_transform(ctx, if copy { "Copy" } else { "Move" }, &ents, kind, copy, None).is_some() {
            self.last = Some(LastXf { ents, kind, copy });
        }
        self.copy.reset();
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        match self.from {
            None => {
                let ents = targets(ctx, &self.hover);
                if ents.is_empty() {
                    return;
                }
                self.ents = ents;
                self.from = self.cur.as_ref().map(|r| r.point);
            }
            Some(_) => {
                if let Some(v) = self.vector() {
                    self.finish(ctx, v);
                }
            }
        }
    }
}

impl Tool for MoveTool {
    fn id(&self) -> ToolId {
        ToolId::Move
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
        if self.clicker.up_is_drag(ctx) && self.from.is_some() {
            self.click(ctx);
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        if key == Key::Escape {
            self.from = None;
            self.ents.clear();
            self.engine.clear_locks();
            return true;
        }
        let r = self.engine.handle_arrow(key);
        if r {
            self.update(ctx);
        }
        r
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        if self.from.is_none()
            && let Some(ok) = vcb_after(ctx, &mut self.last, input, "Copy Array")
        {
            return ok;
        }
        let Some(len) = parse_len(ctx.doc.model.unit, input) else { return false };
        if len.abs() < TOL {
            return false;
        }
        if let Some(v) = self.vector() {
            let dir = v.normalize_or(self.cur.as_ref().and_then(|r| r.constraint_dir).unwrap_or(DVec3::X));
            self.finish(ctx, dir * len);
            return true;
        }
        if let Some(l) = self.last.clone()
            && let XfKind::Translate(v) = l.kind
        {
            if !undo_last(ctx) {
                return false;
            }
            let kind = XfKind::Translate(v.normalize_or(DVec3::X) * len);
            let label = if l.copy { "Copy" } else { "Move" };
            if run_transform(ctx, label, &l.ents, kind, l.copy, None).is_some() {
                self.last = Some(LastXf { kind, ..l });
                return true;
            }
        }
        false
    }
    fn vcb_label(&self) -> &str {
        "Distance"
    }
    fn vcb_value(&self) -> String {
        let d = self.vector().map(|v| v.length()).or(match self.last.as_ref().map(|l| l.kind) {
            Some(XfKind::Translate(v)) => Some(v.length()),
            _ => None,
        });
        match (self.unit, d) {
            (Some(u), Some(d)) => varstotas_kernel::units::format_length(d, u),
            _ => String::new(),
        }
    }
    fn status_text(&self) -> String {
        let copy = if self.copy.on { " (copy)" } else { "" };
        if self.from.is_none() {
            format!(
                "Pick the point to move from{copy}. Moves the selection, or the entity under the cursor. Ctrl/Option = copy; after a copy type `5x` or `/5` for arrays."
            )
        } else {
            format!("Pick the destination or type a distance{copy}. Shift = lock inference, arrows = lock axis.")
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let (Some(from), Some(r)) = (self.from, &self.cur) {
            let t = DMat4::from_translation(r.point - from);
            draw_preview(view, &self.ents, &t, out3d);
            let c = if r.kind.is_direction() { r.kind.color() } else { Color32::from_gray(60) };
            dashed(out3d, from, r.point, c, 1.5);
            label(out2d, (from + r.point) * 0.5, fmt_len(&view.doc.model, from.distance(r.point)));
        } else if let Some(h) = &self.hover {
            highlight_hover(view, h, out3d);
        }
        if let Some(r) = &self.cur {
            inference::draw_result(r, out3d, out2d);
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::Move
    }
    fn is_busy(&self) -> bool {
        self.from.is_some()
    }
}

// ===========================================================================
// Rotate
// ===========================================================================

#[derive(Clone, Copy, Debug)]
enum RotStage {
    Center { c: DVec3, n: DVec3 },
    Angle { c: DVec3, n: DVec3, zero: DVec3 },
}

/// Plane normal for protractors: arrow lock > face under cursor > blue axis.
fn protractor_normal(engine: &InferenceEngine, r: &InferenceResult) -> DVec3 {
    if let Some((d, _)) = engine.arrow_dir() {
        return d;
    }
    r.face.as_ref().map(|f| f.plane.normal).unwrap_or(DVec3::Z)
}

fn on_plane(ctx: &ToolContext, r: &InferenceResult, pl: &Plane) -> DVec3 {
    if r.is_geometry_snap() {
        return pl.project(r.point);
    }
    let ray = ctx.pointer.ray;
    match pl.intersect_ray(&ray) {
        Some(t) if t > 0.0 => ray.at(t),
        _ => pl.project(r.point),
    }
}

/// Protractor overlay shared by Rotate and the Protractor tool.
pub fn draw_protractor(view: &OverlayView, c: DVec3, n: DVec3, zero: Option<DVec3>, out3d: &mut Overlay3d) {
    let px = view.camera.pixel_size_at(c, DVec2::new(view.viewport.x as f64, view.viewport.y as f64));
    let r = px * 45.0;
    let color = normal_color(n);
    let start = zero.unwrap_or(plane_axes(n).0);
    let pts = circle_points(c, n, start, r, 48);
    polyline(out3d, &pts, true, color, 1.5);
    let ticks = circle_points(c, n, start, 1.0, 24);
    for (i, d) in ticks.iter().enumerate() {
        let d = *d - c;
        let inner = if i % 6 == 0 { 0.7 } else { 0.85 };
        line(out3d, c + d * (r * inner), c + d * r, color, 1.0);
    }
}

pub struct RotateTool {
    engine: InferenceEngine,
    hover: Option<Hover>,
    ents: Vec<EntityRef>,
    stage: Option<RotStage>,
    cur: Option<InferenceResult>,
    angle: f64,
    snapped: bool,
    copy: ModToggle,
    clicker: Clicker,
    last: Option<LastXf>,
}

impl Default for RotateTool {
    fn default() -> Self {
        Self {
            engine: InferenceEngine::default(),
            hover: None,
            ents: Vec::new(),
            stage: None,
            cur: None,
            angle: 0.0,
            snapped: false,
            copy: ModToggle::default(),
            clicker: Clicker::default(),
            last: None,
        }
    }
}

impl RotateTool {
    fn update(&mut self, ctx: &ToolContext) {
        self.copy.update(copy_mod(&ctx.pointer.modifiers));
        if self.stage.is_none() {
            self.hover = hover_if_no_selection(ctx);
        }
        let (anchor, plane) = match self.stage {
            Some(RotStage::Center { c, n }) | Some(RotStage::Angle { c, n, .. }) => (Some(c), Some(Plane::new(c, n))),
            None => (None, None),
        };
        let r = infer(&mut self.engine, ctx, anchor, None, plane);
        if let Some(RotStage::Angle { c, n, zero }) = self.stage {
            let q = on_plane(ctx, &r, &Plane::new(c, n));
            let d = q - c;
            if d.length() > 1e-9 {
                let a = signed_angle(zero, d.normalize(), n);
                let (a, s) = if r.is_geometry_snap() { (a, false) } else { snap_angle(a, 15.0, 3.0) };
                self.angle = a;
                self.snapped = s;
            }
        }
        self.cur = Some(r);
    }

    fn finish(&mut self, ctx: &mut ToolContext, c: DVec3, n: DVec3, deg: f64) {
        let ents = std::mem::take(&mut self.ents);
        self.stage = None;
        if deg.abs() < 1e-9 || ents.is_empty() {
            return;
        }
        let kind = XfKind::Rotate { center: c, axis: n, deg };
        let copy = self.copy.on;
        if run_transform(ctx, if copy { "Copy" } else { "Rotate" }, &ents, kind, copy, None).is_some() {
            self.last = Some(LastXf { ents, kind, copy });
        }
        self.copy.reset();
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        let Some(r) = self.cur.clone() else { return };
        match self.stage {
            None => {
                let ents = targets(ctx, &self.hover);
                if ents.is_empty() {
                    return;
                }
                self.ents = ents;
                let n = protractor_normal(&self.engine, &r);
                self.engine.arrow_lock = None;
                self.stage = Some(RotStage::Center { c: r.point, n });
            }
            Some(RotStage::Center { c, n }) => {
                let q = on_plane(ctx, &r, &Plane::new(c, n));
                let d = q - c;
                if d.length() > TOL {
                    self.stage = Some(RotStage::Angle { c, n, zero: d.normalize() });
                    self.angle = 0.0;
                }
            }
            Some(RotStage::Angle { c, n, .. }) => {
                let a = self.angle;
                self.finish(ctx, c, n, a);
            }
        }
    }
}

impl Tool for RotateTool {
    fn id(&self) -> ToolId {
        ToolId::Rotate
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
        if self.clicker.up_is_drag(ctx) && self.stage.is_some() {
            self.click(ctx);
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        if key == Key::Escape {
            self.stage = None;
            self.ents.clear();
            self.engine.clear_locks();
            return true;
        }
        let r = self.engine.handle_arrow(key);
        if r {
            if let (Some(RotStage::Center { c, .. }), Some((d, _))) = (self.stage, self.engine.arrow_dir()) {
                self.stage = Some(RotStage::Center { c, n: d });
                self.engine.arrow_lock = None;
            }
            self.update(ctx);
        }
        r
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        if self.stage.is_none()
            && let Some(ok) = vcb_after(ctx, &mut self.last, input, "Rotate Array")
        {
            return ok;
        }
        let Some(a) = parse_angle(input) else { return false };
        if let Some(RotStage::Angle { c, n, .. }) = self.stage {
            self.finish(ctx, c, n, a);
            return true;
        }
        if self.stage.is_none()
            && let Some(l) = self.last.clone()
            && let XfKind::Rotate { center, axis, .. } = l.kind
        {
            if !undo_last(ctx) {
                return false;
            }
            let kind = XfKind::Rotate { center, axis, deg: a };
            if run_transform(ctx, "Rotate", &l.ents, kind, l.copy, None).is_some() {
                self.last = Some(LastXf { kind, ..l });
                return true;
            }
        }
        false
    }
    fn vcb_label(&self) -> &str {
        "Angle"
    }
    fn vcb_value(&self) -> String {
        match (self.stage, self.last.as_ref().map(|l| l.kind)) {
            (Some(RotStage::Angle { .. }), _) => format!("{:.1}°", self.angle),
            (None, Some(XfKind::Rotate { deg, .. })) => format!("{deg:.1}°"),
            _ => String::new(),
        }
    }
    fn status_text(&self) -> String {
        match self.stage {
            None => "Select the rotation center (face orients the protractor; arrows lock its axis).".into(),
            Some(RotStage::Center { .. }) => "Select the zero direction.".into(),
            Some(RotStage::Angle { .. }) => format!(
                "Select the angle (snaps every 15°) or type degrees. Ctrl/Option = copy{}; after a copy type `6x` for a radial array.",
                if self.copy.on { " (on)" } else { "" }
            ),
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        let r = self.cur.as_ref();
        match self.stage {
            None => {
                if let Some(h) = &self.hover {
                    highlight_hover(view, h, out3d);
                }
                if let Some(r) = r {
                    draw_protractor(view, r.point, protractor_normal(&self.engine, r), None, out3d);
                    inference::draw_result(r, out3d, out2d);
                }
            }
            Some(RotStage::Center { c, n }) => {
                draw_protractor(view, c, n, None, out3d);
                if let Some(r) = r {
                    dashed(out3d, c, r.point, Color32::from_gray(60), 1.5);
                    inference::draw_result(r, out3d, out2d);
                }
            }
            Some(RotStage::Angle { c, n, zero }) => {
                draw_protractor(view, c, n, Some(zero), out3d);
                let px = view.camera.pixel_size_at(c, DVec2::new(view.viewport.x as f64, view.viewport.y as f64));
                let len = px * 70.0;
                dashed(out3d, c, c + zero * len, Color32::from_gray(60), 1.5);
                let cur = DMat4::from_axis_angle(n, self.angle.to_radians()).transform_vector3(zero);
                line(out3d, c, c + cur * len, PREVIEW, 2.0);
                let t = rotation_about(c, n, self.angle);
                draw_preview(view, &self.ents, &t, out3d);
                out2d.push(Marker2d::CursorTooltip { text: format!("{:.1}°", self.angle) });
                out2d.push(Marker2d::Point { at: c, shape: MarkerShape::Ring, color: normal_color(n) });
            }
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::Crosshair
    }
    fn is_busy(&self) -> bool {
        self.stage.is_some()
    }
}

// ===========================================================================
// Scale
// ===========================================================================

#[derive(Clone, Copy, Debug)]
struct ScaleDrag {
    anchor: DVec3,
    grip: DVec3,
    factor: f64,
}

#[derive(Default)]
pub struct ScaleTool {
    engine: InferenceEngine,
    hover: Option<Hover>,
    ents: Vec<EntityRef>,
    bbox: Option<Aabb>,
    hot_grip: Option<DVec3>,
    drag: Option<ScaleDrag>,
    clicker: Clicker,
    last: Option<LastXf>,
}

fn grips(b: &Aabb) -> Vec<DVec3> {
    let mut v: Vec<DVec3> = b.corners().to_vec();
    v.dedup_by(|a, b| a.distance_squared(*b) < 1e-9);
    v
}

impl ScaleTool {
    fn refresh(&mut self, ctx: &ToolContext) {
        if self.drag.is_some() {
            return;
        }
        self.hover = hover_if_no_selection(ctx);
        let ents = targets(ctx, &self.hover);
        if ents != self.ents || self.bbox.is_none() {
            self.bbox = entities_bounds(ctx.doc, &ents);
            self.ents = ents;
        }
        let vp = ctx.viewport_d();
        let c = DVec2::new(ctx.pointer.pos.x as f64, ctx.pointer.pos.y as f64);
        self.hot_grip = self.bbox.as_ref().and_then(|b| {
            grips(b)
                .into_iter()
                .filter_map(|g| Some((ctx.camera.world_to_screen(g, vp)?.distance(c), g)))
                .filter(|(d, _)| *d <= 10.0)
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|x| x.1)
        });
    }

    fn update_factor(&mut self, ctx: &ToolContext) {
        let Some(d) = &mut self.drag else { return };
        let axis = d.grip - d.anchor;
        let l2 = axis.length_squared();
        if l2 < 1e-12 {
            return;
        }
        let r = infer(
            &mut self.engine,
            ctx,
            Some(d.anchor),
            Some(inference::Constraint::Line { origin: d.anchor, dir: axis / l2.sqrt(), kind: InferenceKind::Free }),
            None,
        );
        let f = (r.point - d.anchor).dot(axis) / l2;
        if f.is_finite() && f.abs() > 1e-4 {
            d.factor = f;
        }
    }

    fn finish(&mut self, ctx: &mut ToolContext, anchor: DVec3, factor: f64) {
        self.drag = None;
        if (factor - 1.0).abs() < 1e-9 || factor.abs() < 1e-6 {
            return;
        }
        let kind = XfKind::Scale { anchor, factor };
        let ents = self.ents.clone();
        if run_transform(ctx, "Scale", &ents, kind, false, None).is_some() {
            self.last = Some(LastXf { ents, kind, copy: false });
        }
        self.bbox = None;
    }
}

impl Tool for ScaleTool {
    fn id(&self) -> ToolId {
        ToolId::Scale
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        self.refresh(ctx);
        self.update_factor(ctx);
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if !self.clicker.down(ctx) {
            return;
        }
        self.click(ctx);
    }
    fn on_pointer_up(&mut self, ctx: &mut ToolContext) {
        if self.clicker.up_is_drag(ctx) && self.drag.is_some() {
            self.click(ctx);
        }
    }
    fn on_key(&mut self, _ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if pressed && key == Key::Escape {
            self.drag = None;
            return true;
        }
        false
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        let s = input.trim().replace(',', ".");
        let Ok(f) = s.parse::<f64>() else { return false };
        if !f.is_finite() || f.abs() < 1e-6 {
            return false;
        }
        if let Some(d) = self.drag {
            self.finish(ctx, d.anchor, f);
            return true;
        }
        if let Some(l) = self.last.clone()
            && let XfKind::Scale { anchor, .. } = l.kind
        {
            if !undo_last(ctx) {
                return false;
            }
            let kind = XfKind::Scale { anchor, factor: f };
            if run_transform(ctx, "Scale", &l.ents, kind, false, None).is_some() {
                self.last = Some(LastXf { kind, ..l });
                return true;
            }
        }
        false
    }
    fn vcb_label(&self) -> &str {
        "Scale"
    }
    fn vcb_value(&self) -> String {
        match (self.drag, self.last.as_ref().map(|l| l.kind)) {
            (Some(d), _) => format!("{:.3}", d.factor),
            (None, Some(XfKind::Scale { factor, .. })) => format!("{factor:.3}"),
            _ => String::new(),
        }
    }
    fn status_text(&self) -> String {
        if self.drag.is_some() {
            "Move to scale uniformly about the opposite corner; click or type a factor.".into()
        } else if self.ents.is_empty() {
            "Select entities (or hover one), then drag a corner grip.".into()
        } else {
            "Click a green corner grip to scale.".into()
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let Some(d) = self.drag {
            let t = XfKind::Scale { anchor: d.anchor, factor: d.factor }.at(1.0);
            draw_preview(view, &self.ents, &t, out3d);
            dashed(out3d, d.anchor, d.anchor + (d.grip - d.anchor) * d.factor, Color32::from_gray(60), 1.5);
            out2d.push(Marker2d::Point { at: d.anchor, shape: MarkerShape::Ring, color: Color32::from_rgb(0xFF, 0, 0) });
            out2d.push(Marker2d::CursorTooltip { text: format!("Uniform scale {:.3}", d.factor) });
            return;
        }
        if let Some(h) = &self.hover {
            highlight_hover(view, h, out3d);
        }
        if let Some(b) = &self.bbox {
            let c = b.corners();
            let edges = [(0, 1), (1, 3), (3, 2), (2, 0), (4, 5), (5, 7), (7, 6), (6, 4), (0, 4), (1, 5), (2, 6), (3, 7)];
            for (i, j) in edges {
                line(out3d, c[i], c[j], Color32::from_rgb(0xD0, 0x60, 0x00), 1.0);
            }
            for g in grips(b) {
                let hot = self.hot_grip.is_some_and(|h| h.distance_squared(g) < 1e-9);
                out2d.push(Marker2d::Point {
                    at: g,
                    shape: MarkerShape::Square,
                    color: if hot { Color32::from_rgb(0xFF, 0, 0) } else { Color32::from_rgb(0, 0xC0, 0) },
                });
            }
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::Crosshair
    }
    fn is_busy(&self) -> bool {
        self.drag.is_some()
    }
}

impl ScaleTool {
    fn click(&mut self, ctx: &mut ToolContext) {
        if let Some(d) = self.drag {
            self.update_factor(ctx);
            let f = self.drag.map(|d| d.factor).unwrap_or(d.factor);
            self.finish(ctx, d.anchor, f);
            return;
        }
        self.refresh(ctx);
        if let (Some(g), Some(b)) = (self.hot_grip, self.bbox) {
            let anchor = b.min + b.max - g;
            if anchor.distance(g) > TOL {
                self.drag = Some(ScaleDrag { anchor, grip: g, factor: 1.0 });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_ridge_folds_roof() {
        let mut m = varstotas_kernel::Mesh::default();
        let f = m
            .add_face(&[
                DVec3::ZERO,
                DVec3::new(3000.0, 0.0, 0.0),
                DVec3::new(3000.0, 2000.0, 0.0),
                DVec3::new(0.0, 2000.0, 0.0),
            ])
            .unwrap();
        m.push_pull(f, -2500.0, false).unwrap();
        let ridge = m.add_line(DVec3::new(0.0, 1000.0, 2500.0), DVec3::new(3000.0, 1000.0, 2500.0));
        assert_eq!(ridge.len(), 1);
        let nf = m.faces.len();
        let ents = vec![EntityRef::Edge(ridge[0])];
        apply_transform(&mut m, &ents, &DMat4::from_translation(DVec3::Z * 1000.0), false);
        let top = m.vertices.values().map(|v| v.pos.z).fold(f64::MIN, f64::max);
        assert!(m.faces.len() >= nf);
        assert!((top - 3500.0).abs() < 1e-6);
    }

    #[test]
    fn translate_arrays() {
        let k = XfKind::Translate(DVec3::new(1000.0, 0.0, 0.0));
        let m = array_transforms(&k, Some(ArrayInput::Multiply(3)));
        let xs: Vec<f64> = m.iter().map(|t| t.transform_point3(DVec3::ZERO).x).collect();
        assert_eq!(xs, vec![1000.0, 2000.0, 3000.0]);
        let d = array_transforms(&k, Some(ArrayInput::Divide(4)));
        let xs: Vec<f64> = d.iter().map(|t| t.transform_point3(DVec3::ZERO).x).collect();
        assert_eq!(xs, vec![250.0, 500.0, 750.0, 1000.0]);
        assert_eq!(array_transforms(&k, None).len(), 1);
    }

    #[test]
    fn rotate_arrays() {
        let k = XfKind::Rotate { center: DVec3::ZERO, axis: DVec3::Z, deg: 90.0 };
        let m = array_transforms(&k, Some(ArrayInput::Multiply(3)));
        let p = m[2].transform_point3(DVec3::X);
        assert!((p - DVec3::new(0.0, -1.0, 0.0)).length() < 1e-9);
        let d = array_transforms(&k, Some(ArrayInput::Divide(2)));
        let p = d[0].transform_point3(DVec3::X);
        let h = std::f64::consts::FRAC_1_SQRT_2;
        assert!((p - DVec3::new(h, h, 0.0)).length() < 1e-9);
    }

    #[test]
    fn scale_about_anchor() {
        let k = XfKind::Scale { anchor: DVec3::new(1.0, 1.0, 1.0), factor: 2.0 };
        let p = k.at(1.0).transform_point3(DVec3::new(2.0, 1.0, 1.0));
        assert!((p - DVec3::new(3.0, 1.0, 1.0)).length() < 1e-12);
    }
}
