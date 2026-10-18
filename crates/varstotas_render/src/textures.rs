//! Material textures for faces: a colour (sRGB) and a data (linear) `texture_2d_array` (every
//! image resampled to a common square layer size, CPU mip chain), a repeating sampler, a
//! per-layer info uniform each (1 / tile size in mm) and the PBR material table. Bound as
//! group 1 of the face pipelines.

use crate::TextureData;
use crate::wgpu;

/// Most layers kept (the uniform info array has this many entries; WebGPU guarantees 256).
pub const MAX_LAYERS: usize = 256;
/// Largest layer side, px.
const MAX_SIDE: u32 = 1024;

pub(crate) struct FaceTextures {
    pub bgl: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    info: wgpu::Buffer,
    data_info: wgpu::Buffer,
    /// PBR material table: 6 × vec4 per material (see `set_materials`).
    materials: wgpu::Buffer,
    color_view: wgpu::TextureView,
    data_view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    max_layers: u32,
    /// Colour layer i has cut-out texels; material i is glass (the path tracer's ray queries
    /// test those triangles in the shader).
    pub cut: Vec<bool>,
    pub glass: Vec<bool>,
}

/// Most PBR materials in the table (6 × 16 bytes each: a 24 KB uniform).
pub const MAX_MATERIALS: usize = 256;
/// vec4s per material in the table (`materials` in shader.wgsl).
const MATERIAL_VEC4S: usize = 6;

impl FaceTextures {
    pub fn new(device: &wgpu::Device) -> Self {
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE, ty, count: None };
        let array = wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2Array,
            multisampled: false,
        };
        let uniform =
            wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None };
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("varstotas face textures"),
            entries: &[
                entry(0, array),
                entry(1, wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)),
                entry(2, uniform),
                entry(3, array),
                entry(4, uniform),
                entry(5, uniform),
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("varstotas face textures"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 8,
            ..Default::default()
        });
        let buffer = |label, size: usize| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: size as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let info = buffer("varstotas texture info", MAX_LAYERS * 16);
        let data_info = buffer("varstotas data texture info", MAX_LAYERS * 16);
        let materials = buffer("varstotas pbr materials", MAX_MATERIALS * MATERIAL_VEC4S * 16);
        let max_layers = device.limits().max_texture_array_layers.min(MAX_LAYERS as u32);
        let color_view = Self::array_view(&Self::create_texture(device, 1, 1, 1, wgpu::TextureFormat::Rgba8UnormSrgb));
        let data_view = Self::array_view(&Self::create_texture(device, 1, 1, 1, wgpu::TextureFormat::Rgba8Unorm));
        let mut t = Self {
            bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("varstotas face textures (empty)"),
                layout: &bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&color_view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                    wgpu::BindGroupEntry { binding: 2, resource: info.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&data_view) },
                    wgpu::BindGroupEntry { binding: 4, resource: data_info.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 5, resource: materials.as_entire_binding() },
                ],
            }),
            bgl,
            sampler,
            info,
            data_info,
            materials,
            color_view,
            data_view,
            max_layers,
            cut: Vec::new(),
            glass: Vec::new(),
        };
        t.rebind(device);
        t
    }

    fn create_texture(device: &wgpu::Device, side: u32, layers: u32, mips: u32, format: wgpu::TextureFormat) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("varstotas face textures"),
            size: wgpu::Extent3d { width: side, height: side, depth_or_array_layers: layers.max(1) },
            mip_level_count: mips,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }

    fn array_view(texture: &wgpu::Texture) -> wgpu::TextureView {
        texture.create_view(&wgpu::TextureViewDescriptor { dimension: Some(wgpu::TextureViewDimension::D2Array), ..Default::default() })
    }

    fn rebind(&mut self, device: &wgpu::Device) {
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("varstotas face textures"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&self.color_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: self.info.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&self.data_view) },
                wgpu::BindGroupEntry { binding: 4, resource: self.data_info.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 5, resource: self.materials.as_entire_binding() },
            ],
        });
    }

    /// Replace the colour textures (sRGB); layer `i` (vertex layer index `i + 1`) is `images[i]`.
    pub fn set(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, images: &[TextureData]) {
        self.cut = images.iter().map(|img| img.rgba.iter().skip(3).step_by(4).any(|&a| a < 128)).collect();
        self.color_view = self.upload(device, queue, images, wgpu::TextureFormat::Rgba8UnormSrgb, false);
        self.rebind(device);
    }

    /// Replace the data textures (linear: packed occlusion / roughness / metalness, normals).
    pub fn set_data(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, images: &[TextureData]) {
        self.data_view = self.upload(device, queue, images, wgpu::TextureFormat::Rgba8Unorm, true);
        self.rebind(device);
    }

    /// Replace the PBR material table (material `i` = vertex index `i + 1`).
    pub fn set_materials(&mut self, queue: &wgpu::Queue, mats: &[crate::PbrMaterial]) {
        self.glass = mats.iter().map(|m| m.transmission > 0.5).collect();
        let mut v = vec![[0f32; 4]; MAX_MATERIALS * MATERIAL_VEC4S];
        for (i, m) in mats.iter().take(MAX_MATERIALS).enumerate() {
            let k = i * MATERIAL_VEC4S;
            v[k] = [m.metallic, m.roughness, m.transmission, m.ior];
            v[k + 1] = [m.emission[0], m.emission[1], m.emission[2], m.normal_scale];
            v[k + 2] = [m.emission_layer as f32, m.orm_layer as f32, m.normal_layer as f32, m.orm_mask as f32];
            v[k + 3] = [m.shader as f32, 0.0, 0.0, 0.0];
            v[k + 4] = [m.params[0], m.params[1], m.params[2], m.params[3]];
            v[k + 5] = [m.params[4], m.params[5], m.params[6], m.params[7]];
        }
        queue.write_buffer(&self.materials, 0, bytemuck::cast_slice(&v));
    }

    fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        images: &[TextureData],
        format: wgpu::TextureFormat,
        data: bool,
    ) -> wgpu::TextureView {
        let images = &images[..images.len().min(self.max_layers as usize)];
        let biggest = images.iter().map(|t| t.width.max(t.height)).max().unwrap_or(1);
        // Many textures share a smaller layer size to bound GPU memory (~5.6 MB per 1024² layer).
        let cap = if images.len() > 16 { MAX_SIDE / 2 } else { MAX_SIDE };
        let side = biggest.clamp(1, cap).next_power_of_two().min(cap);
        let mips = side.ilog2() + 1;
        let texture = Self::create_texture(device, side, images.len() as u32, mips, format);
        let mut info = vec![[0f32; 4]; MAX_LAYERS];
        for (layer, img) in images.iter().enumerate() {
            // z: the texture has cut-out texels (the path tracer only samples those for hits).
            let cut = !data && self.cut.get(layer).copied().unwrap_or(false);
            info[layer] = [1.0 / img.size_mm[0].max(1e-3), 1.0 / img.size_mm[1].max(1e-3), if cut { 1.0 } else { 0.0 }, 0.0];
            let mut level = resample(img, side);
            let mut s = side;
            for mip in 0..mips {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: mip,
                        origin: wgpu::Origin3d { x: 0, y: 0, z: layer as u32 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &level,
                    wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * s), rows_per_image: Some(s) },
                    wgpu::Extent3d { width: s, height: s, depth_or_array_layers: 1 },
                );
                if s > 1 {
                    level = halve(&level, s);
                    s /= 2;
                }
            }
        }
        queue.write_buffer(if data { &self.data_info } else { &self.info }, 0, bytemuck::cast_slice(&info));
        Self::array_view(&texture)
    }
}

/// Bilinear resample to `side`×`side`, flipped so texture row 0 is the image's bottom row
/// (v grows upward on faces).
fn resample(img: &TextureData, side: u32) -> Vec<u8> {
    let (w, h) = (img.width.max(1) as usize, img.height.max(1) as usize);
    let src = &img.rgba;
    let px = |x: usize, y: usize, c: usize| src.get((y * w + x) * 4 + c).copied().unwrap_or(255) as f32;
    let n = side as usize;
    let mut out = vec![0u8; n * n * 4];
    for oy in 0..n {
        // Output row oy samples image row from the bottom.
        let fy = ((n - 1 - oy) as f32 + 0.5) * h as f32 / n as f32 - 0.5;
        let y0 = fy.floor().clamp(0.0, (h - 1) as f32) as usize;
        let y1 = (y0 + 1).min(h - 1);
        let ty = (fy - y0 as f32).clamp(0.0, 1.0);
        for ox in 0..n {
            let fx = (ox as f32 + 0.5) * w as f32 / n as f32 - 0.5;
            let x0 = fx.floor().clamp(0.0, (w - 1) as f32) as usize;
            let x1 = (x0 + 1).min(w - 1);
            let tx = (fx - x0 as f32).clamp(0.0, 1.0);
            for c in 0..4 {
                let top = px(x0, y0, c) * (1.0 - tx) + px(x1, y0, c) * tx;
                let bot = px(x0, y1, c) * (1.0 - tx) + px(x1, y1, c) * tx;
                out[(oy * n + ox) * 4 + c] = (top * (1.0 - ty) + bot * ty).round() as u8;
            }
        }
    }
    out
}

/// 2×2 box filter (sRGB bytes averaged directly; good enough for mip levels).
fn halve(src: &[u8], side: u32) -> Vec<u8> {
    let (s, n) = (side as usize, (side / 2) as usize);
    let mut out = vec![0u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            for c in 0..4 {
                let at = |xx: usize, yy: usize| src[(yy * s + xx) * 4 + c] as u32;
                let sum = at(2 * x, 2 * y) + at(2 * x + 1, 2 * y) + at(2 * x, 2 * y + 1) + at(2 * x + 1, 2 * y + 1);
                out[(y * n + x) * 4 + c] = ((sum + 2) / 4) as u8;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_flips_and_halves() {
        // 1×2 image: top red, bottom blue.
        let img = TextureData { width: 1, height: 2, rgba: vec![255, 0, 0, 255, 0, 0, 255, 255], size_mm: [1.0, 1.0] };
        let r = resample(&img, 2);
        assert_eq!(&r[0..4], &[0, 0, 255, 255], "row 0 = bottom of the image");
        assert_eq!(&r[8..12], &[255, 0, 0, 255]);
        let h = halve(&r, 2);
        assert_eq!(h, vec![128, 0, 128, 255]);
    }
}
