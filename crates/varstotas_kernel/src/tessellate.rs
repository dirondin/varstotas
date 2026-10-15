//! Flatten the model into world-space triangles and edges (rendering, picking, export).

use crate::geom::Projector;
use crate::mesh::Mesh;
use crate::model::{ContextPath, Model};
use crate::triangulate::triangulate;
use crate::{DefId, EdgeId, EntityRef, FaceId, MaterialId};
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

/// World triangles of face `f` under transform `m` (winding fixed for mirroring transforms)
/// and, with `with_uv`, their planar texture coordinates in mm (see [`TessFace::uv`]).
pub fn face_world_triangles(mesh: &Mesh, f: FaceId, m: &DMat4, with_uv: bool) -> (Vec<DVec3>, Vec<DVec2>) {
    let Some(face) = mesh.faces.get(f) else { return Default::default() };
    let mirror = m.determinant() < 0.0;
    let tris = triangulate_face(mesh, f);
    let mut positions = Vec::with_capacity(tris.len() * 3);
    let mut uv = Vec::with_capacity(if with_uv { tris.len() * 3 } else { 0 });
    let tex = |p: DVec3| crate::texture::face_uv_mm(p, face.normal, face.uv.as_ref());
    for t in tris {
        let (a, b, c) = (m.transform_point3(t[0]), m.transform_point3(t[1]), m.transform_point3(t[2]));
        let order = if mirror { [0, 2, 1] } else { [0, 1, 2] };
        positions.extend(order.map(|k| [a, b, c][k]));
        if with_uv {
            uv.extend(order.map(|k| tex(t[k])));
        }
    }
    (positions, uv)
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
        let (positions, uv) = face_world_triangles(mesh, fid, &m, true);
        out.faces.push(TessFace {
            owner_path: path.clone(),
            face: Some(fid),
            positions,
            normal: (nm * face.normal).normalize_or_zero(),
            front: face.front,
            back: face.back,
            inherited,
            uv,
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
