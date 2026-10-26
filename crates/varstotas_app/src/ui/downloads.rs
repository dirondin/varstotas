//! 3D Library → sites without an API (see `crate::downloads`): the «Search on» row, the
//! Other Sites section (the Downloads watcher, waiting cards), the Import
//! Download window for a file that came, and files / links dropped on the window.

use crate::app::VarstotasApp;
use crate::downloads::{Expect, Offer, SavedAsset, Watcher, import_offer, search_url};
use crate::ui::online_mats::open_url;
use egui::{Color32, RichText, Ui};
use varstotas_i18n::tr;
use varstotas_kernel::{Attribution, LicenseKind};
use std::path::{Path, PathBuf};

const WARN: Color32 = Color32::from_rgb(220, 160, 50);

/// The address in a dropped `.webloc` (macOS) / `.url` (Windows) link file.
pub fn link_file_url(p: &Path) -> Option<String> {
    let t = std::fs::read_to_string(p).ok()?;
    let ext = p.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    let url = if ext == "webloc" {
        t.split("<string>").nth(1)?.split("</string>").next()?.trim().replace("&amp;", "&")
    } else {
        t.lines().find_map(|l| l.trim().strip_prefix("URL="))?.trim().to_string()
    };
    url.starts_with("http").then_some(url)
}

impl VarstotasApp {
    /// A page the user downloads from in the browser: wait for its file.
    pub(crate) fn downloads_expect(&mut self, e: Expect) {
        let dl = &mut self.models.dl;
        dl.expects.retain(|x| x.credit.url != e.credit.url);
        dl.expects.push(e);
        let n = dl.expects.len();
        if n > 20 {
            dl.expects.drain(..n - 20);
        }
    }

    /// Offer the downloaded file at `p` for import (once).
    pub(crate) fn downloads_offer(&mut self, p: &Path) {
        if !self.models.dl.offered.insert(p.to_path_buf()) {
            return;
        }
        let o = self.models.dl.offer(p, &self.models.state);
        if o.models.is_empty() {
            self.flash_error(tr!("downloads-no-models", path = p.display().to_string()));
            return;
        }
        self.models.dl.offers.push(o);
    }

    /// A file or a link file dropped on the window.
    pub(crate) fn downloads_drop(&mut self, p: &Path, ctx: &egui::Context) {
        let ext = p.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        if ext == "webloc" || ext == "url" {
            let Some(url) = link_file_url(p) else {
                self.flash_error(tr!("downloads-no-address", path = p.display().to_string()));
                return;
            };
            self.downloads_link(&url, ctx);
            return;
        }
        if crate::textures::is_image_file(p) {
            let local = ctx.input(|i| i.pointer.latest_pos()).map(|q| (q - self.viewport.rect.min).to_pos2());
            self.drop_image(p, local);
            return;
        }
        if crate::downloads::is_model_file(p) {
            // Dropped again: offer it again.
            self.models.dl.offered.remove(p);
            self.downloads_offer(p);
        } else {
            self.flash_error(tr!("downloads-not-model", path = p.display().to_string()));
        }
    }

    /// A dropped link: a model file is imported from it; a page becomes a waiting card.
    pub(crate) fn downloads_link(&mut self, url: &str, ctx: &egui::Context) {
        if !crate::online_models::format_of(&crate::online_models::url_file(url)).is_empty() {
            self.models_from_url(url, true, ctx);
            return;
        }
        let sites = self.models.dl.sites().to_vec();
        let mut q = std::collections::HashMap::new();
        q.insert("url".to_string(), url.to_string());
        if let Some(e) = Expect::from_page(&q, &sites) {
            self.flash(tr!("downloads-waiting-site", site = e.credit.source.as_str()));
            self.downloads_expect(e);
            self.models.open = true;
        }
    }

    /// Run every frame: the watcher on / off, what it found, imports done,
    /// dropped files, the Import Download window.
    pub(crate) fn downloads_tick(&mut self, ctx: &egui::Context) {
        let want = self.prefs.watch_downloads || !self.models.dl.expects.is_empty();
        let dir = self.models.dl.dir.clone().or_else(|| self.prefs.downloads_watch_dir.clone()).or_else(crate::downloads::downloads_folder);
        match (&self.models.dl.watcher, want, dir) {
            (Some(w), true, Some(d)) if w.dir == d => {}
            (_, true, Some(d)) if d.is_dir() => self.models.dl.watcher = Some(Watcher::start(d, ctx.clone())),
            (Some(_), false, _) => self.models.dl.watcher = None,
            _ => {}
        }
        let found: Vec<PathBuf> = self.models.dl.watcher.as_ref().map(|w| w.rx.try_iter().collect()).unwrap_or_default();
        for p in found {
            self.downloads_offer(&p);
        }
        let done = self.models.dl.importing.as_ref().and_then(|r| r.try_recv().ok());
        if let Some((o, res)) = done {
            self.models.dl.importing = None;
            self.downloads_imported(o, res);
        }
        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| !p.as_os_str().is_empty()).collect());
        for p in dropped {
            self.downloads_drop(&p, ctx);
        }
        self.import_offer_window(ctx);
    }

    /// A page (address, title, author, licence): a waiting card (harness `dl_page`).
    pub(crate) fn downloads_page(&mut self, q: &std::collections::HashMap<String, String>) {
        let sites = self.models.dl.sites().to_vec();
        match Expect::from_page(q, &sites) {
            Some(e) => {
                self.flash(tr!("downloads-waiting-file", title = e.credit.title.as_str(), site = e.credit.source.as_str()));
                self.downloads_expect(e);
                self.models.open = true;
            }
            None => log::warn!("dl_page: no page address"),
        }
    }

    /// Import the first offer (in the background).
    pub(crate) fn downloads_import(&mut self) {
        if self.models.dl.importing.is_some() || self.models.dl.offers.is_empty() {
            return;
        }
        let Some(lib) = self.library_dir() else {
            self.flash_error(tr!("downloads-no-library"));
            return;
        };
        let o = self.models.dl.offers.remove(0);
        let Some(work) = crate::downloads::manual_dir(o.hash.unwrap_or(0)) else { return };
        let importers = self.model_importers();
        let (tx, rx) = std::sync::mpsc::channel();
        self.models.dl.importing = Some(rx);
        std::thread::spawn(move || {
            let res = import_offer(&o, &lib, &work, &importers);
            let _ = tx.send((o, res));
        });
    }

    fn downloads_imported(&mut self, o: Offer, res: Result<(PathBuf, usize), String>) {
        let name = o.path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        match res {
            Ok((path, n)) => {
                self.models.dl.last = Some(format!("ok {} models={n}", path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()));
                self.refresh_library();
                let hash = o.hash.unwrap_or(0);
                let st = &mut self.models.state;
                st.imported.insert(format!("{hash:016x}"), path.clone());
                let c = &o.credit;
                let saved = SavedAsset {
                    source: "web".into(),
                    id: format!("{hash:016x}"),
                    name: if c.title.is_empty() { name.clone() } else { c.title.clone() },
                    keywords: format!("{} {name}", c.source),
                    author: c.author.clone(),
                    license: c.license.clone(),
                    page: c.url.clone(),
                    category: c.source.clone(),
                    pack: n > 1,
                    format: crate::online_models::format_of(&name),
                    ..Default::default()
                };
                st.downloaded(saved, path.clone());
                st.save();
                self.models.saved_dirty = true;
                if let Some(id) = o.expect {
                    self.models.dl.expects.retain(|e| e.id != id);
                }
                let folder = path.strip_prefix(self.library_dir().unwrap_or_default()).map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                if n == 1 {
                    let i = self.library.items.iter().position(|i| i.path == path);
                    match i {
                        Some(i) if o.place => self.place_component(i),
                        _ => self.flash(tr!("downloads-imported", name = name.as_str(), folder = folder.trim_end_matches(".vrst"))),
                    }
                } else {
                    self.flash(tr!("downloads-imported-many", n = n, name = name.as_str(), folder = folder.as_str()));
                    self.show_library_folder(&folder);
                }
            }
            Err(e) => {
                self.models.dl.last = Some(format!("error {e}"));
                self.flash_error(tr!("downloads-import-failed", name = name.as_str(), error = e));
            }
        }
    }

    /// The «Search on» row and the Other Sites section of the 3D Library.
    pub(crate) fn downloads_ui(&mut self, ui: &mut Ui) {
        let sites = self.models.dl.sites().to_vec();
        let query = self.models.query.clone();
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.label(RichText::new(tr!("downloads-search-on")).small());
            for s in &sites {
                if ui.small_button(&s.name).on_hover_text(tr!("downloads-search-hover", site = s.name.as_str(), query = query.trim())).clicked() {
                    open_url(&search_url(s, &query));
                }
            }
            if let Some(Err(e)) = &self.models.dl.sites {
                ui.colored_label(Color32::from_rgb(200, 60, 40), RichText::new(e).small());
            }
            if ui.small_button(tr!("downloads-edit")).on_hover_text(tr!("downloads-edit-hover")).clicked() {
                if let Some(p) = crate::downloads::sites_path() {
                    open_url(&p.to_string_lossy());
                }
                self.models.dl.sites = None;
            }
        });
        let waiting = self.models.dl.expects.len();
        let title = if waiting > 0 { tr!("downloads-other-waiting", waiting = waiting) } else { tr!("downloads-other") };
        egui::CollapsingHeader::new(title).id_salt("models_other").default_open(waiting > 0).show(ui, |ui| {
            ui.label(RichText::new(tr!("downloads-other-text")).small().weak());
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.prefs.watch_downloads, tr!("downloads-watch")).on_hover_text(tr!("downloads-watch-hover"));
                match &self.models.dl.watcher {
                    Some(w) => {
                        ui.label(RichText::new(tr!("downloads-watching", dir = w.dir.display().to_string())).small().weak());
                    }
                    None => {
                        ui.label(RichText::new(tr!("downloads-off")).small().weak());
                    }
                }
                if ui.button(tr!("downloads-import-file")).on_hover_text(tr!("downloads-import-file-hover")).clicked() {
                    let exts = crate::downloads::FILE_EXTS;
                    if let Some(p) = rfd::FileDialog::new().add_filter(tr!("downloads-models-filter"), &exts).pick_file() {
                        self.models.dl.offered.remove(&p);
                        self.downloads_offer(&p);
                    }
                }
            });
            let mut close = None;
            for e in &self.models.dl.expects {
                ui.horizontal(|ui| {
                    ui.spinner();
                    let lic = if e.credit.license.is_empty() { tr!("tray-info-licence-unknown") } else { e.credit.license.clone() };
                    ui.label(RichText::new(tr!("downloads-waiting-card", title = e.credit.title.as_str(), site = e.credit.source.as_str(), licence = lic)).small()).on_hover_text(&e.credit.url);
                    if ui.small_button(tr!("downloads-page")).clicked() {
                        open_url(&e.credit.url);
                    }
                    for (i, f) in e.files.iter().enumerate().take(4) {
                        let n = crate::online_models::url_file(f);
                        if ui.small_button(format!("⬇ {}", if n.is_empty() { (i + 1).to_string() } else { n })).on_hover_text(tr!("downloads-browser-file", file = f.as_str())).clicked() {
                            open_url(f);
                        }
                    }
                    if ui.small_button("✖").on_hover_text(tr!("downloads-stop")).clicked() {
                        close = Some(e.id);
                    }
                });
            }
            if let Some(id) = close {
                self.models.dl.expects.retain(|e| e.id != id);
            }
        });
    }

    /// The first offered file: its credit (editable), the models to take, Import / Ignore.
    fn import_offer_window(&mut self, ctx: &egui::Context) {
        if self.models.dl.offers.is_empty() && self.models.dl.importing.is_none() {
            return;
        }
        let lib = self.library_dir();
        let mut act: Option<bool> = None;
        let mut show: Option<PathBuf> = None;
        egui::Window::new(tr!("downloads-import-title")).id(egui::Id::new("import_download")).default_pos([220.0, 180.0]).default_width(440.0).collapsible(false).resizable(false).show(ctx, |ui| {
            if self.models.dl.importing.is_some() {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(tr!("downloads-importing"));
                });
                return;
            }
            let more = self.models.dl.offers.len() - 1;
            let o = &mut self.models.dl.offers[0];
            let name = o.path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let size = std::fs::metadata(&o.path).map(|m| m.len()).unwrap_or(0);
            ui.label(RichText::new(tr!("downloads-file-size", name = name.as_str(), mb = format!("{:.1}", size as f64 / 1_048_576.0))).strong());
            if o.expect.is_some() {
                ui.label(RichText::new(tr!("downloads-credit-from-page")).small().weak());
            }
            if let Some(d) = &o.duplicate {
                ui.horizontal(|ui| {
                    ui.colored_label(WARN, RichText::new(tr!("downloads-duplicate")).small());
                    if ui.small_button(tr!("model-lib-show")).clicked() {
                        show = Some(d.clone());
                    }
                });
            }
            egui::Grid::new("offer_credit").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                let c = &mut o.credit;
                for (label, v) in [("Title", &mut c.title), ("Author", &mut c.author), ("Site", &mut c.source), ("Page", &mut c.url)] { // i18n-ignore
                    ui.label(tr!(match label {
                        "Title" => "downloads-field-title", // i18n-ignore
                        "Author" => "downloads-field-author", // i18n-ignore
                        "Site" => "downloads-field-site", // i18n-ignore
                        _ => "downloads-page",
                    }));
                    ui.add(egui::TextEdit::singleline(v).desired_width(300.0).id_salt(("offer", label)));
                    ui.end_row();
                }
                ui.label(tr!("downloads-licence"));
                ui.horizontal(|ui| {
                    let before = o.kind;
                    egui::ComboBox::from_id_salt("offer_lic").selected_text(o.kind.label()).show_ui(ui, |ui| {
                        for k in LicenseKind::ALL {
                            ui.selectable_value(&mut o.kind, k, k.label());
                        }
                    });
                    if o.kind != before {
                        o.credit.license = if o.kind == LicenseKind::Unknown { String::new() } else { o.kind.label() };
                    }
                    if o.credit.license_kind() != o.kind {
                        o.kind = o.credit.license_kind();
                    }
                    ui.add(egui::TextEdit::singleline(&mut o.credit.license).desired_width(140.0).hint_text(tr!("downloads-licence-hint")).id_salt("offer_lic_text"));
                });
                ui.end_row();
            });
            if !o.kind.note().is_empty() {
                ui.colored_label(WARN, RichText::new(format!("{}: {}", o.kind.label(), o.kind.note())).small());
            }
            ui.separator();
            if o.models.len() == 1 {
                ui.checkbox(&mut o.place, tr!("downloads-place"));
            } else {
                let on = o.models.iter().filter(|m| m.1).count();
                ui.horizontal(|ui| {
                    ui.label(tr!("downloads-models-count", on = on, n = o.models.len()));
                    if ui.small_button(tr!("downloads-all")).clicked() {
                        o.models.iter_mut().for_each(|m| m.1 = true);
                    }
                    if ui.small_button(tr!("downloads-none")).clicked() {
                        o.models.iter_mut().for_each(|m| m.1 = false);
                    }
                });
                egui::ScrollArea::vertical().max_height(160.0).id_salt("offer_models").show(ui, |ui| {
                    for (n, take) in o.models.iter_mut() {
                        ui.checkbox(take, RichText::new(n.as_str()).small());
                    }
                });
            }
            let dest = |c: &Attribution| {
                let s = if c.source.trim().is_empty() { "Downloads" } else { c.source.trim() }; // i18n-ignore
                tr!("downloads-destination", folder = s)
            };
            ui.label(RichText::new(dest(&o.credit)).small().weak());
            ui.horizontal(|ui| {
                let can = lib.is_some() && o.models.iter().any(|m| m.1);
                if ui.add_enabled(can, egui::Button::new(tr!("model-lib-import"))).clicked() {
                    act = Some(true);
                }
                if ui.button(tr!("downloads-ignore")).clicked() {
                    act = Some(false);
                }
                if more > 0 {
                    ui.label(RichText::new(tr!("downloads-more-waiting", n = more)).small().weak());
                }
            });
        });
        if let Some(p) = show {
            let lib = lib.unwrap_or_default();
            let rel = p.strip_prefix(&lib).unwrap_or(&p);
            let folder = if p.is_dir() { rel.to_path_buf() } else { rel.parent().map(Path::to_path_buf).unwrap_or_default() };
            self.show_library_folder(&folder.to_string_lossy());
        }
        match act {
            Some(true) => self.downloads_import(),
            Some(false) => {
                self.models.dl.offers.remove(0);
            }
            None => {}
        }
    }
}
