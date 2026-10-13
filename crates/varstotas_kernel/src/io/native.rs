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
