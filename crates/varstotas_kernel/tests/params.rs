//! Parametric components: size by stretching / scaling, copies in a row.

use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{EntityRef, InstanceId, Model, io};

/// A board group 800 × 300 × 18 with a 50 × 50 square drawn on its top near the +X end.
fn board(m: &mut Model) -> InstanceId {
    let mesh = m.root_mesh_mut();
    let f = mesh
        .add_face(&[DVec3::ZERO, DVec3::new(800.0, 0.0, 0.0), DVec3::new(800.0, 300.0, 0.0), DVec3::new(0.0, 300.0, 0.0)])
        .unwrap();
    let n = mesh.faces[f].normal;
    mesh.push_pull(f, if n.z > 0.0 { 18.0 } else { -18.0 }, false).unwrap();
    mesh.add_face(&[
        DVec3::new(700.0, 100.0, 18.0),
        DVec3::new(750.0, 100.0, 18.0),
        DVec3::new(750.0, 150.0, 18.0),
        DVec3::new(700.0, 150.0, 18.0),
    ])
    .unwrap();
    let ents: Vec<EntityRef> = mesh.faces.keys().map(EntityRef::Face).chain(mesh.edges.keys().map(EntityRef::Edge)).collect();
    m.make_group(&[], &ents).unwrap()
}

fn size(m: &Model, i: InstanceId) -> DVec3 {
    let inst = &m.root_mesh().instances[i];
    let b = m.def_bounds(inst.def).unwrap().transformed(&inst.transform);
    b.max - b.min
}

fn close(a: DVec3, b: DVec3) -> bool {
    (a - b).length() < 1e-6
}

#[test]
fn stretch_keeps_details_near_the_ends() {
    let mut m = Model::new();
    let g = board(&mut m);
    m.make_parametric(&[], g).unwrap();
    assert!(close(size(&m, g), DVec3::new(800.0, 300.0, 18.0)));
    let mut p = m.instance_params(&[], g).unwrap().clone();
    p.size[0] = Some(1200.0);
    m.set_instance_params(&[], g, p).unwrap();
    assert!(close(size(&m, g), DVec3::new(1200.0, 300.0, 18.0)));
    // The square moved with the +X end and kept its 50 mm size.
    let p = m.instance_params(&[], g).unwrap().clone();
    let v = &m.defs[p.variant.expect("stretched variant")].mesh;
    assert!(v.is_solid());
    assert!((v.volume() - 1200.0 * 300.0 * 18.0).abs() < 1e-3);
    let xs: Vec<f64> = v.vertices.values().filter(|x| x.pos.y > 99.0 && x.pos.y < 151.0).map(|x| x.pos.x).collect();
    assert!(xs.iter().any(|x| (x - 1100.0).abs() < 1e-6) && xs.iter().any(|x| (x - 1150.0).abs() < 1e-6), "{xs:?}");
    // The base is untouched.
    let b = m.def_bounds(p.base).unwrap();
    assert!(close(b.max - b.min, DVec3::new(800.0, 300.0, 18.0)));
}

#[test]
fn copies_in_a_row_and_scaling() {
    let mut m = Model::new();
    let g = board(&mut m);
    m.make_parametric(&[], g).unwrap();
    let mut p = m.instance_params(&[], g).unwrap().clone();
    p.count = 3;
    p.axis = 1;
    p.gap = 100.0;
    m.set_instance_params(&[], g, p.clone()).unwrap();
    assert!(close(size(&m, g), DVec3::new(800.0, 3.0 * 300.0 + 2.0 * 100.0, 18.0)));
    let d = m.root_mesh().instances[g].def;
    assert_eq!(m.defs[d].mesh.instances.len(), 3);
    // Scaling: copies use the base, the variant is dropped.
    p.stretch = false;
    p.size = [Some(400.0), None, Some(36.0)];
    m.set_instance_params(&[], g, p).unwrap();
    assert!(close(size(&m, g), DVec3::new(400.0, 1100.0, 36.0)));
    let p = m.instance_params(&[], g).unwrap();
    assert!(p.variant.is_none());
    assert!(m.defs[d].mesh.instances.values().all(|i| i.def == p.base));
}

#[test]
fn shrinking_stops_before_the_parts_meet() {
    let mut m = Model::new();
    let g = board(&mut m);
    m.make_parametric(&[], g).unwrap();
    let mut p = m.instance_params(&[], g).unwrap().clone();
    // Fixed vertices are at x = 0, the moved ones start at the square (x = 700).
    p.size[0] = Some(10.0);
    m.set_instance_params(&[], g, p).unwrap();
    let s = size(&m, g);
    assert!(s.x > 100.0 && s.x < 120.0, "{s}");
    let v = m.instance_params(&[], g).unwrap().variant.unwrap();
    assert!(m.defs[v].mesh.is_solid());
}

#[test]
fn saved_copied_and_purged() {
    let mut m = Model::new();
    let g = board(&mut m);
    m.make_parametric(&[], g).unwrap();
    let mut p = m.instance_params(&[], g).unwrap().clone();
    p.size[0] = Some(1000.0);
    m.set_instance_params(&[], g, p.clone()).unwrap();
    // A copy shares the definition until it is changed; then it gets its own.
    let copy = m.root_mesh_mut().duplicate(&[EntityRef::Instance(g)], &varstotas_kernel::glam::DMat4::IDENTITY)[0];
    let EntityRef::Instance(c) = copy else { panic!() };
    p.size[0] = Some(600.0);
    m.set_instance_params(&[], c, p).unwrap();
    assert!((size(&m, g).x - 1000.0).abs() < 1e-6);
    assert!((size(&m, c).x - 600.0).abs() < 1e-6);
    assert_ne!(m.instance_params(&[], g).unwrap().variant, m.instance_params(&[], c).unwrap().variant);
    m.purge_unused_defs();
    let base = m.instance_params(&[], g).unwrap().base;
    assert!(m.defs.contains_key(base), "base kept alive");
    let back = io::from_json(&io::to_json(&m).unwrap()).unwrap();
    assert_eq!(back.instance_params(&[], g), m.instance_params(&[], g));
    assert!((size(&back, c).x - 600.0).abs() < 1e-6);
    // Deleting both parametric groups lets everything go.
    m.root_mesh_mut().instances.clear();
    m.purge_unused_defs();
    assert_eq!(m.defs.len(), 1);
}
