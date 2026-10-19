//! File → Export → Animation…: a scene tour or an orbit as a video (ffmpeg) or a PNG
//! sequence, one frame per app frame — the view's style (`Viewport::export`) or the path tracer
//! (`Viewport::pt`) with a sample budget per frame.

use crate::animation::{FrameSpec, encode, ffmpeg, frame_path, orbit_frames, scene_frames};
use crate::app::{ExportContent, VarstotasApp};
use egui::{Context, vec2};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimSource {
    Scenes,
    Orbit,
}

pub struct AnimWindow {
    pub open: bool,
    pub source: AnimSource,
    /// Orbit length (s) and turns.
    pub seconds: f64,
    pub turns: f64,
    pub fps: u32,
    pub width: u32,
    pub height: u32,
    /// Path traced frames with this many samples each (else the view's style).
    pub traced: bool,
    pub spp: u32,
    /// Encode a video (needs ffmpeg), else keep the PNG files.
    pub video: bool,
    pub run: Option<AnimRun>,
    encoding: Option<Receiver<Result<PathBuf, String>>>,
    /// Outcome of the last export, for the window and the harness.
    pub last: Option<Result<String, String>>,
}

impl Default for AnimWindow {
    fn default() -> Self {
        Self {
            open: false,
            source: AnimSource::Scenes,
            seconds: 8.0,
            turns: 1.0,
            fps: 30,
            width: 1280,
            height: 720,
            traced: false,
            spp: 16,
            video: true,
            run: None,
            encoding: None,
            last: None,
        }
    }
}

pub struct AnimRun {
    frames: Vec<FrameSpec>,
    next: usize,
    dir: PathBuf,
    video: Option<PathBuf>,
    fps: f64,
    size: [u32; 2],
    spp: Option<u32>,
    /// The frame file being rendered.
    waiting: Option<PathBuf>,
    started: std::time::Instant,
    /// The view before the export, restored after it.
    orig: varstotas_render::Camera,
}

impl AnimRun {
    pub fn progress(&self) -> (usize, usize) {
        (self.next.saturating_sub(usize::from(self.waiting.is_some())), self.frames.len())
    }
}

impl AnimWindow {
    /// Frames are being rendered or encoded.
    pub fn busy(&self) -> bool {
        self.run.is_some() || self.encoding.is_some()
    }
}

impl VarstotasApp {
    /// Start exporting with the window's settings into `out` (the video file, or the folder of
    /// the PNG sequence).
    pub(crate) fn start_animation(&mut self, out: PathBuf) {
        let w = &self.anim_window;
        let fps = w.fps.clamp(1, 120) as f64;
        let frames = match w.source {
            AnimSource::Scenes => scene_frames(&self.doc.model.scenes, &self.doc.model.scene_settings, fps),
            AnimSource::Orbit => orbit_frames(&self.camera, w.seconds, w.turns, fps),
        };
        if frames.is_empty() {
            self.anim_window.last = Some(Err("no scenes in the animation".into()));
            self.flash_error("No scenes in the animation");
            return;
        }
        let (dir, video) = if w.video {
            let dir = std::env::temp_dir().join(format!("varstotas_anim_{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            (dir, Some(out))
        } else {
            (out, None)
        };
        if let Err(e) = std::fs::create_dir_all(&dir) {
            self.anim_window.last = Some(Err(e.to_string()));
            self.flash_error(format!("Animation: {e}"));
            return;
        }
        let size = [w.width.clamp(16, 8192), w.height.clamp(16, 8192)];
        let spp = w.traced.then_some(w.spp.max(1));
        self.anim_window.last = None;
        self.anim_window.run = Some(AnimRun { frames, next: 0, dir, video, fps, size, spp, waiting: None, started: std::time::Instant::now(), orig: self.camera.clone() });
    }

    /// Advance the export: collect the frame in flight, then queue the next one. Runs before
    /// the viewport paints so the frame's camera and scene are used.
    pub(crate) fn anim_tick(&mut self, ctx: &Context) {
        if let Some(rx) = &self.anim_window.encoding
            && let Ok(r) = rx.try_recv()
        {
            self.anim_window.encoding = None;
            match r {
                Ok(p) => {
                    self.flash(format!("Saved {}", p.display()));
                    self.anim_window.last = Some(Ok(p.display().to_string()));
                }
                Err(e) => {
                    self.flash_error(format!("Animation: {e}"));
                    self.anim_window.last = Some(Err(e));
                }
            }
        }
        if self.anim_window.encoding.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        let Some(mut run) = self.anim_window.run.take() else { return };
        ctx.request_repaint();
        if let Some(path) = run.waiting.clone() {
            let done = if run.spp.is_some() {
                match &mut self.viewport.pt {
                    Some(job) => match job.saved.take() {
                        Some(r) => Some(r.map(|_| ())),
                        None => {
                            if !job.running && !job.restart && job.save.is_none() {
                                job.save = Some(path);
                            }
                            None
                        }
                    },
                    None => Some(Err("the render was dropped".into())),
                }
            } else {
                self.viewport.export_result.take().map(|r| r.map(|_| ()))
            };
            match done {
                None => {
                    self.anim_window.run = Some(run);
                    return;
                }
                Some(Err(e)) => {
                    self.camera = run.orig;
                    self.flash_error(format!("Animation: {e}"));
                    self.anim_window.last = Some(Err(e));
                    return;
                }
                Some(Ok(())) => run.waiting = None,
            }
        }
        if let Some(frame) = run.frames.get(run.next).cloned() {
            if let Some(i) = frame.scene {
                self.go_to_scene(i);
            }
            self.cam_anim = None;
            self.rig_anim = None;
            if let Some((a, b, s)) = frame.rigs {
                self.frame_rigs(a, b, s);
            }
            self.camera = frame.camera;
            let path = frame_path(&run.dir, run.next);
            match run.spp {
                Some(spp) => {
                    self.start_render(self.camera.clone());
                    if let Some(job) = &mut self.viewport.pt {
                        job.settings.width = run.size[0];
                        job.settings.height = run.size[1];
                        job.max_samples = spp;
                        job.time_limit = 0.0;
                    }
                }
                None => {
                    let content = ExportContent { transparent: false, axes: false, guides: false, dims: false };
                    self.export_png_to(path.clone(), run.size, content);
                }
            }
            run.waiting = Some(path);
            run.next += 1;
            self.anim_window.run = Some(run);
            return;
        }
        // All frames written.
        self.camera = run.orig.clone();
        let n = run.frames.len();
        match run.video {
            Some(out) => match ffmpeg() {
                Some(bin) => {
                    let (tx, rx) = std::sync::mpsc::channel();
                    let (dir, fps) = (run.dir.clone(), run.fps);
                    std::thread::spawn(move || {
                        let r = encode(&bin, &dir, fps, &out).map(|_| out);
                        let _ = std::fs::remove_dir_all(&dir);
                        let _ = tx.send(r);
                    });
                    self.anim_window.encoding = Some(rx);
                }
                None => {
                    let msg = format!("ffmpeg not found; the {n} frames are in {}", run.dir.display());
                    self.flash_error(msg.clone());
                    self.anim_window.last = Some(Err(msg));
                }
            },
            None => {
                let msg = format!("{n} frames in {}", run.dir.display());
                log::info!("animation: {msg} ({:.1} s)", run.started.elapsed().as_secs_f64());
                self.flash(format!("Saved {msg}"));
                self.anim_window.last = Some(Ok(msg));
            }
        }
    }

    pub(crate) fn animation_window(&mut self, ctx: &Context) {
        if !self.anim_window.open {
            return;
        }
        let has_ffmpeg = ffmpeg().is_some();
        let has_scenes = self.doc.model.scenes.iter().any(|s| s.in_animation);
        let mut open = true;
        let mut export = false;
        let mut cancel = false;
        let pos = self.viewport.view_rect.left_top() + vec2(60.0, 40.0);
        let mut win = egui::Window::new("Export Animation")
            .id(egui::Id::new("animation_window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_pos(pos);
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        let w = &mut self.anim_window;
        let progress = w.run.as_ref().map(|r| r.progress());
        win.show(ctx, |ui| {
            ui.set_width(380.0);
            ui.add_enabled_ui(!w.busy(), |ui| {
                egui::Grid::new("anim_settings").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                    ui.label("Animate");
                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(has_scenes, |ui| ui.radio_value(&mut w.source, AnimSource::Scenes, "Scenes"))
                            .response
                            .on_disabled_hover_text("Add scenes (with “Include in animation”) first");
                        ui.radio_value(&mut w.source, AnimSource::Orbit, "Orbit around the view");
                    });
                    ui.end_row();
                    if w.source == AnimSource::Orbit {
                        ui.label("Length");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut w.seconds).range(0.5..=600.0).suffix(" s"));
                            ui.add(egui::DragValue::new(&mut w.turns).range(-10.0..=10.0).speed(0.05).suffix(" turns"));
                        });
                        ui.end_row();
                    } else {
                        ui.label("");
                        ui.label(egui::RichText::new("Holds and transitions: Scenes card → Animation").small().weak());
                        ui.end_row();
                    }
                    ui.label("Size");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut w.width).range(16..=8192).suffix(" px"));
                        ui.label("×");
                        ui.add(egui::DragValue::new(&mut w.height).range(16..=8192).suffix(" px"));
                    });
                    ui.end_row();
                    ui.label("Frame rate");
                    ui.add(egui::DragValue::new(&mut w.fps).range(1..=120).suffix(" fps"));
                    ui.end_row();
                    ui.label("Frames");
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut w.traced, false, "As the view");
                        ui.radio_value(&mut w.traced, true, "Path traced");
                        if w.traced {
                            ui.add(egui::DragValue::new(&mut w.spp).range(1..=4096).suffix(" spp"));
                        }
                    });
                    ui.end_row();
                    ui.label("Output");
                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(has_ffmpeg, |ui| ui.radio_value(&mut w.video, true, "Video (MP4)"))
                            .response
                            .on_disabled_hover_text("Install ffmpeg to make videos");
                        ui.radio_value(&mut w.video, false, "PNG sequence");
                    });
                    ui.end_row();
                });
            });
            if !has_ffmpeg && w.video {
                w.video = false;
            }
            if w.source == AnimSource::Scenes && !has_scenes {
                w.source = AnimSource::Orbit;
            }
            ui.separator();
            if let Some((done, total)) = progress {
                ui.add(egui::ProgressBar::new(done as f32 / total.max(1) as f32).text(format!("Frame {done} / {total}")));
                if ui.button("Cancel").clicked() {
                    cancel = true;
                }
            } else if w.encoding.is_some() {
                ui.add(egui::ProgressBar::new(1.0).text("Encoding the video…").animate(true));
            } else {
                if ui.button("Export…").clicked() {
                    export = true;
                }
                match &w.last {
                    Some(Ok(m)) => ui.label(egui::RichText::new(format!("Saved {m}")).small()),
                    Some(Err(e)) => ui.label(egui::RichText::new(e).small().color(crate::ui::realistic::ERROR_RED)),
                    None => ui.label(""),
                };
            }
        });
        if !open {
            self.anim_window.open = false;
        }
        if cancel {
            if let Some(run) = self.anim_window.run.take() {
                self.camera = run.orig;
            }
            self.anim_window.last = Some(Err("cancelled".into()));
        }
        if export {
            let out = if self.anim_window.video {
                rfd::FileDialog::new().add_filter("MP4 video", &["mp4"]).set_file_name("Animation.mp4").save_file()
            } else {
                rfd::FileDialog::new().pick_folder()
            };
            if let Some(out) = out {
                self.start_animation(out);
            }
        }
    }
}
