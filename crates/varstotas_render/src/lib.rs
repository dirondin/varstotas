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

/// Texture coordinates of a face vertex (parallel to `SceneGeometry::tris`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexVertex {
    /// Planar coordinates in mm; the shader divides by the layer's tile size.
    pub uv: [f32; 2],
    /// Texture layer + 1 for the front and back side (0 = untextured), see `Renderer::set_textures`.
    pub layers: [u16; 2],
    /// PBR material + 1 for the front and back side (0 = the default material), see
    /// `Renderer::set_materials`; used by `FaceStyle::Realistic`.
    pub mats: [u16; 2],
}

/// A material of the Realistic style (`Renderer::set_materials`). The base colour and its map
/// come with the vertices (`TriVertex::color`, `TexVertex::layers`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PbrMaterial {
    pub metallic: f32,
    pub roughness: f32,
    /// 0 = opaque, 1 = clear glass. The vertices' alpha decides which faces are blended.
    pub transmission: f32,
    pub ior: f32,
    /// Emitted light, linear RGB (strength applied).
    pub emission: [f32; 3],
    pub normal_scale: f32,
    /// Colour-texture layer + 1 of the emission map (0 = none), see `Renderer::set_textures`.
    pub emission_layer: u16,
    /// Data-texture layer + 1 (`Renderer::set_data_textures`) of the packed occlusion (R),
    /// roughness (G) and metalness (B) map, as in glTF; 0 = none.
    pub orm_layer: u16,
    /// Which channels of the packed map are present: bit 0 occlusion, 1 roughness, 2 metalness.
    pub orm_mask: u8,
    /// Data-texture layer + 1 of the tangent-space normal map (0 = none).
    pub normal_layer: u16,
    /// Index + 1 of the material's surface in `Renderer::set_custom_shaders` (0 = none).
    pub shader: u16,
    /// Values of the custom surface's parameters.
    pub params: [f32; 8],
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            metallic: 0.0,
            roughness: 0.6,
            transmission: 0.0,
            ior: 1.5,
            emission: [0.0; 3],
            normal_scale: 1.0,
            emission_layer: 0,
            orm_layer: 0,
            orm_mask: 0,
            normal_layer: 0,
            shader: 0,
            params: [0.0; 8],
        }
    }
}

/// An equirectangular HDR panorama for `Renderer::set_environment` (linear RGB, rows top to
/// bottom; the top row looks straight up, u = 0 looks along −X and grows counter-clockwise).
#[derive(Clone, Debug, PartialEq)]
pub struct EnvImage {
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<f32>,
}

/// What surrounds the model in `FaceStyle::Realistic`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnvSource {
    /// A sky built from `RenderStyle::sky_top`, `sky_horizon` and `ground`.
    Sky,
    /// One colour all around (sRGB).
    Color([f32; 3]),
    /// The panorama of `Renderer::set_environment` (the sky when there is none).
    Image,
    /// A daylight sky (Preetham) for the sun toward `sun` (unit, world) and the haze
    /// `turbidity` (2 clear … 10 hazy).
    Physical { sun: [f32; 3], turbidity: f32 },
}

/// Tone mapping of `FaceStyle::Realistic`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToneMap {
    Aces,
    Agx,
    Clamp,
}

/// Light and camera of `FaceStyle::Realistic`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvStyle {
    pub source: EnvSource,
    /// Turns the environment around the vertical axis, degrees.
    pub rotation_deg: f32,
    /// Environment light multiplier.
    pub intensity: f32,
    /// Draw the environment behind the model (else the style's sky gradient).
    pub background: bool,
    /// Sun strength (irradiance relative to the sky; 0 = no sun); direction = the shadows' sun
    /// or `RenderStyle::light_dir`.
    pub sun: f32,
    /// Exposure in stops.
    pub exposure: f32,
    pub tonemap: ToneMap,
    pub effects: Effects,
}

impl Default for EnvStyle {
    fn default() -> Self {
        Self {
            source: EnvSource::Sky,
            rotation_deg: 0.0,
            intensity: 1.0,
            background: true,
            sun: 3.0,
            exposure: 0.0,
            tonemap: ToneMap::Aces,
            effects: Effects::default(),
        }
    }
}

/// A stylized look of `FaceStyle::Realistic` (drawn by the effects pass).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Stylize {
    #[default]
    None,
    Pencil,
    Watercolor,
    Technical,
}

/// Screen effects of `FaceStyle::Realistic`; a zero strength / distance switches one off.
/// Lengths in mm.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Effects {
    /// Soft shadows (PCSS): apparent sun size, 0 = the usual filtered shadows.
    pub soft_shadows: f32,
    pub ao_strength: f32,
    pub ao_radius: f32,
    /// Screen-space reflections.
    pub reflections: f32,
    pub bloom: f32,
    /// Light above this (1 = white) glows.
    pub bloom_threshold: f32,
    /// Depth of field blur (0 = off).
    pub aperture: f32,
    /// 0 = what is in the middle of the view.
    pub focus_distance: f32,
    /// Distance at which fog hides half of the light (0 = no fog).
    pub fog_distance: f32,
    /// sRGB.
    pub fog_color: [f32; 3],
    /// 0 = even fog, 1 = haze near the ground.
    pub fog_height: f32,
    pub vignette: f32,
    /// Every material as matte clay.
    pub clay: bool,
    pub stylize: Stylize,
    pub high_quality: bool,
    /// Seconds, for animated custom surfaces.
    pub time: f32,
}

impl Effects {
    /// Needs the effects path (float scene target + full-screen passes).
    pub fn needs_post(&self) -> bool {
        self.ao_strength > 0.0
            || self.reflections > 0.0
            || self.bloom > 0.0
            || self.aperture > 0.0
            || self.fog_distance > 0.0
            || self.vignette > 0.0
            || self.stylize != Stylize::None
    }
}

/// A material texture for `Renderer::set_textures`.
#[derive(Clone, Debug, PartialEq)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    /// sRGB RGBA8, rows top to bottom.
    pub rgba: Vec<u8>,
    /// Tile size in mm (`[width, height]`).
    pub size_mm: [f32; 2],
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
    /// Texture coordinates per triangle vertex (same length as `tris`); empty = untextured.
    /// A textured side shows the texture (alpha < 0.5 is cut out) with the color's alpha.
    pub tex: Vec<TexVertex>,
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
    /// Screen-aligned text labels (`add_text`, `add_linear_dimension`, `add_leader_text`).
    pub texts: Vec<OverlayText>,
    /// Arrowheads / end marks / point markers (`add_end_mark`, `add_marker`).
    pub marks: Vec<OverlayMark>,
    /// Strokes with a continuous dash pattern (`add_polyline`, `add_arc`, `add_circle`).
    pub polylines: Vec<OverlayPolyline>,
    /// Extra axes for custom frames (`add_axes`).
    pub axes: Vec<OverlayAxes>,
}

// Text, marks, polylines, arcs, dimensions, custom axes — see overlay.rs
mod overlay;
pub use overlay::{
    AxesFrame, DEFAULT_MARK_SIZE, DEFAULT_TEXT_SIZE, DIMENSION_TEXT_GAP, DimensionStyle, HAlign, LeaderStyle,
    MarkShape, OverlayAxes, OverlayMark, OverlayPolyline, OverlayText, TEXT_PADDING, VAlign, arc_points, arc_segments,
};

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
/// Guide point depth offset toward the eye, in cross arm lengths (see `Overlay3d::add_guide_point`).
pub const GUIDE_POINT_PULL: f32 = 6.0;

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
    ///
    /// The cross is screen-aligned, so on a face that does not face the camera half of each arm
    /// would lie behind the face (QA4-002). It is therefore moved toward the eye along the view
    /// ray by `GUIDE_POINT_PULL` arm lengths (same screen position and size): fully visible on
    /// faces tilted up to ~75° from the view, still hidden by geometry further in front.
    pub fn add_guide_point(&mut self, camera: &Camera, viewport: glam::DVec2, p: glam::DVec3, selected: bool) {
        if !p.is_finite() {
            return;
        }
        let b = camera.basis();
        let pull = camera.pixel_size_at(p, viewport) * (GUIDE_POINT_SIZE * GUIDE_POINT_PULL) as f64;
        let q = if camera.ortho {
            p - b.forward * pull
        } else {
            let to_p = p - camera.eye;
            let dist = to_p.length();
            // never past half-way to the eye
            let k = if dist > 1e-9 { 1.0 - (pull / dist).min(0.5) } else { 1.0 };
            camera.eye + to_p * k
        };
        let s = camera.pixel_size_at(q, viewport) * GUIDE_POINT_SIZE as f64;
        let color = Self::guide_color(selected);
        for (u, v) in [(b.right + b.up, -b.right - b.up), (b.right - b.up, b.up - b.right)] {
            self.lines.push(OverlayLine {
                a: (q + u * s).as_vec3().to_array(),
                b: (q + v * s).as_vec3().to_array(),
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
    /// Physically based: PBR materials lit by the environment and the sun, tone mapped.
    Realistic,
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
    /// Sun shadows (shadow map); `None` = off (fixed `light_dir` shading).
    pub shadows: Option<ShadowStyle>,
    /// Drawing grid; `None` = hidden.
    pub grid: Option<GridStyle>,
    /// Environment, exposure and tone mapping of `FaceStyle::Realistic`.
    pub env: EnvStyle,
}

/// Shadow settings for a frame (the classic Shadows panel).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowStyle {
    /// Direction toward the sun (world, unit). Below the horizon (z <= 0) everything is in shade.
    pub sun_dir: [f64; 3],
    /// Shade of sunlit faces facing the sun, 0..1.
    pub light: f32,
    /// Shade of faces in shadow / turned away, 0..1.
    pub dark: f32,
    /// Faces receive shadows.
    pub on_faces: bool,
    /// The ground plane (z = 0) receives shadows.
    pub on_ground: bool,
}

// `impl Default for RenderStyle` lives in renderer.rs (classic light theme).

// ---------------------------------------------------------------------------
// Renderer — see renderer.rs
// ---------------------------------------------------------------------------

mod grid;
pub use grid::{GRID_MIN_PX, GridStyle};
mod custom;
pub use custom::CustomSurface;
pub mod shaders {
    //! Checking custom surface shaders before use.
    pub use crate::custom::check;
}
mod bvh;
mod oidn;
pub use oidn::available as oidn_available;
mod pathtrace;
pub use pathtrace::{PtLight, PtLightKind, PtSettings};
mod renderer;
mod textures;
pub use textures::{MAX_LAYERS as MAX_TEXTURE_LAYERS, MAX_MATERIALS as MAX_PBR_MATERIALS};
pub use renderer::{
    DEFAULT_AMBIENT, DEFAULT_BACK_COLOR, DEFAULT_FRONT_COLOR, DEFAULT_LIGHT_DIR, DIM_EDGE_COLOR, DIM_FACE_MIX,
    DIM_FACE_TARGET, ExportError, ExportOptions, HIGHLIGHT_FACE_MIX, HIGHLIGHT_SELECTED, MAX_EXPORT_DIM,
    OVERLAY_DASH_PERIOD, Renderer, RgbaImage, encode_exr,
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
        // guide point: pulled toward the eye, same screen position, arms still GUIDE_POINT_SIZE pts
        let vp = glam::DVec2::new(800.0, 600.0);
        let p = glam::DVec3::new(300.0, 200.0, 500.0);
        for ortho in [false, true] {
            let mut cam = Camera::default();
            cam.set_ortho(ortho);
            let mut o = Overlay3d::default();
            o.add_guide_point(&cam, vp, p, false);
            let f = |v: [f32; 3]| glam::Vec3::from(v).as_dvec3();
            let centre = (f(o.lines[0].a) + f(o.lines[0].b)) * 0.5;
            let sp = cam.world_to_screen(p, vp).unwrap();
            let sc = cam.world_to_screen(centre, vp).unwrap();
            assert!((sp - sc).length() < 0.05, "{sp} vs {sc}");
            let fwd = cam.basis().forward;
            let pulled = (p - centre).dot(fwd);
            let arm = cam.pixel_size_at(p, vp) * GUIDE_POINT_SIZE as f64;
            assert!(pulled > arm * 4.0, "pulled {pulled} arm {arm}");
            let a = cam.world_to_screen(f(o.lines[0].a), vp).unwrap();
            assert!(((a - sc).length() - GUIDE_POINT_SIZE as f64 * 2f64.sqrt()).abs() < 0.05);
        }
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
