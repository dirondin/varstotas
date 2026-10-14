//! File IO: native `.vrst` JSON, OBJ/STL export and import.

mod native;
mod obj;
mod stl;

pub use native::{from_json, load, save, to_json};
pub use obj::{export_obj, import_obj, import_obj_with, write_obj};
pub use stl::{export_stl, import_stl, import_stl_with, write_stl_binary};

/// Options of the OBJ / STL importers.
#[derive(Clone, Debug)]
pub struct ImportOptions {
    /// Merge edge-adjacent coplanar triangles/polygons with the same material back into
    /// polygonal faces (the classic "merge coplanar faces"); the diagonals disappear.
    pub merge_coplanar: bool,
    /// OBJ: read `mtllib` files and assign `usemtl` materials to faces.
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
