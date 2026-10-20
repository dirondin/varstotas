//! Help → Licences…: Varstotas's own licence, the open-source components it is built from
//! (`licences/third_party.json`, made by `scripts/third_party_licences.py`), the file formats
//! and the online libraries with their terms. Also the terms notice shown above an online
//! source's assets (3D Library, Online Materials).

use crate::app::VarstotasApp;
use egui::{Color32, Context, RichText, Ui};
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

    pub fn label(self) -> &'static str {
        match self {
            Tab::Varstotas => "Varstotas",
            Tab::Components => "Open Source",
            Tab::Formats => "File Formats",
            Tab::Libraries => "Online Libraries",
        }
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

/// File formats: (format, what Varstotas does, how, notes on the format).
pub const FORMATS: &[(&str, &str, &str, &str)] = &[
    (".varstotas", "open, save", "Varstotas's own format", "MIT, like Varstotas"),
    ("glTF / GLB", "import, export", "own reader / writer; Draco meshes via draco-core (Apache-2.0)", "Khronos open standard, royalty-free"),
    ("OBJ + MTL", "import, export", "own reader / writer", "open, public specification"),
    ("STL", "import, export", "own reader / writer", "open, public specification"),
    ("PLY", "import, export", "own reader / writer", "open, public specification"),
    ("3MF", "import, export", "own reader / writer (zip + XML)", "3MF Consortium specification, royalty-free"),
    ("COLLADA (.dae)", "import, export", "own reader / writer (images as files next to it)", "Khronos open standard, royalty-free"),
    ("FBX", "import", "ufbx (MIT / Unlicense), an independent reader", "Autodesk's format; FBX is an Autodesk trademark, Varstotas is not affiliated"),
    ("DXF", "import, export", "own reader / writer", "Autodesk's openly published format; DXF is an Autodesk trademark"),
    ("SVG, PDF", "export (drawings, unfolds, sheets)", "own writers", "W3C / ISO open standards"),
    ("PNG, JPEG, HDR, TGA, BMP", "textures, images, environments", "image crate (MIT / Apache-2.0)", "open formats"),
    ("OpenEXR", "render export", "own writer", "Academy Software Foundation, BSD-3-Clause"),
    ("CSV, JSON", "points import, faces export (plugins)", "own code", "open formats"),
    ("MP4 / video", "animation export", "ffmpeg, installed by you and run as a separate program (not shipped)", "ffmpeg is LGPL / GPL; its licence applies to your copy"),
];

/// Online libraries: (name, what, licence of the assets, terms of the service).
pub const LIBRARIES: &[(&str, &str, &str, &str)] = &[
    ("Poly Haven", "models, materials", "CC0", "Free public API, also for commercial apps; asks to show «Powered by Poly Haven» (done above its assets) and a unique User-Agent (Varstotas)."),
    ("ambientCG", "materials", "CC0", "Free API and downloads; credit appreciated, not required."),
    ("Smithsonian Open Access 3D", "models", "mostly CC0; some «Usage conditions apply» (non-commercial)", "The API does not say which: a model's page has to be checked. Varstotas marks these models as «check the usage» and warns when they are placed or exported."),
    ("NASA 3D Resources", "models", "not copyrighted (US public domain)", "NASA insignia and logos are protected; no use may suggest NASA endorsement."),
    ("Kenney", "model packs", "CC0", "Read from the public asset pages of kenney.nl; Kenney's logo is not CC0."),
    ("Quaternius", "model packs", "CC0", "Downloaded by you in the browser."),
    ("Search on sites (Printables, Thingiverse, Sketchfab, …)", "models", "each model its own", "Varstotas only opens the site's search in your browser; you download yourself and set the licence in the import card."),
    ("Plugins", "models, materials", "per plugin", "A source plugin shows its terms (its @notice line) above its assets; e.g. Sketchfab needs your own account token and a credit to the author and Sketchfab."),
];

/// External programs used when installed: (name, use, licence).
pub const PROGRAMS: &[(&str, &str, &str)] = &[
    ("ffmpeg", "animation video export", "LGPL-2.1+ / GPL — not shipped, run as a separate program"),
    ("Open Image Denoise", "AI denoiser of the renderer", "Apache-2.0 — not shipped, loaded from your installation"),
    ("Python", "Python plugins and scripts", "PSF licence — not shipped, your installation"),
];

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
        let mut win = egui::Window::new("Licences").id(egui::Id::new("licences")).open(&mut open).default_size([680.0, 520.0]).resizable(true);
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
            ui.label(format!("{} components", tp.crates.len()));
            ui.add(egui::TextEdit::singleline(&mut self.licences.filter).hint_text("Filter: name or licence").desired_width(200.0).id_salt("licences_filter"));
            if ui.button("Save as Text…").on_hover_text("All components and their licence texts in one file").clicked()
                && let Some(p) = rfd::FileDialog::new().set_file_name("THIRD-PARTY-LICENSES.txt").save_file()
            {
                match std::fs::write(&p, all_text()) {
                    Ok(()) => self.flash(format!("Saved {}", p.display())),
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
                    ui.label(RichText::new("Pick a component to see its licence.").weak());
                }
            }));
        });
    }
}

fn varstotas_tab(ui: &mut Ui) {
    egui::ScrollArea::vertical().id_salt("licences_varstotas").show(ui, |ui| {
        ui.label("Varstotas is free software under the MIT licence: you may use it for personal and commercial work, copy, change and share it.");
        ui.add_space(6.0);
        ui.label(RichText::new(LICENSE).monospace().small());
        ui.add_space(8.0);
        ui.label(RichText::new("Programs used when you have them").strong());
        egui::Grid::new("licences_programs").striped(true).num_columns(3).show(ui, |ui| {
            for (n, what, lic) in PROGRAMS {
                ui.label(*n);
                ui.label(*what);
                ui.label(RichText::new(*lic).small());
                ui.end_row();
            }
        });
        ui.add_space(8.0);
        ui.label(RichText::new("Your models are yours. Models and materials downloaded from online libraries keep their own licences (Online Libraries; File → Export → Attributions).").small().weak());
    });
}

fn formats_tab(ui: &mut Ui) {
    egui::ScrollArea::vertical().id_salt("licences_formats").show(ui, |ui| {
        egui::Grid::new("licences_formats_grid").striped(true).num_columns(4).spacing([10.0, 4.0]).show(ui, |ui| {
            for h in ["Format", "Varstotas", "How", "The format"] {
                ui.label(RichText::new(h).strong());
            }
            ui.end_row();
            for (f, what, how, note) in FORMATS {
                ui.label(*f);
                ui.label(RichText::new(*what).small());
                ui.label(RichText::new(*how).small());
                ui.label(RichText::new(*note).small());
                ui.end_row();
            }
        });
    });
}

fn libraries_tab(ui: &mut Ui) {
    egui::ScrollArea::vertical().id_salt("licences_libraries").show(ui, |ui| {
        ui.label(RichText::new("Every downloaded model keeps its source, author and licence; Varstotas warns when a placed or exported model's licence limits its use (non-commercial, no derivatives, share-alike, a site's own licence, unknown).").small());
        ui.add_space(6.0);
        egui::Grid::new("licences_libraries_grid").striped(true).num_columns(4).spacing([10.0, 4.0]).show(ui, |ui| {
            for h in ["Library", "Assets", "Licence", "Terms"] {
                ui.label(RichText::new(h).strong());
            }
            ui.end_row();
            for (n, what, lic, terms) in LIBRARIES {
                ui.label(*n);
                ui.label(RichText::new(*what).small());
                ui.label(RichText::new(*lic).small());
                ui.add(egui::Label::new(RichText::new(*terms).small()).wrap());
                ui.end_row();
            }
        });
    });
}

/// Every component and its licence texts (Save as Text…).
pub fn all_text() -> String {
    let tp = third_party();
    let mut s = String::from("Varstotas uses the following open-source components.\n\n");
    for c in &tp.crates {
        s.push_str(&format!("{} {} — {} {}\n", c.name, c.version, c.license, c.repository));
    }
    for (i, t) in tp.texts.iter().enumerate() {
        let users: Vec<String> = tp.crates.iter().filter(|c| c.texts.iter().any(|(_, j)| *j == i)).map(|c| format!("{} {}", c.name, c.version)).collect();
        s.push_str(&format!("\n{}\nApplies to: {}\n\n{t}\n", "=".repeat(78), users.join(", ")));
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
