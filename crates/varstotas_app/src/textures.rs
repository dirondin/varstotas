//! Texture images: decoding (PNG / JPEG), the GPU texture set of a model, and creating
//! textured materials from image files.

use varstotas_kernel::{Model, TextureImage};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Largest side kept after decoding (the renderer uses at most 1024 px layers).
const MAX_DECODED: u32 = 1024;

/// A decoded image: width, height, sRGB RGBA8 rows top to bottom.
pub type Decoded = Arc<(u32, u32, Vec<u8>)>;

pub fn decode(bytes: &[u8]) -> Result<Decoded, String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let img = if img.width().max(img.height()) > MAX_DECODED {
        img.resize(MAX_DECODED, MAX_DECODED, image::imageops::FilterType::Triangle)
    } else {
        img
    };
    let rgba = img.to_rgba8();
    Ok(Arc::new((rgba.width(), rgba.height(), rgba.into_raw())))
}

/// Average colour (sRGB 0..1, opaque) of a decoded image: the material's swatch / OBJ colour.
pub fn average_color(d: &Decoded) -> [f32; 4] {
    let (_, _, px) = &**d;
    let n = (px.len() / 4).max(1) as f64;
    let mut sum = [0f64; 3];
    for p in px.as_chunks::<4>().0 {
        for c in 0..3 {
            sum[c] += p[c] as f64;
        }
    }
    [(sum[0] / n / 255.0) as f32, (sum[1] / n / 255.0) as f32, (sum[2] / n / 255.0) as f32, 1.0]
}

/// A small square-fitted thumbnail for the material swatches.
pub fn thumbnail(bytes: &[u8], side: u32) -> Option<egui::ColorImage> {
    let img = image::load_from_memory(bytes).ok()?.thumbnail(side, side).to_rgba8();
    Some(egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw()))
}

/// Read an image file into a texture `width_mm` wide (height from the aspect ratio); also
/// returns the decoded image (to check it and for the average colour).
pub fn load_file(path: &Path, width_mm: f64) -> Result<(TextureImage, Decoded), String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let decoded = decode(&bytes)?;
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("texture").to_string();
    let mut tex = TextureImage::new(bytes, name, width_mm);
    if tex.pixels == [0, 0] {
        // Header not understood by the kernel: take the decoder's size.
        let (w, h, _) = &*decoded;
        tex.pixels = [*w, *h];
        tex.size[1] = width_mm * *h as f64 / (*w).max(1) as f64;
    }
    Ok((tex, decoded))
}

/// Uploads the model's textures to the renderer when they change, decoding each image once.
#[derive(Default)]
pub struct TextureSync {
    /// (image bytes pointer, tile size) per layer of the uploaded set.
    key: Option<Vec<(usize, [f64; 2])>>,
    cache: HashMap<usize, (Arc<[u8]>, Option<Decoded>)>,
}

impl TextureSync {
    /// Texture data per layer (see `scene_builder::texture_layers`) if the set changed.
    pub fn changed(&mut self, model: &Model) -> Option<Vec<varstotas_render::TextureData>> {
        let layers = crate::scene_builder::texture_layers(model);
        let mut imgs: Vec<&TextureImage> = layers.iter().filter_map(|id| model.materials[*id].image.as_ref()).collect();
        // Emission maps of the Realistic style follow the material textures.
        imgs.extend(
            crate::pbr::emission_layers(model)
                .iter()
                .filter_map(|id| model.materials[*id].pbr.as_ref().and_then(|p| p.emission_map.as_ref())),
        );
        imgs.truncate(varstotas_render::MAX_TEXTURE_LAYERS);
        let key: Vec<(usize, [f64; 2])> = imgs.iter().map(|t| (Arc::as_ptr(&t.bytes) as *const u8 as usize, t.size)).collect();
        if self.key.as_ref() == Some(&key) {
            return None;
        }
        self.cache.retain(|k, _| key.iter().any(|(p, _)| p == k));
        let data = imgs
            .iter()
            .zip(&key)
            .map(|(t, (ptr, _))| {
                let entry = self.cache.entry(*ptr).or_insert_with(|| {
                    let d = decode(&t.bytes).map_err(|e| log::warn!("texture {}: {e}", t.file_name)).ok();
                    (t.bytes.clone(), d)
                });
                match &entry.1 {
                    Some(d) => varstotas_render::TextureData {
                        width: d.0,
                        height: d.1,
                        rgba: d.2.clone(),
                        size_mm: [t.size[0] as f32, t.size[1] as f32],
                    },
                    // Undecodable: a magenta tile so the problem is visible.
                    None => varstotas_render::TextureData { width: 1, height: 1, rgba: vec![255, 0, 255, 255], size_mm: [1000.0; 2] },
                }
            })
            .collect();
        self.key = Some(key);
        Some(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2×1 PNG (red, blue) encoded with the `image` crate.
    pub fn tiny_png() -> Vec<u8> {
        let img = image::RgbaImage::from_raw(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 255]).unwrap();
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png).unwrap();
        out.into_inner()
    }

    #[test]
    fn decode_and_average() {
        let d = decode(&tiny_png()).unwrap();
        assert_eq!((d.0, d.1), (2, 1));
        let c = average_color(&d);
        assert!((c[0] - 0.5).abs() < 0.01 && c[1] == 0.0 && (c[2] - 0.5).abs() < 0.01);
        assert_eq!(varstotas_kernel::texture::image_dimensions(&tiny_png()), Some([2, 1]));
    }

    #[test]
    fn sync_uploads_only_on_change() {
        let mut m = Model::new();
        let mut s = TextureSync::default();
        assert_eq!(s.changed(&m).map(|v| v.len()), Some(0));
        assert!(s.changed(&m).is_none());
        let id = m.add_material("Brick", [1.0; 4]);
        m.materials[id].image = Some(TextureImage::new(tiny_png(), "b.png", 500.0));
        let v = s.changed(&m).unwrap();
        assert_eq!((v.len(), v[0].width, v[0].size_mm), (1, 2, [500.0, 250.0]));
        assert!(s.changed(&m).is_none());
        let snapshot = m.clone(); // undo snapshots share the bytes
        assert!(s.changed(&snapshot).is_none());
        m.materials[id].image.as_mut().unwrap().size = [1000.0, 500.0];
        assert!(s.changed(&m).is_some());
    }
}
