//! glTF 2.0: binary `.glb` export, `.gltf` / `.glb` import.
//!
//! Varstotas is Z-up in millimetres, glTF is Y-up in metres: `(x, y, z)` mm ↔ `(x, z, -y)` m.
//! Export: one node per top-level group / component (loose geometry: one "Model" node), one
//! primitive per material, world-space triangles with normals (smooth where edges are smooth)
//! and texture coordinates, materials as PBR base colours (+ embedded textures), double-sided.
//! Import: every mesh node becomes a group (world transform baked in); triangles are merged
//! back into polygons; base colours and embedded / external textures become materials.

use super::stl::{add_polygon, new_faces};
use super::{ImportOptions, IoError};
use crate::mesh::Mesh;
use crate::model::Definition;
use crate::texture::{TexPlacement, TextureImage, base64_decode};
use crate::{FaceId, Instance, MaterialId, Model, tessellate};
use glam::{DMat4, DQuat, DVec2, DVec3};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::path::Path;

const GLB_MAGIC: u32 = 0x4654_6C67;
const CHUNK_JSON: u32 = 0x4E4F_534A;
const CHUNK_BIN: u32 = 0x004E_4942;

/// Varstotas (Z-up, mm) → glTF (Y-up, m).
fn to_gltf(p: DVec3) -> [f32; 3] {
    [(p.x * 1e-3) as f32, (p.z * 1e-3) as f32, (-p.y * 1e-3) as f32]
}

fn dir_to_gltf(n: DVec3) -> [f32; 3] {
    [n.x as f32, n.z as f32, -n.y as f32]
}

/// glTF (Y-up, m) → Varstotas (Z-up, mm).
fn from_gltf(p: DVec3) -> DVec3 {
    DVec3::new(p.x, -p.z, p.y) * 1e3
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
}

/// Binary buffer under construction: 4-byte aligned views.
#[derive(Default)]
struct Bin {
    data: Vec<u8>,
    views: Vec<Value>,
    accessors: Vec<Value>,
}

impl Bin {
    fn view(&mut self, bytes: &[u8], target: Option<u32>) -> usize {
        while !self.data.len().is_multiple_of(4) {
            self.data.push(0);
        }
        let mut v = json!({"buffer": 0, "byteOffset": self.data.len(), "byteLength": bytes.len()});
        if let Some(t) = target {
            v["target"] = json!(t);
        }
        self.data.extend_from_slice(bytes);
        self.views.push(v);
        self.views.len() - 1
    }

    /// Float accessor of `n`-component vectors.
    fn floats(&mut self, vals: &[f32], n: usize, bounds: bool) -> usize {
        let bytes: Vec<u8> = vals.iter().flat_map(|v| v.to_le_bytes()).collect();
        let view = self.view(&bytes, Some(34962));
        let ty = ["", "SCALAR", "VEC2", "VEC3", "VEC4"][n];
        let mut a = json!({"bufferView": view, "componentType": 5126, "count": vals.len() / n, "type": ty});
        if bounds && !vals.is_empty() {
            let mut lo = vec![f32::MAX; n];
            let mut hi = vec![f32::MIN; n];
            for c in vals.chunks(n) {
                for k in 0..n {
                    lo[k] = lo[k].min(c[k]);
                    hi[k] = hi[k].max(c[k]);
                }
            }
            a["min"] = json!(lo);
            a["max"] = json!(hi);
        }
        self.accessors.push(a);
        self.accessors.len() - 1
    }

    fn indices(&mut self, idx: &[u32]) -> usize {
        let bytes: Vec<u8> = idx.iter().flat_map(|v| v.to_le_bytes()).collect();
        let view = self.view(&bytes, Some(34963));
        self.accessors.push(json!({"bufferView": view, "componentType": 5125, "count": idx.len(), "type": "SCALAR"}));
        self.accessors.len() - 1
    }
}

/// One primitive being built: deduplicated vertices.
#[derive(Default)]
struct Prim {
    pos: Vec<f32>,
    nrm: Vec<f32>,
    uv: Vec<f32>,
    idx: Vec<u32>,
    seen: HashMap<[u32; 8], u32>,
}

impl Prim {
    fn vertex(&mut self, p: [f32; 3], n: [f32; 3], uv: [f32; 2]) {
        let key = [p[0], p[1], p[2], n[0], n[1], n[2], uv[0], uv[1]].map(f32::to_bits);
        let next = (self.pos.len() / 3) as u32;
        let i = *self.seen.entry(key).or_insert_with(|| {
            self.pos.extend(p);
            self.nrm.extend(n);
            self.uv.extend(uv);
            next
        });
        self.idx.push(i);
    }
}

/// GLB bytes of the visible model.
pub fn write_glb(model: &Model) -> Vec<u8> {
    let tess = tessellate(model);
    let root = model.root_mesh();
    // Nodes: top-level owner (None = loose geometry) → primitives per material.
    type Node = (Option<crate::InstanceId>, Vec<(Option<MaterialId>, Prim)>);
    let mut nodes: Vec<Node> = Vec::new();
    for f in &tess.faces {
        let owner = f.owner_path.first().copied();
        let mat = f.front.or(f.inherited).filter(|m| model.materials.contains_key(*m));
        let ni = match nodes.iter().position(|n| n.0 == owner) {
            Some(i) => i,
            None => {
                nodes.push((owner, Vec::new()));
                nodes.len() - 1
            }
        };
        let prims = &mut nodes[ni].1;
        let pi = match prims.iter().position(|p| p.0 == mat) {
            Some(i) => i,
            None => {
                prims.push((mat, Prim::default()));
                prims.len() - 1
            }
        };
        let tile = mat.and_then(|m| model.materials[m].image.as_ref()).map(|i| i.size).filter(|s| s[0] > 0.0 && s[1] > 0.0);
        let prim = &mut prims[pi].1;
        for (k, p) in f.positions.iter().enumerate() {
            let n = f.normals.get(k).copied().unwrap_or(f.normal);
            let uv = match (tile, f.uv.get(k)) {
                // glTF's v grows downward in the image.
                (Some([w, h]), Some(t)) => [(t.x / w) as f32, (-t.y / h) as f32],
                _ => [0.0, 0.0],
            };
            prim.vertex(to_gltf(*p), dir_to_gltf(n), uv);
        }
    }
    let mut bin = Bin::default();
    let mut materials: Vec<Value> = Vec::new();
    let mut textures: Vec<Value> = Vec::new();
    let mut images: Vec<Value> = Vec::new();
    let mut mat_index: HashMap<MaterialId, usize> = HashMap::new();
    let mut meshes = Vec::new();
    let mut node_list = Vec::new();
    for (owner, prims) in nodes {
        let mut out = Vec::new();
        for (mat, p) in prims {
            if p.idx.is_empty() {
                continue;
            }
            let pos = bin.floats(&p.pos, 3, true);
            let nrm = bin.floats(&p.nrm, 3, false);
            let mut attrs = json!({"POSITION": pos, "NORMAL": nrm});
            let textured = mat.is_some_and(|m| model.materials[m].image.is_some());
            if textured {
                attrs["TEXCOORD_0"] = json!(bin.floats(&p.uv, 2, false));
            }
            let ind = bin.indices(&p.idx);
            let mut prim = json!({"attributes": attrs, "indices": ind, "mode": 4});
            if let Some(m) = mat {
                let mi = *mat_index.entry(m).or_insert_with(|| {
                    let mm = &model.materials[m];
                    let c = mm.color;
                    let mut pbr = json!({
                        "baseColorFactor": [srgb_to_linear(c[0]), srgb_to_linear(c[1]), srgb_to_linear(c[2]), c[3]],
                        "metallicFactor": 0.0,
                        "roughnessFactor": 0.9,
                    });
                    if let Some(img) = &mm.image {
                        let view = bin.view(&img.bytes, None);
                        let mime = if img.extension() == "png" { "image/png" } else { "image/jpeg" };
                        images.push(json!({"bufferView": view, "mimeType": mime, "name": img.file_name}));
                        textures.push(json!({"source": images.len() - 1, "sampler": 0}));
                        pbr["baseColorTexture"] = json!({"index": textures.len() - 1});
                        // The texture carries the colour.
                        pbr["baseColorFactor"] = json!([1.0, 1.0, 1.0, c[3]]);
                    }
                    let mut v = json!({"name": mm.name, "pbrMetallicRoughness": pbr, "doubleSided": true});
                    if c[3] < 0.999 {
                        v["alphaMode"] = json!("BLEND");
                    }
                    materials.push(v);
                    materials.len() - 1
                });
                prim["material"] = json!(mi);
            }
            out.push(prim);
        }
        if out.is_empty() {
            continue;
        }
        let name = match owner.and_then(|i| root.instances.get(i)) {
            Some(i) if !i.name.is_empty() => i.name.clone(),
            Some(i) => model.defs.get(i.def).map(|d| d.name.clone()).unwrap_or_default(),
            None => "Model".into(),
        };
        meshes.push(json!({"name": name, "primitives": out}));
        node_list.push(json!({"name": name, "mesh": meshes.len() - 1}));
    }
    let mut doc = json!({
        "asset": {"version": "2.0", "generator": "Varstotas"},
        "scene": 0,
        "scenes": [{"nodes": (0..node_list.len()).collect::<Vec<_>>()}],
        "nodes": node_list,
        "meshes": meshes,
        "accessors": bin.accessors,
        "bufferViews": bin.views,
    });
    if !materials.is_empty() {
        doc["materials"] = json!(materials);
    }
    if !images.is_empty() {
        doc["images"] = json!(images);
        doc["textures"] = json!(textures);
        doc["samplers"] = json!([{"wrapS": 10497, "wrapT": 10497, "magFilter": 9729, "minFilter": 9987}]);
    }
    while !bin.data.len().is_multiple_of(4) {
        bin.data.push(0);
    }
    if !bin.data.is_empty() {
        doc["buffers"] = json!([{"byteLength": bin.data.len()}]);
    }
    let mut js = serde_json::to_vec(&doc).unwrap_or_default();
    while !js.len().is_multiple_of(4) {
        js.push(b' ');
    }
    let total = 12 + 8 + js.len() + if bin.data.is_empty() { 0 } else { 8 + bin.data.len() };
    let mut out = Vec::with_capacity(total);
    out.extend(GLB_MAGIC.to_le_bytes());
    out.extend(2u32.to_le_bytes());
    out.extend((total as u32).to_le_bytes());
    out.extend((js.len() as u32).to_le_bytes());
    out.extend(CHUNK_JSON.to_le_bytes());
    out.extend(js);
    if !bin.data.is_empty() {
        out.extend((bin.data.len() as u32).to_le_bytes());
        out.extend(CHUNK_BIN.to_le_bytes());
        out.extend(bin.data);
    }
    out
}

pub fn export_glb(model: &Model, path: &Path) -> Result<(), IoError> {
    std::fs::write(path, write_glb(model))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------

fn bad(msg: impl Into<String>) -> IoError {
    IoError::Format(msg.into())
}

/// A parsed glTF document with its buffers loaded.
struct Gltf {
    doc: Value,
    buffers: Vec<Vec<u8>>,
}

/// Split a GLB into its JSON and binary chunk, or read `.gltf` JSON.
fn split(data: &[u8]) -> Result<(Value, Option<Vec<u8>>), IoError> {
    if data.len() >= 12 && u32::from_le_bytes([data[0], data[1], data[2], data[3]]) == GLB_MAGIC {
        let mut at = 12;
        let mut json = None;
        let mut bin = None;
        while at + 8 <= data.len() {
            let len = u32::from_le_bytes(data[at..at + 4].try_into().unwrap()) as usize;
            let ty = u32::from_le_bytes(data[at + 4..at + 8].try_into().unwrap());
            let body = data.get(at + 8..at + 8 + len).ok_or_else(|| bad("truncated GLB chunk"))?;
            match ty {
                CHUNK_JSON => json = Some(serde_json::from_slice(body).map_err(|e| bad(format!("GLB JSON: {e}")))?),
                CHUNK_BIN => bin = Some(body.to_vec()),
                _ => {}
            }
            at += 8 + len;
        }
        return Ok((json.ok_or_else(|| bad("GLB without JSON"))?, bin));
    }
    let doc: Value = serde_json::from_slice(data).map_err(|e| bad(format!("not a glTF file: {e}")))?;
    Ok((doc, None))
}

/// Bytes of a `uri`: a base64 data URI or a file next to the glTF.
fn load_uri(uri: &str, dir: Option<&Path>) -> Result<Vec<u8>, IoError> {
    if let Some(rest) = uri.strip_prefix("data:") {
        let (_, b64) = rest.split_once(";base64,").ok_or_else(|| bad("unsupported data URI"))?;
        return base64_decode(b64).ok_or_else(|| bad("bad base64 in data URI"));
    }
    let dir = dir.ok_or_else(|| bad(format!("external file {uri} needs a path")))?;
    let name = uri.replace("%20", " ");
    Ok(std::fs::read(dir.join(name))?)
}

impl Gltf {
    fn parse(data: &[u8], dir: Option<&Path>) -> Result<Self, IoError> {
        let (doc, bin) = split(data)?;
        if doc["asset"]["version"].as_str().is_some_and(|v| !v.starts_with('2')) {
            return Err(bad("only glTF 2.0 is supported"));
        }
        let mut buffers = Vec::new();
        let mut bin = bin;
        for b in doc["buffers"].as_array().into_iter().flatten() {
            buffers.push(match b["uri"].as_str() {
                Some(uri) => load_uri(uri, dir)?,
                None => bin.take().ok_or_else(|| bad("buffer without data"))?,
            });
        }
        Ok(Self { doc, buffers })
    }

    fn view_bytes(&self, view: usize) -> Result<(&[u8], usize), IoError> {
        let v = &self.doc["bufferViews"][view];
        let buf = self.buffers.get(v["buffer"].as_u64().unwrap_or(0) as usize).ok_or_else(|| bad("bad buffer index"))?;
        let off = v["byteOffset"].as_u64().unwrap_or(0) as usize;
        let len = v["byteLength"].as_u64().ok_or_else(|| bad("bufferView without byteLength"))? as usize;
        let stride = v["byteStride"].as_u64().unwrap_or(0) as usize;
        Ok((buf.get(off..off + len).ok_or_else(|| bad("bufferView out of range"))?, stride))
    }

    /// Accessor values as f64 vectors of its component count (normalized integers scaled).
    fn read(&self, acc: usize) -> Result<Vec<Vec<f64>>, IoError> {
        let a = &self.doc["accessors"][acc];
        let count = a["count"].as_u64().ok_or_else(|| bad("accessor without count"))? as usize;
        let n = match a["type"].as_str() {
            Some("SCALAR") => 1,
            Some("VEC2") => 2,
            Some("VEC3") => 3,
            Some("VEC4") => 4,
            Some("MAT4") => 16,
            _ => return Err(bad("unsupported accessor type")),
        };
        let ct = a["componentType"].as_u64().unwrap_or(5126);
        let size = match ct {
            5120 | 5121 => 1,
            5122 | 5123 => 2,
            5125 | 5126 => 4,
            _ => return Err(bad("unsupported component type")),
        };
        let norm = a["normalized"].as_bool().unwrap_or(false);
        let Some(view) = a["bufferView"].as_u64() else {
            return Ok(vec![vec![0.0; n]; count]);
        };
        let (bytes, stride) = self.view_bytes(view as usize)?;
        let off = a["byteOffset"].as_u64().unwrap_or(0) as usize;
        let stride = if stride == 0 { n * size } else { stride };
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            let base = off + i * stride;
            let mut v = Vec::with_capacity(n);
            for k in 0..n {
                let o = base + k * size;
                let b = bytes.get(o..o + size).ok_or_else(|| bad("accessor out of range"))?;
                let x = match ct {
                    5120 => b[0] as i8 as f64 / if norm { 127.0 } else { 1.0 },
                    5121 => b[0] as f64 / if norm { 255.0 } else { 1.0 },
                    5122 => i16::from_le_bytes([b[0], b[1]]) as f64 / if norm { 32767.0 } else { 1.0 },
                    5123 => u16::from_le_bytes([b[0], b[1]]) as f64 / if norm { 65535.0 } else { 1.0 },
                    5125 => u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64,
                    _ => f32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64,
                };
                v.push(x);
            }
            out.push(v);
        }
        Ok(out)
    }

    fn image(&self, tex: usize, dir: Option<&Path>) -> Option<(Vec<u8>, String)> {
        let src = self.doc["textures"][tex]["source"].as_u64()? as usize;
        let img = &self.doc["images"][src];
        let name = img["name"].as_str().or(img["uri"].as_str()).unwrap_or("texture").to_string();
        let bytes = match (img["bufferView"].as_u64(), img["uri"].as_str()) {
            (Some(v), _) => self.view_bytes(v as usize).ok()?.0.to_vec(),
            (None, Some(uri)) => load_uri(uri, dir).ok()?,
            _ => return None,
        };
        crate::texture::image_dimensions(&bytes).map(|_| (bytes, name))
    }
}

fn node_matrix(n: &Value) -> DMat4 {
    if let Some(m) = n["matrix"].as_array().filter(|m| m.len() == 16) {
        let v: Vec<f64> = m.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect();
        return DMat4::from_cols_slice(&v);
    }
    let vec = |k: &str, d: &[f64]| -> Vec<f64> {
        n[k].as_array().map(|a| a.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect()).unwrap_or_else(|| d.to_vec())
    };
    let t = vec("translation", &[0.0; 3]);
    let r = vec("rotation", &[0.0, 0.0, 0.0, 1.0]);
    let s = vec("scale", &[1.0; 3]);
    if t.len() < 3 || r.len() < 4 || s.len() < 3 {
        return DMat4::IDENTITY;
    }
    DMat4::from_scale_rotation_translation(
        DVec3::new(s[0], s[1], s[2]),
        DQuat::from_xyzw(r[0], r[1], r[2], r[3]).normalize(),
        DVec3::new(t[0], t[1], t[2]),
    )
}

/// Import a glTF / GLB file: each mesh node becomes a group of the root.
pub fn import_gltf(model: &mut Model, path: &Path) -> Result<(), IoError> {
    import_gltf_with(model, path, &ImportOptions::default())
}

pub fn import_gltf_with(model: &mut Model, path: &Path, opts: &ImportOptions) -> Result<(), IoError> {
    let data = std::fs::read(path)?;
    add_gltf(model, &data, path.parent(), opts)
}

/// Add glTF / GLB bytes to the model (`dir`: where external files are).
pub fn add_gltf(model: &mut Model, data: &[u8], dir: Option<&Path>, opts: &ImportOptions) -> Result<(), IoError> {
    let g = Gltf::parse(data, dir)?;
    // Nodes of the default scene (or all roots), with world matrices.
    let nodes = g.doc["nodes"].as_array().cloned().unwrap_or_default();
    let roots: Vec<usize> = match g.doc["scenes"][g.doc["scene"].as_u64().unwrap_or(0) as usize]["nodes"].as_array() {
        Some(r) => r.iter().filter_map(|v| v.as_u64().map(|x| x as usize)).collect(),
        None => {
            let children: HashSet<usize> = nodes
                .iter()
                .flat_map(|n| n["children"].as_array().cloned().unwrap_or_default())
                .filter_map(|v| v.as_u64().map(|x| x as usize))
                .collect();
            (0..nodes.len()).filter(|i| !children.contains(i)).collect()
        }
    };
    let mut stack: Vec<(usize, DMat4, usize)> = roots.into_iter().map(|r| (r, DMat4::IDENTITY, 0)).collect();
    let mut placed: Vec<(usize, DMat4, String)> = Vec::new();
    while let Some((i, parent, depth)) = stack.pop() {
        let Some(n) = nodes.get(i) else { continue };
        if depth > 64 {
            return Err(bad("node hierarchy too deep"));
        }
        let m = parent * node_matrix(n);
        if let Some(mesh) = n["mesh"].as_u64() {
            placed.push((mesh as usize, m, n["name"].as_str().unwrap_or("").to_string()));
        }
        for c in n["children"].as_array().into_iter().flatten().filter_map(|v| v.as_u64()) {
            stack.push((c as usize, m, depth + 1));
        }
    }
    if placed.is_empty() {
        return Err(bad("no meshes in the glTF scene"));
    }
    placed.reverse();
    // Materials (created when first used).
    let mut mats: HashMap<usize, Option<MaterialId>> = HashMap::new();
    let mut get_mat = |model: &mut Model, i: usize| -> Option<MaterialId> {
        *mats.entry(i).or_insert_with(|| {
            if !opts.materials {
                return None;
            }
            let m = &g.doc["materials"][i];
            let pbr = &m["pbrMetallicRoughness"];
            let f: Vec<f32> = pbr["baseColorFactor"]
                .as_array()
                .map(|a| a.iter().map(|x| x.as_f64().unwrap_or(1.0) as f32).collect())
                .unwrap_or_else(|| vec![1.0; 4]);
            let c = [linear_to_srgb(f[0]), linear_to_srgb(f[1]), linear_to_srgb(f[2]), f.get(3).copied().unwrap_or(1.0)];
            let name = m["name"].as_str().filter(|s| !s.is_empty()).map(str::to_string).unwrap_or_else(|| format!("Material {i}"));
            let img = pbr["baseColorTexture"]["index"].as_u64().and_then(|t| g.image(t as usize, dir));
            let id = model.add_material(&name, c);
            if let Some((bytes, file)) = img {
                // 1 m tiles: glTF texture coordinates are per metre or per face, fitted below.
                let mut ti = TextureImage::new(bytes, file, 1000.0);
                ti.size = [1000.0, 1000.0];
                model.materials[id].image = Some(ti);
            }
            Some(id)
        })
    };
    let pd = model.root;
    for (mi, m, name) in placed {
        let gm = &g.doc["meshes"][mi];
        let mut mesh = Mesh::new();
        for p in gm["primitives"].as_array().into_iter().flatten() {
            let mode = p["mode"].as_u64().unwrap_or(4);
            if !matches!(mode, 4..=6) {
                continue;
            }
            let Some(pa) = p["attributes"]["POSITION"].as_u64() else { continue };
            let pos: Vec<DVec3> = g
                .read(pa as usize)?
                .iter()
                .map(|v| from_gltf(m.transform_point3(DVec3::new(v[0], v[1], v[2]))))
                .collect();
            let uv: Option<Vec<DVec2>> = match p["attributes"]["TEXCOORD_0"].as_u64() {
                Some(a) => Some(g.read(a as usize)?.iter().map(|v| DVec2::new(v[0], -v[1])).collect()),
                None => None,
            };
            let idx: Vec<usize> = match p["indices"].as_u64() {
                Some(a) => g.read(a as usize)?.iter().map(|v| v[0] as usize).collect(),
                None => (0..pos.len()).collect(),
            };
            let tris: Vec<[usize; 3]> = match mode {
                5 => (2..idx.len()).map(|k| if k % 2 == 0 { [idx[k - 2], idx[k - 1], idx[k]] } else { [idx[k - 1], idx[k - 2], idx[k]] }).collect(),
                6 => (2..idx.len()).map(|k| [idx[0], idx[k - 1], idx[k]]).collect(),
                _ => idx.as_chunks::<3>().0.to_vec(),
            };
            // A mirroring transform flips the winding.
            let flip = m.determinant() < 0.0;
            let mat = p["material"].as_u64().and_then(|i| get_mat(model, i as usize));
            let tile = mat.and_then(|id| model.materials[id].image.as_ref()).map(|i| i.size);
            for t in tris {
                if t.iter().any(|&k| k >= pos.len()) {
                    return Err(bad("index out of range"));
                }
                let t = if flip { [t[0], t[2], t[1]] } else { t };
                let pts = [pos[t[0]], pos[t[1]], pos[t[2]]];
                for f in add_polygon(&mut mesh, &pts) {
                    let Some(face) = mesh.faces.get_mut(f) else { continue };
                    face.front = mat;
                    if let (Some(tile), Some(uv)) = (tile, uv.as_ref())
                        && t.iter().all(|&k| k < uv.len())
                    {
                        let uvs = [uv[t[0]], uv[t[1]], uv[t[2]]];
                        face.uv = TexPlacement::fit(&pts, &uvs, face.normal, tile).filter(|pl| !pl.is_identity());
                    }
                }
            }
        }
        if mesh.faces.is_empty() {
            continue;
        }
        if opts.merge_coplanar {
            let made = new_faces(&mesh, &HashSet::<FaceId>::new());
            mesh.merge_coplanar_faces(&made);
        }
        let def = model.defs.insert(Definition { name: "Group".into(), mesh, is_group: true, guides: Default::default() });
        let inst = Instance { def, transform: DMat4::IDENTITY, name, material: None, tag: None, hidden: false, locked: false };
        model.defs[pd].mesh.instances.insert(inst);
    }
    Ok(())
}
