//! Offscreen render-to-image (File → Export → 2D Graphic / PNG).
//!
//! Renders the current scene (geometry, style, highlight states, the overlay passed in) into
//! offscreen targets of a fixed tile size and reads the tiles back into one RGBA8 buffer.
//! Tiling keeps GPU memory bounded (an MSAA + depth target of 2048² is ~130 MB; a single
//! 10000² target would be ~3 GB) and lifts the device's max texture size limit.

use super::{
    Camera, FrameCounts, HL_HIDE_DIMMED, HL_HIDE_SELECTED, Overlay3d, PassOpts, RenderStyle, Renderer, DEPTH_FORMAT,
    SAMPLES,
};
use crate::wgpu;

/// Largest supported export side in pixels (16384² RGBA8 = 1 GiB of CPU memory).
pub const MAX_EXPORT_DIM: u32 = 16384;

/// Tile edge used for exports (clamped to the device's max texture size).
const EXPORT_TILE: u32 = 2048;

/// Options for `Renderer::render_to_image`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExportOptions {
    /// No sky and no ground; pixels not covered by geometry / overlays are fully transparent.
    /// Antialiased edges and translucent faces get partial alpha (straight, not premultiplied).
    pub transparent_background: bool,
    /// Physical pixels per logical point for line widths and dash patterns (edges, profiles,
    /// overlay lines, axes). To keep the on-screen look when exporting at `k` times the viewport
    /// pixel size, pass `pixels_per_point * k`. Values >= 1.5 draw model edges as quads.
    pub line_scale: f32,
    /// Keep `Highlight::Selected` (selection tint / blue edges). Off for normal exports.
    pub show_selection: bool,
    /// Keep `Highlight::Dimmed` (faded geometry outside the open group / component).
    pub show_context_dimming: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions {
            transparent_background: false,
            line_scale: 1.0,
            show_selection: false,
            show_context_dimming: true,
        }
    }
}

/// Tightly packed RGBA8 image (sRGB, straight alpha, rows top to bottom, `width * 4` bytes
/// per row; no GPU row padding).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl RgbaImage {
    /// Encode as PNG (RGBA8, sRGB chunk). `dpi` (optional) is stored in the pHYs chunk.
    pub fn encode_png(&self, dpi: Option<f32>) -> std::io::Result<Vec<u8>> {
        let mut out = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut out, self.width, self.height);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
            enc.set_compression(png::Compression::Fast);
            if let Some(dpi) = dpi.filter(|d| d.is_finite() && *d > 0.0) {
                let ppm = (dpi / 0.0254).round() as u32;
                enc.set_pixel_dims(Some(png::PixelDimensions { xppu: ppm, yppu: ppm, unit: png::Unit::Meter }));
            }
            let mut w = enc.write_header().map_err(std::io::Error::other)?;
            w.write_image_data(&self.pixels).map_err(std::io::Error::other)?;
            w.finish().map_err(std::io::Error::other)?;
        }
        Ok(out)
    }

    /// `encode_png(None)` written to `path`.
    pub fn write_png(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        std::fs::write(path, self.encode_png(None)?)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportError {
    /// Width or height is 0.
    EmptySize,
    /// A side exceeds `MAX_EXPORT_DIM`.
    TooLarge { requested: [u32; 2], max: u32 },
    /// Readback failed (device lost, mapping error).
    Gpu(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::EmptySize => write!(f, "image size must be at least 1 x 1 pixels"),
            ExportError::TooLarge { requested, max } => write!(
                f,
                "image size {} x {} is too large (at most {max} pixels per side)",
                requested[0], requested[1]
            ),
            ExportError::Gpu(e) => write!(f, "GPU error while exporting the image: {e}"),
        }
    }
}

impl std::error::Error for ExportError {}

/// Whole-image NDC -> tile NDC (`tile = full * xy + zw`) for the tile with top-left pixel
/// `(x0, y0)` and size `tw x th` in an image of `w x h` pixels (y down).
fn tile_transform(w: u32, h: u32, x0: u32, y0: u32, tw: u32, th: u32) -> [f32; 4] {
    let (w, h, x0, y0, tw, th) = (w as f64, h as f64, x0 as f64, y0 as f64, tw as f64, th as f64);
    [
        (w / tw) as f32,
        (h / th) as f32,
        ((w - 2.0 * x0 - tw) / tw) as f32,
        ((th - h + 2.0 * y0) / th) as f32,
    ]
}

/// Premultiplied -> straight alpha, in place. `linear_blend`: colours were blended in linear
/// space and stored sRGB-encoded (sRGB render target), so divide in linear space.
fn unpremultiply(px: &mut [u8], linear_blend: bool) {
    let to_lin: [f32; 256] = std::array::from_fn(|i| {
        let x = i as f32 / 255.0;
        if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
    });
    let to_srgb = |x: f32| {
        let x = x.clamp(0.0, 1.0);
        let s = if x <= 0.003_130_8 { x * 12.92 } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 };
        (s * 255.0).round() as u8
    };
    for p in px.as_chunks_mut::<4>().0 {
        let a = p[3];
        if a == 255 {
            continue;
        }
        if a == 0 {
            p[..3].fill(0);
            continue;
        }
        let af = a as f32 / 255.0;
        for c in &mut p[..3] {
            *c = if linear_blend {
                to_srgb(to_lin[*c as usize] / af)
            } else {
                ((*c as f32 / af).round()).min(255.0) as u8
            };
        }
    }
}

impl Renderer {
    /// Render the scene into an image of `size` pixels for export (blocking: waits for the GPU).
    ///
    /// Uses the geometry from the last `set_geometry`, the highlight states from the last
    /// `update_highlight` (filtered by `opts`), `style` and the given `overlay` only — pass
    /// `&Overlay3d::default()` (or just the guides) to leave out tool previews and selection
    /// boxes. The camera aspect is taken from `size`. Images larger than the tile size are
    /// rendered in tiles, seamlessly. The viewport texture is not touched; the next `render`
    /// re-uploads its own per-frame data.
    pub fn render_to_image(
        &mut self,
        rs: &egui_wgpu::RenderState,
        camera: &Camera,
        overlay: &Overlay3d,
        style: &RenderStyle,
        size: [u32; 2],
        opts: &ExportOptions,
    ) -> Result<RgbaImage, ExportError> {
        let [w, h] = size;
        if w == 0 || h == 0 {
            return Err(ExportError::EmptySize);
        }
        if w > MAX_EXPORT_DIM || h > MAX_EXPORT_DIM {
            return Err(ExportError::TooLarge { requested: size, max: MAX_EXPORT_DIM });
        }
        let ppp = if opts.line_scale.is_finite() && opts.line_scale > 0.0 { opts.line_scale } else { 1.0 };
        let hide_mask = if opts.show_selection { 0 } else { HL_HIDE_SELECTED }
            | if opts.show_context_dimming { 0 } else { HL_HIDE_DIMMED };
        let tile = EXPORT_TILE.min(self.max_texture_dim);
        let (tw, th) = (tile.min(w), tile.min(h));

        let device = &rs.device;
        let extent = wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 };
        let target = |label, samples, format, usage| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: extent,
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        let attach = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let msaa = target("varstotas export msaa", SAMPLES, self.color_format, attach);
        let depth = target("varstotas export depth", SAMPLES, DEPTH_FORMAT, attach);
        let resolve = target("varstotas export resolve", 1, self.color_format, attach | wgpu::TextureUsages::COPY_SRC);
        let (msaa_view, depth_view, resolve_view) =
            (msaa.create_view(&Default::default()), depth.create_view(&Default::default()), resolve.create_view(&Default::default()));
        let row_bytes = tw * 4;
        let padded_row = row_bytes.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("varstotas export readback"),
            size: padded_row as u64 * th as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.update_shadow_map(rs, style);
        self.update_env(rs, style);
        let counts: FrameCounts = self.prepare_frame(rs, camera, overlay, style, size, ppp);
        let pass = PassOpts { ppp, background: !opts.transparent_background, selection_tint: opts.show_selection };
        let mut pixels = vec![0u8; w as usize * h as usize * 4];
        for y0 in (0..h).step_by(th as usize) {
            for x0 in (0..w).step_by(tw as usize) {
                let t = tile_transform(w, h, x0, y0, tw, th);
                let globals = self.globals(camera, style, size, [tw, th], ppp, t, hide_mask);
                rs.queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));
                let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("varstotas export"),
                });
                self.encode_scene(&mut enc, &msaa_view, &depth_view, &resolve_view, style, &counts, pass);
                let (cw, ch) = ((w - x0).min(tw), (h - y0).min(th));
                enc.copy_texture_to_buffer(
                    wgpu::TexelCopyTextureInfo {
                        texture: &resolve,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyBufferInfo {
                        buffer: &readback,
                        layout: wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(padded_row),
                            rows_per_image: Some(th),
                        },
                    },
                    wgpu::Extent3d { width: cw, height: ch, depth_or_array_layers: 1 },
                );
                let submission = rs.queue.submit(Some(enc.finish()));
                let slice = readback.slice(..);
                let (tx, rx) = std::sync::mpsc::channel();
                slice.map_async(wgpu::MapMode::Read, move |r| {
                    let _ = tx.send(r);
                });
                device
                    .poll(wgpu::PollType::Wait { submission_index: Some(submission), timeout: None })
                    .map_err(|e| ExportError::Gpu(e.to_string()))?;
                rx.recv()
                    .map_err(|e| ExportError::Gpu(e.to_string()))?
                    .map_err(|e| ExportError::Gpu(e.to_string()))?;
                {
                    let data = slice.get_mapped_range().map_err(|e| ExportError::Gpu(e.to_string()))?;
                    let row = cw as usize * 4;
                    for r in 0..ch as usize {
                        let src = &data[r * padded_row as usize..][..row];
                        let dst = ((y0 as usize + r) * w as usize + x0 as usize) * 4;
                        pixels[dst..dst + row].copy_from_slice(src);
                    }
                }
                readback.unmap();
            }
        }
        if opts.transparent_background {
            unpremultiply(&mut pixels, !self.manual_gamma);
        } else {
            pixels.as_chunks_mut::<4>().0.iter_mut().for_each(|p| p[3] = 255);
        }
        Ok(RgbaImage { width: w, height: h, pixels })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiles_cover_the_whole_image_ndc() {
        // Full-image NDC of a tile's corners must map to tile NDC -1 / +1.
        let (w, h, tw, th) = (5000u32, 3000u32, 2048u32, 2048u32);
        for (x0, y0) in [(0, 0), (2048, 0), (4096, 2048)] {
            let t = tile_transform(w, h, x0, y0, tw, th);
            let full_x = |px: f64| 2.0 * px / w as f64 - 1.0;
            let full_y = |py: f64| 1.0 - 2.0 * py / h as f64;
            let tx = |f: f64| f * t[0] as f64 + t[2] as f64;
            let ty = |f: f64| f * t[1] as f64 + t[3] as f64;
            assert!((tx(full_x(x0 as f64)) + 1.0).abs() < 1e-5);
            assert!((tx(full_x((x0 + tw) as f64)) - 1.0).abs() < 1e-5);
            assert!((ty(full_y(y0 as f64)) - 1.0).abs() < 1e-5);
            assert!((ty(full_y((y0 + th) as f64)) + 1.0).abs() < 1e-5);
        }
        assert_eq!(tile_transform(800, 600, 0, 0, 800, 600), [1.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn unpremultiply_restores_straight_colour() {
        // sRGB 200 at 50 % coverage over transparent black, blended in linear space.
        let lin = |x: f32| ((x / 255.0 + 0.055) / 1.055).powf(2.4);
        let srgb = |x: f32| ((1.055 * x.powf(1.0 / 2.4) - 0.055) * 255.0).round() as u8;
        let pre = srgb(lin(200.0) * 128.0 / 255.0);
        let mut px = [pre, pre, pre, 128, 9, 9, 9, 0, 10, 20, 30, 255];
        unpremultiply(&mut px, true);
        assert!(px[..3].iter().all(|&c| (c as i32 - 200).abs() <= 2), "{px:?}");
        assert_eq!(px[3], 128);
        assert_eq!(&px[4..8], &[0, 0, 0, 0]);
        assert_eq!(&px[8..], &[10, 20, 30, 255]);
        let mut gamma = [100, 50, 0, 128];
        unpremultiply(&mut gamma, false);
        assert_eq!(gamma, [199, 100, 0, 128]);
    }

    #[test]
    fn png_roundtrip() {
        let img = RgbaImage { width: 3, height: 2, pixels: (0..24).map(|i| i as u8 * 10).collect() };
        let bytes = img.encode_png(Some(144.0)).unwrap();
        let dec = png::Decoder::new(std::io::Cursor::new(bytes));
        let mut r = dec.read_info().unwrap();
        let mut buf = vec![0; r.output_buffer_size().unwrap()];
        let info = r.next_frame(&mut buf).unwrap();
        assert_eq!((info.width, info.height), (3, 2));
        assert_eq!(&buf[..info.buffer_size()], &img.pixels[..]);
        let dims = r.info().pixel_dims.unwrap();
        assert_eq!(dims.xppu, 5669);
    }

    #[test]
    fn export_error_messages() {
        let e = ExportError::TooLarge { requested: [20000, 100], max: MAX_EXPORT_DIM };
        assert!(e.to_string().contains("20000 x 100"));
        assert_eq!(ExportOptions::default().line_scale, 1.0);
        assert!(!ExportOptions::default().show_selection);
    }
}
