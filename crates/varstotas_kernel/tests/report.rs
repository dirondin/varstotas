//! Reports: parts list with cut-list sizes, material areas, CSV.

use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::report::{model_report, report_csv};
use varstotas_kernel::{EntityRef, Instance, LengthUnit, Model};

/// A board component 600 × 18 × 400 (x, y, z) with two instances; the second rotated 90°
/// about Z and moved. Plus one painted cube group.
fn model() -> Model {
    let mut m = Model::default();
    let mesh = m.root_mesh_mut();
    let f = mesh
        .add_face(&[DVec3::ZERO, DVec3::new(600.0, 0.0, 0.0), DVec3::new(600.0, 18.0, 0.0), DVec3::new(0.0, 18.0, 0.0)])
        .unwrap();
    let n = mesh.faces[f].normal;
    mesh.push_pull(f, if n.z > 0.0 { 400.0 } else { -400.0 }, false).unwrap();
    let ents: Vec<EntityRef> = mesh.faces.keys().map(EntityRef::Face).chain(mesh.edges.keys().map(EntityRef::Edge)).collect();
    let board = m.make_component(&[], &ents, "Shelf board").unwrap();
    let def = m.root_mesh().instances[board].def;
    let xf = DMat4::from_translation(DVec3::new(2000.0, 0.0, 0.0)) * DMat4::from_rotation_z(std::f64::consts::FRAC_PI_2);
    m.root_mesh_mut().instances.insert(Instance {
        def,
        transform: xf,
        name: String::new(),
        material: None,
        tag: None,
        hidden: false,
        locked: false,
    });
    let red = m.add_material("Red", [1.0, 0.0, 0.0, 1.0]);
    let mesh = m.root_mesh_mut();
    let z = 1000.0;
    let f = mesh
        .add_face(&[DVec3::new(0.0, 500.0, z), DVec3::new(100.0, 500.0, z), DVec3::new(100.0, 600.0, z), DVec3::new(0.0, 600.0, z)])
        .unwrap();
    let n = mesh.faces[f].normal;
    mesh.push_pull(f, if n.z > 0.0 { 100.0 } else { -100.0 }, false).unwrap();
    // The boards are in the component: the root has only the cube.
    let ents: Vec<EntityRef> = mesh.faces.keys().map(EntityRef::Face).chain(mesh.edges.keys().map(EntityRef::Edge)).collect();
    let g = m.make_group(&[], &ents).unwrap();
    m.root_mesh_mut().instances[g].material = Some(red);
    m.root_mesh_mut().instances[g].name = "Cube".into();
    m
}

#[test]
fn parts_and_areas() {
    let m = model();
    let (rows, areas) = model_report(&m);
    assert_eq!(rows.len(), 2, "{rows:#?}");
    let board = rows.iter().find(|r| r.definition == "Shelf board").unwrap();
    assert_eq!(board.count, 2, "the rotated copy is the same part");
    assert!(board.component);
    assert_eq!(board.size, [600.0, 400.0, 18.0]);
    assert!((board.volume.unwrap() - 600.0 * 400.0 * 18.0).abs() < 1e-3);
    let cube = rows.iter().find(|r| r.name == "Cube").unwrap();
    assert_eq!(cube.material, "Red");
    assert_eq!(cube.size, [100.0, 100.0, 100.0]);
    assert!((cube.area - 60_000.0).abs() < 1e-6);
    let board_area = 2.0 * (600.0 * 18.0 + 600.0 * 400.0 + 18.0 * 400.0);
    let red = areas.iter().find(|(n, _)| n == "Red").unwrap().1;
    let def = areas.iter().find(|(n, _)| n == "Default").unwrap().1;
    assert!((red - 60_000.0).abs() < 1e-6, "inherited from the group: {areas:?}");
    assert!((def - 2.0 * board_area).abs() < 1e-6, "{areas:?}");
}

#[test]
fn csv_in_model_units() {
    let mut m = model();
    m.unit = LengthUnit::Centimeter;
    let csv = report_csv(&m);
    let mut lines = csv.lines();
    assert!(lines.next().unwrap().starts_with("Name,Definition,Type,Level,Count,Length (cm)"));
    assert!(csv.contains("Shelf board,Shelf board,Component,1,2,60,40,1.8,"), "{csv}");
    assert!(csv.contains("\nMaterial,Area (cm²)\n"), "{csv}");
    assert!(csv.contains("Red,600\n"), "{csv}");
}
