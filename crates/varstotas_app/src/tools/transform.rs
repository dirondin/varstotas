//! Transform tools: Move, Rotate, Scale (with Ctrl copy and `Nx` / `/N` arrays).

use super::common::*;
use super::draw::Clicker;
use super::{OverlayView, Tool, ToolContext, ToolId};
use crate::inference::{self, InferenceEngine, InferenceKind, InferenceResult};
use crate::overlay::{Marker2d, MarkerShape};
use egui::{Color32, CursorIcon, Key};
use varstotas_kernel::glam::{DMat4, DVec2, DVec3};
use varstotas_kernel::{Aabb, EntityRef, GuideRef, Plane, TOL};
use varstotas_render::Overlay3d;

/// A parametric transform that can be repeated/divided for arrays.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum XfKind {
    Translate(DVec3),
    Rotate { center: DVec3, axis: DVec3, deg: f64 },
    /// Per-axis (world X/Y/Z) factors about `anchor`.
    Scale { anchor: DVec3, factor: DVec3 },
}

impl XfKind {
    /// The transform scaled by `k` (k = 1 → the operation itself).
    pub fn at(&self, k: f64) -> DMat4 {
        match *self {
            XfKind::Translate(v) => DMat4::from_translation(v * k),
            XfKind::Rotate { center, axis, deg } => rotation_about(center, axis, deg * k),
            XfKind::Scale { anchor, factor } => {
                let f = DVec3::new(factor.x.powf(k), factor.y.powf(k), factor.z.powf(k));
                DMat4::from_translation(anchor) * DMat4::from_scale(f) * DMat4::from_translation(-anchor)
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

/// What a transform tool operates on: entities of the active context and/or guides.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Targets {
    pub ents: Vec<EntityRef>,
    pub guides: Vec<GuideRef>,
    pub dims: Vec<varstotas_kernel::DimId>,
    /// A section plane (index into `model.sections`).
    pub section: Option<usize>,
}

impl Targets {
    pub fn is_empty(&self) -> bool {
        self.ents.is_empty() && self.guides.is_empty() && self.dims.is_empty() && self.section.is_none()
    }
    fn ents(ents: Vec<EntityRef>) -> Self {
        Self { ents, ..Default::default() }
    }
}

#[derive(Clone, Debug)]
struct LastXf {
    tg: Targets,
    kind: XfKind,
    copy: bool,
    /// The array applied after the copy (`5x` / `/5`); a later distance/angle edit keeps it
    /// and only changes the spacing.
    arr: Option<ArrayInput>,
}

/// Apply (or copy / array) a transform to entities of the active context and to guides
/// (world space) as one undo step. Returns the transformed (or new) targets.
fn run_transform(
    ctx: &mut ToolContext,
    label: &str,
    tg: &Targets,
    kind: XfKind,
    copy: bool,
    arr: Option<ArrayInput>,
) -> Option<Targets> {
    let world = array_transforms(&kind, arr);
    let mats: Vec<DMat4> = world.iter().map(|t| world_to_local_xf(ctx.doc, t)).collect();
    let (ents, guides, dims) = (tg.ents.clone(), tg.guides.clone(), tg.dims.clone());
    let path = ctx.doc.active_path.clone();
    let uid = ctx.doc.uid;
    let section = tg.section;
    let r = ctx.doc.commit(label, |model| {
        let mut out = Targets::default();
        if let Some(s) = section.and_then(|i| model.sections.get(i)).cloned() {
            if copy {
                for w in &world {
                    let n = model.sections.len() + 1;
                    model.sections.push(varstotas_kernel::SectionPlane { name: format!("Section {n}"), active: false, ..s.transformed(w) });
                }
                out.section = Some(model.sections.len() - 1);
            } else if let (Some(i), Some(w)) = (section, world.first()) {
                model.sections[i] = s.transformed(w);
                out.section = section;
            }
        }
        if copy {
            for w in &world {
                out.guides.extend(model.copy_guides_in(&path, &guides, w));
            }
        } else if let Some(w) = world.first() {
            model.transform_guides_in(&path, &guides, w);
            out.guides = guides.clone();
        }
        let dims_before: Vec<_> = if copy {
            Vec::new()
        } else {
            let w = world.first().copied().unwrap_or(DMat4::IDENTITY);
            dims.iter().map(|id| (*id, dim_after_transform(model, &path, *id, &ents, &w))).collect()
        };
        // Dimensions: anchors on moved vertices follow them; free anchors move here. Copies
        // are made after the geometry so they attach to the copied vertices.
        let dims_after = |model: &mut varstotas_kernel::Model, out: &mut Targets| {
            if dims.is_empty() {
                return;
            }
            if copy {
                for w in &world {
                    out.dims.extend(model.copy_dimensions_in(&path, &dims, w));
                }
            } else if let Some(w) = world.first() {
                // `dims_before` was computed before the geometry moved.
                model.transform_dimensions_in(&path, &dims, w);
                for (id, g) in &dims_before {
                    if let Some(g) = g {
                        model.set_dimension_offset_in(&path, *id, g.offset);
                    }
                }
                out.dims = dims.clone();
            }
        };
        if ents.is_empty() {
            dims_after(model, &mut out);
            return out;
        }
        // Copies weld into the context (coincident faces / edges merge), which can kill refs
        // of the source and of earlier copies. So copies are made from a snapshot of the
        // source entities (like Paste), never from the live, possibly welded, refs.
        let clip = if copy { crate::clipboard::Clipboard::from_parts(model, uid, &path, &ents, &[]).ok() } else { None };
        if let Some(clip) = clip {
            for w in &world {
                if let Ok((e, _)) = clip.paste_into(model, uid, &path, w) {
                    out.ents.extend(e);
                }
            }
        } else if let Some(mesh) = model.mesh_at_mut(&path) {
            if copy {
                for m in &mats {
                    out.ents.extend(mesh.duplicate(&ents, m));
                }
            } else if let Some(m) = mats.first() {
                out.ents = apply_transform(mesh, &ents, m, false);
            }
        }
        // Keep only refs that survived all welds (set_selection filters too).
        if let Some(mesh) = model.mesh_at(&path) {
            let mut seen = std::collections::HashSet::new();
            out.ents.retain(|e| crate::document::entity_alive(mesh, *e) && seen.insert(*e));
        }
        dims_after(model, &mut out);
        out
    })?;
    if copy {
        ctx.doc.set_selection_full(r.ents.clone(), r.guides.clone(), r.dims.clone());
        if let Some(i) = r.section {
            ctx.doc.select_section(i);
        }
    }
    Some(r)
}

/// The selection (entities + guides), else the hovered entity or guide.
fn targets(ctx: &ToolContext, hover: &Option<Hover>, hover_guide: Option<GuideRef>) -> Targets {
    if ctx.doc.has_selection() {
        return Targets {
            ents: ctx.doc.selection.clone(),
            guides: ctx.doc.guide_sel.clone(),
            dims: ctx.doc.dim_sel.clone(),
            section: ctx.doc.section_sel,
        };
    }
    Targets {
        ents: hover.as_ref().map(|h| vec![h.entity()]).unwrap_or_default(),
        guides: hover_guide.into_iter().collect(),
        dims: Vec::new(),
        section: None,
    }
}

fn hover_if_no_selection(ctx: &ToolContext) -> Option<Hover> {
    if ctx.doc.has_selection() { None } else { pick_entity(ctx) }
}

/// Hovered entity or guide when nothing is selected: edges of the context first, then guides,
/// then faces / instances.
fn hover_with_guides(ctx: &ToolContext) -> (Option<Hover>, Option<GuideRef>) {
    if ctx.doc.has_selection() {
        return (None, None);
    }
    if let Some(e) = pick_edge_at(ctx, ctx.pointer.pos, inference::EDGE_PICK_PX as f32) {
        return (Some(e), None);
    }
    if let Some(g) = pick_guide(ctx) {
        return (None, Some(g));
    }
    (pick_face(ctx), None)
}

fn draw_preview(view: &OverlayView, tg: &Targets, t: &DMat4, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
    for (a, b) in transformed_preview(view.doc, &tg.ents, t, 6000) {
        line(out3d, a, b, SELECT_BLUE, 1.5);
    }
    for g in &tg.guides {
        draw_guide(view, *g, t, SELECT_BLUE, 1.5, out3d, out2d);
    }
    for d in &tg.dims {
        draw_dim_moved(view, *d, &tg.ents, t, out3d);
    }
    if let Some(s) = tg.section.and_then(|i| view.doc.model.sections.get(i)) {
        super::section::draw_plane(&s.transformed(t), view.doc.model.bounds(), SELECT_BLUE, 1.5, out3d);
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
    let ok = run_transform(ctx, label, &l.tg, l.kind, true, Some(arr)).is_some();
    if ok {
        *last = Some(LastXf { arr: Some(arr), ..l });
    }
    Some(ok)
}

// ===========================================================================
// Move
// ===========================================================================

#[derive(Default)]
pub struct MoveTool {
    engine: InferenceEngine,
    hover: Option<Hover>,
    hover_guide: Option<GuideRef>,
    tg: Targets,
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
            (self.hover, self.hover_guide) = hover_with_guides(ctx);
        }
        self.cur = Some(infer(&mut self.engine, ctx, self.from, None, None));
    }

    fn vector(&self) -> Option<DVec3> {
        Some(self.cur.as_ref()?.point - self.from?)
    }

    fn finish(&mut self, ctx: &mut ToolContext, v: DVec3) {
        let tg = std::mem::take(&mut self.tg);
        self.from = None;
        self.engine.arrow_lock = None;
        if v.length() < TOL || tg.is_empty() {
            return;
        }
        let kind = XfKind::Translate(v);
        let copy = self.copy.on;
        if run_transform(ctx, if copy { "Copy" } else { "Move" }, &tg, kind, copy, None).is_some() {
            self.last = Some(LastXf { tg, kind, copy, arr: None });
        }
        self.copy.reset();
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        match self.from {
            None => {
                let tg = targets(ctx, &self.hover, self.hover_guide);
                if tg.is_empty() {
                    return;
                }
                self.tg = tg;
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
            self.tg = Targets::default();
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
            let label = match (l.copy, l.arr) {
                (true, Some(_)) => "Copy Array",
                (true, None) => "Copy",
                _ => "Move",
            };
            if run_transform(ctx, label, &l.tg, kind, l.copy, l.arr).is_some() {
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
            draw_preview(view, &self.tg, &t, out3d, out2d);
            let c = if r.kind.is_direction() { r.kind.color() } else { Color32::from_gray(60) };
            dashed(out3d, from, r.point, c, 1.5);
            label(out2d, (from + r.point) * 0.5, fmt_len(&view.doc.model, from.distance(r.point)));
        } else if let Some(h) = &self.hover {
            highlight_hover(view, h, out3d);
        } else if let Some(g) = self.hover_guide {
            draw_guide(view, g, &DMat4::IDENTITY, SELECT_BLUE, 2.0, out3d, out2d);
        }
        if let Some(r) = &self.cur {
            inference::draw_result(r, out3d, out2d);
        }
    }
    fn cursor_plus(&self) -> bool {
        self.copy.on
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
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
    r.face.as_ref().map(|f| f.plane.normal).unwrap_or(crate::axes::current().z)
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
    hover_guide: Option<GuideRef>,
    tg: Targets,
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
            hover_guide: None,
            tg: Targets::default(),
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
            (self.hover, self.hover_guide) = hover_with_guides(ctx);
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
        let tg = std::mem::take(&mut self.tg);
        self.stage = None;
        if deg.abs() < 1e-9 || tg.is_empty() {
            return;
        }
        let kind = XfKind::Rotate { center: c, axis: n, deg };
        let copy = self.copy.on;
        if run_transform(ctx, if copy { "Copy" } else { "Rotate" }, &tg, kind, copy, None).is_some() {
            self.last = Some(LastXf { tg, kind, copy, arr: None });
        }
        self.copy.reset();
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        let Some(r) = self.cur.clone() else { return };
        match self.stage {
            None => {
                let tg = targets(ctx, &self.hover, self.hover_guide);
                if tg.is_empty() {
                    return;
                }
                self.tg = tg;
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
            self.tg = Targets::default();
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
            let label = if l.arr.is_some() { "Rotate Array" } else { "Rotate" };
            if run_transform(ctx, label, &l.tg, kind, l.copy, l.arr).is_some() {
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
            (Some(RotStage::Angle { .. }), _) => fmt_angle(self.angle),
            (None, Some(XfKind::Rotate { deg, .. })) => fmt_angle(deg),
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
                } else if let Some(g) = self.hover_guide {
                    draw_guide(view, g, &DMat4::IDENTITY, SELECT_BLUE, 2.0, out3d, out2d);
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
                draw_preview(view, &self.tg, &t, out3d, out2d);
                out2d.push(Marker2d::CursorTooltip { text: fmt_angle(self.angle) });
                out2d.push(Marker2d::Point { at: c, shape: MarkerShape::Ring, color: normal_color(n) });
            }
        }
    }
    fn cursor_plus(&self) -> bool {
        self.copy.on
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        self.stage.is_some()
    }
}

// ===========================================================================
// Scale
// ===========================================================================

/// A bounding-box grip: `idx` ∈ {0,1,2}³ (0 = min, 1 = middle, 2 = max per axis).
/// Axes whose index is not 1 are scaled: corners scale 3 axes (uniformly), edge midpoints
/// 2 axes, face centres 1 axis.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Grip {
    idx: [u8; 3],
    pos: DVec3,
}

impl Grip {
    fn axes(&self) -> [bool; 3] {
        [self.idx[0] != 1, self.idx[1] != 1, self.idx[2] != 1]
    }
    fn count(&self) -> usize {
        self.axes().iter().filter(|a| **a).count()
    }
}

fn grip_pos(b: &Aabb, idx: [u8; 3]) -> DVec3 {
    let e = b.max - b.min;
    b.min + DVec3::new(e.x * idx[0] as f64, e.y * idx[1] as f64, e.z * idx[2] as f64) * 0.5
}

/// All grips of a box; axes with (near) zero extent cannot be scaled and are skipped.
fn grips(b: &Aabb) -> Vec<Grip> {
    let e = b.max - b.min;
    let flat = [e.x.abs() < TOL, e.y.abs() < TOL, e.z.abs() < TOL];
    let mut v = Vec::new();
    for i in 0..3u8 {
        for j in 0..3u8 {
            for k in 0..3u8 {
                let idx = [i, j, k];
                if idx == [1, 1, 1] || (0..3).any(|a| flat[a] && idx[a] != 1) {
                    continue;
                }
                v.push(Grip { idx, pos: grip_pos(b, idx) });
            }
        }
    }
    v
}

/// Anchor for a grip: the opposite grip, or the box centre (Ctrl).
fn grip_anchor(b: &Aabb, g: &Grip, about_center: bool) -> DVec3 {
    if about_center {
        return (b.min + b.max) * 0.5;
    }
    grip_pos(b, [2 - g.idx[0], 2 - g.idx[1], 2 - g.idx[2]])
}

/// Per-axis factors for moving `grip` (about `anchor`) to `p`. Corner grips scale uniformly
/// along the diagonal; edge and face grips scale each of their axes independently.
fn grip_factors(g: &Grip, anchor: DVec3, p: DVec3) -> Option<DVec3> {
    let axes = g.axes();
    let d0 = g.pos - anchor;
    let mut f = DVec3::ONE;
    if g.count() == 3 {
        let l2 = d0.length_squared();
        if l2 < 1e-12 {
            return None;
        }
        let u = (p - anchor).dot(d0) / l2;
        f = DVec3::splat(u);
    } else {
        for a in 0..3 {
            if axes[a] {
                if d0[a].abs() < 1e-12 {
                    return None;
                }
                f[a] = (p - anchor)[a] / d0[a];
            }
        }
    }
    let ok = f.is_finite() && (0..3).all(|a| !axes[a] || f[a].abs() > 1e-4);
    ok.then_some(f)
}

/// Parse a scale VCB entry for `n` scaled axes: one factor (applied to all), or `n`
/// factors separated by `;` (or `,` when that yields exactly `n` numbers).
fn parse_factors(input: &str, n: usize) -> Option<Vec<f64>> {
    let num = |t: &str| t.trim().replace(',', ".").parse::<f64>().ok().filter(|f| f.is_finite() && f.abs() > 1e-6);
    let s = input.trim();
    if s.contains(';') {
        let v: Option<Vec<f64>> = s.split(';').map(num).collect();
        return v.filter(|v| v.len() == 1 || v.len() == n);
    }
    if n > 1 && s.contains(',') {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() == n
            && let Some(v) = parts.iter().map(|t| num(t)).collect::<Option<Vec<f64>>>()
        {
            return Some(v);
        }
    }
    num(s).map(|f| vec![f])
}

fn factors_from(axes: [bool; 3], vals: &[f64]) -> DVec3 {
    let mut f = DVec3::ONE;
    let mut it = vals.iter();
    let single = vals.len() == 1;
    for a in 0..3 {
        if axes[a] {
            f[a] = if single { vals[0] } else { *it.next().unwrap_or(&1.0) };
        }
    }
    f
}

fn fmt_factors(axes: [bool; 3], f: DVec3) -> String {
    let n = axes.iter().filter(|a| **a).count();
    if n == 3 && (f.x - f.y).abs() < 1e-9 && (f.y - f.z).abs() < 1e-9 {
        return format!("{:.3}", f.x);
    }
    (0..3).filter(|a| axes[*a]).map(|a| format!("{:.3}", f[a])).collect::<Vec<_>>().join("; ")
}

#[derive(Clone, Copy, Debug)]
struct ScaleDrag {
    grip: Grip,
    anchor: DVec3,
    about_center: bool,
    factor: DVec3,
}

#[derive(Default)]
pub struct ScaleTool {
    engine: InferenceEngine,
    hover: Option<Hover>,
    ents: Vec<EntityRef>,
    bbox: Option<Aabb>,
    hot_grip: Option<Grip>,
    drag: Option<ScaleDrag>,
    clicker: Clicker,
    last: Option<(LastXf, [bool; 3])>,
}

impl ScaleTool {
    fn refresh(&mut self, ctx: &ToolContext) {
        if self.drag.is_some() {
            return;
        }
        self.hover = hover_if_no_selection(ctx);
        let ents = targets(ctx, &self.hover, None).ents;
        if ents != self.ents || self.bbox.is_none() {
            self.bbox = entities_bounds(ctx.doc, &ents);
            self.ents = ents;
        }
        let vp = ctx.viewport_d();
        let c = DVec2::new(ctx.pointer.pos.x as f64, ctx.pointer.pos.y as f64);
        self.hot_grip = self.bbox.as_ref().and_then(|b| {
            grips(b)
                .into_iter()
                .filter_map(|g| Some((ctx.camera.world_to_screen(g.pos, vp)?.distance(c), g)))
                .filter(|(d, _)| *d <= 10.0)
                // Prefer the closest; on ties prefer the grip scaling more axes (corners).
                .min_by(|a, b| a.0.total_cmp(&b.0).then(b.1.count().cmp(&a.1.count())))
                .map(|x| x.1)
        });
    }

    fn update_factor(&mut self, ctx: &ToolContext) {
        let Some(mut d) = self.drag else { return };
        // Ctrl (held) = scale about the centre.
        let about_center = ctx.pointer.modifiers.ctrl || ctx.pointer.modifiers.command;
        if about_center != d.about_center
            && let Some(b) = &self.bbox
        {
            d.about_center = about_center;
            d.anchor = grip_anchor(b, &d.grip, about_center);
        }
        let dir = d.grip.pos - d.anchor;
        let axes = d.grip.axes();
        let constraint = match d.grip.count() {
            // Corner: along the diagonal. Face: along its single axis.
            3 | 1 => dir.try_normalize().map(|n| inference::Constraint::Line { origin: d.anchor, dir: n, kind: InferenceKind::Free }),
            // Edge: in the plane of its two axes through the grip.
            _ => {
                let n = DVec3::new(
                    if axes[0] { 0.0 } else { 1.0 },
                    if axes[1] { 0.0 } else { 1.0 },
                    if axes[2] { 0.0 } else { 1.0 },
                );
                Some(inference::Constraint::Plane(Plane::new(d.grip.pos, n)))
            }
        };
        let r = infer(&mut self.engine, ctx, Some(d.anchor), constraint, None);
        if let Some(f) = grip_factors(&d.grip, d.anchor, r.point) {
            d.factor = f;
        }
        self.drag = Some(d);
    }

    fn finish(&mut self, ctx: &mut ToolContext, anchor: DVec3, factor: DVec3, axes: [bool; 3]) {
        self.drag = None;
        if (factor - DVec3::ONE).abs().max_element() < 1e-9 || factor.abs().min_element() < 1e-6 {
            return;
        }
        let kind = XfKind::Scale { anchor, factor };
        let tg = Targets::ents(self.ents.clone());
        if run_transform(ctx, "Scale", &tg, kind, false, None).is_some() {
            self.last = Some((LastXf { tg, kind, copy: false, arr: None }, axes));
        }
        self.bbox = None;
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        if let Some(d) = self.drag {
            self.update_factor(ctx);
            let d2 = self.drag.unwrap_or(d);
            self.finish(ctx, d2.anchor, d2.factor, d2.grip.axes());
            return;
        }
        self.refresh(ctx);
        if let (Some(g), Some(b)) = (self.hot_grip, self.bbox) {
            let about_center = ctx.pointer.modifiers.ctrl || ctx.pointer.modifiers.command;
            let anchor = grip_anchor(&b, &g, about_center);
            if anchor.distance(g.pos) > TOL {
                self.drag = Some(ScaleDrag { grip: g, anchor, about_center, factor: DVec3::ONE });
            }
        }
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
        if let Some(d) = self.drag {
            let axes = d.grip.axes();
            let Some(v) = parse_factors(input, d.grip.count()) else { return false };
            self.finish(ctx, d.anchor, factors_from(axes, &v), axes);
            return true;
        }
        if let Some((l, axes)) = self.last.clone()
            && let XfKind::Scale { anchor, .. } = l.kind
        {
            let n = axes.iter().filter(|a| **a).count();
            let Some(v) = parse_factors(input, n) else { return false };
            if !undo_last(ctx) {
                return false;
            }
            let kind = XfKind::Scale { anchor, factor: factors_from(axes, &v) };
            if run_transform(ctx, "Scale", &l.tg, kind, false, None).is_some() {
                self.last = Some((LastXf { kind, ..l }, axes));
                return true;
            }
        }
        false
    }
    fn vcb_label(&self) -> &str {
        "Scale"
    }
    fn vcb_value(&self) -> String {
        if let Some(d) = self.drag {
            return fmt_factors(d.grip.axes(), d.factor);
        }
        match &self.last {
            Some((l, axes)) => match l.kind {
                XfKind::Scale { factor, .. } => fmt_factors(*axes, factor),
                _ => String::new(),
            },
            None => String::new(),
        }
    }
    fn status_text(&self) -> String {
        if let Some(d) = self.drag {
            let what = match d.grip.count() {
                3 => "uniformly",
                2 => "along two axes",
                _ => "along one axis",
            };
            format!(
                "Move to scale {what} about the {}; click or type a factor (per axis: `1.5; 2`). Ctrl = about centre.",
                if d.about_center { "centre" } else { "opposite grip" }
            )
        } else if self.ents.is_empty() {
            "Select entities (or hover one), then drag a grip.".into()
        } else {
            "Drag a grip: corners scale uniformly, edge midpoints two axes, face centres one axis. Ctrl = about centre."
                .into()
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        if let Some(d) = self.drag {
            let t = XfKind::Scale { anchor: d.anchor, factor: d.factor }.at(1.0);
            draw_preview(view, &Targets::ents(self.ents.clone()), &t, out3d, out2d);
            let target = t.transform_point3(d.grip.pos);
            dashed(out3d, d.anchor, target, Color32::from_gray(60), 1.5);
            out2d.push(Marker2d::Point { at: d.anchor, shape: MarkerShape::Ring, color: Color32::from_rgb(0xFF, 0, 0) });
            let what = if d.grip.count() == 3 { "Uniform scale" } else { "Scale" };
            out2d.push(Marker2d::CursorTooltip { text: format!("{what} {}", fmt_factors(d.grip.axes(), d.factor)) });
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
                let hot = self.hot_grip.is_some_and(|h| h.idx == g.idx);
                out2d.push(Marker2d::Point {
                    at: g.pos,
                    shape: MarkerShape::Square,
                    color: if hot { Color32::from_rgb(0xFF, 0, 0) } else { Color32::from_rgb(0, 0xC0, 0) },
                });
            }
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        self.drag.is_some()
    }
}

// ===========================================================================
// Paste (Edit → Paste): the clipboard content follows the cursor like a Move
// ===========================================================================

/// Places the clipboard content: its reference point (bounding-box min corner) follows the
/// cursor with inference; a click pastes it there, Esc cancels. Shown as the Move tool.
pub struct PasteTool {
    clip: std::rc::Rc<crate::clipboard::Clipboard>,
    /// A library component's name (status text).
    name: Option<String>,
    engine: InferenceEngine,
    cur: Option<InferenceResult>,
    clicker: Clicker,
}

impl PasteTool {
    pub fn new(clip: std::rc::Rc<crate::clipboard::Clipboard>) -> Self {
        Self { clip, name: None, engine: InferenceEngine::default(), cur: None, clicker: Clicker::default() }
    }

    /// Placing a library component.
    pub fn named(clip: std::rc::Rc<crate::clipboard::Clipboard>, name: String) -> Self {
        Self { name: Some(name), ..Self::new(clip) }
    }

    fn offset(&self) -> Option<DMat4> {
        Some(DMat4::from_translation(self.cur.as_ref()?.point - self.clip.reference_point()))
    }

    fn update(&mut self, ctx: &ToolContext) {
        self.cur = Some(infer(&mut self.engine, ctx, None, None, None));
    }
}

/// Paste `clip` into the document's active context moved by `t` (world); selects the result.
pub fn paste_clipboard(doc: &mut crate::document::Document, clip: &crate::clipboard::Clipboard, t: &DMat4) -> bool {
    let path = doc.active_path.clone();
    let uid = doc.uid;
    match doc.try_commit("Paste", |m| {
        let (e, g) = clip.paste_into(m, uid, &path, t)?;
        let d = clip.paste_dims_into(m, &path, t);
        Ok::<_, varstotas_kernel::OpError>((e, g, d))
    }) {
        Some((ents, guides, dims)) => {
            doc.set_selection_full(ents, guides, dims);
            true
        }
        None => false,
    }
}

impl Tool for PasteTool {
    fn id(&self) -> ToolId {
        ToolId::Move
    }
    fn activate(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
    }
    fn on_pointer_move(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
    }
    fn on_pointer_down(&mut self, ctx: &mut ToolContext) {
        if !self.clicker.down(ctx) {
            return;
        }
        self.update(ctx);
        if let Some(t) = self.offset()
            && paste_clipboard(ctx.doc, &self.clip, &t)
        {
            ctx.requests.push(super::ToolRequest::SwitchTool(ToolId::Select));
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if !pressed {
            return false;
        }
        if key == Key::Escape {
            ctx.requests.push(super::ToolRequest::SwitchTool(ToolId::Select));
            return true;
        }
        let r = self.engine.handle_arrow(key);
        if r {
            self.update(ctx);
        }
        r
    }
    fn status_text(&self) -> String {
        match &self.name {
            Some(n) => format!("Click to place {n} (Esc cancels)."),
            None => format!("Click to place the pasted {} (Esc cancels).", plural(self.clip.count, "entity", "entities")),
        }
    }
    fn overlay(&self, view: &OverlayView, out3d: &mut Overlay3d, out2d: &mut Vec<Marker2d>) {
        let Some(t) = self.offset() else { return };
        for (a, b) in &self.clip.preview {
            line(out3d, t.transform_point3(*a), t.transform_point3(*b), SELECT_BLUE, 1.5);
        }
        for g in self.clip.guide_lines() {
            let g = g.transformed(&t);
            out3d.add_guide_line(view.camera, g.origin, g.dir, true);
        }
        let vp = DVec2::new(view.viewport.x as f64, view.viewport.y as f64);
        for p in self.clip.guide_points() {
            out3d.add_guide_point(view.camera, vp, t.transform_point3(*p), true);
        }
        if let Some(r) = &self.cur {
            inference::draw_result(r, out3d, out2d);
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        true
    }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
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
        let k = XfKind::Scale { anchor: DVec3::new(1.0, 1.0, 1.0), factor: DVec3::splat(2.0) };
        let p = k.at(1.0).transform_point3(DVec3::new(2.0, 1.0, 1.0));
        assert!((p - DVec3::new(3.0, 1.0, 1.0)).length() < 1e-12);
    }

    #[test]
    fn scale_grips_and_factors() {
        let b = Aabb { min: DVec3::ZERO, max: DVec3::new(2.0, 4.0, 6.0) };
        let g = grips(&b);
        assert_eq!(g.len(), 26);
        assert_eq!(g.iter().filter(|g| g.count() == 3).count(), 8);
        assert_eq!(g.iter().filter(|g| g.count() == 2).count(), 12);
        assert_eq!(g.iter().filter(|g| g.count() == 1).count(), 6);
        // Flat box: no grips scaling the flat axis.
        let flat = Aabb { min: DVec3::ZERO, max: DVec3::new(2.0, 4.0, 0.0) };
        assert_eq!(grips(&flat).len(), 8);
        // +X face grip: only X scales, anchored at the -X face.
        let fx = Grip { idx: [2, 1, 1], pos: grip_pos(&b, [2, 1, 1]) };
        let a = grip_anchor(&b, &fx, false);
        assert_eq!(a, DVec3::new(0.0, 2.0, 3.0));
        let f = grip_factors(&fx, a, DVec3::new(3.0, 9.0, 9.0)).unwrap();
        assert_eq!(f, DVec3::new(1.5, 1.0, 1.0));
        // Edge grip (+X,+Y): independent factors.
        let e = Grip { idx: [2, 2, 1], pos: grip_pos(&b, [2, 2, 1]) };
        let a = grip_anchor(&b, &e, false);
        let f = grip_factors(&e, a, DVec3::new(4.0, 2.0, 3.0)).unwrap();
        assert_eq!(f, DVec3::new(2.0, 0.5, 1.0));
        // Corner about the centre: uniform.
        let c = Grip { idx: [2, 2, 2], pos: b.max };
        let a = grip_anchor(&b, &c, true);
        let f = grip_factors(&c, a, a + (b.max - a) * 2.0).unwrap();
        assert!((f - DVec3::splat(2.0)).length() < 1e-12);
        // VCB parsing.
        assert_eq!(parse_factors("1.5", 2), Some(vec![1.5]));
        assert_eq!(parse_factors("1,5", 1), Some(vec![1.5]));
        assert_eq!(parse_factors("2,3", 2), Some(vec![2.0, 3.0]));
        assert_eq!(parse_factors("2;3", 2), Some(vec![2.0, 3.0]));
        assert_eq!(parse_factors("2;3", 3), None);
        assert_eq!(factors_from([true, false, true], &[2.0, 3.0]), DVec3::new(2.0, 1.0, 3.0));
    }
}
