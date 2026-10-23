//! Worker-local transfer progress shared with the online libraries.
use std::sync::{Arc, Mutex};
use varstotas_i18n::tr;
use varstotas_script::network::{ProgressGuard, TransferProgress};

#[derive(Clone, Default)]
pub struct Tracker(Arc<Mutex<Option<TransferProgress>>>);

impl Tracker {
    pub fn observe(&self, ctx: &egui::Context) -> ProgressGuard {
        let (state, ctx) = (self.0.clone(), ctx.clone());
        varstotas_script::network::observe_progress(Arc::new(move |progress| {
            if let Ok(mut value) = state.lock() {
                *value = Some(progress);
            }
            ctx.request_repaint();
        }))
    }

    pub fn snapshot(&self) -> Option<TransferProgress> {
        self.0.lock().ok()?.clone()
    }
}

fn bytes(value: u64) -> String {
    let (unit, scale) = if value >= 1024 * 1024 * 1024 {
        ("GiB", 1073741824.0) // i18n-ignore: IEC unit
    }
    else if value >= 1024 * 1024 {
        ("MiB", 1048576.0) // i18n-ignore: IEC unit
    }
    else if value >= 1024 {
        ("KiB", 1024.0) // i18n-ignore: IEC unit
    } else {
        ("B", 1.0)
    }; // i18n-ignore: IEC units
    format!("{:.1} {unit}", value as f64 / scale)
}

pub fn show(ui: &mut egui::Ui, progress: &TransferProgress) {
    let text = if let Some(total) = progress.total.filter(|t| *t > 0) {
        let percent = ((progress.received as f64 / total as f64) * 100.0).clamp(0.0, 100.0);
        format!(
            "{percent:.0}% · {} / {}",
            bytes(progress.received),
            bytes(total)
        )
    } else {
        tr!(
            "download-progress-unknown",
            received = bytes(progress.received)
        )
    };
    ui.label(egui::RichText::new(&progress.file).small())
        .on_hover_text(&progress.file);
    if let Some(total) = progress.total.filter(|t| *t > 0) {
        ui.add(
            egui::ProgressBar::new((progress.received as f32 / total as f32).clamp(0.0, 1.0))
                .text(text)
                .desired_width(ui.available_width()),
        );
    } else {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(text);
        });
    }
}
