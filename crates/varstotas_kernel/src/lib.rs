//! Varstotas geometry kernel: "sticky" geometry + document model.
//!
//! CONTRACT (see docs/02-ARCHITECTURE.md). Owner: `kernel` agent.
//! Signatures re-exported here are the public API used by `varstotas_app`. Adding items is free;
//! changing/removing them requires updating the architecture doc.
//!
//! Units: millimetres, f64. Z is up. Right-handed.
//!
//! Module map:
//! - [`mesh`] — `Mesh` storage + read-only queries (adjacency caches live here).
//! - `topology` — low-level sticky primitives (vertex merge, edge split, planar face finding, erase).
//! - `ops` — push/pull, offset, follow-me, transform, duplicate, orientation.
//! - [`model`] — `Model` document (definitions, instances, groups, components, materials, tags).
//! - [`curves`] — arc / circle / polygon metadata, construction helpers, regeneration.
//! - [`dims`] — linear dimensions (per context, anchored to vertices).
//! - [`history`] — snapshot undo/redo.
//! - [`tessellate()`] — world-space triangles/edges for rendering and picking.
//! - [`triangulate`] — ear-clipping polygon-with-holes triangulation.
//! - [`units`] — VCB length parsing/formatting.
//! - [`io`] — `.vrst` JSON, OBJ/STL export and import.

pub use glam;
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};

pub mod curves;
pub mod dims;
pub mod geom;
mod grid;
pub mod guides;
pub mod history;
pub mod io;
pub mod mesh;
pub mod model;
mod ops;
pub mod tessellate;
mod topology;
pub mod triangulate;
pub mod units;

pub use curves::{CurveInfo, CurveKind};
pub use dims::{DimAnchor, DimGeom, DimId, Dimension};
pub use guides::{DefGuides, GuideGeom, GuideId, GuidePointId, GuideRef};
pub use history::History;
pub use mesh::Mesh;
pub use ops::IntersectScope;
pub use model::{
    ContextPath, Definition, GuideLine, LengthUnit, Material, Model, ModelHit, Scene, SceneSettings, SceneStyle, Tag,
};
pub use tessellate::{Owner, TessEdge, TessFace, Tessellation, tessellate};

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

impl Plane {
    pub fn new(origin: DVec3, normal: DVec3) -> Self {
        Self { origin, normal: normal.normalize_or(DVec3::Z) }
    }
    /// Signed distance from `p` to the plane (positive on the normal side).
    pub fn distance(&self, p: DVec3) -> f64 {
        (p - self.origin).dot(self.normal)
    }
    /// Orthogonal projection of `p` onto the plane.
    pub fn project(&self, p: DVec3) -> DVec3 {
        p - self.normal * self.distance(p)
    }
    /// Ray/plane intersection parameter (ray `t`), if not parallel.
    pub fn intersect_ray(&self, ray: &Ray) -> Option<f64> {
        let den = ray.dir.dot(self.normal);
        if den.abs() < 1e-12 {
            return None;
        }
        Some((self.origin - ray.origin).dot(self.normal) / den)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: DVec3,
    /// Unit direction.
    pub dir: DVec3,
}

impl Ray {
    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + self.dir * t
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Aabb {
    pub min: DVec3,
    pub max: DVec3,
}

impl Aabb {
    pub fn from_point(p: DVec3) -> Self {
        Self { min: p, max: p }
    }
    pub fn from_points(pts: impl IntoIterator<Item = DVec3>) -> Option<Self> {
        let mut it = pts.into_iter();
        let first = it.next()?;
        let mut b = Self::from_point(first);
        for p in it {
            b.include(p);
        }
        Some(b)
    }
    pub fn include(&mut self, p: DVec3) {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
    }
    pub fn union(&self, o: &Aabb) -> Aabb {
        Aabb { min: self.min.min(o.min), max: self.max.max(o.max) }
    }
    pub fn center(&self) -> DVec3 {
        (self.min + self.max) * 0.5
    }
    pub fn size(&self) -> DVec3 {
        self.max - self.min
    }
    pub fn inflate(&self, d: f64) -> Aabb {
        Aabb { min: self.min - DVec3::splat(d), max: self.max + DVec3::splat(d) }
    }
    pub fn contains(&self, p: DVec3) -> bool {
        p.cmpge(self.min).all() && p.cmple(self.max).all()
    }
    pub fn intersects(&self, o: &Aabb) -> bool {
        self.min.cmple(o.max).all() && o.min.cmple(self.max).all()
    }
    /// The 8 corners.
    pub fn corners(&self) -> [DVec3; 8] {
        let (a, b) = (self.min, self.max);
        [
            DVec3::new(a.x, a.y, a.z),
            DVec3::new(b.x, a.y, a.z),
            DVec3::new(a.x, b.y, a.z),
            DVec3::new(b.x, b.y, a.z),
            DVec3::new(a.x, a.y, b.z),
            DVec3::new(b.x, a.y, b.z),
            DVec3::new(a.x, b.y, b.z),
            DVec3::new(b.x, b.y, b.z),
        ]
    }
    /// Bounds of this box after an affine transform.
    pub fn transformed(&self, m: &DMat4) -> Aabb {
        Aabb::from_points(self.corners().iter().map(|c| m.transform_point3(*c))).unwrap()
    }
    /// Slab test; returns entry `t` (clamped to >= 0) if the ray hits the box.
    pub fn ray_hit(&self, origin: DVec3, dir: DVec3) -> Option<f64> {
        let mut tmin = f64::NEG_INFINITY;
        let mut tmax = f64::INFINITY;
        for i in 0..3 {
            let o = origin[i];
            let d = dir[i];
            if d.abs() < 1e-15 {
                if o < self.min[i] || o > self.max[i] {
                    return None;
                }
            } else {
                let t1 = (self.min[i] - o) / d;
                let t2 = (self.max[i] - o) / d;
                tmin = tmin.max(t1.min(t2));
                tmax = tmax.min(t1.max(t2));
            }
        }
        if tmax < tmin.max(0.0) {
            None
        } else {
            Some(tmin.max(0.0))
        }
    }
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

impl Edge {
    pub fn new(a: VertexId, b: VertexId) -> Self {
        Self { v: [a, b], soft: false, smooth: false, hidden: false, tag: None, curve: None }
    }
    /// The endpoint that is not `v` (or `v[0]` if `v` is not an endpoint).
    pub fn other(&self, v: VertexId) -> VertexId {
        if self.v[0] == v { self.v[1] } else { self.v[0] }
    }
    pub fn has(&self, v: VertexId) -> bool {
        self.v[0] == v || self.v[1] == v
    }
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

impl Face {
    /// A face with no loops and default attributes (used as an attribute template).
    pub fn template() -> Self {
        Self {
            outer: Vec::new(),
            holes: Vec::new(),
            normal: DVec3::Z,
            front: None,
            back: None,
            hidden: false,
            tag: None,
        }
    }
    /// Iterate over all loops (outer first, then holes).
    pub fn loops(&self) -> impl Iterator<Item = &Vec<VertexId>> {
        std::iter::once(&self.outer).chain(self.holes.iter())
    }
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
