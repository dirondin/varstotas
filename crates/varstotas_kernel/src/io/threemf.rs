//! 3MF export (3D printing): one object per top-level group / component (loose geometry: one
//! "Model" object), world-space triangles in millimetres, face colours as base materials.
//! The package is a ZIP with stored (uncompressed) entries.

use super::IoError;
use crate::{MaterialId, Model, tessellate};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::Path;

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
 <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
 <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>
"#;

const RELS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
 <Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>
"#;

pub(super) fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

pub(super) fn num(x: f64) -> String {
    let s = format!("{:.4}", crate::geom::snap(x));
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" || s.is_empty() { "0".into() } else { s.into() }
}

/// The `3D/3dmodel.model` XML of the visible model.
pub fn write_3mf_model(model: &Model) -> String {
    let tess = tessellate(model);
    let root = model.root_mesh();
    // Objects: top-level owner → (vertex index, vertices, triangles with material).
    type Obj = (HashMap<[u64; 3], usize>, Vec<[f64; 3]>, Vec<([usize; 3], Option<MaterialId>)>);
    let mut objs: Vec<(Option<crate::InstanceId>, Obj)> = Vec::new();
    for f in &tess.faces {
        let owner = f.owner_path.first().copied();
        let oi = match objs.iter().position(|o| o.0 == owner) {
            Some(i) => i,
            None => {
                objs.push((owner, Default::default()));
                objs.len() - 1
            }
        };
        let (index, verts, tris) = &mut objs[oi].1;
        let mat = f.front.or(f.inherited).filter(|m| model.materials.contains_key(*m));
        for t in f.positions.as_chunks::<3>().0 {
            let mut ids = [0; 3];
            for (k, p) in t.iter().enumerate() {
                let key = [p.x, p.y, p.z].map(|c| crate::geom::snap(c).to_bits());
                let n = verts.len();
                ids[k] = *index.entry(key).or_insert_with(|| {
                    verts.push([p.x, p.y, p.z]);
                    n
                });
            }
            if ids[0] != ids[1] && ids[1] != ids[2] && ids[0] != ids[2] {
                tris.push((ids, mat));
            }
        }
    }
    // Base materials: every used material (index in the group).
    let mut mat_ids: Vec<MaterialId> = Vec::new();
    for (_, (_, _, tris)) in &objs {
        for (_, m) in tris {
            if let Some(m) = m
                && !mat_ids.contains(m)
            {
                mat_ids.push(*m);
            }
        }
    }
    let mut s = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<model unit=\"millimeter\" xml:lang=\"en-US\" \
         xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\">\n <metadata name=\"Application\">Varstotas</metadata>\n <resources>\n",
    );
    let has_mats = !mat_ids.is_empty();
    if has_mats {
        s.push_str("  <basematerials id=\"1\">\n");
        for m in &mat_ids {
            let mm = &model.materials[*m];
            let c = mm.color.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8);
            let _ = writeln!(
                s,
                "   <base name=\"{}\" displaycolor=\"#{:02X}{:02X}{:02X}{:02X}\"/>",
                xml_escape(&mm.name),
                c[0],
                c[1],
                c[2],
                c[3]
            );
        }
        s.push_str("  </basematerials>\n");
    }
    let mut items = Vec::new();
    for (k, (owner, (_, verts, tris))) in objs.iter().enumerate() {
        if tris.is_empty() {
            continue;
        }
        let id = k + 2;
        let name = match owner.and_then(|i| root.instances.get(i)) {
            Some(i) if !i.name.is_empty() => i.name.clone(),
            Some(i) => model.defs.get(i.def).map(|d| d.name.clone()).unwrap_or_default(),
            None => "Model".into(),
        };
        let _ = writeln!(s, "  <object id=\"{id}\" name=\"{}\" type=\"model\">\n   <mesh>\n    <vertices>", xml_escape(&name));
        for v in verts {
            let _ = writeln!(s, "     <vertex x=\"{}\" y=\"{}\" z=\"{}\"/>", num(v[0]), num(v[1]), num(v[2]));
        }
        s.push_str("    </vertices>\n    <triangles>\n");
        for (t, m) in tris {
            let _ = write!(s, "     <triangle v1=\"{}\" v2=\"{}\" v3=\"{}\"", t[0], t[1], t[2]);
            if let Some(i) = m.and_then(|m| mat_ids.iter().position(|x| *x == m)) {
                let _ = write!(s, " pid=\"1\" p1=\"{i}\"");
            }
            s.push_str("/>\n");
        }
        s.push_str("    </triangles>\n   </mesh>\n  </object>\n");
        items.push(id);
    }
    s.push_str(" </resources>\n <build>\n");
    for id in items {
        let _ = writeln!(s, "  <item objectid=\"{id}\"/>");
    }
    s.push_str(" </build>\n</model>\n");
    s
}

/// 3MF package bytes of the visible model.
pub fn write_3mf(model: &Model) -> Vec<u8> {
    let xml = write_3mf_model(model);
    zip_stored(&[
        ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
        ("_rels/.rels", RELS.as_bytes()),
        ("3D/3dmodel.model", xml.as_bytes()),
    ])
}

pub fn export_3mf(model: &Model, path: &Path) -> Result<(), IoError> {
    std::fs::write(path, write_3mf(model))?;
    Ok(())
}

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, t) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *t = c;
    }
    let mut c = !0u32;
    for &b in data {
        c = table[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    !c
}

/// A ZIP archive with stored (uncompressed) entries.
pub(crate) fn zip_stored(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in files {
        let crc = crc32(data);
        let offset = out.len() as u32;
        let header = |sig: u32, central: bool| {
            let mut h = Vec::new();
            h.extend(sig.to_le_bytes());
            if central {
                h.extend(20u16.to_le_bytes()); // made by
            }
            h.extend(20u16.to_le_bytes()); // version needed
            h.extend(0u16.to_le_bytes()); // flags
            h.extend(0u16.to_le_bytes()); // stored
            h.extend(0u16.to_le_bytes()); // time
            h.extend(0x21u16.to_le_bytes()); // date: 1980-01-01
            h.extend(crc.to_le_bytes());
            h.extend((data.len() as u32).to_le_bytes());
            h.extend((data.len() as u32).to_le_bytes());
            h.extend((name.len() as u16).to_le_bytes());
            h.extend(0u16.to_le_bytes()); // extra
            if central {
                h.extend(0u16.to_le_bytes()); // comment
                h.extend(0u16.to_le_bytes()); // disk
                h.extend(0u16.to_le_bytes()); // internal attrs
                h.extend(0u32.to_le_bytes()); // external attrs
                h.extend(offset.to_le_bytes());
            }
            h.extend(name.as_bytes());
            h
        };
        out.extend(header(0x0403_4B50, false));
        out.extend_from_slice(data);
        central.extend(header(0x0201_4B50, true));
    }
    let cd_offset = out.len() as u32;
    let cd_len = central.len() as u32;
    out.extend(central);
    out.extend(0x0605_4B50u32.to_le_bytes());
    out.extend([0u8; 4]); // disk numbers
    out.extend((files.len() as u16).to_le_bytes());
    out.extend((files.len() as u16).to_le_bytes());
    out.extend(cd_len.to_le_bytes());
    out.extend(cd_offset.to_le_bytes());
    out.extend(0u16.to_le_bytes());
    out
}

// ---- Import ----

/// A 3MF `transform` ("m00 m01 m02 m10 m11 m12 m20 m21 m22 m30 m31 m32", row vectors).
fn transform_3mf(s: Option<&str>) -> glam::DMat4 {
    let v: Vec<f64> = s.unwrap_or("").split_whitespace().filter_map(|x| x.parse().ok()).collect();
    if v.len() != 12 {
        return glam::DMat4::IDENTITY;
    }
    glam::DMat4::from_cols(
        glam::DVec4::new(v[0], v[1], v[2], 0.0),
        glam::DVec4::new(v[3], v[4], v[5], 0.0),
        glam::DVec4::new(v[6], v[7], v[8], 0.0),
        glam::DVec4::new(v[9], v[10], v[11], 1.0),
    )
}

/// One `.model` part of the package: its objects and colour resources.
struct Part {
    unit_mm: f64,
    objects: HashMap<String, super::xml::El>,
    /// Resource id → material indices (basematerials / colorgroup entries).
    colors: HashMap<String, Vec<usize>>,
}

struct Reader3mf {
    files: HashMap<String, Vec<u8>>,
    parts: HashMap<String, Part>,
    mats: Vec<super::meshes::ImpMat>,
}

fn unit_mm(u: Option<&str>) -> f64 {
    match u.unwrap_or("millimeter") {
        "micron" => 0.001,
        "centimeter" => 10.0,
        "inch" => 25.4,
        "foot" => 304.8,
        "meter" => 1000.0,
        _ => 1.0,
    }
}

impl Reader3mf {
    /// Load the model part `path` (once).
    fn part(&mut self, path: &str) -> Result<(), IoError> {
        if self.parts.contains_key(path) {
            return Ok(());
        }
        let key = path.trim_start_matches('/');
        let bytes = self.files.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v.clone()).ok_or_else(|| IoError::Format(format!("3MF: no part {path}")))?;
        let root = super::xml::El::parse(&String::from_utf8_lossy(&bytes))?;
        let mut part = Part { unit_mm: unit_mm(root.attr("unit")), objects: HashMap::new(), colors: HashMap::new() };
        if let Some(res) = root.child("resources") {
            for r in &res.children {
                let id = r.attr("id").unwrap_or("").to_string();
                match r.name.as_str() {
                    "object" => {
                        part.objects.insert(id, clone_el(r));
                    }
                    "basematerials" | "colorgroup" => {
                        let mut list = Vec::new();
                        for (k, b) in r.children.iter().enumerate() {
                            let c = b.attr("displaycolor").or(b.attr("color")).and_then(super::meshes::hex_color).unwrap_or([0.8, 0.8, 0.8, 1.0]);
                            let name = b.attr("name").map(str::to_string).unwrap_or_else(|| format!("Color {}", k + 1));
                            let i = match self.mats.iter().position(|m| m.name == name && m.color == c) {
                                Some(i) => i,
                                None => {
                                    self.mats.push(super::meshes::ImpMat { name, color: c, image: None });
                                    self.mats.len() - 1
                                }
                            };
                            list.push(i);
                        }
                        part.colors.insert(id, list);
                    }
                    _ => {}
                }
            }
        }
        self.parts.insert(path.to_string(), part);
        Ok(())
    }

    /// The triangles of object `id` of part `path` (components followed), into `out`.
    fn object(&mut self, path: &str, id: &str, xf: glam::DMat4, out: &mut super::meshes::ImpMesh, depth: usize) -> Result<(), IoError> {
        if depth > 32 {
            return Err(IoError::Format("3MF: components nested too deep".into()));
        }
        self.part(path)?;
        let part = &self.parts[path];
        let Some(obj) = part.objects.get(id) else { return Err(IoError::Format(format!("3MF: no object {id}"))) };
        if let Some(mesh) = obj.child("mesh") {
            let verts: Vec<glam::DVec3> = mesh
                .child("vertices")
                .map(|v| v.all("vertex").map(|p| glam::DVec3::new(p.num("x").unwrap_or(0.0), p.num("y").unwrap_or(0.0), p.num("z").unwrap_or(0.0))).collect())
                .unwrap_or_default();
            let (opid, oidx) = (obj.attr("pid"), obj.attr("pindex").and_then(|s| s.parse::<usize>().ok()));
            for t in mesh.child("triangles").into_iter().flat_map(|t| t.all("triangle")) {
                let idx: Vec<usize> = ["v1", "v2", "v3"].iter().filter_map(|k| t.attr(k)?.parse().ok()).collect();
                if idx.len() != 3 || idx.iter().any(|i| *i >= verts.len()) {
                    continue;
                }
                let pid = t.attr("pid").or(opid);
                let pi = t.attr("p1").and_then(|s| s.parse::<usize>().ok()).or(oidx).unwrap_or(0);
                let mat = pid.and_then(|p| part.colors.get(p)).and_then(|l| l.get(pi).copied());
                out.polys.push(super::meshes::ImpPoly { pts: idx.iter().map(|i| xf.transform_point3(verts[*i])).collect(), uv: None, mat });
            }
        }
        let comps: Vec<(String, String, glam::DMat4)> = obj
            .child("components")
            .into_iter()
            .flat_map(|c| c.all("component"))
            .map(|c| {
                let p = c.attr("path").map(str::to_string).unwrap_or_else(|| path.to_string());
                (p, c.attr("objectid").unwrap_or("").to_string(), xf * transform_3mf(c.attr("transform")))
            })
            .collect();
        for (p, oid, t) in comps {
            self.object(&p, &oid, t, out, depth + 1)?;
        }
        Ok(())
    }
}

fn clone_el(e: &super::xml::El) -> super::xml::El {
    super::xml::El { name: e.name.clone(), attrs: e.attrs.clone(), children: e.children.iter().map(clone_el).collect(), text: e.text.clone() }
}

/// Import a 3MF package: each build item becomes a group (named after its object), colours
/// of base materials / colour groups become materials.
pub fn import_3mf(model: &mut Model, path: &Path) -> Result<(), IoError> {
    import_3mf_with(model, path, &super::ImportOptions::default())
}

pub fn import_3mf_with(model: &mut Model, path: &Path, opts: &super::ImportOptions) -> Result<(), IoError> {
    let data = std::fs::read(path)?;
    add_3mf(model, &data, opts)
}

/// Add 3MF package bytes to the model.
pub fn add_3mf(model: &mut Model, data: &[u8], opts: &super::ImportOptions) -> Result<(), IoError> {
    let files: HashMap<String, Vec<u8>> = super::zip::unzip(data, |n| n.ends_with(".model") || n.ends_with(".rels")).map_err(IoError::Format)?.into_iter().collect();
    // The start part: the 3D model relationship of _rels/.rels, else the first .model file.
    let start = files
        .get("_rels/.rels")
        .and_then(|r| super::xml::El::parse(&String::from_utf8_lossy(r)).ok())
        .and_then(|r| r.all("Relationship").find(|x| x.attr("Type").is_some_and(|t| t.ends_with("/3dmodel"))).and_then(|x| x.attr("Target").map(str::to_string)))
        .or_else(|| files.keys().find(|k| k.ends_with(".model")).map(|k| format!("/{k}")))
        .ok_or_else(|| IoError::Format("3MF: no 3D model part".into()))?;
    let mut rd = Reader3mf { files, parts: HashMap::new(), mats: Vec::new() };
    rd.part(&start)?;
    let bytes = rd.files.iter().find(|(k, _)| k.eq_ignore_ascii_case(start.trim_start_matches('/'))).map(|(_, v)| v.clone()).unwrap_or_default();
    let root = super::xml::El::parse(&String::from_utf8_lossy(&bytes))?;
    let k = rd.parts[&start].unit_mm;
    let items: Vec<(String, String, glam::DMat4)> = root
        .child("build")
        .into_iter()
        .flat_map(|b| b.all("item"))
        .map(|i| {
            let p = i.attr("path").map(str::to_string).unwrap_or_else(|| start.clone());
            (p, i.attr("objectid").unwrap_or("").to_string(), transform_3mf(i.attr("transform")))
        })
        .collect();
    let mut meshes = Vec::new();
    for (p, oid, t) in items {
        let name = rd.parts.get(&p).and_then(|x| x.objects.get(&oid)).and_then(|o| o.attr("name")).unwrap_or("").to_string();
        let mut m = super::meshes::ImpMesh { name, polys: Vec::new() };
        rd.object(&p, &oid, t, &mut m, 0)?;
        meshes.push(m);
    }
    // Coordinates are in the start part's unit.
    super::meshes::add_meshes(model, meshes, &rd.mats, glam::DMat4::from_scale(glam::DVec3::splat(k)), opts).map(|_| ())
}

#[cfg(test)]
mod tests {
    #[test]
    fn crc32_known_value() {
        assert_eq!(super::crc32(b"123456789"), 0xCBF4_3926);
    }
}
