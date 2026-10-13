//! Modelling operations on a `Mesh`.

mod follow_me;
mod offset;
mod push_pull;
mod transform;

use crate::mesh::Mesh;
use crate::{EdgeId, EntityRef, FaceId, VertexId};
use glam::DMat4;
use std::collections::{HashMap, HashSet, VecDeque};

impl Mesh {
    /// Vertices referenced by a set of entities (for Move/Rotate/Scale).
    pub fn vertices_of(&self, ents: &[EntityRef]) -> HashSet<VertexId> {
        let mut out = HashSet::new();
        for e in ents {
            match *e {
                EntityRef::Vertex(v) => {
                    if self.vertices.contains_key(v) {
                        out.insert(v);
                    }
                }
                EntityRef::Edge(e) => {
                    if let Some(ed) = self.edges.get(e) {
                        out.extend(ed.v);
                    }
                }
                EntityRef::Face(f) => {
                    if let Some(face) = self.faces.get(f) {
                        out.extend(face.loops().flatten().copied());
                    }
                }
                EntityRef::Instance(_) => {}
            }
        }
        out
    }

    /// Duplicate entities transformed by `m` (for Ctrl-copy), returns new refs.
    /// Faces bring their edges along; copied vertices merge with existing vertices at the
    /// destination (within TOL). Instances are copied with `m * transform`.
    pub fn duplicate(&mut self, ents: &[EntityRef], m: &DMat4) -> Vec<EntityRef> {
        self.ensure();
        let mirror = m.determinant() < 0.0;
        let mut faces: Vec<FaceId> = Vec::new();
        let mut edges: Vec<EdgeId> = Vec::new();
        let mut verts: Vec<VertexId> = Vec::new();
        let mut out = Vec::new();
        for e in ents {
            match *e {
                EntityRef::Face(f) if self.faces.contains_key(f) => {
                    if !faces.contains(&f) {
                        faces.push(f);
                    }
                    for e in self.face_edges(f) {
                        if !edges.contains(&e) {
                            edges.push(e);
                        }
                    }
                }
                EntityRef::Edge(e) if self.edges.contains_key(e) => {
                    if !edges.contains(&e) {
                        edges.push(e);
                    }
                }
                EntityRef::Vertex(v) if self.vertices.contains_key(v) => verts.push(v),
                EntityRef::Instance(i) => {
                    if let Some(inst) = self.instances.get(i) {
                        let mut c = inst.clone();
                        c.transform = *m * c.transform;
                        out.push(EntityRef::Instance(self.instances.insert(c)));
                    }
                }
                _ => {}
            }
        }
        let mut vmap: HashMap<VertexId, VertexId> = HashMap::new();
        let mut map_v = |me: &mut Mesh, v: VertexId| -> VertexId {
            if let Some(n) = vmap.get(&v) {
                return *n;
            }
            let p = m.transform_point3(me.pos(v));
            let n = me.v_get_or_new(p);
            vmap.insert(v, n);
            n
        };
        for v in verts {
            let n = map_v(self, v);
            out.push(EntityRef::Vertex(n));
        }
        for e in edges {
            let tmpl = self.edges[e].clone();
            let (a, b) = (map_v(self, tmpl.v[0]), map_v(self, tmpl.v[1]));
            if a == b {
                continue;
            }
            let ne = match self.edge_between(a, b) {
                Some(x) => x,
                None => self.e_new(a, b, Some(&tmpl)),
            };
            out.push(EntityRef::Edge(ne));
        }
        for f in faces {
            let face = self.faces[f].clone();
            let mut outer: Vec<VertexId> = face.outer.iter().map(|v| map_v(self, *v)).collect();
            let mut holes: Vec<Vec<VertexId>> =
                face.holes.iter().map(|h| h.iter().map(|v| map_v(self, *v)).collect()).collect();
            if mirror {
                outer.reverse();
                for h in &mut holes {
                    h.reverse();
                }
            }
            if let Some(ex) = self.find_face_with_outer(&outer) {
                out.push(EntityRef::Face(ex));
                continue;
            }
            let nf = self.f_new(outer, holes, &face);
            out.push(EntityRef::Face(nf));
        }
        out
    }

    /// Reverse a face (flip normal, swap front/back materials).
    pub fn reverse_face(&mut self, f: FaceId) {
        let Some(face) = self.faces.get_mut(f) else { return };
        face.outer.reverse();
        for h in &mut face.holes {
            h.reverse();
        }
        face.normal = -face.normal;
        std::mem::swap(&mut face.front, &mut face.back);
    }

    /// Make all faces in the connected component of `f` consistent with `f` (propagates across
    /// edges shared by exactly two faces: they must traverse the edge in opposite directions).
    pub fn orient_faces_like(&mut self, f: FaceId) {
        if !self.faces.contains_key(f) {
            return;
        }
        let mut seen: HashSet<FaceId> = HashSet::new();
        let mut q = VecDeque::new();
        seen.insert(f);
        q.push_back(f);
        while let Some(cur) = q.pop_front() {
            let face = self.faces[cur].clone();
            for l in face.loops() {
                let n = l.len();
                for i in 0..n {
                    let (a, b) = (l[i], l[(i + 1) % n]);
                    let Some(e) = self.edge_between(a, b) else { continue };
                    let fs = self.edge_faces(e);
                    if fs.len() != 2 {
                        continue;
                    }
                    let g = if fs[0] == cur { fs[1] } else { fs[0] };
                    if seen.contains(&g) {
                        continue;
                    }
                    if self.face_traverses(g, a, b) == Some(true) {
                        self.reverse_face(g);
                    }
                    seen.insert(g);
                    q.push_back(g);
                }
            }
        }
    }
}
