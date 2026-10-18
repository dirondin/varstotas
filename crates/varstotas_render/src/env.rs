//! Environment light of `FaceStyle::Realistic`: an equirectangular Rgba16Float panorama with a
//! CPU mip chain (rough reflections read blurrier mips) and its diffuse irradiance as 9
//! spherical-harmonic coefficients. Bound in group 0 (bindings 3..5) next to the globals.

use crate::wgpu;
use crate::{EnvImage, EnvSource, EnvStyle, RenderStyle, ToneMap};

/// Scene light × this = tone-mapper input at exposure 0 (the usual ACES pre-scale).
const EXPOSURE_SCALE: f32 = 0.6;

/// Widest base level of the panorama (larger images are box-downsampled).
const MAX_WIDTH: u32 = 2048;

/// Uniform block, mirrors `EnvU` in shader.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct EnvUniform {
    /// Irradiance (already convolved with the cosine lobe), rgb per SH basis function.
    sh: [[f32; 4]; 9],
    /// x: rotation (rad); y: intensity; z: last mip level; w: 1 = environment as background.
    p: [f32; 4],
    /// x: exposure multiplier; y: tone map (0 ACES, 1 AgX, 2 clamp); z: sun strength;
    /// w: time (s).
    p2: [f32; 4],
    /// x: 1 = clay; y: soft shadow penumbra per shadow-map depth (0 = off); z: 1 = high quality.
    p3: [f32; 4],
}

/// What the current panorama was built from.
#[derive(Clone, Copy, PartialEq)]
enum SourceKey {
    Sky([u32; 9]),
    Color([u32; 3]),
    Image(u64),
    Physical([u32; 5]),
}

pub(crate) struct Env {
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub uniform: wgpu::Buffer,
    key: Option<SourceKey>,
    /// The panorama is the Sky source (its background is the style's own gradient).
    sky: bool,
    sh: [[f32; 4]; 9],
    mips: u32,
    image: Option<EnvImage>,
    image_rev: u64,
}

impl Env {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("varstotas environment"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("varstotas environment"),
            size: std::mem::size_of::<EnvUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let grey = Panorama { width: 2, height: 1, rgb: vec![0.5; 6] };
        let (view, mips) = upload(device, queue, &grey);
        Self { view, sampler, uniform, key: None, sky: true, sh: [[0.0; 4]; 9], mips, image: None, image_rev: 0 }
    }

    pub fn set_image(&mut self, image: Option<EnvImage>) {
        if self.image != image {
            self.image = image;
            self.image_rev += 1;
        }
    }

    /// Rebuild the panorama if the style's source changed; true = the view was replaced (the
    /// globals bind group must be rebuilt).
    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, style: &RenderStyle) -> bool {
        let source = match style.env.source {
            EnvSource::Image if self.image.is_some() => EnvSource::Image,
            EnvSource::Image => EnvSource::Sky,
            s => s,
        };
        let bits = |c: &[f32]| c.iter().map(|v| v.to_bits()).collect::<Vec<_>>();
        let key = match source {
            EnvSource::Sky => {
                let b = bits(&[&style.sky_top[..3], &style.sky_horizon[..3], &style.ground[..3]].concat());
                SourceKey::Sky(std::array::from_fn(|i| b[i]))
            }
            EnvSource::Color(c) => SourceKey::Color(std::array::from_fn(|i| c[i].to_bits())),
            EnvSource::Image => SourceKey::Image(self.image_rev),
            EnvSource::Physical { sun, turbidity } => {
                // Rebuilt when the sun moves by more than about 0.3°.
                let q = |v: f32| ((v * 200.0).round() as i32) as u32;
                SourceKey::Physical([q(sun[0]), q(sun[1]), q(sun[2]), q(turbidity), style.env.rotation_deg.to_bits()])
            }
        };
        self.sky = source == EnvSource::Sky;
        if self.key == Some(key) {
            return false;
        }
        self.key = Some(key);
        let pano = match source {
            EnvSource::Sky => sky_panorama(style),
            EnvSource::Color(c) => {
                let l = srgb3(c);
                Panorama { width: 2, height: 1, rgb: [l, l].concat() }
            }
            EnvSource::Image => {
                let img = self.image.as_ref().expect("image");
                Panorama::from_image(img)
            }
            EnvSource::Physical { sun, turbidity } => {
                // The panorama is turned with the environment, the sun stays where it is.
                let r = -style.env.rotation_deg.to_radians();
                let (c, s) = (r.cos(), r.sin());
                let sun = [c * sun[0] - s * sun[1], s * sun[0] + c * sun[1], sun[2]];
                physical_sky(sun, turbidity)
            }
        };
        self.sh = irradiance_sh(&pano);
        let (view, mips) = upload(device, queue, &pano);
        self.view = view;
        self.mips = mips;
        true
    }

    /// The environment itself is drawn behind the model (else the style's gradient).
    pub fn shows_background(&self, env: &EnvStyle) -> bool {
        env.background && !self.sky
    }

    /// `shadow_depth_over_width` = depth range / side of the shadow map's box (world).
    pub fn write_uniform(&self, queue: &wgpu::Queue, env: &EnvStyle, shadow_depth_over_width: f32) {
        let fx = &env.effects;
        let u = EnvUniform {
            sh: self.sh,
            p: [
                env.rotation_deg.to_radians(),
                env.intensity.max(0.0),
                (self.mips - 1) as f32,
                // The Sky source shows the style's gradient as is (not tone mapped).
                if env.background && !self.sky { 1.0 } else { 0.0 },
            ],
            p2: [
                exposure_mult(env.exposure),
                match env.tonemap {
                    ToneMap::Aces => 0.0,
                    ToneMap::Agx => 1.0,
                    ToneMap::Clamp => 2.0,
                },
                env.sun.max(0.0),
                fx.time,
            ],
            p3: [
                if fx.clay { 1.0 } else { 0.0 },
                // Penumbra width (map uv) per unit of normalized map depth between the blocker
                // and the receiver: a sun about 1.2° across at softness 1.
                fx.soft_shadows.max(0.0) * 0.01 * shadow_depth_over_width,
                if fx.high_quality { 1.0 } else { 0.0 },
                0.0,
            ],
        };
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&u));
    }
}

/// The light multiplier of an exposure in stops.
pub fn exposure_mult(ev: f32) -> f32 {
    EXPOSURE_SCALE * 2f32.powf(ev.clamp(-20.0, 20.0))
}

fn srgb(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
}

fn srgb3(c: [f32; 3]) -> [f32; 3] {
    [srgb(c[0]), srgb(c[1]), srgb(c[2])]
}

/// Linear RGB panorama, rows top (up) to bottom.
struct Panorama {
    width: u32,
    height: u32,
    rgb: Vec<f32>,
}

impl Panorama {
    fn from_image(img: &EnvImage) -> Self {
        let (w, h) = (img.width.max(1), img.height.max(1));
        let ok = img.rgb.len() >= (w * h * 3) as usize;
        let mut p = if ok {
            Self { width: w, height: h, rgb: img.rgb[..(w * h * 3) as usize].iter().map(|v| v.clamp(0.0, 65000.0)).collect() }
        } else {
            Self { width: 2, height: 1, rgb: vec![0.5; 6] }
        };
        for v in &mut p.rgb {
            if !v.is_finite() {
                *v = 0.0;
            }
        }
        while p.width > MAX_WIDTH && p.height > 1 {
            p = p.halved();
        }
        p
    }

    /// 2×2 box filter (odd edges are dropped; at least 1×1).
    fn halved(&self) -> Self {
        let (w, h) = ((self.width / 2).max(1), (self.height / 2).max(1));
        let mut rgb = vec![0f32; (w * h * 3) as usize];
        for y in 0..h {
            for x in 0..w {
                for c in 0..3 {
                    let mut s = 0.0;
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let sx = (x * 2 + dx).min(self.width - 1);
                        let sy = (y * 2 + dy).min(self.height - 1);
                        s += self.rgb[((sy * self.width + sx) * 3 + c) as usize];
                    }
                    rgb[((y * w + x) * 3 + c) as usize] = s * 0.25;
                }
            }
        }
        Self { width: w, height: h, rgb }
    }

    /// Direction of the texel centre (x, y): see `EnvImage`.
    fn dir(&self, x: u32, y: u32) -> [f32; 3] {
        let u = (x as f32 + 0.5) / self.width as f32;
        let v = (y as f32 + 0.5) / self.height as f32;
        let phi = u * std::f32::consts::TAU - std::f32::consts::PI;
        let theta = v * std::f32::consts::PI;
        [theta.sin() * phi.cos(), theta.sin() * phi.sin(), theta.cos()]
    }
}

/// The Sky source: the style's gradient above the horizon (like the Shaded sky), a dimmed
/// ground colour below it. Its colours light the model; the background stays the style's.
fn sky_panorama(style: &RenderStyle) -> Panorama {
    let (w, h) = (256u32, 128u32);
    let sky = |c: [f32; 4]| srgb3([c[0], c[1], c[2]]);
    let (top, hor, gnd) = (sky(style.sky_top), sky(style.sky_horizon), sky(style.ground));
    let mut p = Panorama { width: w, height: h, rgb: Vec::with_capacity((w * h * 3) as usize) };
    for y in 0..h {
        let z = p.dir(0, y)[2];
        let c: [f32; 3] = if z >= 0.0 {
            let t = (z / 0.6).clamp(0.0, 1.0).sqrt();
            std::array::from_fn(|k| hor[k] + (top[k] - hor[k]) * t)
        } else {
            // a soft transition into the (darker, it is lit by the sky only) ground
            let t = (-z / 0.05).clamp(0.0, 1.0);
            std::array::from_fn(|k| hor[k] + (gnd[k] * 0.45 - hor[k]) * t)
        };
        for _ in 0..w {
            p.rgb.extend_from_slice(&c);
        }
    }
    p
}

/// Preetham et al. 1999 daylight: Perez distributions of luminance and chromaticity around
/// the sun, scaled so a high sun gives a sky about as bright as the style's. Below the
/// horizon: a dim ground. A sun under the horizon leaves a fading dusk sky.
fn physical_sky(sun: [f32; 3], t: f32) -> Panorama {
    let (w, h) = (256u32, 128u32);
    let len = (sun[0] * sun[0] + sun[1] * sun[1] + sun[2] * sun[2]).sqrt().max(1e-6);
    let mut s = [sun[0] / len, sun[1] / len, sun[2] / len];
    let dusk = ((s[2] + 0.1) / 0.15).clamp(0.0, 1.0);
    if s[2] < 0.02 {
        let k = ((1.0 - 0.02f32 * 0.02) / (s[0] * s[0] + s[1] * s[1]).max(1e-9)).sqrt();
        s = [s[0] * k, s[1] * k, 0.02];
    }
    let theta_s = s[2].clamp(-1.0, 1.0).acos();
    let perez = |c: [f32; 5], cos_t: f32, gamma: f32| (1.0 + c[0] * (c[1] / cos_t.max(0.01)).exp()) * (1.0 + c[2] * (c[3] * gamma).exp() + c[4] * gamma.cos().powi(2));
    let cy = [0.1787 * t - 1.4630, -0.3554 * t + 0.4275, -0.0227 * t + 5.3251, 0.1206 * t - 2.5771, -0.0670 * t + 0.3703];
    let cx = [-0.0193 * t - 0.2592, -0.0665 * t + 0.0008, -0.0004 * t + 0.2125, -0.0641 * t - 0.8989, -0.0033 * t + 0.0452];
    let cyy = [-0.0167 * t - 0.2608, -0.0950 * t + 0.0092, -0.0079 * t + 0.2102, -0.0441 * t - 1.6537, -0.0109 * t + 0.0529];
    let chi = (4.0 / 9.0 - t / 120.0) * (std::f32::consts::PI - 2.0 * theta_s);
    let zy = ((4.0453 * t - 4.9710) * chi.tan() - 0.2155 * t + 2.4192).max(0.05);
    let th = [theta_s.powi(3), theta_s.powi(2), theta_s, 1.0];
    let dot4 = |a: [f32; 4]| a[0] * th[0] + a[1] * th[1] + a[2] * th[2] + a[3] * th[3];
    let zx = t * t * dot4([0.00166, -0.00375, 0.00209, 0.0]) + t * dot4([-0.02903, 0.06377, -0.03202, 0.00394]) + dot4([0.11693, -0.21196, 0.06052, 0.25886]);
    let zyc = t * t * dot4([0.00275, -0.00610, 0.00317, 0.0]) + t * dot4([-0.04214, 0.08970, -0.04153, 0.00516]) + dot4([0.15346, -0.26756, 0.06670, 0.26688]);
    let (ny, nx, nyc) = (perez(cy, 1.0, theta_s), perez(cx, 1.0, theta_s), perez(cyy, 1.0, theta_s));
    // kcd/m² → scene units (a white overcast sky ≈ 1), dimmed at dusk.
    let scale = 0.12 * (0.04 + 0.96 * dusk);
    let rgb_of = |d: [f32; 3]| -> [f32; 3] {
        let cos_t = d[2].max(0.0);
        let gamma = (d[0] * s[0] + d[1] * s[1] + d[2] * s[2]).clamp(-1.0, 1.0).acos();
        let yv = zy * perez(cy, cos_t, gamma) / ny;
        let x = zx * perez(cx, cos_t, gamma) / nx;
        let y = (zyc * perez(cyy, cos_t, gamma) / nyc).max(1e-4);
        let (xx, zz) = (x / y * yv, (1.0 - x - y) / y * yv);
        let r = 3.2406 * xx - 1.5372 * yv - 0.4986 * zz;
        let g = -0.9689 * xx + 1.8758 * yv + 0.0415 * zz;
        let b = 0.0557 * xx - 0.2040 * yv + 1.0570 * zz;
        [r.max(0.0) * scale, g.max(0.0) * scale, b.max(0.0) * scale]
    };
    let mut p = Panorama { width: w, height: h, rgb: Vec::with_capacity((w * h * 3) as usize) };
    for y in 0..h {
        for x in 0..w {
            let d = p.dir(x, y);
            let c = if d[2] >= 0.0 {
                rgb_of(d)
            } else {
                // The horizon's colour fading into a dim ground lit by the sky.
                let hz = rgb_of([d[0], d[1], 0.0]);
                let k = (-d[2] / 0.05).clamp(0.0, 1.0);
                let g = [0.3 * scale * 2.5 * (0.2 + 0.8 * s[2].max(0.0)); 3];
                std::array::from_fn(|i| hz[i] + (g[i] - hz[i]) * k)
            };
            p.rgb.extend_from_slice(&c);
        }
    }
    p
}

/// Real SH basis (l ≤ 2) for unit `d`.
pub(crate) fn sh_basis(d: [f32; 3]) -> [f32; 9] {
    let [x, y, z] = d;
    [
        0.282095,
        0.488603 * y,
        0.488603 * z,
        0.488603 * x,
        1.092548 * x * y,
        1.092548 * y * z,
        0.315392 * (3.0 * z * z - 1.0),
        1.092548 * x * z,
        0.546274 * (x * x - y * y),
    ]
}

/// Irradiance E(n) = Σ sh[i] · Y_i(n) of the panorama (radiance projected on SH, convolved
/// with the clamped cosine).
fn irradiance_sh(pano: &Panorama) -> [[f32; 4]; 9] {
    let mut p = Panorama { width: pano.width, height: pano.height, rgb: pano.rgb.clone() };
    while p.width > 128 && p.height > 1 {
        p = p.halved();
    }
    let mut acc = [[0f64; 3]; 9];
    let dphi = std::f64::consts::TAU / p.width as f64;
    let dtheta = std::f64::consts::PI / p.height as f64;
    for y in 0..p.height {
        let theta = (y as f64 + 0.5) * dtheta;
        let dw = theta.sin() * dphi * dtheta;
        for x in 0..p.width {
            let b = sh_basis(p.dir(x, y));
            let i = ((y * p.width + x) * 3) as usize;
            for (a, bk) in acc.iter_mut().zip(&b) {
                for (c, ac) in a.iter_mut().enumerate() {
                    *ac += p.rgb[i + c] as f64 * *bk as f64 * dw;
                }
            }
        }
    }
    let band = |k: usize| match k {
        0 => std::f64::consts::PI,
        1..=3 => std::f64::consts::TAU / 3.0,
        _ => std::f64::consts::FRAC_PI_4,
    };
    std::array::from_fn(|k| {
        let a = band(k);
        [(acc[k][0] * a) as f32, (acc[k][1] * a) as f32, (acc[k][2] * a) as f32, 0.0]
    })
}

/// f32 → IEEE half (round to nearest, clamped to the finite range).
fn f16_bits(v: f32) -> u16 {
    let v = v.clamp(-65000.0, 65000.0);
    let b = v.to_bits();
    let sign = ((b >> 16) & 0x8000) as u16;
    let exp = ((b >> 23) & 0xff) as i32 - 127 + 15;
    let man = b & 0x7f_ffff;
    if exp <= 0 {
        if exp < -10 {
            return sign;
        }
        let m = (man | 0x80_0000) >> (1 - exp);
        return sign | ((m + 0x1000) >> 13) as u16;
    }
    let h = ((exp as u32) << 10) | (man >> 13);
    let round = (man >> 12) & 1;
    sign | (h + round) as u16
}

fn upload(device: &wgpu::Device, queue: &wgpu::Queue, pano: &Panorama) -> (wgpu::TextureView, u32) {
    let mips = pano.width.max(pano.height).ilog2() + 1;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("varstotas environment"),
        size: wgpu::Extent3d { width: pano.width, height: pano.height, depth_or_array_layers: 1 },
        mip_level_count: mips,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let mut level = Panorama { width: pano.width, height: pano.height, rgb: pano.rgb.clone() };
    for mip in 0..mips {
        let mut data = Vec::with_capacity((level.width * level.height * 4) as usize);
        for px in level.rgb.as_chunks::<3>().0 {
            data.extend([f16_bits(px[0]), f16_bits(px[1]), f16_bits(px[2]), f16_bits(1.0)]);
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: mip, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            bytemuck::cast_slice(&data),
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(8 * level.width), rows_per_image: Some(level.height) },
            wgpu::Extent3d { width: level.width, height: level.height, depth_or_array_layers: 1 },
        );
        if mip + 1 < mips {
            level = level.halved();
        }
    }
    (texture.create_view(&Default::default()), mips)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_sky_is_blue_and_dims_at_dusk() {
        let at = |p: &Panorama, d: [f32; 3]| {
            let (mut best, mut i) = (f32::MAX, 0);
            for y in 0..p.height {
                for x in 0..p.width {
                    let q = p.dir(x, y);
                    let e = (q[0] - d[0]).powi(2) + (q[1] - d[1]).powi(2) + (q[2] - d[2]).powi(2);
                    if e < best {
                        best = e;
                        i = (y * p.width + x) as usize * 3;
                    }
                }
            }
            [p.rgb[i], p.rgb[i + 1], p.rgb[i + 2]]
        };
        let noon = physical_sky([0.3, -0.5, 0.8], 3.0);
        let z = at(&noon, [0.0, 0.0, 1.0]);
        eprintln!("zenith {z:?} horizon {:?}", at(&noon, [-1.0, 0.0, 0.05]));
        assert!(z[2] > z[0] && z[2] > 0.3, "{z:?}");
        let dusk = physical_sky([0.3, -0.9, -0.05], 3.0);
        assert!(at(&dusk, [0.0, 0.0, 1.0])[2] < z[2] * 0.2);
    }

    #[test]
    fn half_floats() {
        assert_eq!(f16_bits(0.0), 0);
        assert_eq!(f16_bits(1.0), 0x3c00);
        assert_eq!(f16_bits(-2.0), 0xc000);
        assert_eq!(f16_bits(0.5), 0x3800);
        assert_eq!(f16_bits(1e-5), 0x00a8);
        assert_eq!(f16_bits(1e9), f16_bits(65000.0));
    }

    #[test]
    fn constant_environment_irradiance() {
        // Uniform radiance L: irradiance is π·L in every direction.
        let p = Panorama { width: 64, height: 32, rgb: vec![0.5; 64 * 32 * 3] };
        let sh = irradiance_sh(&p);
        for n in [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, -0.6, -0.8]] {
            let b = sh_basis(n);
            let e: f32 = (0..9).map(|k| sh[k][0] * b[k]).sum();
            assert!((e - std::f32::consts::PI * 0.5).abs() < 0.02, "{e}");
        }
        // sky above, black below: up-facing gets more light than down-facing
        let mut q = Panorama { width: 64, height: 32, rgb: vec![0.0; 64 * 32 * 3] };
        for i in 0..64 * 16 * 3 {
            q.rgb[i] = 1.0;
        }
        let sh = irradiance_sh(&q);
        let e = |n| (0..9).map(|k| sh[k][1] * sh_basis(n)[k]).sum::<f32>();
        assert!((e([0.0, 0.0, 1.0]) - std::f32::consts::PI).abs() < 0.15);
        assert!(e([0.0, 0.0, -1.0]).abs() < 0.15);
    }
}
