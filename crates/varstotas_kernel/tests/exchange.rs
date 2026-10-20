//! glTF / GLB, 3MF and COLLADA exchange.

use varstotas_kernel::glam::DVec3;
use varstotas_kernel::io::{ImportOptions, add_dae, add_gltf, export_dae, import_dae, write_3mf, write_3mf_model, write_dae, write_glb};
use varstotas_kernel::texture::base64_encode;
use varstotas_kernel::{EntityRef, Model};

/// A 2000 × 1000 × 500 box group, painted red.
fn box_model() -> Model {
    let mut m = Model::default();
    let red = m.add_material("Red", [0.8, 0.1, 0.1, 1.0]);
    let mesh = m.root_mesh_mut();
    let f = mesh
        .add_face(&[DVec3::ZERO, DVec3::new(2000.0, 0.0, 0.0), DVec3::new(2000.0, 1000.0, 0.0), DVec3::new(0.0, 1000.0, 0.0)])
        .unwrap();
    let n = mesh.faces[f].normal;
    mesh.push_pull(f, if n.z > 0.0 { 500.0 } else { -500.0 }, false).unwrap();
    for face in mesh.faces.values_mut() {
        face.front = Some(red);
    }
    let ents: Vec<EntityRef> = mesh.faces.keys().map(EntityRef::Face).chain(mesh.edges.keys().map(EntityRef::Edge)).collect();
    let g = m.make_group(&[], &ents).unwrap();
    m.root_mesh_mut().instances[g].name = "Box".into();
    m
}

#[test]
fn glb_round_trip() {
    let src = box_model();
    let glb = write_glb(&src);
    assert_eq!(&glb[..4], b"glTF");
    assert_eq!(u32::from_le_bytes(glb[8..12].try_into().unwrap()) as usize, glb.len());
    let mut m = Model::default();
    add_gltf(&mut m, &glb, None, &ImportOptions::default()).unwrap();
    let root = m.root_mesh();
    assert_eq!(root.instances.len(), 1, "one node → one group");
    let inst = root.instances.values().next().unwrap();
    assert_eq!(inst.name, "Box");
    let mesh = &m.defs[inst.def].mesh;
    assert_eq!(mesh.faces.len(), 6, "triangles merged back into the box faces");
    assert!(mesh.is_solid());
    assert!((mesh.volume().abs() - 2000.0 * 1000.0 * 500.0).abs() < 1e3, "Z-up mm ↔ Y-up m: {}", mesh.volume());
    let (lo, hi) = mesh.vertices.values().fold((DVec3::MAX, DVec3::MIN), |(lo, hi), v| (lo.min(v.pos), hi.max(v.pos)));
    assert!(lo.distance(DVec3::ZERO) < 1e-3 && hi.distance(DVec3::new(2000.0, 1000.0, 500.0)) < 1e-3, "{lo} {hi}");
    let mat = mesh.faces.values().next().unwrap().front.expect("material kept");
    let c = m.materials[mat].color;
    assert_eq!(m.materials[mat].name, "Red");
    assert!((c[0] - 0.8).abs() < 1e-3 && (c[1] - 0.1).abs() < 1e-3, "sRGB ↔ linear: {c:?}");
}

#[test]
fn gltf_json_with_data_uri_and_node_transform() {
    // One triangle in the glTF XY plane (Y up), node moved 1 m along +X and scaled 2×.
    let pos: Vec<u8> = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0].iter().flat_map(|v| v.to_le_bytes()).collect();
    let uri = format!("data:application/octet-stream;base64,{}", base64_encode(&pos));
    let doc = serde_json::json!({
        "asset": {"version": "2.0"},
        "scenes": [{"nodes": [0]}],
        "nodes": [{"name": "Parent", "translation": [1.0, 0.0, 0.0], "children": [1]}, {"mesh": 0, "scale": [2.0, 2.0, 2.0]}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0}}]}],
        "accessors": [{"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3"}],
        "bufferViews": [{"buffer": 0, "byteLength": 36}],
        "buffers": [{"byteLength": 36, "uri": uri}],
    });
    let mut m = Model::default();
    add_gltf(&mut m, &serde_json::to_vec(&doc).unwrap(), None, &ImportOptions::default()).unwrap();
    let inst = m.root_mesh().instances.values().next().unwrap();
    let mesh = &m.defs[inst.def].mesh;
    assert_eq!(mesh.faces.len(), 1);
    let mut pts: Vec<DVec3> = mesh.vertices.values().map(|v| v.pos).collect();
    pts.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.z.total_cmp(&b.z)));
    // (1,0,0) m → (1000, 0, 0) mm; glTF +Y → Varstotas +Z.
    assert!(pts[0].distance(DVec3::new(1000.0, 0.0, 0.0)) < 1e-6, "{pts:?}");
    assert!(pts[1].distance(DVec3::new(1000.0, 0.0, 2000.0)) < 1e-6, "{pts:?}");
    assert!(pts[2].distance(DVec3::new(3000.0, 0.0, 0.0)) < 1e-6, "{pts:?}");
}

#[test]
fn not_gltf() {
    let mut m = Model::default();
    assert!(add_gltf(&mut m, b"solid x", None, &ImportOptions::default()).is_err());
    let empty = serde_json::json!({"asset": {"version": "2.0"}, "nodes": []});
    assert!(add_gltf(&mut m, &serde_json::to_vec(&empty).unwrap(), None, &ImportOptions::default()).is_err());
}

#[test]
fn three_mf_package() {
    let m = box_model();
    let xml = write_3mf_model(&m);
    assert!(xml.contains("unit=\"millimeter\""));
    assert_eq!(xml.matches("<vertex ").count(), 8, "shared corners");
    assert_eq!(xml.matches("<triangle ").count(), 12);
    assert_eq!(xml.matches("<object ").count(), 1);
    assert!(xml.contains("name=\"Box\"") && xml.contains("displaycolor=\"#CC1A1AFF\"") && xml.contains("pid=\"1\" p1=\"0\""));
    let zip = write_3mf(&m);
    assert_eq!(&zip[..4], b"PK\x03\x04");
    let tail = &zip[zip.len() - 22..];
    assert_eq!(&tail[..4], b"PK\x05\x06");
    assert_eq!(u16::from_le_bytes([tail[10], tail[11]]), 3, "three entries");
}

/// A minimal valid PNG header (the kernel only reads its size).
fn png_bytes(tag: u8) -> Vec<u8> {
    let mut b = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x02\0\0\0\x02\x08\x06\0\0\0".to_vec();
    b.extend([0, 0, 0, 0, tag]);
    b
}

#[test]
fn glb_keeps_physical_materials() {
    use varstotas_kernel::{Pbr, TextureImage};
    let mut src = box_model();
    let red = src.materials.keys().next().unwrap();
    src.materials[red].pbr = Some(Pbr {
        metallic: 0.25,
        roughness: 0.4,
        emission: [1.0, 0.5, 0.0],
        emission_strength: 3.0,
        transmission: 0.5,
        ior: 1.33,
        normal_scale: 0.7,
        normal_map: Some(TextureImage::new(png_bytes(1), "n.png", 500.0)),
        roughness_map: Some(TextureImage::new(png_bytes(2), "r.png", 500.0)),
        ao_map: Some(TextureImage::new(png_bytes(3), "ao.png", 500.0)),
        ..Default::default()
    });
    let glb = write_glb(&src);
    let js = String::from_utf8_lossy(&glb);
    for ext in ["KHR_materials_emissive_strength", "KHR_materials_transmission", "KHR_materials_ior", "extensionsUsed"] {
        assert!(js.contains(ext), "{ext}");
    }
    let mut m = Model::default();
    add_gltf(&mut m, &glb, None, &ImportOptions::default()).unwrap();
    let mat = m.materials.values().next().unwrap();
    let p = mat.pbr.as_ref().expect("physical properties imported");
    assert_eq!((p.metallic, p.roughness, p.transmission, p.emission_strength), (0.25, 0.4, 0.5, 3.0));
    assert!((p.ior - 1.33).abs() < 1e-6 && (p.normal_scale - 0.7).abs() < 1e-6);
    assert!((p.emission[1] - 0.5).abs() < 1e-3, "{:?}", p.emission);
    assert_eq!(p.normal_map.as_ref().map(|t| t.bytes[..].to_vec()), Some(png_bytes(1)));
    // glTF packs roughness (G) and metalness (B) in one image
    assert_eq!(p.roughness_map.as_ref().map(|t| t.bytes[..].to_vec()), Some(png_bytes(2)));
    assert_eq!(p.metallic_map.as_ref().map(|t| t.bytes[..].to_vec()), Some(png_bytes(2)));
    assert_eq!(p.ao_map.as_ref().map(|t| t.bytes[..].to_vec()), Some(png_bytes(3)));
    assert!(p.emission_map.is_none());
    // faces got texture coordinates from the maps (no base colour texture)
    let inst = m.root_mesh().instances.values().next().unwrap();
    assert!(m.defs[inst.def].mesh.faces.values().any(|f| f.uv.is_some()));
}

/// A GLB whose square is Draco-compressed (KHR_draco_mesh_compression) imports as one face.
#[test]
fn gltf_draco_primitive() {
    use draco_core::{DataType, EncoderBuffer, EncoderOptions, FaceIndex, GeometryAttributeType, Mesh, MeshEncoder, PointAttribute};
    let mut mesh = Mesh::new();
    mesh.set_num_points(4);
    let mut pos = PointAttribute::new();
    pos.init(GeometryAttributeType::Position, 3, DataType::Float32, false, 4);
    let coords: [f32; 12] = [0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 2.0, 0.0, -1.0, 0.0, 0.0, -1.0];
    for (i, v) in coords.iter().enumerate() {
        pos.buffer_mut().write(i * 4, &v.to_le_bytes());
    }
    let id = mesh.add_attribute(pos);
    mesh.set_num_faces(2);
    mesh.set_face(FaceIndex(0), [0u32.into(), 1u32.into(), 2u32.into()]);
    mesh.set_face(FaceIndex(1), [0u32.into(), 2u32.into(), 3u32.into()]);
    let mut enc = MeshEncoder::new();
    enc.set_mesh(mesh);
    let mut out = EncoderBuffer::new();
    enc.encode(&EncoderOptions::new(), &mut out).unwrap();
    let mut bin = out.data().to_vec();
    let len = bin.len();
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let doc = serde_json::json!({
        "asset": {"version": "2.0"},
        "extensionsUsed": ["KHR_draco_mesh_compression"],
        "extensionsRequired": ["KHR_draco_mesh_compression"],
        "scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0}, "mode": 4,
            "extensions": {"KHR_draco_mesh_compression": {"bufferView": 0, "attributes": {"POSITION": id}}}}]}],
        "accessors": [{"componentType": 5126, "count": 4, "type": "VEC3"}],
        "bufferViews": [{"buffer": 0, "byteLength": len}],
        "buffers": [{"byteLength": bin.len()}]
    });
    let mut json = serde_json::to_vec(&doc).unwrap();
    while !json.len().is_multiple_of(4) {
        json.push(b' ');
    }
    let mut glb = b"glTF".to_vec();
    glb.extend(2u32.to_le_bytes());
    glb.extend(((12 + 8 + json.len() + 8 + bin.len()) as u32).to_le_bytes());
    glb.extend((json.len() as u32).to_le_bytes());
    glb.extend(b"JSON");
    glb.extend(&json);
    glb.extend((bin.len() as u32).to_le_bytes());
    glb.extend(b"BIN\0");
    glb.extend(&bin);
    let mut m = varstotas_kernel::Model::new();
    varstotas_kernel::io::add_gltf(&mut m, &glb, None, &Default::default()).unwrap();
    let faces: usize = m.defs.values().map(|d| d.mesh.faces.len()).sum();
    assert_eq!(faces, 1, "the two triangles merge into one square");
    let b = m.bounds().unwrap();
    // glTF metres, Y-up: 2 m × 1 m on the ground.
    assert!((b.max.x - b.min.x - 2000.0).abs() < 1e-3 && (b.max.y - b.min.y - 1000.0).abs() < 1e-3 && (b.max.z - b.min.z).abs() < 1e-3, "{b:?}");
}

#[test]
fn dae_round_trip() {
    let src = box_model();
    let (text, files) = write_dae(&src);
    assert!(text.contains("<up_axis>Z_UP</up_axis>") && text.contains("meter=\"0.001\""), "mm, Z up");
    assert_eq!(text.matches("<node ").count(), 1);
    assert!(files.is_empty(), "no textures");
    let mut m = Model::default();
    add_dae(&mut m, &text, None, &ImportOptions::default()).unwrap();
    let root = m.root_mesh();
    assert_eq!(root.instances.len(), 1, "one node → one group");
    let inst = root.instances.values().next().unwrap();
    assert_eq!(inst.name, "Box");
    let mesh = &m.defs[inst.def].mesh;
    assert_eq!(mesh.faces.len(), 6, "triangles merged back into the box faces");
    assert!(mesh.is_solid());
    assert!((mesh.volume().abs() - 2000.0 * 1000.0 * 500.0).abs() < 1e3, "{}", mesh.volume());
    let (lo, hi) = mesh.vertices.values().fold((DVec3::MAX, DVec3::MIN), |(lo, hi), v| (lo.min(v.pos), hi.max(v.pos)));
    assert!(lo.distance(DVec3::ZERO) < 1e-3 && hi.distance(DVec3::new(2000.0, 1000.0, 500.0)) < 1e-3, "{lo} {hi}");
    let mat = mesh.faces.values().next().unwrap().front.expect("material kept");
    assert_eq!(m.materials[mat].name, "Red");
    let c = m.materials[mat].color;
    assert!((c[0] - 0.8).abs() < 1e-3 && (c[1] - 0.1).abs() < 1e-3, "{c:?}");
    // Unpainted faces stay unpainted: no material in the file, none made on import.
    let mut plain = box_model();
    plain.materials.clear();
    let (text, _) = write_dae(&plain);
    assert!(!text.contains("<material ") && !text.contains("material=\""), "no material");
    let mut m = Model::default();
    add_dae(&mut m, &text, None, &ImportOptions::default()).unwrap();
    assert!(m.materials.is_empty());
}

#[test]
fn dae_texture_file_next_to_the_document() {
    use varstotas_kernel::TextureImage;
    let mut src = box_model();
    let red = src.materials.keys().next().unwrap();
    src.materials[red].image = Some(TextureImage::new(png_bytes(7), "bricks.png", 500.0));
    let dir = std::env::temp_dir().join(format!("varstotas_dae_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("box.dae");
    export_dae(&src, &path).unwrap();
    assert!(dir.join("Red.png").exists(), "the image is written next to the .dae");
    let mut m = Model::default();
    import_dae(&mut m, &path).unwrap();
    let inst = m.root_mesh().instances.values().next().unwrap();
    let face = m.defs[inst.def].mesh.faces.values().next().unwrap();
    let img = m.materials[face.front.unwrap()].image.as_ref().expect("texture read back");
    assert_eq!(img.bytes[..], png_bytes(7)[..]);
    let _ = std::fs::remove_dir_all(&dir);
}
