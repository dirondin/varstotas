//! Plugins: `.rhai` / `.py` files in `~/.varstotas/plugins` (subfolders too) described by a
//! comment header (see `docs/PLUGINS.md`):
//!
//! ```text
//! // @name Stairs
//! // @menu Furniture/Stairs
//! // @description A straight flight of steps.
//! // @param width = 900 "Width"           (a number: a length in mm, units allowed)
//! // @param steps:int = 12 "Steps"         (:int, :number, :text, :bool)
//! // @pick 2 "Click the start and the end"  (pick points first: `points`)
//! // @import csv / @export json            (File → Import / Export: `path`)
//! // @material_import mat                  (Materials → Import…, see `mat_plugins`)
//! // @material_source                      (a source of the Online Materials window)
//! // @model_import ply                      (3D models: File → Import, 3D Library downloads)
//! // @model_source [search]                 (a source of the 3D Library window)
//! // @texture_generator [max side]          (a pattern of the Procedural Material window)
//! // @notice Models are CC-BY: credit …      (terms shown above a source's assets; repeatable)
//! ```
//!
//! Texts the user reads have localized variants: the tag with a suffix of language codes and /
//! or platform names (`@name.ru Лестница`, `@menu.ru …`, `@description.ru …`,
//! `@param.ru width "Ширина"`, `@pick.ru "…"`, `@select.ru "…"`, `@notice.ru …`,
//! `@menu.macos …`); the most specific match for the UI language and platform wins
//! (`varstotas_i18n::pick_variant`), the plain tag is the fallback.
//!
//! The header ends at the first line that is not a comment. Optional examples live only in
//! the repository (`examples/plugins`); users install selected plugins themselves.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use varstotas_i18n::tr;

#[cfg(test)]
pub const EXAMPLES: &[(&str, &str)] = &[
    ("stairs.rhai", include_str!("../../../examples/plugins/stairs.rhai")),
    ("beam.rhai", include_str!("../../../examples/plugins/beam.rhai")),
    ("import_points_csv.rhai", include_str!("../../../examples/plugins/import_points_csv.rhai")),
    ("export_faces_json.py", include_str!("../../../examples/plugins/export_faces_json.py")),
    ("unity_material.rhai", include_str!("../../../examples/plugins/unity_material.rhai")),
    ("texture_folders.rhai", include_str!("../../../examples/plugins/texture_folders.rhai")),
    ("ambientcg_python.py", include_str!("../../../examples/plugins/ambientcg_python.py")),
    ("model_folders.rhai", include_str!("../../../examples/plugins/model_folders.rhai")),
    ("poly_pizza.rhai", include_str!("../../../examples/plugins/poly_pizza.rhai")),
    ("sketchfab.py", include_str!("../../../examples/plugins/sketchfab.py")),
    ("import_ply.py", include_str!("../../../examples/plugins/import_ply.py")),
    ("basket_weave.rhai", include_str!("../../../examples/plugins/basket_weave.rhai")),
    ("terrazzo.rhai", include_str!("../../../examples/plugins/terrazzo.rhai")),
    ("marble.py", include_str!("../../../examples/plugins/marble.py")),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Rhai,
    Python,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParamKind {
    /// Model length (mm; the text may carry units).
    Length,
    Int,
    Number,
    Text,
    Bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub key: String,
    pub label: String,
    pub kind: ParamKind,
    /// Default value as text ("true"/"false" for Bool).
    pub default: String,
}

impl Param {
    /// A text parameter that names a folder or a file (`folder`, `dir`, `path`…).
    pub fn is_path(&self) -> bool {
        let k = self.key.to_lowercase();
        self.kind == ParamKind::Text && ["folder", "dir", "path", "file"].iter().any(|w| k.contains(w))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Plugin {
    pub name: String,
    /// Menu path under Extensions (last item = the entry).
    pub menu: Vec<String>,
    pub description: String,
    pub lang: Lang,
    pub path: PathBuf,
    pub params: Vec<Param>,
    /// Points to pick in the viewport before running, and the prompt.
    pub pick: Option<(usize, String)>,
    /// `@select "prompt"`: objects to select in the viewport (after the points), the prompt.
    pub select: Option<String>,
    /// File extensions for File → Import / Export (lower case, no dot).
    pub import: Vec<String>,
    pub export: Vec<String>,
    /// `@material_import`: material file extensions it reads (Materials → Import…).
    pub material_import: Vec<String>,
    /// `@material_source`: a catalogue of materials to download (Online Materials window).
    pub material_source: bool,
    /// `@model_import`: 3D model file extensions it reads (see `online_models`).
    pub model_import: Vec<String>,
    /// `@model_source`: a catalogue of 3D models (3D Library window); `@model_source search`:
    /// the source searches itself (`search(query, params)` instead of `catalogue(params)`).
    pub model_source: Option<bool>,
    /// `@notice`: the source's terms (licences, what its site asks for), shown as a warning
    /// above its assets in the 3D Library / Online Materials window (several lines joined).
    pub notice: String,
    /// `@texture_generator [side]`: a pattern of the Procedural Material window, with the
    /// largest image side it is asked for (512 by default; see `mat_plugins::texture`).
    pub texture_generator: Option<u32>,
}

impl Plugin {
    /// A plugin of the Extensions menu (not a file or a material plugin).
    pub fn in_menu(&self) -> bool {
        self.import.is_empty() && self.export.is_empty() && self.material_import.is_empty() && !self.material_source && self.model_import.is_empty() && self.model_source.is_none() && self.texture_generator.is_none()
    }
}

/// A parameter's text value as a script value (lengths in mm, with `unit` for bare numbers).
pub fn param_value(q: &Param, v: &str, unit: varstotas_kernel::LengthUnit) -> Result<serde_json::Value, String> {
    use serde_json::json;
    let t = v.trim();
    let bad = || tr!("ext-param-invalid", label = q.label.clone(), value = t);
    Ok(match q.kind {
        ParamKind::Length => json!(varstotas_kernel::units::parse_length(t, unit).ok_or_else(bad)?),
        ParamKind::Int => json!(t.parse::<i64>().map_err(|_| bad())?),
        ParamKind::Number => json!(t.replace(',', ".").parse::<f64>().map_err(|_| bad())?),
        ParamKind::Bool => json!(t == "true"),
        ParamKind::Text => json!(v),
    })
}

/// The parameters of `p` with `values` (text, in order; the defaults where missing).
pub fn param_values(p: &Plugin, values: &[String], unit: varstotas_kernel::LengthUnit) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    p.params.iter().enumerate().map(|(i, q)| Ok((q.key.clone(), param_value(q, values.get(i).unwrap_or(&q.default), unit)?))).collect()
}

fn parse_param(s: &str) -> Result<Param, String> {
    let (decl, default) = s.split_once('=').ok_or_else(|| tr!("ext-param-needs-default", text = s))?;
    let (key, ty) = match decl.trim().split_once(':') {
        Some((k, t)) => (k.trim(), Some(t.trim())),
        None => (decl.trim(), None),
    };
    if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') || key.starts_with(|c: char| c.is_ascii_digit()) {
        return Err(tr!("ext-param-bad-name", key = key));
    }
    // `= "text" "Label"`: the first quoted string is the default.
    let d = default.trim();
    let (default, label) = if let Some(rest) = d.strip_prefix('"') {
        let end = rest.find('"').ok_or_else(|| tr!("ext-param-unclosed-quote", text = s))?;
        let (_, label) = varstotas_kernel::generator::split_label(&rest[end + 1..]);
        (rest[..end].to_string(), label)
    } else {
        let (v, label) = varstotas_kernel::generator::split_label(d);
        (v.to_string(), label)
    };
    let quoted = d.starts_with('"');
    let kind = match ty {
        Some("int") => ParamKind::Int,
        Some("number") => ParamKind::Number,
        Some("text") => ParamKind::Text,
        Some("bool") => ParamKind::Bool,
        Some("length") => ParamKind::Length,
        Some(t) => return Err(tr!("ext-param-unknown-type", ty = t)),
        None if default == "true" || default == "false" => ParamKind::Bool,
        None if quoted => ParamKind::Text,
        None => ParamKind::Length,
    };
    Ok(Param { key: key.to_string(), label: label.unwrap_or_else(|| key.replace('_', " ")), kind, default })
}

/// Parse a plugin's header.
pub fn parse(src: &str, path: &Path) -> Result<Plugin, String> {
    let lang = match path.extension().and_then(|e| e.to_str()) {
        Some("py") => Lang::Python,
        _ => Lang::Rhai,
    };
    let stem = path.file_stem().map(|s| s.to_string_lossy().replace('_', " ")).unwrap_or_default();
    let mut p = Plugin {
        name: stem,
        menu: Vec::new(),
        description: String::new(),
        lang,
        path: path.to_path_buf(),
        params: Vec::new(),
        pick: None,
        select: None,
        import: Vec::new(),
        export: Vec::new(),
        material_import: Vec::new(),
        material_source: false,
        model_import: Vec::new(),
        model_source: None,
        notice: String::new(),
        texture_generator: None,
    };
    // Localized variants: tag → suffix → text; `@param.<suffix> key "Label"`: key → suffix → label.
    let mut variants: HashMap<&str, BTreeMap<String, String>> = HashMap::new();
    let mut param_labels: HashMap<String, BTreeMap<String, String>> = HashMap::new();
    for line in src.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let Some(c) = t.strip_prefix("//").or_else(|| t.strip_prefix('#')) else { break };
        let c = c.trim();
        let Some(tag) = c.strip_prefix('@') else { continue };
        let (key, rest) = tag.split_once(char::is_whitespace).unwrap_or((tag, ""));
        let rest = rest.trim();
        if let Some((base, suffix)) = key.split_once('.') {
            let suffix = suffix.to_ascii_lowercase();
            match base {
                "name" | "menu" | "description" | "notice" => {
                    let base = match base {
                        "name" => "name",
                        "menu" => "menu",
                        "description" => "description",
                        _ => "notice",
                    };
                    let v = variants.entry(base).or_default().entry(suffix).or_default();
                    if !v.is_empty() {
                        v.push(' ');
                    }
                    v.push_str(rest);
                }
                "pick" | "select" => {
                    let label = varstotas_kernel::generator::split_label(rest).1.unwrap_or_else(|| rest.to_string());
                    variants.entry(if base == "pick" { "pick" } else { "select" }).or_default().insert(suffix, label);
                }
                "param" => {
                    let (k, label) = varstotas_kernel::generator::split_label(rest);
                    param_labels.entry(k.to_string()).or_default().insert(suffix, label.unwrap_or_default());
                }
                _ => {}
            }
            continue;
        }
        let exts = || rest.split([',', ' ']).map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase()).filter(|e| !e.is_empty()).collect();
        match key {
            "name" => p.name = rest.to_string(),
            "menu" => p.menu = rest.split('/').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
            "description" => p.description = rest.to_string(),
            "param" => p.params.push(parse_param(rest)?),
            "pick" => {
                let (n, label) = varstotas_kernel::generator::split_label(rest);
                let n: usize = n.parse().map_err(|_| tr!("ext-pick-needs-number", text = rest))?;
                if !(1..=16).contains(&n) {
                    return Err(tr!("ext-pick-range"));
                }
                p.pick = Some((n, label.unwrap_or_else(|| tr!("ext-pick-default", n = n))));
            }
            "select" => p.select = Some(varstotas_kernel::generator::split_label(rest).1.unwrap_or_else(|| tr!("ext-select-default"))),
            "import" => p.import = exts(),
            "export" => p.export = exts(),
            "material_import" => p.material_import = exts(),
            "material_source" => p.material_source = true,
            "model_import" => p.model_import = exts(),
            "model_source" => p.model_source = Some(rest.eq_ignore_ascii_case("search")),
            "texture_generator" => p.texture_generator = Some(rest.parse::<u32>().unwrap_or(512).clamp(16, 2048)),
            "notice" => {
                if !p.notice.is_empty() {
                    p.notice.push(' ');
                }
                p.notice.push_str(rest);
            }
            // Unknown tags are ignored (room for later additions).
            _ => {}
        }
    }
    let has_menu = variants.contains_key("menu");
    let mut pick = |tag: &str, plain: String| -> String {
        match variants.remove(tag) {
            Some(mut v) => {
                v.insert(String::new(), plain.clone());
                varstotas_i18n::pick_variant(&v).map(str::to_string).unwrap_or(plain)
            }
            None => plain,
        }
    };
    p.name = pick("name", p.name.clone());
    p.description = pick("description", p.description.clone());
    p.notice = pick("notice", p.notice.clone());
    if has_menu {
        let menu = pick("menu", p.menu.join("/"));
        p.menu = menu.split('/').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    }
    if let Some((n, label)) = p.pick.take() {
        p.pick = Some((n, pick("pick", label)));
    }
    if let Some(label) = p.select.take() {
        p.select = Some(pick("select", label));
    }
    for q in &mut p.params {
        if let Some(mut v) = param_labels.remove(&q.key) {
            v.insert(String::new(), q.label.clone());
            q.label = varstotas_i18n::pick_variant(&v).unwrap_or(&q.label).to_string();
        }
    }
    if p.name.trim().is_empty() {
        return Err(tr!("ext-empty-name"));
    }
    if p.menu.is_empty() {
        p.menu = vec![p.name.clone()];
    }
    Ok(p)
}

/// `~/.varstotas/plugins`.
pub fn plugins_dir() -> Option<PathBuf> {
    crate::bridge::home_dir().map(|d| d.join("plugins"))
}

/// All plugins under `dir` (two folder levels), sorted by menu path; parse errors per file.
pub fn scan(dir: &Path) -> (Vec<Plugin>, Vec<String>) {
    let mut files = Vec::new();
    fn walk(d: &Path, depth: usize, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if depth < 2 {
                    walk(&p, depth + 1, out);
                }
            } else if p.extension().is_some_and(|x| x == "rhai" || x == "py") {
                out.push(p);
            }
        }
    }
    walk(dir, 0, &mut files);
    let (mut list, mut errors) = (Vec::new(), Vec::new());
    for f in files {
        match std::fs::read_to_string(&f).map_err(|e| e.to_string()).and_then(|s| parse(&s, &f)) {
            Ok(p) => list.push(p),
            Err(e) => errors.push(format!("{}: {e}", f.display())),
        }
    }
    list.sort_by(|a, b| a.menu.cmp(&b.menu).then(a.path.cmp(&b.path)));
    (list, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localized_header_texts() {
        let src = "// @name Stairs\n// @name.ru Лестница\n// @menu Build/Stairs\n// @menu.ru Построение/Лестница\n// @param width = 900 \"Width\"\n// @param.ru width \"Ширина\"\n// @pick 1 \"Click\"\n// @pick.ru \"Щёлкните\"\n// @description.zz Other\n";
        let p = varstotas_i18n::with_language("en", || parse(src, Path::new("s.rhai")).unwrap());
        assert_eq!((p.name.as_str(), p.params[0].label.as_str()), ("Stairs", "Width"));
        let p = varstotas_i18n::with_language("ru", || parse(src, Path::new("s.rhai")).unwrap());
        assert_eq!(p.name, "Лестница");
        assert_eq!(p.menu, ["Построение", "Лестница"]);
        assert_eq!(p.params[0].label, "Ширина");
        assert_eq!(p.pick.as_ref().unwrap().1, "Щёлкните");
        assert_eq!(p.description, "");
    }

    #[test]
    fn parses_the_header() {
        let src = r#"// @name Stairs
// @menu Furniture / Stairs…
// @description A flight.
// @param width = 900 "Width"
// @param steps:int = 12 "Steps"
// @param label = "Main" "Label text"
// @param rails = true
// @pick 2 "Click the start and the end"
// @select "Select the walls"
// @export JSON, .txt
let x = 1; // @name not a header any more
"#;
        let p = parse(src, Path::new("/x/stairs.rhai")).unwrap();
        assert_eq!(p.name, "Stairs");
        assert_eq!(p.menu, ["Furniture", "Stairs…"]);
        assert_eq!(p.lang, Lang::Rhai);
        let kinds: Vec<_> = p.params.iter().map(|q| (q.key.as_str(), q.kind, q.default.as_str(), q.label.as_str())).collect();
        assert_eq!(
            kinds,
            [
                ("width", ParamKind::Length, "900", "Width"),
                ("steps", ParamKind::Int, "12", "Steps"),
                ("label", ParamKind::Text, "Main", "Label text"),
                ("rails", ParamKind::Bool, "true", "rails"),
            ]
        );
        assert_eq!(p.pick, Some((2, "Click the start and the end".into())));
        assert_eq!(p.select.as_deref(), Some("Select the walls"));
        assert_eq!(parse("// @select\n", Path::new("a.rhai")).unwrap().select.as_deref(), Some("Select objects"));
        assert_eq!(p.export, ["json", "txt"]);
        let p = parse("# @import csv\nimport varstotas\n", Path::new("points_in.py")).unwrap();
        assert_eq!((p.name.as_str(), p.lang, p.import.as_slice(), p.menu.len()), ("points in", Lang::Python, &["csv".to_string()][..], 1));
        let p = parse("// @name Unity\n// @material_import .mat, MATERIAL\n", Path::new("u.rhai")).unwrap();
        assert_eq!((p.material_import.as_slice(), p.material_source, p.in_menu()), (&["mat".to_string(), "material".to_string()][..], false, false));
        let p = parse("# @model_source search\n# @notice Models are CC-BY:\n# @notice credit the authors.\n", Path::new("s.py")).unwrap();
        assert_eq!(p.notice, "Models are CC-BY: credit the authors.");
        let p = parse("# @material_source\n", Path::new("site.py")).unwrap();
        assert!(p.material_source && !p.in_menu());
        assert!(parse("// @name X\n", Path::new("x.rhai")).unwrap().in_menu());
        let p = parse("# @model_import PLY\n# @model_source search\n", Path::new("m.py")).unwrap();
        assert_eq!((p.model_import.as_slice(), p.model_source, p.in_menu()), (&["ply".to_string()][..], Some(true), false));
        assert_eq!(parse("// @model_source\n", Path::new("m.rhai")).unwrap().model_source, Some(false));
        assert!(parse("// @param 1x = 3", Path::new("a.rhai")).is_err());
        assert!(parse("// @param x:float = 3", Path::new("a.rhai")).is_err());
    }

    #[test]
    fn examples_parse() {
        for (name, src) in EXAMPLES {
            let p = parse(src, Path::new(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(!p.description.is_empty(), "{name}");
        }
    }
}
