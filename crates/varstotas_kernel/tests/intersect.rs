//! Intersect Faces, Reverse/Orient Faces, Flip Along, and welding on explode/duplicate.

mod common;
use common::*;
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::*;

fn all_faces(m: &Mesh) -> Vec<EntityRef> {
    m.faces.keys().map(EntityRef::Face).collect()
}

/// Group a closed box [lo, hi] in the root of `model`, returns the instance.
fn group_box(model: &mut Model, lo: DVec3, hi: DVec3) -> InstanceId {
    // Build in a scratch model so the box does not stick to root geometry.
    let mut scratch = Model::new();
    box_at(scratch.root_mesh_mut(), lo, hi);
    let ents = all_faces(scratch.root_mesh());
    let g = scratch.make_group(&[], &ents).unwrap();
    let def = scratch.root_mesh().instances[g].def;
    let d = scratch.defs.remove(def).unwrap();
    let nd = model.defs.insert(d);
    let r = model.root;
    model.defs[r].mesh.instances.insert(Instance {
        def: nd,
        transform: DMat4::IDENTITY,
        name: String::new(),
        material: None,
        tag: None,
        hidden: false,
        locked: false,
    })
}

fn faces_with_normal(m: &Mesh, n: DVec3) -> Vec<FaceId> {
    face_with_normal(m, n)
}

#[test]
fn two_loose_boxes_crossing() {
    let mut m = Mesh::new();
    box_at(&mut m, v(0., 0., 0.), v(100., 100., 100.));
    box_at(&mut m, v(50., 50., 50.), v(150., 150., 150.));
    ok(&m);
    assert_eq!(m.faces.len(), 12);
    let area = total_area(&m);
    let vol = volume(&m);
    let new = m.intersect_all();
    sane(&m);
    // Each box gets three faces split by the other: 12 + 6.
    assert_eq!(m.faces.len(), 18, "faces after intersect");
    // The intersection curve: 6 segments, each lying on a face of both boxes.
    assert_eq!(new.len(), 6);
    assert!((total_area(&m) - area).abs() < 1e-6);
    assert!((volume(&m) - vol).abs() < 1e-3);
    // The overlap pieces: a 50x50 square on A's top at z=100 inside B.
    let top: Vec<FaceId> = faces_with_normal(&m, DVec3::Z).into_iter().filter(|f| (m.face_plane(*f).origin.z - 100.0).abs() < 1e-9).collect();
    let mut areas: Vec<f64> = top.iter().map(|f| m.face_area(*f)).collect();
    areas.sort_by(f64::total_cmp);
    assert_eq!(areas.len(), 2);
    assert!((areas[0] - 2500.0).abs() < 1e-6 && (areas[1] - 7500.0).abs() < 1e-6, "{areas:?}");
    // Idempotent.
    let again = m.intersect_all();
    assert!(again.is_empty(), "second intersect created {} edges", again.len());
    assert_eq!(m.faces.len(), 18);
}

#[test]
fn box_group_through_plane() {
    let mut model = Model::new();
    let plane = model.root_mesh_mut().add_face(&[v(-100., -100., 50.), v(300., -100., 50.), v(300., 300., 50.), v(-100., 300., 50.)]).unwrap();
    let g = group_box(&mut model, v(0., 0., 0.), v(100., 100., 100.));
    let new = model.intersect_with_model(&[], &[EntityRef::Instance(g)]).unwrap();
    let r = model.root_mesh();
    sane(r);
    assert_eq!(new.len(), 4);
    assert_eq!(r.faces.len(), 2);
    let mut areas: Vec<f64> = r.faces.keys().map(|f| r.face_area(f)).collect();
    areas.sort_by(f64::total_cmp);
    assert!((areas[0] - 10_000.0).abs() < 1e-6, "{areas:?}");
    assert!((areas[1] - (160_000.0 - 10_000.0)).abs() < 1e-6, "{areas:?}");
    // The group itself is untouched.
    assert_eq!(model.mesh_at(&[g]).unwrap().faces.len(), 6);
    // Selecting the plane instead gives the same result (idempotent now).
    let plane = if r.faces.contains_key(plane) { plane } else { r.faces.keys().next().unwrap() };
    let again = model.intersect_with_model(&[], &[EntityRef::Face(plane)]).unwrap();
    assert!(again.is_empty());
    assert_eq!(model.root_mesh().faces.len(), 2);
}

#[test]
fn plane_selected_intersects_with_group() {
    let mut model = Model::new();
    let plane = model.root_mesh_mut().add_face(&[v(-100., -100., 50.), v(300., -100., 50.), v(300., 300., 50.), v(-100., 300., 50.)]).unwrap();
    group_box(&mut model, v(0., 0., 0.), v(100., 100., 100.));
    let new = model.intersect_with_model(&[], &[EntityRef::Face(plane)]).unwrap();
    assert_eq!(new.len(), 4);
    sane(model.root_mesh());
    assert_eq!(model.root_mesh().faces.len(), 2);
}

#[test]
fn intersect_scopes() {
    // Two crossing group boxes and a loose plane through both.
    let mut model = Model::new();
    model.root_mesh_mut().add_face(&[v(-100., -100., 75.), v(300., -100., 75.), v(300., 300., 75.), v(-100., 300., 75.)]).unwrap();
    let a = group_box(&mut model, v(0., 0., 0.), v(100., 100., 100.));
    let b = group_box(&mut model, v(50., 50., 50.), v(150., 150., 150.));
    let sel = [EntityRef::Instance(a), EntityRef::Instance(b)];
    // Selection only: loose edges along A ∩ B, the plane is not split.
    let new = model.intersect_with_selection(&[], &sel).unwrap();
    let r = model.root_mesh();
    ok(r);
    assert_eq!(r.faces.len(), 1);
    assert_eq!(new.len(), 6, "edges of the A/B intersection curve");
    assert!(new.iter().all(|e| r.edge_faces(*e).is_empty()));
    // With the model: the plane gets both footprints.
    let n0 = model.root_mesh().faces.len();
    model.intersect_with_model(&[], &[EntityRef::Instance(a)]).unwrap();
    let r = model.root_mesh();
    sane(r);
    assert!(r.faces.len() > n0);
    // Groups untouched.
    assert_eq!(model.mesh_at(&[a]).unwrap().faces.len(), 6);
    assert_eq!(model.mesh_at(&[b]).unwrap().faces.len(), 6);
}

#[test]
fn coplanar_overlap_with_group_imprints_boundary() {
    let mut model = Model::new();
    model.root_mesh_mut().add_face(&[v(0., 0., 10.), v(100., 0., 10.), v(100., 100., 10.), v(0., 100., 10.)]).unwrap();
    // A group face partially overlapping the loose one.
    let mut scratch = Mesh::new();
    let gf = scratch.add_face(&[v(50., 50., 10.), v(150., 50., 10.), v(150., 150., 10.), v(50., 150., 10.)]).unwrap();
    let _ = gf;
    let r = model.root;
    let def = model.defs.insert(Definition { name: "G".into(), mesh: scratch, is_group: true });
    let g = model.defs[r].mesh.instances.insert(Instance {
        def,
        transform: DMat4::IDENTITY,
        name: String::new(),
        material: None,
        tag: None,
        hidden: false,
        locked: false,
    });
    let new = model.intersect_with_model(&[], &[EntityRef::Instance(g)]).unwrap();
    let r = model.root_mesh();
    sane(r);
    assert_eq!(new.len(), 2);
    assert_eq!(r.faces.len(), 2);
    let mut areas: Vec<f64> = r.faces.keys().map(|f| r.face_area(f)).collect();
    areas.sort_by(f64::total_cmp);
    assert!((areas[0] - 2500.0).abs() < 1e-6 && (areas[1] - 7500.0).abs() < 1e-6, "{areas:?}");
}

#[test]
fn faces_touching_along_edges_add_nothing() {
    // A group box exactly next to a loose box (shared wall plane, same extent).
    let mut model = Model::new();
    box_at(model.root_mesh_mut(), v(0., 0., 0.), v(100., 100., 100.));
    let g = group_box(&mut model, v(100., 0., 0.), v(200., 100., 100.));
    let new = model.intersect_with_model(&[], &[EntityRef::Instance(g)]).unwrap();
    assert!(new.is_empty(), "created {} edges", new.len());
    assert_eq!(model.root_mesh().faces.len(), 6);
    // Shifted by half: the shared wall gets the group's wall outline.
    let mut model = Model::new();
    box_at(model.root_mesh_mut(), v(0., 0., 0.), v(100., 100., 100.));
    let g = group_box(&mut model, v(100., 50., 0.), v(200., 150., 100.));
    let new = model.intersect_with_model(&[], &[EntityRef::Instance(g)]).unwrap();
    let r = model.root_mesh();
    sane(r);
    assert_eq!(new.len(), 1);
    assert_eq!(r.faces.len(), 7);
}

#[test]
fn box_group_sitting_on_plane_imprints_footprint() {
    let mut model = Model::new();
    model.root_mesh_mut().add_face(&[v(-100., -100., 0.), v(300., -100., 0.), v(300., 300., 0.), v(-100., 300., 0.)]).unwrap();
    let g = group_box(&mut model, v(0., 0., 0.), v(100., 100., 100.));
    let new = model.intersect_with_model(&[], &[EntityRef::Instance(g)]).unwrap();
    let r = model.root_mesh();
    sane(r);
    assert_eq!(new.len(), 4);
    assert_eq!(r.faces.len(), 2);
}

#[test]
fn plane_with_hole_crossed_by_box() {
    let mut model = Model::new();
    {
        let m = model.root_mesh_mut();
        m.add_face(&[v(0., 0., 0.), v(300., 0., 0.), v(300., 300., 0.), v(0., 300., 0.)]).unwrap();
        let inner = m.add_face(&[v(100., 100., 0.), v(200., 100., 0.), v(200., 200., 0.), v(100., 200., 0.)]).unwrap();
        m.erase(&[EntityRef::Face(inner)]);
        assert_eq!(m.faces.len(), 1);
    }
    let area = total_area(model.root_mesh());
    let g = group_box(&mut model, v(150., 120., -50.), v(250., 180., 50.));
    model.intersect_with_model(&[], &[EntityRef::Instance(g)]).unwrap();
    let r = model.root_mesh();
    sane(r);
    assert_eq!(r.faces.len(), 2);
    assert!((total_area(r) - area).abs() < 1e-6);
    let mut areas: Vec<f64> = r.faces.keys().map(|f| r.face_area(f)).collect();
    areas.sort_by(f64::total_cmp);
    assert!((areas[0] - 50. * 60.).abs() < 1e-6, "{areas:?}");
    // The piece with the hole still has it.
    assert!(r.faces.values().any(|f| f.holes.len() == 1 || f.outer.len() > 4));
}

#[test]
fn nested_transformed_instances() {
    let mut model = Model::new();
    model.root_mesh_mut().add_face(&[v(-1000., -1000., 30.), v(1000., -1000., 30.), v(1000., 1000., 30.), v(-1000., 1000., 30.)]).unwrap();
    // Inner group: box 0..100 x 0..50 x 0..100, rotated 90° about Z inside an outer group that
    // is translated by (200, 0, -20).
    let inner = group_box(&mut model, v(0., 0., 0.), v(100., 50., 100.));
    model.root_mesh_mut().instances[inner].transform = DMat4::from_rotation_z(std::f64::consts::FRAC_PI_2);
    let outer = model.make_group(&[], &[EntityRef::Instance(inner)]).unwrap();
    model.root_mesh_mut().instances[outer].transform = DMat4::from_translation(v(200., 0., -20.));
    let new = model.intersect_with_model(&[], &[EntityRef::Instance(outer)]).unwrap();
    let r = model.root_mesh();
    sane(r);
    assert_eq!(new.len(), 4);
    assert_eq!(r.faces.len(), 2);
    // Footprint: rotated box spans x in [-50, 0] (+200) and y in [0, 100].
    let small = r.faces.keys().min_by(|a, b| r.face_area(*a).total_cmp(&r.face_area(*b))).unwrap();
    assert!((r.face_area(small) - 5000.0).abs() < 1e-6);
    let bb = r.face_aabb(small).unwrap();
    assert!((bb.min.x - 150.).abs() < 1e-9 && (bb.max.x - 200.).abs() < 1e-9, "{bb:?}");
    assert!((bb.min.y - 0.).abs() < 1e-9 && (bb.max.y - 100.).abs() < 1e-9, "{bb:?}");
}

#[test]
fn intersect_inside_group_lands_in_group() {
    let mut model = Model::new();
    let f = model.root_mesh_mut().add_face(&[v(-100., -100., 50.), v(300., -100., 50.), v(300., 300., 50.), v(-100., 300., 50.)]).unwrap();
    let g = model.make_group(&[], &[EntityRef::Face(f)]).unwrap();
    model.root_mesh_mut().instances[g].transform = DMat4::from_translation(v(0., 0., 10.)); // plane at z = 60
    box_at(model.root_mesh_mut(), v(0., 0., 0.), v(100., 100., 100.));
    let gf = model.mesh_at(&[g]).unwrap().faces.keys().next().unwrap();
    // Context scope: nothing else in the group.
    let none = model.intersect_with_context(&[g], &[EntityRef::Face(gf)]).unwrap();
    assert!(none.is_empty());
    let new = model.intersect_with_model(&[g], &[EntityRef::Face(gf)]).unwrap();
    assert_eq!(new.len(), 4);
    let gm = model.mesh_at(&[g]).unwrap();
    sane(gm);
    assert_eq!(gm.faces.len(), 2);
    // The new edges are at local z = 50 (world 60).
    assert!(new.iter().all(|e| {
        let (a, b) = gm.edge_segment(*e);
        (a.z - 50.).abs() < 1e-9 && (b.z - 50.).abs() < 1e-9
    }));
    // Root untouched.
    assert_eq!(model.root_mesh().faces.len(), 6);
    assert!(model.root_mesh().edges.len() == 12);
}

#[test]
fn cylinder_vs_box() {
    let mut m = Mesh::new();
    // Cylinder r = 60 around (100, 50), height 200, 24 segments.
    let pts: Vec<DVec3> = (0..24)
        .map(|i| {
            let a = i as f64 / 24.0 * std::f64::consts::TAU;
            v(100. + 60. * a.cos(), 50. + 60. * a.sin(), -50.)
        })
        .collect();
    m.add_polyline(&pts, true, true);
    let disc = m.faces.keys().next().unwrap();
    let d = if m.faces[disc].normal.z < 0.0 { -200.0 } else { 200.0 };
    m.push_pull(disc, d, false).unwrap();
    box_at(&mut m, v(0., 0., 0.), v(100., 100., 100.));
    ok(&m);
    let area = total_area(&m);
    let new = m.intersect_all();
    sane(&m);
    assert!(!new.is_empty());
    assert!((total_area(&m) - area).abs() < 1e-6);
    // The box top (z = 100) is split into the part inside the cylinder and outside.
    let tops: Vec<FaceId> = m.faces.keys().filter(|f| m.faces[*f].normal.z > 0.999 && (m.face_plane(*f).origin.z - 100.0).abs() < 1e-9).collect();
    assert_eq!(tops.len(), 2);
    assert!(m.intersect_all().is_empty());
}

#[test]
fn intersect_is_one_undo_step() {
    let mut model = Model::new();
    model.root_mesh_mut().add_face(&[v(-100., -100., 50.), v(300., -100., 50.), v(300., 300., 50.), v(-100., 300., 50.)]).unwrap();
    let g = group_box(&mut model, v(0., 0., 0.), v(100., 100., 100.));
    let mut h = History::default();
    let before = model.clone();
    model.intersect_with_model(&[], &[EntityRef::Instance(g)]).unwrap();
    h.push("Intersect Faces", before);
    assert_eq!(model.root_mesh().faces.len(), 2);
    h.undo(&mut model);
    assert_eq!(model.root_mesh().faces.len(), 1);
    assert_eq!(model.root_mesh().edges.len(), 4);
}

#[test]
fn intersect_bad_path_and_empty_selection() {
    let mut model = Model::new();
    let bogus = {
        let mut other = Model::new();
        let f = other.root_mesh_mut().add_face(&[v(0., 0., 1.), v(1., 0., 1.), v(1., 1., 1.)]).unwrap();
        other.make_group(&[], &[EntityRef::Face(f)]).unwrap()
    };
    assert_eq!(model.intersect_with_model(&[bogus], &[]), Err(OpError::NotFound));
    assert_eq!(model.intersect_with_model(&[], &[]).unwrap(), Vec::<EdgeId>::new());
}

// ---- Reverse / Orient / Flip -----------------------------------------------------------------

#[test]
fn reverse_faces_flips_normal_and_materials() {
    let mut model = Model::new();
    let red = model.add_material("red", [1., 0., 0., 1.]);
    let m = model.root_mesh_mut();
    let f = m.add_face(&[v(0., 0., 10.), v(10., 0., 10.), v(10., 10., 10.), v(0., 10., 10.)]).unwrap();
    m.faces[f].front = Some(red);
    let n = m.faces[f].normal;
    assert_eq!(m.reverse_faces(&[f, f]), 1);
    ok(m);
    assert!((m.faces[f].normal + n).length() < 1e-12);
    assert_eq!(m.faces[f].back, Some(red));
    assert_eq!(m.faces[f].front, None);
}

#[test]
fn orient_faces_makes_box_consistent() {
    let mut m = Mesh::new();
    box_at(&mut m, v(0., 0., 0.), v(100., 50., 30.));
    let ids: Vec<FaceId> = m.faces.keys().collect();
    // Reverse three faces, orient from an outward one.
    m.reverse_faces(&ids[1..4]);
    let keep = ids[0];
    let flipped = m.orient_faces_like(keep);
    assert_eq!(flipped, 3);
    ok(&m);
    assert_closed_consistent(&m);
    assert_outward(&m);
    // Orienting from an inward face turns the whole box inside out.
    m.reverse_face(keep);
    assert_eq!(m.orient_faces_like(keep), 5);
    assert_closed_consistent(&m);
    assert!(volume(&m) < 0.0);
}

#[test]
fn flip_along_loose_geometry_keeps_outward_normals() {
    let mut model = Model::new();
    let m = model.root_mesh_mut();
    // An L-shaped prism: box with a notch.
    box_at(m, v(0., 0., 0.), v(100., 50., 30.));
    box_at(m, v(100., 0., 0.), v(130., 20., 30.));
    let vol = volume(m);
    let ents = all_faces(m);
    model.flip_along(&[], &ents, DVec3::X).unwrap();
    let m = model.root_mesh();
    ok(m);
    assert!((volume(m) - vol).abs() < 1e-6, "volume {} -> {}", vol, volume(m));
    // The notch moved to the other side: bbox unchanged, notch now at x in [0, 30].
    let b = m.bounds().unwrap();
    assert!((b.min.x).abs() < 1e-9 && (b.max.x - 130.).abs() < 1e-9);
    assert!(m.vertices.values().any(|vx| (vx.pos - v(0., 20., 30.)).length() < 1e-9));
    assert!(!m.vertices.values().any(|vx| (vx.pos - v(130., 20., 30.)).length() < 1e-9));
}

#[test]
fn flip_along_instance_mirrors_transform_about_its_centre() {
    let mut model = Model::new();
    let g = group_box(&mut model, v(0., 0., 0.), v(100., 50., 30.));
    model.root_mesh_mut().instances[g].transform = DMat4::from_translation(v(500., 0., 0.));
    model.flip_along(&[], &[EntityRef::Instance(g)], DVec3::Y).unwrap();
    let t = model.root_mesh().instances[g].transform;
    assert!(t.determinant() < 0.0);
    let b = model.bounds().unwrap();
    assert!((b.min - v(500., 0., 0.)).length() < 1e-9 && (b.max - v(600., 50., 30.)).length() < 1e-9, "{b:?}");
    // Tessellated normals still point outward (mirrored instance flips the winding).
    let tess = tessellate(&model);
    let c = b.center();
    for f in &tess.faces {
        let p = f.positions[0];
        let p = DVec3::new(p[0], p[1], p[2]);
        let n = DVec3::new(f.normal.x, f.normal.y, f.normal.z);
        assert!(n.dot(p - c) > 0.0 || n.dot(DVec3::new(f.positions[1][0], f.positions[1][1], f.positions[1][2]) - c) > 0.0);
    }
    assert!(model.flip_along(&[], &[], DVec3::X).is_err());
    assert!(model.flip_along(&[], &[EntityRef::Instance(g)], DVec3::ZERO).is_err());
}

// ---- Welding on explode / duplicate ---------------------------------------------------------------

#[test]
fn explode_welds_into_existing_geometry() {
    // A group box half overlapping the wall of a loose box: after exploding, the shared wall
    // part is split cleanly (no T-junctions, no overlapping faces).
    let mut model = Model::new();
    box_at(model.root_mesh_mut(), v(0., 0., 0.), v(100., 100., 100.));
    let g = group_box(&mut model, v(100., 50., 0.), v(200., 150., 100.));
    let out = model.explode(&[], g).unwrap();
    let r = model.root_mesh();
    sane(r);
    assert!(out.iter().all(|e| match e {
        EntityRef::Face(f) => r.faces.contains_key(*f),
        EntityRef::Edge(x) => r.edges.contains_key(*x),
        _ => true,
    }));
    // The copied group's six faces are all in the result.
    assert!(out.iter().filter(|e| matches!(e, EntityRef::Face(_))).count() >= 6);
}

#[test]
fn explode_face_inside_existing_face_makes_hole_partition() {
    let mut model = Model::new();
    model.root_mesh_mut().add_face(&[v(0., 0., 10.), v(100., 0., 10.), v(100., 100., 10.), v(0., 100., 10.)]).unwrap();
    let mut scratch = Model::new();
    let f = scratch.root_mesh_mut().add_face(&[v(25., 25., 10.), v(75., 25., 10.), v(75., 75., 10.), v(25., 75., 10.)]).unwrap();
    let g = scratch.make_group(&[], &[EntityRef::Face(f)]).unwrap();
    let def = scratch.root_mesh().instances[g].def;
    let d = scratch.defs.remove(def).unwrap();
    let nd = model.defs.insert(d);
    let r = model.root;
    let inst = model.defs[r].mesh.instances.insert(Instance {
        def: nd,
        transform: DMat4::IDENTITY,
        name: String::new(),
        material: None,
        tag: None,
        hidden: false,
        locked: false,
    });
    let out = model.explode(&[], inst).unwrap();
    let r = model.root_mesh();
    sane(r);
    assert_eq!(r.faces.len(), 2);
    assert!((total_area(r) - 10_000.0).abs() < 1e-6);
    let inner: Vec<FaceId> = out.iter().filter_map(|e| if let EntityRef::Face(f) = e { Some(*f) } else { None }).collect();
    assert_eq!(inner.len(), 1);
    assert!((r.face_area(inner[0]) - 2500.0).abs() < 1e-6);
}

#[test]
fn duplicate_welds_t_junctions_and_overlaps() {
    let mut m = Mesh::new();
    let f = m.add_face(&[v(0., 0., 10.), v(100., 0., 10.), v(100., 100., 10.), v(0., 100., 10.)]).unwrap();
    // Copy shifted by half: overlaps the original.
    let out = m.duplicate(&[EntityRef::Face(f)], &DMat4::from_translation(v(50., 50., 0.)));
    sane(&m);
    assert!((total_area(&m) - (20_000.0 - 2500.0)).abs() < 1e-6, "area {}", total_area(&m));
    // The returned faces cover the copy's footprint (2 pieces: overlap + rest).
    let area: f64 = out.iter().filter_map(|e| if let EntityRef::Face(f) = e { Some(m.face_area(*f)) } else { None }).sum();
    assert!((area - 10_000.0).abs() < 1e-6, "copied area {area}");
    // A box copied so that it touches the original's wall partially (T-junctions).
    let mut m = Mesh::new();
    box_at(&mut m, v(0., 0., 0.), v(100., 100., 100.));
    let ents = all_faces(&m);
    m.duplicate(&ents, &DMat4::from_translation(v(100., 50., 0.)));
    sane(&m);
}
