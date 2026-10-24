//! Chamfer and Fillet: replace an edge between two faces by a flat bevel face or by a rounded
//! strip of faces (an arc of `segments` pieces tangent to both faces).
//!
//! The profile is built in the plane perpendicular to the edge at each end and then slid along
//! the edge onto the end plane (the plane of the two other edges at that end), so ends meeting
//! the edge at an angle work too. Ends must be simple: besides the two faces of the edge, at most
//! one more face and exactly one more edge in each of the two faces.

use crate::geom;
use crate::mesh::Mesh;
use crate::{EdgeId, FaceId, OpError, OpResult, TOL, VertexId};
use glam::DVec3;
use varstotas_i18n::tr;

/// Flat bevel (setback along both faces) or a round of a given radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChamferKind {
    /// Setback distance measured along each face, perpendicular to the edge.
    Flat { distance: f64 },
    /// Radius of the round and the number of segments of the arc.
    Round { radius: f64, segments: u32 },
}

/// What a chamfer created: the new faces and, for each old edge that was shortened (the edges
/// at the ends of the chamfered edge), the edge that replaced it.
#[derive(Clone, Debug, Default)]
pub struct ChamferResult {
    pub faces: Vec<FaceId>,
    pub replaced: Vec<(EdgeId, EdgeId)>,
}

/// One end of the edge: the vertex, its other edge in each face, and the far ends of those.
struct End {
    v: VertexId,
    /// Other edge at `v` in face 1 / face 2 and their far vertices.
    u: [EdgeId; 2],
    x: [VertexId; 2],
    /// Third face at the vertex (the end face), if any.
    g: Option<FaceId>,
    /// End plane normal (spanned by the two other edges).
    n: DVec3,
}

impl Mesh {
    /// Chamfer / fillet edge `e`. Fails (leaving the mesh unchanged) if the edge does not have
    /// exactly two non-coplanar faces, an end is not simple, or the size does not fit.
    pub fn chamfer_edge(&mut self, e: EdgeId, kind: ChamferKind) -> OpResult<ChamferResult> {
        self.ensure();
        let ed = self.edges.get(e).ok_or(OpError::NotFound)?.clone();
        let faces = self.edge_faces(e);
        let [f1, f2] = faces[..] else {
            return Err(OpError::Invalid(tr!("kernel-err-the-edge-must-be-shared-by")));
        };
        let (a, b) = (ed.v[0], ed.v[1]);
        let (pa, pb) = (self.pos(a), self.pos(b));
        let len = pa.distance(pb);
        if len < TOL {
            return Err(OpError::Invalid(tr!("kernel-err-the-edge-is-too-short")));
        }
        let d = (pb - pa) / len;
        // In-face directions perpendicular to the edge, pointing into each face (the inside of a
        // loop is on its left seen from the front).
        let into = |m: &Mesh, f: FaceId| -> OpResult<DVec3> {
            let fwd = m.face_traverses(f, a, b).ok_or(OpError::Invalid(tr!("kernel-err-broken-face-loop")))?;
            let dir = if fwd { d } else { -d };
            Ok(m.faces[f].normal.cross(dir).normalize_or_zero())
        };
        let (t1, t2) = (into(self, f1)?, into(self, f2)?);
        let cos_phi = t1.dot(t2).clamp(-1.0, 1.0);
        if cos_phi < -1.0 + 1e-6 || t1 == DVec3::ZERO || t2 == DVec3::ZERO {
            return Err(OpError::Invalid(tr!("kernel-err-the-faces-are-coplanar")));
        }
        let phi = cos_phi.acos();
        // Profile: offsets from the edge in its perpendicular plane, from face 1 to face 2.
        let (profile, setback) = match kind {
            ChamferKind::Flat { distance } => {
                if !distance.is_finite() || distance <= TOL {
                    return Err(OpError::Invalid(tr!("kernel-err-the-distance-must-be-positive")));
                }
                (vec![t1 * distance, t2 * distance], distance)
            }
            ChamferKind::Round { radius, segments } => {
                if !radius.is_finite() || radius <= TOL {
                    return Err(OpError::Invalid(tr!("kernel-err-the-radius-must-be-positive")));
                }
                let n = segments.clamp(1, 64) as usize;
                let half = phi * 0.5;
                let s = radius / half.tan();
                let bis = (t1 + t2).normalize();
                let c = bis * (radius / half.sin());
                // Arc from the tangent point on face 1 to the one on face 2 around `c`.
                let (r0, r1) = (t1 * s - c, t2 * s - c);
                let axis = r0.cross(r1).normalize_or_zero();
                let sweep = std::f64::consts::PI - phi;
                let pts = (0..=n)
                    .map(|k| {
                        if k == 0 {
                            return t1 * s;
                        }
                        if k == n {
                            return t2 * s;
                        }
                        let q = glam::DQuat::from_axis_angle(axis, sweep * k as f64 / n as f64);
                        c + q * r0
                    })
                    .collect();
                (pts, s)
            }
        };
        let ends = [self.chamfer_end(a, e, f1, f2)?, self.chamfer_end(b, e, f1, f2)?];
        // Section points at each end, slid along the edge onto the end plane.
        let mut sections: Vec<Vec<DVec3>> = Vec::new();
        for end in &ends {
            let p = self.pos(end.v);
            let dn = end.n.dot(d);
            if dn.abs() < 1e-6 {
                return Err(OpError::Invalid(tr!("kernel-err-an-end-lies-along-the-edge")));
            }
            let pts: Vec<DVec3> = profile.iter().map(|o| p + *o - d * (end.n.dot(*o) / dn)).collect();
            // The first / last point must land inside the shortened edges.
            for (k, i) in [(0usize, 0usize), (pts.len() - 1, 1)] {
                let x = self.pos(end.x[i]);
                let (dist, t) = geom::point_seg_dist(pts[k], p, x);
                // Landing exactly on the far vertex is fine (two equal chamfers meet there).
                if dist > 1e-6 * (1.0 + setback) || (t > 1.0 - TOL / p.distance(x).max(TOL) && pts[k].distance(x) > TOL) {
                    return Err(OpError::Invalid(tr!("kernel-err-the-chamfer-is-too-big-for-2")));
                }
            }
            sections.push(pts);
        }
        // Each end must stay inside the edge: sections must not cross each other.
        for (p0, p1) in sections[0].iter().zip(&sections[1]) {
            if (*p1 - *p0).dot(d) <= TOL {
                return Err(OpError::Invalid(tr!("kernel-err-the-chamfer-is-too-big-for")));
            }
        }

        // ---- topology (nothing has changed so far) ----
        let last = profile.len() - 1;
        let new_verts = |m: &mut Mesh, end: &End, pts: &[DVec3]| -> Vec<VertexId> {
            pts.iter()
                .enumerate()
                .map(|(k, p)| match k {
                    0 if m.pos(end.x[0]).distance(*p) <= TOL => end.x[0],
                    k if k == last && m.pos(end.x[1]).distance(*p) <= TOL => end.x[1],
                    _ => m.v_new(*p),
                })
                .collect()
        };
        let va = new_verts(self, &ends[0], &sections[0]);
        let vb = new_verts(self, &ends[1], &sections[1]);
        let face_tmpls = [self.faces[f1].clone(), self.faces[f2].clone()];
        // Faces 1 and 2 lose the edge: a / b become the first / last section point.
        let replace = |m: &Mesh, f: FaceId, map: &[(VertexId, Vec<VertexId>)]| -> (Vec<VertexId>, Vec<Vec<VertexId>>) {
            let face = &m.faces[f];
            let sub = |l: &Vec<VertexId>| -> Vec<VertexId> {
                let mut out: Vec<VertexId> = l
                    .iter()
                    .flat_map(|v| match map.iter().find(|(x, _)| x == v) {
                        Some((_, r)) => r.clone(),
                        None => vec![*v],
                    })
                    .collect();
                // A section point reusing a neighbour vertex: drop the repeat.
                out.dedup();
                while out.len() > 1 && out.first() == out.last() {
                    out.pop();
                }
                out
            };
            (sub(&face.outer), face.holes.iter().map(sub).collect())
        };
        let mut updates: Vec<(FaceId, Vec<VertexId>, Vec<Vec<VertexId>>)> = Vec::new();
        let (o, h) = replace(self, f1, &[(a, vec![va[0]]), (b, vec![vb[0]])]);
        updates.push((f1, o, h));
        let (o, h) = replace(self, f2, &[(a, vec![va[last]]), (b, vec![vb[last]])]);
        updates.push((f2, o, h));
        // End faces: the vertex becomes the whole section, ordered to follow the loop.
        for (end, vs) in ends.iter().zip([&va, &vb]) {
            let Some(g) = end.g else { continue };
            let after_x1 = self.faces[g].loops().any(|l| {
                let n = l.len();
                (0..n).any(|i| l[i] == end.x[0] && l[(i + 1) % n] == end.v)
            });
            let seq: Vec<VertexId> = if after_x1 { vs.clone() } else { vs.iter().rev().copied().collect() };
            let (o, h) = replace(self, g, &[(end.v, seq)]);
            updates.push((g, o, h));
        }
        let old_edges: Vec<EdgeId> = std::iter::once(e).chain(ends.iter().flat_map(|x| x.u)).collect();
        let tmpl_edges: Vec<crate::Edge> = ends.iter().flat_map(|x| x.u).map(|u| self.edges[u].clone()).collect();
        for (f, o, h) in updates {
            if o.len() < 3 {
                self.f_remove(f);
            } else {
                self.f_set_loops(f, o, h.into_iter().filter(|l| l.len() >= 3).collect());
            }
        }
        // Strip faces between consecutive section points (normal toward the outside of both faces).
        let outward = face_tmpls[0].normal + face_tmpls[1].normal;
        let mut created = Vec::new();
        for k in 0..last {
            let mut l = vec![va[k], va[k + 1], vb[k + 1], vb[k]];
            l.dedup();
            if l.len() < 3 {
                continue;
            }
            let mut n = geom::newell(&self.loop_pos(&l));
            if n.dot(outward) < 0.0 {
                l.reverse();
                n = -n;
            }
            let mut tmpl = face_tmpls[if k * 2 < last { 0 } else { 1 }].clone();
            tmpl.normal = n.normalize_or(DVec3::Z);
            tmpl.uv = None;
            created.push(self.f_new(l, Vec::new(), &tmpl));
        }
        // A face may also have been flat-free (no end face): the open end is closed by edges.
        for (end, vs) in ends.iter().zip([&va, &vb]) {
            if end.g.is_none() {
                for k in 0..last {
                    self.e_ensure(vs[k], vs[k + 1]);
                }
            }
        }
        // Shortened edges keep the attributes of the old ones.
        let mut replaced = Vec::new();
        for (i, (end, vs)) in ends.iter().zip([&va, &vb]).enumerate() {
            for (j, (x, v)) in [(end.x[0], vs[0]), (end.x[1], vs[last])].into_iter().enumerate() {
                if x == v {
                    continue;
                }
                if let Some(ne) = self.edge_between(x, v) {
                    let t = &tmpl_edges[i * 2 + j];
                    let ned = &mut self.edges[ne];
                    (ned.soft, ned.smooth, ned.hidden, ned.tag) = (t.soft, t.smooth, t.hidden, t.tag);
                    replaced.push((end.u[j], ne));
                }
            }
        }
        // Round: the arc edges and the tangent lines are soft and smooth.
        if matches!(kind, ChamferKind::Round { .. }) {
            for k in 0..=last {
                if let Some(se) = self.edge_between(va[k], vb[k]) {
                    let s = &mut self.edges[se];
                    s.soft = true;
                    s.smooth = true;
                }
            }
        }
        for oe in old_edges {
            if self.edges.contains_key(oe) && self.edge_faces(oe).is_empty() {
                self.e_remove(oe);
            }
        }
        self.remove_if_orphan(a);
        self.remove_if_orphan(b);
        Ok(ChamferResult { faces: created, replaced })
    }

    /// The end of edge `e` at `v` (see [`End`]).
    fn chamfer_end(&self, v: VertexId, e: EdgeId, f1: FaceId, f2: FaceId) -> OpResult<End> {
        let bad = || OpError::Invalid(tr!("kernel-err-a-corner-of-the-edge-is"));
        let edges: Vec<EdgeId> = self.vertex_edges(v).into_iter().filter(|x| *x != e).collect();
        let in_face = |f: FaceId| -> OpResult<EdgeId> {
            let fe = self.face_edges(f);
            let mut it = edges.iter().filter(|x| fe.contains(x));
            let u = *it.next().ok_or_else(bad)?;
            if it.next().is_some() {
                return Err(bad());
            }
            Ok(u)
        };
        let u = [in_face(f1)?, in_face(f2)?];
        if u[0] == u[1] || edges.len() != 2 {
            return Err(bad());
        }
        let others: Vec<FaceId> = self.faces_of_vertex(v).into_iter().filter(|f| *f != f1 && *f != f2).collect();
        let g = match others[..] {
            [] => None,
            [g] => {
                let ge = self.face_edges(g);
                if !ge.contains(&u[0]) || !ge.contains(&u[1]) {
                    return Err(bad());
                }
                Some(g)
            }
            _ => return Err(bad()),
        };
        let x = [self.edges[u[0]].other(v), self.edges[u[1]].other(v)];
        let p = self.pos(v);
        let n = (self.pos(x[0]) - p).cross(self.pos(x[1]) - p);
        if n.length() < 1e-9 {
            return Err(OpError::Invalid(tr!("kernel-err-the-edges-at-a-corner-are")));
        }
        let n = match g {
            Some(g) => self.faces[g].normal,
            None => n.normalize(),
        };
        Ok(End { v, u, x, g, n })
    }
}

impl crate::Model {
    /// Chamfer / fillet `edges` of the context `path` one after another (edges shortened by an
    /// earlier one are followed to their replacements). Returns the new faces; fails only if no
    /// edge could be done (then with the first error, the mesh unchanged).
    pub fn chamfer_edges_in(
        &mut self,
        path: &[crate::InstanceId],
        edges: &[EdgeId],
        kind: ChamferKind,
    ) -> OpResult<Vec<FaceId>> {
        let mesh = self.mesh_at_mut(path).ok_or(OpError::NotFound)?;
        let mut map: std::collections::HashMap<EdgeId, EdgeId> = Default::default();
        let mut faces = Vec::new();
        let mut first_err = None;
        for &e0 in edges {
            let mut e = e0;
            while let Some(n) = map.get(&e) {
                e = *n;
            }
            match mesh.chamfer_edge(e, kind) {
                Ok(r) => {
                    faces.extend(r.faces);
                    map.extend(r.replaced);
                }
                Err(err) => {
                    first_err.get_or_insert(err);
                }
            }
        }
        match first_err {
            Some(err) if faces.is_empty() => Err(err),
            _ => Ok(faces),
        }
    }
}
