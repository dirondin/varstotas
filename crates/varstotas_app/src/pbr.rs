//! The Realistic style on the app side: the PBR material table, the extra texture layers
//! (emission maps in the colour array; packed occlusion / roughness / metalness and normal maps
//! in the data array) and the HDRI panorama, uploaded to the renderer when they change.

use crate::textures::{Decoded, decode};
use varstotas_kernel::{MaterialId, Model, TextureImage};
use varstotas_render::{EnvImage, PbrMaterial, TextureData};
use std::collections::HashMap;
use std::sync::Arc;

/// Materials with PBR parameters in table order (`TexVertex::mats` value `i + 1`).
pub fn pbr_materials(model: &Model) -> Vec<MaterialId> {
    let mut v: Vec<MaterialId> = model.materials.iter().filter(|(_, m)| m.pbr.is_some()).map(|(id, _)| id).collect();
    v.truncate(varstotas_render::MAX_PBR_MATERIALS);
    v
}

/// What a data-texture layer holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataKind {
    /// Occlusion (R), roughness (G), metalness (B), from the material's three maps.
    Orm,
    Normal,
}

/// Emission maps: colour layers after `scene_builder::texture_layers`.
pub fn emission_layers(model: &Model) -> Vec<MaterialId> {
    pbr_materials(model)
        .into_iter()
        .filter(|id| model.materials[*id].pbr.as_ref().is_some_and(|p| p.emission_map.is_some()))
        .collect()
}

/// Data-texture layers in order (layer `i + 1`).
pub fn data_layers(model: &Model) -> Vec<(MaterialId, DataKind)> {
    let mut v = Vec::new();
    for id in pbr_materials(model) {
        let p = model.materials[id].pbr.as_ref().expect("pbr");
        if p.ao_map.is_some() || p.roughness_map.is_some() || p.metallic_map.is_some() {
            v.push((id, DataKind::Orm));
        }
        if p.normal_map.is_some() {
            v.push((id, DataKind::Normal));
        }
    }
    v.truncate(varstotas_render::MAX_TEXTURE_LAYERS);
    v
}

fn srgb(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
}

/// The custom surface shaders of the table's materials (each source once, in order of first
/// use): `PbrMaterial::shader` = index + 1.
pub fn custom_surfaces(model: &Model) -> Vec<varstotas_render::CustomSurface> {
    let mut out: Vec<varstotas_render::CustomSurface> = Vec::new();
    for id in pbr_materials(model) {
        if let Some(sh) = model.materials[id].pbr.as_ref().and_then(|p| p.shader.as_ref())
            && !out.iter().any(|c| c.source == sh.source)
        {
            let params = varstotas_kernel::shader_params(&sh.source).into_iter().map(|p| p.name).collect();
            out.push(varstotas_render::CustomSurface { source: sh.source.clone(), params });
        }
    }
    out
}

/// The renderer's material table (`Renderer::set_materials`), in `pbr_materials` order.
pub fn material_table(model: &Model) -> Vec<PbrMaterial> {
    let base = crate::scene_builder::texture_layers(model).len();
    let customs = custom_surfaces(model);
    let emission = emission_layers(model);
    let data = data_layers(model);
    let layer = |i: Option<usize>| i.map_or(0, |i| (i + 1).min(u16::MAX as usize) as u16);
    pbr_materials(model)
        .into_iter()
        .map(|id| {
            let p = model.materials[id].pbr.clone().unwrap_or_default().sanitized();
            let em = emission.iter().position(|e| *e == id).map(|j| base + j).filter(|i| *i < varstotas_render::MAX_TEXTURE_LAYERS);
            let orm = data.iter().position(|d| *d == (id, DataKind::Orm));
            let normal = data.iter().position(|d| *d == (id, DataKind::Normal));
            let mask = p.ao_map.is_some() as u8 | (p.roughness_map.is_some() as u8) << 1 | (p.metallic_map.is_some() as u8) << 2;
            let (shader, params) = match &p.shader {
                Some(sh) => {
                    let mut v = [0.0; 8];
                    for (i, x) in v.iter_mut().enumerate().take(varstotas_kernel::shader_params(&sh.source).len()) {
                        *x = sh.param(i);
                    }
                    (customs.iter().position(|c| c.source == sh.source).map_or(0, |i| i as u16 + 1), v)
                }
                None => (0, [0.0; 8]),
            };
            PbrMaterial {
                metallic: p.metallic,
                roughness: p.roughness,
                transmission: p.transmission,
                ior: p.ior,
                emission: p.emission.map(|c| srgb(c) * p.emission_strength),
                normal_scale: p.normal_scale,
                emission_layer: layer(em),
                orm_layer: layer(orm),
                orm_mask: mask,
                normal_layer: layer(normal),
                shader,
                params,
                clearcoat: p.clearcoat,
                clearcoat_roughness: p.clearcoat_roughness,
                sheen: p.sheen,
                sheen_tint: p.sheen_tint,
                anisotropy: p.anisotropy,
                anti_tiling: p.anti_tiling,
                bevel: p.bevel,
            }
        })
        .collect()
}

/// Face alpha of a material in the Realistic style: transmission makes it see-through, also a
/// custom shader that sets it (its faces must be blended to show what is behind).
pub fn realistic_alpha(color_alpha: f32, pbr: Option<&varstotas_kernel::Pbr>) -> f32 {
    let t = pbr.map_or(0.0, |p| {
        let by_shader = p.shader.as_ref().is_some_and(|sh| sh.source.contains(".transmission"));
        if by_shader { 1.0 } else { p.transmission.clamp(0.0, 1.0) }
    });
    color_alpha * (1.0 - 0.9 * t)
}

fn ptr(t: &TextureImage) -> usize {
    Arc::as_ptr(&t.bytes) as *const u8 as usize
}

/// Per data layer: the image pointers (AO, roughness, metallic or the normal map) and the tile size.
type DataKey = Vec<(usize, usize, usize, [f64; 2])>;

/// Uploads the data textures when the maps change, decoding each image once.
#[derive(Default)]
pub struct DataSync {
    key: Option<DataKey>,
    cache: HashMap<usize, Option<Decoded>>,
}

impl DataSync {
    pub fn changed(&mut self, model: &Model) -> Option<Vec<TextureData>> {
        let layers = data_layers(model);
        let mut key = Vec::new();
        let mut srcs: Vec<(DataKind, [Option<&TextureImage>; 3])> = Vec::new();
        for (id, kind) in &layers {
            let p = model.materials[*id].pbr.as_ref().expect("pbr");
            let maps = match kind {
                DataKind::Orm => [p.ao_map.as_ref(), p.roughness_map.as_ref(), p.metallic_map.as_ref()],
                DataKind::Normal => [p.normal_map.as_ref(), None, None],
            };
            let size = maps.iter().flatten().next().map_or([1000.0; 2], |t| t.size);
            let k = |i: usize| maps[i].map_or(0, ptr);
            key.push((k(0), k(1), k(2), size));
            srcs.push((*kind, maps));
        }
        if self.key.as_ref() == Some(&key) {
            return None;
        }
        let live: Vec<usize> = key.iter().flat_map(|k| [k.0, k.1, k.2]).collect();
        self.cache.retain(|k, _| live.contains(k));
        let mut out = Vec::new();
        for ((kind, maps), k) in srcs.iter().zip(&key) {
            let decoded: Vec<Option<Decoded>> = maps
                .iter()
                .map(|m| {
                    let t = (*m)?;
                    self.cache
                        .entry(ptr(t))
                        .or_insert_with(|| decode(&t.bytes).map_err(|e| log::warn!("map {}: {e}", t.file_name)).ok())
                        .clone()
                })
                .collect();
            let size_mm = [k.3[0] as f32, k.3[1] as f32];
            out.push(match kind {
                DataKind::Normal => match &decoded[0] {
                    Some(d) => TextureData { width: d.0, height: d.1, rgba: d.2.clone(), size_mm },
                    None => TextureData { width: 1, height: 1, rgba: vec![128, 128, 255, 255], size_mm },
                },
                DataKind::Orm => pack_orm(&decoded, size_mm),
            });
        }
        self.key = Some(key);
        Some(out)
    }
}

/// Occlusion from R of the AO map, roughness from G, metalness from B (glTF's packed layout;
/// grey maps have the value in every channel). Missing maps are 1 (the factor alone).
fn pack_orm(maps: &[Option<Decoded>], size_mm: [f32; 2]) -> TextureData {
    let (w, h) = maps.iter().flatten().fold((1, 1), |(w, h), d| (w.max(d.0), h.max(d.1)));
    let mut rgba = vec![255u8; (w * h * 4) as usize];
    for (ch, m) in maps.iter().enumerate() {
        let Some(d) = m else { continue };
        let (mw, mh, px) = &**d;
        for y in 0..h {
            let sy = (y * mh / h).min(mh - 1);
            for x in 0..w {
                let sx = (x * mw / w).min(mw - 1);
                rgba[((y * w + x) * 4) as usize + ch] = px[((sy * mw + sx) * 4) as usize + ch];
            }
        }
    }
    TextureData { width: w, height: h, rgba, size_mm }
}

/// Decode an HDRI (Radiance `.hdr`, or any image the decoder knows, then sRGB → linear).
pub fn decode_hdri(bytes: &[u8]) -> Result<EnvImage, String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let hdr = matches!(img, image::DynamicImage::ImageRgb32F(_) | image::DynamicImage::ImageRgba32F(_));
    let img = if img.width() > 4096 { img.resize(4096, 2048, image::imageops::FilterType::Triangle) } else { img };
    let rgb = img.to_rgb32f();
    let (width, height) = rgb.dimensions();
    let mut data = rgb.into_raw();
    if !hdr {
        for v in &mut data {
            *v = srgb(*v);
        }
    }
    Ok(EnvImage { width, height, rgb: data })
}

/// Hands the document's HDRI to the renderer when it changes (decoded once).
#[derive(Default)]
pub struct EnvSync {
    key: Option<usize>,
}

impl EnvSync {
    /// `Some(image or None)` when the HDRI changed since the last call.
    pub fn changed(&mut self, model: &Model) -> Option<Option<EnvImage>> {
        let key = model.hdri.as_ref().map_or(0, ptr);
        if self.key == Some(key) {
            return None;
        }
        self.key = Some(key);
        Some(model.hdri.as_ref().and_then(|t| decode_hdri(&t.bytes).map_err(|e| log::warn!("HDRI {}: {e}", t.file_name)).ok()))
    }
}

/// The renderer's environment settings from the document's.
/// `sun` = the direction toward the sun (Shadows card), for the physical sky.
pub fn env_style(e: &varstotas_kernel::Environment, sun: [f32; 3]) -> varstotas_render::EnvStyle {
    use varstotas_kernel::{SkyKind, Tonemap};
    varstotas_render::EnvStyle {
        source: match e.sky {
            SkyKind::Sky => varstotas_render::EnvSource::Sky,
            SkyKind::Color => varstotas_render::EnvSource::Color(e.color),
            SkyKind::Hdri => varstotas_render::EnvSource::Image,
            SkyKind::Physical => varstotas_render::EnvSource::Physical { sun, turbidity: e.turbidity },
        },
        rotation_deg: e.rotation_deg,
        intensity: e.intensity,
        background: e.background,
        sun: e.sun,
        exposure: e.exposure,
        tonemap: match e.tonemap {
            Tonemap::Aces => varstotas_render::ToneMap::Aces,
            Tonemap::Agx => varstotas_render::ToneMap::Agx,
            Tonemap::Clamp => varstotas_render::ToneMap::Clamp,
        },
        effects: effects_style(&e.effects),
    }
}

/// The renderer's effects (a switched-off effect gets a zero strength).
pub fn effects_style(f: &varstotas_kernel::Effects) -> varstotas_render::Effects {
    use varstotas_kernel::Stylize;
    let on = |b: bool, v: f32| if b { v } else { 0.0 };
    varstotas_render::Effects {
        soft_shadows: on(f.soft_shadows, f.shadow_softness),
        ao_strength: on(f.ao, f.ao_strength),
        ao_radius: f.ao_radius,
        reflections: on(f.reflections, f.reflection_strength),
        bloom: on(f.bloom, f.bloom_strength),
        bloom_threshold: f.bloom_threshold,
        aperture: on(f.dof, f.aperture),
        focus_distance: f.focus_distance,
        fog_distance: on(f.fog, f.fog_distance),
        fog_color: f.fog_color,
        fog_height: f.fog_height,
        vignette: f.vignette,
        clay: f.clay,
        stylize: match f.stylize {
            Stylize::None => varstotas_render::Stylize::None,
            Stylize::Pencil => varstotas_render::Stylize::Pencil,
            Stylize::Watercolor => varstotas_render::Stylize::Watercolor,
            Stylize::Technical => varstotas_render::Stylize::Technical,
        },
        high_quality: f.high_quality,
        time: 0.0,
    }
}

/// Some custom surface of the model changes with time (redraw continuously in Realistic).
pub fn animated(model: &Model) -> bool {
    pbr_materials(model).iter().any(|id| {
        model.materials[*id].pbr.as_ref().and_then(|p| p.shader.as_ref()).is_some_and(|s| {
            s.source.lines().map(|l| l.split("//").next().unwrap_or("")).any(|l| l.contains(".time"))
        })
    })
}

/// How a render's samples become its image: the environment's exposure and tone curve with the
/// Render window's grading and denoiser.
pub fn pt_output(model: &Model) -> varstotas_render::PtOutput {
    let (e, r) = (&model.environment, &model.render);
    let tonemap = match r.tonemap.unwrap_or(e.tonemap) {
        varstotas_kernel::Tonemap::Aces => 0,
        varstotas_kernel::Tonemap::Agx => 1,
        varstotas_kernel::Tonemap::Clamp => 2,
    };
    varstotas_render::PtOutput {
        exposure: varstotas_render::exposure_mult(e.exposure + r.exposure),
        tonemap,
        white: varstotas_render::white_balance(r.temperature, r.tint),
        saturation: r.saturation,
        contrast: r.contrast,
        vignette: r.vignette,
        ai_denoise: r.ai_denoise,
    }
}

/// The model's enabled lights for the path tracer (linear colours).
pub fn pt_lights(model: &Model) -> Vec<varstotas_render::PtLight> {
    use varstotas_kernel::LightKind;
    model
        .lights
        .iter()
        .filter(|l| l.enabled)
        .map(|l| varstotas_render::PtLight {
            kind: match l.kind {
                LightKind::Point => varstotas_render::PtLightKind::Point,
                LightKind::Spot => varstotas_render::PtLightKind::Spot,
                LightKind::Area => varstotas_render::PtLightKind::Area,
            },
            position: l.position.to_array(),
            direction: l.direction.as_vec3().to_array(),
            color: l.color.map(srgb),
            power: l.power,
            radius: l.radius,
            spot_angle: l.spot_angle,
            spot_blend: l.spot_blend,
            size: l.size,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use varstotas_kernel::Pbr;

    fn png(w: u32, h: u32, px: [u8; 4]) -> Vec<u8> {
        let img = image::RgbaImage::from_fn(w, h, |_, _| image::Rgba(px));
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png).unwrap();
        out.into_inner()
    }

    #[test]
    fn table_layers_and_packing() {
        let mut m = Model::new();
        let plain = m.add_material("Plain", [1.0; 4]);
        let metal = m.add_material("Metal", [0.8, 0.8, 0.8, 1.0]);
        m.materials[metal].pbr = Some(Pbr { metallic: 1.0, roughness: 0.2, ..Default::default() });
        let lamp = m.add_material("Lamp", [1.0; 4]);
        let mut p = Pbr { emission: [1.0, 1.0, 1.0], emission_strength: 4.0, ..Default::default() };
        p.emission_map = Some(TextureImage::new(png(2, 2, [255, 0, 0, 255]), "e.png", 500.0));
        p.roughness_map = Some(TextureImage::new(png(4, 4, [10, 128, 30, 255]), "r.png", 500.0));
        p.normal_map = Some(TextureImage::new(png(2, 2, [128, 128, 255, 255]), "n.png", 500.0));
        m.materials[lamp].pbr = Some(p);
        assert_eq!(pbr_materials(&m), vec![metal, lamp]);
        assert!(!pbr_materials(&m).contains(&plain));
        let t = material_table(&m);
        assert_eq!((t[0].metallic, t[0].roughness, t[0].orm_layer), (1.0, 0.2, 0));
        assert_eq!((t[1].emission, t[1].emission_layer), ([4.0; 3], 1));
        assert_eq!((t[1].orm_layer, t[1].orm_mask, t[1].normal_layer), (1, 2, 2));
        let mut s = DataSync::default();
        let d = s.changed(&m).unwrap();
        assert_eq!(d.len(), 2);
        // roughness comes from G; occlusion / metalness default to 1
        assert_eq!((d[0].width, &d[0].rgba[..4]), (4, &[255, 128, 255, 255][..]));
        assert!(s.changed(&m).is_none());
        let hdr = decode_hdri(include_bytes!("../../../qa/models/studio.hdr")).unwrap();
        assert_eq!((hdr.width, hdr.height, hdr.rgb.len()), (256, 128, 256 * 128 * 3));
        assert!(hdr.rgb.iter().any(|v| *v > 5.0), "HDR values above 1 survive");
        assert!((realistic_alpha(1.0, Some(&Pbr { transmission: 1.0, ..Default::default() })) - 0.1).abs() < 1e-6);
    }
}
