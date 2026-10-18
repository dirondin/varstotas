//! Window → Render…: path traced images of the current view or of every scene (the GPU path
//! tracer of `varstotas_render`, driven by `Viewport::pt`), saved as PNG or EXR; the model's
//! lights and render settings. Edits of lights and settings record one undo step when the
//! pointer is released and no field has focus.

use crate::app::VarstotasApp;
use crate::viewport::PtJob;
use egui::{Context, RichText, vec2};
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{Environment, Light, LightKind, RENDER_PRESETS, RENDER_SIZES, RenderDof, RenderSettings, Tonemap};
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
        let (mut lights, mut settings, mut env) = (m.lights.clone(), m.render, m.environment);
        let before = (lights.clone(), settings, env);
        let mut sel = self.doc.light_sel.filter(|i| *i < lights.len());
        let sel_before = sel;
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
            .default_size(vec2(880.0, 640.0));
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        let tex = self.viewport.pt_texture();
        let mut job = self.viewport.pt.as_mut();
        let batch = self.render_window.batch.as_ref().map(|b| (b.next, b.written.len()));
        win.show(ctx, |ui| {
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
            let label = if now.0 != self.doc.model.lights { "Edit Lights" } else { "Render Settings" };
            self.doc.commit(label, |m| {
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
        ui.label("Name");
        ui.add(egui::TextEdit::singleline(&mut l.name).desired_width(150.0));
        ui.end_row();
        ui.label("Type");
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt((id, "kind")).selected_text(l.kind.key()).width(70.0).show_ui(ui, |ui| {
                for k in LightKind::ALL {
                    ui.selectable_value(&mut l.kind, k, k.key());
                }
            });
            ui.checkbox(&mut l.enabled, "On");
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
    ui.label("Position");
    let p = &mut l.position;
    xyz(ui, [len_drag(&mut p.x, unit), len_drag(&mut p.y, unit), len_drag(&mut p.z, unit)]);
    ui.horizontal_wrapped(|ui| {
        if ui.small_button("To view centre").on_hover_text("Move it 2 m above the camera target").clicked() {
            l.position = camera.target + DVec3::Z * 2000.0;
        }
        if ui.small_button("To camera").clicked() {
            l.position = camera.eye;
            l.direction = (camera.target - camera.eye).normalize_or(DVec3::NEG_Z);
        }
        if l.kind != LightKind::Point && ui.small_button("Aim at target").on_hover_text("Shine at the view's target").clicked() {
            l.direction = (camera.target - l.position).normalize_or(l.direction);
        }
    });
    if l.kind != LightKind::Point {
        ui.label("Aim");
        let mut d = l.direction;
        xyz(ui, [&mut d.x, &mut d.y, &mut d.z].map(|v| egui::DragValue::new(v).speed(0.02).range(-1.0..=1.0).max_decimals(2)));
        if d != l.direction && d.length_squared() > 1e-6 {
            l.direction = d.normalize();
        }
    }
    egui::Grid::new((id, "light")).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
        ui.label("Colour");
        ui.horizontal(|ui| {
            ui.color_edit_button_rgb(&mut l.color);
            ui.label("Power");
            ui.add(egui::DragValue::new(&mut l.power).range(0.0..=100_000.0).speed(1.0).suffix(" W"));
        });
        ui.end_row();
        match l.kind {
            LightKind::Point | LightKind::Spot => {
                ui.label("Radius");
                ui.horizontal(|ui| {
                    let mut r = l.radius as f64;
                    if small(ui, len_drag(&mut r, unit)).on_hover_text("Bulb size: softer shadows").changed() {
                        l.radius = r.max(0.0) as f32;
                    }
                });
                ui.end_row();
                if l.kind == LightKind::Spot {
                    ui.label("Cone");
                    ui.horizontal(|ui| {
                        small(ui, egui::DragValue::new(&mut l.spot_angle).range(1.0..=179.0).suffix("°")).on_hover_text("Cone angle");
                        ui.label("Soft");
                        small(ui, egui::DragValue::new(&mut l.spot_blend).range(0.0..=1.0).speed(0.01)).on_hover_text("Soft edge");
                    });
                    ui.end_row();
                }
            }
            LightKind::Area => {
                ui.label("Size");
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

fn section(ui: &mut egui::Ui, title: &str, open: bool, body: impl FnOnce(&mut egui::Ui)) {
    egui::CollapsingHeader::new(title).id_salt(("render_section", title)).default_open(open).show(ui, |ui| {
        egui::Grid::new(("render_grid", title)).num_columns(4).spacing([8.0, 4.0]).show(ui, body);
    });
}

/// The settings part of the Render window: quality presets, image, sampling, camera, light and
/// colour. Grading (Colour) shows on the render at once; the rest at the next render.
fn settings_ui(ui: &mut egui::Ui, s: &mut RenderSettings, env: &mut Environment, camera: &Camera, unit: varstotas_kernel::LengthUnit) {
    ui.horizontal(|ui| {
        ui.label("Quality");
        for (name, samples, bounces, _) in RENDER_PRESETS {
            let tip = format!("{samples} samples, {bounces} bounces");
            if ui.selectable_label(s.preset() == Some(name), name).on_hover_text(tip).clicked() {
                s.apply_preset(name);
            }
        }
    });
    section(ui, "Image", true, |ui| {
        ui.label("Size");
        ui.add_enabled(!s.match_view, egui::DragValue::new(&mut s.width).range(16..=8192).suffix(" px"));
        ui.add(egui::DragValue::new(&mut s.height).range(16..=8192).suffix(" px"));
        ui.checkbox(&mut s.match_view, "Match view").on_hover_text("The view's proportions at this height");
        ui.end_row();
        ui.label("");
        let current = RENDER_SIZES.iter().find(|r| (r.1, r.2) == (s.width, s.height) && !s.match_view).map_or("Custom", |r| r.0);
        egui::ComboBox::from_id_salt("render_size").selected_text(current).width(150.0).show_ui(ui, |ui| {
            for (name, w, h) in RENDER_SIZES {
                if ui.selectable_label(current == name, name).clicked() {
                    (s.width, s.height, s.match_view) = (w, h, false);
                }
            }
        });
        ui.checkbox(&mut s.transparent, "Transparent background")
            .on_hover_text("The sky and the ground behind the model are transparent in the PNG (they still light it)");
        ui.end_row();
    });
    section(ui, "Sampling", true, |ui| {
        ui.label("Samples");
        ui.add(egui::DragValue::new(&mut s.samples).range(1..=100_000));
        ui.label("Time limit");
        ui.add(egui::DragValue::new(&mut s.time_limit).range(0.0..=36_000.0).suffix(" s")).on_hover_text("0 = until the samples are done");
        ui.end_row();
        ui.label("Bounces");
        ui.add(egui::DragValue::new(&mut s.bounces).range(1..=32));
        ui.label("Clamp");
        ui.add(egui::DragValue::new(&mut s.clamp).range(0.0..=1000.0).speed(0.5))
            .on_hover_text("Brightest indirect light of a sample: lower kills fireflies, higher keeps bright bounce light; 0 = off");
        ui.end_row();
        ui.label("Denoiser");
        let oidn = varstotas_render::oidn_available();
        let current = match (s.denoise, s.ai_denoise && oidn) {
            (false, _) => "Off",
            (true, false) => "Built-in",
            (true, true) => "AI (Open Image Denoise)",
        };
        egui::ComboBox::from_id_salt("render_denoiser").selected_text(current).width(150.0).show_ui(ui, |ui| {
            if ui.selectable_label(!s.denoise, "Off").clicked() {
                s.denoise = false;
            }
            if ui.selectable_label(current == "Built-in", "Built-in").on_hover_text("Edge-aware filter").clicked() {
                (s.denoise, s.ai_denoise) = (true, false);
            }
            let ai = ui
                .add_enabled(oidn, egui::Button::selectable(current.starts_with("AI"), "AI (Open Image Denoise)"))
                .on_disabled_hover_text("Install Open Image Denoise: brew install open-image-denoise");
            if ai.clicked() {
                (s.denoise, s.ai_denoise) = (true, true);
            }
        });
        ui.end_row();
    });
    section(ui, "Camera", false, |ui| {
        ui.label("Depth of field");
        egui::ComboBox::from_id_salt("render_dof").selected_text(s.dof.label()).width(90.0).show_ui(ui, |ui| {
            for d in RenderDof::ALL {
                ui.selectable_value(&mut s.dof, d, d.label());
            }
        })
        .response
        .on_hover_text("As view: the Effects' depth of field when it is on");
        ui.label(RichText::new(format!("{:.0} mm lens", focal_length(camera))).weak()).on_hover_text("The view's field of view on a 35 mm camera");
        ui.end_row();
        if s.dof == RenderDof::On {
            ui.label("Aperture");
            ui.add(egui::DragValue::new(&mut s.f_stop).range(0.5..=64.0).speed(0.05).prefix("f/").max_decimals(1))
                .on_hover_text("Smaller f-number = shallower focus");
            ui.label("Focus");
            ui.horizontal(|ui| {
                let mut f = if s.focus > 0.0 { s.focus } else { camera.distance() };
                if ui.add(len_drag(&mut f, unit)).changed() {
                    s.focus = f.max(1.0);
                }
                if ui.small_button("Target").on_hover_text("Focus on the view's target (orbit centre)").clicked() {
                    s.focus = 0.0;
                }
            });
            ui.end_row();
        }
    });
    section(ui, "Light", false, |ui| {
        ui.label("Sun");
        ui.add(egui::DragValue::new(&mut env.sun).range(0.0..=20.0).speed(0.02)).on_hover_text("Sun strength (its direction: Shadows)");
        ui.label("Sky");
        ui.add(egui::DragValue::new(&mut env.intensity).range(0.0..=20.0).speed(0.02)).on_hover_text("Environment light");
        ui.end_row();
        ui.label("Rotation");
        ui.add(egui::DragValue::new(&mut env.rotation_deg).range(-360.0..=360.0).suffix("°")).on_hover_text("Turns the sky / HDRI");
        ui.label("Exposure");
        ui.add(egui::DragValue::new(&mut env.exposure).range(-10.0..=10.0).speed(0.02).suffix(" EV")).on_hover_text("The environment's exposure (view and render)");
        ui.end_row();
    });
    section(ui, "Colour", false, |ui| {
        ui.label("Exposure");
        ui.add(egui::DragValue::new(&mut s.exposure).range(-10.0..=10.0).speed(0.02).suffix(" EV")).on_hover_text("Added to the environment's (render only)");
        ui.label("Tone curve");
        let label = |t: Option<Tonemap>| match t {
            None => format!("Environment: {}", crate::ui::realistic::tonemap_name(env.tonemap)),
            Some(t) => crate::ui::realistic::tonemap_name(t).to_string(),
        };
        egui::ComboBox::from_id_salt("render_tonemap").selected_text(label(s.tonemap)).width(130.0).show_ui(ui, |ui| {
            for t in [None, Some(Tonemap::Aces), Some(Tonemap::Agx), Some(Tonemap::Clamp)] {
                ui.selectable_value(&mut s.tonemap, t, label(t));
            }
        });
        ui.end_row();
        ui.label("White balance");
        ui.add(egui::DragValue::new(&mut s.temperature).range(1500.0..=15000.0).speed(20.0).suffix(" K"))
            .on_hover_text("6500 K = neutral; higher is warmer");
        ui.label("Tint");
        ui.add(egui::DragValue::new(&mut s.tint).range(-1.0..=1.0).speed(0.01)).on_hover_text("Green (−) .. magenta (+)");
        ui.end_row();
        ui.label("Saturation");
        ui.add(egui::DragValue::new(&mut s.saturation).range(0.0..=2.0).speed(0.01));
        ui.label("Contrast");
        ui.add(egui::DragValue::new(&mut s.contrast).range(0.5..=2.0).speed(0.01));
        ui.end_row();
        ui.label("Vignette");
        ui.add(egui::DragValue::new(&mut s.vignette).range(0.0..=1.0).speed(0.01));
        ui.label("");
        if ui.add_enabled(!s.neutral_grade(), egui::Button::new("Reset")).on_hover_text("Back to the default colour settings").clicked() {
            s.reset_grade();
        }
        ui.end_row();
    });
}
