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
    /// Depth of field: lens radius and focus distance (mm), instead of the style's effects.
    pub lens: Option<[f32; 2]>,
    /// The background (sky and ground plane seen directly) is transparent (alpha in the
    /// image); it still lights the scene.
    pub transparent: bool,
}

impl Default for PtSettings {
    fn default() -> Self {
        Self { width: 1280, height: 720, bounces: 6, clamp: 20.0, lens: None, transparent: false }
    }
}

/// How the samples become the image (`Renderer::pt_set_output`; changes need no new samples).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PtOutput {
    /// Linear multiplier (the environment's exposure and the render's offset).
    pub exposure: f32,
    /// 0 = ACES, 1 = AgX, 2 = clip (as `Tonemap`).
    pub tonemap: u32,
    /// White balance multipliers (linear RGB, see `white_balance`).
    pub white: [f32; 3],
    pub saturation: f32,
    pub contrast: f32,
    pub vignette: f32,
    /// Denoise with Open Image Denoise when it is installed.
    pub ai_denoise: bool,
}

impl Default for PtOutput {
    fn default() -> Self {
        Self { exposure: 1.0, tonemap: 0, white: [1.0; 3], saturation: 1.0, contrast: 1.0, vignette: 0.0, ai_denoise: true }
    }
}

/// Colour of a black body at `kelvin` (linear RGB, max 1; Tanner Helland's fit).
fn blackbody(kelvin: f32) -> [f32; 3] {
    let t = kelvin.clamp(1000.0, 40000.0) / 100.0;
    let r = if t <= 66.0 { 255.0 } else { 329.698_73 * (t - 60.0).powf(-0.133_204_76) };
    let g = if t <= 66.0 { 99.470_8 * t.ln() - 161.119_57 } else { 288.122_16 * (t - 60.0).powf(-0.075_514_85) };
    let b = if t >= 66.0 {
        255.0
    } else if t <= 19.0 {
        0.0
    } else {
        138.517_73 * (t - 10.0).ln() - 305.044_8
    };
    [r, g, b].map(|c| crate::srgb_to_linear(c / 255.0))
}

/// White balance multipliers for a `kelvin` setting (6500 = neutral; higher is warmer, as in a
/// photo editor) and a `tint` (−1 green .. 1 magenta), keeping the brightness.
pub fn white_balance(kelvin: f32, tint: f32) -> [f32; 3] {
    let (n, k) = (blackbody(6500.0), blackbody(kelvin));
    let mut m: [f32; 3] = std::array::from_fn(|i| n[i] / k[i].max(1e-3));
    m[1] *= 1.0 - tint.clamp(-1.0, 1.0) * 0.25;
    let luma = 0.2126 * m[0] + 0.7152 * m[1] + 0.0722 * m[2];
    m.map(|c| c / luma.max(1e-6))
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
    grade: [f32; 4],
    grade2: [f32; 4],
    extra: [f32; 4],
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
        let idx = [3 * t, 3 * t + 1, 3 * t + 2];
        let p = |i: usize| Vec3::from_array(geo.tris[i].pos);
        // Winding defines front/back materials, consistently with rasterization. Authored
        // shading normals can cross the geometric hemisphere on thin/degenerate details;
        // they must never turn a triangle (or the inside/outside of glass) around.
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

/// The acceleration structures of a render with ray queries.
pub(crate) struct Accel {
    tlas: wgpu::Tlas,
    /// Instance offset + primitive → triangle record (`rq_tri`), 4 per vec4: in place of the
    /// BVH.
    map: Vec<[f32; 4]>,
}

/// Build the BLAS of the opaque triangles and of those the shader tests (glass, cut-out
/// textures on a side; `rq_tri` instance offset = the opaque count) and the TLAS of both.
fn accel_structures(rs: &egui_wgpu::RenderState, tri_geo: &[[f32; 4]], test: &ShaderTest<'_>) -> Accel {
    use wgpu::util::DeviceExt;
    let device = &rs.device;
    let n = tri_geo.len() / 3;
    let mut verts: Vec<[f32; 3]> = Vec::with_capacity(n * 3);
    let (mut opaque, mut tested) = (Vec::new(), Vec::new());
    let flag = |list: &[bool], i: u32| i > 0 && list.get(i as usize - 1).copied().unwrap_or(false);
    for t in 0..n {
        let [g0, g1, g2] = [tri_geo[3 * t], tri_geo[3 * t + 1], tri_geo[3 * t + 2]];
        let v0 = [g0[0], g0[1], g0[2]];
        verts.push(v0);
        verts.push([v0[0] + g1[0], v0[1] + g1[1], v0[2] + g1[2]]);
        verts.push([v0[0] + g2[0], v0[1] + g2[1], v0[2] + g2[2]]);
        let layers = g2[3].to_bits();
        let shader = flag(test.glass, g0[3].to_bits())
            || flag(test.glass, g1[3].to_bits())
            || flag(test.cut, layers & 0xffff)
            || flag(test.cut, layers >> 16);
        if shader { &mut tested } else { &mut opaque }.push(t as u32);
    }
    if verts.is_empty() {
        verts.push([0.0; 3]);
    }
    let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("varstotas pt vertices"),
        contents: bytemuck::cast_slice(&verts),
        usage: wgpu::BufferUsages::BLAS_INPUT,
    });
    let mut tlas = device.create_tlas(&wgpu::CreateTlasDescriptor {
        label: Some("varstotas pt tlas"),
        max_instances: 2,
        flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
        update_mode: wgpu::AccelerationStructureUpdateMode::Build,
    });
    let mut parts = Vec::new();
    for (k, (list, geo_flags)) in [
        (&opaque, wgpu::AccelerationStructureGeometryFlags::OPAQUE),
        (&tested, wgpu::AccelerationStructureGeometryFlags::NO_DUPLICATE_ANY_HIT_INVOCATION),
    ]
    .into_iter()
    .enumerate()
    {
        if list.is_empty() {
            continue;
        }
        let idx: Vec<u32> = list.iter().flat_map(|&t| [3 * t, 3 * t + 1, 3 * t + 2]).collect();
        let ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("varstotas pt indices"),
            contents: bytemuck::cast_slice(&idx),
            usage: wgpu::BufferUsages::BLAS_INPUT,
        });
        let size = wgpu::BlasTriangleGeometrySizeDescriptor {
            vertex_format: wgpu::VertexFormat::Float32x3,
            vertex_count: verts.len() as u32,
            index_format: Some(wgpu::IndexFormat::Uint32),
            index_count: Some(idx.len() as u32),
            flags: geo_flags,
        };
        let blas = device.create_blas(
            &wgpu::CreateBlasDescriptor {
                label: Some("varstotas pt blas"),
                flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
                update_mode: wgpu::AccelerationStructureUpdateMode::Build,
            },
            wgpu::BlasGeometrySizeDescriptors::Triangles { descriptors: vec![size.clone()] },
        );
        let offset = if k == 0 { 0 } else { opaque.len() as u32 };
        tlas[k] = Some(wgpu::TlasInstance::new(&blas, [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0], offset, 0xff));
        parts.push((blas, ib, size));
    }
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("varstotas pt accel") });
    let entries: Vec<wgpu::BlasBuildEntry<'_>> = parts
        .iter()
        .map(|(blas, ib, size)| wgpu::BlasBuildEntry {
            blas,
            geometry: wgpu::BlasGeometries::TriangleGeometries(vec![wgpu::BlasTriangleGeometry {
                size,
                vertex_buffer: &vb,
                first_vertex: 0,
                vertex_stride: 12,
                index_buffer: Some(ib),
                first_index: Some(0),
                transform_buffer: None,
                transform_buffer_offset: None,
            }]),
        })
        .collect();
    enc.build_acceleration_structures(entries.iter(), std::iter::once(&tlas));
    rs.queue.submit(Some(enc.finish()));
    let ids: Vec<u32> = opaque.into_iter().chain(tested).collect();
    let map = ids.chunks(4).map(|c| std::array::from_fn(|i| f32::from_bits(c.get(i).copied().unwrap_or(0)))).collect();
    Accel { tlas, map }
}

/// Target length of one tracing dispatch (s).
const BATCH_TIME: f64 = 0.03;

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
    output: PtOutput,
    /// The device traces with ray queries (`pathtrace_rq.wgsl`) instead of the BVH.
    rq: bool,
}

/// Which triangles a ray query hands to the shader (the others are opaque to it): colour
/// layer i has cut-out texels, material i is glass.
pub(crate) struct ShaderTest<'a> {
    pub cut: &'a [bool],
    pub glass: &'a [bool],
}

/// The tracing module with ray queries: `src` (the scene module) with its BVH traversal
/// replaced by `pathtrace_rq.wgsl`.
pub(crate) fn rq_source(src: &str) -> String {
    let (a, b) = ("// ---- bvh traversal", "// ---- end bvh traversal ----"); // i18n-ignore
    let (i, j) = (src.find(a).expect("bvh section"), src.find(b).expect("bvh section end"));
    format!("enable wgpu_ray_query;\n{}{}{}", &src[..i], include_str!("pathtrace_rq.wgsl"), &src[j..])
}

/// Ray queries are used when the device has them (VARSTOTAS_NO_RAY_QUERY=1 turns them off).
fn use_rq(device: &wgpu::Device) -> bool {
    device.features().contains(wgpu::Features::EXPERIMENTAL_RAY_QUERY) && std::env::var_os("VARSTOTAS_NO_RAY_QUERY").is_none()
}

/// Buffers of one render.
struct Job {
    size: [u32; 2],
    u: PtUniform,
    samples: u32,
    /// Samples per pixel of one dispatch (grows while dispatches stay short).
    batch: u32,
    /// dn_in = A, dn_out = B, and the other way round.
    bg_ab: wgpu::BindGroup,
    bg_ba: wgpu::BindGroup,
    out: wgpu::Texture,
    out_view: wgpu::TextureView,
    /// Pixel buffer A (the linear image of `read_linear` ends up there).
    dn_a: wgpu::Buffer,
    /// Buffer B: the denoised image the output passes read; the accumulation and the AOVs (read
    /// back for Open Image Denoise).
    dn_b: wgpu::Buffer,
    accum: wgpu::Buffer,
    aov: wgpu::Buffer,
    /// The samples B holds an OIDN result of (0 = none).
    oidn_samples: u32,
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
    pub fn new(device: &wgpu::Device, shader: &wgpu::ShaderModule, src: &str, globals: &wgpu::BindGroupLayout, faces: &wgpu::BindGroupLayout) -> Self {
        let rq = use_rq(device);
        let mut entries = vec![
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
        ];
        if rq {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: 10,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::AccelerationStructure { vertex_return: false },
                count: None,
            });
        }
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("varstotas pt"), entries: &entries });
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
        let [trace, denoise, resolve, linear] = Self::pipelines(device, &layout, shader, src, rq);
        if rq {
            log::info!("path tracer: ray queries");
        }
        Self { bgl, dn_bgl, layout, trace, denoise, resolve, linear, uniform, dn_uniform, dn_bg, job: None, output: PtOutput::default(), rq }
    }

    fn pipelines(device: &wgpu::Device, layout: &wgpu::PipelineLayout, shader: &wgpu::ShaderModule, src: &str, rq: bool) -> [wgpu::ComputePipeline; 4] {
        let traced = rq.then(|| {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("varstotas path tracer (ray queries)"),
                source: wgpu::ShaderSource::Wgsl(rq_source(src).into()),
            })
        });
        ["pt_trace", "pt_denoise", "pt_resolve", "pt_linear"].map(|entry| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(layout),
                module: if entry == "pt_trace" { traced.as_ref().unwrap_or(shader) } else { shader },
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        })
    }

    /// The scene module changed (custom shaders).
    pub fn rebuild(&mut self, device: &wgpu::Device, shader: &wgpu::ShaderModule, src: &str) {
        [self.trace, self.denoise, self.resolve, self.linear] = Self::pipelines(device, &self.layout, shader, src, self.rq);
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
        test: &ShaderTest<'_>,
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
        let accel = self.rq.then(|| accel_structures(rs, &tri_geo, test));
        let nodes = accel.as_ref().map_or(&bvh.nodes, |a| &a.map);
        self.upload_scene(device, nodes, &tri_geo, &tri_attr, &light_records(lights, origin), [w, h], accel.as_ref());
        let job = self.job.as_mut().expect("job");
        job.u = frame_uniform(camera, style, origin, [w, h], settings, lights.len());
        set_grade(&mut job.u, &self.output);
        let mut r = rs.renderer.write();
        match job.texture_id {
            Some(id) => r.update_egui_texture_from_wgpu_texture(device, &job.out_view, wgpu::FilterMode::Linear, id),
            None => job.texture_id = Some(r.register_native_texture(device, &job.out_view, wgpu::FilterMode::Linear)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn upload_scene(
        &mut self,
        device: &wgpu::Device,
        nodes: &[[f32; 4]],
        tri_geo: &[[f32; 4]],
        tri_attr: &[[f32; 4]],
        lights: &[[f32; 4]],
        size: [u32; 2],
        accel: Option<&Accel>,
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
        let accum = pixels("varstotas pt accumulation", 1, wgpu::BufferUsages::COPY_SRC);
        let aov = pixels("varstotas pt aov", 2, wgpu::BufferUsages::COPY_SRC);
        let dn_a = pixels("varstotas pt denoise a", 1, wgpu::BufferUsages::COPY_SRC);
        let dn_b = pixels("varstotas pt denoise b", 1, wgpu::BufferUsages::COPY_DST);
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
                ]
                .into_iter()
                .chain(accel.map(|a| wgpu::BindGroupEntry { binding: 10, resource: wgpu::BindingResource::AccelerationStructure(&a.tlas) }))
                .collect::<Vec<_>>(),
            })
        };
        let bg_ab = bg(&dn_a, &dn_b);
        let bg_ba = bg(&dn_b, &dn_a);
        let texture_id = self.job.take().and_then(|j| j.texture_id);
        self.job = Some(Job { size, u: PtUniform::default(), samples: 0, batch: 1, bg_ab, bg_ba, out, out_view, dn_a, dn_b, accum, aov, oidn_samples: 0, texture_id });
    }

    /// Exposure, tone curve, grading and denoiser of the image (applied at the next resolve).
    pub fn set_output(&mut self, output: PtOutput) {
        if self.output.ai_denoise != output.ai_denoise
            && let Some(job) = &mut self.job
        {
            job.oidn_samples = 0;
        }
        self.output = output;
        if let Some(job) = &mut self.job {
            set_grade(&mut job.u, &output);
        }
    }

    pub fn size(&self) -> Option<[u32; 2]> {
        self.job.as_ref().map(|j| j.size)
    }

    pub fn texture_id(&self) -> Option<egui::TextureId> {
        self.job.as_ref().and_then(|j| j.texture_id)
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
            let n = job.batch.min(max_samples.max(1) - job.samples);
            job.u.size[2] = job.samples as f32;
            job.u.misc[3] = n as f32;
            let t = std::time::Instant::now();
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
            job.samples += n;
            // About BATCH_TIME per dispatch: few round trips, yet the UI stays responsive.
            let per = t.elapsed().as_secs_f64() / n as f64;
            if n == job.batch {
                job.batch = ((BATCH_TIME / per.max(1e-6)) as u32).clamp(1, 16).min(job.batch * 2);
            }
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
        let oidn = denoise && self.output.ai_denoise && self.oidn(rs);
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
        if denoise && !oidn {
            // Odd steps write B (bg_ab), even ones A; 5 steps end in B.
            for (i, &step) in DENOISE_STEPS.iter().enumerate() {
                let bg = if i % 2 == 0 { &job.bg_ab } else { &job.bg_ba };
                run(&self.denoise, bg, [step, if i == 0 { 1.0 } else { 0.0 }, 0.6, 0.0]);
            }
        }
        let flag = if denoise { 1.0 } else { 0.0 };
        run(if linear { &self.linear } else { &self.resolve }, &job.bg_ba, [0.0, 0.0, 0.0, flag]);
    }

    /// Denoise the samples so far with Open Image Denoise into buffer B (as the à-trous filter
    /// leaves it: light divided by the albedo); false when OIDN is missing or fails.
    fn oidn(&mut self, rs: &egui_wgpu::RenderState) -> bool {
        let Some(job) = &mut self.job else { return false };
        if job.samples == 0 || !crate::oidn::available() {
            return false;
        }
        if job.oidn_samples == job.samples {
            return true;
        }
        let [w, h] = job.size;
        let px = w as u64 * h as u64;
        let read = |src: &wgpu::Buffer, size: u64| {
            let buf = rs.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("varstotas pt readback"),
                size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            let mut enc = rs.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("varstotas pt readback") });
            enc.copy_buffer_to_buffer(src, 0, &buf, 0, size);
            map_read(rs, &buf, enc)
        };
        let (Some(acc), Some(aov)) = (read(&job.accum, px * 16), read(&job.aov, px * 32)) else { return false };
        let acc: &[[f32; 4]] = bytemuck::cast_slice(&acc);
        let aov: &[[f32; 4]] = bytemuck::cast_slice(&aov);
        let mut color = Vec::with_capacity(px as usize * 3);
        let mut albedo = Vec::with_capacity(px as usize * 3);
        let mut normal = Vec::with_capacity(px as usize * 3);
        // Means over the samples (the AOVs are sums too, see pt_trace).
        let mean_albedo = |i: usize| {
            let (al, k) = (aov[2 * i], 1.0 / acc[i][3].max(1.0));
            [0, 1, 2].map(|c| (al[c] * k).clamp(0.02, 1.0))
        };
        for (i, a) in acc.iter().enumerate() {
            let k = 1.0 / a[3].max(1.0);
            color.extend([a[0] * k, a[1] * k, a[2] * k]);
            albedo.extend(mean_albedo(i));
            let n = aov[2 * i + 1];
            normal.extend([n[0] * k, n[1] * k, n[2] * k]);
        }
        let t = std::time::Instant::now();
        let Some(out) = crate::oidn::denoise(w as usize, h as usize, &mut color, &mut albedo, &mut normal) else { return false };
        log::info!("denoiser: {w}x{h} in {:.2} s", t.elapsed().as_secs_f64());
        // B holds light / albedo (pt_resolve multiplies by max(albedo, 0.02)).
        let b: Vec<[f32; 4]> = (0..px as usize)
            .map(|i| {
                let al = mean_albedo(i);
                let c = |k: usize| out[3 * i + k].max(0.0) / al[k];
                [c(0), c(1), c(2), 1.0]
            })
            .collect();
        rs.queue.write_buffer(&job.dn_b, 0, bytemuck::cast_slice(&b));
        job.oidn_samples = job.samples;
        true
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

fn set_grade(u: &mut PtUniform, o: &PtOutput) {
    u.grade = [o.exposure.max(0.0), o.tonemap as f32, o.saturation, o.contrast];
    u.grade2 = [o.white[0], o.white[1], o.white[2], o.vignette];
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
    let (lens, focus) = match s.lens {
        _ if camera.ortho => (0.0, camera.distance()),
        Some([r, f]) => (r.max(0.0) as f64, if f > 0.0 { f as f64 } else { camera.distance() }),
        None => (fx.aperture.max(0.0) as f64 * 20.0, if fx.focus_distance > 0.0 { fx.focus_distance as f64 } else { camera.distance() }),
    };
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
        ground: [crate::srgb_to_linear(style.ground[0]), crate::srgb_to_linear(style.ground[1]), crate::srgb_to_linear(style.ground[2]), -origin.z as f32],
        grade: [1.0, 0.0, 1.0, 1.0],
        grade2: [1.0, 1.0, 1.0, 0.0],
        extra: [if s.transparent { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TexVertex, TriVertex};

    #[test]
    fn white_balance_is_neutral_at_6500_and_warmer_above() {
        let n = white_balance(6500.0, 0.0);
        assert!(n.iter().all(|c| (c - 1.0).abs() < 1e-3), "{n:?}");
        let warm = white_balance(9000.0, 0.0);
        let cool = white_balance(3000.0, 0.0);
        assert!(warm[0] > warm[2] && cool[2] > cool[0], "{warm:?} {cool:?}");
        assert!(white_balance(6500.0, 1.0)[1] < 1.0);
    }

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
    fn the_ray_query_module_is_valid() {
        use crate::wgpu::naga;
        let src = rq_source(&crate::custom::compose(&[]).0);
        assert!(src.starts_with("enable wgpu_ray_query;") && !src.contains("fn node_enter"));
        let module = naga::front::wgsl::parse_str(&src).unwrap_or_else(|e| panic!("{}", e.emit_to_string(&src)));
        let caps = naga::valid::Capabilities::RAY_QUERY;
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), caps)
            .validate(&module)
            .unwrap_or_else(|e| panic!("{}", e.emit_to_string(&src)));
    }

    #[test]
    fn shading_normals_do_not_reverse_triangle_sides() {
        let v = |pos: [f32; 3]| TriVertex { pos, normal: [0.0, 0.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] };
        // Clockwise seen from +Z, normals up: preserve geometry and side materials.
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
        assert!(e1.cross(e2).z < 0.0);
        assert_eq!((g[0][3].to_bits(), g[1][3].to_bits(), g[2][3].to_bits()), (4, 5, 2 | (3 << 16)));
        assert_eq!(a[0][3].to_bits(), 0xff00_00ff);
        assert_eq!(a[1][3].to_bits(), 0xffff_0000);
    }
}
