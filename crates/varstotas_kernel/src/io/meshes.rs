//! Shared by the 3MF / COLLADA / FBX importers: polygons with materials and texture
//! coordinates, one group per mesh in the root.

use super::{ImportOptions, IoError};
use super::stl::{add_polygon, new_faces};
use crate::texture::{TexPlacement, TextureImage};
use crate::{Definition, FaceId, Instance, MaterialId, Mesh, Model};
use glam::{DMat4, DVec2, DVec3};
use std::collections::{HashMap, HashSet};

/// A material of an imported file: its colour (sRGB, alpha) and a colour image.
#[derive(Clone, Debug)]
pub(crate) struct ImpMat {
    pub name: String,
    pub color: [f32; 4],
    /// Image bytes and a file name.
    pub image: Option<(Vec<u8>, String)>,
}

/// One polygon (model space, mm): material index into the materials, texture coordinates
/// (origin bottom left, one per point).
#[derive(Clone, Debug, Default)]
pub(crate) struct ImpPoly {
    pub pts: Vec<DVec3>,
    pub uv: Option<Vec<DVec2>>,
    pub mat: Option<usize>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ImpMesh {
    pub name: String,
    pub polys: Vec<ImpPoly>,
}

/// `#rrggbb` / `#rrggbbaa` (3MF colours) → sRGB + alpha.
pub(crate) fn hex_color(s: &str) -> Option<[f32; 4]> {
    let h = s.trim().trim_start_matches('#');
    let c = |i: usize| u8::from_str_radix(h.get(i..i + 2)?, 16).ok().map(|v| v as f32 / 255.0);
    match h.len() {
        6 => Some([c(0)?, c(2)?, c(4)?, 1.0]),
        8 => Some([c(0)?, c(2)?, c(4)?, c(6)?]),
        _ => None,
    }
}

/// Add `meshes` as groups of the root (`xf` applied to every point; empty meshes skipped).
/// Materials are made when first used (a model material with the same name and colour is
/// reused).
pub(crate) fn add_meshes(model: &mut Model, meshes: Vec<ImpMesh>, mats: &[ImpMat], xf: DMat4, opts: &ImportOptions) -> Result<usize, IoError> {
    let mut ids: HashMap<usize, Option<MaterialId>> = HashMap::new();
    let flip = xf.determinant() < 0.0;
    let mut groups = 0;
    for m in meshes {
        let mut mesh = Mesh::new();
        for p in &m.polys {
            if p.pts.len() < 3 {
                continue;
            }
            let mat = match p.mat.filter(|_| opts.materials) {
                Some(i) if i < mats.len() => *ids.entry(i).or_insert_with(|| Some(material(model, &mats[i]))),
                _ => None,
            };
            let mut pts: Vec<DVec3> = p.pts.iter().map(|q| xf.transform_point3(*q)).collect();
            let mut uv = p.uv.clone().filter(|u| u.len() == pts.len());
            if flip {
                pts.reverse();
                if let Some(u) = uv.as_mut() {
                    u.reverse();
                }
            }
            let tile = mat.and_then(|id| model.materials[id].image.as_ref()).map(|i| i.size);
            for f in add_polygon(&mut mesh, &pts) {
                let Some(face) = mesh.faces.get_mut(f) else { continue };
                face.front = mat;
                if let (Some(tile), Some(uv)) = (tile, uv.as_ref()) {
                    face.uv = TexPlacement::fit(&pts, uv, face.normal, tile).filter(|pl| !pl.is_identity());
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
        let def = model.defs.insert(Definition { name: "Group".into(), mesh, is_group: true, guides: Default::default(), params: None, generated: None, source: None, rig: None, rigged: None, preview: None });
        let root = model.root;
        model.defs[root].mesh.instances.insert(Instance { def, transform: DMat4::IDENTITY, name: m.name, material: None, tag: None, hidden: false, locked: false });
        groups += 1;
    }
    if groups == 0 {
        return Err(IoError::Format("no faces in the file".into()));
    }
    Ok(groups)
}

fn material(model: &mut Model, m: &ImpMat) -> MaterialId {
    let name = if m.name.trim().is_empty() { "Material" } else { m.name.trim() };
    if let Some((id, _)) = model.materials.iter().find(|(_, x)| x.name == name && x.color == m.color && x.image.is_none() == m.image.is_none()) {
        return id;
    }
    let id = model.add_material(name, m.color);
    model.materials[id].image = m.image.as_ref().filter(|(b, _)| crate::texture::image_dimensions(b).is_some()).map(|(bytes, file)| {
        let mut ti = TextureImage::new(bytes.clone(), file.clone(), 1000.0);
        ti.size = [1000.0, 1000.0];
        ti
    });
    id
}
