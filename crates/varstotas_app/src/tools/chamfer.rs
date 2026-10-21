//! Chamfer / Fillet tools: click an edge between two faces to bevel it flat or round it.
//!
//! Like Push/Pull: click an edge (one of the selected edges: all of them), move the cursor away
//! from it to set the size (live preview), click again (or press-drag-release). A double click
//! uses the last size. Typing a size while sizing applies it; typing right after an operation
//! redoes it with the new size.

use varstotas_i18n::tr;
use super::common::*;
use super::draw::Clicker;
use super::{OverlayView, Tool, ToolContext, ToolId};
use crate::inference;
use crate::overlay::Marker2d;
use egui::{Color32, CursorIcon, Key, Pos2};
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{ChamferKind, EdgeId, EntityRef, LengthUnit, Mesh};
use varstotas_render::Overlay3d;
use std::cell::RefCell;

const PREVIEW_ERR: Color32 = Color32::from_rgb(0xD0, 0x30, 0x30);

/// Preview of the operation on a copy of the mesh: new edges (world) or why it fails.
#[derive(Clone, Default)]
struct Preview {
    key: Option<(Vec<EdgeId>, u64, String)>,
    lines: Vec<(DVec3, DVec3)>,
    error: Option<String>,
}

pub struct ChamferTool {
    round: bool,
    distance: f64,
    radius: f64,
    segments: u32,
    unit: LengthUnit,
    /// Edge under the cursor (world segment) and the edges a click would do.
    hover: Option<(EdgeId, DVec3, DVec3)>,
    targets: Vec<EdgeId>,
    /// Edges of the last operation (for redoing it with a typed size).
    last: Option<Vec<EdgeId>>,
    /// Sizing after the first click: the edges, the clicked edge (world) and the size under
    /// the cursor (`None` until the cursor moves).
    sizing: Option<Sizing>,
    clicker: Clicker,
    preview: RefCell<Preview>,
}

#[derive(Clone, Debug)]
struct Sizing {
    edges: Vec<EdgeId>,
    a: DVec3,
    b: DVec3,
    start: Pos2,
    size: Option<f64>,
}

/// Cursor moves under this many points after the first click count as "not moved".
const STILL_PX: f32 = 3.0;

impl ChamferTool {
    pub fn new(round: bool) -> Self {
        Self {
            round,
            distance: 20.0,
            radius: 20.0,
            segments: 8,
            unit: LengthUnit::Millimeter,
            hover: None,
            targets: Vec::new(),
            last: None,
            sizing: None,
            clicker: Clicker::default(),
            preview: RefCell::default(),
        }
    }

    /// The size in use: the dragged one while sizing, else the last one.
    fn size(&self) -> f64 {
        match self.sizing.as_ref().and_then(|s| s.size) {
            Some(v) => v,
            None if self.round => self.radius,
            None => self.distance,
        }
    }

    fn kind(&self) -> ChamferKind {
        if self.round {
            ChamferKind::Round { radius: self.size(), segments: self.segments }
        } else {
            ChamferKind::Flat { distance: self.size() }
        }
    }

    fn set_size(&mut self, v: f64) {
        if self.round {
            self.radius = v;
        } else {
            self.distance = v;
        }
    }

    /// Size under the cursor: distance (in the context's units) from the edge line to the
    /// point of the cursor ray closest to it.
    fn drag_size(ctx: &ToolContext, a: DVec3, b: DVec3) -> Option<f64> {
        let ray = ctx.pointer.ray;
        let d = (b - a).normalize_or_zero();
        let w = ray.origin - a;
        let (dd, rd) = (d.dot(ray.dir), d.dot(w));
        let den = 1.0 - dd * dd;
        if den < 1e-9 {
            return None;
        }
        let t = (dd * rd - ray.dir.dot(w)) / den;
        let q = ray.origin + ray.dir * t.max(0.0);
        let on_edge = a + d * (q - a).dot(d);
        let v = to_local(ctx.doc, q).distance(to_local(ctx.doc, on_edge));
        (v.is_finite() && v > 1e-6).then_some(v)
    }

    fn label(&self) -> String {
        tr!(if self.round { "tool-undo-fillet" } else { "tool-undo-chamfer" })
    }

    fn update(&mut self, ctx: &ToolContext) {
        if let Some(sz) = &mut self.sizing {
            if ctx.pointer.pos.distance(sz.start) > STILL_PX {
                sz.size = Self::drag_size(ctx, sz.a, sz.b);
            }
            self.targets = sz.edges.clone();
            self.refresh_preview(ctx);
            return;
        }
        self.hover = match pick_edge_at(ctx, ctx.pointer.pos, inference::EDGE_PICK_PX as f32) {
            Some(Hover::Edge { edge, a, b }) => Some((edge, a, b)),
            _ => None,
        };
        self.targets = match self.hover {
            Some((e, _, _)) if ctx.doc.selection.contains(&EntityRef::Edge(e)) => ctx
                .doc
                .selection
                .iter()
                .filter_map(|x| if let EntityRef::Edge(e) = x { Some(*e) } else { None })
                .collect(),
            Some((e, _, _)) => vec![e],
            None => Vec::new(),
        };
        self.refresh_preview(ctx);
    }

    fn refresh_preview(&self, ctx: &ToolContext) {
        let key = (self.targets.clone(), ctx.doc.model_version, format!("{:?}", self.kind()));
        let mut pv = self.preview.borrow_mut();
        if pv.key.as_ref() == Some(&key) {
            return;
        }
        *pv = Preview { key: Some(key), ..Default::default() };
        if self.targets.is_empty() {
            return;
        }
        let path = ctx.doc.active_path.clone();
        let Some(mesh) = ctx.doc.model.mesh_at(&path) else { return };
        let mut m: Mesh = mesh.clone();
        let mut map = std::collections::HashMap::new();
        let mut faces = Vec::new();
        for &e0 in &self.targets {
            let mut e = e0;
            while let Some(n) = map.get(&e) {
                e = *n;
            }
            match m.chamfer_edge(e, self.kind()) {
                Ok(r) => {
                    faces.extend(r.faces);
                    map.extend(r.replaced);
                }
                Err(err) => {
                    pv.error.get_or_insert_with(|| err.to_string());
                }
            }
        }
        let xf = context_xf(ctx.doc);
        let mut seen = std::collections::HashSet::new();
        for f in faces {
            for e in m.face_edges(f) {
                if seen.insert(e) {
                    let (a, b) = m.edge_segment(e);
                    pv.lines.push((xf.transform_point3(a), xf.transform_point3(b)));
                }
            }
        }
    }

    fn click(&mut self, ctx: &mut ToolContext) {
        self.update(ctx);
        match self.sizing.take() {
            None => {
                let Some((_, a, b)) = self.hover else { return };
                if self.targets.is_empty() {
                    return;
                }
                self.last = None;
                self.sizing = Some(Sizing { edges: self.targets.clone(), a, b, start: ctx.pointer.pos, size: None });
                self.update(ctx);
            }
            Some(sz) => {
                // Not moved (a double click): the last size.
                if let Some(v) = sz.size.filter(|_| ctx.pointer.pos.distance(sz.start) > STILL_PX) {
                    self.set_size(v);
                }
                if !self.apply(ctx, sz.edges.clone()) {
                    // Does not fit: keep sizing.
                    self.sizing = Some(sz);
                }
                self.update(ctx);
            }
        }
    }

    fn apply(&mut self, ctx: &mut ToolContext, edges: Vec<EdgeId>) -> bool {
        if edges.is_empty() {
            return false;
        }
        let path = ctx.doc.active_path.clone();
        let kind = self.kind();
        let done = ctx.doc.try_commit(&self.label(), |m| m.chamfer_edges_in(&path, &edges, kind)).is_some();
        if done {
            ctx.doc.set_selection_full(Vec::new(), Vec::new(), Vec::new());
            self.last = Some(edges);
        }
        done
    }
}

impl Tool for ChamferTool {
    fn id(&self) -> ToolId {
        if self.round { ToolId::Fillet } else { ToolId::Chamfer }
    }
    fn activate(&mut self, ctx: &mut ToolContext) {
        self.unit = ctx.doc.model.unit;
        self.update(ctx);
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
        // Press-drag-release sizes in one go.
        if self.clicker.up_is_drag(ctx) && self.sizing.is_some() {
            self.click(ctx);
        }
    }
    fn on_key(&mut self, ctx: &mut ToolContext, key: Key, pressed: bool) -> bool {
        if pressed && key == Key::Escape && self.sizing.is_some() {
            self.sizing = None;
            self.update(ctx);
            return true;
        }
        false
    }
    fn on_vcb(&mut self, ctx: &mut ToolContext, input: &str) -> bool {
        let unit = ctx.doc.model.unit;
        let mut size = None;
        let mut segs = None;
        for part in input.split([',', ';']).map(str::trim).filter(|p| !p.is_empty()) {
            if self.round && let Some(n) = parse_segments(part) {
                segs = Some(n);
            } else {
                match parse_len(unit, part).filter(|v| *v > 0.0 && v.is_finite()) {
                    Some(v) => size = Some(v),
                    None => return false,
                }
            }
        }
        if size.is_none() && segs.is_none() {
            return false;
        }
        if let Some(n) = segs {
            self.segments = n.clamp(1, 64);
        }
        if let Some(sz) = self.sizing.take() {
            // Typed while sizing: that size, applied now.
            if let Some(v) = size {
                self.set_size(v);
            }
            if !self.apply(ctx, sz.edges.clone()) {
                self.sizing = Some(sz);
            }
            self.update(ctx);
            return true;
        }
        if let Some(v) = size {
            self.set_size(v);
        }
        // Redo the last operation with the new size.
        if let Some(edges) = self.last.take()
            && undo_last(ctx)
            && !self.apply(ctx, edges.clone())
        {
            // The new size does not fit: keep the old result.
            ctx.doc.redo();
            self.last = Some(edges);
        }
        self.update(ctx);
        true
    }
    fn vcb_label(&self) -> String {
        tr!(if self.round { "tool-vcb-radius" } else { "tool-vcb-distance" })
    }
    fn vcb_value(&self) -> String {
        let v = varstotas_kernel::units::format_length(self.size(), self.unit);
        if self.round { format!("{v}, {}s", self.segments) } else { v }
    }
    fn status_text(&self) -> String {
        if let Some(e) = &self.preview.borrow().error
            && self.hover.is_some()
        {
            return tr!("tool-chamfer-cant", what = tr!(if self.round { "tool-chamfer-what-fillet" } else { "tool-chamfer-what-chamfer" }), error = e);
        }
        match (self.sizing.is_some(), self.round) {
            (true, true) => tr!("tool-fillet-sizing"),
            (true, false) => tr!("tool-chamfer-sizing"),
            (false, true) => {
                tr!("tool-fillet-hover")
            }
            (false, false) => {
                tr!("tool-chamfer-hover")
            }
        }
    }
    fn overlay(&self, _view: &OverlayView, out3d: &mut Overlay3d, _out2d: &mut Vec<Marker2d>) {
        let pv = self.preview.borrow();
        let hover = match &self.sizing {
            Some(sz) => Some((sz.a, sz.b)),
            None => self.hover.map(|(_, a, b)| (a, b)),
        };
        if let Some((a, b)) = hover {
            let color = if pv.error.is_some() && pv.lines.is_empty() { PREVIEW_ERR } else { SELECT_BLUE };
            line(out3d, a, b, color, 3.0);
        }
        for (a, b) in &pv.lines {
            line(out3d, *a, *b, SELECT_BLUE, 1.5);
        }
    }
    fn cursor(&self) -> CursorIcon {
        CursorIcon::None
    }
    fn is_busy(&self) -> bool {
        self.sizing.is_some()
    }
}
