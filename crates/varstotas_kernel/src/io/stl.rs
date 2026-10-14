//! STL export (binary) and import (binary or ASCII).

use super::{ImportOptions, IoError};
use crate::mesh::Mesh;
use crate::{FaceId, Model, tessellate};
use glam::DVec3;
use std::collections::HashSet;
use std::path::Path;

/// Binary STL bytes of the visible model (world space, millimetres).
pub fn write_stl_binary(model: &Model) -> Vec<u8> {
    let tess = tessellate(model);
    let tris: Vec<([DVec3; 3], DVec3)> = tess
        .faces
        .iter()
        .flat_map(|f| f.positions.as_chunks::<3>().0.iter().map(move |t| ([t[0], t[1], t[2]], f.normal)))
        .collect();
    let mut out = Vec::with_capacity(84 + tris.len() * 50);
    let mut header = [0u8; 80];
    let h = b"Varstotas binary STL (mm)";
    header[..h.len()].copy_from_slice(h);
    out.extend_from_slice(&header);
    out.extend_from_slice(&(tris.len() as u32).to_le_bytes());
    for (t, n) in tris {
        for v in std::iter::once(n).chain(t) {
            for c in [v.x, v.y, v.z] {
                out.extend_from_slice(&(c as f32).to_le_bytes());
            }
        }
        out.extend_from_slice(&0u16.to_le_bytes());
    }
    out
}

pub fn export_stl(model: &Model, path: &Path) -> Result<(), IoError> {
    std::fs::write(path, write_stl_binary(model))?;
    Ok(())
}

/// Parse STL bytes (binary or ASCII) into triangles.
pub(crate) fn parse_stl(data: &[u8]) -> Result<Vec<[DVec3; 3]>, IoError> {
    if data.len() >= 84 {
        let n = u32::from_le_bytes([data[80], data[81], data[82], data[83]]) as usize;
        if data.len() == 84 + n * 50 {
            let mut tris = Vec::with_capacity(n);
            for i in 0..n {
                let base = 84 + i * 50 + 12;
                let f = |k: usize| {
                    let o = base + k * 4;
                    f32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]) as f64
                };
                tris.push([
                    DVec3::new(f(0), f(1), f(2)),
                    DVec3::new(f(3), f(4), f(5)),
                    DVec3::new(f(6), f(7), f(8)),
                ]);
            }
            return Ok(tris);
        }
    }
    let text = std::str::from_utf8(data).map_err(|_| IoError::Format("not a valid STL".into()))?;
    if !text.trim_start().starts_with("solid") {
        return Err(IoError::Format("not a valid STL".into()));
    }
    let mut verts = Vec::new();
    for line in text.lines() {
        let mut it = line.split_whitespace();
        if it.next() == Some("vertex") {
            let c: Vec<f64> = it
                .take(3)
                .map(|s| s.parse::<f64>())
                .collect::<Result<_, _>>()
                .map_err(|_| IoError::Format(format!("bad STL vertex: {line}")))?;
            if c.len() != 3 {
                return Err(IoError::Format(format!("bad STL vertex: {line}")));
            }
            verts.push(DVec3::new(c[0], c[1], c[2]));
        }
    }
    if verts.len() % 3 != 0 {
        return Err(IoError::Format("STL vertex count not a multiple of 3".into()));
    }
    Ok(verts.as_chunks::<3>().0.iter().map(|c| [c[0], c[1], c[2]]).collect())
}

/// Add one polygon with exact orientation; non-planar polygons are fanned into triangles.
/// Returns the faces made.
pub(crate) fn add_polygon(mesh: &mut Mesh, pts: &[DVec3]) -> Vec<FaceId> {
    if pts.len() < 3 {
        return Vec::new();
    }
    if let Some(f) = mesh.add_face_exact(pts) {
        return vec![f];
    }
    let mut out = Vec::new();
    if pts.len() > 3 {
        for i in 1..pts.len() - 1 {
            out.extend(mesh.add_face_exact(&[pts[0], pts[i], pts[i + 1]]));
        }
    }
    out
}

/// Faces of `mesh` not in `before` (made by an import, incl. pieces of split faces).
pub(crate) fn new_faces(mesh: &Mesh, before: &HashSet<FaceId>) -> Vec<FaceId> {
    mesh.faces.keys().filter(|f| !before.contains(f)).collect()
}

/// Import an STL file into the root mesh (each triangle via `add_face_exact`), merging
/// coplanar triangles.
pub fn import_stl(model: &mut Model, path: &Path) -> Result<(), IoError> {
    import_stl_with(model, path, &ImportOptions::default())
}

pub fn import_stl_with(model: &mut Model, path: &Path, opts: &ImportOptions) -> Result<(), IoError> {
    let data = std::fs::read(path)?;
    let tris = parse_stl(&data)?;
    let mesh = model.root_mesh_mut();
    let before: HashSet<FaceId> = mesh.faces.keys().collect();
    for t in tris {
        add_polygon(mesh, &t);
    }
    if opts.merge_coplanar {
        let made = new_faces(mesh, &before);
        mesh.merge_coplanar_faces(&made);
    }
    Ok(())
}
