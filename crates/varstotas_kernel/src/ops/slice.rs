//! Slice: cut geometry with a plane.
//!
//! The faces of the target are imprinted with their intersection with the plane (edges along
//! the cut, faces split, like Intersect Faces with an infinite face), loose edges crossing it
//! are split there. Groups and components are cut inside (nested ones too, in their own
//! coordinates); one shared with other copies is made unique first, so the copies stay whole.
//!
//! With `split`, every cut body falls apart into two: the part on the plane's normal side
//! stays, the rest becomes a new group / component next to it (loose geometry of the active
//! context is grouped first, so it ends up as two groups). Closed cut loops are capped with
//! faces on both halves, so a cut solid gives two solids.

use super::intersect::{IPoly, emit_intervals, line_params};
use crate::geom::PointLoc;
use crate::mesh::{Mesh, PTOL};
use crate::model::Model;
use crate::topology::{Classify, Orient};
use crate::{Aabb, DefId, EdgeId, EntityRef, FaceId, InstanceId, OpError, OpResult, Plane, VertexId};
use glam::{DMat4, DVec3};
use std::collections::HashSet;

/// What [`Model::slice`] did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SliceOutcome {
    /// Edges created along the cut (in all cut definitions).
    pub edges: usize,
    /// With `split`: the instances of the active context now holding the parts (each cut body
    /// gives its kept part and its new one).
    pub parts: Vec<InstanceId>,
}

/// Pieces of the plane's intersection with face polygon `p` (its region, boundary included);
/// none for a face parallel to the plane.
fn plane_segments(p: &IPoly, plane: &Plane, out: &mut Vec<(DVec3, DVec3)>) {
    let (na, nb) = (p.plane.normal, plane.normal);
    let d = na.cross(nb);
    let s = d.length();
    if s < 1e-9 {
        return;
    }
    let (ca, cb) = (na.dot(p.plane.origin), nb.dot(plane.origin));
    let p0 = (nb.cross(d) * ca + d.cross(na) * cb) / (s * s);
    let dir = d / s;
    let mut ts = Vec::new();
    line_params(p, plane, p0, dir, &mut ts);
    if ts.len() < 2 {
        return;
    }
    emit_intervals(&mut ts, |t| p0 + dir * t, |m| p.loc(m) != PointLoc::Outside, out);
}

/// Whether box `b` (in the plane's coordinates) has corners strictly on both sides.
fn box_crosses(b: &Aabb, plane: &Plane) -> bool {
    let ds = b.corners().map(|c| plane.distance(c));
    ds.iter().any(|d| *d > PTOL) && ds.iter().any(|d| *d < -PTOL)
}

/// `plane` (in a parent's coordinates) in the coordinates of a child placed by `t`.
fn plane_in_child(plane: &Plane, t: &DMat4) -> Plane {
    let origin = t.inverse().transform_point3(plane.origin);
    let normal = t.transpose().transform_vector3(plane.normal);
    Plane::new(origin, normal)
}

/// Side of a face after slicing: the sign of its farthest vertex; a face lying in the plane
/// belongs to the part it bounds (facing along the normal = the part behind the plane).
fn face_positive(mesh: &Mesh, f: FaceId, plane: &Plane) -> bool {
    let face = &mesh.faces[f];
    let mut far = 0.0f64;
    for v in face.loops().flatten() {
        let d = plane.distance(mesh.pos(*v));
        if d.abs() > far.abs() {
            far = d;
        }
    }
    if far.abs() > PTOL { far > 0.0 } else { face.normal.dot(plane.normal) < 0.0 }
}

impl Mesh {
    /// Cut `faces` and the loose `edges` with `plane`: edges along the cut, faces split, loose
    /// edges split where they cross it. Returns the new edges.
    pub fn slice(&mut self, faces: &[FaceId], edges: &[EdgeId], plane: Plane) -> Vec<EdgeId> {
        self.ensure();
        let mut segs = Vec::new();
        for &f in faces {
            let Some(face) = self.faces.get(f) else { continue };
            let tr = |l: &Vec<VertexId>| -> Vec<DVec3> { l.iter().map(|v| self.pos(*v)).collect() };
            if let Some(p) = IPoly::new(tr(&face.outer), face.holes.iter().map(tr).collect(), 0, true) {
                plane_segments(&p, &plane, &mut segs);
            }
        }
        let mut crossings = Vec::new();
        for &e in edges {
            let Some(ed) = self.edges.get(e) else { continue };
            let (a, b) = (self.pos(ed.v[0]), self.pos(ed.v[1]));
            let (da, db) = (plane.distance(a), plane.distance(b));
            if (da > PTOL && db < -PTOL) || (da < -PTOL && db > PTOL) {
                crossings.push(a + (b - a) * (da / (da - db)));
            }
        }
        let created = self.imprint_segments(&segs);
        for p in crossings {
            self.vertex_at(p);
        }
        created
    }

    /// Keep the part on one side of `plane` (`positive` = its normal side): faces of the other
    /// side are removed, then the edges left without faces there (`loose` = edges that had no
    /// faces before, kept on the plane by the positive part only).
    fn keep_side(&mut self, plane: &Plane, positive: bool, loose: &HashSet<EdgeId>) {
        let faces: Vec<EntityRef> =
            self.faces.keys().filter(|f| face_positive(self, *f, plane) != positive).map(EntityRef::Face).collect();
        self.erase(&faces);
        let edges: Vec<EntityRef> = self
            .edges
            .keys()
            .filter(|e| self.edge_faces(*e).is_empty())
            .filter(|e| {
                let (a, b) = self.edge_segment(*e);
                let d = plane.distance((a + b) * 0.5);
                if d.abs() <= PTOL { !(positive && loose.contains(e)) } else { (d > 0.0) != positive }
            })
            .map(EntityRef::Edge)
            .collect();
        self.erase(&edges);
    }

    /// Faces on the closed loops of edges lying in `plane`, facing `outward`.
    fn cap(&mut self, plane: &Plane, outward: DVec3) {
        self.ensure();
        let set: HashSet<EdgeId> = self
            .edges
            .iter()
            .filter(|(_, ed)| ed.v.iter().all(|v| plane.distance(self.pos(*v)).abs() <= PTOL))
            .map(|(e, _)| e)
            .collect();
        if set.is_empty() {
            return;
        }
        let mut seeds: Vec<VertexId> = set.iter().flat_map(|e| self.edges[*e].v).collect();
        seeds.sort();
        seeds.dedup();
        let made = self.refresh_plane(*plane, &seeds, &[], &Classify::NewEdges(&set), Orient::prefer(outward));
        // A closed loop around empty space (the hole of a tube) is no cap: the walls along
        // its border face into it, while a cap's walls face away from it.
        let holes: Vec<EntityRef> = made
            .iter()
            .filter(|r| r.created && self.faces.contains_key(r.face) && self.walls_face_inward(r.face))
            .map(|r| EntityRef::Face(r.face))
            .collect();
        self.erase(&holes);
    }

    /// Whether most walls along the border of face `f` (the other faces at its edges) have
    /// normals pointing into `f`.
    fn walls_face_inward(&self, f: FaceId) -> bool {
        let face = &self.faces[f];
        let mut votes = 0i32;
        for l in face.loops() {
            for k in 0..l.len() {
                let (a, b) = (l[k], l[(k + 1) % l.len()]);
                let Some(e) = self.edge_between(a, b) else { continue };
                // Left of the loop, seen from the front, is inside the face.
                let inward = face.normal.cross(self.pos(b) - self.pos(a));
                for w in self.edge_faces(e) {
                    if w == f {
                        continue;
                    }
                    let s = self.faces[w].normal.dot(inward);
                    if s.abs() > 1e-9 {
                        votes += if s > 0.0 { 1 } else { -1 };
                    }
                }
            }
        }
        votes > 0
    }

    fn is_blank(&self) -> bool {
        self.faces.is_empty() && self.edges.is_empty() && self.instances.is_empty()
    }
}

impl Model {
    /// Slice with `plane` (in the coordinates of the active context `path`). Targets: the
    /// selection `sel` (faces, loose edges, groups / components), or with nothing selected
    /// everything in the active context. See the module docs for `split`.
    pub fn slice(&mut self, path: &[InstanceId], sel: &[EntityRef], plane: Plane, split: bool) -> OpResult<SliceOutcome> {
        let ad = self.def_at(path).ok_or(OpError::NotFound)?;
        let plane = Plane::new(plane.origin, plane.normal);
        let (mut faces, mut edges, mut insts) = (Vec::new(), Vec::new(), Vec::new());
        {
            let mesh = &self.defs[ad].mesh;
            if sel.is_empty() {
                faces.extend(mesh.faces.iter().filter(|(_, f)| !f.hidden).map(|(f, _)| f));
                edges.extend(mesh.edges.keys().filter(|e| mesh.edge_faces(*e).is_empty()));
                insts.extend(mesh.instances.iter().filter(|(_, i)| !i.hidden && !i.locked).map(|(i, _)| i));
            } else {
                for e in sel {
                    match *e {
                        EntityRef::Face(f) if mesh.faces.contains_key(f) => faces.push(f),
                        EntityRef::Edge(e) if mesh.edges.contains_key(e) => edges.push(e),
                        EntityRef::Instance(i) if mesh.instances.get(i).is_some_and(|x| !x.locked) => insts.push(i),
                        _ => {}
                    }
                }
            }
        }
        let mut out = SliceOutcome::default();
        if !faces.is_empty() || !edges.is_empty() {
            let mesh = &self.defs[ad].mesh;
            let face_pts = faces.iter().flat_map(|f| mesh.faces[*f].loops().flatten().map(|v| mesh.pos(*v)));
            let edge_pts = edges.iter().flat_map(|e| <[DVec3; 2]>::from(mesh.edge_segment(*e)));
            let crosses = Aabb::from_points(face_pts.chain(edge_pts)).is_some_and(|b| box_crosses(&b, &plane));
            if crosses {
                if split {
                    // Loose geometry becomes a group first: it is split like any group.
                    let mut ents: Vec<EntityRef> = faces.iter().map(|f| EntityRef::Face(*f)).collect();
                    ents.extend(edges.iter().map(|e| EntityRef::Edge(*e)));
                    insts.push(self.make_group(path, &ents)?);
                } else {
                    out.edges += self.defs[ad].mesh.slice(&faces, &edges, plane).len();
                }
            }
        }
        for i in insts {
            let Some(inst) = self.defs[ad].mesh.instances.get(i) else { continue };
            let Some(b) = self.def_bounds(inst.def) else { continue };
            if !box_crosses(&b.transformed(&inst.transform), &plane) {
                continue;
            }
            if self.can_make_unique(path, i) {
                self.make_unique(path, i)?;
            }
            let inst = self.defs[ad].mesh.instances[i].clone();
            let mut cpath = path.to_vec();
            cpath.push(i);
            let neg = self.slice_def(&cpath, inst.def, &plane_in_child(&plane, &inst.transform), split, &mut out.edges, 0)?;
            if let Some(nd) = neg {
                let mut copy = inst.clone();
                copy.def = nd;
                let ni = self.defs[ad].mesh.instances.insert(copy);
                out.parts.extend([i, ni]);
            }
        }
        Ok(out)
    }

    /// Cut definition `d` (reached by `cpath`, used once) with `plane` in its coordinates.
    /// With `split`, returns the new definition holding the part behind the plane, if both
    /// parts have something.
    fn slice_def(&mut self, cpath: &[InstanceId], d: DefId, plane: &Plane, split: bool, edges: &mut usize, depth: usize) -> OpResult<Option<DefId>> {
        if depth > 64 {
            return Ok(None);
        }
        // Cut geometry is no longer what a generator or a rig variant would rebuild.
        self.defs[d].generated = None;
        self.defs[d].rigged = None;
        let mesh = &mut self.defs[d].mesh;
        let faces: Vec<FaceId> = mesh.faces.keys().collect();
        let loose: Vec<EdgeId> = mesh.edges.keys().filter(|e| mesh.edge_faces(*e).is_empty()).collect();
        *edges += mesh.slice(&faces, &loose, *plane).len();
        // Nested groups / components crossing the plane are cut too.
        let mut nested_split: Vec<(InstanceId, DefId)> = Vec::new();
        let nested: Vec<InstanceId> = self.defs[d].mesh.instances.iter().filter(|(_, i)| !i.hidden).map(|(i, _)| i).collect();
        for i in nested {
            let inst = &self.defs[d].mesh.instances[i];
            let Some(b) = self.def_bounds(inst.def) else { continue };
            if !box_crosses(&b.transformed(&inst.transform), plane) {
                continue;
            }
            if self.can_make_unique(cpath, i) {
                self.make_unique(cpath, i)?;
            }
            let inst = self.defs[d].mesh.instances[i].clone();
            let mut sub = cpath.to_vec();
            sub.push(i);
            if let Some(nd) = self.slice_def(&sub, inst.def, &plane_in_child(plane, &inst.transform), split, edges, depth + 1)? {
                nested_split.push((i, nd));
            }
        }
        if !split {
            return Ok(None);
        }
        let mesh = &self.defs[d].mesh;
        let loose: HashSet<EdgeId> = mesh.edges.keys().filter(|e| mesh.edge_faces(*e).is_empty()).collect();
        let (mut pos, mut neg) = (mesh.clone(), mesh.clone());
        pos.keep_side(plane, true, &loose);
        neg.keep_side(plane, false, &loose);
        // Instances: a split one keeps its def on the positive side and takes the new one on
        // the other; whole ones go by the centre of their box.
        let split_ids: HashSet<InstanceId> = nested_split.iter().map(|(i, _)| *i).collect();
        for (i, inst) in &mesh.instances {
            if split_ids.contains(&i) {
                continue;
            }
            let c = self.def_bounds(inst.def).map_or(inst.transform.w_axis.truncate(), |b| b.transformed(&inst.transform).center());
            if plane.distance(c) >= 0.0 {
                neg.instances.remove(i);
            } else {
                pos.instances.remove(i);
            }
        }
        for (i, nd) in &nested_split {
            neg.instances[*i].def = *nd;
        }
        if pos.is_blank() || neg.is_blank() {
            return Ok(None);
        }
        pos.cap(plane, -plane.normal);
        neg.cap(plane, plane.normal);
        let mut nd = self.defs[d].clone();
        nd.name = self.free_def_name(&nd.name);
        nd.mesh = neg;
        self.defs[d].mesh = pos;
        Ok(Some(self.defs.insert(nd)))
    }
}
