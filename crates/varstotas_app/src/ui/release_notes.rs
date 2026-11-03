//! Help → What's New: the versions and what each brought, from `CHANGELOG.md` (built into the
//! app; the same section heads a release's notes on GitHub).

use crate::app::VarstotasApp;
use crate::ui::theme;
use egui::{Context, RichText};
use varstotas_i18n::tr;

const CHANGELOG: &str = include_str!("../../../../CHANGELOG.md");

/// One version: `## <version> — <date>` and its `- ` lines.
pub struct Release<'a> {
    pub version: &'a str,
    pub date: &'a str,
    pub items: Vec<&'a str>,
}

/// The releases in `text`, newest first as written.
pub fn parse(text: &str) -> Vec<Release<'_>> {
    let mut out: Vec<Release> = Vec::new();
    for line in text.lines() {
        if let Some(head) = line.strip_prefix("## ") {
            let (version, date) = head.split_once('—').unwrap_or((head, ""));
            out.push(Release { version: version.trim(), date: date.trim(), items: Vec::new() });
        } else if let (Some(item), Some(r)) = (line.strip_prefix("- "), out.last_mut()) {
            r.items.push(item.trim());
        }
    }
    out
}

/// Help → What's New: the list of versions, and the lines of the one picked.
#[derive(Default)]
pub struct ReleaseNotesWindow {
    pub open: bool,
    /// The version shown (index in the changelog's sections).
    pub sel: usize,
}

impl ReleaseNotesWindow {
    /// Open on this version (the newest when it has no section).
    pub fn show(&mut self) {
        self.open = true;
        self.sel = parse(CHANGELOG).iter().position(|r| r.version == env!("CARGO_PKG_VERSION")).unwrap_or(0);
    }

    /// For the debug harness: open and the version shown.
    pub fn dump(&self) -> (bool, String) {
        (self.open, parse(CHANGELOG).get(self.sel).map(|r| r.version.to_string()).unwrap_or_default())
    }
}

impl VarstotasApp {
    pub(crate) fn release_notes_window(&mut self, ctx: &Context) {
        if !self.release_notes.open {
            return;
        }
        let releases = parse(CHANGELOG);
        let mut open = true;
        let win = self.placed(egui::Window::new(tr!("release-notes-title")).id(egui::Id::new("release_notes")).open(&mut open).default_size([600.0, 380.0]).resizable(true), [160.0, 140.0]);
        let sel = &mut self.release_notes.sel;
        win.show(ctx, |ui| {
            let h = ui.available_height();
            ui.horizontal_top(|ui| {
                egui::ScrollArea::vertical().id_salt("release_notes_list").max_height(h).max_width(140.0).show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_width(130.0);
                        for (i, r) in releases.iter().enumerate() {
                            let resp = ui.selectable_label(*sel == i, RichText::new(r.version).strong());
                            super::tray_lib::spot(format!("release_notes:{}", r.version), resp.rect.center());
                            if resp.clicked() {
                                *sel = i;
                            }
                            ui.label(RichText::new(r.date).small().color(theme::text_dim()));
                            ui.add_space(4.0);
                        }
                    });
                });
                ui.separator();
                let Some(r) = releases.get(*sel) else { return };
                egui::ScrollArea::vertical().id_salt("release_notes_items").max_height(h).auto_shrink([false, false]).show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(r.version).size(18.0).strong());
                            let current = r.version == env!("CARGO_PKG_VERSION");
                            let date = if current { tr!("release-notes-current", date = r.date) } else { r.date.to_string() };
                            ui.label(RichText::new(date).color(theme::text_dim()));
                        });
                        ui.add_space(4.0);
                        for item in &r.items {
                            ui.horizontal_top(|ui| {
                                ui.label("•");
                                ui.add(egui::Label::new(*item).wrap());
                            });
                        }
                    });
                });
            });
        });
        self.release_notes.open &= open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_version_has_notes() {
        let releases = parse(CHANGELOG);
        let current = releases.iter().find(|r| r.version == env!("CARGO_PKG_VERSION")).expect("CHANGELOG.md has no section for this version");
        assert!(!current.items.is_empty() && !current.date.is_empty());
    }

    #[test]
    fn parses_sections() {
        let r = parse("# Title\nintro\n- not an item\n## 1.1 — 2030-01-02\n- a\n- b\n\n## 1.0\n- c\n");
        assert_eq!(r.len(), 2);
        assert_eq!((r[0].version, r[0].date, r[0].items.clone()), ("1.1", "2030-01-02", vec!["a", "b"]));
        assert_eq!((r[1].version, r[1].date, r[1].items.clone()), ("1.0", "", vec!["c"]));
    }
}
