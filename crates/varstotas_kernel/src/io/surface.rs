//! Shared surface construction for mesh importers. Never run the interactive CAD face
//! finder on source polygons: it splits nearby layers and overwrites overlapping materials.
use super::{ImportOptions, IoError};
use crate::glam::DVec3;
use crate::{Edge, Face, FaceId, Mesh, Vertex, VertexId};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct SurfaceBuilder {
    vertices: HashMap<[u64; 3], VertexId>,
    edges: HashSet<[VertexId; 2]>,
    faces: Vec<FaceId>,
}
fn point_key(p: DVec3) -> [u64; 3] {
    p.to_array().map(|x| if x == 0.0 { 0 } else { x.to_bits() })
}
fn edge_key(a: VertexId, b: VertexId) -> [VertexId; 2] {
    if a < b { [a, b] } else { [b, a] }
}
impl SurfaceBuilder {
    pub fn from_mesh(mesh: &Mesh) -> Self {
        Self {
            vertices: mesh
                .vertices
                .iter()
                .map(|(v, p)| (point_key(p.pos), v))
                .collect(),
            edges: mesh
                .edges
                .values()
                .map(|e| edge_key(e.v[0], e.v[1]))
                .collect(),
            faces: Vec::new(),
        }
    }
    fn face(&mut self, mesh: &mut Mesh, points: &[DVec3], normal: DVec3) -> FaceId {
        let vertices: Vec<_> = points
            .iter()
            .map(|&pos| {
                *self
                    .vertices
                    .entry(point_key(pos))
                    .or_insert_with(|| mesh.vertices.insert(Vertex { pos }))
            })
            .collect();
        for i in 0..vertices.len() {
            let (a, b) = (vertices[i], vertices[(i + 1) % vertices.len()]);
            if self.edges.insert(edge_key(a, b)) {
                mesh.edges.insert(Edge::new(a, b));
            }
        }
        let mut face = Face::template();
        face.outer = vertices;
        face.normal = normal;
        mesh.invalidate_caches();
        let id = mesh.faces.insert(face);
        self.faces.push(id);
        id
    }
    pub fn triangle(
        &mut self,
        mesh: &mut Mesh,
        points: [DVec3; 3],
    ) -> Result<Option<FaceId>, IoError> {
        if points.iter().any(|p| !p.is_finite()) {
            return Err(IoError::Format("Invalid imported positions".into()));
        }
        let cross = (points[1] - points[0]).cross(points[2] - points[0]);
        let length = cross.length();
        if !length.is_finite() {
            return Err(IoError::Format("Invalid imported triangle".into()));
        }
        if length == 0.0 {
            return Ok(None);
        }
        Ok(Some(self.face(mesh, &points, cross / length)))
    }
    /// Complete adjacency once, preserve authored corner shading and optionally remove only
    /// coplanar edges whose surface attributes agree. Never flatten authored curved shading.
    pub fn finish(&self, mesh: &mut Mesh, opts: &ImportOptions) {
        mesh.rebuild_caches();
        let smooth: Vec<_> = mesh.edges.iter().filter_map(|(edge, data)| {
            let adjacent=mesh.edge_faces(edge);
            if adjacent.len()!=2 {return None;}
            data.v.iter().all(|v| {
                let at=|f:FaceId| {
                    let face=&mesh.faces[f];
                    face.loops().flatten().position(|w|w==v).and_then(|i|face.normals.get(i)).copied()
                };
                matches!((at(adjacent[0]),at(adjacent[1])),(Some(a),Some(b)) if a.dot(b)>1.0-1e-4)
            }).then_some(edge)
        }).collect();
        for edge in smooth {
            mesh.edges[edge].smooth = true;
            mesh.edges[edge].soft = true;
        }
        let curved = self
            .faces
            .iter()
            .filter_map(|f| mesh.faces.get(*f))
            .any(|f| f.normals.iter().any(|n| n.dot(f.normal) < 1.0 - 1e-6));
        if opts.merge_coplanar && !curved {
            mesh.merge_coplanar_faces(&self.faces);
        }
    }
    pub fn polygon(&mut self, mesh: &mut Mesh, points: &[DVec3]) -> Result<Vec<FaceId>, IoError> {
        if points.iter().any(|p| !p.is_finite()) {
            return Err(IoError::Format("Invalid imported positions".into()));
        }
        let mut points = points.to_vec();
        points.dedup();
        if points.len() > 1 && points.first() == points.last() {
            points.pop();
        }
        if points.len() < 3 {
            return Ok(Vec::new());
        }
        if points.len() == 3 {
            return Ok(self
                .triangle(mesh, [points[0], points[1], points[2]])?
                .into_iter()
                .collect());
        }
        // Translate first to avoid cancellation on small polygons far from the origin.
        let relative: Vec<_> = points.iter().map(|p| *p - points[0]).collect();
        let n = crate::geom::newell(&relative).normalize_or_zero();
        if n != DVec3::ZERO && relative.iter().all(|p| p.dot(n).abs() <= crate::TOL * 10.0) {
            return Ok(vec![self.face(mesh, &points, n)]);
        }
        let mut out = Vec::new();
        for i in 1..points.len() - 1 {
            out.extend(self.triangle(mesh, [points[0], points[i], points[i + 1]])?);
        }
        Ok(out)
    }
}

/// Assign source normals to a polygon or its triangulated fragments without averaging seams.
pub(super) fn corner_normals(
    mesh: &mut Mesh,
    face: FaceId,
    points: &[DVec3],
    normals: Option<&[DVec3]>,
) {
    let Some(normals) = normals.filter(|n| {
        n.len() == points.len()
            && n.iter()
                .all(|n| n.is_finite() && n.length_squared() > 1e-12)
    }) else {
        return;
    };
    let values: Option<Vec<_>> = mesh.faces[face]
        .loops()
        .flatten()
        .map(|v| {
            points
                .iter()
                .position(|p| *p == mesh.vertices[*v].pos)
                .map(|i| normals[i].normalize())
        })
        .collect();
    if let Some(values) = values {
        mesh.faces[face].normals = values;
    }
}

/// A mesh of triangles (`tris` index `positions`), welded where positions are equal, with a
/// normal per position for smooth shading: edges whose corner normals agree become soft and
/// smooth (a draped cloth, a scanned surface). Degenerate triangles are skipped. `style` sets
/// the attributes of the face made from triangle `i` (materials, texture placement).
pub fn mesh_from_triangles(positions: &[DVec3], tris: &[[u32; 3]], normals: &[DVec3], mut style: impl FnMut(usize, &mut Face)) -> Mesh {
    let mut mesh = Mesh::new();
    let mut surface = SurfaceBuilder::default();
    for (i, t) in tris.iter().enumerate() {
        let pts = t.map(|i| positions[i as usize]);
        if let Ok(Some(f)) = surface.triangle(&mut mesh, pts) {
            let ns: Option<Vec<DVec3>> = t.iter().map(|i| normals.get(*i as usize).copied()).collect();
            corner_normals(&mut mesh, f, &pts, ns.as_deref());
            style(i, &mut mesh.faces[f]);
        }
    }
    surface.finish(&mut mesh, &ImportOptions { merge_coplanar: false, ..ImportOptions::default() });
    mesh
}
