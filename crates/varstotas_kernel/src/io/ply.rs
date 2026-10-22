//! Stanford PLY: import (ASCII and binary, vertex / face elements, per-face or per-vertex
//! colours) and binary export (world triangles in millimetres, face colours). PLY has no
//! units: a `comment units mm|cm|m|in|ft` line is honoured, else the file is read in metres
//! (scanners and most tools write metres). Varstotas writes `comment units mm`.

use super::meshes::{ImpMat, ImpMesh, ImpPoly, add_meshes};
use super::{ImportOptions, IoError};
use crate::{Model, tessellate};
use glam::{DMat4, DVec3};
use std::collections::HashMap;
use std::path::Path;
use varstotas_i18n::tr;

fn bad(s: impl Into<String>) -> IoError {
    let msg: String = s.into();
    IoError::Format(tr!("kernel-io-ply", msg = msg))
}

#[derive(Clone, Copy, PartialEq)]
enum Ty {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    F32,
    F64,
}

fn ty(s: &str) -> Result<Ty, IoError> {
    Ok(match s {
        "char" | "int8" => Ty::I8,
        "uchar" | "uint8" => Ty::U8,
        "short" | "int16" => Ty::I16,
        "ushort" | "uint16" => Ty::U16,
        "int" | "int32" => Ty::I32,
        "uint" | "uint32" => Ty::U32,
        "float" | "float32" => Ty::F32,
        "double" | "float64" => Ty::F64,
        _ => return Err(bad(tr!("kernel-io-unknown-type-s", s = s.to_string()))),
    })
}

impl Ty {
    fn size(self) -> usize {
        match self {
            Ty::I8 | Ty::U8 => 1,
            Ty::I16 | Ty::U16 => 2,
            Ty::I32 | Ty::U32 | Ty::F32 => 4,
            Ty::F64 => 8,
        }
    }
}

struct Prop {
    name: String,
    ty: Ty,
    /// A list: the type of its length.
    list: Option<Ty>,
}

struct Element {
    name: String,
    count: usize,
    props: Vec<Prop>,
}

/// The values of one element row: scalars and lists.
#[derive(Default)]
struct Row {
    vals: HashMap<String, f64>,
    list: Vec<usize>,
}

enum Body<'a> {
    Ascii(std::str::SplitWhitespace<'a>),
    Bin { data: &'a [u8], at: usize, big: bool },
}

impl Body<'_> {
    fn read(&mut self, t: Ty) -> Result<f64, IoError> {
        match self {
            Body::Ascii(it) => it.next().and_then(|s| s.parse::<f64>().ok()).ok_or_else(|| bad(tr!("kernel-io-truncated-data"))),
            Body::Bin { data, at, big } => {
                let n = t.size();
                let b = data.get(*at..*at + n).ok_or_else(|| bad(tr!("kernel-io-truncated-data")))?;
                *at += n;
                let mut a = [0u8; 8];
                a[..n].copy_from_slice(b);
                if *big {
                    a[..n].reverse();
                }
                Ok(match t {
                    Ty::I8 => a[0] as i8 as f64,
                    Ty::U8 => a[0] as f64,
                    Ty::I16 => i16::from_le_bytes([a[0], a[1]]) as f64,
                    Ty::U16 => u16::from_le_bytes([a[0], a[1]]) as f64,
                    Ty::I32 => i32::from_le_bytes([a[0], a[1], a[2], a[3]]) as f64,
                    Ty::U32 => u32::from_le_bytes([a[0], a[1], a[2], a[3]]) as f64,
                    Ty::F32 => f32::from_le_bytes([a[0], a[1], a[2], a[3]]) as f64,
                    Ty::F64 => f64::from_le_bytes(a),
                })
            }
        }
    }
}

/// Millimetres per unit of a `comment units …` line.
fn unit_mm(u: &str) -> Option<f64> {
    Some(match u.to_ascii_lowercase().as_str() {
        "mm" | "millimeter" | "millimeters" | "millimetre" => 1.0,
        "cm" | "centimeter" | "centimeters" => 10.0,
        "m" | "meter" | "meters" | "metre" => 1000.0,
        "in" | "inch" | "inches" => 25.4,
        "ft" | "foot" | "feet" => 304.8,
        _ => return None,
    })
}

pub fn import_ply(model: &mut Model, path: &Path) -> Result<(), IoError> {
    import_ply_with(model, path, &ImportOptions::default())
}

pub fn import_ply_with(model: &mut Model, path: &Path, opts: &ImportOptions) -> Result<(), IoError> {
    let data = super::read_limited(path, super::MAX_MODEL_BYTES)?;
    let name = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    add_ply(model, &data, &name, opts)
}

/// Add PLY bytes to the model as one group `name`.
pub fn add_ply(model: &mut Model, data: &[u8], name: &str, opts: &ImportOptions) -> Result<(), IoError> {
    if !data.starts_with(b"ply") {
        return Err(bad(tr!("kernel-io-not-a-ply-file")));
    }
    let end = data.windows(10).position(|w| w == b"end_header").ok_or_else(|| bad(tr!("kernel-io-no-end-header")))?;
    let body_at = data[end..].iter().position(|b| *b == b'\n').map(|p| end + p + 1).unwrap_or(data.len());
    let header = String::from_utf8_lossy(&data[..end]);
    let (mut format, mut elements, mut k) = (String::from("ascii"), Vec::<Element>::new(), 1000.0);
    for line in header.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        match w.as_slice() {
            ["format", f, ..] => format = f.to_string(),
            ["element", n, c] => elements.push(Element { name: n.to_string(), count: c.parse().map_err(|_| bad(tr!("kernel-io-bad-element-count")))?, props: Vec::new() }),
            ["property", "list", lt, it, n] => {
                let e = elements.last_mut().ok_or_else(|| bad(tr!("kernel-io-property-before-element")))?;
                e.props.push(Prop { name: n.to_string(), ty: ty(it)?, list: Some(ty(lt)?) });
            }
            ["property", t, n] => {
                let e = elements.last_mut().ok_or_else(|| bad(tr!("kernel-io-property-before-element")))?;
                e.props.push(Prop { name: n.to_string(), ty: ty(t)?, list: None });
            }
            ["comment", "units", u, ..] => k = unit_mm(u).unwrap_or(k),
            _ => {}
        }
    }
    let text;
    let mut body = match format.as_str() {
        "ascii" => {
            text = String::from_utf8_lossy(&data[body_at..]).to_string();
            Body::Ascii(text.split_whitespace())
        }
        "binary_little_endian" => Body::Bin { data: &data[body_at..], at: 0, big: false },
        "binary_big_endian" => Body::Bin { data: &data[body_at..], at: 0, big: true },
        f => return Err(bad(tr!("kernel-io-unknown-format-f", f = f.to_string()))),
    };
    let mut verts: Vec<(DVec3, Option<[u8; 3]>)> = Vec::new();
    let mut faces: Vec<(Vec<usize>, Option<[u8; 3]>)> = Vec::new();
    let color = |r: &Row| -> Option<[u8; 3]> {
        let c = |n: &str, alt: &str| r.vals.get(n).or(r.vals.get(alt)).copied();
        Some([c("red", "r")? as u8, c("green", "g")? as u8, c("blue", "b")? as u8])
    };
    for e in &elements {
        for _ in 0..e.count {
            let mut row = Row::default();
            for p in &e.props {
                match p.list {
                    Some(lt) => {
                        let n = body.read(lt)? as usize;
                        if n > 1 << 20 {
                            return Err(bad(tr!("kernel-io-list-too-long")));
                        }
                        let mut l = Vec::with_capacity(n);
                        for _ in 0..n {
                            l.push(body.read(p.ty)? as usize);
                        }
                        if p.name == "vertex_indices" || p.name == "vertex_index" {
                            row.list = l;
                        }
                    }
                    None => {
                        let v = body.read(p.ty)?;
                        row.vals.insert(p.name.clone(), v);
                    }
                }
            }
            match e.name.as_str() {
                "vertex" => {
                    let g = |n: &str| row.vals.get(n).copied().unwrap_or(0.0);
                    verts.push((DVec3::new(g("x"), g("y"), g("z")), color(&row)));
                }
                "face" => faces.push((std::mem::take(&mut row.list), color(&row))),
                _ => {}
            }
        }
    }
    if faces.is_empty() {
        return Err(bad(tr!("kernel-io-no-faces-a-point-cloud-can")));
    }
    // Colours (the face's, else its first vertex's) → materials, at most 64 (then grey).
    let mut mats: Vec<ImpMat> = Vec::new();
    let mut by_color: HashMap<[u8; 3], usize> = HashMap::new();
    let mut polys = Vec::with_capacity(faces.len());
    for (idx, fc) in faces {
        if idx.len() < 3 || idx.iter().any(|i| *i >= verts.len()) {
            continue;
        }
        let c = fc.or(verts[idx[0]].1);
        let mat = c.and_then(|c| {
            if let Some(i) = by_color.get(&c) {
                return Some(*i);
            }
            if mats.len() >= 64 {
                return None;
            }
            // i18n-ignore-next
            mats.push(ImpMat { name: format!("Color {:02X}{:02X}{:02X}", c[0], c[1], c[2]), color: [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, 1.0], image: None });
            by_color.insert(c, mats.len() - 1);
            Some(mats.len() - 1)
        });
        polys.push(ImpPoly { pts: idx.iter().map(|i| verts[*i].0).collect(), uv: None, mat });
    }
    add_meshes(model, vec![ImpMesh { name: name.to_string(), polys }], &mats, DMat4::from_scale(DVec3::splat(k)), opts).map(|_| ())
}

/// Binary PLY of the visible model: world triangles (mm), a colour per face (its material's,
/// white without one).
pub fn write_ply(model: &Model) -> Vec<u8> {
    let tess = tessellate(model);
    let mut index: HashMap<[u64; 3], u32> = HashMap::new();
    let mut verts: Vec<DVec3> = Vec::new();
    let mut tris: Vec<([u32; 3], [u8; 3])> = Vec::new();
    for f in &tess.faces {
        let mat = f.front.or(f.inherited).and_then(|m| model.materials.get(m));
        let c = mat.map_or([255, 255, 255], |m| [0, 1, 2].map(|i| (m.color[i].clamp(0.0, 1.0) * 255.0).round() as u8));
        for t in f.positions.as_chunks::<3>().0 {
            let ids = t.map(|p| {
                let key = [p.x, p.y, p.z].map(|v| crate::geom::snap(v).to_bits());
                *index.entry(key).or_insert_with(|| {
                    verts.push(p);
                    (verts.len() - 1) as u32
                })
            });
            tris.push((ids, c));
        }
    }
    let mut out = format!(
        // i18n-ignore-next
        "ply\nformat binary_little_endian 1.0\ncomment Varstotas\ncomment units mm\nelement vertex {}\nproperty float x\nproperty float y\nproperty float z\nelement face {}\nproperty list uchar uint vertex_indices\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nend_header\n",
        verts.len(),
        tris.len()
    )
    .into_bytes();
    for v in &verts {
        for c in [v.x, v.y, v.z] {
            out.extend_from_slice(&(c as f32).to_le_bytes());
        }
    }
    for (t, c) in &tris {
        out.push(3);
        for i in t {
            out.extend_from_slice(&i.to_le_bytes());
        }
        out.extend_from_slice(c);
    }
    out
}

pub fn export_ply(model: &Model, path: &Path) -> Result<(), IoError> {
    std::fs::write(path, write_ply(model))?;
    Ok(())
}
