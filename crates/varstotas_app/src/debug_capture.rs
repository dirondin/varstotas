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

    /// A script drives the tools directly (synthetic input): real egui input is ignored.
    /// False for raw-input scripts, whose events go through the normal egui path.
    pub fn synthetic_input(&self) -> bool {
        self.script.as_ref().is_some_and(|s| s.raw.is_none())
    }

    /// `eframe::App::raw_input_hook` for raw-input scripts.
    pub fn raw_input_hook(&mut self, raw: &mut egui::RawInput) {
        if let Some(inj) = self.script.as_mut().and_then(|s| s.raw.as_mut()) {
            inj.apply(raw);
        }
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
    /// `material <palette name>|default` — set the Paint Bucket material.
    Material(String),
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
    /// `raw ...` — see [`RawCmd`].
    Raw(RawCmd),
    /// `rawp <sub> x y z [x y z] [steps]` — like `raw <sub>` but with world points, projected
    /// with the camera when the command runs (pointer subcommands only).
    RawWorld(String, Vec<DVec3>, Option<u32>),
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
    /// `expect <text>` — check that the `dump` summary contains `text`; logs `expect FAIL` otherwise.
    Expect(String),
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
    /// `open <path>` — open a model file (no file dialog, no "save changes?" prompt).
    Open(String),
    /// `texture <path> [width_mm]` — new material from an image (Materials → Texture…).
    Texture(String, f64),
    /// `export_png <path> [w h] [transparent] [noaxes] [noguides] [nodims]` — File → Export → 2D Graphic without dialogs
    /// (default size: the viewport's).
    ExportPng(String, Option<[u32; 2]>, crate::app::ExportContent),
    /// `action <name>` — run a menu action (see `script_action`).
    Action(String),
}

fn tool_by_name(s: &str) -> Option<ToolId> {
    let s = s.to_ascii_lowercase().replace(['/', '_', '-', ' '], "");
    ToolId::ALL.iter().copied().find(|t| {
        let n = t.name().to_ascii_lowercase().replace(['/', ' '], "");
        n == s || format!("{t:?}").to_ascii_lowercase() == s
    })
}

fn parse_mods(s: &str) -> Modifiers {
    let mut m = Modifiers::NONE;
    for part in s.split('+') {
        match part.trim().to_ascii_lowercase().as_str() {
            "shift" => m.shift = true,
            "ctrl" | "control" => m.ctrl = true,
            "alt" | "option" | "opt" => m.alt = true,
            "cmd" | "command" => {
                m.command = true;
                m.mac_cmd = true;
            }
            _ => {}
        }
    }
    m
}

/// `raw <sub> ...` — genuine egui input events, injected through `raw_input_hook`.
fn parse_raw(rest: &str) -> Option<RawCmd> {
    let (sub, args) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    let args = args.trim();
    let n: Vec<f32> = args.split(|c: char| c.is_whitespace() || c == ',').filter_map(|t| t.parse().ok()).collect();
    let p = |i: usize| n.get(i + 1).map(|y| Pos2::new(n[i], *y));
    Some(match sub.to_ascii_lowercase().as_str() {
        "move" => RawCmd::Move(p(0)?),
        "down" => RawCmd::Down(p(0)?),
        "up" => RawCmd::Up(p(0)?),
        "click" => RawCmd::Click(p(0)?),
        "tap" => RawCmd::Tap(p(0)?),
        "rclick" => RawCmd::RightClick(p(0)?),
        "dblclick" => RawCmd::DoubleClick(p(0)?),
        "drag" => RawCmd::Drag { a: p(0)?, b: p(2)?, steps: n.get(4).map_or(8, |s| *s as u32).max(1), fast: false },
        "dragfast" => RawCmd::Drag { a: p(0)?, b: p(2)?, steps: n.get(4).map_or(8, |s| *s as u32).max(1), fast: true },
        "mods" => RawCmd::Mods(parse_mods(args)),
        "copy" => RawCmd::Copy,
        "cut" => RawCmd::Cut,
        "paste" => RawCmd::Paste(if args.is_empty() { "Varstotas".into() } else { args.to_string() }),
        "text" if !args.is_empty() => RawCmd::Text(args.to_string()),
        "key" => {
            let (k, st) = args.split_once(char::is_whitespace).unwrap_or((args, "tap"));
            let key = Key::from_name(k.trim())?;
            match st.trim() {
                "down" => RawCmd::Key(key, Some(true)),
                "up" => RawCmd::Key(key, Some(false)),
                _ => RawCmd::Key(key, None),
            }
        }
        _ => {
            log::warn!("script: unknown raw command `{rest}`");
            return None;
        }
    })
}

/// Raw (egui-level) input commands. Points are viewport-local screen points, like `movepx`.
/// Each listed step is delivered in its own frame, like real input, unless noted.
#[derive(Clone, Debug, PartialEq)]
pub enum RawCmd {
    /// `raw move x y` — PointerMoved.
    Move(Pos2),
    /// `raw down x y` / `raw up x y` — move, then primary press / release (2 frames).
    Down(Pos2),
    Up(Pos2),
    /// `raw click x y` — move, press, release (3 frames).
    Click(Pos2),
    /// `raw tap x y` — move + press + release in ONE frame (trackpad tap-to-click).
    Tap(Pos2),
    /// `raw rclick x y` — move, secondary press, release (3 frames): context menu.
    RightClick(Pos2),
    /// `raw dblclick x y` — move, press, release, press, release (5 frames).
    DoubleClick(Pos2),
    /// `raw drag x0 y0 x1 y1 [steps]` — move, press, `steps` moves (one per frame), release.
    /// `raw dragfast ...` delivers the last move and the release in the same frame.
    Drag { a: Pos2, b: Pos2, steps: u32, fast: bool },
    /// `raw mods shift+alt|none` — `ModifiersChanged` only (what winit emits for modifier
    /// keys; egui 0.36 tracks `InputState::modifiers` from it). Later pointer/key events
    /// carry the new state in their `modifiers` field. No `Key` events are sent.
    Mods(Modifiers),
    /// `raw key <Name> [down|up]` — Key event(s); without down/up: press then release.
    Key(Key, Option<bool>),
    /// `raw copy` / `raw cut` — what egui-winit emits for Cmd+C / Cmd+X.
    Copy,
    Cut,
    /// `raw paste [text]` — what egui-winit emits for Cmd+V (with the OS clipboard text);
    /// combine with `raw mods cmd+shift` for Paste in Place.
    Paste(String),
    /// `raw text <s>` — typed text (e.g. VCB input).
    Text(String),
}

/// Pending raw events, one `Vec` per frame, plus the scripted modifier state (copied into
/// the events' `modifiers` fields).
#[derive(Default)]
pub struct RawInjector {
    pub frames: std::collections::VecDeque<Vec<Event>>,
    pub mods: Modifiers,
}

impl RawInjector {
    /// Replace the OS input of this frame by the scripted events (the OS mouse/keyboard must
    /// not interfere).
    pub fn apply(&mut self, raw: &mut egui::RawInput) {
        raw.events.retain(|e| {
            !matches!(
                e,
                Event::PointerMoved(_)
                    | Event::MouseMoved(_)
                    | Event::PointerButton { .. }
                    | Event::PointerGone
                    | Event::Key { .. }
                    | Event::Text(_)
                    | Event::MouseWheel { .. }
                    | Event::Zoom(_)
                    | Event::Touch { .. }
                    | Event::ModifiersChanged(_)
                    | Event::WindowFocused(_)
            )
        });
        // The scripted window counts as focused (it usually isn't the frontmost app while a
        // script runs; egui's `Response::has_focus` requires a focused window).
        raw.focused = true;
        if let Some(evs) = self.frames.pop_front() {
            raw.events.extend(evs);
        }
    }
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
        "mods" => ScriptCmd::Mods(parse_mods(rest)),
        "raw" => ScriptCmd::Raw(parse_raw(rest)?),
        "rawp" => {
            let (sub, args) = rest.split_once(char::is_whitespace)?;
            let v: Vec<f64> = args.split(|c: char| c.is_whitespace() || c == ',').filter_map(|t| t.parse().ok()).collect();
            let pts: Vec<DVec3> = v.as_chunks::<3>().0.iter().map(|c| DVec3::new(c[0], c[1], c[2])).collect();
            if pts.is_empty() {
                return None;
            }
            let steps = (v.len() % 3 == 1).then(|| v[v.len() - 1] as u32);
            ScriptCmd::RawWorld(sub.to_string(), pts, steps)
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
        "material" => ScriptCmd::Material(rest.to_string()),
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
        "expect" => ScriptCmd::Expect(rest.to_string()),
        "downpx" if nums.len() >= 2 => ScriptCmd::DownPx(Pos2::new(nums[0] as f32, nums[1] as f32)),
        "uppx" if nums.len() >= 2 => ScriptCmd::UpPx(Pos2::new(nums[0] as f32, nums[1] as f32)),
        "stress" => ScriptCmd::Stress(nums.first().copied().unwrap_or(41.0) as u32),
        "box" if nums.len() >= 6 => ScriptCmd::Box(p3(0), p3(3)),
        "style" => ScriptCmd::Style(match rest.to_ascii_lowercase().as_str() {
            "mono" | "monochrome" => crate::app::FaceStylePref::Monochrome,
            "hidden" | "hiddenline" => crate::app::FaceStylePref::HiddenLine,
            "wire" | "wireframe" => crate::app::FaceStylePref::Wireframe,
            "color" | "shadedcolor" => crate::app::FaceStylePref::ShadedColor,
            _ => crate::app::FaceStylePref::Shaded,
        }),
        "perf" => ScriptCmd::Perf(nums.first().copied().unwrap_or(60.0) as u32),
        "autosave" => ScriptCmd::Autosave,
        "save" if !rest.is_empty() => ScriptCmd::SaveTo(rest.to_string()),
        "open" if !rest.is_empty() => ScriptCmd::Open(rest.to_string()),
        "texture" if !rest.is_empty() => {
            let (path, w) = match rest.rsplit_once(char::is_whitespace) {
                Some((p, w)) if w.parse::<f64>().is_ok() => (p.trim(), w.parse().unwrap_or(1000.0)),
                _ => (rest, 1000.0),
            };
            ScriptCmd::Texture(path.to_string(), w)
        }
        "export_png" if !rest.is_empty() => {
            let mut it = rest.split_whitespace();
            let path = it.next()?.to_string();
            let tail: Vec<&str> = it.collect();
            let n: Vec<u32> = tail.iter().filter_map(|t| t.parse().ok()).collect();
            let size = (n.len() >= 2).then(|| [n[0], n[1]]);
            let has = |w: &str| tail.iter().any(|t| t.eq_ignore_ascii_case(w));
            let content = crate::app::ExportContent {
                transparent: has("transparent"),
                axes: !has("noaxes"),
                guides: !has("noguides"),
                dims: !has("nodims"),
            };
            ScriptCmd::ExportPng(path, size, content)
        }
        // 3D text keeps its case.
        "action" if rest.starts_with("text3d ") => ScriptCmd::Action(rest.to_string()),
        "action" if !rest.is_empty() => ScriptCmd::Action(rest.to_ascii_lowercase()),
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
    /// Present when the script uses `raw` commands: input then flows through egui exactly
    /// like real input (and OS input is replaced, see [`RawInjector::apply`]).
    pub raw: Option<RawInjector>,
}

impl Script {
    pub fn parse(text: &str) -> Self {
        let cmds: Vec<ScriptCmd> = text.lines().filter_map(parse_cmd).collect();
        let raw = cmds.iter().any(|c| matches!(c, ScriptCmd::Raw(_) | ScriptCmd::RawWorld(..))).then(RawInjector::default);
        Self {
            cmds,
            raw,
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
        if script.raw.as_ref().is_some_and(|r| !r.frames.is_empty()) {
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
            ScriptCmd::Material(name) => {
                self.doc.current_material = crate::ui::tray::PALETTE
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case(&name))
                    .map(|(n, c)| crate::document::PaletteMaterial::color(*n, crate::ui::tray::rgb(*c)))
                    .or_else(|| {
                        let m = self.doc.model.materials.values().find(|m| m.name.eq_ignore_ascii_case(&name))?;
                        Some(crate::document::PaletteMaterial::of(m))
                    });
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
            ScriptCmd::Expect(text) => {
                let summary = self.debug_summary();
                if summary.contains(&text) {
                    log::info!("expect ok: {text}");
                } else {
                    log::error!("expect FAIL: {text:?} not in: {summary}");
                }
            }
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
            ScriptCmd::Recover(c) => {
                if self.autosave.recoveries.is_empty() {
                    log::warn!("script: `recover` but no autosave recovery is pending (already answered, or none found)");
                }
                self.resolve_recovery(c)
            }
            ScriptCmd::Texture(p, w) => self.add_texture_material(std::path::Path::new(&p), w),
            ScriptCmd::Open(p) => {
                self.doc.dirty = false;
                self.actions.push(crate::app::Action::OpenPath(p.into()));
            }
            ScriptCmd::ExportPng(p, size, content) => {
                let vp = self.viewport.size();
                let size = size.unwrap_or([vp.x.round() as u32, vp.y.round() as u32]);
                self.export_png_to(p.into(), size, content);
            }
            ScriptCmd::Action(name) => self.script_action(&name),
            ScriptCmd::Raw(r) => self.queue_raw(r),
            ScriptCmd::RawWorld(sub, pts, steps) => {
                let mut txt = sub.clone();
                for p in &pts {
                    let s = self.script_screen(*p);
                    txt.push_str(&format!(" {} {}", s.x, s.y));
                }
                if let Some(n) = steps {
                    txt.push_str(&format!(" {n}"));
                }
                match parse_raw(&txt) {
                    Some(r) => self.queue_raw(r),
                    None => log::warn!("script: bad rawp `{txt}`"),
                }
            }
        }
    }

    /// `action <name>`: menu actions by name.
    fn script_action(&mut self, name: &str) {
        use crate::app::Action;
        use varstotas_kernel::IntersectScope;
        // `shadow_time HH:MM`, `shadow_date <month> <day>`
        if let Some(t) = name.strip_prefix("shadow_time") {
            let mut sh = self.doc.model.shadows;
            if let Some((h, m)) = t.trim().split_once(':')
                && let (Ok(h), Ok(m)) = (h.parse::<u32>(), m.parse::<u32>())
            {
                sh.minutes = h * 60 + m;
                self.actions.push(Action::SetShadows(sh));
            } else {
                log::warn!("script: shadow_time needs HH:MM");
            }
            return;
        }
        if let Some(t) = name.strip_prefix("shadow_date") {
            let mut sh = self.doc.model.shadows;
            let v: Vec<u32> = t.split_whitespace().filter_map(|x| x.parse().ok()).collect();
            if let [m, d] = v[..] {
                sh.day = varstotas_kernel::shadows::day_of_year(m, d);
                self.actions.push(Action::SetShadows(sh));
            } else {
                log::warn!("script: shadow_date needs <month> <day>");
            }
            return;
        }
        // `text3d <height> <depth|flat|outline> [mono] <text…>` (`\n` = new line): starts placing.
        if let Some(args) = name.strip_prefix("text3d ") {
            let mut it = args.trim().splitn(3, ' ');
            let (Some(h), Some(mode), Some(mut rest)) = (it.next(), it.next(), it.next()) else {
                log::warn!("script: text3d needs <height> <depth|flat|outline> <text>");
                return;
            };
            let mut p = crate::text3d::Text3dParams::default();
            if let Some(r) = rest.strip_prefix("mono ") {
                p.font = crate::text3d::Text3dFont::Mono;
                rest = r;
            }
            p.text = rest.replace("\\n", "\n");
            p.height = h.parse().unwrap_or(p.height);
            match mode {
                "flat" => p.extruded = false,
                "outline" => p.filled = false,
                d => p.depth = d.parse().unwrap_or(p.depth),
            }
            self.actions.push(Action::PlaceText3d(p));
            return;
        }
        // `export_file <path>` / `import_file <path>`: File → Export / Import without dialogs
        // (the format follows the extension).
        if let Some(path) = name.strip_prefix("export_file ") {
            self.export_to(std::path::Path::new(path.trim()));
            return;
        }
        if let Some(path) = name.strip_prefix("import_file ") {
            let path = std::path::PathBuf::from(path.trim());
            let label = path.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_uppercase();
            self.import_from(&path, &label);
            return;
        }
        // `solid <outer_shell|intersect|union|subtract|trim|split>`: Solid Tools (selection or tool).
        if let Some(arg) = name.strip_prefix("solid ") {
            let arg = arg.trim().replace('_', " ");
            match varstotas_kernel::SolidOp::ALL.into_iter().find(|o| o.name().eq_ignore_ascii_case(&arg)) {
                Some(op) => self.actions.push(Action::SolidTool(op)),
                None => log::warn!("script: unknown solid op {arg}"),
            }
            return;
        }
        // `soften_edges <deg> [nosmooth] [coplanar]`
        if let Some(args) = name.strip_prefix("soften_edges") {
            let mut it = args.split_whitespace();
            let Some(angle) = it.next().and_then(|a| a.parse::<f64>().ok()) else {
                log::warn!("script: soften_edges needs an angle");
                return;
            };
            let rest: Vec<&str> = it.collect();
            let s = crate::ui::tray::SoftenSettings { angle, smooth: !rest.contains(&"nosmooth"), coplanar: rest.contains(&"coplanar") };
            self.actions.push(Action::SoftenEdges(s));
            return;
        }
        let a = match name {
            "copy" => Action::Copy,
            "cut" => Action::Cut,
            "paste" => Action::Paste,
            "paste_in_place" => Action::PasteInPlace,
            "delete" => Action::Delete,
            "delete_guides" => Action::DeleteGuides,
            "toggle_guides" => Action::ToggleGuides,
            "toggle_shadows" | "shadows" => Action::ToggleShadows,
            "text3d_dialog" => Action::Text3d,
            "section_planes" => Action::ToggleSectionDisplay(0),
            "section_cuts" => Action::ToggleSectionDisplay(1),
            "section_fill" => Action::ToggleSectionDisplay(2),
            "active_cut" => Action::ActiveCut,
            "reverse_section" => Action::ReverseSection,
            "align_section_view" => Action::AlignSectionView,
            "make_group" => Action::MakeGroup,
            "explode" => Action::Explode,
            "exit_context" => Action::ExitContext,
            "intersect_model" => Action::IntersectFaces(IntersectScope::Model),
            "intersect_selection" => Action::IntersectFaces(IntersectScope::Selection),
            "intersect_context" => Action::IntersectFaces(IntersectScope::Context),
            "add_scene" => Action::AddScene,
            "update_scene" => Action::UpdateScene(self.tray.selected_scene.unwrap_or(0)),
            "next_scene" => Action::NextScene,
            "prev_scene" => Action::PrevScene,
            "play_animation" | "toggle_animation" => Action::ToggleAnimation,
            "scene_right" => Action::MoveScene(self.tray.selected_scene.unwrap_or(0), true),
            "reverse_faces" => Action::ReverseFaces,
            "orient_faces" => Action::OrientFaces,
            "flip_red" => Action::FlipAlong(0),
            "flip_green" => Action::FlipAlong(1),
            "flip_blue" => Action::FlipAlong(2),
            "export_dialog" => Action::ExportImage,
            "new" => Action::New,
            "preferences" => Action::Preferences,
            "about" => Action::About,
            "undo" => Action::Undo,
            "redo" => Action::Redo,
            "template_chooser" => {
                self.show_templates = true;
                return;
            }
            "tag_toggle" => {
                // Toggle the first tag's visibility (creating a tag "QA" if there is none).
                self.doc.mutate_untracked(|m| {
                    if m.tags.is_empty() {
                        m.tags.insert(varstotas_kernel::Tag { name: "QA".into(), visible: true, color: [0.5; 4] });
                    }
                    if let Some(t) = m.tags.values_mut().next() {
                        t.visible = !t.visible;
                    }
                });
                return;
            }
            // `scene_timing_<transition>_<delay>` (seconds; transition 0 = cuts).
            n if n.starts_with("scene_timing_") => {
                let v: Vec<f64> = n[13..].split('_').filter_map(|t| t.parse().ok()).collect();
                if let [t, d] = v[..] {
                    let s = &mut self.doc.model.scene_settings;
                    s.transitions = t > 0.0;
                    s.transition_secs = t;
                    s.delay_secs = d;
                }
                return;
            }
            n if n.starts_with("scene_") && n[6..].parse::<usize>().is_ok() => Action::ActivateScene(n[6..].parse().unwrap()),
            n => match crate::templates::Template::ALL
                .iter()
                .find(|t| format!("template_{t:?}").eq_ignore_ascii_case(n))
            {
                Some(t) => Action::NewFromTemplate(*t),
                None => {
                    log::warn!("script: unknown action `{n}`");
                    return;
                }
            },
        };
        self.actions.push(a);
    }

    fn queue_raw(&mut self, cmd: RawCmd) {
        let origin = self.viewport.rect.min.to_vec2();
        let Some(inj) = self.debug.script.as_mut().and_then(|s| s.raw.as_mut()) else { return };
        let m = inj.mods;
        let mv = |p: Pos2| Event::PointerMoved(p + origin);
        let btn = |p: Pos2, pressed: bool| Event::PointerButton {
            pos: p + origin,
            button: PointerButton::Primary,
            pressed,
            modifiers: m,
        };
        let key = |k: Key, pressed: bool| Event::Key { key: k, physical_key: None, pressed, repeat: false, modifiers: m };
        let frames: Vec<Vec<Event>> = match cmd {
            RawCmd::Move(p) => vec![vec![mv(p)]],
            RawCmd::Down(p) => vec![vec![mv(p)], vec![btn(p, true)]],
            RawCmd::Up(p) => vec![vec![mv(p)], vec![btn(p, false)]],
            RawCmd::Click(p) => vec![vec![mv(p)], vec![btn(p, true)], vec![btn(p, false)]],
            RawCmd::Tap(p) => vec![vec![mv(p), btn(p, true), btn(p, false)]],
            RawCmd::RightClick(p) => {
                let rb = |pressed: bool| Event::PointerButton {
                    pos: p + origin,
                    button: PointerButton::Secondary,
                    pressed,
                    modifiers: m,
                };
                vec![vec![mv(p)], vec![rb(true)], vec![rb(false)]]
            }
            RawCmd::DoubleClick(p) => {
                vec![vec![mv(p)], vec![btn(p, true)], vec![btn(p, false)], vec![btn(p, true)], vec![btn(p, false)]]
            }
            RawCmd::Drag { a, b, steps, fast } => {
                let mut f = vec![vec![mv(a)], vec![btn(a, true)]];
                for i in 1..=steps {
                    f.push(vec![mv(a.lerp(b, i as f32 / steps as f32))]);
                }
                if fast {
                    if let Some(last) = f.last_mut() {
                        last.push(btn(b, false));
                    }
                } else {
                    f.push(vec![btn(b, false)]);
                }
                f
            }
            RawCmd::Mods(new) => {
                inj.mods = new;
                vec![vec![Event::ModifiersChanged(new)]]
            }
            RawCmd::Key(k, Some(p)) => vec![vec![key(k, p)]],
            RawCmd::Key(k, None) => vec![vec![key(k, true)], vec![key(k, false)]],
            RawCmd::Copy => vec![vec![Event::Copy]],
            RawCmd::Cut => vec![vec![Event::Cut]],
            RawCmd::Paste(t) => vec![vec![Event::Paste(t)]],
            RawCmd::Text(t) => vec![vec![Event::Text(t)]],
        };
        inj.frames.extend(frames);
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
        let m = &self.doc.model;
        let all_faces: usize = m.defs.values().map(|d| d.mesh.faces.len()).sum();
        let guides = format!(
            "guides L={} P={} inner={} sel={} hidden={} defs={} model_faces={all_faces} unit={:?} undo={:?}",
            m.guides.len(),
            m.guide_points.len(),
            m.defs.values().map(|d| d.guides.lines.len() + d.guides.points.len()).sum::<usize>(),
            self.doc.guide_sel.len(),
            m.guides_hidden,
            m.defs.len(),
            m.unit,
            self.doc.history.undo_label(),
        );
        // Paint: current material and front-material histogram of the active mesh.
        let mut mats = std::collections::BTreeMap::<String, usize>::new();
        if let Some(mesh) = self.doc.active_mesh() {
            for f in mesh.faces.values() {
                let n = f.front.and_then(|id| m.materials.get(id)).map_or("-".to_string(), |mm| mm.name.clone());
                *mats.entry(n).or_default() += 1;
            }
        }
        let cur = self.doc.current_material.as_ref().map_or("Default", |p| p.name.as_str());
        let guides = format!("{guides} paint={cur} mats={mats:?}");
        // M4: dimensions, curves (arcs/circles) of the active mesh, custom axes.
        let dims = m.all_dimensions_world();
        let dim_txt: Vec<String> = dims
            .iter()
            .map(|(p, id, g)| {
                let label = m.dimension_label_in(p, *id).unwrap_or_else(|| format!("{:.0}", g.a.distance(g.b)));
                format!("{}:{}{label}", p.len(), if g.leader { "T:" } else { "" })
            })
            .collect();
        let curves: Vec<String> = self
            .doc
            .active_mesh()
            .map(|me| {
                me.curves
                    .values()
                    .map(|c| format!("{:?}r{:.1}s{}a{:.0}", c.kind, c.radius, c.segments, (c.end_angle - c.start_angle).to_degrees()))
                    .collect()
            })
            .unwrap_or_default();
        let ax = crate::axes::Axes::of_model(m);
        let guides = format!(
            "{guides} dims={} {dim_txt:?} dimsel={} curves={curves:?} axes={}",
            dims.len(),
            self.doc.dim_sel.len(),
            if m.axes.is_some() { format!("o({:.0},{:.0},{:.0}) x({:.2},{:.2},{:.2})", ax.origin.x, ax.origin.y, ax.origin.z, ax.x.x, ax.x.y, ax.x.z) } else { "world".into() }
        );
        let textures: Vec<String> = m
            .materials
            .values()
            .filter_map(|mm| mm.image.as_ref().map(|i| format!("{}:{:.0}x{:.0}", mm.name, i.size[0], i.size[1])))
            .collect();
        let uvs: Vec<String> = self.doc.active_mesh().map_or(vec![], |me| {
            me.faces
                .values()
                .filter_map(|f| f.uv)
                .map(|p| format!("o({:.0},{:.0}) a{:.0} s({:.2},{:.2})", p.offset[0], p.offset[1], p.angle_deg, p.scale[0], p.scale[1]))
                .collect()
        });
        let extra = format!("textures={textures:?} uv_placed={} uvs={uvs:?}", uvs.len());
        let sc = &m.scene_settings;
        let extra = format!(
            "{extra} scenes={:?} scene={:?} playing={} anim={} cam=({:.0},{:.0},{:.0}) style={} edges={} tags_hidden={} transition={:.1}s/{}",
            m.scenes.iter().map(|s| format!("{}{}", s.name, if s.in_animation { "" } else { "(-)" })).collect::<Vec<_>>(),
            self.tray.selected_scene,
            self.scene_player.is_some(),
            self.cam_anim.is_some(),
            self.camera.eye.x,
            self.camera.eye.y,
            self.camera.eye.z,
            self.prefs.face_style.key(),
            self.prefs.show_edges,
            m.tags.values().filter(|t| !t.visible).count(),
            sc.transition_secs,
            sc.transitions,
        );
        let sd = m.section_display;
        let cut = self.section_cut.borrow().as_ref().filter(|_| m.active_section().is_some()).map_or((0, 0), |c| (c.1.len(), c.2.len()));
        let extra = format!(
            "{extra} sections={:?} secsel={:?} secdisp={}{}{} cut={}/{}",
            m.sections
                .iter()
                .map(|s| format!(
                    "{}{} o({:.0},{:.0},{:.0}) n({:.2},{:.2},{:.2})",
                    s.name,
                    if s.active { "*" } else { "" },
                    s.origin.x,
                    s.origin.y,
                    s.origin.z,
                    s.normal.x,
                    s.normal.y,
                    s.normal.z
                ))
                .collect::<Vec<_>>(),
            self.doc.section_sel,
            if sd.planes { "P" } else { "-" },
            if sd.cuts { "C" } else { "-" },
            if sd.fill { "F" } else { "-" },
            cut.0,
            cut.1,
        );
        let (soft, smooth) = self.doc.active_mesh().map_or((0, 0), |mesh| {
            (mesh.edges.values().filter(|e| e.soft).count(), mesh.edges.values().filter(|e| e.smooth).count())
        });
        let f = self.camera.basis().forward;
        let extra = format!(
            "{extra} soft={soft} smooth={smooth} fov={:.0} look=({:.2},{:.2},{:.2})",
            self.camera.fov_y_deg,
            f.x + 0.0,
            f.y + 0.0,
            f.z + 0.0
        );
        let sh = &m.shadows;
        let sun = sh.sun_direction();
        let extra = format!(
            "{extra} shadows={} shadow_time={} shadow_date=\"{}\" sun=({:.2},{:.2},{:.2})",
            if sh.enabled { "on" } else { "off" },
            sh.time_label(),
            sh.date_label(),
            sun.x + 0.0,
            sun.y + 0.0,
            sun.z + 0.0
        );
        format!(
            "{guides} tool={:?} sel F={faces} E={edges} I={insts} context_depth={} active_mesh F={mf} E={me} ortho={} dirty={} tris={} rebuilds={} last_rebuild={:.1}ms (tess {:.1} build {:.1} upload {:.1}) highlights={} last_highlight={:.1}ms",
            self.tool_id(),
            self.doc.active_path.len(),
            self.camera.ortho,
            self.doc.dirty,
            self.viewport.tri_count,
            self.viewport.rebuild_count,
            self.viewport.last_rebuild_ms,
            self.viewport.last_rebuild_parts[0],
            self.viewport.last_rebuild_parts[1],
            self.viewport.last_rebuild_parts[2],
            self.viewport.highlight_count,
            self.viewport.last_highlight_ms,
        ) + " " + &extra
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
    fn raw_commands_parse_and_replace_os_input() {
        assert_eq!(
            parse_cmd("raw dragfast 10 20 30 40 3"),
            Some(ScriptCmd::Raw(RawCmd::Drag { a: Pos2::new(10.0, 20.0), b: Pos2::new(30.0, 40.0), steps: 3, fast: true }))
        );
        let alt_shift = Modifiers { alt: true, shift: true, ..Modifiers::NONE };
        assert_eq!(parse_cmd("raw mods shift+option"), Some(ScriptCmd::Raw(RawCmd::Mods(alt_shift))));
        assert_eq!(parse_cmd("raw key Escape down"), Some(ScriptCmd::Raw(RawCmd::Key(Key::Escape, Some(true)))));
        assert!(Script::parse("raw tap 1 2").raw.is_some());
        assert!(Script::parse("clickpx 1 2").raw.is_none());

        let mut inj = RawInjector::default();
        inj.frames.push_back(vec![Event::PointerMoved(Pos2::new(5.0, 5.0))]);
        let mut raw = egui::RawInput {
            events: vec![Event::PointerMoved(Pos2::new(99.0, 99.0)), Event::ModifiersChanged(Modifiers::SHIFT)],
            ..Default::default()
        };
        inj.apply(&mut raw);
        assert_eq!(raw.events, vec![Event::PointerMoved(Pos2::new(5.0, 5.0))]);
    }

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
        assert_eq!(parse_cmd("raw paste"), Some(ScriptCmd::Raw(RawCmd::Paste("Varstotas".into()))));
        assert_eq!(parse_cmd("raw copy"), Some(ScriptCmd::Raw(RawCmd::Copy)));
        assert_eq!(parse_cmd("raw rclick 3 4"), Some(ScriptCmd::Raw(RawCmd::RightClick(Pos2::new(3.0, 4.0)))));
        assert_eq!(
            parse_cmd("export_png a.png 640 480 transparent"),
            Some(ScriptCmd::ExportPng(
                "a.png".into(),
                Some([640, 480]),
                crate::app::ExportContent { transparent: true, ..Default::default() }
            ))
        );
        assert_eq!(
            parse_cmd("export_png a.png noaxes nodims"),
            Some(ScriptCmd::ExportPng(
                "a.png".into(),
                None,
                crate::app::ExportContent { axes: false, dims: false, ..Default::default() }
            ))
        );
        assert_eq!(parse_cmd("open x.vrst"), Some(ScriptCmd::Open("x.vrst".into())));
        assert_eq!(parse_cmd("action Flip_Red"), Some(ScriptCmd::Action("flip_red".into())));
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
