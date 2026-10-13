//! The document: definitions (root, groups, components), instances, materials, tags.

use crate::mesh::Mesh;
use crate::{
    Aabb, DefId, EdgeId, EntityRef, FaceId, Instance, InstanceId, MaterialId, OpError, OpResult, Ray, TagId, VertexId,
};
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use std::collections::HashMap;

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
    /// Empty model: a root definition, no materials/tags, millimetres.
    pub fn new() -> Self {
        let mut defs = SlotMap::with_key();
        let root = defs.insert(Definition { name: "Model".into(), mesh: Mesh::new(), is_group: false });
        Self {
            defs,
            root,
            materials: SlotMap::with_key(),
            tags: SlotMap::with_key(),
            scenes: Vec::new(),
            guides: Vec::new(),
            guide_points: Vec::new(),
            unit: LengthUnit::Millimeter,
        }
    }
    pub fn root_mesh(&self) -> &Mesh {
        &self.defs[self.root].mesh
    }
    pub fn root_mesh_mut(&mut self) -> &mut Mesh {
        let r = self.root;
        &mut self.defs[r].mesh
    }

    /// Definition reached through `path` (empty path = root).
    pub fn def_at(&self, path: &[InstanceId]) -> Option<DefId> {
        let mut d = self.root;
        for i in path {
            d = self.defs.get(d)?.mesh.instances.get(*i)?.def;
        }
        self.defs.contains_key(d).then_some(d)
    }

    /// Mesh of the definition reached through `path` (empty path = root).
    pub fn mesh_at(&self, path: &[InstanceId]) -> Option<&Mesh> {
        let d = self.def_at(path)?;
        Some(&self.defs[d].mesh)
    }
    pub fn mesh_at_mut(&mut self, path: &[InstanceId]) -> Option<&mut Mesh> {
        let d = self.def_at(path)?;
        Some(&mut self.defs[d].mesh)
    }

    /// World transform of the context reached through `path`.
    pub fn transform_at(&self, path: &[InstanceId]) -> DMat4 {
        let mut m = DMat4::IDENTITY;
        let mut d = self.root;
        for i in path {
            let Some(inst) = self.defs.get(d).and_then(|def| def.mesh.instances.get(*i)) else { break };
            m *= inst.transform;
            d = inst.def;
        }
        m
    }

    pub(crate) fn tag_visible(&self, t: Option<TagId>) -> bool {
        t.and_then(|t| self.tags.get(t)).is_none_or(|t| t.visible)
    }

    /// Local bounds of a definition including nested instances.
    pub fn def_bounds(&self, d: DefId) -> Option<Aabb> {
        self.def_bounds_depth(d, 0)
    }

    fn def_bounds_depth(&self, d: DefId, depth: usize) -> Option<Aabb> {
        if depth > 64 {
            return None;
        }
        let mesh = &self.defs.get(d)?.mesh;
        let mut b = mesh.bounds();
        for inst in mesh.instances.values() {
            if let Some(cb) = self.def_bounds_depth(inst.def, depth + 1) {
                let wb = cb.transformed(&inst.transform);
                b = Some(b.map_or(wb, |x| x.union(&wb)));
            }
        }
        b
    }

    /// Closest visible face hit, descending into visible instances.
    pub fn raycast(&self, ray: &Ray) -> Option<ModelHit> {
        let mut best: Option<ModelHit> = None;
        self.raycast_rec(self.root, DMat4::IDENTITY, &mut Vec::new(), ray, &mut best, 0);
        best
    }

    fn raycast_rec(
        &self,
        d: DefId,
        m: DMat4,
        path: &mut Vec<InstanceId>,
        ray: &Ray,
        best: &mut Option<ModelHit>,
        depth: usize,
    ) {
        if depth > 64 {
            return;
        }
        let Some(def) = self.defs.get(d) else { return };
        let inv = m.inverse();
        let lo = inv.transform_point3(ray.origin);
        let ld = inv.transform_vector3(ray.dir);
        if let Some((f, t)) = def.mesh.raycast_faces_filtered(lo, ld, |face| !face.hidden && self.tag_visible(face.tag))
            && best.as_ref().is_none_or(|b| t < b.t) {
                *best = Some(ModelHit { path: path.clone(), face: f, point: ray.at(t), t });
            }
        for (iid, inst) in &def.mesh.instances {
            if inst.hidden || !self.tag_visible(inst.tag) {
                continue;
            }
            let cm = m * inst.transform;
            if let Some(b) = self.def_bounds(inst.def) {
                let wb = b.transformed(&cm).inflate(crate::TOL);
                match wb.ray_hit(ray.origin, ray.dir) {
                    None => continue,
                    Some(t) if best.as_ref().is_some_and(|h| t > h.t) => continue,
                    _ => {}
                }
            } else {
                continue;
            }
            path.push(iid);
            self.raycast_rec(inst.def, cm, path, ray, best, depth + 1);
            path.pop();
        }
    }

    /// World bounds of the whole model.
    pub fn bounds(&self) -> Option<Aabb> {
        self.def_bounds(self.root)
    }

    fn new_def_from(&mut self, path: &[InstanceId], ents: &[EntityRef], name: &str, is_group: bool) -> OpResult<InstanceId> {
        let pd = self.def_at(path).ok_or(OpError::NotFound)?;
        if ents.is_empty() {
            return Err(OpError::Invalid("nothing selected".into()));
        }
        let parent = &mut self.defs[pd].mesh;
        parent.ensure();
        let mut faces: Vec<FaceId> = Vec::new();
        let mut edges: Vec<EdgeId> = Vec::new();
        let mut verts: Vec<VertexId> = Vec::new();
        let mut insts: Vec<InstanceId> = Vec::new();
        for e in ents {
            match *e {
                EntityRef::Face(f) if parent.faces.contains_key(f) => {
                    if !faces.contains(&f) {
                        faces.push(f);
                    }
                    for e in parent.face_edges(f) {
                        if !edges.contains(&e) {
                            edges.push(e);
                        }
                    }
                }
                EntityRef::Edge(e) if parent.edges.contains_key(e) => {
                    if !edges.contains(&e) {
                        edges.push(e);
                    }
                }
                EntityRef::Vertex(v) if parent.vertices.contains_key(v) => verts.push(v),
                EntityRef::Instance(i) if parent.instances.contains_key(i)
                    && !insts.contains(&i) => {
                        insts.push(i);
                    }
                _ => {}
            }
        }
        // Build the new mesh.
        let mut mesh = Mesh::new();
        let mut vmap: HashMap<VertexId, VertexId> = HashMap::new();
        let mut touched: Vec<VertexId> = verts.clone();
        for e in &edges {
            let ed = parent.edges[*e].clone();
            touched.extend(ed.v);
            let mut ids = [ed.v[0], ed.v[1]];
            for x in ids.iter_mut() {
                *x = *vmap.entry(*x).or_insert_with(|| mesh.v_new(parent.vertices[*x].pos));
            }
            mesh.e_new(ids[0], ids[1], Some(&ed));
        }
        for v in &verts {
            vmap.entry(*v).or_insert_with(|| mesh.v_new(parent.vertices[*v].pos));
        }
        for f in &faces {
            let face = parent.faces[*f].clone();
            let outer = face.outer.iter().map(|v| vmap[v]).collect();
            let holes = face.holes.iter().map(|h| h.iter().map(|v| vmap[v]).collect()).collect();
            mesh.f_new(outer, holes, &face);
        }
        for i in &insts {
            let inst = parent.instances.remove(*i).unwrap();
            mesh.instances.insert(inst);
        }
        // Remove from parent: faces, then edges no longer used by remaining faces.
        for f in &faces {
            parent.f_remove(*f);
        }
        for e in &edges {
            if parent.edges.contains_key(*e) && parent.edge_faces(*e).is_empty() {
                parent.e_remove(*e);
            }
        }
        for v in touched {
            parent.remove_if_orphan(v);
        }
        let def = self.defs.insert(Definition { name: name.to_string(), mesh, is_group });
        let inst = Instance {
            def,
            transform: DMat4::IDENTITY,
            name: String::new(),
            material: None,
            tag: None,
            hidden: false,
            locked: false,
        };
        Ok(self.defs[pd].mesh.instances.insert(inst))
    }

    /// Move `ents` from the mesh at `path` into a new group; returns the new instance.
    /// Edges still used by faces that stay in the parent are copied (they remain in both).
    pub fn make_group(&mut self, path: &[InstanceId], ents: &[EntityRef]) -> OpResult<InstanceId> {
        self.new_def_from(path, ents, "Group", true)
    }

    /// Like make_group but the definition is a reusable component.
    pub fn make_component(&mut self, path: &[InstanceId], ents: &[EntityRef], name: &str) -> OpResult<InstanceId> {
        self.new_def_from(path, ents, name, false)
    }

    /// Number of instances (anywhere in the model) that use definition `d`.
    pub fn instance_count(&self, d: DefId) -> usize {
        self.defs.values().map(|def| def.mesh.instances.values().filter(|i| i.def == d).count()).sum()
    }

    /// Remove definitions (other than the root) that are no longer instanced.
    pub fn purge_unused_defs(&mut self) {
        loop {
            let unused: Vec<DefId> =
                self.defs.keys().filter(|d| *d != self.root && self.instance_count(*d) == 0).collect();
            if unused.is_empty() {
                break;
            }
            for d in unused {
                self.defs.remove(d);
            }
        }
    }

    /// Explode an instance: copy its transformed geometry into the parent mesh (vertices merge
    /// with coincident parent vertices; existing edges/faces are reused). The instance material
    /// is applied to faces without a material. Returns the new entities in the parent.
    pub fn explode(&mut self, path: &[InstanceId], inst: InstanceId) -> OpResult<Vec<EntityRef>> {
        let pd = self.def_at(path).ok_or(OpError::NotFound)?;
        let instance = self.defs[pd].mesh.instances.get(inst).ok_or(OpError::NotFound)?.clone();
        let child = self.defs.get(instance.def).ok_or(OpError::NotFound)?.mesh.clone();
        let t = instance.transform;
        let mirror = t.determinant() < 0.0;
        let parent = &mut self.defs[pd].mesh;
        parent.ensure();
        parent.instances.remove(inst);
        let mut out = Vec::new();
        let mut vmap: HashMap<VertexId, VertexId> = HashMap::new();
        for (v, vd) in &child.vertices {
            let nv = parent.v_get_or_new(t.transform_point3(vd.pos));
            vmap.insert(v, nv);
        }
        for (_, ed) in &child.edges {
            let (a, b) = (vmap[&ed.v[0]], vmap[&ed.v[1]]);
            if a == b {
                continue;
            }
            let e = match parent.edge_between(a, b) {
                Some(e) => e,
                None => parent.e_new(a, b, Some(ed)),
            };
            out.push(EntityRef::Edge(e));
        }
        for (_, face) in &child.faces {
            let mut outer: Vec<VertexId> = face.outer.iter().map(|v| vmap[v]).collect();
            let mut holes: Vec<Vec<VertexId>> = face.holes.iter().map(|h| h.iter().map(|v| vmap[v]).collect()).collect();
            if mirror {
                outer.reverse();
                for h in &mut holes {
                    h.reverse();
                }
            }
            if let Some(ex) = parent.find_face_with_outer(&outer) {
                out.push(EntityRef::Face(ex));
                continue;
            }
            let mut tmpl = face.clone();
            if tmpl.front.is_none() {
                tmpl.front = instance.material;
            }
            let f = parent.f_new(outer, holes, &tmpl);
            out.push(EntityRef::Face(f));
        }
        for (_, ci) in &child.instances {
            let mut c = ci.clone();
            c.transform = t * c.transform;
            if c.material.is_none() {
                c.material = instance.material;
            }
            out.push(EntityRef::Instance(parent.instances.insert(c)));
        }
        if self.instance_count(instance.def) == 0 && instance.def != self.root {
            self.defs.remove(instance.def);
        }
        Ok(out)
    }

    /// Give an instance its own copy of the definition.
    pub fn make_unique(&mut self, path: &[InstanceId], inst: InstanceId) -> OpResult<()> {
        let pd = self.def_at(path).ok_or(OpError::NotFound)?;
        let d = self.defs[pd].mesh.instances.get(inst).ok_or(OpError::NotFound)?.def;
        if self.instance_count(d) <= 1 {
            return Ok(());
        }
        let copy = self.defs[d].clone();
        let nd = self.defs.insert(Definition { name: format!("{}#{}", copy.name, self.defs.len()), ..copy });
        self.defs[pd].mesh.instances[inst].def = nd;
        Ok(())
    }

    pub fn add_material(&mut self, name: &str, color: [f32; 4]) -> MaterialId {
        self.materials.insert(Material { name: name.to_string(), color, texture: None })
    }

    pub fn add_tag(&mut self, name: &str) -> TagId {
        self.tags.insert(Tag { name: name.to_string(), visible: true, color: [0.5, 0.5, 0.5, 1.0] })
    }

    /// Rebuild caches of every mesh (call after deserializing a model by hand).
    pub fn rebuild_caches(&mut self) {
        for d in self.defs.values_mut() {
            d.mesh.rebuild_caches();
        }
    }
}
