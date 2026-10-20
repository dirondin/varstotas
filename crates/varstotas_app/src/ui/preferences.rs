//! Window → Preferences…: one window with the sections on the left (General, Drawing,
//! Libraries, Downloads, View) and a search. The menus keep their own switches (Snapping,
//! View, the cards' … menus, the Downloads window); the window shows the same values.

use std::path::{Path, PathBuf};

use egui::{Context, RichText, Ui};
use serde::{Deserialize, Serialize};

use super::theme;
use crate::app::{FaceStylePref, VarstotasApp};
use crate::keymap::{self, KeyCmd, Shortcut};
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
    Interface,
    Shortcuts,
    Toolbar,
    Render,
}

impl Section {
    pub const ALL: [Section; 10] = [
        Section::General,
        Section::Interface,
        Section::Drawing,
        Section::Libraries,
        Section::Downloads,
        Section::View,
        Section::Navigation,
        Section::Shortcuts,
        Section::Render,
        Section::Toolbar,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Section::General => "General",
            Section::Drawing => "Drawing",
            Section::Libraries => "Libraries",
            Section::Downloads => "Downloads",
            Section::View => "View",
            Section::Navigation => "Navigation",
            Section::Interface => "Interface",
            Section::Shortcuts => "Shortcuts",
            Section::Toolbar => "Toolbar",
            Section::Render => "Render",
        }
    }

    /// What the search finds a section by (besides its name): the labels of its settings.
    fn words(self) -> &'static str {
        match self {
            Section::General => "auto-save autosave minutes template new document units undo steps history",
            Section::Render => "render image size samples bounces quality denoise programs ffmpeg video animation oidn open image denoise",
            Section::Drawing => "snapping inference endpoint midpoint edge face axis grid step major increments move rotate scale angle",
            Section::Libraries => "components materials folder examples in model tiles small large list",
            Section::Downloads => "watch downloads folder browser",
            Section::Interface => "theme light dark system appearance scale size zoom toolbar icons",
            Section::Navigation => "mouse trackpad orbit zoom pan speed invert wheel cursor middle button two fingers gesture",
            Section::Shortcuts => "keyboard shortcuts hotkeys keys",
            Section::Toolbar => "toolbar buttons order reserve separator customize tools",
            Section::View => "new document style shaded textures monochrome hidden line wireframe realistic edges profiles axes ground sky grid shadows perspective parallel projection fov field of view camera",
        }
    }

    pub fn matches(self, search: &str) -> bool {
        let s = search.trim().to_lowercase();
        s.is_empty()
            || self.name().to_lowercase().contains(&s)
            || self.words().contains(&s)
            || (self == Section::Shortcuts && shortcut_rows(&s).len() < KeyCmd::all().len())
    }
}

/// The commands of the Shortcuts table that `search` finds (all of them when it names none).
fn shortcut_rows(search: &str) -> Vec<KeyCmd> {
    let s = search.trim().to_lowercase();
    let all = KeyCmd::all();
    let found: Vec<KeyCmd> = all.iter().copied().filter(|c| !s.is_empty() && c.name().to_lowercase().contains(&s)).collect();
    if found.is_empty() { all } else { found }
}

/// The window's own state (not saved).
#[derive(Default)]
pub struct PrefsWindow {
    pub open: bool,
    pub section: Section,
    pub search: String,
    /// Shortcuts: the command waiting for its new keys.
    pub capture: Option<KeyCmd>,
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
        ui.add(egui::Label::new(RichText::new(text).small().color(theme::text_dim())).truncate());
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

/// A program / library row: the one in use (or why there is none), Choose…, Default.
fn file_row(ui: &mut Ui, label: &str, custom: &mut Option<PathBuf>, found: Option<PathBuf>, missing: &str) {
    ui.label(label);
    let text = match (&custom, &found) {
        (Some(c), Some(f)) if c == f => c.display().to_string(),
        (Some(c), _) => format!("{} (not usable)", c.display()),
        (None, Some(f)) => format!("{} (found)", f.display()),
        (None, None) => missing.to_string(),
    };
    ui.add(egui::Label::new(RichText::new(text).small().color(theme::text_dim())).truncate());
    ui.horizontal(|ui| {
        if ui.button("Choose…").clicked()
            && let Some(f) = rfd::FileDialog::new().pick_file()
        {
            *custom = Some(f);
        }
        if ui.add_enabled(custom.is_some(), egui::Button::new("Default")).clicked() {
            *custom = None;
        }
    });
}

fn note(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).small().color(theme::text_dim()));
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
        let shown = win.show(ctx, |ui| {
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
                            Section::Interface => self.prefs_interface(ui),
                            Section::Shortcuts => self.prefs_shortcuts(ui),
                            Section::Toolbar => self.prefs_toolbar(ui),
                            Section::Render => self.prefs_render(ui),
                        }
                    });
                });
            });
        });
        if let Some(r) = shown {
            // the title bar's ✕ (harness)
            super::tray_lib::spot("prefs_close", egui::pos2(r.response.rect.max.x - 16.0, r.response.rect.min.y + 14.0));
        }
        if !open {
            self.prefs_win.open = false;
            self.prefs_win.capture = None;
        }
    }

    fn reset_prefs_section(&mut self, s: Section) {
        let d = crate::app::Prefs::default();
        let p = &mut self.prefs;
        match s {
            Section::Render => {
                p.new_render = d.new_render;
                p.ffmpeg_path = None;
                p.oidn_path = None;
            }
            Section::General => {
                p.undo_steps = d.undo_steps;
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
            }
            Section::View => p.new_view = ViewDefaults::default(),
            Section::Navigation => p.nav = Default::default(),
            Section::Shortcuts => p.shortcuts.clear(),
            Section::Toolbar => p.toolbar.clear(),
            Section::Interface => {
                p.theme = d.theme;
                p.ui_scale = d.ui_scale;
                p.toolbar_icons = d.toolbar_icons;
            }
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
        heading(ui, "Undo");
        ui.horizontal(|ui| {
            ui.label("Keep");
            let r = ui.add(egui::DragValue::new(&mut self.prefs.undo_steps).range(10..=1000).speed(1.0));
            super::tray_lib::spot("undo_steps", r.rect.center());
            ui.label("undo steps per document");
        });
        note(ui, "Each step keeps a copy of the model: big models with many steps take more memory.");
    }

    fn prefs_render(&mut self, ui: &mut Ui) {
        use varstotas_kernel::{RENDER_PRESETS, RENDER_SIZES};
        heading(ui, "New documents");
        let r = &mut self.prefs.new_render;
        ui.horizontal(|ui| {
            ui.label("Size");
            let name = RENDER_SIZES
                .iter()
                .find(|s| s.1 == r.width && s.2 == r.height)
                .map(|s| s.0.to_string())
                .unwrap_or_else(|| format!("{} × {}", r.width, r.height));
            egui::ComboBox::from_id_salt("new_render_size").selected_text(name).width(200.0).show_ui(ui, |ui| {
                for (n, w, h) in RENDER_SIZES {
                    if ui.selectable_label(r.width == w && r.height == h, n).clicked() {
                        r.width = w;
                        r.height = h;
                    }
                }
            });
            ui.checkbox(&mut r.match_view, "Match the view").on_hover_text("The height follows the viewport's shape");
        });
        ui.horizontal(|ui| {
            ui.label("Quality");
            for (n, samples, bounces, clamp) in RENDER_PRESETS {
                let on = r.samples == samples && r.bounces == bounces;
                let b = ui.add(egui::Button::selectable(on, n));
                super::tray_lib::spot(format!("rq:{n}"), b.rect.center());
                if b.clicked() {
                    r.samples = samples;
                    r.bounces = bounces;
                    r.clamp = clamp;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Samples");
            ui.add(egui::DragValue::new(&mut r.samples).range(1..=65536));
            ui.label("Bounces");
            ui.add(egui::DragValue::new(&mut r.bounces).range(1..=32));
            ui.label("Time limit");
            ui.add(egui::DragValue::new(&mut r.time_limit).range(0.0..=3600.0).suffix(" s"));
        });
        ui.checkbox(&mut r.denoise, "Denoise when the render stops");
        ui.add_enabled(r.denoise, egui::Checkbox::new(&mut r.ai_denoise, "With Open Image Denoise when installed"));
        ui.horizontal(|ui| {
            let b = ui.button("Use Current Document's").on_hover_text("Take the render settings of the open document");
            super::tray_lib::spot("render_use", b.rect.center());
            if b.clicked() {
                self.prefs.new_render = self.doc.model.render;
            }
            let b = ui.button("Apply to Current Document").on_hover_text("The open document gets these settings (Undo puts them back)");
            super::tray_lib::spot("render_apply", b.rect.center());
            if b.clicked() {
                let r = self.prefs.new_render;
                self.doc.commit("Render Settings", |m| m.render = r);
            }
        });
        note(ui, "File → New starts with these; Window → Render… changes the open document's.");

        heading(ui, "Programs");
        let ffmpeg = crate::animation::ffmpeg(self.prefs.ffmpeg_path.as_deref());
        file_row(ui, "ffmpeg (videos of File → Export → Animation)", &mut self.prefs.ffmpeg_path, ffmpeg, "not found: install it (brew install ffmpeg) or choose the program");
        let oidn = varstotas_render::oidn_library();
        file_row(
            ui,
            "Open Image Denoise library (the render's denoiser)",
            &mut self.prefs.oidn_path,
            oidn,
            "not found: the built-in filter denoises (brew install open-image-denoise, or choose the library)",
        );
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
        note(ui, "Also in Window → 3D Library… → Other Sites.");
    }

    /// The toolbar's buttons in order (↑ ↓, → to the reserve) and the reserve (← back).
    fn prefs_toolbar(&mut self, ui: &mut Ui) {
        use super::toolbar::{Item, layout, reserve};
        let mut items = layout(&self.prefs.toolbar);
        let spare = reserve(&items);
        // (index on the toolbar, -1 up / 1 down / 0 to the reserve) or an item to add
        let mut op: Option<(usize, i32)> = None;
        let mut add: Option<Item> = None;
        note(ui, "The buttons of the toolbar, in order; the others wait in reserve (they stay in the menus).");
        // a row: the icon (a line for a separator) and the name
        let label = |ui: &mut Ui, it: Item| {
            let (r, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
            match it.icon() {
                Some(icon) => super::icons::paint_icon(ui.painter(), r, icon, false),
                None => {
                    ui.painter().hline(r.x_range(), r.center().y, egui::Stroke::new(1.0, theme::text_dim()));
                }
            }
            let text = RichText::new(it.name());
            // long names (Export Attributions …) are cut, the window keeps its width
            let text = if it == Item::Sep { text.color(theme::text_dim()) } else { text };
            ui.add(egui::Label::new(text).truncate()).on_hover_text(it.name());
        };
        // the lists down to the bottom of the window (less the column headings)
        let list_h = (ui.clip_rect().bottom() - ui.cursor().top() - 40.0).max(160.0);
        ui.columns(2, |cols| {
            heading(&mut cols[0], "On the toolbar");
            egui::ScrollArea::vertical().id_salt("tb_on").max_height(list_h).auto_shrink([false, true]).show(&mut cols[0], |ui| {
                let n = items.len();
                for (i, &it) in items.iter().enumerate() {
                    // separators are told apart by their place
                    let key = if it == Item::Sep { format!("|{i}") } else { it.key() };
                    ui.horizontal(|ui| {
                        label(ui, it);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // room for the scroll bar
                            ui.add_space(14.0);
                            for (glyph, d, tip, ok) in [("→", 0, "To the reserve", true), ("⏷", 1, "Down", i + 1 < n), ("⏶", -1, "Up", i > 0)] {
                                let r = ui.add_enabled(ok, egui::Button::new(glyph).small()).on_hover_text(tip);
                                let name = match d {
                                    0 => "out",
                                    1 => "down",
                                    _ => "up",
                                };
                                super::tray_lib::spot(format!("tb_{name}:{key}"), r.rect.center());
                                if r.clicked() {
                                    op = Some((i, d));
                                }
                            }
                        });
                    });
                }
            });
            heading(&mut cols[1], "Reserve");
            egui::ScrollArea::vertical().id_salt("tb_off").max_height(list_h).auto_shrink([false, true]).show(&mut cols[1], |ui| {
                let mut group = None;
                for it in std::iter::once(Item::Sep).chain(spare) {
                    if it.group().is_some() && it.group() != group {
                        group = it.group();
                        ui.add_space(4.0);
                        ui.label(RichText::new(group.unwrap_or_default()).small().color(theme::text_dim()));
                    }
                    ui.horizontal(|ui| {
                        let r = ui.add(egui::Button::new("←").small()).on_hover_text("To the end of the toolbar");
                        super::tray_lib::spot(format!("tb_in:{}", it.key()), r.rect.center());
                        if r.clicked() {
                            add = Some(it);
                        }
                        label(ui, it);
                    });
                }
            });
        });
        let changed = op.is_some() || add.is_some();
        match op {
            Some((i, 0)) => {
                items.remove(i);
            }
            Some((i, d)) => items.swap(i, (i as i32 + d) as usize),
            None => {}
        }
        if let Some(it) = add {
            items.push(it);
        }
        if changed {
            self.prefs.toolbar = items.iter().map(|i| i.key()).collect();
        }
    }

    fn prefs_interface(&mut self, ui: &mut Ui) {
        use super::theme::ThemePref;
        heading(ui, "Theme");
        ui.horizontal(|ui| {
            for (t, label) in [(ThemePref::Light, "Light"), (ThemePref::Dark, "Dark"), (ThemePref::System, "As the System")] {
                let r = ui.radio_value(&mut self.prefs.theme, t, label);
                super::tray_lib::spot(format!("theme:{label}"), r.rect.center());
            }
        });
        note(ui, "The viewport keeps its own colours (sky, ground); sheets and unfold patterns stay white paper.");
        heading(ui, "Scale");
        ui.horizontal(|ui| {
            for pct in [80, 90, 100, 110, 125, 150] {
                let on = (self.prefs.ui_scale * 100.0).round() as i32 == pct;
                let r = ui.add(egui::Button::selectable(on, format!("{pct} %")));
                super::tray_lib::spot(format!("scale:{pct}"), r.rect.center());
                if r.clicked() {
                    self.prefs.ui_scale = pct as f32 / 100.0;
                }
            }
        });
        note(ui, &crate::keys::tr("Menus, panels and windows; %CMDK%+ / %CMDK%− / %CMDK%0 change it too."));
        heading(ui, "Toolbar icons");
        ui.horizontal(|ui| {
            for (label, side) in super::toolbar::ICON_SIZES {
                let r = ui.radio_value(&mut self.prefs.toolbar_icons, side, label);
                super::tray_lib::spot(format!("icons:{label}"), r.rect.center());
            }
        });
    }

    /// Theme, scale and toolbar icon size from the preferences (every frame; cheap when unchanged).
    pub(crate) fn apply_interface(&mut self, ctx: &Context) {
        super::theme::set_look(ctx, super::theme::wants_dark(ctx, self.prefs.theme));
        let key = egui::Id::new("ui_scale_applied");
        let zoom = ctx.zoom_factor();
        let applied = ctx.data(|d| d.get_temp::<f32>(key));
        // ⌘ + / − / 0 (egui's own zoom keys) changed it: keep that in the preferences
        if applied.is_some_and(|a| (zoom - a).abs() > 1e-3) {
            self.prefs.ui_scale = zoom;
        }
        let scale = self.prefs.ui_scale.clamp(0.5, 2.0);
        if (zoom - scale).abs() > 1e-3 {
            ctx.set_zoom_factor(scale);
        }
        ctx.data_mut(|d| d.insert_temp(key, scale));
        super::toolbar::set_button_side(self.prefs.toolbar_icons);
    }

    fn prefs_shortcuts(&mut self, ui: &mut Ui) {
        note(ui, "Click a shortcut, then press the new keys: Esc cancels, Backspace removes the shortcut. A key another command had is taken from it.");
        if let Some(cmd) = self.prefs_win.capture {
            let press = ui.input(|i| {
                i.events.iter().find_map(|e| match e {
                    egui::Event::Key { key, pressed: true, modifiers, .. } => Some((*key, *modifiers)),
                    _ => None,
                })
            });
            if let Some((key, m)) = press {
                // the keys are the table's: no button or window reacts to them
                ui.ctx().input_mut(|i| i.events.retain(|e| !matches!(e, egui::Event::Key { .. } | egui::Event::Text(_))));
                self.prefs_win.capture = None;
                let sc = Shortcut::from_event(key, m);
                match key {
                    egui::Key::Escape => {}
                    egui::Key::Backspace | egui::Key::Delete if !sc.cmd && !sc.alt && !sc.shift => {
                        keymap::assign(&mut self.prefs.shortcuts, cmd, None);
                        self.flash(format!("{}: no shortcut", cmd.name()));
                    }
                    _ => match sc.reserved() {
                        Some(why) => self.flash_error(format!("{} can't be a shortcut: {why}", sc.label())),
                        None => {
                            let taken = keymap::assign(&mut self.prefs.shortcuts, cmd, Some(sc));
                            self.flash(match taken {
                                Some(o) => format!("{}: {} (taken from {o})", cmd.name(), sc.label()),
                                None => format!("{}: {}", cmd.name(), sc.label()),
                            });
                        }
                    },
                }
            }
        }
        // the table shows the published keys: make it this frame's
        keymap::publish(&self.prefs.shortcuts);
        let rows = shortcut_rows(&self.prefs_win.search);
        egui::Grid::new("shortcuts_grid").num_columns(3).striped(true).spacing([12.0, 4.0]).show(ui, |ui| {
            let mut tools_heading = false;
            for cmd in rows {
                if matches!(cmd, KeyCmd::Tool(_)) && !tools_heading {
                    tools_heading = true;
                    ui.label(RichText::new("Tools").strong());
                    ui.end_row();
                }
                ui.label(cmd.name());
                let waiting = self.prefs_win.capture == Some(cmd);
                let keys = keymap::labels(cmd);
                let text = if waiting {
                    RichText::new("Press keys…").italics()
                } else if keys.is_empty() {
                    RichText::new("—").color(theme::text_dim())
                } else {
                    RichText::new(keys).monospace()
                };
                let changed = self.prefs.shortcuts.contains_key(&cmd.id());
                let r = ui.add(egui::Button::new(text).selected(waiting).min_size(egui::vec2(140.0, 0.0)));
                super::tray_lib::spot(format!("key:{}", cmd.id()), r.rect.center());
                if r.clicked() {
                    self.prefs_win.capture = if waiting { None } else { Some(cmd) };
                }
                if changed {
                    let def: Vec<String> = cmd.defaults().iter().map(|k| k.label()).collect();
                    let tip = if def.is_empty() { "Default: none".to_string() } else { format!("Default: {}", def.join(", ")) };
                    let r = ui.small_button("↺").on_hover_text(tip);
                    super::tray_lib::spot(format!("keyreset:{}", cmd.id()), r.rect.center());
                    if r.clicked() {
                        keymap::reset(&mut self.prefs.shortcuts, cmd);
                    }
                } else {
                    ui.label("");
                }
                ui.end_row();
            }
        });
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
        note(ui, &crate::keys::tr("Shift: Pan (Orbit when two fingers pan). Pinch and %CMD% + two fingers always zoom."));
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
