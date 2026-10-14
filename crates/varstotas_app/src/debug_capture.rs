//! Debug helpers driven by env vars (used for automated visual checks):
//! - `VARSTOTAS_DEMO=1` inserts demo geometry on startup.
//! - `VARSTOTAS_SELECT_ALL=1` selects everything after ~20 frames.
//! - `VARSTOTAS_SCREENSHOT=<file.bmp>` captures the window after ~40 frames, writes a BMP, quits.
//! - `VARSTOTAS_SCRIPT=<file>` drives tools with scripted input (see [`ScriptCmd`] for the
//!   command list), taking screenshots along the way; quits at the end of the script.

use egui::{Context, Event, ViewportCommand};

#[derive(Default)]
pub struct DebugCapture {
    path: Option<String>,
    frame: u32,
    requested: bool,
    /// Set once the capture is written: the app should close without the save prompt.
    pub force_close: bool,
    script: Option<Script>,
    /// Frame-time measurement window started by the `perf` script command.
    pub perf: Option<PerfWindow>,
}

/// Collects CPU time of `N` frames (`ui()` duration) and logs the statistics.
#[derive(Default)]
pub struct PerfWindow {
    pub remaining: u32,
    pub samples: Vec<f64>,
    pub rebuilds_at_start: u64,
}

impl PerfWindow {
    /// Record one frame; returns a summary line once the window is complete.
    pub fn record(&mut self, ms: f64, rebuilds: u64) -> Option<String> {
        self.samples.push(ms);
        self.remaining = self.remaining.saturating_sub(1);
        if self.remaining > 0 {
            return None;
        }
        let n = self.samples.len().max(1) as f64;
        let avg = self.samples.iter().sum::<f64>() / n;
        let max = self.samples.iter().copied().fold(0.0, f64::max);
        let mut sorted = self.samples.clone();
        sorted.sort_by(f64::total_cmp);
        let med = sorted.get(sorted.len() / 2).copied().unwrap_or(0.0);
        Some(format!(
            "perf: frames={} avg={avg:.2}ms median={med:.2}ms max={max:.2}ms scene_rebuilds={}",
            self.samples.len(),
            rebuilds - self.rebuilds_at_start
        ))
    }
}

impl DebugCapture {
    pub fn from_env() -> Self {
        let script = std::env::var("VARSTOTAS_SCRIPT").ok().and_then(|p| match std::fs::read_to_string(&p) {
            Ok(text) => Some(Script::parse(&text)),
            Err(e) => {
                log::error!("cannot read script {p}: {e}");
                None
            }
        });
        let path = if script.is_some() { None } else { std::env::var("VARSTOTAS_SCREENSHOT").ok() };
        Self { path, script, ..Default::default() }
    }

    pub fn script_active(&self) -> bool {
        self.script.is_some()
    }

    pub fn demo_requested() -> bool {
        std::env::var("VARSTOTAS_DEMO").is_ok_and(|v| v == "1")
    }

    /// Returns true on the frame when the debug "select all" should happen.
    pub fn update(&mut self, ctx: &Context) -> bool {
        let Some(path) = self.path.clone() else { return false };
        self.frame += 1;
        let select_all = self.frame == 20 && std::env::var("VARSTOTAS_SELECT_ALL").is_ok_and(|v| v == "1");
        ctx.request_repaint();
        if !self.requested && self.frame >= 40 {
            self.requested = true;
            ctx.send_viewport_cmd(ViewportCommand::Screenshot(Default::default()));
        }
        let shot = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = shot {
            if let Err(e) = write_bmp(&path, img.size[0], img.size[1], &img.pixels) {
                log::error!("screenshot write failed: {e}");
            }
            self.path = None;
            self.force_close = true;
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        select_all
    }
}

fn write_bmp(path: &str, w: usize, h: usize, px: &[egui::Color32]) -> std::io::Result<()> {
    let row = w * 3;
    let pad = (4 - row % 4) % 4;
    let data_len = (row + pad) * h;
    let mut out = Vec::with_capacity(54 + data_len);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&((54 + data_len) as u32).to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(w as i32).to_le_bytes());
    out.extend_from_slice(&(h as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&(data_len as u32).to_le_bytes());
    out.extend_from_slice(&[0; 16]);
    for y in (0..h).rev() {
        for x in 0..w {
            let c = px[y * w + x];
            out.extend_from_slice(&[c.b(), c.g(), c.r()]);
        }
        out.extend(std::iter::repeat_n(0u8, pad));
    }
    std::fs::write(path, out)
}

// ---------------------------------------------------------------------------
// Scripted input
// ---------------------------------------------------------------------------

use crate::app::VarstotasApp;
use crate::tools::ToolId;
use egui::{Key, Modifiers, PointerButton, Pos2};
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_render::StandardView;

/// One script line. Points are world coordinates (mm) unless the command ends in `px`.
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptCmd {
    /// `tool line|rectangle|circle|polygon|arc|eraser|paint|pushpull|move|rotate|scale|offset|followme|tape|protractor|select`
    Tool(ToolId),
    /// `move x y z` — hover a world point.
    Move(DVec3),
    MovePx(Pos2),
    /// `click x y z` — move, press, release.
    Click(DVec3),
    ClickPx(Pos2),
    /// `dblclick x y z`
    DoubleClick(DVec3),
    /// `drag x y z  x y z` — press at the first point, move in steps, release at the second.
    Drag(DVec3, DVec3),
    /// `vcb <text>` — type into the Measurements box and press Enter.
    Vcb(String),
    /// `key Escape|ArrowRight|ArrowLeft|ArrowUp|ArrowDown|Enter`
    Key(Key),
    /// `mods none|shift|ctrl|alt|cmd` (combinations with `+`).
    Mods(Modifiers),
    /// `wait N` frames.
    Wait(u32),
    /// `view iso|top|front|right|left|back|bottom`
    View(StandardView),
    ZoomExtents,
    SelectAll,
    SelectNone,
    Undo,
    /// `screenshot <file.bmp>`
    Screenshot(String),
    Quit,
    /// `parallel on|off` — Camera > Parallel Projection / Perspective.
    Parallel(bool),
    /// `dragpx x0 y0 x1 y1` — primary-button drag in screen points (e.g. selection box).
    DragPx(Pos2, Pos2),
    /// `zoompx factor [x y]` — wheel zoom toward the point under screen position (x, y), or
    /// under the last pointer position.
    ZoomPx(f64, Pos2),
    /// `orbitpx dx dy` / `panpx dx dy` — middle-button orbit / pan by a screen delta,
    /// pivoting on the point under the last pointer position.
    OrbitPx(egui::Vec2),
    PanPx(egui::Vec2),
    /// `downpx x y` / `uppx x y` — press / release the primary button (with `movepx` in
    /// between while held, e.g. to screenshot a selection box mid-drag).
    DownPx(Pos2),
    UpPx(Pos2),
    /// `dump [label]` — log selection / model statistics.
    Dump(String),
    /// `stress N` — insert an N×N grid of boxes (6 faces each) as one undo step.
    Stress(u32),
    /// `box x y z w d h` — insert a push-pulled box (one undo step), e.g. far from the origin.
    Box(DVec3, DVec3),
    /// `style shaded|mono|hidden|wire` — View > Face Style.
    Style(crate::app::FaceStylePref),
    /// `perf N` — log CPU frame-time statistics over the next N frames (non-blocking: the
    /// following commands run inside the measured window, one every 3 frames).
    Perf(u32),
    /// `save <path>` — save the document to `path` (no file dialog).
    SaveTo(String),
    /// `autosave` — write the autosave now.
    Autosave,
    /// `recover yes|no|later` — answer the autosave recovery prompt.
    Recover(Option<bool>),
}

fn tool_by_name(s: &str) -> Option<ToolId> {
    let s = s.to_ascii_lowercase().replace(['/', '_', '-', ' '], "");
    ToolId::ALL.iter().copied().find(|t| {
        let n = t.name().to_ascii_lowercase().replace(['/', ' '], "");
        n == s || format!("{t:?}").to_ascii_lowercase() == s
    })
}

fn parse_cmd(line: &str) -> Option<ScriptCmd> {
    let line = line.split('#').next().unwrap_or("").trim();
    if line.is_empty() {
        return None;
    }
    let (cmd, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
    let rest = rest.trim();
    let nums: Vec<f64> = rest.split(|c: char| c.is_whitespace() || c == ',').filter(|t| !t.is_empty()).filter_map(|t| t.parse().ok()).collect();
    let p3 = |i: usize| DVec3::new(nums[i], nums[i + 1], nums[i + 2]);
    let r = match cmd.to_ascii_lowercase().as_str() {
        "tool" => ScriptCmd::Tool(tool_by_name(rest)?),
        "move" | "hover" if nums.len() >= 3 => ScriptCmd::Move(p3(0)),
        "movepx" if nums.len() >= 2 => ScriptCmd::MovePx(Pos2::new(nums[0] as f32, nums[1] as f32)),
        "click" if nums.len() >= 3 => ScriptCmd::Click(p3(0)),
        "clickpx" if nums.len() >= 2 => ScriptCmd::ClickPx(Pos2::new(nums[0] as f32, nums[1] as f32)),
        "dblclick" if nums.len() >= 3 => ScriptCmd::DoubleClick(p3(0)),
        "drag" if nums.len() >= 6 => ScriptCmd::Drag(p3(0), p3(3)),
        "vcb" => ScriptCmd::Vcb(rest.to_string()),
        "key" => ScriptCmd::Key(Key::from_name(rest)?),
        "mods" => {
            let mut m = Modifiers::NONE;
            for part in rest.split('+') {
                match part.trim().to_ascii_lowercase().as_str() {
                    "shift" => m.shift = true,
                    "ctrl" => m.ctrl = true,
                    "alt" | "option" => m.alt = true,
                    "cmd" | "command" => {
                        m.command = true;
                        m.mac_cmd = true;
                    }
                    _ => {}
                }
            }
            ScriptCmd::Mods(m)
        }
        "wait" => ScriptCmd::Wait(nums.first().copied().unwrap_or(1.0) as u32),
        "view" => ScriptCmd::View(match rest.to_ascii_lowercase().as_str() {
            "top" => StandardView::Top,
            "front" => StandardView::Front,
            "right" => StandardView::Right,
            "left" => StandardView::Left,
            "back" => StandardView::Back,
            "bottom" => StandardView::Bottom,
            _ => StandardView::Iso,
        }),
        "zoom_extents" | "zoomextents" => ScriptCmd::ZoomExtents,
        "select_all" => ScriptCmd::SelectAll,
        "select_none" => ScriptCmd::SelectNone,
        "undo" => ScriptCmd::Undo,
        "screenshot" => ScriptCmd::Screenshot(rest.to_string()),
        "quit" => ScriptCmd::Quit,
        "parallel" | "ortho" => ScriptCmd::Parallel(matches!(rest.to_ascii_lowercase().as_str(), "on" | "1" | "true" | "")),
        "dragpx" if nums.len() >= 4 => ScriptCmd::DragPx(
            Pos2::new(nums[0] as f32, nums[1] as f32),
            Pos2::new(nums[2] as f32, nums[3] as f32),
        ),
        "zoompx" if nums.len() >= 3 => ScriptCmd::ZoomPx(nums[0], Pos2::new(nums[1] as f32, nums[2] as f32)),
        // `zoompx f` alone zooms toward the last pointer position (NaN marker).
        "zoompx" if nums.len() == 1 => ScriptCmd::ZoomPx(nums[0], Pos2::new(f32::NAN, f32::NAN)),
        "orbitpx" if nums.len() >= 2 => ScriptCmd::OrbitPx(egui::vec2(nums[0] as f32, nums[1] as f32)),
        "panpx" if nums.len() >= 2 => ScriptCmd::PanPx(egui::vec2(nums[0] as f32, nums[1] as f32)),
        "dump" => ScriptCmd::Dump(rest.to_string()),
        "downpx" if nums.len() >= 2 => ScriptCmd::DownPx(Pos2::new(nums[0] as f32, nums[1] as f32)),
        "uppx" if nums.len() >= 2 => ScriptCmd::UpPx(Pos2::new(nums[0] as f32, nums[1] as f32)),
        "stress" => ScriptCmd::Stress(nums.first().copied().unwrap_or(41.0) as u32),
        "box" if nums.len() >= 6 => ScriptCmd::Box(p3(0), p3(3)),
        "style" => ScriptCmd::Style(match rest.to_ascii_lowercase().as_str() {
            "mono" | "monochrome" => crate::app::FaceStylePref::Monochrome,
            "hidden" | "hiddenline" => crate::app::FaceStylePref::HiddenLine,
            "wire" | "wireframe" => crate::app::FaceStylePref::Wireframe,
            _ => crate::app::FaceStylePref::Shaded,
        }),
        "perf" => ScriptCmd::Perf(nums.first().copied().unwrap_or(60.0) as u32),
        "autosave" => ScriptCmd::Autosave,
        "save" if !rest.is_empty() => ScriptCmd::SaveTo(rest.to_string()),
        "recover" => ScriptCmd::Recover(match rest.to_ascii_lowercase().as_str() {
            "yes" | "y" | "recover" => Some(true),
            "no" | "n" | "discard" => Some(false),
            _ => None,
        }),
        _ => {
            log::warn!("script: unknown command `{line}`");
            return None;
        }
    };
    Some(r)
}

pub struct Script {
    cmds: Vec<ScriptCmd>,
    pc: usize,
    wait: u32,
    frame: u32,
    shot: Option<String>,
    /// Viewport size seen last frame and for how many frames it has been unchanged.
    last_vp: Option<egui::Vec2>,
    stable: u32,
}

impl Script {
    pub fn parse(text: &str) -> Self {
        Self {
            cmds: text.lines().filter_map(parse_cmd).collect(),
            pc: 0,
            wait: 0,
            frame: 0,
            shot: None,
            last_vp: None,
            stable: 0,
        }
    }
}

impl VarstotasApp {
    fn script_screen(&self, p: DVec3) -> Pos2 {
        let vp = self.viewport.size();
        let s = self
            .camera
            .world_to_screen(p, DVec2::new(vp.x as f64, vp.y as f64))
            .map(|s| Pos2::new(s.x as f32, s.y as f32));
        match s {
            Some(s) if s.x >= 0.0 && s.y >= 0.0 && s.x <= vp.x && s.y <= vp.y => s,
            _ => {
                log::warn!("script: world point {p} is off-screen ({s:?}); results may not match intent");
                s.unwrap_or(Pos2::new(vp.x * 0.5, vp.y * 0.5))
            }
        }
    }

    fn script_move(&mut self, pos: Pos2, primary_down: bool) {
        let primary_down = primary_down || self.tool_pressed;
        let delta = self.last_pointer.map(|l| pos - l).unwrap_or_default();
        self.last_pointer = Some(pos);
        let pi = self.pointer_info(pos, delta, None, primary_down);
        self.with_tool(pi, |t, c| t.on_pointer_move(c));
    }

    fn script_button(&mut self, pos: Pos2, pressed: bool) {
        self.last_pointer = Some(pos);
        let pi = self.pointer_info(pos, egui::Vec2::ZERO, Some(PointerButton::Primary), pressed);
        if pressed {
            self.tool_pressed = true;
            self.with_tool(pi, |t, c| t.on_pointer_down(c));
        } else {
            self.tool_pressed = false;
            self.with_tool(pi, |t, c| t.on_pointer_up(c));
        }
    }

    fn script_click(&mut self, pos: Pos2) {
        self.script_move(pos, false);
        self.script_button(pos, true);
        self.script_button(pos, false);
        self.after_primary_click(pos, self.last_modifiers);
    }

    /// Execute scripted input (one command per frame; screenshots wait for the capture).
    pub(crate) fn script_step(&mut self, ctx: &Context) {
        let Some(script) = self.debug.script.as_mut() else { return };
        ctx.request_repaint();
        script.frame += 1;
        // Let the window and renderer settle: the viewport size must be stable for a while
        // before the first command (world → screen mapping depends on it).
        let vp = self.viewport.size();
        if script.pc == 0 && script.frame < 600 {
            if script.last_vp == Some(vp) {
                script.stable += 1;
            } else {
                script.stable = 0;
                script.last_vp = Some(vp);
            }
            if script.frame < 30 || script.stable < 15 {
                return;
            }
        }
        if let Some(path) = script.shot.clone() {
            let shot = ctx.input(|i| {
                i.events.iter().find_map(|e| match e {
                    Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            });
            if let Some(img) = shot {
                if let Err(e) = write_bmp(&path, img.size[0], img.size[1], &img.pixels) {
                    log::error!("screenshot write failed: {e}");
                }
                log::info!("script: wrote {path}");
                script.shot = None;
            }
            return;
        }
        if script.wait > 0 {
            script.wait -= 1;
            return;
        }
        let Some(cmd) = script.cmds.get(script.pc).cloned() else {
            self.debug.force_close = true;
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        };
        script.pc += 1;
        script.wait = 2;
        log::info!("script: {cmd:?}");
        match cmd {
            ScriptCmd::Tool(id) => self.set_tool(id),
            ScriptCmd::Move(p) => {
                let s = self.script_screen(p);
                self.script_move(s, false);
            }
            ScriptCmd::MovePx(s) => self.script_move(s, false),
            ScriptCmd::Click(p) => {
                let s = self.script_screen(p);
                self.script_click(s);
            }
            ScriptCmd::ClickPx(s) => self.script_click(s),
            ScriptCmd::DoubleClick(p) => {
                let s = self.script_screen(p);
                self.script_click(s);
                self.script_click(s);
                let mut pi = self.pointer_info(s, egui::Vec2::ZERO, Some(PointerButton::Primary), false);
                pi.click_count = 2;
                self.with_tool(pi, |t, c| t.on_double_click(c));
            }
            ScriptCmd::Drag(a, b) => {
                let (sa, sb) = (self.script_screen(a), self.script_screen(b));
                self.script_move(sa, false);
                self.script_button(sa, true);
                for i in 1..=10 {
                    self.script_move(sa.lerp(sb, i as f32 / 10.0), true);
                }
                self.script_button(sb, false);
            }
            ScriptCmd::Vcb(text) => {
                self.vcb = text;
                self.submit_vcb();
            }
            ScriptCmd::Key(k) => {
                let p = self.last_pointer.unwrap_or_default();
                let pi = self.pointer_info(p, egui::Vec2::ZERO, None, false);
                self.with_tool(pi, |t, c| t.on_key(c, k, true));
            }
            ScriptCmd::Mods(m) => {
                self.last_modifiers = m;
                if let Some(p) = self.last_pointer {
                    self.script_move(p, false);
                }
            }
            ScriptCmd::Wait(n) => {
                if let Some(s) = self.debug.script.as_mut() {
                    s.wait = n;
                }
            }
            ScriptCmd::View(v) => {
                self.actions.push(crate::app::Action::View(v));
                if let Some(s) = self.debug.script.as_mut() {
                    s.wait = 45;
                }
            }
            ScriptCmd::ZoomExtents => {
                self.actions.push(crate::app::Action::ZoomExtents);
                if let Some(s) = self.debug.script.as_mut() {
                    s.wait = 45;
                }
            }
            ScriptCmd::SelectAll => self.doc.select_all(),
            ScriptCmd::SelectNone => self.doc.clear_selection(),
            ScriptCmd::Undo => {
                self.doc.undo();
            }
            ScriptCmd::Screenshot(path) => {
                if let Some(s) = self.debug.script.as_mut() {
                    s.shot = Some(path);
                }
                ctx.send_viewport_cmd(ViewportCommand::Screenshot(Default::default()));
            }
            ScriptCmd::Quit => {
                if let Some(s) = self.debug.script.as_mut() {
                    s.pc = s.cmds.len();
                }
            }
            ScriptCmd::Parallel(on) => self.actions.push(crate::app::Action::SetParallel(on)),
            ScriptCmd::DragPx(a, b) => {
                self.script_move(a, false);
                self.script_button(a, true);
                for i in 1..=10 {
                    self.script_move(a.lerp(b, i as f32 / 10.0), true);
                }
                self.script_button(b, false);
            }
            ScriptCmd::ZoomPx(f, at) => {
                let vp = self.viewport.size();
                let at = if at.x.is_nan() { self.last_pointer.unwrap_or((vp * 0.5).to_pos2()) } else { at };
                let target = crate::nav::point_under(&self.doc.model, &self.camera, at, vp);
                self.camera.zoom(f, target);
            }
            ScriptCmd::OrbitPx(d) | ScriptCmd::PanPx(d) => {
                let vp = self.viewport.size();
                let at = self.last_pointer.unwrap_or((vp * 0.5).to_pos2());
                let pivot = crate::nav::point_under(&self.doc.model, &self.camera, at, vp);
                if matches!(cmd, ScriptCmd::OrbitPx(_)) {
                    self.camera.orbit(crate::nav::dvec2(d), pivot);
                } else {
                    self.camera.pan(crate::nav::dvec2(d), pivot, crate::nav::dvec2(vp));
                }
            }
            ScriptCmd::DownPx(p) => {
                self.script_move(p, false);
                self.script_button(p, true);
            }
            ScriptCmd::UpPx(p) => {
                self.script_move(p, true);
                self.script_button(p, false);
            }
            ScriptCmd::Dump(label) => log::info!("dump {label}: {}", self.debug_summary()),
            ScriptCmd::Stress(n) => self.insert_stress_grid(n),
            ScriptCmd::Box(at, size) => {
                self.doc.commit("Insert Box", |m| {
                    let mesh = m.root_mesh_mut();
                    let pts = [
                        at,
                        at + DVec3::new(size.x, 0.0, 0.0),
                        at + DVec3::new(size.x, size.y, 0.0),
                        at + DVec3::new(0.0, size.y, 0.0),
                    ];
                    if let Some(f) = mesh.add_face(&pts) {
                        let up = mesh.faces.get(f).map(|f| f.normal.z >= 0.0).unwrap_or(true);
                        let _ = mesh.push_pull(f, if up { size.z } else { -size.z }, false);
                    }
                });
            }
            ScriptCmd::Perf(n) => {
                self.debug.perf = Some(PerfWindow {
                    remaining: n.max(1),
                    samples: Vec::new(),
                    rebuilds_at_start: self.viewport.rebuild_count,
                });
            }
            ScriptCmd::Autosave => self.autosave_now(),
            ScriptCmd::Style(st) => self.prefs.face_style = st,
            ScriptCmd::SaveTo(p) => {
                self.save_to(std::path::Path::new(&p));
            }
            ScriptCmd::Recover(c) => self.resolve_recovery(c),
        }
    }
}

impl VarstotasApp {
    /// One-line state summary for `dump`.
    pub(crate) fn debug_summary(&self) -> String {
        use varstotas_kernel::EntityRef;
        let sel = &self.doc.selection;
        let count = |f: fn(&EntityRef) -> bool| sel.iter().filter(|e| f(e)).count();
        let faces = count(|e| matches!(e, EntityRef::Face(_)));
        let edges = count(|e| matches!(e, EntityRef::Edge(_)));
        let insts = count(|e| matches!(e, EntityRef::Instance(_)));
        let (mf, me) = self.doc.active_mesh().map(|m| (m.faces.len(), m.edges.len())).unwrap_or_default();
        format!(
            "tool={:?} sel F={faces} E={edges} I={insts} context_depth={} active_mesh F={mf} E={me} ortho={} dirty={} tris={} rebuilds={} last_rebuild={:.1}ms highlights={} last_highlight={:.1}ms",
            self.tool_id(),
            self.doc.active_path.len(),
            self.camera.ortho,
            self.doc.dirty,
            self.viewport.tri_count,
            self.viewport.rebuild_count,
            self.viewport.last_rebuild_ms,
            self.viewport.highlight_count,
            self.viewport.last_highlight_ms,
        )
    }

    /// Stress model: `n`×`n` separate boxes (6 faces each), one undo step.
    pub(crate) fn insert_stress_grid(&mut self, n: u32) {
        let t0 = std::time::Instant::now();
        self.doc.commit("Insert Stress Grid", |m| {
            let mesh = m.root_mesh_mut();
            for i in 0..n {
                for j in 0..n {
                    let (x, y) = (i as f64 * 1500.0, j as f64 * 1500.0);
                    let h = 500.0 + ((i * 7 + j * 13) % 10) as f64 * 200.0;
                    let pts = [
                        DVec3::new(x, y, 0.0),
                        DVec3::new(x + 1000.0, y, 0.0),
                        DVec3::new(x + 1000.0, y + 1000.0, 0.0),
                        DVec3::new(x, y + 1000.0, 0.0),
                    ];
                    if let Some(f) = mesh.add_face(&pts) {
                        let up = mesh.faces.get(f).map(|f| f.normal.z >= 0.0).unwrap_or(true);
                        let _ = mesh.push_pull(f, if up { h } else { -h }, false);
                    }
                }
            }
        });
        log::info!(
            "stress: {n}x{n} boxes, {} faces, built in {:?}",
            self.doc.model.root_mesh().faces.len(),
            t0.elapsed()
        );
        self.actions.push(crate::app::Action::ZoomExtents);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_script_lines() {
        assert_eq!(parse_cmd("tool rectangle"), Some(ScriptCmd::Tool(ToolId::Rectangle)));
        assert_eq!(parse_cmd("tool pushpull"), Some(ScriptCmd::Tool(ToolId::PushPull)));
        assert_eq!(parse_cmd("click 1 2 3 # c"), Some(ScriptCmd::Click(DVec3::new(1.0, 2.0, 3.0))));
        assert_eq!(parse_cmd("vcb 3000,2000"), Some(ScriptCmd::Vcb("3000,2000".into())));
        assert_eq!(parse_cmd("key ArrowUp"), Some(ScriptCmd::Key(Key::ArrowUp)));
        assert_eq!(parse_cmd("# only comment"), None);
        assert_eq!(parse_cmd("parallel on"), Some(ScriptCmd::Parallel(true)));
        assert_eq!(parse_cmd("parallel off"), Some(ScriptCmd::Parallel(false)));
        assert_eq!(
            parse_cmd("dragpx 10 20 300 400"),
            Some(ScriptCmd::DragPx(Pos2::new(10.0, 20.0), Pos2::new(300.0, 400.0)))
        );
        assert_eq!(parse_cmd("recover yes"), Some(ScriptCmd::Recover(Some(true))));
        assert_eq!(parse_cmd("recover later"), Some(ScriptCmd::Recover(None)));
        assert_eq!(parse_cmd("stress 10"), Some(ScriptCmd::Stress(10)));
    }

    #[test]
    fn perf_window_summarizes() {
        let mut w = PerfWindow { remaining: 3, samples: Vec::new(), rebuilds_at_start: 5 };
        assert!(w.record(1.0, 5).is_none());
        assert!(w.record(3.0, 5).is_none());
        let s = w.record(2.0, 6).unwrap();
        assert!(s.contains("frames=3") && s.contains("avg=2.00ms") && s.contains("scene_rebuilds=1"), "{s}");
    }
}
