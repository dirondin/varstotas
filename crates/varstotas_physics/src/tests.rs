use super::*;
use std::collections::HashSet;

/// A box `size` with its corner at `at`, as a group of the root (or loose when `group` is false).
fn add_box(m: &mut Model, at: DVec3, size: DVec3, group: bool) -> Option<InstanceId> {
    let before: HashSet<EntityRef> = all(m).into_iter().collect();
    let mesh = m.root_mesh_mut();
    let pts = [at, at + DVec3::new(size.x, 0.0, 0.0), at + DVec3::new(size.x, size.y, 0.0), at + DVec3::new(0.0, size.y, 0.0)];
    let f = mesh.add_face(&pts).unwrap();
    let up = mesh.faces[f].normal.z >= 0.0;
    mesh.push_pull(f, if up { size.z } else { -size.z }, false).unwrap();
    if !group {
        return None;
    }
    let new: Vec<_> = all(m).into_iter().filter(|e| !before.contains(e)).collect();
    Some(m.make_group(&[], &new).unwrap())
}

fn all(m: &Model) -> Vec<EntityRef> {
    let mesh = m.root_mesh();
    mesh.faces.keys().map(EntityRef::Face).chain(mesh.edges.keys().map(EntityRef::Edge)).collect()
}

fn run(m: &Model, bodies: &[InstanceId], s: DropSettings) -> DropFrame {
    let mut sim = DropSim::new(m, &[], bodies, s).unwrap();
    loop {
        let f = sim.step(60);
        if f.done {
            return f;
        }
    }
}

fn bottom(m: &Model, id: InstanceId) -> varstotas_kernel::Aabb {
    m.instance_world_bounds(&[], id).unwrap()
}

#[test]
fn a_box_falls_onto_another_and_rests() {
    let mut m = Model::new();
    let base = add_box(&mut m, DVec3::ZERO, DVec3::new(100.0, 100.0, 50.0), true).unwrap();
    let cube = add_box(&mut m, DVec3::new(30.0, 30.0, 200.0), DVec3::splat(20.0), true).unwrap();
    let f = run(&m, &[cube], DropSettings::default());
    assert!(f.done && f.time < 9.0, "came to rest in {} s", f.time);
    assert_eq!(f.fell, vec![false]);
    set_transforms(&mut m, &[], &[cube], &f.transforms);
    let b = bottom(&m, cube);
    assert!((b.min.z - 50.0).abs() < 1.0, "rests on the base: {}", b.min.z);
    assert!((b.max.z - b.min.z - 20.0).abs() < 0.5, "upright: {:?}", b);
    // the static base did not move
    assert_eq!(bottom(&m, base).min.z, 0.0);
}

#[test]
fn bodies_fall_onto_the_floor_and_static_loose_geometry() {
    let mut m = Model::new();
    // loose geometry: a slab under one of the bodies
    add_box(&mut m, DVec3::new(200.0, 0.0, 0.0), DVec3::new(100.0, 100.0, 10.0), false);
    let a = add_box(&mut m, DVec3::new(0.0, 0.0, 80.0), DVec3::splat(30.0), true).unwrap();
    let b = add_box(&mut m, DVec3::new(230.0, 30.0, 80.0), DVec3::splat(30.0), true).unwrap();
    let f = run(&m, &[a, b], DropSettings::default());
    set_transforms(&mut m, &[], &[a, b], &f.transforms);
    assert!(bottom(&m, a).min.z.abs() < 1.0, "on the floor: {}", bottom(&m, a).min.z);
    assert!((bottom(&m, b).min.z - 10.0).abs() < 1.0, "on the slab: {}", bottom(&m, b).min.z);
}

#[test]
fn without_a_floor_a_body_over_nothing_falls_into_the_void_and_stays() {
    let mut m = Model::new();
    add_box(&mut m, DVec3::ZERO, DVec3::new(50.0, 50.0, 10.0), true);
    let far = add_box(&mut m, DVec3::new(500.0, 0.0, 100.0), DVec3::splat(10.0), true).unwrap();
    let s = DropSettings { floor: false, ..DropSettings::default() };
    let f = run(&m, &[far], s);
    assert!(f.done && f.time < 9.0, "{}", f.time);
    assert_eq!(f.fell, vec![true]);
    let start = m.root_mesh().instances[far].transform;
    assert_eq!(f.transforms[0], start);
}

#[test]
fn a_flat_body_has_a_shape_and_pieces_work() {
    let mut m = Model::new();
    let pts = [DVec3::new(0.0, 0.0, 50.0), DVec3::new(40.0, 0.0, 50.0), DVec3::new(40.0, 40.0, 50.0), DVec3::new(0.0, 40.0, 50.0)];
    let f = m.root_mesh_mut().add_face(&pts).unwrap();
    let mesh = m.root_mesh();
    let mut ents = vec![EntityRef::Face(f)];
    ents.extend(mesh.face_edges(f).into_iter().map(EntityRef::Edge));
    let sheet = m.make_group(&[], &ents).unwrap();
    let cube = add_box(&mut m, DVec3::new(100.0, 0.0, 60.0), DVec3::splat(20.0), true).unwrap();
    let s = DropSettings { shape: DropShape::Pieces, ..DropSettings::default() };
    let fr = run(&m, &[sheet, cube], s);
    set_transforms(&mut m, &[], &[sheet, cube], &fr.transforms);
    assert!(bottom(&m, sheet).min.z.abs() < 1.5, "{}", bottom(&m, sheet).min.z);
    assert!(bottom(&m, cube).min.z.abs() < 1.0, "{}", bottom(&m, cube).min.z);
}

#[test]
fn bodies_are_groups_that_are_not_locked() {
    let mut m = Model::new();
    let a = add_box(&mut m, DVec3::ZERO, DVec3::splat(10.0), true).unwrap();
    let b = add_box(&mut m, DVec3::new(50.0, 0.0, 0.0), DVec3::splat(10.0), true).unwrap();
    m.root_mesh_mut().instances[b].locked = true;
    let face = m.root_mesh().faces.keys().next();
    let mut sel = vec![EntityRef::Instance(a), EntityRef::Instance(b)];
    sel.extend(face.map(EntityRef::Face));
    assert_eq!(drop_bodies(&m, &[], &sel), vec![a]);
    assert_eq!(DropSim::new(&m, &[], &[], DropSettings::default()).err(), Some(DropError::NoBodies));
}
