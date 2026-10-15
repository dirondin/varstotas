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

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn num(x: f64) -> String {
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

#[cfg(test)]
mod tests {
    #[test]
    fn crc32_known_value() {
        assert_eq!(super::crc32(b"123456789"), 0xCBF4_3926);
    }
}
