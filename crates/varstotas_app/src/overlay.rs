//! 2D overlay markers drawn with egui's painter on top of the rendered 3D image
//! (inference markers, selection rectangle, tooltips). Screen positions are
//! viewport-local logical points (origin = top-left of the 3D view).

#![allow(dead_code)] // marker kinds are produced by tools/inference (Tools agent)

use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, vec2};
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_render::Camera;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkerShape {
    /// Filled circle (Endpoint, Midpoint).
    Dot,
    /// Hollow circle (Center).
    Ring,
    /// Filled square (On Edge).
    Square,
    /// Filled diamond (On Face).
    Diamond,
    /// ✕ (Intersection).
    Cross,
}

#[derive(Clone, Debug)]
pub enum Marker2d {
    /// Marker anchored at a world point.
    Point { at: DVec3, shape: MarkerShape, color: Color32 },
    /// Tooltip box next to the cursor (e.g. "Endpoint", "On Red Axis").
    CursorTooltip { text: String },
    /// Text label anchored at a world point (e.g. dimension readout).
    Label { at: DVec3, text: String },
    /// Screen-space rectangle (selection window). Dashed = crossing selection.
    ScreenRect { min: Pos2, max: Pos2, dashed: bool, color: Color32 },
    /// Screen-space line.
    ScreenLine { a: Pos2, b: Pos2, color: Color32, width: f32, dashed: bool },
    /// Tool cursor drawn at the pointer: a small crosshair (hotspot) plus the tool's icon,
    /// with an optional "+" badge (copy / keep face / guides). The OS cursor is hidden.
    ToolCursor { icon: crate::ui::icons::Icon, plus: bool },
    /// An image stretched over a world quad (corners at image (0,0), (1,0), (1,1), (0,1),
    /// image v up), faded to `alpha`: the full texture tile of Texture Position.
    TileImage { bytes: std::sync::Arc<[u8]>, corners: [DVec3; 4], alpha: f32 },
}

pub struct OverlayPainter<'a> {
    pub painter: &'a Painter,
    pub rect: Rect,
    pub camera: &'a Camera,
    pub cursor: Option<Pos2>,
}

impl OverlayPainter<'_> {
    fn to_screen(&self, p: DVec3) -> Option<Pos2> {
        let vp = DVec2::new(self.rect.width() as f64, self.rect.height() as f64);
        let s = self.camera.world_to_screen(p, vp)?;
        Some(self.rect.min + vec2(s.x as f32, s.y as f32))
    }

    pub fn paint(&self, markers: &[Marker2d]) {
        for m in markers {
            match m {
                Marker2d::Point { at, shape, color } => {
                    if let Some(p) = self.to_screen(*at) {
                        paint_marker(self.painter, p, *shape, *color);
                    }
                }
                Marker2d::CursorTooltip { text } => {
                    if let Some(c) = self.cursor {
                        // Right of the tool cursor glyph (crosshair + icon box spanning
                        // c + 7..31 px), so the glyph never covers the text.
                        tooltip(self.painter, c + vec2(TOOLTIP_DX, 12.0), text);
                    }
                }
                Marker2d::Label { at, text } => {
                    if let Some(p) = self.to_screen(*at) {
                        tooltip(self.painter, p + vec2(6.0, -22.0), text);
                    }
                }
                Marker2d::ScreenRect { min, max, dashed, color } => {
                    let r = Rect::from_two_pos(self.rect.min + min.to_vec2(), self.rect.min + max.to_vec2());
                    self.painter.rect_filled(r, 0.0, color.gamma_multiply(0.08));
                    let st = Stroke::new(1.0, *color);
                    if *dashed {
                        let pts = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
                        self.painter.extend(Shape::dashed_line(&pts, st, 5.0, 3.0));
                    } else {
                        self.painter.rect_stroke(r, 0.0, st, egui::StrokeKind::Middle);
                    }
                }
                Marker2d::ToolCursor { icon, plus } => {
                    if let Some(c) = self.cursor {
                        paint_tool_cursor(self.painter, c, *icon, *plus);
                    }
                }
                Marker2d::TileImage { bytes, corners, alpha } => self.paint_tile(bytes, corners, *alpha),
                Marker2d::ScreenLine { a, b, color, width, dashed } => {
                    let pts = [self.rect.min + a.to_vec2(), self.rect.min + b.to_vec2()];
                    let st = Stroke::new(*width, *color);
                    if *dashed {
                        self.painter.extend(Shape::dashed_line(&pts, st, 6.0, 4.0));
                    } else {
                        self.painter.line_segment(pts, st);
                    }
                }
            }
        }
    }
}

impl OverlayPainter<'_> {
    /// The image on a world quad: a grid of small quads, so the perspective stays close
    /// (cells behind the camera are skipped).
    fn paint_tile(&self, bytes: &std::sync::Arc<[u8]>, c: &[DVec3; 4], alpha: f32) {
        let Some(tex) = tile_texture(self.painter.ctx(), bytes) else { return };
        const N: usize = 12;
        let at = |s: f64, t: f64| {
            let a = c[0].lerp(c[1], s);
            let b = c[3].lerp(c[2], s);
            a.lerp(b, t)
        };
        let tint = Color32::from_white_alpha((alpha.clamp(0.0, 1.0) * 255.0) as u8);
        let mut mesh = egui::Mesh::with_texture(tex.id());
        let mut idx = vec![None; (N + 1) * (N + 1)];
        for j in 0..=N {
            for i in 0..=N {
                let (s, t) = (i as f64 / N as f64, j as f64 / N as f64);
                if let Some(p) = self.to_screen(at(s, t)) {
                    idx[j * (N + 1) + i] = Some(mesh.vertices.len() as u32);
                    mesh.vertices.push(egui::epaint::Vertex { pos: p, uv: egui::pos2(s as f32, 1.0 - t as f32), color: tint });
                }
            }
        }
        for j in 0..N {
            for i in 0..N {
                let k = |di: usize, dj: usize| idx[(j + dj) * (N + 1) + i + di];
                if let (Some(a), Some(b), Some(cc), Some(d)) = (k(0, 0), k(1, 0), k(1, 1), k(0, 1)) {
                    mesh.add_triangle(a, b, cc);
                    mesh.add_triangle(a, cc, d);
                }
            }
        }
        self.painter.with_clip_rect(self.rect).add(Shape::mesh(mesh));
    }
}

thread_local! {
    /// The last tile image's texture (keyed by its bytes), for [`Marker2d::TileImage`].
    static TILE_TEX: std::cell::RefCell<Option<(usize, usize, egui::TextureHandle)>> = const { std::cell::RefCell::new(None) };
}

/// The texture of a tile image: decoded once (at most 1024 px) and kept while it is shown.
fn tile_texture(ctx: &egui::Context, bytes: &std::sync::Arc<[u8]>) -> Option<egui::TextureHandle> {
    let key = (bytes.as_ptr() as usize, bytes.len());
    TILE_TEX.with(|c| {
        let mut c = c.borrow_mut();
        if let Some((p, l, t)) = c.as_ref()
            && (*p, *l) == key
        {
            return Some(t.clone());
        }
        let img = image::load_from_memory(bytes).ok()?;
        let img = if img.width().max(img.height()) > 1024 { img.thumbnail(1024, 1024) } else { img }.to_rgba8();
        let ci = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
        let t = ctx.load_texture("texture_position_tile", ci, egui::TextureOptions::LINEAR);
        *c = Some((key.0, key.1, t.clone()));
        Some(t)
    })
}

/// Horizontal offset of the cursor tooltip from the hotspot (clears the tool icon).
const TOOLTIP_DX: f32 = 35.0;

/// Crosshair at the hotspot `c` plus a small tool icon to its lower right.
pub fn paint_tool_cursor(painter: &Painter, c: Pos2, icon: crate::ui::icons::Icon, plus: bool) {
    let halo = Stroke::new(3.0, Color32::from_white_alpha(220));
    let ink = Stroke::new(1.0, Color32::from_gray(20));
    for st in [halo, ink] {
        painter.line_segment([c - vec2(7.0, 0.0), c + vec2(7.0, 0.0)], st);
        painter.line_segment([c - vec2(0.0, 7.0), c + vec2(0.0, 7.0)], st);
    }
    let r = Rect::from_min_size(c + vec2(8.0, 8.0), vec2(22.0, 22.0));
    painter.rect_filled(r.expand(1.0), 4.0, Color32::from_white_alpha(170));
    crate::ui::icons::paint_icon(painter, r, icon, false);
    if plus {
        let p = c + vec2(-9.0, 9.0);
        painter.circle(p, 5.0, Color32::WHITE, Stroke::new(1.0, Color32::from_gray(40)));
        let st = Stroke::new(1.6, Color32::from_gray(20));
        painter.line_segment([p - vec2(3.0, 0.0), p + vec2(3.0, 0.0)], st);
        painter.line_segment([p - vec2(0.0, 3.0), p + vec2(0.0, 3.0)], st);
    }
}

pub fn paint_marker(painter: &Painter, p: Pos2, shape: MarkerShape, color: Color32) {
    let outline = Stroke::new(1.0, Color32::from_black_alpha(160));
    match shape {
        MarkerShape::Dot => {
            painter.circle(p, 5.0, color, outline);
        }
        MarkerShape::Ring => {
            painter.circle_stroke(p, 6.0, Stroke::new(2.0, color));
        }
        MarkerShape::Square => {
            painter.rect(Rect::from_center_size(p, vec2(9.0, 9.0)), 0.0, color, outline, egui::StrokeKind::Middle);
        }
        MarkerShape::Diamond => {
            let r = 6.0;
            painter.add(Shape::convex_polygon(
                vec![p + vec2(0.0, -r), p + vec2(r, 0.0), p + vec2(0.0, r), p + vec2(-r, 0.0)],
                color,
                outline,
            ));
        }
        MarkerShape::Cross => {
            let st = Stroke::new(2.5, color);
            painter.line_segment([p + vec2(-5.0, -5.0), p + vec2(5.0, 5.0)], st);
            painter.line_segment([p + vec2(-5.0, 5.0), p + vec2(5.0, -5.0)], st);
        }
    }
}

/// Small rounded label box, classic tooltip.
pub fn tooltip(painter: &Painter, at: Pos2, text: &str) {
    let font = FontId::proportional(12.0);
    let galley = painter.layout_no_wrap(text.to_string(), font, Color32::from_gray(30));
    let r = Rect::from_min_size(at, galley.size() + vec2(10.0, 6.0));
    painter.rect(
        r,
        4.0,
        Color32::from_rgb(0xFF, 0xFD, 0xE8),
        Stroke::new(1.0, Color32::from_gray(150)),
        egui::StrokeKind::Middle,
    );
    painter.galley(r.min + vec2(5.0, 3.0), galley, Color32::BLACK);
}

/// Text helper used by the viewport HUD.
pub fn hud_text(painter: &Painter, pos: Pos2, align: Align2, text: &str, color: Color32) {
    painter.text(pos, align, text, FontId::proportional(12.0), color);
}
