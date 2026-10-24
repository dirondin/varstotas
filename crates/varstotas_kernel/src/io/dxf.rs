//! DXF (ASCII) export and import.
//!
//! Export: AutoCAD R12 — faces as 3DFACE triangles, hard edges as LINE (world space, mm).
//! Import: LINE, LWPOLYLINE, POLYLINE (2D/3D polylines and polyface meshes), 3DFACE, CIRCLE,
//! ARC and POINT-free geometry of the ENTITIES section; units from `$INSUNITS`.
//! Lines and polylines become edges (closed planar loops find faces), 3DFACEs become faces.

use super::{ImportOptions, IoError};
use crate::{Model, tessellate};
use glam::DVec3;
use std::fmt::Write as _;
use std::path::Path;
use varstotas_i18n::tr;

/// Segments of a full imported circle (arcs: proportionally, at least 2).
const CIRCLE_SEGMENTS: usize = 24;

/// DXF text of the visible model.
pub fn write_dxf(model: &Model) -> String {
    let tess = tessellate(model);
    let mut s = String::new();
    let mut pair = |code: i32, v: &str| {
        let _ = write!(s, "{code:>3}\n{v}\n");
    };
    pair(0, "SECTION");
    pair(2, "HEADER");
    pair(9, "$ACADVER");
    pair(1, "AC1009");
    pair(9, "$INSUNITS");
    pair(70, "4");
    pair(0, "ENDSEC");
    pair(0, "SECTION");
    pair(2, "ENTITIES");
    let num = |v: f64| format!("{:.6}", if v.abs() < 5e-7 { 0.0 } else { v });
    for f in &tess.faces {
        for t in f.positions.as_chunks::<3>().0 {
            pair(0, "3DFACE");
            pair(8, "0");
            for (i, p) in [t[0], t[1], t[2], t[2]].into_iter().enumerate() {
                let i = i as i32;
                pair(10 + i, &num(p.x));
                pair(20 + i, &num(p.y));
                pair(30 + i, &num(p.z));
            }
        }
    }
    for e in tess.edges.iter().filter(|e| !e.soft) {
        pair(0, "LINE");
        pair(8, "0");
        for (i, p) in [e.a, e.b].into_iter().enumerate() {
            let i = i as i32;
            pair(10 + i, &num(p.x));
            pair(20 + i, &num(p.y));
            pair(30 + i, &num(p.z));
        }
    }
    pair(0, "ENDSEC");
    pair(0, "EOF");
    s
}

pub fn export_dxf(model: &Model, path: &Path) -> Result<(), IoError> {
    std::fs::write(path, write_dxf(model))?;
    Ok(())
}

/// Geometry read from a DXF file (millimetres).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DxfData {
    /// Polylines: points and whether closed.
    pub polylines: Vec<(Vec<DVec3>, bool)>,
    /// Faces (3DFACE, polyface mesh faces).
    pub faces: Vec<Vec<DVec3>>,
}

/// One entity: its type and group codes in order.
struct Entity {
    kind: String,
    codes: Vec<(i32, String)>,
}

impl Entity {
    fn get(&self, code: i32) -> Option<f64> {
        self.codes.iter().find(|(c, _)| *c == code).and_then(|(_, v)| v.trim().parse().ok())
    }
    fn f(&self, code: i32) -> f64 {
        self.get(code).unwrap_or(0.0)
    }
    fn point(&self, k: i32) -> DVec3 {
        DVec3::new(self.f(10 + k), self.f(20 + k), self.f(30 + k))
    }
    fn flags(&self) -> i32 {
        self.get(70).unwrap_or(0.0) as i32
    }
}

/// Millimetres per drawing unit (`$INSUNITS`; unitless = mm).
fn unit_scale(insunits: i32) -> f64 {
    match insunits {
        1 => 25.4,
        2 => 304.8,
        3 => 1_609_344.0,
        5 => 10.0,
        6 => 1000.0,
        7 => 1_000_000.0,
        8 => 25.4e-6,
        9 => 25.4e-3,
        10 => 914.4,
        14 => 100.0,
        _ => 1.0,
    }
}

/// Points of an arc (degrees, counter-clockwise from `a0` to `a1`) in the XY plane at `c`.
fn arc_points(c: DVec3, r: f64, a0: f64, a1: f64) -> Vec<DVec3> {
    let mut sweep = (a1 - a0).rem_euclid(360.0);
    if sweep < 1e-9 {
        sweep = 360.0;
    }
    let n = ((CIRCLE_SEGMENTS as f64 * sweep / 360.0).ceil() as usize).max(2);
    (0..=n)
        .map(|i| {
            let a = (a0 + sweep * i as f64 / n as f64).to_radians();
            c + DVec3::new(r * a.cos(), r * a.sin(), 0.0)
        })
        .collect()
}

/// A polyline vertex list with bulges (LWPOLYLINE / POLYLINE): arc segments are sampled.
fn bulge_points(verts: &[(DVec3, f64)], closed: bool) -> Vec<DVec3> {
    let mut out = Vec::new();
    let n = verts.len();
    for i in 0..n {
        let (p, bulge) = verts[i];
        out.push(p);
        if !closed && i + 1 == n {
            break;
        }
        let q = verts[(i + 1) % n].0;
        if bulge.abs() < 1e-9 || p.truncate().distance(q.truncate()) < 1e-9 {
            continue;
        }
        // Bulge = tan(sweep / 4), positive = counter-clockwise; the center is to the left of
        // the chord by L (1 - b²) / 4b.
        let sweep = 4.0 * bulge.atan();
        let chord = q - p;
        let len = chord.truncate().length();
        let left = DVec3::new(-chord.y, chord.x, 0.0) / len;
        let c = (p + q) * 0.5 + left * len * (1.0 - bulge * bulge) / (4.0 * bulge);
        let r = len * (1.0 + bulge * bulge) / (4.0 * bulge.abs());
        let a0 = (p.y - c.y).atan2(p.x - c.x);
        let segs = ((CIRCLE_SEGMENTS as f64 * sweep.abs() / std::f64::consts::TAU).ceil() as usize).max(2);
        for k in 1..segs {
            let a = a0 + sweep * k as f64 / segs as f64;
            out.push(DVec3::new(c.x + r * a.cos(), c.y + r * a.sin(), p.z));
        }
    }
    out
}

/// Parse DXF text.
pub fn parse_dxf(text: &str) -> Result<DxfData, IoError> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < 2 {
        return Err(IoError::Format(tr!("kernel-io-not-a-dxf-file")));
    }
    let mut pairs = Vec::with_capacity(lines.len() / 2);
    for ch in lines.chunks(2) {
        let [c, v] = ch else { break };
        let code: i32 =
            c.trim().parse().map_err(|_| IoError::Format(format!("bad DXF group code: {}", c.trim())))?;
        pairs.push((code, v.trim_end_matches('\r').to_string()));
    }
    if !pairs.iter().any(|(c, v)| *c == 0 && v.trim() == "SECTION") {
        return Err(IoError::Format(tr!("kernel-io-not-a-dxf-file-no-section")));
    }
    // Units (header) and the entities, split at group code 0.
    let mut scale = 1.0;
    let mut entities: Vec<Entity> = Vec::new();
    let mut section = String::new();
    let mut i = 0;
    while i < pairs.len() {
        let (code, ref v) = pairs[i];
        let v = v.trim();
        if code == 0 && v == "SECTION" {
            section = pairs.get(i + 1).map(|p| p.1.trim().to_string()).unwrap_or_default();
            i += 2;
            continue;
        }
        if section == "HEADER" && code == 9 && v == "$INSUNITS" {
            scale = unit_scale(pairs.get(i + 1).and_then(|p| p.1.trim().parse().ok()).unwrap_or(0));
        }
        if section == "ENTITIES" && code == 0 {
            entities.push(Entity { kind: v.to_string(), codes: Vec::new() });
        } else if section == "ENTITIES"
            && let Some(e) = entities.last_mut()
        {
            e.codes.push((code, pairs[i].1.clone()));
        }
        i += 1;
    }
    let mut data = DxfData::default();
    let mut k = 0;
    while k < entities.len() {
        let e = &entities[k];
        k += 1;
        match e.kind.as_str() {
            "LINE" => data.polylines.push((vec![e.point(0), e.point(1)], false)),
            "3DFACE" => {
                let mut pts = vec![e.point(0), e.point(1), e.point(2), e.point(3)];
                if pts[3].distance(pts[2]) < 1e-9 {
                    pts.pop();
                }
                data.faces.push(pts);
            }
            "CIRCLE" => {
                let mut pts = arc_points(e.point(0), e.f(40), 0.0, 360.0);
                pts.pop();
                data.polylines.push((pts, true));
            }
            "ARC" => data.polylines.push((arc_points(e.point(0), e.f(40), e.f(50), e.f(51)), false)),
            "LWPOLYLINE" => {
                let z = e.f(38);
                let mut verts: Vec<(DVec3, f64)> = Vec::new();
                let mut x = None;
                for (c, v) in &e.codes {
                    let val: f64 = v.trim().parse().unwrap_or(0.0);
                    match c {
                        10 => x = Some(val),
                        20 => {
                            if let Some(x) = x.take() {
                                verts.push((DVec3::new(x, val, z), 0.0));
                            }
                        }
                        42 => {
                            if let Some(last) = verts.last_mut() {
                                last.1 = val;
                            }
                        }
                        _ => {}
                    }
                }
                let closed = e.flags() & 1 != 0;
                data.polylines.push((bulge_points(&verts, closed), closed));
            }
            "POLYLINE" => {
                let flags = e.flags();
                let mut verts = Vec::new();
                while k < entities.len() && entities[k].kind == "VERTEX" {
                    verts.push(&entities[k]);
                    k += 1;
                }
                if entities.get(k).is_some_and(|e| e.kind == "SEQEND") {
                    k += 1;
                }
                if flags & 64 != 0 {
                    // Polyface mesh: position vertices, then faces by 1-based indices (71..74).
                    let pos: Vec<DVec3> = verts.iter().filter(|v| v.flags() & 128 == 0 || v.flags() & 64 != 0).map(|v| v.point(0)).collect();
                    for v in verts.iter().filter(|v| v.flags() & 128 != 0 && v.flags() & 64 == 0) {
                        let idx: Vec<usize> = (71..=74)
                            .filter_map(|c| v.get(c))
                            .map(|i| i.abs() as usize)
                            .filter(|i| *i >= 1 && *i <= pos.len())
                            .collect();
                        if idx.len() >= 3 {
                            data.faces.push(idx.iter().map(|i| pos[i - 1]).collect());
                        }
                    }
                } else {
                    let vs: Vec<(DVec3, f64)> = verts.iter().map(|v| (v.point(0), v.f(42))).collect();
                    let closed = flags & 1 != 0;
                    data.polylines.push((bulge_points(&vs, closed), closed));
                }
            }
            _ => {}
        }
    }
    let s = |p: &mut DVec3| *p *= scale;
    data.polylines.iter_mut().for_each(|(pts, _)| pts.iter_mut().for_each(s));
    data.faces.iter_mut().for_each(|f| f.iter_mut().for_each(s));
    Ok(data)
}

/// Import a DXF file into the root mesh.
pub fn import_dxf(model: &mut Model, path: &Path) -> Result<(), IoError> {
    import_dxf_with(model, path, &ImportOptions::default())
}

pub fn import_dxf_with(model: &mut Model, path: &Path, opts: &ImportOptions) -> Result<(), IoError> {
    let bytes = super::read_limited(path, super::MAX_MODEL_BYTES)?;
    let data = parse_dxf(&String::from_utf8_lossy(&bytes))?;
    add_dxf(model, &data, opts)
}

/// Add parsed DXF geometry to the root mesh.
pub fn add_dxf(model: &mut Model, data: &DxfData, opts: &ImportOptions) -> Result<(), IoError> {
    if data.polylines.is_empty() && data.faces.is_empty() {
        return Err(IoError::Format(tr!("kernel-io-no-supported-entities-line-polyline-dface")));
    }
    let mesh = model.root_mesh_mut();
    let mut surface = super::surface::SurfaceBuilder::from_mesh(mesh);
    for f in &data.faces {
        let mut pts: Vec<DVec3> = Vec::with_capacity(f.len());
        for p in f {
            if pts.last().is_none_or(|q: &DVec3| q.distance(*p) > 1e-9) {
                pts.push(*p);
            }
        }
        if pts.len() > 3 && pts[0].distance(pts[pts.len() - 1]) < 1e-9 {
            pts.pop();
        }
        surface.polygon(mesh, &pts)?;
    }
    surface.finish(mesh, opts);
    for (pts, closed) in &data.polylines {
        let mut pts = pts.clone();
        pts.dedup_by(|a, b| a.distance(*b) < 1e-9);
        if *closed && pts.len() > 2 && pts[0].distance(pts[pts.len() - 1]) < 1e-9 {
            pts.pop();
        }
        if pts.len() >= 2 {
            mesh.add_polyline(&pts, *closed && pts.len() > 2, false);
        }
    }
    Ok(())
}
