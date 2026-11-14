//! What the graphics adapter can do, decided once (by the first renderer, or the report window)
//! from what the adapter and the driver say. The renderer takes its capability decisions from
//! here instead of trusting a graphics API to have what its standard promises: wgpu assumes
//! OpenGL ES 3.0's 4× multisampling, which virgl in a Linux VM on a macOS host does not have
//! (GL_MAX_SAMPLES 1: the multisampled depth was incomplete and the view black). One line of it
//! goes to the log and into Report a Problem.

use crate::renderer::post::{GBUF_FORMAT, HDR_FORMAT};
use crate::wgpu;
use std::sync::OnceLock;

/// Samples per pixel of the scene targets where the adapter can multisample.
const MSAA: u32 = 4;
/// Storage buffers the path tracer binds in its compute shaders (pathtrace.rs `storage_entry`).
const PT_STORAGE_BUFFERS: u32 = 8;

/// The renderer's features on this adapter.
#[derive(Clone, Debug)]
pub struct GpuCaps {
    pub backend: wgpu::Backend,
    pub name: String,
    /// A software adapter (llvmpipe, WARP, SwiftShader).
    pub software: bool,
    /// GL_MAX_SAMPLES of an OpenGL adapter.
    pub gl_max_samples: Option<i32>,
    /// Samples per pixel of the scene: 4, or 1 where the adapter cannot multisample its targets.
    pub samples: u32,
    /// sRGB views of the scene's colour target; without them the shaders apply the gamma.
    pub srgb_views: bool,
    /// Thick edges read from storage buffers in the vertex shader; without it, instanced quads.
    pub segments: bool,
    /// The effects of the Realistic style: float targets, a multisampled depth read in shaders.
    pub effects: bool,
    /// The effects' linear depth: R32Float, or R16Float where R32Float cannot be rendered to
    /// (OpenGL without EXT_color_buffer_float).
    pub lin_format: wgpu::TextureFormat,
    /// The path tracer (Render): compute shaders, its storage buffers and a storage texture.
    pub path_tracer: bool,
    /// Ray queries in the path tracer (else its own BVH traversal).
    pub ray_query: bool,
}

static CAPS: OnceLock<GpuCaps> = OnceLock::new();

impl GpuCaps {
    /// The capabilities of this render state's adapter (detected and logged on the first call).
    pub fn of(rs: &egui_wgpu::RenderState) -> &'static GpuCaps {
        CAPS.get_or_init(|| {
            let caps = Self::detect(&rs.adapter, &rs.device);
            log::info!("graphics features: {}", caps.summary());
            caps
        })
    }

    /// Detect what `device` on `adapter` can do.
    pub fn detect(adapter: &wgpu::Adapter, device: &wgpu::Device) -> Self {
        let info = adapter.get_info();
        let gl = info.backend == wgpu::Backend::Gl;
        let down = adapter.get_downlevel_capabilities().flags;
        let limits = device.limits();
        let features = |f| adapter.get_texture_format_features(f);
        let usable = |f, usage| features(f).allowed_usages.contains(usage);
        let msaa = |f| features(f).flags.sample_count_supported(MSAA);

        let srgb_views = down.contains(wgpu::DownlevelFlags::VIEW_FORMATS);
        let color = if srgb_views { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm };
        let gl_max_samples = gl_max_samples(adapter);
        let samples = if gl_max_samples.is_some_and(|n| n < MSAA as i32) || !msaa(color) || !msaa(crate::renderer::DEPTH_FORMAT) {
            1
        } else {
            MSAA
        };

        let attach = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let lin_format = if usable(wgpu::TextureFormat::R32Float, attach) { wgpu::TextureFormat::R32Float } else { wgpu::TextureFormat::R16Float };
        let hdr = features(HDR_FORMAT);
        let effects = samples > 1
            && hdr.allowed_usages.contains(attach | wgpu::TextureUsages::TEXTURE_BINDING)
            && hdr.flags.contains(wgpu::TextureFormatFeatureFlags::FILTERABLE | wgpu::TextureFormatFeatureFlags::BLENDABLE)
            && msaa(HDR_FORMAT)
            && msaa(GBUF_FORMAT)
            && usable(lin_format, attach)
            && usable(wgpu::TextureFormat::R16Float, attach)
            // multisampled textures in shaders: OpenGL ES 3.1, the version with compute shaders
            && (!gl || down.contains(wgpu::DownlevelFlags::COMPUTE_SHADERS));

        let path_tracer = down.contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
            && limits.max_storage_buffers_per_shader_stage >= PT_STORAGE_BUFFERS
            && limits.max_storage_textures_per_shader_stage >= 1
            && usable(wgpu::TextureFormat::Rgba8Unorm, wgpu::TextureUsages::STORAGE_BINDING);
        // OpenGL: the segments' storage buffers are not used there
        let segments = !gl && limits.max_storage_buffers_per_shader_stage > 0 && down.contains(wgpu::DownlevelFlags::VERTEX_STORAGE);

        Self {
            backend: info.backend,
            name: info.name,
            software: info.device_type == wgpu::DeviceType::Cpu,
            gl_max_samples,
            samples,
            srgb_views,
            segments,
            effects,
            lin_format,
            path_tracer,
            ray_query: path_tracer && device.features().contains(wgpu::Features::EXPERIMENTAL_RAY_QUERY),
        }
    }

    /// One line for the log and reports (not translated), e.g.
    /// `Gl virgl: msaa 1 (GL_MAX_SAMPLES 1), srgb views, effects off, path tracer off`.
    pub fn summary(&self) -> String {
        let on = |b: bool, name: &str| if b { name.to_string() } else { format!("{name} off") };
        let mut parts = vec![format!("msaa {}", self.samples)];
        if let Some(n) = self.gl_max_samples {
            parts[0].push_str(&format!(" (GL_MAX_SAMPLES {n})"));
        }
        parts.extend([
            on(self.srgb_views, "srgb views"),
            on(self.segments, "segments"),
            on(self.effects, "effects"),
            on(self.path_tracer, "path tracer"),
            on(self.ray_query, "ray queries"),
        ]);
        if self.software {
            parts.push("software".into());
        }
        format!("{:?} {}: {}", self.backend, self.name, parts.join(", "))
    }
}

/// GL_MAX_SAMPLES of an OpenGL adapter, which wgpu does not report (it assumes OpenGL ES 3.0's
/// 4 below 8); None on the other graphics APIs.
fn gl_max_samples(adapter: &wgpu::Adapter) -> Option<i32> {
    #[cfg(all(not(target_arch = "wasm32"), not(target_vendor = "apple")))]
    {
        use glow::HasContext;
        // SAFETY: reads one limit through the adapter's own GL context, changes nothing
        let hal = unsafe { adapter.as_hal::<wgpu::hal::api::Gles>() }?;
        let gl = hal.adapter_context().lock();
        Some(unsafe { gl.get_parameter_i32(glow::MAX_SAMPLES) })
    }
    #[cfg(any(target_arch = "wasm32", target_vendor = "apple"))]
    {
        let _ = adapter;
        None
    }
}
