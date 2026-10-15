//! Chamfer / Fillet tools: click an edge between two faces to bevel it flat or round it.
//!
//! Clicking one of the selected edges does all selected edges. The preview shows the result
//! for the edge under the cursor (or the selection). Typing a size right after an operation
//! redoes it with the new size, like other classic tools.

use super::common::*;
use super::{OverlayView, Tool, ToolContext, ToolId};
use crate::inference;
use crate::overlay::Marker2d;
use egui::{Color32, CursorIcon};
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
    preview: RefCell<Preview>,
}

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
            preview: RefCell::default(),
        }
    }

    fn kind(&self) -> ChamferKind {
        if self.round {
            ChamferKind::Round { radius: self.radius, segments: self.segments }
        } else {
            ChamferKind::Flat { distance: self.distance }
        }
    }

    fn label(&self) -> &'static str {
        if self.round { "Fillet" } else { "Chamfer" }
    }

    fn update(&mut self, ctx: &ToolContext) {
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

    fn apply(&mut self, ctx: &mut ToolContext, edges: Vec<EdgeId>) -> bool {
        if edges.is_empty() {
            return false;
        }
        let path = ctx.doc.active_path.clone();
        let kind = self.kind();
        let done = ctx.doc.try_commit(self.label(), |m| m.chamfer_edges_in(&path, &edges, kind)).is_some();
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
        if ctx.pointer.button != Some(egui::PointerButton::Primary) {
            return;
        }
        self.update(ctx);
        let edges = self.targets.clone();
        self.last = None;
        self.apply(ctx, edges);
        self.update(ctx);
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
        if let Some(v) = size {
            if self.round {
                self.radius = v;
            } else {
                self.distance = v;
            }
        }
        if let Some(n) = segs {
            self.segments = n.clamp(1, 64);
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
    fn vcb_label(&self) -> &str {
        if self.round { "Radius" } else { "Distance" }
    }
    fn vcb_value(&self) -> String {
        if self.round {
            format!("{}, {}s", varstotas_kernel::units::format_length(self.radius, self.unit), self.segments)
        } else {
            varstotas_kernel::units::format_length(self.distance, self.unit)
        }
    }
    fn status_text(&self) -> String {
        if let Some(e) = &self.preview.borrow().error
            && self.hover.is_some()
        {
            return format!("Can't {} this edge: {e}", self.label().to_lowercase());
        }
        if self.round {
            "Click an edge between two faces to round it (selected edges: all of them). Type a radius, `12s` for segments.".into()
        } else {
            "Click an edge between two faces to bevel it (selected edges: all of them). Type a distance.".into()
        }
    }
    fn overlay(&self, _view: &OverlayView, out3d: &mut Overlay3d, _out2d: &mut Vec<Marker2d>) {
        let pv = self.preview.borrow();
        if let Some((_, a, b)) = self.hover {
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
}
