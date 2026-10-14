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
    /// sRGB color, alpha < 1 = translucent (drawn after opaque, sorted back-to-front).
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
    /// Scene radius around `origin` for clip planes.
    pub radius: f32,
    /// World position (f64) that all positions above are relative to (default: world origin).
    /// Put it near the model (e.g. bounding-box center) to keep f32 precision for models far
    /// from the world origin. Overlay coordinates stay in world space.
    pub origin: [f64; 3],
    /// Optional highlight ids (see `Renderer::update_highlight`): one per triangle (`tris.len() / 3`),
    /// e.g. the index of the source face. Empty = the triangle index is its id.
    pub face_ids: Vec<u32>,
    /// One per edge segment (`edges.len() / 2`). Empty = the segment index is its id.
    pub edge_ids: Vec<u32>,
    /// One per profile segment (`profile_edges.len() / 2`), in the same id space as `edge_ids`.
    /// Empty = profiles are never highlighted.
    pub profile_ids: Vec<u32>,
}

/// Per-primitive highlight state applied by the renderer (selection / editing context).
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Highlight {
    #[default]
    None = 0,
    /// Faces tinted toward `HIGHLIGHT_SELECTED`, edges/profiles drawn in it.
    Selected = 1,
    /// Outside the active editing context: faces faded toward `DIM_FACE_TARGET`, edges `DIM_EDGE_COLOR`.
    Dimmed = 2,
}

/// Full highlight state for `Renderer::update_highlight`, indexed by highlight id.
#[derive(Clone, Copy, Debug, Default)]
pub struct HighlightUpdate<'a> {
    /// `faces[face_id]`; ids past the end are `Highlight::None`.
    pub faces: &'a [Highlight],
    /// `edges[edge_id]` (edges and profiles); ids past the end are `Highlight::None`.
    pub edges: &'a [Highlight],
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

impl Overlay3d {
    /// Add the 12 edges of a (possibly transformed) box as depth-tested solid lines, e.g. the
    /// classic bounding box of a selected group / component instance (QA2-005):
    /// `overlay.add_box(corners, HIGHLIGHT_SELECTED, 1.0)`.
    /// `corners[i]` is the corner with x = max if `i & 1`, y = max if `i & 2`, z = max if `i & 4`
    /// (local min/max mapped through the instance transform), in world coordinates.
    pub fn add_box(&mut self, corners: [[f32; 3]; 8], color: [f32; 4], width_px: f32) {
        const EDGES: [(usize, usize); 12] = [
            (0, 1), (2, 3), (4, 5), (6, 7), // along x
            (0, 2), (1, 3), (4, 6), (5, 7), // along y
            (0, 4), (1, 5), (2, 6), (3, 7), // along z
        ];
        self.lines.extend(EDGES.iter().map(|&(a, b)| OverlayLine {
            a: corners[a],
            b: corners[b],
            color,
            width_px,
            dashed: false,
            on_top: false,
        }));
    }

    /// Axis-aligned convenience for `add_box`.
    pub fn add_aabb(&mut self, min: [f32; 3], max: [f32; 3], color: [f32; 4], width_px: f32) {
        let c = |i: usize| {
            [
                if i & 1 == 0 { min[0] } else { max[0] },
                if i & 2 == 0 { min[1] } else { max[1] },
                if i & 4 == 0 { min[2] } else { max[2] },
            ]
        };
        self.add_box(std::array::from_fn(c), color, width_px);
    }
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
    /// Direction toward the sun in world space (need not be normalized). Fixed, not camera-relative.
    pub light_dir: [f32; 3],
    /// Ambient shade 0..1 for faces turned away from the sun (never black).
    pub ambient: f32,
}

// `impl Default for RenderStyle` lives in renderer.rs (classic light theme).

// ---------------------------------------------------------------------------
// Renderer — see renderer.rs
// ---------------------------------------------------------------------------

mod renderer;
pub use renderer::{
    DEFAULT_AMBIENT, DEFAULT_BACK_COLOR, DEFAULT_FRONT_COLOR, DEFAULT_LIGHT_DIR, DIM_EDGE_COLOR, DIM_FACE_MIX,
    DIM_FACE_TARGET, HIGHLIGHT_FACE_MIX, HIGHLIGHT_SELECTED, Renderer,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aabb_overlay_has_12_axis_aligned_edges() {
        let mut o = Overlay3d::default();
        o.add_aabb([0.0, 0.0, 0.0], [1.0, 2.0, 3.0], HIGHLIGHT_SELECTED, 1.0);
        assert_eq!(o.lines.len(), 12);
        let mut per_axis = [0; 3];
        for l in &o.lines {
            let d: Vec<f32> = (0..3).map(|k| (l.b[k] - l.a[k]).abs()).collect();
            let axis = d.iter().position(|&x| x > 0.0).unwrap();
            assert_eq!(d.iter().filter(|&&x| x > 0.0).count(), 1);
            assert_eq!(d[axis], [1.0, 2.0, 3.0][axis]);
            per_axis[axis] += 1;
            assert!(!l.on_top && !l.dashed);
        }
        assert_eq!(per_axis, [4, 4, 4]);
    }
}
