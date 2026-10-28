//! wgpu renderer: offscreen MSAA color+depth, resolved into a texture registered with egui.

use crate::wgpu;
use crate::{
    AxesFrame, Camera, FaceStyle, Highlight, HighlightUpdate, LineVertex, Overlay3d, RenderStyle, SceneGeometry,
    TexVertex, TextureData, TriVertex,
};
use glam::{DMat4, DVec2, DVec3};

const SAMPLES: u32 = 4;

/// Straight-alpha colour blending; alpha accumulates with the "over" operator
/// (a = a_src + a_dst * (1 - a_src)). On the normal opaque background this is identical to
/// `ALPHA_BLENDING`; on a transparent export background it leaves premultiplied colour with
/// correct coverage alpha (un-premultiplied on readback).
const BLEND: wgpu::BlendState = wgpu::BlendState {
    color: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::SrcAlpha,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    },
    alpha: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    },
};

/// Dash period of `OverlayLine::dashed` lines in logical points (60 % on).
pub const OVERLAY_DASH_PERIOD: f32 = 9.0;

#[path = "export.rs"]
mod export;
#[path = "annot.rs"]
mod annot;
#[path = "env.rs"]
mod env;
pub use env::{exposure_mult, srgb_to_linear};
#[path = "post.rs"]
mod post;
use annot::{LineInst, ScreenProj, SpriteVertex, TextAtlas};
pub use export::{ExportError, ExportOptions, MAX_EXPORT_DIM, RgbaImage, encode_exr};
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Uniform block, mirrors `Globals` in shader.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
    origin: [f32; 4],
    eye_rel: [f32; 4],
    right: [f32; 4],
    up: [f32; 4],
    fwd: [f32; 4],
    light: [f32; 4],
    viewport: [f32; 4],
    sky_top: [f32; 4],
    sky_horizon: [f32; 4],
    ground: [f32; 4],
    edge_color: [f32; 4],
    mono_front: [f32; 4],
    mono_back: [f32; 4],
    hidden_face: [f32; 4],
    params: [f32; 4],
    params2: [f32; 4],
    /// rgb: selection tint (sRGB); w: face mix factor.
    hl_selected: [f32; 4],
    /// rgb: dimmed-face target (sRGB); w: face mix factor.
    hl_dim_face: [f32; 4],
    /// Dimmed edge color (linear).
    hl_dim_edge: [f32; 4],
    /// Export tiling: tile NDC = full-image NDC * xy + zw. `TILE_FULL` = the whole image.
    tile: [f32; 4],
    /// Active section cut in the local frame: keep `dot(xyz, p) >= w`; off = `(0, 0, 0, -1)`.
    section: [f32; 4],
    /// Local frame -> shadow map clip space (x, y in -1..1, z in 0..1).
    shadow_vp: [[f32; 4]; 4],
    /// x: faces receive shadows; y: ground receives shadows; z: light shade; w: dark shade.
    shadow: [f32; 4],
    /// x: normal offset of receivers (local units); y: 1 / map size; z: 1 = sun above the horizon.
    shadow2: [f32; 4],
    /// Grid: xyz origin relative to the render origin; w: finest step.
    grid_o: [f32; 4],
    /// xyz: grid u axis; w: major ratio.
    grid_u: [f32; 4],
    /// xyz: grid v axis; w: 1 = shown.
    grid_v: [f32; 4],
    /// Grid line colour (linear), alpha = strength.
    grid_color: [f32; 4],
}

/// Shadow map side in texels (clamped to the device limit).
const SHADOW_MAP_SIZE: u32 = 4096;
const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Uniform of the shadow pass, mirrors `ShadowU` in shader.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShadowUniform {
    vp: [[f32; 4]; 4],
    section: [f32; 4],
}

/// What the shadow map was rendered for.
#[derive(Clone, Copy, PartialEq)]
struct ShadowKey {
    revision: u64,
    sun: [f64; 3],
    section: Option<[f64; 4]>,
}

/// Sun shadow map: depth from an orthographic view along the sun, fitted to the scene bounds.
/// Re-rendered only when the geometry, the sun or the section cut changes.
struct ShadowMap {
    size: u32,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    key: Option<ShadowKey>,
    /// Local frame -> light clip of the current map.
    vp: glam::Mat4,
    /// World size of one texel.
    texel: f32,
}

const TILE_FULL: [f32; 4] = [1.0, 1.0, 0.0, 0.0];

/// `Globals::params2.z`: highlight states the shader ignores (bit 0 Selected, bit 1 Dimmed).
const HL_SHOW_ALL: u32 = 0;
const HL_HIDE_SELECTED: u32 = 1;
const HL_HIDE_DIMMED: u32 = 2;

/// Per-frame overlay / axes / blend draw counts produced by `prepare_frame`.
#[derive(Clone, Copy, Default)]
struct FrameCounts {
    lines_depth: u32,
    lines_top: u32,
    polys_depth: u32,
    polys_top: u32,
    sprites_depth: u32,
    sprites_top: u32,
    axes: u32,
    blend: u32,
}

/// How `encode_scene` draws the frame.
#[derive(Clone, Copy)]
struct PassOpts {
    /// Pixels per logical point (line widths, HiDPI edge path).
    ppp: f32,
    /// Sky + ground (false = transparent background: clear to 0 and skip both).
    background: bool,
    /// Draw the Wireframe selection tint pass.
    selection_tint: bool,
}

/// One thick-line segment instance (screen-space quad expansion in the vertex shader).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ThickLine {
    a: [f32; 3],
    /// Width in logical points; negative = use the style's profile width.
    width: f32,
    b: [f32; 3],
    /// Dash period in points; 0 = solid; negative = dotted with |period|.
    dash: f32,
    color: [f32; 4],
}

/// Grow-only GPU buffer for per-frame data.
struct DynBuffer {
    buf: wgpu::Buffer,
    cap: u64,
    usage: wgpu::BufferUsages,
    label: &'static str,
}

impl DynBuffer {
    fn new(device: &wgpu::Device, label: &'static str, usage: wgpu::BufferUsages) -> Self {
        let cap = 4096;
        let usage = usage | wgpu::BufferUsages::COPY_DST;
        let buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: cap,
            usage,
            mapped_at_creation: false,
        });
        Self { buf, cap, usage, label }
    }

    fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        let need = data.len() as u64;
        if need > self.cap {
            self.cap = need.next_power_of_two();
            self.buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label),
                size: self.cap,
                usage: self.usage,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.buf, 0, data);
    }
}

struct Targets {
    size: [u32; 2],
    msaa_view: wgpu::TextureView,
    depth_view: wgpu::TextureView,
    /// Resolve target view used as render attachment.
    resolve_view: wgpu::TextureView,
    _resolve: wgpu::Texture,
}

struct StaticGeo {
    /// Bumped on every `set_geometry` (invalidates the translucent sort).
    revision: u64,
    /// World position of the local frame the f32 vertex positions are relative to.
    origin: DVec3,
    /// Triangle centroids (local frame), same order as the vertex buffer (opaque, then translucent).
    centroids: Vec<[f32; 3]>,
    /// Bounds of the triangles (local frame); `None` = no faces.
    bounds: Option<(glam::Vec3, glam::Vec3)>,
    tris: Option<wgpu::Buffer>,
    backs: Option<wgpu::Buffer>,
    /// Vertex counts: opaque range first, then translucent.
    /// Triangle vertices in `tris` (source order, never reordered).
    face_verts: u32,
    /// Source indices of translucent triangles (blend pass when not X-ray).
    translucent: Vec<u32>,
    /// Index buffer of the opaque triangles; only used when some triangles are translucent
    /// (otherwise the opaque pass draws `0..face_verts` directly).
    opaque_index: Option<wgpu::Buffer>,
    opaque_index_count: u32,
    edges: Option<wgpu::Buffer>,
    /// Same edges as thick-line instances (used on HiDPI: 1 logical point wide).
    edges_thick: Option<wgpu::Buffer>,
    edge_verts: u32,
    profiles: Option<wgpu::Buffer>,
    profile_count: u32,
    /// Storage-buffer bind groups for `edges_thick` / `profiles` (fast non-instanced path).
    edges_bg: Option<wgpu::BindGroup>,
    profiles_bg: Option<wgpu::BindGroup>,
    radius: f64,
    /// Highlight group of each triangle (vertex-buffer order) / edge segment / profile segment.
    tri_groups: Vec<u32>,
    edge_groups: Vec<u32>,
    /// Empty = profiles are never highlighted.
    profile_groups: Vec<u32>,
    /// Highlight state streams (`Highlight as u32`): per face vertex, per edge vertex (line list),
    /// per edge segment, per profile segment. Zero-initialised by `set_geometry`.
    face_states: Option<wgpu::Buffer>,
    /// `TexVertex` per face vertex (zeros when the scene has no textures).
    face_tex: Option<wgpu::Buffer>,
    /// Leading `TexVertex`es of `face_tex` known to be zero (skip re-uploading zeros).
    face_tex_zeros: usize,
    edge_states_vtx: Option<wgpu::Buffer>,
    edge_states_seg: Option<wgpu::Buffer>,
    profile_states: Option<wgpu::Buffer>,
    /// Some face is `Highlight::Selected` (Wireframe draws a selection tint only then).
    any_face_selected: bool,
}

/// CPU scratch reused by `set_geometry` (avoids re-allocating / page-faulting large vectors).
#[derive(Default)]
struct GeoScratch {
    backs: Vec<[f32; 4]>,
    tex: Vec<TexVertex>,
    opaque_idx: Vec<u32>,
    thick: Vec<ThickLine>,
    zeros: Vec<u32>,
}

/// What the current back-to-front index buffer was sorted for.
#[derive(Clone, Copy, PartialEq)]
struct SortKey {
    revision: u64,
    /// true = all triangles (X-ray), false = only the translucent range.
    all: bool,
    eye: [f64; 3],
    fwd: [f64; 3],
    ortho: bool,
}

/// Back-to-front order of the blended triangles, kept in a persistent index buffer and
/// re-sorted only when the camera, the blended set or the geometry changes.
struct BlendSort {
    key: Option<SortKey>,
    keys: Vec<u32>,
    tris: Vec<u32>,
    tmp_keys: Vec<u32>,
    tmp_tris: Vec<u32>,
    indices: Vec<u32>,
    buf: DynBuffer,
    /// Number of indices in `buf` valid for `key`.
    count: u32,
}

/// Optional frame timing (`VARSTOTAS_RENDER_TIMING=1`): averages logged to stderr every 120 frames.
/// GPU time is measured by blocking until the frame's submission completes (debug only).
#[derive(Default)]
struct Timing {
    frames: u32,
    cpu: std::time::Duration,
    sort: std::time::Duration,
    gpu: std::time::Duration,
    sorts: u32,
}

struct Pipelines {
    sky: wgpu::RenderPipeline,
    ground: wgpu::RenderPipeline,
    grid: wgpu::RenderPipeline,
    face_opaque: wgpu::RenderPipeline,
    /// Translucent / X-ray faces: indexed back-to-front, no cull, no depth write.
    face_blend: wgpu::RenderPipeline,
    edge_thick: wgpu::RenderPipeline,
    /// Same as `edge_thick` / `thick_depth`, but segments are read from a storage buffer and
    /// drawn as a plain (non-instanced) triangle list: tiny 6-vertex instances are slow on
    /// tile-based GPUs. `None` when vertex-stage storage buffers are unavailable (GL).
    edge_thick_seg: Option<wgpu::RenderPipeline>,
    thick_depth_seg: Option<wgpu::RenderPipeline>,
    /// Instanced profiles with a per-segment highlight stream (fallback when `thick_depth_seg` is None).
    thick_depth_state: wgpu::RenderPipeline,
    edge: wgpu::RenderPipeline,
    thick_depth: wgpu::RenderPipeline,
    thick_top: wgpu::RenderPipeline,
    poly_depth: wgpu::RenderPipeline,
    poly_top: wgpu::RenderPipeline,
    /// Text / marks: screen-aligned sprites anchored at 3D points (glyph atlas in group 1).
    sprite_depth: wgpu::RenderPipeline,
    sprite_top: wgpu::RenderPipeline,
}

/// Pipeline layouts kept to rebuild the pipelines with another shader module (custom surfaces).
struct Layouts {
    scene: wgpu::PipelineLayout,
    face: wgpu::PipelineLayout,
    seg: Option<wgpu::PipelineLayout>,
    text: wgpu::PipelineLayout,
}

pub struct Renderer {
    color_format: wgpu::TextureFormat,
    layouts: Layouts,
    shader: wgpu::ShaderModule,
    /// Its WGSL (the path tracer composes its ray query variant from it).
    shader_src: String,
    /// Custom surfaces of the current module and their errors (`set_custom_shaders`).
    custom: Vec<crate::CustomSurface>,
    custom_errors: Vec<Option<String>>,
    /// Scene pipelines of the effects path (float target + G-buffer), made on first use.
    pipelines_hdr: Option<Pipelines>,
    post: Option<post::Post>,
    post_targets: Option<post::PostTargets>,
    /// Format of the view handed to egui (non-sRGB view of the same texture).
    egui_view_format: wgpu::TextureFormat,
    manual_gamma: bool,
    pipelines: Pipelines,
    globals: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    /// Layout of group 0 (globals + shadow map), kept to rebuild `bind_group` when the map grows.
    bgl: wgpu::BindGroupLayout,
    shadow: ShadowMap,
    /// Environment of `FaceStyle::Realistic` (group 0, bindings 3..5).
    env: env::Env,
    targets: Option<Targets>,
    texture_id: Option<egui::TextureId>,
    geo: StaticGeo,
    overlay_lines: DynBuffer,
    overlay_polys: DynBuffer,
    axes: DynBuffer,
    sprites: DynBuffer,
    text: TextAtlas,
    face_textures: crate::textures::FaceTextures,
    /// Frame of the main axes (`None` = world axes).
    axes_frame: Option<AxesFrame>,
    /// Active section cut (world): keep points with `n · p >= d`, as `[n.x, n.y, n.z, d]`.
    section: Option<[f64; 4]>,
    blend: BlendSort,
    /// Reused per-frame scratch (no per-frame allocations).
    scratch_lines: Vec<LineInst>,
    scratch_polys: Vec<LineVertex>,
    scratch_axes: Vec<LineInst>,
    scratch_sprites: Vec<SpriteVertex>,
    scratch_states: Vec<u32>,
    scratch_geo: GeoScratch,
    max_buffer: u64,
    max_texture_dim: u32,
    /// Layout of the segment storage bind group (group 1); `None` = instanced fallback only.
    seg_bgl: Option<wgpu::BindGroupLayout>,
    max_storage_binding: u64,
    timing: Option<Timing>,
    /// The path tracer (`pt_*`), made on first use.
    pt: Option<crate::pathtrace::PathTracer>,
}

/// An sRGB colour with alpha as linear light (alpha kept).
fn srgb4_to_linear(c: [f32; 4]) -> [f32; 4] {
    [srgb_to_linear(c[0]), srgb_to_linear(c[1]), srgb_to_linear(c[2]), c[3]]
}

fn hex(rgb: u32, a: f32) -> [f32; 4] {
    [
        ((rgb >> 16) & 255) as f32 / 255.0,
        ((rgb >> 8) & 255) as f32 / 255.0,
        (rgb & 255) as f32 / 255.0,
        a,
    ]
}

impl Default for RenderStyle {
    fn default() -> Self {
        RenderStyle {
            face_style: FaceStyle::Shaded,
            xray: false,
            show_edges: true,
            show_triangles: false,
            profiles: true,
            profile_width_px: 2.5,
            show_axes: true,
            show_ground: true,
            sky_top: hex(0xA8C8E8, 1.0),
            sky_horizon: hex(0xE8F0F8, 1.0),
            ground: hex(0xC8C0B0, 0.9),
            edge_color: hex(0x000000, 1.0),
            light_dir: DEFAULT_LIGHT_DIR,
            ambient: DEFAULT_AMBIENT,
            shadows: None,
            grid: None,
            env: crate::EnvStyle::default(),
        }
    }
}

/// Default sun direction (toward the light, world space): from upper-left-front.
pub const DEFAULT_LIGHT_DIR: [f32; 3] = [-0.4, -0.6, 0.7];
/// Default ambient term (0..1, perceptual): faces turned away from the sun get this shade.
pub const DEFAULT_AMBIENT: f32 = 0.68;

/// Highlight colors applied by the renderer for `Highlight` states (sRGB; same as the app's
/// previous baked-in selection colors). Faces mix toward the color, edges take it as-is.
pub const HIGHLIGHT_SELECTED: [f32; 4] = [0.0, 0x50 as f32 / 255.0, 1.0, 1.0];
pub const HIGHLIGHT_FACE_MIX: f32 = 0.45;
pub const DIM_FACE_TARGET: [f32; 4] = [0.86, 0.86, 0.87, 1.0];
pub const DIM_FACE_MIX: f32 = 0.6;
pub const DIM_EDGE_COLOR: [f32; 4] = [0.62, 0.62, 0.64, 1.0];

/// Default face colors used by `FaceStyle::Monochrome` (sRGB).
pub const DEFAULT_FRONT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const DEFAULT_BACK_COLOR: [f32; 4] = [0xA4 as f32 / 255.0, 0xB0 as f32 / 255.0, 0xC8 as f32 / 255.0, 1.0];

impl Renderer {
    /// Create from eframe's wgpu render state (`frame.wgpu_render_state()`).
    pub fn new(rs: &egui_wgpu::RenderState) -> Self {
        let device = &rs.device;
        let view_formats_ok = rs
            .adapter
            .get_downlevel_capabilities()
            .flags
            .contains(wgpu::DownlevelFlags::VIEW_FORMATS);
        let (color_format, egui_view_format, manual_gamma) = if view_formats_ok {
            (
                wgpu::TextureFormat::Rgba8UnormSrgb,
                wgpu::TextureFormat::Rgba8Unorm,
                false,
            )
        } else {
            (wgpu::TextureFormat::Rgba8Unorm, wgpu::TextureFormat::Rgba8Unorm, true)
        };

        let shader_src = crate::custom::compose(&[]).0;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("varstotas scene shader"),
            source: wgpu::ShaderSource::Wgsl(shader_src.clone().into()),
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("varstotas globals"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("varstotas globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let shadow = ShadowMap::new(device, &shader);
        let env = env::Env::new(device, &rs.queue);
        let bind_group = globals_bind_group(device, &bgl, &globals, &shadow, &env);
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("varstotas"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });

        let limits = device.limits();
        let seg_bgl = (limits.max_storage_buffers_per_shader_stage > 0
            && rs.adapter.get_info().backend != wgpu::Backend::Gl)
            .then(|| {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("varstotas segments"),
                    entries: &[0, 1].map(|binding| wgpu::BindGroupLayoutEntry {
                        binding,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }),
                })
            });
        let seg_layout = seg_bgl.as_ref().map(|sb| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("varstotas segments"),
                bind_group_layouts: &[Some(&bgl), Some(sb)],
                immediate_size: 0,
            })
        });
        let text = TextAtlas::new(device, limits.max_texture_dimension_2d);
        let text_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("varstotas sprites"),
            bind_group_layouts: &[Some(&bgl), Some(&text.bgl)],
            immediate_size: 0,
        });
        let face_textures = crate::textures::FaceTextures::new(device);
        let face_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("varstotas faces"),
            bind_group_layouts: &[Some(&bgl), Some(&face_textures.bgl)],
            immediate_size: 0,
        });
        let layouts = Layouts { scene: layout, face: face_layout, seg: seg_layout, text: text_layout };
        let pipelines = build_pipelines(device, &shader, &layouts, color_format, false);
        let vb = wgpu::BufferUsages::VERTEX;
        Renderer {
            color_format,
            layouts,
            shader,
            shader_src,
            custom: Vec::new(),
            custom_errors: Vec::new(),
            pipelines_hdr: None,
            post: None,
            post_targets: None,
            egui_view_format,
            manual_gamma,
            pipelines,
            globals,
            bind_group,
            bgl,
            shadow,
            env,
            targets: None,
            texture_id: None,
            geo: StaticGeo {
                revision: 0,
                origin: DVec3::ZERO,
                centroids: Vec::new(),
                bounds: None,
                tris: None,
                backs: None,
                face_verts: 0,
                translucent: Vec::new(),
                opaque_index: None,
                opaque_index_count: 0,
                edges: None,
                edges_thick: None,
                edge_verts: 0,
                profiles: None,
                profile_count: 0,
                edges_bg: None,
                profiles_bg: None,
                tri_groups: Vec::new(),
                edge_groups: Vec::new(),
                profile_groups: Vec::new(),
                face_states: None,
                face_tex: None,
                face_tex_zeros: 0,
                edge_states_vtx: None,
                edge_states_seg: None,
                profile_states: None,
                any_face_selected: false,
                radius: 0.0,
            },
            overlay_lines: DynBuffer::new(device, "varstotas overlay lines", vb),
            overlay_polys: DynBuffer::new(device, "varstotas overlay polys", vb),
            axes: DynBuffer::new(device, "varstotas axes", vb),
            sprites: DynBuffer::new(device, "varstotas overlay sprites", vb),
            text,
            face_textures,
            axes_frame: None,
            section: None,
            blend: BlendSort {
                key: None,
                keys: Vec::new(),
                tris: Vec::new(),
                tmp_keys: Vec::new(),
                tmp_tris: Vec::new(),
                indices: Vec::new(),
                buf: DynBuffer::new(device, "varstotas blend indices", wgpu::BufferUsages::INDEX),
                count: 0,
            },
            scratch_lines: Vec::new(),
            scratch_polys: Vec::new(),
            scratch_axes: Vec::new(),
            scratch_sprites: Vec::new(),
            scratch_states: Vec::new(),
            scratch_geo: GeoScratch::default(),
            max_buffer: device.limits().max_buffer_size,
            max_texture_dim: limits.max_texture_dimension_2d,
            seg_bgl,
            max_storage_binding: limits.max_storage_buffer_binding_size,
            timing: std::env::var("VARSTOTAS_RENDER_TIMING")
                .is_ok_and(|v| !v.is_empty() && v != "0")
                .then(Timing::default),
            pt: None,
        }
    }

    /// Upload new static geometry. GPU buffers are reused when the new data fits (grow-only),
    /// opaque-only scenes are uploaded straight from `geo` (no partition copy), and CPU scratch
    /// vectors are kept between calls. Resets all highlight states to `Highlight::None`.
    pub fn set_geometry(&mut self, rs: &egui_wgpu::RenderState, geo: &SceneGeometry) {
        let t0 = std::time::Instant::now();
        let device = &rs.device;
        let queue = &rs.queue;
        let max_tris_verts = (self.max_buffer / std::mem::size_of::<TriVertex>() as u64) as usize;
        let mut n = geo.tris.len() - geo.tris.len() % 3;
        if n > max_tris_verts {
            log::warn!(
                "varstotas_render: {n} triangle vertices exceed the device buffer limit, truncating to {max_tris_verts}"
            );
            n = max_tris_verts - max_tris_verts % 3;
        }
        let src = &geo.tris[..n];
        let back_of = |i: usize| geo.back_colors.get(i).copied().unwrap_or(DEFAULT_BACK_COLOR);
        let group_of = |t: usize| geo.face_ids.get(t).copied().unwrap_or(t as u32);
        let ntris = n / 3;
        let sc = &mut self.scratch_geo;
        let g = &mut self.geo;

        // ---- faces: uploaded in source order; translucent triangles are drawn via indices ----
        g.translucent.clear();
        g.translucent.extend((0..ntris as u32).filter(|&t| {
            let t = t as usize;
            (t * 3..t * 3 + 3).any(|k| src[k].color[3] < 0.999 || back_of(k)[3] < 0.999)
        }));
        let translucent_tris = g.translucent.len();
        g.tri_groups.clear();
        g.tri_groups.extend((0..ntris).map(group_of));
        let backs: &[[f32; 4]] = if geo.back_colors.len() >= n {
            &geo.back_colors[..n]
        } else {
            sc.backs.clear();
            sc.backs.extend((0..n).map(back_of));
            &sc.backs
        };
        let tris = src;
        let usage_v = wgpu::BufferUsages::VERTEX;
        upload(device, queue, &mut g.tris, "varstotas tris", usage_v, bytemuck::cast_slice(tris));
        upload(device, queue, &mut g.backs, "varstotas backs", usage_v, bytemuck::cast_slice(backs));
        if geo.tex.len() >= n {
            upload(device, queue, &mut g.face_tex, "varstotas face tex", usage_v, bytemuck::cast_slice(&geo.tex[..n]));
            g.face_tex_zeros = 0;
        } else if g.face_tex_zeros < n || g.face_tex.is_none() {
            sc.tex.clear();
            sc.tex.resize(n, TexVertex::default());
            let old = g.face_tex.as_ref().map(|b| b.size());
            upload(device, queue, &mut g.face_tex, "varstotas face tex", usage_v, bytemuck::cast_slice(&sc.tex));
            // A fresh (grown) buffer is zero-initialised past `n` too.
            let size = g.face_tex.as_ref().map_or(0, |b| b.size()) as usize / std::mem::size_of::<TexVertex>();
            g.face_tex_zeros = if old != g.face_tex.as_ref().map(|b| b.size()) { size } else { n };
        }
        g.face_verts = n as u32;
        g.opaque_index_count = 0;
        if translucent_tris > 0 {
            sc.opaque_idx.clear();
            let mut next = g.translucent.iter().copied().peekable();
            for t in 0..ntris as u32 {
                if next.peek() == Some(&t) {
                    next.next();
                } else {
                    sc.opaque_idx.extend([t * 3, t * 3 + 1, t * 3 + 2]);
                }
            }
            g.opaque_index_count = sc.opaque_idx.len() as u32;
            upload(
                device,
                queue,
                &mut g.opaque_index,
                "varstotas opaque indices",
                wgpu::BufferUsages::INDEX,
                bytemuck::cast_slice(&sc.opaque_idx),
            );
        }
        g.centroids.clear();
        g.centroids.extend(tris.as_chunks::<3>().0.iter().map(|t| {
            let c = (glam::Vec3::from(t[0].pos) + glam::Vec3::from(t[1].pos) + glam::Vec3::from(t[2].pos)) / 3.0;
            c.to_array()
        }));
        g.bounds = tris.iter().map(|v| glam::Vec3::from(v.pos)).filter(|p| p.is_finite()).fold(None, |b, p| {
            Some(b.map_or((p, p), |(lo, hi): (glam::Vec3, glam::Vec3)| (lo.min(p), hi.max(p))))
        });
        g.revision += 1;
        self.blend.key = None;

        // ---- edges & profiles -------------------------------------------------------------
        // Storage path: the thick-line shader reads the raw `LineVertex` pairs (no conversion,
        // one buffer shared with the 1 px line-list path). Fallback: instanced `ThickLine`s.
        let seg_bytes = |segs: usize| (segs * 2 * std::mem::size_of::<LineVertex>()) as u64;
        let can_seg = |segs: usize| self.seg_bgl.is_some() && segs > 0 && seg_bytes(segs) <= self.max_storage_binding;
        let seg_usage = if self.seg_bgl.is_some() {
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE
        } else {
            wgpu::BufferUsages::VERTEX
        };
        let ne = geo.edges.len() - geo.edges.len() % 2;
        let edge_segs = ne / 2;
        let edges_seg = can_seg(edge_segs);
        let edge_usage = if edges_seg { usage_v | wgpu::BufferUsages::STORAGE } else { usage_v };
        upload(device, queue, &mut g.edges, "varstotas edges", edge_usage, bytemuck::cast_slice(&geo.edges[..ne]));
        g.edge_verts = ne as u32;
        let to_thick = |p: &[LineVertex; 2], width: f32| ThickLine {
            a: p[0].pos,
            width,
            b: p[1].pos,
            dash: 0.0,
            color: p[0].color,
        };
        if !edges_seg {
            sc.thick.clear();
            sc.thick.extend(geo.edges[..ne].as_chunks::<2>().0.iter().map(|p| to_thick(p, 0.0))); // 0 = 1 pt
            upload(device, queue, &mut g.edges_thick, "varstotas edges thick", usage_v, bytemuck::cast_slice(&sc.thick));
        }
        g.edge_groups.clear();
        g.edge_groups.extend((0..edge_segs as u32).map(|i| geo.edge_ids.get(i as usize).copied().unwrap_or(i)));

        let np = geo.profile_edges.len() - geo.profile_edges.len() % 2;
        let profile_segs = np / 2;
        let profiles_seg = can_seg(profile_segs);
        if profiles_seg {
            let usage = usage_v | wgpu::BufferUsages::STORAGE;
            upload(device, queue, &mut g.profiles, "varstotas profiles", usage, bytemuck::cast_slice(&geo.profile_edges[..np]));
        } else {
            sc.thick.clear();
            sc.thick.extend(geo.profile_edges[..np].as_chunks::<2>().0.iter().map(|p| to_thick(p, -1.0)));
            upload(device, queue, &mut g.profiles, "varstotas profiles thick", usage_v, bytemuck::cast_slice(&sc.thick));
        }
        g.profile_count = profile_segs as u32;
        g.profile_groups.clear();
        if !geo.profile_ids.is_empty() {
            g.profile_groups
                .extend((0..profile_segs).map(|i| geo.profile_ids.get(i).copied().unwrap_or(u32::MAX)));
        }

        // ---- highlight states (reset to None) ---------------------------------------------
        sc.zeros.clear();
        sc.zeros.resize(n.max(ne).max(profile_segs), 0);
        let zeros = |count: usize| -> &[u8] { bytemuck::cast_slice(&sc.zeros[..count]) };
        let state_usage = wgpu::BufferUsages::VERTEX;
        upload(device, queue, &mut g.face_states, "varstotas face states", state_usage, zeros(n));
        upload(device, queue, &mut g.edge_states_vtx, "varstotas edge vertex states", state_usage, zeros(ne));
        upload(device, queue, &mut g.edge_states_seg, "varstotas edge states", seg_usage, zeros(edge_segs));
        upload(device, queue, &mut g.profile_states, "varstotas profile states", seg_usage, zeros(profile_segs));
        g.any_face_selected = false;

        let bind = |on: bool, segs: &Option<wgpu::Buffer>, states: &Option<wgpu::Buffer>, count: usize| {
            if !on {
                return None;
            }
            seg_bind_group(device, self.seg_bgl.as_ref()?, segs.as_ref()?, states.as_ref()?, count)
        };
        g.edges_bg = bind(edges_seg, &g.edges, &g.edge_states_seg, edge_segs);
        g.profiles_bg = bind(profiles_seg, &g.profiles, &g.profile_states, profile_segs);

        let origin = DVec3::from_array(geo.origin);
        g.origin = if origin.is_finite() { origin } else { DVec3::ZERO };
        // The camera treats the scene as a ball around the world origin.
        g.radius = geo.radius.max(0.0) as f64 + g.origin.length();
        if self.timing.is_some() {
            eprintln!(
                "varstotas_render timing: set_geometry {} tris ({} translucent), {} edges: {:.2} ms", // i18n-ignore
                ntris,
                translucent_tris,
                ne / 2,
                t0.elapsed().as_secs_f64() * 1000.0
            );
        }
    }

    /// Fast path for selection / editing-context changes: rewrites only the small per-vertex /
    /// per-segment highlight streams (no geometry re-upload, no re-sort, no re-partition).
    /// `faces[id]` / `edges[id]` are indexed by the ids given in `SceneGeometry::face_ids` /
    /// `edge_ids` / `profile_ids` (or the triangle / edge-segment index when those are empty);
    /// ids beyond the slices get `Highlight::None`. Replaces the whole previous state.
    pub fn update_highlight(&mut self, rs: &egui_wgpu::RenderState, update: &HighlightUpdate<'_>) {
        let t0 = std::time::Instant::now();
        let queue = &rs.queue;
        let state = |states: &[Highlight], id: u32| states.get(id as usize).map_or(0, |&h| h as u32);
        let write = |buf: &wgpu::Buffer, v: &[u32]| {
            if !v.is_empty() {
                queue.write_buffer(buf, 0, bytemuck::cast_slice(v));
            }
        };
        let g = &mut self.geo;
        let v = &mut self.scratch_states;

        if let Some(buf) = &g.face_states {
            v.clear();
            v.extend(g.tri_groups.iter().flat_map(|&id| [state(update.faces, id); 3]));
            write(buf, v);
            g.any_face_selected = v.contains(&(Highlight::Selected as u32));
        }
        if let Some(buf) = &g.edge_states_seg {
            v.clear();
            v.extend(g.edge_groups.iter().map(|&id| state(update.edges, id)));
            write(buf, v);
            if let Some(vb) = &g.edge_states_vtx {
                // Expand in place (back to front) to two vertices per segment.
                let n = v.len();
                v.resize(n * 2, 0);
                for i in (0..n).rev() {
                    v[2 * i + 1] = v[i];
                    v[2 * i] = v[i];
                }
                write(vb, v);
            }
        }
        if let Some(buf) = &g.profile_states
            && !g.profile_groups.is_empty()
        {
            v.clear();
            v.extend(g.profile_groups.iter().map(|&id| state(update.edges, id)));
            write(buf, v);
        }
        if self.timing.is_some() {
            eprintln!(
                "varstotas_render timing: update_highlight {} tris, {} edges: {:.3} ms", // i18n-ignore
                g.tri_groups.len(),
                g.edge_groups.len(),
                t0.elapsed().as_secs_f64() * 1000.0
            );
        }
    }

    /// Re-sort the blended triangles back-to-front if the camera or the set changed.
    /// Returns the number of indices to draw from `blend.buf`.
    fn update_blend_order(&mut self, rs: &egui_wgpu::RenderState, camera: &Camera, all: bool) -> u32 {
        let g = &self.geo;
        let n = if all { g.face_verts as usize / 3 } else { g.translucent.len() };
        if n == 0 {
            return 0;
        }
        let eye_local = camera.eye - g.origin;
        let fwd = camera.basis().forward;
        let key = SortKey {
            revision: g.revision,
            all,
            eye: eye_local.to_array(),
            fwd: fwd.to_array(),
            ortho: camera.ortho,
        };
        if self.blend.key == Some(key) {
            return self.blend.count;
        }
        let t0 = std::time::Instant::now();
        let s = &mut self.blend;
        let eye = eye_local.as_vec3();
        let f = fwd.as_vec3();
        s.tris.clear();
        if all {
            s.tris.extend(0..n as u32);
        } else {
            s.tris.extend_from_slice(&g.translucent);
        }
        let cents = &g.centroids;
        let c = |t: u32| glam::Vec3::from(cents[t as usize]) - eye;
        s.keys.clear();
        if camera.ortho {
            s.keys.extend(s.tris.iter().map(|&t| !depth_key(c(t).dot(f))));
        } else {
            s.keys.extend(s.tris.iter().map(|&t| !depth_key(c(t).length_squared())));
        }
        radix_sort_pairs(&mut s.keys, &mut s.tris, &mut s.tmp_keys, &mut s.tmp_tris);
        s.indices.clear();
        s.indices.extend(s.tris.iter().flat_map(|&t| [t * 3, t * 3 + 1, t * 3 + 2]));
        s.buf.write(&rs.device, &rs.queue, bytemuck::cast_slice(&s.indices));
        s.count = s.indices.len() as u32;
        s.key = Some(key);
        if let Some(t) = self.timing.as_mut() {
            t.sort += t0.elapsed();
            t.sorts += 1;
        }
        s.count
    }

    fn ensure_targets(&mut self, rs: &egui_wgpu::RenderState, size: [u32; 2]) {
        if self.targets.as_ref().is_some_and(|t| t.size == size) {
            return;
        }
        let device = &rs.device;
        let extent = wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        };
        let msaa = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("varstotas msaa color"),
            size: extent,
            mip_level_count: 1,
            sample_count: SAMPLES,
            dimension: wgpu::TextureDimension::D2,
            format: self.color_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("varstotas depth"),
            size: extent,
            mip_level_count: 1,
            sample_count: SAMPLES,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view_formats = [self.egui_view_format];
        let resolve = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("varstotas resolved color"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.color_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: if self.egui_view_format != self.color_format {
                &view_formats
            } else {
                &[]
            },
        });
        let resolve_view = resolve.create_view(&wgpu::TextureViewDescriptor::default());
        let egui_view = resolve.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.egui_view_format),
            ..Default::default()
        });
        {
            let mut r = rs.renderer.write();
            match self.texture_id {
                Some(id) => r.update_egui_texture_from_wgpu_texture(
                    device,
                    &egui_view,
                    wgpu::FilterMode::Linear,
                    id,
                ),
                None => {
                    self.texture_id =
                        Some(r.register_native_texture(device, &egui_view, wgpu::FilterMode::Linear))
                }
            }
        }
        self.targets = Some(Targets {
            size,
            msaa_view: msaa.create_view(&Default::default()),
            depth_view: depth.create_view(&Default::default()),
            resolve_view,
            _resolve: resolve,
        });
    }

    /// Render the scene offscreen at `size_px` (physical pixels) and return an egui texture
    /// id to display with `egui::Image` sized to the viewport in points.
    pub fn render(
        &mut self,
        rs: &egui_wgpu::RenderState,
        camera: &Camera,
        overlay: &Overlay3d,
        style: &RenderStyle,
        size_px: [u32; 2],
        pixels_per_point: f32,
    ) -> egui::TextureId {
        let t_start = std::time::Instant::now();
        let max_dim = self.max_texture_dim;
        let size = [size_px[0].clamp(1, max_dim), size_px[1].clamp(1, max_dim)];
        self.ensure_targets(rs, size);
        let ppp = if pixels_per_point > 0.0 { pixels_per_point } else { 1.0 };
        let device = &rs.device;
        let queue = &rs.queue;

        self.update_shadow_map(rs, style);
        self.update_env(rs, style);
        let globals = self.globals(camera, style, size, size, ppp, TILE_FULL, HL_SHOW_ALL);
        queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));
        let counts = self.prepare_frame(rs, camera, overlay, style, size, ppp);
        let post = self.prepare_post(rs, style, size);
        if post {
            self.write_post_uniform(rs, style, &globals, size, ppp);
        }

        let t = self.targets.as_ref().expect("targets");
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("varstotas render"),
        });
        let opts = PassOpts { ppp, background: true, selection_tint: true };
        let pt = if post { self.post_targets.as_ref() } else { None };
        self.encode_scene(&mut enc, &t.msaa_view, &t.depth_view, &t.resolve_view, pt, style, &counts, opts);
        let submission = queue.submit(Some(enc.finish()));
        if let Some(t) = self.timing.as_mut() {
            t.cpu += t_start.elapsed();
            let g0 = std::time::Instant::now();
            let _ = device.poll(wgpu::PollType::Wait { submission_index: Some(submission), timeout: None });
            t.gpu += g0.elapsed();
            t.frames += 1;
            if t.frames >= 120 {
                let ms = |d: std::time::Duration| d.as_secs_f64() * 1000.0 / t.frames as f64;
                eprintln!(
                    "varstotas_render timing ({} frames): cpu {:.3} ms/frame (incl. sort {:.3} ms, {} sorts), gpu wait {:.3} ms/frame", // i18n-ignore
                    t.frames,
                    ms(t.cpu),
                    ms(t.sort),
                    t.sorts,
                    ms(t.gpu)
                );
                *t = Timing::default();
            }
        }
        self.texture_id.expect("texture registered")
    }

    /// Uniforms for a frame of `image` pixels; `target` = size of the render target (the tile
    /// when exporting in tiles, otherwise `image`), `tile` maps whole-image NDC to tile NDC.
    #[allow(clippy::too_many_arguments)]
    fn globals(
        &self,
        camera: &Camera,
        style: &RenderStyle,
        image: [u32; 2],
        target: [u32; 2],
        ppp: f32,
        tile: [f32; 4],
        hide_mask: u32,
    ) -> Globals {
        let aspect = image[0] as f64 / image[1] as f64;
        let b = camera.basis();
        let scene_radius = self.geo.radius;
        // Relative-to-eye: vertices are f32 relative to `geo.origin` (world, f64). The shader
        // subtracts `origin` (≈ eye in that local frame, f32-representable) before the f32
        // matrix, so precision is relative to the camera, not to the world origin.
        let geo_origin = self.geo.origin;
        let origin = (camera.eye - geo_origin).as_vec3().as_dvec3();
        let tile_m = DMat4::from_cols_array(&[
            tile[0] as f64, 0.0, 0.0, 0.0,
            0.0, tile[1] as f64, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            tile[2] as f64, tile[3] as f64, 0.0, 1.0,
        ]);
        let view_proj: DMat4 = tile_m
            * camera.proj(aspect, scene_radius)
            * camera.view()
            * DMat4::from_translation(geo_origin + origin);
        let eye_rel = camera.eye - geo_origin - origin;
        let viewport_pts = DVec2::new(image[0] as f64 / ppp as f64, image[1] as f64 / ppp as f64);
        let (rw, uw) = if camera.ortho {
            let hh = camera.ortho_height * 0.5;
            (hh * aspect, hh)
        } else {
            let t = (camera.fov_y_deg.to_radians() * 0.5).tan();
            (t * aspect, t)
        };
        let ld = DVec3::new(
            style.light_dir[0] as f64,
            style.light_dir[1] as f64,
            style.light_dir[2] as f64,
        );
        let mut light = if ld.length_squared() > 1e-12 { ld.normalize() } else { DVec3::Z };
        let sh = style.shadows.filter(|s| DVec3::from_array(s.sun_dir).is_finite());
        if let Some(s) = sh {
            light = DVec3::from_array(s.sun_dir).normalize_or(DVec3::Z);
        }
        let (bias_rel, bias_abs) = if camera.ortho {
            let (n, f) = camera.ortho_depth_range(scene_radius);
            let px = camera.pixel_size_at(camera.target, viewport_pts);
            (0.0, (px * 0.75 / (f - n)).max(2e-6) as f32)
        } else {
            (4e-4f32, 0.0)
        };
        // Shading is done in linear space; uniforms are converted once here.
        let conv = srgb4_to_linear;
        let face_mode = match style.face_style {
            FaceStyle::Shaded => 0.0,
            // Wireframe: faces are only drawn as a flat selection tint (QA2-006).
            FaceStyle::Wireframe => 3.0,
            FaceStyle::Monochrome => 1.0,
            FaceStyle::HiddenLine => 2.0,
            FaceStyle::Realistic => 4.0,
        };
        let with_w = |c: [f32; 4], w: f32| [c[0], c[1], c[2], w];
        let v4 = |v: DVec3, w: f64| [v.x as f32, v.y as f32, v.z as f32, w as f32];
        let mut globals = Globals {
            view_proj: view_proj.as_mat4().to_cols_array_2d(),
            origin: v4(origin, geo_origin.z),
            eye_rel: v4(eye_rel, if camera.ortho { 1.0 } else { 0.0 }),
            right: v4(b.right, rw),
            up: v4(b.up, uw),
            fwd: v4(b.forward, 0.0),
            light: v4(light, style.ambient.clamp(0.0, 1.0) as f64),
            viewport: [
                target[0] as f32,
                target[1] as f32,
                ppp,
                (style.profile_width_px * ppp).max(1.0),
            ],
            sky_top: conv(style.sky_top),
            sky_horizon: conv(style.sky_horizon),
            ground: conv(style.ground),
            edge_color: conv(style.edge_color),
            mono_front: conv(DEFAULT_FRONT_COLOR),
            mono_back: conv(DEFAULT_BACK_COLOR),
            hidden_face: conv(style.sky_horizon),
            params: [
                face_mode,
                if style.xray { 0.5 } else { 1.0 },
                bias_rel,
                if self.manual_gamma { 1.0 } else { 0.0 },
            ],
            params2: [bias_abs, if style.show_ground { 1.0 } else { 0.0 }, hide_mask as f32, 0.0],
            hl_selected: with_w(HIGHLIGHT_SELECTED, HIGHLIGHT_FACE_MIX),
            hl_dim_face: with_w(DIM_FACE_TARGET, DIM_FACE_MIX),
            hl_dim_edge: conv(DIM_EDGE_COLOR),
            tile,
            section: match self.section {
                Some([x, y, z, d]) => {
                    let n = DVec3::new(x, y, z);
                    v4(n, d - n.dot(geo_origin))
                }
                None => [0.0, 0.0, 0.0, -1.0],
            },
            shadow_vp: self.shadow.vp.to_cols_array_2d(),
            shadow: match sh {
                Some(s) if self.shadow.key.is_some() => [
                    if s.on_faces { 1.0 } else { 0.0 },
                    if s.on_ground { 1.0 } else { 0.0 },
                    s.light.clamp(0.0, 1.0),
                    s.dark.clamp(0.0, 1.0),
                ],
                Some(s) => [0.0, 0.0, s.light.clamp(0.0, 1.0), s.dark.clamp(0.0, 1.0)],
                None => [0.0; 4],
            },
            shadow2: [
                self.shadow.texel * 1.5,
                1.0 / self.shadow.size as f32,
                if sh.is_some_and(|s| s.sun_dir[2] > 0.0) { 1.0 } else { 0.0 },
                if sh.is_some() { 1.0 } else { 0.0 },
            ],
            grid_o: [0.0; 4],
            grid_u: [1.0, 0.0, 0.0, 10.0],
            grid_v: [0.0, 1.0, 0.0, 0.0],
            grid_color: [0.0; 4],
        };
        if let Some(gs) = style.grid.filter(|g| g.step > 0.0 && g.step.is_finite()) {
            // Keep the plane coordinates small near the camera (f32 in the shader).
            let n = gs.normal();
            let o = DVec3::from_array(gs.origin);
            let near = camera.target - n * (camera.target - o).dot(n);
            let extent = (camera.eye - camera.target).length() + (camera.eye - o).dot(n).abs();
            let o = gs.origin_near(near, extent.max(gs.step));
            globals.grid_o = v4(o - geo_origin - origin, gs.step);
            globals.grid_u = v4(DVec3::from_array(gs.u), gs.major.max(2) as f64);
            globals.grid_v = v4(DVec3::from_array(gs.v), 1.0);
            globals.grid_color = conv(gs.color);
        }
        globals.hidden_face[3] = 1.0;
        globals
    }

    /// Upload the per-frame overlay / axes / sprite buffers and refresh the transparency sort.
    /// `image` = size in pixels of the whole image, `ppp` = pixels per logical point (text
    /// rasterisation, sprite sizes, polyline dash phases).
    fn prepare_frame(
        &mut self,
        rs: &egui_wgpu::RenderState,
        camera: &Camera,
        overlay: &Overlay3d,
        style: &RenderStyle,
        image: [u32; 2],
        ppp: f32,
    ) -> FrameCounts {
        let device = &rs.device;
        let queue = &rs.queue;
        let geo_origin = self.geo.origin;
        let scene_radius = self.geo.radius;
        // Overlay coordinates are world f32; shift them into the geometry's local frame.
        let to_local = |p: [f32; 3]| -> [f32; 3] {
            if geo_origin == DVec3::ZERO {
                p
            } else {
                (DVec3::new(p[0] as f64, p[1] as f64, p[2] as f64) - geo_origin)
                    .as_vec3()
                    .to_array()
            }
        };
        let to_local_d = |p: DVec3| -> [f32; 3] { (p - geo_origin).as_vec3().to_array() };
        let proj = ScreenProj {
            camera,
            viewport: DVec2::new(image[0].max(1) as f64 / ppp as f64, image[1].max(1) as f64 / ppp as f64),
        };
        // Overlay lines (+ polylines, overlay axes): depth-tested first, then on-top.
        let lines = &mut self.scratch_lines;
        lines.clear();
        let mut lines_depth = 0u32;
        for pass_top in [false, true] {
            lines.extend(overlay.lines.iter().filter(|l| l.on_top == pass_top).map(|l| LineInst {
                a: to_local(l.a),
                width: l.width_px.max(0.5),
                b: to_local(l.b),
                dash: if l.dashed { OVERLAY_DASH_PERIOD } else { 0.0 },
                color: l.color,
                phase: 0.0,
                _pad: [0.0; 3],
            }));
            for pl in overlay.polylines.iter().filter(|p| p.on_top == pass_top) {
                annot::push_polyline(lines, pl, &proj, OVERLAY_DASH_PERIOD, to_local_d);
            }
            for ax in overlay.axes.iter().filter(|a| a.on_top == pass_top) {
                let len = ax
                    .length
                    .unwrap_or_else(|| annot::infinite_axis_length(camera, scene_radius, ax.frame.origin));
                annot::push_axes(lines, &ax.frame, len, ax.width_px.max(0.5), to_local_d);
            }
            if !pass_top {
                lines_depth = lines.len() as u32;
            }
        }
        let lines_top = lines.len() as u32 - lines_depth;
        self.overlay_lines.write(device, queue, bytemuck::cast_slice(lines));

        let polys = &mut self.scratch_polys;
        polys.clear();
        let mut polys_depth = 0u32;
        for pass_top in [false, true] {
            for p in overlay.polygons.iter().filter(|p| p.on_top == pass_top) {
                let n = p.tris.len() - p.tris.len() % 3;
                polys.extend(p.tris[..n].iter().map(|&pos| LineVertex { pos: to_local(pos), color: p.color }));
            }
            if !pass_top {
                polys_depth = polys.len() as u32;
            }
        }
        let polys_top = polys.len() as u32 - polys_depth;
        self.overlay_polys.write(device, queue, bytemuck::cast_slice(polys));

        // Text and marks: screen-aligned sprites (depth-tested, then on top).
        let sprites = &mut self.scratch_sprites;
        sprites.clear();
        let mut sprites_depth = 0u32;
        if !overlay.texts.is_empty() {
            self.text.begin();
        }
        for pass_top in [false, true] {
            for m in overlay.marks.iter().filter(|m| m.on_top == pass_top) {
                annot::push_mark(sprites, m, &proj, ppp, to_local(m.pos));
            }
            for t in overlay.texts.iter().filter(|t| t.on_top == pass_top) {
                let galley = self.text.layout(&t.text, t.size_pt, ppp);
                annot::push_text(sprites, t, &galley, &proj, ppp, to_local(t.pos));
            }
            if !pass_top {
                sprites_depth = sprites.len() as u32;
            }
        }
        let sprites_top = sprites.len() as u32 - sprites_depth;
        if !overlay.texts.is_empty() {
            self.text.upload(device, queue);
        }
        self.sprites.write(device, queue, bytemuck::cast_slice(sprites));

        let axes = &mut self.scratch_axes;
        axes.clear();
        if style.show_axes {
            let frame = self.axes_frame.unwrap_or(AxesFrame::WORLD);
            let len = annot::infinite_axis_length(camera, scene_radius, frame.origin);
            annot::push_axes(axes, &frame, len, 1.0, to_local_d);
        }
        let axes_count = axes.len() as u32;
        self.axes.write(device, queue, bytemuck::cast_slice(axes));

        // Translucent faces / X-ray: back-to-front index order (re-sorted on camera change).
        let draw_faces = style.face_style != FaceStyle::Wireframe;
        let opaque_all = matches!(style.face_style, FaceStyle::Monochrome | FaceStyle::HiddenLine);
        let blend = if draw_faces && (style.xray || !opaque_all) {
            self.update_blend_order(rs, camera, style.xray)
        } else {
            0
        };
        FrameCounts {
            lines_depth,
            lines_top,
            polys_depth,
            polys_top,
            sprites_depth,
            sprites_top,
            axes: axes_count,
            blend,
        }
    }

    /// Record the scene render pass (globals and per-frame buffers must be up to date).
    #[allow(clippy::too_many_arguments)]
    /// Record the frame into `msaa_view` / `resolve_view`. With `post` (the effects path) the
    /// scene goes to its float target and G-buffer first, then the effects passes, then the
    /// result and the overlays are drawn into `msaa_view`.
    #[allow(clippy::too_many_arguments)]
    fn encode_scene(
        &self,
        enc: &mut wgpu::CommandEncoder,
        msaa_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        resolve_view: &wgpu::TextureView,
        post: Option<&post::PostTargets>,
        style: &RenderStyle,
        c: &FrameCounts,
        opts: PassOpts,
    ) {
        let clear = if opts.background { wgpu::Color::WHITE } else { wgpu::Color::TRANSPARENT };
        let color = |view, resolve, load| {
            Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: Some(resolve),
                ops: wgpu::Operations { load, store: wgpu::StoreOp::Discard },
            })
        };
        let depth = |view, load, store| {
            Some(wgpu::RenderPassDepthStencilAttachment {
                view,
                depth_ops: Some(wgpu::Operations { load, store }),
                stencil_ops: None,
            })
        };
        let (Some(pt), Some(fx), Some(hp)) = (post, self.post.as_ref(), self.pipelines_hdr.as_ref()) else {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("varstotas scene"),
                color_attachments: &[color(msaa_view, resolve_view, wgpu::LoadOp::Clear(clear))],
                depth_stencil_attachment: depth(depth_view, wgpu::LoadOp::Clear(0.0), wgpu::StoreOp::Discard),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.bind_group, &[]);
            self.draw_scene(&mut pass, &self.pipelines, style, c, opts);
            self.draw_overlays(&mut pass, c);
            return;
        };
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("varstotas scene (effects)"),
                color_attachments: &[
                    color(&pt.hdr_ms, &pt.hdr, wgpu::LoadOp::Clear(clear)),
                    color(&pt.gbuf_ms, &pt.gbuf, wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)),
                ],
                depth_stencil_attachment: depth(&pt.depth_ms, wgpu::LoadOp::Clear(0.0), wgpu::StoreOp::Store),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.bind_group, &[]);
            self.draw_scene(&mut pass, hp, style, c, opts);
        }
        pt.encode(enc, fx, &self.bind_group, &style.env.effects);
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("varstotas overlays (effects)"),
            color_attachments: &[color(msaa_view, resolve_view, wgpu::LoadOp::Clear(clear))],
            depth_stencil_attachment: depth(&pt.depth_ms, wgpu::LoadOp::Load, wgpu::StoreOp::Discard),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_pipeline(&fx.pipes.final_);
        pass.set_bind_group(1, pt.final_group(&style.env.effects), &[]);
        pass.draw(0..3, 0..1);
        self.draw_overlays(&mut pass, c);
    }

    /// Sky, faces, edges, profiles, ground, grid, axes and translucent faces.
    fn draw_scene(&self, pass: &mut wgpu::RenderPass<'_>, p: &Pipelines, style: &RenderStyle, c: &FrameCounts, opts: PassOpts) {
        let ppp = opts.ppp;
        let draw_faces = style.face_style != FaceStyle::Wireframe;
        let opaque_all = matches!(style.face_style, FaceStyle::Monochrome | FaceStyle::HiddenLine);
        let g = &self.geo;
        let pass = &mut *pass;

        // 1. sky
        if opts.background {
            pass.set_pipeline(&p.sky);
            pass.draw(0..3, 0..1);
        }

        let face_tex_bg = &self.face_textures.bind_group;
        let set_face_buffers = |pass: &mut wgpu::RenderPass<'_>| {
            if let (Some(tb), Some(bb), Some(sb), Some(xb)) = (&g.tris, &g.backs, &g.face_states, &g.face_tex) {
                pass.set_vertex_buffer(0, tb.slice(..));
                pass.set_vertex_buffer(1, bb.slice(..));
                pass.set_vertex_buffer(2, sb.slice(..));
                pass.set_vertex_buffer(3, xb.slice(..));
                pass.set_bind_group(1, face_tex_bg, &[]);
                true
            } else {
                false
            }
        };
        let total_face_verts = g.face_verts;

        // 2. opaque faces
        if draw_faces && !style.xray && set_face_buffers(pass) {
            pass.set_pipeline(&p.face_opaque);
            if opaque_all || g.translucent.is_empty() {
                if total_face_verts > 0 {
                    pass.draw(0..total_face_verts, 0..1);
                }
            } else if g.opaque_index_count > 0
                && let Some(ib) = &g.opaque_index
            {
                pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..g.opaque_index_count, 0, 0..1);
            }
        }

        // 3. edges (write depth so x-ray / translucent faces don't wash them out)
        if (style.show_edges || style.show_triangles || style.face_style == FaceStyle::Wireframe) && g.edge_verts > 0 {
            if ppp >= 1.5 {
                // HiDPI: 1 physical px is too faint; draw 1 logical point quads.
                if let (Some(pl), Some(bg)) = (&p.edge_thick_seg, &g.edges_bg) {
                    pass.set_pipeline(pl);
                    pass.set_bind_group(1, bg, &[]);
                    pass.draw(0..g.edge_verts * 3, 0..1); // 6 vertices per segment
                } else if let (Some(eb), Some(sb)) = (&g.edges_thick, &g.edge_states_seg) {
                    pass.set_pipeline(&p.edge_thick);
                    pass.set_vertex_buffer(0, eb.slice(..));
                    pass.set_vertex_buffer(1, sb.slice(..));
                    pass.draw(0..6, 0..g.edge_verts / 2);
                }
            } else if let (Some(eb), Some(sb)) = (&g.edges, &g.edge_states_vtx) {
                pass.set_pipeline(&p.edge);
                pass.set_vertex_buffer(0, eb.slice(..));
                pass.set_vertex_buffer(1, sb.slice(..));
                pass.draw(0..g.edge_verts, 0..1);
            }
        }

        // 4. profiles
        if style.profiles
            && style.face_style != FaceStyle::Wireframe
            && g.profile_count > 0
            && let Some(pb) = &g.profiles
        {
            if let (Some(pl), Some(bg)) = (&p.thick_depth_seg, &g.profiles_bg) {
                pass.set_pipeline(pl);
                pass.set_bind_group(1, bg, &[]);
                pass.draw(0..g.profile_count * 6, 0..1);
            } else if let Some(sb) = &g.profile_states {
                pass.set_pipeline(&p.thick_depth_state);
                pass.set_vertex_buffer(0, pb.slice(..));
                pass.set_vertex_buffer(1, sb.slice(..));
                pass.draw(0..6, 0..g.profile_count);
            }
        }

        // 5. ground (translucent, tints geometry below z = 0)
        if style.show_ground && opts.background {
            pass.set_pipeline(&p.ground);
            pass.draw(0..3, 0..1);
        }

        // 5b. grid (translucent lines on its plane, behind coplanar faces)
        if style.grid.is_some() && opts.background {
            pass.set_pipeline(&p.grid);
            pass.draw(0..3, 0..1);
        }

        // 6. axes — after the ground: they lie in z = 0 and write no depth, so the ground
        //    would otherwise wash them out.
        if c.axes > 0 {
            pass.set_pipeline(&p.thick_depth);
            pass.set_vertex_buffer(0, self.axes.buf.slice(..));
            pass.draw(0..6, 0..c.axes);
        }

        // 7. translucent faces / x-ray: sorted back-to-front, no depth write, both sides
        //    in one pass (the fragment shader picks the front/back color).
        if c.blend > 0 && set_face_buffers(pass) {
            pass.set_pipeline(&p.face_blend);
            pass.set_index_buffer(self.blend.buf.buf.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..c.blend, 0, 0..1);
        }

        // 7b. Wireframe: selected faces as a flat translucent tint (the vertex shader drops
        //     every other triangle; identical colours blend order-independently).
        if style.face_style == FaceStyle::Wireframe
            && g.any_face_selected
            && opts.selection_tint
            && set_face_buffers(pass)
        {
            pass.set_pipeline(&p.face_blend);
            pass.draw(0..total_face_verts, 0..1);
        }
    }

    /// 8. overlays: depth-tested, then on top.
    fn draw_overlays(&self, pass: &mut wgpu::RenderPass<'_>, c: &FrameCounts) {
        let p = &self.pipelines;
        let pass = &mut *pass;
        if c.polys_depth > 0 {
            pass.set_pipeline(&p.poly_depth);
            pass.set_vertex_buffer(0, self.overlay_polys.buf.slice(..));
            pass.draw(0..c.polys_depth, 0..1);
        }
        if c.lines_depth > 0 {
            pass.set_pipeline(&p.thick_depth);
            pass.set_vertex_buffer(0, self.overlay_lines.buf.slice(..));
            pass.draw(0..6, 0..c.lines_depth);
        }
        if c.sprites_depth + c.sprites_top > 0 {
            pass.set_bind_group(1, &self.text.bind_group, &[]);
        }
        if c.sprites_depth > 0 {
            pass.set_pipeline(&p.sprite_depth);
            pass.set_vertex_buffer(0, self.sprites.buf.slice(..));
            pass.draw(0..c.sprites_depth, 0..1);
        }
        if c.polys_top > 0 {
            pass.set_pipeline(&p.poly_top);
            pass.set_vertex_buffer(0, self.overlay_polys.buf.slice(..));
            pass.draw(c.polys_depth..c.polys_depth + c.polys_top, 0..1);
        }
        if c.lines_top > 0 {
            pass.set_pipeline(&p.thick_top);
            pass.set_vertex_buffer(0, self.overlay_lines.buf.slice(..));
            pass.draw(0..6, c.lines_depth..c.lines_depth + c.lines_top);
        }
        if c.sprites_top > 0 {
            pass.set_pipeline(&p.sprite_top);
            pass.set_vertex_buffer(0, self.sprites.buf.slice(..));
            pass.draw(c.sprites_depth..c.sprites_depth + c.sprites_top, 0..1);
        }
    }

    /// Frame of the main axes drawn when `RenderStyle::show_axes` is on (custom axes /
    /// axes of the open group or component). `None` = world axes. Kept until changed; also used
    /// by `render_to_image`.
    /// Replace the face textures: `TexVertex::layers` value `i + 1` samples `images[i]`. Images
    /// are resampled to a common power-of-two square (at most 1024 px) with mipmaps.
    pub fn set_textures(&mut self, rs: &egui_wgpu::RenderState, images: &[TextureData]) {
        self.face_textures.set(&rs.device, &rs.queue, images);
    }

    /// Replace the data textures of `FaceStyle::Realistic` (linear RGBA8: packed occlusion /
    /// roughness / metalness maps, tangent-space normal maps); `PbrMaterial::orm_layer` and
    /// `normal_layer` value `i + 1` sample `images[i]`.
    pub fn set_data_textures(&mut self, rs: &egui_wgpu::RenderState, images: &[TextureData]) {
        self.face_textures.set_data(&rs.device, &rs.queue, images);
    }

    /// Replace the PBR materials of `FaceStyle::Realistic`: `TexVertex::mats` value `i + 1`
    /// uses `materials[i]` (at most `MAX_PBR_MATERIALS`; 0 = `PbrMaterial::default()`).
    pub fn set_materials(&mut self, rs: &egui_wgpu::RenderState, materials: &[crate::PbrMaterial]) {
        self.face_textures.set_materials(&rs.queue, materials);
    }

    /// The HDR panorama of `EnvSource::Image` (`None` = that source shows the sky). Uploaded
    /// on the next frame that uses it.
    pub fn set_environment(&mut self, image: Option<crate::EnvImage>) {
        self.env.set_image(image);
    }

    /// Rebuild the environment panorama when the Realistic style's source changed; write its
    /// uniform for the frame.
    fn update_env(&mut self, rs: &egui_wgpu::RenderState, style: &RenderStyle) {
        if style.face_style == FaceStyle::Realistic && self.env.update(&rs.device, &rs.queue, style) {
            self.bind_group = globals_bind_group(&rs.device, &self.bgl, &self.globals, &self.shadow, &self.env);
        }
        // The shadow box is 3 r deep and 2 r wide (update_shadow_map).
        self.env.write_uniform(&rs.queue, &style.env, 1.5);
    }

    /// The effects path is used for this style: make its pipelines and `size` targets.
    fn prepare_post(&mut self, rs: &egui_wgpu::RenderState, style: &RenderStyle, size: [u32; 2]) -> bool {
        if !self.ensure_post(rs, style) {
            return false;
        }
        if self.post_targets.as_ref().is_none_or(|t| t.size != size) {
            let post = self.post.as_ref().expect("post");
            self.post_targets = Some(post::PostTargets::new(&rs.device, post, size));
        }
        true
    }

    /// Effects pipelines (no targets) when `style` needs them.
    fn ensure_post(&mut self, rs: &egui_wgpu::RenderState, style: &RenderStyle) -> bool {
        if style.face_style != FaceStyle::Realistic || !style.env.effects.needs_post() {
            return false;
        }
        if self.post.is_none() {
            self.post = Some(post::Post::new(&rs.device, &self.bgl, &self.shader, self.color_format, post::lin_format(&rs.adapter)));
        }
        if self.pipelines_hdr.is_none() {
            self.pipelines_hdr = Some(build_pipelines(&rs.device, &self.shader, &self.layouts, self.color_format, true));
        }
        true
    }

    fn write_post_uniform(&self, rs: &egui_wgpu::RenderState, style: &RenderStyle, globals: &Globals, size: [u32; 2], ppp: f32) {
        let Some(post) = &self.post else { return };
        let inv = glam::Mat4::from_cols_array_2d(&globals.view_proj).inverse();
        let u = post::PostUniform::new(
            &style.env.effects,
            inv,
            size,
            ppp,
            env::exposure_mult(style.env.exposure),
            self.geo.radius as f32,
            !self.env.shows_background(&style.env),
        );
        rs.queue.write_buffer(&post.uniform, 0, bytemuck::bytes_of(&u));
    }

    /// Replace the custom surface shaders (`PbrMaterial::shader` = index + 1). Shaders that do
    /// not compile are left out (their materials keep their factors); the result has one entry
    /// per shader, `Some(message)` for those. Unchanged lists cost nothing.
    pub fn set_custom_shaders(&mut self, rs: &egui_wgpu::RenderState, shaders: &[crate::CustomSurface]) -> Vec<Option<String>> {
        if self.custom == shaders {
            return self.custom_errors.clone();
        }
        let errors: Vec<Option<String>> = shaders.iter().map(crate::custom::check).collect();
        let ok: Vec<Option<&crate::CustomSurface>> =
            shaders.iter().zip(&errors).map(|(s, e)| e.is_none().then_some(s)).collect();
        let (src, _) = crate::custom::compose(&ok);
        if let Err((line, msg)) = crate::custom::validate(&src) {
            // Each one compiles alone but not together (a name clash): keep the old module.
            log::warn!("custom shaders: line {line:?}: {msg}");
            let all = Some(varstotas_i18n::tr!("render-shader-clash", error = msg));
            return vec![all; shaders.len()];
        }
        let device = &rs.device;
        self.shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("varstotas scene shader"),
            source: wgpu::ShaderSource::Wgsl(src.clone().into()),
        });
        self.shader_src = src;
        self.pipelines = build_pipelines(device, &self.shader, &self.layouts, self.color_format, false);
        if self.pipelines_hdr.is_some() {
            self.pipelines_hdr = Some(build_pipelines(device, &self.shader, &self.layouts, self.color_format, true));
        }
        if let Some(p) = &mut self.post {
            p.rebuild(device, &self.shader, self.color_format);
        }
        if let Some(pt) = &mut self.pt {
            pt.rebuild(device, &self.shader, &self.shader_src);
        }
        self.custom = shaders.to_vec();
        self.custom_errors = errors.clone();
        errors
    }

    /// Start a path traced render of `geo` (any geometry, e.g. the one of `set_geometry`) from
    /// `camera` with the Realistic environment, materials and effects of `style`; `pt_step`
    /// adds samples. The image size may be lowered to fit the device.
    pub fn pt_start(
        &mut self,
        rs: &egui_wgpu::RenderState,
        geo: &SceneGeometry,
        lights: &[crate::PtLight],
        camera: &Camera,
        style: &RenderStyle,
        settings: &crate::PtSettings,
    ) {
        self.update_env(rs, style);
        let pt = self.pt.get_or_insert_with(|| {
            crate::pathtrace::PathTracer::new(&rs.device, &self.shader, &self.shader_src, &self.bgl, &self.face_textures.bgl)
        });
        let test = crate::pathtrace::ShaderTest { cut: &self.face_textures.cut, glass: &self.face_textures.glass };
        pt.start(rs, geo, lights, camera, style, settings, &test);
    }

    /// Add samples to the render (up to `max_samples`, for about `budget`); the samples so far.
    /// `style` = the one of `pt_start` (its environment is current on the GPU).
    pub fn pt_step(&mut self, rs: &egui_wgpu::RenderState, style: &RenderStyle, max_samples: u32, budget: std::time::Duration) -> u32 {
        self.update_env(rs, style);
        let Some(pt) = &mut self.pt else { return 0 };
        pt.step(rs, [&self.bind_group, &self.face_textures.bind_group], max_samples, budget)
    }

    /// Exposure, tone curve, grading and denoiser of the render's image (next `pt_resolve`).
    pub fn pt_set_output(&mut self, output: crate::PtOutput) {
        if let Some(pt) = &mut self.pt {
            pt.set_output(output);
        }
    }

    /// Update the render's texture (`pt_texture`) with the samples so far.
    pub fn pt_resolve(&mut self, rs: &egui_wgpu::RenderState, denoise: bool) {
        let Some(pt) = &mut self.pt else { return };
        pt.resolve(rs, [&self.bind_group, &self.face_textures.bind_group], denoise);
    }

    /// The render's image in egui and its size.
    pub fn pt_texture(&self) -> Option<(egui::TextureId, [u32; 2])> {
        let pt = self.pt.as_ref()?;
        Some((pt.texture_id()?, pt.size()?))
    }

    /// The shown image (after `pt_resolve`), sRGB.
    pub fn pt_image(&self, rs: &egui_wgpu::RenderState) -> Option<RgbaImage> {
        self.pt.as_ref()?.read_image(rs)
    }

    /// The render as linear RGB with the exposure applied (for EXR): width, height, pixels.
    pub fn pt_linear(&mut self, rs: &egui_wgpu::RenderState, denoise: bool) -> Option<(u32, u32, Vec<f32>)> {
        let pt = self.pt.as_mut()?;
        pt.read_linear(rs, [&self.bind_group, &self.face_textures.bind_group], denoise)
    }

    /// Section cut (world plane `[n.x, n.y, n.z, d]`, unit `n`): model faces and edges with
    /// `n · p < d` are not drawn. Overlays are never cut.
    /// Re-render the shadow map if shadows are on and the geometry, the sun or the section cut
    /// changed since the last one (separate submission, before the frame).
    fn update_shadow_map(&mut self, rs: &egui_wgpu::RenderState, style: &RenderStyle) {
        let Some(sh) = style.shadows else { return };
        let sun = DVec3::from_array(sh.sun_dir);
        let key = ShadowKey { revision: self.geo.revision, sun: sh.sun_dir, section: self.section };
        if self.shadow.key == Some(key) {
            return;
        }
        let device = &rs.device;
        let size = SHADOW_MAP_SIZE.min(self.max_texture_dim);
        if self.shadow.size != size {
            self.shadow.resize(device, size);
            self.bind_group = globals_bind_group(device, &self.bgl, &self.globals, &self.shadow, &self.env);
        }
        self.shadow.key = Some(key);
        let Some((lo, hi)) = self.geo.bounds else {
            // No faces: nothing casts; an empty (cleared) map keeps everything lit.
            self.shadow.vp = glam::Mat4::IDENTITY;
            self.encode_shadow_pass(rs, false);
            return;
        };
        let dir = if sun.is_finite() && sun.length_squared() > 1e-12 { sun.normalize() } else { DVec3::Z };
        let (lo, hi) = (lo.as_dvec3(), hi.as_dvec3());
        let c = (lo + hi) * 0.5;
        let r = ((hi - lo).length() * 0.5).max(1.0);
        let up = if dir.z.abs() > 0.99 { DVec3::Y } else { DVec3::Z };
        // Right-handed look-at from the sun side, then an ortho box (depth 0 = near .. 1 = far).
        let eye = c + dir * 2.0 * r;
        let f = -dir;
        let rt = f.cross(up).normalize();
        let u = rt.cross(f);
        let view = DMat4::from_cols(
            glam::DVec4::new(rt.x, u.x, -f.x, 0.0),
            glam::DVec4::new(rt.y, u.y, -f.y, 0.0),
            glam::DVec4::new(rt.z, u.z, -f.z, 0.0),
            glam::DVec4::new(-rt.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
        );
        let (n, fa) = (0.5 * r, 3.5 * r);
        let proj = DMat4::from_cols(
            glam::DVec4::new(1.0 / r, 0.0, 0.0, 0.0),
            glam::DVec4::new(0.0, 1.0 / r, 0.0, 0.0),
            glam::DVec4::new(0.0, 0.0, -1.0 / (fa - n), 0.0),
            glam::DVec4::new(0.0, 0.0, -n / (fa - n), 1.0),
        );
        self.shadow.vp = (proj * view).as_mat4();
        self.shadow.texel = (2.0 * r / size as f64) as f32;
        self.encode_shadow_pass(rs, true);
    }

    fn encode_shadow_pass(&self, rs: &egui_wgpu::RenderState, draw: bool) {
        let geo_origin = self.geo.origin;
        let section = match self.section {
            Some([x, y, z, d]) => {
                let n = DVec3::new(x, y, z);
                [n.x as f32, n.y as f32, n.z as f32, (d - n.dot(geo_origin)) as f32]
            }
            None => [0.0, 0.0, 0.0, -1.0],
        };
        let u = ShadowUniform { vp: self.shadow.vp.to_cols_array_2d(), section };
        rs.queue.write_buffer(&self.shadow.uniform, 0, bytemuck::bytes_of(&u));
        let mut enc = rs.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("varstotas shadow") });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("varstotas shadow map"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow.view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if draw
                && let Some(tb) = &self.geo.tris
                && self.geo.face_verts > 0
            {
                pass.set_pipeline(&self.shadow.pipeline);
                pass.set_bind_group(1, &self.shadow.bind_group, &[]);
                pass.set_vertex_buffer(0, tb.slice(..));
                pass.draw(0..self.geo.face_verts, 0..1);
            }
        }
        rs.queue.submit(Some(enc.finish()));
    }

    pub fn set_section(&mut self, plane: Option<[f64; 4]>) {
        self.section = plane;
    }

    pub fn set_axes_frame(&mut self, frame: Option<AxesFrame>) {
        self.axes_frame = frame;
    }

    pub fn axes_frame(&self) -> Option<AxesFrame> {
        self.axes_frame
    }

    /// The egui texture id of the resolved image (after the first `render`).
    pub fn texture_id(&self) -> Option<egui::TextureId> {
        self.texture_id
    }
}

/// Write `data` into `slot`, reusing the buffer when it is large enough (grow-only, 25 % slack).
/// Empty data keeps the old buffer (draws are gated by counts).
fn globals_bind_group(
    device: &wgpu::Device,
    bgl: &wgpu::BindGroupLayout,
    globals: &wgpu::Buffer,
    shadow: &ShadowMap,
    env: &env::Env,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("varstotas globals"),
        layout: bgl,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: globals.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&shadow.view) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&shadow.sampler) },
            wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&env.view) },
            wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&env.sampler) },
            wgpu::BindGroupEntry { binding: 5, resource: env.uniform.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(&shadow.view) },
        ],
    })
}

impl ShadowMap {
    /// A 1×1 placeholder map (shadows off) plus the shadow pass pipeline.
    fn new(device: &wgpu::Device, shader: &wgpu::ShaderModule) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("varstotas shadow sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("varstotas shadow uniform"),
            size: std::mem::size_of::<ShadowUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("varstotas shadow pass"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("varstotas shadow pass"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("varstotas shadow pass"),
            bind_group_layouts: &[None, Some(&bgl)],
            immediate_size: 0,
        });
        let pos_attr = wgpu::vertex_attr_array![0 => Float32x3];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("varstotas shadow pass"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_shadow"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<TriVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &pos_attr,
                })],
            },
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: SHADOW_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: wgpu::DepthBiasState { constant: 2, slope_scale: 2.0, clamp: 0.0 },
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_shadow"),
                compilation_options: Default::default(),
                targets: &[],
            }),
            multiview_mask: None,
            cache: None,
        });
        let mut m = ShadowMap {
            size: 0,
            view: Self::texture(device, 1),
            sampler,
            pipeline,
            uniform,
            bind_group,
            key: None,
            vp: glam::Mat4::IDENTITY,
            texel: 0.0,
        };
        m.size = 1;
        m
    }

    fn texture(device: &wgpu::Device, size: u32) -> wgpu::TextureView {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("varstotas shadow map"),
                size: wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: SHADOW_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default())
    }

    fn resize(&mut self, device: &wgpu::Device, size: u32) {
        self.view = Self::texture(device, size);
        self.size = size;
        self.key = None;
    }
}

fn upload(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    slot: &mut Option<wgpu::Buffer>,
    label: &str,
    usage: wgpu::BufferUsages,
    data: &[u8],
) {
    if data.is_empty() {
        return;
    }
    let usage = usage | wgpu::BufferUsages::COPY_DST;
    let len = data.len() as u64;
    if let Some(b) = slot
        && b.size() >= len
        && b.usage() == usage
    {
        queue.write_buffer(b, 0, data);
        return;
    }
    let size = (len + len / 4).next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT);
    let b = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: true,
    });
    match b.slice(..len).get_mapped_range_mut() {
        Ok(mut view) => view.copy_from_slice(data),
        Err(_) => {
            b.unmap();
            queue.write_buffer(&b, 0, data);
            *slot = Some(b);
            return;
        }
    }
    b.unmap();
    *slot = Some(b);
}

/// Storage bind group (segments + per-segment states) for the non-instanced thick-line path.
fn seg_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    segs: &wgpu::Buffer,
    states: &wgpu::Buffer,
    count: usize,
) -> Option<wgpu::BindGroup> {
    // Raw `LineVertex` pairs: 2 x 28 bytes per segment.
    let seg_bytes = (count * 2 * std::mem::size_of::<LineVertex>()) as u64;
    if count == 0 || !segs.usage().contains(wgpu::BufferUsages::STORAGE) {
        return None;
    }
    fn binding(b: &wgpu::Buffer, size: u64) -> wgpu::BindingResource<'_> {
        wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: b, offset: 0, size: std::num::NonZeroU64::new(size) })
    }
    Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("varstotas segments"),
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: binding(segs, seg_bytes) },
            wgpu::BindGroupEntry { binding: 1, resource: binding(states, count as u64 * 4) },
        ],
    }))
}

/// Order-preserving map of an f32 to u32 (ascending floats -> ascending keys).
fn depth_key(d: f32) -> u32 {
    let b = d.to_bits();
    if b & 0x8000_0000 != 0 { !b } else { b | 0x8000_0000 }
}

/// LSD radix sort of `vals` by `keys` (ascending), 3 passes of 11 bits. O(n), stable.
fn radix_sort_pairs(keys: &mut Vec<u32>, vals: &mut Vec<u32>, tmp_k: &mut Vec<u32>, tmp_v: &mut Vec<u32>) {
    let n = keys.len();
    tmp_k.resize(n, 0);
    tmp_v.resize(n, 0);
    for shift in [0u32, 11, 22] {
        let mut count = [0usize; 2048];
        for &k in keys.iter() {
            count[((k >> shift) & 2047) as usize] += 1;
        }
        let mut sum = 0;
        for c in count.iter_mut() {
            let x = *c;
            *c = sum;
            sum += x;
        }
        for i in 0..n {
            let k = keys[i];
            let b = ((k >> shift) & 2047) as usize;
            tmp_k[count[b]] = k;
            tmp_v[count[b]] = vals[i];
            count[b] += 1;
        }
        std::mem::swap(keys, tmp_k);
        std::mem::swap(vals, tmp_v);
    }
}

/// `hdr`: the scene pass of the effects path (float colour + G-buffer targets, `HDR_OUT`).
fn build_pipelines(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layouts: &Layouts,
    color_format: wgpu::TextureFormat,
    hdr: bool,
) -> Pipelines {
    let layout = &layouts.scene;
    let face_layout = &layouts.face;
    let seg_layout = layouts.seg.as_ref();
    let text_layout = &layouts.text;
    let color_format = if hdr { post::HDR_FORMAT } else { color_format };
    let constants: &[(&str, f64)] = if hdr { &[("HDR_OUT", 1.0)] } else { &[] };
    let tri_attrs = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x4];
    let back_attrs = wgpu::vertex_attr_array![3 => Float32x4];
    let face_state_attrs = wgpu::vertex_attr_array![4 => Uint32];
    let face_tex_attrs = wgpu::vertex_attr_array![5 => Float32x2, 6 => Uint16x2, 7 => Uint16x2];
    let edge_state_attrs = wgpu::vertex_attr_array![2 => Uint32];
    let seg_state_attrs = wgpu::vertex_attr_array![5 => Uint32];
    let line_attrs = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4];
    let thick_attrs = wgpu::vertex_attr_array![
        0 => Float32x3, 1 => Float32, 2 => Float32x3, 3 => Float32, 4 => Float32x4
    ];
    let tri_layouts = [
        Some(wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TriVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &tri_attrs,
        }),
        Some(wgpu::VertexBufferLayout {
            array_stride: 16,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &back_attrs,
        }),
        Some(wgpu::VertexBufferLayout {
            array_stride: 4,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &face_state_attrs,
        }),
        Some(wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TexVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &face_tex_attrs,
        }),
    ];
    let edge_layouts = [
        Some(wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &line_attrs,
        }),
        Some(wgpu::VertexBufferLayout {
            array_stride: 4,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &edge_state_attrs,
        }),
    ];
    let line_layouts = [Some(wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<LineVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &line_attrs,
    })];
    let thick_layouts = [Some(wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<ThickLine>() as u64,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &thick_attrs,
    })];
    let line_inst_attrs = wgpu::vertex_attr_array![
        0 => Float32x3, 1 => Float32, 2 => Float32x3, 3 => Float32, 4 => Float32x4, 5 => Float32
    ];
    let line_inst_layouts = [Some(wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<LineInst>() as u64,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &line_inst_attrs,
    })];
    let sprite_attrs = wgpu::vertex_attr_array![
        0 => Float32x3, 1 => Float32, 2 => Float32x2, 3 => Float32x2, 4 => Float32x4
    ];
    let sprite_layouts = [Some(wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<SpriteVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &sprite_attrs,
    })];
    let thick_state_layouts = [
        thick_layouts[0].clone(),
        Some(wgpu::VertexBufferLayout {
            array_stride: 4,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &seg_state_attrs,
        }),
    ];

    struct Desc<'a> {
        label: &'a str,
        vs: &'a str,
        fs: &'a str,
        buffers: &'a [Option<wgpu::VertexBufferLayout<'a>>],
        topology: wgpu::PrimitiveTopology,
        blend: bool,
        depth_write: bool,
        depth_compare: wgpu::CompareFunction,
        bias: wgpu::DepthBiasState,
        cull: Option<wgpu::Face>,
    }
    let make_with = |d: Desc<'_>, layout: &wgpu::PipelineLayout| {
        // Effects path: opaque faces and the ground write the G-buffer (and the ground its
        // depth, for the fog and ambient occlusion); everything else leaves it.
        let gbuf_write = (d.fs == "fs_face" && !d.blend) || d.fs == "fs_ground";
        let color = Some(wgpu::ColorTargetState {
            format: color_format,
            blend: if d.blend { Some(BLEND) } else { None },
            write_mask: wgpu::ColorWrites::ALL,
        });
        let gbuf = Some(wgpu::ColorTargetState {
            format: post::GBUF_FORMAT,
            blend: None,
            write_mask: if gbuf_write { wgpu::ColorWrites::ALL } else { wgpu::ColorWrites::empty() },
        });
        let targets = [color, gbuf];
        let opts = wgpu::PipelineCompilationOptions { constants, ..Default::default() };
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(d.label),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some(d.vs),
                compilation_options: opts.clone(),
                buffers: d.buffers,
            },
            primitive: wgpu::PrimitiveState {
                topology: d.topology,
                cull_mode: d.cull,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(d.depth_write || (hdr && d.fs == "fs_ground")),
                depth_compare: Some(d.depth_compare),
                stencil: Default::default(),
                bias: d.bias,
            }),
            multisample: wgpu::MultisampleState {
                count: SAMPLES,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some(d.fs),
                compilation_options: opts,
                targets: if hdr { &targets } else { &targets[..1] },
            }),
            multiview_mask: None,
            cache: None,
        })
    };
    let make = |d: Desc<'_>| make_with(d, layout);
    use wgpu::CompareFunction::{Always, GreaterEqual};
    use wgpu::PrimitiveTopology::{LineList, TriangleList};
    let no_bias = wgpu::DepthBiasState::default();
    // Push faces slightly away from the camera (reverse-Z: negative = farther).
    let face_bias = wgpu::DepthBiasState {
        constant: -2,
        slope_scale: -1.5,
        clamp: 0.0,
    };
    let seg = |label: &'static str, vs: &'static str, depth_write: bool| {
        seg_layout.map(|l| {
            make_with(
                Desc {
                    label,
                    vs,
                    fs: "fs_thick",
                    buffers: &[],
                    topology: TriangleList,
                    blend: true,
                    depth_write,
                    depth_compare: GreaterEqual,
                    bias: no_bias,
                    cull: None,
                },
                l,
            )
        })
    };
    Pipelines {
        edge_thick_seg: seg("varstotas edges thick (storage)", "vs_edge_seg", true),
        thick_depth_seg: seg("varstotas profiles (storage)", "vs_profile_seg", false),
        sky: make(Desc {
            label: "varstotas sky",
            vs: "vs_fullscreen",
            fs: "fs_sky",
            buffers: &[],
            topology: TriangleList,
            blend: false,
            depth_write: false,
            depth_compare: Always,
            bias: no_bias,
            cull: None,
        }),
        ground: make(Desc {
            label: "varstotas ground",
            vs: "vs_fullscreen",
            fs: "fs_ground",
            buffers: &[],
            topology: TriangleList,
            blend: true,
            depth_write: false,
            depth_compare: GreaterEqual,
            bias: no_bias,
            cull: None,
        }),
        grid: make(Desc {
            label: "varstotas grid",
            vs: "vs_fullscreen",
            fs: "fs_grid",
            buffers: &[],
            topology: TriangleList,
            blend: true,
            depth_write: false,
            depth_compare: GreaterEqual,
            bias: no_bias,
            cull: None,
        }),
        face_opaque: make_with(Desc {
            label: "varstotas faces",
            vs: "vs_face",
            fs: "fs_face",
            buffers: &tri_layouts,
            topology: TriangleList,
            blend: false,
            depth_write: true,
            depth_compare: GreaterEqual,
            bias: face_bias,
            cull: None,
        }, face_layout),
        face_blend: make_with(Desc {
            label: "varstotas faces blend",
            vs: "vs_face",
            fs: "fs_face",
            buffers: &tri_layouts,
            topology: TriangleList,
            blend: true,
            depth_write: false,
            depth_compare: GreaterEqual,
            bias: face_bias,
            cull: None,
        }, face_layout),
        edge_thick: make(Desc {
            label: "varstotas edges thick",
            vs: "vs_thick_state",
            fs: "fs_thick",
            buffers: &thick_state_layouts,
            topology: TriangleList,
            blend: true,
            depth_write: true,
            depth_compare: GreaterEqual,
            bias: no_bias,
            cull: None,
        }),
        edge: make(Desc {
            label: "varstotas edges",
            vs: "vs_edge",
            fs: "fs_flat",
            buffers: &edge_layouts,
            topology: LineList,
            blend: true,
            depth_write: true,
            depth_compare: GreaterEqual,
            bias: no_bias,
            cull: None,
        }),
        thick_depth: make(Desc {
            label: "varstotas thick lines",
            vs: "vs_line",
            fs: "fs_thick",
            buffers: &line_inst_layouts,
            topology: TriangleList,
            blend: true,
            depth_write: false,
            depth_compare: GreaterEqual,
            bias: no_bias,
            cull: None,
        }),
        thick_depth_state: make(Desc {
            label: "varstotas profiles (instanced)",
            vs: "vs_thick_state",
            fs: "fs_thick",
            buffers: &thick_state_layouts,
            topology: TriangleList,
            blend: true,
            depth_write: false,
            depth_compare: GreaterEqual,
            bias: no_bias,
            cull: None,
        }),
        thick_top: make(Desc {
            label: "varstotas thick lines on top",
            vs: "vs_line",
            fs: "fs_thick",
            buffers: &line_inst_layouts,
            topology: TriangleList,
            blend: true,
            depth_write: false,
            depth_compare: Always,
            bias: no_bias,
            cull: None,
        }),
        poly_depth: make(Desc {
            label: "varstotas overlay polys",
            vs: "vs_poly",
            fs: "fs_flat",
            buffers: &line_layouts,
            topology: TriangleList,
            blend: true,
            depth_write: false,
            depth_compare: GreaterEqual,
            bias: no_bias,
            cull: None,
        }),
        poly_top: make(Desc {
            label: "varstotas overlay polys on top",
            vs: "vs_poly",
            fs: "fs_flat",
            buffers: &line_layouts,
            topology: TriangleList,
            blend: true,
            depth_write: false,
            depth_compare: Always,
            bias: no_bias,
            cull: None,
        }),
        sprite_depth: make_with(
            Desc {
                label: "varstotas sprites",
                vs: "vs_sprite",
                fs: "fs_sprite",
                buffers: &sprite_layouts,
                topology: TriangleList,
                blend: true,
                depth_write: false,
                depth_compare: GreaterEqual,
                bias: no_bias,
                cull: None,
            },
            text_layout,
        ),
        sprite_top: make_with(
            Desc {
                label: "varstotas sprites on top",
                vs: "vs_sprite",
                fs: "fs_sprite",
                buffers: &sprite_layouts,
                topology: TriangleList,
                blend: true,
                depth_write: false,
                depth_compare: Always,
                bias: no_bias,
                cull: None,
            },
            text_layout,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layouts_match_shader() {
        assert_eq!(std::mem::size_of::<Globals>(), 64 + 21 * 16 + 64 + 2 * 16 + 4 * 16);
        assert_eq!(std::mem::size_of::<ShadowUniform>(), 80);
        assert_eq!(std::mem::size_of::<ThickLine>(), 48);
        assert_eq!(std::mem::size_of::<LineInst>(), 64);
        assert_eq!(std::mem::size_of::<SpriteVertex>(), 48);
        assert_eq!(std::mem::size_of::<TriVertex>(), 40);
        assert_eq!(std::mem::size_of::<LineVertex>(), 28);
        assert_eq!(std::mem::size_of::<TexVertex>(), 16);
        assert_eq!(std::mem::size_of::<env::EnvUniform>(), 12 * 16);
        assert_eq!(std::mem::size_of::<post::PostUniform>(), 12 * 16);
    }

    #[test]
    fn shader_parses() {
        // Validate WGSL at test time via naga (re-exported by wgpu).
        let src = &crate::custom::compose(&[]).0;
        let module = wgpu::naga::front::wgsl::parse_str(src).expect("wgsl parse");
        let mut v = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::empty(),
        );
        v.validate(&module).expect("wgsl validate");
    }

    #[test]
    fn highlight_values_match_shader() {
        let src = include_str!("shader.wgsl");
        assert_eq!(Highlight::None as u32, 0);
        assert!(src.contains(&format!("const HL_SELECTED: u32 = {}u;", Highlight::Selected as u32)));
        assert!(src.contains(&format!("const HL_DIMMED: u32 = {}u;", Highlight::Dimmed as u32)));
    }

    #[test]
    fn depth_key_is_monotonic() {
        let xs = [-1e30f32, -5.0, -1.0, -1e-20, -0.0, 0.0, 1e-20, 0.5, 1.0, 7.0, 1e30];
        for w in xs.windows(2) {
            assert!(depth_key(w[0]) <= depth_key(w[1]), "{} vs {}", w[0], w[1]);
        }
    }

    #[test]
    fn radix_sort_back_to_front() {
        // Pseudo-random depths; sorting by !depth_key gives descending depth (far first).
        let mut seed = 12345u32;
        let depths: Vec<f32> = (0..5000)
            .map(|_| {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (seed >> 8) as f32 / 1000.0 - 3000.0
            })
            .collect();
        let mut keys: Vec<u32> = depths.iter().map(|&d| !depth_key(d)).collect();
        let mut vals: Vec<u32> = (0..depths.len() as u32).collect();
        let (mut tk, mut tv) = (Vec::new(), Vec::new());
        radix_sort_pairs(&mut keys, &mut vals, &mut tk, &mut tv);
        for w in vals.windows(2) {
            assert!(depths[w[0] as usize] >= depths[w[1] as usize]);
        }
        let mut seen = vals.clone();
        seen.sort_unstable();
        assert_eq!(seen, (0..depths.len() as u32).collect::<Vec<_>>());
    }

    #[test]
    fn default_style_colors() {
        let s = RenderStyle::default();
        assert_eq!(s.face_style, FaceStyle::Shaded);
        assert!((s.sky_top[0] - 0xA8 as f32 / 255.0).abs() < 1e-6);
    }
}
