//! 3D Library: free 3D models from the web into the components library. Built-in sources:
//! Poly Haven (CC0 models, glTF with 1K textures), Smithsonian Open Access 3D (CC0, searched
//! on the server), NASA 3D Resources (GLB files of the NASA GitHub repository); plugins add
//! more (`@model_source`, see `docs/PLUGINS.md`). A download goes to
//! `~/.varstotas/downloads/<source>/<id>/`, is imported (glTF / OBJ / STL / DXF / `.vrst`, or
//! a `@model_import` plugin), wrapped into one component that carries its [`Attribution`]
//! and saved as `<components library>/<source>/<name>.vrst`.
//!
//! A model spec (what a plugin's `download` / `import_model` answers, JSON):
//!
//! ```text
//! { "file": "chair.glb" }  /  { "url": "https://…/chair.glb" }  /  { "zip": "chair.zip" }
//! { "meshes": [{ "name": "Box", "vertices": [[0,0,0], …], "faces": [[0,1,2,3], …],
//!                "material": "Oak" }], "materials": [{ "name": "Oak", "color": "#c8a064" }] }
//! + "unit": "m" | "cm" | "mm" | "in" | "ft"  (or "scale": 1000), "up": "y"
//! + "preview": "preview.jpg" | "https://…/preview.png"  (else the asset's `thumb`)
//! ```
//!
//! The saved file keeps the preview picture ([`Model::preview`]), which the Components card
//! shows instead of the drawn model.

use crate::mat_plugins::call;
use crate::online_mats::{Msg, Online, PluginSource, cached, fetch, file_safe, unzip};
use crate::plugins::Plugin;
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::{Attribution, EntityRef, Model};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug)]
pub enum ModelSource {
    PolyHaven,
    Smithsonian,
    Nasa,
    /// Import from URL (not in the Source list).
    Web,
    Plugin(Arc<PluginSource>),
}

impl PartialEq for ModelSource {
    fn eq(&self, o: &Self) -> bool {
        match (self, o) {
            (ModelSource::Plugin(a), ModelSource::Plugin(b)) => a.plugin.path == b.plugin.path,
            _ => std::mem::discriminant(self) == std::mem::discriminant(o),
        }
    }
}

impl Eq for ModelSource {}

impl std::hash::Hash for ModelSource {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        std::mem::discriminant(self).hash(h);
        if let ModelSource::Plugin(p) = self {
            p.plugin.path.hash(h);
        }
    }
}

impl ModelSource {
    pub const BUILT_IN: [ModelSource; 3] = [ModelSource::PolyHaven, ModelSource::Smithsonian, ModelSource::Nasa];

    pub fn label(&self) -> &str {
        match self {
            ModelSource::PolyHaven => "Poly Haven",
            ModelSource::Smithsonian => "Smithsonian",
            ModelSource::Nasa => "NASA",
            ModelSource::Web => "Web",
            ModelSource::Plugin(p) => &p.plugin.name,
        }
    }

    /// Short name (harness, cache folders): `polyhaven`, …, `plugin-<file stem>`.
    pub fn key(&self) -> String {
        match self {
            ModelSource::PolyHaven => "polyhaven".into(),
            ModelSource::Smithsonian => "smithsonian".into(),
            ModelSource::Nasa => "nasa".into(),
            ModelSource::Web => "web".into(),
            ModelSource::Plugin(p) => format!("plugin-{}", p.plugin.path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()),
        }
    }

    pub fn from_key(s: &str) -> Option<ModelSource> {
        ModelSource::BUILT_IN.into_iter().find(|x| x.key().eq_ignore_ascii_case(s.trim()))
    }

    /// The components library folder of its downloads.
    pub fn folder(&self) -> String {
        crate::ui::mat_library::safe_name(self.label())
    }

    /// The source searches itself (the query goes to it on Enter; no local filtering).
    pub fn searches(&self) -> bool {
        match self {
            ModelSource::Smithsonian => true,
            ModelSource::Plugin(p) => p.plugin.model_source == Some(true),
            _ => false,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            ModelSource::PolyHaven => "Free CC0 models (glTF, 1K textures) at their real size.",
            ModelSource::Smithsonian => "Smithsonian Open Access 3D (CC0): scans of museum objects. Type a search and press Enter.",
            ModelSource::Nasa => "NASA 3D Resources: spacecraft, rovers, planets (GLB; free for non-commercial and educational use, see NASA media guidelines).",
            ModelSource::Web => "",
            ModelSource::Plugin(p) => &p.plugin.description,
        }
    }
}

/// One model of a catalogue.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelAsset {
    pub source: ModelSource,
    pub id: String,
    pub name: String,
    pub category: String,
    pub keywords: String,
    pub author: String,
    pub license: String,
    /// Thumbnail: a URL or a local file.
    pub thumb: String,
    pub page: String,
    /// Triangles (0 = unknown).
    pub polycount: u64,
    /// Real size (mm), when the catalogue tells.
    pub size_mm: Option<[f64; 3]>,
    /// Download size in bytes (0 = unknown).
    pub bytes: u64,
    /// What to download (a file URL; source specific).
    pub file: String,
}

impl ModelAsset {
    pub fn key(&self) -> String {
        format!("{}:{}", self.source.key(), self.id)
    }

    pub fn matches(&self, query: &str) -> bool {
        let hay = format!("{} {} {} {}", self.name, self.category, self.keywords, self.author).to_lowercase();
        query.to_lowercase().split_whitespace().all(|w| hay.contains(w))
    }

    pub fn attribution(&self) -> Attribution {
        Attribution {
            source: self.source.label().to_string(),
            title: self.name.clone(),
            author: self.author.clone(),
            license: self.license.clone(),
            url: if self.page.is_empty() { self.file.clone() } else { self.page.clone() },
        }
    }

    fn blank(source: ModelSource, id: String) -> Self {
        Self {
            source,
            name: id.clone(),
            id,
            category: String::new(),
            keywords: String::new(),
            author: String::new(),
            license: String::new(),
            thumb: String::new(),
            page: String::new(),
            polycount: 0,
            size_mm: None,
            bytes: 0,
            file: String::new(),
        }
    }
}

/// Percent-encode a URL path (keeps `/`).
fn encode_path(p: &str) -> String {
    p.bytes()
        .map(|b| if b.is_ascii_alphanumeric() || b"/-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") })
        .collect()
}

/// Poly Haven's `assets?t=models` (most downloaded first).
pub fn parse_polyhaven(json: &[u8]) -> Result<Vec<ModelAsset>, String> {
    let v: Value = serde_json::from_slice(json).map_err(|e| e.to_string())?;
    let obj = v.as_object().ok_or("not a catalogue")?;
    let mut out: Vec<(i64, ModelAsset)> = obj
        .iter()
        .map(|(id, a)| {
            let strs = |k: &str| a[k].as_array().map(|l| l.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>()).unwrap_or_default();
            let cats = strs("categories");
            let category = a["category"].as_str().and_then(|c| c.split('/').next()).map(str::to_string).or_else(|| cats.first().map(|c| c.to_string())).unwrap_or_default();
            let dims: Vec<f64> = a["dimensions"].as_array().map(|d| d.iter().filter_map(Value::as_f64).collect()).unwrap_or_default();
            let mut m = ModelAsset::blank(ModelSource::PolyHaven, id.clone());
            m.name = a["name"].as_str().unwrap_or(id).to_string();
            m.category = category;
            m.keywords = format!("{} {} {id}", strs("tags").join(" "), cats.join(" "));
            m.author = a["authors"].as_object().map(|o| o.keys().cloned().collect::<Vec<_>>().join(", ")).unwrap_or_default();
            m.license = "CC0".into();
            m.thumb = format!("https://cdn.polyhaven.com/asset_img/thumbs/{id}.png?width=128&height=128");
            m.page = format!("https://polyhaven.com/a/{id}");
            m.polycount = a["polycount"].as_u64().unwrap_or(0);
            m.size_mm = (dims.len() == 3).then(|| [dims[0], dims[1], dims[2]]);
            (a["download_count"].as_i64().unwrap_or(0), m)
        })
        .collect();
    out.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
    Ok(out.into_iter().map(|(_, a)| a).collect())
}

/// Triangles from a Smithsonian file name (`…-150k-4096.glb` → 150 000).
fn k_count(name: &str) -> u64 {
    name.split(['-', '_', '.'])
        .filter_map(|w| w.strip_suffix('k').and_then(|n| n.parse::<u64>().ok()))
        .find(|n| *n > 0 && *n < 100_000)
        .map_or(0, |n| n * 1000)
}

/// Smithsonian's `content/file/search`: one model per 3D package, its uncompressed
/// "Low_resolution" GLB (Draco-compressed files are left out: no decoder here).
pub fn parse_smithsonian(json: &[u8]) -> Result<Vec<ModelAsset>, String> {
    let v: Value = serde_json::from_slice(json).map_err(|e| e.to_string())?;
    let rows = v["rows"].as_array().ok_or("not a search answer")?;
    let mut out: Vec<ModelAsset> = Vec::new();
    for r in rows {
        let c = &r["content"];
        let Some(pkg) = c["model_url"].as_str().or_else(|| c["uri"].as_str().and_then(|u| u.split('/').find(|p| p.starts_with("3d_package:")))) else { continue };
        let id = pkg.trim_start_matches("3d_package:").to_string();
        let i = match out.iter().position(|a| a.id == id) {
            Some(i) => i,
            None => {
                let mut m = ModelAsset::blank(ModelSource::Smithsonian, id.clone());
                m.name = r["title"].as_str().unwrap_or(&id).to_string();
                m.author = "Smithsonian Institution".into();
                m.license = "CC0".into();
                m.category = "Smithsonian".into();
                out.push(m);
                out.len() - 1
            }
        };
        let uri = c["uri"].as_str().unwrap_or_default();
        let (ty, quality, usage) = (c["file_type"].as_str().unwrap_or_default(), c["quality"].as_str().unwrap_or_default(), c["usage"].as_str().unwrap_or_default());
        let a = &mut out[i];
        if ty == "jpg" && quality == "Thumb" {
            a.thumb = uri.to_string();
        }
        let draco = c["draco_compressed"].as_bool() == Some(true) || uri.contains("draco");
        if ty == "glb" && usage == "Download3D" && !draco && (a.file.is_empty() || quality == "Low_resolution") {
            a.file = uri.to_string();
            a.polycount = k_count(uri.rsplit('/').next().unwrap_or_default());
        }
    }
    out.retain(|a| !a.file.is_empty());
    Ok(out)
}

/// NASA-3D-Resources' git tree: every `3D Models/<name>/….glb`, its folder's image as the
/// thumbnail.
pub fn parse_nasa(json: &[u8]) -> Result<Vec<ModelAsset>, String> {
    const RAW: &str = "https://raw.githubusercontent.com/nasa/NASA-3D-Resources/master/";
    let v: Value = serde_json::from_slice(json).map_err(|e| e.to_string())?;
    let tree = v["tree"].as_array().ok_or_else(|| v["message"].as_str().unwrap_or("not a git tree").to_string())?;
    let blobs: Vec<(&str, u64)> = tree.iter().filter(|t| t["type"] == "blob").filter_map(|t| Some((t["path"].as_str()?, t["size"].as_u64().unwrap_or(0)))).collect();
    let mut out = Vec::new();
    for (path, size) in &blobs {
        let Some(rest) = path.strip_prefix("3D Models/") else { continue };
        if !path.to_lowercase().ends_with(".glb") {
            continue;
        }
        let Some((folder, file)) = rest.split_once('/') else { continue };
        let dir = format!("3D Models/{folder}/");
        let thumb = blobs
            .iter()
            .map(|(p, _)| *p)
            .find(|p| p.starts_with(&dir) && !p[dir.len()..].contains('/') && [".png", ".jpg", ".jpeg"].iter().any(|e| p.to_lowercase().ends_with(e)));
        let several = blobs.iter().filter(|(p, _)| p.starts_with(&dir) && p.to_lowercase().ends_with(".glb")).count() > 1;
        let mut m = ModelAsset::blank(ModelSource::Nasa, path.to_string());
        m.name = if several { file.trim_end_matches(".glb").trim_end_matches(".GLB").to_string() } else { folder.to_string() };
        m.category = "Space".into();
        m.keywords = folder.to_string();
        m.author = "NASA".into();
        m.license = "NASA media guidelines".into();
        m.thumb = thumb.map(|t| format!("{RAW}{}", encode_path(t))).unwrap_or_default();
        m.page = format!("https://github.com/nasa/NASA-3D-Resources/tree/master/{}", encode_path(&dir));
        m.bytes = *size;
        m.file = format!("{RAW}{}", encode_path(path));
        out.push(m);
    }
    out.sort_by_key(|a| a.name.to_lowercase());
    Ok(out)
}

fn text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Array(a) => Some(a.iter().filter_map(text).collect::<Vec<_>>().join(" ")),
        _ => None,
    }
}

/// A plugin's catalogue / search answer: `[{ id, name, category, tags, author, license,
/// thumb, page, polycount, size: [x, y, z] mm }]`.
pub fn parse_plugin_catalogue(source: &ModelSource, v: &Value) -> Result<Vec<ModelAsset>, String> {
    let list = v.as_array().or_else(|| v["models"].as_array()).ok_or("the plugin must return an array of models")?;
    Ok(list
        .iter()
        .filter_map(|a| {
            let id = text(&a["id"]).filter(|s| !s.is_empty())?;
            let s = |k: &str| text(&a[k]).unwrap_or_default();
            let mut m = ModelAsset::blank(source.clone(), id.clone());
            m.name = text(&a["name"]).unwrap_or(id.clone());
            m.category = s("category");
            m.keywords = format!("{} {} {id}", s("keywords"), s("tags"));
            m.author = s("author");
            m.license = s("license");
            m.thumb = s("thumb");
            m.page = s("page");
            m.polycount = a["polycount"].as_u64().unwrap_or(0);
            m.size_mm = a["size"].as_array().filter(|d| d.len() == 3).and_then(|d| Some([d[0].as_f64()?, d[1].as_f64()?, d[2].as_f64()?]));
            m.bytes = a["bytes"].as_u64().unwrap_or(0);
            Some(m)
        })
        .collect())
}

/// Names made unique in a list ("Side Chair", "Side Chair 2"): each is a library file name.
fn unique_names(list: &mut [ModelAsset]) {
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for a in list.iter_mut() {
        let n = seen.entry(crate::ui::mat_library::safe_name(&a.name).to_lowercase()).or_insert(0);
        *n += 1;
        if *n > 1 {
            a.name = format!("{} {n}", a.name);
        }
    }
}

/// The models of `source` for `query` (the whole catalogue for sources that don't search).
pub fn catalogue(source: &ModelSource, query: &str) -> Result<Vec<ModelAsset>, String> {
    let mut list = catalogue_raw(source, query)?;
    unique_names(&mut list);
    Ok(list)
}

fn catalogue_raw(source: &ModelSource, query: &str) -> Result<Vec<ModelAsset>, String> {
    let day = Some(Duration::from_secs(24 * 3600));
    match source {
        ModelSource::PolyHaven => parse_polyhaven(&cached("https://api.polyhaven.com/assets?t=models", "polyhaven_models.json", day)?),
        ModelSource::Smithsonian => {
            let q = query.trim();
            if q.is_empty() {
                return Ok(Vec::new());
            }
            let url = format!("https://3d-api.si.edu/api/v1.0/content/file/search?q={}&rows=1000", encode_path(q).replace('/', "%2F"));
            parse_smithsonian(&cached(&url, &format!("smithsonian/{}.json", file_safe(&q.to_lowercase())), day)?)
        }
        ModelSource::Nasa => parse_nasa(&cached("https://api.github.com/repos/nasa/NASA-3D-Resources/git/trees/master?recursive=1", "nasa_tree.json", day)?),
        ModelSource::Web => Ok(Vec::new()),
        ModelSource::Plugin(p) => {
            let params = Value::Object(p.params.clone());
            let v = if source.searches() { call(&p.plugin, "search", vec![("query", json!(query)), ("params", params)])? } else { call(&p.plugin, "catalogue", vec![("params", params)])? };
            parse_plugin_catalogue(source, &v)
        }
    }
}

/// `~/.varstotas/downloads`.
pub fn downloads_dir() -> Option<PathBuf> {
    crate::bridge::home_dir().map(|d| d.join("downloads"))
}

fn save(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

/// A relative path from an archive / a web listing, safe to join to a folder (no `..`, not
/// absolute).
fn safe_rel(p: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = p.split(['/', '\\']).filter(|s| !s.is_empty() && *s != ".").collect();
    (!parts.is_empty() && !parts.contains(&"..") && !parts[0].contains(':')).then(|| parts.iter().collect())
}

/// The last part of a URL path (decoded spaces), for a file name.
fn url_file(url: &str) -> String {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let f = path.rsplit('/').next().unwrap_or("model").replace("%20", " ");
    if f.is_empty() { "model".into() } else { f }
}

/// Fetch the files of `asset` into `dir`: its model spec.
fn fetch_asset(asset: &ModelAsset, dir: &Path) -> Result<Value, String> {
    match &asset.source {
        ModelSource::PolyHaven => {
            let files: Value = serde_json::from_slice(&fetch(&format!("https://api.polyhaven.com/files/{}", asset.id))?).map_err(|e| e.to_string())?;
            let g = &files["gltf"]["1k"]["gltf"];
            let url = g["url"].as_str().ok_or("no glTF download for this model")?;
            let main = dir.join(url_file(url));
            save(&main, &fetch(url)?)?;
            for (rel, f) in g["include"].as_object().into_iter().flatten() {
                let (Some(rel), Some(u)) = (safe_rel(rel), f["url"].as_str()) else { continue };
                save(&dir.join(rel), &fetch(u)?)?;
            }
            Ok(json!({ "file": main.to_string_lossy() }))
        }
        ModelSource::Smithsonian | ModelSource::Nasa | ModelSource::Web => Ok(json!({ "url": asset.file })),
        ModelSource::Plugin(p) => call(&p.plugin, "download", vec![("id", json!(asset.id)), ("dir", json!(dir.to_string_lossy())), ("params", Value::Object(p.params.clone()))]),
    }
}

/// Model files by preference.
const MODEL_EXTS: [&str; 6] = ["glb", "gltf", "obj", "stl", "dxf", "vrst"];

fn ext_of(p: &Path) -> String {
    p.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default()
}

/// The model file in a folder (subfolders too): the first by [`MODEL_EXTS`], then one an
/// importer plugin reads.
fn find_model(dir: &Path, importers: &[Plugin]) -> Option<PathBuf> {
    let files: Vec<PathBuf> = varstotas_script::list_files(dir, "").into_iter().map(PathBuf::from).collect();
    MODEL_EXTS
        .iter()
        .find_map(|e| files.iter().find(|f| ext_of(f) == *e).cloned())
        .or_else(|| files.iter().find(|f| importers.iter().any(|p| p.model_import.contains(&ext_of(f)))).cloned())
}

/// Extract a zip into `dir` (its paths made safe).
fn extract(zip: &Path, dir: &Path) -> Result<(), String> {
    let data = std::fs::read(zip).map_err(|e| format!("{}: {e}", zip.display()))?;
    for (name, bytes) in unzip(&data, |_| true)? {
        if let Some(rel) = safe_rel(&name) {
            save(&dir.join(rel), &bytes)?;
        }
    }
    Ok(())
}

/// Import a model file (by extension, or with an importer plugin).
pub fn import_file(path: &Path, importers: &[Plugin], depth: usize) -> Result<(Model, DMat4), String> {
    use varstotas_kernel::io;
    let ext = ext_of(path);
    let mut m = Model::new();
    let r = match ext.as_str() {
        "glb" | "gltf" => io::import_gltf(&mut m, path),
        "obj" => io::import_obj(&mut m, path),
        "stl" => io::import_stl(&mut m, path),
        "dxf" => io::import_dxf(&mut m, path),
        "vrst" => io::load(path).map(|x| m = x),
        "zip" => return model_from_spec(&json!({ "zip": path.to_string_lossy() }), path.parent().unwrap_or(Path::new(".")), importers, depth),
        _ => {
            let p = importers.iter().find(|p| p.model_import.contains(&ext)).ok_or_else(|| format!("can't import .{ext} files (no importer)"))?;
            if depth > 3 {
                return Err("importer plugins call each other too deep".into());
            }
            let params = crate::plugins::param_values(p, &[], varstotas_kernel::LengthUnit::Millimeter)?;
            let abs = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
            let v = call(p, "import_model", vec![("path", json!(abs.to_string_lossy())), ("params", Value::Object(params))]).map_err(|e| format!("{}: {e}", p.name))?;
            return model_from_spec(&v, abs.parent().unwrap_or(Path::new(".")), importers, depth + 1);
        }
    };
    crate::document::guarded(|| r)?.map_err(|e| format!("{}: {e}", path.display()))?;
    Ok((m, DMat4::IDENTITY))
}

/// A model from a spec (see the module docs); relative paths resolve against `base`. Also the
/// spec's unit / up-axis change, to apply to the content.
pub fn model_from_spec(spec: &Value, base: &Path, importers: &[Plugin], depth: usize) -> Result<(Model, DMat4), String> {
    let spec = match spec {
        Value::String(s) => json!({ "file": s }),
        Value::Array(a) => a.first().cloned().ok_or("an empty answer")?,
        Value::Null => return Err("the plugin returned nothing".into()),
        v => v.clone(),
    };
    let o = spec.as_object().ok_or("a model spec must be an object")?;
    let path = |k: &str| -> Option<PathBuf> {
        let s = o.get(k)?.as_str().filter(|s| !s.is_empty())?;
        let p = PathBuf::from(s.trim_start_matches("file://"));
        Some(if p.is_absolute() { p } else { base.join(p) })
    };
    let (mut model, mut xf) = if let Some(url) = o.get("url").and_then(Value::as_str) {
        let f = base.join(url_file(url));
        save(&f, &fetch(url)?)?;
        if ext_of(&f) == "zip" { from_zip(&f, base, importers, depth)? } else { import_file(&f, importers, depth)? }
    } else if let Some(z) = path("zip") {
        from_zip(&z, base, importers, depth)?
    } else if let Some(f) = path("file") {
        import_file(&f, importers, depth)?
    } else if let Some(meshes) = o.get("meshes").and_then(Value::as_array) {
        (from_meshes(meshes, o.get("materials"), base)?, DMat4::IDENTITY)
    } else {
        return Err("the model spec has no file, url, zip or meshes".into());
    };
    let k = match o.get("unit").and_then(Value::as_str).map(str::to_ascii_lowercase).as_deref() {
        Some("m") => 1000.0,
        Some("cm") => 10.0,
        Some("in") => 25.4,
        Some("ft") => 304.8,
        _ => o.get("scale").and_then(Value::as_f64).filter(|k| *k > 0.0).unwrap_or(1.0),
    };
    if o.get("up").and_then(Value::as_str).is_some_and(|u| u.eq_ignore_ascii_case("y")) {
        // Y-up → Z-up: (x, y, z) → (x, -z, y).
        xf = DMat4::from_rotation_x(std::f64::consts::FRAC_PI_2) * xf;
    }
    if k != 1.0 {
        xf = DMat4::from_scale(DVec3::splat(k)) * xf;
    }
    if let Some(n) = o.get("name").and_then(Value::as_str).filter(|n| !n.trim().is_empty()) {
        model.defs[model.root].name = n.to_string();
    }
    Ok((model, xf))
}

fn from_zip(zip: &Path, base: &Path, importers: &[Plugin], depth: usize) -> Result<(Model, DMat4), String> {
    let dir = base.join(format!("{}_files", zip.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()));
    let _ = std::fs::remove_dir_all(&dir);
    extract(zip, &dir)?;
    let f = find_model(&dir, importers).ok_or("no model file (glb, gltf, obj, stl, dxf) in the archive")?;
    import_file(&f, importers, depth)
}

/// Faces from `meshes` (one group per mesh), painted with `materials` (material specs).
fn from_meshes(meshes: &[Value], materials: Option<&Value>, base: &Path) -> Result<Model, String> {
    let mut m = Model::new();
    let mut mats = std::collections::HashMap::new();
    for spec in materials.and_then(Value::as_array).into_iter().flatten() {
        let mat = crate::mat_plugins::material_from_spec(spec, base, "Material", 1000.0)?;
        let id = m.add_material(&mat.name, mat.color);
        mats.insert(mat.name.clone(), id);
        m.materials[id] = mat;
    }
    for (k, mesh) in meshes.iter().enumerate() {
        let pts: Vec<DVec3> = mesh["vertices"]
            .as_array()
            .ok_or_else(|| format!("mesh {k}: no vertices"))?
            .iter()
            .map(|p| {
                let c = |i: usize| p.get(i).and_then(Value::as_f64).unwrap_or(0.0);
                DVec3::new(c(0), c(1), c(2))
            })
            .collect();
        let mat = mesh["material"].as_str().and_then(|n| mats.get(n).copied());
        let mut new = Vec::new();
        for f in mesh["faces"].as_array().ok_or_else(|| format!("mesh {k}: no faces"))? {
            let idx: Vec<usize> = f.as_array().into_iter().flatten().filter_map(Value::as_u64).map(|i| i as usize).collect();
            if idx.len() < 3 || idx.iter().any(|i| *i >= pts.len()) {
                continue;
            }
            let poly: Vec<DVec3> = idx.iter().map(|i| pts[*i]).collect();
            if let Some(fid) = m.root_mesh_mut().add_face(&poly) {
                if let Some(face) = m.root_mesh_mut().faces.get_mut(fid) {
                    face.front = mat;
                }
                new.push(fid);
            }
        }
        if new.is_empty() {
            continue;
        }
        // Group this mesh's faces with their edges.
        let mesh_ref = m.root_mesh();
        let mut ents: Vec<EntityRef> = new.iter().filter(|f| mesh_ref.faces.contains_key(**f)).map(|f| EntityRef::Face(*f)).collect();
        let mut edges: Vec<_> = new.iter().filter(|f| mesh_ref.faces.contains_key(**f)).flat_map(|f| mesh_ref.face_edges(*f)).collect();
        edges.sort();
        edges.dedup();
        ents.extend(edges.into_iter().map(EntityRef::Edge));
        if let Ok(g) = m.make_group(&[], &ents)
            && let Some(name) = mesh["name"].as_str()
            && let Some(i) = m.root_mesh_mut().instances.get_mut(g)
        {
            i.name = name.to_string();
        }
    }
    if m.root_mesh().instances.is_empty() && m.root_mesh().faces.is_empty() {
        return Err("no faces in the meshes".into());
    }
    Ok(m)
}

/// Faces of the model (all definitions).
pub fn face_count(m: &Model) -> usize {
    m.defs.values().map(|d| d.mesh.faces.len()).sum()
}

/// The imported content as one component `name` (with `credit`) moved by `xf`, its bounding
/// box starting at the origin: a components library file.
pub fn library_model(mut m: Model, xf: DMat4, name: &str, credit: Option<Attribution>) -> Result<Model, String> {
    let root = m.root_mesh();
    let ents: Vec<EntityRef> = root
        .faces
        .keys()
        .map(EntityRef::Face)
        .chain(root.edges.keys().map(EntityRef::Edge))
        .chain(root.instances.keys().map(EntityRef::Instance))
        .collect();
    if ents.is_empty() {
        return Err("the model is empty".into());
    }
    let inst = crate::document::guarded(|| m.make_component(&[], &ents, name))?.map_err(|e| e.to_string())?;
    let def = m.root_mesh().instances[inst].def;
    m.defs[def].name = name.to_string();
    m.defs[def].source = credit;
    m.root_mesh_mut().instances[inst].transform = xf;
    let b = m.bounds().ok_or("the model has no size")?;
    m.root_mesh_mut().instances[inst].transform = DMat4::from_translation(-b.min) * xf;
    m.guides.clear();
    m.guide_points.clear();
    Ok(m)
}

/// Download `asset`, import it and save it into the library folder `lib` (`<lib>/<source>/`).
pub fn download(asset: &ModelAsset, lib: &Path, importers: &[Plugin]) -> Result<(PathBuf, usize), String> {
    let id = if asset.id.len() > 80 { format!("{:016x}", fxhash(&asset.id)) } else { file_safe(&asset.id) };
    let dir = downloads_dir().ok_or("no home folder")?.join(asset.source.key()).join(id);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let spec = fetch_asset(asset, &dir)?;
    let (m, xf) = model_from_spec(&spec, &dir, importers, 0)?;
    let faces = face_count(&m);
    let name = crate::ui::mat_library::safe_name(&asset.name);
    let mut m = library_model(m, xf, &name, Some(asset.attribution()))?;
    m.set_preview(preview_of(asset, &spec, &dir));
    let path = lib.join(asset.source.folder()).join(format!("{name}.vrst"));
    if let Some(d) = path.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    varstotas_kernel::io::save(&m, &path).map_err(|e| e.to_string())?;
    Ok((path, faces))
}

/// The preview picture of a download: the spec's `preview` (a file of `dir` or a URL), else
/// the asset's thumbnail (Poly Haven's at 512 px), made by [`crate::textures::preview_bytes`].
/// None when there is none or it can't be read (a download doesn't fail for it).
fn preview_of(asset: &ModelAsset, spec: &Value, dir: &Path) -> Option<Vec<u8>> {
    let read = |src: &str, cache: &str| -> Option<Vec<u8>> {
        if src.starts_with("http://") || src.starts_with("https://") {
            cached(src, cache, None).ok()
        } else {
            let p = Path::new(src.trim_start_matches("file://"));
            std::fs::read(if p.is_absolute() { p.to_path_buf() } else { dir.join(p) }).ok()
        }
    };
    let key = file_safe(&format!("model_{}", asset.key()));
    let bytes = match spec.get("preview").and_then(Value::as_str).filter(|s| !s.is_empty()) {
        Some(src) => read(src, &format!("previews/{key}.img"))?,
        None if matches!(asset.source, ModelSource::PolyHaven) => {
            read(&format!("https://cdn.polyhaven.com/asset_img/thumbs/{}.png?width=512&height=512", asset.id), &format!("previews/{key}.img"))?
        }
        None if !asset.thumb.is_empty() => read(&asset.thumb, &format!("thumbs/{key}.png"))?,
        None => return None,
    };
    crate::textures::preview_bytes(&bytes).inspect_err(|e| log::warn!("preview of {}: {e}", asset.name)).ok()
}

/// A stable hash for file names.
fn fxhash(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100_0000_01b3))
}

/// An asset for a file URL (Import from URL).
pub fn url_asset(url: &str) -> ModelAsset {
    let file = url_file(url);
    let name = Path::new(&file).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Model".into());
    let mut a = ModelAsset::blank(ModelSource::Web, format!("{:016x}", fxhash(url)));
    a.name = name;
    a.file = url.to_string();
    a.page = url.to_string();
    a
}

impl Online {
    /// Load `source`'s models for `query` (background).
    pub fn load_models(&mut self, source: ModelSource, query: String, ctx: &egui::Context) {
        let (tx, ctx) = (self.tx.clone(), ctx.clone());
        self.busy += 1;
        std::thread::spawn(move || {
            let res = catalogue(&source, &query);
            let _ = tx.send(Msg::Models(source, query, res));
            ctx.request_repaint();
        });
    }

    /// Download `asset` into the components library `lib` (background).
    pub fn download_model(&mut self, asset: ModelAsset, lib: PathBuf, importers: Vec<Plugin>, place: bool, ctx: &egui::Context) {
        let (tx, ctx) = (self.tx.clone(), ctx.clone());
        self.busy += 1;
        std::thread::spawn(move || {
            let res = download(&asset, &lib, &importers);
            let _ = tx.send(Msg::Model(asset.key(), res, place));
            ctx.request_repaint();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogues_parse() {
        let ph = br#"{"chair":{"name":"Arm Chair","categories":["furniture"],"tags":["wood"],"authors":{"Kirill":"All"},"polycount":5626,"dimensions":[848.4,765.7,1065.0],"download_count":3,"category":"Furniture/Seating"},
            "rock":{"name":"Rock","categories":["nature"],"download_count":9}}"#;
        let a = parse_polyhaven(ph).unwrap();
        assert_eq!(a.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), ["rock", "chair"]);
        assert_eq!((a[1].category.as_str(), a[1].author.as_str(), a[1].polycount, a[1].size_mm.map(|s| s[2])), ("Furniture", "Kirill", 5626, Some(1065.0)));
        assert!(a[1].matches("wood chair") && !a[1].matches("rock"));
        let si = br#"{"rows":[
            {"title":"Side Chair","content":{"usage":"Web3D","quality":"Low","uri":"https://x/3d_package:abc/c-150k-1024-low.glb","file_type":"glb","draco_compressed":true,"model_url":"3d_package:abc"}},
            {"title":"Side Chair","content":{"usage":"Download3D","quality":"Low_resolution","uri":"https://x/3d_package:abc/resources/c-150k-4096.glb","file_type":"glb","model_url":"3d_package:abc"}},
            {"title":"Side Chair","content":{"usage":"Download3D","quality":"Low_resolution","uri":"https://x/3d_package:abc/resources/c-100k-2048_std_draco.glb","file_type":"glb","model_url":"3d_package:abc"}},
            {"title":"Side Chair","content":{"usage":"Image2D","quality":"Thumb","uri":"https://x/3d_package:abc/scene-image-thumb.jpg","file_type":"jpg","model_url":"3d_package:abc"}},
            {"title":"Only Draco","content":{"usage":"Web3D","quality":"High","uri":"https://x/3d_package:def/d.glb","file_type":"glb","draco_compressed":true,"model_url":"3d_package:def"}}]}"#;
        let s = parse_smithsonian(si).unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!((s[0].id.as_str(), s[0].file.as_str(), s[0].polycount), ("abc", "https://x/3d_package:abc/resources/c-150k-4096.glb", 150_000));
        assert!(s[0].thumb.ends_with("thumb.jpg"));
        let nasa = br#"{"tree":[{"path":"3D Models/Apollo LM","type":"tree"},{"path":"3D Models/Apollo LM/Apollo LM.glb","type":"blob","size":716840},
            {"path":"3D Models/Apollo LM/Apollo LM.png","type":"blob","size":10},{"path":"Images/x.png","type":"blob","size":1}]}"#;
        let n = parse_nasa(nasa).unwrap();
        assert_eq!(n.len(), 1);
        assert_eq!((n[0].name.as_str(), n[0].bytes), ("Apollo LM", 716840));
        assert_eq!(n[0].file, "https://raw.githubusercontent.com/nasa/NASA-3D-Resources/master/3D%20Models/Apollo%20LM/Apollo%20LM.glb");
        assert!(n[0].thumb.ends_with("Apollo%20LM.png"));
        assert!(parse_nasa(br#"{"message":"API rate limit exceeded"}"#).unwrap_err().contains("rate limit"));
    }

    #[test]
    fn paths_and_names() {
        assert_eq!(safe_rel("textures/a.jpg"), Some(PathBuf::from("textures").join("a.jpg")));
        assert_eq!(safe_rel("../x"), None);
        assert_eq!(safe_rel("/etc/x").map(|p| p.to_string_lossy().to_string()), Some("etc/x".into()));
        assert_eq!(safe_rel("C:/x"), None);
        assert_eq!(url_file("https://a/b/My%20Chair.glb?x=1"), "My Chair.glb");
        assert_eq!(url_asset("https://a/b/chair.glb").name, "chair");
        assert_eq!(k_count("chair-(england)-1750-60-150k-4096.glb"), 150_000);
        let mut l = vec![url_asset("https://a/Side Chair.glb"), url_asset("https://b/side chair.glb"), url_asset("https://c/Side Chair.glb")];
        unique_names(&mut l);
        assert_eq!(l.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(), ["Side Chair", "side chair 2", "Side Chair 3"]);
    }

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("varstotas_models_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// A spec with meshes (Y-up, metres) → a library component standing on the ground.
    #[test]
    fn meshes_spec_to_library_component() {
        let d = tmp("meshes");
        let spec = json!({
            "name": "Box",
            "unit": "m",
            "up": "y",
            "materials": [{ "name": "Red", "color": "#ff0000" }],
            "meshes": [{
                "name": "Cube",
                "material": "Red",
                "vertices": [[0,0,0],[1,0,0],[1,1,0],[0,1,0],[0,0,1],[1,0,1],[1,1,1],[0,1,1]],
                "faces": [[0,3,2,1],[4,5,6,7],[0,1,5,4],[1,2,6,5],[2,3,7,6],[3,0,4,7],[0,1,99]]
            }]
        });
        let (m, xf) = model_from_spec(&spec, &d, &[], 0).unwrap();
        assert_eq!(face_count(&m), 6);
        let credit = Attribution { source: "Test".into(), title: "Box".into(), license: "CC-BY 4.0".into(), ..Default::default() };
        let lib = library_model(m, xf, "Box", Some(credit.clone())).unwrap();
        let b = lib.bounds().unwrap();
        assert!(b.min.length() < 1e-6 && (b.max - DVec3::splat(1000.0)).length() < 1e-6, "{b:?}");
        assert_eq!(lib.attributions(), vec![credit]);
        // The file round trip keeps the credit; placed from the library it stays one component.
        let f = d.join("Box.vrst");
        varstotas_kernel::io::save(&lib, &f).unwrap();
        let back = varstotas_kernel::io::load(&f).unwrap();
        let clip = crate::clipboard::Clipboard::from_library(back, "Box").unwrap();
        let mut doc = Model::new();
        clip.paste_into(&mut doc, 7, &[], &DMat4::IDENTITY).unwrap();
        assert_eq!(doc.attributions().len(), 1);
        assert!(model_from_spec(&json!({ "nothing": 1 }), &d, &[], 0).is_err());
        let _ = std::fs::remove_dir_all(d);
    }

    /// A zip with a glTF inside (written by the kernel's exporter) is found and imported.
    #[test]
    fn zip_with_a_gltf() {
        let d = tmp("zip");
        let mut src = Model::new();
        src.root_mesh_mut().add_face(&[DVec3::ZERO, DVec3::new(500.0, 0.0, 0.0), DVec3::new(500.0, 500.0, 0.0), DVec3::new(0.0, 500.0, 0.0)]).unwrap();
        let glb = varstotas_kernel::io::write_glb(&src);
        let zip = crate::online_mats::tests::zip(&[("models/thing.glb", &glb, true), ("readme.txt", b"hi", false)]);
        std::fs::write(d.join("pack.zip"), zip).unwrap();
        let (m, _) = model_from_spec(&json!({ "zip": "pack.zip" }), &d, &[], 0).unwrap();
        assert_eq!(face_count(&m), 1);
        let b = m.bounds().unwrap();
        assert!((b.max.x - 500.0).abs() < 1e-3, "{b:?}");
        let _ = std::fs::remove_dir_all(d);
    }

    /// The PLY importer example (Python) through a model spec with meshes.
    #[test]
    fn ply_importer_example() {
        if std::process::Command::new("python3").arg("--version").output().is_err() {
            return;
        }
        let (file, src) = crate::plugins::EXAMPLES.iter().find(|(n, _)| *n == "import_ply.py").unwrap();
        let d = tmp("ply");
        let path = d.join(file);
        std::fs::write(&path, src).unwrap();
        let p = crate::plugins::parse(src, &path).unwrap();
        assert_eq!(p.model_import, ["ply"]);
        let ply = "ply\nformat ascii 1.0\ncomment a tetrahedron\nelement vertex 4\nproperty float x\nproperty float y\nproperty float z\nelement face 4\nproperty list uchar int vertex_indices\nend_header\n0 0 0\n1 0 0\n0 1 0\n0 0 1\n3 0 2 1\n3 0 1 3\n3 1 2 3\n3 0 3 2\n";
        std::fs::write(d.join("tetra.ply"), ply).unwrap();
        let (m, xf) = import_file(&d.join("tetra.ply"), &[p], 0).unwrap();
        assert_eq!(face_count(&m), 4);
        let lib = library_model(m, xf, "tetra", None).unwrap();
        let b = lib.bounds().unwrap();
        assert!((b.max.z - 1000.0).abs() < 1e-6, "metres by default: {b:?}");
        let _ = std::fs::remove_dir_all(d);
    }

    /// A download's preview: the spec's file (relative to the download folder) first, then
    /// the asset's local thumbnail; opaque pictures become JPEG, large ones shrink.
    #[test]
    fn preview_from_spec_or_thumb() {
        let d = tmp("preview");
        image::RgbImage::from_pixel(1024, 512, image::Rgb([200, 100, 50])).save(d.join("big.png")).unwrap();
        image::RgbaImage::from_pixel(8, 8, image::Rgba([0, 0, 0, 0])).save(d.join("clear.png")).unwrap();
        let mut a = url_asset("https://a/chair.glb");
        a.thumb = d.join("clear.png").to_string_lossy().to_string();
        let p = preview_of(&a, &json!({ "preview": "big.png" }), &d).unwrap();
        assert!(p.starts_with(&[0xFF, 0xD8]), "JPEG");
        assert_eq!(varstotas_kernel::texture::image_dimensions(&p), Some([512, 256]));
        let p = preview_of(&a, &json!({}), &d).unwrap();
        assert!(p.starts_with(b"\x89PNG"), "transparent → PNG");
        a.thumb.clear();
        assert!(preview_of(&a, &json!({}), &d).is_none());
        assert!(preview_of(&a, &json!({ "preview": "missing.png" }), &d).is_none());
        let _ = std::fs::remove_dir_all(d);
    }
}
