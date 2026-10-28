//! Sphere thumbnails of physical (PBR) materials, rendered on the CPU: base colour and
//! texture, metalness, roughness, glass and emission under one light and a sky gradient.

use egui::{Color32, ColorImage, TextureHandle};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use varstotas_kernel::Material;
use varstotas_kernel::glam::Vec3;

/// Thumbnail side in pixels.
const SIDE: usize = 96;

thread_local! {
    static CACHE: RefCell<HashMap<u64, TextureHandle>> = RefCell::new(HashMap::new());
}

/// The material looks physical enough for a sphere (else the flat swatch / texture tile).
pub fn wants_sphere(m: &Material) -> bool {
    m.pbr.as_ref().is_some_and(|p| {
        p.metallic > 0.05 || p.roughness < 0.85 || p.transmission > 0.05 || (p.emission_strength > 0.0 && p.emission.iter().any(|c| *c > 0.0))
    })
}

/// What the picture depends on (cheap: the image by its bytes pointer).
fn key(m: &Material) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    m.color.map(f32::to_bits).hash(&mut h);
    m.image.as_ref().map(|i| (std::sync::Arc::as_ptr(&i.bytes) as *const u8 as usize, i.look.hue.to_bits(), i.look.saturation.to_bits(), i.look.lightness.to_bits(), i.look.colorize)).hash(&mut h);
    let p = m.pbr_or_default();
    [p.metallic, p.roughness, p.transmission, p.emission_strength, p.emission[0], p.emission[1], p.emission[2], p.clearcoat].map(f32::to_bits).hash(&mut h);
    h.finish()
}

/// The sphere thumbnail of `m` (rendered once per look).
pub fn sphere_texture(ctx: &egui::Context, m: &Material) -> egui::TextureId {
    let k = key(m);
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if let Some(t) = c.get(&k) {
            return t.id();
        }
        if c.len() > 512 {
            c.clear();
        }
        let t = ctx.load_texture(format!("mat_sphere_{k:x}"), render(m), egui::TextureOptions::LINEAR);
        let id = t.id();
        c.insert(k, t);
        id
    })
}

fn to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
}

/// The backdrop: a soft vertical gradient with a faint checker, so glass shows.
fn backdrop(x: f32, y: f32) -> Vec3 {
    let g = 0.55 - 0.25 * y;
    let check = ((x * 6.0).floor() as i32 + (y * 6.0).floor() as i32).rem_euclid(2) as f32;
    Vec3::splat(varstotas_render::srgb_to_linear(g + 0.04 * check))
}

/// Sky seen along a direction (y up): bright above, darker ground.
fn sky(d: Vec3) -> Vec3 {
    let t = d.y * 0.5 + 0.5;
    Vec3::new(0.18, 0.17, 0.16).lerp(Vec3::new(0.95, 0.97, 1.0), t.clamp(0.0, 1.0).powf(0.8))
}

/// Render the sphere (2 × 2 samples per pixel).
pub fn render(m: &Material) -> ColorImage {
    let p = m.pbr_or_default();
    let tint = Vec3::new(varstotas_render::srgb_to_linear(m.color[0]), varstotas_render::srgb_to_linear(m.color[1]), varstotas_render::srgb_to_linear(m.color[2]));
    let tex = m.image.as_ref().and_then(|i| {
        let d = crate::textures::decode(&i.bytes).ok()?;
        let mut rgba = d.2.clone();
        i.look.apply(&mut rgba, [m.color[0], m.color[1], m.color[2]]);
        Some((d.0 as usize, d.1 as usize, rgba))
    });
    let sample = |u: f32, v: f32| -> Vec3 {
        match &tex {
            Some((w, h, px)) => {
                let x = ((u.rem_euclid(1.0) * *w as f32) as usize).min(w - 1);
                let y = ((v.clamp(0.0, 1.0) * *h as f32) as usize).min(h - 1);
                let i = (y * w + x) * 4;
                Vec3::new(varstotas_render::srgb_to_linear(px[i] as f32 / 255.0), varstotas_render::srgb_to_linear(px[i + 1] as f32 / 255.0), varstotas_render::srgb_to_linear(px[i + 2] as f32 / 255.0))
            }
            None => tint,
        }
    };
    let light = Vec3::new(-0.5, 0.7, 0.6).normalize();
    let rough = p.roughness.clamp(0.03, 1.0);
    let shine = 2.0 / (rough * rough * rough * rough) - 2.0;
    let emission = Vec3::from(p.emission) * p.emission_strength.min(4.0);
    let shade = |x: f32, y: f32| -> Vec3 {
        // sphere of radius 0.42 in the middle of the unit square, y down
        let (sx, sy) = ((x - 0.5) / 0.42, -(y - 0.5) / 0.42);
        let r2 = sx * sx + sy * sy;
        if r2 > 1.0 {
            // the sphere's shadow on the backdrop
            let d = ((x - 0.56).powi(2) + ((y - 0.94) * 4.0).powi(2)).sqrt();
            return backdrop(x, y) * (1.0 - 0.35 * (1.0 - d / 0.4).clamp(0.0, 1.0));
        }
        let n = Vec3::new(sx, sy, (1.0 - r2).sqrt());
        let v = Vec3::Z;
        // texture: longitude / latitude, two repeats round the sphere
        let u = n.x.atan2(n.z) / std::f32::consts::TAU + 0.5;
        let base = sample(u * 2.0, n.y.acos() / std::f32::consts::PI);
        let ndl = n.dot(light).max(0.0);
        let refl = (2.0 * n.dot(v) * n - v).normalize();
        let ndv = n.dot(v).max(0.0);
        let f0 = Vec3::splat(0.04).lerp(base, p.metallic);
        let fresnel = f0 + (Vec3::ONE - f0) * (1.0 - ndv).powi(5) * (1.0 - rough).powi(2);
        // rough surfaces see a blurred sky: towards its average
        let env = sky(refl).lerp(Vec3::splat(0.5), rough * 0.85);
        let highlight = refl.dot(light).max(0.0).powf(shine) * (shine + 2.0) / 8.0 * ndl.min(1.0);
        let spec = fresnel * (env + Vec3::splat(highlight.min(12.0)));
        let diffuse = base * (1.0 - p.metallic) * (0.22 + 0.78 * ndl) * 0.9;
        let mut c = diffuse * (Vec3::ONE - fresnel) + spec;
        if p.transmission > 0.0 {
            // what is behind, bent towards the centre and tinted
            let behind = backdrop(x - n.x * 0.12, y + n.y * 0.12) * base.lerp(Vec3::ONE, 0.3);
            c = c.lerp(behind + spec, p.transmission);
        }
        if p.clearcoat > 0.0 {
            c += Vec3::splat(refl.dot(light).max(0.0).powf(200.0) * 4.0 * p.clearcoat + 0.04 * p.clearcoat) * sky(refl);
        }
        c + emission * base.lerp(Vec3::ONE, 0.5)
    };
    let mut pixels = Vec::with_capacity(SIDE * SIDE);
    for py in 0..SIDE {
        for px in 0..SIDE {
            let mut acc = Vec3::ZERO;
            for (ox, oy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                acc += shade((px as f32 + ox) / SIDE as f32, (py as f32 + oy) / SIDE as f32);
            }
            // a gentle shoulder instead of a hard clip
            let c = acc / 4.0;
            let c = c / (Vec3::ONE + c * 0.25) * 1.25;
            pixels.push(Color32::from_rgb((to_srgb(c.x) * 255.0) as u8, (to_srgb(c.y) * 255.0) as u8, (to_srgb(c.z) * 255.0) as u8));
        }
    }
    ColorImage { size: [SIDE, SIDE], pixels, source_size: egui::vec2(SIDE as f32, SIDE as f32) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spheres_differ_by_material() {
        let mut m = Material { name: "a".into(), color: [0.8, 0.2, 0.1, 1.0], texture: None, image: None, pbr: None, stock: Vec::new() };
        assert!(!wants_sphere(&m));
        m.pbr = Some(varstotas_kernel::Pbr { metallic: 1.0, roughness: 0.2, ..Default::default() });
        assert!(wants_sphere(&m));
        let metal = render(&m);
        m.pbr = Some(varstotas_kernel::Pbr { metallic: 0.0, roughness: 0.9, ..Default::default() });
        let matte = render(&m);
        assert_eq!(metal.pixels.len(), SIDE * SIDE);
        // the corner is the backdrop in both; the middle differs
        assert_eq!(metal.pixels[0], matte.pixels[0]);
        let mid = SIDE * SIDE / 2 + SIDE / 2;
        assert_ne!(metal.pixels[mid], matte.pixels[mid]);
    }
}
