//! The Script Console's code editor: `egui_code_editor` (Rhai highlighting, line numbers,
//! completion from the API, keywords and the script's own words) plus parameter hints — the
//! signature of the call at the cursor with the current argument in bold
//! (`varstotas_script::help`).

use egui::text::{CCursor, LayoutJob, TextFormat};
use egui::text_edit::TextEditOutput;
use egui::{Align2, Color32, FontId, Id, Order, Ui};
use egui_code_editor::{CodeEditor, ColorTheme, Completer, Syntax};
use varstotas_script::help;
use std::collections::BTreeSet;

pub struct ScriptEditor {
    syntax: Syntax,
    completer: Completer,
    /// Id of the text field (known after the first frame), for focus checks.
    pub text_id: Option<Id>,
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
        let mut completer = Completer::new_with_syntax(&syntax).with_user_words().with_auto_indent();
        // Properties / methods of entities and vectors (array methods would only crowd the list).
        for (m, _) in help::MEMBERS.iter().filter(|(m, _)| !matches!(*m, "push" | "len")) {
            completer.push_word(m);
        }
        Self { syntax, completer, text_id: None }
    }
}

impl ScriptEditor {
    pub fn has_focus(&self, ui: &Ui) -> bool {
        self.text_id.is_some_and(|id| ui.memory(|m| m.has_focus(id)))
    }

    /// The editor filling `height` (scrolls when the code is longer).
    pub fn show(&mut self, ui: &mut Ui, code: &mut String, height: f32) -> TextEditOutput {
        let font = ui.text_style_height(&egui::TextStyle::Monospace);
        let rows = ((height - 8.0) / (font + 2.0)).floor().max(2.0) as usize;
        let focused = self.has_focus(ui);
        let theme = if ui.visuals().dark_mode { ColorTheme::GITHUB_DARK } else { ColorTheme::GITHUB_LIGHT };
        // The completer reads keys before the field (↑ / ↓ / Tab / Esc in its popup); only
        // while the editor has focus, so Esc and arrows still reach the viewport otherwise.
        if focused {
            self.completer.handle_input(ui.ctx());
        }
        let mut editor = CodeEditor::default()
            .id_source("script_console_code")
            .with_rows(rows)
            .with_fontsize(font)
            .with_theme(theme)
            .with_numlines(true)
            .with_clickable_links(false)
            .vscroll(false);
        let mut out = egui::ScrollArea::vertical()
            .id_salt("console_code_scroll")
            .max_height(height)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| editor.show(ui, code, &self.syntax).0)
            .inner;
        self.text_id = Some(out.response.id);
        if out.response.has_focus() {
            self.completer.show(&self.syntax, &theme, font, &mut out);
            param_hint(ui, &out, font);
        }
        out
    }
}

/// The signature of the call around the cursor, above the cursor line.
fn param_hint(ui: &Ui, out: &TextEditOutput, font: f32) {
    let Some(range) = out.cursor_range else { return };
    let text = out.galley.text();
    let idx = range.primary.index.0.min(text.chars().count());
    let Some((name, arg, member)) = help::call_context(text, idx) else { return };
    let job = if member {
        let Some((_, doc)) = help::MEMBERS.iter().find(|(m, _)| *m == name) else { return };
        hint_job(ui, &format!("{name}()"), usize::MAX, doc, font)
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
