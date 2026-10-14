//! The 3D view widget: renders through `varstotas_render::Renderer` into a texture shown as an
//! image, rebuilds GPU geometry when the model/selection changes, paints 2D overlays on top.

use crate::document::{Document, guarded};
use crate::nav::NavController;
use crate::overlay::{Marker2d, OverlayPainter, hud_text};
use crate::scene_builder::{self, BuildParams};
use egui::{Align2, Color32, Pos2, Rect, Response, Sense, Shape, Ui, pos2, vec2};
use varstotas_render::{Camera, FaceStyle, Overlay3d, RenderStyle, Renderer};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct BuiltKey {
    model: u64,
    selection: u64,
    face_style: FaceStyle,
}

pub struct Viewport {
    renderer: Option<Renderer>,
    renderer_error: Option<String>,
    built: Option<BuiltKey>,
    pub nav: NavController,
    pub rect: Rect,
    /// Geometry stats for the HUD.
    pub tri_count: usize,
    /// Number of scene geometry rebuilds so far (perf diagnostics).
    pub rebuild_count: u64,
    /// CPU time of the last rebuild (tessellation + upload), milliseconds.
    pub last_rebuild_ms: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            renderer: None,
            renderer_error: None,
            built: None,
            nav: NavController::default(),
            rect: Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0)),
            tri_count: 0,
            rebuild_count: 0,
            last_rebuild_ms: 0.0,
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
                let key = BuiltKey {
                    model: doc.model_version,
                    selection: doc.selection_version,
                    face_style: style.face_style,
                };
                if self.built != Some(key) {
                    let t0 = std::time::Instant::now();
                    let geo = scene_builder::build(
                        &doc.model,
                        &BuildParams {
                            selection: &doc.selection,
                            active_path: &doc.active_path,
                            face_style: style.face_style,
                        },
                    );
                    self.tri_count = geo.tris.len() / 3;
                    self.rebuild_count += 1;
                    if let Err(e) = guarded(|| renderer.set_geometry(rs, &geo)) {
                        log::warn!("set_geometry failed: {e}");
                    }
                    self.built = Some(key);
                    self.last_rebuild_ms = t0.elapsed().as_secs_f64() * 1e3;
                }
                let size_px = [
                    ((rect.width() * ppp).round() as u32).max(1),
                    ((rect.height() * ppp).round() as u32).max(1),
                ];
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
