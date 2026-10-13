//! Varstotas geometry kernel: "sticky" geometry + document model.
//!
//! CONTRACT (see docs/02-ARCHITECTURE.md). Owner: `kernel` agent.
//! Signatures below are the public API used by `varstotas_app`. Adding items is free;
//! changing/removing them requires updating the architecture doc.
//!
//! Units: millimetres, f64. Z is up. Right-handed.

pub use glam;
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use std::collections::HashSet;

/// Point merge tolerance in millimetres.
pub const TOL: f64 = 1e-3;

slotmap::new_key_type! {
    pub struct VertexId;
    pub struct EdgeId;
    pub struct FaceId;
    pub struct InstanceId;
    pub struct DefId;
    pub struct MaterialId;
    pub struct TagId;
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum OpError {
    #[error("entity not found")]
    NotFound,
    #[error("degenerate geometry")]
    Degenerate,
    #[error("operation not possible: {0}")]
    Invalid(String),
}
pub type OpResult<T> = Result<T, OpError>;

// ---------------------------------------------------------------------------
// Geometry primitives
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    pub origin: DVec3,
    /// Unit normal.
    pub normal: DVec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: DVec3,
    /// Unit direction.
    pub dir: DVec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Aabb {
    pub min: DVec3,
    pub max: DVec3,
}

// ---------------------------------------------------------------------------
// Entities
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vertex {
    pub pos: DVec3,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub v: [VertexId; 2],
    pub soft: bool,
    pub smooth: bool,
    pub hidden: bool,
    pub tag: Option<TagId>,
    /// Edges created together as one arc/circle share a curve id (selected/erased as a unit).
    pub curve: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Face {
    /// Outer loop, counter-clockwise when viewed from the front (normal side).
    pub outer: Vec<VertexId>,
    /// Inner loops (holes), clockwise when viewed from the front.
    pub holes: Vec<Vec<VertexId>>,
    pub normal: DVec3,
    pub front: Option<MaterialId>,
    pub back: Option<MaterialId>,
    pub hidden: bool,
    pub tag: Option<TagId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Instance {
    pub def: DefId,
    pub transform: DMat4,
    pub name: String,
    pub material: Option<MaterialId>,
    pub tag: Option<TagId>,
    pub hidden: bool,
    pub locked: bool,
}

/// Reference to an entity inside one `Mesh` (an editing context).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityRef {
    Vertex(VertexId),
    Edge(EdgeId),
    Face(FaceId),
    Instance(InstanceId),
}

/// Sticky geometry of one context (model root, a group, or a component definition).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Mesh {
    pub vertices: SlotMap<VertexId, Vertex>,
    pub edges: SlotMap<EdgeId, Edge>,
    pub faces: SlotMap<FaceId, Face>,
    pub instances: SlotMap<InstanceId, Instance>,
    // Kernel may add private adjacency caches (mark #[serde(skip)] and rebuild on load).
}

impl Mesh {
    pub fn new() -> Self {
        Self::default()
    }

    // ---- queries -------------------------------------------------------
    pub fn edge_faces(&self, _e: EdgeId) -> Vec<FaceId> {
        todo!()
    }
    pub fn vertex_edges(&self, _v: VertexId) -> Vec<EdgeId> {
        todo!()
    }
    /// All edges bounding the face (outer + holes).
    pub fn face_edges(&self, _f: FaceId) -> Vec<EdgeId> {
        todo!()
    }
    pub fn face_plane(&self, _f: FaceId) -> Plane {
        todo!()
    }
    pub fn face_area(&self, _f: FaceId) -> f64 {
        todo!()
    }
    pub fn edge_length(&self, _e: EdgeId) -> f64 {
        todo!()
    }
    pub fn edge_segment(&self, _e: EdgeId) -> (DVec3, DVec3) {
        todo!()
    }
    /// All entities connected to `start` through edges/faces (triple-click).
    pub fn connected(&self, _start: EntityRef) -> HashSet<EntityRef> {
        todo!()
    }
    pub fn bounds(&self) -> Option<Aabb> {
        todo!()
    }
    /// Closest face hit in this mesh (instances are NOT descended; see `Model::raycast`).
    pub fn raycast_faces(&self, _ray: &Ray) -> Option<(FaceId, f64)> {
        todo!()
    }

    // ---- sticky editing ---------------------------------------------------
    /// Add a straight line. Merges with existing vertices within TOL, splits crossed edges,
    /// splits faces it cuts, and creates new faces for newly closed planar loops.
    /// Returns ids of all edge pieces that make up the line.
    pub fn add_line(&mut self, _a: DVec3, _b: DVec3) -> Vec<EdgeId> {
        todo!()
    }
    /// Add a closed or open polyline (circles/arcs/polygons). `curve` groups edges into one curve.
    pub fn add_polyline(&mut self, _pts: &[DVec3], _closed: bool, _as_curve: bool) -> Vec<EdgeId> {
        todo!()
    }
    /// Convenience: closed planar loop → edges + face. Returns the face if one was created.
    pub fn add_face(&mut self, _pts: &[DVec3]) -> Option<FaceId> {
        todo!()
    }
    /// classic eraser semantics: erasing an edge removes faces bounded by it; then
    /// tries to merge/heal faces; erasing a face keeps its edges.
    pub fn erase(&mut self, _ents: &[EntityRef]) {
        todo!()
    }
    /// Push/Pull a face along its normal by `distance` (negative = into the solid).
    /// Creates side faces, removes the original face unless `keep_original`.
    /// Pushing fully through a solid to a parallel face cuts a hole.
    /// Returns the moved (cap) face.
    pub fn push_pull(&mut self, _f: FaceId, _distance: f64, _keep_original: bool) -> OpResult<FaceId> {
        todo!()
    }
    /// Apply transform to vertices; connected edges/faces stretch. Non-planar faces that
    /// result are triangulated/split (folding) rather than left invalid.
    pub fn transform_vertices(&mut self, _vs: &HashSet<VertexId>, _m: &DMat4) {
        todo!()
    }
    /// Vertices referenced by a set of entities (for Move/Rotate/Scale).
    pub fn vertices_of(&self, _ents: &[EntityRef]) -> HashSet<VertexId> {
        todo!()
    }
    /// Duplicate entities (for Ctrl-copy), returns new refs.
    pub fn duplicate(&mut self, _ents: &[EntityRef], _m: &DMat4) -> Vec<EntityRef> {
        todo!()
    }
    /// Offset face boundary inward (negative) or outward (positive) within the face plane,
    /// splitting the face. Returns the new inner face.
    pub fn offset_face(&mut self, _f: FaceId, _distance: f64) -> OpResult<FaceId> {
        todo!()
    }
    /// Sweep a profile face along a path of connected edges.
    pub fn follow_me(&mut self, _profile: FaceId, _path: &[EdgeId]) -> OpResult<()> {
        todo!()
    }
    pub fn reverse_face(&mut self, _f: FaceId) {
        todo!()
    }
    /// Make all faces in the connected component face consistently (outward).
    pub fn orient_faces_like(&mut self, _f: FaceId) {
        todo!()
    }
}

// ---------------------------------------------------------------------------
// Document
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Definition {
    pub name: String,
    pub mesh: Mesh,
    /// Groups are definitions with exactly one instance and `is_group = true`.
    pub is_group: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    /// Linear-ish sRGB 0..1 + alpha (opacity).
    pub color: [f32; 4],
    /// Optional texture (path or embedded later).
    pub texture: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub visible: bool,
    pub color: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LengthUnit {
    Millimeter,
    Centimeter,
    Meter,
    Inch,
    Foot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub eye: DVec3,
    pub target: DVec3,
    pub up: DVec3,
    pub fov_y_deg: f64,
    pub ortho: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuideLine {
    pub origin: DVec3,
    pub dir: DVec3,
}

/// A path from the root into nested instances, identifying an editing context.
pub type ContextPath = Vec<InstanceId>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    pub defs: SlotMap<DefId, Definition>,
    pub root: DefId,
    pub materials: SlotMap<MaterialId, Material>,
    pub tags: SlotMap<TagId, Tag>,
    pub scenes: Vec<Scene>,
    pub guides: Vec<GuideLine>,
    pub guide_points: Vec<DVec3>,
    pub unit: LengthUnit,
}

/// Result of a model-wide ray pick.
#[derive(Clone, Debug)]
pub struct ModelHit {
    /// Instance chain from the root to the mesh containing `face`.
    pub path: ContextPath,
    pub face: FaceId,
    pub point: DVec3,
    pub t: f64,
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

impl Model {
    pub fn new() -> Self {
        todo!()
    }
    pub fn root_mesh(&self) -> &Mesh {
        &self.defs[self.root].mesh
    }
    pub fn root_mesh_mut(&mut self) -> &mut Mesh {
        let r = self.root;
        &mut self.defs[r].mesh
    }
    /// Mesh of the definition reached through `path` (empty path = root).
    pub fn mesh_at(&self, _path: &[InstanceId]) -> Option<&Mesh> {
        todo!()
    }
    pub fn mesh_at_mut(&mut self, _path: &[InstanceId]) -> Option<&mut Mesh> {
        todo!()
    }
    /// World transform of the context reached through `path`.
    pub fn transform_at(&self, _path: &[InstanceId]) -> DMat4 {
        todo!()
    }
    pub fn raycast(&self, _ray: &Ray) -> Option<ModelHit> {
        todo!()
    }
    pub fn bounds(&self) -> Option<Aabb> {
        todo!()
    }
    /// Move `ents` from the mesh at `path` into a new group; returns the new instance.
    pub fn make_group(&mut self, _path: &[InstanceId], _ents: &[EntityRef]) -> OpResult<InstanceId> {
        todo!()
    }
    /// Like make_group but the definition is a reusable component.
    pub fn make_component(&mut self, _path: &[InstanceId], _ents: &[EntityRef], _name: &str) -> OpResult<InstanceId> {
        todo!()
    }
    /// Explode an instance: copy its transformed geometry into the parent mesh (sticky merge).
    pub fn explode(&mut self, _path: &[InstanceId], _inst: InstanceId) -> OpResult<Vec<EntityRef>> {
        todo!()
    }
    /// Give an instance its own copy of the definition.
    pub fn make_unique(&mut self, _path: &[InstanceId], _inst: InstanceId) -> OpResult<()> {
        todo!()
    }
    pub fn add_material(&mut self, _name: &str, _color: [f32; 4]) -> MaterialId {
        todo!()
    }
    pub fn add_tag(&mut self, _name: &str) -> TagId {
        todo!()
    }
}

// ---------------------------------------------------------------------------
// Undo/Redo
// ---------------------------------------------------------------------------

/// Snapshot-based history. Usage: `history.commit("Push/Pull", &model_before)` AFTER
/// mutating, or `begin`/`commit` pairs — see implementation docs.
#[derive(Default)]
pub struct History {
    undo: Vec<(String, Model)>,
    redo: Vec<(String, Model)>,
}

impl History {
    /// Record `before` as the state to return to on undo. Clears redo stack.
    pub fn push(&mut self, label: &str, before: Model) {
        self.undo.push((label.to_string(), before));
        self.redo.clear();
    }
    pub fn undo(&mut self, current: &mut Model) -> Option<String> {
        let (label, prev) = self.undo.pop()?;
        let cur = std::mem::replace(current, prev);
        self.redo.push((label.clone(), cur));
        Some(label)
    }
    pub fn redo(&mut self, current: &mut Model) -> Option<String> {
        let (label, next) = self.redo.pop()?;
        let cur = std::mem::replace(current, next);
        self.undo.push((label.clone(), cur));
        Some(label)
    }
    pub fn undo_label(&self) -> Option<&str> {
        self.undo.last().map(|(l, _)| l.as_str())
    }
    pub fn redo_label(&self) -> Option<&str> {
        self.redo.last().map(|(l, _)| l.as_str())
    }
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

// ---------------------------------------------------------------------------
// Tessellation (for rendering & picking)
// ---------------------------------------------------------------------------

/// Which entity a tessellated primitive came from.
#[derive(Clone, Debug, PartialEq)]
pub struct Owner {
    pub path: ContextPath,
    pub entity: EntityRef,
}

#[derive(Clone, Debug, Default)]
pub struct TessFace {
    pub owner_path: ContextPath,
    pub face: Option<FaceId>,
    /// World-space triangle list positions (len % 3 == 0).
    pub positions: Vec<DVec3>,
    pub normal: DVec3,
    pub front: Option<MaterialId>,
    pub back: Option<MaterialId>,
    /// Inherited material from enclosing instances (applies to faces with no material).
    pub inherited: Option<MaterialId>,
}

#[derive(Clone, Debug)]
pub struct TessEdge {
    pub owner_path: ContextPath,
    pub edge: EdgeId,
    pub a: DVec3,
    pub b: DVec3,
    pub soft: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Tessellation {
    pub faces: Vec<TessFace>,
    pub edges: Vec<TessEdge>,
}

/// Flatten the whole model (respecting hidden flags and tag visibility) to world-space data.
pub fn tessellate(_model: &Model) -> Tessellation {
    todo!()
}

// ---------------------------------------------------------------------------
// Units / VCB parsing
// ---------------------------------------------------------------------------

pub mod units {
    use super::LengthUnit;

    /// Parse a length like `2500`, `2.5m`, `250cm`, `10'`, `6"`, `1'6"`, `-30` into mm.
    /// Bare numbers use `default_unit`.
    pub fn parse_length(_s: &str, _default_unit: LengthUnit) -> Option<f64> {
        todo!()
    }
    /// Parse a list separated by `;` or `,` (when not used as decimal sep): `300;200`.
    pub fn parse_lengths(_s: &str, _default_unit: LengthUnit) -> Option<Vec<f64>> {
        todo!()
    }
    /// Format mm in the given unit, e.g. `2500 mm`, `2.5 m`, `8' 2 7/16"`.
    pub fn format_length(_mm: f64, _unit: LengthUnit) -> String {
        todo!()
    }
}

// ---------------------------------------------------------------------------
// File IO
// ---------------------------------------------------------------------------

pub mod io {
    use super::Model;
    use std::path::Path;

    #[derive(thiserror::Error, Debug)]
    pub enum IoError {
        #[error("io: {0}")]
        Io(#[from] std::io::Error),
        #[error("format: {0}")]
        Format(String),
    }

    pub fn save(_model: &Model, _path: &Path) -> Result<(), IoError> {
        todo!()
    }
    pub fn load(_path: &Path) -> Result<Model, IoError> {
        todo!()
    }
    pub fn export_obj(_model: &Model, _path: &Path) -> Result<(), IoError> {
        todo!()
    }
    pub fn export_stl(_model: &Model, _path: &Path) -> Result<(), IoError> {
        todo!()
    }
    pub fn import_obj(_model: &mut Model, _path: &Path) -> Result<(), IoError> {
        todo!()
    }
    pub fn import_stl(_model: &mut Model, _path: &Path) -> Result<(), IoError> {
        todo!()
    }
}
