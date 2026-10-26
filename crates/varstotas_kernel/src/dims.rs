//! Linear dimensions as model entities.
//!
//! A [`Dimension`] measures between two anchors. An anchor is attached to a vertex of its
//! context's mesh (`vertex: Some(..)`) and follows it through Move / Rotate / Scale / Resize /
//! push-pull / undo; `point` is the fallback position used when the anchor is free. When an
//! attached vertex is deleted the anchor becomes free at the vertex's last position; when it is
//! merged into another vertex the anchor moves to that vertex.
//!
//! Like guides, dimensions belong to an editing context: they live in the [`Mesh::dims`] of the
//! definition (root, group or component) in its local coordinates, so they move with a group
//! and are shared by all instances of a component. A [`DimId`] is relative to a context path;
//! the `*_in` methods of [`Model`] take that path and work in world space. The displayed value
//! is the world-space distance between the anchors in model units ([`crate::units`]).

use crate::mesh::Mesh;
use crate::model::{ContextPath, Model};
use crate::{DefId, InstanceId, LengthUnit, TOL, VertexId};
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};

slotmap::new_key_type! {
    pub struct DimId;
}

/// One end of a dimension (mesh-local coordinates).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DimAnchor {
    /// Vertex the anchor follows (when alive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vertex: Option<VertexId>,
    /// Fallback / free position.
    pub point: DVec3,
}

impl DimAnchor {
    pub fn free(point: DVec3) -> Self {
        Self { vertex: None, point }
    }
}

/// A linear dimension (mesh-local coordinates).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Dimension {
    pub a: DimAnchor,
    pub b: DimAnchor,
    /// The dimension line is drawn from `a + offset` to `b + offset` (extension lines from the
    /// anchors to it).
    pub offset: DVec3,
    /// Text override. `<>` inside it is replaced by the measured value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// A text note (Text tool) instead of a linear dimension: `a` is the point the leader
    /// points to (`b == a`), the text sits at `a + offset` (no leader when the offset is zero).
    #[serde(default, skip_serializing_if = "is_false")]
    pub leader: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// World-space geometry of a dimension.
#[derive(Clone, Debug, PartialEq)]
pub struct DimGeom {
    pub a: DVec3,
    pub b: DVec3,
    pub offset: DVec3,
    pub text: Option<String>,
    /// Text note (see [`Dimension::leader`]).
    pub leader: bool,
}

impl DimGeom {
    pub fn length(&self) -> f64 {
        self.a.distance(self.b)
    }
    /// End points of the dimension line.
    pub fn line(&self) -> (DVec3, DVec3) {
        (self.a + self.offset, self.b + self.offset)
    }
    /// Displayed text: the override (with `<>` replaced by the value) or the formatted length.
    pub fn label(&self, unit: LengthUnit) -> String {
        if self.leader {
            return self.text.clone().unwrap_or_default();
        }
        dim_label(self.length(), self.text.as_deref(), unit)
    }

    /// Where a text note's text sits.
    pub fn text_pos(&self) -> DVec3 {
        self.a + self.offset
    }
    pub fn transformed(&self, m: &DMat4) -> DimGeom {
        DimGeom {
            a: m.transform_point3(self.a),
            b: m.transform_point3(self.b),
            offset: m.transform_vector3(self.offset),
            text: self.text.clone(),
            leader: self.leader,
        }
    }
}

/// Text shown for a dimension of length `mm`.
pub fn dim_label(mm: f64, text: Option<&str>, unit: LengthUnit) -> String {
    let value = crate::units::format_length(mm, unit);
    match text {
        Some(t) if !t.is_empty() => t.replace("<>", &value),
        _ => value,
    }
}

impl Mesh {
    /// Current position of an anchor (its vertex if alive, else its point).
    pub fn anchor_pos(&self, a: &DimAnchor) -> DVec3 {
        a.vertex.and_then(|v| self.vertices.get(v)).map_or(a.point, |v| v.pos)
    }

    /// Anchor at `p`, attached to a vertex within TOL if there is one.
    pub fn anchor_at(&mut self, p: DVec3) -> DimAnchor {
        self.ensure();
        match self.v_find(p, TOL) {
            Some(v) => DimAnchor { vertex: Some(v), point: self.pos(v) },
            None => DimAnchor::free(p),
        }
    }

    /// Add a dimension between `a` and `b` (anchors attach to vertices at those points).
    pub fn add_dimension(&mut self, a: DVec3, b: DVec3, offset: DVec3) -> DimId {
        let (a, b) = (self.anchor_at(a), self.anchor_at(b));
        self.dims.insert(Dimension { a, b, offset, text: None, leader: false })
    }

    /// Add a text note pointing at `at` (attached to a vertex there) with its text at `at + offset`.
    pub fn add_text_note(&mut self, at: DVec3, offset: DVec3, text: String) -> DimId {
        let a = self.anchor_at(at);
        self.dims.insert(Dimension { a, b: a, offset, text: Some(text), leader: true })
    }

    /// Local geometry of dimension `id` with current anchor positions.
    pub fn dim_geom(&self, id: DimId) -> Option<DimGeom> {
        let d = self.dims.get(id)?;
        Some(DimGeom {
            a: self.anchor_pos(&d.a),
            b: self.anchor_pos(&d.b),
            offset: d.offset,
            text: d.text.clone(),
            leader: d.leader,
        })
    }

    /// Vertex `v` is about to be removed: anchors on it become free at its position.
    pub(crate) fn detach_dims_from(&mut self, v: VertexId) {
        if self.dims.is_empty() {
            return;
        }
        let Some(p) = self.vertices.get(v).map(|x| x.pos) else { return };
        for d in self.dims.values_mut() {
            for a in [&mut d.a, &mut d.b] {
                if a.vertex == Some(v) {
                    *a = DimAnchor::free(p);
                }
            }
        }
    }

    /// Vertex `gone` is merged into `keep`: anchors move over.
    pub(crate) fn repoint_dims(&mut self, gone: VertexId, keep: VertexId) {
        if self.dims.is_empty() {
            return;
        }
        let Some(p) = self.vertices.get(keep).map(|x| x.pos) else { return };
        for d in self.dims.values_mut() {
            for a in [&mut d.a, &mut d.b] {
                if a.vertex == Some(gone) {
                    *a = DimAnchor { vertex: Some(keep), point: p };
                }
            }
        }
    }

    /// Refresh the fallback points of attached anchors from their vertices.
    pub fn sync_dim_points(&mut self) {
        let Mesh { dims, vertices, .. } = self;
        for d in dims.values_mut() {
            for a in [&mut d.a, &mut d.b] {
                if let Some(x) = a.vertex.and_then(|v| vertices.get(v)) {
                    a.point = x.pos;
                }
            }
        }
    }

    /// Transform dimensions by the local transform `m`: free anchors move, the offset is
    /// mapped by the linear part. Attached anchors stay on their vertices.
    pub(crate) fn transform_dims_local(&mut self, ids: &[DimId], m: &DMat4) {
        for id in ids {
            let Some(d) = self.dims.get_mut(*id) else { continue };
            for a in [&mut d.a, &mut d.b] {
                a.point = m.transform_point3(a.point);
            }
            d.offset = m.transform_vector3(d.offset);
        }
        self.sync_dim_points();
    }
}

impl Model {
    /// Dimensions of the context `path` with their world geometry.
    pub fn dimensions_in(&self, path: &[InstanceId]) -> Vec<(DimId, DimGeom)> {
        let Some(mesh) = self.mesh_at(path) else { return Vec::new() };
        let xf = self.transform_at(path);
        mesh.dims.keys().filter_map(|id| Some((id, mesh.dim_geom(id)?.transformed(&xf)))).collect()
    }

    /// World geometry of dimension `id` of the context `path`.
    pub fn dimension_in(&self, path: &[InstanceId], id: DimId) -> Option<DimGeom> {
        Some(self.mesh_at(path)?.dim_geom(id)?.transformed(&self.transform_at(path)))
    }

    /// Displayed text of dimension `id` (world length in model units, or the override).
    pub fn dimension_label_in(&self, path: &[InstanceId], id: DimId) -> Option<String> {
        Some(self.dimension_in(path, id)?.label(self.unit))
    }

    pub fn has_dimension_in(&self, path: &[InstanceId], id: DimId) -> bool {
        self.mesh_at(path).is_some_and(|m| m.dims.contains_key(id))
    }

    /// All dimensions of the context `path`.
    pub fn all_dimensions_in(&self, path: &[InstanceId]) -> Vec<DimId> {
        self.mesh_at(path).map(|m| m.dims.keys().collect()).unwrap_or_default()
    }

    /// Add a dimension given in world space to the context `path`. Anchors attach to vertices
    /// of that context lying at `a` / `b` (within TOL).
    pub fn add_dimension_in(&mut self, path: &[InstanceId], a: DVec3, b: DVec3, offset: DVec3) -> Option<DimId> {
        let inv = self.transform_at(path).inverse();
        let mesh = self.mesh_at_mut(path)?;
        Some(mesh.add_dimension(inv.transform_point3(a), inv.transform_point3(b), inv.transform_vector3(offset)))
    }

    /// Add a text note given in world space to the context `path` (see [`Mesh::add_text_note`]).
    pub fn add_text_note_in(&mut self, path: &[InstanceId], at: DVec3, offset: DVec3, text: String) -> Option<DimId> {
        let inv = self.transform_at(path).inverse();
        let mesh = self.mesh_at_mut(path)?;
        Some(mesh.add_text_note(inv.transform_point3(at), inv.transform_vector3(offset), text))
    }

    /// Remove dimensions of the context `path`. Returns how many were removed.
    pub fn remove_dimensions_in(&mut self, path: &[InstanceId], ids: &[DimId]) -> usize {
        let Some(mesh) = self.mesh_at_mut(path) else { return 0 };
        ids.iter().filter(|id| mesh.dims.remove(**id).is_some()).count()
    }

    /// Move dimensions of the context `path` by the world transform `m`. Free anchors move;
    /// anchors attached to vertices stay on them (move the geometry to move those); the
    /// offset is rotated / scaled with `m`.
    pub fn transform_dimensions_in(&mut self, path: &[InstanceId], ids: &[DimId], m: &DMat4) {
        let xf = self.transform_at(path);
        let local = xf.inverse() * *m * xf;
        if let Some(mesh) = self.mesh_at_mut(path) {
            mesh.transform_dims_local(ids, &local);
        }
    }

    /// Insert copies of dimensions of the context `path` moved by the world transform `m`.
    /// Copied anchors attach to vertices found at their new positions (copy the geometry
    /// first so the copies attach to the copied vertices). Returns the new ids (same order).
    pub fn copy_dimensions_in(&mut self, path: &[InstanceId], ids: &[DimId], m: &DMat4) -> Vec<DimId> {
        let xf = self.transform_at(path);
        let local = xf.inverse() * *m * xf;
        let Some(mesh) = self.mesh_at_mut(path) else { return Vec::new() };
        let mut out = Vec::new();
        for id in ids {
            let Some(g) = mesh.dim_geom(*id) else { continue };
            let (a, b) = (mesh.anchor_at(local.transform_point3(g.a)), mesh.anchor_at(local.transform_point3(g.b)));
            let offset = local.transform_vector3(g.offset);
            let b = if g.leader { a } else { b };
            out.push(mesh.dims.insert(Dimension { a, b, offset, text: g.text, leader: g.leader }));
        }
        out
    }

    /// Set (or clear with `None` / empty) the text override. Returns false if not found.
    /// A text note keeps its old text when given an empty one.
    pub fn set_dimension_text_in(&mut self, path: &[InstanceId], id: DimId, text: Option<String>) -> bool {
        let Some(d) = self.mesh_at_mut(path).and_then(|m| m.dims.get_mut(id)) else { return false };
        let text = text.filter(|t| !t.is_empty());
        if d.leader && text.is_none() {
            return false;
        }
        d.text = text;
        true
    }

    /// Set the offset (world vector). Returns false if not found.
    pub fn set_dimension_offset_in(&mut self, path: &[InstanceId], id: DimId, offset: DVec3) -> bool {
        let local = self.transform_at(path).inverse().transform_vector3(offset);
        let Some(d) = self.mesh_at_mut(path).and_then(|m| m.dims.get_mut(id)) else { return false };
        d.offset = local;
        true
    }

    /// Any dimension anywhere in the model.
    pub fn has_any_dimensions(&self) -> bool {
        self.defs.values().any(|d| !d.mesh.dims.is_empty())
    }

    /// Every dimension to draw: (context path, id, world geometry) through all visible
    /// instances (a component's dimensions appear once per instance).
    pub fn all_dimensions_world(&self) -> Vec<(ContextPath, DimId, DimGeom)> {
        let mut out = Vec::new();
        self.walk_visible(self.root, DMat4::IDENTITY, &mut Vec::new(), 0, &mut |def, xf, path| {
            let mesh = &self.defs[def].mesh;
            for id in mesh.dims.keys() {
                if let Some(g) = mesh.dim_geom(id) {
                    out.push((path.to_vec(), id, g.transformed(xf)));
                }
            }
        });
        out
    }

    /// World positions of every guide point in the model, including those inside (visible)
    /// groups and components, once per instance (for Zoom Extents).
    pub fn guide_points_world(&self) -> Vec<DVec3> {
        let mut out: Vec<DVec3> = self.guide_points.values().copied().collect();
        self.walk_visible(self.root, DMat4::IDENTITY, &mut Vec::new(), 0, &mut |def, xf, path| {
            if !path.is_empty() {
                out.extend(self.defs[def].guides.points.values().map(|p| xf.transform_point3(*p)));
            }
        });
        out
    }

    /// World "wire" of the visible child instances of context `path` (for picking a group by
    /// its edges, dimensions or guide points): (child instance of `path`, segment a, b).
    /// Guide points are zero-length segments; infinite guide lines are skipped.
    pub fn nested_wire_world(&self, path: &[InstanceId]) -> Vec<(InstanceId, DVec3, DVec3)> {
        let mut out = Vec::new();
        let Some(d) = self.def_at(path) else { return out };
        self.walk_visible(d, self.transform_at(path), &mut Vec::new(), 0, &mut |def, xf, sub| {
            let Some(&child) = sub.first() else { return };
            let def = &self.defs[def];
            let mesh = &def.mesh;
            for (e, edge) in mesh.edges.iter() {
                if !edge.hidden {
                    let (a, b) = mesh.edge_segment(e);
                    out.push((child, xf.transform_point3(a), xf.transform_point3(b)));
                }
            }
            for id in mesh.dims.keys() {
                if let Some(g) = mesh.dim_geom(id) {
                    let g = g.transformed(xf);
                    let (p, q) = g.line();
                    out.extend([(child, p, q), (child, g.a, p), (child, g.b, q)]);
                }
            }
            if !self.guides_hidden {
                out.extend(def.guides.points.values().map(|p| {
                    let p = xf.transform_point3(*p);
                    (child, p, p)
                }));
            }
        });
        out
    }

    /// World anchor and dimension-line points of every dimension (for Zoom Extents).
    pub fn dimension_points_world(&self) -> Vec<DVec3> {
        self.all_dimensions_world()
            .into_iter()
            .flat_map(|(_, _, g)| {
                let (p, q) = g.line();
                [g.a, g.b, p, q]
            })
            .collect()
    }

    /// Visit `d` and every visible nested instance: (definition, world transform, path).
    fn walk_visible(
        &self,
        d: DefId,
        xf: DMat4,
        path: &mut Vec<InstanceId>,
        depth: usize,
        f: &mut dyn FnMut(DefId, &DMat4, &[InstanceId]),
    ) {
        if depth > 64 {
            return;
        }
        let Some(def) = self.defs.get(d) else { return };
        f(d, &xf, path);
        for (iid, inst) in &def.mesh.instances {
            if inst.hidden || !self.tag_visible(d, inst.tag) {
                continue;
            }
            path.push(iid);
            self.walk_visible(inst.def, xf * inst.transform, path, depth + 1, f);
            path.pop();
        }
    }

    /// Transform every dimension stored in definition `d` (local transform `m`).
    pub(crate) fn transform_def_dims(&mut self, d: DefId, m: &DMat4) {
        if let Some(def) = self.defs.get_mut(d) {
            let ids: Vec<DimId> = def.mesh.dims.keys().collect();
            def.mesh.transform_dims_local(&ids, m);
        }
    }
}
