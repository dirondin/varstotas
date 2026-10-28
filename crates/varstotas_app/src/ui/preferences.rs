//! Window → Preferences: one window with the sections on the left (General, Drawing,
//! Libraries, Downloads, View) and a search. The menus keep their own switches (Snapping,
//! View, the cards' … menus, the Downloads window); the window shows the same values.

use std::path::{Path, PathBuf};

use egui::{Context, RichText, Ui};
use serde::{Deserialize, Serialize};

use super::theme;
use crate::app::{FaceStylePref, VarstotasApp};
use crate::keymap::{self, KeyCmd, Shortcut};
use crate::templates::Template;
use varstotas_i18n::tr;
use varstotas_kernel::{render_preset_label, render_size_label};

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

    pub fn name(self) -> String {
        varstotas_i18n::tr(match self {
            Section::General => "prefs-section-general",
            Section::Drawing => "prefs-section-drawing",
            Section::Libraries => "prefs-section-libraries",
            Section::Downloads => "prefs-section-downloads",
            Section::View => "prefs-section-view",
            Section::Navigation => "prefs-section-navigation",
            Section::Interface => "prefs-section-interface",
            Section::Shortcuts => "prefs-section-shortcuts",
            Section::Toolbar => "prefs-section-toolbar",
            Section::Render => "prefs-section-render",
        })
    }

    /// What the search finds a section by (besides its name): the labels of its settings.
    fn words(self) -> &'static str {
        match self {
            Section::General => "auto-save autosave backup previous version minutes template folder directory new document units undo steps history", // i18n-ignore
            Section::Render => "render image size samples bounces quality denoise programs image editor texture gif animation oidn open image denoise", // i18n-ignore
            Section::Drawing => "snapping inference endpoint midpoint edge face axis grid step major increments move rotate scale angle", // i18n-ignore
            Section::Libraries => "components materials folder examples in model tiles small large list", // i18n-ignore
            Section::Downloads => "watch downloads folder browser", // i18n-ignore
            Section::Interface => "theme light dark system appearance scale size zoom toolbar icons language translation startup welcome start screen", // i18n-ignore
            Section::Navigation => "mouse trackpad orbit zoom pan speed invert wheel cursor middle button two fingers gesture pinch twist roll multitouch", // i18n-ignore
            Section::Shortcuts => "keyboard shortcuts hotkeys keys", // i18n-ignore
            Section::Toolbar => "toolbar buttons order reserve separator customize tools", // i18n-ignore
            Section::View => "new document style shaded textures monochrome hidden line wireframe realistic edges profiles axes ground sky grid shadows perspective parallel projection fov field of view camera", // i18n-ignore
        }
    }

    /// The search words in the UI language (a translation may list its own).
    fn words_translated(self) -> String {
        varstotas_i18n::tr(match self {
            Section::General => "prefs-words-general",
            Section::Drawing => "prefs-words-drawing",
            Section::Libraries => "prefs-words-libraries",
            Section::Downloads => "prefs-words-downloads",
            Section::View => "prefs-words-view",
            Section::Navigation => "prefs-words-navigation",
            Section::Interface => "prefs-words-interface",
            Section::Shortcuts => "prefs-words-shortcuts",
            Section::Toolbar => "prefs-words-toolbar",
            Section::Render => "prefs-words-render",
        })
        .to_lowercase()
    }

    pub fn matches(self, search: &str) -> bool {
        let s = search.trim().to_lowercase();
        s.is_empty()
            || self.name().to_lowercase().contains(&s)
            || self.words().contains(&s)
            || self.words_translated().contains(&s)
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
    /// Interface: the available languages (code, own name), read when the section is shown.
    pub languages: Option<Vec<(String, String)>>,
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
    // Adopt the new grid-on default even when old preferences saved the former
    // grid-off default. Future explicit choices persist under the new key.
    #[serde(rename = "grid_enabled")]
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
            grid: true,
            shadows: false,
            perspective: true,
            fov: 35.0,
        }
    }
}

/// A folder row: the path (or "default: …"), Choose…, Default, Show in Finder.
fn folder_row(ui: &mut Ui, label: &str, custom: &mut Option<PathBuf>, default: Option<PathBuf>, tip: &str) {
    ui.label(label).on_hover_text(tip);
    let shown = custom.clone().or(default);
    ui.horizontal(|ui| {
        let text = match (&custom, &shown) {
            (Some(p), _) => p.display().to_string(),
            (None, Some(p)) => tr!("prefs-folder-default", path = p.display().to_string()),
            (None, None) => "—".to_string(),
        };
        ui.add(egui::Label::new(RichText::new(text).small().color(theme::text_dim())).truncate());
    });
    ui.horizontal(|ui| {
        if ui.button(tr!("prefs-choose")).clicked()
            && let Some(d) = rfd::FileDialog::new().pick_folder()
        {
            *custom = Some(d);
        }
        if ui.add_enabled(custom.is_some(), egui::Button::new(tr!("prefs-default"))).clicked() {
            *custom = None;
        }
        if ui.add_enabled(shown.as_deref().is_some_and(Path::is_dir), egui::Button::new(tr!("prefs-show-in-finder"))).clicked()
            && let Some(d) = &shown
        {
            crate::ui::extensions::reveal(d);
        }
    });
}

/// A program / library row: the one in use (or why there is none), Choose…, Default.
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
        let win = egui::Window::new(tr!("prefs-title"))
            .id(egui::Id::new("preferences_window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([620.0, 460.0]);
        let win = self.placed(win, pos);
        let shown = win.show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                // the list of sections and the search
                ui.vertical(|ui| {
                    ui.set_width(140.0);
                    let r = ui.add(egui::TextEdit::singleline(&mut self.prefs_win.search).hint_text(tr!("prefs-search")).desired_width(140.0));
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
                        note(ui, &tr!("prefs-nothing-found"));
                    }
                });
                ui.separator();
                ui.vertical(|ui| {
                    egui::ScrollArea::vertical().id_salt("prefs_body").auto_shrink([false, false]).show(ui, |ui| {
                        let section = self.prefs_win.section;
                        ui.horizontal(|ui| {
                            ui.heading(section.name());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let r = ui.button(tr!("prefs-reset")).on_hover_text(tr!("prefs-reset-tip"));
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
            }
            Section::General => {
                p.undo_steps = d.undo_steps;
                p.autosave = d.autosave;
                p.save_backups = d.save_backups;
                p.bridge_external = d.bridge_external;
                p.autosave_minutes = d.autosave_minutes;
                p.template = d.template;
                p.templates_dir = None;
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
                p.show_start_screen = d.show_start_screen;
                p.theme = d.theme;
                p.ui_scale = d.ui_scale;
                p.toolbar_icons = d.toolbar_icons;
            }
        }
        if matches!(s, Section::General) { self.ensure_templates_dir(); }
        self.flash(tr!("prefs-defaults-flash", section = s.name()));
    }

    /// The graphics API (see gpu.rs): a choice for the next start and a restart button.
    fn prefs_graphics(&mut self, ui: &mut Ui) {
        use crate::gpu::Api;
        let st = crate::gpu::status();
        heading(ui, &tr!("prefs-gpu"));
        let label = Api::label;
        if st.managed && Api::available().len() > 1 {
            let mut choice = st.choice;
            egui::ComboBox::from_id_salt("prefs_gpu").selected_text(label(choice)).show_ui(ui, |ui| {
                for a in std::iter::once(Api::Auto).chain(Api::available().iter().copied()) {
                    ui.selectable_value(&mut choice, a, label(a));
                }
            });
            if choice != st.choice {
                crate::gpu::set_choice(choice);
            }
            if choice != st.running_choice && ui.button(tr!("prefs-gpu-restart")).clicked() {
                self.restart_for_graphics(ui.ctx());
            }
        }
        if let Some(u) = &st.in_use {
            ui.label(tr!("prefs-gpu-in-use", api = u.clone()));
        }
        if let Some(f) = st.fell_back {
            ui.colored_label(ui.visuals().warn_fg_color, tr!("prefs-gpu-fell-back", api = f.label()));
        }
        if std::env::var_os("WGPU_BACKEND").is_some() {
            note(ui, &tr!("prefs-gpu-env"));
        } else if st.managed {
            note(ui, &tr!("prefs-gpu-note"));
        }
    }

    fn prefs_general(&mut self, ui: &mut Ui) {
        if let Some(dir) = crate::paths::data_dir() {
            heading(ui, &tr!("prefs-data-folder"));
            ui.horizontal(|ui| {
                ui.monospace(crate::paths::display(&dir));
                if ui.button(tr!("reveal-in-file-manager")).clicked() {
                    super::extensions::reveal(&dir);
                }
            });
            note(ui, &tr!("prefs-data-folder-note"));
        }
        self.prefs_graphics(ui);
        if crate::appimage::running().is_some() {
            heading(ui, &tr!("prefs-appimage-menu"));
            if crate::appimage::installed() {
                if ui.button(tr!("prefs-appimage-remove")).clicked() {
                    self.appimage_menu(false);
                }
            } else if ui.button(tr!("prefs-appimage-add")).clicked() {
                self.appimage_menu(true);
            }
            note(ui, &tr!("prefs-appimage-note"));
        }
        heading(ui, &tr!("prefs-bridge"));
        let r = ui.checkbox(&mut self.prefs.bridge_external, tr!("prefs-bridge-external"));
        super::tray_lib::spot("prefs:bridge_external", r.rect.center());
        note(ui, &tr!("prefs-bridge-note"));
        heading(ui, &tr!("prefs-autosave"));
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.prefs.autosave, tr!("prefs-autosave-every"));
            ui.add_enabled(self.prefs.autosave, egui::DragValue::new(&mut self.prefs.autosave_minutes).range(1..=120).speed(0.2));
            ui.label(tr!("prefs-minutes"));
        });
        note(ui, &tr!("prefs-autosave-note"));
        let r = ui.checkbox(&mut self.prefs.save_backups, tr!("prefs-save-backups"));
        super::tray_lib::spot("prefs:save_backups", r.rect.center());
        note(ui, &tr!("prefs-save-backups-note"));
        heading(ui, &tr!("prefs-new-document"));
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
        note(ui, &tr!("prefs-template-note"));
        let old_dir = self.prefs.templates_dir.clone();
        folder_row(ui, &tr!("template-folder-label"), &mut self.prefs.templates_dir, crate::templates::default_dir(), &tr!("template-folder-tip"));
        if self.prefs.templates_dir != old_dir { self.ensure_templates_dir(); }

        heading(ui, &tr!("prefs-undo"));
        ui.horizontal(|ui| {
            ui.label(tr!("prefs-keep"));
            let r = ui.add(egui::DragValue::new(&mut self.prefs.undo_steps).range(10..=1000).speed(1.0));
            super::tray_lib::spot("undo_steps", r.rect.center());
            ui.label(tr!("prefs-undo-steps"));
        });
        note(ui, &tr!("prefs-undo-note"));
    }

    fn prefs_render(&mut self, ui: &mut Ui) {
        use varstotas_kernel::{RENDER_PRESETS, RENDER_SIZES};
        heading(ui, &tr!("prefs-new-documents"));
        let r = &mut self.prefs.new_render;
        ui.horizontal(|ui| {
            ui.label(tr!("render-size"));
            let name = RENDER_SIZES
                .iter()
                .find(|s| s.1 == r.width && s.2 == r.height)
                .map(|s| render_size_label(s.0))
                .unwrap_or_else(|| format!("{} × {}", r.width, r.height));
            egui::ComboBox::from_id_salt("new_render_size").selected_text(name).width(200.0).show_ui(ui, |ui| {
                for (n, w, h) in RENDER_SIZES {
                    if ui.selectable_label(r.width == w && r.height == h, render_size_label(n)).clicked() {
                        r.width = w;
                        r.height = h;
                    }
                }
            });
            ui.checkbox(&mut r.match_view, tr!("prefs-match-view")).on_hover_text(tr!("prefs-match-view-tip"));
        });
        ui.horizontal(|ui| {
            ui.label(tr!("render-quality"));
            for (n, samples, bounces, clamp) in RENDER_PRESETS {
                let on = r.samples == samples && r.bounces == bounces;
                let b = ui.add(egui::Button::selectable(on, render_preset_label(n)));
                super::tray_lib::spot(format!("rq:{n}"), b.rect.center());
                if b.clicked() {
                    r.samples = samples;
                    r.bounces = bounces;
                    r.clamp = clamp;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label(tr!("render-samples"));
            ui.add(egui::DragValue::new(&mut r.samples).range(1..=65536));
            ui.label(tr!("render-bounces"));
            ui.add(egui::DragValue::new(&mut r.bounces).range(1..=32));
            ui.label(tr!("render-time-limit"));
            ui.add(egui::DragValue::new(&mut r.time_limit).range(0.0..=3600.0).suffix(" s"));
        });
        ui.checkbox(&mut r.denoise, tr!("prefs-denoise"));
        ui.add_enabled(r.denoise, egui::Checkbox::new(&mut r.ai_denoise, tr!("prefs-denoise-ai")));
        ui.horizontal(|ui| {
            let b = ui.button(tr!("prefs-use-current-doc")).on_hover_text(tr!("prefs-use-current-doc-tip"));
            super::tray_lib::spot("render_use", b.rect.center());
            if b.clicked() {
                self.prefs.new_render = self.doc.model.render;
            }
            let b = ui.button(tr!("prefs-apply-current-doc")).on_hover_text(tr!("prefs-apply-current-doc-tip"));
            super::tray_lib::spot("render_apply", b.rect.center());
            if b.clicked() {
                let r = self.prefs.new_render;
                self.doc.commit(&tr!("render-undo-settings"), |m| m.render = r);
            }
        });
        note(ui, &tr!("prefs-render-note"));

        heading(ui, &tr!("prefs-programs"));
        ui.label(tr!("prefs-oidn"));
        let oidn = match varstotas_render::oidn_library() {
            Some(f) => tr!("prefs-file-found", path = f.display().to_string()),
            None => tr!("prefs-oidn-missing"),
        };
        ui.add(egui::Label::new(RichText::new(oidn).small().color(theme::text_dim())).truncate());
    }

    fn prefs_drawing(&mut self, ui: &mut Ui) {
        heading(ui, &tr!("prefs-grid"));
        let unit = self.doc.model.unit;
        ui.horizontal(|ui| {
            let mut grid = self.prefs.grid.show;
            if ui.checkbox(&mut grid, tr!("prefs-show-grid")).clicked() {
                self.actions.push(crate::app::Action::ToggleGrid);
            }
            ui.label(tr!("prefs-grid-step"));
            ui.add(
                super::len_drag(&mut self.prefs.grid.step, unit).range(0.01..=1.0e6),
            );
            ui.label(tr!("prefs-grid-major"));
            ui.add(egui::DragValue::new(&mut self.prefs.grid.major).range(2..=20).speed(0.1));
        });
        note(ui, &tr!("prefs-grid-note"));
        heading(ui, &tr!("prefs-snapping"));
        note(ui, &tr!("prefs-snapping-note"));
        self.snapping_menu(ui);
    }

    fn prefs_libraries(&mut self, ui: &mut Ui) {
        let home = crate::paths::data_dir();
        heading(ui, &tr!("prefs-components"));
        folder_row(ui, &tr!("prefs-library-folder"), &mut self.prefs.library_dir, home.as_ref().map(|h| h.join("components")), &tr!("prefs-components-folder-tip"));
        if crate::debug_capture::qa_examples_dir().is_some() {
            let mut show_ex = !self.prefs.hide_examples;
            ui.checkbox(&mut show_ex, tr!("prefs-show-examples")).on_hover_text(tr!("prefs-show-examples-tip"));
            self.prefs.hide_examples = !show_ex;
        }
        let mut show_doc = !self.prefs.hide_document;
        ui.checkbox(&mut show_doc, tr!("prefs-show-in-model")).on_hover_text(tr!("prefs-show-in-model-tip"));
        self.prefs.hide_document = !show_doc;
        heading(ui, &tr!("prefs-materials"));
        folder_row(ui, &tr!("prefs-library-folder"), &mut self.prefs.mat_library_dir, home.as_ref().map(|h| h.join("materials")), &tr!("prefs-materials-folder-tip"));
        if crate::debug_capture::qa_examples_dir().is_some() {
            let mut show_ex = !self.prefs.hide_mat_examples;
            ui.checkbox(&mut show_ex, tr!("prefs-show-examples")).on_hover_text(tr!("prefs-show-mat-examples-tip"));
            self.prefs.hide_mat_examples = !show_ex;
        }
        heading(ui, &tr!("prefs-tiles"));
        ui.horizontal(|ui| {
            for (s, label) in super::tray_lib::TILE_SIZES {
                let r = ui.radio_value(&mut self.prefs.lib_tiles, s, super::tray_lib::tile_label(label));
                super::tray_lib::spot(format!("tiles:{label}"), r.rect.center());
            }
        });
        note(ui, &tr!("prefs-tiles-note"));
    }

    fn prefs_downloads(&mut self, ui: &mut Ui) {
        heading(ui, &tr!("prefs-watch-downloads"));
        ui.checkbox(&mut self.prefs.watch_downloads, tr!("prefs-watch-offer")).on_hover_text(tr!("prefs-watch-offer-tip"));
        folder_row(ui, &tr!("prefs-watched-folder"), &mut self.prefs.downloads_watch_dir, crate::downloads::downloads_folder(), &tr!("prefs-watched-folder-tip"));
        note(ui, &tr!("prefs-downloads-note"));
    }

    /// The toolbar's buttons in order (↑ ↓, → to the reserve) and the reserve (← back).
    fn prefs_toolbar(&mut self, ui: &mut Ui) {
        use super::toolbar::{Item, layout, reserve};
        let mut items = layout(&self.prefs.toolbar);
        let spare = reserve(&items);
        // (index on the toolbar, -1 up / 1 down / 0 to the reserve) or an item to add
        let mut op: Option<(usize, i32)> = None;
        let mut add: Option<Item> = None;
        note(ui, &tr!("prefs-toolbar-note"));
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
            heading(&mut cols[0], &tr!("prefs-toolbar-on"));
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
                            for (glyph, d, tip, ok) in [("→", 0, tr!("prefs-toolbar-to-reserve"), true), ("⏷", 1, tr!("prefs-toolbar-down"), i + 1 < n), ("⏶", -1, tr!("prefs-toolbar-up"), i > 0)] {
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
            heading(&mut cols[1], &tr!("prefs-toolbar-reserve"));
            egui::ScrollArea::vertical().id_salt("tb_off").max_height(list_h).auto_shrink([false, true]).show(&mut cols[1], |ui| {
                let mut group = None;
                for it in std::iter::once(Item::Sep).chain(spare) {
                    if it.group().is_some() && it.group() != group {
                        group = it.group();
                        ui.add_space(4.0);
                        ui.label(RichText::new(super::toolbar::group_label(group.unwrap_or_default())).small().color(theme::text_dim()));
                    }
                    ui.horizontal(|ui| {
                        let r = ui.add(egui::Button::new("←").small()).on_hover_text(tr!("prefs-toolbar-to-end"));
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
        heading(ui, &tr!("prefs-theme"));
        ui.horizontal(|ui| {
            for (t, label) in [(ThemePref::Light, "Light"), (ThemePref::Dark, "Dark"), (ThemePref::System, "As the System")] { // i18n-ignore
                let r = ui.radio_value(&mut self.prefs.theme, t, tr(match t {
                    ThemePref::Light => "prefs-theme-light",
                    ThemePref::Dark => "prefs-theme-dark",
                    ThemePref::System => "prefs-theme-system",
                }));
                super::tray_lib::spot(format!("theme:{label}"), r.rect.center());
            }
        });
        note(ui, &tr!("prefs-theme-note"));
        heading(ui, &tr!("prefs-scale"));
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
        note(ui, &tr!("prefs-scale-note"));
        heading(ui, &tr!("prefs-toolbar-icons"));
        ui.horizontal(|ui| {
            for (label, side) in super::toolbar::ICON_SIZES {
                let r = ui.radio_value(&mut self.prefs.toolbar_icons, side, super::toolbar::icon_size_label(label));
                super::tray_lib::spot(format!("icons:{label}"), r.rect.center());
            }
        });
        heading(ui, &tr!("prefs-language"));
        let langs = self.prefs_win.languages.get_or_insert_with(varstotas_i18n::languages).clone();
        let name_of = |code: &str| langs.iter().find(|(c, _)| c == code).map(|(_, n)| n.clone()).unwrap_or_else(|| code.to_string());
        let system = tr!("prefs-language-system", language = name_of(&varstotas_i18n::system_language()));
        let mut lang = self.prefs.language.clone();
        let shown = lang.as_deref().map(name_of).unwrap_or_else(|| system.clone());
        let r = egui::ComboBox::from_id_salt("ui_language").selected_text(shown).width(260.0).show_ui(ui, |ui| {
            ui.selectable_value(&mut lang, None, &system);
            for (code, name) in &langs {
                ui.selectable_value(&mut lang, Some(code.clone()), name);
            }
        });
        super::tray_lib::spot("language", r.response.rect.center());
        if lang != self.prefs.language {
            varstotas_i18n::set_language(lang.as_deref());
            self.prefs.language = lang;
            // Plugin headers carry their texts per language.
            self.load_plugins();
        }
        note(ui, &tr!("prefs-language-note"));
        ui.separator();
        let r = ui.checkbox(&mut self.prefs.show_start_screen, tr!("start-show"));
        super::tray_lib::spot("prefs:start_show", r.rect.center());
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
        note(ui, &tr!("prefs-shortcuts-note"));
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
                        self.flash(tr!("prefs-shortcut-removed", command = cmd.name()));
                    }
                    _ => match sc.reserved() {
                        Some(why) => self.flash_error(tr!("prefs-shortcut-reserved", keys = sc.label(), why = why)),
                        None => {
                            let taken = keymap::assign(&mut self.prefs.shortcuts, cmd, Some(sc));
                            self.flash(match taken {
                                Some(o) => tr!("prefs-shortcut-taken", command = cmd.name(), keys = sc.label(), other = o),
                                None => tr!("prefs-shortcut-set", command = cmd.name(), keys = sc.label()),
                            });
                        }
                    },
                }
            }
        }
        // the table shows the published keys: make it this frame's
        keymap::publish(&self.prefs.shortcuts);
        let rows = shortcut_rows(&self.prefs_win.search);
        egui::Grid::new("shortcuts_grid").num_columns(4).spacing([12.0, 4.0]).show(ui, |ui| {
            let mut tools_heading = false;
            for cmd in rows {
                if matches!(cmd, KeyCmd::Tool(_)) && !tools_heading {
                    tools_heading = true;
                    ui.label("");
                    ui.label(RichText::new(tr!("prefs-shortcuts-tools")).strong());
                    ui.end_row();
                }
                let (rect, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                super::icons::paint_icon(ui.painter(), rect, cmd.icon(), false);
                ui.label(cmd.name());
                let waiting = self.prefs_win.capture == Some(cmd);
                let keys = keymap::labels(cmd);
                let text = if waiting {
                    RichText::new(tr!("prefs-press-keys")).italics()
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
                    let tip = if def.is_empty() { tr!("prefs-default-none") } else { tr!("prefs-default-keys", keys = def.join(", ")) };
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
        use crate::nav::{NavGesture, TwistGesture};
        let n = &mut self.prefs.nav;
        heading(ui, &tr!("prefs-speed"));
        egui::Grid::new("nav_speed").num_columns(2).spacing([12.0, 4.0]).show(ui, |ui| {
            ui.label(tr!("prefs-nav-orbit"));
            let r = ui.add(egui::Slider::new(&mut n.orbit_speed, 0.25..=4.0).logarithmic(true).suffix("×"));
            super::tray_lib::spot("nav:orbit", r.rect.center());
            ui.end_row();
            ui.label(tr!("prefs-nav-zoom"));
            let r = ui.add(egui::Slider::new(&mut n.zoom_speed, 0.25..=4.0).logarithmic(true).suffix("×"));
            super::tray_lib::spot("nav:zoom", r.rect.center());
            ui.end_row();
        });
        ui.horizontal(|ui| {
            ui.label(tr!("prefs-nav-pan"));
            ui.add(egui::Slider::new(&mut n.pan_speed, 0.25..=4.0).logarithmic(true).suffix("×"));
        });
        heading(ui, &tr!("prefs-nav-zoom"));
        let r = ui.checkbox(&mut n.invert_zoom, tr!("prefs-invert-wheel")).on_hover_text(tr!("prefs-invert-wheel-tip"));
        super::tray_lib::spot("nav:invert", r.rect.center());
        let r = ui.checkbox(&mut n.zoom_to_cursor, tr!("prefs-zoom-cursor")).on_hover_text(tr!("prefs-zoom-cursor-tip"));
        super::tray_lib::spot("nav:cursor", r.rect.center());
        heading(ui, &tr!("prefs-mouse"));
        ui.horizontal(|ui| {
            ui.label(tr!("prefs-middle-drag"));
            ui.radio_value(&mut n.middle, NavGesture::Orbit, tr!("prefs-nav-orbit"));
            let r = ui.radio_value(&mut n.middle, NavGesture::Pan, tr!("prefs-nav-pan"));
            super::tray_lib::spot("nav:middle_pan", r.rect.center());
        });
        note(ui, &tr!("prefs-shift-other"));
        heading(ui, &tr!("prefs-trackpad"));
        let r = ui.checkbox(&mut n.trackpad_enabled, tr!("prefs-trackpad-enabled"));
        super::tray_lib::spot("nav:trackpad", r.rect.center());
        ui.add_enabled_ui(n.trackpad_enabled, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(tr!("prefs-two-fingers"));
                ui.radio_value(&mut n.two_finger, NavGesture::Orbit, tr!("prefs-nav-orbit"));
                let r = ui.radio_value(&mut n.two_finger, NavGesture::Pan, tr!("prefs-nav-pan"));
                super::tray_lib::spot("nav:two_pan", r.rect.center());
                let r = ui.radio_value(&mut n.two_finger, NavGesture::Zoom, tr!("prefs-nav-zoom"));
                super::tray_lib::spot("nav:two_zoom", r.rect.center());
            });
            ui.horizontal(|ui| {
                ui.label(tr!("prefs-gesture-speed"));
                ui.add(egui::Slider::new(&mut n.trackpad_speed, 0.25..=4.0).logarithmic(true).suffix("×"));
            });
            let r = ui.checkbox(&mut n.invert_swipe, tr!("prefs-invert-swipe"));
            super::tray_lib::spot("nav:invert_swipe", r.rect.center());
            note(ui, &tr!("prefs-trackpad-note"));
            ui.separator();
            let r = ui.checkbox(&mut n.pinch_enabled, tr!("prefs-pinch-enabled"));
            super::tray_lib::spot("nav:pinch", r.rect.center());
            ui.add_enabled_ui(n.pinch_enabled, |ui| {
                ui.horizontal(|ui| {
                    ui.label(tr!("prefs-gesture-speed"));
                    ui.add(egui::Slider::new(&mut n.pinch_speed, 0.25..=4.0).logarithmic(true).suffix("×"));
                });
                let r = ui.checkbox(&mut n.invert_pinch, tr!("prefs-invert-pinch"));
                super::tray_lib::spot("nav:invert_pinch", r.rect.center());
            });
            ui.separator();
            ui.label(RichText::new(tr!("prefs-twist")).strong());
            ui.horizontal_wrapped(|ui| {
                for (gesture, label, spot) in [
                    (TwistGesture::Disabled, "prefs-twist-off", "nav:twist_off"),
                    (TwistGesture::Orbit, "prefs-twist-orbit", "nav:twist_orbit"),
                    (TwistGesture::Roll, "prefs-twist-roll", "nav:twist_roll"),
                ] {
                    let r = ui.radio_value(&mut n.twist, gesture, tr!(label));
                    super::tray_lib::spot(spot, r.rect.center());
                }
            });
            ui.add_enabled_ui(n.twist != TwistGesture::Disabled, |ui| {
                ui.horizontal(|ui| {
                    ui.label(tr!("prefs-gesture-speed"));
                    ui.add(egui::Slider::new(&mut n.twist_speed, 0.25..=4.0).logarithmic(true).suffix("×"));
                });
                ui.checkbox(&mut n.invert_twist, tr!("prefs-invert-twist"));
            });
            note(ui, &tr!("prefs-trackpad-platform"));
        });
    }

    fn prefs_view(&mut self, ui: &mut Ui) {
        note(ui, &tr!("prefs-view-note"));
        let v = &mut self.prefs.new_view;
        heading(ui, &tr!("realistic-style"));
        let name = super::view_hud::STYLES.iter().find(|s| s.0 == v.face_style).map(|s| tr(s.2)).unwrap_or_else(|| "?".to_string()); // i18n-ignore
        let r = egui::ComboBox::from_id_salt("new_view_style").selected_text(name).width(200.0).show_ui(ui, |ui| {
            for (s, _, label, _) in super::view_hud::STYLES {
                let label = tr(label);
                let r = ui.selectable_value(&mut v.face_style, s, label.clone());
                super::tray_lib::spot(format!("pstyle:{label}"), r.rect.center());
            }
        });
        super::tray_lib::spot("prefs_style", r.response.rect.center());
        egui::Grid::new("new_view_show").num_columns(4).spacing([16.0, 2.0]).show(ui, |ui| {
            for (k, (on, label, key)) in [(&mut v.edges, "Edges", "view-edges"), (&mut v.profiles, "Profiles", "view-profiles"), (&mut v.axes, "Axes", "view-show-axes"), (&mut v.ground, "Ground", "view-show-ground"), (&mut v.sky, "Sky", "view-show-sky"), (&mut v.grid, "Grid", "view-show-grid"), (&mut v.shadows, "Shadows", "view-show-shadows")].into_iter().enumerate() { // i18n-ignore
                let r = ui.checkbox(on, tr(key));
                super::tray_lib::spot(format!("pv:{label}"), r.rect.center());
                if k == 3 {
                    ui.end_row();
                }
            }
            ui.end_row();
        });
        heading(ui, &tr!("prefs-camera"));
        ui.horizontal(|ui| {
            ui.radio_value(&mut v.perspective, true, tr!("view-perspective"));
            let r = ui.radio_value(&mut v.perspective, false, tr!("view-parallel-projection"));
            super::tray_lib::spot("pv:Parallel", r.rect.center());
        });
        ui.horizontal(|ui| {
            ui.label(tr!("prefs-fov"));
            ui.add_enabled(v.perspective, egui::DragValue::new(&mut v.fov).range(10.0..=120.0).speed(0.5).suffix("°"));
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button(tr!("prefs-use-current-view")).on_hover_text(tr!("prefs-use-current-view-tip")).clicked() {
                self.prefs.new_view = self.current_view_defaults();
            }
            if ui.button(tr!("prefs-apply-current-view")).on_hover_text(tr!("prefs-apply-current-view-tip")).clicked() {
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
