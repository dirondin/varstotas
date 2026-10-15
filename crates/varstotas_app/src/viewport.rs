//! The 3D view widget: renders through `varstotas_render::Renderer` into a texture shown as an
//! image, paints 2D overlays on top. GPU geometry is rebuilt only when the model or face style
//! changes; selection / editing-context changes only push highlight states
//! (`Renderer::update_highlight`) computed from the cached tessellation.

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
}

/// Inputs the uploaded highlight states depend on.
#[derive(Clone, PartialEq, Eq, Debug)]
struct HighlightKey {
    selection: u64,
    active_path: ContextPath,
}

pub struct Viewport {
    renderer: Option<Renderer>,
    renderer_error: Option<String>,
    built: Option<GeometryKey>,
    highlighted: Option<HighlightKey>,
    /// Tessellation of the uploaded geometry (highlight ids index into it).
    tess: Tessellation,
    pub nav: NavController,
    pub rect: Rect,
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
    /// Material textures uploaded to the renderer.
    textures: crate::textures::TextureSync,
    /// Render origin of the uploaded geometry (kept stable between rebuilds).
    origin: Option<varstotas_kernel::glam::DVec3>,
    /// PNG export to run at the next paint (needs the GPU renderer).
    pub export: Option<crate::app::ExportJob>,
    /// Outcome of the last export (path written or error), taken by the app.
    pub export_result: Option<Result<std::path::PathBuf, String>>,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            renderer: None,
            renderer_error: None,
            built: None,
            highlighted: None,
            tess: Tessellation::default(),
            nav: NavController::default(),
            rect: Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0)),
            tri_count: 0,
            rebuild_count: 0,
            last_rebuild_ms: 0.0,
            last_rebuild_parts: [0.0; 3],
            highlight_count: 0,
            last_highlight_ms: 0.0,
            textures: Default::default(),
            origin: None,
            export: None,
            export_result: None,
        }
    }
}

impl Viewport {
    /// Allocate the remaining space for the 3D view and return its interaction response.
    pub fn allocate(&mut self, ui: &mut Ui) -> Response {
        let rect = ui.available_rect_before_wrap();
        self.rect = rect;
        ui.allocate_rect(rect, Sense::click_and_drag())
    }

    pub fn size(&self) -> egui::Vec2 {
        self.rect.size()
    }

    /// Force a geometry rebuild on the next paint (e.g. after loading a file).
    pub fn invalidate(&mut self) {
        self.built = None;
        self.highlighted = None;
        self.origin = None;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &mut self,
        ui: &Ui,
        frame: &eframe::Frame,
        doc: &Document,
        camera: &Camera,
        style: &RenderStyle,
        overlay3d: &Overlay3d,
        markers: &[Marker2d],
        cursor: Option<Pos2>,
    ) {
        let rect = self.rect;
        let painter = ui.painter_at(rect);
        let ppp = ui.ctx().pixels_per_point();

        let mut drew = false;
        if let Some(rs) = frame.wgpu_render_state() {
            if self.renderer.is_none() && self.renderer_error.is_none() {
                match guarded(|| Renderer::new(rs)) {
                    Ok(r) => self.renderer = Some(r),
                    Err(e) => {
                        log::warn!("renderer unavailable: {e}");
                        self.renderer_error = Some(e);
                    }
                }
            }
            if let Some(renderer) = self.renderer.as_mut() {
                let key = GeometryKey { model: doc.model_version, face_style: style.face_style };
                if self.built != Some(key) {
                    let t0 = std::time::Instant::now();
                    let old = std::mem::replace(&mut self.tess, scene_builder::tessellate_model(&doc.model));
                    // Freeing 100k+ tessellated faces takes milliseconds: do it off the UI thread.
                    if old.faces.len() + old.edges.len() > 10_000 {
                        std::thread::spawn(move || drop(old));
                    }
                    let t1 = std::time::Instant::now();
                    let params = BuildParams { face_style: style.face_style, prev_origin: self.origin };
                    let geo = guarded(|| scene_builder::build_from(&doc.model, &self.tess, &params))
                        .unwrap_or_else(|e| {
                            log::warn!("scene build failed: {e}");
                            self.tess = Tessellation::default();
                            varstotas_render::SceneGeometry { radius: 10_000.0, ..Default::default() }
                        });
                    self.tri_count = geo.tris.len() / 3;
                    self.origin = Some(varstotas_kernel::glam::DVec3::from_array(geo.origin));
                    self.rebuild_count += 1;
                    let t2 = std::time::Instant::now();
                    if let Some(images) = self.textures.changed(&doc.model)
                        && let Err(e) = guarded(|| renderer.set_textures(rs, &images))
                    {
                        log::warn!("set_textures failed: {e}");
                    }
                    if let Err(e) = guarded(|| renderer.set_geometry(rs, &geo)) {
                        log::warn!("set_geometry failed: {e}");
                    }
                    let ms = |a: std::time::Instant, b: std::time::Instant| (b - a).as_secs_f64() * 1e3;
                    self.last_rebuild_parts = [ms(t0, t1), ms(t1, t2), ms(t2, std::time::Instant::now())];
                    self.built = Some(key);
                    // set_geometry resets all highlight states.
                    self.highlighted = None;
                    self.last_rebuild_ms = t0.elapsed().as_secs_f64() * 1e3;
                }
                let hkey = HighlightKey { selection: doc.selection_version, active_path: doc.active_path.clone() };
                if self.highlighted.as_ref() != Some(&hkey) {
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
                    self.highlighted = Some(hkey);
                    self.highlight_count += 1;
                    self.last_highlight_ms = t0.elapsed().as_secs_f64() * 1e3;
                }
                // Custom drawing axes (Axes tool) replace the main axes.
                let frame = doc.model.axes.map(|a| varstotas_render::AxesFrame::from_transform(&varstotas_kernel::glam::DMat4::from(a)));
                renderer.set_axes_frame(frame);
                let size_px = [
                    ((rect.width() * ppp).round() as u32).max(1),
                    ((rect.height() * ppp).round() as u32).max(1),
                ];
                if let Some(job) = self.export.take() {
                    let opts = ExportOptions {
                        transparent_background: job.transparent,
                        line_scale: job.line_scale,
                        show_selection: false,
                        show_context_dimming: true,
                    };
                    let style = RenderStyle { show_axes: style.show_axes && job.axes, ..style.clone() };
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
                        self.renderer = None;
                        self.renderer_error = Some(e);
                    }
                }
            }
        }
        if !drew {
            if self.export.take().is_some() {
                self.export_result = Some(Err("the 3D renderer is not available".into()));
            }
            paint_fallback_background(&painter, rect);
            let msg = match &self.renderer_error {
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
        if camera.ortho {
            hud_text(
                &painter,
                rect.right_top() + vec2(-10.0, 10.0),
                Align2::RIGHT_TOP,
                "Parallel Projection",
                Color32::from_gray(70),
            );
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
