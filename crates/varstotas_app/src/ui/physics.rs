//! Tools → Physics…: drop the selected groups / components under gravity onto the rest of the
//! model (`varstotas_physics`). The simulation runs on a thread; its frames are shown live by
//! writing the bodies' transforms into the model without an undo step (`preview_change`).
//! Apply records one undo step from the model as it was before the run; Cancel restores it.
//! While a run is shown, tools and model edits are held: an edit (any action but the view's)
//! cancels the run first.

use crate::app::{Action, VarstotasApp};
use egui::{Context, RichText};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use varstotas_i18n::tr;
use varstotas_kernel::{ContextPath, InstanceId, Model};
use varstotas_physics::{DropFrame, DropSettings, DropShape, DropSim, drop_bodies, set_transforms};

#[derive(Default)]
pub struct PhysicsState {
    pub open: bool,
    pub settings: DropSettings,
    pub(crate) run: Option<PhysicsRun>,
    /// The last run's outcome (shown under the buttons).
    pub(crate) message: Option<String>,
}

pub(crate) struct PhysicsRun {
    path: ContextPath,
    bodies: Vec<InstanceId>,
    /// The model before the run (Cancel restores it, Apply records it as the undo state).
    before: Model,
    was_dirty: bool,
    /// The model version this run last wrote: anything else means the model was edited.
    version: u64,
    rx: Receiver<Result<DropFrame, String>>,
    stop: Arc<AtomicBool>,
    frame: Option<DropFrame>,
    /// No more frames will come (done, stopped or failed).
    finished: bool,
}

impl PhysicsRun {
    fn state(&self) -> &'static str {
        match (&self.frame, self.finished) {
            (_, false) => "running",
            (Some(f), true) if f.done => "done",
            _ => "stopped",
        }
    }
}

/// Simulated time per frame sent to the window (steps of 1/120 s).
const STEPS_PER_FRAME: usize = 4;

impl VarstotasApp {
    /// A run is shown (running or waiting for Apply / Cancel).
    pub(crate) fn physics_live(&self) -> bool {
        self.physics.run.is_some()
    }

    fn physics_bodies(&self) -> Vec<InstanceId> {
        drop_bodies(&self.doc.model, &self.doc.active_path, &self.doc.selection)
    }

    pub(crate) fn start_physics(&mut self, ctx: &Context) {
        self.physics_cancel();
        let bodies = self.physics_bodies();
        if bodies.is_empty() {
            self.physics.message = Some(tr!("physics-no-bodies"));
            return;
        }
        let path = self.doc.active_path.clone();
        let model = self.doc.model.clone();
        let settings = self.physics.settings.clone();
        let (tx, rx) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let (t_path, t_bodies, t_stop, t_ctx) = (path.clone(), bodies.clone(), stop.clone(), ctx.clone());
        std::thread::spawn(move || {
            let run = crate::document::guarded(|| {
                let mut sim = match DropSim::new(&model, &t_path, &t_bodies, settings) {
                    Ok(sim) => sim,
                    Err(e) => return Err(e.to_string()),
                };
                loop {
                    let frame = sim.step(STEPS_PER_FRAME);
                    let done = frame.done;
                    if tx.send(Ok(frame)).is_err() {
                        return Ok(());
                    }
                    t_ctx.request_repaint();
                    if done || t_stop.load(Ordering::Relaxed) {
                        return Ok(());
                    }
                }
            });
            if let Err(e) = run.and_then(|r| r) {
                let _ = tx.send(Err(e));
            }
            t_ctx.request_repaint();
        });
        self.physics.message = None;
        self.physics.run = Some(PhysicsRun {
            path,
            bodies,
            before: self.doc.model.clone(),
            was_dirty: self.doc.dirty,
            version: self.doc.model_version,
            rx,
            stop,
            frame: None,
            finished: false,
        });
    }

    /// Show the newest frame of the run (every frame of the app).
    pub(crate) fn poll_physics(&mut self) {
        let Some(run) = self.physics.run.as_mut() else { return };
        if self.doc.model_version != run.version {
            // edited behind the run (a panel's commit): keep the model as it is now
            run.stop.store(true, Ordering::Relaxed);
            self.physics.run = None;
            self.physics.message = Some(tr!("physics-interrupted"));
            return;
        }
        let mut newest = None;
        loop {
            match run.rx.try_recv() {
                Ok(Ok(frame)) => newest = Some(frame),
                Ok(Err(e)) => {
                    run.finished = true;
                    self.physics.message = Some(tr!("physics-failed", error = e));
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    run.finished = true;
                    break;
                }
            }
        }
        if let Some(frame) = newest {
            let (path, bodies) = (run.path.clone(), run.bodies.clone());
            self.doc.preview_change(|m| set_transforms(m, &path, &bodies, &frame.transforms));
            run.version = self.doc.model_version;
            run.finished |= frame.done;
            run.frame = Some(frame);
        }
    }

    pub(crate) fn physics_stop(&mut self) {
        if let Some(run) = &self.physics.run {
            run.stop.store(true, Ordering::Relaxed);
        }
    }

    /// Keep the shown positions as one undo step.
    pub(crate) fn physics_apply(&mut self) {
        let Some(run) = self.physics.run.take() else { return };
        run.stop.store(true, Ordering::Relaxed);
        let moved = run.frame.as_ref().map_or(0, |f| f.fell.iter().filter(|x| !**x).count());
        if moved == 0 {
            self.doc.preview_change(|m| *m = run.before);
            self.doc.dirty = run.was_dirty;
            return;
        }
        self.doc.record_step(&tr!("physics-undo"), run.before);
        self.physics.message = Some(tr!("physics-applied", n = moved));
    }

    /// Put everything back where it was before the run.
    pub(crate) fn physics_cancel(&mut self) {
        let Some(run) = self.physics.run.take() else { return };
        run.stop.store(true, Ordering::Relaxed);
        self.doc.preview_change(|m| *m = run.before);
        self.doc.dirty = run.was_dirty;
    }

    /// Before an action runs while a run is shown: the view's own actions pass, Undo just
    /// cancels the run, anything else cancels it and then runs. Returns false to drop `action`.
    pub(crate) fn physics_before_action(&mut self, action: &Action) -> bool {
        if !self.physics_live() {
            return true;
        }
        let passes = matches!(
            action,
            Action::Physics
                | Action::View(_)
                | Action::SetParallel(_)
                | Action::ZoomExtents
                | Action::SetPanes(_)
                | Action::ViewPreset(_)
                | Action::ToggleGrid
                | Action::ToggleTray
                | Action::ToggleCard(_)
                | Action::ToggleLog
        );
        if passes {
            return true;
        }
        self.physics_cancel();
        !matches!(action, Action::Undo)
    }

    pub(crate) fn physics_window(&mut self, ctx: &Context) {
        self.poll_physics();
        if !self.physics.open {
            if self.physics_live() {
                self.physics_cancel();
            }
            return;
        }
        let pos = self.viewport.view_rect.min + egui::vec2(40.0, 40.0);
        let mut open = true;
        let mut win = egui::Window::new(tr!("physics-title"))
            .id(egui::Id::new("physics_window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_pos(pos);
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        let bodies = if self.physics_live() { 0 } else { self.physics_bodies().len() };
        let (mut run, mut stop, mut apply, mut cancel) = (false, false, false, false);
        win.show(ctx, |ui| {
            ui.set_width(340.0);
            ui.label(RichText::new(tr!("physics-drop")).strong());
            ui.label(RichText::new(tr!("physics-drop-hint")).small().weak());
            ui.add_space(4.0);
            let live = self.physics.run.is_some();
            ui.add_enabled_ui(!live, |ui| {
                let s = &mut self.physics.settings;
                egui::Grid::new("physics_opts").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                    ui.label(tr!("physics-shape"));
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut s.shape, DropShape::Hull, tr!("physics-shape-hull"))
                            .on_hover_text(tr!("physics-shape-hull-hint"));
                        ui.selectable_value(&mut s.shape, DropShape::Pieces, tr!("physics-shape-pieces"))
                            .on_hover_text(tr!("physics-shape-pieces-hint"));
                    });
                    ui.end_row();
                    ui.label(tr!("physics-friction"));
                    ui.add(egui::Slider::new(&mut s.friction, 0.0..=1.5).fixed_decimals(2));
                    ui.end_row();
                    ui.label(tr!("physics-bounce"));
                    ui.add(egui::Slider::new(&mut s.restitution, 0.0..=0.9).fixed_decimals(2));
                    ui.end_row();
                    ui.label(tr!("physics-gravity"));
                    ui.add(egui::DragValue::new(&mut s.gravity).range(0.1..=100.0).speed(0.1).suffix(" m/s²"));
                    ui.end_row();
                    ui.label(tr!("physics-time"));
                    ui.add(egui::DragValue::new(&mut s.max_seconds).range(1.0..=60.0).speed(0.1).suffix(" s"));
                    ui.end_row();
                });
                ui.checkbox(&mut s.floor, tr!("physics-floor"));
            });
            ui.separator();
            let button = |ui: &mut egui::Ui, label: String, id: &str, enabled: bool| {
                let r = ui.add_enabled(enabled, egui::Button::new(label));
                super::tray_lib::spot(id, r.rect.center());
                r.clicked()
            };
            match &self.physics.run {
                None => {
                    ui.label(if bodies == 0 { tr!("physics-no-bodies") } else { tr!("physics-bodies", n = bodies) });
                    run = button(ui, tr!("physics-run"), "physics:run", bodies > 0);
                }
                Some(r) => {
                    let time = r.frame.as_ref().map_or(0.0, |f| f.time);
                    let fell = r.frame.as_ref().map_or(0, |f| f.fell.iter().filter(|x| **x).count());
                    ui.label(match r.state() {
                        "running" => tr!("physics-running", time = format!("{time:.1}")),
                        "done" => tr!("physics-done", time = format!("{time:.1}")),
                        _ => tr!("physics-stopped", time = format!("{time:.1}")),
                    });
                    if fell > 0 {
                        ui.label(RichText::new(tr!("physics-fell", n = fell)).color(egui::Color32::from_rgb(0xC0, 0x60, 0x00)));
                    }
                    ui.horizontal(|ui| {
                        if r.finished {
                            apply = button(ui, tr!("physics-apply"), "physics:apply", true);
                            run = button(ui, tr!("physics-again"), "physics:run", true);
                        } else {
                            stop = button(ui, tr!("physics-stop"), "physics:stop", true);
                        }
                        cancel = button(ui, tr!("physics-cancel"), "physics:cancel", true);
                    });
                }
            }
            if let Some(m) = &self.physics.message {
                ui.label(RichText::new(m).small());
            }
        });
        if !open {
            self.physics.open = false;
            self.physics_cancel();
            return;
        }
        if stop {
            self.physics_stop();
        }
        if apply {
            self.physics_apply();
        }
        if cancel {
            self.physics_cancel();
        }
        if run {
            // again: from the start positions, with the same bodies
            if let Some(r) = self.physics.run.take() {
                r.stop.store(true, Ordering::Relaxed);
                self.doc.preview_change(|m| *m = r.before);
                self.doc.dirty = r.was_dirty;
            }
            self.start_physics(ctx);
        }
    }

    /// `physics=(state bodies=N fell=N time=T) physics_z=[…]`: the run and the bottom of each
    /// selected group / component (QA).
    pub(crate) fn physics_dump(&self) -> String {
        let (state, bodies, fell, time) = match &self.physics.run {
            None => ("idle", self.physics_bodies().len(), 0, 0.0),
            Some(r) => (
                r.state(),
                r.bodies.len(),
                r.frame.as_ref().map_or(0, |f| f.fell.iter().filter(|x| **x).count()),
                r.frame.as_ref().map_or(0.0, |f| f.time),
            ),
        };
        let z: Vec<String> = self
            .doc
            .selection
            .iter()
            .filter_map(|e| match e {
                varstotas_kernel::EntityRef::Instance(i) => self.doc.model.instance_world_bounds(&self.doc.active_path, *i),
                _ => None,
            })
            .map(|b| format!("{:.0}", b.min.z + 0.0))
            .collect();
        format!("physics=({state} bodies={bodies} fell={fell} time={time:.1}) physics_z=[{}]", z.join(","))
    }
}
