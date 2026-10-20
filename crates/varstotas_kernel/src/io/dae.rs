//! COLLADA (`.dae`) import: geometries (triangles, polylist, polygons) placed by the visual
//! scene's nodes (matrix / translate / rotate / scale, instance_node), materials from the
//! effects (diffuse colour or image), the asset's unit and up axis. Each placed geometry
//! becomes a group.
//!
//! Export (COLLADA 1.4.1): millimetres, Z up (said in `<asset>`, so coordinates stay as they
//! are); one node per top-level group / component (loose geometry: one "Model" node) with
//! world-space triangles, one `<triangles>` per material; normals (smooth where edges are
//! smooth) and texture coordinates; materials as Lambert colours or colour textures (unpainted
//! faces: no material), the images as files next to the `.dae`.

use super::meshes::{ImpMat, ImpMesh, ImpPoly, add_meshes};
use super::obj::TextureFile;
use super::threemf::{num, xml_escape};
use super::xml::El;
use super::{ImportOptions, IoError};
use crate::{MaterialId, Model, tessellate};
use std::fmt::Write as _;
use glam::{DMat4, DVec2, DVec3, DVec4};
use std::collections::HashMap;
use std::path::Path;

fn bad(s: impl Into<String>) -> IoError {
    IoError::Format(format!("COLLADA: {}", s.into()))
}

/// Elements by `id` (the whole document).
fn index<'a>(e: &'a El, out: &mut HashMap<String, &'a El>) {
    if let Some(id) = e.attr("id") {
        out.insert(id.to_string(), e);
    }
    for c in &e.children {
        index(c, out);
    }
}

struct Doc<'a> {
    ids: HashMap<String, &'a El>,
    dir: Option<&'a Path>,
    mats: Vec<ImpMat>,
    /// Material id → index into `mats`.
    mat_of: HashMap<String, usize>,
}

/// A source: its values grouped by the accessor stride.
fn source_values(s: &El) -> (Vec<f64>, usize) {
    let vals = s.child("float_array").map(El::numbers).unwrap_or_default();
    let stride = s.child("technique_common").and_then(|t| t.child("accessor")).and_then(|a| a.num("stride")).unwrap_or(3.0).max(1.0) as usize;
    (vals, stride)
}

fn url(s: Option<&str>) -> &str {
    s.unwrap_or("").trim_start_matches('#')
}

impl<'a> Doc<'a> {
    fn get(&self, id: &str) -> Option<&'a El> {
        self.ids.get(url(Some(id))).copied()
    }

    /// The material `id` (made once): the effect's diffuse colour or image.
    fn material(&mut self, id: &str) -> Option<usize> {
        if let Some(i) = self.mat_of.get(id) {
            return Some(*i);
        }
        let m = self.get(id)?;
        let name = m.attr("name").or(m.attr("id")).unwrap_or("Material").to_string();
        let mut color = [0.8, 0.8, 0.8, 1.0];
        let mut image = None;
        if let Some(fx) = m.child("instance_effect").and_then(|e| self.get(url(e.attr("url")))) {
            let mut tech = Vec::new();
            fx.find_all("technique", &mut tech);
            let shading = tech.iter().find_map(|t| ["phong", "lambert", "blinn", "constant"].iter().find_map(|k| t.child(k)));
            if let Some(d) = shading.and_then(|s| s.child("diffuse").or(s.child("emission"))) {
                if let Some(c) = d.child("color") {
                    let v = c.numbers();
                    if v.len() >= 3 {
                        color = [v[0] as f32, v[1] as f32, v[2] as f32, v.get(3).copied().unwrap_or(1.0) as f32];
                    }
                }
                if let Some(t) = d.child("texture") {
                    image = self.image(fx, t.attr("texture").unwrap_or(""));
                    if image.is_some() {
                        color = [1.0, 1.0, 1.0, 1.0];
                    }
                }
            }
            if let Some(tr) = shading.and_then(|s| s.child("transparency")).and_then(|t| t.child("float")).and_then(|f| f.text.trim().parse::<f32>().ok())
                && (0.0..1.0).contains(&tr)
                && tr > 0.0
            {
                color[3] = tr;
            }
        }
        self.mats.push(ImpMat { name, color, image });
        self.mat_of.insert(id.to_string(), self.mats.len() - 1);
        Some(self.mats.len() - 1)
    }

    /// A texture's image: sampler → surface → image, or the image id directly.
    fn image(&self, fx: &El, sampler: &str) -> Option<(Vec<u8>, String)> {
        let mut params = Vec::new();
        fx.find_all("newparam", &mut params);
        let param = |sid: &str| params.iter().find(|p| p.attr("sid") == Some(sid)).copied();
        let mut img_id = sampler.to_string();
        if let Some(s) = param(sampler).and_then(|p| p.child("sampler2D")) {
            img_id = s.child("source").map(|x| x.text.trim().to_string()).unwrap_or_default();
            if let Some(i) = s.child("instance_image") {
                img_id = url(i.attr("url")).to_string();
            }
            if let Some(f) = param(&img_id).and_then(|p| p.child("surface")).and_then(|s| s.child("init_from")) {
                img_id = f.text.trim().to_string();
            }
        }
        let img = self.get(&img_id)?;
        let from = img.child("init_from")?;
        let file = from.child("ref").map(|r| r.text.trim().to_string()).unwrap_or_else(|| from.text.trim().to_string());
        let file = file.trim_start_matches("file://").replace("%20", " ");
        let p = Path::new(&file);
        let path = if p.is_absolute() { p.to_path_buf() } else { self.dir?.join(p) };
        let bytes = std::fs::read(&path).ok().or_else(|| std::fs::read(self.dir?.join(p.file_name()?)).ok())?;
        Some((bytes, p.file_name()?.to_string_lossy().to_string()))
    }

    /// Geometry `id`'s polygons (in its own coordinates), materials bound by `binds`
    /// (symbol → material id).
    fn geometry(&mut self, id: &str, binds: &HashMap<String, String>) -> Result<Vec<ImpPoly>, IoError> {
        let Some(g) = self.get(id) else { return Ok(Vec::new()) };
        let Some(mesh) = g.child("mesh") else { return Ok(Vec::new()) };
        let sources: HashMap<&str, (Vec<f64>, usize)> = mesh.all("source").filter_map(|s| Some((s.attr("id")?, source_values(s)))).collect();
        // <vertices>: its POSITION source.
        let vertices: HashMap<&str, &str> = mesh
            .all("vertices")
            .filter_map(|v| Some((v.attr("id")?, url(v.all("input").find(|i| i.attr("semantic") == Some("POSITION"))?.attr("source")))))
            .collect();
        let mut out = Vec::new();
        for prim in &mesh.children {
            let kind = prim.name.as_str();
            if !matches!(kind, "triangles" | "polylist" | "polygons" | "trifans" | "tristrips") {
                continue;
            }
            let inputs: Vec<&El> = prim.all("input").collect();
            let stride = inputs.iter().filter_map(|i| i.num("offset")).fold(0.0, f64::max) as usize + 1;
            let find = |sem: &str| -> Option<(usize, &(Vec<f64>, usize))> {
                let i = inputs.iter().filter(|i| i.attr("semantic") == Some(sem)).min_by_key(|i| i.num("set").unwrap_or(0.0) as i64)?;
                let mut src = url(i.attr("source"));
                if sem == "VERTEX" {
                    src = vertices.get(src).copied().unwrap_or(src);
                }
                Some((i.num("offset").unwrap_or(0.0) as usize, sources.get(src)?))
            };
            let Some((po, (pv, ps))) = find("VERTEX") else { continue };
            let uv = find("TEXCOORD");
            let mat = prim.attr("material").map(|s| binds.get(s).cloned().unwrap_or_else(|| s.to_string())).and_then(|m| self.material(&m));
            // Index lists, one per polygon.
            let mut polys: Vec<Vec<usize>> = Vec::new();
            let p_all: Vec<Vec<usize>> = prim.all("p").map(|p| p.numbers().into_iter().map(|x| x as usize).collect()).collect();
            match kind {
                "triangles" => {
                    for p in &p_all {
                        polys.extend(p.chunks(stride * 3).filter(|c| c.len() == stride * 3).map(|c| c.to_vec()));
                    }
                }
                "polylist" => {
                    let counts: Vec<usize> = prim.child("vcount").map(|v| v.numbers().into_iter().map(|x| x as usize).collect()).unwrap_or_default();
                    let p = p_all.first().cloned().unwrap_or_default();
                    let mut at = 0;
                    for n in counts {
                        if at + n * stride > p.len() {
                            break;
                        }
                        polys.push(p[at..at + n * stride].to_vec());
                        at += n * stride;
                    }
                }
                "polygons" => polys = p_all,
                // Fans and strips: one <p> each, as triangles.
                _ => {
                    for p in &p_all {
                        let n = p.len() / stride;
                        let v = |k: usize| p[k * stride..(k + 1) * stride].to_vec();
                        for k in 2..n {
                            let t = if kind == "trifans" { [0, k - 1, k] } else if k % 2 == 0 { [k - 2, k - 1, k] } else { [k - 1, k - 2, k] };
                            polys.push(t.iter().flat_map(|i| v(*i)).collect());
                        }
                    }
                }
            }
            for idx in polys {
                let n = idx.len() / stride;
                let at = |vals: &[f64], st: usize, o: usize, k: usize, dims: usize| -> Option<Vec<f64>> {
                    let i = *idx.get(k * stride + o)?;
                    vals.get(i * st..i * st + dims).map(<[f64]>::to_vec)
                };
                let pts: Option<Vec<DVec3>> = (0..n).map(|k| at(pv, *ps, po, k, 3).map(|v| DVec3::new(v[0], v[1], v[2]))).collect();
                let Some(pts) = pts else { continue };
                let uvs = uv.and_then(|(o, s)| (0..n).map(|k| at(&s.0, s.1, o, k, 2).map(|v| DVec2::new(v[0], v[1]))).collect::<Option<Vec<_>>>());
                out.push(ImpPoly { pts, uv: uvs, mat });
            }
        }
        Ok(out)
    }

    /// A node's transform (its elements in order).
    fn node_matrix(n: &El) -> DMat4 {
        let mut m = DMat4::IDENTITY;
        for c in &n.children {
            let v = c.numbers();
            m *= match (c.name.as_str(), v.len()) {
                ("matrix", 16) => DMat4::from_cols(
                    DVec4::new(v[0], v[4], v[8], v[12]),
                    DVec4::new(v[1], v[5], v[9], v[13]),
                    DVec4::new(v[2], v[6], v[10], v[14]),
                    DVec4::new(v[3], v[7], v[11], v[15]),
                ),
                ("translate", 3) => DMat4::from_translation(DVec3::new(v[0], v[1], v[2])),
                ("rotate", 4) if DVec3::new(v[0], v[1], v[2]).length() > 0.0 => DMat4::from_axis_angle(DVec3::new(v[0], v[1], v[2]).normalize(), v[3].to_radians()),
                ("scale", 3) => DMat4::from_scale(DVec3::new(v[0], v[1], v[2])),
                _ => DMat4::IDENTITY,
            };
        }
        m
    }

    /// Walk node `n` (world `parent`), collecting placed geometries.
    fn node(&mut self, n: &'a El, parent: DMat4, out: &mut Vec<ImpMesh>, depth: usize) -> Result<(), IoError> {
        if depth > 64 {
            return Err(bad("nodes nested too deep"));
        }
        let m = parent * Self::node_matrix(n);
        for ig in n.all("instance_geometry") {
            let mut binds = HashMap::new();
            let mut ims = Vec::new();
            ig.find_all("instance_material", &mut ims);
            for im in ims {
                if let (Some(s), Some(t)) = (im.attr("symbol"), im.attr("target")) {
                    binds.insert(s.to_string(), url(Some(t)).to_string());
                }
            }
            let gid = url(ig.attr("url")).to_string();
            let mut polys = self.geometry(&gid, &binds)?;
            for p in &mut polys {
                for q in &mut p.pts {
                    *q = m.transform_point3(*q);
                }
                if m.determinant() < 0.0 {
                    p.pts.reverse();
                    if let Some(u) = p.uv.as_mut() {
                        u.reverse();
                    }
                }
            }
            let name = n.attr("name").or(n.attr("id")).or(self.get(&gid).and_then(|g| g.attr("name"))).unwrap_or("").to_string();
            out.push(ImpMesh { name, polys });
        }
        for c in n.all("node") {
            self.node(c, m, out, depth + 1)?;
        }
        for i in n.all("instance_node") {
            if let Some(t) = self.get(url(i.attr("url"))) {
                self.node(t, m, out, depth + 1)?;
            }
        }
        Ok(())
    }
}

pub fn import_dae(model: &mut Model, path: &Path) -> Result<(), IoError> {
    import_dae_with(model, path, &ImportOptions::default())
}

pub fn import_dae_with(model: &mut Model, path: &Path, opts: &ImportOptions) -> Result<(), IoError> {
    let text = std::fs::read(path)?;
    add_dae(model, &String::from_utf8_lossy(&text), path.parent(), opts)
}

/// Add a COLLADA document (`dir`: where its images are).
pub fn add_dae(model: &mut Model, text: &str, dir: Option<&Path>, opts: &ImportOptions) -> Result<(), IoError> {
    let root = El::parse(text)?;
    if root.name != "COLLADA" {
        return Err(bad("not a COLLADA document"));
    }
    let mut ids = HashMap::new();
    index(&root, &mut ids);
    let mut doc = Doc { ids, dir, mats: Vec::new(), mat_of: HashMap::new() };
    let asset = root.child("asset");
    let meter = asset.and_then(|a| a.child("unit")).and_then(|u| u.num("meter")).filter(|m| *m > 0.0).unwrap_or(1.0);
    let up = asset.and_then(|a| a.child("up_axis")).map(|u| u.text.trim().to_string()).unwrap_or_else(|| "Y_UP".into());
    let mut xf = DMat4::from_scale(DVec3::splat(meter * 1000.0));
    match up.as_str() {
        // (x, y, z) → (x, -z, y).
        "Y_UP" => xf = DMat4::from_rotation_x(std::f64::consts::FRAC_PI_2) * xf,
        // +X up → +Z.
        "X_UP" => xf = DMat4::from_rotation_y(-std::f64::consts::FRAC_PI_2) * xf,
        _ => {}
    }
    let mut meshes = Vec::new();
    let scene = root.child("scene").and_then(|s| s.child("instance_visual_scene")).and_then(|i| doc.get(url(i.attr("url")))).or_else(|| root.find("visual_scene"));
    match scene {
        Some(vs) => {
            for n in vs.all("node") {
                doc.node(n, DMat4::IDENTITY, &mut meshes, 0)?;
            }
        }
        None => {
            // No scene: every geometry where it is.
            let mut gs = Vec::new();
            root.find_all("geometry", &mut gs);
            for g in gs {
                let id = g.attr("id").unwrap_or("").to_string();
                let polys = doc.geometry(&id, &HashMap::new())?;
                meshes.push(ImpMesh { name: g.attr("name").unwrap_or("").to_string(), polys });
            }
        }
    }
    add_meshes(model, meshes, &doc.mats, xf, opts).map(|_| ())
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

/// One node's geometry: shared deduplicated vertices, triangles per material.
#[derive(Default)]
struct OutGeo {
    pos: Vec<f64>,
    nrm: Vec<f64>,
    uv: Vec<f64>,
    seen: HashMap<[u64; 8], u32>,
    tris: Vec<(Option<MaterialId>, Vec<u32>)>,
}

impl OutGeo {
    fn vertex(&mut self, mat: Option<MaterialId>, p: DVec3, n: DVec3, uv: DVec2) {
        let key = [p.x, p.y, p.z, n.x, n.y, n.z, uv.x, uv.y].map(f64::to_bits);
        let next = (self.pos.len() / 3) as u32;
        let i = *self.seen.entry(key).or_insert_with(|| {
            self.pos.extend([p.x, p.y, p.z]);
            self.nrm.extend([n.x, n.y, n.z]);
            self.uv.extend([uv.x, uv.y]);
            next
        });
        match self.tris.iter_mut().find(|t| t.0 == mat) {
            Some(t) => t.1.push(i),
            None => self.tris.push((mat, vec![i])),
        }
    }
}

/// `seconds` since 1970 as `YYYY-MM-DDThh:mm:ssZ`.
fn iso_time(seconds: u64) -> String {
    let (days, rem) = ((seconds / 86_400) as i64, seconds % 86_400);
    // Civil date from days (H. Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem / 60 % 60, rem % 60)
}

fn nums(v: &[f64]) -> String {
    v.iter().map(|x| num(*x)).collect::<Vec<_>>().join(" ")
}

/// COLLADA text of the visible model and the image files it references: (file name, bytes).
pub fn write_dae(model: &Model) -> (String, Vec<TextureFile>) {
    let tess = tessellate(model);
    let root = model.root_mesh();
    let mut nodes: Vec<(Option<crate::InstanceId>, OutGeo)> = Vec::new();
    for f in &tess.faces {
        let owner = f.owner_path.first().copied();
        let mat = f.front.or(f.inherited).filter(|m| model.materials.contains_key(*m));
        let ni = match nodes.iter().position(|n| n.0 == owner) {
            Some(i) => i,
            None => {
                nodes.push((owner, OutGeo::default()));
                nodes.len() - 1
            }
        };
        let tile = mat.and_then(|m| model.materials[m].image.as_ref()).map(|i| i.size).filter(|s| s[0] > 0.0 && s[1] > 0.0);
        for (k, p) in f.positions.iter().enumerate() {
            let n = f.normals.get(k).copied().unwrap_or(f.normal);
            let uv = match (tile, f.uv.get(k)) {
                (Some([w, h]), Some(t)) => DVec2::new(t.x / w, t.y / h),
                _ => DVec2::ZERO,
            };
            nodes[ni].1.vertex(mat, *p, n, uv);
        }
    }
    // Materials in the order they are first used, with unique ids.
    let mut mats: Vec<MaterialId> = Vec::new();
    for (_, g) in &nodes {
        for (m, _) in &g.tris {
            if let Some(m) = m
                && !mats.contains(m)
            {
                mats.push(*m);
            }
        }
    }
    let mat_id = |m: MaterialId| format!("material{}", mats.iter().position(|x| *x == m).unwrap_or(0));
    let mut images = String::new();
    let mut effects = String::new();
    let mut materials = String::new();
    let mut files: Vec<TextureFile> = Vec::new();
    let mut effect = |id: &str, name: &str, color: [f32; 4], image: Option<&crate::texture::TextureImage>| {
        let diffuse = match image {
            Some(img) => {
                let base: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
                let mut file = format!("{base}.{}", img.extension());
                let mut k = 2;
                while files.iter().any(|f| f.0 == file) {
                    file = format!("{base}_{k}.{}", img.extension());
                    k += 1;
                }
                let _ = writeln!(images, "    <image id=\"{id}-image\" name=\"{}\"><init_from>{}</init_from></image>", xml_escape(name), xml_escape(&file));
                files.push((file, img.bytes.clone()));
                format!("<texture texture=\"{id}-sampler\" texcoord=\"UVSET0\"/>")
            }
            None => format!("<color sid=\"diffuse\">{} {} {} {}</color>", num(color[0] as f64), num(color[1] as f64), num(color[2] as f64), num(color[3] as f64)),
        };
        let params = if image.is_some() {
            format!(
                "        <newparam sid=\"{id}-surface\"><surface type=\"2D\"><init_from>{id}-image</init_from></surface></newparam>\n        <newparam sid=\"{id}-sampler\"><sampler2D><source>{id}-surface</source></sampler2D></newparam>\n"
            )
        } else {
            String::new()
        };
        let transparency = if color[3] < 0.999 { format!("<transparent opaque=\"A_ONE\"><color>1 1 1 {}</color></transparent><transparency><float>1</float></transparency>", num(color[3] as f64)) } else { String::new() };
        let _ = writeln!(
            effects,
            "    <effect id=\"{id}-effect\">\n      <profile_COMMON>\n{params}        <technique sid=\"common\"><lambert><diffuse>{diffuse}</diffuse>{transparency}</lambert></technique>\n      </profile_COMMON>\n    </effect>"
        );
        let _ = writeln!(materials, "    <material id=\"{id}\" name=\"{}\"><instance_effect url=\"#{id}-effect\"/></material>", xml_escape(name));
    };
    for m in &mats {
        let mm = &model.materials[*m];
        effect(&mat_id(*m), &mm.name, mm.color, mm.image.as_ref());
    }
    let mut geometries = String::new();
    let mut scene = String::new();
    let mut names: Vec<String> = Vec::new();
    for (gi, (owner, g)) in nodes.iter().enumerate() {
        if g.tris.is_empty() {
            continue;
        }
        let name = match owner.and_then(|i| root.instances.get(i)) {
            Some(i) if !i.name.is_empty() => i.name.clone(),
            Some(i) => model.defs.get(i.def).map(|d| d.name.clone()).unwrap_or_default(),
            None => "Model".into(),
        };
        names.push(name.clone());
        let id = format!("geometry{gi}");
        let n = g.pos.len() / 3;
        let _ = writeln!(geometries, "    <geometry id=\"{id}\" name=\"{}\">\n      <mesh>", xml_escape(&name));
        for (suffix, vals, stride, params) in [("positions", &g.pos, 3, "XYZ"), ("normals", &g.nrm, 3, "XYZ"), ("uv", &g.uv, 2, "ST")] {
            let ps: String = params.chars().map(|c| format!("<param name=\"{c}\" type=\"float\"/>")).collect();
            let _ = writeln!(
                geometries,
                "        <source id=\"{id}-{suffix}\">\n          <float_array id=\"{id}-{suffix}-array\" count=\"{}\">{}</float_array>\n          <technique_common><accessor source=\"#{id}-{suffix}-array\" count=\"{}\" stride=\"{stride}\">{ps}</accessor></technique_common>\n        </source>",
                vals.len(),
                nums(vals),
                n
            );
        }
        let _ = writeln!(geometries, "        <vertices id=\"{id}-vertices\"><input semantic=\"POSITION\" source=\"#{id}-positions\"/></vertices>");
        let mut binds = String::new();
        for (m, idx) in &g.tris {
            // Unpainted faces: no material.
            let attr = m.map(|m| format!(" material=\"{}\"", mat_id(m))).unwrap_or_default();
            let p: Vec<String> = idx.iter().map(|i| format!("{i} {i} {i}")).collect();
            let _ = writeln!(
                geometries,
                "        <triangles{attr} count=\"{}\">\n          <input semantic=\"VERTEX\" source=\"#{id}-vertices\" offset=\"0\"/>\n          <input semantic=\"NORMAL\" source=\"#{id}-normals\" offset=\"1\"/>\n          <input semantic=\"TEXCOORD\" source=\"#{id}-uv\" offset=\"2\" set=\"0\"/>\n          <p>{}</p>\n        </triangles>",
                idx.len() / 3,
                p.join(" ")
            );
            if let Some(m) = m {
                let mid = mat_id(*m);
                let _ = write!(binds, "<instance_material symbol=\"{mid}\" target=\"#{mid}\"><bind_vertex_input semantic=\"UVSET0\" input_semantic=\"TEXCOORD\" input_set=\"0\"/></instance_material>");
            }
        }
        let _ = writeln!(geometries, "      </mesh>\n    </geometry>");
        let bind = if binds.is_empty() { String::new() } else { format!("<bind_material><technique_common>{binds}</technique_common></bind_material>") };
        let _ = writeln!(scene, "      <node id=\"node{gi}\" name=\"{}\" type=\"NODE\"><instance_geometry url=\"#{id}\">{bind}</instance_geometry></node>", xml_escape(&name));
    }
    let now = iso_time(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0));
    let lib = |tag: &str, body: &str| if body.is_empty() { String::new() } else { format!("  <{tag}>\n{body}  </{tag}>\n") };
    let text = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<COLLADA xmlns=\"http://www.collada.org/2005/11/COLLADASchema\" version=\"1.4.1\">\n  <asset>\n    <contributor><authoring_tool>Varstotas</authoring_tool></contributor>\n    <created>{now}</created>\n    <modified>{now}</modified>\n    <unit name=\"millimeter\" meter=\"0.001\"/>\n    <up_axis>Z_UP</up_axis>\n  </asset>\n{}{}{}{}  <library_visual_scenes>\n    <visual_scene id=\"scene\" name=\"Scene\">\n{scene}    </visual_scene>\n  </library_visual_scenes>\n  <scene><instance_visual_scene url=\"#scene\"/></scene>\n</COLLADA>\n",
        lib("library_images", &images),
        lib("library_effects", &effects),
        lib("library_materials", &materials),
        lib("library_geometries", &geometries),
    );
    (text, files)
}

/// Write `path` (`.dae`) and its images next to it.
pub fn export_dae(model: &Model, path: &Path) -> Result<(), IoError> {
    let (text, files) = write_dae(model);
    std::fs::write(path, text)?;
    let dir = path.parent().unwrap_or(Path::new("."));
    for (name, bytes) in files {
        std::fs::write(dir.join(name), &bytes[..])?;
    }
    Ok(())
}
