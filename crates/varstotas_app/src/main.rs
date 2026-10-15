//! Varstotas application entry point. Owner: `shell` agent (tools/ and inference/ — `tools` agent).

// Release builds on Windows: GUI subsystem, no console window.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod autosave;
mod axes;
mod clipboard;
mod debug_capture;
mod document;
mod inference;
mod nav;
mod overlay;
mod scene_builder;
mod templates;
mod text3d;
mod textures;
mod tools;
mod ui;
mod viewport;

fn main() -> eframe::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title("Varstotas")
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Varstotas",
        options,
        Box::new(|cc| Ok(Box::new(app::VarstotasApp::new(cc)))),
    )
}
