//! Debug helpers driven by env vars (used for automated visual checks):
//! - `VARSTOTAS_DEMO=1` inserts demo geometry on startup.
//! - `VARSTOTAS_SELECT_ALL=1` selects everything after ~20 frames.
//! - `VARSTOTAS_SCREENSHOT=<file.bmp>` captures the window after ~40 frames, writes a BMP, quits.

use egui::{Context, Event, ViewportCommand};

#[derive(Default)]
pub struct DebugCapture {
    path: Option<String>,
    frame: u32,
    requested: bool,
    /// Set once the capture is written: the app should close without the save prompt.
    pub force_close: bool,
}

impl DebugCapture {
    pub fn from_env() -> Self {
        Self { path: std::env::var("VARSTOTAS_SCREENSHOT").ok(), ..Default::default() }
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
