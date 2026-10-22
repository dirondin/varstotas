//! Plain-text exchange: points from CSV (a polyline, a face when closed and planar) and every
//! face as JSON.

use super::IoError;
use crate::tessellate::triangulate_face;
use crate::{DefId, Model};
use glam::{DMat3, DMat4, DVec3};
use serde_json::{Value, json};
use std::path::Path;
use varstotas_i18n::tr;

/// Points of CSV text: `x,y[,z]` per line in mm (`;` or tabs also separate; lines that
/// aren't numbers — headers, comments — are skipped).
pub fn parse_points_csv(text: &str) -> Vec<DVec3> {
    text.lines()
        .filter_map(|l| {
            let c: Vec<f64> = l.split([',', ';', '\t']).map(str::trim).map(|s| s.parse::<f64>()).collect::<Result<_, _>>().ok()?;
            (c.len() >= 2).then(|| DVec3::new(c[0], c[1], c.get(2).copied().unwrap_or(0.0)))
        })
        .collect()
}

/// Import the points of a CSV file into the root: lines through them; when the last point
/// repeats the first, a closed outline (a face when planar).
pub fn import_points_csv(model: &mut Model, path: &Path) -> Result<usize, IoError> {
    let mut pts = parse_points_csv(&String::from_utf8(super::read_limited(path, super::MAX_MODEL_BYTES)?)
            .map_err(|e| IoError::Format(e.to_string()))?,
    );
    if pts.len() < 2 {
        return Err(IoError::Format(tr!("kernel-io-no-points-in-display", display = path.display().to_string())));
    }
    let closed = pts.len() > 3 && pts[0].distance(pts[pts.len() - 1]) < 1e-9;
    if closed {
        pts.pop();
    }
    let n = pts.len();
    let mesh = model.root_mesh_mut();
    if !(closed && mesh.add_face(&pts).is_some()) {
        mesh.add_polyline(&pts, closed, false);
    }
    Ok(n)
}

/// Every visible face of the model in world coordinates (mm): its outer loop, normal, area
/// and material name.
pub fn faces_json(model: &Model) -> Value {
    fn walk(model: &Model, d: DefId, xf: DMat4, out: &mut Vec<Value>, depth: usize) {
        let Some(def) = model.defs.get(d) else { return };
        let nx = DMat3::from_mat4(xf).inverse().transpose();
        for (f, face) in def.mesh.faces.iter() {
            let verts: Vec<Value> = face.outer.iter().map(|v| {
                let p = xf.transform_point3(def.mesh.vertices[*v].pos);
                json!([p.x, p.y, p.z])
            }).collect();
            let area: f64 = triangulate_face(&def.mesh, f).iter().map(|t| {
                let p = t.map(|p| xf.transform_point3(p));
                (p[1] - p[0]).cross(p[2] - p[0]).length() * 0.5
            }).sum();
            let n = (nx * face.normal).normalize_or_zero();
            let mat = face.front.and_then(|m| model.materials.get(m)).map(|m| Value::String(m.name.clone())).unwrap_or(Value::Null);
            out.push(json!({ "vertices": verts, "normal": [n.x, n.y, n.z], "area": area, "material": mat }));
        }
        if depth < 64 {
            for i in def.mesh.instances.values().filter(|i| !i.hidden) {
                walk(model, i.def, xf * i.transform, out, depth + 1);
            }
        }
    }
    let mut faces = Vec::new();
    walk(model, model.root, DMat4::IDENTITY, &mut faces, 0);
    json!({ "units": "mm", "faces": faces })
}

pub fn export_faces_json(model: &Model, path: &Path) -> Result<(), IoError> {
    let s = serde_json::to_string_pretty(&faces_json(model)).map_err(|e| IoError::Format(e.to_string()))?;
    std::fs::write(path, s)?;
    Ok(())
}
