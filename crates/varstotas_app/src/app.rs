//! `VarstotasApp`: window layout, actions, input routing, file handling.

use crate::document::{Document, guarded};
use crate::nav::{self, CameraAnimation};
use crate::overlay::Marker2d;
use crate::tools::{self, OverlayView, PointerInfo, Tool, ToolContext, ToolId, ToolRequest};
use crate::ui::tray::TrayState;
use crate::viewport::Viewport;
use egui::{Key, KeyboardShortcut, Modifiers, PointerButton, Pos2, Response, ViewportCommand};
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{ContextPath, EntityRef, InstanceId, Model, Scene, io};
use varstotas_render::{Camera, FaceStyle, Overlay3d, RenderStyle, StandardView};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const PREFS_KEY: &str = "varstotas_prefs";
const MAX_RECENT: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FaceStylePref {
    Shaded,
    Monochrome,
    HiddenLine,
    Wireframe,
}

impl FaceStylePref {
    pub fn to_render(self) -> FaceStyle {
        match self {
            Self::Shaded => FaceStyle::Shaded,
            Self::Monochrome => FaceStyle::Monochrome,
            Self::HiddenLine => FaceStyle::HiddenLine,
            Self::Wireframe => FaceStyle::Wireframe,
        }
    }
}

/// UI preferences persisted between runs.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub tray_open: bool,
    pub face_style: FaceStylePref,
    pub xray: bool,
    pub show_edges: bool,
    pub profiles: bool,
    pub show_axes: bool,
    pub show_ground: bool,
    pub recent: Vec<PathBuf>,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            tray_open: true,
            face_style: FaceStylePref::Shaded,
            xray: false,
            show_edges: true,
            profiles: true,
            show_axes: true,
            show_ground: true,
            recent: Vec::new(),
        }
    }
}

/// Everything the menus, toolbar, tray and hotkeys can ask the app to do.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    New,
    Open,
    OpenPath(PathBuf),
    Save,
    SaveAs,
    ImportObj,
    ImportStl,
    ExportObj,
    ExportStl,
    InsertDemo,
    Quit,
    Undo,
    Redo,
    SelectAll,
    SelectNone,
    Delete,
    MakeGroup,
    MakeComponent,
    Explode,
    HideSelected,
    UnhideAll,
    ExitContext,
    SetTool(ToolId),
    View(StandardView),
    SetParallel(bool),
    ZoomExtents,
    ToggleTray,
    SelectInstance(ContextPath, InstanceId),
    ToggleInstanceHidden(ContextPath, InstanceId),
    AddScene,
    UpdateScene(usize),
    DeleteScene(usize),
    ActivateScene(usize),
    About,
}

/// Something waiting for "save changes?" confirmation.
#[derive(Clone, Debug, PartialEq)]
pub enum Pending {
    New,
    Open,
    OpenPath(PathBuf),
    Quit,
}

pub struct VarstotasApp {
    pub(crate) doc: Document,
    pub(crate) camera: Camera,
    pub(crate) cam_anim: Option<CameraAnimation>,
    pub(crate) viewport: Viewport,
    pub(crate) tool: Box<dyn Tool>,
    pub(crate) vcb: String,
    pub(crate) vcb_error: bool,
    pub(crate) prefs: Prefs,
    pub(crate) base_style: RenderStyle,
    pub(crate) actions: Vec<Action>,
    pub(crate) confirm: Option<Pending>,
    pub(crate) show_about: bool,
    /// Transient status message (text, expiry time).
    pub(crate) flash: Option<(String, f64)>,
    pub(crate) tray: TrayState,
    pub(crate) debug: crate::debug_capture::DebugCapture,
    last_title: String,
    allow_close: bool,
    pub(crate) tool_pressed: bool,
    pub(crate) last_pointer: Option<Pos2>,
    pub(crate) last_modifiers: Modifiers,
    now: f64,
}

fn fallback_style() -> RenderStyle {
    RenderStyle {
        face_style: FaceStyle::Shaded,
        xray: false,
        show_edges: true,
        profiles: true,
        profile_width_px: 2.5,
        show_axes: true,
        show_ground: true,
        sky_top: [0xA8 as f32 / 255.0, 0xC8 as f32 / 255.0, 0xE8 as f32 / 255.0, 1.0],
        sky_horizon: [0xE8 as f32 / 255.0, 0xF0 as f32 / 255.0, 0xF8 as f32 / 255.0, 1.0],
        ground: [0xC8 as f32 / 255.0, 0xC0 as f32 / 255.0, 0xB0 as f32 / 255.0, 1.0],
        edge_color: [0.0, 0.0, 0.0, 1.0],
        light_dir: varstotas_render::DEFAULT_LIGHT_DIR,
        ambient: varstotas_render::DEFAULT_AMBIENT,
    }
}

impl VarstotasApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        crate::ui::theme::apply(&cc.egui_ctx);
        let prefs: Prefs = cc
            .storage
            .and_then(|s| eframe::get_value(s, PREFS_KEY))
            .unwrap_or_default();
        let base_style = guarded(RenderStyle::default).unwrap_or_else(|_| fallback_style());
        let mut actions = Vec::new();
        if crate::debug_capture::DebugCapture::demo_requested() {
            actions.push(Action::InsertDemo);
        }
        Self {
            doc: Document::new(),
            camera: Camera::default(),
            cam_anim: None,
            viewport: Viewport::default(),
            tool: tools::make_tool(ToolId::Select),
            vcb: String::new(),
            vcb_error: false,
            prefs,
            base_style,
            actions,
            confirm: None,
            show_about: false,
            flash: None,
            tray: TrayState::default(),
            debug: crate::debug_capture::DebugCapture::from_env(),
            last_title: String::new(),
            allow_close: false,
            tool_pressed: false,
            last_pointer: None,
            last_modifiers: Modifiers::NONE,
            now: 0.0,
        }
    }

    pub fn render_style(&self) -> RenderStyle {
        let mut s = self.base_style.clone();
        s.face_style = self.prefs.face_style.to_render();
        s.xray = self.prefs.xray;
        s.show_edges = self.prefs.show_edges;
        s.profiles = self.prefs.profiles;
        s.show_axes = self.prefs.show_axes;
        s.show_ground = self.prefs.show_ground;
        s
    }

    pub fn tool_id(&self) -> ToolId {
        self.tool.id()
    }

    pub(crate) fn flash(&mut self, msg: impl Into<String>) {
        self.flash = Some((msg.into(), self.now + 4.0));
    }

    fn aspect(&self) -> f64 {
        let s = self.viewport.size();
        (s.x / s.y.max(1.0)) as f64
    }

    // ---- tools -----------------------------------------------------------

    pub(crate) fn pointer_info(&self, pos: Pos2, delta: egui::Vec2, button: Option<PointerButton>, primary_down: bool) -> PointerInfo {
        let vp = self.viewport.size();
        let ray = guarded(|| nav::ray_at(&self.camera, pos, vp)).unwrap_or(varstotas_kernel::Ray {
            origin: self.camera.eye,
            dir: (self.camera.target - self.camera.eye).normalize_or(DVec3::NEG_Z),
        });
        PointerInfo {
            pos,
            delta,
            ray,
            modifiers: self.last_modifiers,
            button,
            primary_down,
            press_origin: None,
            click_count: 1,
        }
    }

    /// Run `f` with the active tool and a fresh `ToolContext`.
    pub(crate) fn with_tool<R>(&mut self, pointer: PointerInfo, f: impl FnOnce(&mut dyn Tool, &mut ToolContext) -> R) -> R {
        let mut requests = Vec::new();
        let viewport = self.viewport.size();
        let r = {
            let mut ctx = ToolContext {
                doc: &mut self.doc,
                camera: &mut self.camera,
                viewport,
                pointer,
                time: self.now,
                requests: &mut requests,
            };
            f(self.tool.as_mut(), &mut ctx)
        };
        for req in requests {
            match req {
                ToolRequest::SwitchTool(id) => self.actions.push(Action::SetTool(id)),
                ToolRequest::ZoomExtents => self.actions.push(Action::ZoomExtents),
            }
        }
        r
    }

    fn current_pointer(&self) -> PointerInfo {
        let pos = self.last_pointer.unwrap_or((self.viewport.size() * 0.5).to_pos2());
        self.pointer_info(pos, egui::Vec2::ZERO, None, false)
    }

    pub(crate) fn set_tool(&mut self, id: ToolId) {
        let p = self.current_pointer();
        self.with_tool(p.clone(), |t, ctx| t.deactivate(ctx));
        self.tool = tools::make_tool(id);
        self.with_tool(p, |t, ctx| t.activate(ctx));
        self.vcb.clear();
        self.vcb_error = false;
        self.tool_pressed = false;
    }

    pub(crate) fn submit_vcb(&mut self) {
        let input = std::mem::take(&mut self.vcb);
        let p = self.current_pointer();
        let ok = self.with_tool(p.clone(), |t, ctx| t.on_vcb(ctx, &input));
        if !ok {
            self.vcb_error = true;
            self.flash(format!("Invalid value: {input}"));
        } else {
            self.vcb_error = false;
            // Refresh hover/inference so no stale pre-commit overlay lingers.
            self.with_tool(p, |t, ctx| t.on_pointer_move(ctx));
        }
    }

    // ---- keyboard ---------------------------------------------------------

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        if ctx.text_edit_focused() || self.confirm.is_some() {
            return;
        }
        let cmd = Modifiers::COMMAND;
        let cmd_shift = Modifiers::COMMAND | Modifiers::SHIFT;
        let shortcuts: [(Modifiers, Key, Action); 10] = [
            (cmd_shift, Key::Z, Action::Redo),
            (cmd, Key::Z, Action::Undo),
            (cmd, Key::Y, Action::Redo),
            (cmd_shift, Key::S, Action::SaveAs),
            (cmd, Key::S, Action::Save),
            (cmd, Key::O, Action::Open),
            (cmd, Key::N, Action::New),
            (cmd, Key::A, Action::SelectAll),
            (cmd, Key::T, Action::SelectNone),
            (cmd, Key::Q, Action::Quit),
        ];
        for (m, k, a) in shortcuts {
            if ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(m, k))) {
                self.actions.push(a);
            }
        }

        let events = ctx.input(|i| i.events.clone());
        for ev in events {
            match ev {
                egui::Event::Key { key, pressed, modifiers, .. } => {
                    self.last_modifiers = modifiers;
                    if modifiers.command || modifiers.ctrl || modifiers.alt {
                        if !pressed {
                            let p = self.current_pointer();
                            self.with_tool(p, |t, c| t.on_key(c, key, false));
                        }
                        continue;
                    }
                    if !pressed {
                        let p = self.current_pointer();
                        self.with_tool(p, |t, c| t.on_key(c, key, false));
                        continue;
                    }
                    let typing = !self.vcb.is_empty();
                    match key {
                        Key::Backspace if typing => {
                            self.vcb.pop();
                        }
                        Key::Backspace | Key::Delete => self.actions.push(Action::Delete),
                        Key::Enter if typing => self.submit_vcb(),
                        Key::Escape if typing => {
                            self.vcb.clear();
                            self.vcb_error = false;
                        }
                        Key::Z if modifiers.shift && !typing => self.actions.push(Action::ZoomExtents),
                        Key::G if !typing && !modifiers.shift => self.actions.push(Action::MakeGroup),
                        _ => {
                            // Letters while typing in the VCB go to the VCB (via Text events).
                            let is_letter = key.name().len() == 1 && key.name().chars().all(|c| c.is_ascii_alphabetic());
                            if typing && (is_letter || key == Key::Space) {
                                continue;
                            }
                            if !modifiers.shift
                                && let Some(id) = ToolId::ALL.iter().copied().find(|t| t.shortcut() == Some(key)) {
                                    self.actions.push(Action::SetTool(id));
                                    continue;
                                }
                            let p = self.current_pointer();
                            self.with_tool(p, |t, c| t.on_key(c, key, true));
                        }
                    }
                }
                egui::Event::Text(text) => {
                    for ch in text.chars() {
                        let starts = ch.is_ascii_digit() || "-.,;'\"/<~".contains(ch);
                        let cont = !self.vcb.is_empty() && (ch.is_alphabetic() || ch == ' ' || ch == '*');
                        if starts || cont {
                            if self.vcb_error {
                                self.vcb.clear();
                                self.vcb_error = false;
                            }
                            self.vcb.push(ch);
                        }
                    }
                }
                egui::Event::ModifiersChanged(m) => self.last_modifiers = m,
                _ => {}
            }
        }
    }

    // ---- viewport input ---------------------------------------------------

    fn handle_viewport_input(&mut self, ctx: &egui::Context, response: &Response) {
        let rect = self.viewport.rect;
        let user_nav = self.viewport.nav.handle(response, rect, &self.doc.model, &mut self.camera);
        if user_nav {
            self.cam_anim = None;
        }
        let over = response.contains_pointer();
        let (events, modifiers, primary_down) =
            ctx.input(|i| (i.events.clone(), i.modifiers, i.pointer.primary_down()));
        self.last_modifiers = modifiers;
        if self.confirm.is_some() {
            return;
        }
        for ev in events {
            match ev {
                egui::Event::PointerMoved(p) => {
                    if over || self.tool_pressed {
                        let local = (p - rect.min).to_pos2();
                        let delta = self.last_pointer.map(|l| local - l).unwrap_or_default();
                        self.last_pointer = Some(local);
                        let pi = self.pointer_info(local, delta, None, primary_down);
                        self.with_tool(pi, |t, c| t.on_pointer_move(c));
                    }
                }
                egui::Event::PointerButton { pos, button, pressed, modifiers } => {
                    if button == PointerButton::Middle {
                        continue;
                    }
                    self.last_modifiers = modifiers;
                    let local = (pos - rect.min).to_pos2();
                    if pressed && over && rect.contains(pos) {
                        self.tool_pressed = true;
                        self.last_pointer = Some(local);
                        let pi = self.pointer_info(local, egui::Vec2::ZERO, Some(button), button == PointerButton::Primary);
                        self.with_tool(pi, |t, c| t.on_pointer_down(c));
                    } else if !pressed && self.tool_pressed {
                        self.tool_pressed = false;
                        let pi = self.pointer_info(local, egui::Vec2::ZERO, Some(button), false);
                        self.with_tool(pi, |t, c| t.on_pointer_up(c));
                    }
                }
                _ => {}
            }
        }
        let clicks = if response.triple_clicked() {
            3
        } else if response.double_clicked() {
            2
        } else {
            0
        };
        if clicks > 0
            && let Some(local) = self.last_pointer {
                let mut pi = self.pointer_info(local, egui::Vec2::ZERO, Some(PointerButton::Primary), false);
                pi.click_count = clicks;
                self.with_tool(pi, |t, c| t.on_double_click(c));
            }
        if over {
            let icon = if self.viewport.nav.is_dragging() { egui::CursorIcon::Grabbing } else { self.tool.cursor() };
            ctx.set_cursor_icon(icon);
        } else if !self.tool_pressed && !self.debug.script_active() {
            self.last_pointer = None;
        }
    }

    // ---- actions ------------------------------------------------------------

    fn run_actions(&mut self, ctx: &egui::Context) {
        let actions = std::mem::take(&mut self.actions);
        for a in actions {
            self.run_action(ctx, a);
        }
    }

    fn run_action(&mut self, ctx: &egui::Context, action: Action) {
        match action {
            Action::New => self.guard_dirty(Pending::New),
            Action::Open => self.guard_dirty(Pending::Open),
            Action::OpenPath(p) => self.guard_dirty(Pending::OpenPath(p)),
            Action::Quit => {
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            Action::Save => {
                self.save();
            }
            Action::SaveAs => {
                self.save_as();
            }
            Action::ImportObj => self.import(&["obj"], "OBJ"),
            Action::ImportStl => self.import(&["stl"], "STL"),
            Action::ExportObj => self.export("obj"),
            Action::ExportStl => self.export("stl"),
            Action::InsertDemo => self.insert_demo(),
            Action::Undo => match self.doc.undo() {
                Some(l) => self.flash(format!("Undo {l}")),
                None => self.flash("Nothing to undo"),
            },
            Action::Redo => match self.doc.redo() {
                Some(l) => self.flash(format!("Redo {l}")),
                None => self.flash("Nothing to redo"),
            },
            Action::SelectAll => {
                self.doc.select_all();
            }
            Action::SelectNone => self.doc.clear_selection(),
            Action::Delete => self.doc.erase_selection(),
            Action::MakeGroup | Action::MakeComponent => {
                if self.doc.selection.is_empty() {
                    self.flash("Select something to group first");
                    return;
                }
                let sel = self.doc.selection.clone();
                let path = self.doc.active_path.clone();
                let r = if action == Action::MakeGroup {
                    self.doc.try_commit("Make Group", |m| m.make_group(&path, &sel))
                } else {
                    let n = self.doc.model.defs.len();
                    self.doc
                        .try_commit("Make Component", |m| m.make_component(&path, &sel, &format!("Component#{n}")))
                };
                if let Some(id) = r {
                    self.doc.set_selection(vec![EntityRef::Instance(id)]);
                }
            }
            Action::Explode => {
                let path = self.doc.active_path.clone();
                let insts: Vec<InstanceId> = self
                    .doc
                    .selection
                    .iter()
                    .filter_map(|e| if let EntityRef::Instance(i) = e { Some(*i) } else { None })
                    .collect();
                if insts.is_empty() {
                    self.flash("Select a group or component to explode");
                    return;
                }
                let r = self.doc.try_commit("Explode", |m| {
                    let mut out = Vec::new();
                    for i in insts {
                        out.extend(m.explode(&path, i)?);
                    }
                    Ok::<_, varstotas_kernel::OpError>(out)
                });
                if let Some(ents) = r {
                    self.doc.set_selection(ents);
                }
            }
            Action::HideSelected => {
                let sel = self.doc.selection.clone();
                let path = self.doc.active_path.clone();
                if sel.is_empty() {
                    return;
                }
                self.doc.commit("Hide", |m| {
                    if let Some(mesh) = m.mesh_at_mut(&path) {
                        for e in sel {
                            match e {
                                EntityRef::Face(id) => {
                                    if let Some(f) = mesh.faces.get_mut(id) {
                                        f.hidden = true;
                                    }
                                }
                                EntityRef::Edge(id) => {
                                    if let Some(x) = mesh.edges.get_mut(id) {
                                        x.hidden = true;
                                    }
                                }
                                EntityRef::Instance(id) => {
                                    if let Some(x) = mesh.instances.get_mut(id) {
                                        x.hidden = true;
                                    }
                                }
                                EntityRef::Vertex(_) => {}
                            }
                        }
                    }
                });
                self.doc.clear_selection();
            }
            Action::UnhideAll => {
                self.doc.commit("Unhide All", |m| {
                    for (_, d) in m.defs.iter_mut() {
                        d.mesh.faces.values_mut().for_each(|f| f.hidden = false);
                        d.mesh.edges.values_mut().for_each(|e| e.hidden = false);
                        d.mesh.instances.values_mut().for_each(|i| i.hidden = false);
                    }
                });
            }
            Action::ExitContext => {
                self.doc.exit_context();
            }
            Action::SetTool(id) => {
                self.set_tool(id);
                if !id.implemented() {
                    self.flash(format!("{} is not implemented yet", id.name()));
                }
            }
            Action::View(v) => {
                let to = nav::standard_view_camera(&self.doc.model, &self.camera, v, self.aspect());
                self.animate_to(to);
            }
            Action::SetParallel(p) => {
                self.cam_anim = None;
                self.camera.set_ortho(p);
            }
            Action::ZoomExtents => {
                let to = nav::zoom_extents_camera(&self.doc.model, &self.camera, self.aspect());
                self.animate_to(to);
            }
            Action::ToggleTray => self.prefs.tray_open = !self.prefs.tray_open,
            Action::SelectInstance(path, id) => {
                self.doc.set_active_path(path);
                self.doc.set_selection(vec![EntityRef::Instance(id)]);
            }
            Action::ToggleInstanceHidden(path, id) => {
                self.doc.commit("Hide", |m| {
                    if let Some(i) = m.mesh_at_mut(&path).and_then(|mesh| mesh.instances.get_mut(id)) {
                        i.hidden = !i.hidden;
                    }
                });
            }
            Action::AddScene => {
                let n = self.doc.model.scenes.len() + 1;
                let scene = self.scene_from_camera(format!("Scene {n}"));
                self.doc.commit("Add Scene", |m| m.scenes.push(scene));
                self.tray.selected_scene = Some(self.doc.model.scenes.len().saturating_sub(1));
            }
            Action::UpdateScene(i) => {
                let name = self.doc.model.scenes.get(i).map(|s| s.name.clone()).unwrap_or_default();
                let scene = self.scene_from_camera(name);
                self.doc.commit("Update Scene", |m| {
                    if let Some(s) = m.scenes.get_mut(i) {
                        *s = scene;
                    }
                });
            }
            Action::DeleteScene(i) => {
                self.doc.commit("Delete Scene", |m| {
                    if i < m.scenes.len() {
                        m.scenes.remove(i);
                    }
                });
                self.tray.selected_scene = None;
            }
            Action::ActivateScene(i) => {
                if let Some(s) = self.doc.model.scenes.get(i).cloned() {
                    let mut to = self.camera.clone();
                    to.eye = s.eye;
                    to.target = s.target;
                    to.up = s.up;
                    to.fov_y_deg = s.fov_y_deg;
                    to.ortho = s.ortho;
                    if s.ortho {
                        let d = (s.eye - s.target).length();
                        to.ortho_height = 2.0 * d * (s.fov_y_deg.to_radians() * 0.5).tan();
                    }
                    self.tray.selected_scene = Some(i);
                    self.animate_to(to);
                }
            }
            Action::About => self.show_about = true,
        }
    }

    fn scene_from_camera(&self, name: String) -> Scene {
        Scene {
            name,
            eye: self.camera.eye,
            target: self.camera.target,
            up: self.camera.up,
            fov_y_deg: self.camera.fov_y_deg,
            ortho: self.camera.ortho,
        }
    }

    fn animate_to(&mut self, to: Camera) {
        self.cam_anim = Some(CameraAnimation::new(self.camera.clone(), to, 0.55));
    }

    // ---- files ----------------------------------------------------------------

    fn guard_dirty(&mut self, p: Pending) {
        if self.doc.dirty {
            self.confirm = Some(p);
        } else {
            self.proceed(p);
        }
    }

    /// Continue a pending operation after the user confirmed (or nothing was dirty).
    pub(crate) fn proceed(&mut self, p: Pending) {
        match p {
            Pending::New => self.replace_document(Document::new()),
            Pending::Open => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Varstotas model", &["vrst"])
                    .pick_file()
                {
                    self.open_path(&path);
                }
            }
            Pending::OpenPath(path) => self.open_path(&path),
            Pending::Quit => {
                self.allow_close = true;
                self.tray.close_requested = true;
            }
        }
    }

    fn replace_document(&mut self, doc: Document) {
        self.doc = doc;
        self.viewport.invalidate();
        self.tool = tools::make_tool(ToolId::Select);
        self.vcb.clear();
        self.tray = TrayState::default();
        let mut cam = guarded(Camera::default).unwrap_or_else(|_| self.camera.clone());
        if !self.doc.model.scenes.is_empty() {
            let s = &self.doc.model.scenes[0];
            cam.eye = s.eye;
            cam.target = s.target;
            cam.up = s.up;
            cam.fov_y_deg = s.fov_y_deg;
            cam.ortho = s.ortho;
            self.camera = cam;
        } else {
            self.camera = nav::zoom_extents_camera(&self.doc.model, &cam, self.aspect());
        }
        self.cam_anim = None;
    }

    fn open_path(&mut self, path: &Path) {
        match guarded(|| io::load(path)) {
            Ok(Ok(model)) => {
                self.replace_document(Document::from_model(model, Some(path.to_path_buf())));
                self.add_recent(path);
                self.flash(format!("Opened {}", path.display()));
            }
            Ok(Err(e)) => self.flash(format!("Could not open {}: {e}", path.display())),
            Err(e) => self.flash(format!("Could not open {}: {e}", path.display())),
        }
    }

    fn add_recent(&mut self, path: &Path) {
        self.prefs.recent.retain(|p| p != path);
        self.prefs.recent.insert(0, path.to_path_buf());
        self.prefs.recent.truncate(MAX_RECENT);
    }

    /// Returns true if saved.
    pub(crate) fn save(&mut self) -> bool {
        match self.doc.path.clone() {
            Some(p) => self.save_to(&p),
            None => self.save_as(),
        }
    }

    fn save_as(&mut self) -> bool {
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter("Varstotas model", &["vrst"])
            .set_file_name(format!("{}.vrst", self.doc.display_name()))
            .save_file()
        else {
            return false;
        };
        if path.extension().is_none() {
            path.set_extension("vrst");
        }
        self.save_to(&path)
    }

    fn save_to(&mut self, path: &Path) -> bool {
        match guarded(|| io::save(&self.doc.model, path)) {
            Ok(Ok(())) => {
                self.doc.path = Some(path.to_path_buf());
                self.doc.dirty = false;
                self.add_recent(path);
                self.flash(format!("Saved {}", path.display()));
                true
            }
            Ok(Err(e)) => {
                self.flash(format!("Save failed: {e}"));
                false
            }
            Err(e) => {
                self.flash(format!("Save failed: {e}"));
                false
            }
        }
    }

    fn export(&mut self, ext: &str) {
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter(ext.to_uppercase(), &[ext])
            .set_file_name(format!("{}.{ext}", self.doc.display_name()))
            .save_file()
        else {
            return;
        };
        if path.extension().is_none() {
            path.set_extension(ext);
        }
        let model = &self.doc.model;
        let r = guarded(|| if ext == "obj" { io::export_obj(model, &path) } else { io::export_stl(model, &path) });
        match r {
            Ok(Ok(())) => self.flash(format!("Exported {}", path.display())),
            Ok(Err(e)) => self.flash(format!("Export failed: {e}")),
            Err(e) => self.flash(format!("Export failed: {e}")),
        }
    }

    fn import(&mut self, exts: &[&str], label: &str) {
        let Some(path) = rfd::FileDialog::new().add_filter(label, exts).pick_file() else {
            return;
        };
        let is_obj = exts.contains(&"obj");
        let r = self.doc.try_commit(&format!("Import {label}"), |m| {
            if is_obj { io::import_obj(m, &path) } else { io::import_stl(m, &path) }
        });
        if r.is_some() {
            self.flash(format!("Imported {}", path.display()));
            self.actions.push(Action::ZoomExtents);
        } else if let Some(e) = self.doc.last_error.clone() {
            self.flash(e);
        }
    }

    fn insert_demo(&mut self) {
        fn make_box(mesh: &mut varstotas_kernel::Mesh, x: f64, y: f64, w: f64, d: f64, h: f64) {
            let pts = [
                DVec3::new(x, y, 0.0),
                DVec3::new(x + w, y, 0.0),
                DVec3::new(x + w, y + d, 0.0),
                DVec3::new(x, y + d, 0.0),
            ];
            if let Some(f) = mesh.add_face(&pts) {
                let up = mesh.faces.get(f).map(|f| f.normal.z >= 0.0).unwrap_or(true);
                let _ = mesh.push_pull(f, if up { h } else { -h }, false);
            }
        }
        let r = self.doc.commit("Insert Demo Geometry", |m: &mut Model| {
            let brick = m.add_material("Brick", [0.70, 0.33, 0.24, 1.0]);
            let wood = m.add_material("Wood", [0.78, 0.60, 0.40, 1.0]);
            let mesh = m.root_mesh_mut();
            make_box(mesh, 0.0, 0.0, 3000.0, 2000.0, 2400.0);
            make_box(mesh, 4000.0, 0.0, 1000.0, 1000.0, 3200.0);
            // A painted group.
            let before: std::collections::HashSet<EntityRef> = all_refs(m.root_mesh());
            make_box(m.root_mesh_mut(), 0.0, 3000.0, 1800.0, 900.0, 750.0);
            let new: Vec<EntityRef> = all_refs(m.root_mesh()).difference(&before).copied().collect();
            if let Ok(g) = m.make_group(&[], &new)
                && let Some(i) = m.root_mesh_mut().instances.get_mut(g) {
                    i.name = "Table".into();
                    i.material = Some(wood);
                }
            // Paint the tall box's faces.
            let mesh = m.root_mesh_mut();
            let ids: Vec<_> = mesh.faces.iter().filter(|(_, f)| {
                f.outer.iter().all(|v| mesh.vertices[*v].pos.x >= 4000.0 - 1e-6)
            }).map(|(id, _)| id).collect();
            for id in ids {
                if let Some(f) = mesh.faces.get_mut(id) {
                    f.front = Some(brick);
                }
            }
        });
        if r.is_some() {
            self.actions.push(Action::ZoomExtents);
        } else if let Some(e) = self.doc.last_error.clone() {
            self.flash(e);
        }
    }

    fn update_title(&mut self, ctx: &egui::Context) {
        let title = format!("{}{} — Varstotas", self.doc.display_name(), if self.doc.dirty { "*" } else { "" });
        if title != self.last_title {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
            self.last_title = title;
        }
    }

    fn handle_close_request(&mut self, ctx: &egui::Context) {
        if self.tray.close_requested {
            self.tray.close_requested = false;
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        if ctx.input(|i| i.viewport().close_requested()) && self.doc.dirty && !self.allow_close && !self.debug.force_close {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.confirm = Some(Pending::Quit);
        }
    }

    /// Collect overlays from the active tool.
    fn tool_overlays(&self) -> (Overlay3d, Vec<Marker2d>) {
        let mut o3 = Overlay3d::default();
        let mut o2 = Vec::new();
        let view = OverlayView {
            doc: &self.doc,
            camera: &self.camera,
            viewport: self.viewport.size(),
            pointer: self.last_pointer,
        };
        tools::common::draw_guides(&view, &mut o3, &mut o2);
        self.tool.overlay(&view, &mut o3, &mut o2);
        (o3, o2)
    }

    /// Status-bar text for the left side.
    pub(crate) fn status_text(&self) -> (String, bool) {
        if let Some((msg, until)) = &self.flash
            && *until > self.now {
                return (msg.clone(), false);
            }
        if let Some(e) = &self.doc.last_error {
            return (e.clone(), true);
        }
        (self.tool.status_text(), false)
    }
}

fn all_refs(mesh: &varstotas_kernel::Mesh) -> std::collections::HashSet<EntityRef> {
    let mut s = std::collections::HashSet::new();
    s.extend(mesh.faces.keys().map(EntityRef::Face));
    s.extend(mesh.edges.keys().map(EntityRef::Edge));
    s
}

impl eframe::App for VarstotasApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.now = ctx.input(|i| i.time);
        let dt = ctx.input(|i| i.stable_dt).min(0.1);

        self.handle_close_request(&ctx);
        self.handle_keyboard(&ctx);

        egui::Panel::top("menu_bar")
            .frame(crate::ui::theme::bar_frame(&ctx))
            .show(ui, |ui| self.menu_bar(ui));
        egui::Panel::top("toolbar")
            .frame(crate::ui::theme::toolbar_frame(&ctx))
            .show(ui, |ui| self.toolbar(ui));
        egui::Panel::bottom("status_bar")
            .frame(crate::ui::theme::bar_frame(&ctx))
            .show(ui, |ui| self.status_bar(ui));
        let mut tray_open = self.prefs.tray_open;
        egui::Panel::right("default_tray")
            .resizable(true)
            .default_size(290.0)
            .size_range(220.0..=480.0)
            .frame(crate::ui::theme::tray_frame(&ctx))
            .show_collapsible(ui, &mut tray_open, |ui| self.tray_ui(ui));
        self.prefs.tray_open = tray_open;

        egui::CentralPanel::no_frame().show(ui, |ui| {
            let response = self.viewport.allocate(ui);
            self.handle_viewport_input(&ctx, &response);
            if let Some(anim) = self.cam_anim.as_mut() {
                if !anim.step(dt, &mut self.camera) {
                    self.cam_anim = None;
                }
                ctx.request_repaint();
            }
            let (o3, o2) = self.tool_overlays();
            let style = self.render_style();
            let cursor = response.hover_pos().or_else(|| {
                let p = self.last_pointer.filter(|_| self.debug.script_active())?;
                Some(self.viewport.rect.min + p.to_vec2())
            });
            self.viewport.paint(ui, frame, &self.doc, &self.camera, &style, &o3, &o2, cursor);
        });

        self.dialogs(&ctx);
        self.run_actions(&ctx);
        self.update_title(&ctx);
        if self.debug.update(&ctx) {
            self.actions.push(Action::SelectAll);
        }
        self.script_step(&ctx);
        if self.flash.as_ref().is_some_and(|(_, t)| *t > self.now) {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, PREFS_KEY, &self.prefs);
    }
}
