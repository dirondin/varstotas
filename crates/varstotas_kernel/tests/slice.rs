//! Slice.

use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::{EntityRef, InstanceId, Model, Plane};

/// A loose cube of size `s` with its min corner at `at` in the root.
fn loose_cube(m: &mut Model, at: DVec3, s: f64) {
    let mesh = m.root_mesh_mut();
    let z = at.z + s;
    let f = mesh
        .add_face(&[at.with_z(z), at.with_z(z) + DVec3::X * s, at.with_z(z) + DVec3::new(s, s, 0.0), at.with_z(z) + DVec3::Y * s])
        .unwrap();
    let n = mesh.faces[f].normal;
    mesh.push_pull(f, if n.z > 0.0 { -s } else { s }, false).unwrap();
}

fn all(m: &Model) -> Vec<EntityRef> {
    let mesh = m.root_mesh();
    mesh.faces.keys().map(EntityRef::Face).chain(mesh.edges.keys().map(EntityRef::Edge)).collect()
}

fn cube_group(m: &mut Model, at: DVec3, s: f64) -> InstanceId {
    loose_cube(m, at, s);
    let ents = all(m);
    m.make_group(&[], &ents).unwrap()
}

fn solid_volume(m: &Model, i: InstanceId) -> f64 {
    let inst = &m.root_mesh().instances[i];
    let mesh = &m.defs[inst.def].mesh;
    assert!(mesh.is_solid(), "part is a solid");
    mesh.volume() * inst.transform.determinant()
}

fn mid(z: f64) -> Plane {
    Plane::new(DVec3::new(0.0, 0.0, z), DVec3::Z)
}

#[test]
fn cut_loose_cube_adds_a_ring_of_edges() {
    let mut m = Model::new();
    loose_cube(&mut m, DVec3::ZERO, 1000.0);
    let r = m.slice(&[], &[], mid(400.0), false).unwrap();
    assert_eq!(r.edges, 4);
    assert!(r.parts.is_empty());
    let mesh = m.root_mesh();
    assert_eq!(mesh.faces.len(), 10, "four sides split in two");
    assert_eq!(mesh.edges.len(), 20, "and four vertical edges split");
}

#[test]
fn split_loose_cube_gives_two_solid_groups() {
    let mut m = Model::new();
    loose_cube(&mut m, DVec3::ZERO, 1000.0);
    let r = m.slice(&[], &[], mid(400.0), true).unwrap();
    assert_eq!(r.parts.len(), 2);
    let root = m.root_mesh();
    assert!(root.faces.is_empty() && root.edges.is_empty(), "loose geometry went into the groups");
    let (a, b) = (solid_volume(&m, r.parts[0]), solid_volume(&m, r.parts[1]));
    assert!((a - 600.0 * 1e6).abs() < 1.0, "upper part kept: {a}");
    assert!((b - 400.0 * 1e6).abs() < 1.0, "lower part new: {b}");
    for p in &r.parts {
        let mesh = &m.defs[root.instances[*p].def].mesh;
        assert_eq!(mesh.faces.len(), 6);
        assert!(m.defs[root.instances[*p].def].is_group);
    }
}

#[test]
fn split_selected_group_only() {
    let mut m = Model::new();
    let a = cube_group(&mut m, DVec3::ZERO, 1000.0);
    let b = cube_group(&mut m, DVec3::new(2000.0, 0.0, 0.0), 1000.0);
    // A slanted plane through the first cube.
    let plane = Plane::new(DVec3::new(500.0, 500.0, 500.0), DVec3::new(1.0, 0.0, 1.0));
    let r = m.slice(&[], &[EntityRef::Instance(a)], plane, true).unwrap();
    assert_eq!(r.parts.len(), 2);
    let total = solid_volume(&m, r.parts[0]) + solid_volume(&m, r.parts[1]);
    assert!((total - 1e9).abs() < 10.0, "parts add up: {total}");
    assert!((solid_volume(&m, r.parts[0]) - 0.5e9).abs() < 10.0);
    assert_eq!(m.root_mesh().instances.len(), 3);
    assert_eq!(m.defs[m.root_mesh().instances[b].def].mesh.faces.len(), 6, "unselected group untouched");
}

#[test]
fn moved_group_is_cut_in_its_own_coordinates() {
    let mut m = Model::new();
    let a = cube_group(&mut m, DVec3::ZERO, 1000.0);
    m.root_mesh_mut().instances[a].transform = DMat4::from_translation(DVec3::new(0.0, 0.0, 5000.0));
    // Misses the group where it was drawn, crosses it where it is.
    assert_eq!(m.slice(&[], &[], mid(500.0), true).unwrap(), Default::default());
    let r = m.slice(&[], &[], mid(5250.0), true).unwrap();
    assert!((solid_volume(&m, r.parts[0]) - 750.0 * 1e6).abs() < 1.0);
    assert!((solid_volume(&m, r.parts[1]) - 250.0 * 1e6).abs() < 1.0);
}

#[test]
fn cutting_one_copy_of_a_component_leaves_the_others() {
    let mut m = Model::new();
    loose_cube(&mut m, DVec3::ZERO, 1000.0);
    let ents = all(&m);
    let a = m.make_component(&[], &ents, "Block").unwrap();
    let mut copy = m.root_mesh().instances[a].clone();
    copy.transform = DMat4::from_translation(DVec3::new(3000.0, 0.0, 0.0));
    let b = m.root_mesh_mut().instances.insert(copy);
    let r = m.slice(&[], &[EntityRef::Instance(a)], mid(500.0), false).unwrap();
    assert_eq!(r.edges, 4);
    let (da, db) = (m.root_mesh().instances[a].def, m.root_mesh().instances[b].def);
    assert_ne!(da, db, "the cut copy was made unique");
    assert_eq!(m.defs[da].mesh.faces.len(), 10);
    assert_eq!(m.defs[db].mesh.faces.len(), 6);
    assert_eq!(m.defs[db].name, "Block");
}

#[test]
fn nested_groups_are_split_too() {
    let mut m = Model::new();
    let inner = cube_group(&mut m, DVec3::ZERO, 1000.0);
    let outer = m.make_group(&[], &[EntityRef::Instance(inner)]).unwrap();
    let r = m.slice(&[], &[], mid(300.0), true).unwrap();
    assert_eq!(r.parts, vec![outer, r.parts[1]]);
    let root = m.root_mesh();
    let vol = |p: InstanceId| {
        let od = root.instances[p].def;
        let insts: Vec<_> = m.defs[od].mesh.instances.values().collect();
        assert_eq!(insts.len(), 1, "each part holds one part of the inner group");
        let mesh = &m.defs[insts[0].def].mesh;
        assert!(mesh.is_solid());
        mesh.volume()
    };
    assert!((vol(r.parts[0]) - 700.0 * 1e6).abs() < 1.0);
    assert!((vol(r.parts[1]) - 300.0 * 1e6).abs() < 1.0);
}

#[test]
fn plane_on_a_face_changes_nothing() {
    let mut m = Model::new();
    let a = cube_group(&mut m, DVec3::ZERO, 1000.0);
    let r = m.slice(&[], &[EntityRef::Instance(a)], mid(1000.0), true).unwrap();
    assert_eq!(r, Default::default());
    assert_eq!(m.root_mesh().instances.len(), 1);
}

#[test]
fn tube_split_keeps_its_hole() {
    // A square frame (face with a hole) pushed up: the caps are rings too.
    let mut m = Model::new();
    let mesh = m.root_mesh_mut();
    let sq = |c: f64, s: f64| [DVec3::new(c, c, 0.0), DVec3::new(c + s, c, 0.0), DVec3::new(c + s, c + s, 0.0), DVec3::new(c, c + s, 0.0)];
    mesh.add_face(&sq(0.0, 1000.0)).unwrap();
    mesh.add_polyline(&sq(250.0, 500.0), true, false);
    let ring = mesh.faces.keys().find(|f| !mesh.faces[*f].holes.is_empty()).unwrap();
    let hole: Vec<_> = mesh.faces.keys().filter(|f| *f != ring).collect();
    mesh.erase(&hole.into_iter().map(EntityRef::Face).collect::<Vec<_>>());
    let n = mesh.faces[ring].normal;
    mesh.push_pull(ring, if n.z > 0.0 { 1000.0 } else { -1000.0 }, false).unwrap();
    let r = m.slice(&[], &[], mid(600.0), true).unwrap();
    let root = m.root_mesh();
    let caps = |p: InstanceId| m.defs[root.instances[p].def].mesh.faces.values().filter(|f| !f.holes.is_empty()).count();
    assert_eq!(caps(r.parts[0]), 2, "top ring and the cut ring");
    assert_eq!(caps(r.parts[1]), 2);
    let v = solid_volume(&m, r.parts[0]) + solid_volume(&m, r.parts[1]);
    assert!((v - 0.75e9).abs() < 10.0, "{v}");
}

