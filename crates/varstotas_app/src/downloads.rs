//! Sites without an API, done "semi-automatically": everything on the site is done by the
//! user in their own browser. Varstotas never reads the site's pages or downloads from it by
//! itself; it removes the routine before and after:
//!
//! - **Search on sites**: `~/.varstotas/sources.toml` keeps each site's own search address
//!   (`{query}` is replaced); a click opens the site's results in the browser.
//! - **A page link dropped on the window** (or a pack downloaded in the browser): Varstotas
//!   shows a *waiting card* with the page as the credit.
//! - **Downloads watcher** (while a card waits, or always when switched on): a new model
//!   file or archive in the Downloads folder is offered for import, its credit taken from
//!   the card whose page the file came from (`kMDItemWhereFroms` on macOS, the
//!   Zone.Identifier stream on Windows).
//! - A file dropped on the window is offered the same way.
//!
//! The import itself (`ImportJob`): the file is copied to `~/.varstotas/downloads/manual/`,
//! an archive is unpacked, the chosen models become library components with the credit
//! (one component, or a pack folder for several), duplicates are found by the file's hash.

use varstotas_i18n::tr;
use varstotas_kernel::{Attribution, LicenseKind};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Model files and archives the watcher offers.
pub const FILE_EXTS: [&str; 11] = ["glb", "gltf", "fbx", "dae", "obj", "3mf", "ply", "stl", "dxf", "vrst", "zip"];

// ---- Sites (sources.toml) ---------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Site {
    pub name: String,
    /// The site's search page; `{query}` is replaced by the search (URL-encoded).
    pub search: String,
    /// The licence the site's free models usually have (prefills the card; "" = varies).
    #[serde(default)]
    pub license: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SitesFile {
    #[serde(default, rename = "site")]
    sites: Vec<Site>,
}

const SITES_HEADER: &str = "# Sites searched by Window → 3D Online Library → Search on.\n# `{query}` is replaced by the search. Edit, remove or add sites; Varstotas reads the file when the 3D Library opens.\n\n";

pub fn default_sites() -> Vec<Site> {
    let s = |name: &str, search: &str, license: &str| Site { name: name.into(), search: search.into(), license: license.into() };
    vec![
        s("Printables", "https://www.printables.com/search/models?q={query}", ""), // i18n-ignore
        s("Thingiverse", "https://www.thingiverse.com/search?q={query}&type=things&sort=relevant", ""), // i18n-ignore
        s("MyMiniFactory", "https://www.myminifactory.com/search/?query={query}&price=free", ""), // i18n-ignore
        s("Cults3D", "https://cults3d.com/en/search?q={query}&only_free=true", ""), // i18n-ignore
        s("GrabCAD", "https://grabcad.com/library?query={query}", "Standard / personal use"), // i18n-ignore
        s("Sketchfab", "https://sketchfab.com/search?features=downloadable&q={query}&type=models", ""), // i18n-ignore
        s("Poly Pizza", "https://poly.pizza/search/{query}", ""), // i18n-ignore
        s("Free3D", "https://free3d.com/3d-models/{query}", "Standard / personal use"), // i18n-ignore
        s("CGTrader (free)", "https://www.cgtrader.com/free-3d-models?keywords={query}", "Royalty Free"), // i18n-ignore
        s("TurboSquid (free)", "https://www.turbosquid.com/Search/3D-Models/free/{query}", "Royalty Free"), // i18n-ignore
    ]
}

/// `~/.varstotas/sources.toml`.
pub fn sites_path() -> Option<PathBuf> {
    crate::bridge::home_dir().map(|d| d.join("sources.toml"))
}

/// The sites of `sources.toml` (written with the defaults when missing).
pub fn load_sites() -> Result<Vec<Site>, String> {
    let Some(p) = sites_path() else { return Ok(default_sites()) };
    match std::fs::read_to_string(&p) {
        Ok(t) => toml::from_str::<SitesFile>(&t).map(|f| f.sites).map_err(|e| format!("{}: {e}", p.display())),
        Err(_) => {
            let sites = default_sites();
            let body = toml::to_string(&SitesFile { sites: sites.clone() }).unwrap_or_default();
            if let Some(d) = p.parent() {
                let _ = std::fs::create_dir_all(d);
            }
            let _ = std::fs::write(&p, format!("{SITES_HEADER}{body}"));
            Ok(sites)
        }
    }
}

/// Percent-encode a search query.
pub fn url_encode(s: &str) -> String {
    s.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect() // i18n-ignore
}

/// The address of `site`'s search for `query`.
pub fn search_url(site: &Site, query: &str) -> String {
    site.search.replace("{query}", &url_encode(query.trim()))
}

// ---- Waiting cards ---------------------------------------------------------------------

/// A page the user is downloading from: the credit for the file that comes.
#[derive(Clone, Debug, PartialEq)]
pub struct Expect {
    pub id: u64,
    pub credit: Attribution,
    /// Links to model files found on the page (opened in the browser on a click).
    pub files: Vec<String>,
    pub since: SystemTime,
}

fn host(url: &str) -> String {
    let rest = url.split("://").nth(1).unwrap_or(url);
    rest.split(['/', '?', '#']).next().unwrap_or("").trim_start_matches("www.").to_string()
}

/// A site's name for an address: a known site's name, else its host.
pub fn site_name(url: &str, sites: &[Site]) -> String {
    let h = host(url);
    sites.iter().find(|s| !h.is_empty() && host(&s.search) == h).map(|s| s.name.clone()).unwrap_or(h)
}

static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl Expect {
    pub fn new(credit: Attribution, files: Vec<String>) -> Self {
        Self { id: NEXT_ID.fetch_add(1, Ordering::Relaxed), credit, files, since: SystemTime::now() }
    }

    /// A pack of a catalogue downloaded in the browser (Quaternius).
    pub fn from_asset(a: &crate::online_models::ModelAsset) -> Self {
        Self::new(a.attribution(), Vec::new())
    }

    /// A page: its `url` and, when known, `title`, `author`, `license` and `files` lines (a
    /// page link dropped on the window; the harness).
    pub fn from_page(q: &HashMap<String, String>, sites: &[Site]) -> Option<Self> {
        let url = q.get("url").filter(|u| u.starts_with("http"))?.clone();
        let site = sites.iter().find(|s| host(&s.search) == host(&url));
        let mut license = q.get("license").cloned().unwrap_or_default();
        if license.trim().is_empty() {
            license = site.map(|s| s.license.clone()).unwrap_or_default();
        }
        let title = q.get("title").map(|t| clean_title(t)).filter(|t| !t.is_empty()).unwrap_or_else(|| host(&url));
        let credit = Attribution { source: site_name(&url, sites), title, author: q.get("author").cloned().unwrap_or_default().trim().to_string(), license: license.trim().to_string(), url };
        let files = q.get("files").map(|f| f.lines().map(str::trim).filter(|l| l.starts_with("http")).map(str::to_string).take(30).collect()).unwrap_or_default();
        Some(Self::new(credit, files))
    }
}

/// "Chair by Bob | Printables" → "Chair by Bob" (the site's suffix dropped).
fn clean_title(t: &str) -> String {
    let t = t.trim();
    let cut = [" | ", " - ", " – ", " · "].iter().filter_map(|s| t.rfind(s)).max().filter(|i| *i > 3);
    match cut {
        Some(i) if t.len() - i < 40 => t[..i].trim().to_string(),
        _ => t.to_string(),
    }
}

/// The card a downloaded file belongs to: the one whose page (or its site) the file came
/// from, else the newest of the last hour.
pub fn match_expect<'a>(expects: &'a [Expect], from: &[String]) -> Option<&'a Expect> {
    let by_page = expects.iter().rev().find(|e| from.iter().any(|f| *f == e.credit.url || e.files.contains(f)));
    let by_host = || expects.iter().rev().find(|e| from.iter().any(|f| !host(f).is_empty() && host(f) == host(&e.credit.url)));
    let recent = || expects.iter().rev().find(|e| e.since.elapsed().is_ok_and(|d| d < Duration::from_secs(3600)));
    by_page.or_else(by_host).or_else(recent)
}

// ---- Downloaded files ------------------------------------------------------------------

/// The Downloads folder (`VARSTOTAS_DOWNLOADS` overrides it, for the harness).
pub fn downloads_folder() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("VARSTOTAS_DOWNLOADS") {
        return Some(PathBuf::from(d));
    }
    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(|h| PathBuf::from(h).join("Downloads"))
}

fn ext(p: &Path) -> String {
    p.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default()
}

/// A finished model file or archive (not a browser's partial download).
pub fn is_model_file(p: &Path) -> bool {
    let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    !name.starts_with('.') && FILE_EXTS.contains(&ext(p).as_str())
}

/// Where a downloaded file came from: its download address and the page (macOS Spotlight
/// metadata, Windows Zone.Identifier); empty when unknown.
pub fn where_froms(p: &Path) -> Vec<String> {
    if cfg!(target_os = "macos") {
        let out = std::process::Command::new("mdls").args(["-raw", "-name", "kMDItemWhereFroms"]).arg(p).output();
        let Ok(out) = out else { return Vec::new() };
        return parse_mdls_list(&String::from_utf8_lossy(&out.stdout));
    }
    if cfg!(windows) {
        let ads = format!("{}:Zone.Identifier", p.display());
        if let Ok(t) = std::fs::read_to_string(ads) {
            return t.lines().filter_map(|l| l.strip_prefix("ReferrerUrl=").or(l.strip_prefix("HostUrl="))).map(|s| s.trim().to_string()).collect();
        }
    }
    Vec::new()
}

/// `mdls -raw` of a list: `(\n    "https://…",\n    "https://…"\n)` (or `(null)`).
pub fn parse_mdls_list(s: &str) -> Vec<String> {
    s.split('"').skip(1).step_by(2).filter(|x| x.starts_with("http")).map(str::to_string).collect()
}

/// A stable hash of a file's bytes (duplicates).
pub fn file_hash(p: &Path) -> Option<u64> {
    let b = std::fs::read(p).ok()?;
    Some(b.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, x| (h ^ *x as u64).wrapping_mul(0x100_0000_01b3)) ^ b.len() as u64)
}

/// The model files of an archive (its names), or the file itself.
pub fn archive_models(p: &Path) -> Vec<String> {
    if ext(p) != "zip" {
        return vec![p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()];
    }
    let Ok(data) = std::fs::read(p) else { return Vec::new() };
    let mut names: Vec<String> = Vec::new();
    // Only the names: read entries without inflating them is not possible here; the archive
    // is small enough to read whole (a model download).
    if let Ok(list) = varstotas_kernel::io::unzip(&data, |n| {
        let e = n.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
        !n.contains("__MACOSX") && FILE_EXTS[..FILE_EXTS.len() - 1].contains(&e.as_str())
    }) {
        names = list.into_iter().map(|(n, _)| n).collect();
    }
    names.sort();
    names
}

/// The Downloads watcher: a thread listing the folder every 1.5 s; a new model file whose
/// size held still for one round is sent once.
pub struct Watcher {
    pub dir: PathBuf,
    pub rx: Receiver<PathBuf>,
    stop: Arc<AtomicBool>,
}

impl Watcher {
    pub fn start(dir: PathBuf, ctx: egui::Context) -> Self {
        let (tx, rx) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let (d, s) = (dir.clone(), stop.clone());
        std::thread::Builder::new()
            .name("downloads".into())
            .spawn(move || {
                let list = |d: &Path| -> HashMap<PathBuf, u64> {
                    std::fs::read_dir(d).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.is_file()).map(|p| {
                        let n = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                        (p, n)
                    }).collect()
                };
                // What is there already is not new.
                let mut seen: HashSet<PathBuf> = list(&d).into_keys().collect();
                let mut sizes: HashMap<PathBuf, u64> = HashMap::new();
                while !s.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(1500));
                    for (p, n) in list(&d) {
                        if seen.contains(&p) {
                            continue;
                        }
                        if !is_model_file(&p) {
                            seen.insert(p);
                            continue;
                        }
                        // Ready when the size held still (and isn't 0).
                        if n > 0 && sizes.get(&p) == Some(&n) {
                            seen.insert(p.clone());
                            sizes.remove(&p);
                            if tx.send(p).is_err() {
                                return;
                            }
                            ctx.request_repaint();
                        } else {
                            sizes.insert(p, n);
                        }
                    }
                }
            })
            .ok();
        Self { dir, rx, stop }
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

// ---- Queries ---------------------------------------------------------------------------

/// A `?a=1&b=2` query (percent-decoded, `+` as space).
pub fn parse_query(q: &str) -> HashMap<String, String> {
    fn dec(s: &str) -> String {
        let b = s.as_bytes();
        let mut out = Vec::with_capacity(b.len());
        let mut i = 0;
        while i < b.len() {
            match b[i] {
                b'+' => out.push(b' '),
                b'%' if i + 2 < b.len() => {
                    match u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or("zz"), 16) {
                        Ok(v) => {
                            out.push(v);
                            i += 2;
                        }
                        Err(_) => out.push(b'%'),
                    }
                }
                c => out.push(c),
            }
            i += 1;
        }
        String::from_utf8_lossy(&out).to_string()
    }
    q.trim_start_matches('?').split('&').filter_map(|kv| kv.split_once('=').map(|(k, v)| (dec(k), dec(v)))).collect()
}

// ---- The import of a downloaded file --------------------------------------------------

/// A file offered for import, with its credit (editable before Import).
#[derive(Clone, Debug)]
pub struct Offer {
    pub path: PathBuf,
    pub credit: Attribution,
    /// The licence as chosen in the list (what `credit.license` reads as).
    pub kind: LicenseKind,
    /// The model files (an archive's entries; one for a model file), and which to import.
    pub models: Vec<(String, bool)>,
    /// The card it was matched to (closed after the import).
    pub expect: Option<u64>,
    /// Already imported (same bytes): its library file / folder.
    pub duplicate: Option<PathBuf>,
    pub hash: Option<u64>,
    /// Place it in the model after the import (one model).
    pub place: bool,
}

/// `~/.varstotas/downloads/manual/<hash>/`: where an offer is copied and unpacked.
pub fn manual_dir(hash: u64) -> Option<PathBuf> {
    crate::online_models::downloads_dir().map(|d| d.join("manual").join(format!("{hash:016x}")))
}

/// Import `o` into the library `lib` (copied to and unpacked in the folder `dir`, see
/// [`manual_dir`]): one model → `<lib>/<source>/<title>.vrst`, several → the pack folder
/// `<lib>/<source>/<title>/`. The path made and the models saved.
pub fn import_offer(o: &Offer, lib: &Path, dir: &Path, importers: &[crate::plugins::Plugin]) -> Result<(PathBuf, usize), String> {
    use crate::ui::mat_library::safe_name;
    let dir = dir.to_path_buf();
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let name = o.path.file_name().ok_or_else(|| tr!("downloads-err-no-file-name"))?;
    let copy = dir.join(name);
    std::fs::copy(&o.path, &copy).map_err(|e| format!("{}: {e}", o.path.display()))?;
    let source = safe_name(if o.credit.source.trim().is_empty() { "Downloads" } else { o.credit.source.trim() }); // i18n-ignore
    let stem = o.path.file_stem().map(|s| crate::online_models::pretty_name(&s.to_string_lossy())).unwrap_or_default();
    let title = safe_name(if o.credit.title.trim().is_empty() { &stem } else { o.credit.title.trim() });
    let chosen: Vec<&String> = o.models.iter().filter(|(_, on)| *on).map(|(n, _)| n).collect();
    if chosen.is_empty() {
        return Err(tr!("downloads-err-no-model-chosen"));
    }
    let files = if ext(&copy) == "zip" {
        let out = dir.join("files");
        let data = std::fs::read(&copy).map_err(|e| e.to_string())?;
        for (n, bytes) in varstotas_kernel::io::unzip(&data, |_| true)? {
            let Some(rel) = safe_rel(&n) else { continue };
            let p = out.join(rel);
            if let Some(d) = p.parent() {
                let _ = std::fs::create_dir_all(d);
            }
            std::fs::write(&p, bytes).map_err(|e| format!("{}: {e}", p.display()))?;
        }
        // Only the chosen models (the rest of the archive stays: textures, previews).
        for (n, on) in &o.models {
            if !*on && let Some(rel) = safe_rel(n) {
                let _ = std::fs::remove_file(out.join(rel));
            }
        }
        out
    } else {
        dir.clone()
    };
    if chosen.len() == 1 {
        let f = match safe_rel(chosen[0]) {
            Some(rel) if ext(&copy) == "zip" => files.join(rel),
            _ => copy.clone(),
        };
        let (m, xf) = crate::online_models::import_file(&f, importers, 0)?;
        let mut m = crate::online_models::library_model(m, xf, &title, Some(o.credit.clone()))?;
        let stem = f.file_stem().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default();
        let pic = varstotas_script::list_files(&files, "").into_iter().map(PathBuf::from).find(|p| {
            ["png", "jpg", "jpeg"].contains(&ext(p).as_str()) && p.file_stem().is_some_and(|s| s.to_string_lossy().to_lowercase() == stem)
        });
        m.set_preview(pic.and_then(|p| std::fs::read(p).ok()).and_then(|b| crate::textures::preview_bytes(&b).ok()));
        let path = lib.join(&source).join(format!("{title}.vrst"));
        if let Some(d) = path.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        varstotas_kernel::io::save(&m, &path).map_err(|e| e.to_string())?;
        return Ok((path, 1));
    }
    let out = lib.join(&source).join(&title);
    let n = crate::online_models::import_pack(&files, &out, &o.credit, importers)?;
    Ok((out, n))
}

/// A relative archive path made safe (no `..`, not absolute).
fn safe_rel(p: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = p.split(['/', '\\']).filter(|s| !s.is_empty() && *s != ".").collect();
    (!parts.is_empty() && !parts.contains(&"..") && !parts[0].contains(':')).then(|| parts.iter().collect())
}

// ---- Licences --------------------------------------------------------------------------

/// The reminder an export or a placement of models with these credits gets: non-commercial,
/// share-alike, no-derivatives, a site's own (royalty-free, personal use, usage conditions)
/// or unknown licences (None when all are CC0 / CC-BY).
pub fn licence_warning(credits: &[Attribution]) -> Option<String> {
    let mut by: Vec<(LicenseKind, Vec<&str>)> = Vec::new();
    for c in credits {
        let k = c.license_kind();
        if matches!(k, LicenseKind::Cc0 | LicenseKind::CcBy) {
            continue;
        }
        match by.iter_mut().find(|(x, _)| *x == k) {
            Some((_, t)) => t.push(&c.title),
            None => by.push((k, vec![&c.title])),
        }
    }
    if by.is_empty() {
        return None;
    }
    by.sort_by_key(|(k, _)| LicenseKind::ALL.iter().position(|x| x == k));
    let parts: Vec<String> = by
        .iter()
        .map(|(k, t)| {
            let more = if t.len() > 2 { format!(" +{}", t.len() - 2) } else { String::new() };
            tr!("downloads-licence-part", label = k.label(), names = t[..t.len().min(2)].join(", "), more = more, note = k.note())
        })
        .collect();
    Some(tr!("downloads-licences", list = parts.join("; ")))
}

// ---- Favourites, history, imported files ---------------------------------------------

/// A model of the 3D Library kept between runs (favourites, the download history).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SavedAsset {
    /// `ModelSource::key` (`polyhaven`, `plugin-<stem>`, `web`, `manual`).
    pub source: String,
    pub id: String,
    pub name: String,
    pub category: String,
    pub keywords: String,
    pub author: String,
    pub license: String,
    pub thumb: String,
    pub page: String,
    pub polycount: u64,
    pub size_mm: Option<[f64; 3]>,
    pub bytes: u64,
    pub file: String,
    pub pack: bool,
    pub format: String,
    /// When it was downloaded (seconds since 1970).
    pub when: u64,
    /// The library file / pack folder it became.
    pub path: Option<PathBuf>,
}

pub fn now_secs() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

impl SavedAsset {
    pub fn of(a: &crate::online_models::ModelAsset) -> Self {
        Self {
            source: a.source.key(),
            id: a.id.clone(),
            name: a.name.clone(),
            category: a.category.clone(),
            keywords: a.keywords.clone(),
            author: a.author.clone(),
            license: a.license.clone(),
            thumb: a.thumb.clone(),
            page: a.page.clone(),
            polycount: a.polycount,
            size_mm: a.size_mm,
            bytes: a.bytes,
            file: a.file.clone(),
            pack: a.pack,
            format: a.format.clone(),
            when: now_secs(),
            path: None,
        }
    }

    /// The model again, on `source` (what its key names now).
    pub fn asset(&self, source: crate::online_models::ModelSource) -> crate::online_models::ModelAsset {
        crate::online_models::ModelAsset {
            source,
            id: self.id.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            keywords: self.keywords.clone(),
            author: self.author.clone(),
            license: self.license.clone(),
            thumb: self.thumb.clone(),
            page: self.page.clone(),
            polycount: self.polycount,
            size_mm: self.size_mm,
            bytes: self.bytes,
            file: self.file.clone(),
            pack: self.pack,
            format: self.format.clone(),
        }
    }

    pub fn key(&self) -> String {
        format!("{}:{}", self.source, self.id)
    }
}

/// `~/.varstotas/library_state.json`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LibraryState {
    pub favourites: Vec<SavedAsset>,
    /// Newest first.
    pub history: Vec<SavedAsset>,
    /// Imported downloads by the hash of their bytes (hex) → the library file / folder.
    pub imported: HashMap<String, PathBuf>,
}

const HISTORY: usize = 300;

impl LibraryState {
    fn path() -> Option<PathBuf> {
        crate::bridge::home_dir().map(|d| d.join("library_state.json"))
    }

    pub fn load() -> Self {
        Self::path().and_then(|p| std::fs::read(p).ok()).and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
    }

    pub fn save(&self) {
        if let Some(p) = Self::path() {
            if let Some(d) = p.parent() {
                let _ = std::fs::create_dir_all(d);
            }
            if let Ok(b) = serde_json::to_vec_pretty(self) {
                let _ = std::fs::write(p, b);
            }
        }
    }

    pub fn is_favourite(&self, key: &str) -> bool {
        self.favourites.iter().any(|f| f.key() == key)
    }

    /// Add or remove `a` from the favourites; whether it is one now.
    pub fn toggle_favourite(&mut self, a: &crate::online_models::ModelAsset) -> bool {
        let key = a.key();
        if self.is_favourite(&key) {
            self.favourites.retain(|f| f.key() != key);
            false
        } else {
            self.favourites.push(SavedAsset::of(a));
            true
        }
    }

    /// A finished download (moved to the top of the history).
    pub fn downloaded(&mut self, mut s: SavedAsset, path: PathBuf) {
        let key = s.key();
        self.history.retain(|h| h.key() != key);
        s.when = now_secs();
        s.path = Some(path);
        self.history.insert(0, s);
        self.history.truncate(HISTORY);
    }

    /// The library file / folder of an earlier import of the same bytes (if still there).
    pub fn duplicate(&self, hash: u64) -> Option<PathBuf> {
        self.imported.get(&format!("{hash:016x}")).filter(|p| p.exists()).cloned()
    }
}

/// An offer and what its import made (the library path and the models saved).
pub type Imported = (Offer, Result<(PathBuf, usize), String>);

/// The state of the semi-automatic downloads (kept by the 3D Library window, run every frame).
#[derive(Default)]
pub struct Downloads {
    pub expects: Vec<Expect>,
    /// Files waiting for Import / Ignore (the first is shown).
    pub offers: Vec<Offer>,
    pub watcher: Option<Watcher>,
    /// `sources.toml` (read when the 3D Library opens; None = not yet).
    pub sites: Option<Result<Vec<Site>, String>>,
    pub importing: Option<Receiver<Imported>>,
    /// The last finished import: "ok <name> models=N" / "error <message>" (harness).
    pub last: Option<String>,
    /// Files offered once (not again when seen another way).
    pub offered: HashSet<PathBuf>,
    /// The folder watched instead of Downloads (harness `dl_watch`).
    pub dir: Option<PathBuf>,
}

impl Downloads {
    pub fn sites(&mut self) -> &[Site] {
        if self.sites.is_none() {
            self.sites = Some(load_sites());
        }
        match &self.sites {
            Some(Ok(s)) => s,
            _ => &[],
        }
    }

    /// The offer for the file at `path` (its credit from the matching card, else from where
    /// it came from).
    pub fn offer(&mut self, path: &Path, state: &LibraryState) -> Offer {
        let from = where_froms(path);
        let card = match_expect(&self.expects, &from).cloned();
        let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let credit = match &card {
            Some(c) => c.credit.clone(),
            None => {
                let page = from.last().cloned().unwrap_or_default();
                let sites = self.sites().to_vec();
                let site = if page.is_empty() { "Downloads".to_string() } else { site_name(&page, &sites) }; // i18n-ignore
                let license = sites.iter().find(|s| s.name == site).map(|s| s.license.clone()).unwrap_or_default();
                Attribution { source: site, title: crate::online_models::pretty_name(&stem), author: String::new(), license, url: page }
            }
        };
        let hash = file_hash(path);
        let models = archive_models(path);
        let single = models.len() == 1;
        Offer {
            path: path.to_path_buf(),
            kind: credit.license_kind(),
            credit,
            models: models.into_iter().map(|m| (m, true)).collect(),
            expect: card.map(|c| c.id),
            duplicate: hash.and_then(|h| state.duplicate(h)),
            hash,
            place: single,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_titles_and_matching() {
        let q = parse_query("t=abc&url=https%3A%2F%2Fwww.printables.com%2Fmodel%2F12-chair&title=Chair+by+Bob+%7C+Printables&license=CC+BY-NC+4.0&files=https%3A%2F%2Fx.org%2Fa.stl%0Ahttps%3A%2F%2Fx.org%2Fb.zip");
        assert_eq!(q["t"], "abc");
        let sites = default_sites();
        let e = Expect::from_page(&q, &sites).unwrap();
        assert_eq!(e.credit.source, "Printables");
        assert_eq!(e.credit.title, "Chair by Bob");
        assert_eq!(e.credit.license_kind(), LicenseKind::CcByNc);
        assert_eq!(e.files, ["https://x.org/a.stl", "https://x.org/b.zip"]);
        // A site's default licence when the page had none.
        let q2 = parse_query("url=https://free3d.com/3d-model/lamp-1.html&title=Lamp");
        let e2 = Expect::from_page(&q2, &sites).unwrap();
        assert_eq!(e2.credit.license_kind(), LicenseKind::Standard);
        // The file from the first card's page goes to it; an unknown one to the newest card.
        let cards = vec![e.clone(), e2.clone()];
        assert_eq!(match_expect(&cards, &["https://x.org/a.stl".into()]).unwrap().id, e.id);
        assert_eq!(match_expect(&cards, &["https://www.printables.com/dl/9".into()]).unwrap().id, e.id);
        assert_eq!(match_expect(&cards, &[]).unwrap().id, e2.id);
        assert!(Expect::from_page(&parse_query("title=x"), &sites).is_none());
        assert_eq!(search_url(&sites[0], "red chair"), "https://www.printables.com/search/models?q=red%20chair");
        assert_eq!(parse_mdls_list("(\n    \"https://a.org/f.zip\",\n    \"https://a.org/page\"\n)"), ["https://a.org/f.zip", "https://a.org/page"]);
        assert!(parse_mdls_list("(null)").is_empty());
        assert!(is_model_file(Path::new("/d/chair.STL")) && !is_model_file(Path::new("/d/chair.stl.crdownload")) && !is_model_file(Path::new("/d/.x.glb")));
    }

    /// A dropped archive: its models listed, a pack of the chosen ones with the card's credit;
    /// one model file → one component.
    #[test]
    fn offers_import_archives_and_files() {
        let d = std::env::temp_dir().join(format!("varstotas_dl_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let tri = b"v 0 0 0\nv 1000 0 0\nv 0 1000 0\nf 1 2 3\n";
        let quad = b"v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n";
        let zip = crate::online_mats::tests::zip(&[("chairs/a.obj", tri, true), ("chairs/b.obj", quad, false), ("chairs/c.stl.txt", b"x", false), ("__MACOSX/chairs/._a.obj", b"x", false)]);
        let zp = d.join("chairs.zip");
        std::fs::write(&zp, zip).unwrap();
        assert_eq!(archive_models(&zp), ["chairs/a.obj", "chairs/b.obj"]);
        let mut dl = Downloads { sites: Some(Ok(default_sites())), ..Default::default() };
        dl.expects.push(Expect::new(Attribution { source: "Printables".into(), title: "Two Chairs".into(), author: "Bob".into(), license: "CC BY 4.0".into(), url: "https://www.printables.com/model/1".into() }, Vec::new()));
        let state = LibraryState::default();
        let mut o = dl.offer(&zp, &state);
        assert_eq!((o.credit.title.as_str(), o.kind, o.models.len(), o.place), ("Two Chairs", LicenseKind::CcBy, 2, false));
        assert!(o.expect.is_some() && o.duplicate.is_none());
        let lib = d.join("lib");
        assert_eq!(import_offer(&o, &lib, &d.join("work"), &[]).unwrap(), (lib.join("Printables/Two Chairs"), 2));
        let a = varstotas_kernel::io::load(&lib.join("Printables/Two Chairs/A.vrst")).unwrap();
        assert_eq!(a.attributions().first().map(|c| (c.title.clone(), c.author.clone())), Some(("Two Chairs: A".into(), "Bob".into())));
        // Only one chosen: one component named after the credit.
        o.models[0].1 = false;
        assert_eq!(import_offer(&o, &lib, &d.join("work"), &[]).unwrap(), (lib.join("Printables/Two Chairs.vrst"), 1));
        // A plain model file, no card: the credit from its name; a duplicate once recorded.
        let f = d.join("red_stool.obj");
        std::fs::write(&f, tri).unwrap();
        dl.expects.clear();
        let mut st = LibraryState::default();
        let o = dl.offer(&f, &st);
        assert_eq!((o.credit.title.as_str(), o.credit.source.as_str(), o.kind, o.place), ("Red Stool", "Downloads", LicenseKind::Unknown, true));
        let (p, n) = import_offer(&o, &lib, &d.join("work"), &[]).unwrap();
        assert_eq!((p.clone(), n), (lib.join("Downloads/Red Stool.vrst"), 1));
        st.imported.insert(format!("{:016x}", o.hash.unwrap()), p);
        assert!(dl.offer(&f, &st).duplicate.is_some());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn licence_warnings_and_saved_assets() {
        let c = |t: &str, l: &str| Attribution { title: t.into(), license: l.into(), ..Default::default() };
        assert_eq!(licence_warning(&[c("Rock", "CC0"), c("Lamp", "CC-BY 4.0")]), None);
        assert!(licence_warning(&[c("Desk", "Royalty Free")]).unwrap().contains("Standard / personal use (Desk)"));
        let w = licence_warning(&[c("Chair", "CC BY-NC 4.0"), c("Bust", ""), c("Car", "CC-BY-NC")]).unwrap();
        assert!(w.contains("CC-BY-NC (Chair, Car)") && w.contains("Unknown (Bust)") && w.find("NC").unwrap() < w.find("Unknown").unwrap(), "{w}");
        let a = crate::online_models::url_asset("https://x.org/m/chair.glb");
        let mut st = LibraryState::default();
        assert!(st.toggle_favourite(&a) && st.is_favourite(&a.key()));
        assert!(!st.toggle_favourite(&a) && st.favourites.is_empty());
        st.downloaded(SavedAsset::of(&a), PathBuf::from("/lib/Web/chair.vrst"));
        st.downloaded(SavedAsset::of(&a), PathBuf::from("/lib/Web/chair 2.vrst"));
        assert_eq!(st.history.len(), 1);
        let back = st.history[0].asset(crate::online_models::ModelSource::Web);
        assert_eq!((back.key(), back.format.as_str()), (a.key(), "GLB"));
        let json = serde_json::to_string(&st).unwrap();
        let st2: LibraryState = serde_json::from_str(&json).unwrap();
        assert_eq!(st2.history, st.history);
    }

    /// The watcher: files there before are not new; a new model file is sent once it is
    /// complete, other files never.
    #[test]
    fn watcher_finds_new_model_files() {
        let d = std::env::temp_dir().join(format!("varstotas_watch_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("old.stl"), b"x").unwrap();
        let w = Watcher::start(d.clone(), egui::Context::default());
        std::thread::sleep(Duration::from_millis(300));
        std::fs::write(d.join("notes.txt"), b"x").unwrap();
        std::fs::write(d.join("chair.glb.crdownload"), b"x").unwrap();
        std::fs::write(d.join("chair.glb"), b"glTF").unwrap();
        let got = w.rx.recv_timeout(Duration::from_secs(8)).unwrap();
        assert_eq!(got, d.join("chair.glb"));
        assert!(w.rx.recv_timeout(Duration::from_secs(4)).is_err());
        drop(w);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn licence_kinds() {
        for (t, k) in [
            ("CC0", LicenseKind::Cc0),
            ("Creative Commons Zero", LicenseKind::Cc0),
            ("Public Domain", LicenseKind::Cc0),
            ("CC-BY 4.0", LicenseKind::CcBy),
            ("Creative Commons - Attribution - Share Alike", LicenseKind::CcBySa),
            ("CC BY-NC-SA 4.0", LicenseKind::CcByNc),
            ("CC-BY-ND", LicenseKind::CcByNd),
            ("Royalty Free", LicenseKind::Standard),
            ("NASA media guidelines", LicenseKind::Standard),
            ("Smithsonian Open Access: check the usage", LicenseKind::Standard),
            ("", LicenseKind::Unknown),
            ("All rights reserved", LicenseKind::Unknown),
        ] {
            assert_eq!(LicenseKind::parse(t), k, "{t}");
        }
    }
}
