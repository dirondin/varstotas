//! 3D Library → sites without an API (see `crate::downloads`): the «Search on» row, the
//! Other Sites section (Send to Varstotas, the Downloads watcher, waiting cards), the Import
//! Download window for a file that came, and files / links dropped on the window.

use crate::app::VarstotasApp;
use crate::downloads::{Expect, Offer, SavedAsset, SendServer, Watcher, import_offer, search_url, send_token};
use crate::ui::online_mats::open_url;
use egui::{Color32, RichText, Ui};
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
            self.flash_error(format!("{}: no model files in it", p.display()));
            return;
        }
        self.models.dl.offers.push(o);
    }

    /// A file or a link file dropped on the window.
    pub(crate) fn downloads_drop(&mut self, p: &Path, ctx: &egui::Context) {
        let ext = p.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        if ext == "webloc" || ext == "url" {
            let Some(url) = link_file_url(p) else {
                self.flash_error(format!("{}: no web address in it", p.display()));
                return;
            };
            self.downloads_link(&url, ctx);
            return;
        }
        if crate::downloads::is_model_file(p) {
            // Dropped again: offer it again.
            self.models.dl.offered.remove(p);
            self.downloads_offer(p);
        } else {
            self.flash_error(format!("{}: not a model file (glb, gltf, fbx, dae, obj, 3mf, ply, stl, dxf, varstotas, zip)", p.display()));
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
        if let Some(e) = Expect::from_send(&q, &sites) {
            self.flash(format!("Waiting for a file from {}: download it in the browser", e.credit.source));
            self.downloads_expect(e);
            self.models.open = true;
        }
    }

    /// Run every frame: the watcher and the listener on / off, what they found, imports done,
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
        if self.prefs.send_to_varstotas && self.models.dl.send.is_none() && self.models.dl.send_error.is_none() {
            match SendServer::start(send_token(), ctx.clone()) {
                Ok(s) => self.models.dl.send = Some(s),
                Err(e) => self.models.dl.send_error = Some(e.to_string()),
            }
        } else if !self.prefs.send_to_varstotas {
            self.models.dl.send = None;
            self.models.dl.send_error = None;
        }
        let found: Vec<PathBuf> = self.models.dl.watcher.as_ref().map(|w| w.rx.try_iter().collect()).unwrap_or_default();
        for p in found {
            self.downloads_offer(&p);
        }
        let sent: Vec<_> = self.models.dl.send.as_ref().map(|s| s.rx.try_iter().collect()).unwrap_or_default();
        for q in sent {
            self.downloads_sent(&q);
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

    /// What the bookmarklet sent: a waiting card.
    pub(crate) fn downloads_sent(&mut self, q: &std::collections::HashMap<String, String>) {
        let sites = self.models.dl.sites().to_vec();
        match Expect::from_send(q, &sites) {
            Some(e) => {
                self.flash(format!("Waiting for the file of {} ({}): download it in the browser", e.credit.title, e.credit.source));
                self.downloads_expect(e);
                self.models.open = true;
            }
            None => log::warn!("send: no page address"),
        }
    }

    /// Import the first offer (in the background).
    pub(crate) fn downloads_import(&mut self) {
        if self.models.dl.importing.is_some() || self.models.dl.offers.is_empty() {
            return;
        }
        let Some(lib) = self.library_dir() else {
            self.flash_error("No components library folder".to_string());
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
                        _ => self.flash(format!("Imported {name} to Components → {}", folder.trim_end_matches(".vrst"))),
                    }
                } else {
                    self.flash(format!("Imported {n} models of {name} to Components → {folder}"));
                    self.show_library_folder(&folder);
                }
            }
            Err(e) => {
                self.models.dl.last = Some(format!("error {e}"));
                self.flash_error(format!("Import {name}: {e}"));
            }
        }
    }

    /// The «Search on» row and the Other Sites section of the 3D Library.
    pub(crate) fn downloads_ui(&mut self, ui: &mut Ui) {
        let sites = self.models.dl.sites().to_vec();
        let query = self.models.query.clone();
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.label(RichText::new("Search on:").small());
            for s in &sites {
                if ui.small_button(&s.name).on_hover_text(format!("Open {}'s search for «{}» in the browser", s.name, query.trim())).clicked() {
                    open_url(&search_url(s, &query));
                }
            }
            if let Some(Err(e)) = &self.models.dl.sites {
                ui.colored_label(Color32::from_rgb(200, 60, 40), RichText::new(e).small());
            }
            if ui.small_button("Edit…").on_hover_text("Edit the sites (~/.varstotas/sources.toml); the list is read again").clicked() {
                if let Some(p) = crate::downloads::sites_path() {
                    open_url(&p.to_string_lossy());
                }
                self.models.dl.sites = None;
            }
        });
        let waiting = self.models.dl.expects.len();
        let title = if waiting > 0 { format!("Other Sites — waiting for {waiting} download(s)") } else { "Other Sites".to_string() };
        egui::CollapsingHeader::new(title).id_salt("models_other").default_open(waiting > 0).show(ui, |ui| {
            ui.label(RichText::new("Download from any site in your browser: Varstotas doesn't visit the sites itself. It takes the file from your Downloads folder and keeps the page as the credit.").small().weak());
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.prefs.watch_downloads, "Watch Downloads").on_hover_text("Offer every new model file or archive of the Downloads folder for import (always on while a page waits)");
                match &self.models.dl.watcher {
                    Some(w) => {
                        ui.label(RichText::new(format!("watching {}", w.dir.display())).small().weak());
                    }
                    None => {
                        ui.label(RichText::new("off").small().weak());
                    }
                }
                if ui.button("Import File…").on_hover_text("Import a model file or an archive you downloaded").clicked() {
                    let exts = crate::downloads::FILE_EXTS;
                    if let Some(p) = rfd::FileDialog::new().add_filter("Models", &exts).pick_file() {
                        self.models.dl.offered.remove(&p);
                        self.downloads_offer(&p);
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.prefs.send_to_varstotas, "Send to Varstotas").on_hover_text("A bookmarklet: on a model's page, click it to send the page's title, author and licence here");
                if let Some(s) = &self.models.dl.send {
                    ui.label(RichText::new(format!("listening on 127.0.0.1:{}", s.port)).small().weak());
                    let port = s.port;
                    if ui.button("Copy Bookmarklet").on_hover_text("Make a bookmark in the browser and paste this as its address").clicked() {
                        ui.ctx().copy_text(crate::downloads::bookmarklet(port, &send_token()));
                        self.flash("Bookmarklet copied: make a bookmark and paste it as the address");
                    }
                }
                if let Some(e) = &self.models.dl.send_error {
                    ui.colored_label(Color32::from_rgb(200, 60, 40), RichText::new(e).small());
                }
            });
            let mut close = None;
            for e in &self.models.dl.expects {
                ui.horizontal(|ui| {
                    ui.spinner();
                    let lic = if e.credit.license.is_empty() { "licence unknown" } else { &e.credit.license };
                    ui.label(RichText::new(format!("Waiting for a file from this page: {} ({}, {lic})", e.credit.title, e.credit.source)).small()).on_hover_text(&e.credit.url);
                    if ui.small_button("Page").clicked() {
                        open_url(&e.credit.url);
                    }
                    for (i, f) in e.files.iter().enumerate().take(4) {
                        let n = crate::online_models::url_file(f);
                        if ui.small_button(format!("⬇ {}", if n.is_empty() { (i + 1).to_string() } else { n })).on_hover_text(format!("Download in the browser: {f}")).clicked() {
                            open_url(f);
                        }
                    }
                    if ui.small_button("✖").on_hover_text("Stop waiting").clicked() {
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
        egui::Window::new("Import Download").id(egui::Id::new("import_download")).default_pos([220.0, 180.0]).default_width(440.0).collapsible(false).resizable(false).show(ctx, |ui| {
            if self.models.dl.importing.is_some() {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Importing…");
                });
                return;
            }
            let more = self.models.dl.offers.len() - 1;
            let o = &mut self.models.dl.offers[0];
            let name = o.path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let size = std::fs::metadata(&o.path).map(|m| m.len()).unwrap_or(0);
            ui.label(RichText::new(format!("{name}  ({:.1} MB)", size as f64 / 1_048_576.0)).strong());
            if o.expect.is_some() {
                ui.label(RichText::new("✔ The credit is from the page you sent").small().weak());
            }
            if let Some(d) = &o.duplicate {
                ui.horizontal(|ui| {
                    ui.colored_label(WARN, RichText::new("⚠ The same file was imported before").small());
                    if ui.small_button("Show").clicked() {
                        show = Some(d.clone());
                    }
                });
            }
            egui::Grid::new("offer_credit").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                let c = &mut o.credit;
                for (label, v) in [("Title", &mut c.title), ("Author", &mut c.author), ("Site", &mut c.source), ("Page", &mut c.url)] {
                    ui.label(label);
                    ui.add(egui::TextEdit::singleline(v).desired_width(300.0).id_salt(("offer", label)));
                    ui.end_row();
                }
                ui.label("Licence");
                ui.horizontal(|ui| {
                    let before = o.kind;
                    egui::ComboBox::from_id_salt("offer_lic").selected_text(o.kind.label()).show_ui(ui, |ui| {
                        for k in LicenseKind::ALL {
                            ui.selectable_value(&mut o.kind, k, k.label());
                        }
                    });
                    if o.kind != before {
                        o.credit.license = if o.kind == LicenseKind::Unknown { String::new() } else { o.kind.label().to_string() };
                    }
                    if o.credit.license_kind() != o.kind {
                        o.kind = o.credit.license_kind();
                    }
                    ui.add(egui::TextEdit::singleline(&mut o.credit.license).desired_width(140.0).hint_text("as the page says").id_salt("offer_lic_text"));
                });
                ui.end_row();
            });
            if !o.kind.note().is_empty() {
                ui.colored_label(WARN, RichText::new(format!("{}: {}", o.kind.label(), o.kind.note())).small());
            }
            ui.separator();
            if o.models.len() == 1 {
                ui.checkbox(&mut o.place, "Place it in the model");
            } else {
                let on = o.models.iter().filter(|m| m.1).count();
                ui.horizontal(|ui| {
                    ui.label(format!("{on} of {} models (each becomes a component)", o.models.len()));
                    if ui.small_button("All").clicked() {
                        o.models.iter_mut().for_each(|m| m.1 = true);
                    }
                    if ui.small_button("None").clicked() {
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
                let s = if c.source.trim().is_empty() { "Downloads" } else { c.source.trim() };
                format!("Components → {s}")
            };
            ui.label(RichText::new(dest(&o.credit)).small().weak());
            ui.horizontal(|ui| {
                let can = lib.is_some() && o.models.iter().any(|m| m.1);
                if ui.add_enabled(can, egui::Button::new("Import")).clicked() {
                    act = Some(true);
                }
                if ui.button("Ignore").clicked() {
                    act = Some(false);
                }
                if more > 0 {
                    ui.label(RichText::new(format!("{more} more waiting")).small().weak());
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
