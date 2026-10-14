//! wgpu renderer: offscreen MSAA color+depth, resolved into a texture registered with egui.

use crate::wgpu;
use crate::{Camera, FaceStyle, LineVertex, Overlay3d, RenderStyle, SceneGeometry, TriVertex};
use glam::{DMat4, DVec2, DVec3};
use wgpu::util::DeviceExt;

const SAMPLES: u32 = 4;
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
    tris: Option<wgpu::Buffer>,
    backs: Option<wgpu::Buffer>,
    /// Vertex counts: opaque range first, then translucent.
    opaque_verts: u32,
    translucent_verts: u32,
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
    face_opaque: wgpu::RenderPipeline,
    /// Translucent / X-ray faces: indexed back-to-front, no cull, no depth write.
    face_blend: wgpu::RenderPipeline,
    edge_thick: wgpu::RenderPipeline,
    /// Same as `edge_thick` / `thick_depth`, but segments are read from a storage buffer and
    /// drawn as a plain (non-instanced) triangle list: tiny 6-vertex instances are slow on
    /// tile-based GPUs. `None` when vertex-stage storage buffers are unavailable (GL).
    edge_thick_seg: Option<wgpu::RenderPipeline>,
    thick_depth_seg: Option<wgpu::RenderPipeline>,
    edge: wgpu::RenderPipeline,
    thick_depth: wgpu::RenderPipeline,
    thick_top: wgpu::RenderPipeline,
    poly_depth: wgpu::RenderPipeline,
    poly_top: wgpu::RenderPipeline,
}

pub struct Renderer {
    color_format: wgpu::TextureFormat,
    /// Format of the view handed to egui (non-sRGB view of the same texture).
    egui_view_format: wgpu::TextureFormat,
    manual_gamma: bool,
    pipelines: Pipelines,
    globals: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    targets: Option<Targets>,
    texture_id: Option<egui::TextureId>,
    geo: StaticGeo,
    overlay_lines: DynBuffer,
    overlay_polys: DynBuffer,
    axes: DynBuffer,
    blend: BlendSort,
    /// Reused per-frame scratch (no per-frame allocations).
    scratch_lines: Vec<ThickLine>,
    scratch_polys: Vec<LineVertex>,
    scratch_axes: Vec<ThickLine>,
    max_buffer: u64,
    max_texture_dim: u32,
    /// Layout of the segment storage bind group (group 1); `None` = instanced fallback only.
    seg_bgl: Option<wgpu::BindGroupLayout>,
    max_storage_binding: u64,
    timing: Option<Timing>,
}

fn srgb_to_linear(c: [f32; 4]) -> [f32; 4] {
    let f = |x: f32| {
        let x = x.clamp(0.0, 1.0);
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    };
    [f(c[0]), f(c[1]), f(c[2]), c[3]]
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
        }
    }
}

/// Default sun direction (toward the light, world space): from upper-left-front.
pub const DEFAULT_LIGHT_DIR: [f32; 3] = [-0.4, -0.6, 0.7];
/// Default ambient term (0..1, perceptual): faces turned away from the sun get this shade.
pub const DEFAULT_AMBIENT: f32 = 0.68;

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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("varstotas scene shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("varstotas globals"),
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
        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("varstotas globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("varstotas globals"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals.as_entire_binding(),
            }],
        });
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
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                })
            });
        let seg_layout = seg_bgl.as_ref().map(|sb| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("varstotas segments"),
                bind_group_layouts: &[Some(&bgl), Some(sb)],
                immediate_size: 0,
            })
        });
        let pipelines = build_pipelines(device, &shader, &layout, seg_layout.as_ref(), color_format);
        let vb = wgpu::BufferUsages::VERTEX;
        Renderer {
            color_format,
            egui_view_format,
            manual_gamma,
            pipelines,
            globals,
            bind_group,
            targets: None,
            texture_id: None,
            geo: StaticGeo {
                revision: 0,
                origin: DVec3::ZERO,
                centroids: Vec::new(),
                tris: None,
                backs: None,
                opaque_verts: 0,
                translucent_verts: 0,
                edges: None,
                edges_thick: None,
                edge_verts: 0,
                profiles: None,
                profile_count: 0,
                edges_bg: None,
                profiles_bg: None,
                radius: 0.0,
            },
            overlay_lines: DynBuffer::new(device, "varstotas overlay lines", vb),
            overlay_polys: DynBuffer::new(device, "varstotas overlay polys", vb),
            axes: DynBuffer::new(device, "varstotas axes", vb),
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
            max_buffer: device.limits().max_buffer_size,
            max_texture_dim: limits.max_texture_dimension_2d,
            seg_bgl,
            max_storage_binding: limits.max_storage_buffer_binding_size,
            timing: std::env::var("VARSTOTAS_RENDER_TIMING")
                .is_ok_and(|v| !v.is_empty() && v != "0")
                .then(Timing::default),
        }
    }

    /// Upload new static geometry.
    pub fn set_geometry(&mut self, rs: &egui_wgpu::RenderState, geo: &SceneGeometry) {
        let t0 = std::time::Instant::now();
        let device = &rs.device;
        let max_tris_verts = (self.max_buffer / std::mem::size_of::<TriVertex>() as u64) as usize;
        let mut n = geo.tris.len() - geo.tris.len() % 3;
        if n > max_tris_verts {
            log::warn!(
                "varstotas_render: {} triangle vertices exceed the device buffer limit, truncating to {}",
                n,
                max_tris_verts
            );
            n = max_tris_verts - max_tris_verts % 3;
        }
        let default_back = DEFAULT_BACK_COLOR;
        let back_of = |i: usize| geo.back_colors.get(i).copied().unwrap_or(default_back);

        // Split triangles into opaque and translucent ranges.
        let mut tris: Vec<TriVertex> = Vec::with_capacity(n);
        let mut backs: Vec<[f32; 4]> = Vec::with_capacity(n);
        let mut t_tris: Vec<TriVertex> = Vec::new();
        let mut t_backs: Vec<[f32; 4]> = Vec::new();
        for t in 0..n / 3 {
            let i = t * 3;
            let translucent = (i..i + 3).any(|k| geo.tris[k].color[3] < 0.999 || back_of(k)[3] < 0.999);
            let (dt, db) = if translucent {
                (&mut t_tris, &mut t_backs)
            } else {
                (&mut tris, &mut backs)
            };
            for k in i..i + 3 {
                dt.push(geo.tris[k]);
                db.push(back_of(k));
            }
        }
        let opaque_verts = tris.len() as u32;
        let translucent_verts = t_tris.len() as u32;
        tris.extend_from_slice(&t_tris);
        backs.extend_from_slice(&t_backs);
        self.geo.centroids.clear();
        self.geo.centroids.extend(tris.as_chunks::<3>().0.iter().map(|t| {
            let c = (glam::Vec3::from(t[0].pos) + glam::Vec3::from(t[1].pos) + glam::Vec3::from(t[2].pos))
                / 3.0;
            c.to_array()
        }));
        self.geo.revision += 1;
        self.blend.key = None;

        let mk = |label: &str, data: &[u8]| -> Option<wgpu::Buffer> {
            if data.is_empty() {
                None
            } else {
                Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents: data,
                    usage: wgpu::BufferUsages::VERTEX,
                }))
            }
        };
        self.geo.tris = mk("varstotas tris", bytemuck::cast_slice(&tris));
        self.geo.backs = mk("varstotas backs", bytemuck::cast_slice(&backs));
        self.geo.opaque_verts = opaque_verts;
        self.geo.translucent_verts = translucent_verts;

        let ne = geo.edges.len() - geo.edges.len() % 2;
        self.geo.edges = mk("varstotas edges", bytemuck::cast_slice(&geo.edges[..ne]));
        self.geo.edge_verts = ne as u32;
        let edges_thick: Vec<ThickLine> = geo.edges[..ne]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| ThickLine {
                a: p[0].pos,
                width: 0.0, // 0 = one logical point
                b: p[1].pos,
                dash: 0.0,
                color: p[0].color,
            })
            .collect();
        self.geo.edges_thick = self.seg_buffer(device, "varstotas edges thick", &edges_thick);
        self.geo.edges_bg = self.seg_bind_group(device, self.geo.edges_thick.as_ref());

        let profiles: Vec<ThickLine> = geo
            .profile_edges
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| ThickLine {
                a: p[0].pos,
                width: -1.0,
                b: p[1].pos,
                dash: 0.0,
                color: p[0].color,
            })
            .collect();
        self.geo.profiles = self.seg_buffer(device, "varstotas profiles", &profiles);
        self.geo.profiles_bg = self.seg_bind_group(device, self.geo.profiles.as_ref());
        self.geo.profile_count = profiles.len() as u32;
        let origin = DVec3::from_array(geo.origin);
        self.geo.origin = if origin.is_finite() { origin } else { DVec3::ZERO };
        // The camera treats the scene as a ball around the world origin.
        self.geo.radius = geo.radius.max(0.0) as f64 + self.geo.origin.length();
        if self.timing.is_some() {
            eprintln!(
                "varstotas_render timing: set_geometry {} tris ({} translucent), {} edges: {:.2} ms",
                n / 3,
                translucent_verts / 3,
                ne / 2,
                t0.elapsed().as_secs_f64() * 1000.0
            );
        }
    }

    /// Thick-line segment buffer, usable as instanced vertex buffer and (if supported) storage.
    fn seg_buffer(&self, device: &wgpu::Device, label: &str, segs: &[ThickLine]) -> Option<wgpu::Buffer> {
        if segs.is_empty() {
            return None;
        }
        let mut usage = wgpu::BufferUsages::VERTEX;
        if self.seg_bgl.is_some() && std::mem::size_of_val(segs) as u64 <= self.max_storage_binding {
            usage |= wgpu::BufferUsages::STORAGE;
        }
        Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(segs),
            usage,
        }))
    }

    fn seg_bind_group(&self, device: &wgpu::Device, buf: Option<&wgpu::Buffer>) -> Option<wgpu::BindGroup> {
        let buf = buf?;
        if !buf.usage().contains(wgpu::BufferUsages::STORAGE) {
            return None;
        }
        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("varstotas segments"),
            layout: self.seg_bgl.as_ref()?,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buf.as_entire_binding(),
            }],
        }))
    }

    /// Re-sort the blended triangles back-to-front if the camera or the set changed.
    /// Returns the number of indices to draw from `blend.buf`.
    fn update_blend_order(&mut self, rs: &egui_wgpu::RenderState, camera: &Camera, all: bool) -> u32 {
        let g = &self.geo;
        let total = g.opaque_verts / 3 + g.translucent_verts / 3;
        let first = if all { 0 } else { g.opaque_verts / 3 };
        let n = (total - first) as usize;
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
        let cents = &g.centroids[first as usize..total as usize];
        s.keys.clear();
        s.tris.clear();
        if camera.ortho {
            s.keys.extend(cents.iter().map(|c| !depth_key((glam::Vec3::from(*c) - eye).dot(f))));
        } else {
            s.keys.extend(cents.iter().map(|c| !depth_key((glam::Vec3::from(*c) - eye).length_squared())));
        }
        s.tris.extend(first..total);
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

        // ---- uniforms -------------------------------------------------------
        let aspect = size[0] as f64 / size[1] as f64;
        let b = camera.basis();
        let scene_radius = self.geo.radius;
        // Relative-to-eye: vertices are f32 relative to `geo.origin` (world, f64). The shader
        // subtracts `origin` (≈ eye in that local frame, f32-representable) before the f32
        // matrix, so precision is relative to the camera, not to the world origin.
        let geo_origin = self.geo.origin;
        let origin = (camera.eye - geo_origin).as_vec3().as_dvec3();
        let view_proj: DMat4 = camera.proj(aspect, scene_radius)
            * camera.view()
            * DMat4::from_translation(geo_origin + origin);
        let eye_rel = camera.eye - geo_origin - origin;
        let viewport_pts = DVec2::new(size[0] as f64 / ppp as f64, size[1] as f64 / ppp as f64);
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
        let light = if ld.length_squared() > 1e-12 { ld.normalize() } else { DVec3::Z };
        let (bias_rel, bias_abs) = if camera.ortho {
            let (n, f) = camera.ortho_depth_range(scene_radius);
            let px = camera.pixel_size_at(camera.target, viewport_pts);
            (0.0, (px * 0.75 / (f - n)).max(2e-6) as f32)
        } else {
            (4e-4f32, 0.0)
        };
        // Shading is done in linear space; uniforms are converted once here.
        let conv = srgb_to_linear;
        let face_mode = match style.face_style {
            FaceStyle::Shaded | FaceStyle::Wireframe => 0.0,
            FaceStyle::Monochrome => 1.0,
            FaceStyle::HiddenLine => 2.0,
        };
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
                size[0] as f32,
                size[1] as f32,
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
            params2: [bias_abs, if style.show_ground { 1.0 } else { 0.0 }, 0.0, 0.0],
        };
        globals.hidden_face[3] = 1.0;
        queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));

        // ---- per-frame buffers (reused scratch vectors) --------------------
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
        // Overlay lines: depth-tested first, then on-top.
        let lines = &mut self.scratch_lines;
        lines.clear();
        for pass_top in [false, true] {
            lines.extend(overlay.lines.iter().filter(|l| l.on_top == pass_top).map(|l| ThickLine {
                a: to_local(l.a),
                width: l.width_px.max(0.5),
                b: to_local(l.b),
                dash: if l.dashed { 9.0 } else { 0.0 },
                color: l.color,
            }));
        }
        let lines_depth = overlay.lines.iter().filter(|l| !l.on_top).count() as u32;
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

        let axes = &mut self.scratch_axes;
        axes.clear();
        if style.show_axes {
            axes_lines(axes, camera, scene_radius, to_local([0.0; 3]));
        }
        let axes_count = axes.len() as u32;
        self.axes.write(device, queue, bytemuck::cast_slice(axes));

        // Translucent faces / X-ray: back-to-front index order (re-sorted on camera change).
        let draw_faces = style.face_style != FaceStyle::Wireframe;
        let opaque_all = matches!(style.face_style, FaceStyle::Monochrome | FaceStyle::HiddenLine);
        let blend_count = if draw_faces && (style.xray || !opaque_all) {
            self.update_blend_order(rs, camera, style.xray)
        } else {
            0
        };

        // ---- encode --------------------------------------------------------
        let t = self.targets.as_ref().expect("targets");
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("varstotas render"),
        });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("varstotas scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &t.msaa_view,
                    depth_slice: None,
                    resolve_target: Some(&t.resolve_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &t.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.bind_group, &[]);
            let p = &self.pipelines;
            let g = &self.geo;

            // 1. sky
            pass.set_pipeline(&p.sky);
            pass.draw(0..3, 0..1);

            let set_face_buffers = |pass: &mut wgpu::RenderPass<'_>| {
                if let (Some(tb), Some(bb)) = (&g.tris, &g.backs) {
                    pass.set_vertex_buffer(0, tb.slice(..));
                    pass.set_vertex_buffer(1, bb.slice(..));
                    true
                } else {
                    false
                }
            };
            let total_face_verts = g.opaque_verts + g.translucent_verts;

            // 2. opaque faces
            if draw_faces && !style.xray && set_face_buffers(&mut pass) {
                pass.set_pipeline(&p.face_opaque);
                let end = if opaque_all { total_face_verts } else { g.opaque_verts };
                if end > 0 {
                    pass.draw(0..end, 0..1);
                }
            }

            // 3. edges (write depth so x-ray / translucent faces don't wash them out)
            if (style.show_edges || style.face_style == FaceStyle::Wireframe) && g.edge_verts > 0 {
                if ppp >= 1.5 {
                    // HiDPI: 1 physical px is too faint; draw 1 logical point quads.
                    if let (Some(pl), Some(bg)) = (&p.edge_thick_seg, &g.edges_bg) {
                        pass.set_pipeline(pl);
                        pass.set_bind_group(1, bg, &[]);
                        pass.draw(0..g.edge_verts * 3, 0..1); // 6 vertices per segment
                    } else if let Some(eb) = &g.edges_thick {
                        pass.set_pipeline(&p.edge_thick);
                        pass.set_vertex_buffer(0, eb.slice(..));
                        pass.draw(0..6, 0..g.edge_verts / 2);
                    }
                } else if let Some(eb) = &g.edges {
                    pass.set_pipeline(&p.edge);
                    pass.set_vertex_buffer(0, eb.slice(..));
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
                } else {
                    pass.set_pipeline(&p.thick_depth);
                    pass.set_vertex_buffer(0, pb.slice(..));
                    pass.draw(0..6, 0..g.profile_count);
                }
            }

            // 5. axes
            if axes_count > 0 {
                pass.set_pipeline(&p.thick_depth);
                pass.set_vertex_buffer(0, self.axes.buf.slice(..));
                pass.draw(0..6, 0..axes_count);
            }

            // 6. ground (translucent, tints geometry below z = 0)
            if style.show_ground {
                pass.set_pipeline(&p.ground);
                pass.draw(0..3, 0..1);
            }

            // 7. translucent faces / x-ray: sorted back-to-front, no depth write, both sides
            //    in one pass (the fragment shader picks the front/back color).
            if blend_count > 0 && set_face_buffers(&mut pass) {
                pass.set_pipeline(&p.face_blend);
                pass.set_index_buffer(self.blend.buf.buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..blend_count, 0, 0..1);
            }

            // 8. overlays: depth-tested, then on top
            if polys_depth > 0 {
                pass.set_pipeline(&p.poly_depth);
                pass.set_vertex_buffer(0, self.overlay_polys.buf.slice(..));
                pass.draw(0..polys_depth, 0..1);
            }
            if lines_depth > 0 {
                pass.set_pipeline(&p.thick_depth);
                pass.set_vertex_buffer(0, self.overlay_lines.buf.slice(..));
                pass.draw(0..6, 0..lines_depth);
            }
            if polys_top > 0 {
                pass.set_pipeline(&p.poly_top);
                pass.set_vertex_buffer(0, self.overlay_polys.buf.slice(..));
                pass.draw(polys_depth..polys_depth + polys_top, 0..1);
            }
            if lines_top > 0 {
                pass.set_pipeline(&p.thick_top);
                pass.set_vertex_buffer(0, self.overlay_lines.buf.slice(..));
                pass.draw(0..6, lines_depth..lines_depth + lines_top);
            }
        }
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
                    "varstotas_render timing ({} frames): cpu {:.3} ms/frame (incl. sort {:.3} ms, {} sorts), gpu wait {:.3} ms/frame",
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

    /// The egui texture id of the resolved image (after the first `render`).
    pub fn texture_id(&self) -> Option<egui::TextureId> {
        self.texture_id
    }
}

/// classic axes: solid in the positive direction, dotted in the negative.
fn axes_lines(v: &mut Vec<ThickLine>, camera: &Camera, scene_radius: f64, o: [f32; 3]) {
    // Long enough to look infinite at the current zoom level.
    let len = (camera.distance() * 50.0)
        .max(scene_radius * 4.0)
        .max(camera.ortho_height * 50.0)
        .max(camera.eye.length() * 4.0)
        .min(1e9) as f32;
    let cols = [
        [0.85, 0.0, 0.0, 1.0],
        [0.0, 0.62, 0.0, 1.0],
        [0.0, 0.0, 0.9, 1.0],
    ];
    for (i, c) in cols.iter().enumerate() {
        let mut e = o;
        e[i] += len;
        let mut ne = o;
        ne[i] -= len;
        v.push(ThickLine { a: o, width: 1.0, b: e, dash: 0.0, color: *c });
        v.push(ThickLine { a: o, width: 1.0, b: ne, dash: -4.0, color: *c });
    }
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

fn build_pipelines(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    seg_layout: Option<&wgpu::PipelineLayout>,
    color_format: wgpu::TextureFormat,
) -> Pipelines {
    let tri_attrs = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x4];
    let back_attrs = wgpu::vertex_attr_array![3 => Float32x4];
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
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(d.label),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some(d.vs),
                compilation_options: Default::default(),
                buffers: d.buffers,
            },
            primitive: wgpu::PrimitiveState {
                topology: d.topology,
                cull_mode: d.cull,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(d.depth_write),
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
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: if d.blend { Some(wgpu::BlendState::ALPHA_BLENDING) } else { None },
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
    let seg = |label: &'static str, depth_write: bool| {
        seg_layout.map(|l| {
            make_with(
                Desc {
                    label,
                    vs: "vs_thick_seg",
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
        edge_thick_seg: seg("varstotas edges thick (storage)", true),
        thick_depth_seg: seg("varstotas thick lines (storage)", false),
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
        face_opaque: make(Desc {
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
        }),
        face_blend: make(Desc {
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
        }),
        edge_thick: make(Desc {
            label: "varstotas edges thick",
            vs: "vs_thick",
            fs: "fs_thick",
            buffers: &thick_layouts,
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
            buffers: &line_layouts,
            topology: LineList,
            blend: true,
            depth_write: true,
            depth_compare: GreaterEqual,
            bias: no_bias,
            cull: None,
        }),
        thick_depth: make(Desc {
            label: "varstotas thick lines",
            vs: "vs_thick",
            fs: "fs_thick",
            buffers: &thick_layouts,
            topology: TriangleList,
            blend: true,
            depth_write: false,
            depth_compare: GreaterEqual,
            bias: no_bias,
            cull: None,
        }),
        thick_top: make(Desc {
            label: "varstotas thick lines on top",
            vs: "vs_thick",
            fs: "fs_thick",
            buffers: &thick_layouts,
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layouts_match_shader() {
        assert_eq!(std::mem::size_of::<Globals>(), 64 + 16 * 16);
        assert_eq!(std::mem::size_of::<ThickLine>(), 48);
        assert_eq!(std::mem::size_of::<TriVertex>(), 40);
        assert_eq!(std::mem::size_of::<LineVertex>(), 28);
    }

    #[test]
    fn shader_parses() {
        // Validate WGSL at test time via naga (re-exported by wgpu).
        let src = include_str!("shader.wgsl");
        let module = wgpu::naga::front::wgsl::parse_str(src).expect("wgsl parse");
        let mut v = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::empty(),
        );
        v.validate(&module).expect("wgsl validate");
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
