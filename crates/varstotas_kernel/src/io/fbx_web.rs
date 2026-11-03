//! FBX import in the web build: the C library it uses is not built for the browser.

use super::{ImportOptions, IoError};
use crate::Model;
use std::path::Path;

fn unavailable() -> IoError {
    IoError::Format(varstotas_i18n::tr!("kernel-io-fbx-web"))
}

pub fn import_fbx(_model: &mut Model, _path: &Path) -> Result<(), IoError> {
    Err(unavailable())
}

pub fn import_fbx_with(_model: &mut Model, _path: &Path, _opts: &ImportOptions) -> Result<(), IoError> {
    Err(unavailable())
}

pub fn add_fbx(_model: &mut Model, _data: &[u8], _dir: Option<&Path>, _opts: &ImportOptions) -> Result<(), IoError> {
    Err(unavailable())
}
