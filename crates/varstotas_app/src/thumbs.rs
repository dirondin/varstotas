//! Rendered thumbnails of components without a preview picture (the Components card): a
//! second [`Renderer`] draws each model once — Shaded with Textures, iso view, transparent
//! background — a few per frame. Library files keep theirs in `<data folder>/cache/thumbs`
//! (named after the file's path and modification time), so they are rendered once.

use crate::document::guarded;
use varstotas_kernel::Model;
use varstotas_render::{Camera, ExportOptions, FaceStyle, Overlay3d, RenderStyle, Renderer};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

/// Thumbnail side in pixels (tiles are 78 points: sharp at 2× scale).
pub const SIDE: u32 = 160;

/// Thumbnails rendered per frame at most (each takes a few milliseconds).
const PER_FRAME: usize = 2;

struct Job {
    key: String,
    model: Model,
    /// PNG cache file to write.
    disk: Option<PathBuf>,
}

#[derive(Default)]
pub struct Thumbs {
    renderer: Option<Renderer>,
    /// The renderer could not be created: tiles keep their drawn edges.
    failed: bool,
    queue: VecDeque<Job>,
    queued: HashSet<String>,
    done: HashMap<String, egui::TextureHandle>,
    /// Thumbnails rendered (harness).
    pub rendered: usize,
}

/// `<data folder>/cache/thumbs`.
pub fn cache_dir() -> Option<PathBuf> {
    crate::paths::data_dir().map(|d| d.join("cache").join("thumbs"))
}

/// Cache file of the thumbnail of library file `path` (changes with the file).
pub fn disk_path(path: &Path) -> Option<PathBuf> {
    let mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok()?;
    let stamp = mtime.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in format!("{}|{stamp}|{SIDE}", path.display()).bytes() {
        h = (h ^ b as u64).wrapping_mul(0x100_0000_01b3);
    }
    Some(cache_dir()?.join(format!("{h:016x}.png")))
}

impl Thumbs {
    /// The thumbnail `key` when it is ready; else it is loaded from `disk` or queued for
    /// rendering the model `make` returns (called only then). None until it's there (or when
    /// thumbnails can't be rendered).
    pub fn get(
        &mut self,
        ctx: &egui::Context,
        key: &str,
        disk: impl FnOnce() -> Option<PathBuf>,
        make: impl FnOnce() -> Option<Model>,
    ) -> Option<egui::TextureId> {
        if let Some(t) = self.done.get(key) {
            return Some(t.id());
        }
        if self.failed || self.queued.contains(key) {
            return None;
        }
        let disk = disk();
        if let Some(img) = disk.as_ref().and_then(|p| std::fs::read(p).ok()).and_then(|b| crate::textures::thumbnail(&b, SIDE)) {
            let t = ctx.load_texture(format!("thumb:{key}"), img, egui::TextureOptions::LINEAR);
            let id = t.id();
            self.done.insert(key.to_string(), t);
            return Some(id);
        }
        let model = make()?;
        self.queued.insert(key.to_string());
        self.queue.push_back(Job { key: key.to_string(), model, disk });
        ctx.request_repaint();
        None
    }

    /// Drop thumbnail `key` (a newer one replaced it).
    pub fn remove(&mut self, key: &str) {
        self.done.remove(key);
    }

    /// Render a few queued thumbnails (called once per frame).
    pub fn process(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        if self.queue.is_empty() {
            return;
        }
        let Some(rs) = frame.wgpu_render_state() else { return };
        if self.renderer.is_none() && !self.failed {
            match guarded(|| Renderer::new(rs)) {
                Ok(r) => self.renderer = Some(r),
                Err(e) => {
                    log::warn!("thumbnail renderer unavailable: {e}");
                    self.failed = true;
                    self.queue.clear();
                    self.queued.clear();
                    return;
                }
            }
        }
        for _ in 0..PER_FRAME {
            let Some(job) = self.queue.pop_front() else { break };
            self.queued.remove(&job.key);
            let renderer = self.renderer.as_mut().expect("renderer");
            match guarded(|| render(renderer, rs, &job.model)).and_then(|r| r) {
                Ok(img) => {
                    if let Some(p) = &job.disk {
                        let _ = std::fs::create_dir_all(p.parent().unwrap_or(Path::new(".")));
                        if let Err(e) = img.write_png(p) {
                            log::warn!("thumbnail {}: {e}", p.display());
                        }
                    }
                    let ci = egui::ColorImage::from_rgba_unmultiplied([img.width as usize, img.height as usize], &img.pixels);
                    let t = ctx.load_texture(format!("thumb:{}", job.key), ci, egui::TextureOptions::LINEAR);
                    self.done.insert(job.key, t);
                    self.rendered += 1;
                }
                Err(e) => log::warn!("thumbnail {}: {e}", job.key),
            }
        }
        ctx.request_repaint();
    }
}

/// `model` drawn Shaded with Textures from the iso direction, fitted, on a transparent
/// background.
fn render(renderer: &mut Renderer, rs: &egui_wgpu::RenderState, model: &Model) -> Result<varstotas_render::RgbaImage, String> {
    let b = model.bounds().ok_or("empty model")?; // i18n-ignore
    let tess = crate::scene_builder::tessellate_model(model);
    let params = crate::scene_builder::BuildParams { show_triangles: false, clean_xray: false, face_style: FaceStyle::Shaded, textures: true, prev_origin: None };
    let geo = crate::scene_builder::build_from(model, &tess, &params);
    let mut sync = crate::textures::TextureSync::default();
    if let Some(images) = sync.changed(model, false) {
        renderer.set_textures(rs, &images);
    }
    renderer.set_geometry(rs, &geo);
    let mut camera = Camera::default();
    camera.zoom_extents(b.min, b.max, 1.0);
    let style = RenderStyle { face_style: FaceStyle::Shaded, show_axes: false, grid: None, shadows: None, ..RenderStyle::default() };
    let opts = ExportOptions { transparent_background: true, line_scale: 1.0, show_selection: false, show_context_dimming: false };
    renderer.render_to_image(rs, &camera, &Overlay3d::default(), &style, [SIDE, SIDE], &opts).map_err(|e| e.to_string())
}
