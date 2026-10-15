//! Wavefront OBJ (+ MTL) export and import.

use super::{ImportOptions, IoError};
use crate::{FaceId, MaterialId, Model, tessellate};
use glam::DVec3;
use std::collections::{HashMap, HashSet};
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
    let (obj, mtl, _) = write_obj_with_textures(model, mtl_file);
    (obj, mtl)
}

/// A texture file referenced by an exported MTL: (file name, bytes).
pub type TextureFile = (String, std::sync::Arc<[u8]>);

/// [`write_obj`] plus the texture files the MTL references (`map_Kd <name>`): (file name, bytes).
/// Textured faces get `vt` coordinates (planar mm divided by the tile size).
pub fn write_obj_with_textures(model: &Model, mtl_file: &str) -> (String, String, Vec<TextureFile>) {
    let mut textures = Vec::new();
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
    let mut used: HashSet<String> = HashSet::new();
    for (m, _) in &groups {
        // Plain (sanitized) material names, unique; "default" means "no material".
        let name = match m {
            None => "default".to_string(),
            Some(id) => {
                let base = sanitize(&model.materials[*id].name);
                let mut name = base.clone();
                let mut k = 2;
                while name == "default" || used.contains(&name) {
                    name = format!("{base}_{k}");
                    k += 1;
                }
                name
            }
        };
        used.insert(name.clone());
        let c = m.map(|id| model.materials[id].color).unwrap_or([1.0, 1.0, 1.0, 1.0]);
        let _ = writeln!(mtl, "\nnewmtl {name}\nKa 0 0 0\nKd {} {} {}\nKs 0 0 0\nd {}\nillum 1", c[0], c[1], c[2], c[3]);
        if let Some(img) = m.and_then(|id| model.materials[id].image.as_ref()) {
            let file = format!("{name}.{}", img.extension());
            let _ = writeln!(mtl, "map_Kd {file}");
            textures.push((file, img.bytes.clone()));
        } else if let Some(tex) = m.and_then(|id| model.materials[id].texture.as_ref()) {
            let _ = writeln!(mtl, "map_Kd {tex}");
        }
        names.insert(*m, name);
    }
    let mut obj = format!("# Varstotas OBJ export (units: mm, Z axis up)\nmtllib {mtl_file}\n");
    let mut vidx: HashMap<[u64; 3], usize> = HashMap::new();
    let mut nidx: HashMap<[u64; 3], usize> = HashMap::new();
    let mut vlines = String::new();
    let mut nlines = String::new();
    let mut tlines = String::new();
    let mut tcount = 0usize;
    let mut flines = String::new();
    for (m, faces) in &groups {
        let tile = m.and_then(|id| model.materials[id].image.as_ref()).map(|img| img.size);
        let _ = writeln!(flines, "g {}\nusemtl {}", names[m], names[m]);
        for &fi in faces {
            let f = &tess.faces[fi];
            let nk = key(f.normal);
            let nlen = nidx.len();
            let ni = *nidx.entry(nk).or_insert_with(|| {
                let _ = writeln!(nlines, "vn {} {} {}", num(f.normal.x), num(f.normal.y), num(f.normal.z));
                nlen + 1
            });
            for (ti, t) in f.positions.as_chunks::<3>().0.iter().enumerate() {
                let mut ids = [0usize; 3];
                for (k, p) in t.iter().enumerate() {
                    let len = vidx.len();
                    ids[k] = *vidx.entry(key(*p)).or_insert_with(|| {
                        let _ = writeln!(vlines, "v {} {} {}", num(p.x), num(p.y), num(p.z));
                        len + 1
                    });
                }
                match tile {
                    Some([w, h]) if f.uv.len() == f.positions.len() && w > 0.0 && h > 0.0 => {
                        for k in 0..3 {
                            let uv = f.uv[ti * 3 + k];
                            let _ = writeln!(tlines, "vt {} {}", num(uv.x / w), num(uv.y / h));
                        }
                        let t0 = tcount;
                        tcount += 3;
                        let _ = writeln!(
                            flines,
                            "f {}/{}/{ni} {}/{}/{ni} {}/{}/{ni}",
                            ids[0],
                            t0 + 1,
                            ids[1],
                            t0 + 2,
                            ids[2],
                            t0 + 3
                        );
                    }
                    _ => {
                        let _ = writeln!(flines, "f {}//{ni} {}//{ni} {}//{ni}", ids[0], ids[1], ids[2]);
                    }
                }
            }
        }
    }
    obj.push_str(&vlines);
    obj.push_str(&tlines);
    obj.push_str(&nlines);
    obj.push_str(&flines);
    (obj, mtl, textures)
}

pub fn export_obj(model: &Model, path: &Path) -> Result<(), IoError> {
    let mtl_path = path.with_extension("mtl");
    let mtl_name = mtl_path.file_name().and_then(|s| s.to_str()).unwrap_or("model.mtl").to_string();
    let (obj, mtl, textures) = write_obj_with_textures(model, &mtl_name);
    std::fs::write(path, obj)?;
    std::fs::write(&mtl_path, mtl)?;
    let dir = path.parent().unwrap_or(Path::new("."));
    for (name, bytes) in textures {
        std::fs::write(dir.join(name), &bytes[..])?;
    }
    Ok(())
}

/// Parsed OBJ: polygons (with the index of their `usemtl` name), polylines, `mtllib`
/// files and the material names used.
#[derive(Default)]
pub(crate) struct ObjData {
    pub polys: Vec<(Vec<DVec3>, Option<usize>)>,
    pub lines: Vec<Vec<DVec3>>,
    pub mtllibs: Vec<String>,
    pub mtl_names: Vec<String>,
}

pub(crate) fn parse_obj(text: &str) -> Result<ObjData, IoError> {
    let mut verts: Vec<DVec3> = Vec::new();
    let mut d = ObjData::default();
    let mut cur: Option<usize> = None;
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
                if tag == "f" { d.polys.push((pts, cur)) } else { d.lines.push(pts) }
            }
            "usemtl" => {
                let name = line["usemtl".len()..].trim().to_string();
                cur = Some(match d.mtl_names.iter().position(|n| *n == name) {
                    Some(i) => i,
                    None => {
                        d.mtl_names.push(name);
                        d.mtl_names.len() - 1
                    }
                });
            }
            "mtllib" => d.mtllibs.push(line["mtllib".len()..].trim().to_string()),
            _ => {}
        }
    }
    Ok(d)
}

/// A material of an MTL file.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MtlMaterial {
    pub name: String,
    pub color: [f32; 4],
    pub texture: Option<String>,
}

/// Parse MTL text: `newmtl`, `Kd` (diffuse colour), `d` (opacity) or `Tr` (transparency,
/// used when there is no `d`), `map_Kd` (texture path). Unknown statements are ignored.
pub(crate) fn parse_mtl(text: &str) -> Vec<MtlMaterial> {
    let mut out: Vec<MtlMaterial> = Vec::new();
    let mut has_d = false;
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let mut it = line.split_whitespace();
        let Some(tag) = it.next() else { continue };
        let nums: Vec<f32> = line.split_whitespace().skip(1).filter_map(|s| s.parse().ok()).collect();
        if tag == "newmtl" {
            out.push(MtlMaterial { name: line["newmtl".len()..].trim().to_string(), color: [1.0, 1.0, 1.0, 1.0], texture: None });
            has_d = false;
            continue;
        }
        let Some(m) = out.last_mut() else { continue };
        match tag {
            "Kd" if nums.len() >= 3 => m.color[..3].copy_from_slice(&[nums[0], nums[1], nums[2]].map(|c| c.clamp(0.0, 1.0))),
            "d" if !nums.is_empty() => {
                m.color[3] = nums[0].clamp(0.0, 1.0);
                has_d = true;
            }
            "Tr" if !nums.is_empty() && !has_d => m.color[3] = (1.0 - nums[0]).clamp(0.0, 1.0),
            // The file name is the last token (options such as `-s 1 1 1` come first).
            "map_Kd" => m.texture = line.split_whitespace().last().filter(|t| *t != "map_Kd").map(str::to_string),
            _ => {}
        }
    }
    out
}

/// Import an OBJ file into the root mesh with the default [`ImportOptions`]: materials from
/// the `.mtl` files, coplanar triangles merged into polygons.
pub fn import_obj(model: &mut Model, path: &Path) -> Result<(), IoError> {
    import_obj_with(model, path, &ImportOptions::default())
}

/// Import an OBJ file into the root mesh (sticky: each polygon via `add_face_exact`;
/// non-planar polygons are fanned into triangles; `l` polylines become edges). `usemtl`
/// materials defined in the `mtllib` files become model materials (a material with the same
/// name and colour is reused; `default` means no material) on the faces' front side.
pub fn import_obj_with(model: &mut Model, path: &Path, opts: &ImportOptions) -> Result<(), IoError> {
    let text = std::fs::read_to_string(path)?;
    let data = parse_obj(&text)?;
    // Material ids per used name.
    let mut mat_of: Vec<Option<MaterialId>> = vec![None; data.mtl_names.len()];
    if opts.materials && !data.mtl_names.is_empty() {
        let dir = path.parent().unwrap_or(Path::new("."));
        let mut defs: Vec<MtlMaterial> = Vec::new();
        for lib in &data.mtllibs {
            // A missing or unreadable library leaves its materials out (geometry still loads).
            if let Ok(t) = std::fs::read_to_string(dir.join(lib)) {
                defs.extend(parse_mtl(&t));
            }
        }
        for (i, name) in data.mtl_names.iter().enumerate() {
            if name == "default" {
                continue;
            }
            let Some(def) = defs.iter().rev().find(|d| d.name == *name) else { continue };
            let existing = model.materials.iter().find(|(_, m)| m.name == def.name && m.color == def.color && m.texture == def.texture).map(|(id, _)| id);
            mat_of[i] = Some(existing.unwrap_or_else(|| {
                let id = model.add_material(&def.name, def.color);
                model.materials[id].texture = def.texture.clone();
                // Embed the image when the file is there (coordinates use the default projection).
                if let Some(t) = &def.texture
                    && let Ok(bytes) = std::fs::read(dir.join(t))
                    && crate::texture::image_dimensions(&bytes).is_some()
                {
                    let name = Path::new(t).file_name().and_then(|n| n.to_str()).unwrap_or(t).to_string();
                    model.materials[id].image = Some(crate::texture::TextureImage::new(bytes, name, 1000.0));
                }
                id
            }));
        }
    }
    let mesh = model.root_mesh_mut();
    let before: HashSet<FaceId> = mesh.faces.keys().collect();
    for (p, mi) in &data.polys {
        let made = super::stl::add_polygon(mesh, p);
        if let Some(m) = mi.and_then(|i| mat_of[i]) {
            for f in made {
                if let Some(face) = mesh.faces.get_mut(f) {
                    face.front = Some(m);
                }
            }
        }
    }
    for l in &data.lines {
        mesh.add_polyline(l, false, false);
    }
    if opts.merge_coplanar {
        let made = super::stl::new_faces(mesh, &before);
        mesh.merge_coplanar_faces(&made);
    }
    Ok(())
}

/// Clean number formatting: float noise removed, no `-0`, shortest round-trip digits.
fn num(x: f64) -> String {
    format!("{}", crate::geom::snap(x))
}
