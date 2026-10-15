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
//! ```
//!
//! The header ends at the first line that is not a comment. Examples ship in `examples/plugins`
//! and are installed (refreshed on every start) to `~/.varstotas/plugins/examples`.

use std::path::{Path, PathBuf};

pub const EXAMPLES: &[(&str, &str)] = &[
    ("stairs.rhai", include_str!("../../../examples/plugins/stairs.rhai")),
    ("beam.rhai", include_str!("../../../examples/plugins/beam.rhai")),
    ("import_points_csv.rhai", include_str!("../../../examples/plugins/import_points_csv.rhai")),
    ("export_faces_json.py", include_str!("../../../examples/plugins/export_faces_json.py")),
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
    /// File extensions for File → Import / Export (lower case, no dot).
    pub import: Vec<String>,
    pub export: Vec<String>,
}

/// Split `text "quoted label"` → (text, label).
fn split_label(s: &str) -> (&str, Option<String>) {
    match s.find('"') {
        Some(i) => {
            let rest = &s[i + 1..];
            let label = rest.rfind('"').map_or(rest, |j| &rest[..j]);
            (s[..i].trim(), Some(label.to_string()))
        }
        None => (s.trim(), None),
    }
}

fn parse_param(s: &str) -> Result<Param, String> {
    let (decl, default) = s.split_once('=').ok_or_else(|| format!("@param needs `name = default`: {s}"))?;
    let (key, ty) = match decl.trim().split_once(':') {
        Some((k, t)) => (k.trim(), Some(t.trim())),
        None => (decl.trim(), None),
    };
    if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') || key.starts_with(|c: char| c.is_ascii_digit()) {
        return Err(format!("bad parameter name `{key}`"));
    }
    // `= "text" "Label"`: the first quoted string is the default.
    let d = default.trim();
    let (default, label) = if let Some(rest) = d.strip_prefix('"') {
        let end = rest.find('"').ok_or_else(|| format!("unclosed quote: {s}"))?;
        let (_, label) = split_label(&rest[end + 1..]);
        (rest[..end].to_string(), label)
    } else {
        let (v, label) = split_label(d);
        (v.to_string(), label)
    };
    let quoted = d.starts_with('"');
    let kind = match ty {
        Some("int") => ParamKind::Int,
        Some("number") => ParamKind::Number,
        Some("text") => ParamKind::Text,
        Some("bool") => ParamKind::Bool,
        Some("length") => ParamKind::Length,
        Some(t) => return Err(format!("unknown parameter type `{t}` (length, int, number, text, bool)")),
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
        import: Vec::new(),
        export: Vec::new(),
    };
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
        let exts = || rest.split([',', ' ']).map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase()).filter(|e| !e.is_empty()).collect();
        match key {
            "name" => p.name = rest.to_string(),
            "menu" => p.menu = rest.split('/').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
            "description" => p.description = rest.to_string(),
            "param" => p.params.push(parse_param(rest)?),
            "pick" => {
                let (n, label) = split_label(rest);
                let n: usize = n.parse().map_err(|_| format!("@pick needs a number of points: {rest}"))?;
                if !(1..=16).contains(&n) {
                    return Err("@pick: 1 to 16 points".into());
                }
                p.pick = Some((n, label.unwrap_or_else(|| format!("Click {n} point(s)"))));
            }
            "import" => p.import = exts(),
            "export" => p.export = exts(),
            // Unknown tags are ignored (room for later additions).
            _ => {}
        }
    }
    if p.name.trim().is_empty() {
        return Err("empty @name".into());
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

/// Write the example plugins to `<dir>/examples` (replacing older copies).
pub fn install_examples(dir: &Path) -> std::io::Result<()> {
    let ex = dir.join("examples");
    std::fs::create_dir_all(&ex)?;
    for (name, src) in EXAMPLES {
        let p = ex.join(name);
        if std::fs::read_to_string(&p).ok().as_deref() != Some(*src) {
            std::fs::write(p, src)?;
        }
    }
    Ok(())
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
    fn parses_the_header() {
        let src = r#"// @name Stairs
// @menu Furniture / Stairs…
// @description A flight.
// @param width = 900 "Width"
// @param steps:int = 12 "Steps"
// @param label = "Main" "Label text"
// @param rails = true
// @pick 2 "Click the start and the end"
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
        assert_eq!(p.export, ["json", "txt"]);
        let p = parse("# @import csv\nimport varstotas\n", Path::new("points_in.py")).unwrap();
        assert_eq!((p.name.as_str(), p.lang, p.import.as_slice(), p.menu.len()), ("points in", Lang::Python, &["csv".to_string()][..], 1));
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
