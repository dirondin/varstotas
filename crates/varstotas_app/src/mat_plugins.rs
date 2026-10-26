//! Material plugins (see `docs/PLUGINS.md`):
//!
//! - `@material_import ext, …` — Materials → Import… reads those files with the plugin:
//!   Rhai `fn import_material(path, params)`, Python `plugin.py import`;
//! - `@material_source` — a source of the Online Materials window: Rhai
//!   `fn catalogue(params)` and `fn download(id, dir, params)`, Python `plugin.py catalogue` /
//!   `plugin.py download`;
//! - `@texture_generator [side]` — a pattern of the Procedural Material window: Rhai
//!   `fn texture(size, params)`, Python `plugin.py texture` (stdin: `size`, `params`, `dir`).
//!
//! A Rhai plugin runs on its own (an empty model; the files, `fetch_text`, `fetch_file` and
//! `parse_json` of the script API). A Python plugin gets its arguments as a JSON object on
//! stdin (`params`, `id`, `dir`, `path`) and prints its answer as JSON (the last line of
//! stdout). Both answer with material specs — JSON objects (see [`material_from_spec`]):
//!
//! ```text
//! { "name": "Bricks", "width": 2000, "color": "#a0522d",
//!   "maps": { "color": "bricks_diff.jpg", "normal": "bricks_nor_gl.jpg", "roughness": … },
//!   "folder": "…" / "zip": "…"   (maps found by their file names),
//!   "metallic": 0, "roughness": 0.8, "clearcoat": 0.5, … }
//! ```
//!
//! Any image (`texture`, `maps.*`) may also be given inline: `{ "size": [w, h] (or one side),
//! "rgb" | "rgba" | "grey": bytes }`, the bytes an array of 0..255, base64 text or a Rhai blob
//! (`{"$blob": base64}` in JSON), rows from the top.

use crate::online_mats::{Asset, MapKind, PluginSource, Source, embeddable, guess_map, material_from_maps, unzip};
use crate::plugins::{Lang, Plugin};
use varstotas_i18n::tr;
use varstotas_kernel::Material;
use serde_json::{Map, Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Run `func` of plugin `p` with `args` (JSON values; Rhai gets them as arguments in this
/// order, Python as one object on stdin). The answer as JSON.
pub fn call(p: &Plugin, func: &str, args: Vec<(&str, Value)>) -> Result<Value, String> {
    match p.lang {
        Lang::Rhai => {
            let src = std::fs::read_to_string(&p.path).map_err(|e| format!("{}: {e}", p.path.display()))?;
            let names: Vec<&str> = args.iter().map(|(k, _)| *k).collect();
            let call = format!("{src}\n{func}({})\n", names.join(", "));
            let vars: Map<String, Value> = args.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
            let ctx = varstotas_script::ScriptCtx { path: Vec::new(), selection: Vec::new(), time_limit: Some(std::time::Duration::from_secs(60)),
                trusted_io: true};
            let r = varstotas_script::eval_json(&mut varstotas_kernel::Model::new(), &ctx, &call, &vars);
            for l in r.run.output.lines() {
                log::info!("{}: {l}", p.name);
            }
            match r.run.error {
                Some(e) => Err(thrown(&e)),
                None => Ok(r.value),
            }
        }
        Lang::Python => {
            use std::process::{Command, Stdio};
            let py = std::env::var("VARSTOTAS_PYTHON").unwrap_or_else(|_| if cfg!(windows) { "python".into() } else { "python3".into() });
            let input = Value::Object(args.into_iter().map(|(k, v)| (k.to_string(), v)).collect()).to_string();
            let mut cmd = Command::new(&py);
            cmd.arg(&p.path).arg(func).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
            if let Some(d) = p.path.parent() {
                cmd.current_dir(d);
            }
            if let Some(d) = crate::bridge::python_dir() {
                cmd.env("PYTHONPATH", d);
            }
            let out = varstotas_script::process::output_input(
                &mut cmd,
                std::time::Duration::from_secs(60),
                8 * 1024 * 1024,
                || false,
                Some(input.into_bytes()),
            )?;
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
            let last = lines.pop();
            for l in lines {
                log::info!("{}: {l}", p.name);
            }
            if !out.status.success() {
                let err = String::from_utf8_lossy(&out.stderr);
                return Err(err.trim().lines().last().map(str::to_string).unwrap_or_else(|| tr!("ext-mat-plugin-failed")));
            }
            let last = last.ok_or_else(|| tr!("ext-mat-no-answer"))?;
            serde_json::from_str(last).map_err(|e| tr!("ext-mat-not-json", error = e.to_string(), answer = last.chars().take(120).collect::<String>()))
        }
    }
}

/// The message of a Rhai `throw` without its position ("Runtime error: X (line 3, …)" → X);
/// other errors as they are.
fn thrown(e: &str) -> String {
    let first = e.lines().next().unwrap_or(e);
    match first.strip_prefix("Runtime error: ") {
        Some(m) => m.rfind(" (line ").map_or(m, |i| &m[..i]).to_string(),
        None => e.to_string(),
    }
}


/// The catalogue answer: an array of `{ id, name, category, keywords, width, thumb, page,
/// license }` (only `id` needed).
pub fn parse_catalogue(source: &Source, v: &Value) -> Result<Vec<Asset>, String> {
    let list = v.as_array().or_else(|| v["assets"].as_array()).ok_or_else(|| tr!("ext-mat-catalogue-array"))?;
    Ok(list
        .iter()
        .filter_map(|a| {
            let id = crate::online_models::json_text(&a["id"]).filter(|s| !s.is_empty())?;
            let s = |k: &str| crate::online_models::json_text(&a[k]).unwrap_or_default();
            Some(Asset {
                source: source.clone(),
                name: crate::online_models::json_text(&a["name"]).unwrap_or_else(|| id.clone()),
                category: s("category"),
                keywords: format!("{} {} {id}", s("keywords"), s("tags")),
                width_mm: a["width"].as_f64().filter(|w| *w > 0.0).unwrap_or(1000.0),
                thumb: s("thumb"),
                page: s("page"),
                license: s("license"),
                id,
            })
        })
        .collect())
}

pub fn catalogue(src: &Arc<PluginSource>) -> Result<Vec<Asset>, String> {
    let v = call(&src.plugin, "catalogue", vec![("params", Value::Object(src.params.clone()))])?;
    parse_catalogue(&Source::Plugin(src.clone()), &v)
}

/// The folder a plugin downloads `id` into (`~/.varstotas/cache/online/<plugin>/<id>`).
fn work_dir(p: &Plugin, id: &str) -> Result<PathBuf, String> {
    let safe = |s: &str| s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' }).collect::<String>();
    let stem = p.path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let d = crate::online_mats::cache_dir().ok_or_else(|| tr!("ext-mat-no-home"))?.join(safe(&stem)).join(safe(id).trim_matches('.'));
    std::fs::create_dir_all(&d).map_err(|e| format!("{}: {e}", d.display()))?;
    Ok(d)
}

pub fn download(src: &Arc<PluginSource>, asset: &Asset) -> Result<Material, String> {
    let dir = work_dir(&src.plugin, &asset.id)?;
    let v = call(
        &src.plugin,
        "download",
        vec![("id", json!(asset.id)), ("dir", json!(dir.to_string_lossy())), ("params", Value::Object(src.params.clone()))],
    )?;
    let spec = if let Value::Array(a) = &v { a.first().cloned().unwrap_or(Value::Null) } else { v };
    material_from_spec(&spec, &dir, &asset.name, asset.width_mm)
}

/// Run texture generator `p` for a `size` × `size` tile (the plugin may answer another size):
/// the material it describes, named `name` and `width_mm` wide unless the answer says so.
pub fn texture(p: &Plugin, size: u32, params: Map<String, Value>, name: &str, width_mm: f64) -> Result<Material, String> {
    let dir = work_dir(p, "texture")?;
    let mut args = vec![("size", json!(size)), ("params", Value::Object(params))];
    if p.lang == Lang::Python {
        args.push(("dir", json!(dir.to_string_lossy())));
    }
    let v = call(p, "texture", args)?;
    let spec = if let Value::Array(a) = &v { a.first().cloned().unwrap_or(Value::Null) } else { v };
    if spec.is_null() {
        return Err(tr!("ext-mat-import-nothing"));
    }
    material_from_spec(&spec, &dir, name, width_mm)
}

/// An inline image of a spec (`{ size, rgb | rgba | grey }`) as PNG bytes.
fn inline_image(o: &Map<String, Value>) -> Result<Vec<u8>, String> {
    let (w, h) = match &o.get("size") {
        Some(Value::Array(a)) => (a.first().and_then(Value::as_u64).unwrap_or(0), a.get(1).and_then(Value::as_u64).unwrap_or(0)),
        Some(v) => (v.as_u64().unwrap_or(0), v.as_u64().unwrap_or(0)),
        None => (0, 0),
    };
    if !(1..=8192).contains(&w) || !(1..=8192).contains(&h) {
        return Err(tr!("ext-tex-bad-size"));
    }
    let (key, ch) = [("rgba", 4), ("rgb", 3), ("grey", 1)].into_iter().find(|(k, _)| o.contains_key(*k)).ok_or_else(|| tr!("ext-tex-no-pixels"))?;
    let bytes: Vec<u8> = match &o[key] {
        Value::Array(a) => a.iter().map(|x| x.as_f64().unwrap_or(0.0).round().clamp(0.0, 255.0) as u8).collect(),
        Value::String(s) => varstotas_kernel::texture::base64_decode(s).ok_or_else(|| tr!("ext-tex-no-pixels"))?,
        Value::Object(b) => b.get("$blob").and_then(Value::as_str).and_then(varstotas_kernel::texture::base64_decode).ok_or_else(|| tr!("ext-tex-no-pixels"))?,
        _ => return Err(tr!("ext-tex-no-pixels")),
    };
    let n = (w * h) as usize;
    if bytes.len() != n * ch {
        return Err(tr!("ext-tex-pixel-count", want = n * ch, got = bytes.len()));
    }
    let rgba: Vec<u8> = match ch {
        4 => bytes,
        3 => bytes.as_chunks::<3>().0.iter().flat_map(|c| [c[0], c[1], c[2], 255]).collect(),
        _ => bytes.iter().flat_map(|&g| [g, g, g, 255]).collect(),
    };
    let img = image::RgbaImage::from_raw(w as u32, h as u32, rgba).ok_or_else(|| tr!("ext-tex-bad-size"))?;
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    Ok(out.into_inner())
}

/// Read `path` with importer plugin `p`: its materials.
pub fn import(p: &Plugin, path: &Path, params: Map<String, Value>) -> Result<Vec<Material>, String> {
    let abs = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let path = abs.as_path();
    let v = call(p, "import_material", vec![("path", json!(path.to_string_lossy())), ("params", Value::Object(params))])?;
    let base = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| tr!("ext-mat-default-name"));
    let specs = match v {
        Value::Array(a) => a,
        Value::Null => return Err(tr!("ext-mat-import-nothing")),
        v => vec![v],
    };
    specs.iter().map(|s| material_from_spec(s, base, &stem, 1000.0)).collect()
}

/// A colour: `"#rrggbb"`, `[r, g, b]` (0..1, or 0..255 when any is above 1) → linear-ish
/// 0..1 as the material stores it (sRGB values, like the colour picker).
fn color(v: &Value) -> Option<[f32; 3]> {
    if let Some(s) = v.as_str() {
        let h = s.trim().trim_start_matches('#');
        if h.len() != 6 {
            return None;
        }
        let c = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok().map(|x| x as f32 / 255.0);
        return Some([c(0)?, c(2)?, c(4)?]);
    }
    let a = v.as_array()?;
    let mut c = [0.0f32; 3];
    for (k, x) in c.iter_mut().enumerate() {
        *x = a.get(k)?.as_f64()? as f32;
    }
    if c.iter().any(|x| *x > 1.0) {
        c = c.map(|x| x / 255.0);
    }
    Some(c.map(|x| x.clamp(0.0, 1.0)))
}

fn map_key(k: &str) -> Option<MapKind> {
    Some(match k.to_ascii_lowercase().as_str() {
        "color" | "colour" | "albedo" | "diffuse" | "basecolor" | "base_color" | "texture" => MapKind::Color,
        "normal" => MapKind::Normal,
        "roughness" | "rough" => MapKind::Roughness,
        "ao" | "occlusion" => MapKind::Ao,
        "metallic" | "metal" | "metalness" => MapKind::Metal,
        "emission" | "emissive" => MapKind::Emission,
        _ => return None,
    })
}

/// The spec key of a map kind (inline images are named after it).
fn map_label(k: MapKind) -> &'static str {
    ["color", "normal", "roughness", "ao", "metallic", "emission"].into_iter().find(|n| map_key(n) == Some(k)).unwrap_or("map")
}

fn is_image(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    [".png", ".jpg", ".jpeg", ".tga", ".bmp"].iter().any(|e| n.ends_with(e))
}

/// Images named like maps (`guess_map`); with no colour map among them, the first image that
/// is no other map is the colour.
fn sort_maps(files: Vec<(String, Vec<u8>)>) -> Vec<(MapKind, String, Vec<u8>)> {
    let mut out: Vec<(MapKind, String, Vec<u8>)> = Vec::new();
    let mut rest = Vec::new();
    for (f, b) in files {
        match guess_map(&f) {
            Some(k) if !out.iter().any(|(x, _, _)| *x == k) => out.push((k, f, b)),
            Some(_) => {}
            None => rest.push((f, b)),
        }
    }
    if !out.iter().any(|(k, _, _)| *k == MapKind::Color)
        && let Some((f, b)) = rest.into_iter().next()
    {
        out.push((MapKind::Color, f, b));
    }
    out
}

fn file_name(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
}

/// A material from a spec (`base` resolves relative paths; `name` / `width_mm` when the spec
/// has none):
///
/// - `name`, `width` (mm, the texture's real-world width), `color` (`"#hex"` / `[r, g, b]`),
///   `opacity`;
/// - `maps`: `{ color, normal (OpenGL), roughness, ao, metallic, emission }` image files;
///   `texture` = the colour map; `folder` / `zip`: images found by their names;
/// - factors: `metallic`, `roughness`, `transmission`, `ior`, `emission` (colour),
///   `emission_strength`, `normal_scale`, `clearcoat`, `clearcoat_roughness`, `sheen`,
///   `sheen_tint`, `anisotropy`, `anti_tiling`, `bevel` (mm).
pub fn material_from_spec(spec: &Value, base: &Path, name: &str, width_mm: f64) -> Result<Material, String> {
    let o = spec.as_object().ok_or_else(|| tr!("ext-mat-spec-object"))?;
    let path = |v: &Value| -> Option<PathBuf> {
        let s = v.as_str().filter(|s| !s.is_empty())?;
        let p = PathBuf::from(s.trim_start_matches("file://"));
        Some(if p.is_absolute() { p } else { base.join(p) })
    };
    let read = |p: &Path| std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()));
    let mut maps: Vec<(MapKind, String, Vec<u8>)> = Vec::new();
    if let Some(z) = o.get("zip").and_then(path) {
        maps = sort_maps(unzip(&read(&z)?, is_image)?);
    } else if let Some(d) = o.get("folder").and_then(path) {
        let mut files: Vec<PathBuf> = std::fs::read_dir(&d).map_err(|e| format!("{}: {e}", d.display()))?.flatten().map(|e| e.path()).filter(|p| is_image(&file_name(p))).collect();
        files.sort();
        maps = sort_maps(files.iter().map(|p| Ok((file_name(p), read(p)?))).collect::<Result<_, String>>()?);
    }
    let mut named: Vec<(MapKind, &Value)> = o.get("maps").and_then(Value::as_object).into_iter().flatten().filter_map(|(k, v)| Some((map_key(k)?, v))).collect();
    if let Some(t) = o.get("texture") {
        named.push((MapKind::Color, t));
    }
    for (kind, v) in named {
        if let Some(o) = v.as_object() {
            maps.retain(|(k, _, _)| *k != kind);
            maps.push((kind, format!("{}.png", map_label(kind)), inline_image(o)?));
            continue;
        }
        let Some(p) = path(v) else { continue };
        maps.retain(|(k, _, _)| *k != kind);
        maps.push((kind, file_name(&p), read(&p)?));
    }
    let maps = maps.into_iter().map(|(k, f, b)| embeddable(b, &f).map(|(b, f)| (k, f, b))).collect::<Result<Vec<_>, String>>()?;
    let name = o.get("name").and_then(Value::as_str).filter(|s| !s.trim().is_empty()).unwrap_or(name);
    let width = o.get("width").and_then(Value::as_f64).filter(|w| *w > 0.0).unwrap_or(width_mm);
    let mut m = material_from_maps(name, width, maps);
    if let Some(c) = o.get("color").and_then(color) {
        m.color = [c[0], c[1], c[2], m.color[3]];
    }
    if let Some(a) = o.get("opacity").and_then(Value::as_f64) {
        m.color[3] = a.clamp(0.0, 1.0) as f32;
    }
    let mut p = m.pbr.take().unwrap_or_default();
    let num = |k: &str| o.get(k).and_then(Value::as_f64).map(|x| x as f32);
    let fields: [(&str, &mut f32); 13] = [
        ("metallic", &mut p.metallic),
        ("roughness", &mut p.roughness),
        ("transmission", &mut p.transmission),
        ("ior", &mut p.ior),
        ("emission_strength", &mut p.emission_strength),
        ("normal_scale", &mut p.normal_scale),
        ("clearcoat", &mut p.clearcoat),
        ("clearcoat_roughness", &mut p.clearcoat_roughness),
        ("sheen", &mut p.sheen),
        ("sheen_tint", &mut p.sheen_tint),
        ("anisotropy", &mut p.anisotropy),
        ("anti_tiling", &mut p.anti_tiling),
        ("bevel", &mut p.bevel),
    ];
    for (k, f) in fields {
        if let Some(x) = num(k) {
            *f = x;
        }
    }
    if let Some(c) = o.get("emission").and_then(color) {
        p.emission = c;
    }
    let p = p.sanitized();
    m.pbr = (p != varstotas_kernel::Pbr::default()).then_some(p);
    Ok(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(c: [u8; 3]) -> Vec<u8> {
        let mut b = Vec::new();
        image::RgbImage::from_pixel(4, 4, image::Rgb(c)).write_to(&mut std::io::Cursor::new(&mut b), image::ImageFormat::Png).unwrap();
        b
    }

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("varstotas_matplug_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn map_names() {
        for (f, k) in [
            ("brick_wall_02_nor_gl_1k.jpg", Some(MapKind::Normal)),
            ("brick_wall_02_diff_1k.jpg", Some(MapKind::Color)),
            ("Bricks090_1K-JPG_NormalGL.jpg", Some(MapKind::Normal)),
            ("Bricks090_1K-JPG_NormalDX.jpg", None),
            ("Wood_BaseColor.png", Some(MapKind::Color)),
            ("T_Rock_Roughness.tga", Some(MapKind::Roughness)),
            ("rock_ao_2k.png", Some(MapKind::Ao)),
            ("steel_Metalness.jpg", Some(MapKind::Metal)),
            ("lamp_emissive.png", Some(MapKind::Emission)),
            ("ground_disp_1k.png", None),
            ("photo.jpg", None),
        ] {
            assert_eq!(guess_map(f), k, "{f}");
        }
    }

    #[test]
    fn spec_with_maps_folder_and_factors() {
        let d = tmp("spec");
        std::fs::write(d.join("a_diff.png"), png([200, 0, 0])).unwrap();
        std::fs::write(d.join("a_nor_gl.png"), png([128, 128, 255])).unwrap();
        std::fs::write(d.join("a_nor_dx.png"), png([128, 0, 255])).unwrap();
        let m = material_from_spec(&json!({ "folder": ".", "width": 500, "clearcoat": 0.5, "roughness": 2 }), &d, "Fallback", 1000.0).unwrap();
        assert_eq!(m.name, "Fallback");
        assert_eq!(m.image.as_ref().unwrap().size[0], 500.0);
        let p = m.pbr.unwrap();
        assert!(p.normal_map.is_some() && p.roughness_map.is_none());
        assert_eq!((p.clearcoat, p.roughness), (0.5, 1.0));
        let m = material_from_spec(&json!({ "name": "Paint", "color": "#ff8000", "opacity": 0.5, "metallic": 1 }), &d, "x", 1000.0).unwrap();
        assert_eq!((m.name.as_str(), m.color), ("Paint", [1.0, 128.0 / 255.0, 0.0, 0.5]));
        assert!(m.image.is_none() && m.pbr.unwrap().metallic == 1.0);
        let m = material_from_spec(&json!({ "maps": { "albedo": "a_diff.png", "roughness": "a_diff.png" }, "color": [255, 255, 255] }), &d, "x", 800.0).unwrap();
        assert_eq!(m.color, [1.0, 1.0, 1.0, 1.0]);
        assert!(m.pbr.unwrap().roughness_map.is_some());
        assert!(material_from_spec(&json!({ "texture": "missing.png" }), &d, "x", 1.0).is_err());
        assert!(material_from_spec(&json!(3), &d, "x", 1.0).is_err());
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn thrown_messages() {
        assert_eq!(thrown("Runtime error: Set the folder (line 20, position 30)\nin call to function 'catalogue' (line 44, position 1)"), "Set the folder");
        assert_eq!(thrown("Syntax error (line 2)"), "Syntax error (line 2)");
    }

    #[test]
    fn catalogue_answer() {
        let src = Source::PolyHaven;
        let v = json!([{ "id": 7, "name": "Oak", "tags": ["wood", "floor"], "width": 1200, "thumb": "/t/oak.png", "license": "CC-BY" }, { "name": "no id" }]);
        let a = parse_catalogue(&src, &v).unwrap();
        assert_eq!(a.len(), 1);
        assert_eq!((a[0].id.as_str(), a[0].width_mm, a[0].license.as_str()), ("7", 1200.0, "CC-BY"));
        assert!(a[0].matches("floor oak"));
        assert!(parse_catalogue(&src, &json!({})).is_err());
    }

    fn example(name: &str) -> Plugin {
        let (file, src) = crate::plugins::EXAMPLES.iter().find(|(n, _)| *n == name).unwrap();
        let d = tmp(&format!("ex_{name}"));
        let path = d.join(file);
        std::fs::write(&path, src).unwrap();
        crate::plugins::parse(src, &path).unwrap()
    }

    /// The Unity example: a `.mat` with its textures found by GUID in the `.meta` files.
    #[test]
    fn unity_material_example() {
        let p = example("unity_material.rhai");
        assert_eq!(p.material_import, ["mat"]);
        let root = tmp("unity");
        let mats = root.join("Assets").join("Materials");
        let tex = root.join("Assets").join("Textures");
        std::fs::create_dir_all(&mats).unwrap();
        std::fs::create_dir_all(&tex).unwrap();
        std::fs::write(tex.join("brick_albedo.png"), png([180, 60, 40])).unwrap();
        std::fs::write(tex.join("brick_albedo.png.meta"), "fileFormatVersion: 2\nguid: 0123456789abcdef0123456789abcdef\n").unwrap();
        std::fs::write(tex.join("brick_normal.png"), png([128, 128, 255])).unwrap();
        std::fs::write(tex.join("brick_normal.png.meta"), "fileFormatVersion: 2\nguid: fedcba9876543210fedcba9876543210\n").unwrap();
        let mat = r"%YAML 1.1
%TAG !u! tag:unity3d.com,2011:
--- !u!21 &2100000
Material:
  serializedVersion: 8
  m_Name: Red Brick
  m_Shader: {fileID: 4800000, guid: 933532a4fcc9baf4fa0491de14d08ed7, type: 3}
  m_SavedProperties:
    serializedVersion: 3
    m_TexEnvs:
    - _BaseMap:
        m_Texture: {fileID: 2800000, guid: 0123456789abcdef0123456789abcdef, type: 3}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _BumpMap:
        m_Texture: {fileID: 2800000, guid: fedcba9876543210fedcba9876543210, type: 3}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    - _OcclusionMap:
        m_Texture: {fileID: 0}
        m_Scale: {x: 1, y: 1}
        m_Offset: {x: 0, y: 0}
    m_Floats:
    - _BumpScale: 0.8
    - _Metallic: 0.25
    - _Smoothness: 0.75
    m_Colors:
    - _BaseColor: {r: 1, g: 0.5, b: 0.25, a: 1}
    - _EmissionColor: {r: 0, g: 0, b: 0, a: 1}
";
        let f = mats.join("RedBrick.mat");
        std::fs::write(&f, mat).unwrap();
        let ms = import(&p, &f, Map::new()).unwrap();
        assert_eq!(ms.len(), 1);
        let m = &ms[0];
        assert_eq!(m.name, "Red Brick");
        assert_eq!(m.image.as_ref().map(|i| i.file_name.as_str()), Some("brick_albedo.png"));
        let pbr = m.pbr.as_ref().unwrap();
        assert!(pbr.normal_map.is_some());
        assert!((pbr.metallic - 0.25).abs() < 1e-6 && (pbr.roughness - 0.25).abs() < 1e-6 && (pbr.normal_scale - 0.8).abs() < 1e-6);
        // No texture: the base colour.
        std::fs::write(&f, mat.replace("0123456789abcdef0123456789abcdef", "00000000000000000000000000000000")).unwrap();
        let m = &import(&p, &f, Map::new()).unwrap()[0];
        assert!(m.image.is_none());
        assert_eq!(m.color, [1.0, 0.5, 0.25, 1.0]);
        let _ = std::fs::remove_dir_all(root);
    }

    /// The texture-folder source: one material per subfolder.
    #[test]
    fn texture_folders_example() {
        let p = example("texture_folders.rhai");
        assert!(p.material_source);
        let root = tmp("folders");
        for (d, files) in [("Oak Floor", vec!["oak_diff.jpg", "oak_rough.png"]), ("Slate", vec!["slate.png"]), ("empty", vec![])] {
            std::fs::create_dir_all(root.join(d)).unwrap();
            for f in files {
                std::fs::write(root.join(d).join(f), png([90, 60, 30])).unwrap();
            }
        }
        let src = Arc::new(PluginSource { plugin: p, params: [("folder".to_string(), json!(root.to_string_lossy())), ("width".to_string(), json!(800.0))].into_iter().collect() });
        let list = catalogue(&src).unwrap();
        assert_eq!(list.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(), ["Oak Floor", "Slate"]);
        assert!(list[0].thumb.ends_with("oak_diff.jpg"));
        let m = download(&src, &list[0]).unwrap();
        assert_eq!((m.name.as_str(), m.image.as_ref().unwrap().size[0]), ("Oak Floor", 800.0));
        assert!(m.pbr.unwrap().roughness_map.is_some());
        let _ = std::fs::remove_dir_all(root);
    }

    /// The texture generator examples answer `texture` with an inline colour map.
    #[test]
    fn texture_generator_examples() {
        let python = std::process::Command::new("python3").arg("--version").output().is_ok();
        for name in ["basket_weave.rhai", "terrazzo.rhai", "marble.py"] {
            if name.ends_with(".py") && !python {
                continue;
            }
            let p = example(name);
            assert!(p.texture_generator.is_some(), "{name}");
            let params = p.params.iter().map(|q| (q.key.clone(), crate::plugins::param_value(q, &q.default, varstotas_kernel::LengthUnit::Millimeter).unwrap())).collect();
            let m = texture(&p, 32, params, "T", 500.0).unwrap_or_else(|e| panic!("{name}: {e}"));
            let img = m.image.as_ref().unwrap();
            assert_eq!(crate::textures::decode(&img.bytes).unwrap().0, 32, "{name}");
        }
    }

    /// Inline images: a grey map as a byte array, a wrong pixel count is an error.
    #[test]
    fn inline_images() {
        let d = tmp("inline");
        let spec = json!({ "texture": { "size": [2, 1], "rgb": [255, 0, 0, 0, 0, 255] }, "maps": { "roughness": { "size": 2, "grey": [0, 64, 128, 255] } } });
        let m = material_from_spec(&spec, &d, "I", 100.0).unwrap();
        let d0 = crate::textures::decode(&m.image.as_ref().unwrap().bytes).unwrap();
        assert_eq!((d0.0, d0.1, &d0.2[..4]), (2, 1, &[255, 0, 0, 255][..]));
        assert!(m.pbr.unwrap().roughness_map.is_some());
        let bad = json!({ "texture": { "size": 2, "rgb": [1, 2, 3] } });
        assert!(material_from_spec(&bad, &d, "I", 100.0).is_err());
    }

    /// The Python example answers `catalogue` from a cached catalogue file (no network).
    #[test]
    fn python_protocol() {
        if std::process::Command::new("python3").arg("--version").output().is_err() {
            return;
        }
        let d = tmp("py");
        let path = d.join("echo.py");
        std::fs::write(
            &path,
            "# @material_source\nimport json, sys\nargs = json.load(sys.stdin)\nprint('working')\nif sys.argv[1] == 'catalogue':\n    print(json.dumps([{'id': 'a', 'name': args['params']['n']}]))\nelse:\n    sys.exit('no ' + sys.argv[1])\n",
        )
        .unwrap();
        let p = crate::plugins::parse(&std::fs::read_to_string(&path).unwrap(), &path).unwrap();
        let src = Arc::new(PluginSource { plugin: p.clone(), params: [("n".to_string(), json!("Echo"))].into_iter().collect() });
        assert_eq!(catalogue(&src).unwrap()[0].name, "Echo");
        assert_eq!(call(&p, "download", vec![]).unwrap_err(), "no download");
        let _ = std::fs::remove_dir_all(d);
    }
}
