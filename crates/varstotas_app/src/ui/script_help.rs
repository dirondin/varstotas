//! Help → Script Reference: the scripting documentation inside the app. Topics for each place
//! scripts run (Script Console and plugins, generators, the component parameter script), every
//! API function by section and the properties of objects (both from `varstotas_script::help`, so
//! the reference follows the API), with a search. The "?" buttons of the three editors open it
//! on their topic; F1 in an editor looks up the word at the cursor.

use super::theme;
use crate::app::VarstotasApp;
use egui::{Color32, Context, RichText, ScrollArea, Ui, text::LayoutJob};
use varstotas_i18n::tr;
use varstotas_script::help;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Topic {
    #[default]
    Basics,
    Console,
    Generators,
    Component,
    Functions,
    Properties,
}

/// The topics with the keys of their names.
pub const TOPICS: &[(Topic, &str)] = &[
    (Topic::Basics, "console-topic-basics"),
    (Topic::Console, "console-topic-console"),
    (Topic::Generators, "console-topic-generators"),
    (Topic::Component, "console-topic-component"),
    (Topic::Functions, "console-topic-functions"),
    (Topic::Properties, "console-topic-properties"),
];

#[derive(Default)]
pub struct ScriptHelp {
    pub open: bool,
    pub topic: Topic,
    pub search: String,
    /// Bring the window to the front on the next frame (it was just asked for).
    raise: bool,
}

/// A block of a topic page: a heading, a paragraph (`code` in backticks), a code example or a
/// table row (term, meaning). Headings, paragraphs and meanings are message keys; code examples
/// and terms are shown as written.
enum B {
    /// Key of a heading.
    H(&'static str),
    /// Key of a paragraph.
    P(&'static str),
    Code(&'static str),
    /// A term as written, the key of its meaning.
    Row(&'static str, &'static str),
}

const BASICS: &[B] = &[
    B::P("console-ref-basics-1"),
    B::H("console-ref-basics-2"),
    B::Code("let w = 1200;                 // a variable\nconst H = 30;                 // a constant\nlet v = vec3(0, 0, H);        // a point / vector\nlet a = [1, 2, 3];            // an array\nlet m = #{ w: 10, d: 20 };    // a map: m.w, m[\"d\"]\nfn half(x) { x / 2 }          // a function (the last expression is returned)\nif w > 1000 { print(\"wide\") } else { print(\"narrow\") }\nfor i in 0..3 { box(vec3(i * 400, 0, 0), 300, 300, 300); }\nlet big = a.filter(|x| x > 1);   // a closure"), // i18n-ignore
    B::P("console-ref-basics-4"),
    B::P("console-ref-basics-5"),
    B::H("console-ref-basics-6"),
    B::P("console-ref-basics-7"),
    B::P("console-ref-basics-8"),
    B::P("console-ref-basics-9"),
    B::H("console-ref-basics-10"),
    B::P("console-ref-basics-11"),
    B::P("console-ref-basics-12"),
    B::P("console-ref-basics-13"),
];

const CONSOLE: &[B] = &[
    B::P("console-ref-console-1"),
    B::P("console-ref-console-2"),
    B::H("console-ref-console-3"),
    B::P("console-ref-console-4"),
    B::Code("// @name Shelf Row\n// @menu Furniture / Shelf Row\n// @param count:int = 3 \"Shelves\"\n// @param gap = 400 \"Gap\"\n// @pick 1 \"Where to start\"\nfor i in 0..count {\n    box(points[0] + X * i * gap, 300, 250, 18);\n}"), // i18n-ignore
    B::H("console-ref-console-6"),
];

const GENERATORS: &[B] = &[
    B::P("console-ref-generators-1"),
    B::P("console-ref-generators-2"),
    B::Code("// @generator Shelf\n// @description An open shelf unit.\n// @param width:length(200..3000/10) = 800 \"Width\"\n// @param shelves:int(0..12) = 3 \"Shelves\"\n// @param finish:choice(Oak|White) = Oak \"Finish\"\n// @formula step = 1000 / (shelves + 1) \"Step\"\n// @check width >= 300 || shelves == 0 \"Too narrow for shelves\"\n\nfn build(p) {\n    let m = material(p.finish, if p.finish == \"Oak\" { \"#C8A26B\" } else { \"#F2F0EB\" });\n    for i in 0..=p.shelves + 1 {\n        paint(box(vec3(0, 0, i * p.step), p.width, 300, 18), m);\n    }\n}"), // i18n-ignore
    B::H("console-ref-generators-4"),
    B::Row("@generator Name", "console-ref-generators-5"), // i18n-ignore
    B::Row("@param key:type(min..max/step) = default \"Label\"", "console-ref-generators-6"), // i18n-ignore
    B::Row("@formula key[:type] = expression \"Label\"", "console-ref-generators-7"), // i18n-ignore
    B::Row("@check expression \"Message\"", "console-ref-generators-8"), // i18n-ignore
    B::H("console-ref-generators-9"),
];

const COMPONENT: &[B] = &[
    B::P("console-ref-component-1"),
    B::P("console-ref-component-2"),
    B::Code("fn update(p) {\n    let door = child(\"Door\");                 // a part by name\n    rotate(door, door.min, Z, -p.open);       // turn about its own corner\n    show(child(\"Knob\", true), p.open < 45);  // a nested part (deep search)\n    if p.shelf {\n        let s = box(vec3(10, 10, door.size.z / 2), 580, 380, 18);\n        s.name = \"Shelf\";\n    }\n    set_params(child(\"Leg\"), #{ height: p.h });  // a generated part's values\n}"), // i18n-ignore
    B::H("console-ref-component-4"),
    B::Row("child(name)", "console-ref-component-5"), // i18n-ignore
    B::Row("child(name, true)", "console-ref-component-6"), // i18n-ignore
    B::Row("child(part, name, deep)", "console-ref-component-7"), // i18n-ignore
    B::Row("children(), children(true)", "console-ref-component-8"), // i18n-ignore
    B::Row("part.min / max / size / bounds", "console-ref-component-9"), // i18n-ignore
    B::Row("part.hidden = true", "console-ref-component-10"), // i18n-ignore
    B::H("console-ref-component-11"),
    B::P("console-ref-component-12"),
    B::P("console-ref-component-13"),
    B::P("console-ref-component-14"),
];

fn page(topic: Topic) -> &'static [B] {
    match topic {
        Topic::Basics => BASICS,
        Topic::Console => CONSOLE,
        Topic::Generators => GENERATORS,
        Topic::Component => COMPONENT,
        Topic::Functions | Topic::Properties => &[],
    }
}

const CODE_BG: Color32 = Color32::from_rgb(246, 246, 243);

/// Text with `code` spans in the monospace font and **strong** spans.
pub(crate) fn rich(ui: &mut Ui, text: &str) {
    let mut job = LayoutJob::default();
    let body = egui::TextFormat { font_id: egui::TextStyle::Body.resolve(ui.style()), color: ui.visuals().text_color(), ..Default::default() };
    let code = egui::TextFormat {
        font_id: egui::TextStyle::Monospace.resolve(ui.style()),
        color: ui.visuals().strong_text_color(),
        background: if ui.visuals().dark_mode { Color32::from_gray(50) } else { CODE_BG },
        ..Default::default()
    };
    let strong = egui::TextFormat { color: ui.visuals().strong_text_color(), ..body.clone() };
    for (i, part) in text.split('`').enumerate() {
        if i % 2 == 1 {
            job.append(part, 0.0, code.clone());
            continue;
        }
        for (j, p) in part.split("**").enumerate() {
            job.append(p, 0.0, if j % 2 == 1 { strong.clone() } else { body.clone() });
        }
    }
    job.wrap.max_width = ui.available_width();
    ui.label(job);
}

/// A code example with a Copy button.
fn code_block(ui: &mut Ui, code: &str) {
    let bg = if ui.visuals().dark_mode { Color32::from_gray(35) } else { CODE_BG };
    egui::Frame::new().fill(bg).inner_margin(6.0).corner_radius(3.0).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_top(|ui| {
            ui.add(egui::Label::new(RichText::new(code).monospace()).wrap_mode(egui::TextWrapMode::Extend));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if ui.small_button(tr!("log-copy")).on_hover_text(tr!("console-copy-example")).clicked() {
                    ui.ctx().copy_text(code.to_string());
                }
            });
        });
    });
}

pub(crate) fn row(ui: &mut Ui, term: &str, what: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(term).monospace().strong());
        ui.label(RichText::new(format!("— {what}")).color(theme::text_dim()));
    });
}

/// A function's entry: its overloads (click copies) and what it does.
fn fn_entry(ui: &mut Ui, d: &help::FnDoc) {
    ui.add_space(2.0);
    for sig in d.sigs {
        if ui.add(egui::Label::new(RichText::new(*sig).monospace().strong()).sense(egui::Sense::click())).on_hover_text(tr!("console-click-copy")).clicked() {
            ui.ctx().copy_text(sig.to_string());
        }
    }
    ui.indent(d.name, |ui| {
        ui.label(RichText::new(d.text()).color(theme::text_dim()));
    });
}

fn member_entry(ui: &mut Ui, m: &help::Member) {
    let on: Vec<String> = m.on.iter().map(|t| t.label()).collect();
    let name = match m.sig {
        Some(sig) => format!(".{sig}"),
        None => format!(".{}", m.name),
    };
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(name).monospace().strong());
        ui.label(RichText::new(format!("→ {}", m.ty.label())).small());
        ui.label(RichText::new(format!("— {} ({})", m.text(), on.join(", "))).color(theme::text_dim()));
    });
}

/// Functions whose name, signature or description contain `q` (lower case).
pub fn find_fns(q: &str) -> Vec<&'static help::FnDoc> {
    help::FNS.iter().filter(|d| d.name.contains(q) || d.sigs.iter().any(|s| s.to_lowercase().contains(q)) || d.text().to_lowercase().contains(q)).collect()
}

fn find_members(q: &str) -> Vec<&'static help::Member> {
    help::MEMBERS.iter().filter(|m| m.name.contains(q) || m.text().to_lowercase().contains(q)).collect()
}

impl VarstotasApp {
    /// Open the reference on `topic` (or on the search `word` when it names a function or a
    /// property).
    pub(crate) fn open_script_help(&mut self, topic: Topic, word: Option<String>) {
        self.script_help.open = true;
        self.script_help.raise = true;
        self.script_help.topic = topic;
        let word = word.filter(|w| help::fn_doc(w).is_some() || help::MEMBERS.iter().any(|m| m.name == w.as_str()) || help::TAGS.iter().any(|t| t.0 == w.as_str()));
        self.script_help.search = word.unwrap_or_default();
    }

    pub(crate) fn script_help_window(&mut self, ctx: &Context) {
        if !self.script_help.open {
            return;
        }
        let mut open = true;
        let pos = self.viewport.view_rect.right_top() + egui::vec2(-660.0, 30.0);
        let mut win = egui::Window::new(tr!("console-ref-title")).id(egui::Id::new("script_help")).open(&mut open).default_pos(pos).default_size([620.0, 520.0]).resizable(true);
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        win.show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(tr!("console-search"));
                let r = ui.add(egui::TextEdit::singleline(&mut self.script_help.search).hint_text(tr!("console-search-hint")).desired_width(220.0));
                if !self.script_help.search.is_empty() && ui.small_button("×").clicked() {
                    self.script_help.search.clear();
                    r.request_focus();
                }
                ui.label(RichText::new(tr!("console-fulltext")).small().color(theme::text_dim()));
            });
            ui.separator();
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(150.0);
                    for (t, key) in TOPICS {
                        let sel = self.script_help.search.is_empty() && self.script_help.topic == *t;
                        if ui.add(egui::Button::selectable(sel, varstotas_i18n::tr(key))).clicked() {
                            self.script_help.topic = *t;
                            self.script_help.search.clear();
                        }
                    }
                });
                ui.separator();
                ui.vertical(|ui| {
                    ScrollArea::vertical().id_salt(("script_help", self.script_help.topic as u8, self.script_help.search.is_empty())).auto_shrink([false, false]).show(ui, |ui| {
                        let q = self.script_help.search.trim().to_lowercase();
                        if q.is_empty() {
                            topic_ui(ui, self.script_help.topic);
                        } else {
                            search_ui(ui, &q);
                        }
                    });
                });
            });
        });
        self.script_help.open = open;
        if std::mem::take(&mut self.script_help.raise) {
            ctx.move_to_top(egui::LayerId::new(egui::Order::Middle, egui::Id::new("script_help")));
        }
    }
}

fn topic_ui(ui: &mut Ui, topic: Topic) {
    for b in page(topic) {
        match b {
            B::H(t) => {
                ui.add_space(6.0);
                ui.label(RichText::new(varstotas_i18n::tr(t)).strong().size(15.0));
            }
            B::P(t) => rich(ui, &varstotas_i18n::tr(t)),
            B::Code(c) => code_block(ui, c),
            B::Row(a, b) => row(ui, a, &varstotas_i18n::tr(b)),
        }
        ui.add_space(3.0);
    }
    match topic {
        Topic::Console => {
            for (_, syntax, what) in help::TAGS.iter().filter(|t| !matches!(t.0, "generator" | "formula" | "check")) {
                row(ui, syntax, what);
            }
            ui.add_space(6.0);
            ui.label(RichText::new(tr!("console-param-types")).strong().size(15.0));
            for (t, what) in help::PARAM_TYPES.iter().filter(|t| !help::is_generator_type(t.0)) {
                row(ui, t, &help::param_type_text(what, false));
            }
        }
        Topic::Generators => {
            for (t, what) in help::PARAM_TYPES {
                row(ui, t, &help::param_type_text(what, true));
            }
            ui.add_space(6.0);
            rich(ui, &tr!("console-generators-note"));
        }
        Topic::Functions => {
            rich(ui, &tr!("console-functions-intro"));
            for (title, names) in help::GROUPS {
                ui.add_space(6.0);
                egui::CollapsingHeader::new(RichText::new(tr(title)).strong()).default_open(true).show(ui, |ui| {
                    for n in *names {
                        if let Some(d) = help::fn_doc(n) {
                            fn_entry(ui, d);
                        }
                    }
                });
            }
        }
        Topic::Properties => {
            rich(ui, &tr!("console-properties-intro"));
            let kinds: Vec<(String, &[help::Ty])> = vec![
                (tr!("console-kind-objects"), &[help::Ty::Face, help::Ty::Edge, help::Ty::Group]),
                (tr!("console-kind-vectors"), &[help::Ty::Vector]),
                (tr!("console-kind-bounds"), &[help::Ty::Bounds]),
                (tr!("console-kind-arrays"), &[help::Ty::Array]),
                (tr!("console-kind-strings"), &[help::Ty::Str]),
            ];
            for (title, tys) in kinds {
                ui.add_space(6.0);
                egui::CollapsingHeader::new(RichText::new(title).strong()).default_open(true).show(ui, |ui| {
                    for m in help::MEMBERS.iter().filter(|m| m.on.iter().any(|t| tys.contains(t))) {
                        member_entry(ui, m);
                    }
                });
            }
        }
        _ => {}
    }
}

fn search_ui(ui: &mut Ui, q: &str) {
    let fns = find_fns(q);
    let members = find_members(q);
    let tags: Vec<_> = help::TAGS.iter().filter(|t| t.0.contains(q) || tr(t.2).to_lowercase().contains(q)).collect();
    if fns.is_empty() && members.is_empty() && tags.is_empty() {
        ui.label(RichText::new(tr!("console-nothing-found")).color(theme::text_dim()));
        return;
    }
    if !fns.is_empty() {
        ui.label(RichText::new(tr!("console-topic-functions")).strong().size(15.0));
        for d in fns {
            fn_entry(ui, d);
        }
    }
    if !members.is_empty() {
        ui.add_space(6.0);
        ui.label(RichText::new(tr!("console-members")).strong().size(15.0));
        for m in members {
            member_entry(ui, m);
        }
    }
    if !tags.is_empty() {
        ui.add_space(6.0);
        ui.label(RichText::new(tr!("console-tags")).strong().size(15.0));
        for (_, syntax, what) in tags {
            row(ui, syntax, &tr(what));
        }
    }
}

/// The "?" button of a script editor: opens the reference on `topic`.
pub fn help_button(ui: &mut Ui) -> bool {
    ui.button(tr!("console-reference")).on_hover_text(tr!("console-reference-hint")).clicked()
}
