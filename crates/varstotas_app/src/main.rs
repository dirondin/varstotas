//! Varstotas application entry point. Owner: `shell` agent (tools/ and inference/ — `tools` agent).

fn main() -> eframe::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title("Varstotas")
            .with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Varstotas",
        options,
        Box::new(|_cc| Ok(Box::new(VarstotasApp::default()))),
    )
}

#[derive(Default)]
struct VarstotasApp;

impl eframe::App for VarstotasApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.label("Varstotas — skeleton");
    }
}
