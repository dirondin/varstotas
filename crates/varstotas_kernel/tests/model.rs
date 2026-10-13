mod common;
use common::*;
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::*;

fn tmpdir(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("varstotas_kernel_test_{}_{}", name, std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn box_model() -> Model {
    let mut model = Model::new();
    make_box(model.root_mesh_mut(), 100., 50., 30.);
    model
}

#[test]
fn new_model_is_empty() {
    let m = Model::new();
    assert_eq!(m.defs.len(), 1);
    assert!(m.materials.is_empty());
    assert_eq!(m.unit, LengthUnit::Millimeter);
    assert!(m.bounds().is_none());
    assert!(m.mesh_at(&[]).is_some());
}

#[test]
fn group_and_explode_round_trip() {
    let mut model = box_model();
    let ents: Vec<EntityRef> = model.root_mesh().faces.keys().map(EntityRef::Face).collect();
    let inst = model.make_group(&[], &ents).unwrap();
    assert_eq!(model.root_mesh().faces.len(), 0);
    assert_eq!(model.root_mesh().edges.len(), 0);
    assert_eq!(model.root_mesh().vertices.len(), 0);
    let gm = model.mesh_at(&[inst]).unwrap();
    assert_eq!(gm.faces.len(), 6);
    ok(gm);
    assert!(model.defs[model.root_mesh().instances[inst].def].is_group);
    // Move the group and explode.
    model.root_mesh_mut().instances[inst].transform = DMat4::from_translation(v(0., 0., 100.));
    let b = model.bounds().unwrap();
    assert!((b.min.z - 100.).abs() < 1e-9);
    let out = model.explode(&[], inst).unwrap();
    assert_eq!(out.iter().filter(|r| matches!(r, EntityRef::Face(_))).count(), 6);
    let root = model.root_mesh();
    ok(root);
    assert_eq!(root.faces.len(), 6);
    assert_eq!(root.instances.len(), 0);
    assert_eq!(model.defs.len(), 1);
    assert!((volume(root) - 150_000.).abs() < 1e-6);
    assert!((root.bounds().unwrap().min.z - 100.).abs() < 1e-9);
}

#[test]
fn group_single_face_keeps_shared_edges_in_parent() {
    let mut model = Model::new();
    let m = model.root_mesh_mut();
    m.add_face(&[v(0., 0., 1.), v(10., 0., 1.), v(10., 10., 1.), v(0., 10., 1.)]);
    let f2 = m.add_face(&[v(10., 0., 1.), v(20., 0., 1.), v(20., 10., 1.), v(10., 10., 1.)]).unwrap();
    let inst = model.make_group(&[], &[EntityRef::Face(f2)]).unwrap();
    let root = model.root_mesh();
    ok(root);
    assert_eq!(root.faces.len(), 1);
    assert_eq!(root.edges.len(), 4); // shared edge stays
    assert_eq!(model.mesh_at(&[inst]).unwrap().edges.len(), 4);
}

#[test]
fn nested_group_paths_and_transforms() {
    let mut model = box_model();
    let ents: Vec<EntityRef> = model.root_mesh().faces.keys().map(EntityRef::Face).collect();
    let inner = model.make_group(&[], &ents).unwrap();
    model.root_mesh_mut().instances[inner].transform = DMat4::from_translation(v(10., 0., 0.));
    let outer = model.make_group(&[], &[EntityRef::Instance(inner)]).unwrap();
    model.root_mesh_mut().instances[outer].transform = DMat4::from_translation(v(0., 20., 0.));
    let path = [outer, inner];
    assert_eq!(model.mesh_at(&path).unwrap().faces.len(), 6);
    let t = model.transform_at(&path);
    assert!((t.transform_point3(DVec3::ZERO) - v(10., 20., 0.)).length() < 1e-9);
    let b = model.bounds().unwrap();
    assert!((b.min - v(10., 20., 0.)).length() < 1e-9);
    // Raycast reaches nested geometry and reports the path.
    let hit = model.raycast(&Ray { origin: v(50., 40., 500.), dir: DVec3::NEG_Z }).unwrap();
    assert_eq!(hit.path, path.to_vec());
    assert!((hit.point.z - 30.).abs() < 1e-9);
    assert!((hit.t - 470.).abs() < 1e-9);
}

#[test]
fn raycast_respects_hidden_and_tags() {
    let mut model = box_model();
    let ents: Vec<EntityRef> = model.root_mesh().faces.keys().map(EntityRef::Face).collect();
    let inst = model.make_group(&[], &ents).unwrap();
    let ray = Ray { origin: v(50., 25., 500.), dir: DVec3::NEG_Z };
    assert!(model.raycast(&ray).is_some());
    model.root_mesh_mut().instances[inst].hidden = true;
    assert!(model.raycast(&ray).is_none());
    model.root_mesh_mut().instances[inst].hidden = false;
    let tag = model.add_tag("Walls");
    model.root_mesh_mut().instances[inst].tag = Some(tag);
    model.tags[tag].visible = false;
    assert!(model.raycast(&ray).is_none());
    assert!(tessellate(&model).faces.is_empty());
    model.tags[tag].visible = true;
    assert_eq!(tessellate(&model).faces.len(), 6);
}

#[test]
fn component_instances_and_make_unique() {
    let mut model = box_model();
    let ents: Vec<EntityRef> = model.root_mesh().faces.keys().map(EntityRef::Face).collect();
    let a = model.make_component(&[], &ents, "Box").unwrap();
    let d = model.root_mesh().instances[a].def;
    assert!(!model.defs[d].is_group);
    assert_eq!(model.defs[d].name, "Box");
    let mut copy = model.root_mesh().instances[a].clone();
    copy.transform = DMat4::from_translation(v(200., 0., 0.));
    let b = model.root_mesh_mut().instances.insert(copy);
    assert_eq!(model.instance_count(d), 2);
    assert_eq!(tessellate(&model).faces.len(), 12);
    model.make_unique(&[], b).unwrap();
    let d2 = model.root_mesh().instances[b].def;
    assert_ne!(d, d2);
    assert_eq!(model.instance_count(d), 1);
    // Editing the unique copy doesn't affect the original.
    let f = model.defs[d2].mesh.faces.keys().next().unwrap();
    model.defs[d2].mesh.erase(&[EntityRef::Face(f)]);
    assert_eq!(model.defs[d].mesh.faces.len(), 6);
    assert_eq!(model.defs[d2].mesh.faces.len(), 5);
}

#[test]
fn materials_and_tags() {
    let mut model = Model::new();
    let red = model.add_material("Red", [1., 0., 0., 1.]);
    let tag = model.add_tag("Layer");
    assert_eq!(model.materials[red].name, "Red");
    assert!(model.tags[tag].visible);
}

#[test]
fn tessellate_world_space_and_inherited_material() {
    let mut model = box_model();
    let red = model.add_material("Red", [1., 0., 0., 1.]);
    let ents: Vec<EntityRef> = model.root_mesh().faces.keys().map(EntityRef::Face).collect();
    let inst = model.make_group(&[], &ents).unwrap();
    {
        let i = &mut model.root_mesh_mut().instances[inst];
        i.transform = DMat4::from_translation(v(0., 0., 1000.));
        i.material = Some(red);
    }
    let t = tessellate(&model);
    assert_eq!(t.faces.len(), 6);
    assert_eq!(t.edges.len(), 12);
    let tris: usize = t.faces.iter().map(|f| f.positions.len() / 3).sum();
    assert_eq!(tris, 12);
    for f in &t.faces {
        assert_eq!(f.inherited, Some(red));
        assert_eq!(f.owner_path, vec![inst]);
        assert!(f.positions.iter().all(|p| p.z >= 1000. - 1e-9));
        // Triangles wind CCW about the normal.
        for tr in f.positions.chunks(3) {
            assert!((tr[1] - tr[0]).cross(tr[2] - tr[0]).dot(f.normal) > 0.0);
        }
    }
}

#[test]
fn tessellate_face_with_hole_and_hidden() {
    let mut model = Model::new();
    let m = model.root_mesh_mut();
    rect_lines(m, 0., 0., 100., 100., 0.);
    rect_lines(m, 30., 30., 60., 60., 0.);
    let inner = m.faces.iter().find(|(_, f)| f.holes.is_empty()).unwrap().0;
    m.faces[inner].hidden = true;
    let t = tessellate(&model);
    assert_eq!(t.faces.len(), 1);
    let area: f64 = t.faces[0]
        .positions
        .chunks(3)
        .map(|tr| (tr[1] - tr[0]).cross(tr[2] - tr[0]).length() * 0.5)
        .sum();
    assert!((area - 9100.).abs() < 1e-6);
}

#[test]
fn tessellate_mirrored_instance_winding() {
    let mut model = box_model();
    let ents: Vec<EntityRef> = model.root_mesh().faces.keys().map(EntityRef::Face).collect();
    let inst = model.make_group(&[], &ents).unwrap();
    model.root_mesh_mut().instances[inst].transform = DMat4::from_scale(v(-1., 1., 1.));
    let t = tessellate(&model);
    let c = model.bounds().unwrap().center();
    for f in &t.faces {
        let fc = f.positions.iter().copied().sum::<DVec3>() / f.positions.len() as f64;
        assert!(f.normal.dot(fc - c) > 0.0, "mirrored normal points inward");
        for tr in f.positions.chunks(3) {
            assert!((tr[1] - tr[0]).cross(tr[2] - tr[0]).dot(f.normal) > 0.0);
        }
    }
}

#[test]
fn save_load_round_trip() {
    let mut model = box_model();
    let red = model.add_material("Red", [1., 0., 0., 0.5]);
    let f = model.root_mesh().faces.keys().next().unwrap();
    model.root_mesh_mut().faces[f].front = Some(red);
    let ents: Vec<EntityRef> = vec![EntityRef::Face(f)];
    let inst = model.make_group(&[], &ents).unwrap();
    model.unit = LengthUnit::Meter;
    let dir = tmpdir("roundtrip");
    let path = dir.join("test.vrst");
    io::save(&model, &path).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.starts_with("{\"format\":\"varstotas\",\"version\":1,"));
    let mut loaded = io::load(&path).unwrap();
    assert_eq!(loaded.unit, LengthUnit::Meter);
    assert_eq!(loaded.root_mesh().faces.len(), 5);
    ok(loaded.root_mesh());
    ok(loaded.mesh_at(&[inst]).unwrap());
    let gf = loaded.mesh_at(&[inst]).unwrap().faces.values().next().unwrap().front;
    assert_eq!(gf, Some(red));
    // Caches were rebuilt: sticky editing works after load.
    let e = loaded.root_mesh().edges.keys().next().unwrap();
    assert!(!loaded.root_mesh().edge_faces(e).is_empty());
    loaded.root_mesh_mut().add_line(v(0., 0., 30.), v(100., 50., 30.));
    ok(loaded.root_mesh());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn load_rejects_foreign_json() {
    assert!(io::from_json("{\"format\":\"other\",\"version\":1,\"model\":{}}").is_err());
    assert!(io::from_json("not json").is_err());
}

#[test]
fn obj_export_sanity() {
    let mut model = box_model();
    let red = model.add_material("Red Brick", [1., 0., 0., 1.]);
    let f = model.root_mesh().faces.keys().next().unwrap();
    model.root_mesh_mut().faces[f].front = Some(red);
    let (obj, mtl) = io::write_obj(&model, "x.mtl");
    assert!(obj.contains("mtllib x.mtl"));
    assert_eq!(obj.lines().filter(|l| l.starts_with("v ")).count(), 8);
    assert_eq!(obj.lines().filter(|l| l.starts_with("f ")).count(), 12);
    assert_eq!(obj.lines().filter(|l| l.starts_with("usemtl")).count(), 2);
    assert!(mtl.contains("newmtl default"));
    assert!(mtl.contains("Red_Brick"));
    let dir = tmpdir("obj");
    let p = dir.join("box.obj");
    io::export_obj(&model, &p).unwrap();
    assert!(dir.join("box.mtl").exists());
    // Import back into a new model.
    let mut m2 = Model::new();
    io::import_obj(&mut m2, &p).unwrap();
    let r = m2.root_mesh();
    ok(r);
    assert_closed_consistent(r);
    assert!((volume(r) - 150_000.).abs() < 1e-3);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn stl_export_import_sanity() {
    let model = box_model();
    let bytes = io::write_stl_binary(&model);
    assert_eq!(bytes.len(), 84 + 12 * 50);
    assert_eq!(u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]), 12);
    let dir = tmpdir("stl");
    let p = dir.join("box.stl");
    io::export_stl(&model, &p).unwrap();
    let mut m2 = Model::new();
    io::import_stl(&mut m2, &p).unwrap();
    let r = m2.root_mesh();
    ok(r);
    assert_eq!(r.vertices.len(), 8);
    assert!((volume(r) - 150_000.).abs() < 1e-3);
    // ASCII STL
    let ascii = "solid t\nfacet normal 0 0 1\nouter loop\nvertex 0 0 5\nvertex 10 0 5\nvertex 0 10 5\nendloop\nendfacet\nendsolid t\n";
    let pa = dir.join("tri.stl");
    std::fs::write(&pa, ascii).unwrap();
    let mut m3 = Model::new();
    io::import_stl(&mut m3, &pa).unwrap();
    assert_eq!(m3.root_mesh().faces.len(), 1);
    assert!(m3.root_mesh().faces.values().next().unwrap().normal.z > 0.999);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn history_undo_redo() {
    let mut model = Model::new();
    let mut h = History::default();
    let before = model.clone();
    rect_lines(model.root_mesh_mut(), 0., 0., 10., 10., 0.);
    h.push("Rectangle", before);
    assert_eq!(model.root_mesh().faces.len(), 1);
    assert_eq!(h.undo(&mut model).as_deref(), Some("Rectangle"));
    assert_eq!(model.root_mesh().faces.len(), 0);
    assert_eq!(h.redo(&mut model).as_deref(), Some("Rectangle"));
    assert_eq!(model.root_mesh().faces.len(), 1);
    // Model after undo/redo is still editable.
    model.root_mesh_mut().add_line(v(5., 0., 0.), v(5., 10., 0.));
    assert_eq!(model.root_mesh().faces.len(), 2);
}

#[test]
fn explode_onto_existing_geometry_merges_vertices() {
    let mut model = Model::new();
    let m = model.root_mesh_mut();
    m.add_face(&[v(0., 0., 1.), v(10., 0., 1.), v(10., 10., 1.), v(0., 10., 1.)]);
    let f2 = m.add_face(&[v(10., 0., 1.), v(20., 0., 1.), v(20., 10., 1.), v(10., 10., 1.)]).unwrap();
    let inst = model.make_group(&[], &[EntityRef::Face(f2)]).unwrap();
    model.explode(&[], inst).unwrap();
    let r = model.root_mesh();
    ok(r);
    assert_eq!(r.faces.len(), 2);
    assert_eq!(r.edges.len(), 7);
    assert_eq!(r.vertices.len(), 6);
}

#[test]
fn group_inside_group_context_editing() {
    let mut model = box_model();
    let ents: Vec<EntityRef> = model.root_mesh().faces.keys().map(EntityRef::Face).collect();
    let g = model.make_group(&[], &ents).unwrap();
    // Edit inside the group: draw a line across the top face.
    let mesh = model.mesh_at_mut(&[g]).unwrap();
    mesh.add_line(v(50., 0., 30.), v(50., 50., 30.));
    assert_eq!(mesh.faces.len(), 7);
    // Make a sub-group from one top half inside the group.
    let half = mesh.faces.iter().find(|(id, f)| f.normal.z > 0.9 && face_centroid(mesh, *id).x > 50.).unwrap().0;
    let sub = model.make_group(&[g], &[EntityRef::Face(half)]).unwrap();
    assert_eq!(model.mesh_at(&[g, sub]).unwrap().faces.len(), 1);
    assert_eq!(model.mesh_at(&[g]).unwrap().faces.len(), 6);
}

#[test]
fn obj_numbers_are_clean_for_rotated_groups() {
    let mut model = box_model();
    let ents: Vec<EntityRef> = model.root_mesh().faces.keys().map(EntityRef::Face).collect();
    let g = model.make_group(&[], &ents).unwrap();
    let rot = DMat4::from_rotation_z(std::f64::consts::FRAC_PI_3 * 1.5) * DMat4::from_translation(DVec3::new(0.1, 0.2, 0.3));
    model.root_mesh_mut().instances[g].transform = rot;
    let (obj, _) = io::write_obj(&model, "x.mtl");
    for l in obj.lines().filter(|l| l.starts_with("v ") || l.starts_with("vn ")) {
        for t in l.split_whitespace().skip(1) {
            assert!(!t.contains('e') && t != "-0" && t.len() < 24, "noisy number {t} in `{l}`");
        }
    }
}
