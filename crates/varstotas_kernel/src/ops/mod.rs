//! Modelling operations on a `Mesh`.

mod follow_me;
mod intersect;
mod offset;
mod push_pull;
mod transform;

pub use intersect::IntersectScope;

use crate::mesh::Mesh;
use crate::geom::{self, PointLoc, Projector};
use crate::mesh::PTOL;
use crate::model::Model;
use crate::topology::interior_sample;
use crate::{Aabb, EdgeId, EntityRef, FaceId, InstanceId, OpError, OpResult, TOL, VertexId};
use glam::{DMat3, DMat4, DVec2, DVec3};
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
        // Copied curves get fresh ids (and their metadata, transformed).
        let src_curves = self.curves.clone();
        let cids: Vec<u32> = edges.iter().filter_map(|e| self.edges[*e].curve).collect();
        let cmap = self.remap_curves(&src_curves, cids, m);
        for e in edges {
            let mut tmpl = self.edges[e].clone();
            tmpl.curve = tmpl.curve.and_then(|c| cmap.get(&c).copied());
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
        let mut verts: Vec<VertexId> = vmap.values().copied().collect();
        verts.sort(); // deterministic welding order
        verts.dedup();
        self.absorb_refs(&verts, out)
    }

    /// Make geometry just added at `verts` (with entities `out`) sticky with the rest of the
    /// mesh -- see [`Mesh::absorb`] -- and return `out` updated to the entities now occupying
    /// the same places (ids change when edges/faces are split or merged).
    pub(crate) fn absorb_refs(&mut self, verts: &[VertexId], out: Vec<EntityRef>) -> Vec<EntityRef> {
        let faces: Vec<FaceId> = out.iter().filter_map(|e| if let EntityRef::Face(f) = e { Some(*f) } else { None }).collect();
        let fp = Footprint::of(self, &out);
        if !self.absorb(verts, &faces) {
            return out;
        }
        fp.locate(self)
    }

    /// Weld freshly added geometry (vertices `verts`, faces `faces`) to the rest of the mesh:
    /// merge coincident vertices, split edges at T-junctions and crossings, split faces by edges
    /// now lying inside them, and merge coplanar overlapping faces into one partition (no
    /// overlapping faces remain; the smaller face's attributes win on the overlap).
    /// Faces are not intersected with faces in other planes (that is Intersect Faces).
    /// Returns true if anything beyond the plain addition changed.
    pub(crate) fn absorb(&mut self, verts: &[VertexId], faces: &[FaceId]) -> bool {
        let split_at = self.weld(verts);
        let mut changed = !split_at.is_empty() || verts.iter().any(|v| !self.vertices.contains_key(*v));
        let alive: Vec<FaceId> = faces.iter().copied().filter(|f| self.faces.contains_key(*f)).collect();
        changed |= !self.resolve_overlaps(&alive, true).is_empty();
        let mut check: Vec<FaceId> = faces.iter().copied().filter(|f| self.faces.contains_key(*f)).collect();
        // Faces at the new vertices (an added edge may be a chord of an existing face whose
        // loop already had both its endpoints) and at the split points.
        for v in verts.iter().copied().chain(split_at) {
            if self.vertices.contains_key(v) {
                for g in self.faces_of_vertex(v) {
                    if !check.contains(&g) {
                        check.push(g);
                    }
                }
            }
        }
        changed |= self.split_faces_by_inner_edges(&check) > 0;
        changed
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

    /// Reverse several faces (Context → Reverse Faces). Dead ids are skipped; returns how many
    /// faces were reversed.
    pub fn reverse_faces(&mut self, faces: &[FaceId]) -> usize {
        let mut seen: HashSet<FaceId> = HashSet::new();
        let mut n = 0;
        for &f in faces {
            if self.faces.contains_key(f) && seen.insert(f) {
                self.reverse_face(f);
                n += 1;
            }
        }
        n
    }

    /// Make all faces in the connected component of `f` consistent with `f` (propagates across
    /// edges shared by exactly two faces: they must traverse the edge in opposite directions).
    /// Returns how many faces were reversed.
    pub fn orient_faces_like(&mut self, f: FaceId) -> usize {
        let mut flipped = 0;
        if !self.faces.contains_key(f) {
            return 0;
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
                        flipped += 1;
                    }
                    seen.insert(g);
                    q.push_back(g);
                }
            }
        }
        flipped
    }
}

/// A face's projector, 2D outer loop, 2D holes and bounds.
type FaceShape = (Projector, Vec<DVec2>, Vec<Vec<DVec2>>, Aabb);

/// Where a set of entities lies, to find them again after welding changed their ids.
struct Footprint {
    verts: Vec<DVec3>,
    edges: Vec<(DVec3, DVec3)>,
    /// (projector, outer, holes, bounds) of each face.
    faces: Vec<FaceShape>,
    insts: Vec<InstanceId>,
}

impl Footprint {
    fn of(m: &Mesh, ents: &[EntityRef]) -> Self {
        let mut fp = Footprint { verts: Vec::new(), edges: Vec::new(), faces: Vec::new(), insts: Vec::new() };
        for e in ents {
            match *e {
                EntityRef::Vertex(v) if m.vertices.contains_key(v) => fp.verts.push(m.pos(v)),
                EntityRef::Edge(e) if m.edges.contains_key(e) => fp.edges.push(m.edge_segment(e)),
                EntityRef::Face(f) if m.faces.contains_key(f) => {
                    let (pr, o, h) = m.face_2d(f);
                    let Some(b) = m.face_aabb(f) else { continue };
                    fp.faces.push((pr, o, h, b));
                }
                EntityRef::Instance(i) => fp.insts.push(i),
                _ => {}
            }
        }
        fp
    }

    /// Entities of `m` occupying the footprint: vertices at the points, edges lying on the
    /// segments, faces of the same planes whose interior sample is inside the polygons.
    fn locate(&self, m: &Mesh) -> Vec<EntityRef> {
        let mut out = Vec::new();
        let mut seen: HashSet<EntityRef> = HashSet::new();
        let mut push = |r: EntityRef, out: &mut Vec<EntityRef>| {
            if seen.insert(r) {
                out.push(r);
            }
        };
        for p in &self.verts {
            if let Some(v) = m.v_find(*p, TOL) {
                push(EntityRef::Vertex(v), &mut out);
            }
        }
        for (a, b) in &self.edges {
            let mut es = m.cache.egrid.query_segment(*a, *b);
            es.sort();
            for e in es {
                let Some(ed) = m.edges.get(e) else { continue };
                let on = |v: VertexId| geom::point_seg_dist(m.pos(v), *a, *b).0 <= TOL;
                if on(ed.v[0]) && on(ed.v[1]) {
                    push(EntityRef::Edge(e), &mut out);
                }
            }
        }
        for (pr, o, h, b) in &self.faces {
            let b = b.inflate(TOL);
            let mut fs = m.cache.fgrid.query_box(b.min, b.max);
            fs.sort();
            for f in fs {
                let Some(face) = m.faces.get(f) else { continue };
                if face.normal.dot(pr.n).abs() < 1.0 - 1e-6 || face.outer.iter().any(|v| (m.pos(*v) - pr.origin).dot(pr.n).abs() > PTOL) {
                    continue;
                }
                let (fpr, fo, fh) = m.face_2d(f);
                let sample = fpr.to3(interior_sample(&fo, &fh));
                if geom::point_in_region(pr.to2(sample), o, h, TOL) == PointLoc::Inside {
                    push(EntityRef::Face(f), &mut out);
                }
            }
        }
        for i in &self.insts {
            if m.instances.contains_key(*i) {
                push(EntityRef::Instance(*i), &mut out);
            }
        }
        out
    }
}

/// Reflection about the plane through `n * c` with unit normal `n`.
fn reflection(n: DVec3, c: f64) -> DMat4 {
    let r = DMat3::IDENTITY - DMat3::from_cols(n * n.x, n * n.y, n * n.z) * 2.0;
    DMat4::from_mat3_translation(r, n * (2.0 * c))
}

impl Model {
    /// Flip Along (Context → Flip Along red/green/blue): mirror the selection in the context at
    /// `path` about the plane through the centre of its extent along `axis` (context
    /// coordinates, e.g. `DVec3::X` for red), perpendicular to `axis`. Instances get a mirrored
    /// transform; loose geometry is transformed (connected unselected geometry stretches) and
    /// its fully selected faces are reversed so normals keep pointing outward.
    pub fn flip_along(&mut self, path: &[InstanceId], sel: &[EntityRef], axis: DVec3) -> OpResult<()> {
        let d = self.def_at(path).ok_or(OpError::NotFound)?;
        let n = axis.normalize_or_zero();
        if n == DVec3::ZERO || !n.is_finite() {
            return Err(OpError::Invalid("flip axis is zero".into()));
        }
        let mesh = &self.defs[d].mesh;
        let verts = mesh.vertices_of(sel);
        let insts: Vec<InstanceId> = sel
            .iter()
            .filter_map(|e| match e {
                EntityRef::Instance(i) if mesh.instances.contains_key(*i) => Some(*i),
                _ => None,
            })
            .collect();
        let mut ext: Option<(f64, f64)> = None;
        let mut include = |p: DVec3| {
            let t = p.dot(n);
            ext = Some(ext.map_or((t, t), |(lo, hi)| (lo.min(t), hi.max(t))));
        };
        for v in &verts {
            include(mesh.vertices[*v].pos);
        }
        for i in &insts {
            let inst = &mesh.instances[*i];
            if let Some(b) = self.def_bounds(inst.def) {
                for c in b.corners() {
                    include(inst.transform.transform_point3(c));
                }
            }
        }
        let Some((lo, hi)) = ext else { return Err(OpError::Invalid("nothing to flip".into())) };
        let m = reflection(n, (lo + hi) * 0.5);
        let mesh = &mut self.defs[d].mesh;
        for i in insts {
            let inst = &mut mesh.instances[i];
            inst.transform = m * inst.transform;
        }
        if !verts.is_empty() {
            mesh.transform_vertices(&verts, &m);
        }
        Ok(())
    }
}
