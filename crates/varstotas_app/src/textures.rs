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

/// Longest side of a file's preview picture.
pub const PREVIEW_SIDE: u32 = 512;

/// An image (any readable format) as a file preview picture: at most [`PREVIEW_SIDE`]
/// pixels, JPEG when opaque, PNG when it has transparency.
pub fn preview_bytes(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let img = if img.width().max(img.height()) > PREVIEW_SIDE { img.thumbnail(PREVIEW_SIDE, PREVIEW_SIDE) } else { img };
    let rgba = img.to_rgba8();
    let mut out = Vec::new();
    if rgba.pixels().any(|p| p[3] < 255) {
        rgba.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png).map_err(|e| e.to_string())?;
    } else {
        let rgb = image::DynamicImage::ImageRgba8(rgba).to_rgb8();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 88).encode_image(&rgb).map_err(|e| e.to_string())?;
    }
    Ok(out)
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

/// Texture slots of a material: the base (colour) texture, then the physical maps.
pub const SLOTS: [&str; 6] = ["base", "normal", "roughness", "metallic", "ao", "emission"];

/// Images larger than this (longest side, pixels) get a size warning and a downscale button:
/// the renderer samples at most 1024 px, larger files only grow the document.
pub const LARGE_SIDE: u32 = 2048;

/// The slot an image file belongs to by its name suffix (`wood_color.png`,
/// `Bricks097_4K-PNG_NormalGL.png`, `brick_rough_2k.jpg`…), and whether it is a DirectX
/// normal map (green channel flipped; an OpenGL one is preferred).
pub fn slot_of_file(name: &str) -> Option<(&'static str, bool)> {
    let stem = name.rsplit_once('.').map_or(name, |(s, _)| s).to_ascii_lowercase();
    let tokens: Vec<&str> = stem.split(|c: char| !c.is_ascii_alphanumeric()).filter(|t| !t.is_empty()).collect();
    let dx = tokens.iter().any(|t| *t == "dx" || t.ends_with("dx") && t.starts_with("normal"));
    // the last meaningful token decides (resolution / format tokens are skipped)
    for t in tokens.iter().rev() {
        let slot = match *t {
            "color" | "colour" | "col" | "albedo" | "basecolor" | "diffuse" | "diff" | "base" => "base",
            "normal" | "normalgl" | "normaldx" | "nor" | "nrm" | "norm" => "normal",
            "rough" | "roughness" | "rgh" => "roughness",
            "metal" | "metallic" | "metalness" | "met" => "metallic",
            "ao" | "occlusion" | "ambientocclusion" | "occ" => "ao",
            "emission" | "emissive" | "emit" => "emission",
            "gl" | "dx" | "png" | "jpg" | "map" | "tex" | "texture" => continue,
            t if t.len() <= 3 && t.ends_with('k') && t[..t.len() - 1].chars().all(|c| c.is_ascii_digit()) => continue,
            _ => return None,
        };
        // "base" alone is a name word unless it is followed by colour ("base_color")
        if slot == "base" && *t == "base" {
            return None;
        }
        return Some((slot, slot == "normal" && dx));
    }
    None
}

/// The image files of `dir` sorted into slots by name (the first match per slot; an OpenGL
/// normal map wins over a DirectX one).
pub fn map_set_in(dir: &Path) -> Vec<(&'static str, std::path::PathBuf)> {
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .map(|r| r.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    files.retain(|p| p.extension().and_then(|e| e.to_str()).is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "png" | "jpg" | "jpeg")));
    files.sort();
    let mut out: Vec<(&'static str, std::path::PathBuf, bool)> = Vec::new();
    for f in files {
        let Some((slot, dx)) = f.file_name().and_then(|n| n.to_str()).and_then(slot_of_file) else { continue };
        match out.iter_mut().find(|(s, _, _)| *s == slot) {
            Some(e) if e.2 && !dx => *e = (slot, f, dx),
            Some(_) => {}
            None => out.push((slot, f, dx)),
        }
    }
    out.sort_by_key(|(s, _, _)| SLOTS.iter().position(|x| x == s));
    out.into_iter().map(|(s, f, _)| (s, f)).collect()
}

/// Re-encode an image with its longest side at most `max` pixels (PNG stays PNG, anything
/// else becomes JPEG).
pub fn downscale(bytes: &[u8], max: u32) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let img = img.resize(max, max, image::imageops::FilterType::Lanczos3);
    let mut out = Vec::new();
    if bytes.starts_with(b"\x89PNG") {
        img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png).map_err(|e| e.to_string())?;
    } else {
        let rgb = img.to_rgb8();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 90).encode_image(&rgb).map_err(|e| e.to_string())?;
    }
    Ok(out)
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

    #[test]
    fn slots_by_file_name() {
        for (name, want) in [
            ("wood_color.png", Some(("base", false))),
            ("Bricks097_4K-PNG_Color.png", Some(("base", false))),
            ("Bricks097_4K-PNG_NormalGL.png", Some(("normal", false))),
            ("Bricks097_4K-PNG_NormalDX.png", Some(("normal", true))),
            ("red_brick_nor_gl_2k.jpg", Some(("normal", false))),
            ("red_brick_nor_dx_2k.jpg", Some(("normal", true))),
            ("red_brick_rough_2k.jpg", Some(("roughness", false))),
            ("metal_plate_metal_1k.png", Some(("metallic", false))),
            ("red_brick_ao_2k.jpg", Some(("ao", false))),
            ("lamp_Emissive.png", Some(("emission", false))),
            ("red_brick_diff_2k.jpg", Some(("base", false))),
            ("Wall_BaseColor.png", Some(("base", false))),
            ("red_brick_disp_2k.png", None),
            ("photo.jpg", None),
            ("base.png", None),
        ] {
            assert_eq!(slot_of_file(name), want, "{name}");
        }
    }

    #[test]
    fn downscale_keeps_png_and_aspect() {
        let img = image::RgbaImage::from_pixel(64, 32, image::Rgba([10, 20, 30, 255]));
        let mut png = std::io::Cursor::new(Vec::new());
        img.write_to(&mut png, image::ImageFormat::Png).unwrap();
        let out = downscale(&png.into_inner(), 16).unwrap();
        assert!(out.starts_with(b"\x89PNG"));
        assert_eq!(varstotas_kernel::texture::image_dimensions(&out), Some([16, 8]));
    }
}
