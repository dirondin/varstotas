//! Help → Tutorial: a practical walkthrough that can stay beside the working model.
use super::app_help::Topic;
use crate::{
    app::{Action, VarstotasApp},
    tools::ToolId,
};
use egui::{Context, RichText};
use varstotas_i18n::tr;

#[derive(Default)]
pub struct TutorialWindow {
    pub open: bool,
    pub step: usize,
}

const STEPS: usize = 10;
const TOOLS: [&[ToolId]; STEPS] = [
    &[],
    &[ToolId::Orbit, ToolId::Pan, ToolId::Zoom],
    &[ToolId::Rectangle, ToolId::Circle, ToolId::Line],
    &[ToolId::PushPull, ToolId::Box, ToolId::Cylinder],
    &[ToolId::Line, ToolId::Tape],
    &[ToolId::Select, ToolId::Move, ToolId::Rotate, ToolId::Scale],
    &[ToolId::Select],
    &[ToolId::Paint, ToolId::TexturePosition],
    &[ToolId::Section, ToolId::Dimension],
    &[],
];
const TOPICS: [Topic; STEPS] = [
    Topic::Start,
    Topic::Navigation,
    Topic::Drawing,
    Topic::Shapes,
    Topic::Inference,
    Topic::Groups,
    Topic::Organizing,
    Topic::Materials,
    Topic::Scenes,
    Topic::Files,
];

impl VarstotasApp {
    pub(crate) fn tutorial_window(&mut self, ctx: &Context) {
        if !self.tutorial.open {
            return;
        }
        self.tutorial.step = self.tutorial.step.min(STEPS - 1);
        let step = self.tutorial.step;
        let mut open = true;
        let mut finish = false;
        egui::Window::new(tr!("tutorial-title"))
            .id(egui::Id::new("tutorial_window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_pos([80.0, 100.0])
            .default_size([660.0, 520.0])
            .min_size([480.0, 360.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(tr!(&format!("tutorial-{step}-title")));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(tr!("tutorial-step", current = step + 1, total = STEPS))
                                .weak(),
                        );
                    });
                });
                ui.add(
                    egui::ProgressBar::new((step + 1) as f32 / STEPS as f32).desired_height(4.0),
                );
                ui.add_space(12.0);
                let height = (ui.available_height() - 52.0).max(180.0);
                egui::ScrollArea::vertical()
                    .id_salt(("tutorial_body", step))
                    .max_height(height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            let icon = TOOLS[step].first().copied().unwrap_or(ToolId::Box).icon();
                            let (rect, _) = ui
                                .allocate_exact_size(egui::vec2(96.0, 96.0), egui::Sense::hover());
                            super::icons::paint_icon(ui.painter(), rect.shrink(12.0), icon, false);
                            ui.add_space(14.0);
                            ui.vertical(|ui| {
                                ui.label(tr!(&format!("tutorial-{step}-body")));
                                ui.add_space(12.0);
                                egui::Frame::group(ui.style()).show(ui, |ui| {
                                    ui.label(RichText::new(tr!("tutorial-try")).strong());
                                    ui.add_space(4.0);
                                    ui.label(tr!(&format!("tutorial-{step}-task")));
                                });
                            });
                        });
                        ui.add_space(16.0);
                        if !TOOLS[step].is_empty() {
                            ui.label(RichText::new(tr!("tutorial-tools")).small().weak());
                            ui.horizontal_wrapped(|ui| {
                                for tool in TOOLS[step] {
                                    let r =
                                        ui.button(tool.name()).on_hover_text(tool.description());
                                    super::tray_lib::spot(
                                        format!("tutorial:tool:{tool:?}"),
                                        r.rect.center(),
                                    );
                                    if r.clicked() {
                                        self.actions.push(Action::SetTool(*tool));
                                    }
                                }
                            });
                        }
                        ui.add_space(12.0);
                        let r = ui.link(tr!("tutorial-details"));
                        super::tray_lib::spot("tutorial:details", r.rect.center());
                        if r.clicked() {
                            self.open_app_help(TOPICS[step]);
                        }
                        ui.label(RichText::new(tr!("tutorial-practice-note")).small().weak());
                    });
                ui.separator();
                ui.horizontal(|ui| {
                    let r = ui.add_enabled(step > 0, egui::Button::new(tr!("tutorial-back")));
                    super::tray_lib::spot("tutorial:back", r.rect.center());
                    if r.clicked() {
                        self.tutorial.step -= 1;
                    }
                    if step > 0 {
                        let r = ui.button(tr!("tutorial-restart"));
                        super::tray_lib::spot("tutorial:restart", r.rect.center());
                        if r.clicked() {
                            self.tutorial.step = 0;
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let r = ui.button(if step + 1 == STEPS {
                            tr!("tutorial-finish")
                        } else {
                            tr!("tutorial-next")
                        });
                        super::tray_lib::spot("tutorial:next", r.rect.center());
                        if r.clicked() {
                            if step + 1 == STEPS {
                                finish = true;
                            } else {
                                self.tutorial.step += 1;
                            }
                        }
                    });
                });
            });
        self.tutorial.open = open && !finish;
    }
}
