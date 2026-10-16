//! Varstotas application entry point. Owner: `shell` agent (tools/ and inference/ — `tools` agent).

// Release builds on Windows: GUI subsystem, no console window.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod autosave;
mod axes;
mod bridge;
mod bridge_ops;
mod clipboard;
mod debug_capture;
mod document;
mod inference;
mod layout;
mod nav;
mod overlay;
mod plugins;
mod scene_builder;
mod templates;
mod text3d;
mod textures;
mod vector_export;
mod tools;
mod ui;
mod viewport;

fn main() -> eframe::Result<()> {
    env_logger::init();
    let mut wgpu_options = egui_wgpu::WgpuConfiguration::default();
    if std::env::var_os("VARSTOTAS_SCRIPT").is_some() {
        // Scripted runs must not stall when the window is hidden or on another desktop:
        // with vsync macOS stops handing out frames to an occluded window.
        wgpu_options.surface.present_mode = egui_wgpu::wgpu::PresentMode::AutoNoVsync;
    }
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title("Varstotas")
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 500.0]),
        wgpu_options,
        ..Default::default()
    };
    eframe::run_native(
        "Varstotas",
        options,
        Box::new(|cc| Ok(Box::new(app::VarstotasApp::new(cc)))),
    )
}
