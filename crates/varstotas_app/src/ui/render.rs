//! Window → Render: path traced images of the current view or of every scene (the GPU path
//! tracer of `varstotas_render`, driven by `Viewport::pt`), saved as PNG or EXR; the model's
//! lights and render settings. Edits of lights and settings record one undo step when the
//! pointer is released and no field has focus.

use crate::app::VarstotasApp;
use crate::viewport::PtJob;
use varstotas_i18n::tr;
use egui::{Context, RichText, vec2};
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{Environment, Light, LightKind, RENDER_PRESETS, RENDER_SIZES, RenderDof, RenderSettings, Tonemap, render_preset_label, render_size_label};
use varstotas_render::{Camera, FaceStyle, PtSettings};
use std::path::PathBuf;

type RenderState = (Vec<Light>, RenderSettings, Environment);

#[derive(Default)]
pub struct RenderWindow {
    pub open: bool,
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

/// Lens opening (radius, mm) of a 35 mm camera with the view's field of view at `f_stop`.
pub fn lens_radius(camera: &Camera, f_stop: f32) -> f32 {
    (focal_length(camera) / (2.0 * f_stop.max(0.5) as f64)) as f32
}

/// The 35 mm focal length of the view (mm; a 24 mm high frame).
pub fn focal_length(camera: &Camera) -> f64 {
    12.0 / (camera.fov_y_deg.to_radians() * 0.5).tan().max(1e-3)
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
        let lens = match rs.dof {
            RenderDof::View => None,
            RenderDof::Off => Some([0.0, 0.0]),
            RenderDof::On => Some([lens_radius(&camera, rs.f_stop), rs.focus as f32]),
        };
        let settings = PtSettings { width: w, height: h, bounces: rs.bounces, clamp: rs.clamp, lens, transparent: rs.transparent };
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
                        self.flash_error(tr!("render-failed", error = e.to_string()));
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
                self.flash(tr!("render-scenes-done", n = b.written.len(), folder = b.folder.display().to_string()));
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
        let (mut lights, mut settings, mut env) = (m.lights.clone(), m.render, m.environment);
        // Sanitized like the result below, so values that only re-sanitize differently (a
        // re-normalized light direction) are not an edit.
        let before = (lights.iter().cloned().map(Light::sanitized).collect::<Vec<_>>(), settings.sanitized(), env.sanitized());
        let mut sel = self.doc.light_sel.filter(|i| *i < lights.len());
        let sel_before = sel;
        let mut open = true;
        let mut start = false;
        let mut save: Option<&'static str> = None;
        let mut all = false;
        let has_scenes = !m.scenes.is_empty();
        let pos = self.viewport.view_rect.left_top() + vec2(40.0, 30.0);
        let mut win = egui::Window::new(tr!("render-title"))
            .id(egui::Id::new("render_window"))
            .open(&mut open)
            .collapsible(false)
            .default_pos(pos)
            .default_size(vec2(880.0, 640.0));
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        let tex = self.viewport.pt_texture();
        let mut job = self.viewport.pt.as_mut();
        let batch = self.render_window.batch.as_ref().map(|b| (b.next, b.written.len()));
        win.show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button(tr!("render-title")).on_hover_text(tr!("render-current-view")).clicked() {
                    start = true;
                }
                if let Some(job) = job.as_deref_mut() {
                    if job.running {
                        if ui.button(tr!("render-stop")).clicked() {
                            job.running = false;
                        }
                    } else if job.samples > 0 && !job.done() && ui.button(tr!("render-continue")).clicked() {
                        job.running = true;
                    } else if job.samples > 0 && job.done() && ui.button(tr!("render-more-samples")).on_hover_text(tr!("render-more-samples-tip")).clicked() {
                        job.max_samples = job.samples * 2;
                        job.time_limit = 0.0;
                        job.running = true;
                    }
                    job.denoise = settings.denoise;
                    let rendered = job.samples > 0;
                    if ui.add_enabled(rendered, egui::Button::new(tr!("render-save-png"))).clicked() {
                        save = Some("png");
                    }
                    if ui.add_enabled(rendered, egui::Button::new(tr!("render-save-exr"))).on_hover_text(tr!("render-save-exr-tip")).clicked() {
                        save = Some("exr");
                    }
                }
                if ui.add_enabled(has_scenes, egui::Button::new(tr!("render-all-scenes"))).clicked() {
                    all = true;
                }
            });
            if let Some((next, written)) = batch {
                ui.label(tr!("render-scenes-progress", written = written, next = next));
            }
            if let Some(job) = job.as_deref() {
                let left = job.max_samples.saturating_sub(job.samples);
                let frac = if job.time_limit > 0.0 {
                    (job.samples as f64 / job.max_samples as f64).max(job.elapsed / job.time_limit)
                } else {
                    job.samples as f64 / job.max_samples as f64
                };
                let status = if job.running {
                    tr!("render-status-running", samples = job.samples, max = job.max_samples, secs = format!("{:.1}", job.elapsed))
                } else if left == 0 || job.done() {
                    tr!("render-status-done", samples = job.samples, secs = format!("{:.1}", job.elapsed), denoised = if job.denoised { "yes" } else { "no" })
                } else {
                    tr!("render-status-stopped", samples = job.samples, secs = format!("{:.1}", job.elapsed))
                };
                ui.add(egui::ProgressBar::new(frac.clamp(0.0, 1.0) as f32).text(status));
                if let Some(Err(e)) = &job.saved {
                    ui.label(RichText::new(e).color(crate::ui::realistic::ERROR_RED));
                }
            }
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(360.0);
                    egui::ScrollArea::vertical().id_salt("render_settings_scroll").max_height(440.0).show(ui, |ui| {
                        settings_ui(ui, &mut settings, &mut env, &self.camera, unit);
                    });
                });
                if let Some((id, size)) = tex {
                    let avail = ui.available_width().max(100.0);
                    let k = (avail / size[0] as f32).min(440.0 / size[1] as f32).min(1.0);
                    ui.image(egui::load::SizedTexture::new(id, vec2(size[0] as f32 * k, size[1] as f32 * k)));
                }
            });
            egui::CollapsingHeader::new(tr!("render-lights", n = lights.len())).id_salt("render_lights").default_open(true).show(ui, |ui| {
                ui.horizontal(|ui| {
                    for k in LightKind::ALL {
                        let label = match k {
                            LightKind::Point => tr!("render-add-point"),
                            LightKind::Spot => tr!("render-add-spot"),
                            LightKind::Area => tr!("render-add-area"),
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
                        ui.checkbox(&mut l.enabled, "").on_hover_text(tr!("render-on"));
                        if ui.selectable_label(sel == Some(i), &l.name).clicked() {
                            sel = Some(i);
                        }
                        ui.label(RichText::new(l.kind.label()).small().weak());
                        if ui.small_button(tr!("delete")).clicked() {
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
                    light_editor(ui, "render_light", l, unit, &self.camera);
                }
            });
        });
        if !open {
            self.render_window.open = false;
        }
        let settings = settings.sanitized();
        let lights: Vec<Light> = lights.into_iter().map(Light::sanitized).collect();
        let env = env.sanitized();
        if (lights.clone(), settings, env) != before {
            if self.render_window.orig.is_none() {
                self.render_window.orig = Some(before);
            }
            self.doc.model.lights = lights;
            self.doc.model.render = settings;
            self.doc.model.environment = env;
        }
        if sel != sel_before {
            match sel {
                Some(i) => self.doc.select_light(i),
                None => self.doc.clear_selection(),
            }
        }
        self.finish_render_edit(ctx);
        if start {
            self.render_window.batch = None;
            self.start_render(self.camera.clone());
        }
        if let Some(ext) = save {
            let name = if ext == "exr" { "Render.exr" } else { "Render.png" }; // i18n-ignore
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
                Ok(p) => self.flash(tr!("render-saved", path = p.display().to_string())),
                Err(e) => self.flash_error(tr!("render-error", error = e.to_string())),
            }
        }
    }

    /// Change light `i` as part of the edit in progress (recorded by `finish_render_edit`).
    pub(crate) fn edit_light(&mut self, i: usize, l: Light) {
        let m = &self.doc.model;
        if i >= m.lights.len() {
            return;
        }
        if self.render_window.orig.is_none() {
            self.render_window.orig = Some((m.lights.clone(), m.render, m.environment));
        }
        self.doc.model.lights[i] = l;
    }

    /// Record the edit in progress as one undo step once the pointer is up and nothing has focus.
    fn finish_render_edit(&mut self, ctx: &Context) {
        if self.render_window.orig.is_some() && !ctx.egui_is_using_pointer() && ctx.memory(|m| m.focused().is_none()) {
            let Some((lights, settings, env)) = self.render_window.orig.take() else { return };
            let m = &mut self.doc.model;
            let now = (
                std::mem::replace(&mut m.lights, lights),
                std::mem::replace(&mut m.render, settings),
                std::mem::replace(&mut m.environment, env),
            );
            let label = tr(if now.0 != self.doc.model.lights { "render-undo-lights" } else { "render-undo-settings" });
            self.doc.commit(&label, |m| {
                m.lights = now.0;
                m.render = now.1;
                m.environment = now.2;
            });
        }
    }
}

/// The properties of a light (Render window, Entity Info): label | widgets rows, narrow enough
/// for the tray.
pub(crate) fn light_editor(ui: &mut egui::Ui, id: &str, l: &mut Light, unit: varstotas_kernel::LengthUnit, camera: &Camera) {
    let small = |ui: &mut egui::Ui, w: egui::DragValue<'_>| ui.add_sized([58.0, 18.0], w);
    egui::Grid::new(id).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
        ui.label(tr!("render-light-name"));
        ui.add(egui::TextEdit::singleline(&mut l.name).desired_width(150.0));
        ui.end_row();
        ui.label(tr!("render-light-type"));
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt((id, "kind")).selected_text(l.kind.label()).width(70.0).show_ui(ui, |ui| {
                for k in LightKind::ALL {
                    ui.selectable_value(&mut l.kind, k, k.label());
                }
            });
            ui.checkbox(&mut l.enabled, tr!("render-on"));
        });
        ui.end_row();
    });
    // Coordinates in full-width rows (three lengths do not fit beside a label in the tray).
    let xyz = |ui: &mut egui::Ui, w: [egui::DragValue<'_>; 3]| {
        ui.horizontal(|ui| {
            let fw = ((ui.available_width() - 2.0 * ui.spacing().item_spacing.x) / 3.0).clamp(50.0, 90.0);
            for d in w {
                ui.add_sized([fw, 18.0], d);
            }
        });
    };
    ui.label(tr!("render-light-position"));
    let p = &mut l.position;
    xyz(ui, [len_drag(&mut p.x, unit), len_drag(&mut p.y, unit), len_drag(&mut p.z, unit)]);
    ui.horizontal_wrapped(|ui| {
        if ui.small_button(tr!("render-light-to-centre")).on_hover_text(tr!("render-light-to-centre-tip")).clicked() {
            l.position = camera.target + DVec3::Z * 2000.0;
        }
        if ui.small_button(tr!("render-light-to-camera")).clicked() {
            l.position = camera.eye;
            l.direction = (camera.target - camera.eye).normalize_or(DVec3::NEG_Z);
        }
        if l.kind != LightKind::Point && ui.small_button(tr!("render-light-aim-at-target")).on_hover_text(tr!("render-light-aim-at-target-tip")).clicked() {
            l.direction = (camera.target - l.position).normalize_or(l.direction);
        }
    });
    if l.kind != LightKind::Point {
        ui.label(tr!("render-light-aim"));
        let mut d = l.direction;
        xyz(ui, [&mut d.x, &mut d.y, &mut d.z].map(|v| egui::DragValue::new(v).speed(0.02).range(-1.0..=1.0).max_decimals(2)));
        if d != l.direction && d.length_squared() > 1e-6 {
            l.direction = d.normalize();
        }
    }
    egui::Grid::new((id, "light")).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
        ui.label(tr!("render-colour"));
        ui.horizontal(|ui| {
            ui.color_edit_button_rgb(&mut l.color);
            ui.label(tr!("render-light-power"));
            ui.add(egui::DragValue::new(&mut l.power).range(0.0..=100_000.0).speed(1.0).suffix(" W"));
        });
        ui.end_row();
        match l.kind {
            LightKind::Point | LightKind::Spot => {
                ui.label(tr!("render-light-radius"));
                ui.horizontal(|ui| {
                    let mut r = l.radius as f64;
                    if small(ui, len_drag(&mut r, unit)).on_hover_text(tr!("render-light-radius-tip")).changed() {
                        l.radius = r.max(0.0) as f32;
                    }
                });
                ui.end_row();
                if l.kind == LightKind::Spot {
                    ui.label(tr!("render-light-cone"));
                    ui.horizontal(|ui| {
                        small(ui, egui::DragValue::new(&mut l.spot_angle).range(1.0..=179.0).suffix("°")).on_hover_text(tr!("render-light-cone-angle"));
                        ui.label(tr!("render-light-soft"));
                        small(ui, egui::DragValue::new(&mut l.spot_blend).range(0.0..=1.0).speed(0.01)).on_hover_text(tr!("render-light-soft-edge"));
                    });
                    ui.end_row();
                }
            }
            LightKind::Area => {
                ui.label(tr!("render-size"));
                ui.horizontal(|ui| {
                    let (mut a, mut b) = (l.size[0] as f64, l.size[1] as f64);
                    if small(ui, len_drag(&mut a, unit)).changed() | small(ui, len_drag(&mut b, unit)).changed() {
                        l.size = [a.max(1.0) as f32, b.max(1.0) as f32];
                    }
                });
                ui.end_row();
            }
        }
    });
}

/// A collapsible group: `id` is fixed, `title` is shown.
fn section(ui: &mut egui::Ui, id: &str, title: String, open: bool, body: impl FnOnce(&mut egui::Ui)) {
    egui::CollapsingHeader::new(title).id_salt(("render_section", id)).default_open(open).show(ui, |ui| {
        egui::Grid::new(("render_grid", id)).num_columns(4).spacing([8.0, 4.0]).show(ui, body);
    });
}

/// The settings part of the Render window: quality presets, image, sampling, camera, light and
/// colour. Grading (Colour) shows on the render at once; the rest at the next render.
fn settings_ui(ui: &mut egui::Ui, s: &mut RenderSettings, env: &mut Environment, camera: &Camera, unit: varstotas_kernel::LengthUnit) {
    ui.horizontal(|ui| {
        ui.label(tr!("render-quality"));
        for (name, samples, bounces, _) in RENDER_PRESETS {
            let tip = tr!("render-quality-tip", samples = samples, bounces = bounces);
            if ui.selectable_label(s.preset() == Some(name), render_preset_label(name)).on_hover_text(tip).clicked() {
                s.apply_preset(name);
            }
        }
    });
    section(ui, "Image", tr!("render-sec-image"), true, |ui| { // i18n-ignore
        ui.label(tr!("render-size"));
        ui.add_enabled(!s.match_view, egui::DragValue::new(&mut s.width).range(16..=8192).suffix(" px"));
        ui.add(egui::DragValue::new(&mut s.height).range(16..=8192).suffix(" px"));
        ui.checkbox(&mut s.match_view, tr!("render-match-view")).on_hover_text(tr!("render-match-view-tip"));
        ui.end_row();
        ui.label("");
        let current = RENDER_SIZES.iter().find(|r| (r.1, r.2) == (s.width, s.height) && !s.match_view).map(|r| r.0);
        let current_label = current.map_or_else(|| tr!("render-size-custom"), render_size_label);
        egui::ComboBox::from_id_salt("render_size").selected_text(current_label).width(150.0).show_ui(ui, |ui| {
            for (name, w, h) in RENDER_SIZES {
                if ui.selectable_label(current == Some(name), render_size_label(name)).clicked() {
                    (s.width, s.height, s.match_view) = (w, h, false);
                }
            }
        });
        ui.checkbox(&mut s.transparent, tr!("render-transparent")).on_hover_text(tr!("render-transparent-tip"));
        ui.end_row();
    });
    section(ui, "Sampling", tr!("render-sec-sampling"), true, |ui| { // i18n-ignore
        ui.label(tr!("render-samples"));
        ui.add(egui::DragValue::new(&mut s.samples).range(1..=100_000));
        ui.label(tr!("render-time-limit"));
        ui.add(egui::DragValue::new(&mut s.time_limit).range(0.0..=36_000.0).suffix(" s")).on_hover_text(tr!("render-time-limit-tip"));
        ui.end_row();
        ui.label(tr!("render-bounces"));
        ui.add(egui::DragValue::new(&mut s.bounces).range(1..=32));
        ui.label(tr!("render-clamp"));
        ui.add(egui::DragValue::new(&mut s.clamp).range(0.0..=1000.0).speed(0.5))
            .on_hover_text(tr!("render-clamp-tip"));
        ui.end_row();
        ui.label(tr!("render-denoiser"));
        let oidn = varstotas_render::oidn_available();
        let current = match (s.denoise, s.ai_denoise && oidn) {
            (false, _) => 0,
            (true, false) => 1,
            (true, true) => 2,
        };
        let current_label = tr(["render-denoiser-off", "render-denoiser-builtin", "render-denoiser-ai"][current]);
        egui::ComboBox::from_id_salt("render_denoiser").selected_text(current_label).width(150.0).show_ui(ui, |ui| {
            if ui.selectable_label(!s.denoise, tr!("render-denoiser-off")).clicked() {
                s.denoise = false;
            }
            if ui.selectable_label(current == 1, tr!("render-denoiser-builtin")).on_hover_text(tr!("render-denoiser-builtin-tip")).clicked() {
                (s.denoise, s.ai_denoise) = (true, false);
            }
            let ai = ui
                .add_enabled(oidn, egui::Button::selectable(current == 2, tr!("render-denoiser-ai")))
                .on_disabled_hover_text(tr!("render-denoiser-install"));
            if ai.clicked() {
                (s.denoise, s.ai_denoise) = (true, true);
            }
        });
        ui.end_row();
    });
    section(ui, "Camera", tr!("render-sec-camera"), false, |ui| { // i18n-ignore
        ui.label(tr!("render-dof"));
        egui::ComboBox::from_id_salt("render_dof").selected_text(s.dof.label()).width(90.0).show_ui(ui, |ui| {
            for d in RenderDof::ALL {
                ui.selectable_value(&mut s.dof, d, d.label());
            }
        })
        .response
        .on_hover_text(tr!("render-dof-tip"));
        ui.label(RichText::new(tr!("render-lens", mm = format!("{:.0}", focal_length(camera)))).weak()).on_hover_text(tr!("render-lens-tip"));
        ui.end_row();
        if s.dof == RenderDof::On {
            ui.label(tr!("render-aperture"));
            ui.add(egui::DragValue::new(&mut s.f_stop).range(0.5..=64.0).speed(0.05).prefix("f/").max_decimals(1))
                .on_hover_text(tr!("render-aperture-tip"));
            ui.label(tr!("render-focus"));
            ui.horizontal(|ui| {
                let mut f = if s.focus > 0.0 { s.focus } else { camera.distance() };
                if ui.add(len_drag(&mut f, unit)).changed() {
                    s.focus = f.max(1.0);
                }
                if ui.small_button(tr!("render-focus-target")).on_hover_text(tr!("render-focus-target-tip")).clicked() {
                    s.focus = 0.0;
                }
            });
            ui.end_row();
        }
    });
    section(ui, "Light", tr!("render-sec-light"), false, |ui| { // i18n-ignore
        ui.label(tr!("render-sun"));
        ui.add(egui::DragValue::new(&mut env.sun).range(0.0..=20.0).speed(0.02)).on_hover_text(tr!("render-sun-tip"));
        ui.label(tr!("render-sky"));
        ui.add(egui::DragValue::new(&mut env.intensity).range(0.0..=20.0).speed(0.02)).on_hover_text(tr!("render-sky-tip"));
        ui.end_row();
        ui.label(tr!("render-rotation"));
        ui.add(egui::DragValue::new(&mut env.rotation_deg).range(-360.0..=360.0).suffix("°")).on_hover_text(tr!("render-rotation-tip"));
        ui.label(tr!("render-exposure"));
        ui.add(egui::DragValue::new(&mut env.exposure).range(-10.0..=10.0).speed(0.02).suffix(" EV")).on_hover_text(tr!("render-env-exposure-tip"));
        ui.end_row();
    });
    section(ui, "Colour", tr!("render-sec-colour"), false, |ui| { // i18n-ignore
        ui.label(tr!("render-exposure"));
        ui.add(egui::DragValue::new(&mut s.exposure).range(-10.0..=10.0).speed(0.02).suffix(" EV")).on_hover_text(tr!("render-exposure-tip"));
        ui.label(tr!("render-tone-curve"));
        let label = |t: Option<Tonemap>| match t {
            None => tr!("render-tone-environment", name = crate::ui::realistic::tonemap_name(env.tonemap).to_string()),
            Some(t) => crate::ui::realistic::tonemap_name(t).to_string(),
        };
        egui::ComboBox::from_id_salt("render_tonemap").selected_text(label(s.tonemap)).width(130.0).show_ui(ui, |ui| {
            for t in [None, Some(Tonemap::Aces), Some(Tonemap::Agx), Some(Tonemap::Clamp)] {
                ui.selectable_value(&mut s.tonemap, t, label(t));
            }
        });
        ui.end_row();
        ui.label(tr!("render-white-balance"));
        ui.add(egui::DragValue::new(&mut s.temperature).range(1500.0..=15000.0).speed(20.0).suffix(" K"))
            .on_hover_text(tr!("render-white-balance-tip"));
        ui.label(tr!("render-tint"));
        ui.add(egui::DragValue::new(&mut s.tint).range(-1.0..=1.0).speed(0.01)).on_hover_text(tr!("render-tint-tip"));
        ui.end_row();
        ui.label(tr!("render-saturation"));
        ui.add(egui::DragValue::new(&mut s.saturation).range(0.0..=2.0).speed(0.01));
        ui.label(tr!("render-contrast"));
        ui.add(egui::DragValue::new(&mut s.contrast).range(0.5..=2.0).speed(0.01));
        ui.end_row();
        ui.label(tr!("render-vignette"));
        ui.add(egui::DragValue::new(&mut s.vignette).range(0.0..=1.0).speed(0.01));
        ui.label("");
        if ui.add_enabled(!s.neutral_grade(), egui::Button::new(tr!("render-reset"))).on_hover_text(tr!("render-reset-tip")).clicked() {
            s.reset_grade();
        }
        ui.end_row();
    });
}
