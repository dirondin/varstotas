//! Help → User Guide: the documentation of the whole application inside it. Topics from the
//! first steps to files and scripts; the tools are shown from their own texts (`ToolId::
//! description` / `instructor`, the same as the Instructor card), so the guide follows the
//! tools. A search goes through every topic and tool. F1 over the model opens the topic of
//! the active tool; the scripting topics link to Help → Script Reference.

use super::script_help::rich;
use super::theme;
use crate::app::{Action, VarstotasApp};
use crate::tools::ToolId;
use egui::{Context, RichText, ScrollArea, Ui};
use varstotas_i18n::tr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Topic {
    #[default]
    Start,
    Navigation,
    Selecting,
    Drawing,
    Shapes,
    Modifying,
    Inference,
    Measurements,
    Groups,
    Library,
    Materials,
    Organizing,
    Construction,
    Styles,
    Scenes,
    Files,
    Scripts,
    Shortcuts,
    Problems,
}

pub const TOPICS: &[(Topic, &str)] = &[
    (Topic::Start, "help-topic-start"),
    (Topic::Navigation, "help-topic-navigation"),
    (Topic::Selecting, "help-topic-selecting"),
    (Topic::Drawing, "help-topic-drawing"),
    (Topic::Shapes, "help-topic-shapes"),
    (Topic::Modifying, "help-topic-modifying"),
    (Topic::Inference, "help-topic-inference"),
    (Topic::Measurements, "help-topic-measurements"),
    (Topic::Groups, "help-topic-groups"),
    (Topic::Library, "help-topic-library"),
    (Topic::Materials, "help-topic-materials"),
    (Topic::Organizing, "help-topic-organizing"),
    (Topic::Construction, "help-topic-construction"),
    (Topic::Styles, "help-topic-styles"),
    (Topic::Scenes, "help-topic-scenes"),
    (Topic::Files, "help-topic-files"),
    (Topic::Scripts, "help-topic-scripts"),
    (Topic::Shortcuts, "help-topic-shortcuts"),
    (Topic::Problems, "help-topic-problems"),
];

/// A block of a topic: a heading, a paragraph (`code` in backticks), a row (term, meaning),
/// cards of tools, or a button opening the Script Reference on a topic.
enum B {
    H(&'static str),
    P(&'static str),
    Row(&'static str, &'static str),
    Tools(&'static [ToolId]),
    Script(super::script_help::Topic, &'static str),
}

const START: &[B] = &[
    B::P("help-start-1"),
    B::H("help-start-2"),
    B::Row("help-start-3", "help-start-4"),
    B::Row("help-start-5", "help-start-6"),
    B::Row("help-start-7", "help-start-8"),
    B::Row("help-start-9", "help-start-10"),
    B::Row("help-start-11", "help-start-12"),
    B::H("help-start-13"),
    B::P("help-start-14"),
    B::P("help-start-15"),
    B::P("help-start-16"),
    B::P("help-start-17"),
    B::P("help-start-18"),
    B::H("help-start-19"),
    B::P("help-start-20"),
];

const NAVIGATION: &[B] = &[
    B::H("help-navigation-1"),
    B::Row("help-navigation-2", "help-navigation-3"),
    B::Row("help-navigation-4", "help-navigation-5"),
    B::Row("help-navigation-6", "help-navigation-7"),
    B::Row("help-navigation-8", "help-navigation-9"),
    B::Row("help-navigation-10", "help-navigation-11"),
    B::P("help-navigation-12"),
    B::P("help-navigation-13"),
    B::H("help-navigation-14"),
    B::P("help-navigation-15"),
    B::Tools(&[ToolId::Orbit, ToolId::Pan, ToolId::Zoom, ToolId::PositionCamera, ToolId::LookAround, ToolId::Walk]),
    B::P("help-navigation-16"),
    B::H("help-navigation-17"),
    B::P("help-navigation-18"),
    B::Row("help-navigation-19", "help-navigation-20"),
    B::Row("help-navigation-21", "help-navigation-22"),
    B::Row("help-navigation-23", "help-navigation-24"),
    B::P("help-navigation-25"),
    B::H("help-navigation-26"),
    B::P("help-navigation-27"),
    B::P("help-navigation-28"),
];

const SELECTING: &[B] = &[
    B::Tools(&[ToolId::Select]),
    B::P("help-selecting-1"),
    B::P("help-selecting-2"),
    B::P("help-selecting-3"),
];

const DRAWING: &[B] = &[
    B::P("help-drawing-1"),
    B::P("help-drawing-2"),
    B::Tools(&[ToolId::Line, ToolId::Rectangle, ToolId::RotatedRectangle, ToolId::Circle, ToolId::Polygon, ToolId::ArcCenter, ToolId::Arc, ToolId::Arc3, ToolId::Pie, ToolId::Bezier, ToolId::SmoothCurve, ToolId::Freehand, ToolId::EditCurve]),
];

const SHAPES: &[B] = &[
    B::P("help-shapes-1"),
    B::Tools(&crate::tools::SHAPES),
];

const MODIFYING: &[B] = &[
    B::P("help-modifying-1"),
    B::Row("help-modifying-2", "help-modifying-3"),
    B::Row("help-modifying-4", "help-modifying-5"),
    B::Tools(&[ToolId::Move, ToolId::Rotate, ToolId::Scale, ToolId::PushPull, ToolId::FollowMe, ToolId::Offset, ToolId::Chamfer, ToolId::Fillet, ToolId::Eraser, ToolId::SolidTools, ToolId::Slice]),
    B::H("help-modifying-6"),
    B::Row("help-modifying-7", "help-modifying-8"),
    B::Row("help-modifying-9", "help-modifying-10"),
    B::Row("help-modifying-11", "help-modifying-12"),
    B::Row("help-modifying-13", "help-modifying-14"),
    B::Row("help-modifying-15", "help-modifying-16"),
    B::Row("help-modifying-17", "help-modifying-18"),
];

const INFERENCE: &[B] = &[
    B::P("help-inference-1"),
    B::Row("help-inference-2", "help-inference-3"),
    B::Row("help-inference-4", "help-inference-5"),
    B::Row("help-inference-6", "help-inference-7"),
    B::Row("help-inference-8", "help-inference-9"),
    B::Row("help-inference-10", "help-inference-11"),
    B::Row("help-inference-12", "help-inference-13"),
    B::H("help-inference-14"),
    B::P("help-inference-15"),
    B::H("help-inference-16"),
    B::P("help-inference-17"),
    B::P("help-inference-18"),
    B::H("help-inference-19"),
    B::P("help-inference-20"),
];

const MEASUREMENTS: &[B] = &[
    B::P("help-measurements-1"),
    B::Row("help-measurements-2", "help-measurements-3"),
    B::Row("help-measurements-4", "help-measurements-5"),
    B::Row("help-measurements-6", "help-measurements-7"),
    B::Row("help-measurements-8", "help-measurements-9"),
    B::Row("help-measurements-10", "help-measurements-11"),
    B::Row("help-measurements-12", "help-measurements-13"),
    B::P("help-measurements-14"),
    B::P("help-measurements-15"),
];

const GROUPS: &[B] = &[
    B::P("help-groups-1"),
    B::Row("help-groups-2", "help-groups-3"),
    B::Row("help-groups-4", "help-groups-5"),
    B::Row("help-groups-6", "help-groups-7"),
    B::Row("help-groups-8", "help-groups-9"),
    B::Row("help-groups-10", "help-groups-11"),
    B::Row("help-groups-12", "help-groups-13"),
    B::H("help-groups-14"),
    B::P("help-groups-15"),
    B::P("help-groups-16"),
    B::P("help-groups-17"),
    B::Script(super::script_help::Topic::Component, "help-groups-18"),
    B::Script(super::script_help::Topic::Generators, "help-groups-19"),
];

const LIBRARY: &[B] = &[
    B::P("help-library-1"),
    B::Row("help-library-2", "help-library-3"),
    B::Row("help-library-4", "help-library-5"),
    B::Row("help-library-6", "help-library-7"),
    B::Row("help-library-8", "help-library-9"),
    B::P("help-library-10"),
    B::Row("help-library-11", "help-library-12"),
    B::Row("help-library-13", "help-library-14"),
    B::Row("help-library-15", "help-library-16"),
    B::H("help-library-17"),
    B::P("help-library-18"),
    B::H("help-library-19"),
    B::P("help-library-20"),
    B::P("help-library-21"),
];

const MATERIALS: &[B] = &[
    B::Tools(&[ToolId::Paint, ToolId::TexturePosition]),
    B::P("help-materials-1"),
    B::Row("help-materials-2", "help-materials-3"),
    B::Row("help-materials-4", "help-materials-5"),
    B::Row("help-materials-6", "help-materials-7"),
    B::P("help-materials-8"),
];

const ORGANIZING: &[B] = &[
    B::Row("help-organizing-1", "help-organizing-2"),
    B::Row("help-organizing-3", "help-organizing-4"),
    B::Row("help-organizing-5", "help-organizing-6"),
    B::Row("help-organizing-7", "help-modifying-6"),
    B::Row("help-organizing-8", "help-organizing-9"),
];

const CONSTRUCTION: &[B] = &[
    B::Tools(&[ToolId::Tape, ToolId::Protractor, ToolId::Dimension, ToolId::Text, ToolId::Axes, ToolId::Section, ToolId::GuidePlane]),
    B::P("help-construction-1"),
    B::P("help-construction-2"),
];

const STYLES: &[B] = &[
    B::H("help-styles-1"),
    B::Row("help-styles-2", "help-styles-3"),
    B::Row("help-styles-4", "help-styles-5"),
    B::Row("help-styles-6", "help-styles-7"),
    B::Row("help-styles-8", "help-styles-9"),
    B::Row("help-styles-10", "help-styles-11"),
    B::H("help-styles-12"),
    B::P("help-styles-13"),
    B::P("help-styles-14"),
    B::P("help-styles-15"),
];

const SCENES: &[B] = &[
    B::Row("help-scenes-1", "help-scenes-2"),
    B::Row("help-scenes-3", "help-scenes-4"),
    B::Row("help-scenes-5", "help-scenes-6"),
    B::Row("help-scenes-7", "help-scenes-8"),
    B::Row("help-scenes-9", "help-scenes-10"),
];

const FILES: &[B] = &[
    B::P("help-files-1"),
    B::P("help-files-2"),
    B::P("help-files-3"),
    B::Row("help-files-4", "help-files-5"),
    B::Row("help-files-6", "help-files-7"),
    B::Row("help-files-8", "help-files-9"),
    B::Row("help-files-10", "help-files-11"),
];

const SCRIPTS: &[B] = &[
    B::P("help-scripts-1"),
    B::P("help-scripts-2"),
    B::P("help-scripts-3"),
    B::Script(super::script_help::Topic::Basics, "help-scripts-4"),
    B::Script(super::script_help::Topic::Console, "help-scripts-5"),
    B::Script(super::script_help::Topic::Functions, "help-scripts-6"),
];

const SHORTCUTS: &[B] = &[
    B::P("help-shortcuts-1"),
];

const PROBLEMS: &[B] = &[
    B::Row("help-problems-1", "help-problems-2"),
    B::Row("help-problems-3", "help-problems-4"),
    B::Row("help-problems-5", "help-problems-6"),
    B::Row("help-problems-7", "help-problems-8"),
    B::Row("help-problems-9", "help-problems-10"),
    B::Row("help-problems-11", "help-problems-12"),
];

fn page(topic: Topic) -> &'static [B] {
    match topic {
        Topic::Start => START,
        Topic::Navigation => NAVIGATION,
        Topic::Selecting => SELECTING,
        Topic::Drawing => DRAWING,
        Topic::Shapes => SHAPES,
        Topic::Modifying => MODIFYING,
        Topic::Inference => INFERENCE,
        Topic::Measurements => MEASUREMENTS,
        Topic::Groups => GROUPS,
        Topic::Library => LIBRARY,
        Topic::Materials => MATERIALS,
        Topic::Organizing => ORGANIZING,
        Topic::Construction => CONSTRUCTION,
        Topic::Styles => STYLES,
        Topic::Scenes => SCENES,
        Topic::Files => FILES,
        Topic::Scripts => SCRIPTS,
        Topic::Shortcuts => SHORTCUTS,
        Topic::Problems => PROBLEMS,
    }
}

/// The topic that shows `tool`.
pub fn topic_of(tool: ToolId) -> Topic {
    TOPICS
        .iter()
        .find(|(t, _)| page(*t).iter().any(|b| matches!(b, B::Tools(ts) if ts.contains(&tool))))
        .map(|(t, _)| *t)
        .unwrap_or_default()
}

/// Keys other than the tools' own.
fn keys() -> Vec<(String, String)> {
    use crate::keymap::KeyCmd;
    // reassignable ones (Preferences → Shortcuts), then the fixed keys
    let mut v: Vec<(String, String)> = KeyCmd::all()
        .into_iter()
        .filter(|k| !matches!(k, KeyCmd::Tool(_)))
        .map(|k| (crate::keymap::labels(k), k.name()))
        .filter(|(k, _)| !k.is_empty())
        .collect();
    v.push((tr!("help-key-cut-copy-paste-keys"), tr!("help-key-cut-copy-paste")));
    v.extend(
        ["delete", "esc", "shift", "arrows", "enter", "f1"]
            .into_iter()
            .map(|k| (tr(&format!("help-key-{k}")), tr(&format!("help-key-{k}-what")))),
    );
    v
}

#[derive(Default)]
pub struct AppHelp {
    pub open: bool,
    pub topic: Topic,
    pub search: String,
    /// Bring the window to the front on the next frame.
    raise: bool,
}

/// What a click in the guide asks the app to do.
enum Ask {
    Tool(ToolId),
    Script(super::script_help::Topic),
    Topic(Topic),
}

/// A term in bold and what it means (`code` spans in both).
fn row(ui: &mut Ui, term: &str, what: &str) {
    ui.horizontal_top(|ui| {
        ui.add_space(4.0);
        ui.vertical(|ui| rich(ui, &format!("**{term}** — {what}")));
    });
}

/// A tool's card: icon, name, key, what it does and how (`Use` picks it).
fn tool_card(ui: &mut Ui, id: ToolId, ask: &mut Option<Ask>) {
    egui::Frame::new().fill(theme::tray_bg()).inner_margin(6.0).corner_radius(4.0).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::hover());
            super::icons::paint_icon(ui.painter(), rect, id.icon(), false);
            ui.label(RichText::new(id.name()).strong().size(14.0));
            if let Some(k) = id.shortcut_label() {
                ui.label(RichText::new(k).monospace().small().color(theme::text_dim()));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button(tr!("help-use")).on_hover_text(tr!("help-use-hint")).clicked() {
                    *ask = Some(Ask::Tool(id));
                }
            });
        });
        ui.label(RichText::new(id.description()).italics());
        rich(ui, &id.instructor());
    });
    ui.add_space(4.0);
}

fn blocks_ui(ui: &mut Ui, blocks: &[B], ask: &mut Option<Ask>) {
    for b in blocks {
        match b {
            B::H(t) => {
                ui.add_space(6.0);
                ui.label(RichText::new(tr!(t)).strong().size(15.0));
            }
            B::P(t) => rich(ui, &tr!(t)),
            B::Row(a, b) => row(ui, &tr!(a), &tr!(b)),
            B::Tools(ids) => {
                ui.add_space(2.0);
                for id in *ids {
                    tool_card(ui, *id, ask);
                }
            }
            B::Script(t, label) => {
                if ui.link(format!("→ {}", tr!(label))).clicked() {
                    *ask = Some(Ask::Script(*t));
                }
            }
        }
        ui.add_space(3.0);
    }
}

fn topic_ui(ui: &mut Ui, topic: Topic, ask: &mut Option<Ask>) {
    ui.label(RichText::new(tr!(TOPICS.iter().find(|t| t.0 == topic).map(|t| t.1).unwrap_or_default())).strong().size(18.0));
    ui.add_space(4.0);
    blocks_ui(ui, page(topic), ask);
    if topic == Topic::Shortcuts {
        ui.label(RichText::new(tr!("help-tools")).strong().size(15.0));
        for id in ToolId::ALL {
            if id.shortcut_label().is_some() {
                row(ui, &crate::keymap::labels(crate::keymap::KeyCmd::Tool(id)), &id.name());
            }
        }
        ui.add_space(6.0);
        ui.label(RichText::new(tr!("help-commands")).strong().size(15.0));
        for (k, w) in keys() {
            row(ui, &k, &w);
        }
        ui.add_space(6.0);
        rich(ui, &tr!("help-shortcuts-note"));
    }
}

/// The text of a block that a search looks at.
fn block_text(b: &B) -> Option<String> {
    match b {
        B::H(t) | B::P(t) => Some(tr!(t)),
        B::Row(a, b) => Some(format!("{} {}", tr!(a), tr!(b))),
        B::Script(_, l) => Some(tr!(l)),
        B::Tools(_) => None,
    }
}

/// Tools whose name, description or instructions contain `q` (lower case).
pub fn find_tools(q: &str) -> Vec<ToolId> {
    ToolId::ALL
        .iter()
        .copied()
        .filter(|t| t.name().to_lowercase().contains(q) || t.description().to_lowercase().contains(q) || t.instructor().to_lowercase().contains(q))
        .collect()
}

/// Topic blocks containing `q` (lower case): the topic and the block's index.
fn find_blocks(q: &str) -> Vec<(Topic, usize)> {
    let mut v = Vec::new();
    for (t, name) in TOPICS {
        if tr!(name).to_lowercase().contains(q) {
            v.push((*t, usize::MAX));
        }
        for (i, b) in page(*t).iter().enumerate() {
            if block_text(b).is_some_and(|s| s.to_lowercase().contains(q)) {
                v.push((*t, i));
            }
        }
    }
    v
}

fn search_ui(ui: &mut Ui, q: &str, ask: &mut Option<Ask>) {
    let tools = find_tools(q);
    let hits = find_blocks(q);
    if tools.is_empty() && hits.is_empty() {
        ui.label(RichText::new(tr!("help-nothing-found")).color(theme::text_dim()));
        return;
    }
    if !hits.is_empty() {
        ui.label(RichText::new(tr!("help-in-the-guide")).strong().size(15.0));
        let mut last = None;
        for (t, i) in hits {
            if last != Some(t) {
                ui.add_space(4.0);
                let name = tr!(TOPICS.iter().find(|x| x.0 == t).map(|x| x.1).unwrap_or_default());
                if ui.link(RichText::new(name).strong()).clicked() {
                    *ask = Some(Ask::Topic(t));
                }
                last = Some(t);
            }
            if let Some(b) = page(t).get(i) {
                ui.indent(("hit", t as u8, i), |ui| blocks_ui(ui, std::slice::from_ref(b), ask));
            }
        }
    }
    if !tools.is_empty() {
        ui.add_space(6.0);
        ui.label(RichText::new(tr!("help-tools")).strong().size(15.0));
        for id in tools {
            tool_card(ui, id, ask);
        }
    }
}

impl VarstotasApp {
    /// Open the guide on `topic` (in front).
    pub(crate) fn open_app_help(&mut self, topic: Topic) {
        self.app_help.open = true;
        self.app_help.raise = true;
        self.app_help.topic = topic;
        self.app_help.search.clear();
    }

    /// F1 over the model: the guide on the active tool.
    pub(crate) fn app_help_for_tool(&mut self) {
        let t = topic_of(self.tool_id());
        self.open_app_help(t);
    }

    pub(crate) fn app_help_window(&mut self, ctx: &Context) {
        if !self.app_help.open {
            return;
        }
        let mut open = true;
        let mut ask = None;
        let pos = self.viewport.view_rect.left_top() + egui::vec2(60.0, 30.0);
        let win = self.placed(egui::Window::new(tr!("help-window-title")).id(egui::Id::new("app_help")).open(&mut open).default_size([700.0, 640.0]).resizable(true), pos);
        win.show(ctx, |ui| {
            let h = &mut self.app_help;
            ui.horizontal(|ui| {
                ui.label(tr!("help-search"));
                let r = ui.add(egui::TextEdit::singleline(&mut h.search).hint_text(tr!("help-search-hint")).desired_width(240.0));
                if !h.search.is_empty() && ui.small_button("×").clicked() {
                    h.search.clear();
                    r.request_focus();
                }
            });
            ui.separator();
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(190.0);
                    ScrollArea::vertical().id_salt("app_help_topics").auto_shrink([true, false]).show(ui, |ui| {
                        for (t, name) in TOPICS {
                            let sel = h.search.is_empty() && h.topic == *t;
                            if ui.add(egui::Button::selectable(sel, tr!(name))).clicked() {
                                h.topic = *t;
                                h.search.clear();
                            }
                        }
                    });
                });
                ui.separator();
                ui.vertical(|ui| {
                    ScrollArea::vertical().id_salt(("app_help", h.topic as u8, h.search.is_empty())).auto_shrink([false, false]).show(ui, |ui| {
                        let q = h.search.trim().to_lowercase();
                        if q.is_empty() {
                            topic_ui(ui, h.topic, &mut ask);
                        } else {
                            search_ui(ui, &q, &mut ask);
                        }
                    });
                });
            });
        });
        self.app_help.open = open;
        if std::mem::take(&mut self.app_help.raise) {
            ctx.move_to_top(egui::LayerId::new(egui::Order::Middle, egui::Id::new("app_help")));
        }
        match ask {
            Some(Ask::Tool(id)) => self.actions.push(Action::SetTool(id)),
            Some(Ask::Script(t)) => self.open_script_help(t, None),
            Some(Ask::Topic(t)) => self.open_app_help(t),
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every tool is shown in exactly one topic (F1 finds it there).
    #[test]
    fn every_tool_is_in_one_topic() {
        for id in ToolId::ALL {
            let n = TOPICS.iter().filter(|(t, _)| page(*t).iter().any(|b| matches!(b, B::Tools(ts) if ts.contains(&id)))).count();
            assert_eq!(n, 1, "{} is in {n} topics", id.name());
        }
        assert_eq!(topic_of(ToolId::PushPull), Topic::Modifying);
        assert_eq!(topic_of(ToolId::Sphere), Topic::Shapes);
    }

    #[test]
    fn search_finds_topics_and_tools() {
        assert!(find_tools("push").contains(&ToolId::PushPull));
        assert!(find_blocks("autosave").iter().any(|h| h.0 == Topic::Files));
        assert!(find_blocks("zoom extents").iter().any(|h| h.0 == Topic::Navigation));
    }
}
