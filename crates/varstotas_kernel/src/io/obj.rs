//! Wavefront OBJ (+ MTL) export and import.

use super::IoError;
use crate::{MaterialId, Model, tessellate};
use glam::DVec3;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::Path;

fn key(p: DVec3) -> [u64; 3] {
    [p.x.to_bits(), p.y.to_bits(), p.z.to_bits()]
}

fn sanitize(s: &str) -> String {
    let s: String = s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    if s.is_empty() { "material".into() } else { s }
}

/// Produce OBJ text and MTL text. `mtl_file` is referenced by `mtllib` in the OBJ.
/// Faces are grouped per effective front material (face material, else inherited, else
/// `default`). Coordinates are millimetres.
pub fn write_obj(model: &Model, mtl_file: &str) -> (String, String) {
    let tess = tessellate(model);
    let mut names: HashMap<Option<MaterialId>, String> = HashMap::new();
    let mut groups: Vec<(Option<MaterialId>, Vec<usize>)> = Vec::new();
    for (i, f) in tess.faces.iter().enumerate() {
        let m = f.front.or(f.inherited).filter(|m| model.materials.contains_key(*m));
        match groups.iter_mut().find(|g| g.0 == m) {
            Some(g) => g.1.push(i),
            None => groups.push((m, vec![i])),
        }
    }
    let mut mtl = String::from("# Varstotas MTL\n");
    for (idx, (m, _)) in groups.iter().enumerate() {
        let name = match m {
            None => "default".to_string(),
            Some(id) => format!("m{}_{}", idx, sanitize(&model.materials[*id].name)),
        };
        let c = m.map(|id| model.materials[id].color).unwrap_or([1.0, 1.0, 1.0, 1.0]);
        let _ = writeln!(mtl, "\nnewmtl {name}\nKa 0 0 0\nKd {} {} {}\nKs 0 0 0\nd {}\nillum 1", c[0], c[1], c[2], c[3]);
        names.insert(*m, name);
    }
    let mut obj = format!("# Varstotas OBJ export (units: mm)\nmtllib {mtl_file}\n");
    let mut vidx: HashMap<[u64; 3], usize> = HashMap::new();
    let mut nidx: HashMap<[u64; 3], usize> = HashMap::new();
    let mut vlines = String::new();
    let mut nlines = String::new();
    let mut flines = String::new();
    for (m, faces) in &groups {
        let _ = writeln!(flines, "g {}\nusemtl {}", names[m], names[m]);
        for &fi in faces {
            let f = &tess.faces[fi];
            let nk = key(f.normal);
            let nlen = nidx.len();
            let ni = *nidx.entry(nk).or_insert_with(|| {
                let _ = writeln!(nlines, "vn {} {} {}", f.normal.x, f.normal.y, f.normal.z);
                nlen + 1
            });
            for t in f.positions.as_chunks::<3>().0 {
                let mut ids = [0usize; 3];
                for (k, p) in t.iter().enumerate() {
                    let len = vidx.len();
                    ids[k] = *vidx.entry(key(*p)).or_insert_with(|| {
                        let _ = writeln!(vlines, "v {} {} {}", p.x, p.y, p.z);
                        len + 1
                    });
                }
                let _ = writeln!(flines, "f {}//{ni} {}//{ni} {}//{ni}", ids[0], ids[1], ids[2]);
            }
        }
    }
    obj.push_str(&vlines);
    obj.push_str(&nlines);
    obj.push_str(&flines);
    (obj, mtl)
}

pub fn export_obj(model: &Model, path: &Path) -> Result<(), IoError> {
    let mtl_path = path.with_extension("mtl");
    let mtl_name = mtl_path.file_name().and_then(|s| s.to_str()).unwrap_or("model.mtl").to_string();
    let (obj, mtl) = write_obj(model, &mtl_name);
    std::fs::write(path, obj)?;
    std::fs::write(mtl_path, mtl)?;
    Ok(())
}

/// Parse OBJ text into polygons and polylines.
/// Polygons (`f`) and polylines (`l`) of an OBJ file.
pub(crate) type ObjShapes = (Vec<Vec<DVec3>>, Vec<Vec<DVec3>>);

pub(crate) fn parse_obj(text: &str) -> Result<ObjShapes, IoError> {
    let mut verts: Vec<DVec3> = Vec::new();
    let mut polys = Vec::new();
    let mut lines = Vec::new();
    for (ln, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        let mut it = line.split_whitespace();
        let Some(tag) = it.next() else { continue };
        let bad = || IoError::Format(format!("OBJ line {}: {line}", ln + 1));
        match tag {
            "v" => {
                let c: Vec<f64> = it.take(3).map(|s| s.parse::<f64>()).collect::<Result<_, _>>().map_err(|_| bad())?;
                if c.len() != 3 {
                    return Err(bad());
                }
                verts.push(DVec3::new(c[0], c[1], c[2]));
            }
            "f" | "l" => {
                let mut pts = Vec::new();
                for tok in it {
                    let i: i64 = tok.split('/').next().unwrap_or("").parse().map_err(|_| bad())?;
                    let idx = if i < 0 { verts.len() as i64 + i } else { i - 1 };
                    let p = *verts.get(usize::try_from(idx).map_err(|_| bad())?).ok_or_else(bad)?;
                    pts.push(p);
                }
                if tag == "f" { polys.push(pts) } else { lines.push(pts) }
            }
            _ => {}
        }
    }
    Ok((polys, lines))
}

/// Import an OBJ file into the root mesh (sticky: each polygon via `add_face_exact`;
/// non-planar polygons are fanned into triangles; `l` polylines become edges).
pub fn import_obj(model: &mut Model, path: &Path) -> Result<(), IoError> {
    let text = std::fs::read_to_string(path)?;
    let (polys, lines) = parse_obj(&text)?;
    let mesh = model.root_mesh_mut();
    for p in polys {
        super::stl::add_polygon(mesh, &p);
    }
    for l in lines {
        mesh.add_polyline(&l, false, false);
    }
    Ok(())
}
