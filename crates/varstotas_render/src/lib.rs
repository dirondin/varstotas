//! Varstotas renderer: wgpu (via egui_wgpu re-export) + camera math.
//!
//! CONTRACT (see docs/02-ARCHITECTURE.md). Owner: `render` agent.
//! Knows nothing about the kernel: consumes flat buffers built by the app.
//! Renders offscreen (MSAA + depth), resolves to a texture registered with egui.

pub use egui_wgpu::wgpu;

// ---------------------------------------------------------------------------
// Camera (f64, Z-up) — see camera.rs
// ---------------------------------------------------------------------------

mod camera;
pub use camera::{Camera, CameraBasis, StandardView};

// ---------------------------------------------------------------------------
// Scene data
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TriVertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    /// sRGB color, alpha < 1 = translucent (sorted/OIT left to renderer).
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineVertex {
    pub pos: [f32; 3],
    pub color: [f32; 4],
}

/// Static model geometry; re-uploaded only when the model or selection changes.
#[derive(Clone, Debug, Default)]
pub struct SceneGeometry {
    /// Triangle list. Two-sided: app provides front-side triangles; back-face color is
    /// supplied in `back_colors` per-vertex parallel array (same length as `tris`).
    pub tris: Vec<TriVertex>,
    pub back_colors: Vec<[f32; 4]>,
    /// Line list (pairs) for model edges.
    pub edges: Vec<LineVertex>,
    /// Edges drawn as thick "profiles" (silhouette-ish outlines), line list pairs.
    pub profile_edges: Vec<LineVertex>,
    /// Scene radius around origin for clip planes.
    pub radius: f32,
}

/// Per-frame 3D overlay line (tool previews, inference axes, guides).
#[derive(Clone, Copy, Debug)]
pub struct OverlayLine {
    pub a: [f32; 3],
    pub b: [f32; 3],
    pub color: [f32; 4],
    pub width_px: f32,
    pub dashed: bool,
    /// Draw without depth test (always visible).
    pub on_top: bool,
}

/// Per-frame translucent 3D polygon (e.g. protractor, highlighted face preview).
#[derive(Clone, Debug)]
pub struct OverlayPolygon {
    /// Triangle list.
    pub tris: Vec<[f32; 3]>,
    pub color: [f32; 4],
    pub on_top: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Overlay3d {
    pub lines: Vec<OverlayLine>,
    pub polygons: Vec<OverlayPolygon>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceStyle {
    Shaded,
    Monochrome,
    HiddenLine,
    Wireframe,
}

#[derive(Clone, Debug)]
pub struct RenderStyle {
    pub face_style: FaceStyle,
    pub xray: bool,
    pub show_edges: bool,
    pub profiles: bool,
    pub profile_width_px: f32,
    pub show_axes: bool,
    pub show_ground: bool,
    pub sky_top: [f32; 4],
    pub sky_horizon: [f32; 4],
    pub ground: [f32; 4],
    pub edge_color: [f32; 4],
}

// `impl Default for RenderStyle` lives in renderer.rs (classic light theme).

// ---------------------------------------------------------------------------
// Renderer — see renderer.rs
// ---------------------------------------------------------------------------

mod renderer;
pub use renderer::{DEFAULT_BACK_COLOR, DEFAULT_FRONT_COLOR, Renderer};
