//! Renderer side of the annotation overlays (see overlay.rs): CPU expansion of texts and marks
//! into screen-aligned sprite vertices, polyline dash phases, axes instances, and the glyph atlas.
//!
//! Text is laid out and rasterised by egui's own font stack (`epaint::text::Fonts`, the same
//! fonts and rasteriser as the UI; no extra dependency) into a glyph atlas that is uploaded to
//! the GPU only when new glyphs appear. Glyphs are rasterised at the target pixel density
//! (`pixels_per_point` in the viewport, `line_scale` in exports), so exported text is crisp.
//!
//! Sprites are anchored at a 3D point: the vertex shader projects the anchor, snaps it to the
//! pixel grid (unrotated text) and adds the vertex's pixel offset. Offsets are in whole-image
//! pixels, so a label crossing export tile borders is drawn identically by every tile.

use crate::wgpu;
use crate::{AxesFrame, Camera, HAlign, MarkShape, OverlayMark, OverlayPolyline, OverlayText, TEXT_PADDING, VAlign};
use egui::epaint::text::{FontDefinitions, FontId, Fonts, Galley, TextOptions};
use glam::{DVec2, DVec3};
use std::sync::Arc;

/// `SpriteVertex::mode` bit: multiply alpha by the glyph atlas coverage.
pub(super) const SPRITE_GLYPH: f32 = 1.0;
/// `SpriteVertex::mode` bit: do not snap the anchor to the pixel grid (rotated text, marks).
pub(super) const SPRITE_NO_SNAP: f32 = 2.0;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct SpriteVertex {
    /// Anchor in the geometry's local frame.
    pub anchor: [f32; 3],
    /// Sum of `SPRITE_*` bits.
    pub mode: f32,
    /// Offset from the (snapped) anchor in physical pixels, x right, y down.
    pub offset: [f32; 2],
    /// Glyph atlas texel coordinates (unused for solid sprites).
    pub uv: [f32; 2],
    /// sRGB.
    pub color: [f32; 4],
}

/// Overlay / axes thick-line instance (`ThickLine` + dash phase for polylines).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct LineInst {
    pub a: [f32; 3],
    pub width: f32,
    pub b: [f32; 3],
    /// Dash period in points; 0 = solid; negative = dotted.
    pub dash: f32,
    pub color: [f32; 4],
    /// Dash phase at `a` in points (continuous polylines).
    pub phase: f32,
    pub _pad: [f32; 3],
}

/// classic axis colours (sRGB): red, green, blue.
pub(super) const AXIS_COLORS: [[f32; 4]; 3] = [[0.85, 0.0, 0.0, 1.0], [0.0, 0.62, 0.0, 1.0], [0.0, 0.0, 0.9, 1.0]];

/// Projection of the frame / image being drawn (logical points, y down).
pub(super) struct ScreenProj<'a> {
    pub camera: &'a Camera,
    pub viewport: DVec2,
}

impl ScreenProj<'_> {
    fn to_screen(&self, p: DVec3) -> Option<DVec2> {
        self.camera.world_to_screen(p, self.viewport)
    }

    /// Unit screen direction (y down) of the world direction `d` at `p`.
    pub fn dir_at(&self, p: DVec3, d: DVec3) -> Option<DVec2> {
        let d = d.normalize_or_zero();
        if d == DVec3::ZERO {
            return None;
        }
        let eps = self.camera.pixel_size_at(p, self.viewport) * 8.0;
        let v = self.to_screen(p + d * eps)? - self.to_screen(p - d * eps)?;
        (v.length() > 1e-9).then(|| v.normalize())
    }
}

/// Text direction that reads left-to-right (or bottom-to-top for vertical lines).
pub(super) fn upright(d: DVec2) -> DVec2 {
    if d.x < -1e-4 || (d.x.abs() <= 1e-4 && d.y > 0.0) { -d } else { d }
}

fn v3(p: [f32; 3]) -> DVec3 {
    DVec3::new(p[0] as f64, p[1] as f64, p[2] as f64)
}

/// Emits rotated / translated sprite triangles for one anchor.
struct Emitter<'a> {
    out: &'a mut Vec<SpriteVertex>,
    anchor: [f32; 3],
    /// Rotation (cos, sin) in y-down screen space.
    rot: DVec2,
    color: [f32; 4],
    mode: f32,
}

impl Emitter<'_> {
    fn vert(&mut self, p: DVec2, uv: [f32; 2]) {
        let r = self.rot;
        let q = DVec2::new(p.x * r.x - p.y * r.y, p.x * r.y + p.y * r.x);
        self.out.push(SpriteVertex {
            anchor: self.anchor,
            mode: self.mode,
            offset: [q.x as f32, q.y as f32],
            uv,
            color: self.color,
        });
    }

    fn tri(&mut self, a: DVec2, b: DVec2, c: DVec2) {
        for p in [a, b, c] {
            self.vert(p, [0.0; 2]);
        }
    }

    /// Axis-aligned (in the sprite frame) rectangle.
    fn rect(&mut self, min: DVec2, max: DVec2) {
        let (a, b, c, d) = (min, DVec2::new(max.x, min.y), max, DVec2::new(min.x, max.y));
        self.tri(a, b, c);
        self.tri(a, c, d);
    }

    /// Stroke of width `w` from `p` to `q`.
    fn stroke(&mut self, p: DVec2, q: DVec2, w: f64) {
        let d = (q - p).normalize_or_zero();
        let n = d.perp() * (w * 0.5);
        self.tri(p - n, q - n, q + n);
        self.tri(p - n, q + n, p + n);
    }
}

/// Pixel-space layout of a text box (before rotation), for tests and `push_text`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TextBox {
    /// Box top-left and size in pixels relative to the anchor.
    pub min: DVec2,
    pub size: DVec2,
    /// Galley origin in pixels relative to the anchor.
    pub text_origin: DVec2,
}

pub(super) fn text_box(t: &OverlayText, galley_size_pt: DVec2, ppp: f64, snap: bool) -> TextBox {
    let pad = if t.background.is_some() || t.border.is_some() { TEXT_PADDING as f64 } else { 0.0 };
    let size = galley_size_pt + DVec2::splat(2.0 * pad);
    let x = match t.h_align {
        HAlign::Left => 0.0,
        HAlign::Center => -size.x * 0.5,
        HAlign::Right => -size.x,
    };
    let y = match t.v_align {
        VAlign::Top => 0.0,
        VAlign::Center => -size.y * 0.5,
        VAlign::Bottom => -size.y,
    };
    let min_pt = DVec2::new(x + t.offset_pt[0] as f64, y + t.offset_pt[1] as f64);
    let round = |v: DVec2| if snap { v.round() } else { v };
    TextBox {
        min: round(min_pt * ppp),
        size: round(size * ppp),
        text_origin: round((min_pt + DVec2::splat(pad)) * ppp),
    }
}

/// Expand a laid-out text label into sprite triangles (box, border, glyphs).
pub(super) fn push_text(out: &mut Vec<SpriteVertex>, t: &OverlayText, galley: &Galley, proj: &ScreenProj<'_>, ppp: f32, anchor: [f32; 3]) {
    let ppp = ppp as f64;
    let rot = t
        .along
        .and_then(|d| proj.dir_at(v3(t.pos), v3(d)))
        .map(upright)
        .unwrap_or(DVec2::X);
    let unrotated = (rot - DVec2::X).length() < 1e-6;
    let mode = if unrotated { 0.0 } else { SPRITE_NO_SNAP };
    let rect = galley.rect;
    let b = text_box(t, DVec2::new(rect.width() as f64, rect.height() as f64), ppp, unrotated);
    let mut e = Emitter { out, anchor, rot, color: t.color, mode };
    if let Some(bg) = t.background {
        e.color = bg;
        e.rect(b.min, b.min + b.size);
    }
    if let Some(bc) = t.border {
        e.color = bc;
        let w = ppp.max(1.0).round();
        let (min, max) = (b.min, b.min + b.size);
        e.rect(min, DVec2::new(max.x, min.y + w));
        e.rect(DVec2::new(min.x, max.y - w), max);
        e.rect(DVec2::new(min.x, min.y + w), DVec2::new(min.x + w, max.y - w));
        e.rect(DVec2::new(max.x - w, min.y + w), DVec2::new(max.x, max.y - w));
    }
    e.color = t.color;
    e.mode = mode + SPRITE_GLYPH;
    let origin = b.text_origin - DVec2::new(rect.min.x as f64, rect.min.y as f64) * ppp;
    for row in &galley.rows {
        let mesh = &row.visuals.mesh;
        let row_pos = DVec2::new(row.pos.x as f64, row.pos.y as f64);
        for &i in &mesh.indices {
            let v = &mesh.vertices[i as usize];
            let p = origin + (row_pos + DVec2::new(v.pos.x as f64, v.pos.y as f64)) * ppp;
            e.vert(p, [v.uv.x, v.uv.y]);
        }
    }
}

/// Expand an end mark / marker into sprite triangles.
pub(super) fn push_mark(out: &mut Vec<SpriteVertex>, m: &OverlayMark, proj: &ScreenProj<'_>, ppp: f32, anchor: [f32; 3]) {
    let ppp = ppp as f64;
    let rot = match m.from {
        Some(f) => match proj.dir_at(v3(m.pos), v3(m.pos) - v3(f)) {
            Some(d) => d,
            None => return, // degenerate or behind the eye
        },
        None => DVec2::X,
    };
    let s = m.size_pt.max(0.5) as f64 * ppp;
    let w = (m.width_pt.max(0.25) as f64 * ppp).max(1.0);
    let mut e = Emitter { out, anchor, rot, color: m.color, mode: SPRITE_NO_SNAP };
    let p = DVec2::new;
    match m.shape {
        MarkShape::Arrow => {
            let h = s * 0.28;
            e.tri(DVec2::ZERO, p(-s, h), p(-s, -h));
        }
        MarkShape::OpenArrow => {
            let h = s * 0.35;
            e.stroke(DVec2::ZERO, p(-s, h), w);
            e.stroke(DVec2::ZERO, p(-s, -h), w);
        }
        MarkShape::Dot => {
            let r = s * 0.35;
            let n = 16;
            for i in 0..n {
                let a0 = std::f64::consts::TAU * i as f64 / n as f64;
                let a1 = std::f64::consts::TAU * (i + 1) as f64 / n as f64;
                e.tri(DVec2::ZERO, DVec2::from_angle(a0) * r, DVec2::from_angle(a1) * r);
            }
        }
        MarkShape::Slash => {
            let a = s * 0.4;
            e.stroke(p(-a, a), p(a, -a), w);
        }
        MarkShape::Cross => {
            let a = s * 0.5;
            e.stroke(p(-a, 0.0), p(a, 0.0), w);
            e.stroke(p(0.0, -a), p(0.0, a), w);
        }
        MarkShape::XCross => {
            let a = s * 0.5 * std::f64::consts::FRAC_1_SQRT_2;
            e.stroke(p(-a, -a), p(a, a), w);
            e.stroke(p(-a, a), p(a, -a), w);
        }
        MarkShape::Square => {
            let a = s * 0.3;
            e.rect(p(-a, -a), p(a, a));
        }
        MarkShape::None => {}
    }
}

/// Polyline segments with a continuous dash phase (screen length from the first point, points).
pub(super) fn push_polyline(
    out: &mut Vec<LineInst>,
    pl: &OverlayPolyline,
    proj: &ScreenProj<'_>,
    dash_period: f32,
    to_local: impl Fn(DVec3) -> [f32; 3],
) {
    let n = pl.points.len();
    if n < 2 {
        return;
    }
    let segs = if pl.closed && n > 2 { n } else { n - 1 };
    let mut phase = 0.0f64;
    let mut prev = proj.to_screen(v3(pl.points[0]));
    for i in 0..segs {
        let (a, b) = (v3(pl.points[i]), v3(pl.points[(i + 1) % n]));
        out.push(LineInst {
            a: to_local(a),
            width: pl.width_px.max(0.5),
            b: to_local(b),
            dash: if pl.dashed { dash_period } else { 0.0 },
            color: pl.color,
            // modulo the period: exact continuity, small values (f32 precision)
            phase: if pl.dashed && dash_period > 0.0 { (phase % dash_period as f64) as f32 } else { 0.0 },
            _pad: [0.0; 3],
        });
        let next = proj.to_screen(b);
        phase = match (prev, next) {
            (Some(p), Some(q)) => phase + p.distance(q),
            _ => 0.0,
        };
        prev = next;
    }
}

/// Axes of `frame`: solid toward +, dotted toward -, each `len` long.
pub(super) fn push_axes(out: &mut Vec<LineInst>, frame: &AxesFrame, len: f64, width: f32, to_local: impl Fn(DVec3) -> [f32; 3]) {
    let o = frame.origin;
    if !o.is_finite() || !len.is_finite() || len <= 0.0 {
        return;
    }
    for (d, c) in [frame.x, frame.y, frame.z].into_iter().zip(AXIS_COLORS) {
        let d = d.normalize_or_zero();
        if d == DVec3::ZERO {
            continue;
        }
        let inst = |b: DVec3, dash: f32| LineInst {
            a: to_local(o),
            width,
            b: to_local(b),
            dash,
            color: c,
            phase: 0.0,
            _pad: [0.0; 3],
        };
        out.push(inst(o + d * len, 0.0));
        out.push(inst(o - d * len, -4.0));
    }
}

/// Length that makes axes look infinite at the current zoom level.
pub(super) fn infinite_axis_length(camera: &Camera, scene_radius: f64, origin: DVec3) -> f64 {
    (camera.distance() * 50.0)
        .max(scene_radius * 4.0)
        .max(camera.ortho_height * 50.0)
        .max((camera.eye - origin).length() * 4.0)
        .min(1e9)
}

/// Glyph atlas: egui's fonts (CPU layout + rasterisation) and the GPU texture with its bind group.
pub(super) struct TextAtlas {
    fonts: Option<Fonts>,
    options: TextOptions,
    pub bgl: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    size: [u32; 2],
    texture: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl TextAtlas {
    pub fn new(device: &wgpu::Device, max_texture_dim: u32) -> Self {
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("varstotas glyph atlas"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("varstotas glyph atlas"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let (texture, bind_group) = Self::create(device, &bgl, &sampler, [1, 1]);
        let options = TextOptions {
            max_texture_side: max_texture_dim.min(4096) as usize,
            // Raw coverage; the shader adapts it to linear-space blending.
            color_transfer_function: egui::epaint::FontColorTransferFunction::Off,
            ..Default::default()
        };
        TextAtlas { fonts: None, options, bgl, sampler, size: [1, 1], texture, bind_group }
    }

    fn create(
        device: &wgpu::Device,
        bgl: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        size: [u32; 2],
    ) -> (wgpu::Texture, wgpu::BindGroup) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("varstotas glyph atlas"),
            size: wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("varstotas glyph atlas"),
            layout: bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) },
            ],
        });
        (texture, bind_group)
    }

    /// Start a frame of text layout (creates the fonts on first use; recycles a full atlas).
    pub fn begin(&mut self) {
        match &mut self.fonts {
            Some(f) => f.begin_pass(self.options),
            None => self.fonts = Some(Fonts::new(self.options, FontDefinitions::default())),
        }
    }

    /// Lay out one label (after `begin`). Cached by egui across frames.
    pub fn layout(&mut self, text: &str, size_pt: f32, ppp: f32) -> Arc<Galley> {
        let fonts = self.fonts.get_or_insert_with(|| Fonts::new(self.options, FontDefinitions::default()));
        let size = if size_pt.is_finite() { size_pt.clamp(1.0, 512.0) } else { crate::DEFAULT_TEXT_SIZE };
        fonts
            .with_pixels_per_point(ppp)
            .layout_no_wrap(text.to_owned(), FontId::proportional(size), egui::epaint::Color32::WHITE)
    }

    /// Upload the atlas if glyphs were added since the last call (after all `layout`s).
    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let Some(fonts) = &mut self.fonts else { return };
        if fonts.font_image_delta().is_none() {
            return;
        }
        let img = fonts.image();
        let size = [img.size[0] as u32, img.size[1] as u32];
        if size != self.size {
            (self.texture, self.bind_group) = Self::create(device, &self.bgl, &self.sampler, size);
            self.size = size;
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&img.pixels),
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(size[0] * 4), rows_per_image: Some(size[1]) },
            wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Overlay3d, OverlayText};

    fn proj(cam: &Camera) -> ScreenProj<'_> {
        ScreenProj { camera: cam, viewport: DVec2::new(800.0, 600.0) }
    }

    fn top_view() -> Camera {
        let mut cam = Camera::default();
        cam.set_standard_view(crate::StandardView::Top);
        cam
    }

    #[test]
    fn upright_text_direction() {
        assert_eq!(upright(DVec2::X), DVec2::X);
        assert_eq!(upright(-DVec2::X), DVec2::X);
        // vertical lines read bottom-to-top (y down: (0, -1))
        assert_eq!(upright(DVec2::Y), -DVec2::Y);
        assert_eq!(upright(-DVec2::Y), -DVec2::Y);
        let d = DVec2::new(-0.6, 0.8);
        assert_eq!(upright(d), DVec2::new(0.6, -0.8));
    }

    #[test]
    fn screen_direction_in_top_view() {
        let cam = top_view();
        let p = proj(&cam);
        let d = p.dir_at(cam.target, DVec3::X).unwrap();
        assert!((d - DVec2::X).length() < 1e-6, "{d}");
        let d = p.dir_at(cam.target, DVec3::Y).unwrap();
        assert!((d + DVec2::Y).length() < 1e-6, "{d}"); // +Y is up on screen
        assert!(p.dir_at(cam.target, DVec3::Z).is_none()); // along the view direction
    }

    #[test]
    fn text_box_alignment_and_snapping() {
        let t = OverlayText::new(DVec3::ZERO, "x").align(HAlign::Center, VAlign::Bottom).offset(0.0, -3.0);
        let b = text_box(&t, DVec2::new(21.0, 14.0), 2.0, true);
        assert_eq!(b.size, DVec2::new(42.0, 28.0));
        assert_eq!(b.min, DVec2::new(-21.0, -34.0)); // above the anchor by the 3 pt gap
        assert_eq!(b.text_origin, b.min);
        let boxed = t.clone().background([1.0; 4]).align(HAlign::Left, VAlign::Top).offset(0.0, 0.0);
        let b = text_box(&boxed, DVec2::new(20.0, 14.0), 1.5, true);
        assert_eq!(b.min, DVec2::ZERO);
        assert_eq!(b.size, DVec2::new(39.0, 30.0)); // (20 + 6) * 1.5, (14 + 6) * 1.5
        assert_eq!(b.text_origin, DVec2::new(5.0, 5.0)); // 4.5 rounded: whole pixels for crisp glyphs
        let b = text_box(&boxed.align(HAlign::Right, VAlign::Center), DVec2::new(20.0, 14.0), 1.0, false);
        assert_eq!(b.min, DVec2::new(-26.0, -10.0));
    }

    #[test]
    fn text_layout_and_sprites() {
        // CPU-only: egui's fonts lay out and rasterise without a GPU.
        let mut fonts = Fonts::new(TextOptions::default(), FontDefinitions::default());
        let galley = fonts.with_pixels_per_point(2.0).layout_no_wrap("1250 mm".into(), FontId::proportional(12.0), egui::epaint::Color32::WHITE);
        assert!(galley.rect.width() > 30.0 && galley.rect.height() > 10.0);
        let cam = top_view();
        let p = proj(&cam);
        let mut out = Vec::new();
        let t = OverlayText::new(cam.target, "1250 mm").background([1.0; 4]).border([0.0, 0.0, 0.0, 1.0]);
        push_text(&mut out, &t, &galley, &p, 2.0, [0.0; 3]);
        let glyph_quads = "1250mm".len();
        assert_eq!(out.len(), 6 + 4 * 6 + glyph_quads * 6);
        assert!(out[..30].iter().all(|v| v.mode == 0.0));
        assert!(out[30..].iter().all(|v| v.mode == SPRITE_GLYPH && v.color == t.color));
        // centred: box symmetric around the anchor, whole pixels
        let xs: Vec<f32> = out[..6].iter().map(|v| v.offset[0]).collect();
        let (lo, hi) = (xs.iter().cloned().fold(f32::MAX, f32::min), xs.iter().cloned().fold(f32::MIN, f32::max));
        assert!((lo + hi).abs() <= 1.0 && lo.fract() == 0.0, "{lo} {hi}");
        // glyph quads use atlas texels and stay inside the box
        assert!(out[30..].iter().all(|v| v.offset[0] >= lo && v.offset[0] <= hi && v.uv[0] > 0.0));

        // along a direction going right-to-left on screen: flipped to read left-to-right
        let mut out = Vec::new();
        push_text(&mut out, &OverlayText::new(cam.target, "1250 mm").along(-DVec3::X), &galley, &p, 2.0, [0.0; 3]);
        assert!(out.iter().all(|v| v.mode == SPRITE_GLYPH));
        // along +Y (up on screen): rotated, unsnapped, the text runs upward
        let mut out = Vec::new();
        push_text(&mut out, &OverlayText::new(cam.target, "1250 mm").along(DVec3::Y), &galley, &p, 2.0, [0.0; 3]);
        assert!(out.iter().all(|v| v.mode == SPRITE_GLYPH + SPRITE_NO_SNAP));
        let ys: Vec<f32> = out.iter().map(|v| v.offset[1]).collect();
        let span_y = ys.iter().cloned().fold(f32::MIN, f32::max) - ys.iter().cloned().fold(f32::MAX, f32::min);
        assert!(span_y > galley.rect.width() * 2.0 * 0.9, "{span_y}");
    }

    #[test]
    fn marks_point_along_their_segment() {
        let cam = top_view();
        let p = proj(&cam);
        let c = cam.target;
        let mut o = Overlay3d::default();
        o.add_end_mark(c - DVec3::X * 100.0, c, MarkShape::Arrow, 10.0, [1.0; 4], true);
        o.add_end_mark(c + DVec3::Y * 100.0, c, MarkShape::Arrow, 10.0, [1.0; 4], true);
        let mut out = Vec::new();
        push_mark(&mut out, &o.marks[0], &p, 2.0, [0.0; 3]);
        // tip at the anchor, base 20 px to the left (screen), half-width 5.6 px
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].offset, [0.0, 0.0]);
        assert!((out[1].offset[0] + 20.0).abs() < 1e-3 && (out[1].offset[1].abs() - 5.6).abs() < 1e-3);
        assert_eq!(out[0].mode, SPRITE_NO_SNAP);
        out.clear();
        push_mark(&mut out, &o.marks[1], &p, 2.0, [0.0; 3]); // from above (+Y): points down on screen
        assert!(out[1].offset[1] < -19.0, "{:?}", out[1].offset);
        for (shape, n) in [(MarkShape::Dot, 48), (MarkShape::Cross, 12), (MarkShape::Slash, 6), (MarkShape::Square, 6)] {
            out.clear();
            push_mark(&mut out, &OverlayMark { shape, from: None, ..o.marks[0] }, &p, 1.0, [0.0; 3]);
            assert_eq!(out.len(), n, "{shape:?}");
        }
    }

    #[test]
    fn polyline_dash_phase_is_continuous() {
        let cam = top_view();
        let p = proj(&cam);
        let c = cam.target;
        let px = cam.pixel_size_at(c, p.viewport);
        let pts: Vec<DVec3> = (0..5).map(|i| c + DVec3::X * (i as f64 * 3.0 * px)).collect();
        let mut o = Overlay3d::default();
        o.add_polyline(&pts, false, [1.0; 4], 1.0, true, false);
        let mut out = Vec::new();
        push_polyline(&mut out, &o.polylines[0], &p, 9.0, |q| q.as_vec3().to_array());
        assert_eq!(out.len(), 4);
        for (i, l) in out.iter().enumerate() {
            assert!((l.phase - (3.0 * i as f32) % 9.0).abs() < 0.01, "{i}: {}", l.phase);
            assert_eq!(l.dash, 9.0);
        }
        o.polylines[0].closed = true;
        out.clear();
        push_polyline(&mut out, &o.polylines[0], &p, 9.0, |q| q.as_vec3().to_array());
        assert_eq!(out.len(), 5);
    }

    #[test]
    fn custom_frame_axes() {
        let f = AxesFrame { origin: DVec3::new(10.0, 20.0, 0.0), x: DVec3::new(1.0, 1.0, 0.0), y: DVec3::new(-2.0, 2.0, 0.0), z: DVec3::Z };
        let mut out = Vec::new();
        push_axes(&mut out, &f, 100.0, 1.0, |q| q.as_vec3().to_array());
        assert_eq!(out.len(), 6);
        assert!(out.iter().all(|l| l.a == [10.0, 20.0, 0.0]));
        let s = 100.0 / 2f32.sqrt();
        assert!((out[0].b[0] - (10.0 + s)).abs() < 1e-3 && (out[0].b[1] - (20.0 + s)).abs() < 1e-3);
        assert_eq!(out[0].dash, 0.0);
        assert_eq!(out[1].dash, -4.0); // negative half dotted
        assert!((out[2].b[0] - (10.0 - s)).abs() < 1e-3); // y axis normalised
        assert_eq!(out[2].color, AXIS_COLORS[1]);
    }
}
