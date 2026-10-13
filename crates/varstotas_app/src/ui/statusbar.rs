//! Status bar: tool hint on the left, Measurements box (VCB) on the right.

use super::icons::{Icon, paint_icon};
use super::theme;
use crate::app::VarstotasApp;
use egui::{Align, Color32, FontId, Layout, RichText, Sense, Stroke, Ui, vec2};

impl VarstotasApp {
    pub(crate) fn status_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.set_min_height(26.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                self.vcb_box(ui);
                ui.add_space(6.0);
                ui.label(RichText::new(self.tool.vcb_label()).color(theme::TEXT_DIM));
                ui.add_space(12.0);
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
                    paint_icon(ui.painter(), rect, Icon::Info, false);
                    let (text, is_error) = self.status_text();
                    let color = if is_error { Color32::from_rgb(0xC0, 0x30, 0x20) } else { Color32::from_gray(50) };
                    ui.add(egui::Label::new(RichText::new(text).color(color)).truncate());
                });
            });
        });
    }

    fn vcb_box(&mut self, ui: &mut Ui) {
        let typing = !self.vcb.is_empty();
        let text = if typing { self.vcb.clone() } else { self.tool.vcb_value() };
        let (rect, resp) = ui.allocate_exact_size(vec2(170.0, 22.0), Sense::click());
        let p = ui.painter();
        let border = if self.vcb_error {
            Color32::from_rgb(0xD0, 0x40, 0x30)
        } else if typing {
            theme::ACCENT
        } else {
            theme::BORDER
        };
        p.rect(rect, 5.0, Color32::WHITE, Stroke::new(if typing { 1.5 } else { 1.0 }, border), egui::StrokeKind::Inside);
        let color = if typing { Color32::BLACK } else { Color32::from_gray(70) };
        let galley = p.layout_no_wrap(text, FontId::monospace(13.0), color);
        let pos = rect.left_center() + vec2(8.0, -galley.size().y * 0.5);
        let caret_x = pos.x + galley.size().x + 1.0;
        p.galley(pos, galley, color);
        if typing {
            let t = ui.input(|i| i.time);
            if (t * 2.0) as i64 % 2 == 0 {
                p.line_segment(
                    [egui::pos2(caret_x, rect.top() + 4.0), egui::pos2(caret_x, rect.bottom() - 4.0)],
                    Stroke::new(1.0, Color32::BLACK),
                );
            }
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
        }
        resp.on_hover_text("Measurements: just type a value and press Enter (e.g. 2500, 2.5m, 300,200, 24s)");
    }
}
