//! Solid Tools.

use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{EntityRef, InstanceId, Model, SolidOp};

/// A cube group of size `s` with its min corner at `at`.
fn cube(m: &mut Model, at: DVec3, s: f64) -> InstanceId {
    let mesh = m.root_mesh_mut();
    let z = at.z + s;
    let f = mesh
        .add_face(&[at.with_z(z), at.with_z(z) + DVec3::X * s, at.with_z(z) + DVec3::new(s, s, 0.0), at.with_z(z) + DVec3::Y * s])
        .unwrap();
    let n = mesh.faces[f].normal;
    mesh.push_pull(f, if n.z > 0.0 { -s } else { s }, false).unwrap();
    let ents: Vec<EntityRef> = mesh
        .faces
        .keys()
        .map(EntityRef::Face)
        .chain(mesh.edges.keys().map(EntityRef::Edge))
        .collect();
    m.make_group(&[], &ents).unwrap()
}

fn volume(m: &Model, i: InstanceId) -> f64 {
    let inst = &m.root_mesh().instances[i];
    let mesh = &m.defs[inst.def].mesh;
    assert!(mesh.is_solid(), "result is a solid");
    mesh.volume() * inst.transform.determinant()
}

fn two(m: &mut Model, off: DVec3) -> (InstanceId, InstanceId) {
    let a = cube(m, DVec3::ZERO, 1000.0);
    let b = cube(m, off, 1000.0);
    (a, b)
}

const OFF: DVec3 = DVec3::new(500.0, 500.0, 500.0);

#[test]
fn cubes_are_solids() {
    let mut m = Model::new();
    let a = cube(&mut m, DVec3::ZERO, 1000.0);
    assert!((volume(&m, a) - 1e9).abs() < 1.0);
}

#[test]
fn union_intersect_subtract() {
    let mut m = Model::new();
    let (a, b) = two(&mut m, OFF);
    let r = m.solid_op_in(&[], SolidOp::Union, a, b).unwrap();
    assert_eq!(r.len(), 1);
    assert!((volume(&m, r[0]) - (2e9 - 1.25e8)).abs() < 10.0, "{}", volume(&m, r[0]));
    assert_eq!(m.root_mesh().instances.len(), 1);

    let mut m = Model::new();
    let (a, b) = two(&mut m, OFF);
    let r = m.solid_op_in(&[], SolidOp::Intersect, a, b).unwrap();
    assert!((volume(&m, r[0]) - 1.25e8).abs() < 10.0);
    // The overlap is a 500 mm cube: 6 faces.
    assert_eq!(m.defs[m.root_mesh().instances[r[0]].def].mesh.faces.len(), 6);

    let mut m = Model::new();
    let (a, b) = two(&mut m, OFF);
    let r = m.solid_op_in(&[], SolidOp::Subtract, a, b).unwrap();
    assert!((volume(&m, r[0]) - (1e9 - 1.25e8)).abs() < 10.0);
    assert_eq!(m.root_mesh().instances.len(), 1, "both consumed");
}

#[test]
fn trim_keeps_the_first_and_split_makes_three() {
    let mut m = Model::new();
    let (a, b) = two(&mut m, OFF);
    let r = m.solid_op_in(&[], SolidOp::Trim, a, b).unwrap();
    assert_eq!(m.root_mesh().instances.len(), 2);
    assert!(m.root_mesh().instances.contains_key(a));
    assert!((volume(&m, r[0]) - (1e9 - 1.25e8)).abs() < 10.0);

    let mut m = Model::new();
    let (a, b) = two(&mut m, OFF);
    let r = m.solid_op_in(&[], SolidOp::Split, a, b).unwrap();
    assert_eq!(r.len(), 3);
    let total: f64 = r.iter().map(|i| volume(&m, *i)).sum();
    assert!((total - (2e9 - 1.25e8)).abs() < 10.0, "{total}");
}

#[test]
fn touching_cubes_union_into_one_box() {
    let mut m = Model::new();
    let (a, b) = two(&mut m, DVec3::new(1000.0, 0.0, 0.0));
    let r = m.solid_op_in(&[], SolidOp::Union, a, b).unwrap();
    assert!((volume(&m, r[0]) - 2e9).abs() < 10.0);
    // The shared wall is gone and the coplanar sides merged: a plain box.
    assert_eq!(m.defs[m.root_mesh().instances[r[0]].def].mesh.faces.len(), 6);
}

#[test]
fn disjoint_intersect_fails_and_keeps_the_model() {
    let mut m = Model::new();
    let (a, b) = two(&mut m, DVec3::new(3000.0, 0.0, 0.0));
    assert!(m.solid_op_in(&[], SolidOp::Intersect, a, b).is_err());
    assert_eq!(m.root_mesh().instances.len(), 2);
}
