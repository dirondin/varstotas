//! File IO: native `.vrst` JSON, OBJ/STL export and import.

mod native;
mod obj;
mod stl;

pub use native::{from_json, load, save, to_json};
pub use obj::{export_obj, import_obj, write_obj};
pub use stl::{export_stl, import_stl, write_stl_binary};

#[derive(thiserror::Error, Debug)]
pub enum IoError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("format: {0}")]
    Format(String),
}
