//! The Script Console's code editor: `egui_code_editor` (Rhai highlighting, line numbers,
//! completion from the API, keywords and the script's own words), plus our own completion of
//! properties and methods after a dot by the receiver's type (`g.` after `let g = box(…)` →
//! `name`, `definition`, …) and of header tags (`// @param`, `@param key:type`), and hints —
//! the signature of the call at the cursor with the current argument underlined, or the
//! syntax of the header tag on the line (`varstotas_script::help`).

use egui::text::{CCursor, LayoutJob, TextFormat};
use egui::text_edit::TextEditOutput;
use egui::{Align2, Color32, Event, FontId, Id, Key, Modifiers, Order, Ui};
use egui_code_editor::{CodeEditor, ColorTheme, Completer, Syntax};
use varstotas_script::help;
use std::collections::BTreeSet;

/// The language an editor is for.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    /// Rhai scripts (`varstotas_script::help`).
    Rhai,
    /// WGSL custom surface shaders (`crate::shader_help`).
    Wgsl,
}

pub struct ScriptEditor {
    lang: Lang,
    syntax: Syntax,
    completer: Completer,
    /// Id of the text field (known after the first frame), for focus checks.
    pub text_id: Option<Id>,
    /// The member list shown last frame (its keys are read before the field).
    members: Option<MemberPopup>,
    /// Esc closed the member list for the member starting here.
    dismissed: Option<usize>,
    /// Header variables in the completer's dictionary (it is rebuilt when they change).
    header_vars: Vec<String>,
    /// Id of the editor (two editors can be open at once).
    id: &'static str,
    /// Cursor (char index) last frame.
    cursor: Option<usize>,
}

struct MemberPopup {
    items: Vec<Item>,
    sel: usize,
    /// Char index where the completed word starts.
    start: usize,
}

/// A row of our completion list.
struct Item {
    name: String,
    /// Shown instead of the name (`dot(b)`, `@param`).
    label: String,
    /// Inserted after the typed prefix.
    insert: String,
    detail: String,
}

impl Item {
    /// A property or method (methods get their parentheses).
    fn member(m: &help::Member, prefix: &str) -> Self {
        let rest = m.name.get(prefix.len()..).unwrap_or_default();
        let insert = match m.sig {
            Some(sig) if sig.ends_with("()") => format!("{rest}()"),
            Some(_) => format!("{rest}("),
            None => rest.to_string(),
        };
        let what = if m.sig.is_some() { m.ty.label().to_string() } else { format!(": {}", m.ty.label()) };
        Self { name: m.name.to_string(), label: m.sig.unwrap_or(m.name).to_string(), insert, detail: format!("{what}  — {}", m.doc) }
    }
}

/// The list for the text before the cursor: header tags, `@param` types or members after a dot.
fn items_at(lang: Lang, text: &str, idx: usize) -> Option<(Vec<Item>, String, usize)> {
    if lang == Lang::Wgsl {
        let (rows, prefix, start) = crate::shader_help::items_at(text, idx)?;
        let items = rows.into_iter().map(|(name, label, insert, detail)| Item { name, label, insert, detail }).collect();
        return Some((items, prefix, start));
    }
    let rest = |name: &str, prefix: &str| name.get(prefix.len()..).unwrap_or_default().to_string();
    if let Some((ctx, prefix, start)) = help::tag_context(text, idx) {
        let items = match ctx {
            help::TagCtx::Tag => help::TAGS
                .iter()
                .filter(|t| t.0.starts_with(prefix.as_str()))
                .map(|t| Item { name: t.0.to_string(), label: format!("@{}", t.0), insert: format!("{} ", rest(t.0, &prefix)), detail: format!("{}  — {}", t.1, t.2) })
                .collect(),
            help::TagCtx::ParamType => help::PARAM_TYPES
                .iter()
                .filter(|t| t.0.starts_with(prefix.as_str()))
                .map(|t| Item { name: t.0.to_string(), label: t.0.to_string(), insert: rest(t.0, &prefix), detail: format!("— {}", t.1) })
                .collect(),
        };
        return Some((items, prefix, start));
    }
    let (ty, prefix, start) = help::member_context(text, idx)?;
    let items = help::members_for(ty).into_iter().filter(|m| m.name.starts_with(prefix.as_str())).map(|m| Item::member(m, &prefix)).collect();
    Some((items, prefix, start))
}

impl Default for ScriptEditor {
    fn default() -> Self {
        let syntax = Syntax::new("rhai")
            .with_case_sensitive(true)
            .with_comment("//")
            .with_comment_multiline(["/*", "*/"])
            .with_keywords(BTreeSet::from_iter(help::KEYWORDS.iter().copied()))
            .with_types(BTreeSet::from_iter(help::CONSTANTS.iter().copied()))
            .with_special(BTreeSet::from_iter(help::FNS.iter().map(|f| f.name)));
        let completer = Completer::new_with_syntax(&syntax).with_user_words().with_auto_indent();
        Self { lang: Lang::Rhai, syntax, completer, text_id: None, members: None, dismissed: None, header_vars: Vec::new(), id: "script_console_code", cursor: None }
    }
}

impl ScriptEditor {
    /// An editor with its own id (the default one is the Script Console's).
    pub fn with_id(id: &'static str) -> Self {
        Self { id, ..Self::default() }
    }

    /// A WGSL shader editor (`crate::shader_help`): `@param`s marked and completed after `p.`.
    pub fn wgsl(id: &'static str) -> Self {
        use crate::shader_help as sh;
        let syntax = Syntax::new("wgsl")
            .with_case_sensitive(true)
            .with_comment("//")
            .with_comment_multiline(["/*", "*/"])
            .with_keywords(BTreeSet::from_iter(sh::KEYWORDS.iter().copied()))
            .with_types(BTreeSet::from_iter(sh::TYPES.iter().copied()))
            .with_special(BTreeSet::from_iter(sh::FNS.iter().map(|f| f.0)));
        let completer = Completer::new_with_syntax(&syntax).with_user_words().with_auto_indent();
        Self { lang: Lang::Wgsl, syntax, completer, id, ..Self::default() }
    }

    /// The line of `code` the cursor was on last frame (for the harness).
    pub fn cursor_line(&self, code: &str) -> String {
        let Some(c) = self.cursor else { return String::new() };
        let chars: Vec<char> = code.chars().collect();
        let c = c.min(chars.len());
        let start = chars[..c].iter().rposition(|x| *x == '\n').map_or(0, |p| p + 1);
        let end = chars[c..].iter().position(|x| *x == '\n').map_or(chars.len(), |p| c + p);
        chars[start..end].iter().collect()
    }

    /// Names in the member list now open (for the harness).
    pub fn member_names(&self) -> Vec<String> {
        self.members.as_ref().map(|p| p.items.iter().map(|i| i.name.clone()).collect()).unwrap_or_default()
    }

    pub fn has_focus(&self, ui: &Ui) -> bool {
        self.text_id.is_some_and(|id| ui.memory(|m| m.has_focus(id)))
    }

    /// The editor filling `height` (scrolls when the code is longer).
    pub fn show(&mut self, ui: &mut Ui, code: &mut String, height: f32) -> TextEditOutput {
        let font = ui.text_style_height(&egui::TextStyle::Monospace);
        let rows = ((height - 8.0) / (font + 2.0)).floor().max(2.0) as usize;
        let focused = self.has_focus(ui);
        // GitHub colours with comments in dark green (a lighter green on the dark theme).
        let theme = if ui.visuals().dark_mode {
            ColorTheme { comments: "6a9955", ..ColorTheme::GITHUB_DARK }
        } else {
            ColorTheme { comments: "1a7f37", ..ColorTheme::GITHUB_LIGHT }
        };
        // Variables from the script header (`@param` keys, points, objects, path): dark teal,
        // and offered by the completer.
        let vars: Vec<String> = match self.lang {
            Lang::Rhai => help::header_vars(code).into_iter().map(|(n, _)| n).collect(),
            Lang::Wgsl => varstotas_kernel::shader_params(code).into_iter().map(|p| p.name).collect(),
        };
        if vars != self.header_vars {
            self.completer = Completer::new_with_syntax(&self.syntax).with_user_words().with_auto_indent();
            for v in &vars {
                self.completer.push_word(v);
            }
            self.header_vars = vars.clone();
        }
        let marked = if ui.visuals().dark_mode { Color32::from_rgb(0x4e, 0xc9, 0xb0) } else { Color32::from_rgb(0x00, 0x7a, 0x87) };
        // The completer reads keys before the field (↑ / ↓ / Tab / Esc in its popup); only
        // while the editor has focus, so Esc and arrows still reach the viewport otherwise.
        if focused && let Some(pop) = &mut self.members {
            // The member list owns ↑ / ↓ / Tab / Enter / Esc while it is open.
            ui.input_mut(|i| {
                let n = pop.items.len();
                if i.consume_key(Modifiers::NONE, Key::ArrowDown) {
                    pop.sel = (pop.sel + 1) % n;
                } else if i.consume_key(Modifiers::NONE, Key::ArrowUp) {
                    pop.sel = (pop.sel + n - 1) % n;
                } else if i.consume_key(Modifiers::NONE, Key::Tab) || i.consume_key(Modifiers::NONE, Key::Enter) {
                    i.events.push(Event::Paste(pop.items[pop.sel].insert.clone()));
                } else if i.consume_key(Modifiers::NONE, Key::Escape) {
                    // Esc closes the list only; egui would also take the focus from the field.
                    self.dismissed = Some(pop.start);
                }
            });
        } else if focused {
            self.completer.handle_input(ui.ctx());
        }
        let mut editor = CodeEditor::default()
            .id_source(self.id)
            .with_rows(rows)
            .with_fontsize(font)
            .with_theme(theme)
            .with_numlines(true)
            .with_clickable_links(false)
            .with_doc_tags(true)
            .with_marked_words(vars, marked)
            .vscroll(false);
        let mut out = egui::ScrollArea::vertical()
            .id_salt((self.id, "scroll"))
            .max_height(height)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| editor.show(ui, code, &self.syntax).0)
            .inner;
        self.text_id = Some(out.response.id);
        self.cursor = out.cursor_range.map(|r| r.primary.index.0);
        if out.response.has_focus() {
            self.update_members(&out);
            match &self.members {
                Some(pop) => member_popup(ui, &out, pop, font),
                None => self.completer.show(&self.syntax, &theme, font, &mut out),
            }
            param_hint(ui, &out, font, self.lang);
            // egui drops the focus on Esc before any widget runs; while a popup is open the
            // field keeps it (the popup closes instead).
            let popup = self.members.is_some() || self.completer.is_open();
            let filter = egui::EventFilter { tab: true, horizontal_arrows: true, vertical_arrows: true, escape: popup };
            ui.memory_mut(|m| m.set_focus_lock_filter(out.response.id, filter));
        } else {
            self.members = None;
        }
        out
    }

    /// The member list for the text at the cursor (after `receiver.`), keeping the selection.
    fn update_members(&mut self, out: &TextEditOutput) {
        let text = out.galley.text();
        let lang = self.lang;
        let ctx = out.cursor_range.and_then(|r| items_at(lang, text, r.primary.index.0.min(text.chars().count())));
        let Some((items, prefix, start)) = ctx else {
            self.members = None;
            self.dismissed = None;
            return;
        };
        if self.dismissed == Some(start) {
            self.members = None;
            return;
        }
        self.dismissed = None;
        if items.is_empty() || (items.len() == 1 && items[0].name == prefix) {
            self.members = None;
            return;
        }
        let sel = match &self.members {
            Some(old) if old.start == start => old.items.get(old.sel).and_then(|m| items.iter().position(|x| x.name == m.name)).unwrap_or(0),
            _ => 0,
        };
        self.members = Some(MemberPopup { items, sel, start });
    }
}

/// The member list under the cursor: name, call / type, description; the selected row framed.
fn member_popup(ui: &Ui, out: &TextEditOutput, pop: &MemberPopup, font: f32) {
    let text = out.galley.text();
    let idx = out.cursor_range.map_or(0, |r| r.primary.index.0.min(text.chars().count()));
    let cursor = out.galley.pos_from_cursor(CCursor::new(idx)).translate(out.galley_pos.to_vec2());
    let weak = ui.visuals().weak_text_color();
    let strong = ui.visuals().text_color();
    let sel_bg = ui.visuals().selection.bg_fill.gamma_multiply(0.35);
    egui::Area::new(Id::new("script_member_list"))
        .order(Order::Tooltip)
        .interactable(false)
        .fixed_pos(cursor.left_bottom() + egui::vec2(0.0, 2.0))
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style()).inner_margin(egui::Margin::symmetric(4, 3)).show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                ui.spacing_mut().item_spacing.y = 0.0;
                // Up to 12 rows around the selection.
                let from = pop.sel.saturating_sub(11);
                for (i, m) in pop.items.iter().enumerate().skip(from).take(12) {
                    let mut job = LayoutJob::default();
                    job.append(&m.label, 0.0, TextFormat::simple(FontId::monospace(font), strong));
                    job.append(&format!("  {}", m.detail), 0.0, TextFormat::simple(FontId::proportional(font * 0.85), weak));
                    let fill = if i == pop.sel { sel_bg } else { Color32::TRANSPARENT };
                    egui::Frame::new().fill(fill).corner_radius(3.0).inner_margin(egui::Margin::symmetric(4, 1)).show(ui, |ui| {
                        ui.label(job);
                    });
                }
                if pop.items.len() > 12 {
                    ui.label(egui::RichText::new(format!("{} of {}", pop.sel + 1, pop.items.len())).small().color(weak));
                }
            });
        });
}

/// The signature of the call around the cursor, above the cursor line.
fn param_hint(ui: &Ui, out: &TextEditOutput, font: f32, lang: Lang) {
    let Some(range) = out.cursor_range else { return };
    let text = out.galley.text();
    let idx = range.primary.index.0.min(text.chars().count());
    if lang == Lang::Wgsl {
        let t = crate::shader_help::PARAM_TAG;
        let job = if crate::shader_help::on_param_line(text, idx) {
            let mut job = LayoutJob::default();
            job.append(t.1, 0.0, TextFormat::simple(FontId::monospace(font), ui.visuals().text_color()));
            job.append(&format!("  — {}", t.2), 0.0, TextFormat::simple(FontId::proportional(font * 0.9), ui.visuals().weak_text_color()));
            job
        } else {
            let Some((name, arg, false)) = help::call_context(text, idx) else { return };
            let Some((sig, doc)) = crate::shader_help::fn_doc(&name) else { return };
            hint_job(ui, sig, arg, doc, font)
        };
        return show_hint(ui, out, idx, job);
    }
    if let Some(t) = help::tag_at(text, idx) {
        let mut job = LayoutJob::default();
        job.append(t.1, 0.0, TextFormat::simple(FontId::monospace(font), ui.visuals().text_color()));
        job.append(&format!("  — {}", t.2), 0.0, TextFormat::simple(FontId::proportional(font * 0.9), ui.visuals().weak_text_color()));
        return show_hint(ui, out, idx, job);
    }
    let Some((name, arg, member)) = help::call_context(text, idx) else { return };
    let job = if member {
        let Some(m) = help::MEMBERS.iter().find(|m| m.name == name && m.sig.is_some()) else { return };
        hint_job(ui, m.sig.unwrap_or(m.name), arg, m.doc, font)
    } else {
        let Some(d) = help::fn_doc(&name) else { return };
        // The first overload with enough arguments, else the longest.
        let sig = d.sigs.iter().find(|s| help::sig_args(s).len() > arg).unwrap_or_else(|| d.sigs.last().unwrap());
        let mut job = hint_job(ui, sig, arg, d.doc, font);
        if d.sigs.len() > 1 {
            let more = format!("   (+{} more)", d.sigs.len() - 1);
            job.append(&more, 0.0, TextFormat::simple(FontId::proportional(font * 0.85), ui.visuals().weak_text_color()));
        }
        job
    };
    show_hint(ui, out, idx, job);
}

/// A one-line hint just above the cursor line.
fn show_hint(ui: &Ui, out: &TextEditOutput, idx: usize, job: LayoutJob) {
    let cursor = out.galley.pos_from_cursor(CCursor::new(idx)).translate(out.galley_pos.to_vec2());
    egui::Area::new(Id::new("script_param_hint"))
        .order(Order::Tooltip)
        .interactable(false)
        .pivot(Align2::LEFT_BOTTOM)
        .fixed_pos(cursor.left_top() - egui::vec2(4.0, 2.0))
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style()).inner_margin(egui::Margin::symmetric(6, 3)).show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                ui.label(job);
            });
        });
}

/// `name(a, b, c)` with argument `arg` in bold and colour, then the description.
fn hint_job(ui: &Ui, sig: &str, arg: usize, doc: &str, font: f32) -> LayoutJob {
    let mono = FontId::monospace(font);
    let base = ui.visuals().text_color();
    let strong = if ui.visuals().dark_mode { Color32::from_rgb(120, 180, 255) } else { Color32::from_rgb(20, 90, 200) };
    let mut job = LayoutJob::default();
    let plain = TextFormat::simple(mono.clone(), base);
    let hi = TextFormat { underline: egui::Stroke::new(1.0, strong), ..TextFormat::simple(mono.clone(), strong) };
    match sig.find('(') {
        Some(open) => {
            job.append(&sig[..=open], 0.0, plain.clone());
            let args = help::sig_args(sig);
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    job.append(", ", 0.0, plain.clone());
                }
                job.append(a, 0.0, if i == arg { hi.clone() } else { plain.clone() });
            }
            job.append(")", 0.0, plain);
        }
        None => job.append(sig, 0.0, plain),
    }
    job.append(&format!("  — {doc}"), 0.0, TextFormat::simple(FontId::proportional(font * 0.9), ui.visuals().weak_text_color()));
    job
}
