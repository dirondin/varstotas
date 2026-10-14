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

/// Guide line / point colour (sRGB, classic dark grey); selected guides use `HIGHLIGHT_SELECTED`.
pub const GUIDE_COLOR: [f32; 4] = [0x50 as f32 / 255.0, 0x50 as f32 / 255.0, 0x60 as f32 / 255.0, 1.0];
/// Guide line width in logical points.
pub const GUIDE_WIDTH: f32 = 1.0;
/// Half size of the guide point cross in logical points.
pub const GUIDE_POINT_SIZE: f32 = 4.0;

impl Overlay3d {
    /// Colour for a guide in the given selection state.
    pub fn guide_color(selected: bool) -> [f32; 4] {
        if selected { HIGHLIGHT_SELECTED } else { GUIDE_COLOR }
    }

    /// Infinite guide line through `origin` along `dir` (world space), drawn as one dashed,
    /// depth-tested overlay line in `GUIDE_COLOR` / `HIGHLIGHT_SELECTED`. The segment is centred
    /// on the point of the line closest to the camera target and long enough to look infinite;
    /// the renderer clips it to the view (also behind the eye), so dashes stay crisp and stable.
    /// Cost: one instance per guide; hundreds of guides are negligible.
    pub fn add_guide_line(&mut self, camera: &Camera, origin: glam::DVec3, dir: glam::DVec3, selected: bool) {
        let d = dir.normalize_or_zero();
        if d == glam::DVec3::ZERO || !origin.is_finite() {
            return;
        }
        let mid = origin + d * (camera.target - origin).dot(d);
        // ~50 view sizes: endpoints are near the vanishing point in perspective, while the f32
        // overlay coordinates keep ~1e-6 relative precision (the line does not drift).
        let reach = camera.distance().max(camera.ortho_height).max(1.0) * 50.0;
        self.add_guide_segment(mid - d * reach, mid + d * reach, selected);
    }

    /// Finite guide segment (e.g. a Tape Measure guide preview) with the guide style.
    pub fn add_guide_segment(&mut self, a: glam::DVec3, b: glam::DVec3, selected: bool) {
        self.lines.push(OverlayLine {
            a: a.as_vec3().to_array(),
            b: b.as_vec3().to_array(),
            color: Self::guide_color(selected),
            width_px: GUIDE_WIDTH,
            dashed: true,
            on_top: false,
        });
    }

    /// Guide point as a small screen-aligned cross (`GUIDE_POINT_SIZE` points), depth-tested.
    /// Drawn in 3D (unlike a 2D egui marker) so it also appears in `Renderer::render_to_image`.
    /// `viewport` = viewport size in logical points (as for `Camera::pixel_size_at`).
    pub fn add_guide_point(&mut self, camera: &Camera, viewport: glam::DVec2, p: glam::DVec3, selected: bool) {
        if !p.is_finite() {
            return;
        }
        let s = camera.pixel_size_at(p, viewport) * GUIDE_POINT_SIZE as f64;
        let b = camera.basis();
        let color = Self::guide_color(selected);
        for (u, v) in [(b.right + b.up, -b.right - b.up), (b.right - b.up, b.up - b.right)] {
            self.lines.push(OverlayLine {
                a: (p + u * s).as_vec3().to_array(),
                b: (p + v * s).as_vec3().to_array(),
                color,
                width_px: GUIDE_WIDTH,
                dashed: false,
                on_top: false,
            });
        }
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
    DIM_FACE_TARGET, ExportError, ExportOptions, HIGHLIGHT_FACE_MIX, HIGHLIGHT_SELECTED, MAX_EXPORT_DIM,
    OVERLAY_DASH_PERIOD, Renderer, RgbaImage,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guide_overlays() {
        let cam = Camera::default();
        let mut o = Overlay3d::default();
        o.add_guide_line(&cam, glam::DVec3::new(100.0, 0.0, 0.0), glam::DVec3::Y, false);
        o.add_guide_line(&cam, glam::DVec3::ZERO, glam::DVec3::ZERO, true); // degenerate: skipped
        o.add_guide_point(&cam, glam::DVec2::new(800.0, 600.0), glam::DVec3::new(0.0, 0.0, 500.0), true);
        assert_eq!(o.lines.len(), 3);
        let g = &o.lines[0];
        assert!(g.dashed && !g.on_top && g.color == GUIDE_COLOR);
        assert!(g.a[0] == 100.0 && g.b[0] == 100.0 && g.a[1] < -1e4 && g.b[1] > 1e4);
        // centred on the closest point to the camera target
        let mid_y = (g.a[1] + g.b[1]) * 0.5;
        assert!((mid_y as f64 - cam.target.y).abs() < 1.0);
        assert!(o.lines[1..].iter().all(|l| l.color == HIGHLIGHT_SELECTED && !l.dashed));
        // hundreds of guides: one overlay line each
        let mut many = Overlay3d::default();
        for i in 0..500 {
            many.add_guide_line(&cam, glam::DVec3::new(i as f64 * 10.0, 0.0, 0.0), glam::DVec3::X + glam::DVec3::Y, i % 7 == 0);
        }
        assert_eq!(many.lines.len(), 500);
    }

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
