//! Window → Render…: path traced images of the current view or of every scene (the GPU path
//! tracer of `varstotas_render`, driven by `Viewport::pt`), saved as PNG or EXR; the model's
//! lights and render settings. Edits of lights and settings record one undo step when the
//! pointer is released and no field has focus.

use crate::app::VarstotasApp;
use crate::viewport::PtJob;
use egui::{Context, RichText, vec2};
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{Light, LightKind, RenderSettings};
use varstotas_render::{Camera, FaceStyle, PtSettings};
use std::path::PathBuf;

type RenderState = (Vec<Light>, RenderSettings);

#[derive(Default)]
pub struct RenderWindow {
    pub open: bool,
    pub selected: Option<usize>,
    /// Model state before the edit in progress.
    orig: Option<RenderState>,
    /// Rendering every scene into a folder.
    pub batch: Option<Batch>,
}

pub struct Batch {
    pub folder: PathBuf,
    /// The next scene to render.
    pub next: usize,
    /// The file the current render goes to.
    pub current: Option<PathBuf>,
    pub written: Vec<PathBuf>,
}

fn len_drag(v: &mut f64, unit: varstotas_kernel::LengthUnit) -> egui::DragValue<'_> {
    egui::DragValue::new(v)
        .speed(5.0)
        .custom_formatter(move |v, _| varstotas_kernel::units::format_length(v, unit))
        .custom_parser(move |s| varstotas_kernel::units::parse_length(s, unit))
}

/// A file name for a scene (no path separators or odd characters).
pub fn file_stem(name: &str) -> String {
    let s: String = name.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' }).collect();
    let s = s.trim().to_string();
    if s.is_empty() { "scene".into() } else { s }
}

impl VarstotasApp {
    /// A path traced render of `camera` with the model's environment, lights and settings.
    pub(crate) fn start_render(&mut self, camera: Camera) {
        let m = &self.doc.model;
        let rs = m.render;
        let (w, h) = if rs.match_view {
            ((rs.height as f64 * self.aspect()).round().max(8.0) as u32, rs.height)
        } else {
            (rs.width, rs.height)
        };
        let mut style = self.render_style();
        style.face_style = FaceStyle::Realistic;
        let settings = PtSettings { width: w, height: h, bounces: rs.bounces, clamp: 20.0 };
        let mut job = PtJob::new(camera, style, crate::pbr::pt_lights(m), settings);
        job.max_samples = rs.samples.max(1);
        job.time_limit = rs.time_limit as f64;
        job.denoise = rs.denoise;
        self.viewport.pt = Some(job);
    }

    /// Start rendering every scene into `folder` (one PNG per scene, named after it).
    pub(crate) fn render_all_scenes(&mut self, folder: PathBuf) {
        self.render_window.batch = Some(Batch { folder, next: 0, current: None, written: Vec::new() });
    }

    /// Advance the scene batch: save a finished render, then go to the next scene.
    fn batch_tick(&mut self) {
        let Some(mut b) = self.render_window.batch.take() else { return };
        if let Some(path) = b.current.clone() {
            let Some(job) = &mut self.viewport.pt else {
                // The render was dropped: stop.
                return;
            };
            if let Some(r) = job.saved.take() {
                match r {
                    Ok(p) => b.written.push(p),
                    Err(e) => {
                        self.flash_error(format!("Render failed: {e}"));
                        return;
                    }
                }
                b.current = None;
            } else if !job.running && !job.restart && job.save.is_none() {
                job.save = Some(path);
            }
        }
        if b.current.is_none() {
            let scenes = &self.doc.model.scenes;
            if b.next >= scenes.len() {
                self.flash(format!("Rendered {} scene(s) to {}", b.written.len(), b.folder.display()));
                return;
            }
            let i = b.next;
            let name = file_stem(&scenes[i].name);
            b.next += 1;
            self.go_to_scene(i);
            self.cam_anim = None;
            let s = self.doc.model.scenes[i].clone();
            self.camera = self.scene_camera(&s);
            self.start_render(self.camera.clone());
            b.current = Some(b.folder.join(format!("{name}.png")));
        }
        self.render_window.batch = Some(b);
    }

    pub(crate) fn render_window(&mut self, ctx: &Context) {
        self.batch_tick();
        if !self.render_window.open {
            self.finish_render_edit(ctx);
            return;
        }
        let m = &self.doc.model;
        let unit = m.unit;
        let (mut lights, mut settings) = (m.lights.clone(), m.render);
        let before = (lights.clone(), settings);
        let mut sel = self.render_window.selected.filter(|i| *i < lights.len());
        let mut open = true;
        let mut start = false;
        let mut save: Option<&'static str> = None;
        let mut all = false;
        let has_scenes = !m.scenes.is_empty();
        let pos = self.viewport.rect.left_top() + vec2(40.0, 30.0);
        let mut win = egui::Window::new("Render")
            .id(egui::Id::new("render_window"))
            .open(&mut open)
            .collapsible(false)
            .default_pos(pos)
            .default_size(vec2(620.0, 640.0));
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        let tex = self.viewport.pt_texture();
        let mut job = self.viewport.pt.as_mut();
        let batch = self.render_window.batch.as_ref().map(|b| (b.next, b.written.len()));
        win.show(ctx, |ui| {
            egui::Grid::new("render_settings").num_columns(4).spacing([8.0, 4.0]).show(ui, |ui| {
                ui.label("Size");
                ui.add_enabled(!settings.match_view, egui::DragValue::new(&mut settings.width).range(16..=8192).suffix(" px"));
                ui.add(egui::DragValue::new(&mut settings.height).range(16..=8192).suffix(" px"));
                ui.checkbox(&mut settings.match_view, "Match view").on_hover_text("The view's proportions at this height");
                ui.end_row();
                ui.label("Samples");
                ui.add(egui::DragValue::new(&mut settings.samples).range(1..=100_000));
                ui.label("Time limit");
                ui.add(egui::DragValue::new(&mut settings.time_limit).range(0.0..=36_000.0).suffix(" s"))
                    .on_hover_text("0 = until the samples are done");
                ui.end_row();
                ui.label("Bounces");
                ui.add(egui::DragValue::new(&mut settings.bounces).range(1..=32));
                let oidn = varstotas_render::oidn_available();
                ui.checkbox(&mut settings.denoise, "Denoise").on_hover_text(if oidn {
                    "Intel Open Image Denoise (AI denoiser) on the finished render"
                } else {
                    "Built-in edge-aware filter; install Open Image Denoise (brew install open-image-denoise) for a cleaner result"
                });
                ui.end_row();
            });
            ui.horizontal(|ui| {
                if ui.button("Render").on_hover_text("Render the current view").clicked() {
                    start = true;
                }
                if let Some(job) = job.as_deref_mut() {
                    if job.running {
                        if ui.button("Stop").clicked() {
                            job.running = false;
                        }
                    } else if job.samples > 0 && !job.done() && ui.button("Continue").clicked() {
                        job.running = true;
                    } else if job.samples > 0 && job.done() && ui.button("More Samples").on_hover_text("Double the samples").clicked() {
                        job.max_samples = job.samples * 2;
                        job.time_limit = 0.0;
                        job.running = true;
                    }
                    job.denoise = settings.denoise;
                    let rendered = job.samples > 0;
                    if ui.add_enabled(rendered, egui::Button::new("Save PNG…")).clicked() {
                        save = Some("png");
                    }
                    if ui.add_enabled(rendered, egui::Button::new("Save EXR…")).on_hover_text("Linear light, 32-bit float").clicked() {
                        save = Some("exr");
                    }
                }
                if ui.add_enabled(has_scenes, egui::Button::new("Render All Scenes…")).clicked() {
                    all = true;
                }
            });
            if let Some((next, written)) = batch {
                ui.label(format!("Rendering scenes: {written} saved, scene {next} in progress"));
            }
            if let Some(job) = job.as_deref() {
                let left = job.max_samples.saturating_sub(job.samples);
                let frac = if job.time_limit > 0.0 {
                    (job.samples as f64 / job.max_samples as f64).max(job.elapsed / job.time_limit)
                } else {
                    job.samples as f64 / job.max_samples as f64
                };
                let status = if job.running {
                    format!("{} / {} samples · {:.1} s", job.samples, job.max_samples, job.elapsed)
                } else if left == 0 || job.done() {
                    format!("Done: {} samples in {:.1} s{}", job.samples, job.elapsed, if job.denoised { ", denoised" } else { "" })
                } else {
                    format!("Stopped at {} samples · {:.1} s", job.samples, job.elapsed)
                };
                ui.add(egui::ProgressBar::new(frac.clamp(0.0, 1.0) as f32).text(status));
                if let Some(Err(e)) = &job.saved {
                    ui.label(RichText::new(e).color(crate::ui::realistic::ERROR_RED));
                }
            }
            if let Some((id, size)) = tex {
                let avail = ui.available_width().max(100.0);
                let k = (avail / size[0] as f32).min(420.0 / size[1] as f32).min(1.0);
                ui.image(egui::load::SizedTexture::new(id, vec2(size[0] as f32 * k, size[1] as f32 * k)));
            }
            egui::CollapsingHeader::new(format!("Lights ({})", lights.len())).id_salt("render_lights").default_open(true).show(ui, |ui| {
                ui.horizontal(|ui| {
                    for k in LightKind::ALL {
                        let label = match k {
                            LightKind::Point => "+ Point",
                            LightKind::Spot => "+ Spot",
                            LightKind::Area => "+ Area",
                        };
                        if ui.button(label).clicked() {
                            let at = self.camera.target + DVec3::Z * 2000.0;
                            lights.push(Light::new(k, at));
                            sel = Some(lights.len() - 1);
                        }
                    }
                });
                let mut remove = None;
                egui::Grid::new("render_light_list").num_columns(4).spacing([8.0, 2.0]).show(ui, |ui| {
                    for (i, l) in lights.iter_mut().enumerate() {
                        ui.checkbox(&mut l.enabled, "").on_hover_text("On");
                        if ui.selectable_label(sel == Some(i), &l.name).clicked() {
                            sel = Some(i);
                        }
                        ui.label(RichText::new(l.kind.key()).small().weak());
                        if ui.small_button("Delete").clicked() {
                            remove = Some(i);
                        }
                        ui.end_row();
                    }
                });
                if let Some(i) = remove {
                    lights.remove(i);
                    sel = None;
                }
                if let Some(l) = sel.and_then(|i| lights.get_mut(i)) {
                    ui.separator();
                    egui::Grid::new("render_light").num_columns(4).spacing([8.0, 4.0]).show(ui, |ui| {
                        ui.label("Name");
                        ui.add_sized([180.0, 20.0], egui::TextEdit::singleline(&mut l.name));
                        egui::ComboBox::from_id_salt("light_kind").selected_text(l.kind.key()).show_ui(ui, |ui| {
                            for k in LightKind::ALL {
                                ui.selectable_value(&mut l.kind, k, k.key());
                            }
                        });
                        ui.end_row();
                        ui.label("Position");
                        ui.add(len_drag(&mut l.position.x, unit));
                        ui.add(len_drag(&mut l.position.y, unit));
                        ui.add(len_drag(&mut l.position.z, unit));
                        ui.end_row();
                        ui.label("");
                        if ui.small_button("To view centre").on_hover_text("Move it 2 m above the camera target").clicked() {
                            l.position = self.camera.target + DVec3::Z * 2000.0;
                        }
                        if ui.small_button("To camera").clicked() {
                            l.position = self.camera.eye;
                            l.direction = (self.camera.target - self.camera.eye).normalize_or(DVec3::NEG_Z);
                        }
                        ui.end_row();
                        if l.kind != LightKind::Point {
                            ui.label("Aim");
                            let mut d = l.direction;
                            ui.add(egui::DragValue::new(&mut d.x).speed(0.02).range(-1.0..=1.0));
                            ui.add(egui::DragValue::new(&mut d.y).speed(0.02).range(-1.0..=1.0));
                            ui.add(egui::DragValue::new(&mut d.z).speed(0.02).range(-1.0..=1.0));
                            if d != l.direction && d.length_squared() > 1e-6 {
                                l.direction = d.normalize();
                            }
                            ui.end_row();
                        }
                        ui.label("Colour");
                        ui.color_edit_button_rgb(&mut l.color);
                        ui.label("Power");
                        ui.add(egui::DragValue::new(&mut l.power).range(0.0..=100_000.0).speed(1.0).suffix(" W"));
                        ui.end_row();
                        match l.kind {
                            LightKind::Point | LightKind::Spot => {
                                ui.label("Radius");
                                let mut r = l.radius as f64;
                                if ui.add(len_drag(&mut r, unit)).changed() {
                                    l.radius = r.max(0.0) as f32;
                                }
                                if l.kind == LightKind::Spot {
                                    ui.add(egui::DragValue::new(&mut l.spot_angle).range(1.0..=179.0).suffix("°")).on_hover_text("Cone angle");
                                    ui.add(egui::DragValue::new(&mut l.spot_blend).range(0.0..=1.0).speed(0.01)).on_hover_text("Soft edge");
                                }
                                ui.end_row();
                            }
                            LightKind::Area => {
                                ui.label("Size");
                                let (mut a, mut b) = (l.size[0] as f64, l.size[1] as f64);
                                if ui.add(len_drag(&mut a, unit)).changed() | ui.add(len_drag(&mut b, unit)).changed() {
                                    l.size = [a.max(1.0) as f32, b.max(1.0) as f32];
                                }
                                ui.end_row();
                            }
                        }
                    });
                }
            });
        });
        self.render_window.selected = sel;
        if !open {
            self.render_window.open = false;
        }
        let settings = settings.sanitized();
        let lights: Vec<Light> = lights.into_iter().map(Light::sanitized).collect();
        if (lights.clone(), settings) != before {
            if self.render_window.orig.is_none() {
                self.render_window.orig = Some(before);
            }
            self.doc.model.lights = lights;
            self.doc.model.render = settings;
        }
        self.finish_render_edit(ctx);
        if start {
            self.render_window.batch = None;
            self.start_render(self.camera.clone());
        }
        if let Some(ext) = save {
            let name = if ext == "exr" { "Render.exr" } else { "Render.png" };
            if let Some(path) = rfd::FileDialog::new().add_filter(ext.to_uppercase(), &[ext]).set_file_name(name).save_file()
                && let Some(job) = &mut self.viewport.pt
            {
                job.save = Some(path);
            }
        }
        if all && let Some(folder) = rfd::FileDialog::new().pick_folder() {
            self.render_all_scenes(folder);
        }
        if let Some(job) = &mut self.viewport.pt
            && self.render_window.batch.is_none()
            && let Some(r) = job.saved.take()
        {
            match r {
                Ok(p) => self.flash(format!("Saved {}", p.display())),
                Err(e) => self.flash_error(format!("Render: {e}")),
            }
        }
    }

    /// Light markers in the view while the Render window is open.
    pub(crate) fn light_overlays(&self, o3: &mut varstotas_render::Overlay3d) {
        if !self.render_window.open {
            return;
        }
        for (i, l) in self.doc.model.lights.iter().enumerate() {
            let sel = self.render_window.selected == Some(i);
            let c = if !l.enabled {
                [0.6, 0.6, 0.6, 1.0]
            } else if sel {
                varstotas_render::HIGHLIGHT_SELECTED
            } else {
                [0.95, 0.7, 0.1, 1.0]
            };
            o3.add_marker(l.position, varstotas_render::MarkShape::Dot, 12.0, c, true);
            if l.kind != LightKind::Point {
                let reach = 400.0;
                o3.lines.push(varstotas_render::OverlayLine {
                    a: l.position.as_vec3().to_array(),
                    b: (l.position + l.direction * reach).as_vec3().to_array(),
                    color: c,
                    width_px: 1.5,
                    dashed: true,
                    on_top: true,
                });
            }
        }
    }

    /// Record the edit in progress as one undo step once the pointer is up and nothing has focus.
    fn finish_render_edit(&mut self, ctx: &Context) {
        if self.render_window.orig.is_some() && !ctx.egui_is_using_pointer() && ctx.memory(|m| m.focused().is_none()) {
            let Some((lights, settings)) = self.render_window.orig.take() else { return };
            let m = &mut self.doc.model;
            let now = (std::mem::replace(&mut m.lights, lights), std::mem::replace(&mut m.render, settings));
            self.doc.commit("Edit Lights", |m| {
                m.lights = now.0;
                m.render = now.1;
            });
        }
    }
}
