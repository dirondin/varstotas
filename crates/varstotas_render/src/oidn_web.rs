//! Open Image Denoise in the web build: there is no library to load, the path tracer uses its
//! own denoiser.

use std::path::PathBuf;

pub fn library() -> Option<PathBuf> {
    None
}

pub fn available() -> bool {
    false
}

pub(crate) fn denoise(_w: usize, _h: usize, _color: &mut [f32], _albedo: &mut [f32], _normal: &mut [f32]) -> Option<Vec<f32>> {
    None
}
