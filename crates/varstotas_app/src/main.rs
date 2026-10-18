//! Varstotas application entry point. Owner: `shell` agent (tools/ and inference/ — `tools` agent).

// Release builds on Windows: GUI subsystem, no console window.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod animation;
mod app;
mod autosave;
mod axes;
mod bridge;
mod bridge_ops;
mod clipboard;
mod debug_capture;
mod demo;
mod document;
mod inference;
mod layout;
mod nav;
mod mat_plugins;
mod online_models;
mod online_mats;
mod pbr;
mod shader_help;
mod shaders;
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
    if let egui_wgpu::WgpuSetup::CreateNew(setup) = &mut wgpu_options.wgpu_setup {
        // Ray queries (hardware / driver ray tracing) for the path tracer, where there are any.
        let base = setup.device_descriptor.clone();
        setup.device_descriptor = std::sync::Arc::new(move |adapter| {
            use egui_wgpu::wgpu;
            let mut d = base(adapter);
            if adapter.features().contains(wgpu::Features::EXPERIMENTAL_RAY_QUERY) && std::env::var_os("VARSTOTAS_NO_RAY_QUERY").is_none() {
                d.required_features |= wgpu::Features::EXPERIMENTAL_RAY_QUERY;
                let have = adapter.limits();
                d.required_limits.max_blas_primitive_count = have.max_blas_primitive_count;
                d.required_limits.max_blas_geometry_count = have.max_blas_geometry_count;
                d.required_limits.max_tlas_instance_count = have.max_tlas_instance_count;
                d.required_limits.max_acceleration_structures_per_shader_stage = have.max_acceleration_structures_per_shader_stage.min(4);
                d.required_limits.max_buffers_and_acceleration_structures_per_shader_stage =
                    d.required_limits.max_buffers_and_acceleration_structures_per_shader_stage.max(have.max_buffers_and_acceleration_structures_per_shader_stage.min(64));
                // SAFETY: only the path tracer uses the experimental feature (ray queries in
                // its own shader module); the rest of the renderer does not depend on it.
                d.experimental_features = unsafe { wgpu::ExperimentalFeatures::enabled() };
            }
            d
        });
    }
    if std::env::var_os("VARSTOTAS_SCRIPT").is_some() {
        // Scripted runs must not stall when the window is hidden or on another desktop:
        // with vsync macOS stops handing out frames to an occluded window.
        wgpu_options.surface.present_mode = egui_wgpu::wgpu::PresentMode::AutoNoVsync;
    }
    // Scripted runs stay in the background (the harness feeds its own input and fakes the
    // window focus): the window opens inactive, on macOS the app is not activated and has no
    // Dock icon. It still has to stay uncovered: eframe skips frames of an occluded window
    // and macOS gives it no drawables. VARSTOTAS_FOREGROUND=1 activates the window as before.
    let background = std::env::var_os("VARSTOTAS_SCRIPT").is_some() && std::env::var_os("VARSTOTAS_FOREGROUND").is_none();
    #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
    let mut options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title("Varstotas")
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 500.0])
            .with_active(!background),
        wgpu_options,
        ..Default::default()
    };
    #[cfg(target_os = "macos")]
    if background {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        options.event_loop_builder = Some(Box::new(|b| {
            b.with_activation_policy(ActivationPolicy::Accessory).with_activate_ignoring_other_apps(false);
        }));
    }
    eframe::run_native(
        "Varstotas",
        options,
        Box::new(|cc| Ok(Box::new(app::VarstotasApp::new(cc)))),
    )
}
