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
        }
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
    }
}
