//! Help → User Guide: the documentation of the whole application inside it. Topics from the
//! first steps to files and scripts; the tools are shown from their own texts (`ToolId::
//! description` / `instructor`, the same as the Tool Guide card), so the guide follows the
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
    Workshop,
    WsParts,
    WsCutList,
    WsDrilling,
    WsJoinery,
    WsHardware,
    WsFittings,
    WsCarcass,
    WsAssembly,
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
    (Topic::Workshop, "help-topic-workshop"),
    (Topic::WsParts, "help-topic-ws-parts"),
    (Topic::WsCutList, "help-topic-ws-cutlist"),
    (Topic::WsDrilling, "help-topic-ws-drilling"),
    (Topic::WsJoinery, "help-topic-ws-joinery"),
    (Topic::WsHardware, "help-topic-ws-hardware"),
    (Topic::WsFittings, "help-topic-ws-fittings"),
    (Topic::WsCarcass, "help-topic-ws-carcass"),
    (Topic::WsAssembly, "help-topic-ws-assembly"),
    (Topic::Files, "help-topic-files"),
    (Topic::Scripts, "help-topic-scripts"),
    (Topic::Shortcuts, "help-topic-shortcuts"),
    (Topic::Problems, "help-topic-problems"),
];

/// A block of a topic: a heading, a paragraph (`code` in backticks, `**bold**`), a row (term,
/// meaning), a bullet, a numbered step (counted from the last heading), a framed tip, a link to
/// a subtopic with what is in it, cards of tools, or a button opening the Script Reference on a
/// topic.
enum B {
    H(&'static str),
    P(&'static str),
    Row(&'static str, &'static str),
    Li(&'static str),
    Step(&'static str),
    Tip(&'static str),
    Sub(Topic, &'static str),
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
    B::Row("help-navigation-29", "help-navigation-30"),
    B::Row("help-navigation-31", "help-navigation-32"),
    B::Row("help-navigation-33", "help-navigation-34"),
    B::P("help-navigation-35"),
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

const WORKSHOP: &[B] = &[
    B::P("help-workshop-1"),
    B::H("help-workshop-2"),
    B::Step("help-workshop-3"),
    B::Step("help-workshop-4"),
    B::Step("help-workshop-5"),
    B::Step("help-workshop-6"),
    B::Step("help-workshop-7"),
    B::Tip("help-workshop-8"),
    B::H("help-workshop-9"),
    B::Sub(Topic::WsParts, "help-workshop-10"),
    B::Sub(Topic::WsCutList, "help-workshop-11"),
    B::Sub(Topic::WsDrilling, "help-workshop-12"),
    B::Sub(Topic::WsJoinery, "help-workshop-13"),
    B::Sub(Topic::WsHardware, "help-workshop-14"),
    B::Sub(Topic::WsFittings, "help-workshop-15"),
    B::Sub(Topic::WsCarcass, "help-workshop-16"),
    B::Sub(Topic::WsAssembly, "help-workshop-17"),
    B::Script(super::script_help::Topic::Functions, "help-workshop-18"),
];

const WS_PARTS: &[B] = &[
    B::P("help-ws-parts-1"),
    B::H("help-ws-parts-2"),
    B::Li("help-ws-parts-3"),
    B::Li("help-ws-parts-4"),
    B::Li("help-ws-parts-5"),
    B::H("help-ws-parts-6"),
    B::P("help-ws-parts-7"),
    B::Li("help-ws-parts-8"),
    B::Li("help-ws-parts-9"),
    B::Li("help-ws-parts-10"),
    B::Tip("help-ws-parts-11"),
    B::H("help-ws-parts-12"),
    B::P("help-ws-parts-13"),
];

const WS_CUT_LIST: &[B] = &[
    B::P("help-ws-cutlist-1"),
    B::H("help-ws-cutlist-2"),
    B::Li("help-ws-cutlist-3"),
    B::Li("help-ws-cutlist-4"),
    B::Li("help-ws-cutlist-5"),
    B::H("help-ws-cutlist-6"),
    B::Li("help-ws-cutlist-7"),
    B::Li("help-ws-cutlist-8"),
    B::Li("help-ws-cutlist-9"),
    B::Tip("help-ws-cutlist-10"),
    B::H("help-ws-cutlist-11"),
    B::P("help-ws-cutlist-12"),
    B::P("help-ws-cutlist-13"),
    B::H("help-ws-cutlist-14"),
    B::Li("help-ws-cutlist-15"),
    B::Li("help-ws-cutlist-16"),
    B::H("help-ws-cutlist-17"),
    B::P("help-ws-cutlist-18"),
    B::H("help-ws-cutlist-19"),
    B::Li("help-ws-cutlist-20"),
    B::Li("help-ws-cutlist-21"),
    B::Li("help-ws-cutlist-22"),
    B::Sub(Topic::WsDrilling, "help-ws-cutlist-23"),
];

const WS_DRILLING: &[B] = &[
    B::P("help-ws-drilling-1"),
    B::H("help-ws-drilling-2"),
    B::Li("help-ws-drilling-3"),
    B::Li("help-ws-drilling-4"),
    B::Li("help-ws-drilling-5"),
    B::H("help-ws-drilling-6"),
    B::P("help-ws-drilling-7"),
    B::Tip("help-ws-drilling-8"),
    B::H("help-ws-drilling-9"),
    B::Li("help-ws-drilling-10"),
    B::Li("help-ws-drilling-11"),
    B::H("help-ws-drilling-12"),
    B::P("help-ws-drilling-13"),
    B::Tip("help-ws-drilling-14"),
];

const WS_JOINERY: &[B] = &[
    B::P("help-ws-joinery-1"),
    B::H("help-ws-joinery-2"),
    B::Li("help-ws-joinery-3"),
    B::Li("help-ws-joinery-4"),
    B::Li("help-ws-joinery-5"),
    B::Sub(Topic::WsHardware, "help-ws-joinery-6"),
    B::H("help-ws-joinery-7"),
    B::P("help-ws-joinery-8"),
    B::H("help-ws-joinery-9"),
    B::Li("help-ws-joinery-10"),
    B::Li("help-ws-joinery-11"),
    B::Li("help-ws-joinery-12"),
    B::P("help-ws-joinery-13"),
    B::H("help-ws-joinery-14"),
    B::P("help-ws-joinery-15"),
    B::Li("help-ws-joinery-16"),
    B::Li("help-ws-joinery-17"),
    B::Li("help-ws-joinery-18"),
    B::H("help-ws-joinery-19"),
    B::P("help-ws-joinery-20"),
    B::H("help-ws-joinery-21"),
    B::P("help-ws-joinery-22"),
];

const WS_HARDWARE: &[B] = &[
    B::P("help-ws-hardware-1"),
    B::H("help-ws-hardware-2"),
    B::Li("help-ws-hardware-3"),
    B::Li("help-ws-hardware-4"),
    B::Li("help-ws-hardware-5"),
    B::Li("help-ws-hardware-6"),
    B::Li("help-ws-hardware-7"),
    B::Li("help-ws-hardware-8"),
    B::Li("help-ws-hardware-9"),
    B::Tip("help-ws-hardware-10"),
];

const WS_FITTINGS: &[B] = &[
    B::P("help-ws-fittings-1"),
    B::H("help-ws-fittings-2"),
    B::Step("help-ws-fittings-3"),
    B::Step("help-ws-fittings-4"),
    B::Step("help-ws-fittings-5"),
    B::Step("help-ws-fittings-6"),
    B::H("help-ws-fittings-7"),
    B::P("help-ws-fittings-8"),
    B::Tip("help-ws-fittings-9"),
    B::H("help-ws-fittings-10"),
    B::P("help-ws-fittings-11"),
    B::Li("help-ws-fittings-12"),
    B::Li("help-ws-fittings-13"),
    B::Li("help-ws-fittings-14"),
    B::H("help-ws-fittings-15"),
    B::P("help-ws-fittings-16"),
];

const WS_CARCASS: &[B] = &[
    B::P("help-ws-carcass-1"),
    B::H("help-ws-carcass-2"),
    B::P("help-ws-carcass-3"),
    B::Li("help-ws-carcass-4"),
    B::Li("help-ws-carcass-5"),
    B::H("help-ws-carcass-6"),
    B::P("help-ws-carcass-7"),
    B::H("help-ws-carcass-8"),
    B::P("help-ws-carcass-9"),
    B::H("help-ws-carcass-10"),
    B::P("help-ws-carcass-11"),
];

const WS_ASSEMBLY: &[B] = &[
    B::P("help-ws-assembly-1"),
    B::H("help-ws-assembly-2"),
    B::P("help-ws-assembly-3"),
    B::H("help-ws-assembly-4"),
    B::P("help-ws-assembly-5"),
    B::H("help-ws-assembly-6"),
    B::P("help-ws-assembly-7"),
    B::H("help-ws-assembly-8"),
    B::P("help-ws-assembly-9"),
    B::P("help-ws-assembly-10"),
    B::H("help-ws-assembly-11"),
    B::P("help-ws-assembly-12"),
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

/// The topic `topic` is a subtopic of, shown under it in the list of topics.
pub fn parent(topic: Topic) -> Option<Topic> {
    use Topic::*;
    matches!(topic, WsParts | WsCutList | WsDrilling | WsJoinery | WsHardware | WsFittings | WsCarcass | WsAssembly).then_some(Workshop)
}

fn topic_name(topic: Topic) -> String {
    tr!(TOPICS.iter().find(|t| t.0 == topic).map(|t| t.1).unwrap_or_default())
}

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
        Topic::Workshop => WORKSHOP,
        Topic::WsParts => WS_PARTS,
        Topic::WsCutList => WS_CUT_LIST,
        Topic::WsDrilling => WS_DRILLING,
        Topic::WsJoinery => WS_JOINERY,
        Topic::WsHardware => WS_HARDWARE,
        Topic::WsFittings => WS_FITTINGS,
        Topic::WsCarcass => WS_CARCASS,
        Topic::WsAssembly => WS_ASSEMBLY,
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

/// A line of text after a mark (a bullet or a step's number) kept in its own column.
fn marked(ui: &mut Ui, mark: &str, text: &str) {
    ui.horizontal_top(|ui| {
        ui.add_space(8.0);
        ui.add_sized([16.0, 14.0], egui::Label::new(RichText::new(mark).strong().color(theme::text_dim())));
        ui.vertical(|ui| rich(ui, text));
    });
}

fn blocks_ui(ui: &mut Ui, blocks: &[B], ask: &mut Option<Ask>) {
    let mut step = 0;
    for b in blocks {
        match b {
            B::H(t) => {
                step = 0;
                ui.add_space(8.0);
                ui.label(RichText::new(tr!(t)).strong().size(15.0));
            }
            B::P(t) => rich(ui, &tr!(t)),
            B::Row(a, b) => row(ui, &tr!(a), &tr!(b)),
            B::Li(t) => marked(ui, "•", &tr!(t)),
            B::Step(t) => {
                step += 1;
                marked(ui, &format!("{step}."), &tr!(t));
            }
            B::Tip(t) => {
                egui::Frame::new().fill(theme::tray_bg()).stroke(egui::Stroke::new(1.0, theme::text_dim())).inner_margin(6.0).corner_radius(4.0).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    rich(ui, &tr!(t));
                });
            }
            B::Sub(t, what) => {
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(8.0);
                    if ui.link(RichText::new(format!("→ {}", topic_name(*t))).strong()).clicked() {
                        *ask = Some(Ask::Topic(*t));
                    }
                    ui.label(RichText::new(format!("— {}", tr!(what))).color(theme::text_dim()));
                });
            }
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
    if let Some(p) = parent(topic)
        && ui.link(RichText::new(format!("← {}", topic_name(p))).small()).clicked()
    {
        *ask = Some(Ask::Topic(p));
    }
    ui.label(RichText::new(topic_name(topic)).strong().size(18.0));
    ui.add_space(4.0);
    blocks_ui(ui, page(topic), ask);
    // the next subtopic, to read a section through
    if let Some(p) = parent(topic) {
        let subs: Vec<Topic> = TOPICS.iter().map(|t| t.0).filter(|t| parent(*t) == Some(p)).collect();
        if let Some(next) = subs.iter().position(|t| *t == topic).and_then(|i| subs.get(i + 1)) {
            ui.add_space(8.0);
            if ui.link(format!("{} →", topic_name(*next))).clicked() {
                *ask = Some(Ask::Topic(*next));
            }
        }
    }
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
        B::H(t) | B::P(t) | B::Li(t) | B::Step(t) | B::Tip(t) => Some(tr!(t)),
        B::Sub(t, w) => Some(format!("{} {}", topic_name(*t), tr!(w))),
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
                let name = topic_name(t);
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
                        // subtopics only under the open topic, so the list stays short
                        let open = parent(h.topic).unwrap_or(h.topic);
                        for (t, name) in TOPICS {
                            let sub = parent(*t);
                            if sub.is_some_and(|p| p != open) {
                                continue;
                            }
                            let sel = h.search.is_empty() && h.topic == *t;
                            let button = egui::Button::selectable(sel, tr!(name));
                            let r = if sub.is_some() {
                                ui.horizontal(|ui| {
                                    ui.add_space(14.0);
                                    ui.add(button)
                                })
                                .inner
                            } else {
                                ui.add(button)
                            };
                            super::tray_lib::spot(format!("help:topic:{t:?}"), r.rect.center());
                            if r.clicked() {
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
        assert!(find_blocks("crossed-out").iter().any(|h| h.0 == Topic::WsDrilling));
    }

    /// Every subtopic is linked from its topic's page.
    #[test]
    fn subtopics_are_linked_from_their_topic() {
        for (t, _) in TOPICS {
            if let Some(p) = parent(*t) {
                assert!(page(p).iter().any(|b| matches!(b, B::Sub(s, _) if s == t)), "{t:?} is not linked from {p:?}");
            }
        }
    }
}
