mod common;
use varstotas_kernel::{
    Edge, EntityRef, Face, InstanceId, Model, Vertex,
    glam::{DMat4, DVec3},
    printing::{PrintIssueKind as K, PrintRepairOptions, PrintSelection},
};
fn cube() -> Model {
    let mut m = Model::new();
    common::box_at(m.root_mesh_mut(), DVec3::ZERO, DVec3::splat(20.));
    m
}
fn group(m: &mut Model) -> InstanceId {
    let es: Vec<_> = m
        .root_mesh()
        .faces
        .keys()
        .map(EntityRef::Face)
        .chain(m.root_mesh().edges.keys().map(EntityRef::Edge))
        .collect();
    m.make_group(&[], &es).unwrap()
}
fn top(m: &Model) -> varstotas_kernel::FaceId {
    m.root_mesh()
        .faces
        .iter()
        .find(|(_, f)| f.normal.z > 0.99)
        .unwrap()
        .0
}
#[test]
fn closed_cube_check_is_read_only_and_uses_world_size() {
    let mut m = cube();
    let i = group(&mut m);
    m.root_mesh_mut().instances[i].transform =
        DMat4::from_translation(DVec3::splat(1e7)) * DMat4::from_scale(DVec3::new(-2., 3., 4.));
    let before = varstotas_kernel::io::to_json(&m).unwrap();
    let r = m.print_check(None, 1e-5);
    assert_eq!((r.faces.len(), r.triangles, r.shells.len()), (6, 12, 1));
    assert!(!r.has_surface_errors());
    assert!(r.shells[0].closed);
    assert!((r.shells[0].volume_mm3 - 8000. * 24.).abs() < 1e-3);
    assert_eq!(
        r.bounds.unwrap().max - r.bounds.unwrap().min,
        DVec3::new(40., 60., 80.)
    );
    assert_eq!(before, varstotas_kernel::io::to_json(&m).unwrap());
    let result = m.repair_for_printing(None, &Default::default()).unwrap();
    assert!(!result.changed());
    assert_eq!(before, varstotas_kernel::io::to_json(&m).unwrap());
}
#[test]
fn hole_repair_is_opt_in_and_keeps_flat_sheets_open() {
    let mut m = cube();
    let f = top(&m);
    m.root_mesh_mut().faces.remove(f);
    m.rebuild_caches();
    assert_eq!(m.print_check(None, 1e-5).count(K::OpenEdges), 4);
    let r = m.repair_for_printing(None, &Default::default()).unwrap();
    assert_eq!(r.filled_holes, 0);
    assert_eq!(m.print_check(None, 1e-5).count(K::OpenEdges), 4);
    let options = PrintRepairOptions {
        fill_holes: true,
        max_hole_mm: 100.,
        ..Default::default()
    };
    let r = m.repair_for_printing(None, &options).unwrap();
    assert_eq!(r.filled_holes, 1);
    assert!(!m.print_check(None, 1e-5).has_surface_errors());
    assert!((m.root_mesh().volume() - 8000.).abs() < 1e-6);
    let mut sheet = Model::new();
    sheet
        .root_mesh_mut()
        .add_face_exact(&[DVec3::ZERO, DVec3::X, DVec3::Y])
        .unwrap();
    let r = sheet.repair_for_printing(None, &options).unwrap();
    assert_eq!(r.filled_holes, 0);
    assert_eq!(sheet.root_mesh().faces.len(), 1);
}
#[test]
fn winding_and_inverted_shells_are_repaired() {
    for all in [false, true] {
        let mut m = cube();
        let faces: Vec<_> = m.root_mesh().faces.keys().collect();
        m.root_mesh_mut()
            .reverse_faces(if all { &faces } else { &faces[..1] });
        let r = m.print_check(None, 1e-5);
        assert!(r.has_surface_errors());
        if all {
            assert_eq!(r.count(K::InvertedShells), 1);
        } else {
            assert_eq!(r.count(K::InconsistentWinding), 4);
            assert!(r.shells[0].closed);assert!(!r.shells[0].consistent);
        }
        let r = m.repair_for_printing(None, &Default::default()).unwrap();
        assert_eq!(r.reversed_faces,if all {6}else{1});
        assert!(!m.print_check(None, 1e-5).has_surface_errors());
        assert!(m.root_mesh().volume() > 0.);
    }
}
#[test]
fn nested_cavity_orientation_works_across_groups() {
    let mut m = cube();
    let outer = group(&mut m);
    common::box_at(m.root_mesh_mut(), DVec3::splat(5.), DVec3::splat(15.));
    let inner = group(&mut m);
    let report = m.print_check(None, 1e-5);
    assert_eq!(report.shells.len(), 2);
    assert_eq!(report.count(K::InvertedShells), 1);
    m.repair_for_printing(None, &Default::default()).unwrap();
    assert!(!m.print_check(None, 1e-5).has_surface_errors());
    let a = m.root_mesh().instances[outer].def;
    let b = m.root_mesh().instances[inner].def;
    assert!(m.defs[a].mesh.volume() > 0.);
    assert!(m.defs[b].mesh.volume() < 0.);
    let before = varstotas_kernel::io::to_json(&m).unwrap();
    assert!(
        !m.repair_for_printing(None, &Default::default())
            .unwrap()
            .changed()
    );
    assert_eq!(before, varstotas_kernel::io::to_json(&m).unwrap());
}
#[test]
fn selected_repair_does_not_edit_other_copies_or_locks() {
    let mut m = cube();
    let f = top(&m);
    m.root_mesh_mut().reverse_face(f);
    let i = group(&mut m);
    let mut other = m.root_mesh().instances[i].clone();
    other.transform = DMat4::from_translation(DVec3::X * 50.);
    let j = m.root_mesh_mut().instances.insert(other);
    let old = m.root_mesh().instances[j].def;
    let original = serde_json::to_string(&m.defs[old]).unwrap();
    let selected = PrintSelection {
        path: vec![],
        entities: vec![EntityRef::Instance(i)],
    };
    let result = m
        .repair_for_printing(Some(&selected), &Default::default())
        .unwrap();
    assert!(result.changed());
    assert_ne!(m.root_mesh().instances[i].def, old);
    assert_eq!(serde_json::to_string(&m.defs[old]).unwrap(), original);
    assert!(!m.print_check(Some(&selected), 1e-5).has_surface_errors());
    m.root_mesh_mut().instances[j].locked = true;
    let before = varstotas_kernel::io::to_json(&m).unwrap();
    let selected = PrintSelection {
        path: vec![],
        entities: vec![EntityRef::Instance(j)],
    };
    let result = m
        .repair_for_printing(Some(&selected), &Default::default())
        .unwrap();
    assert!(!result.changed());
    assert!(result.skipped_objects > 0);
    assert_eq!(before, varstotas_kernel::io::to_json(&m).unwrap());
}
#[test]
fn duplicates_degenerates_and_nonmanifold_edges_are_reported() {
    let mut m = Model::new();
    let mesh = m.root_mesh_mut();
    let vs = [DVec3::ZERO, DVec3::X, DVec3::Y, DVec3::Y * 2.]
        .map(|pos| mesh.vertices.insert(Vertex { pos }));
    for ids in [[0, 1, 2], [0, 1, 2], [0, 1, 3], [0, 0, 1]] {
        let vertices = ids.map(|i| vs[i]);
        for i in 0..3 {
            mesh.edges
                .insert(Edge::new(vertices[i], vertices[(i + 1) % 3]));
        }
        let mut f = Face::template();
        f.outer = vertices.to_vec();
        f.normal = DVec3::Z;
        mesh.faces.insert(f);
    }
    m.rebuild_caches();
    let r = m.print_check(None, 1e-5);
    assert_eq!(r.count(K::DuplicateTriangles), 1);
    assert_eq!(r.count(K::DegenerateTriangles), 1);
    assert!(r.count(K::NonManifoldEdges) > 0);
    let result = m.repair_for_printing(None, &Default::default()).unwrap();
    assert!(result.removed_faces >= 2);
    varstotas_kernel::io::validate_model(&m).unwrap();
}
#[test]
fn hidden_surfaces_and_partial_face_repairs_are_preserved() {
    let mut m = cube();
    let f = top(&m);
    m.root_mesh_mut().faces[f].hidden = true;
    let before = varstotas_kernel::io::to_json(&m).unwrap();
    let r = m.print_check(None, 1e-5);
    assert_eq!(r.faces.len(), 5);
    assert_eq!(r.count(K::OpenEdges), 4);
    let s = PrintSelection {
        path: vec![],
        entities: vec![EntityRef::Face(f)],
    };
    assert_eq!(m.print_check(Some(&s), 1e-5).faces.len(), 0);
    assert!(
        m.repair_for_printing(Some(&s), &Default::default())
            .is_err()
    );
    assert_eq!(before, varstotas_kernel::io::to_json(&m).unwrap());
}

#[test]
fn planar_hole_limits_and_nested_boundaries_are_conservative() {
    let mut m = cube();
    let f = top(&m);
    m.root_mesh_mut().faces.remove(f);
    m.rebuild_caches();
    let r = m
        .repair_for_printing(
            None,
            &PrintRepairOptions {
                fill_holes: true,
                max_hole_mm: 1.,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(r.filled_holes, 0);
    common::box_at(
        m.root_mesh_mut(),
        DVec3::new(5., 5., 5.),
        DVec3::new(15., 15., 20.),
    );
    let inner = m
        .root_mesh()
        .faces
        .iter()
        .find(|(_, f)| f.normal.z > 0.99)
        .unwrap()
        .0;
    m.root_mesh_mut().faces.remove(inner);
    m.rebuild_caches();
    let r = m
        .repair_for_printing(
            None,
            &PrintRepairOptions {
                fill_holes: true,
                max_hole_mm: 100.,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        r.filled_holes, 0,
        "nested contours must not be sealed as overlapping disks"
    );
}
#[test]
fn welding_uses_world_tolerance_and_retains_corner_shading() {
    let mut m = Model::new();
    let mesh = m.root_mesh_mut();
    let n = [
        DVec3::new(0.1, 0., 1.).normalize(),
        DVec3::new(0., 0.1, 1.).normalize(),
        DVec3::Z,
    ];
    for points in [
        [DVec3::ZERO, DVec3::X, DVec3::Y],
        [
            DVec3::new(0.00002, 0., 0.),
            DVec3::X,
            DVec3::new(0., -1., 0.),
        ],
    ] {
        let ids = points.map(|pos| mesh.vertices.insert(Vertex { pos }));
        for i in 0..3 {
            mesh.edges.insert(Edge::new(ids[i], ids[(i + 1) % 3]));
        }
        let mut face = Face::template();
        face.outer = ids.to_vec();
        face.normal = DVec3::Z;
        face.normals = n.to_vec();
        mesh.faces.insert(face);
    }
    m.rebuild_caches();
    let i = group(&mut m);
    m.root_mesh_mut().instances[i].transform = DMat4::from_scale(DVec3::new(10., 1., 1.));
    let before = m.clone();
    let options = PrintRepairOptions {
        tolerance_mm: 0.0001,
        orient_faces: false,
        orient_shells: false,
        ..Default::default()
    };
    let summary = m.repair_for_printing(None, &options).unwrap();
    assert_eq!(
        summary.welded_vertices, 1,
        "only the exactly matching seam vertex is within the world tolerance"
    );
    let d = m.root_mesh().instances[i].def;
    assert_eq!(m.defs[d].mesh.vertices.len(), 5);
    assert!(m.defs[d].mesh.faces.values().all(|f| f.normals == n));
    let mut m = before;
    let summary = m
        .repair_for_printing(
            None,
            &PrintRepairOptions {
                tolerance_mm: 0.0003,
                ..options
            },
        )
        .unwrap();
    assert_eq!(summary.welded_vertices, 2);
    varstotas_kernel::io::validate_model(&m).unwrap();
}

#[test]
fn procedural_ancestors_are_not_modified_through_plain_mesh_children() {
 let mut m=cube();let faces:Vec<_>=m.root_mesh().faces.keys().collect();m.root_mesh_mut().reverse_faces(&faces);let child=group(&mut m);
 let parent=m.make_group(&[],&[EntityRef::Instance(child)]).unwrap();let d=m.root_mesh().instances[parent].def;
 m.defs[d].generated=Some(varstotas_kernel::generator::Generated{generator:"Procedural".into(),values:Default::default()});
 let before=varstotas_kernel::io::to_json(&m).unwrap();let result=m.repair_for_printing(None,&Default::default()).unwrap();
 assert!(!result.changed());assert!(result.skipped_objects>0);assert_eq!(before,varstotas_kernel::io::to_json(&m).unwrap());
}

#[test]
fn overlapping_parts_are_not_mistaken_for_nested_cavities() {
 let mut m=cube();group(&mut m);common::box_at(m.root_mesh_mut(),DVec3::new(10.,0.,0.),DVec3::new(30.,20.,20.));group(&mut m);
 let before=varstotas_kernel::io::to_json(&m).unwrap();assert_eq!(m.print_check(None,1e-5).count(K::InvertedShells),0);
 assert!(!m.repair_for_printing(None,&Default::default()).unwrap().changed());assert_eq!(before,varstotas_kernel::io::to_json(&m).unwrap());
}
