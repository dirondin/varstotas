//! FBX import through ufbx (binary and ASCII, any version): each mesh node becomes a group,
//! materials get their diffuse / base colour and colour texture (embedded or a file next to
//! the model). ufbx converts the scene to Z up and millimetres.

use super::meshes::{ImpMat, ImpMesh, ImpPoly, add_meshes};
use super::{ImportOptions, IoError};
use crate::Model;
use glam::{DMat4, DVec2, DVec3, DVec4};
use std::collections::HashMap;
use std::path::Path;
use varstotas_i18n::tr;

fn matrix(m: &ufbx::Matrix) -> DMat4 {
    DMat4::from_cols(
        DVec4::new(m.m00, m.m10, m.m20, 0.0),
        DVec4::new(m.m01, m.m11, m.m21, 0.0),
        DVec4::new(m.m02, m.m12, m.m22, 0.0),
        DVec4::new(m.m03, m.m13, m.m23, 1.0),
    )
}

fn texture_bytes(t: &ufbx::Texture, dir: Option<&Path>) -> Option<(Vec<u8>, String)> {
    let name = Path::new(&*t.filename).file_name().or(Path::new(&*t.relative_filename).file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "texture.png".into());
    if !t.content.is_empty() {
        return Some((t.content.to_vec(), name));
    }
    let tries = [t.absolute_filename.to_string(), t.filename.to_string()];
    for f in tries.iter().filter(|f| !f.is_empty()) {
        if let Ok(b) = std::fs::read(f) {
            return Some((b, name));
        }
    }
    let dir = dir?;
    [t.relative_filename.to_string(), name.clone()].iter().filter(|f| !f.is_empty()).find_map(|f| std::fs::read(dir.join(f.replace('\\', "/"))).ok()).map(|b| (b, name.clone()))
}

fn material(m: &ufbx::Material, dir: Option<&Path>) -> ImpMat {
    let pick = |a: &ufbx::MaterialMap, b: &ufbx::MaterialMap| if a.has_value { a.value_vec4 } else { b.value_vec4 };
    let c = pick(&m.pbr.base_color, &m.fbx.diffuse_color);
    let mut color = [c.x as f32, c.y as f32, c.z as f32, 1.0];
    if color == [0.0, 0.0, 0.0, 1.0] && !m.pbr.base_color.has_value && !m.fbx.diffuse_color.has_value {
        color = [0.8, 0.8, 0.8, 1.0];
    }
    if m.pbr.opacity.has_value && (0.0..1.0).contains(&m.pbr.opacity.value_vec4.x) {
        color[3] = m.pbr.opacity.value_vec4.x as f32;
    }
    let tex = m.pbr.base_color.texture.as_ref().or(m.fbx.diffuse_color.texture.as_ref());
    let image = tex.and_then(|t| texture_bytes(t, dir));
    if image.is_some() {
        color = [1.0, 1.0, 1.0, color[3]];
    }
    ImpMat { name: m.element.name.to_string(), color, image }
}

pub fn import_fbx(model: &mut Model, path: &Path) -> Result<(), IoError> {
    import_fbx_with(model, path, &ImportOptions::default())
}

pub fn import_fbx_with(model: &mut Model, path: &Path, opts: &ImportOptions) -> Result<(), IoError> {
    let data = std::fs::read(path)?;
    add_fbx(model, &data, path.parent(), opts)
}

/// Add FBX bytes (`dir`: where external textures are).
pub fn add_fbx(model: &mut Model, data: &[u8], dir: Option<&Path>, opts: &ImportOptions) -> Result<(), IoError> {
    let lo = ufbx::LoadOpts {
        target_axes: ufbx::CoordinateAxes::right_handed_z_up(),
        target_unit_meters: 0.001,
        space_conversion: ufbx::SpaceConversion::ModifyGeometry,
        generate_missing_normals: false,
        ignore_animation: true,
        ..Default::default()
    };
    let scene = ufbx::load_memory(data, lo).map_err(|e| IoError::Format(tr!("kernel-io-fbx-description", description = e.description.to_string())))?;
    let mut mats: Vec<ImpMat> = Vec::new();
    let mut mat_index: HashMap<u32, usize> = HashMap::new();
    let mut meshes = Vec::new();
    for node in scene.nodes.iter() {
        let Some(mesh) = node.mesh.as_ref() else { continue };
        if !node.visible {
            continue;
        }
        let xf = matrix(&node.geometry_to_world);
        let flip = xf.determinant() < 0.0;
        let node_mats: Vec<&ufbx::Material> = if node.materials.is_empty() { mesh.materials.iter().map(|m| &**m).collect() } else { node.materials.iter().map(|m| &**m).collect() };
        let mut polys = Vec::with_capacity(mesh.faces.len());
        for (fi, face) in mesh.faces.iter().enumerate() {
            if face.num_indices < 3 {
                continue;
            }
            let range = face.index_begin as usize..(face.index_begin + face.num_indices) as usize;
            let mut pts: Vec<DVec3> = range.clone().map(|i| { let p = mesh.vertex_position[i]; xf.transform_point3(DVec3::new(p.x, p.y, p.z)) }).collect();
            let mut uv: Option<Vec<DVec2>> = mesh.vertex_uv.exists.then(|| range.map(|i| { let u = mesh.vertex_uv[i]; DVec2::new(u.x, u.y) }).collect());
            if flip {
                pts.reverse();
                if let Some(u) = uv.as_mut() {
                    u.reverse();
                }
            }
            let mi = mesh.face_material.get(fi).copied().unwrap_or(0) as usize;
            let mat = node_mats.get(mi).map(|m| {
                *mat_index.entry(m.element.element_id).or_insert_with(|| {
                    mats.push(material(m, dir));
                    mats.len() - 1
                })
            });
            polys.push(ImpPoly { pts, uv, mat });
        }
        meshes.push(ImpMesh { name: node.element.name.to_string(), polys });
    }
    add_meshes(model, meshes, &mats, DMat4::IDENTITY, opts).map(|_| ())
}
