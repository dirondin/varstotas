//! Help → Licences…: Varstotas's own licence, the open-source components it is built from
//! (`licences/third_party.json`, made by `scripts/third_party_licences.py`), the file formats
//! and the online libraries with their terms. Also the terms notice shown above an online
//! source's assets (3D Library, Online Materials).

use crate::app::VarstotasApp;
use egui::{Color32, Context, RichText, Ui};
use varstotas_i18n::tr;
use serde::Deserialize;
use std::sync::OnceLock;

const LICENSE: &str = include_str!("../../../../LICENSE");
const THIRD_PARTY: &str = include_str!("../../licences/third_party.json");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Varstotas,
    Components,
    Formats,
    Libraries,
}

impl Tab {
    pub const ALL: [Tab; 4] = [Tab::Varstotas, Tab::Components, Tab::Formats, Tab::Libraries];

    pub fn label(self) -> String {
        varstotas_i18n::tr(match self {
            Tab::Varstotas => "licences-tab-varstotas",
            Tab::Components => "licences-tab-components",
            Tab::Formats => "licences-tab-formats",
            Tab::Libraries => "licences-tab-libraries",
        })
    }

    pub fn key(self) -> &'static str {
        match self {
            Tab::Varstotas => "varstotas",
            Tab::Components => "components",
            Tab::Formats => "formats",
            Tab::Libraries => "libraries",
        }
    }
}

#[derive(Default)]
pub struct LicencesWindow {
    pub open: bool,
    pub tab: Tab,
    pub filter: String,
    /// The component whose licence texts are shown (index in [`crates`]).
    pub sel: Option<usize>,
}

#[derive(Deserialize)]
pub struct Crate {
    pub name: String,
    pub version: String,
    pub license: String,
    pub repository: String,
    /// (file name, index in `texts`).
    pub texts: Vec<(String, usize)>,
}

#[derive(Deserialize)]
struct ThirdParty {
    crates: Vec<Crate>,
    texts: Vec<String>,
}

fn third_party() -> &'static ThirdParty {
    static TP: OnceLock<ThirdParty> = OnceLock::new();
    TP.get_or_init(|| serde_json::from_str(THIRD_PARTY).unwrap_or(ThirdParty { crates: Vec::new(), texts: Vec::new() }))
}

/// The open-source components (crates) in the app.
pub fn crates() -> &'static [Crate] {
    &third_party().crates
}

/// File formats: (format name, id of its texts). A format name is not translated.
const FORMATS: &[(&str, &str)] = &[
    (".varstotas", "varstotas"), // i18n-ignore
    ("glTF / GLB", "gltf"), // i18n-ignore
    ("OBJ + MTL", "obj"), // i18n-ignore
    ("STL", "stl"), // i18n-ignore
    ("PLY", "ply"), // i18n-ignore
    ("3MF", "3mf"), // i18n-ignore
    ("COLLADA (.dae)", "collada"), // i18n-ignore
    ("FBX", "fbx"), // i18n-ignore
    ("DXF", "dxf"), // i18n-ignore
    ("SVG, PDF", "svg-pdf"), // i18n-ignore
    ("PNG, JPEG, HDR, TGA, BMP", "images"), // i18n-ignore
    ("OpenEXR", "openexr"), // i18n-ignore
    ("CSV, JSON", "csv-json"), // i18n-ignore
    ("MP4 / video", "video"), // i18n-ignore
];

/// Online libraries: (name, id of its texts). A name of "" is a text (`licence-lib-<id>-name`).
const LIBRARIES: &[(&str, &str)] = &[
    ("Poly Haven", "poly-haven"), // i18n-ignore
    ("ambientCG", "ambientcg"), // i18n-ignore
    ("Smithsonian Open Access 3D", "smithsonian"), // i18n-ignore
    ("NASA 3D Resources", "nasa"), // i18n-ignore
    ("Kenney", "kenney"), // i18n-ignore
    ("Quaternius", "quaternius"), // i18n-ignore
    ("", "search"), // i18n-ignore
    ("", "plugins"), // i18n-ignore
];

/// External programs used when installed: (name, id of its texts).
const PROGRAMS: &[(&str, &str)] = &[
    ("ffmpeg", "ffmpeg"), // i18n-ignore
    ("Open Image Denoise", "oidn"), // i18n-ignore
    ("Python", "python"), // i18n-ignore
];

/// A text of a table row: `licence-<what>-<id>-<field>`.
fn cell(what: &str, id: &str, field: &str) -> String {
    varstotas_i18n::tr(&format!("licence-{what}-{id}-{field}"))
}

/// The terms of an online source above its assets (nothing when `text` is empty).
pub fn terms_notice(ui: &mut Ui, text: &str) {
    if text.trim().is_empty() {
        return;
    }
    let c = if ui.visuals().dark_mode { Color32::from_rgb(230, 180, 80) } else { Color32::from_rgb(150, 95, 0) };
    egui::Frame::new().stroke(egui::Stroke::new(1.0, c)).corner_radius(4.0).inner_margin(egui::Margin::symmetric(6, 3)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(format!("⚠ {text}")).small().color(c));
    });
}

impl VarstotasApp {
    pub(crate) fn open_licences(&mut self, tab: Tab) {
        self.licences.open = true;
        self.licences.tab = tab;
    }

    pub(crate) fn licences_window(&mut self, ctx: &Context) {
        if !self.licences.open {
            return;
        }
        let mut open = true;
        let mut win = egui::Window::new(tr!("licences-title")).id(egui::Id::new("licences")).open(&mut open).default_size([680.0, 520.0]).resizable(true);
        if self.debug.script_active() {
            win = win.fixed_pos([140.0, 160.0]);
        }
        win.show(ctx, |ui| {
            ui.horizontal(|ui| {
                for t in Tab::ALL {
                    ui.selectable_value(&mut self.licences.tab, t, t.label());
                }
            });
            ui.separator();
            match self.licences.tab {
                Tab::Varstotas => varstotas_tab(ui),
                Tab::Components => self.components_tab(ui),
                Tab::Formats => formats_tab(ui),
                Tab::Libraries => libraries_tab(ui),
            }
        });
        self.licences.open &= open;
    }

    fn components_tab(&mut self, ui: &mut Ui) {
        let tp = third_party();
        ui.horizontal(|ui| {
            ui.label(tr!("licences-count", n = tp.crates.len()));
            ui.add(egui::TextEdit::singleline(&mut self.licences.filter).hint_text(tr!("licences-filter")).desired_width(200.0).id_salt("licences_filter"));
            if ui.button(tr!("licences-save")).on_hover_text(tr!("licences-save-hint")).clicked()
                && let Some(p) = rfd::FileDialog::new().set_file_name("THIRD-PARTY-LICENSES.txt") // i18n-ignore
.save_file()
            {
                match std::fs::write(&p, all_text()) {
                    Ok(()) => self.flash(tr!("licences-saved", path = p.display().to_string())),
                    Err(e) => self.flash_error(format!("{}: {e}", p.display())),
                }
            }
        });
        let f = self.licences.filter.to_lowercase();
        let h = ui.available_height();
        ui.horizontal_top(|ui| {
            egui::ScrollArea::vertical().id_salt("licences_list").max_height(h).max_width(250.0).show(ui, |ui| {
                ui.vertical(|ui| {
                ui.set_width(240.0);
                for (i, c) in tp.crates.iter().enumerate() {
                    if !f.is_empty() && !c.name.to_lowercase().contains(&f) && !c.license.to_lowercase().contains(&f) {
                        continue;
                    }
                    let r = ui.selectable_label(self.licences.sel == Some(i), format!("{} {}", c.name, c.version)).on_hover_text(&c.license);
                    if r.clicked() {
                        self.licences.sel = Some(i);
                    }
                }
                });
            });
            ui.separator();
            egui::ScrollArea::vertical().id_salt("licences_text").max_height(h).show(ui, |ui| ui.vertical(|ui| match self.licences.sel.and_then(|i| tp.crates.get(i)) {
                Some(c) => {
                    ui.heading(format!("{} {}", c.name, c.version));
                    ui.label(RichText::new(&c.license).strong());
                    if !c.repository.is_empty() {
                        ui.hyperlink(&c.repository);
                    }
                    for (file, t) in &c.texts {
                        ui.add_space(6.0);
                        ui.label(RichText::new(file).small().weak());
                        ui.label(RichText::new(tp.texts.get(*t).map(String::as_str).unwrap_or("")).monospace().small());
                    }
                }
                None => {
                    ui.label(RichText::new(tr!("licences-pick")).weak());
                }
            }));
        });
    }
}

fn varstotas_tab(ui: &mut Ui) {
    egui::ScrollArea::vertical().id_salt("licences_varstotas").show(ui, |ui| {
        ui.label(tr!("licences-own"));
        ui.add_space(6.0);
        ui.label(RichText::new(LICENSE).monospace().small());
        ui.add_space(8.0);
        ui.label(RichText::new(tr!("licences-programs")).strong());
        egui::Grid::new("licences_programs").striped(true).num_columns(3).show(ui, |ui| {
            for (n, id) in PROGRAMS {
                ui.label(*n);
                ui.label(cell("program", id, "use"));
                ui.label(RichText::new(cell("program", id, "licence")).small());
                ui.end_row();
            }
        });
        ui.add_space(8.0);
        ui.label(RichText::new(tr!("licences-yours")).small().weak());
    });
}

fn formats_tab(ui: &mut Ui) {
    egui::ScrollArea::vertical().id_salt("licences_formats").show(ui, |ui| {
        egui::Grid::new("licences_formats_grid").striped(true).num_columns(4).spacing([10.0, 4.0]).show(ui, |ui| {
            for h in [tr!("licence-formats-col-format"), tr!("licence-formats-col-varstotas"), tr!("licence-formats-col-how"), tr!("licence-formats-col-note")] {
                ui.label(RichText::new(h).strong());
            }
            ui.end_row();
            for (f, id) in FORMATS {
                ui.label(*f);
                ui.label(RichText::new(cell("format", id, "what")).small());
                ui.label(RichText::new(cell("format", id, "how")).small());
                ui.label(RichText::new(cell("format", id, "note")).small());
                ui.end_row();
            }
        });
    });
}

fn libraries_tab(ui: &mut Ui) {
    egui::ScrollArea::vertical().id_salt("licences_libraries").show(ui, |ui| {
        ui.label(RichText::new(tr!("licence-libs-intro")).small());
        ui.add_space(6.0);
        egui::Grid::new("licences_libraries_grid").striped(true).num_columns(4).spacing([10.0, 4.0]).show(ui, |ui| {
            for h in [tr!("licence-libs-col-library"), tr!("licence-libs-col-assets"), tr!("licence-libs-col-licence"), tr!("licence-libs-col-terms")] {
                ui.label(RichText::new(h).strong());
            }
            ui.end_row();
            for (n, id) in LIBRARIES {
                ui.label(if n.is_empty() { cell("lib", id, "name") } else { n.to_string() });
                ui.label(RichText::new(cell("lib", id, "what")).small());
                ui.label(RichText::new(cell("lib", id, "licence")).small());
                ui.add(egui::Label::new(RichText::new(cell("lib", id, "terms")).small()).wrap());
                ui.end_row();
            }
        });
    });
}

/// Every component and its licence texts (Save as Text…).
pub fn all_text() -> String {
    let tp = third_party();
    let mut s = tr!("licences-all-text-title") + "\n\n";
    for c in &tp.crates {
        s.push_str(&format!("{} {} — {} {}\n", c.name, c.version, c.license, c.repository));
    }
    for (i, t) in tp.texts.iter().enumerate() {
        let users: Vec<String> = tp.crates.iter().filter(|c| c.texts.iter().any(|(_, j)| *j == i)).map(|c| format!("{} {}", c.name, c.version)).collect();
        s.push_str(&format!("\n{}\n{}\n\n{t}\n", "=".repeat(78), tr!("licences-applies-to", users = users.join(", "))));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_component_list_is_embedded() {
        let c = crates();
        assert!(c.len() > 100, "{}", c.len());
        assert!(c.iter().any(|c| c.name == "egui"));
        assert!(c.iter().all(|c| !c.texts.is_empty() && !c.license.is_empty()), "every component has a licence text");
        assert!(c.iter().all(|c| !c.name.starts_with("varstotas")));
        assert!(LICENSE.starts_with("MIT License"));
        assert!(all_text().contains("Apache License"));
    }
}
