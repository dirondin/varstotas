//! GPU path tracer (`Renderer::pt_*`): the scene's triangles in a BVH (`bvh.rs`), compute
//! passes of `pathtrace.wgsl` (part of the scene module, so it shares the environment, the
//! material table, the textures and the custom surfaces of `FaceStyle::Realistic`).
//!
//! Progressive: each `step` adds samples to a float accumulation buffer; `resolve` runs the
//! à-trous denoiser (optional) and tone maps into an RGBA8 texture shown by egui.

use crate::wgpu;
use crate::{Camera, RenderStyle, SceneGeometry};
use glam::{DVec3, Vec3};

/// A light of the path tracer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PtLight {
    pub kind: PtLightKind,
    /// World position (mm).
    pub position: [f64; 3],
    /// Where a spot or an area light shines (unit, world).
    pub direction: [f32; 3],
    /// Linear RGB.
    pub color: [f32; 3],
    /// Watts (100 W ≈ a bright bulb).
    pub power: f32,
    /// Bulb radius (mm) of point and spot lights: soft shadows.
    pub radius: f32,
    /// Full cone angle of a spot, degrees, and the soft part of its edge (0..1).
    pub spot_angle: f32,
    pub spot_blend: f32,
    /// Width and height of an area light (mm).
    pub size: [f32; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PtLightKind {
    Point,
    Spot,
    Area,
}

/// What `Renderer::pt_start` renders.
#[derive(Clone, Copy, Debug)]
pub struct PtSettings {
    pub width: u32,
    pub height: u32,
    /// Bounces after the first hit.
    pub bounces: u32,
    /// Brightest indirect contribution of a sample (kills fireflies); 0 = no clamp.
    pub clamp: f32,
}

impl Default for PtSettings {
    fn default() -> Self {
        Self { width: 1280, height: 720, bounces: 6, clamp: 20.0 }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct PtUniform {
    eye: [f32; 4],
    right: [f32; 4],
    up: [f32; 4],
    fwd: [f32; 4],
    size: [f32; 4],
    lens: [f32; 4],
    sun: [f32; 4],
    misc: [f32; 4],
    ground: [f32; 4],
}

/// Per-light records (4 vec4 each, `lights` in pathtrace.wgsl), positions relative to `origin`.
pub(crate) fn light_records(lights: &[PtLight], origin: DVec3) -> Vec<[f32; 4]> {
    let mut out = Vec::with_capacity(lights.len() * 4);
    for l in lights {
        let p = (DVec3::from_array(l.position) - origin).as_vec3();
        let d = Vec3::from_array(l.direction).try_normalize().unwrap_or(Vec3::NEG_Z);
        let c = Vec3::from_array(l.color).max(Vec3::ZERO) * l.power.max(0.0) * 1e6;
        let half = (l.spot_angle.clamp(1.0, 179.0) * 0.5).to_radians();
        let inner = half * (1.0 - l.spot_blend.clamp(0.0, 1.0));
        let (kind, extra, intensity, u, cos_outer, cos_inner) = match l.kind {
            PtLightKind::Point => (0.0, l.radius.max(0.0), c / (4.0 * std::f32::consts::PI), Vec3::ZERO, -2.0, -1.0),
            PtLightKind::Spot => (1.0, l.radius.max(0.0), c / (4.0 * std::f32::consts::PI), Vec3::ZERO, half.cos(), inner.cos()),
            PtLightKind::Area => {
                let helper = if d.z.abs() < 0.9 { Vec3::Z } else { Vec3::X };
                let u = d.cross(helper).normalize() * (l.size[0].max(1.0) * 0.5);
                (2.0, l.size[1].max(1.0) * 0.5, c / std::f32::consts::PI, u, 0.0, 0.0)
            }
        };
        out.push([p.x, p.y, p.z, kind]);
        out.push([d.x, d.y, d.z, extra]);
        out.push([intensity.x, intensity.y, intensity.z, cos_outer]);
        out.push([u.x, u.y, u.z, cos_inner]);
    }
    out
}

fn pack_rgba8(c: [f32; 4]) -> f32 {
    let b = |x: f32| (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u32;
    f32::from_bits(b(c[0]) | (b(c[1]) << 8) | (b(c[2]) << 16) | (b(c[3]) << 24))
}

/// Triangle records in BVH order: `tri_geo` (3 vec4 each) and `tri_attr` (5 vec4 each).
pub(crate) fn triangle_records(geo: &SceneGeometry) -> (crate::bvh::Bvh, Vec<[f32; 4]>, Vec<[f32; 4]>) {
    let n = geo.tris.len() / 3;
    let corners: Vec<[[f32; 3]; 3]> =
        (0..n).map(|t| [geo.tris[3 * t].pos, geo.tris[3 * t + 1].pos, geo.tris[3 * t + 2].pos]).collect();
    let bvh = crate::bvh::build(&corners);
    let mut tri_geo = Vec::with_capacity(n * 3);
    let mut tri_attr = Vec::with_capacity(n * 5);
    for &t in &bvh.order {
        let t = t as usize;
        let mut idx = [3 * t, 3 * t + 1, 3 * t + 2];
        let p = |i: usize| Vec3::from_array(geo.tris[i].pos);
        // The geometric normal points to the front side (the vertex normals' side).
        let ng = (p(idx[1]) - p(idx[0])).cross(p(idx[2]) - p(idx[0]));
        let vn = Vec3::from_array(geo.tris[idx[0]].normal) + Vec3::from_array(geo.tris[idx[1]].normal) + Vec3::from_array(geo.tris[idx[2]].normal);
        if ng.dot(vn) < 0.0 {
            idx.swap(1, 2);
        }
        let tex = |i: usize| geo.tex.get(i).copied().unwrap_or_default();
        let (v0, e1, e2) = (p(idx[0]), p(idx[1]) - p(idx[0]), p(idx[2]) - p(idx[0]));
        let t0 = tex(idx[0]);
        let bits = |x: u32| f32::from_bits(x);
        tri_geo.push([v0.x, v0.y, v0.z, bits(t0.mats[0] as u32)]);
        tri_geo.push([e1.x, e1.y, e1.z, bits(t0.mats[1] as u32)]);
        tri_geo.push([e2.x, e2.y, e2.z, bits(t0.layers[0] as u32 | ((t0.layers[1] as u32) << 16))]);
        let back = |i: usize| geo.back_colors.get(i).copied().unwrap_or(crate::DEFAULT_BACK_COLOR);
        let nrm = |i: usize| geo.tris[i].normal;
        let [n0, n1, n2] = [nrm(idx[0]), nrm(idx[1]), nrm(idx[2])];
        tri_attr.push([n0[0], n0[1], n0[2], pack_rgba8(geo.tris[idx[0]].color)]);
        tri_attr.push([n1[0], n1[1], n1[2], pack_rgba8(back(idx[0]))]);
        tri_attr.push([n2[0], n2[1], n2[2], 0.0]);
        let [u0, u1, u2] = [tex(idx[0]).uv, tex(idx[1]).uv, tex(idx[2]).uv];
        tri_attr.push([u0[0], u0[1], u1[0], u1[1]]);
        tri_attr.push([u2[0], u2[1], 0.0, 0.0]);
    }
    (bvh, tri_geo, tri_attr)
}

/// Denoise passes (à-trous step widths).
const DENOISE_STEPS: [f32; 5] = [1.0, 2.0, 4.0, 8.0, 16.0];

pub(crate) struct PathTracer {
    bgl: wgpu::BindGroupLayout,
    dn_bgl: wgpu::BindGroupLayout,
    layout: wgpu::PipelineLayout,
    trace: wgpu::ComputePipeline,
    denoise: wgpu::ComputePipeline,
    resolve: wgpu::ComputePipeline,
    linear: wgpu::ComputePipeline,
    uniform: wgpu::Buffer,
    dn_uniform: wgpu::Buffer,
    dn_bg: wgpu::BindGroup,
    job: Option<Job>,
}

/// Buffers of one render.
struct Job {
    size: [u32; 2],
    u: PtUniform,
    samples: u32,
    /// dn_in = A, dn_out = B, and the other way round.
    bg_ab: wgpu::BindGroup,
    bg_ba: wgpu::BindGroup,
    out: wgpu::Texture,
    out_view: wgpu::TextureView,
    /// Pixel buffer A (the linear image of `read_linear` ends up there).
    dn_a: wgpu::Buffer,
    texture_id: Option<egui::TextureId>,
}

fn buf_entry(binding: u32, buf: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry { binding, resource: buf.as_entire_binding() }
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
        count: None,
    }
}

impl PathTracer {
    pub fn new(device: &wgpu::Device, shader: &wgpu::ShaderModule, globals: &wgpu::BindGroupLayout, faces: &wgpu::BindGroupLayout) -> Self {
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("varstotas pt"),
            entries: &[
                uniform_entry(0),
                storage_entry(1, true),
                storage_entry(2, true),
                storage_entry(3, true),
                storage_entry(4, true),
                storage_entry(5, false),
                storage_entry(6, false),
                storage_entry(7, true),
                storage_entry(8, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let dn_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("varstotas pt denoise"),
            entries: &[uniform_entry(0)],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("varstotas pt"),
            bind_group_layouts: &[Some(globals), Some(faces), Some(&bgl), Some(&dn_bgl)],
            immediate_size: 0,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("varstotas pt uniform"),
            size: std::mem::size_of::<PtUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let dn_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("varstotas pt denoise"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let dn_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("varstotas pt denoise"),
            layout: &dn_bgl,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: dn_uniform.as_entire_binding() }],
        });
        let [trace, denoise, resolve, linear] = Self::pipelines(device, &layout, shader);
        Self { bgl, dn_bgl, layout, trace, denoise, resolve, linear, uniform, dn_uniform, dn_bg, job: None }
    }

    fn pipelines(device: &wgpu::Device, layout: &wgpu::PipelineLayout, shader: &wgpu::ShaderModule) -> [wgpu::ComputePipeline; 4] {
        ["pt_trace", "pt_denoise", "pt_resolve", "pt_linear"].map(|entry| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(layout),
                module: shader,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        })
    }

    /// The scene module changed (custom shaders).
    pub fn rebuild(&mut self, device: &wgpu::Device, shader: &wgpu::ShaderModule) {
        [self.trace, self.denoise, self.resolve, self.linear] = Self::pipelines(device, &self.layout, shader);
        let _ = &self.dn_bgl;
    }

    /// Most pixels a render may have on this device (its per-pixel buffers must fit a binding).
    pub fn max_pixels(device: &wgpu::Device) -> u64 {
        let l = device.limits();
        l.max_storage_buffer_binding_size.min(l.max_buffer_size) / 32
    }

    /// Set up a render of `geo` from `camera` (restarts the accumulation).
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &mut self,
        rs: &egui_wgpu::RenderState,
        geo: &SceneGeometry,
        lights: &[PtLight],
        camera: &Camera,
        style: &RenderStyle,
        settings: &PtSettings,
    ) {
        let device = &rs.device;
        let max = Self::max_pixels(device);
        let (mut w, mut h) = (settings.width.max(8), settings.height.max(8));
        let dim = device.limits().max_texture_dimension_2d;
        while (w as u64 * h as u64) > max || w > dim || h > dim {
            w = (w * 3 / 4).max(8);
            h = (h * 3 / 4).max(8);
        }
        let origin = DVec3::from_array(geo.origin);
        let (bvh, tri_geo, tri_attr) = triangle_records(geo);
        self.upload_scene(device, &bvh.nodes, &tri_geo, &tri_attr, &light_records(lights, origin), [w, h]);
        let job = self.job.as_mut().expect("job");
        job.u = frame_uniform(camera, style, origin, [w, h], settings, lights.len());
        let mut r = rs.renderer.write();
        match job.texture_id {
            Some(id) => r.update_egui_texture_from_wgpu_texture(device, &job.out_view, wgpu::FilterMode::Linear, id),
            None => job.texture_id = Some(r.register_native_texture(device, &job.out_view, wgpu::FilterMode::Linear)),
        }
    }

    fn upload_scene(
        &mut self,
        device: &wgpu::Device,
        nodes: &[[f32; 4]],
        tri_geo: &[[f32; 4]],
        tri_attr: &[[f32; 4]],
        lights: &[[f32; 4]],
        size: [u32; 2],
    ) {
        use wgpu::util::DeviceExt;
        let buf = |label: &str, data: &[[f32; 4]]| {
            // Empty storage bindings are not allowed: at least one vec4.
            let pad = [[0f32; 4]];
            let data = if data.is_empty() { &pad[..] } else { data };
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(data),
                usage: wgpu::BufferUsages::STORAGE,
            })
        };
        let bvh = buf("varstotas pt bvh", nodes);
        let geo = buf("varstotas pt triangles", tri_geo);
        let attr = buf("varstotas pt attributes", tri_attr);
        let lights = buf("varstotas pt lights", lights);
        let px = size[0] as u64 * size[1] as u64;
        let pixels = |label: &str, vec4s: u64, extra: wgpu::BufferUsages| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: px * vec4s * 16,
                usage: wgpu::BufferUsages::STORAGE | extra,
                mapped_at_creation: false,
            })
        };
        let accum = pixels("varstotas pt accumulation", 1, wgpu::BufferUsages::empty());
        let aov = pixels("varstotas pt aov", 2, wgpu::BufferUsages::empty());
        let dn_a = pixels("varstotas pt denoise a", 1, wgpu::BufferUsages::COPY_SRC);
        let dn_b = pixels("varstotas pt denoise b", 1, wgpu::BufferUsages::empty());
        let out = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("varstotas pt image"),
            size: wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let out_view = out.create_view(&Default::default());
        let bg = |a: &wgpu::Buffer, b: &wgpu::Buffer| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("varstotas pt"),
                layout: &self.bgl,
                entries: &[
                    buf_entry(0, &self.uniform),
                    buf_entry(1, &bvh),
                    buf_entry(2, &geo),
                    buf_entry(3, &attr),
                    buf_entry(4, &lights),
                    buf_entry(5, &accum),
                    buf_entry(6, &aov),
                    buf_entry(7, a),
                    buf_entry(8, b),
                    wgpu::BindGroupEntry { binding: 9, resource: wgpu::BindingResource::TextureView(&out_view) },
                ],
            })
        };
        let bg_ab = bg(&dn_a, &dn_b);
        let bg_ba = bg(&dn_b, &dn_a);
        let texture_id = self.job.take().and_then(|j| j.texture_id);
        self.job = Some(Job { size, u: PtUniform::default(), samples: 0, bg_ab, bg_ba, out, out_view, dn_a, texture_id });
    }

    pub fn samples(&self) -> u32 {
        self.job.as_ref().map_or(0, |j| j.samples)
    }

    pub fn size(&self) -> Option<[u32; 2]> {
        self.job.as_ref().map(|j| j.size)
    }

    pub fn texture_id(&self) -> Option<egui::TextureId> {
        self.job.as_ref().and_then(|j| j.texture_id)
    }

    /// Free the render's buffers and its egui texture.
    pub fn clear(&mut self, rs: &egui_wgpu::RenderState) {
        if let Some(id) = self.job.take().and_then(|j| j.texture_id) {
            rs.renderer.write().free_texture(&id);
        }
    }

    fn groups(&self, groups: [&wgpu::BindGroup; 2]) -> impl Fn(&mut wgpu::ComputePass<'_>, &wgpu::BindGroup) + use<> {
        let [g0, g1] = groups.map(|g| g.clone());
        let dn = self.dn_bg.clone();
        move |pass: &mut wgpu::ComputePass<'_>, own: &wgpu::BindGroup| {
            pass.set_bind_group(0, &g0, &[]);
            pass.set_bind_group(1, &g1, &[]);
            pass.set_bind_group(2, own, &[]);
            pass.set_bind_group(3, &dn, &[]);
        }
    }

    /// Add samples until `max_samples` or about `budget` has passed (at least one sample).
    /// `groups` = the scene's group 0 and 1. Returns the samples now accumulated.
    pub fn step(&mut self, rs: &egui_wgpu::RenderState, groups: [&wgpu::BindGroup; 2], max_samples: u32, budget: std::time::Duration) -> u32 {
        let set = self.groups(groups);
        let Some(job) = &mut self.job else { return 0 };
        let t0 = std::time::Instant::now();
        let wg = [job.size[0].div_ceil(8), job.size[1].div_ceil(8)];
        while job.samples < max_samples.max(1) {
            job.u.size[2] = job.samples as f32;
            rs.queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&job.u));
            let mut enc = rs.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("varstotas pt") });
            {
                let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("pt_trace"), timestamp_writes: None });
                pass.set_pipeline(&self.trace);
                set(&mut pass, &job.bg_ab);
                pass.dispatch_workgroups(wg[0], wg[1], 1);
            }
            let sub = rs.queue.submit(Some(enc.finish()));
            let _ = rs.device.poll(wgpu::PollType::Wait { submission_index: Some(sub), timeout: None });
            job.samples += 1;
            if t0.elapsed() >= budget {
                break;
            }
        }
        job.samples
    }

    /// Show the samples so far in the texture, denoised or not.
    pub fn resolve(&mut self, rs: &egui_wgpu::RenderState, groups: [&wgpu::BindGroup; 2], denoise: bool) {
        self.run_output(rs, groups, denoise, false);
    }

    /// Denoise steps (ending in buffer B), then `pt_resolve` or `pt_linear` reading B.
    fn run_output(&mut self, rs: &egui_wgpu::RenderState, groups: [&wgpu::BindGroup; 2], denoise: bool, linear: bool) {
        let set = self.groups(groups);
        let Some(job) = &self.job else { return };
        if job.samples == 0 {
            return;
        }
        let wg = [job.size[0].div_ceil(8), job.size[1].div_ceil(8)];
        rs.queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&job.u));
        let run = |pipeline: &wgpu::ComputePipeline, bg: &wgpu::BindGroup, dn: [f32; 4]| {
            rs.queue.write_buffer(&self.dn_uniform, 0, bytemuck::bytes_of(&dn));
            let mut enc = rs.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("varstotas pt output") });
            {
                let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("pt output"), timestamp_writes: None });
                pass.set_pipeline(pipeline);
                set(&mut pass, bg);
                pass.dispatch_workgroups(wg[0], wg[1], 1);
            }
            rs.queue.submit(Some(enc.finish()));
        };
        if denoise {
            // Odd steps write B (bg_ab), even ones A; 5 steps end in B.
            for (i, &step) in DENOISE_STEPS.iter().enumerate() {
                let bg = if i % 2 == 0 { &job.bg_ab } else { &job.bg_ba };
                run(&self.denoise, bg, [step, if i == 0 { 1.0 } else { 0.0 }, 0.6, 0.0]);
            }
        }
        let flag = if denoise { 1.0 } else { 0.0 };
        run(if linear { &self.linear } else { &self.resolve }, &job.bg_ba, [0.0, 0.0, 0.0, flag]);
    }

    /// The shown image as sRGB RGBA8 (after `resolve`).
    pub fn read_image(&self, rs: &egui_wgpu::RenderState) -> Option<crate::RgbaImage> {
        let job = self.job.as_ref()?;
        let [w, h] = job.size;
        let padded = (w * 4).div_ceil(256) * 256;
        let buf = rs.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("varstotas pt readback"),
            size: padded as u64 * h as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = rs.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("varstotas pt readback") });
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &job.out, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo {
                buffer: &buf,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(h) },
            },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        let data = map_read(rs, &buf, enc)?;
        let mut pixels = Vec::with_capacity((w * h * 4) as usize);
        for r in 0..h as usize {
            pixels.extend_from_slice(&data[r * padded as usize..][..w as usize * 4]);
        }
        Some(crate::RgbaImage { width: w, height: h, pixels })
    }

    /// The image as linear RGB (exposure applied, no tone map), rows top to bottom.
    pub fn read_linear(&mut self, rs: &egui_wgpu::RenderState, groups: [&wgpu::BindGroup; 2], denoise: bool) -> Option<(u32, u32, Vec<f32>)> {
        self.run_output(rs, groups, denoise, true);
        let job = self.job.as_ref()?;
        let [w, h] = job.size;
        let size = w as u64 * h as u64 * 16;
        let buf = rs.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("varstotas pt readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = rs.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("varstotas pt readback") });
        enc.copy_buffer_to_buffer(&job.dn_a, 0, &buf, 0, size);
        let data = map_read(rs, &buf, enc)?;
        let px: &[[f32; 4]] = bytemuck::cast_slice(&data);
        let rgb = px.iter().flat_map(|p| [p[0], p[1], p[2]]).collect();
        // Leave the shown texture as it was.
        Some((w, h, rgb))
    }
}

/// Submit `enc` and read `buf` back.
fn map_read(rs: &egui_wgpu::RenderState, buf: &wgpu::Buffer, enc: wgpu::CommandEncoder) -> Option<Vec<u8>> {
    let sub = rs.queue.submit(Some(enc.finish()));
    let slice = buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    rs.device.poll(wgpu::PollType::Wait { submission_index: Some(sub), timeout: None }).ok()?;
    rx.recv().ok()?.ok()?;
    let data = slice.get_mapped_range().ok()?.to_vec();
    buf.unmap();
    Some(data)
}

fn lin(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
}

/// The camera, sun and lens of a render.
fn frame_uniform(camera: &Camera, style: &RenderStyle, origin: DVec3, size: [u32; 2], s: &PtSettings, lights: usize) -> PtUniform {
    let b = camera.basis();
    let aspect = size[0] as f64 / size[1] as f64;
    let (rw, uw) = if camera.ortho {
        let hh = camera.ortho_height * 0.5;
        (hh * aspect, hh)
    } else {
        let t = (camera.fov_y_deg.to_radians() * 0.5).tan();
        (t * aspect, t)
    };
    let fx = &style.env.effects;
    let eye = camera.eye - origin;
    let v4 = |v: DVec3, w: f64| [v.x as f32, v.y as f32, v.z as f32, w as f32];
    let lens = if camera.ortho { 0.0 } else { fx.aperture.max(0.0) as f64 * 20.0 };
    let focus = if fx.focus_distance > 0.0 { fx.focus_distance as f64 } else { camera.distance() };
    let ld = DVec3::new(style.light_dir[0] as f64, style.light_dir[1] as f64, style.light_dir[2] as f64);
    let mut sun = ld.try_normalize().unwrap_or(DVec3::Z);
    let mut strength = style.env.sun.max(0.0);
    if let Some(sh) = style.shadows.filter(|s| DVec3::from_array(s.sun_dir).is_finite()) {
        sun = DVec3::from_array(sh.sun_dir).normalize_or(DVec3::Z);
        if sun.z <= 0.0 {
            strength = 0.0;
        }
    }
    // The sun's disc grows with the soft shadows setting (0.27° radius at 1).
    let sun_radius = (0.27f64 * (fx.soft_shadows.max(0.0) as f64).max(0.2)).to_radians();
    PtUniform {
        eye: v4(eye, if camera.ortho { 1.0 } else { 0.0 }),
        right: v4(b.right, rw),
        up: v4(b.up, uw),
        fwd: v4(b.forward, lens),
        size: [size[0] as f32, size[1] as f32, 0.0, s.bounces.min(64) as f32],
        lens: [focus as f32, sun_radius.cos() as f32, lights as f32, if s.clamp > 0.0 { s.clamp } else { 1e9 }],
        sun: v4(sun, strength as f64),
        misc: [origin.z as f32, fx.time, if style.show_ground { 1.0 } else { 0.0 }, 0.0],
        ground: [lin(style.ground[0]), lin(style.ground[1]), lin(style.ground[2]), -origin.z as f32],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TexVertex, TriVertex};

    #[test]
    fn light_records_are_local_and_scaled() {
        let l = PtLight {
            kind: PtLightKind::Spot,
            position: [1000.0, 0.0, 3000.0],
            direction: [0.0, 0.0, -2.0],
            color: [1.0, 0.5, 0.0],
            power: 100.0,
            radius: 50.0,
            spot_angle: 60.0,
            spot_blend: 0.5,
            size: [0.0, 0.0],
        };
        let area = PtLight { kind: PtLightKind::Area, size: [800.0, 400.0], ..l };
        let r = light_records(&[l, area], DVec3::new(1000.0, 0.0, 1000.0));
        assert_eq!(r.len(), 8);
        assert_eq!(r[0], [0.0, 0.0, 2000.0, 1.0]);
        assert_eq!(r[1], [0.0, 0.0, -1.0, 50.0]);
        let i = 100.0 * 1e6 / (4.0 * std::f32::consts::PI);
        assert!((r[2][0] - i).abs() < 1.0 && (r[2][1] - i * 0.5).abs() < 1.0 && r[2][2] == 0.0);
        // cones: outer 30°, inner 15°
        assert!((r[2][3] - 30f32.to_radians().cos()).abs() < 1e-6 && (r[3][3] - 15f32.to_radians().cos()).abs() < 1e-6);
        // area: kind 2, half height, half-width vector across the direction, I = P / π
        assert_eq!((r[4][3], r[5][3]), (2.0, 200.0));
        let u = Vec3::new(r[7][0], r[7][1], r[7][2]);
        assert!((u.length() - 400.0).abs() < 1e-3 && u.z.abs() < 1e-6);
        assert!((r[6][0] - 100.0 * 1e6 / std::f32::consts::PI).abs() < 10.0);
    }

    #[test]
    fn triangles_face_their_normals() {
        let v = |pos: [f32; 3]| TriVertex { pos, normal: [0.0, 0.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] };
        // Clockwise seen from +Z, normals up: the corners get swapped.
        let geo = SceneGeometry {
            tris: vec![v([0.0, 0.0, 0.0]), v([0.0, 1.0, 0.0]), v([1.0, 0.0, 0.0])],
            back_colors: vec![[0.0, 0.0, 1.0, 1.0]; 3],
            tex: vec![TexVertex { uv: [0.0, 0.0], layers: [2, 3], mats: [4, 5] }; 3],
            ..Default::default()
        };
        let (bvh, g, a) = triangle_records(&geo);
        assert_eq!(bvh.order, vec![0]);
        let e1 = Vec3::new(g[1][0], g[1][1], g[1][2]);
        let e2 = Vec3::new(g[2][0], g[2][1], g[2][2]);
        assert!(e1.cross(e2).z > 0.0);
        assert_eq!((g[0][3].to_bits(), g[1][3].to_bits(), g[2][3].to_bits()), (4, 5, 2 | (3 << 16)));
        assert_eq!(a[0][3].to_bits(), 0xff00_00ff);
        assert_eq!(a[1][3].to_bits(), 0xffff_0000);
    }
}
