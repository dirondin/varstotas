//! Effects path of the Realistic style: the scene pass renders linear light into a float
//! target plus a G-buffer (normal, roughness, metalness), then full-screen passes (post.wgsl)
//! add ambient occlusion, reflections, fog, depth of field and bloom, tone map, vignette and
//! the stylized looks; the result is drawn into the usual multisampled target before overlays.

use super::{DEPTH_FORMAT, SAMPLES};
use crate::wgpu;
use crate::{Effects, Stylize};

/// Float colour of the scene pass and the post chain.
pub(crate) const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
/// G-buffer of the scene pass (see `gbuf_pack` in shader.wgsl).
pub(crate) const GBUF_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// Linear depth: R32Float, or R16Float where R32Float cannot be rendered to (OpenGL without
/// EXT_color_buffer_float); see [`lin_format`].
const LIN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;

/// The linear depth format this adapter can render to.
pub(crate) fn lin_format(adapter: &wgpu::Adapter) -> wgpu::TextureFormat {
    let renderable = adapter.get_texture_format_features(LIN_FORMAT).allowed_usages.contains(wgpu::TextureUsages::RENDER_ATTACHMENT);
    if renderable { LIN_FORMAT } else { wgpu::TextureFormat::R16Float }
}
const AO_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;
const MAX_BLOOM_LEVELS: u32 = 6;

/// Mirrors `PostU` in post.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct PostUniform {
    inv_vp: [[f32; 4]; 4],
    size: [f32; 4],
    ao: [f32; 4],
    ssr: [f32; 4],
    bloom: [f32; 4],
    dof: [f32; 4],
    fog: [f32; 4],
    fog_color: [f32; 4],
    look: [f32; 4],
}

impl PostUniform {
    /// `inv_vp` = inverse of the frame's `view_proj` (target clip → relative position),
    /// `exposure` = the scene pass's exposure multiplier, `radius` = scene radius (mm).
    pub(crate) fn new(fx: &Effects, inv_vp: glam::Mat4, size: [u32; 2], ppp: f32, exposure: f32, radius: f32, gradient_bg: bool) -> Self {
        let on = |b: bool| if b { 1.0 } else { 0.0 };
        let hq = fx.high_quality;
        let fog_lin = fx.fog_color.map(super::srgb_to_linear);
        let e = exposure.max(1e-6);
        Self {
            inv_vp: inv_vp.to_cols_array_2d(),
            size: [size[0] as f32, size[1] as f32, 1.0 / size[0].max(1) as f32, 1.0 / size[1].max(1) as f32],
            ao: [fx.ao_radius.max(1.0), fx.ao_strength.max(0.0), if hq { 16.0 } else { 8.0 }, on(fx.ao_strength > 0.0)],
            ssr: [fx.reflections, if hq { 48.0 } else { 24.0 }, radius.max(1000.0), on(fx.reflections > 0.0)],
            bloom: [fx.bloom, fx.bloom_threshold.max(0.0), 0.0, on(fx.bloom > 0.0)],
            dof: [fx.focus_distance.max(0.0), fx.aperture.max(0.0) * 12.0 * ppp, if hq { 48.0 } else { 24.0 }, on(fx.aperture > 0.0)],
            fog: [
                if fx.fog_distance > 0.0 { std::f32::consts::LN_2 / fx.fog_distance } else { 0.0 },
                fx.fog_height.max(0.0) / 2000.0,
                fx.fog_distance * 4.0,
                on(fx.fog_distance > 0.0),
            ],
            fog_color: [fog_lin[0] / e, fog_lin[1] / e, fog_lin[2] / e, 1.0],
            look: [
                fx.vignette.clamp(0.0, 1.0),
                match fx.stylize {
                    Stylize::None => 0.0,
                    Stylize::Pencil => 1.0,
                    Stylize::Watercolor => 2.0,
                    Stylize::Technical => 3.0,
                },
                on(gradient_bg),
                ppp,
            ],
        }
    }
}

/// Pipelines and resources shared by all sizes.
pub(crate) struct Post {
    bgl: wgpu::BindGroupLayout,
    pub(crate) layout: wgpu::PipelineLayout,
    pub(crate) uniform: wgpu::Buffer,
    sampler: wgpu::Sampler,
    dummy_float: wgpu::TextureView,
    dummy_lin: wgpu::TextureView,
    lin_format: wgpu::TextureFormat,
    dummy_depth: wgpu::TextureView,
    pub(crate) pipes: PostPipelines,
}

pub(crate) struct PostPipelines {
    depth: wgpu::RenderPipeline,
    ao: wgpu::RenderPipeline,
    ao_blur: wgpu::RenderPipeline,
    composite: wgpu::RenderPipeline,
    dof: wgpu::RenderPipeline,
    bloom_pre: wgpu::RenderPipeline,
    bloom_down: wgpu::RenderPipeline,
    bloom_up: wgpu::RenderPipeline,
    /// Into the multisampled overlay pass (colour format of the renderer, depth attached).
    pub(crate) final_: wgpu::RenderPipeline,
}

fn tex(
    device: &wgpu::Device,
    label: &str,
    size: [u32; 2],
    format: wgpu::TextureFormat,
    samples: u32,
    mips: u32,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width: size[0].max(1), height: size[1].max(1), depth_or_array_layers: 1 },
        mip_level_count: mips,
        sample_count: samples,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

impl Post {
    pub(crate) fn new(
        device: &wgpu::Device,
        globals_bgl: &wgpu::BindGroupLayout,
        shader: &wgpu::ShaderModule,
        color_format: wgpu::TextureFormat,
        lin_format: wgpu::TextureFormat,
    ) -> Self {
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::FRAGMENT, ty, count: None };
        let float = |filterable| wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        };
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("varstotas post"),
            entries: &[
                entry(0, float(true)),
                entry(1, float(false)),
                entry(2, float(true)),
                entry(3, float(true)),
                entry(4, float(true)),
                entry(
                    5,
                    wgpu::BindingType::Texture {
                        // read with textureLoad as float (post.wgsl): GLSL has no depth textureLoad
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: true,
                    },
                ),
                entry(6, wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)),
                entry(
                    7,
                    wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                ),
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("varstotas post"),
            bind_group_layouts: &[Some(globals_bgl), Some(&bgl)],
            immediate_size: 0,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("varstotas post uniform"),
            size: std::mem::size_of::<PostUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("varstotas post"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let b = wgpu::TextureUsages::TEXTURE_BINDING;
        let view = |t: wgpu::Texture| t.create_view(&Default::default());
        let dummy_float = view(tex(device, "varstotas post dummy", [1, 1], HDR_FORMAT, 1, 1, b));
        let dummy_lin = view(tex(device, "varstotas post dummy depth", [1, 1], lin_format, 1, 1, b));
        let dummy_depth = view(tex(
            device,
            "varstotas post dummy ms depth",
            [1, 1],
            DEPTH_FORMAT,
            SAMPLES,
            1,
            b | wgpu::TextureUsages::RENDER_ATTACHMENT,
        ));
        let pipes = PostPipelines::new(device, &layout, shader, color_format, lin_format);
        Self { bgl, layout, uniform, sampler, dummy_float, dummy_lin, lin_format, dummy_depth, pipes }
    }

    pub(crate) fn rebuild(&mut self, device: &wgpu::Device, shader: &wgpu::ShaderModule, color_format: wgpu::TextureFormat) {
        self.pipes = PostPipelines::new(device, &self.layout, shader, color_format, self.lin_format);
    }
}

impl PostPipelines {
    fn new(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        color_format: wgpu::TextureFormat,
        lin_format: wgpu::TextureFormat,
    ) -> Self {
        let make = |label: &str, fs: &str, format: wgpu::TextureFormat, additive: bool, msaa: bool| {
            let add = wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add };
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(layout),
                vertex: wgpu::VertexState { module: shader, entry_point: Some("vs_post"), compilation_options: Default::default(), buffers: &[] },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: msaa.then(|| wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::Always),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState { count: if msaa { SAMPLES } else { 1 }, mask: !0, alpha_to_coverage_enabled: false },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: Some(fs),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: additive.then_some(wgpu::BlendState { color: add, alpha: add }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        Self {
            depth: make("varstotas post depth", "fs_post_depth", lin_format, false, false),
            ao: make("varstotas post ao", "fs_post_ao", AO_FORMAT, false, false),
            ao_blur: make("varstotas post ao blur", "fs_post_ao_blur", AO_FORMAT, false, false),
            composite: make("varstotas post composite", "fs_post_composite", HDR_FORMAT, false, false),
            dof: make("varstotas post dof", "fs_post_dof", HDR_FORMAT, false, false),
            bloom_pre: make("varstotas bloom prefilter", "fs_bloom_prefilter", HDR_FORMAT, false, false),
            bloom_down: make("varstotas bloom down", "fs_bloom_down", HDR_FORMAT, false, false),
            bloom_up: make("varstotas bloom up", "fs_bloom_up", HDR_FORMAT, true, false),
            final_: make("varstotas post final", "fs_post_final", color_format, false, true),
        }
    }
}

/// Render targets of one size and the bind groups of each pass.
pub(crate) struct PostTargets {
    pub(crate) size: [u32; 2],
    pub(crate) hdr_ms: wgpu::TextureView,
    pub(crate) hdr: wgpu::TextureView,
    pub(crate) gbuf_ms: wgpu::TextureView,
    pub(crate) gbuf: wgpu::TextureView,
    pub(crate) depth_ms: wgpu::TextureView,
    lin: wgpu::TextureView,
    ao_raw: wgpu::TextureView,
    ao: wgpu::TextureView,
    comp: wgpu::TextureView,
    dof: wgpu::TextureView,
    bloom_mips: Vec<wgpu::TextureView>,
    bg_depth: wgpu::BindGroup,
    bg_ao: wgpu::BindGroup,
    bg_blur: wgpu::BindGroup,
    bg_comp: wgpu::BindGroup,
    bg_dof: wgpu::BindGroup,
    bg_bloom_pre: wgpu::BindGroup,
    /// Down: level i from level i - 1 (index i - 1); up: level i from level i + 1 (index i).
    bg_bloom_down: Vec<wgpu::BindGroup>,
    bg_bloom_up: Vec<wgpu::BindGroup>,
    bg_final_comp: wgpu::BindGroup,
    bg_final_dof: wgpu::BindGroup,
}

impl PostTargets {
    pub(crate) fn new(device: &wgpu::Device, post: &Post, size: [u32; 2]) -> Self {
        let att = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let bind = wgpu::TextureUsages::TEXTURE_BINDING;
        let view = |t: &wgpu::Texture| t.create_view(&Default::default());
        let hdr_ms = view(&tex(device, "varstotas post hdr ms", size, HDR_FORMAT, SAMPLES, 1, att));
        let hdr = view(&tex(device, "varstotas post hdr", size, HDR_FORMAT, 1, 1, att | bind));
        let gbuf_ms = view(&tex(device, "varstotas post gbuf ms", size, GBUF_FORMAT, SAMPLES, 1, att));
        let gbuf = view(&tex(device, "varstotas post gbuf", size, GBUF_FORMAT, 1, 1, att | bind));
        let depth_ms = view(&tex(device, "varstotas post depth ms", size, DEPTH_FORMAT, SAMPLES, 1, att | bind));
        let lin = view(&tex(device, "varstotas post linear depth", size, post.lin_format, 1, 1, att | bind));
        let ao_raw = view(&tex(device, "varstotas post ao raw", size, AO_FORMAT, 1, 1, att | bind));
        let ao = view(&tex(device, "varstotas post ao", size, AO_FORMAT, 1, 1, att | bind));
        let comp = view(&tex(device, "varstotas post composite", size, HDR_FORMAT, 1, 1, att | bind));
        let dof = view(&tex(device, "varstotas post dof", size, HDR_FORMAT, 1, 1, att | bind));
        let half = [(size[0] / 2).max(1), (size[1] / 2).max(1)];
        let mut levels = 1;
        while levels < MAX_BLOOM_LEVELS && (half[0] >> levels).min(half[1] >> levels) >= 8 {
            levels += 1;
        }
        let bloom = tex(device, "varstotas bloom", half, HDR_FORMAT, 1, levels, att | bind);
        let bloom_all = view(&bloom);
        let bloom_mips: Vec<wgpu::TextureView> = (0..levels)
            .map(|m| bloom.create_view(&wgpu::TextureViewDescriptor { base_mip_level: m, mip_level_count: Some(1), ..Default::default() }))
            .collect();
        let group = |src: &wgpu::TextureView,
                     lin: &wgpu::TextureView,
                     ao: &wgpu::TextureView,
                     bloom: &wgpu::TextureView,
                     depth: &wgpu::TextureView| {
            let views = [src, lin, &gbuf, ao, bloom, depth];
            let mut entries: Vec<wgpu::BindGroupEntry> = views
                .iter()
                .enumerate()
                .map(|(i, v)| wgpu::BindGroupEntry { binding: i as u32, resource: wgpu::BindingResource::TextureView(v) })
                .collect();
            entries.push(wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::Sampler(&post.sampler) });
            entries.push(wgpu::BindGroupEntry { binding: 7, resource: post.uniform.as_entire_binding() });
            device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("varstotas post"), layout: &post.bgl, entries: &entries })
        };
        let (df, dl, dd) = (&post.dummy_float, &post.dummy_lin, &post.dummy_depth);
        let bg_depth = group(&hdr, dl, &ao, &bloom_all, &depth_ms);
        let bg_ao = group(&hdr, &lin, &ao, &bloom_all, &depth_ms);
        let bg_blur = group(&ao_raw, &lin, df, &bloom_all, &depth_ms);
        let bg_comp = group(&hdr, &lin, &ao, &bloom_all, &depth_ms);
        let bg_dof = group(&comp, &lin, &ao, &bloom_all, &depth_ms);
        let bg_bloom_pre = group(&comp, &lin, &ao, df, &depth_ms);
        let bg_bloom_down = (1..levels as usize).map(|i| group(&bloom_mips[i - 1], &lin, &ao, df, &depth_ms)).collect();
        let bg_bloom_up = (0..levels as usize - 1).map(|i| group(&bloom_mips[i + 1], &lin, &ao, df, &depth_ms)).collect();
        let bg_final_comp = group(&comp, &lin, &ao, &bloom_all, dd);
        let bg_final_dof = group(&dof, &lin, &ao, &bloom_all, dd);
        Self {
            size,
            hdr_ms,
            hdr,
            gbuf_ms,
            gbuf,
            depth_ms,
            lin,
            ao_raw,
            ao,
            comp,
            dof,
            bloom_mips,
            bg_depth,
            bg_ao,
            bg_blur,
            bg_comp,
            bg_dof,
            bg_bloom_pre,
            bg_bloom_down,
            bg_bloom_up,
            bg_final_comp,
            bg_final_dof,
        }
    }

    /// The full-screen passes between the scene pass and the overlay pass.
    pub(crate) fn encode(&self, enc: &mut wgpu::CommandEncoder, post: &Post, globals: &wgpu::BindGroup, fx: &Effects) {
        let p = &post.pipes;
        let run = |enc: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, pipe: &wgpu::RenderPipeline, bg: &wgpu::BindGroup, clear: bool| {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("varstotas post"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if clear { wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT) } else { wgpu::LoadOp::Load },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, globals, &[]);
            pass.set_bind_group(1, bg, &[]);
            pass.draw(0..3, 0..1);
        };
        run(enc, &self.lin, &p.depth, &self.bg_depth, true);
        if fx.ao_strength > 0.0 {
            run(enc, &self.ao_raw, &p.ao, &self.bg_ao, true);
            run(enc, &self.ao, &p.ao_blur, &self.bg_blur, true);
        }
        run(enc, &self.comp, &p.composite, &self.bg_comp, true);
        if fx.aperture > 0.0 {
            run(enc, &self.dof, &p.dof, &self.bg_dof, true);
        }
        if fx.bloom > 0.0 {
            let n = self.bloom_mips.len();
            run(enc, &self.bloom_mips[0], &p.bloom_pre, &self.bg_bloom_pre, true);
            for i in 1..n {
                run(enc, &self.bloom_mips[i], &p.bloom_down, &self.bg_bloom_down[i - 1], true);
            }
            for i in (0..n - 1).rev() {
                run(enc, &self.bloom_mips[i], &p.bloom_up, &self.bg_bloom_up[i], false);
            }
        }
    }

    /// Bind group of the final pass (reads the depth-of-field result when it ran).
    pub(crate) fn final_group(&self, fx: &Effects) -> &wgpu::BindGroup {
        if fx.aperture > 0.0 { &self.bg_final_dof } else { &self.bg_final_comp }
    }
}
