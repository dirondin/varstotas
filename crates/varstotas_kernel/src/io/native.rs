//! Native `.vrst` format: `{"format":"varstotas","version":4,"model":{...}}`.

use super::IoError;
use crate::Model;
use serde::{Deserialize, Serialize};
use std::path::Path;
use varstotas_i18n::tr;

pub const FORMAT: &str = "varstotas";
/// V1: original; V2: metadata; V3: corner normals; V4: affine UV placement with shear.
pub const VERSION: u32 = 4;
pub const MIN_VERSION: u32 = 1;
/// Material files have their own independent schema version.
pub const MATERIAL_VERSION: u32 = 1;

#[derive(Serialize)]
struct FileOut<'a> {
    format: &'a str,
    version: u32,
    model: &'a Model,
}

#[derive(Deserialize)]
struct FileHeader {
    format: String,
    version: u32,
}

#[derive(Deserialize)]
struct FileIn {
    model: Model,
}

/// Serialize a model to the `.vrst` JSON text.
pub fn to_json(model: &Model) -> Result<String, IoError> {
    serde_json::to_string(&FileOut { format: FORMAT, version: VERSION, model }).map_err(|e| IoError::Format(e.to_string()))
}

/// Parse `.vrst` JSON text (caches are rebuilt).
pub fn from_json(text: &str) -> Result<Model, IoError> {
    if text.len() > super::MAX_MODEL_BYTES {
        return Err(IoError::Format("Model exceeds file size limit".into()));
    }
    let f: FileHeader = serde_json::from_str(text).map_err(|e| IoError::Format(e.to_string()))?;
    if f.format != FORMAT {
        return Err(IoError::Format(tr!("kernel-io-not-a-varstotas-file-format-format", format = format!("{:?}", f.format))));
    }
    if !(MIN_VERSION..=VERSION).contains(&f.version) {
        return Err(IoError::Format(tr!("kernel-io-unsupported-version-version", version = f.version.to_string())));
    }
    // Validate the envelope before deserializing a possibly incompatible model schema.
    // V1 has no metadata; Model's serde defaults supply empty fields during migration.
    let mut m = serde_json::from_str::<FileIn>(text)
        .map_err(|e| IoError::Format(e.to_string()))?.model;
    if !m.defs.contains_key(m.root) {
        return Err(IoError::Format(tr!("kernel-io-missing-root-definition")));
    }
    super::validate_model(&m)?;
    m.rebuild_caches();
    Ok(m)
}

pub fn save(model: &Model, path: &Path) -> Result<(), IoError> {
    super::validate_model(model)?;
    let s = to_json(model)?;
    super::atomic_write(path, s.as_bytes())?;
    Ok(())
}

/// One previous explicit save, kept as a normal, reopenable `.vrst` document.
pub fn backup_path(path: &Path) -> std::path::PathBuf {
    let mut name = path.file_stem().unwrap_or_default().to_os_string();
    name.push(".bak.vrst");
    path.with_file_name(name)
}

/// Preserve the previous file before replacing it. Backup failure leaves the original intact.
/// Autosaves and generated fixtures use `save` and do not rotate document backups.
pub fn save_with_backup(model: &Model, path: &Path) -> Result<(), IoError> {
    super::validate_model(model)?;
    let text = to_json(model)?;
    match std::fs::symlink_metadata(path) {
        Ok(meta) => {
            if !meta.is_file() || meta.file_type().is_symlink() {
                return Err(std::io::Error::other("Expected a regular document file").into());
            }
            let previous = super::read_limited(path, super::MAX_MODEL_BYTES)?;
            // Repeated Save without changes must not discard the last distinct version.
            if previous == text.as_bytes() { return Ok(()); }
            super::atomic_write(backup_path(path), &previous)?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    super::atomic_write(path, text.as_bytes())?;
    Ok(())
}

pub fn load(path: &Path) -> Result<Model, IoError> {
    let bytes = super::read_limited(path, super::MAX_MODEL_BYTES)?;
    let s = std::str::from_utf8(&bytes).map_err(|e| IoError::Format(e.to_string()))?;
    from_json(s)
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
    let s = serde_json::to_string(&MatOut { format: MATERIAL_FORMAT, version: MATERIAL_VERSION, material: mat }).map_err(|e| IoError::Format(e.to_string()))?;
    super::atomic_write(path, s.as_bytes())?;
    Ok(())
}

/// Read a material file written by [`save_material`].
pub fn load_material(path: &Path) -> Result<crate::Material, IoError> {
    let f: MatIn = serde_json::from_slice(&super::read_limited(path, super::MAX_MODEL_BYTES)?).map_err(|e| IoError::Format(e.to_string()))?;
    if f.format != MATERIAL_FORMAT {
        return Err(IoError::Format(tr!("kernel-io-not-a-varstotas-material-format-format", format = format!("{:?}", f.format))));
    }
    if f.version != MATERIAL_VERSION {
        return Err(IoError::Format(tr!("kernel-io-unsupported-version-version", version = f.version.to_string())));
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
        mat.image = Some(crate::TextureImage { bytes: vec![1u8, 2, 3].into(), pixels: [1, 1], size: [500.0, 500.0], file_name: "b.png".into(), look: crate::TexLook { decal: true, ..Default::default() } });
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
