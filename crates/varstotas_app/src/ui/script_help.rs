//! Help → Script Reference: the scripting documentation inside the app. Topics for each place
//! scripts run (Script Console and plugins, generators, the component parameter script), every
//! API function by section and the properties of objects (both from `varstotas_script::help`, so
//! the reference follows the API), with a search. The "?" buttons of the three editors open it
//! on their topic; F1 in an editor looks up the word at the cursor.

use super::theme;
use crate::app::VarstotasApp;
use egui::{Color32, Context, RichText, ScrollArea, Ui, text::LayoutJob};
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

pub const TOPICS: &[(Topic, &str)] = &[
    (Topic::Basics, "Basics"),
    (Topic::Console, "Console and plugins"),
    (Topic::Generators, "Generators"),
    (Topic::Component, "Component script"),
    (Topic::Functions, "Functions"),
    (Topic::Properties, "Properties"),
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
/// table row (term, meaning).
enum B {
    H(&'static str),
    P(&'static str),
    Code(&'static str),
    Row(&'static str, &'static str),
}

const BASICS: &[B] = &[
    B::P("Scripts are written in Rhai — a small language with Rust / JavaScript-like syntax. Coordinates are world millimetres; lengths with units are strings: `length(\"1.2m\")`."),
    B::H("Language"),
    B::Code("let w = 1200;                 // a variable\nconst H = 30;                 // a constant\nlet v = vec3(0, 0, H);        // a point / vector\nlet a = [1, 2, 3];            // an array\nlet m = #{ w: 10, d: 20 };    // a map: m.w, m[\"d\"]\nfn half(x) { x / 2 }          // a function (the last expression is returned)\nif w > 1000 { print(\"wide\") } else { print(\"narrow\") }\nfor i in 0..3 { box(vec3(i * 400, 0, 0), 300, 300, 300); }\nlet big = a.filter(|x| x > 1);   // a closure"),
    B::P("Strings: `\"text\"`, joined with `+`: `\"w = \" + w`. String methods `trim()`, `to_upper()` change the string in place and return nothing: `s.trim(); if s == \"\" { … }`."),
    B::P("Constants: `ORIGIN`, `X`, `Y`, `Z` (unit axes). Vectors add, subtract and scale: `ORIGIN + X * 100`; `.x .y .z`, `length()`, `normalize()`, `dot(b)`, `cross(b)`, `distance(b)`."),
    B::H("Objects"),
    B::P("Faces, edges, groups and components are references: `let g = box(ORIGIN, 100, 100, 100); g.name = \"Crate\";`. After operations that merge or cut geometry an old reference may become invalid — check `e.valid`."),
    B::P("Anywhere an object is expected, a string names a group / component of the current context: `move(\"Door\", vec3(0, 10, 0))`. `child(\"Knob\", true)` searches nested groups too."),
    B::P("Every object has `kind`, `bounds`, `size`, `min`, `max`, `center`, `hidden`; groups also `name`, `definition`, `material`, `tag` (see Properties)."),
    B::H("Context and undo"),
    B::P("New geometry goes into the current context: the group open when the script started, or the one entered with `enter(g)` (leave with `exit()`)."),
    B::P("A run is one undo step. A script that fails changes nothing; the error with its line is shown — and kept in Window → Log."),
    B::P("`print(x)` writes to the output; the value of the last expression is printed too. Scripts stop after 30 s (generators 10 s)."),
];

const CONSOLE: &[B] = &[
    B::P("Window → Script Console runs code against the open model. Cmd+Enter runs, Cmd+↑ / ↓ goes through earlier runs, Open… / Save… work with `.rhai` files. File → Run Script… runs a file."),
    B::P("Completion shows API functions while typing, properties after a dot (by the type on the left), and the signature of the function being called above the cursor. F1 opens this reference on the word at the cursor."),
    B::H("Header: dialogs and menu items"),
    B::P("Lines `// @tag …` at the top turn a script into a plugin: Run first asks the parameters, points and objects. A file in `~/.varstotas/plugins` becomes an Extensions menu item."),
    B::Code("// @name Shelf Row\n// @menu Furniture / Shelf Row\n// @param count:int = 3 \"Shelves\"\n// @param gap = 400 \"Gap\"\n// @pick 1 \"Where to start\"\nfor i in 0..count {\n    box(points[0] + X * i * gap, 300, 250, 18);\n}"),
    B::H("Tags"),
];

const GENERATORS: &[B] = &[
    B::P("A generator is a component with parameters: a script whose `build(p)` builds the component's content in its own axes (the origin is the insertion point). Each copy has its own values in Entity Info; copies with equal values share one definition."),
    B::P("Window → Generator Editor… (or Edit Generator… on a tile): the source on the left, a live preview with parameter fields on the right; Apply updates the document, Save to Library writes the file."),
    B::Code("// @generator Shelf\n// @description An open shelf unit.\n// @param width:length(200..3000/10) = 800 \"Width\"\n// @param shelves:int(0..12) = 3 \"Shelves\"\n// @param finish:choice(Oak|White) = Oak \"Finish\"\n// @formula step = 1000 / (shelves + 1) \"Step\"\n// @check width >= 300 || shelves == 0 \"Too narrow for shelves\"\n\nfn build(p) {\n    let m = material(p.finish, if p.finish == \"Oak\" { \"#C8A26B\" } else { \"#F2F0EB\" });\n    for i in 0..=p.shelves + 1 {\n        paint(box(vec3(0, 0, i * p.step), p.width, 300, 18), m);\n    }\n}"),
    B::H("Header"),
    B::Row("@generator Name", "required; the generator's key in the document"),
    B::Row("@param key:type(min..max/step) = default \"Label\"", "a value in `p`; every part of the range is optional"),
    B::Row("@formula key[:type] = expression \"Label\"", "a value computed from the parameters (read-only in Entity Info)"),
    B::Row("@check expression \"Message\"", "values for which it is false are refused"),
    B::H("Parameter types"),
];

const COMPONENT: &[B] = &[
    B::P("Window → Component Parameters… gives any component parameters: rows of values, formulas, motions of its parts (turn, slide, show, paint, set a part's parameter) and an optional script."),
    B::P("The script defines `update(p)`. It runs after the motions, inside the component, with the whole script API — the same functions as the console and generators. `p` holds the values and the formulas."),
    B::Code("fn update(p) {\n    let door = child(\"Door\");                 // a part by name\n    rotate(door, door.min, Z, -p.open);       // turn about its own corner\n    show(child(\"Knob\", true), p.open < 45);  // a nested part (deep search)\n    if p.shelf {\n        let s = box(vec3(10, 10, door.size.z / 2), 580, 380, 18);\n        s.name = \"Shelf\";\n    }\n    set_params(child(\"Leg\"), #{ height: p.h });  // a generated part's values\n}"),
    B::H("Finding parts"),
    B::Row("child(name)", "a group / component directly inside the component"),
    B::Row("child(name, true)", "searched in nested groups too (the nearest one wins)"),
    B::Row("child(part, name, deep)", "inside a part"),
    B::Row("children(), children(true)", "all parts (nested ones too)"),
    B::Row("part.min / max / size / bounds", "the part's extents now (world)"),
    B::Row("part.hidden = true", "hide, like hide(part)"),
    B::H("How it works"),
    B::P("The script changes a copy (the variant for these values), never the master: parts it changes are copied into the variant, parts it leaves alone stay shared. Copies with equal values share one variant."),
    B::P("Edits made inside a copy (double click) go back to the master without what the script did: what it added is dropped, moves / hiding / paint are undone."),
    B::P("Errors are shown when Apply is pressed, and kept in Window → Log."),
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
                if ui.small_button("Copy").on_hover_text("Copy the example").clicked() {
                    ui.ctx().copy_text(code.to_string());
                }
            });
        });
    });
}

pub(crate) fn row(ui: &mut Ui, term: &str, what: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(term).monospace().strong());
        ui.label(RichText::new(format!("— {what}")).color(theme::TEXT_DIM));
    });
}

/// A function's entry: its overloads (click copies) and what it does.
fn fn_entry(ui: &mut Ui, d: &help::FnDoc) {
    ui.add_space(2.0);
    for sig in d.sigs {
        if ui.add(egui::Label::new(RichText::new(*sig).monospace().strong()).sense(egui::Sense::click())).on_hover_text("Click to copy").clicked() {
            ui.ctx().copy_text(sig.to_string());
        }
    }
    ui.indent(d.name, |ui| {
        ui.label(RichText::new(d.doc).color(theme::TEXT_DIM));
    });
}

fn member_entry(ui: &mut Ui, m: &help::Member) {
    let on: Vec<&str> = m.on.iter().map(|t| t.label()).collect();
    let name = match m.sig {
        Some(sig) => format!(".{sig}"),
        None => format!(".{}", m.name),
    };
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(name).monospace().strong());
        ui.label(RichText::new(format!("→ {}", m.ty.label())).small());
        ui.label(RichText::new(format!("— {} ({})", m.doc, on.join(", "))).color(theme::TEXT_DIM));
    });
}

/// Functions whose name, signature or description contain `q` (lower case).
pub fn find_fns(q: &str) -> Vec<&'static help::FnDoc> {
    help::FNS.iter().filter(|d| d.name.contains(q) || d.sigs.iter().any(|s| s.to_lowercase().contains(q)) || d.doc.to_lowercase().contains(q)).collect()
}

fn find_members(q: &str) -> Vec<&'static help::Member> {
    help::MEMBERS.iter().filter(|m| m.name.contains(q) || m.doc.to_lowercase().contains(q)).collect()
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
        let mut win = egui::Window::new("Script Reference").id(egui::Id::new("script_help")).open(&mut open).default_pos(pos).default_size([620.0, 520.0]).resizable(true);
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        win.show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Search");
                let r = ui.add(egui::TextEdit::singleline(&mut self.script_help.search).hint_text("function, property or word").desired_width(220.0));
                if !self.script_help.search.is_empty() && ui.small_button("×").clicked() {
                    self.script_help.search.clear();
                    r.request_focus();
                }
                ui.label(RichText::new("Full text: docs/SCRIPTING.md, docs/GENERATORS.md").small().color(theme::TEXT_DIM));
            });
            ui.separator();
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(150.0);
                    for (t, name) in TOPICS {
                        let sel = self.script_help.search.is_empty() && self.script_help.topic == *t;
                        if ui.add(egui::Button::selectable(sel, *name)).clicked() {
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
                ui.label(RichText::new(*t).strong().size(15.0));
            }
            B::P(t) => rich(ui, t),
            B::Code(c) => code_block(ui, c),
            B::Row(a, b) => row(ui, a, b),
        }
        ui.add_space(3.0);
    }
    match topic {
        Topic::Console => {
            for (_, syntax, what) in help::TAGS.iter().filter(|t| !matches!(t.0, "generator" | "formula" | "check")) {
                row(ui, syntax, what);
            }
            ui.add_space(6.0);
            ui.label(RichText::new("Parameter types").strong().size(15.0));
            for (t, what) in help::PARAM_TYPES.iter().filter(|t| !t.1.starts_with("generator")) {
                row(ui, t, what);
            }
        }
        Topic::Generators => {
            for (t, what) in help::PARAM_TYPES {
                row(ui, t, what.trim_start_matches("generator: "));
            }
            ui.add_space(6.0);
            rich(ui, "Inside `build` the whole API works (see Functions). The script runs in an empty model holding the document's materials and tags; everything it builds becomes the component's content.");
        }
        Topic::Functions => {
            rich(ui, "Every function of the API, by section. Click a signature to copy it. `x` is an object, an array of objects or a name of a child group.");
            for (title, names) in help::GROUPS {
                ui.add_space(6.0);
                egui::CollapsingHeader::new(RichText::new(*title).strong()).default_open(true).show(ui, |ui| {
                    for n in *names {
                        if let Some(d) = help::fn_doc(n) {
                            fn_entry(ui, d);
                        }
                    }
                });
            }
        }
        Topic::Properties => {
            rich(ui, "After a dot: `g.name`, `f.normal.x`, `door.size.z`, `[1, 2].len()`. Properties marked assignable can be set: `g.name = \"Crate\"`, `g.hidden = true`.");
            let kinds: &[(&str, &[help::Ty])] = &[
                ("Objects (faces, edges, groups, components)", &[help::Ty::Face, help::Ty::Edge, help::Ty::Group]),
                ("Vectors", &[help::Ty::Vector]),
                ("Bounds — bounds(x), x.bounds", &[help::Ty::Bounds]),
                ("Arrays", &[help::Ty::Array]),
                ("Strings", &[help::Ty::Str]),
            ];
            for (title, tys) in kinds {
                ui.add_space(6.0);
                egui::CollapsingHeader::new(RichText::new(*title).strong()).default_open(true).show(ui, |ui| {
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
    let tags: Vec<_> = help::TAGS.iter().filter(|t| t.0.contains(q) || t.2.to_lowercase().contains(q)).collect();
    if fns.is_empty() && members.is_empty() && tags.is_empty() {
        ui.label(RichText::new("Nothing found").color(theme::TEXT_DIM));
        return;
    }
    if !fns.is_empty() {
        ui.label(RichText::new("Functions").strong().size(15.0));
        for d in fns {
            fn_entry(ui, d);
        }
    }
    if !members.is_empty() {
        ui.add_space(6.0);
        ui.label(RichText::new("Properties and methods").strong().size(15.0));
        for m in members {
            member_entry(ui, m);
        }
    }
    if !tags.is_empty() {
        ui.add_space(6.0);
        ui.label(RichText::new("Header tags").strong().size(15.0));
        for (_, syntax, what) in tags {
            row(ui, syntax, what);
        }
    }
}

/// The "?" button of a script editor: opens the reference on `topic`.
pub fn help_button(ui: &mut Ui) -> bool {
    ui.button("? Reference").on_hover_text("Help → Script Reference: functions, properties, examples (F1 in the editor: the word at the cursor)").clicked()
}
