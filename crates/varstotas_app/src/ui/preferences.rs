//! Window → Preferences…: one window with the sections on the left (General, Drawing,
//! Libraries, Downloads, View) and a search. The menus keep their own switches (Snapping,
//! View, the cards' … menus, the Downloads window); the window shows the same values.

use std::path::{Path, PathBuf};

use egui::{Context, RichText, Ui};
use serde::{Deserialize, Serialize};

use super::theme;
use crate::app::{FaceStylePref, VarstotasApp};
use crate::templates::Template;

/// The sections, in the order of the list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Section {
    #[default]
    General,
    Drawing,
    Libraries,
    Downloads,
    View,
    Navigation,
}

impl Section {
    pub const ALL: [Section; 6] = [Section::General, Section::Drawing, Section::Libraries, Section::Downloads, Section::View, Section::Navigation];

    pub fn name(self) -> &'static str {
        match self {
            Section::General => "General",
            Section::Drawing => "Drawing",
            Section::Libraries => "Libraries",
            Section::Downloads => "Downloads",
            Section::View => "View",
            Section::Navigation => "Navigation",
        }
    }

    /// What the search finds a section by (besides its name): the labels of its settings.
    fn words(self) -> &'static str {
        match self {
            Section::General => "auto-save autosave minutes template new document units",
            Section::Drawing => "snapping inference endpoint midpoint edge face axis grid step major increments move rotate scale angle",
            Section::Libraries => "components materials folder examples in model tiles small large list",
            Section::Downloads => "watch downloads folder send to varstotas bookmarklet browser",
            Section::Navigation => "mouse trackpad orbit zoom pan speed invert wheel cursor middle button two fingers gesture",
            Section::View => "new document style shaded textures monochrome hidden line wireframe realistic edges profiles axes ground sky grid shadows perspective parallel projection fov field of view camera",
        }
    }

    pub fn matches(self, search: &str) -> bool {
        let s = search.trim().to_lowercase();
        s.is_empty() || self.name().to_lowercase().contains(&s) || self.words().contains(&s)
    }
}

/// The window's own state (not saved).
#[derive(Default)]
pub struct PrefsWindow {
    pub open: bool,
    pub section: Section,
    pub search: String,
}

/// The view of a new document (File → New, New from Template) — Preferences → View.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ViewDefaults {
    pub face_style: FaceStylePref,
    pub edges: bool,
    pub profiles: bool,
    pub axes: bool,
    pub ground: bool,
    pub sky: bool,
    pub grid: bool,
    pub shadows: bool,
    /// Perspective (else parallel projection).
    pub perspective: bool,
    /// Vertical field of view, degrees.
    pub fov: f64,
}

impl Default for ViewDefaults {
    fn default() -> Self {
        Self {
            face_style: FaceStylePref::Shaded,
            edges: true,
            profiles: true,
            axes: true,
            ground: true,
            sky: true,
            grid: false,
            shadows: false,
            perspective: true,
            fov: 35.0,
        }
    }
}

/// A folder row: the path (or "default: …"), Choose…, Default, Show in Finder.
fn folder_row(ui: &mut Ui, label: &str, custom: &mut Option<PathBuf>, default: Option<PathBuf>, tip: &str) {
    ui.label(label).on_hover_text(tip);
    let shown = custom.clone().or(default.clone());
    ui.horizontal(|ui| {
        let text = match (&custom, &shown) {
            (Some(p), _) => p.display().to_string(),
            (None, Some(p)) => format!("{} (default)", p.display()),
            (None, None) => "—".to_string(),
        };
        ui.add(egui::Label::new(RichText::new(text).small().color(theme::TEXT_DIM)).truncate());
    });
    ui.horizontal(|ui| {
        if ui.button("Choose…").clicked()
            && let Some(d) = rfd::FileDialog::new().pick_folder()
        {
            *custom = Some(d);
        }
        if ui.add_enabled(custom.is_some(), egui::Button::new("Default")).clicked() {
            *custom = None;
        }
        if ui.add_enabled(shown.as_deref().is_some_and(Path::is_dir), egui::Button::new("Show in Finder")).clicked()
            && let Some(d) = &shown
        {
            crate::ui::extensions::reveal(d);
        }
    });
}

fn note(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).small().color(theme::TEXT_DIM));
}

fn heading(ui: &mut Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(text).strong());
}

impl VarstotasApp {
    pub(crate) fn preferences_window(&mut self, ctx: &Context) {
        if !self.prefs_win.open {
            return;
        }
        let mut open = true;
        let pos = self.viewport.view_rect.min + egui::vec2(60.0, 40.0);
        let mut win = egui::Window::new("Preferences")
            .id(egui::Id::new("preferences_window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_pos(pos)
            .default_size([620.0, 460.0]);
        if self.debug.script_active() {
            win = win.fixed_pos(pos);
        }
        win.show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                // the list of sections and the search
                ui.vertical(|ui| {
                    ui.set_width(140.0);
                    let r = ui.add(egui::TextEdit::singleline(&mut self.prefs_win.search).hint_text("Search").desired_width(140.0));
                    super::tray_lib::spot("prefs_search", r.rect.center());
                    ui.add_space(4.0);
                    let search = self.prefs_win.search.clone();
                    let shown: Vec<Section> = Section::ALL.into_iter().filter(|s| s.matches(&search)).collect();
                    if !shown.is_empty() && !shown.contains(&self.prefs_win.section) {
                        self.prefs_win.section = shown[0];
                    }
                    for s in &shown {
                        let r = ui.add_sized([140.0, 22.0], egui::Button::selectable(self.prefs_win.section == *s, s.name()));
                        super::tray_lib::spot(format!("prefs:{}", s.name()), r.rect.center());
                        if r.clicked() {
                            self.prefs_win.section = *s;
                        }
                    }
                    if shown.is_empty() {
                        note(ui, "Nothing found");
                    }
                });
                ui.separator();
                ui.vertical(|ui| {
                    egui::ScrollArea::vertical().id_salt("prefs_body").auto_shrink([false, false]).show(ui, |ui| {
                        let section = self.prefs_win.section;
                        ui.horizontal(|ui| {
                            ui.heading(section.name());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let r = ui.button("Reset").on_hover_text("This section's settings back to the defaults");
                                super::tray_lib::spot("prefs_reset", r.rect.center());
                                if r.clicked() {
                                    self.reset_prefs_section(section);
                                }
                            });
                        });
                        match section {
                            Section::General => self.prefs_general(ui),
                            Section::Drawing => self.prefs_drawing(ui),
                            Section::Libraries => self.prefs_libraries(ui),
                            Section::Downloads => self.prefs_downloads(ui),
                            Section::View => self.prefs_view(ui),
                            Section::Navigation => self.prefs_navigation(ui),
                        }
                    });
                });
            });
        });
        if !open {
            self.prefs_win.open = false;
        }
    }

    fn reset_prefs_section(&mut self, s: Section) {
        let d = crate::app::Prefs::default();
        let p = &mut self.prefs;
        match s {
            Section::General => {
                p.autosave = d.autosave;
                p.autosave_minutes = d.autosave_minutes;
                p.template = d.template;
            }
            Section::Drawing => {
                p.snapping = d.snapping;
                p.grid.step = d.grid.step;
                p.grid.major = d.grid.major;
            }
            Section::Libraries => {
                p.library_dir = None;
                p.mat_library_dir = None;
                p.hide_examples = d.hide_examples;
                p.hide_document = d.hide_document;
                p.hide_mat_examples = d.hide_mat_examples;
                p.lib_tiles = d.lib_tiles;
            }
            Section::Downloads => {
                p.watch_downloads = d.watch_downloads;
                p.downloads_watch_dir = None;
                p.send_to_varstotas = d.send_to_varstotas;
            }
            Section::View => p.new_view = ViewDefaults::default(),
            Section::Navigation => p.nav = Default::default(),
        }
        self.flash(format!("Preferences → {}: defaults", s.name()));
    }

    fn prefs_general(&mut self, ui: &mut Ui) {
        heading(ui, "Auto-save");
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.prefs.autosave, "Auto-save every");
            ui.add_enabled(self.prefs.autosave, egui::DragValue::new(&mut self.prefs.autosave_minutes).range(1..=120).speed(0.2));
            ui.label("minutes");
        });
        note(ui, "Unsaved changes are written to <name>.vrst~autosave next to the file.");
        heading(ui, "New document");
        let cur = self.prefs.template.unwrap_or(Template::ArchitecturalMm);
        let mut t = cur;
        egui::ComboBox::from_id_salt("default_template").selected_text(t.name()).width(260.0).show_ui(ui, |ui| {
            for x in Template::ALL {
                ui.selectable_value(&mut t, x, x.name());
            }
        });
        if t != cur || self.prefs.template.is_none() {
            self.prefs.template = Some(t);
        }
        note(ui, "The template of File → New (units and scale); its view is set in the View section.");
    }

    fn prefs_drawing(&mut self, ui: &mut Ui) {
        heading(ui, "Grid");
        let unit = self.doc.model.unit;
        ui.horizontal(|ui| {
            let mut grid = self.prefs.grid.show;
            if ui.checkbox(&mut grid, "Show grid").clicked() {
                self.actions.push(crate::app::Action::ToggleGrid);
            }
            ui.label("step");
            ui.add(
                egui::DragValue::new(&mut self.prefs.grid.step)
                    .range(0.01..=1.0e6)
                    .speed(1.0)
                    .custom_formatter(move |v, _| varstotas_kernel::units::format_length(v, unit))
                    .custom_parser(move |t| varstotas_kernel::units::parse_length(t, unit)),
            );
            ui.label("major every");
            ui.add(egui::DragValue::new(&mut self.prefs.grid.major).range(2..=20).speed(0.1));
        });
        note(ui, "Zooming out multiplies the step by the major count. With Select active, typing a length sets the step.");
        heading(ui, "Snapping and increments");
        note(ui, "The same switches as the Snapping menu.");
        self.snapping_menu(ui);
    }

    fn prefs_libraries(&mut self, ui: &mut Ui) {
        let home = crate::bridge::home_dir();
        heading(ui, "Components");
        folder_row(ui, "Library folder", &mut self.prefs.library_dir, home.as_ref().map(|h| h.join("components")), "Components (.vrst) and generators (.rhai) of the Components card");
        let mut show_ex = !self.prefs.hide_examples;
        ui.checkbox(&mut show_ex, "Show Examples").on_hover_text("The bundled example generators");
        self.prefs.hide_examples = !show_ex;
        let mut show_doc = !self.prefs.hide_document;
        ui.checkbox(&mut show_doc, "Show In Model").on_hover_text("The open document's components and generators");
        self.prefs.hide_document = !show_doc;
        heading(ui, "Materials");
        folder_row(ui, "Library folder", &mut self.prefs.mat_library_dir, home.as_ref().map(|h| h.join("materials")), "Materials (.vrstmat) of the Materials card; subfolders are categories");
        let mut show_ex = !self.prefs.hide_mat_examples;
        ui.checkbox(&mut show_ex, "Show Examples").on_hover_text("The materials bundled with Varstotas (the Material Gallery demo)");
        self.prefs.hide_mat_examples = !show_ex;
        heading(ui, "Tiles");
        ui.horizontal(|ui| {
            for (s, label) in super::tray_lib::TILE_SIZES {
                let r = ui.radio_value(&mut self.prefs.lib_tiles, s, label);
                super::tray_lib::spot(format!("tiles:{label}"), r.rect.center());
            }
        });
        note(ui, "Both cards; also in their … menus.");
    }

    fn prefs_downloads(&mut self, ui: &mut Ui) {
        heading(ui, "Watch Downloads");
        ui.checkbox(&mut self.prefs.watch_downloads, "Offer new model files for import")
            .on_hover_text("Every new model file or archive in the folder is offered for import (always on while a page of Other Sites waits)");
        folder_row(ui, "Watched folder", &mut self.prefs.downloads_watch_dir, crate::downloads::downloads_folder(), "Where your browser saves files");
        heading(ui, "Send to Varstotas");
        ui.checkbox(&mut self.prefs.send_to_varstotas, "Listen for the bookmarklet")
            .on_hover_text("A bookmarklet: on a model's page, click it to send the page's title, author and licence here");
        if let Some(s) = &self.models.dl.send {
            let port = s.port;
            ui.horizontal(|ui| {
                note(ui, &format!("listening on 127.0.0.1:{port}"));
                if ui.button("Copy Bookmarklet").on_hover_text("Make a bookmark in the browser and paste this as its address").clicked() {
                    ui.ctx().copy_text(crate::downloads::bookmarklet(port, &crate::downloads::send_token()));
                    self.flash("Bookmarklet copied: make a bookmark and paste it as the address");
                }
            });
        }
        if let Some(e) = &self.models.dl.send_error {
            ui.colored_label(egui::Color32::from_rgb(200, 60, 40), RichText::new(e).small());
        }
        note(ui, "Also in Window → 3D Library… → Other Sites.");
    }

    fn prefs_navigation(&mut self, ui: &mut Ui) {
        use crate::nav::NavGesture;
        let n = &mut self.prefs.nav;
        heading(ui, "Speed");
        egui::Grid::new("nav_speed").num_columns(2).spacing([12.0, 4.0]).show(ui, |ui| {
            ui.label("Orbit");
            let r = ui.add(egui::Slider::new(&mut n.orbit_speed, 0.25..=4.0).logarithmic(true).suffix("×"));
            super::tray_lib::spot("nav:orbit", r.rect.center());
            ui.end_row();
            ui.label("Zoom");
            let r = ui.add(egui::Slider::new(&mut n.zoom_speed, 0.25..=4.0).logarithmic(true).suffix("×"));
            super::tray_lib::spot("nav:zoom", r.rect.center());
            ui.end_row();
        });
        heading(ui, "Zoom");
        let r = ui.checkbox(&mut n.invert_zoom, "Invert the wheel").on_hover_text("Wheel away from you zooms out");
        super::tray_lib::spot("nav:invert", r.rect.center());
        let r = ui.checkbox(&mut n.zoom_to_cursor, "Zoom toward the cursor").on_hover_text("Off: toward the middle of the view");
        super::tray_lib::spot("nav:cursor", r.rect.center());
        heading(ui, "Mouse");
        ui.horizontal(|ui| {
            ui.label("Middle button drag:");
            ui.radio_value(&mut n.middle, NavGesture::Orbit, "Orbit");
            let r = ui.radio_value(&mut n.middle, NavGesture::Pan, "Pan");
            super::tray_lib::spot("nav:middle_pan", r.rect.center());
        });
        note(ui, "Shift gives the other one.");
        heading(ui, "Trackpad");
        ui.horizontal(|ui| {
            ui.label("Two fingers:");
            ui.radio_value(&mut n.two_finger, NavGesture::Orbit, "Orbit");
            let r = ui.radio_value(&mut n.two_finger, NavGesture::Pan, "Pan");
            super::tray_lib::spot("nav:two_pan", r.rect.center());
            let r = ui.radio_value(&mut n.two_finger, NavGesture::Zoom, "Zoom");
            super::tray_lib::spot("nav:two_zoom", r.rect.center());
        });
        note(ui, "Shift: Pan (Orbit when two fingers pan). Pinch and ⌘ + two fingers always zoom.");
    }

    fn prefs_view(&mut self, ui: &mut Ui) {
        note(ui, "How a new document looks (File → New, New from Template). The View menu and the viewport's buttons change the current view.");
        let v = &mut self.prefs.new_view;
        heading(ui, "Style");
        let name = super::view_hud::STYLES.iter().find(|s| s.0 == v.face_style).map(|s| s.2).unwrap_or("?");
        let r = egui::ComboBox::from_id_salt("new_view_style").selected_text(name).width(200.0).show_ui(ui, |ui| {
            for (s, _, label, _) in super::view_hud::STYLES {
                let r = ui.selectable_value(&mut v.face_style, s, label);
                super::tray_lib::spot(format!("pstyle:{label}"), r.rect.center());
            }
        });
        super::tray_lib::spot("prefs_style", r.response.rect.center());
        egui::Grid::new("new_view_show").num_columns(4).spacing([16.0, 2.0]).show(ui, |ui| {
            for (k, (on, label)) in [(&mut v.edges, "Edges"), (&mut v.profiles, "Profiles"), (&mut v.axes, "Axes"), (&mut v.ground, "Ground"), (&mut v.sky, "Sky"), (&mut v.grid, "Grid"), (&mut v.shadows, "Shadows")].into_iter().enumerate() {
                let r = ui.checkbox(on, label);
                super::tray_lib::spot(format!("pv:{label}"), r.rect.center());
                if k == 3 {
                    ui.end_row();
                }
            }
            ui.end_row();
        });
        heading(ui, "Camera");
        ui.horizontal(|ui| {
            ui.radio_value(&mut v.perspective, true, "Perspective");
            let r = ui.radio_value(&mut v.perspective, false, "Parallel Projection");
            super::tray_lib::spot("pv:Parallel", r.rect.center());
        });
        ui.horizontal(|ui| {
            ui.label("Field of view");
            ui.add_enabled(v.perspective, egui::DragValue::new(&mut v.fov).range(10.0..=120.0).speed(0.5).suffix("°"));
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("Use Current View").on_hover_text("Take the style, switches and camera of the active viewport").clicked() {
                self.prefs.new_view = self.current_view_defaults();
            }
            if ui.button("Apply to Current View").on_hover_text("Set the active viewport like a new document").clicked() {
                self.apply_view_defaults();
            }
        });
    }

    /// The active viewport's look as [`ViewDefaults`].
    pub(crate) fn current_view_defaults(&self) -> ViewDefaults {
        let p = &self.prefs;
        ViewDefaults {
            face_style: p.face_style,
            edges: p.show_edges,
            profiles: p.profiles,
            axes: p.show_axes,
            ground: p.show_ground,
            sky: p.show_sky,
            grid: p.grid.show,
            shadows: self.doc.model.shadows.enabled,
            perspective: !self.camera.ortho,
            fov: self.camera.fov_y_deg,
        }
    }

    /// The active viewport (and the document's shadows) set from `Prefs::new_view`.
    pub(crate) fn apply_view_defaults(&mut self) {
        let v = self.prefs.new_view.clone();
        let p = &mut self.prefs;
        p.face_style = v.face_style;
        p.show_edges = v.edges;
        p.profiles = v.profiles;
        p.show_axes = v.axes;
        p.show_ground = v.ground;
        p.show_sky = v.sky;
        p.grid.show = v.grid;
        if self.doc.model.shadows.enabled != v.shadows {
            self.doc.mutate_untracked(|m| m.shadows.enabled = v.shadows);
        }
        self.camera.fov_y_deg = v.fov.clamp(10.0, 120.0);
        self.camera.set_ortho(!v.perspective);
        self.viewport.invalidate();
    }
}
