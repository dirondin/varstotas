//! Moving vertices with stretching and classic folding of non-planar faces.

use crate::geom::{self, Projector};
use crate::mesh::{Mesh, PTOL};
use crate::triangulate::triangulate;
use crate::{EdgeId, FaceId, TOL, VertexId};
use crate::texture::{TexPlacement, carried_placement};
use glam::{DMat4, DVec2, DVec3};
use std::collections::{HashMap, HashSet};

/// A face's texture state before a transform: normal, placement and corner positions.
type FaceTexture = (DVec3, Option<TexPlacement>, Vec<(VertexId, DVec3)>);

impl Mesh {
    /// Apply transform to vertices; connected edges/faces stretch. Non-planar faces that
    /// result are triangulated and coplanar triangles re-merged (folding), so faces
    /// are never left invalid. Moved vertices landing on other vertices merge with them.
    pub fn transform_vertices(&mut self, vs: &HashSet<VertexId>, m: &DMat4) {
        self.ensure();
        let vs: Vec<VertexId> = vs.iter().copied().filter(|v| self.vertices.contains_key(*v)).collect();
        if vs.is_empty() {
            return;
        }
        let mut affected: Vec<FaceId> = Vec::new();
        for &v in &vs {
            for f in self.faces_of_vertex(v) {
                if !affected.contains(&f) {
                    affected.push(f);
                }
            }
        }
        // Texture coordinates of every corner before the move: faces keep their texture on
        // the geometry (moved whole it travels along, partly moved it stretches).
        let before: HashMap<FaceId, FaceTexture> = affected
            .iter()
            .map(|&f| {
                let face = &self.faces[f];
                let pts = face.loops().flatten().map(|v| (*v, self.pos(*v))).collect();
                (f, (face.normal, face.uv, pts))
            })
            .collect();
        for &v in &vs {
            let p = m.transform_point3(self.pos(v));
            self.v_set_pos(v, p);
        }
        let vset: HashSet<VertexId> = vs.iter().copied().collect();
        self.transform_curve_infos(&vset, m);
        let mirror = m.determinant() < 0.0;
        let normal_matrix = m.inverse().transpose();
        let mut new_edges: Vec<EdgeId> = Vec::new();
        for f in affected {
            if !self.faces.contains_key(f) {
                continue;
            }
            let whole_face = self.faces[f].loops().flatten().all(|v| vset.contains(v));
            if whole_face {
                for n in &mut self.faces[f].normals { *n = normal_matrix.transform_vector3(*n).normalize_or_zero(); }
            } else { self.faces[f].normals.clear(); }
            if mirror && whole_face {
                let face = &mut self.faces[f];
                face.reverse_shading_normals(false);
                face.outer.reverse();
                for h in &mut face.holes {
                    h.reverse();
                }
            }
            let (diag, split) = self.fix_face_planarity_ex(f);
            new_edges.extend(diag);
            let (old_n, old_uv, old_pts) = &before[&f];
            for g in if self.faces.contains_key(f) { vec![f] } else { split } {
                self.carry_texture(g, *old_n, old_uv.as_ref(), old_pts);
            }
        }
        self.merge_coplanar_at(&new_edges);
        // Merge coincident vertices.
        for &v in &vs {
            if !self.vertices.contains_key(v) {
                continue;
            }
            let p = self.pos(v);
            let d = glam::DVec3::splat(TOL);
            let other = self
                .cache
                .vgrid
                .query_box(p - d, p + d)
                .into_iter()
                .find(|w| *w != v && self.vertices.get(*w).is_some_and(|x| (x.pos - p).length() <= TOL));
            if let Some(w) = other {
                self.merge_vertices(w, v);
            }
        }
    }

    /// Recompute a face's normal; if it is no longer planar, replace it by triangles.
    /// Returns newly created (diagonal) edges.
    pub(crate) fn fix_face_planarity(&mut self, f: FaceId) -> Vec<EdgeId> {
        self.fix_face_planarity_ex(f).0
    }

    /// Re-fit face `g`'s texture placement from where its corners were before a change:
    /// `old_pts` are `(vertex, old position)` of the source face (normal `old_n`).
    pub(crate) fn carry_texture(&mut self, g: FaceId, old_n: DVec3, old_uv: Option<&TexPlacement>, old_pts: &[(VertexId, DVec3)]) {
        let Some(face) = self.faces.get(g) else { return };
        // Unpainted faces with the default projection have no texture to keep.
        if old_uv.is_none() && face.front.is_none() && face.back.is_none() {
            return;
        }
        let (mut old, mut new) = (Vec::new(), Vec::new());
        for v in face.loops().flatten() {
            if let Some((_, p)) = old_pts.iter().find(|(w, _)| w == v) {
                old.push(*p);
                new.push(self.pos(*v));
            }
        }
        if old.len() < 3 {
            return;
        }
        let n = face.normal;
        let uv = carried_placement(&old, old_n, old_uv, &new, n);
        self.faces[g].uv = uv;
    }

    /// [`Self::fix_face_planarity`], also returning the triangles that replaced a non-planar face.
    pub(crate) fn fix_face_planarity_ex(&mut self, f: FaceId) -> (Vec<EdgeId>, Vec<FaceId>) {
        let face = self.faces[f].clone();
        let outer_pos = self.loop_pos(&face.outer);
        let nn = geom::newell(&outer_pos);
        if nn.length() < 1e-12 {
            self.f_remove(f);
            return (Vec::new(), Vec::new());
        }
        let n = nn.normalize();
        let all: Vec<VertexId> = face.loops().flatten().copied().collect();
        let c = outer_pos.iter().copied().sum::<glam::DVec3>() / outer_pos.len() as f64;
        let planar = all.iter().all(|v| (self.pos(*v) - c).dot(n).abs() <= PTOL);
        if planar {
            self.f_unregister(f);
            self.f_register(f);
            return (Vec::new(), Vec::new());
        }
        let pr = Projector::new(c, n);
        let o2: Vec<DVec2> = face.outer.iter().map(|v| pr.to2(self.pos(*v))).collect();
        let h2: Vec<Vec<DVec2>> = face.holes.iter().map(|h| h.iter().map(|v| pr.to2(self.pos(*v))).collect()).collect();
        let hr: Vec<&[DVec2]> = h2.iter().map(|h| h.as_slice()).collect();
        let tris = triangulate(&o2, &hr);
        let ids: Vec<VertexId> = all.clone();
        self.f_remove(f);
        let mut diag: Vec<EdgeId> = Vec::new();
        let mut split = Vec::new();
        for t in tris {
            let tri = vec![ids[t[0]], ids[t[1]], ids[t[2]]];
            if tri[0] == tri[1] || tri[1] == tri[2] || tri[0] == tri[2] {
                continue;
            }
            let area = geom::newell(&self.loop_pos(&tri)).length();
            if area < 1e-12 {
                continue;
            }
            for i in 0..3 {
                let (a, b) = (tri[i], tri[(i + 1) % 3]);
                if !crate::mesh::loop_has_pair(&face.outer, a, b) && !face.holes.iter().any(|h| crate::mesh::loop_has_pair(h, a, b)) {
                    let (e, _) = self.e_ensure(a, b);
                    if !diag.contains(&e) {
                        diag.push(e);
                    }
                }
            }
            split.push(self.f_new(tri, Vec::new(), &face));
        }
        (diag, split)
    }
}
