//! Online Materials window (Materials card → Library → Online…): browse the Poly Haven and
//! ambientCG catalogues, download a material's 1K maps into the materials library
//! (`<library>/Poly Haven/…`) and paint with it. See `crate::online_mats`.

use crate::app::VarstotasApp;
use crate::online_mats::{Asset, Msg, Online, Source};
use egui::{Color32, RichText, Ui};
use std::collections::{HashMap, HashSet};

const TILE: f32 = 96.0;

pub enum ThumbState {
    Loading,
    Ready(egui::TextureHandle),
    Failed,
}

pub struct OnlineWindow {
    pub open: bool,
    pub source: Source,
    pub query: String,
    /// "" = every category.
    pub category: String,
    pub catalogues: HashMap<Source, Result<Vec<Asset>, String>>,
    loading: HashSet<Source>,
    thumbs: HashMap<String, ThumbState>,
    /// Asset keys being downloaded.
    pub downloading: HashSet<String>,
    /// The selected asset's key.
    pub selected: Option<String>,
    /// The last finished download: "<key> ok <file>" / "<key> error <message>" (harness).
    pub last: Option<String>,
    pub work: Online,
}

impl Default for OnlineWindow {
    fn default() -> Self {
        Self {
            open: false,
            source: Source::PolyHaven,
            query: String::new(),
            category: String::new(),
            catalogues: HashMap::new(),
            loading: HashSet::new(),
            thumbs: HashMap::new(),
            downloading: HashSet::new(),
            selected: None,
            last: None,
            work: Online::default(),
        }
    }
}

impl OnlineWindow {
    pub fn busy(&self) -> bool {
        self.work.busy > 0
    }

    /// The assets of the current source passing the search and the category.
    pub fn shown(&self) -> Vec<&Asset> {
        match self.catalogues.get(&self.source) {
            Some(Ok(list)) => list.iter().filter(|a| (self.category.is_empty() || a.category == self.category) && a.matches(&self.query)).collect(),
            _ => Vec::new(),
        }
    }

    pub fn asset(&self, key: &str) -> Option<&Asset> {
        self.catalogues.values().flatten().flatten().find(|a| a.key() == key)
    }

    /// Start loading the current source's catalogue if it isn't there.
    pub fn ensure_catalogue(&mut self, ctx: &egui::Context) {
        if !self.catalogues.contains_key(&self.source) && self.loading.insert(self.source) {
            self.work.load_catalogue(self.source, ctx);
        }
    }

    fn thumb(&mut self, ctx: &egui::Context, a: &Asset) -> Option<egui::TextureId> {
        match self.thumbs.get(&a.thumb) {
            Some(ThumbState::Ready(t)) => Some(t.id()),
            Some(_) => None,
            None => {
                if !a.thumb.is_empty() {
                    self.thumbs.insert(a.thumb.clone(), ThumbState::Loading);
                    self.work.thumb(&a.thumb, &a.key(), ctx);
                }
                None
            }
        }
    }
}

/// Open `url` in the browser (not in scripted runs).
fn open_url(url: &str) {
    if std::env::var_os("VARSTOTAS_SCRIPT").is_some() {
        log::info!("open {url}");
        return;
    }
    let mut cmd = if cfg!(target_os = "macos") {
        std::process::Command::new("open")
    } else if cfg!(windows) {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", ""]);
        c
    } else {
        std::process::Command::new("xdg-open")
    };
    let _ = cmd.arg(url).spawn();
}

impl VarstotasApp {
    /// Download asset `key` into the library (and paint with it when `use_it`).
    pub(crate) fn online_download(&mut self, key: &str, use_it: bool, ctx: &egui::Context) {
        let (Some(asset), Some(dir)) = (self.online.asset(key).cloned(), self.material_library_dir()) else {
            log::warn!("online: no asset {key}");
            return;
        };
        if self.online.downloading.insert(key.to_string()) {
            self.online.work.download(asset, dir, use_it, ctx);
        }
    }

    /// The library material downloaded for `a` (same folder and name), if any.
    fn online_in_library(&self, a: &Asset) -> Option<varstotas_kernel::Material> {
        self.mat_library.items.iter().find(|i| i.folder == a.source.label() && i.mat.name == a.name).map(|i| i.mat.clone())
    }

    /// Take the results of the background work.
    fn online_poll(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.online.work.rx.try_recv() {
            match msg {
                Msg::Catalogue(src, res) => {
                    self.online.work.busy -= 1;
                    self.online.loading.remove(&src);
                    if let Err(e) = &res {
                        log::warn!("online: {} catalogue: {e}", src.label());
                    }
                    self.online.catalogues.insert(src, res);
                }
                Msg::Thumb(url, img) => {
                    let state = match img {
                        Some((w, h, px)) => {
                            let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &px);
                            ThumbState::Ready(ctx.load_texture(format!("online:{url}"), ci, egui::TextureOptions::LINEAR))
                        }
                        None => ThumbState::Failed,
                    };
                    self.online.thumbs.insert(url, state);
                }
                Msg::Material(key, res, use_it) => {
                    self.online.work.busy -= 1;
                    self.online.downloading.remove(&key);
                    match res {
                        Ok((path, mat)) => {
                            self.online.last = Some(format!("{key} ok {}", path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()));
                            self.refresh_materials();
                            if use_it {
                                self.use_material(&mat);
                            }
                            self.flash(format!("Downloaded {} to the materials library", mat.name));
                        }
                        Err(e) => {
                            self.online.last = Some(format!("{key} error {e}"));
                            self.flash_error(format!("Download failed: {e}"));
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn online_window(&mut self, ctx: &egui::Context) {
        self.online_poll(ctx);
        if !self.online.open {
            return;
        }
        self.online.ensure_catalogue(ctx);
        let mut open = true;
        egui::Window::new("Online Materials").open(&mut open).default_pos([120.0, 180.0]).default_size([640.0, 560.0]).min_width(420.0).collapsible(false).show(ctx, |ui| self.online_ui(ui, ctx));
        self.online.open &= open;
    }

    fn online_ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            for s in Source::ALL {
                if ui.selectable_label(self.online.source == s, s.label()).clicked() && self.online.source != s {
                    self.online.source = s;
                    self.online.category.clear();
                    self.online.ensure_catalogue(ctx);
                }
            }
            ui.separator();
            ui.add(egui::TextEdit::singleline(&mut self.online.query).hint_text("Search: brick, oak, concrete…").desired_width(180.0).id_salt("online_search"));
            let cats: Vec<String> = match self.online.catalogues.get(&self.online.source) {
                Some(Ok(list)) => {
                    let mut c: Vec<String> = list.iter().map(|a| a.category.clone()).filter(|c| !c.is_empty()).collect();
                    c.sort_by_key(|c| c.to_lowercase());
                    c.dedup();
                    c
                }
                _ => Vec::new(),
            };
            let label = if self.online.category.is_empty() { "All categories".to_string() } else { self.online.category.clone() };
            egui::ComboBox::from_id_salt("online_cat").selected_text(label).height(400.0).show_ui(ui, |ui| {
                ui.selectable_value(&mut self.online.category, String::new(), "All categories");
                for c in cats {
                    ui.selectable_value(&mut self.online.category, c.clone(), c);
                }
            });
        });
        ui.label(RichText::new("Free CC0 photo scans (colour, normal, roughness, occlusion, metal maps at real-world size). Downloads go to the materials library.").small().weak());
        ui.separator();
        // Details of the selected asset (bottom), the grid above.
        let sel = self.online.selected.clone().and_then(|k| self.online.asset(&k).cloned());
        egui::Panel::bottom("online_details").resizable(false).min_size(64.0).show(ui, |ui| {
            ui.add_space(4.0);
            match &sel {
                None => {
                    ui.label(RichText::new("Click a material to see it; double-click downloads it and paints with it.").weak());
                }
                Some(a) => {
                    let in_lib = self.online_in_library(a);
                    ui.horizontal(|ui| {
                        if let Some(t) = self.online.thumb(ctx, a) {
                            ui.image((t, egui::vec2(48.0, 48.0)));
                        }
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&a.name).strong());
                            ui.label(RichText::new(format!("{} · {} · {} wide · CC0", a.source.label(), a.category, varstotas_kernel::units::format_length(a.width_mm, self.doc.model.unit))).small().weak());
                            ui.horizontal(|ui| {
                                let key = a.key();
                                if self.online.downloading.contains(&key) {
                                    ui.spinner();
                                    ui.label("Downloading…");
                                } else if let Some(m) = &in_lib {
                                    if ui.button("Use").on_hover_text("Already in the library: add it to the model and paint").clicked() {
                                        self.use_material(m);
                                    }
                                    if ui.button("Download Again").clicked() {
                                        self.online_download(&key, false, ctx);
                                    }
                                } else {
                                    if ui.button("Download & Use").on_hover_text("Save it to the library, add it to the model and paint").clicked() {
                                        self.online_download(&key, true, ctx);
                                    }
                                    if ui.button("Download").on_hover_text("Save it to the materials library").clicked() {
                                        self.online_download(&key, false, ctx);
                                    }
                                }
                                if ui.button("Web Page").clicked() {
                                    open_url(&a.source.page_url(&a.id));
                                }
                            });
                        });
                    });
                }
            }
        });
        let state = self.online.catalogues.get(&self.online.source).map(|r| r.as_ref().map(|l| l.len()).map_err(|e| e.clone()));
        match state {
            None => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(format!("Loading the {} catalogue…", self.online.source.label()));
                });
                return;
            }
            Some(Err(e)) => {
                ui.colored_label(Color32::from_rgb(200, 60, 40), format!("Can't load the catalogue: {e}"));
                if ui.button("Retry").clicked() {
                    self.online.catalogues.remove(&self.online.source);
                }
                return;
            }
            Some(Ok(_)) => {}
        }
        let shown: Vec<Asset> = self.online.shown().into_iter().cloned().collect();
        ui.label(RichText::new(format!("{} materials", shown.len())).small().weak());
        let cols = ((ui.available_width() + 6.0) / (TILE + 6.0)).floor().max(1.0) as usize;
        let rows = shown.len().div_ceil(cols);
        let row_h = TILE + 22.0;
        let mut pick: Option<(String, bool)> = None;
        egui::ScrollArea::vertical().auto_shrink(false).show_rows(ui, row_h, rows, |ui, range| {
            for r in range {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    for a in shown.iter().skip(r * cols).take(cols) {
                        let key = a.key();
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(TILE, row_h - 4.0), egui::Sense::click());
                        let img = egui::Rect::from_min_size(rect.min, egui::vec2(TILE, TILE));
                        let p = ui.painter();
                        p.rect_filled(img, 4.0, Color32::from_gray(60));
                        if let Some(t) = self.online.thumb(ctx, a) {
                            p.image(t, img.shrink(2.0), egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                        }
                        let selected = self.online.selected.as_deref() == Some(key.as_str());
                        if selected || resp.hovered() {
                            let c = if selected { super::theme::ACCENT } else { Color32::from_gray(160) };
                            p.rect_stroke(img, 4.0, egui::Stroke::new(2.0, c), egui::StrokeKind::Inside);
                        }
                        if self.online.downloading.contains(&key) {
                            p.text(img.center(), egui::Align2::CENTER_CENTER, "⏳", egui::FontId::proportional(22.0), Color32::WHITE);
                        } else if self.online_in_library(a).is_some() {
                            p.text(img.right_top() + egui::vec2(-6.0, 4.0), egui::Align2::RIGHT_TOP, "✔", egui::FontId::proportional(14.0), Color32::from_rgb(120, 220, 120));
                        }
                        let name = egui::text::LayoutJob::simple_singleline(a.name.clone(), egui::FontId::proportional(11.0), ui.visuals().text_color());
                        let galley = ui.fonts_mut(|f| f.layout_job(name));
                        let tp = egui::pos2(rect.min.x + ((TILE - galley.size().x) / 2.0).max(0.0), img.max.y + 3.0);
                        ui.painter().with_clip_rect(rect).galley(tp, galley, Color32::WHITE);
                        let resp = resp.on_hover_text(format!("{}\n{} · {} wide", a.name, a.category, varstotas_kernel::units::format_length(a.width_mm, self.doc.model.unit)));
                        if resp.double_clicked() {
                            pick = Some((key.clone(), true));
                        } else if resp.clicked() {
                            pick = Some((key.clone(), false));
                        }
                    }
                });
            }
        });
        if let Some((key, go)) = pick {
            self.online.selected = Some(key.clone());
            if go {
                let in_lib = self.online.asset(&key).cloned().and_then(|a| self.online_in_library(&a));
                match in_lib {
                    Some(m) => {
                        self.use_material(&m);
                    }
                    None => self.online_download(&key, true, ctx),
                }
            }
        }
    }
}
