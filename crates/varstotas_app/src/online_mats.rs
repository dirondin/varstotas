//! Online PBR materials: the catalogues of Poly Haven and ambientCG (CC0 photo scans: colour,
//! normal, roughness, occlusion, metalness maps at their real-world size). Downloads go
//! through the system `curl` (no HTTP stack in the app) on background threads; catalogues and
//! thumbnails are cached in `~/.varstotas/cache/online`. A downloaded material becomes a
//! `.vrstmat` in the materials library (`<library>/Poly Haven/…`, `<library>/ambientCG/…`).

use varstotas_kernel::{Material, Pbr, TextureImage};
use serde_json::Value;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    PolyHaven,
    AmbientCg,
}

impl Source {
    pub const ALL: [Source; 2] = [Source::PolyHaven, Source::AmbientCg];

    pub fn label(self) -> &'static str {
        match self {
            Source::PolyHaven => "Poly Haven",
            Source::AmbientCg => "ambientCG",
        }
    }

    /// Short name (harness, cache files).
    pub fn key(self) -> &'static str {
        match self {
            Source::PolyHaven => "polyhaven",
            Source::AmbientCg => "ambientcg",
        }
    }

    pub fn from_key(s: &str) -> Option<Source> {
        Source::ALL.into_iter().find(|x| x.key().eq_ignore_ascii_case(s.trim()))
    }

    fn catalogue_url(self) -> &'static str {
        match self {
            Source::PolyHaven => "https://api.polyhaven.com/assets?t=textures",
            Source::AmbientCg => "https://ambientcg.com/api/v2/full_json?type=Material&limit=600&include=displayData,dimensionsData,previewData&sort=Popular",
        }
    }

    /// The asset's page on the site.
    pub fn page_url(self, id: &str) -> String {
        match self {
            Source::PolyHaven => format!("https://polyhaven.com/a/{id}"),
            Source::AmbientCg => format!("https://ambientcg.com/view?id={id}"),
        }
    }
}

/// One material of a catalogue.
#[derive(Clone, Debug, PartialEq)]
pub struct Asset {
    pub source: Source,
    pub id: String,
    pub name: String,
    pub category: String,
    /// Words to search in (tags, categories, id).
    pub keywords: String,
    /// Real-world width of the texture (mm).
    pub width_mm: f64,
    pub thumb: String,
}

impl Asset {
    pub fn key(&self) -> String {
        format!("{}:{}", self.source.key(), self.id)
    }

    /// Every word of `query` is in the name, category or keywords.
    pub fn matches(&self, query: &str) -> bool {
        let hay = format!("{} {} {}", self.name, self.category, self.keywords).to_lowercase();
        query.to_lowercase().split_whitespace().all(|w| hay.contains(w))
    }
}

/// Which map a downloaded image is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapKind {
    Color,
    Normal,
    Roughness,
    Ao,
    Metal,
}

fn curl() -> &'static str {
    if cfg!(windows) { "curl.exe" } else { "curl" }
}

/// GET `url` with the system curl (follows redirects; fails on HTTP errors).
pub fn fetch(url: &str) -> Result<Vec<u8>, String> {
    let out = std::process::Command::new(curl())
        .args(["-sSfL", "--max-time", "180", "--retry", "2", "--user-agent", "Varstotas"])
        .arg(url)
        .output()
        .map_err(|e| format!("curl: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(out.stdout)
}

/// `~/.varstotas/cache/online`.
pub fn cache_dir() -> Option<PathBuf> {
    crate::bridge::home_dir().map(|d| d.join("cache").join("online"))
}

/// `url` through the cache file `name` (re-fetched when older than `max_age`; a stale copy
/// still serves when offline).
fn cached(url: &str, name: &str, max_age: Option<Duration>) -> Result<Vec<u8>, String> {
    let path = cache_dir().map(|d| d.join(name));
    if let Some(p) = &path
        && let Ok(bytes) = std::fs::read(p)
    {
        let fresh = max_age.is_none_or(|age| std::fs::metadata(p).and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|e| e < age));
        if fresh {
            return Ok(bytes);
        }
        return Ok(fetch(url).inspect(|b| store(p, b)).unwrap_or(bytes));
    }
    let bytes = fetch(url)?;
    if let Some(p) = &path {
        store(p, &bytes);
    }
    Ok(bytes)
}

fn store(path: &Path, bytes: &[u8]) {
    if let Some(d) = path.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    let _ = std::fs::write(path, bytes);
}

fn file_safe(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect()
}

/// Poly Haven's `assets?t=textures` (most downloaded first).
pub fn parse_polyhaven(json: &[u8]) -> Result<Vec<Asset>, String> {
    let v: Value = serde_json::from_slice(json).map_err(|e| e.to_string())?;
    let obj = v.as_object().ok_or("not a catalogue")?;
    let mut out: Vec<(i64, Asset)> = obj
        .iter()
        .map(|(id, a)| {
            let strs = |k: &str| a[k].as_array().map(|l| l.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>()).unwrap_or_default();
            let cats = strs("categories");
            // "Asphalt & Bitumen/Asphalt/Worn Road Surface" → its first part.
            let category = a["category"].as_str().and_then(|c| c.split('/').next()).map(str::to_string).or_else(|| cats.first().map(|c| c.to_string())).unwrap_or_default();
            let width_mm = a["dimensions"][0].as_f64().filter(|w| *w > 0.0).map_or(2000.0, f64::round);
            let asset = Asset {
                source: Source::PolyHaven,
                id: id.clone(),
                name: a["name"].as_str().unwrap_or(id).to_string(),
                category,
                keywords: format!("{} {} {id}", strs("tags").join(" "), cats.join(" ")),
                width_mm,
                thumb: format!("https://cdn.polyhaven.com/asset_img/thumbs/{id}.png?width=128&height=128"),
            };
            (a["download_count"].as_i64().unwrap_or(0), asset)
        })
        .collect();
    out.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
    Ok(out.into_iter().map(|(_, a)| a).collect())
}

/// ambientCG's `full_json` (in its order: most popular first). Sizes are in cm; unknown ones
/// get a typical size for the kind of material.
pub fn parse_ambientcg(json: &[u8]) -> Result<Vec<Asset>, String> {
    let v: Value = serde_json::from_slice(json).map_err(|e| e.to_string())?;
    let list = v["foundAssets"].as_array().ok_or("not a catalogue")?;
    Ok(list
        .iter()
        .filter_map(|a| {
            let id = a["assetId"].as_str()?.to_string();
            let category = a["displayCategory"].as_str().unwrap_or_default().to_string();
            let small = ["Wood", "Fabric", "Leather", "Plastic", "Metal", "Paint", "Wicker", "Carpet"].iter().any(|k| category.contains(k));
            let width_mm = a["dimensionX"].as_f64().filter(|w| *w > 0.0).map_or(if small { 1000.0 } else { 2000.0 }, |cm| cm * 10.0);
            let tags = a["tags"].as_array().map(|l| l.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(" ")).unwrap_or_default();
            Some(Asset {
                source: Source::AmbientCg,
                name: a["displayName"].as_str().unwrap_or(&id).to_string(),
                category,
                keywords: format!("{tags} {id}"),
                width_mm,
                thumb: a["previewImage"]["128-PNG"].as_str().unwrap_or_default().to_string(),
                id,
            })
        })
        .collect())
}

/// The 1K JPG maps of a Poly Haven asset (its `files` JSON).
pub fn polyhaven_maps(json: &[u8]) -> Result<Vec<(MapKind, String)>, String> {
    let v: Value = serde_json::from_slice(json).map_err(|e| e.to_string())?;
    let keys = [("Diffuse", MapKind::Color), ("nor_gl", MapKind::Normal), ("Rough", MapKind::Roughness), ("AO", MapKind::Ao), ("Metal", MapKind::Metal)];
    let maps: Vec<(MapKind, String)> = keys.iter().filter_map(|(k, kind)| Some((*kind, v[*k]["1k"]["jpg"]["url"].as_str()?.to_string()))).collect();
    if !maps.iter().any(|(k, _)| *k == MapKind::Color) {
        return Err("no colour map".into());
    }
    Ok(maps)
}

/// The map an ambientCG file is (`Bricks090_1K-JPG_NormalGL.jpg` → normal).
pub fn ambientcg_map(file: &str) -> Option<MapKind> {
    let stem = file.rsplit_once('.')?.0;
    let suffix = stem.rsplit_once('_')?.1;
    Some(match suffix {
        "Color" => MapKind::Color,
        "NormalGL" => MapKind::Normal,
        "Roughness" => MapKind::Roughness,
        "AmbientOcclusion" => MapKind::Ao,
        "Metalness" => MapKind::Metal,
        _ => return None,
    })
}

/// The files of a zip archive whose names pass `want` (stored or deflated; no zip64).
pub fn unzip(data: &[u8], want: impl Fn(&str) -> bool) -> Result<Vec<(String, Vec<u8>)>, String> {
    let u16_at = |o: usize| data.get(o..o + 2).map(|b| u16::from_le_bytes([b[0], b[1]]) as usize).ok_or("truncated zip");
    let u32_at = |o: usize| data.get(o..o + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize).ok_or("truncated zip");
    // End of central directory: the last "PK\5\6" (a comment may follow it).
    let eocd = (0..data.len().saturating_sub(21)).rev().take(65_557).find(|&i| data[i..].starts_with(b"PK\x05\x06")).ok_or("not a zip archive")?;
    let count = u16_at(eocd + 10)?;
    let mut at = u32_at(eocd + 16)?;
    let mut out = Vec::new();
    for _ in 0..count {
        if !data.get(at..).is_some_and(|d| d.starts_with(b"PK\x01\x02")) {
            return Err("bad zip directory".into());
        }
        let method = u16_at(at + 10)?;
        let (csize, usize_) = (u32_at(at + 20)?, u32_at(at + 24)?);
        let (nlen, elen, clen) = (u16_at(at + 28)?, u16_at(at + 30)?, u16_at(at + 32)?);
        let local = u32_at(at + 42)?;
        let name = String::from_utf8_lossy(data.get(at + 46..at + 46 + nlen).ok_or("truncated zip")?).to_string();
        at += 46 + nlen + elen + clen;
        if name.ends_with('/') || !want(&name) {
            continue;
        }
        if !data.get(local..).is_some_and(|d| d.starts_with(b"PK\x03\x04")) {
            return Err(format!("bad zip entry {name}"));
        }
        let start = local + 30 + u16_at(local + 26)? + u16_at(local + 28)?;
        let raw = data.get(start..start + csize).ok_or("truncated zip")?;
        let bytes = match method {
            0 => raw.to_vec(),
            8 => {
                let mut v = Vec::with_capacity(usize_);
                flate2::read::DeflateDecoder::new(raw).read_to_end(&mut v).map_err(|e| format!("{name}: {e}"))?;
                v
            }
            m => return Err(format!("{name}: zip method {m} not supported")),
        };
        out.push((name, bytes));
    }
    Ok(out)
}

/// The average colour of an image (for the material's swatch colour), sRGB 0..1.
fn average_color(bytes: &[u8]) -> Option<[f32; 3]> {
    let img = image::load_from_memory(bytes).ok()?.thumbnail(32, 32).to_rgb8();
    let n = (img.width() * img.height()).max(1) as f32;
    let sum = img.pixels().fold([0.0f32; 3], |s, p| [s[0] + p[0] as f32, s[1] + p[1] as f32, s[2] + p[2] as f32]);
    Some(sum.map(|c| c / n / 255.0))
}

/// A material from downloaded maps (`(kind, file name, bytes)`): the colour map as its texture
/// at `width_mm`, the others as PBR maps of the same size (their factors 1: the maps decide).
pub fn build_material(name: &str, width_mm: f64, maps: Vec<(MapKind, String, Vec<u8>)>) -> Result<Material, String> {
    let mut pbr = Pbr { roughness: 0.8, ..Pbr::default() };
    let mut image = None;
    let mut color = [0.6, 0.6, 0.6];
    for (kind, file, bytes) in maps {
        if bytes.is_empty() {
            continue;
        }
        let tex = TextureImage::new(bytes, file, width_mm);
        match kind {
            MapKind::Color => {
                color = average_color(&tex.bytes).unwrap_or(color);
                image = Some(tex);
            }
            MapKind::Normal => pbr.normal_map = Some(tex),
            MapKind::Roughness => {
                pbr.roughness = 1.0;
                pbr.roughness_map = Some(tex);
            }
            MapKind::Ao => pbr.ao_map = Some(tex),
            MapKind::Metal => {
                pbr.metallic = 1.0;
                pbr.metallic_map = Some(tex);
            }
        }
    }
    let image = image.ok_or("no colour map")?;
    Ok(Material { name: name.to_string(), color: [color[0], color[1], color[2], 1.0], texture: None, image: Some(image), pbr: Some(pbr) })
}

/// Download `asset`'s 1K maps and build its material.
pub fn download(asset: &Asset) -> Result<Material, String> {
    let maps = match asset.source {
        Source::PolyHaven => {
            let files = fetch(&format!("https://api.polyhaven.com/files/{}", asset.id))?;
            polyhaven_maps(&files)?
                .into_iter()
                .map(|(kind, url)| {
                    let file = url.rsplit('/').next().unwrap_or("map.jpg").to_string();
                    Ok((kind, file, fetch(&url)?))
                })
                .collect::<Result<Vec<_>, String>>()?
        }
        Source::AmbientCg => {
            let zip = fetch(&format!("https://ambientcg.com/get?file={}_1K-JPG.zip", asset.id))?;
            unzip(&zip, |n| ambientcg_map(n).is_some())?.into_iter().filter_map(|(file, bytes)| Some((ambientcg_map(&file)?, file, bytes))).collect()
        }
    };
    build_material(&asset.name, asset.width_mm, maps)
}

/// The catalogue of `source` (cached for a day).
pub fn catalogue(source: Source) -> Result<Vec<Asset>, String> {
    let json = cached(source.catalogue_url(), &format!("{}.json", source.key()), Some(Duration::from_secs(24 * 3600)))?;
    match source {
        Source::PolyHaven => parse_polyhaven(&json),
        Source::AmbientCg => parse_ambientcg(&json),
    }
}

/// A decoded thumbnail (RGBA).
pub type Thumb = (u32, u32, Vec<u8>);

/// Results of the background work.
pub enum Msg {
    Catalogue(Source, Result<Vec<Asset>, String>),
    Thumb(String, Option<Thumb>),
    /// Asset key, the saved file and its material (or the error), "use it" flag.
    Material(String, Result<(PathBuf, Box<Material>), String>, bool),
}

/// Background workers: one thread per catalogue / download, a few for thumbnails.
pub struct Online {
    tx: Sender<Msg>,
    pub rx: Receiver<Msg>,
    thumbs: Option<Sender<(String, String)>>,
    pub busy: usize,
}

impl Default for Online {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self { tx, rx, thumbs: None, busy: 0 }
    }
}

impl Online {
    pub fn load_catalogue(&mut self, source: Source, ctx: &egui::Context) {
        let (tx, ctx) = (self.tx.clone(), ctx.clone());
        self.busy += 1;
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Catalogue(source, catalogue(source)));
            ctx.request_repaint();
        });
    }

    /// Download `asset` into the library folder `dir/<source>/`.
    pub fn download(&mut self, asset: Asset, dir: PathBuf, use_it: bool, ctx: &egui::Context) {
        let (tx, ctx) = (self.tx.clone(), ctx.clone());
        self.busy += 1;
        std::thread::spawn(move || {
            let res = download(&asset).and_then(|m| {
                let path = dir.join(asset.source.label()).join(format!("{}.{}", crate::ui::mat_library::safe_name(&m.name), varstotas_kernel::io::MATERIAL_EXT));
                let _ = std::fs::create_dir_all(path.parent().unwrap_or(&dir));
                varstotas_kernel::io::save_material(&m, &path).map_err(|e| e.to_string())?;
                Ok((path, Box::new(m)))
            });
            let _ = tx.send(Msg::Material(asset.key(), res, use_it));
            ctx.request_repaint();
        });
    }

    /// Fetch thumbnail `url` (cached on disk as `name`).
    pub fn thumb(&mut self, url: &str, name: &str, ctx: &egui::Context) {
        let sender = self.thumbs.get_or_insert_with(|| {
            let (qtx, qrx) = std::sync::mpsc::channel::<(String, String)>();
            let qrx = Arc::new(Mutex::new(qrx));
            for _ in 0..4 {
                let (qrx, tx, ctx) = (qrx.clone(), self.tx.clone(), ctx.clone());
                std::thread::spawn(move || {
                    loop {
                        // Take the job, then let the lock go (the others fetch meanwhile).
                        let job = qrx.lock().ok().and_then(|r| r.recv().ok());
                        let Some((url, name)) = job else { break };
                        let img = cached(&url, &format!("thumbs/{}.png", file_safe(&name)), None)
                            .ok()
                            .and_then(|b| image::load_from_memory(&b).ok())
                            .map(|i| i.to_rgba8())
                            .map(|i| (i.width(), i.height(), i.into_raw()));
                        if tx.send(Msg::Thumb(url, img)).is_err() {
                            break;
                        }
                        ctx.request_repaint();
                    }
                });
            }
            qtx
        });
        let _ = sender.send((url.to_string(), name.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogues_parse() {
        let ph = br#"{"brick_wall_02":{"name":"Brick Wall 02","categories":["brick","wall"],"tags":["red"],"dimensions":[3000,3000],"download_count":5,"category":"Brick/Red"},
            "oak":{"name":"Oak","categories":["wood"],"tags":[],"download_count":9}}"#;
        let a = parse_polyhaven(ph).unwrap();
        assert_eq!(a.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), ["oak", "brick_wall_02"]);
        assert_eq!((a[1].category.as_str(), a[1].width_mm), ("Brick", 3000.0));
        assert_eq!((a[0].category.as_str(), a[0].width_mm), ("wood", 2000.0));
        assert!(a[1].matches("RED brick") && !a[1].matches("wood"));
        let ac = br#"{"foundAssets":[{"assetId":"Tiles143","displayName":"Tiles 143","displayCategory":"Tiles","dimensionX":200,"tags":["floor"],"previewImage":{"128-PNG":"https://x/Tiles143.png"}},
            {"assetId":"Wood059","displayName":"Wood 059","displayCategory":"Wood","dimensionX":0}]}"#;
        let b = parse_ambientcg(ac).unwrap();
        assert_eq!((b[0].width_mm, b[0].thumb.as_str()), (2000.0, "https://x/Tiles143.png"));
        assert_eq!(b[1].width_mm, 1000.0);
        assert!(b[0].matches("floor tiles"));
    }

    #[test]
    fn polyhaven_files_and_ambientcg_names() {
        let f = br#"{"Diffuse":{"1k":{"jpg":{"url":"https://d/a_diff_1k.jpg"}}},"nor_gl":{"1k":{"jpg":{"url":"https://d/a_nor_gl_1k.jpg"}}},"nor_dx":{"1k":{"jpg":{"url":"x"}}}}"#;
        assert_eq!(polyhaven_maps(f).unwrap(), vec![(MapKind::Color, "https://d/a_diff_1k.jpg".into()), (MapKind::Normal, "https://d/a_nor_gl_1k.jpg".into())]);
        assert!(polyhaven_maps(br#"{"nor_gl":{}}"#).is_err());
        assert_eq!(ambientcg_map("Bricks090_1K-JPG_NormalGL.jpg"), Some(MapKind::Normal));
        assert_eq!(ambientcg_map("Bricks090_1K-JPG_NormalDX.jpg"), None);
        assert_eq!(ambientcg_map("Bricks090_1K-JPG_Color.jpg"), Some(MapKind::Color));
        assert_eq!(ambientcg_map("Bricks090.png"), None);
    }

    /// A zip with one stored and one deflated file, written by hand.
    fn zip(files: &[(&str, &[u8], bool)]) -> Vec<u8> {
        use std::io::Write;
        let (mut out, mut dir) = (Vec::new(), Vec::new());
        for (name, data, deflate) in files {
            let body = if *deflate {
                let mut e = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
                e.write_all(data).unwrap();
                e.finish().unwrap()
            } else {
                data.to_vec()
            };
            let method: u16 = if *deflate { 8 } else { 0 };
            let off = out.len() as u32;
            let hdr = |sig: &[u8], central: bool| {
                let mut h = sig.to_vec();
                if central {
                    h.extend(20u16.to_le_bytes());
                }
                h.extend(20u16.to_le_bytes());
                h.extend(0u16.to_le_bytes());
                h.extend(method.to_le_bytes());
                h.extend([0u8; 8]); // time, date, crc
                h.extend((body.len() as u32).to_le_bytes());
                h.extend((data.len() as u32).to_le_bytes());
                h.extend((name.len() as u16).to_le_bytes());
                h.extend(0u16.to_le_bytes());
                if central {
                    h.extend([0u8; 10]); // comment length, disk, attributes
                    h.extend(off.to_le_bytes());
                }
                h.extend(name.as_bytes());
                h
            };
            let local = hdr(b"PK\x03\x04", false);
            let central = hdr(b"PK\x01\x02", true);
            out.extend(local);
            out.extend(&body);
            dir.extend(central);
        }
        let at = out.len() as u32;
        let n = files.len() as u16;
        let len = dir.len() as u32;
        out.extend(dir);
        out.extend(b"PK\x05\x06");
        out.extend([0u8; 4]);
        out.extend(n.to_le_bytes());
        out.extend(n.to_le_bytes());
        out.extend(len.to_le_bytes());
        out.extend(at.to_le_bytes());
        out.extend(0u16.to_le_bytes());
        out
    }

    #[test]
    fn unzip_reads_stored_and_deflated() {
        let text = b"hello hello hello hello hello".repeat(20);
        let z = zip(&[("a_Color.jpg", b"stored", false), ("b.usdc", b"skip", false), ("c_NormalGL.jpg", &text, true)]);
        let files = unzip(&z, |n| n.ends_with(".jpg")).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0], ("a_Color.jpg".to_string(), b"stored".to_vec()));
        assert_eq!(files[1].1, text);
        assert!(unzip(b"nope", |_| true).is_err());
    }

    #[test]
    fn material_from_maps() {
        let png = |c: [u8; 3]| {
            let mut b = Vec::new();
            image::RgbImage::from_pixel(4, 2, image::Rgb(c)).write_to(&mut std::io::Cursor::new(&mut b), image::ImageFormat::Png).unwrap();
            b
        };
        let m = build_material(
            "Brick",
            3000.0,
            vec![(MapKind::Color, "c.png".into(), png([255, 0, 0])), (MapKind::Normal, "n.png".into(), png([128, 128, 255])), (MapKind::Roughness, "r.png".into(), png([200, 200, 200]))],
        )
        .unwrap();
        let img = m.image.as_ref().unwrap();
        assert_eq!((img.pixels, img.size), ([4, 2], [3000.0, 1500.0]));
        assert!((m.color[0] - 1.0).abs() < 0.01 && m.color[1] < 0.01);
        let p = m.pbr.unwrap();
        assert!(p.normal_map.is_some() && p.roughness_map.is_some() && p.ao_map.is_none());
        assert_eq!((p.roughness, p.metallic), (1.0, 0.0));
        assert!(build_material("x", 1000.0, vec![(MapKind::Normal, "n.png".into(), png([1, 2, 3]))]).is_err());
    }
}
