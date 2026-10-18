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
fn untagged_hidden_only_at_top_level() {
    // A loose box at the top level and a tagged group with untagged faces inside.
    let mut model = Model::new();
    make_box(model.root_mesh_mut(), 10., 10., 10.);
    let small: Vec<EntityRef> = model.root_mesh().faces.keys().map(EntityRef::Face).collect();
    let inst = model.make_group(&[], &small).unwrap();
    make_box(model.root_mesh_mut(), 100., 50., 30.);
    let tag = model.add_tag("Furniture");
    model.root_mesh_mut().instances[inst].tag = Some(tag);
    assert_eq!(tessellate(&model).faces.len(), 12);
    model.untagged_hidden = true;
    // The loose top-level faces hide; the group's untagged faces stay with their tagged group.
    assert_eq!(tessellate(&model).faces.len(), 6);
    let ray = Ray { origin: v(50., 25., 500.), dir: DVec3::NEG_Z };
    assert!(model.raycast(&ray).is_none());
    // Untagged groups at the top level hide too.
    model.root_mesh_mut().instances[inst].tag = None;
    assert!(tessellate(&model).faces.is_empty());
    let json = serde_json::to_string(&model).unwrap();
    assert!(serde_json::from_str::<Model>(&json).unwrap().untagged_hidden);
}

#[test]
fn delete_tag_untags_entities_and_scenes() {
    let mut model = box_model();
    let ents: Vec<EntityRef> = model.root_mesh().faces.keys().map(EntityRef::Face).collect();
    let inst = model.make_group(&[], &ents).unwrap();
    let (a, b) = (model.add_tag("A"), model.add_tag("B"));
    model.root_mesh_mut().instances[inst].tag = Some(a);
    let d = model.root_mesh().instances[inst].def;
    let f = model.defs[d].mesh.faces.keys().next().unwrap();
    model.defs[d].mesh.faces[f].tag = Some(a);
    let e = model.defs[d].mesh.edges.keys().next().unwrap();
    model.defs[d].mesh.edges[e].tag = Some(b);
    let mut s = Scene::camera("S", DVec3::ONE, DVec3::ZERO, DVec3::Z, 35.0, false);
    s.hidden_tags = Some(vec![a, b]);
    model.scenes.push(s);
    assert_eq!(model.tag_uses(a), 2);
    model.tags[a].visible = false;
    assert!(tessellate(&model).faces.is_empty());
    assert!(model.delete_tag(a));
    assert!(!model.delete_tag(a));
    assert_eq!(model.tags.len(), 1);
    assert_eq!(model.root_mesh().instances[inst].tag, None);
    assert_eq!(model.defs[d].mesh.faces[f].tag, None);
    assert_eq!(model.defs[d].mesh.edges[e].tag, Some(b));
    assert_eq!(model.scenes[0].hidden_tags, Some(vec![b]));
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

/// The file's preview picture survives save / load; files without one don't write the key.
#[test]
fn preview_image_round_trip() {
    let mut model = box_model();
    let dir = tmpdir("preview");
    let path = dir.join("p.vrst");
    io::save(&model, &path).unwrap();
    assert!(!std::fs::read_to_string(&path).unwrap().contains("\"preview\""));
    let png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x02\0\0\0\x03\x08\x06\0\0\0".to_vec();
    model.set_preview(Some(png.clone()));
    io::save(&model, &path).unwrap();
    let back = io::load(&path).unwrap();
    let p = back.preview.as_ref().unwrap();
    assert_eq!((&p.bytes[..], p.pixels, p.extension()), (&png[..], [2, 3], "png"));
    let mut back = back;
    back.set_preview(None);
    assert!(back.preview.is_none());
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
    // Quads, not triangle pairs.
    assert_eq!(obj.lines().filter(|l| l.starts_with("f ")).count(), 6);
    assert!(obj.lines().filter(|l| l.starts_with("f ")).all(|l| l.split(' ').count() == 5));
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

/// Convex faces go out as polygons; concave faces and faces with holes stay triangulated.
#[test]
fn obj_export_polygons_only_for_convex_faces_without_holes() {
    let face_sizes = |model: &Model| {
        let (obj, _) = io::write_obj(model, "x.mtl");
        let mut n: Vec<usize> = obj.lines().filter(|l| l.starts_with("f ")).map(|l| l.split(' ').count() - 1).collect();
        n.sort();
        n
    };
    let mut m = Model::new();
    m.root_mesh_mut()
        .add_face(&[v(0., 0., 0.), v(300., 0., 0.), v(300., 100., 0.), v(200., 200., 0.), v(0., 100., 0.)])
        .unwrap();
    assert_eq!(face_sizes(&m), [5]);
    // L shape: concave.
    let mut m = Model::new();
    m.root_mesh_mut()
        .add_face(&[v(0., 0., 0.), v(200., 0., 0.), v(200., 100., 0.), v(100., 100., 0.), v(100., 200., 0.), v(0., 200., 0.)])
        .unwrap();
    assert_eq!(face_sizes(&m), [3, 3, 3, 3]);
    // Square with a square hole: the hole's face is a quad, the ring stays triangles.
    let mut m = Model::new();
    let r = m.root_mesh_mut();
    r.add_face(&[v(0., 0., 0.), v(300., 0., 0.), v(300., 300., 0.), v(0., 300., 0.)]).unwrap();
    r.add_face(&[v(100., 100., 0.), v(200., 100., 0.), v(200., 200., 0.), v(100., 200., 0.)]).unwrap();
    let n = face_sizes(&m);
    assert_eq!(n.iter().filter(|k| **k == 4).count(), 1, "{n:?}");
    assert!(n.iter().filter(|k| **k != 4).all(|k| *k == 3), "{n:?}");
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

fn obj_round_trip(model: &Model, name: &str, opts: &io::ImportOptions) -> Model {
    let dir = tmpdir(name);
    let p = dir.join("m.obj");
    io::export_obj(model, &p).unwrap();
    let mut m2 = Model::new();
    io::import_obj_with(&mut m2, &p, opts).unwrap();
    let _ = std::fs::remove_dir_all(dir);
    m2
}

#[test]
fn obj_round_trip_merges_triangles_and_keeps_materials() {
    let mut model = box_model();
    let red = model.add_material("Red Brick", [1., 0., 0., 1.]);
    let glass = model.add_material("Glass", [0.2, 0.4, 0.6, 0.5]);
    let fs: Vec<FaceId> = model.root_mesh().faces.keys().collect();
    model.root_mesh_mut().faces[fs[0]].front = Some(red);
    model.root_mesh_mut().faces[fs[1]].front = Some(glass);
    let m2 = obj_round_trip(&model, "objrt", &io::ImportOptions::default());
    let r = m2.root_mesh();
    ok(r);
    assert_closed_consistent(r);
    assert_eq!((r.faces.len(), r.edges.len(), r.vertices.len()), (6, 12, 8), "triangles merged back into quads");
    assert!((volume(r) - 150_000.).abs() < 1e-3);
    assert_eq!(m2.materials.len(), 2);
    let by_name = |n: &str| m2.materials.iter().find(|(_, m)| m.name == n).map(|(id, m)| (id, m.color)).unwrap();
    let (red2, c) = by_name("Red_Brick");
    assert_eq!(c, [1., 0., 0., 1.]);
    let (glass2, c) = by_name("Glass");
    assert_eq!(c, [0.2, 0.4, 0.6, 0.5]);
    assert_eq!(r.faces.values().filter(|f| f.front == Some(red2)).count(), 1);
    assert_eq!(r.faces.values().filter(|f| f.front == Some(glass2)).count(), 1);
    assert_eq!(r.faces.values().filter(|f| f.front.is_none()).count(), 4);
    // Without merging: the exported polygons (convex quads, not triangles).
    let opts = io::ImportOptions { merge_coplanar: false, materials: false };
    let m3 = obj_round_trip(&model, "objrt_raw", &opts);
    assert_eq!(m3.root_mesh().faces.len(), 6);
    assert!(m3.materials.is_empty());
}

#[test]
fn obj_round_trip_face_with_hole_and_same_material_neighbours() {
    // Box with a square hole through it: the rings come back as faces with holes.
    let mut model = Model::new();
    let m = model.root_mesh_mut();
    make_box(m, 100., 100., 30.);
    rect_lines(m, 30., 30., 60., 60., 30.);
    let inner = m.faces.iter().find(|(id, f)| f.normal.z > 0.9 && f.holes.is_empty() && (m.face_area(*id) - 900.).abs() < 1e-6).unwrap().0;
    m.push_pull(inner, -30.0, false).unwrap();
    let (nf, ne) = (m.faces.len(), m.edges.len());
    let m2 = obj_round_trip(&model, "objhole", &io::ImportOptions::default());
    let r = m2.root_mesh();
    ok(r);
    assert_eq!((r.faces.len(), r.edges.len()), (nf, ne));
    assert_eq!(r.faces.values().filter(|f| f.holes.len() == 1).count(), 2);
    // Two coplanar faces with different materials stay apart.
    let mut model = Model::new();
    let m = model.root_mesh_mut();
    rect_lines(m, 0., 0., 100., 100., 0.);
    m.add_line(DVec3::new(50., 0., 0.), DVec3::new(50., 100., 0.));
    let red = model.add_material("Red", [1., 0., 0., 1.]);
    let f = model.root_mesh().faces.keys().next().unwrap();
    model.root_mesh_mut().faces[f].front = Some(red);
    let m2 = obj_round_trip(&model, "objmat2", &io::ImportOptions::default());
    assert_eq!(m2.root_mesh().faces.len(), 2);
    ok(m2.root_mesh());
}

#[test]
fn obj_import_reads_mtl_details() {
    let dir = tmpdir("objmtl");
    std::fs::write(
        dir.join("a.mtl"),
        "newmtl Wood Oak\nKd 0.5 0.25 0.125\nTr 0.25\nmap_Kd -s 1 1 1 textures/oak.png\n\nnewmtl Clear\nKd 1 1 1\nd 0.3\nTr 0.9\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("a.obj"),
        "mtllib a.mtl\nv 0 0 0\nv 10 0 0\nv 10 10 0\nv 0 10 0\nv 20 0 0\nv 20 10 0\nusemtl Wood Oak\nf 1 2 3\nf 1 3 4\nusemtl Clear\nf 2 5 6 3\nusemtl Missing\nf 5 6 3\n",
    )
    .unwrap();
    let mut m = Model::new();
    io::import_obj(&mut m, &dir.join("a.obj")).unwrap();
    let wood = m.materials.values().find(|x| x.name == "Wood Oak").unwrap();
    assert_eq!(wood.color, [0.5, 0.25, 0.125, 0.75]);
    assert_eq!(wood.texture.as_deref(), Some("textures/oak.png"));
    let clear = m.materials.values().find(|x| x.name == "Clear").unwrap();
    assert_eq!(clear.color[3], 0.3, "d wins over Tr");
    assert_eq!(m.materials.len(), 2, "undefined materials are not invented");
    let r = m.root_mesh();
    ok(r);
    // The two Wood triangles merge into one square.
    let wood_id = m.materials.iter().find(|(_, x)| x.name == "Wood Oak").unwrap().0;
    let woods: Vec<_> = r.faces.iter().filter(|(_, f)| f.front == Some(wood_id)).collect();
    assert_eq!(woods.len(), 1);
    assert!((r.face_area(woods[0].0) - 100.).abs() < 1e-9);
    // Importing the same file again reuses the materials.
    io::import_obj(&mut m, &dir.join("a.obj")).unwrap();
    assert_eq!(m.materials.len(), 2);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn stl_import_merges_coplanar_triangles() {
    let model = box_model();
    let dir = tmpdir("stlmerge");
    let p = dir.join("box.stl");
    io::export_stl(&model, &p).unwrap();
    let mut m2 = Model::new();
    io::import_stl(&mut m2, &p).unwrap();
    assert_eq!(m2.root_mesh().faces.len(), 6);
    let mut m3 = Model::new();
    io::import_stl_with(&mut m3, &p, &io::ImportOptions { merge_coplanar: false, materials: true }).unwrap();
    assert_eq!(m3.root_mesh().faces.len(), 12);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn obj_import_merges_flat_triangle_grids() {
    // n x n cells of two triangles; with `hole` the centre cell is missing (a ring with a hole).
    // Odd sizes put the merged square's centre on a cell diagonal.
    for (n, hole) in [(1usize, false), (2, false), (3, false), (3, true), (5, true), (10, false)] {
        let mut s = String::new();
        for j in 0..=n {
            for i in 0..=n {
                s += &format!("v {} {} 0\n", i as f64 * 100.0, j as f64 * 100.0);
            }
        }
        let at = |i: usize, j: usize| j * (n + 1) + i + 1;
        for j in 0..n {
            for i in 0..n {
                if hole && i == n / 2 && j == n / 2 {
                    continue;
                }
                s += &format!("f {} {} {}\nf {} {} {}\n", at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j), at(i + 1, j + 1), at(i, j + 1));
            }
        }
        let dir = tmpdir(&format!("flat{n}{hole}"));
        std::fs::write(dir.join("f.obj"), s).unwrap();
        let mut m = Model::new();
        io::import_obj(&mut m, &dir.join("f.obj")).unwrap();
        let _ = std::fs::remove_dir_all(dir);
        let r = m.root_mesh();
        ok(r);
        assert_eq!(r.faces.len(), 1, "n {n} hole {hole}");
        let (f, face) = r.faces.iter().next().unwrap();
        assert!(face.normal.z > 0.999);
        assert_eq!(face.holes.len(), hole as usize);
        let area = (n * n - hole as usize) as f64 * 10_000.0;
        assert!((r.face_area(f) - area).abs() < 1e-6, "n {n} hole {hole}");
        assert_eq!(r.edges.len(), if hole { 8 } else { 4 }, "collinear boundary pieces healed");
    }
}

#[test]
fn scenes_style_tags_and_settings_round_trip() {
    let mut m = box_model();
    let tag = m.tags.insert(Tag { name: "Roof".into(), visible: false, color: [1.0; 4] });
    let mut s = Scene::camera("A", DVec3::new(1.0, 2.0, 3.0), DVec3::ZERO, DVec3::Z, 35.0, false);
    s.style = Some(SceneStyle { face_style: "wireframe".into(), xray: true, edges: false, profiles: true });
    s.hidden_tags = Some(vec![tag]);
    s.in_animation = false;
    m.scenes.push(s);
    m.scenes.push(Scene::camera("B", DVec3::X, DVec3::ZERO, DVec3::Z, 35.0, true));
    m.scene_settings = SceneSettings { transitions: false, transition_secs: 0.5, delay_secs: 3.0 };
    let back = io::from_json(&io::to_json(&m).unwrap()).unwrap();
    assert_eq!(back.scene_settings, m.scene_settings);
    let (a, b) = (&back.scenes[0], &back.scenes[1]);
    assert_eq!(a.style, m.scenes[0].style);
    assert_eq!(a.hidden_tags, Some(vec![tag]));
    assert!(!a.in_animation && b.in_animation && b.style.is_none() && b.hidden_tags.is_none());

    // Files from before scene styles: camera-only scenes, default settings.
    let mut v: serde_json::Value = serde_json::from_str(&io::to_json(&m).unwrap()).unwrap();
    let obj = v.as_object_mut().unwrap();
    let model = obj.get_mut("model").and_then(|x| x.as_object_mut()).unwrap();
    model.remove("scene_settings");
    for sc in model["scenes"].as_array_mut().unwrap() {
        let sc = sc.as_object_mut().unwrap();
        sc.remove("style");
        sc.remove("hidden_tags");
        sc.remove("in_animation");
    }
    let old = io::from_json(&v.to_string()).unwrap();
    assert_eq!(old.scene_settings, SceneSettings::default());
    assert!(old.scenes.iter().all(|s| s.in_animation && s.style.is_none()));
}

/// A 2×1 PNG header is enough for the kernel (it never decodes pixels).
fn fake_png(w: u32, h: u32) -> Vec<u8> {
    let mut b = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    b.extend(w.to_be_bytes());
    b.extend(h.to_be_bytes());
    b.extend([8, 6, 0, 0, 0]);
    b
}

fn top_face(mesh: &Mesh) -> FaceId {
    mesh.faces.iter().find(|(_, f)| f.normal.z > 0.9).map(|(id, _)| id).unwrap()
}

#[test]
fn texture_coordinates_follow_groups_and_placement() {
    let mut m = box_model();
    let tex = m.add_material("Brick", [1.0; 4]);
    m.materials[tex].image = Some(TextureImage::new(fake_png(200, 100), "brick.png", 400.0));
    assert_eq!(m.materials[tex].image.as_ref().unwrap().size, [400.0, 200.0]);
    let top = top_face(m.root_mesh());
    m.root_mesh_mut().faces[top].front = Some(tex);
    let uv_of = |m: &Model| {
        let t = tessellate(m);
        let f = t.faces.iter().find(|f| f.front == Some(tex)).unwrap();
        assert_eq!(f.uv.len(), f.positions.len());
        f.positions.iter().zip(&f.uv).map(|(p, u)| (*p, *u)).collect::<Vec<_>>()
    };
    // Top face: u = x, v = y (mm).
    for (p, u) in uv_of(&m) {
        assert!((u.x - p.x).abs() < 1e-9 && (u.y - p.y).abs() < 1e-9);
    }
    // Inside a moved group the coordinates stay those of the local geometry.
    let ents: Vec<EntityRef> = m.root_mesh().faces.keys().map(EntityRef::Face).collect();
    let g = m.make_group(&[], &ents).unwrap();
    m.root_mesh_mut().instances[g].transform = DMat4::from_translation(DVec3::new(5000.0, 0.0, 0.0));
    for (p, u) in uv_of(&m) {
        assert!((u.x - (p.x - 5000.0)).abs() < 1e-6, "{p} {u}");
    }
    // Placement: rotate 90° → u follows -y... and survives splitting the face.
    let def = m.root_mesh().instances[g].def;
    let mesh = &mut m.defs[def].mesh;
    let top = top_face(mesh);
    mesh.faces[top].uv = Some(TexPlacement { angle_deg: 90.0, ..Default::default() });
    mesh.add_line(DVec3::new(50.0, 0.0, 30.0), DVec3::new(50.0, 50.0, 30.0));
    let tops: Vec<&Face> = mesh.faces.values().filter(|f| f.normal.z > 0.9).collect();
    assert_eq!(tops.len(), 2);
    assert!(tops.iter().all(|f| f.uv.is_some_and(|p| p.angle_deg == 90.0) && f.front == Some(tex)));
    for (p, u) in uv_of(&m) {
        assert!((u.x - (p.y)).abs() < 1e-6 && (u.y + (p.x - 5000.0)).abs() < 1e-6, "{p} {u}");
    }

    // .vrst keeps the image bytes, size and the face placement.
    let back = io::from_json(&io::to_json(&m).unwrap()).unwrap();
    let img = back.materials.values().find_map(|mm| mm.image.as_ref()).unwrap();
    assert_eq!((&img.bytes[..], img.size, img.pixels), (&fake_png(200, 100)[..], [400.0, 200.0], [200, 100]));
    assert_eq!(back.defs[def].mesh.faces.values().filter(|f| f.uv.is_some()).count(), 2);
}

#[test]
fn obj_export_writes_texture_and_uvs_import_embeds_it() {
    let mut m = box_model();
    let tex = m.add_material("Brick Wall", [0.6, 0.3, 0.2, 1.0]);
    m.materials[tex].image = Some(TextureImage::new(fake_png(64, 64), "bricks.png", 1000.0));
    let top = top_face(m.root_mesh());
    m.root_mesh_mut().faces[top].front = Some(tex);
    let (obj, mtl, files) = io::write_obj_with_textures(&m, "t.mtl");
    assert!(mtl.contains("map_Kd Brick_Wall.png"), "{mtl}");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].0, "Brick_Wall.png");
    // Top face 100×50 → one quad with texture coordinates in tiles of 1000 mm.
    assert_eq!(obj.lines().filter(|l| l.starts_with("vt ")).count(), 4);
    assert!(obj.lines().any(|l| l.starts_with("vt 0.1 0.05")));
    assert_eq!(obj.lines().filter(|l| l.starts_with("f ") && l.split(' ').nth(1).unwrap().split('/').nth(1) != Some("")).count(), 1);

    let dir = tmpdir("objtex");
    let path = dir.join("t.obj");
    io::export_obj(&m, &path).unwrap();
    assert_eq!(std::fs::read(dir.join("Brick_Wall.png")).unwrap(), fake_png(64, 64));
    let mut back = Model::new();
    io::import_obj(&mut back, &path).unwrap();
    let mm = back.materials.values().find(|mm| mm.name == "Brick_Wall").unwrap();
    let img = mm.image.as_ref().expect("embedded texture");
    assert_eq!((img.pixels, img.file_name.as_str()), ([64, 64], "Brick_Wall.png"));
    assert_eq!(back.root_mesh().faces.len(), 6);
    // `vt` come back as the same texture position (top face, merged from two triangles).
    let pl = TexPlacement { offset: [30.0, -20.0], angle_deg: 30.0, scale: [0.5, 2.0] };
    m.root_mesh_mut().faces[top].uv = Some(pl);
    io::export_obj(&m, &path).unwrap();
    let mut back = Model::new();
    io::import_obj(&mut back, &path).unwrap();
    assert_eq!(back.root_mesh().faces.len(), 6);
    let f = back.root_mesh().faces.values().find(|f| f.front.is_some()).unwrap();
    let got = f.uv.expect("fitted placement");
    for v in &f.outer {
        let p = back.root_mesh().vertices[*v].pos;
        let (a, b) = (texture::face_uv_mm(p, f.normal, Some(&got)), texture::face_uv_mm(p, f.normal, Some(&pl)));
        assert!((a - b).length() < 1e-3, "{got:?}");
    }
}

#[test]
fn section_cut_lines_and_clipped_raycast() {
    let mut m = Model::new();
    box_at(m.root_mesh_mut(), DVec3::ZERO, DVec3::new(1000.0, 500.0, 300.0));
    let s = SectionPlane::new("Section 1", DVec3::new(400.0, 0.0, 0.0), -DVec3::X);
    let cut = m.section_cut(&s);
    assert_eq!(cut.len(), 4, "{cut:?}");
    let len: f64 = cut.iter().map(|[a, b]| (*b - *a).length()).sum();
    assert!((len - 1600.0).abs() < 1e-6, "{len}");
    assert!(cut.iter().all(|[a, b]| (a.x - 400.0).abs() < 1e-9 && (b.x - 400.0).abs() < 1e-9));
    let tri_area = |t: &[DVec3; 3]| (t[1] - t[0]).cross(t[2] - t[0]).length() * 0.5;
    let fill = varstotas_kernel::section::section_fill(&s, &cut);
    assert!((fill.iter().map(tri_area).sum::<f64>() - 500.0 * 300.0).abs() < 1e-6);
    // Two separate boxes: two filled rectangles.
    let mut tube = Model::new();
    box_at(tube.root_mesh_mut(), DVec3::ZERO, DVec3::new(1000.0, 500.0, 300.0));
    box_at(tube.root_mesh_mut(), DVec3::new(0.0, 2000.0, 0.0), DVec3::new(1000.0, 2500.0, 300.0));
    let both = tube.section_cut(&s);
    assert_eq!(both.len(), 8);
    let fill2 = varstotas_kernel::section::section_fill(&s, &both);
    assert!((fill2.iter().map(tri_area).sum::<f64>() - 2.0 * 500.0 * 300.0).abs() < 1e-6);
    // Ray from +x into the box: hits the face x=1000 without a section, the far face x=0 with it
    // (the box beyond x=400 is cut away, and the inside of the back face is hit).
    let ray = Ray { origin: DVec3::new(2000.0, 250.0, 150.0), dir: -DVec3::X };
    assert!((m.raycast(&ray).unwrap().point.x - 1000.0).abs() < 1e-9);
    m.sections.push(s);
    let h = m.raycast(&ray).unwrap();
    assert!(h.point.x.abs() < 1e-9, "{:?}", h.point);
    m.section_display.cuts = false;
    assert!((m.raycast(&ray).unwrap().point.x - 1000.0).abs() < 1e-9);
    // Round trip.
    m.section_display.cuts = true;
    let back: Model = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
    assert_eq!(back.sections, m.sections);
    assert!(back.active_section().is_some());
}
