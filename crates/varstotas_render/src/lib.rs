//! Varstotas renderer: wgpu (via egui_wgpu re-export) + camera math.
//!
//! CONTRACT (see docs/02-ARCHITECTURE.md). Owner: `render` agent.
//! Knows nothing about the kernel: consumes flat buffers built by the app.
//! Renders offscreen (MSAA + depth), resolves to a texture registered with egui.

pub use egui_wgpu::wgpu;
use glam::{DMat4, DVec2, DVec3};

// ---------------------------------------------------------------------------
// Camera (f64, Z-up)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StandardView {
    Iso,
    Top,
    Bottom,
    Front,
    Back,
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Camera {
    pub eye: DVec3,
    pub target: DVec3,
    pub up: DVec3,
    pub fov_y_deg: f64,
    /// Parallel projection; `ortho_height` is the visible world height in mm.
    pub ortho: bool,
    pub ortho_height: f64,
}

impl Default for Camera {
    /// classic default: iso-ish view of the origin, ~1.7 m person scale visible.
    fn default() -> Self {
        todo!()
    }
}

impl Camera {
    pub fn view(&self) -> DMat4 {
        todo!()
    }
    /// Reverse-Z or standard is the renderer's choice; near/far derived from scene `extent`.
    pub fn proj(&self, _aspect: f64, _scene_radius: f64) -> DMat4 {
        todo!()
    }
    /// World ray through a pixel. `px` in logical points, origin top-left.
    pub fn screen_ray(&self, _px: DVec2, _viewport: DVec2) -> (DVec3, DVec3) {
        todo!()
    }
    /// Project world point to logical-point screen coords; None if behind camera.
    pub fn world_to_screen(&self, _p: DVec3, _viewport: DVec2) -> Option<DVec2> {
        todo!()
    }
    /// World size of one screen pixel at point `p` (for pick tolerances / constant-size gizmos).
    pub fn pixel_size_at(&self, _p: DVec3, _viewport: DVec2) -> f64 {
        todo!()
    }
    /// Orbit around `pivot` by screen delta in points (feel: Z stays up).
    pub fn orbit(&mut self, _delta: DVec2, _pivot: DVec3) {
        todo!()
    }
    /// Pan so the world point under the cursor follows the cursor; `depth_point` is the
    /// point that was under the cursor when the drag began.
    pub fn pan(&mut self, _delta: DVec2, _depth_point: DVec3, _viewport: DVec2) {
        todo!()
    }
    /// Zoom toward world point `toward` (factor > 1 zooms in).
    pub fn zoom(&mut self, _factor: f64, _toward: DVec3) {
        todo!()
    }
    pub fn zoom_extents(&mut self, _min: DVec3, _max: DVec3, _aspect: f64) {
        todo!()
    }
    pub fn set_standard_view(&mut self, _v: StandardView) {
        todo!()
    }
}

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

impl Default for RenderStyle {
    fn default() -> Self {
        todo!()
    }
}

// ---------------------------------------------------------------------------
// Renderer
// ---------------------------------------------------------------------------

pub struct Renderer {
    // private GPU state
}

impl Renderer {
    /// Create from eframe's wgpu render state (`frame.wgpu_render_state()`).
    pub fn new(_rs: &egui_wgpu::RenderState) -> Self {
        todo!()
    }
    /// Upload new static geometry.
    pub fn set_geometry(&mut self, _rs: &egui_wgpu::RenderState, _geo: &SceneGeometry) {
        todo!()
    }
    /// Render the scene offscreen at `size_px` (physical pixels) and return an egui texture
    /// id to display with `egui::Image` sized to the viewport in points.
    pub fn render(
        &mut self,
        _rs: &egui_wgpu::RenderState,
        _camera: &Camera,
        _overlay: &Overlay3d,
        _style: &RenderStyle,
        _size_px: [u32; 2],
        _pixels_per_point: f32,
    ) -> egui::TextureId {
        todo!()
    }
}
