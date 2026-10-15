//! Flatten the model into world-space triangles and edges (rendering, picking, export).

use crate::geom::Projector;
use crate::mesh::Mesh;
use crate::model::{ContextPath, Model};
use crate::triangulate::triangulate;
use crate::{DefId, EdgeId, EntityRef, FaceId, MaterialId, VertexId};
use glam::{DMat3, DMat4, DVec2, DVec3};

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
    /// World-space triangle list positions (len % 3 == 0), CCW about `normal`.
    pub positions: Vec<DVec3>,
    pub normal: DVec3,
    pub front: Option<MaterialId>,
    pub back: Option<MaterialId>,
    /// Inherited material from enclosing instances (applies to faces with no material).
    pub inherited: Option<MaterialId>,
    /// Planar texture coordinates in mm, one per position (from the face's local coordinates,
    /// so textures move with their group). Divide by the material's tile size for UVs.
    pub uv: Vec<DVec2>,
    /// Smoothed world normals, one per position, when the face borders smooth edges (empty =
    /// flat, use `normal`).
    pub normals: Vec<DVec3>,
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

/// Triangulate one face in local coordinates: CCW triangles about the face normal.
pub fn triangulate_face(mesh: &Mesh, f: FaceId) -> Vec<[DVec3; 3]> {
    let Some(face) = mesh.faces.get(f) else { return Vec::new() };
    if face.outer.len() < 3 {
        return Vec::new();
    }
    let outer3: Vec<DVec3> = face.outer.iter().map(|v| mesh.vertices[*v].pos).collect();
    if face.holes.is_empty() && outer3.len() == 3 {
        return vec![[outer3[0], outer3[1], outer3[2]]];
    }
    let pr = Projector::new(outer3[0], face.normal);
    let o2: Vec<DVec2> = outer3.iter().map(|p| pr.to2(*p)).collect();
    let holes3: Vec<Vec<DVec3>> = face.holes.iter().map(|h| h.iter().map(|v| mesh.vertices[*v].pos).collect()).collect();
    let h2: Vec<Vec<DVec2>> = holes3.iter().map(|h| h.iter().map(|p| pr.to2(*p)).collect()).collect();
    let hr: Vec<&[DVec2]> = h2.iter().map(|h| h.as_slice()).collect();
    let all: Vec<DVec3> = outer3.iter().chain(holes3.iter().flatten()).copied().collect();
    triangulate(&o2, &hr).into_iter().map(|t| [all[t[0]], all[t[1]], all[t[2]]]).collect()
}

/// Smoothed corner normals (local) of face `f`, keyed by vertex position: at each vertex, the
/// average normal of the faces reached around it by crossing smooth edges (the classic
/// "Smooth normals"). `None` when no edge of the face is smooth (flat shading).
pub fn face_smooth_normals(mesh: &Mesh, f: FaceId) -> Option<Vec<(DVec3, DVec3)>> {
    let face = mesh.faces.get(f)?;
    if !mesh.face_edges(f).iter().any(|e| mesh.edges.get(*e).is_some_and(|ed| ed.smooth)) {
        return None;
    }
    // Neighbours of `v` in the loops of face `g` (the two face edges at `v`).
    let around = |g: FaceId, v: VertexId| -> Vec<VertexId> {
        let Some(gf) = mesh.faces.get(g) else { return Vec::new() };
        let mut out = Vec::new();
        for lp in std::iter::once(&gf.outer).chain(gf.holes.iter()) {
            if let Some(i) = lp.iter().position(|x| *x == v) {
                out.push(lp[(i + 1) % lp.len()]);
                out.push(lp[(i + lp.len() - 1) % lp.len()]);
            }
        }
        out
    };
    let mut out = Vec::new();
    for &v in face.outer.iter().chain(face.holes.iter().flatten()) {
        let mut seen = vec![f];
        let mut queue = vec![f];
        let mut sum = DVec3::ZERO;
        while let Some(g) = queue.pop() {
            let Some(gf) = mesh.faces.get(g) else { continue };
            // Faces wound the other way still smooth together.
            sum += if gf.normal.dot(face.normal) < 0.0 { -gf.normal } else { gf.normal };
            for w in around(g, v) {
                let Some(e) = mesh.edge_between(v, w) else { continue };
                if !mesh.edges[e].smooth {
                    continue;
                }
                for h in mesh.edge_faces(e) {
                    if !seen.contains(&h) && seen.len() < 64 {
                        seen.push(h);
                        queue.push(h);
                    }
                }
            }
        }
        out.push((mesh.vertices[v].pos, sum.normalize_or(face.normal)));
    }
    Some(out)
}

/// World triangles of face `f` under transform `m` (winding fixed for mirroring transforms),
/// with `with_uv` their planar texture coordinates in mm (see [`TessFace::uv`]), and smoothed
/// world normals per position when the face borders smooth edges (else empty).
pub fn face_world_triangles(mesh: &Mesh, f: FaceId, m: &DMat4, with_uv: bool) -> (Vec<DVec3>, Vec<DVec2>, Vec<DVec3>) {
    let Some(face) = mesh.faces.get(f) else { return Default::default() };
    let mirror = m.determinant() < 0.0;
    let tris = triangulate_face(mesh, f);
    let mut positions = Vec::with_capacity(tris.len() * 3);
    let mut uv = Vec::with_capacity(if with_uv { tris.len() * 3 } else { 0 });
    let tex = |p: DVec3| crate::texture::face_uv_mm(p, face.normal, face.uv.as_ref());
    let smooth = face_smooth_normals(mesh, f);
    let nm = DMat3::from_mat4(*m).inverse().transpose();
    let mut normals = Vec::with_capacity(if smooth.is_some() { tris.len() * 3 } else { 0 });
    for t in tris {
        let (a, b, c) = (m.transform_point3(t[0]), m.transform_point3(t[1]), m.transform_point3(t[2]));
        let order = if mirror { [0, 2, 1] } else { [0, 1, 2] };
        positions.extend(order.map(|k| [a, b, c][k]));
        if with_uv {
            uv.extend(order.map(|k| tex(t[k])));
        }
        if let Some(sm) = &smooth {
            normals.extend(order.map(|k| {
                let n = sm.iter().find(|(p, _)| *p == t[k]).map_or(face.normal, |(_, n)| *n);
                (nm * n).normalize_or_zero()
            }));
        }
    }
    (positions, uv, normals)
}

/// Flatten the whole model (respecting hidden flags and tag visibility) to world-space data.
pub fn tessellate(model: &Model) -> Tessellation {
    let mut out = Tessellation::default();
    rec(model, model.root, DMat4::IDENTITY, &mut Vec::new(), None, &mut out, 0);
    out
}

fn rec(
    model: &Model,
    d: DefId,
    m: DMat4,
    path: &mut ContextPath,
    inherited: Option<MaterialId>,
    out: &mut Tessellation,
    depth: usize,
) {
    if depth > 64 {
        return;
    }
    let Some(def) = model.defs.get(d) else { return };
    let mesh = &def.mesh;
    let nm = DMat3::from_mat4(m).inverse().transpose();
    for (fid, face) in &mesh.faces {
        if face.hidden || !model.tag_visible(face.tag) {
            continue;
        }
        let (positions, uv, normals) = face_world_triangles(mesh, fid, &m, true);
        out.faces.push(TessFace {
            owner_path: path.clone(),
            face: Some(fid),
            positions,
            normal: (nm * face.normal).normalize_or_zero(),
            front: face.front,
            back: face.back,
            inherited,
            uv,
            normals,
        });
    }
    for (eid, ed) in &mesh.edges {
        if ed.hidden || !model.tag_visible(ed.tag) {
            continue;
        }
        let (a, b) = mesh.edge_segment(eid);
        out.edges.push(TessEdge {
            owner_path: path.clone(),
            edge: eid,
            a: m.transform_point3(a),
            b: m.transform_point3(b),
            soft: ed.soft,
        });
    }
    for (iid, inst) in &mesh.instances {
        if inst.hidden || !model.tag_visible(inst.tag) {
            continue;
        }
        path.push(iid);
        rec(model, inst.def, m * inst.transform, path, inst.material.or(inherited), out, depth + 1);
        path.pop();
    }
}
