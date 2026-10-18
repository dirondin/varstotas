//! Native `.vrst` format: `{"format":"varstotas","version":1,"model":{...}}`.

use super::IoError;
use crate::Model;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const FORMAT: &str = "varstotas";
pub const VERSION: u32 = 1;

#[derive(Serialize)]
struct FileOut<'a> {
    format: &'a str,
    version: u32,
    model: &'a Model,
}

#[derive(Deserialize)]
struct FileIn {
    format: String,
    version: u32,
    model: Model,
}

/// Serialize a model to the `.vrst` JSON text.
pub fn to_json(model: &Model) -> Result<String, IoError> {
    serde_json::to_string(&FileOut { format: FORMAT, version: VERSION, model }).map_err(|e| IoError::Format(e.to_string()))
}

/// Parse `.vrst` JSON text (caches are rebuilt).
pub fn from_json(text: &str) -> Result<Model, IoError> {
    let f: FileIn = serde_json::from_str(text).map_err(|e| IoError::Format(e.to_string()))?;
    if f.format != FORMAT {
        return Err(IoError::Format(format!("not a varstotas file (format = {:?})", f.format)));
    }
    if f.version > VERSION {
        return Err(IoError::Format(format!("unsupported version {}", f.version)));
    }
    let mut m = f.model;
    if !m.defs.contains_key(m.root) {
        return Err(IoError::Format("missing root definition".into()));
    }
    m.rebuild_caches();
    Ok(m)
}

pub fn save(model: &Model, path: &Path) -> Result<(), IoError> {
    let s = to_json(model)?;
    // Write atomically-ish: temp file then rename.
    let tmp = path.with_extension("vrst.tmp");
    std::fs::write(&tmp, s)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub fn load(path: &Path) -> Result<Model, IoError> {
    let s = std::fs::read_to_string(path)?;
    from_json(&s)
}

// ---- material files ------------------------------------------------------------------------

/// Format tag of a `.vrstmat` material file:
/// `{"format":"varstotas-material","version":1,"material":{...}}` (maps embedded).
pub const MATERIAL_FORMAT: &str = "varstotas-material";
/// Extension of material files.
pub const MATERIAL_EXT: &str = "vrstmat";

#[derive(Serialize)]
struct MatOut<'a> {
    format: &'a str,
    version: u32,
    material: &'a crate::Material,
}

#[derive(Deserialize)]
struct MatIn {
    format: String,
    version: u32,
    material: crate::Material,
}

/// Write one material (colour, texture, PBR parameters, maps and shader) to a file.
pub fn save_material(mat: &crate::Material, path: &Path) -> Result<(), IoError> {
    let s = serde_json::to_string(&MatOut { format: MATERIAL_FORMAT, version: VERSION, material: mat }).map_err(|e| IoError::Format(e.to_string()))?;
    let tmp = path.with_extension("vrstmat.tmp");
    std::fs::write(&tmp, s)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Read a material file written by [`save_material`].
pub fn load_material(path: &Path) -> Result<crate::Material, IoError> {
    let f: MatIn = serde_json::from_str(&std::fs::read_to_string(path)?).map_err(|e| IoError::Format(e.to_string()))?;
    if f.format != MATERIAL_FORMAT {
        return Err(IoError::Format(format!("not a varstotas material (format = {:?})", f.format)));
    }
    if f.version > VERSION {
        return Err(IoError::Format(format!("unsupported version {}", f.version)));
    }
    Ok(f.material)
}

#[cfg(test)]
mod material_tests {
    use super::*;

    #[test]
    fn a_material_file_round_trips() {
        let mut m = crate::Model::new();
        let id = m.add_material("Brass", [0.9, 0.7, 0.3, 1.0]);
        let mat = &mut m.materials[id];
        mat.pbr = Some(crate::Pbr { metallic: 1.0, roughness: 0.2, shader: Some(crate::CustomShader::new("// x\nfn surface(s: Surface, p: Params) -> Surface { return s; }")), ..Default::default() });
        mat.image = Some(crate::TextureImage { bytes: vec![1u8, 2, 3].into(), pixels: [1, 1], size: [500.0, 500.0], file_name: "b.png".into() });
        let path = std::env::temp_dir().join(format!("varstotas_mat_{}.{MATERIAL_EXT}", std::process::id()));
        save_material(&m.materials[id], &path).unwrap();
        let back = load_material(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(back.name, "Brass");
        assert_eq!(back.pbr, m.materials[id].pbr);
        assert_eq!(back.image, m.materials[id].image);
        // A model file is not a material.
        let p2 = std::env::temp_dir().join(format!("varstotas_notmat_{}.vrst", std::process::id()));
        save(&m, &p2).unwrap();
        assert!(load_material(&p2).is_err());
        let _ = std::fs::remove_file(&p2);
    }
}
