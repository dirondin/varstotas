//! The 3D view widget: renders through `varstotas_render::Renderer` into a texture shown as an
//! image, paints 2D overlays on top. GPU geometry is rebuilt only when the model or face style
//! changes; selection / editing-context changes only push highlight states
//! (`Renderer::update_highlight`) computed from the cached tessellation.
//!
//! A split view (1 / 2 / 4 panes) has a renderer per pane (`Gpu`: its own geometry for the
//! pane's face style, its own image); the tessellation is shared. The path traced render lives
//! in pane 0's renderer, a PNG export runs in the active pane's.

use crate::document::{Document, guarded};
use crate::nav::NavController;
use crate::overlay::{Marker2d, OverlayPainter, hud_text};
use crate::scene_builder::{self, BuildParams, HighlightParams};
use egui::{Align2, Color32, Pos2, Rect, Response, Sense, Shape, Ui, pos2, vec2};
use varstotas_kernel::{ContextPath, Tessellation};
use varstotas_render::{Camera, ExportOptions, FaceStyle, HighlightUpdate, Overlay3d, RenderStyle, Renderer};

/// Inputs the uploaded geometry depends on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct GeometryKey {
    model: u64,
    face_style: FaceStyle,
    textures: bool,
}

/// Inputs the uploaded highlight states depend on.
#[derive(Clone, PartialEq, Eq, Debug)]
struct HighlightKey {
    selection: u64,
    active_path: ContextPath,
}

/// The GPU side of one pane.
#[derive(Default)]
struct Gpu {
    renderer: Option<Renderer>,
    renderer_error: Option<String>,
    built: Option<GeometryKey>,
    highlighted: Option<HighlightKey>,
    /// Material textures uploaded to the renderer.
    textures: crate::textures::TextureSync,
    /// Data textures and the HDRI of the Realistic style uploaded to the renderer.
    data_textures: crate::pbr::DataSync,
    hdri: crate::pbr::EnvSync,
    /// Render origin of the uploaded geometry (kept stable between rebuilds).
    origin: Option<varstotas_kernel::glam::DVec3>,
}

pub struct Viewport {
    /// One per pane (made as panes are first painted).
    gpus: Vec<Gpu>,
    /// Tessellation of the model (highlight ids index into it) and the model version it is of.
    tess: Tessellation,
    tess_version: Option<u64>,
    pub nav: NavController,
    /// The active pane.
    pub rect: Rect,
    /// The whole view area (all panes): where windows open, the harness's raw origin.
    pub view_rect: Rect,
    /// Geometry stats for the HUD.
    pub tri_count: usize,
    /// Number of scene geometry rebuilds so far (perf diagnostics).
    pub rebuild_count: u64,
    /// CPU time of the last rebuild (tessellation + upload), milliseconds.
    pub last_rebuild_ms: f64,
    /// Breakdown of the last rebuild: (tessellate, build, upload) milliseconds.
    pub last_rebuild_parts: [f64; 3],
    /// Number of highlight-only updates (selection / context changes).
    pub highlight_count: u64,
    /// CPU time of the last highlight update, milliseconds.
    pub last_highlight_ms: f64,
    /// Draw material textures (set by the app from the face style before `paint`).
    pub show_textures: bool,
    /// Custom surface shaders that do not compile: (source, message).
    pub shader_errors: Vec<(String, String)>,
    /// PNG export to run at the next paint (needs the GPU renderer).
    pub export: Option<crate::app::ExportJob>,
    /// Outcome of the last export (path written or error), taken by the app.
    pub export_result: Option<Result<std::path::PathBuf, String>>,
    /// The path traced render of the Render window.
    pub pt: Option<PtJob>,
}

/// A path traced render (Window → Render…), advanced by every `paint` while it runs.
pub struct PtJob {
    pub camera: Camera,
    /// A Realistic style (environment, materials, effects).
    pub style: RenderStyle,
    pub lights: Vec<varstotas_render::PtLight>,
    pub settings: varstotas_render::PtSettings,
    pub max_samples: u32,
    /// Seconds (0 = no limit).
    pub time_limit: f64,
    pub denoise: bool,
    /// Set up the buffers (again) at the next paint.
    pub restart: bool,
    pub running: bool,
    pub samples: u32,
    /// Seconds rendered so far.
    pub elapsed: f64,
    /// When the render (re)started: wall-clock time for the log.
    started: std::time::Instant,
    /// The texture shows the denoised image.
    pub denoised: bool,
    /// The image settings the texture was resolved with (`pbr::pt_output`).
    output: Option<varstotas_render::PtOutput>,
    /// Save the image at the next paint: `.exr` = linear float, else PNG.
    pub save: Option<std::path::PathBuf>,
    pub saved: Option<Result<std::path::PathBuf, String>>,
}

impl PtJob {
    pub fn new(camera: Camera, style: RenderStyle, lights: Vec<varstotas_render::PtLight>, settings: varstotas_render::PtSettings) -> Self {
        Self {
            camera,
            style,
            lights,
            settings,
            max_samples: 256,
            time_limit: 0.0,
            denoise: true,
            restart: true,
            running: true,
            samples: 0,
            elapsed: 0.0,
            started: std::time::Instant::now(),
            denoised: false,
            output: None,
            save: None,
            saved: None,
        }
    }

    /// Finished: enough samples or time.
    pub fn done(&self) -> bool {
        self.samples >= self.max_samples || (self.time_limit > 0.0 && self.elapsed >= self.time_limit)
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            gpus: Vec::new(),
            tess: Tessellation::default(),
            tess_version: None,
            nav: NavController::default(),
            rect: Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0)),
            view_rect: Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0)),
            tri_count: 0,
            rebuild_count: 0,
            last_rebuild_ms: 0.0,
            last_rebuild_parts: [0.0; 3],
            highlight_count: 0,
            last_highlight_ms: 0.0,
            shader_errors: Vec::new(),
            show_textures: true,
            export: None,
            export_result: None,
            pt: None,
        }
    }
}

impl Viewport {
    /// Allocate `rect` for the active pane and return its interaction response.
    pub fn allocate(&mut self, ui: &mut Ui, rect: Rect) -> Response {
        self.rect = rect;
        ui.allocate_rect(rect, Sense::click_and_drag())
    }

    pub fn size(&self) -> egui::Vec2 {
        self.rect.size()
    }

    /// The path traced image and its size.
    pub fn pt_texture(&self) -> Option<(egui::TextureId, [u32; 2])> {
        self.gpus.first()?.renderer.as_ref()?.pt_texture()
    }

    /// Force a geometry rebuild on the next paint (e.g. after loading a file).
    pub fn invalidate(&mut self) {
        self.tess_version = None;
        for g in &mut self.gpus {
            g.built = None;
            g.highlighted = None;
            g.origin = None;
        }
    }

    /// Draw pane `pane` into `rect`; the `active` one also runs a pending PNG export.
    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &mut self,
        ui: &Ui,
        frame: &eframe::Frame,
        pane: usize,
        rect: Rect,
        active: bool,
        doc: &Document,
        camera: &Camera,
        style: &RenderStyle,
        overlay3d: &Overlay3d,
        markers: &[Marker2d],
        cursor: Option<Pos2>,
    ) {
        let painter = ui.painter_at(rect);
        let ppp = ui.ctx().pixels_per_point();
        while self.gpus.len() <= pane {
            self.gpus.push(Gpu::default());
        }

        let mut drew = false;
        if let Some(rs) = frame.wgpu_render_state() {
            let g = &mut self.gpus[pane];
            if g.renderer.is_none() && g.renderer_error.is_none() {
                match guarded(|| Renderer::new(rs)) {
                    Ok(r) => g.renderer = Some(r),
                    Err(e) => {
                        log::warn!("renderer unavailable: {e}");
                        g.renderer_error = Some(e);
                    }
                }
            }
            if let Some(renderer) = g.renderer.as_mut() {
                let key = GeometryKey { model: doc.model_version, face_style: style.face_style, textures: self.show_textures };
                if g.built != Some(key) {
                    let t0 = std::time::Instant::now();
                    if self.tess_version != Some(doc.model_version) {
                        let old = std::mem::replace(&mut self.tess, scene_builder::tessellate_model(&doc.model));
                        self.tess_version = Some(doc.model_version);
                        // Freeing 100k+ tessellated faces takes milliseconds: do it off the UI thread.
                        if old.faces.len() + old.edges.len() > 10_000 {
                            std::thread::spawn(move || drop(old));
                        }
                    }
                    let t1 = std::time::Instant::now();
                    let params = BuildParams { face_style: style.face_style, textures: self.show_textures, prev_origin: g.origin };
                    let geo = guarded(|| scene_builder::build_from(&doc.model, &self.tess, &params))
                        .unwrap_or_else(|e| {
                            log::warn!("scene build failed: {e}");
                            self.tess = Tessellation::default();
                            varstotas_render::SceneGeometry { radius: 10_000.0, ..Default::default() }
                        });
                    self.tri_count = geo.tris.len() / 3;
                    g.origin = Some(varstotas_kernel::glam::DVec3::from_array(geo.origin));
                    self.rebuild_count += 1;
                    let t2 = std::time::Instant::now();
                    if let Some(images) = g.textures.changed(&doc.model)
                        && let Err(e) = guarded(|| renderer.set_textures(rs, &images))
                    {
                        log::warn!("set_textures failed: {e}");
                    }
                    if style.face_style == FaceStyle::Realistic {
                        if let Some(images) = g.data_textures.changed(&doc.model)
                            && let Err(e) = guarded(|| renderer.set_data_textures(rs, &images))
                        {
                            log::warn!("set_data_textures failed: {e}");
                        }
                        let customs = crate::pbr::custom_surfaces(&doc.model);
                        match guarded(|| renderer.set_custom_shaders(rs, &customs)) {
                            Ok(errs) => {
                                self.shader_errors = customs
                                    .into_iter()
                                    .zip(errs)
                                    .filter_map(|(c, e)| Some((c.source, e?)))
                                    .collect();
                            }
                            Err(e) => log::warn!("set_custom_shaders failed: {e}"),
                        }
                        let table = crate::pbr::material_table(&doc.model);
                        if let Err(e) = guarded(|| renderer.set_materials(rs, &table)) {
                            log::warn!("set_materials failed: {e}");
                        }
                    }
                    if let Err(e) = guarded(|| renderer.set_geometry(rs, &geo)) {
                        log::warn!("set_geometry failed: {e}");
                    }
                    let ms = |a: std::time::Instant, b: std::time::Instant| (b - a).as_secs_f64() * 1e3;
                    self.last_rebuild_parts = [ms(t0, t1), ms(t1, t2), ms(t2, std::time::Instant::now())];
                    g.built = Some(key);
                    // set_geometry resets all highlight states.
                    g.highlighted = None;
                    self.last_rebuild_ms = t0.elapsed().as_secs_f64() * 1e3;
                }
                if style.face_style == FaceStyle::Realistic
                    && let Some(img) = g.hdri.changed(&doc.model)
                {
                    renderer.set_environment(img);
                }
                let hkey = HighlightKey { selection: doc.selection_version, active_path: doc.active_path.clone() };
                if g.highlighted.as_ref() != Some(&hkey) {
                    let t0 = std::time::Instant::now();
                    let (faces, edges) = scene_builder::highlight(
                        &self.tess,
                        &HighlightParams { selection: &doc.selection, active_path: &doc.active_path },
                    );
                    if let Err(e) =
                        guarded(|| renderer.update_highlight(rs, &HighlightUpdate { faces: &faces, edges: &edges }))
                    {
                        log::warn!("update_highlight failed: {e}");
                    }
                    g.highlighted = Some(hkey);
                    self.highlight_count += 1;
                    self.last_highlight_ms = t0.elapsed().as_secs_f64() * 1e3;
                }
                // Custom drawing axes (Axes tool) or the active work plane replace the main axes.
                let custom = doc.model.axes.is_some() || doc.model.active_work_plane().is_some();
                let frame = custom.then(|| doc.model.drawing_frame()).map(|a| varstotas_render::AxesFrame::from_transform(&varstotas_kernel::glam::DMat4::from(a)));
                renderer.set_axes_frame(frame);
                renderer.set_section(doc.model.active_section().map(|s| s.plane()));
                let size_px = [
                    ((rect.width() * ppp).round() as u32).max(1),
                    ((rect.height() * ppp).round() as u32).max(1),
                ];
                if let Some(job) = self.pt.as_mut().filter(|_| pane == 0) {
                    if job.restart {
                        job.restart = false;
                        // The Realistic resources may be stale when the view shows another style.
                        if let Some(images) = g.textures.changed(&doc.model) {
                            let _ = guarded(|| renderer.set_textures(rs, &images));
                        }
                        if let Some(images) = g.data_textures.changed(&doc.model) {
                            let _ = guarded(|| renderer.set_data_textures(rs, &images));
                        }
                        if let Some(img) = g.hdri.changed(&doc.model) {
                            renderer.set_environment(img);
                        }
                        let customs = crate::pbr::custom_surfaces(&doc.model);
                        let _ = guarded(|| renderer.set_custom_shaders(rs, &customs));
                        let table = crate::pbr::material_table(&doc.model);
                        let _ = guarded(|| renderer.set_materials(rs, &table));
                        let params = BuildParams { face_style: FaceStyle::Realistic, textures: true, prev_origin: g.origin };
                        let r = guarded(|| scene_builder::build_from(&doc.model, &self.tess, &params)).and_then(|geo| {
                            guarded(|| renderer.pt_start(rs, &geo, &job.lights, &job.camera, &job.style, &job.settings))
                        });
                        if let Err(e) = r {
                            log::warn!("path tracer: {e}");
                            job.running = false;
                            job.saved = Some(Err(e));
                        }
                        job.samples = 0;
                        job.elapsed = 0.0;
                        job.started = std::time::Instant::now();
                        job.denoised = false;
                        job.output = None;
                    }
                    // Exposure, tone curve, grading: live, without new samples.
                    let out = crate::pbr::pt_output(&doc.model);
                    if job.output != Some(out) {
                        let first = job.output.is_none();
                        renderer.pt_set_output(out);
                        job.output = Some(out);
                        if !first && !job.running && job.samples > 0 {
                            let _ = guarded(|| renderer.pt_resolve(rs, job.denoised));
                        }
                    }
                    if job.running && !job.done() {
                        let t0 = std::time::Instant::now();
                        let budget = std::time::Duration::from_millis(40);
                        job.samples = guarded(|| renderer.pt_step(rs, &job.style, job.max_samples, budget)).unwrap_or(job.max_samples);
                        job.elapsed += t0.elapsed().as_secs_f64();
                        let finished = job.done();
                        let _ = guarded(|| renderer.pt_resolve(rs, finished && job.denoise));
                        job.denoised = finished && job.denoise;
                        if finished {
                            job.running = false;
                            let [w, h] = [job.settings.width, job.settings.height];
                            log::info!(
                                "path tracer: {} spp {w}x{h} in {:.2} s ({:.2} s wall)",
                                job.samples,
                                job.elapsed,
                                job.started.elapsed().as_secs_f64()
                            );
                        }
                        ui.ctx().request_repaint();
                    } else if job.running {
                        job.running = false;
                    }
                    if !job.running && job.denoise != job.denoised && job.samples > 0 {
                        // Stopped early, or the denoise switch changed.
                        let _ = guarded(|| renderer.pt_resolve(rs, job.denoise));
                        job.denoised = job.denoise;
                    }
                    if let Some(path) = job.save.take() {
                        let exr = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("exr"));
                        let r = if exr {
                            renderer
                                .pt_linear(rs, job.denoised)
                                .ok_or_else(|| "nothing rendered".to_string())
                                .and_then(|(w, h, rgb)| std::fs::write(&path, varstotas_render::encode_exr(w, h, &rgb)).map_err(|e| e.to_string()))
                        } else {
                            renderer
                                .pt_image(rs)
                                .ok_or_else(|| "nothing rendered".to_string())
                                .and_then(|img| img.write_png(&path).map_err(|e| e.to_string()))
                        };
                        job.saved = Some(r.map(|_| path));
                    }
                }
                if let Some(job) = self.export.take_if(|_| active) {
                    let opts = ExportOptions {
                        transparent_background: job.transparent,
                        line_scale: job.line_scale,
                        show_selection: false,
                        show_context_dimming: true,
                    };
                    let style = RenderStyle { show_axes: style.show_axes && job.axes, grid: None, ..style.clone() };
                    let r = guarded(|| renderer.render_to_image(rs, &job.camera, &job.overlay, &style, job.size, &opts))
                        .and_then(|r| r.map_err(|e| e.to_string()))
                        .and_then(|img| img.write_png(&job.path).map_err(|e| e.to_string()))
                        .map(|_| job.path.clone());
                    self.export_result = Some(r);
                }
                match guarded(|| renderer.render(rs, camera, overlay3d, style, size_px, ppp)) {
                    Ok(tex) => {
                        painter.image(tex, rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                        drew = true;
                    }
                    Err(e) => {
                        log::warn!("render failed: {e}");
                        g.renderer = None;
                        g.renderer_error = Some(e);
                    }
                }
            }
        }
        if !drew {
            if self.export.take_if(|_| active).is_some() {
                self.export_result = Some(Err("the 3D renderer is not available".into()));
            }
            paint_fallback_background(&painter, rect);
            let msg = match self.gpus.get(pane).and_then(|g| g.renderer_error.as_ref()) {
                Some(e) => format!("3D renderer not available yet ({e})"),
                None => "3D renderer not available (wgpu backend required)".to_string(),
            };
            hud_text(&painter, rect.center(), Align2::CENTER_CENTER, &msg, Color32::from_gray(90));
        }

        // 2D overlays
        OverlayPainter { painter: &painter, rect, camera, cursor }.paint(markers);

        // Context breadcrumb HUD
        if !doc.active_path.is_empty() {
            let names: Vec<String> = guarded(|| {
                let mut out = Vec::new();
                for i in 0..doc.active_path.len() {
                    let parent = &doc.active_path[..i];
                    let id = doc.active_path[i];
                    let name = doc
                        .model
                        .mesh_at(parent)
                        .and_then(|m| m.instances.get(id))
                        .map(|inst| {
                            if !inst.name.is_empty() {
                                inst.name.clone()
                            } else {
                                doc.model.defs.get(inst.def).map(|d| d.name.clone()).unwrap_or_default()
                            }
                        })
                        .unwrap_or_else(|| "Group".into());
                    out.push(if name.is_empty() { "Group".into() } else { name });
                }
                out
            })
            .unwrap_or_default();
            let text = format!("Editing: Model › {}   (Esc to close)", names.join(" › "));
            let galley = painter.layout_no_wrap(text, egui::FontId::proportional(12.5), Color32::from_gray(40));
            let r = Rect::from_min_size(rect.min + vec2(10.0, 10.0), galley.size() + vec2(16.0, 8.0));
            painter.rect_filled(r, 6.0, Color32::from_rgba_unmultiplied(255, 255, 255, 210));
            painter.galley(r.min + vec2(8.0, 4.0), galley, Color32::BLACK);
        }
    }
}

/// Sky/ground gradient used when the GPU renderer is unavailable.
fn paint_fallback_background(painter: &egui::Painter, rect: Rect) {
    let top = Color32::from_rgb(0xA8, 0xC8, 0xE8);
    let horizon = Color32::from_rgb(0xE8, 0xF0, 0xF8);
    let ground = Color32::from_rgb(0xC8, 0xC0, 0xB0);
    let hy = rect.top() + rect.height() * 0.6;
    let mut mesh = egui::Mesh::default();
    let quad = |mesh: &mut egui::Mesh, r: Rect, c0: Color32, c1: Color32| {
        let i = mesh.vertices.len() as u32;
        mesh.colored_vertex(r.left_top(), c0);
        mesh.colored_vertex(r.right_top(), c0);
        mesh.colored_vertex(r.right_bottom(), c1);
        mesh.colored_vertex(r.left_bottom(), c1);
        mesh.add_triangle(i, i + 1, i + 2);
        mesh.add_triangle(i, i + 2, i + 3);
    };
    quad(&mut mesh, Rect::from_min_max(rect.min, pos2(rect.right(), hy)), top, horizon);
    quad(&mut mesh, Rect::from_min_max(pos2(rect.left(), hy), rect.max), ground, ground);
    painter.add(Shape::mesh(mesh));
}
