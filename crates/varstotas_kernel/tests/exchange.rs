//! glTF / GLB and 3MF exchange.

use varstotas_kernel::glam::DVec3;
use varstotas_kernel::io::{ImportOptions, add_gltf, write_3mf, write_3mf_model, write_glb};
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
