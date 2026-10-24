//! Debug helpers driven by env vars (used for automated visual checks):
//! - `VARSTOTAS_SELECT_ALL=1` selects everything after ~20 frames.
//! - `VARSTOTAS_SCREENSHOT=<file.bmp>` captures the window after ~40 frames, writes a BMP, quits.
//! - `VARSTOTAS_SCRIPT=<file>` drives tools with scripted input (see [`ScriptCmd`] for the
//!   command list), taking screenshots along the way; quits at the end of the script.

use crate::ui::tray_lib::key_names;
use egui::{Context, Event, ViewportCommand};

/// Repository fixtures for opt-in debug QA only; never read during normal app startup.
pub(crate) fn qa_examples_dir() -> Option<std::path::PathBuf> {
    #[cfg(debug_assertions)]
    if std::env::var_os("VARSTOTAS_SCRIPT").is_some() {
        return std::env::var_os("VARSTOTAS_QA_EXAMPLES").filter(|p| !p.is_empty()).map(std::path::PathBuf::from);
    }
    None
}

#[derive(Default)]
pub struct DebugCapture {
    path: Option<String>,
    frame: u32,
    requested: bool,
    /// Set once the capture is written: the app should close without the save prompt.
    pub force_close: bool,
    script: Option<Script>,
    /// Last occlusion state of the window seen by a scripted run (logged when it changes).
    occluded: Option<bool>,
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
    /// A window screenshot (a script's `screenshot` or the capture) is on its way.
    pub fn screenshot_pending(&self) -> bool {
        (self.requested && !self.force_close) || self.script.as_ref().is_some_and(|s| s.shot.is_some())
    }

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

    /// Returns true on the frame when the debug "select all" should happen.
    pub fn update(&mut self, ctx: &Context) -> bool {
        if self.script_active() {
            let occ = ctx.input(|i| i.viewport().occluded);
            if occ != self.occluded {
                self.occluded = occ;
                log::info!("script: window occluded={occ:?}");
            }
        }
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
    /// `sheetp <sub> x y [x y] [steps]` — like `raw <sub>` with points in paper mm on the
    /// Sheets window's preview (pointer subcommands only).
    RawSheet(String, Vec<f64>),
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
    /// `toolbar KEY …`: the toolbar's layout (Preferences → Toolbar), `toolbar all` = every button.
    Toolbar(String),
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
    /// `texture <path> [width_mm]` — new material from an image (Materials → + → New from Texture…).
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
        "scroll" => RawCmd::Scroll(p(0)?, *n.get(2)?),
        "pinch" => RawCmd::Pinch(p(0)?, *n.get(2)?),
        "wheel" => RawCmd::Wheel(p(0)?, *n.get(2)?),
        "twist" => RawCmd::Twist(p(0)?, *n.get(2)?),
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
    /// `raw scroll x y dy` — move, then a trackpad swipe of `dy` points (negative: the
    /// content moves up, i.e. scroll down).
    Scroll(Pos2, f32),
    Pinch(Pos2, f32),
    Wheel(Pos2, f32),
    Twist(Pos2, f32),
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
        "sheetp" => {
            let (sub, args) = rest.split_once(char::is_whitespace)?;
            let v: Vec<f64> = args.split(|c: char| c.is_whitespace() || c == ',').filter_map(|t| t.parse().ok()).collect();
            ScriptCmd::RawSheet(sub.to_string(), v)
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
        "toolbar" => ScriptCmd::Toolbar(rest.to_string()),
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
            "realistic" => crate::app::FaceStylePref::Realistic,
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
        // The action name is case-insensitive; its arguments (paths, script code) keep their case.
        "action" if !rest.is_empty() => ScriptCmd::Action(match rest.split_once(' ') {
            Some((name, args)) => format!("{} {args}", name.to_ascii_lowercase()),
            None => rest.to_ascii_lowercase(),
        }),
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
    /// Wait for the path traced render (and a scene batch) to finish.
    pt_wait: bool,
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
        let raw = cmds.iter().any(|c| matches!(c, ScriptCmd::Raw(_) | ScriptCmd::RawWorld(..) | ScriptCmd::RawSheet(..))
            || matches!(c, ScriptCmd::Action(name) if name.starts_with("click_spot "))).then(RawInjector::default);
        Self {
            cmds,
            raw,
            pc: 0,
            wait: 0,
            pt_wait: false,
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
        if script.pt_wait {
            let busy = self.viewport.pt.as_ref().is_some_and(|j| j.running || j.restart || j.save.is_some()) || self.render_window.batch.is_some() || self.anim_window.busy() || self.online.busy() || self.models.busy();
            if busy {
                return;
            }
            script.pt_wait = false;
        }
        // A Python script started by `run_script x.py` runs to its end first; so do
        // bridge calls waiting for a confirmation that the script itself may answer.
        if self.bridge.python.is_some() && self.bridge.confirm.is_empty() {
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
            ScriptCmd::Toolbar(keys) => {
                self.prefs.toolbar = if keys.trim() == "all" {
                    crate::ui::toolbar::Item::all().iter().map(|i| i.key()).collect()
                } else {
                    keys.split_whitespace().map(String::from).collect()
                };
            }
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
            ScriptCmd::Action(name) => self.script_action(&name, ctx),
            ScriptCmd::Raw(r) => self.queue_raw(r),
            ScriptCmd::RawWorld(sub, pts, steps) => {
                let mut txt = sub.clone();
                // (projected in the active pane; raw positions count from the whole view's corner)
                let pane = self.viewport.rect.min - self.viewport.view_rect.min;
                for p in &pts {
                    let s = self.script_screen(*p) + pane;
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
            ScriptCmd::RawSheet(sub, v) => {
                let Some((o, k)) = self.sheets.preview else {
                    log::warn!("script: sheetp without the Sheets window");
                    return;
                };
                let origin = self.viewport.view_rect.min;
                let mut txt = sub.clone();
                for c in v.as_chunks::<2>().0 {
                    let s = o + egui::vec2(c[0] as f32, c[1] as f32) * k - origin.to_vec2();
                    txt.push_str(&format!(" {} {}", s.x, s.y));
                }
                if v.len() % 2 == 1 {
                    txt.push_str(&format!(" {}", v[v.len() - 1]));
                }
                match parse_raw(&txt) {
                    Some(r) => self.queue_raw(r),
                    None => log::warn!("script: bad sheetp `{txt}`"),
                }
            }
        }
    }

    /// `action <name>`: menu actions by name.
    fn script_action(&mut self, name: &str, ctx: &Context) {
        use crate::app::Action;
        use varstotas_kernel::IntersectScope;
        if name == "console_close" { self.console.open = false; return; }
        if let Some(value) = name.strip_prefix("xray ") {
            self.prefs.xray = value.trim() == "on";
            return;
        }
        if let Some(value) = name.strip_prefix("triangles ") {
            self.prefs.show_triangles = value.trim() == "on";
            return;
        }
        if let Some(name) = name.strip_prefix("click_spot ") {
            let spots = crate::ui::tray_lib::spots(self.viewport.view_rect.min);
            let (_, x, y) = spots.iter().find(|(n, _, _)| n == name).expect("visible QA click spot");
            self.queue_raw(RawCmd::Click(egui::pos2(*x as f32, *y as f32)));
            return;
        }
        if let Some(path) = name.strip_prefix("file_template ") {
            self.actions.push(Action::NewFromFileTemplate(path.trim().into()));
            return;
        }
        if let Some(path) = name.strip_prefix("templates_dir ") {
            self.prefs.templates_dir = if path.trim() == "default" { None } else { Some(path.trim().into()) };
            self.ensure_templates_dir();
            return;
        }
        if name == "templates_folder" {
            if let Some(dir) = crate::templates::directory(self.prefs.templates_dir.as_deref()) {
                crate::ui::extensions::reveal(&dir);
            }
            return;
        }

        // Realistic style: `env key=value…`, `hdri <path|none>`, `mat_pbr <material> key=value…`,
        // `mat_map <material> <map> <path|none>`.
        if let Some(args) = name.strip_prefix("env ") {
            let mut e = self.doc.model.environment;
            for kv in args.split_whitespace() {
                let Some((k, v)) = kv.split_once('=') else { continue };
                let f = v.parse::<f32>().ok();
                match k {
                    "sky" => {
                        e.sky = match v {
                            "color" => varstotas_kernel::SkyKind::Color,
                            "hdri" => varstotas_kernel::SkyKind::Hdri,
                            "physical" => varstotas_kernel::SkyKind::Physical,
                            _ => varstotas_kernel::SkyKind::Sky,
                        }
                    }
                    "color" => {
                        let c: Vec<f32> = v.split(',').filter_map(|x| x.parse().ok()).collect();
                        if let [r, g, b] = c[..] {
                            e.color = [r, g, b];
                        }
                    }
                    "rotation" => e.rotation_deg = f.unwrap_or(e.rotation_deg),
                    "intensity" => e.intensity = f.unwrap_or(e.intensity),
                    "sun" => e.sun = f.unwrap_or(e.sun),
                    "exposure" => e.exposure = f.unwrap_or(e.exposure),
                    "turbidity" => e.turbidity = f.unwrap_or(e.turbidity),
                    "background" => e.background = matches!(v, "on" | "1" | "true"),
                    "tonemap" => e.tonemap = varstotas_kernel::Tonemap::from_key(v).unwrap_or(e.tonemap),
                    _ => log::warn!("script: env: unknown key {k}"),
                }
            }
            self.actions.push(Action::SetEnvironment(e));
            return;
        }
        // Path tracing: `render_window` / `render_window_close`; `render_set w= h= samples=
        // time= bounces= denoise= match=`; `light_add point|spot|area x y z` (mm) and
        // `light key=value…` (power, color=r,g,b, dir=x,y,z, angle, blend, size=w,h, radius,
        // on) for the last light; `render_start` renders the view and waits for the end (`render_wait`
        // only waits);
        // `render_save <path>` (.png / .exr); `render_scenes <folder>` renders every scene.
        if let Some(i) = name.strip_prefix("select_light ").and_then(|v| v.trim().parse::<usize>().ok()) {
            self.doc.select_light(i);
            return;
        }
        if name == "render_window" || name == "render_window_close" {
            self.render_window.open = name == "render_window";
            return;
        }
        if let Some(args) = name.strip_prefix("render_set ") {
            let mut r = self.doc.model.render;
            for kv in args.split_whitespace() {
                let Some((k, v)) = kv.split_once('=') else { continue };
                let n = v.parse::<f32>().ok();
                let on = matches!(v, "on" | "1" | "true");
                match k {
                    "w" => r.width = n.map_or(r.width, |x| x as u32),
                    "h" => r.height = n.map_or(r.height, |x| x as u32),
                    "samples" => r.samples = n.map_or(r.samples, |x| x as u32),
                    "time" => r.time_limit = n.unwrap_or(r.time_limit),
                    "bounces" => r.bounces = n.map_or(r.bounces, |x| x as u32),
                    "denoise" => r.denoise = on,
                    "match" => r.match_view = on,
                    "preset" => {
                        r.apply_preset(v);
                    }
                    "clamp" => r.clamp = n.unwrap_or(r.clamp),
                    "ai" => r.ai_denoise = on,
                    "dof" => r.dof = varstotas_kernel::RenderDof::ALL.into_iter().find(|d| d.label().eq_ignore_ascii_case(v) || (v == "view" && *d == varstotas_kernel::RenderDof::View)).unwrap_or(r.dof),
                    "fstop" => r.f_stop = n.unwrap_or(r.f_stop),
                    "focus" => r.focus = n.map_or(r.focus, |x| x as f64),
                    "transparent" => r.transparent = on,
                    "exposure" => r.exposure = n.unwrap_or(r.exposure),
                    "tonemap" => r.tonemap = varstotas_kernel::Tonemap::from_key(v),
                    "temp" => r.temperature = n.unwrap_or(r.temperature),
                    "tint" => r.tint = n.unwrap_or(r.tint),
                    "sat" => r.saturation = n.unwrap_or(r.saturation),
                    "contrast" => r.contrast = n.unwrap_or(r.contrast),
                    "vignette" => r.vignette = n.unwrap_or(r.vignette),
                    "reset_grade" => r.reset_grade(),
                    _ => log::warn!("script: render_set: unknown key {k}"),
                }
            }
            self.doc.commit("Render Settings", |m| m.render = r.sanitized());
            return;
        }
        if let Some(args) = name.strip_prefix("light_add ") {
            let v: Vec<&str> = args.split_whitespace().collect();
            let kind = v.first().and_then(|k| varstotas_kernel::LightKind::from_key(k));
            let xyz: Vec<f64> = v.iter().skip(1).filter_map(|x| x.parse().ok()).collect();
            if let (Some(kind), &[x, y, z]) = (kind, &xyz[..]) {
                self.doc.commit("Add Light", |m| m.lights.push(varstotas_kernel::Light::new(kind, DVec3::new(x, y, z))));
            } else {
                log::warn!("script: light_add needs point|spot|area x y z");
            }
            return;
        }
        if let Some(args) = name.strip_prefix("light ") {
            let Some(mut l) = self.doc.model.lights.last().cloned() else { return };
            for kv in args.split_whitespace() {
                let Some((k, v)) = kv.split_once('=') else { continue };
                let nums: Vec<f32> = v.split(',').filter_map(|x| x.parse().ok()).collect();
                match (k, &nums[..]) {
                    ("power", [p]) => l.power = *p,
                    ("color", [r, g, b]) => l.color = [*r, *g, *b],
                    ("dir", [x, y, z]) => l.direction = DVec3::new(*x as f64, *y as f64, *z as f64).normalize_or(DVec3::NEG_Z),
                    ("angle", [a]) => l.spot_angle = *a,
                    ("blend", [b]) => l.spot_blend = *b,
                    ("size", [w, h]) => l.size = [*w, *h],
                    ("radius", [r]) => l.radius = *r,
                    ("on", _) => l.enabled = matches!(v, "on" | "1" | "true"),
                    _ => log::warn!("script: light: bad {kv}"),
                }
            }
            self.doc.commit("Edit Light", |m| {
                if let Some(last) = m.lights.last_mut() {
                    *last = l.sanitized();
                }
            });
            return;
        }
        // `anim key=value… out=<path>`: export an animation (source=scenes|orbit seconds= turns=
        // fps= w= h= traced=on|off spp= video=on|off) and wait for it; `anim_window` opens the window.
        if name == "anim_window" {
            self.anim_window.open = true;
            return;
        }
        if let Some(args) = name.strip_prefix("anim ") {
            let mut out = None;
            let w = &mut self.anim_window;
            for kv in args.split_whitespace() {
                let Some((k, v)) = kv.split_once('=') else { continue };
                let n = v.parse::<f64>().ok();
                let on = matches!(v, "on" | "1" | "true");
                match k {
                    "source" => w.source = if v == "orbit" { crate::ui::animation::AnimSource::Orbit } else { crate::ui::animation::AnimSource::Scenes },
                    "seconds" => w.seconds = n.unwrap_or(w.seconds),
                    "turns" => w.turns = n.unwrap_or(w.turns),
                    "fps" => w.fps = n.map_or(w.fps, |x| x as u32),
                    "w" => w.width = n.map_or(w.width, |x| x as u32),
                    "h" => w.height = n.map_or(w.height, |x| x as u32),
                    "traced" => w.traced = on,
                    "spp" => w.spp = n.map_or(w.spp, |x| x as u32),
                    "video" => w.video = on,
                    "out" => out = Some(std::path::PathBuf::from(v)),
                    _ => log::warn!("script: anim: unknown key {k}"),
                }
            }
            if let Some(out) = out {
                self.start_animation(out);
                if let Some(s) = self.debug.script.as_mut() {
                    s.pt_wait = true;
                }
            }
            return;
        }
        if name == "render_wait" {
            if let Some(s) = self.debug.script.as_mut() {
                s.pt_wait = true;
            }
            return;
        }
        if name == "render_start" {
            self.start_render(self.camera.clone());
            if let Some(s) = self.debug.script.as_mut() {
                s.pt_wait = true;
            }
            return;
        }
        if let Some(path) = name.strip_prefix("render_save ") {
            if let Some(job) = &mut self.viewport.pt {
                job.save = Some(std::path::PathBuf::from(path.trim()));
            }
            if let Some(s) = self.debug.script.as_mut() {
                s.pt_wait = true;
            }
            return;
        }
        if let Some(folder) = name.strip_prefix("render_scenes ") {
            let folder = std::path::PathBuf::from(folder.trim());
            let _ = std::fs::create_dir_all(&folder);
            self.render_all_scenes(folder);
            if let Some(s) = self.debug.script.as_mut() {
                s.pt_wait = true;
            }
            return;
        }
        // `fx key=value…`: effects of the Realistic style (`preset=off|fast|balanced|quality` first).
        if let Some(args) = name.strip_prefix("fx ") {
            let mut e = self.doc.model.environment;
            let on = |v: &str| matches!(v, "on" | "1" | "true");
            for kv in args.split_whitespace() {
                let Some((k, v)) = kv.split_once('=') else { continue };
                let f = v.parse::<f32>().ok();
                let x = &mut e.effects;
                match k {
                    "preset" => *x = x.with_preset(v).unwrap_or(*x),
                    "soft" => x.soft_shadows = on(v),
                    "softness" => x.shadow_softness = f.unwrap_or(x.shadow_softness),
                    "ao" => x.ao = on(v),
                    "ao_radius" => x.ao_radius = f.unwrap_or(x.ao_radius),
                    "ao_strength" => x.ao_strength = f.unwrap_or(x.ao_strength),
                    "ssr" => x.reflections = on(v),
                    "ssr_strength" => x.reflection_strength = f.unwrap_or(x.reflection_strength),
                    "bloom" => x.bloom = on(v),
                    "bloom_strength" => x.bloom_strength = f.unwrap_or(x.bloom_strength),
                    "threshold" => x.bloom_threshold = f.unwrap_or(x.bloom_threshold),
                    "dof" => x.dof = on(v),
                    "focus" => x.focus_distance = f.unwrap_or(x.focus_distance),
                    "aperture" => x.aperture = f.unwrap_or(x.aperture),
                    "fog" => x.fog = on(v),
                    "fog_distance" => x.fog_distance = f.unwrap_or(x.fog_distance),
                    "fog_height" => x.fog_height = f.unwrap_or(x.fog_height),
                    "fog_color" => {
                        let c: Vec<f32> = v.split(',').filter_map(|x| x.parse().ok()).collect();
                        if let [r, g, b] = c[..] {
                            x.fog_color = [r, g, b];
                        }
                    }
                    "vignette" => x.vignette = f.unwrap_or(x.vignette),
                    "clay" => x.clay = on(v),
                    "stylize" => x.stylize = varstotas_kernel::Stylize::from_key(v).unwrap_or(x.stylize),
                    "hq" => x.high_quality = on(v),
                    _ => log::warn!("script: fx: unknown key {k}"),
                }
            }
            self.actions.push(Action::SetEnvironment(e));
            return;
        }
        // `mat_shader <material> <preset name without spaces|none>`, `mat_param <material> <i> <value>`.
        if let Some(args) = name.strip_prefix("mat_shader ") {
            let v: Vec<&str> = args.split_whitespace().collect();
            if let [mat, preset] = v[..] {
                let src = crate::shaders::PRESETS
                    .iter()
                    .find(|(n, _)| n.replace(' ', "").eq_ignore_ascii_case(preset))
                    .map(|(_, s)| *s);
                if src.is_none() && preset != "none" {
                    log::warn!("script: mat_shader: unknown preset {preset}");
                    return;
                }
                self.edit_pbr(&mat.replace('_', " "), |p| p.shader = src.map(varstotas_kernel::CustomShader::new));
            } else {
                log::warn!("script: mat_shader needs <material> <preset|none>");
            }
            return;
        }
        // `shader_editor <material>` opens the shader editor; `shader_code <text>` (`\n` = new
        // line) replaces its code as if typed (applied after the usual pause).
        if let Some(mat) = name.strip_prefix("shader_editor ") {
            let mat = mat.trim().replace('_', " ");
            let code = self.doc.model.materials.values().find(|m| m.name == mat).and_then(|m| m.pbr.as_ref()?.shader.as_ref()).map(|sh| sh.source.clone());
            self.tray.shader_edit = Some(crate::ui::realistic::ShaderEdit::new(mat, code.unwrap_or_default()));
            return;
        }
        if name == "shader_editor_close" {
            self.tray.shader_edit = None;
            return;
        }
        if let Some(code) = name.strip_prefix("shader_code ") {
            if let Some(ed) = &mut self.tray.shader_edit {
                ed.code = code.replace("\\n", "\n");
                ed.changed = Some(std::time::Instant::now());
            }
            return;
        }
        if let Some(args) = name.strip_prefix("mat_param ") {
            let v: Vec<&str> = args.split_whitespace().collect();
            if let [mat, i, val] = v[..]
                && let (Ok(i), Ok(val)) = (i.parse::<usize>(), val.parse::<f32>())
            {
                self.edit_pbr(&mat.replace('_', " "), |p| {
                    if let Some(sh) = &mut p.shader {
                        while sh.params.len() <= i {
                            let k = sh.params.len();
                            sh.params.push(sh.param(k));
                        }
                        sh.params[i] = val;
                    }
                });
            } else {
                log::warn!("script: mat_param needs <material> <index> <value>");
            }
            return;
        }
        if let Some(path) = name.strip_prefix("hdri ") {
            if path.trim() == "none" {
                self.doc.mutate_untracked(|m| m.hdri = None);
            } else {
                self.load_hdri(std::path::Path::new(path.trim()));
            }
            return;
        }
        if let Some(args) = name.strip_prefix("mat_pbr ") {
            let mut it = args.split_whitespace();
            let Some(mat) = it.next() else { return };
            let mat = mat.replace('_', " ");
            let kvs: Vec<(String, String)> =
                it.filter_map(|kv| kv.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))).collect();
            self.edit_pbr(&mat, |p| {
                for (k, v) in &kvs {
                    let f = v.parse::<f32>().unwrap_or(0.0);
                    match k.as_str() {
                        "metallic" => p.metallic = f,
                        "roughness" => p.roughness = f,
                        "transmission" | "glass" => p.transmission = f,
                        "ior" => p.ior = f,
                        "strength" => p.emission_strength = f,
                        "normal_scale" => p.normal_scale = f,
                        "clearcoat" | "coat" => p.clearcoat = f,
                        "coat_rough" => p.clearcoat_roughness = f,
                        "sheen" => p.sheen = f,
                        "sheen_tint" => p.sheen_tint = f,
                        "aniso" | "anisotropy" => p.anisotropy = f,
                        "anti_tiling" => p.anti_tiling = f,
                        "bevel" => p.bevel = f,
                        "emission" => {
                            let c: Vec<f32> = v.split(',').filter_map(|x| x.parse().ok()).collect();
                            if let [r, g, b] = c[..] {
                                p.emission = [r, g, b];
                            }
                        }
                        _ => log::warn!("script: mat_pbr: unknown key {k}"),
                    }
                }
            });
            return;
        }
        // Texture slots of the current material (Material Editor → Textures): `mat_slot
        // <slot>|<path|none>`, `mat_slot_export <slot>|<path>`, `mat_maps <folder>`,
        // `mat_slot_downscale <slot>`, `mat_slot_edit <slot>` (writes the file, no program).
        if let Some(args) = name.strip_prefix("mat_slot")
            && let Some((cmd, rest)) = args.split_once(' ')
        {
            let cur = self.doc.current_material.as_ref().map(|c| c.name.clone()).unwrap_or_default();
            let (slot, path) = rest.split_once('|').map_or((rest.trim(), ""), |(a, b)| (a.trim(), b.trim()));
            let Some(slot) = crate::textures::SLOTS.iter().find(|s| **s == slot).copied() else {
                log::warn!("script: unknown texture slot {slot}");
                return;
            };
            match cmd {
                "" if path == "none" => self.set_material_slots(&cur, vec![(slot, None)], "Clear Texture"), // i18n-ignore
                "" => self.load_slot_file(&cur, slot, std::path::Path::new(path)),
                "_export" => self.export_slot(&cur, slot, std::path::Path::new(path)),
                "_downscale" => self.downscale_slot(&cur, slot),
                "_edit" => self.edit_slot_externally(&cur, slot),
                _ => log::warn!("script: unknown mat_slot command {cmd}"),
            }
            return;
        }
        // `copy_file <src>|<dst>`: stands in for another program saving a file.
        if let Some(args) = name.strip_prefix("copy_file ") {
            if let Some((src, dst)) = args.split_once('|')
                && let Err(e) = std::fs::copy(src.trim(), dst.trim())
            {
                log::warn!("script: copy_file: {e}");
            }
            return;
        }
        if let Some(dir) = name.strip_prefix("mat_maps ") {
            let cur = self.doc.current_material.as_ref().map(|c| c.name.clone()).unwrap_or_default();
            self.load_map_folder(&cur, std::path::Path::new(dir.trim()));
            return;
        }
        if let Some(args) = name.strip_prefix("mat_map ") {
            let v: Vec<&str> = args.splitn(3, ' ').collect();
            if let [mat, map, path] = v[..]
                && let Some(map) = varstotas_kernel::PBR_MAPS.iter().find(|m| **m == map)
            {
                let mat = mat.replace('_', " ");
                if path == "none" {
                    self.set_pbr_map(&mat, map, None);
                } else if let Some((mid, _)) = self.doc.model.materials.iter().find(|(_, m)| m.name == mat) {
                    self.load_pbr_map(mid, map, std::path::Path::new(path));
                }
            } else {
                log::warn!("script: mat_map needs <material> <map> <path|none>");
            }
            return;
        }
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
        // `unfold_opts [scale=N] [paper=fit|a4|a3|a2] [landscape] [notabs] [nonumbers] [softfolds]
        // [tab=W]`: Unfold options (unlisted flags reset to the defaults).
        if let Some(args) = name.strip_prefix("unfold_opts") {
            let mut u = crate::ui::unfold::UnfoldState::default();
            u.open = self.unfold.open;
            for a in args.split_whitespace() {
                match a.split_once('=') {
                    Some(("scale", v)) => u.scale_n = v.parse().unwrap_or(1.0),
                    Some(("tab", v)) => u.opts.tab_width = v.parse().unwrap_or(u.opts.tab_width),
                    Some(("paper", v)) => {
                        u.paper = crate::ui::unfold::Paper::ALL
                            .into_iter()
                            .find(|p| p.name().to_ascii_lowercase().starts_with(&v.to_ascii_lowercase()))
                            .unwrap_or(u.paper)
                    }
                    _ => match a {
                        "landscape" => u.landscape = true,
                        "notabs" => u.opts.tabs = false,
                        "nonumbers" => u.opts.numbers = false,
                        "softfolds" => u.opts.soft_folds = true,
                        _ => log::warn!("script: unknown unfold_opts flag {a}"),
                    },
                }
            }
            self.unfold = u;
            return;
        }
        // `export_unfold <path>`: write the unfolding of the selection (svg / pdf / dxf).
        if let Some(path) = name.strip_prefix("export_unfold ") {
            self.export_unfold(std::path::Path::new(path.trim()));
            return;
        }
        // Sheets: `sheet_add [a4|a3|a2|a1] [portrait]`, `sheet_viewport [scene=Name] [scale=N]
        // [frame] [x y w h]` (the current view unless a scene is named), `sheet_text x y size text`,
        // `sheet_title title|author|date text`, `sheet_select N`, `export_sheets <path>`.
        if let Some(args) = name.strip_prefix("sheet_add") {
            let a: Vec<&str> = args.split_whitespace().collect();
            let paper = a.iter().find(|s| s.len() == 2 && s.to_ascii_lowercase().starts_with('a')).copied().unwrap_or("A3");
            self.add_sheet(paper, a.contains(&"portrait"));
            self.sheets.open = true;
            return;
        }
        if let Some(args) = name.strip_prefix("sheet_viewport") {
            let Some(sheet) = self.doc.model.sheets.get(self.sheets.current) else {
                log::warn!("script: sheet_viewport without a sheet");
                return;
            };
            let mut vp = self.new_viewport(sheet);
            let mut nums = Vec::new();
            for a in args.split_whitespace() {
                match a.split_once('=') {
                    Some(("scene", v)) => vp.scene = Some(v.replace('_', " ")),
                    Some(("scale", v)) => vp.scale = v.parse().ok(),
                    _ if a == "frame" => vp.frame = true,
                    _ if a == "hidden" => vp.hidden = true,
                    _ if a == "autodims" => vp.auto_dims = true,
                    _ => nums.extend(a.parse::<f64>().ok()),
                }
            }
            if let [x, y, w, h] = nums[..] {
                vp.min = DVec2::new(x, y);
                vp.size = DVec2::new(w, h);
            }
            self.edit_sheet("Add Viewport", |s| s.viewports.push(vp));
            return;
        }
        if let Some(args) = name.strip_prefix("sheet_text ") {
            let mut it = args.splitn(4, ' ');
            let n: Vec<f64> = it.by_ref().take(3).filter_map(|t| t.parse().ok()).collect();
            let text = it.next().unwrap_or("Text").replace("\\n", "\n");
            if let [x, y, size] = n[..] {
                let t = varstotas_kernel::layout::SheetText { at: DVec2::new(x, y), size, text };
                self.edit_sheet("Add Text", |s| s.texts.push(t));
            }
            return;
        }
        if let Some(args) = name.strip_prefix("sheet_title ") {
            let (field, v) = args.split_once(' ').unwrap_or((args, ""));
            let (field, v) = (field.to_string(), v.to_string());
            self.edit_sheet("Edit Sheet", |s| {
                let tb = s.title.get_or_insert_with(Default::default);
                match field.as_str() {
                    "author" => tb.author = v,
                    "date" => tb.date = v,
                    _ => tb.title = v,
                }
            });
            return;
        }
        // `sheet_views [scale=N]`: standard views on the current sheet; `sheet_table x y [h]`.
        if let Some(args) = name.strip_prefix("sheet_views") {
            let scale = args.trim().strip_prefix("scale=").and_then(|v| v.parse().ok());
            let Some(sheet) = self.doc.model.sheets.get(self.sheets.current) else { return };
            let views = crate::layout::standard_views(&self.doc.model, sheet, scale);
            self.edit_sheet("Standard Views", |s| s.viewports.extend(views));
            return;
        }
        if let Some(args) = name.strip_prefix("sheet_table ") {
            let n: Vec<f64> = args.split_whitespace().filter_map(|t| t.parse().ok()).collect();
            if let [x, y, ..] = n[..] {
                let t = varstotas_kernel::layout::SheetTable { at: DVec2::new(x, y), size: n.get(2).copied().unwrap_or(3.0) };
                self.edit_sheet("Add Table", |s| s.tables.push(t));
            }
            return;
        }
        // `snap on|off|toggle` (master switch), `snap_kind <key> on|off` (one inference),
        // `snap_all on|off`.
        if let Some(arg) = name.strip_prefix("snap ") {
            match arg.trim() {
                "toggle" => self.actions.push(Action::ToggleSnapping),
                v => self.prefs.snapping.enabled = v == "on",
            }
            crate::inference::settings::publish(&self.prefs.snapping);
            return;
        }
        if let Some(args) = name.strip_prefix("snap_kind ") {
            let mut it = args.split_whitespace();
            if let (Some(k), Some(v)) = (it.next().and_then(crate::inference::InferenceKind::from_key), it.next()) {
                self.prefs.snapping.set(k, v == "on");
            } else {
                eprintln!("[debug] snap_kind: unknown kind in {args:?}");
            }
            crate::inference::settings::publish(&self.prefs.snapping);
            return;
        }
        // `workplane add ox oy oz nx ny nz` | `activate N|none` | `angles tilt az spin` |
        // `origin x y z` | `hide N` | `show N` | `remove N` (N is 1-based).
        if let Some(args) = name.strip_prefix("workplane ") {
            self.workplane_cmd(args);
            return;
        }
        // `grid on|off [step=mm] [major=N]`.
        if let Some(args) = name.strip_prefix("grid ") {
            for a in args.split_whitespace() {
                match a {
                    "on" => self.prefs.grid.show = true,
                    "off" => self.prefs.grid.show = false,
                    _ => {
                        if let Some(v) = a.strip_prefix("step=").and_then(|v| v.parse().ok()) {
                            self.prefs.grid.step = v;
                        } else if let Some(v) = a.strip_prefix("major=").and_then(|v| v.parse().ok()) {
                            self.prefs.grid.major = v;
                        }
                    }
                }
            }
            crate::inference::settings::publish_grid(&self.prefs.grid);
            return;
        }
        // `snap_inc move|rotate|scale on|off [step=V]`.
        if let Some(args) = name.strip_prefix("snap_inc ") {
            let inc = &mut self.prefs.snapping.increments;
            let mut it = args.split_whitespace();
            let what = it.next().unwrap_or("");
            for a in it {
                let on = match a {
                    "on" => Some(true),
                    "off" => Some(false),
                    _ => None,
                };
                let step = a.strip_prefix("step=").and_then(|v| v.parse::<f64>().ok());
                let (flag, val) = match what {
                    "move" => (&mut inc.move_on, &mut inc.move_step),
                    "rotate" => (&mut inc.rotate_on, &mut inc.angle_step),
                    "scale" => (&mut inc.scale_on, &mut inc.scale_step),
                    _ => {
                        eprintln!("[debug] snap_inc: unknown {what:?}");
                        break;
                    }
                };
                if let Some(on) = on {
                    *flag = on;
                }
                if let Some(v) = step {
                    *val = v;
                }
            }
            crate::inference::settings::publish(&self.prefs.snapping);
            return;
        }
        if let Some(v) = name.strip_prefix("snap_all ") {
            for k in crate::inference::settings::all_kinds() {
                self.prefs.snapping.set(k, v.trim() == "on");
            }
            crate::inference::settings::publish(&self.prefs.snapping);
            return;
        }
        if let Some(i) = name.strip_prefix("sheet_select ") {
            self.sheets.current = i.trim().parse::<usize>().unwrap_or(1).saturating_sub(1);
            self.sheets.selected = None;
            return;
        }
        if let Some(path) = name.strip_prefix("export_sheets ") {
            self.export_sheets(std::path::Path::new(path.trim()));
            return;
        }
        // `vector_opts <fit|N> [hidden] [noprofiles] [nodims] [nosections]`: 2D Drawing options.
        if let Some(args) = name.strip_prefix("vector_opts ") {
            let mut it = args.split_whitespace();
            let mut o = crate::vector_export::VectorOptions { scale: it.next().and_then(|s| s.parse().ok()), ..Default::default() };
            for a in it {
                match a {
                    "hidden" => o.hidden = true,
                    "noprofiles" => o.profiles = false,
                    "nodims" => o.dims = false,
                    "nosections" => o.sections = false,
                    _ => log::warn!("script: unknown vector_opts flag {a}"),
                }
            }
            self.vector_opts = o;
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
        // `run_script <path>` / `script <code>` (`\n` = new line): Rhai scripts (Script Console).
        if let Some(path) = name.strip_prefix("run_script ") {
            self.run_script_file(std::path::Path::new(path.trim()));
            return;
        }
        // `plugin <name>` / `plugin <name> @ <file>`: start a plugin like its menu entry (the
        // parameter dialog opens when it has parameters); `reload_plugins`.
        if let Some(arg) = name.strip_prefix("plugin ") {
            let (n, file) = match arg.split_once(" @ ") {
                Some((n, f)) => (n, Some(std::path::PathBuf::from(f.trim()))),
                None => (arg, None),
            };
            match self.plugin_by_name(n) {
                Some(i) => self.actions.push(crate::app::Action::Plugin(i, file)),
                None => log::warn!("script: no plugin named {n}"),
            }
            return;
        }
        // `library_save <name>` (the selection), `library_insert <name>` (then click),
        // `library_refresh`, `components` (open the tray card).
        if let Some(n) = name.strip_prefix("library_save ") {
            self.actions.push(crate::app::Action::SaveToLibrary(n.trim().to_string()));
            return;
        }
        if let Some(n) = name.strip_prefix("library_insert ") {
            match self.library_index(n) {
                Some(i) => self.actions.push(crate::app::Action::PlaceComponent(i)),
                None => log::warn!("script: no library item {n}"),
            }
            return;
        }
        // `generator <name>` (then click to place), `gen key=value …` (the selected generated
        // components), `gen_reset`, `make_unique`.
        if let Some(n) = name.strip_prefix("generator ") {
            if !self.library.gens.scanned {
                self.refresh_generators();
            }
            match self.generator_source(n) {
                Some(src) => self.actions.push(Action::PlaceGenerator(src, Default::default())),
                None => log::warn!("script: no generator named {n}"),
            }
            return;
        }
        if let Some(args) = name.strip_prefix("gen ") {
            let values = args
                .split_whitespace()
                .filter_map(|a| a.split_once('='))
                .map(|(k, v)| {
                    let v = match v {
                        "true" | "false" => varstotas_kernel::GenValue::Bool(v == "true"),
                        _ => varstotas_kernel::GenValue::Text(v.replace('_', " ")),
                    };
                    (k.to_string(), v)
                })
                .collect();
            self.actions.push(Action::SetGenValues(values, false));
            return;
        }
        // Generator Editor: `gen_editor [name]` (open that generator, else a new one),
        // `ged key=value …` (preview values), `ged_sub old|new` (edit the source),
        // `ged_save`, `ged_apply`, `ged_place`, `ged_close`.
        if name == "gen_editor" || name.starts_with("gen_editor ") {
            if !self.library.gens.scanned {
                self.refresh_generators();
            }
            // `gen_editor doc:<name>`: the document's generator (In Model)
            if let Some(n) = name.strip_prefix("gen_editor doc:")
                && let Some(g) = self.doc.model.generators.get(n.trim())
            {
                let (src, n) = (g.source.clone(), g.name.clone());
                self.gen_editor.edit_doc(&src, &n);
                return;
            }
            match name.strip_prefix("gen_editor ").and_then(|n| self.generator_items().into_iter().find(|g| g.name.eq_ignore_ascii_case(n.trim()) && g.folder != super::ui::generators::DOCUMENT_FOLDER)) {
                Some(it) => self.gen_editor.edit(&it.source, it.file.clone()),
                None => self.gen_editor.new_generator(),
            }
            return;
        }
        if let Some(args) = name.strip_prefix("ged ") {
            for (k, v) in args.split_whitespace().filter_map(|a| a.split_once('=')) {
                let v = match v {
                    "true" | "false" => varstotas_kernel::GenValue::Bool(v == "true"),
                    _ => varstotas_kernel::GenValue::Text(v.replace('_', " ")),
                };
                if let Ok(g) = &self.gen_editor.parsed
                    && let Some(v) = g.param(k).and_then(|p| p.normalize(&v).ok())
                {
                    self.gen_editor.values.insert(k.to_string(), v);
                }
            }
            return;
        }
        if let Some(args) = name.strip_prefix("ged_sub ") {
            if let Some((a, b)) = args.split_once('|') {
                self.gen_editor.source = self.gen_editor.source.replacen(a, b, 1);
            }
            return;
        }
        if name == "ged_close" {
            self.gen_editor.open = false;
            return;
        }
        if name == "ged_save" {
            self.actions.push(Action::GenEditorSave);
            return;
        }
        if name == "ged_apply" {
            self.actions.push(Action::GenEditorApply);
            return;
        }
        if name == "ged_place" {
            self.actions.push(Action::PlaceGenerator(self.gen_editor.source.clone(), self.gen_editor.values.clone()));
            return;
        }
        if name == "gen_reset" {
            self.actions.push(Action::SetGenValues(Default::default(), true));
            return;
        }
        if name == "make_unique" {
            self.actions.push(Action::MakeUnique);
            return;
        }
        // Components with parameters: `rig_edit` (Window → Component Parameters for the
        // selection), editor rows `rig_param <@param text>`, `rig_formula <@formula text>`,
        // `rig_joint part|motion(0 rotate,1 slide,2 show,3 paint,4 set parameter)|param|from|to|ox oy oz|ax ay az|expr|key`,
        // `rig_script <file>`, `rig_clear`, then `rig_apply`, `rig_remove`, `rig_pick <joint>`;
        // values of the selected copies: `rig key=value …`, `rig_reset`.
        // The log window's entries: `log_clear`.
        use crate::ui::printing_tools::{PrintTab,PrintMap};
        for (action,tab) in [("print_analysis",PrintTab::Analysis),("print_bed",PrintTab::Bed),("print_split",PrintTab::Split),("print_export",PrintTab::Export)] {if name==action{self.open_print_tools(ctx,tab);return;}}
        for (action,mode) in [("print_drop",varstotas_kernel::printing::preparation::Placement::Drop),("print_center",varstotas_kernel::printing::preparation::Placement::Center),("print_arrange",varstotas_kernel::printing::preparation::Placement::Arrange)] {if name==action{self.print_place_action(mode);return;}}
        if name=="print_lay"{self.print_lay_action();return;}
        if name=="print_cut"||name=="print_cut_to_bed"{self.start_print_cut(ctx,name=="print_cut_to_bed");return;}
        if name=="print_open_slicer"{self.open_print_slicer();return;}
        if let Some(path)=name.strip_prefix("print_export_to "){self.start_print_export(ctx,path.into());return;}
        if let Some(path)=name.strip_prefix("print_slicer "){self.prefs.printing.slicer=Some(path.into());return;}
        if let Some(mode)=name.strip_prefix("print_map "){self.printing.map=match mode{"wall"=>PrintMap::Thickness,"overhang"=>PrintMap::Overhang,_=>PrintMap::None};return;}
        if let Some(args)=name.strip_prefix("print_cut_set "){for arg in args.split_whitespace(){if let Some((k,v))=arg.split_once('='){match k{"axis"=>self.printing.cut_normal=match v{"x"=>DVec3::X,"y"=>DVec3::Y,_=>DVec3::Z},"pos"=>if let Ok(v)=v.parse(){self.printing.cut_position=v},"connectors"=>self.printing.connectors.enabled=v=="on",_=>{}}}}return;}
        if name == "print_check" {self.open_printing(ctx, false);return;}
        if name == "print_repair" {self.open_printing(ctx, true);return;}
        if name == "print_run_repair" {self.start_print_job(ctx, true);return;}
        if name == "print_run_check" {self.start_print_job(ctx, false);return;}
        if name == "log_clear" {
            crate::applog::clear();
            return;
        }
        if name == "rig_edit" {
            self.actions.push(Action::RigEditor);
            return;
        }
        if let Some(t) = name.strip_prefix("rig_param ") {
            match varstotas_kernel::GenParam::parse(t) {
                Ok(p) => self.rig_editor.params.push(crate::ui::rig_editor::ParamRow::of(&p)),
                Err(e) => log::warn!("script: rig_param: {e}"),
            }
            return;
        }
        if let Some(t) = name.strip_prefix("rig_formula ") {
            match varstotas_kernel::GenFormula::parse(t) {
                Ok(f) => self.rig_editor.formulas.push(crate::ui::rig_editor::FormulaRow::of(&f)),
                Err(e) => log::warn!("script: rig_formula: {e}"),
            }
            return;
        }
        if let Some(t) = name.strip_prefix("rig_joint ") {
            let f: Vec<&str> = t.split('|').collect();
            let num = |i: usize| f.get(i).and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(0.0);
            let v3 = |i: usize, d: [f64; 3]| {
                let n: Vec<f64> = f.get(i).map(|x| x.split_whitespace().filter_map(|s| s.parse().ok()).collect()).unwrap_or_default();
                if n.len() == 3 { [n[0], n[1], n[2]] } else { d }
            };
            self.rig_editor.joints.push(crate::ui::rig_editor::JointRow {
                part: f.first().map(|s| s.trim().to_string()).unwrap_or_default(),
                motion: num(1) as usize,
                param: f.get(2).map(|s| s.trim().to_string()).unwrap_or_default(),
                from: num(3),
                to: num(4),
                origin: v3(5, [0.0; 3]),
                axis: v3(6, [0.0, 0.0, 1.0]),
                expr: f.get(7).map(|s| s.trim().to_string()).unwrap_or_default(),
                key: f.get(8).map(|s| s.trim().to_string()).unwrap_or_default(),
            });
            return;
        }
        if let Some(p) = name.strip_prefix("rig_script ") {
            match std::fs::read_to_string(p.trim()) {
                Ok(s) => self.rig_editor.script = s,
                Err(e) => log::warn!("script: rig_script {p}: {e}"),
            }
            return;
        }
        if name == "rig_clear" {
            self.rig_editor.params.clear();
            self.rig_editor.formulas.clear();
            self.rig_editor.joints.clear();
            self.rig_editor.script.clear();
            return;
        }
        if name == "rig_apply" {
            self.rig_apply();
            return;
        }
        if name == "rig_remove" {
            self.rig_remove();
            return;
        }
        if let Some(j) = name.strip_prefix("rig_pick ") {
            self.rig_pick(j.trim().parse().unwrap_or(0));
            return;
        }
        if name == "rig_reset" {
            self.actions.push(Action::SetRigValues(Default::default(), true));
            return;
        }
        if let Some(args) = name.strip_prefix("rig ") {
            let values = args
                .split_whitespace()
                .filter_map(|a| a.split_once('='))
                .map(|(k, v)| {
                    let v = match v {
                        "true" | "false" => varstotas_kernel::GenValue::Bool(v == "true"),
                        _ => varstotas_kernel::GenValue::Text(v.replace('_', " ")),
                    };
                    (k.to_string(), v)
                })
                .collect();
            self.actions.push(Action::SetRigValues(values, false));
            return;
        }
        if name == "library_refresh" {
            self.refresh_library();
            return;
        }
        // Preview pictures: `library_preview <item>|<image path>` (`clear` removes it), and the
        // document's `preview_file <image path>`, `preview_view`, `preview_clear`.
        if let Some(rest) = name.strip_prefix("library_preview ") {
            let (n, img) = rest.split_once('|').unwrap_or((rest, "clear"));
            match self.library_index(n) {
                Some(i) => {
                    let path = self.library.items[i].path.clone();
                    let img = img.trim();
                    let target = crate::ui::library::PreviewTarget::File(path);
                    self.preview_of_from_file(&target, (img != "clear").then(|| std::path::Path::new(img)));
                }
                None => log::warn!("script: no library item {n}"),
            }
            return;
        }
        if let Some(f) = name.strip_prefix("preview_file ") {
            self.preview_from_file(std::path::Path::new(f.trim()));
            return;
        }
        if name == "preview_view" {
            self.actions.push(Action::PreviewFromView);
            return;
        }
        if name == "preview_clear" {
            self.actions.push(Action::PreviewRemove);
            return;
        }
        // Materials library: `mat_save <model material>`, `mat_use <library / example material>`,
        // `mat_export <model material>|<path>`, `mat_import <path>`, `mat_delete <library file stem>`
        // (asks), `mat_refresh`.
        if let Some(n) = name.strip_prefix("mat_save ") {
            match self.doc.model.materials.iter().find(|(_, m)| m.name == n.trim()) {
                Some((id, _)) => self.save_material_to_library(id),
                None => log::warn!("script: no model material {n}"),
            }
            return;
        }
        if let Some(n) = name.strip_prefix("mat_use ") {
            match self.library_materials(true).into_iter().find(|(_, m)| m.name == n.trim()) {
                Some((_, m)) => {
                    self.use_material(&m);
                }
                None => log::warn!("script: no library material {n}"),
            }
            return;
        }
        if let Some(args) = name.strip_prefix("mat_export ") {
            let (n, path) = args.split_once('|').unwrap_or((args, "material.vrstmat"));
            match self.doc.model.materials.iter().find(|(_, m)| m.name == n.trim()) {
                Some((id, _)) => self.export_material_to(id, std::path::Path::new(path.trim())),
                None => log::warn!("script: no model material {n}"),
            }
            return;
        }
        if let Some(path) = name.strip_prefix("mat_import ") {
            self.import_material_from(std::path::Path::new(path.trim()));
            return;
        }
        if let Some(n) = name.strip_prefix("mat_delete ") {
            self.mat_library.delete_ask = self.mat_library.items.iter().filter_map(|i| i.path.clone()).find(|p| p.file_stem().is_some_and(|s| s == n.trim()));
            return;
        }
        // `drop_image <path>|<x> <y> <z>`: an image file dropped over that model point.
        if let Some(args) = name.strip_prefix("drop_image ") {
            let (path, at) = args.split_once('|').unwrap_or((args, ""));
            let v: Vec<f64> = at.split_whitespace().filter_map(|t| t.parse().ok()).collect();
            let local = (v.len() == 3).then(|| self.script_screen(varstotas_kernel::glam::DVec3::new(v[0], v[1], v[2])));
            self.drop_image(std::path::Path::new(path.trim()), local);
            return;
        }
        // `procedural <kind>` / `procedural plugin:<file stem>`: the Procedural Material window
        // with that pattern.
        if let Some(k) = name.strip_prefix("procedural ") {
            use crate::ui::procedural::{Kind, Params};
            if let Some(kind) = Kind::ALL.into_iter().find(|x| x.key() == k.trim()) {
                self.procedural.params = Params::new(kind);
                self.procedural.plugin = None;
            }
            if let Some(stem) = k.trim().strip_prefix("plugin:") {
                self.procedural.plugin = self.plugins.list.iter().find(|p| p.texture_generator.is_some() && p.path.file_stem().is_some_and(|s| s == stem)).map(|p| p.path.clone());
            }
            self.procedural.open = true;
            return;
        }
        // `mat_replace <name>`: the model material's uses repainted with the current material.
        if let Some(n) = name.strip_prefix("mat_replace ") {
            if let Some(id) = self.doc.model.materials.iter().find(|(_, m)| m.name == n.trim()).map(|(id, _)| id) {
                self.replace_with_current_material(id);
            }
            return;
        }
        // `tex_look hue=30 saturation=-1 lightness=0 cutoff=0.5 colorize=1 decal=0 world=1`:
        // the current material's texture look (one undo step).
        if let Some(args) = name.strip_prefix("tex_look ") {
            let Some(mut l) = self.doc.current_material.as_ref().and_then(|c| c.image.as_ref()).map(|i| i.look) else { return };
            for (k, v) in args.split_whitespace().filter_map(|t| t.split_once('=')) {
                let f = v.parse::<f32>().unwrap_or(0.0);
                match k {
                    "hue" => l.hue = f,
                    "saturation" => l.saturation = f,
                    "lightness" => l.lightness = f,
                    "cutoff" => l.alpha_cutoff = f,
                    "colorize" => l.colorize = f != 0.0,
                    "decal" => l.decal = f != 0.0,
                    "world" => l.world = f != 0.0,
                    _ => {}
                }
            }
            self.set_texture_look(l);
            return;
        }
        // UV Editor: `uv_open`, `uv <command>` (see `uv_run`), `uv_select <face index>…`
        // (faces of the UV Editor's set, in its order) selects their islands.
        if name == "uv_open" {
            self.actions.push(crate::app::Action::UvEditor);
            return;
        }
        if let Some(op) = name.strip_prefix("uv ") {
            self.uv_run(op.trim());
            return;
        }
        if let Some(list) = name.strip_prefix("uv_select ") {
            let faces = self.uv_faces();
            let islands = self.uv_islands();
            self.uv.selected.clear();
            for i in list.split_whitespace().filter_map(|t| t.parse::<usize>().ok()) {
                if let Some(isl) = faces.get(i).and_then(|f| islands.iter().find(|isl| isl.contains(f))) {
                    self.uv.selected.extend(isl.iter().copied());
                }
            }
            return;
        }
        // `tray_only <Card>,<Card>…`: hide the other side bar cards.
        if let Some(list) = name.strip_prefix("tray_only ") {
            let keep: Vec<&str> = list.split(',').map(str::trim).collect();
            self.prefs.tray_hidden = crate::ui::tray::TRAY_CARDS.iter().filter(|c| !keep.iter().any(|k| k.eq_ignore_ascii_case(c))).map(|c| c.to_string()).collect();
            return;
        }
        // Online Materials: `online [polyhaven|ambientcg|<source plugin name>|plugin-<file stem>]`
        // opens the window (on that source), `online_param <key> <value>` sets a source plugin's
        // parameter and loads its catalogue again, `online_search <words>`,
        // `online_get <source>:<id> [use]` downloads into the library, `online_wait` waits for
        // catalogues and downloads.
        if name == "online" || name.starts_with("online ") {
            self.online.open = true;
            if let Some(key) = name.strip_prefix("online ") {
                match crate::online_mats::Source::from_key(key) {
                    Some(s) => self.online_set_source(s, false, ctx),
                    None => {
                        if !self.online_plugin_source(key, false, ctx) {
                            log::warn!("online: no source {key}");
                        }
                    }
                }
            }
            self.online.ensure_catalogue(ctx);
            return;
        }
        if let Some(args) = name.strip_prefix("online_param ") {
            let (key, value) = args.trim().split_once(' ').unwrap_or((args.trim(), ""));
            if let crate::online_mats::Source::Plugin(src) = self.online.source.clone() {
                let p = &src.plugin;
                if let Some(i) = p.params.iter().position(|q| q.key == key) {
                    let values = self.online.plugin_values.entry(p.path.clone()).or_insert_with(|| p.params.iter().map(|q| q.default.clone()).collect());
                    values[i] = value.to_string();
                }
                self.online_plugin_source(&p.name, true, ctx);
            }
            return;
        }
        if let Some(q) = name.strip_prefix("online_search ") {
            self.online.query = q.trim().to_string();
            return;
        }
        if let Some(args) = name.strip_prefix("online_get ") {
            let mut it = args.split_whitespace();
            let key = it.next().unwrap_or_default().to_string();
            self.online.selected = Some(key.clone());
            self.online_download(&key, it.next() == Some("use"), ctx);
            return;
        }
        if name == "online_wait" {
            if let Some(s) = self.debug.script.as_mut() {
                s.pt_wait = true;
            }
            return;
        }
        // 3D Library: `models [polyhaven|smithsonian|nasa|<source plugin name>|plugin-<file stem>]`
        // opens the window (on that source), `models_param <key> <value>` sets a source plugin's
        // parameter, `models_search <words>` searches (a searching source asks again),
        // `models_get <source>:<id>|@<n> [place]` downloads into the components library,
        // `models_url <url>` imports a model file, `import_model <path>` imports a file with the
        // model importers, `export_attributions <path>`; `online_wait` waits for them.
        if name == "models" || name.starts_with("models ") {
            self.models.open = true;
            if let Some(key) = name.strip_prefix("models ") {
                match crate::online_models::ModelSource::from_key(key) {
                    Some(s) => self.models_set_source(s, false, ctx),
                    None => {
                        if !self.models_plugin_source(key, false, ctx) {
                            log::warn!("models: no source {key}");
                        }
                    }
                }
            }
            self.models.ensure(ctx);
            return;
        }
        if let Some(args) = name.strip_prefix("models_param ") {
            let (key, value) = args.trim().split_once(' ').unwrap_or((args.trim(), ""));
            if let crate::online_models::ModelSource::Plugin(src) = self.models.source.clone() {
                let p = &src.plugin;
                if let Some(i) = p.params.iter().position(|q| q.key == key) {
                    let values = self.models.plugin_values.entry(p.path.clone()).or_insert_with(|| p.params.iter().map(|q| q.default.clone()).collect());
                    values[i] = value.to_string();
                }
                self.models_plugin_source(&p.name, true, ctx);
            }
            return;
        }
        if let Some(q) = name.strip_prefix("models_search ") {
            self.models.query = q.trim().to_string();
            self.models_search(ctx);
            return;
        }
        if let Some(args) = name.strip_prefix("models_get ") {
            let mut it = args.split_whitespace();
            let mut key = it.next().unwrap_or_default().to_string();
            // `@N`: the N-th model shown.
            if let Some(n) = key.strip_prefix('@').and_then(|n| n.parse::<usize>().ok()) {
                key = self.models.shown().get(n).map(|a| a.key()).unwrap_or_default();
            }
            self.models.selected = Some(key.clone());
            self.models_download(&key, it.next() == Some("place"), ctx);
            return;
        }
        // `models_fav <key|@n>` toggles a favourite, `models_cancel <key|@n>` stops a download,
        // `models_filter [license=any|commercial|cc0|known] [format=<name>|-] [tris=<n>]`.
        if let Some(k) = name.strip_prefix("models_fav ").or(name.strip_prefix("models_cancel ")) {
            let mut key = k.trim().to_string();
            if let Some(n) = key.strip_prefix('@').and_then(|n| n.parse::<usize>().ok()) {
                key = self.models.shown().get(n).map(|a| a.key()).unwrap_or_default();
            }
            if name.starts_with("models_cancel ") {
                self.models_cancel(&key);
            } else if let Some(a) = self.models.asset(&key).cloned() {
                self.models_toggle_favourite(&a);
            } else {
                log::warn!("models_fav: no asset {key}");
            }
            return;
        }
        if let Some(args) = name.strip_prefix("models_filter") {
            let mut f = crate::ui::model_library::Filter::default();
            for a in args.split_whitespace() {
                match a.split_once('=') {
                    Some(("license", v)) => f.license = crate::ui::model_library::LicFilter::ALL.into_iter().find(|l| l.key() == v).unwrap_or_default(),
                    Some(("format", v)) if v != "-" => f.format = v.to_string(),
                    Some(("tris", v)) => f.max_tris = v.parse().unwrap_or(0),
                    _ => {}
                }
            }
            self.models.filter = f;
            return;
        }
        // Sites without an API: `dl_watch <dir>` watches that folder (instead of Downloads),
        // `dl_page <query>` makes a waiting card for a page (`url=…&title=…&license=…`),
        // `dl_drop <path>` drops a file (or .webloc / .url) on the window, `dl_title <text>` /
        // `dl_license <text>` edit the offer's credit, `dl_pick <n> on|off` a model of an
        // archive, `dl_import` / `dl_ignore` answer it (`online_wait` waits for the import).
        // `licences [varstotas|components|formats|libraries]`: Help → Licences… on that tab;
        // `licences_pick <crate>` shows a component's licence.
        if let Some(t) = name.strip_prefix("licences") {
            if let Some(c) = t.strip_prefix("_pick ") {
                self.licences.sel = crate::ui::licences::crates().iter().position(|x| x.name == c.trim());
                return;
            }
            let tab = crate::ui::licences::Tab::ALL.into_iter().find(|x| x.key() == t.trim()).unwrap_or_default();
            self.open_licences(tab);
            return;
        }
        if let Some(d) = name.strip_prefix("dl_watch ") {
            self.models.dl.dir = Some(std::path::PathBuf::from(d.trim()));
            self.prefs.watch_downloads = true;
            return;
        }
        if let Some(q) = name.strip_prefix("dl_page ") {
            let q = crate::downloads::parse_query(q.trim());
            self.downloads_page(&q);
            return;
        }
        if let Some(p) = name.strip_prefix("dl_drop ") {
            self.downloads_drop(std::path::Path::new(p.trim()), ctx);
            return;
        }
        if let Some(o) = self.models.dl.offers.first_mut() {
            if let Some(t) = name.strip_prefix("dl_title ") {
                o.credit.title = t.trim().to_string();
                return;
            }
            if let Some(t) = name.strip_prefix("dl_license ") {
                o.credit.license = t.trim().to_string();
                o.kind = o.credit.license_kind();
                return;
            }
            if let Some(args) = name.strip_prefix("dl_pick ") {
                let mut it = args.split_whitespace();
                if let (Some(Ok(i)), Some(v)) = (it.next().map(str::parse::<usize>), it.next())
                    && let Some(m) = o.models.get_mut(i)
                {
                    m.1 = v == "on";
                }
                return;
            }
        }
        if name == "dl_import" {
            self.downloads_import();
            return;
        }
        if name == "dl_ignore" {
            if !self.models.dl.offers.is_empty() {
                self.models.dl.offers.remove(0);
            }
            return;
        }
        if let Some(url) = name.strip_prefix("models_url ") {
            self.models_from_url(url, false, ctx);
            return;
        }
        // `import_model <path>` (the 3D Library's importers) / `import_model_plugin <plugin> <path>`.
        if let Some(rest) = name.strip_prefix("import_model_plugin ") {
            let (who, path) = rest.trim().split_once(' ').unwrap_or((rest.trim(), ""));
            let p = self.plugins.list.iter().find(|p| p.path.file_stem().is_some_and(|s| s.to_string_lossy().eq_ignore_ascii_case(who))).cloned();
            match p {
                Some(p) => self.import_model_file(std::path::Path::new(path.trim()), Some(&p)),
                None => log::warn!("import_model_plugin: no plugin {who}"),
            }
            return;
        }
        if let Some(path) = name.strip_prefix("import_model ") {
            self.import_model_file(std::path::Path::new(path.trim()), None);
            return;
        }
        if let Some(path) = name.strip_prefix("export_attributions ") {
            self.export_to(std::path::Path::new(path.trim()));
            return;
        }
        if name == "mat_refresh" {
            self.refresh_materials();
            return;
        }
        if name == "components" {
            self.actions.push(crate::app::Action::ShowComponents);
            return;
        }
        if name == "reload_plugins" {
            self.actions.push(crate::app::Action::ReloadPlugins);
            return;
        }
        // `bridge_confirm allow|deny`: answer the confirmation an external script asked for.
        if let Some(a) = name.strip_prefix("bridge_confirm ") {
            self.bridge.answer = Some(a.trim() == "allow");
            return;
        }
        if let Some(code) = name.strip_prefix("script ") {
            self.run_script_source(&code.replace("\\n", "\n"), "Script");
            return;
        }
        // `params [x=<len|->] [y=…] [z=…] [n=<copies>] [axis=x|y|z] [gap=<len>] [stretch|scale]`:
        // attributes of the selected parametric group ("-" = as modelled).
        if let Some(args) = name.strip_prefix("params ") {
            let unit = self.doc.model.unit;
            let cur = match self.doc.selection.as_slice() {
                [varstotas_kernel::EntityRef::Instance(i)] => self.doc.model.instance_params(&self.doc.active_path, *i).cloned(),
                _ => None,
            };
            let Some(mut p) = cur else {
                log::warn!("script: params needs one selected parametric group");
                return;
            };
            for a in args.split_whitespace() {
                let len = |v: &str| varstotas_kernel::units::parse_length(v, unit);
                match a.split_once('=') {
                    Some((k @ ("x" | "y" | "z"), v)) => p.size[(k.as_bytes()[0] - b'x') as usize] = len(v),
                    Some(("n", v)) => p.count = v.parse().unwrap_or(p.count),
                    Some(("axis", v)) => p.axis = ["x", "y", "z"].iter().position(|x| *x == v).unwrap_or(0) as u8,
                    Some(("gap", v)) => p.gap = len(v).unwrap_or(p.gap),
                    None if a == "stretch" || a == "scale" => p.stretch = a == "stretch",
                    _ => log::warn!("script: unknown params argument {a}"),
                }
            }
            self.actions.push(Action::SetParams(p));
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
            "make_parametric" => Action::MakeParametric,
            "console" => Action::ToggleConsole,
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
            // QA opens demo fixtures directly from the repository.
            "demo_house" | "demo_gallery" => {
                let file = if name == "demo_house" { "country_house.vrst" } else { "material_gallery.vrst" }; // i18n-ignore
                self.doc.dirty = false;
                match qa_examples_dir().map(|d| d.join("demos")) {
                    Some(d) => Action::OpenPath(d.join(file)),
                    None => return,
                }
            }
            "scene_right" => Action::MoveScene(self.tray.selected_scene.unwrap_or(0), true),
            "reverse_faces" => Action::ReverseFaces,
            "orient_faces" => Action::OrientFaces,
            "flip_red" => Action::FlipAlong(0),
            "flip_green" => Action::FlipAlong(1),
            "flip_blue" => Action::FlipAlong(2),
            "export_dialog" => Action::ExportImage,
            "unfold" => Action::Unfold,
            "sheets" => Action::Sheets,
            "work_planes" => Action::WorkPlanes,
            "model_info" => Action::ModelInfo,
            "preview_show" => Action::PreviewShow,
            "purge_unused" => Action::PurgeUnused,
            "new" => Action::New,
            "preferences" => Action::Preferences,
            "tutorial" => Action::Tutorial,
            "about" => Action::About,
            "start_screen" => Action::StartScreen,
            "report_problem" => Action::ReportProblem,
            "lock" => Action::LockSelected,
            "unlock_all" => Action::UnlockAll,
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
                        m.tags.insert(varstotas_kernel::Tag { locked: false, name: "QA".into(), visible: true, color: [0.5; 4] });
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
        let origin = self.viewport.view_rect.min.to_vec2();
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
            RawCmd::Scroll(p, dy) => {
                // A trackpad swipe: egui applies it at once (a wheel notch is smoothed over time).
                let wheel = |phase: egui::TouchPhase, dy: f32| Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, dy),
                    phase,
                    modifiers: m,
                };
                let ph = egui::TouchPhase::Start;
                vec![vec![mv(p)], vec![wheel(ph, 0.0)], vec![wheel(egui::TouchPhase::Move, dy)], vec![wheel(egui::TouchPhase::End, 0.0)]]
            }
            RawCmd::Wheel(p, delta) => vec![vec![mv(p)], vec![Event::MouseWheel {
                unit: egui::MouseWheelUnit::Line, delta: egui::vec2(0.0, delta),
                phase: egui::TouchPhase::Move, modifiers: m,
            }]],
            RawCmd::Pinch(p, factor) => vec![vec![mv(p)], vec![Event::Zoom(factor)]],
            RawCmd::Twist(p, radians) => vec![vec![mv(p)], vec![Event::Rotate(radians)]],
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
                    .chain(me.splines.iter().map(|(cid, s)| {
                        let ok = if me.spline(*cid).is_some() { "" } else { "-stale" };
                        let smooth = s.anchors.iter().filter(|a| !a.is_corner()).count();
                        format!("Spline{}p{smooth}h{}s{}{ok}", s.anchors.len(), s.segs, if s.closed { "c" } else { "" })
                    }))
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
        // Selected parametric group: world size, copies, resize mode.
        let param = match sel.as_slice() {
            [EntityRef::Instance(i)] => {
                let inst = self.doc.active_mesh().and_then(|me| me.instances.get(*i));
                match (inst, self.doc.model.instance_params(&self.doc.active_path, *i)) {
                    (Some(inst), Some(p)) => {
                        let b = m.def_bounds(inst.def).map(|b| b.transformed(&inst.transform)).map_or(Default::default(), |b| b.max - b.min);
                        format!("{:.0}x{:.0}x{:.0}/n{}/{}", b.x, b.y, b.z, p.count, if p.stretch { "stretch" } else { "scale" })
                    }
                    _ => "none".into(),
                }
            }
            _ => "none".into(),
        };
        let extra = format!("textures={textures:?} uv_placed={} uvs={uvs:?} vector={:?} param={param} script={:?} bridge_calls={} bridge_confirm={} plugins={} plugin_errors={} plugin_dialog={:?} library={} unfold={:?} unfold_open={} sheets={:?} sheet_sel={:?} sheet_vp={:?} sheet_texts={:?} snapping={:?} inc={:?} vcbv={:?} snap={:?} grid={:?} code={:?} code_h={} gen={} gens={:?} gen_defs={} ged={} lib_folders={:?} revealed={:?} lib_delete={:?} tray_hidden={:?} env={} hdri={:?} pbr={:?} fx={} shaders={:?} shader_err={:?} shader_edit={:?} render={} anim_win={} {:?} lightsel={} shader_members={:?} shader_line={:?} outliner={} matlib={:?} online={} models={} credits={:?} pbr_more={:?} mat_delete={:?} current_mat={:?} previews={:?} stats={} def_delete={:?} rig={} downloads={} preview_win={} thumbs={} rename={:?} lib_names={:?} log=({} {} {}) help=({} {:?} {:?}) guide=({} {:?} {:?}) hud=({:?} {} {:?} {}) hud_show={:?} panes=({} {} {:?}) hud_open={:?} lib_spots={:?} lib_tiles={:?} mat_shown={:?} view_min={:?} favs={:?} recent={:?} tray_drag={:?} prefs_win=({} {:?}) new_view={:?} watch_dir={:?} nav={:?} cam_dist={:.0} look=({} {:.2} {}) keys={:?} key_capture={:?} new_render=({}x{} {}spp b{} dn={} match={}) undo_steps=({} {}) status={:?} toolbar={:?} folder_delete={:?} licences=({} {} {:?}) mated_win={} slots={:?} tex_view={:?} ext_edits={:?} uv={} tex_look={:?}", uvs.len(), self.last_vector, self.console.last, self.bridge.calls, self.bridge.confirm.len(), self.plugins.list.len(), self.plugins.errors.len(), self.plugins.dialog.as_ref().and_then(|d| self.plugin_at(d.idx)).map(|p| p.name.clone()), self.library.items.len(), self.unfold.last, self.unfold.open, self.sheets_dump(), self.sheets.selected, self.sheet_vp_dump(), self.sheet_texts_dump(), (self.prefs.snapping.enabled, &self.prefs.snapping.off), self.inc_dump(), self.tool.vcb_value(), crate::inference::last_snap().map(|(k, p)| (k, p.x.round() as i64, p.y.round() as i64, p.z.round() as i64)), (self.prefs.grid.show, self.prefs.grid.step, self.prefs.grid.major), self.prefs.console_code, self.console.code_h.round(), self.gen_dump(), m.generators.keys().collect::<Vec<_>>(), m.defs.values().filter(|d| d.generated.is_some()).count(), self.ged_dump(), self.library.shown_folders, crate::ui::extensions::LAST_REVEALED.lock().ok().and_then(|l| l.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string())), self.library.delete_ask.as_ref().map(|d| d.1.clone()), self.prefs.tray_hidden, self.env_dump(), m.hdri.as_ref().map(|h| h.file_name.clone()), self.pbr_dump(), self.fx_dump(), self.shaders_dump(), self.viewport.shader_errors.iter().map(|(_, e)| e.clone()).collect::<Vec<_>>(), self.tray.shader_edit.as_ref().map(|e| (e.material.clone(), e.error.clone())), self.render_dump(), self.anim_window.open, self.anim_window.last, self.light_sel_dump(), self.tray.shader_edit.as_ref().map(|e| e.editor.member_names()).unwrap_or_default(), self.tray.shader_edit.as_ref().map(|e| e.editor.cursor_line(&e.code)).unwrap_or_default(), self.outliner_dump(), self.matlib_dump(), self.online_dump(), self.models_dump(), m.attributions().iter().map(|a| a.title.clone()).collect::<Vec<_>>(), self.pbr_more_dump(), self.mat_library.delete_ask.as_ref().and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().to_string()), self.doc.current_material.as_ref().map(|c| (c.name.clone(), c.pbr.as_ref().map(|p| (p.metallic, p.transmission)))), self.previews_dump(), self.stats_dump(), self.library.def_delete.and_then(|d| m.defs.get(d)).map(|d| d.name.clone()), self.rig_dump(), self.downloads_dump(), self.model_info.preview_open, self.thumbs.rendered, self.library.rename.as_ref().map(|r| r.1.clone()), self.library.items.iter().map(|i| i.name.clone()).collect::<Vec<_>>(), self.log_window.open, crate::applog::snapshot().1.len(), crate::applog::unseen(), self.script_help.open, self.script_help.topic, self.script_help.search, self.app_help.open, self.app_help.topic, self.app_help.search, self.camera.standard_view(), self.camera.ortho, self.prefs.face_style, self.prefs.xray, self.hud_show_items().iter().filter(|i| i.2).map(|i| i.1.as_str()).collect::<Vec<_>>(), self.pane_count, self.active_pane, self.panes_dump(), self.hud_open, crate::ui::tray_lib::spots(self.viewport.view_rect.min), self.prefs.lib_tiles, self.mat_library.shown, self.viewport.view_rect.min.round(), (key_names(&self.prefs.lib_favourites), key_names(&self.prefs.mat_favourites)), (key_names(&self.prefs.lib_recent), key_names(&self.prefs.mat_recent)), self.tray_drag.as_ref().map(|d| (d.name.clone(), d.entered)), self.prefs_win.open, self.prefs_win.section, self.prefs.new_view, self.prefs.downloads_watch_dir, self.prefs.nav, self.camera.eye.distance(self.camera.target), crate::ui::theme::dark(), self.prefs.ui_scale, self.prefs.toolbar_icons, self.prefs.shortcuts, self.prefs_win.capture.map(|c| c.id()), self.prefs.new_render.width, self.prefs.new_render.height, self.prefs.new_render.samples, self.prefs.new_render.bounces, self.prefs.new_render.denoise, self.prefs.new_render.match_view, self.prefs.undo_steps, self.doc.history.undo_steps(), varstotas_i18n::platform_keys(&self.tool.status_text()), crate::ui::toolbar::layout(&self.prefs.toolbar).iter().map(|i| i.key()).collect::<Vec<_>>().join(" "), self.library.folder_delete.as_ref().map(|d| (d.dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(), d.files, d.dirs)), self.licences.open, self.licences.tab.key(), self.licences.sel.and_then(|i| crate::ui::licences::crates().get(i)).map(|c| c.name.clone()), self.mat_editor.window, self.slots_dump(), self.mat_editor.maps.view.as_ref().map(|v| v.1), self.mat_editor.maps.edits.iter().map(|e| e.path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()).collect::<Vec<_>>(), self.uv_dump(), self.tex_look_dump());
        let template_files = crate::templates::directory(self.prefs.templates_dir.as_deref())
            .and_then(|d| crate::templates::scan(&d).ok()).unwrap_or_default()
            .into_iter().map(|f| f.name).collect::<Vec<_>>();
        let extra = format!("{extra} template_files={template_files:?} document_path={:?} document_dirty={} save_confirm={}", self.doc.path, self.doc.dirty, self.confirm.is_some());
        let lock_counts = (
            m.defs.values().map(|d| d.mesh.instances.values().filter(|i| i.locked).count()).sum::<usize>(),
            m.defs.values().map(|d| d.mesh.faces.values().filter(|f| f.locked).count()).sum::<usize>(),
            m.defs.values().map(|d| d.mesh.edges.values().filter(|e| e.locked).count()).sum::<usize>(),
            m.tags.values().filter(|t| t.locked).count(), m.untagged_locked,
        );
        let extra = format!("{extra} locks={lock_counts:?} metadata={:?} license={:?} model_info_tab={:?}", m.metadata.text, m.metadata.license, self.model_info.tab);
        let extra = format!("{extra} start=({} {}) report=({} {})", self.show_start,
            self.prefs.show_start_screen, self.feedback.open, self.feedback.include_errors);
        let extra = format!("{extra} about=({} {})", self.show_about, self.about_icon.is_some());
        let extra = format!("{extra} tutorial=({} {})", self.tutorial.open, self.tutorial.step + 1);
        let transfers = |work: &crate::online_mats::Online| work.progress.values()
            .filter_map(|p| p.snapshot()).map(|p| (p.received, p.total)).collect::<Vec<_>>();
        let extra = format!("{extra} model_transfer={:?} material_transfer={:?}", transfers(&self.models.work), transfers(&self.online.work));
        let extra = format!("{extra} triangles={}", self.prefs.show_triangles);
        let sc = &m.scene_settings;
        let extra = format!(
            "{extra} scenes={:?} scene={:?} playing={} anim={} cam=({:.0},{:.0},{:.0}) style={} edges={} tags_hidden={} tags={:?} untagged_hidden={} drawn_faces={} transition={:.1}s/{}",
            m.scenes.iter().map(|s| format!("{}{}", s.name, if s.in_animation { "" } else { "(-)" })).collect::<Vec<_>>(),
            self.tray.selected_scene,
            self.scene_player.is_some(),
            self.cam_anim.is_some(),
            self.camera.eye.x,
            self.camera.eye.y,
            self.camera.eye.z,
            self.prefs.face_style.key(),
            self.prefs.edges_on(),
            m.tags.values().filter(|t| !t.visible).count(),
            m.tags.values().map(|t| t.name.as_str()).collect::<Vec<_>>(),
            m.untagged_hidden,
            varstotas_kernel::tessellate(m).faces.len(),
            sc.transition_secs,
            sc.transitions,
        );
        let sd = m.section_display;
        let cut = self.section_cut.borrow().as_ref().filter(|_| m.active_section().is_some()).map_or((0, 0), |c| (c.1.len(), c.2.len()));
        let planes: Vec<String> = m
            .work_planes
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let (n, (t, a, sp)) = (p.normal(), p.angles());
                format!(
                    "{}{}{} o({:.0},{:.0},{:.0}) n({:.2},{:.2},{:.2}) a({:.0},{:.0},{:.0})",
                    p.name,
                    if m.active_plane == Some(i) { "*" } else { "" },
                    if p.hidden { " hidden" } else { "" },
                    p.origin.x,
                    p.origin.y,
                    p.origin.z,
                    n.x,
                    n.y,
                    n.z,
                    t,
                    a,
                    sp
                )
            })
            .collect();
        let gizmo = crate::tools::workplane::gizmo_screen(&self.doc, &self.camera, self.viewport.size());
        let extra = format!("{extra} planes={planes:?} gizmo={gizmo:?}");
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
        let extra = format!("{extra} {} {}",self.printing_dump(),self.print_tools_dump());
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
        ) + " " + extra.as_str()
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

impl VarstotasApp {
    /// Increments for the dump: (move, rotate, scale) steps, `None` when off.
    fn inc_dump(&self) -> (Option<f64>, Option<f64>, Option<f64>) {
        let i = self.prefs.snapping.increments;
        (i.move_on.then_some(i.move_step), i.rotate_on.then_some(i.angle_step), i.scale_on.then_some(i.scale_step))
    }

    /// Harness `workplane …` sub-commands (each one undo step).
    fn workplane_cmd(&mut self, args: &str) {
        let mut it = args.split_whitespace();
        let cmd = it.next().unwrap_or("");
        let rest: Vec<&str> = it.collect();
        let nums: Vec<f64> = rest.iter().filter_map(|t| t.parse().ok()).collect();
        let idx = rest.first().and_then(|t| t.parse::<usize>().ok()).and_then(|n| n.checked_sub(1));
        match (cmd, &nums[..]) {
            ("add", [ox, oy, oz, nx, ny, nz, ..]) => {
                let o = DVec3::new(*ox, *oy, *oz);
                match varstotas_kernel::WorkPlane::from_normal("", o, DVec3::new(*nx, *ny, *nz), None) {
                    Some(p) => {
                        self.doc.commit("Add Guide Plane", |m| m.add_work_plane(p));
                    }
                    None => eprintln!("[debug] workplane add: degenerate normal"),
                }
            }
            ("activate", _) => {
                let a = idx.filter(|&i| i < self.doc.model.work_planes.len());
                self.doc.commit("Activate Work Plane", |m| m.active_plane = a);
            }
            ("angles", [t, a, sp, ..]) => {
                if let Some(i) = self.doc.model.active_plane {
                    self.doc.commit("Edit Work Planes", |m| m.work_planes[i].set_angles(*t, *a, *sp));
                }
            }
            ("origin", [x, y, z, ..]) => {
                if let Some(i) = self.doc.model.active_plane {
                    self.doc.commit("Edit Work Planes", |m| m.work_planes[i].origin = DVec3::new(*x, *y, *z));
                }
            }
            ("hide" | "show", _) => {
                if let Some(i) = idx.filter(|&i| i < self.doc.model.work_planes.len()) {
                    let h = cmd == "hide";
                    self.doc.commit("Edit Work Planes", |m| m.work_planes[i].hidden = h);
                }
            }
            ("remove", _) => {
                if let Some(i) = idx {
                    self.doc.commit("Delete Work Plane", |m| m.remove_work_plane(i));
                }
            }
            _ => eprintln!("[debug] workplane: bad command {args:?}"),
        }
        crate::axes::sync(&self.doc.model);
    }
}

impl VarstotasApp {
    /// Selected generated components for the dump: `Name{k=v,…}@WxDxH` per instance, or none.
    /// Generator Editor: `-` closed, else `Name{p=…,f=…}:preview(edges,WxDxH)|err(…):status:h<code height>`.
    /// `(sky, rotation, intensity, sun, exposure, tonemap, background)`.
    fn env_dump(&self) -> String {
        let e = &self.doc.model.environment;
        format!(
            "({:?}, {}, {}, {}, {}, {}, {})",
            e.sky,
            e.rotation_deg,
            e.intensity,
            e.sun,
            e.exposure,
            e.tonemap.key(),
            e.background
        )
    }

    /// The effects that are on, `-` for none: `soft,ao,ssr,bloom,dof,fog,vig,clay,<stylize>,hq`.
    /// The selected light: index, position, aim, kind, and its marker / aim handle on screen
    /// (viewport points, for `raw` clicks).
    fn light_sel_dump(&self) -> String {
        let Some((i, l)) = self.doc.light_sel.and_then(|i| self.doc.model.lights.get(i).map(|l| (i, l))) else { return "-".into() };
        let p = l.position;
        let d = l.direction;
        let (m, a) = crate::tools::light::light_screen(l, &self.camera, self.viewport.size());
        format!(
            "({i} {} ({:.0},{:.0},{:.0}) aim=({:.2},{:.2},{:.2}) {}W at={m:?} handle={a:?})",
            l.kind.key(),
            p.x,
            p.y,
            p.z,
            d.x,
            d.y,
            d.z,
            l.power
        )
    }

    fn render_dump(&self) -> String {
        let r = &self.doc.model.render;
        let job = self.viewport.pt.as_ref().map(|j| {
            let size = self.viewport.pt_texture().map(|t| t.1);
            format!("{}spp{:?}{}{}{}", j.samples, size, if j.running { ",running" } else { "" }, if j.denoised { ",denoised" } else { "" }, match &j.saved {
                Some(Err(e)) => format!(",error:{e}"),
                _ => String::new(),
            })
        });
        let lights: Vec<String> = self.doc.model.lights.iter().map(|l| format!("{}:{}W", l.kind.key(), l.power)).collect();
        let grade = if r.neutral_grade() {
            "neutral".to_string()
        } else {
            format!("{}ev,{:?},{}K,{},{},{},{}", r.exposure, r.tonemap.map(|t| t.key()), r.temperature, r.tint, r.saturation, r.contrast, r.vignette)
        };
        format!(
            "({}x{} {}spp b{}{}{} job={:?} lights={:?} window={}) q={:?} clamp={} ai={} dof={}/f{} focus={} transparent={} grade={grade}",
            r.width,
            r.height,
            r.samples,
            r.bounces,
            if r.denoise { " dn" } else { "" },
            if r.match_view { " match" } else { "" },
            job,
            lights,
            self.render_window.open,
            r.preset(),
            r.clamp,
            r.ai_denoise,
            r.dof.label(),
            r.f_stop,
            r.focus,
            r.transparent
        )
    }

    fn fx_dump(&self) -> String {
        let x = &self.doc.model.environment.effects;
        let mut v: Vec<&str> = Vec::new();
        for (on, n) in [
            (x.soft_shadows, "soft"),
            (x.ao, "ao"),
            (x.reflections, "ssr"),
            (x.bloom, "bloom"),
            (x.dof, "dof"),
            (x.fog, "fog"),
            (x.vignette > 0.0, "vig"),
            (x.clay, "clay"),
            (x.stylize != varstotas_kernel::Stylize::None, x.stylize.key()),
            (x.high_quality, "hq"),
        ] {
            if on {
                v.push(n);
            }
        }
        if v.is_empty() { "-".into() } else { v.join(",") }
    }

    /// Materials with a custom shader: (name, preset or `custom`, params).
    fn shaders_dump(&self) -> Vec<(String, &'static str, Vec<f32>)> {
        self.doc
            .model
            .materials
            .values()
            .filter_map(|m| {
                let sh = m.pbr.as_ref()?.shader.as_ref()?;
                Some((m.name.clone(), crate::shaders::preset_name(&sh.source).unwrap_or("custom"), sh.params.clone()))
            })
            .collect()
    }

    /// Materials with physical properties: (name, metallic, roughness, transmission, maps).
    /// The instance tree: `name[child child]` ("*" = selected, "~" = hidden), the open context's depth.
    fn outliner_dump(&self) -> String {
        fn level(m: &varstotas_kernel::Model, mesh: &varstotas_kernel::Mesh, sel: &[varstotas_kernel::EntityRef], path: &mut Vec<varstotas_kernel::InstanceId>, active: &[varstotas_kernel::InstanceId], out: &mut String) {
            let mut first = true;
            for (id, inst) in mesh.instances.iter() {
                if !first {
                    out.push(' ');
                }
                first = false;
                let def = m.defs.get(inst.def);
                let name = if inst.name.is_empty() { def.map(|d| d.name.clone()).unwrap_or_default() } else { inst.name.clone() };
                if path.as_slice() == active && sel.contains(&varstotas_kernel::EntityRef::Instance(id)) {
                    out.push('*');
                }
                if inst.hidden {
                    out.push('~');
                }
                out.push_str(&name.replace(' ', "_"));
                if let Some(d) = def.filter(|d| !d.mesh.instances.is_empty()) {
                    out.push('[');
                    path.push(id);
                    level(m, &d.mesh, sel, path, active, out);
                    path.pop();
                    out.push(']');
                }
            }
        }
        let mut out = String::new();
        let m = &self.doc.model;
        if let Some(mesh) = m.mesh_at(&[]) {
            level(m, mesh, &self.doc.selection, &mut Vec::new(), &self.doc.active_path, &mut out);
        }
        format!("{{{out}}} ctx={}", self.doc.active_path.len())
    }

    /// `(open source catalogue=N shown=N busy=N sel=key last=…)`.
    fn online_dump(&self) -> String {
        let o = &self.online;
        let n = match o.catalogues.get(&o.source) {
            Some(Ok(l)) => l.len().to_string(),
            Some(Err(e)) => format!("error {e:?}"),
            None => "-".into(),
        };
        format!("({} {} catalogue={n} shown={} busy={} sel={:?} last={:?})", o.open, o.source.key(), o.shown().len(), o.work.busy, o.selected, o.last)
    }

    /// `(open source models=N shown=N busy=N sel=key last=…)`.
    /// Model Info: `(scene faces, triangles, groups, components placed | component defs used /
    /// all, group defs used / all, materials used / all) last purge`.
    fn stats_dump(&self) -> String {
        let s = self.doc.model.statistics();
        format!(
            "({} {} {} {} | {}/{} {}/{} {}/{}) {:?} window={}",
            s.scene.faces, s.scene.triangles, s.scene.groups, s.scene.components, s.component_defs_used, s.component_defs, s.group_defs_used, s.group_defs, s.materials_used, s.materials, self.model_info.last, self.model_info.open
        )
    }

    /// The document's preview picture (pixel size) and the library items with one.
    /// (document picture size, library items with a picture, document components with one).
    fn previews_dump(&self) -> (Option<[u32; 2]>, Vec<String>, Vec<String>) {
        let lib = self.library.items.iter().filter(|i| i.preview.is_some()).map(|i| i.name.clone()).collect();
        let mut defs: Vec<String> = self.doc.model.defs.values().filter(|d| d.preview.is_some()).map(|d| d.name.clone()).collect();
        defs.sort();
        (self.doc.model.preview.as_ref().map(|p| p.pixels), lib, defs)
    }

    fn models_dump(&self) -> String {
        let o = &self.models;
        let n = match o.list() {
            Some(Ok(l)) => l.len().to_string(),
            Some(Err(e)) => format!("error {e:?}"),
            None => "-".into(),
        };
        format!("({} {} models={n} shown={} busy={} sel={:?} last={:?} favs={} history={} filter={:?})", o.open, o.source.key(), o.shown().len(), o.work.busy, o.selected, o.last, o.state.favourites.len(), o.state.history.len(), (o.filter.license.key(), &o.filter.format, o.filter.max_tris))
    }

    /// `(waiting=[titles] offers=N first=(title source licence on/models dup place) watching send last)`.
    fn downloads_dump(&self) -> String {
        let d = &self.models.dl;
        let first = d.offers.first().map(|o| format!("({:?} {:?} {:?} {}/{} dup={} place={})", o.credit.title, o.credit.source, o.kind.label(), o.models.iter().filter(|m| m.1).count(), o.models.len(), o.duplicate.is_some(), o.place));
        format!(
            "(waiting={:?} offers={} first={} watching={} last={:?})",
            d.expects.iter().map(|e| format!("{} [{}]", e.credit.title, e.credit.license)).collect::<Vec<_>>(),
            d.offers.len(),
            first.unwrap_or_else(|| "-".into()),
            d.watcher.as_ref().map(|w| w.dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()).unwrap_or_else(|| "-".into()),
            d.last
        )
    }

    /// Library materials as "folder/name" (the examples not built).
    fn matlib_dump(&self) -> Vec<String> {
        self.mat_library.items.iter().map(|i| if i.folder.is_empty() { i.mat.name.clone() } else { format!("{}/{}", i.folder, i.mat.name) }).collect()
    }

    fn pbr_dump(&self) -> Vec<(String, f32, f32, f32, Vec<&'static str>)> {
        self.doc
            .model
            .materials
            .values()
            .filter_map(|m| {
                let p = m.pbr.as_ref()?;
                let maps = varstotas_kernel::PBR_MAPS.iter().copied().filter(|n| p.map(n).is_some()).collect();
                Some((m.name.clone(), p.metallic, p.roughness, p.transmission, maps))
            })
            .collect()
    }

    /// Clear coat, sheen, anisotropy, anti-tiling, bevel of each material that has any.
    /// The current material's texture look: "h s l cut" + flags (c colorize, d decal, w world).
    fn tex_look_dump(&self) -> Option<String> {
        let l = self.doc.current_material.as_ref()?.image.as_ref()?.look;
        let flags: String = [(l.colorize, 'c'), (l.decal, 'd'), (l.world, 'w')].iter().filter(|f| f.0).map(|f| f.1).collect();
        Some(format!("{:.0} {:.2} {:.2} {:.2} {flags}", l.hue, l.saturation, l.lightness, l.alpha_cutoff)) // i18n-ignore
    }

    /// Texture slots of the current material: (slot, file, pixels).
    fn slots_dump(&self) -> Vec<(&'static str, String, [u32; 2])> {
        let Some(cur) = &self.doc.current_material else { return Vec::new() };
        crate::textures::SLOTS
            .iter()
            .filter_map(|s| crate::ui::mat_maps::slot_image(cur, s).map(|i| (*s, i.file_name.clone(), i.pixels)))
            .collect()
    }

    fn pbr_more_dump(&self) -> Vec<(String, [f32; 5])> {
        self.doc
            .model
            .materials
            .values()
            .filter_map(|m| {
                let p = m.pbr.as_ref()?;
                let v = [p.clearcoat, p.sheen, p.anisotropy, p.anti_tiling, p.bevel];
                v.iter().any(|x| *x != 0.0).then(|| (m.name.clone(), v))
            })
            .collect()
    }

    fn ged_dump(&self) -> String {
        let ed = &self.gen_editor;
        if !ed.open {
            return "-".into();
        }
        let head = match &ed.parsed {
            Ok(g) => format!("{}{{p={},f={}}}", g.name, g.params.len(), g.formulas.len()),
            Err(e) => format!("bad({})", e.chars().take(60).collect::<String>()),
        };
        let pre = match &ed.preview {
            Some(Ok(p)) => format!("preview({},{:.0}x{:.0}x{:.0})", p.edges.len(), p.size.x, p.size.y, p.size.z),
            Some(Err(e)) => format!("err({})", e.chars().take(80).collect::<String>().replace(' ', "_")),
            None => "none".into(),
        };
        format!("{head}:{pre}:{}:h{}", ed.status.replace(' ', "_"), ed.code_h.round())
    }

    fn gen_dump(&self) -> String {
        let Some((ids, g, values)) = self.selected_generated() else { return "none".into() };
        let mesh = self.doc.active_mesh();
        let parts: Vec<String> = ids
            .iter()
            .zip(&values)
            .map(|(i, v)| {
                let kv: Vec<String> = g
                    .params
                    .iter()
                    .filter_map(|p| v.get(&p.key).map(|x| (p, x)))
                    .map(|(p, x)| match x {
                        varstotas_kernel::GenValue::Num(n) => format!("{}={}", p.key, (n * 1000.0).round() / 1000.0),
                        varstotas_kernel::GenValue::Bool(b) => format!("{}={b}", p.key),
                        varstotas_kernel::GenValue::Text(t) => format!("{}={t}", p.key),
                    })
                    .collect();
                let b = mesh
                    .and_then(|me| me.instances.get(*i))
                    .and_then(|inst| self.doc.model.def_bounds(inst.def).map(|b| b.transformed(&inst.transform)))
                    .map_or(Default::default(), |b| b.max - b.min);
                format!("{}{{{}}}@{:.0}x{:.0}x{:.0}", g.name, kv.join(","), b.x, b.y, b.z)
            })
            .collect();
        parts.join(";")
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
        assert!(Script::parse("action click_spot start:continue").raw.is_some());
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
        assert_eq!(parse_cmd("raw scroll 3 4 -200"), Some(ScriptCmd::Raw(RawCmd::Scroll(Pos2::new(3.0, 4.0), -200.0))));
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
