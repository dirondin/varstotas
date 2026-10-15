//! File IO: native `.vrst` JSON, OBJ/STL/DXF/glTF export and import, 3MF export.

mod dxf;
mod gltf;
mod native;
mod obj;
mod stl;
mod threemf;

pub use dxf::{DxfData, add_dxf, export_dxf, import_dxf, import_dxf_with, parse_dxf, write_dxf};
pub use gltf::{add_gltf, export_glb, import_gltf, import_gltf_with, write_glb};
pub use native::{from_json, load, save, to_json};
pub use obj::{export_obj, import_obj, import_obj_with, write_obj, write_obj_with_textures};
pub use threemf::{export_3mf, write_3mf, write_3mf_model};
pub use stl::{export_stl, import_stl, import_stl_with, write_stl_binary};

/// Options of the OBJ / STL / DXF / glTF importers.
#[derive(Clone, Debug)]
pub struct ImportOptions {
    /// Merge edge-adjacent coplanar triangles/polygons with the same material back into
    /// polygonal faces (the classic "merge coplanar faces"); the diagonals disappear.
    pub merge_coplanar: bool,
    /// OBJ / glTF: read materials (and textures) and assign them to faces.
    pub materials: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self { merge_coplanar: true, materials: true }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum IoError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("format: {0}")]
    Format(String),
}
