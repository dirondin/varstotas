//! File IO: native `.vrst` JSON, OBJ/STL/DXF/glTF export and import, 3MF export and
//! import, COLLADA import and export, FBX import, PLY import and export, CSV points, faces as JSON.

mod dae;
mod dxf;
mod fbx;
mod gltf;
mod meshes;
mod native;
mod obj;
mod ply;
mod points;
mod security;
mod surface;
pub use surface::mesh_from_triangles;
mod stl;
mod threemf;
mod xml;
mod zip;
pub use security::{
    MAX_MODEL_BYTES, atomic_write, export_bundle, read_limited, resource_path, validate_model,
    write_bundle, write_bundle_approved,
};

pub use dae::{add_dae, export_dae, import_dae, import_dae_with, write_dae};
pub use dxf::{DxfData, add_dxf, export_dxf, import_dxf, import_dxf_with, parse_dxf, write_dxf};
pub use fbx::{add_fbx, import_fbx, import_fbx_with};
pub use gltf::{add_gltf, export_glb, import_gltf, import_gltf_with, write_glb};
pub use native::{VERSION as NATIVE_VERSION, MIN_VERSION as NATIVE_MIN_VERSION, MATERIAL_VERSION, MATERIAL_EXT, MATERIAL_FORMAT, from_json, load, load_material, save, save_with_backup, save_material, to_json};
pub use obj::{export_obj, import_obj, import_obj_with, write_obj, write_obj_with_textures};
pub use threemf::{add_3mf, export_3mf, import_3mf, import_3mf_with, write_3mf, write_3mf_model};
pub use zip::unzip;
pub use ply::{add_ply, export_ply, import_ply, import_ply_with, write_ply};
pub use points::{export_faces_json, faces_json, import_points_csv, parse_points_csv};
pub use stl::{export_stl, import_stl, import_stl_with, write_stl_binary};

/// Options of the OBJ / STL / DXF / glTF importers.
#[derive(Clone, Debug)]
pub struct ImportOptions {
    /// Merge edge-adjacent coplanar triangles/polygons with the same material back into
    /// polygonal faces (the classic "merge coplanar faces"); the diagonals disappear.
    pub merge_coplanar: bool,
    /// OBJ / glTF: read materials (and textures) and assign them to faces.
    pub materials: bool,
    /// Recover Blender exports of legacy Lambert/Blinn material sets that incorrectly mark
    /// every opaque surface as transmission=1 / IOR=1. Disable for literal source factors.
    pub repair_legacy_materials: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self { merge_coplanar: true, materials: true, repair_legacy_materials: true }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum IoError {
    Io(#[from] std::io::Error),
    Format(String),
}

impl std::fmt::Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoError::Io(e) => f.write_str(&varstotas_i18n::tr!("kernel-io-error-io", why = e.to_string())),
            IoError::Format(why) => f.write_str(&varstotas_i18n::tr!("kernel-io-error-format", why = why.as_str())),
        }
    }
}
