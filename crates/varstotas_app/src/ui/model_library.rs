//! 3D Library window (Window → 3D Online Library, the cloud button of the Components card): browse free models
//! of Poly Haven, the Smithsonian and NASA (and of `@model_source` plugins), download one into
//! the components library (`<library>/<source>/<name>.vrst`, the credit kept in the
//! component) and place it; a model file URL imports the same way. See `crate::online_models`.

use crate::app::VarstotasApp;
use crate::online_mats::{Msg, Online, PluginSource};
use crate::downloads::{Downloads, LibraryState, SavedAsset};
use crate::online_models::{ModelAsset, ModelSource};
use varstotas_kernel::LicenseKind;
use varstotas_i18n::tr;
use crate::plugins::Plugin;
use crate::ui::online_mats::{ThumbState, open_url, plugin_params_ui};
use egui::{Color32, RichText, Ui};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const TILE: f32 = 96.0;
/// Above this many triangles a model is marked heavy.
const HEAVY: u64 = 100_000;

/// The licence filter of the 3D Library.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum LicFilter {
    #[default]
    Any,
    /// CC0, CC-BY, CC-BY-SA: may be used in work you sell.
    Commercial,
    Cc0,
    /// Hide models whose licence isn't known.
    Known,
}

impl LicFilter {
    pub const ALL: [LicFilter; 4] = [LicFilter::Any, LicFilter::Commercial, LicFilter::Cc0, LicFilter::Known];

    pub fn label(self) -> String {
        varstotas_i18n::tr(match self {
            LicFilter::Any => "model-lib-licence-any",
            LicFilter::Commercial => "model-lib-licence-commercial",
            LicFilter::Cc0 => "model-lib-licence-cc0",
            LicFilter::Known => "model-lib-licence-known",
        })
    }

    pub fn key(self) -> &'static str {
        match self {
            LicFilter::Any => "any",
            LicFilter::Commercial => "commercial",
            LicFilter::Cc0 => "cc0",
            LicFilter::Known => "known",
        }
    }

    fn pass(self, k: LicenseKind) -> bool {
        match self {
            LicFilter::Any => true,
            LicFilter::Commercial => matches!(k, LicenseKind::Cc0 | LicenseKind::CcBy | LicenseKind::CcBySa),
            LicFilter::Cc0 => k == LicenseKind::Cc0,
            LicFilter::Known => k != LicenseKind::Unknown,
        }
    }
}

/// The filters of the shown models (licence, format, triangles).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Filter {
    pub license: LicFilter,
    /// "" = any; else a format the model (or the pack) has.
    pub format: String,
    /// 0 = any; models of unknown size pass.
    pub max_tris: u64,
}

impl Filter {
    pub fn pass(&self, a: &ModelAsset) -> bool {
        self.license.pass(LicenseKind::parse(&a.license))
            && (self.format.is_empty() || a.format.split(" · ").any(|f| f.eq_ignore_ascii_case(&self.format)))
            && (self.max_tris == 0 || a.polycount <= self.max_tris)
    }
}

/// The triangle limits of the size filter and their names.
fn tris_options() -> [(u64, String); 4] {
    [(0, tr!("model-lib-size-any")), (10_000, tr!("model-lib-size-10k")), (50_000, tr!("model-lib-size-50k")), (100_000, tr!("model-lib-size-100k"))]
}

pub struct ModelWindow {
    pub open: bool,
    pub source: ModelSource,
    pub query: String,
    /// "" = every category.
    pub category: String,
    /// Each source's models and the search they answer ("" for whole catalogues).
    pub results: HashMap<ModelSource, (String, Result<Vec<ModelAsset>, String>)>,
    loading: HashSet<ModelSource>,
    thumbs: HashMap<String, ThumbState>,
    /// Asset keys being downloaded, with their cancel flags.
    pub downloading: HashMap<String, Arc<AtomicBool>>,
    /// Cancelled downloads whose answer hasn't come yet.
    cancelled: HashSet<String>,
    pub filter: Filter,
    /// Favourites, the download history, imported files (`<data folder>/library_state.json`).
    pub state: LibraryState,
    /// The Favourites / Downloaded lists are to be made again.
    pub(crate) saved_dirty: bool,
    /// Sites without an API: waiting cards, the Downloads watcher, offers.
    pub dl: Downloads,
    pub selected: Option<String>,
    /// The last finished download: "<key> ok <file> faces=N" / "<key> error <message>" (harness).
    pub last: Option<String>,
    /// Parameter values (text) of the source plugins, by plugin file.
    pub plugin_values: HashMap<PathBuf, Vec<String>>,
    /// Import from URL.
    pub url: String,
    /// Assets of URL imports (by key).
    web: Vec<ModelAsset>,
    pub work: Online,
}

impl Default for ModelWindow {
    fn default() -> Self {
        Self {
            open: false,
            source: ModelSource::PolyHaven,
            query: String::new(),
            category: String::new(),
            results: HashMap::new(),
            loading: HashSet::new(),
            thumbs: HashMap::new(),
            downloading: HashMap::new(),
            cancelled: HashSet::new(),
            filter: Filter::default(),
            state: LibraryState::load(),
            saved_dirty: true,
            dl: Downloads::default(),
            selected: None,
            last: None,
            plugin_values: HashMap::new(),
            url: String::new(),
            web: Vec::new(),
            work: Online::default(),
        }
    }
}

impl ModelWindow {
    pub fn busy(&self) -> bool {
        self.work.busy > 0 || self.dl.importing.is_some()
    }

    pub fn list(&self) -> Option<&Result<Vec<ModelAsset>, String>> {
        self.results.get(&self.source).map(|(_, r)| r)
    }

    /// The models of the current source passing the search (sources that search themselves
    /// answered it already) and the category.
    pub fn shown(&self) -> Vec<&ModelAsset> {
        let local = !self.source.searches();
        match self.list() {
            Some(Ok(list)) => list.iter().filter(|a| (self.category.is_empty() || a.category == self.category) && (!local || a.matches(&self.query)) && self.filter.pass(a)).collect(),
            _ => Vec::new(),
        }
    }

    pub fn asset(&self, key: &str) -> Option<&ModelAsset> {
        self.results.values().filter_map(|(_, r)| r.as_ref().ok()).flatten().chain(&self.web).find(|a| a.key() == key)
    }

    /// Load the current source's models (for a searching source: for the current search) if
    /// they aren't there.
    pub fn ensure(&mut self, ctx: &egui::Context) {
        if matches!(self.source, ModelSource::Favourites | ModelSource::History) {
            return;
        }
        let q = if self.source.searches() { self.query.trim().to_string() } else { String::new() };
        if self.source.searches() && q.is_empty() {
            return;
        }
        let have = self.results.get(&self.source).is_some_and(|(s, _)| *s == q);
        if !have && self.loading.insert(self.source.clone()) {
            self.work.load_models(self.source.clone(), q, ctx);
        }
    }

    fn thumb(&mut self, ctx: &egui::Context, a: &ModelAsset) -> Option<egui::TextureId> {
        match self.thumbs.get(&a.thumb) {
            Some(ThumbState::Ready(t)) => Some(t.id()),
            Some(_) => None,
            None => {
                if !a.thumb.is_empty() {
                    self.thumbs.insert(a.thumb.clone(), ThumbState::Loading);
                    self.work.thumb(&a.thumb, &format!("model_{}", a.key()), ctx);
                }
                None
            }
        }
    }
}


/// "12.3 MB".
fn megabytes(b: u64) -> String {
    tr!("model-lib-megabytes", value = format!("{:.1}", b as f64 / 1_048_576.0))
}

impl VarstotasApp {
    /// The `@model_source` plugins.
    pub(crate) fn model_source_plugins(&self) -> Vec<Plugin> {
        self.plugins.list.iter().filter(|p| p.model_source.is_some()).cloned().collect()
    }

    /// The `@model_import` plugins.
    pub(crate) fn model_importers(&self) -> Vec<Plugin> {
        self.plugins.list.iter().filter(|p| !p.model_import.is_empty()).cloned().collect()
    }

    fn model_plugin_source(&self, p: &Plugin) -> Result<ModelSource, String> {
        let values = self.models.plugin_values.get(&p.path).cloned().unwrap_or_default();
        let mut params = crate::plugins::param_values(p, &values, self.doc.model.unit)?;
        for q in p.params.iter().filter(|q| q.is_path()) {
            if let Some(serde_json::Value::String(v)) = params.get_mut(&q.key)
                && !v.trim().is_empty()
            {
                *v = std::path::absolute(v.trim()).map(|a| a.to_string_lossy().to_string()).unwrap_or_else(|_| v.clone());
            }
        }
        Ok(ModelSource::Plugin(Arc::new(PluginSource { plugin: p.clone(), params })))
    }

    /// Show `source` (its models are asked for again when `reload`).
    pub(crate) fn models_set_source(&mut self, source: ModelSource, reload: bool, ctx: &egui::Context) {
        if self.models.source != source {
            self.models.category.clear();
            self.models.selected = None;
        }
        if reload && !self.models.loading.contains(&source) {
            self.models.results.remove(&source);
        }
        self.models.source = source;
        self.models.ensure(ctx);
    }

    /// Switch to plugin source `key` (its name or `plugin-<file stem>`).
    pub(crate) fn models_plugin_source(&mut self, key: &str, reload: bool, ctx: &egui::Context) -> bool {
        let found = self.model_source_plugins().into_iter().find(|p| {
            p.name.eq_ignore_ascii_case(key.trim()) || p.path.file_stem().is_some_and(|s| format!("plugin-{}", s.to_string_lossy()).eq_ignore_ascii_case(key.trim()))
        });
        let Some(p) = found else { return false };
        match self.model_plugin_source(&p) {
            Ok(s) => self.models_set_source(s, reload, ctx),
            Err(e) => self.flash_error(e),
        }
        true
    }

    /// Search: a searching source asks again, the others filter what they have.
    pub(crate) fn models_search(&mut self, ctx: &egui::Context) {
        if self.models.source.searches() {
            self.models.category.clear();
            self.models.ensure(ctx);
        }
    }

    /// Download model `key` into the components library (and place it when `place`).
    pub(crate) fn models_download(&mut self, key: &str, place: bool, ctx: &egui::Context) {
        let (Some(asset), Some(dir)) = (self.models.asset(key).cloned(), self.library_dir()) else {
            log::warn!("models: no asset {key}");
            return;
        };
        if !self.models.downloading.contains_key(key) {
            let flag = Arc::new(AtomicBool::new(false));
            self.models.downloading.insert(key.to_string(), flag.clone());
            let importers = self.model_importers();
            self.models.work.download_model(asset, dir, importers, place, flag, ctx);
        }
    }

    /// Stop downloading `key` (the download stops; a plugin's own download finishes unseen).
    pub(crate) fn models_cancel(&mut self, key: &str) {
        if let Some(flag) = self.models.downloading.remove(key) {
            self.models.work.progress.remove(key);
            flag.store(true, Ordering::Relaxed);
            self.models.cancelled.insert(key.to_string());
            self.flash(tr!("model-lib-cancelled", name = self.models.asset(key).map(|a| a.name.clone()).unwrap_or_else(|| key.to_string())));
        }
    }

    /// Mark / unmark `a` as a favourite.
    pub(crate) fn models_toggle_favourite(&mut self, a: &ModelAsset) {
        let on = self.models.state.toggle_favourite(a);
        self.models.state.save();
        self.models.saved_dirty = true;
        self.flash(if on { tr!("model-lib-fav-added", name = a.name.as_str()) } else { tr!("model-lib-fav-removed", name = a.name.as_str()) });
    }

    /// The source a saved model's key names now (a plugin by its file stem).
    fn saved_source(&self, key: &str, plugins: &[Plugin]) -> Option<ModelSource> {
        if let Some(s) = ModelSource::from_key(key) {
            return Some(s);
        }
        let stem = key.strip_prefix("plugin-")?;
        let p = plugins.iter().find(|p| p.path.file_stem().is_some_and(|s| s.to_string_lossy() == stem))?;
        self.model_plugin_source(p).ok()
    }

    /// Make the Favourites and Downloaded lists (after a change).
    fn models_refresh_saved(&mut self) {
        if !self.models.saved_dirty {
            return;
        }
        self.models.saved_dirty = false;
        let plugins = self.model_source_plugins();
        let list = |saved: &[SavedAsset]| -> Vec<ModelAsset> { saved.iter().filter_map(|s| Some(s.asset(self.saved_source(&s.source, &plugins)?))).collect() };
        let (fav, hist) = (list(&self.models.state.favourites), list(&self.models.state.history));
        self.models.results.insert(ModelSource::Favourites, (String::new(), Ok(fav)));
        self.models.results.insert(ModelSource::History, (String::new(), Ok(hist)));
    }

    /// The library file / pack folder an earlier download of `a` became (still there).
    fn models_saved_path(&self, a: &ModelAsset) -> Option<PathBuf> {
        let key = a.key();
        self.models.state.history.iter().find(|h| h.key() == key).and_then(|h| h.path.clone()).filter(|p| p.exists())
    }

    /// Download and import the model file at `url` (glb, gltf, fbx, dae, obj, 3mf, ply, stl, dxf, zip…).
    pub(crate) fn models_from_url(&mut self, url: &str, place: bool, ctx: &egui::Context) {
        let url = url.trim();
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            self.flash_error(tr!("model-lib-paste-url"));
            return;
        }
        let a = crate::online_models::url_asset(url);
        let key = a.key();
        if !self.models.web.iter().any(|w| w.key() == key) {
            self.models.web.push(a);
        }
        self.models_download(&key, place, ctx);
    }

    /// The components library item downloaded for `a`, if any (a pack: its first model).
    fn models_in_library(&self, a: &ModelAsset) -> Option<usize> {
        if let Some(p) = self.models_saved_path(a) {
            let found = if p.is_dir() { self.library.items.iter().position(|i| i.path.starts_with(&p)) } else { self.library.items.iter().position(|i| i.path == p) };
            if found.is_some() {
                return found;
            }
        }
        let (folder, name) = (a.source.folder(), crate::files::safe_name(&a.name, crate::files::MODEL));
        if a.pack {
            let sub = std::path::Path::new(&folder).join(&name).to_string_lossy().to_string();
            return self.library.items.iter().position(|i| i.folder == sub);
        }
        self.library.items.iter().position(|i| i.folder == folder && i.name == name)
    }

    /// Show a downloaded pack's folder in the Components card (unfolded).
    pub(crate) fn library_show_folder(&mut self, a: &ModelAsset) {
        let sub = match self.models_in_library(a) {
            Some(i) => self.library.items[i].folder.clone(),
            None => Path::new(&a.source.folder()).join(crate::files::safe_name(&a.name, crate::files::MODEL)).to_string_lossy().to_string(),
        };
        self.show_library_folder(&sub);
    }

    /// Show components library folder `sub` ("Kenney/Car Kit") unfolded.
    pub(crate) fn show_library_folder(&mut self, sub: &str) {
        let top = sub.split(['/', '\\']).next().unwrap_or_default().to_string();
        self.prefs.library_collapsed.retain(|f| f != sub && *f != top);
        self.actions.push(crate::app::Action::ShowComponents);
    }

    /// The models of a downloaded pack in the library.
    fn models_pack_count(&self, a: &ModelAsset) -> usize {
        let Some(i) = self.models_in_library(a) else { return 0 };
        let sub = self.library.items[i].folder.clone();
        self.library.items.iter().filter(|i| i.folder == sub).count()
    }

    /// A pack downloaded in the browser (Quaternius): open its download page (found on the
    /// pack's page, in the background) and wait for the archive in Downloads.
    pub(crate) fn models_download_in_browser(&mut self, a: &ModelAsset) {
        self.downloads_expect(crate::downloads::Expect::from_asset(a));
        let page = a.page.clone();
        std::thread::spawn(move || {
            let link = crate::online_mats::fetch(&page).ok().and_then(|h| crate::online_models::quaternius_download(&String::from_utf8_lossy(&h)));
            open_url(link.as_deref().unwrap_or(&page));
        });
        self.flash(tr!("model-lib-browser-download", name = a.name.as_str()));
    }

    fn models_poll(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.models.work.rx.try_recv() {
            match msg {
                Msg::Models(src, query, res) => {
                    self.models.work.progress.remove(&format!("@catalogue:{}", src.key()));
                    self.models.work.busy -= 1;
                    self.models.loading.remove(&src);
                    if let Err(e) = &res {
                        log::warn!("models: {} catalogue: {e}", src.label());
                    }
                    self.models.results.insert(src, (query, res));
                    // The search changed meanwhile.
                    self.models.ensure(ctx);
                }
                Msg::Thumb(url, img) => {
                    self.models.work.progress.remove(&format!("@thumb:{url}"));
                    let state = match img {
                        Some((w, h, px)) => {
                            let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &px);
                            ThumbState::Ready(ctx.load_texture(format!("models:{url}"), ci, egui::TextureOptions::LINEAR))
                        }
                        None => ThumbState::Failed,
                    };
                    self.models.thumbs.insert(url, state);
                }
                Msg::Model(key, res, place) => {
                    self.models.work.progress.remove(&key);
                    self.models.work.busy -= 1;
                    if self.models.cancelled.remove(&key) {
                        self.models.last = Some(format!("{key} cancelled"));
                        if res.is_ok() {
                            self.refresh_library();
                        }
                        continue;
                    }
                    self.models.downloading.remove(&key);
                    if let (Ok((path, _)), Some(a)) = (&res, self.models.asset(&key)) {
                        let saved = SavedAsset::of(a);
                        self.models.state.downloaded(saved, path.clone());
                        self.models.state.save();
                        self.models.saved_dirty = true;
                    }
                    match res {
                        Ok((path, n)) if path.is_dir() => {
                            self.models.last = Some(format!("{key} ok {} models={n}", path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()));
                            self.refresh_library();
                            let folder = path.parent().and_then(|p| p.file_name()).map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
                            self.flash(tr!("model-lib-downloaded-pack", n = n, folder = folder, pack = path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default()));
                        }
                        Ok((path, faces)) => {
                            self.models.last = Some(format!("{key} ok {} faces={faces}", path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()));
                            self.refresh_library();
                            let name = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                            match self.library.items.iter().position(|i| i.path == path) {
                                Some(i) if place => self.place_component(i),
                                Some(_) => self.flash(tr!("model-lib-downloaded", name = name, faces = faces, folder = path.parent().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default())),
                                None => self.flash_error(tr!("model-lib-not-in-library", path = path.display().to_string())),
                            }
                        }
                        Err(e) => {
                            self.models.last = Some(format!("{key} error {e}"));
                            self.flash_error(tr!("model-lib-download-failed", error = e));
                        }
                    }
                }
                Msg::Catalogue(..) | Msg::Material(..) => {}
            }
        }
    }

    pub(crate) fn models_window(&mut self, ctx: &egui::Context) {
        self.models_poll(ctx);
        if !self.models.open {
            return;
        }
        self.models.ensure(ctx);
        let mut open = true;
        egui::Window::new(tr!("model-lib-title")).id(egui::Id::new("3D Library")).open(&mut open).default_pos([140.0, 160.0]).default_size([660.0, 580.0]).min_width(440.0).collapsible(false).show(ctx, |ui| self.models_ui(ui, ctx));
        self.models.open &= open;
    }

    fn models_ui(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        self.models.work.progress_ui(ui);
        self.models_refresh_saved();
        let plugins = self.model_source_plugins();
        let mut pick: Option<Result<ModelSource, String>> = None;
        let mut search = false;
        ui.horizontal(|ui| {
            ui.label(tr!("model-lib-source"));
            let cur = self.models.source.clone();
            egui::ComboBox::from_id_salt("models_source").selected_text(cur.display()).width(150.0).height(500.0).show_ui(ui, |ui| {
                for s in ModelSource::BUILT_IN {
                    if ui.selectable_label(cur == s, s.display()).clicked() {
                        pick = Some(Ok(s));
                    }
                }
                ui.separator();
                for p in &plugins {
                    let on = matches!(&cur, ModelSource::Plugin(x) if x.plugin.path == p.path);
                    if ui.selectable_label(on, &p.name).on_hover_text(format!("{}\n{}", p.description, p.path.display())).clicked() {
                        pick = Some(self.model_plugin_source(p));
                    }
                }
                if plugins.is_empty() {
                    ui.label(RichText::new(tr!("model-lib-plugins-hint")).small().weak());
                }
                ui.separator();
                for s in [ModelSource::Favourites, ModelSource::History] {
                    let n = match &s {
                        ModelSource::Favourites => self.models.state.favourites.len(),
                        _ => self.models.state.history.len(),
                    };
                    if ui.selectable_label(cur == s, tr!("model-lib-source-count", label = s.display(), n = n)).clicked() {
                        pick = Some(Ok(s));
                    }
                }
            });
            ui.separator();
            let hint = tr!(if cur.searches() { "model-lib-search-enter" } else { "model-lib-search" });
            let r = ui.add(egui::TextEdit::singleline(&mut self.models.query).hint_text(hint).desired_width(180.0).id_salt("models_search"));
            search = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let cats: Vec<String> = match self.models.list() {
                Some(Ok(list)) => {
                    let mut c: Vec<String> = list.iter().map(|a| a.category.clone()).filter(|c| !c.is_empty()).collect();
                    c.sort_by_key(|c| c.to_lowercase());
                    c.dedup();
                    c
                }
                _ => Vec::new(),
            };
            let label = if self.models.category.is_empty() { tr!("model-lib-all-categories") } else { self.models.category.clone() };
            egui::ComboBox::from_id_salt("models_cat").selected_text(label).height(400.0).show_ui(ui, |ui| {
                ui.selectable_value(&mut self.models.category, String::new(), tr!("model-lib-all-categories"));
                for c in cats {
                    ui.selectable_value(&mut self.models.category, c.clone(), c);
                }
            });
        });
        match &pick {
            Some(Ok(s)) if *s != self.models.source => self.models_set_source(s.clone(), false, ctx),
            Some(Err(e)) => self.flash_error(e.clone()),
            _ => {}
        }
        if search {
            self.models_search(ctx);
        }
        let plugin = match &self.models.source {
            ModelSource::Plugin(p) => plugins.iter().find(|x| x.path == p.plugin.path).cloned(),
            _ => None,
        };
        match &plugin {
            Some(p) => self.models_plugin_header(ui, p, ctx),
            None => {
                ui.label(RichText::new(tr!("model-lib-downloads-go", description = self.models.source.description())).small().weak());
            }
        }
        crate::ui::licences::terms_notice(ui, &self.models.source.terms());
        // Import from URL.
        ui.horizontal(|ui| {
            ui.label(tr!("model-lib-from-url"));
            // i18n-ignore-next
            let r = ui.add(egui::TextEdit::singleline(&mut self.models.url).hint_text("https://…/model.glb (gltf, fbx, dae, obj, 3mf, ply, stl, zip…)").desired_width(ui.available_width() - 70.0).id_salt("models_url"));
            let go = (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) | ui.button(tr!("model-lib-import")).on_hover_text(tr!("model-lib-import-hover")).clicked();
            if go && !self.models.url.trim().is_empty() {
                let url = self.models.url.clone();
                self.models_from_url(&url, true, ctx);
            }
        });
        self.models_filters(ui);
        self.downloads_ui(ui);
        ui.separator();
        let sel = self.models.selected.clone().and_then(|k| self.models.asset(&k).cloned());
        egui::Panel::bottom("models_details").resizable(false).min_size(64.0).show(ui, |ui| {
            ui.add_space(4.0);
            match &sel {
                None => {
                    ui.label(RichText::new(tr!("model-lib-click-hint")).weak());
                }
                Some(a) => self.models_details(ui, a, ctx),
            }
        });
        if self.models.source.searches() && self.models.query.trim().is_empty() && !self.models.results.contains_key(&self.models.source) {
            ui.label(RichText::new(tr!("model-lib-type-search")).weak());
            return;
        }
        match self.models.list().map(|r| r.as_ref().map(|l| l.len()).map_err(|e| e.clone())) {
            None => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(tr!("model-lib-loading", source = self.models.source.display()));
                });
                return;
            }
            Some(Err(e)) => {
                ui.colored_label(Color32::from_rgb(200, 60, 40), tr!("model-lib-load-failed", error = e));
                if ui.button(tr!("model-lib-retry")).clicked() {
                    let s = self.models.source.clone();
                    self.models_set_source(s, true, ctx);
                }
                return;
            }
            Some(Ok(_)) => {}
        }
        let loading = self.models.loading.contains(&self.models.source);
        let shown: Vec<ModelAsset> = self.models.shown().into_iter().cloned().collect();
        ui.horizontal(|ui| {
            ui.label(RichText::new(tr!("model-lib-count", n = shown.len())).small().weak());
            if loading {
                ui.spinner();
            }
        });
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
                        match self.models.thumb(ctx, a) {
                            Some(t) => {
                                p.image(t, img.shrink(2.0), egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                            }
                            None => {
                                p.text(img.center(), egui::Align2::CENTER_CENTER, "3D", egui::FontId::proportional(22.0), Color32::from_gray(110));
                            }
                        }
                        let selected = self.models.selected.as_deref() == Some(key.as_str());
                        if selected || resp.hovered() {
                            let c = if selected { super::theme::ACCENT } else { Color32::from_gray(160) };
                            p.rect_stroke(img, 4.0, egui::Stroke::new(2.0, c), egui::StrokeKind::Inside);
                        }
                        if self.models.state.is_favourite(&key) {
                            p.text(img.left_bottom() + egui::vec2(5.0, -4.0), egui::Align2::LEFT_BOTTOM, "★", egui::FontId::proportional(13.0), Color32::from_rgb(250, 210, 80));
                        }
                        let kind = LicenseKind::parse(&a.license);
                        let badge = match kind {
                            LicenseKind::CcByNc => "NC",
                            LicenseKind::CcByNd => "ND",
                            LicenseKind::CcBySa => "SA",
                            LicenseKind::Unknown => "?",
                            _ => "",
                        };
                        if !badge.is_empty() {
                            let r = egui::Rect::from_min_size(img.right_bottom() - egui::vec2(24.0, 16.0), egui::vec2(21.0, 13.0));
                            p.rect_filled(r, 3.0, Color32::from_rgba_unmultiplied(30, 30, 30, 200));
                            p.text(r.center(), egui::Align2::CENTER_CENTER, badge, egui::FontId::proportional(9.0), Color32::from_rgb(240, 180, 60));
                        }
                        if self.models.downloading.contains_key(&key) {
                            p.text(img.center(), egui::Align2::CENTER_CENTER, "⏳", egui::FontId::proportional(22.0), Color32::WHITE);
                        } else if self.models_in_library(a).is_some() {
                            p.text(img.right_top() + egui::vec2(-6.0, 4.0), egui::Align2::RIGHT_TOP, "✔", egui::FontId::proportional(14.0), Color32::from_rgb(120, 220, 120));
                        }
                        if a.polycount > HEAVY {
                            p.text(img.left_top() + egui::vec2(5.0, 4.0), egui::Align2::LEFT_TOP, "⚠", egui::FontId::proportional(13.0), Color32::from_rgb(240, 180, 60));
                        }
                        let name = egui::text::LayoutJob::simple_singleline(a.name.clone(), egui::FontId::proportional(11.0), ui.visuals().text_color());
                        let galley = ui.fonts_mut(|f| f.layout_job(name));
                        let tp = egui::pos2(rect.min.x + ((TILE - galley.size().x) / 2.0).max(0.0), img.max.y + 3.0);
                        ui.painter().with_clip_rect(rect).galley(tp, galley, Color32::WHITE);
                        let resp = resp.on_hover_text(format!("{}\n{}", a.name, [a.category.as_str(), a.author.as_str()].into_iter().filter(|x| !x.is_empty()).collect::<Vec<_>>().join(" · ")));
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
            self.models.selected = Some(key.clone());
            if go && let Some(a) = self.models.asset(&key).cloned() {
                match self.models_in_library(&a) {
                    Some(_) if a.pack => self.library_show_folder(&a),
                    Some(i) => self.place_component(i),
                    None if a.source.manual() => self.models_download_in_browser(&a),
                    None => self.models_download(&key, !a.pack, ctx),
                }
            }
        }
    }

    /// Licence, format and size filters.
    fn models_filters(&mut self, ui: &mut Ui) {
        let mut formats: Vec<String> = match self.models.list() {
            Some(Ok(list)) => list.iter().flat_map(|a| a.format.split(" · ").map(str::to_string).collect::<Vec<_>>()).filter(|f| !f.is_empty()).collect(),
            _ => Vec::new(),
        };
        formats.sort();
        formats.dedup();
        let f = &mut self.models.filter;
        ui.horizontal(|ui| {
            ui.label(tr!("model-lib-show"));
            egui::ComboBox::from_id_salt("models_lic").selected_text(f.license.label()).show_ui(ui, |ui| {
                for l in LicFilter::ALL {
                    ui.selectable_value(&mut f.license, l, l.label());
                }
            });
            let label = if f.format.is_empty() { tr!("model-lib-format-any") } else { f.format.clone() };
            egui::ComboBox::from_id_salt("models_fmt").selected_text(label).show_ui(ui, |ui| {
                ui.selectable_value(&mut f.format, String::new(), tr!("model-lib-format-any"));
                for x in formats {
                    ui.selectable_value(&mut f.format, x.clone(), x);
                }
            });
            let label = tris_options().into_iter().find(|t| t.0 == f.max_tris).map(|t| t.1).unwrap_or_else(|| tr!("model-lib-size-any"));
            egui::ComboBox::from_id_salt("models_tris").selected_text(label).show_ui(ui, |ui| {
                for (n, l) in tris_options() {
                    ui.selectable_value(&mut f.max_tris, n, l);
                }
            });
            if *f != Filter::default() && ui.small_button(tr!("model-lib-clear")).clicked() {
                *f = Filter::default();
            }
        });
    }

    /// The selected model: its credit, size, and the buttons.
    fn models_details(&mut self, ui: &mut Ui, a: &ModelAsset, ctx: &egui::Context) {
        let in_lib = self.models_in_library(a);
        let unit = self.doc.model.unit;
        ui.horizontal(|ui| {
            if let Some(t) = self.models.thumb(ctx, a) {
                ui.image((t, egui::vec2(48.0, 48.0)));
            }
            ui.vertical(|ui| {
                ui.label(RichText::new(&a.name).strong());
                let mut info = vec![a.source.display()];
                info.extend([a.category.clone(), a.author.clone(), a.license.clone(), a.format.clone()].into_iter().filter(|x| !x.is_empty()));
                ui.label(RichText::new(info.join(" · ")).small().weak());
                let mut more = Vec::new();
                if let Some(s) = a.size_mm {
                    more.push(s.iter().map(|v| varstotas_kernel::units::format_length(*v, unit)).collect::<Vec<_>>().join(" × "));
                }
                if a.polycount > 0 {
                    more.push(tr!("model-lib-triangles", n = a.polycount));
                }
                if a.bytes > 0 {
                    more.push(megabytes(a.bytes));
                }
                if !more.is_empty() {
                    ui.label(RichText::new(more.join(" · ")).small().weak());
                }
                if a.polycount > HEAVY {
                    ui.label(RichText::new(tr!("model-lib-heavy")).small().color(Color32::from_rgb(220, 160, 50)));
                }
                let kind = LicenseKind::parse(&a.license);
                if !kind.note().is_empty() {
                    ui.label(RichText::new(format!("{}: {}", kind.label(), kind.note())).small().color(Color32::from_rgb(220, 160, 50)));
                }
                ui.horizontal(|ui| {
                    let key = a.key();
                    if self.models.downloading.contains_key(&key) {
                        ui.spinner();
                        ui.label(tr!("model-lib-downloading"));
                        if ui.button(tr!("cancel")).clicked() {
                            self.models_cancel(&key);
                        }
                    } else if a.pack {
                        if in_lib.is_some() {
                            ui.label(RichText::new(tr!("model-lib-in-components", n = self.models_pack_count(a))).small());
                            if ui.button(tr!("model-lib-show-in-components")).clicked() {
                                self.library_show_folder(a);
                            }
                        }
                        if a.source.manual() {
                            if ui.button(tr!("model-lib-download-browser")).on_hover_text(tr!("model-lib-download-browser-hover")).clicked() {
                                self.models_download_in_browser(a);
                            }
                        } else if ui.button(tr!(if in_lib.is_some() { "model-lib-download-again" } else { "model-lib-download-pack" })).on_hover_text(tr!("model-lib-download-pack-hover")).clicked() {
                            self.models_download(&key, false, ctx);
                        }
                    } else if let Some(i) = in_lib {
                        if ui.button(tr!("library-place")).on_hover_text(tr!("model-lib-place-hover")).clicked() {
                            self.place_component(i);
                        }
                        if !a.file.is_empty() && ui.button(tr!("model-lib-download-again")).clicked() {
                            self.models_download(&key, false, ctx);
                        }
                    } else {
                        if ui.button(tr!("model-lib-download-place")).on_hover_text(tr!("model-lib-download-place-hover")).clicked() {
                            self.models_download(&key, true, ctx);
                        }
                        if ui.button(tr!("model-lib-download")).on_hover_text(tr!("model-lib-download-hover")).clicked() {
                            self.models_download(&key, false, ctx);
                        }
                    }
                    if !a.page.is_empty() && ui.button(tr!("model-lib-web-page")).clicked() {
                        open_url(&a.page);
                    }
                    let fav = self.models.state.is_favourite(&key);
                    if ui.button(if fav { "★" } else { "☆" }).on_hover_text(tr!(if fav { "tray-lib-fav-remove" } else { "tray-lib-fav-add" })).clicked() {
                        self.models_toggle_favourite(a);
                    }
                });
            });
        });
    }

    /// A source plugin's description, its parameters and Load.
    fn models_plugin_header(&mut self, ui: &mut Ui, p: &Plugin, ctx: &egui::Context) {
        if !p.description.is_empty() {
            ui.label(RichText::new(&p.description).small().weak());
        }
        let searches = p.model_source == Some(true);
        if p.params.is_empty() {
            if !searches && ui.small_button(tr!("model-lib-reload")).clicked() {
                self.models_plugin_source(&p.name, true, ctx);
            }
            return;
        }
        let values = self.models.plugin_values.entry(p.path.clone()).or_insert_with(|| p.params.iter().map(|q| q.default.clone()).collect());
        values.resize(p.params.len(), String::new());
        let mut load = false;
        ui.horizontal_wrapped(|ui| {
            load |= plugin_params_ui(ui, "models_param", &p.params, values);
            let tip = tr!(if searches { "model-lib-plugin-apply-hover" } else { "model-lib-plugin-load-hover" });
            load |= ui.button(tr!(if searches { "apply" } else { "model-lib-load" })).on_hover_text(tip).clicked();
        });
        if load {
            self.models_plugin_source(&p.name, true, ctx);
        }
    }

    /// File → Import with the `@model_import` plugin `plugin` (else any file the 3D Library
    /// reads): the model as one component at the origin.
    pub(crate) fn import_model_file(&mut self, path: &std::path::Path, plugin: Option<&Plugin>) {
        let importers = self.model_importers();
        let name = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Model".into()); // i18n-ignore
        let read = match plugin {
            Some(p) => crate::online_models::import_with_plugin(path, p, &importers, 0),
            None => crate::online_models::import_file(path, &importers, 0),
        };
        let clip = read
            .and_then(|(m, xf)| crate::online_models::library_model(m, xf, &name, None))
            .and_then(|m| crate::clipboard::Clipboard::from_library(m, &name).map_err(|e| e.to_string()));
        let clip = match clip {
            Ok(c) => c,
            Err(e) => {
                self.flash_error(tr!("model-lib-import-failed", path = path.display().to_string(), error = e));
                return;
            }
        };
        let (uid, ctx) = (self.doc.uid, self.doc.active_path.clone());
        let t = self.doc.model.transform_at(&ctx);
        let r = self.doc.try_commit(&tr!("model-lib-undo-import", name = name.as_str()), |m| clip.paste_into(m, uid, &ctx, &t));
        if let Some((ents, _)) = r {
            self.doc.set_selection(ents);
            self.flash(tr!("model-lib-imported", path = path.display().to_string()));
            self.actions.push(crate::app::Action::ZoomExtents);
        } else if let Some(e) = self.doc.last_error.clone() {
            self.flash_error(e);
        }
    }
}
