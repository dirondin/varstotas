//! Tools → Physics…: drop the selected groups / components under gravity onto the rest of the
//! model, or let a selected face fall as cloth and drape over it (`varstotas_physics`). The
//! simulation runs on a thread; its frames are shown live by writing into the model without an
//! undo step (`preview_change`): the bodies' transforms, or the cloth group's mesh (which takes
//! the source face's place).
//! Apply records one undo step from the model as it was before the run; Cancel restores it.
//! While a run is shown, tools and model edits are held: an edit (any action but the view's)
//! cancels the run first.

use crate::app::{Action, VarstotasApp};
use egui::{Context, RichText};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use varstotas_i18n::tr;
use std::time::{Duration, Instant};
use varstotas_kernel::{ContextPath, EntityRef, InstanceId, Mesh, Model};
use varstotas_physics::cloth::{ClothError, ClothSettings, ClothSim, ClothSource, cloth_grid, cloth_mesh, cloth_source, place_cloth, set_cloth_mesh};
use varstotas_physics::{DropFrame, DropSettings, DropShape, DropSim, drop_bodies, set_transforms};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PhysicsMode {
    #[default]
    Drop,
    Cloth,
}

#[derive(Default)]
pub struct PhysicsState {
    pub open: bool,
    pub mode: PhysicsMode,
    pub settings: DropSettings,
    pub cloth: ClothSettings,
    pub(crate) run: Option<PhysicsRun>,
    /// The last run's outcome (shown under the buttons).
    pub(crate) message: Option<String>,
}

/// A frame from the simulation thread.
enum Shot {
    Drop(DropFrame),
    /// The cloth as a mesh in the axes of the run's context.
    Cloth { mesh: Box<Mesh>, time: f64, done: bool },
}

pub(crate) struct PhysicsRun {
    path: ContextPath,
    /// Drop: the falling groups / components.
    bodies: Vec<InstanceId>,
    /// Cloth: its source, and the group that shows it once the first frame came.
    cloth: Option<ClothSource>,
    placed: Option<InstanceId>,
    /// The model before the run (Cancel restores it, Apply records it as the undo state).
    before: Model,
    was_dirty: bool,
    /// The selection before the run (a cloth run clears it: its face is gone while shown).
    selection: Vec<EntityRef>,
    /// The model version this run last wrote: anything else means the model was edited.
    version: u64,
    rx: Receiver<Result<Shot, String>>,
    stop: Arc<AtomicBool>,
    /// The newest frame: simulated time, at rest, and which bodies fell into the void (drop).
    time: f64,
    done: bool,
    fell: Vec<bool>,
    shown: bool,
    /// No more frames will come (done, stopped or failed).
    finished: bool,
}

impl PhysicsRun {
    fn state(&self) -> &'static str {
        match (self.finished, self.shown && self.done) {
            (false, _) => "running",
            (true, true) => "done",
            _ => "stopped",
        }
    }

    fn fell(&self) -> usize {
        self.fell.iter().filter(|x| **x).count()
    }
}

/// Simulated time per frame sent to the window (steps of 1/120 s).
const STEPS_PER_FRAME: usize = 4;
/// A cloth frame is turned into a mesh at most this often (a fine cloth takes ~20 ms).
const CLOTH_FRAME_EVERY: Duration = Duration::from_millis(30);

fn cloth_error(e: &ClothError) -> String {
    match e {
        ClothError::NoFace => tr!("physics-cloth-no-face"),
        ClothError::TooSmall => tr!("physics-cloth-too-small"),
        ClothError::TooFine(n) => tr!("physics-cloth-too-fine", n = *n),
    }
}

impl VarstotasApp {
    /// A run is shown (running or waiting for Apply / Cancel).
    pub(crate) fn physics_live(&self) -> bool {
        self.physics.run.is_some()
    }

    fn physics_bodies(&self) -> Vec<InstanceId> {
        drop_bodies(&self.doc.model, &self.doc.active_path, &self.doc.selection)
    }

    fn physics_cloth(&self) -> Option<ClothSource> {
        cloth_source(&self.doc.model, &self.doc.active_path, &self.doc.selection)
    }

    /// What a cloth run would start from: its particle count, or why it cannot start.
    fn cloth_check(&self) -> Result<usize, String> {
        let src = self.physics_cloth().ok_or_else(|| cloth_error(&ClothError::NoFace))?;
        cloth_grid(&src, self.physics.cloth.spacing).map(|g| g.positions.len()).map_err(|e| cloth_error(&e))
    }

    pub(crate) fn start_physics(&mut self, ctx: &Context) {
        self.physics_cancel();
        let path = self.doc.active_path.clone();
        let model = self.doc.model.clone();
        let (tx, rx) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let (mut bodies, mut cloth) = (Vec::new(), None);
        let (t_path, t_stop, t_ctx) = (path.clone(), stop.clone(), ctx.clone());
        let (err_tx, err_ctx) = (tx.clone(), ctx.clone());
        let send = move |shot: Shot| {
            let ok = tx.send(Ok(shot)).is_ok();
            t_ctx.request_repaint();
            ok
        };
        let work: Box<dyn FnOnce() -> Result<(), String> + Send> = match self.physics.mode {
            PhysicsMode::Drop => {
                bodies = self.physics_bodies();
                if bodies.is_empty() {
                    self.physics.message = Some(tr!("physics-no-bodies"));
                    return;
                }
                let (t_bodies, settings) = (bodies.clone(), self.physics.settings.clone());
                Box::new(move || {
                    let mut sim = DropSim::new(&model, &t_path, &t_bodies, settings).map_err(|e| e.to_string())?;
                    loop {
                        let frame = sim.step(STEPS_PER_FRAME);
                        let done = frame.done;
                        if !send(Shot::Drop(frame)) || done || t_stop.load(Ordering::Relaxed) {
                            return Ok(());
                        }
                    }
                })
            }
            PhysicsMode::Cloth => {
                let settings = self.physics.cloth.clone();
                let src = match self.physics_cloth().ok_or(ClothError::NoFace).and_then(|s| cloth_grid(&s, settings.spacing).map(|_| s)) {
                    Ok(src) => src,
                    Err(e) => {
                        self.physics.message = Some(cloth_error(&e));
                        return;
                    }
                };
                cloth = Some(src.clone());
                Box::new(move || {
                    let mut sim = ClothSim::new(&model, &src, settings).map_err(|e| e.to_string())?;
                    let parent = model.transform_at(&t_path);
                    let mut sent: Option<Instant> = None;
                    loop {
                        let f = sim.step(STEPS_PER_FRAME);
                        let last = f.done || t_stop.load(Ordering::Relaxed);
                        if last || sent.is_none_or(|t| t.elapsed() >= CLOTH_FRAME_EVERY) {
                            sent = Some(Instant::now());
                            let mesh = cloth_mesh(&sim.grid, &f.positions, parent);
                            if !send(Shot::Cloth { mesh: Box::new(mesh), time: f.time, done: f.done }) {
                                return Ok(());
                            }
                        }
                        if last {
                            return Ok(());
                        }
                    }
                })
            }
        };
        std::thread::spawn(move || {
            if let Err(e) = crate::document::guarded(work).and_then(|r| r) {
                let _ = err_tx.send(Err(e));
            }
            err_ctx.request_repaint();
        });
        let selection = self.doc.selection.clone();
        if cloth.is_some() {
            // the face is replaced while the cloth is shown
            self.doc.selection.clear();
            self.doc.selection_version += 1;
        }
        self.physics.message = None;
        self.physics.run = Some(PhysicsRun {
            path,
            bodies,
            cloth,
            placed: None,
            before: self.doc.model.clone(),
            was_dirty: self.doc.dirty,
            selection,
            version: self.doc.model_version,
            rx,
            stop,
            time: 0.0,
            done: false,
            fell: Vec::new(),
            shown: false,
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
                Ok(Ok(shot)) => newest = Some(shot),
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
        let Some(shot) = newest else { return };
        let path = run.path.clone();
        match shot {
            Shot::Drop(frame) => {
                let bodies = run.bodies.clone();
                self.doc.preview_change(|m| set_transforms(m, &path, &bodies, &frame.transforms));
                (run.time, run.done, run.fell) = (frame.time, frame.done, frame.fell);
            }
            Shot::Cloth { mesh, time, done } => {
                match (run.placed, &run.cloth) {
                    (Some(id), _) => self.doc.preview_change(|m| set_cloth_mesh(m, &path, id, *mesh)),
                    (None, Some(src)) => {
                        let mut placed = None;
                        self.doc.preview_change(|m| placed = place_cloth(m, &path, src, *mesh));
                        run.placed = placed;
                    }
                    (None, None) => {}
                }
                (run.time, run.done) = (time, done);
            }
        }
        run.shown = true;
        run.version = self.doc.model_version;
        run.finished |= run.done;
    }

    pub(crate) fn physics_stop(&mut self) {
        if let Some(run) = &self.physics.run {
            run.stop.store(true, Ordering::Relaxed);
        }
    }

    /// Keep the shown result as one undo step.
    pub(crate) fn physics_apply(&mut self) {
        let Some(run) = self.physics.run.take() else { return };
        run.stop.store(true, Ordering::Relaxed);
        if let Some(id) = run.placed {
            self.doc.record_step(&tr!("physics-cloth-undo"), run.before);
            self.doc.selection = vec![EntityRef::Instance(id)];
            self.doc.selection_version += 1;
            self.physics.message = Some(tr!("physics-cloth-applied"));
            return;
        }
        let moved = if run.shown { run.fell.iter().filter(|x| !**x).count() } else { 0 };
        if moved == 0 {
            self.physics_restore(run);
            return;
        }
        self.doc.record_step(&tr!("physics-undo"), run.before);
        self.physics.message = Some(tr!("physics-applied", n = moved));
    }

    /// Put everything back where it was before the run.
    pub(crate) fn physics_cancel(&mut self) {
        if let Some(run) = self.physics.run.take() {
            run.stop.store(true, Ordering::Relaxed);
            self.physics_restore(run);
        }
    }

    fn physics_restore(&mut self, run: PhysicsRun) {
        self.doc.preview_change(|m| *m = run.before);
        self.doc.dirty = run.was_dirty;
        if run.cloth.is_some() {
            self.doc.selection = run.selection;
            self.doc.selection_version += 1;
        }
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
        // what a run would start from (not while one is shown: the selection is the run's)
        let ready: Result<String, String> = match (self.physics_live(), self.physics.mode) {
            (true, _) => Ok(String::new()),
            (false, PhysicsMode::Drop) => match self.physics_bodies().len() {
                0 => Err(tr!("physics-no-bodies")),
                n => Ok(tr!("physics-bodies", n = n)),
            },
            (false, PhysicsMode::Cloth) => self.cloth_check().map(|n| tr!("physics-cloth-particles", n = n)),
        };
        let (mut run, mut stop, mut apply, mut cancel) = (false, false, false, false);
        win.show(ctx, |ui| {
            ui.set_width(340.0);
            let live = self.physics.run.is_some();
            let button = |ui: &mut egui::Ui, label: String, id: &str, enabled: bool| {
                let r = ui.add_enabled(enabled, egui::Button::new(label));
                super::tray_lib::spot(id, r.rect.center());
                r.clicked()
            };
            ui.add_enabled_ui(!live, |ui| {
                ui.horizontal(|ui| {
                    for (mode, label, id) in [(PhysicsMode::Drop, tr!("physics-drop"), "physics:mode-drop"), (PhysicsMode::Cloth, tr!("physics-cloth"), "physics:mode-cloth")] {
                        let r = ui.selectable_label(self.physics.mode == mode, RichText::new(label).strong());
                        super::tray_lib::spot(id, r.rect.center());
                        if r.clicked() {
                            self.physics.mode = mode;
                            self.physics.message = None;
                        }
                    }
                });
                let hint = match self.physics.mode {
                    PhysicsMode::Drop => tr!("physics-drop-hint"),
                    PhysicsMode::Cloth => tr!("physics-cloth-hint"),
                };
                ui.label(RichText::new(hint).small().weak());
                ui.add_space(4.0);
                match self.physics.mode {
                    PhysicsMode::Drop => self.drop_options(ui),
                    PhysicsMode::Cloth => self.cloth_options(ui),
                }
            });
            ui.separator();
            match &self.physics.run {
                None => {
                    match &ready {
                        Ok(t) => ui.label(t),
                        Err(t) => ui.label(t),
                    };
                    run = button(ui, tr!("physics-run"), "physics:run", ready.is_ok());
                }
                Some(r) => {
                    let time = format!("{:.1}", r.time);
                    ui.label(match r.state() {
                        "running" => tr!("physics-running", time = time),
                        "done" => tr!("physics-done", time = time),
                        _ => tr!("physics-stopped", time = time),
                    });
                    if r.fell() > 0 {
                        ui.label(RichText::new(tr!("physics-fell", n = r.fell())).color(egui::Color32::from_rgb(0xC0, 0x60, 0x00)));
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
            // again: from the start, with the same selection (start cancels the shown run)
            self.start_physics(ctx);
        }
    }

    fn drop_options(&mut self, ui: &mut egui::Ui) {
        let s = &mut self.physics.settings;
        egui::Grid::new("physics_opts").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
            ui.label(tr!("physics-shape"));
            ui.horizontal(|ui| {
                ui.selectable_value(&mut s.shape, DropShape::Hull, tr!("physics-shape-hull")).on_hover_text(tr!("physics-shape-hull-hint"));
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
    }

    fn cloth_options(&mut self, ui: &mut egui::Ui) {
        let s = &mut self.physics.cloth;
        egui::Grid::new("physics_cloth_opts").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
            ui.label(tr!("physics-cloth-spacing")).on_hover_text(tr!("physics-cloth-spacing-hint"));
            ui.add(egui::DragValue::new(&mut s.spacing).range(1.0..=500.0).speed(0.2).suffix(" mm"));
            ui.end_row();
            ui.label(tr!("physics-cloth-stiffness")).on_hover_text(tr!("physics-cloth-stiffness-hint"));
            ui.add(egui::Slider::new(&mut s.stiffness, 0.0..=1.0).fixed_decimals(2));
            ui.end_row();
            ui.label(tr!("physics-cloth-bending")).on_hover_text(tr!("physics-cloth-bending-hint"));
            ui.add(egui::Slider::new(&mut s.bending, 0.0..=1.0).fixed_decimals(2));
            ui.end_row();
            ui.label(tr!("physics-cloth-thickness")).on_hover_text(tr!("physics-cloth-thickness-hint"));
            ui.add(egui::DragValue::new(&mut s.thickness).range(0.1..=50.0).speed(0.05).suffix(" mm"));
            ui.end_row();
            ui.label(tr!("physics-friction"));
            ui.add(egui::Slider::new(&mut s.friction, 0.0..=1.5).fixed_decimals(2));
            ui.end_row();
            ui.label(tr!("physics-gravity"));
            ui.add(egui::DragValue::new(&mut s.gravity).range(0.1..=100.0).speed(0.1).suffix(" m/s²"));
            ui.end_row();
            ui.label(tr!("physics-time"));
            ui.add(egui::DragValue::new(&mut s.max_seconds).range(1.0..=60.0).speed(0.1).suffix(" s"));
            ui.end_row();
        });
        ui.checkbox(&mut s.floor, tr!("physics-floor"));
        ui.checkbox(&mut s.self_collision, tr!("physics-cloth-self")).on_hover_text(tr!("physics-cloth-self-hint"));
    }

    /// `physics=(state bodies=N fell=N time=T) physics_z=[…]` for a drop: the run and the bottom
    /// of each selected group / component; `physics=(state cloth=N time=T) cloth_z=[lo,hi]` for
    /// a cloth: its particles (or why it cannot start) and the height of the shown cloth (QA).
    pub(crate) fn physics_dump(&self) -> String {
        let run = self.physics.run.as_ref();
        let state = run.map_or("idle", |r| r.state());
        let time = run.map_or(0.0, |r| r.time);
        if self.physics.mode == PhysicsMode::Cloth {
            let what = match run {
                Some(r) => r.placed.map_or("none".into(), |_| "shown".into()),
                None => self.cloth_check().map_or_else(|e| format!("\"{e}\""), |n| n.to_string()),
            };
            let cloth = run.and_then(|r| r.placed).or(match self.doc.selection.as_slice() {
                [EntityRef::Instance(i)] => Some(*i),
                _ => None,
            });
            let z = cloth
                .and_then(|i| self.doc.model.instance_world_bounds(&self.doc.active_path, i))
                .map_or(String::new(), |b| format!("{:.0},{:.0}", b.min.z + 0.0, b.max.z + 0.0));
            return format!("physics=({state} cloth={what} time={time:.1}) cloth_z=[{z}]");
        }
        let (bodies, fell) = match run {
            None => (self.physics_bodies().len(), 0),
            Some(r) => (r.bodies.len(), r.fell()),
        };
        let z: Vec<String> = self
            .doc
            .selection
            .iter()
            .filter_map(|e| match e {
                EntityRef::Instance(i) => self.doc.model.instance_world_bounds(&self.doc.active_path, *i),
                _ => None,
            })
            .map(|b| format!("{:.0}", b.min.z + 0.0))
            .collect();
        format!("physics=({state} bodies={bodies} fell={fell} time={time:.1}) physics_z=[{}]", z.join(","))
    }
}
