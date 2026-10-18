mod common;
use common::*;
use varstotas_kernel::glam::DMat4;
use varstotas_kernel::*;

/// A component "Box" placed twice (a red face), a component "Spare" placed nowhere (its own
/// blue face, and nested group), an unused green material.
fn model() -> (Model, DefId, DefId) {
    let mut m = Model::new();
    let red = m.add_material("Red", [1., 0., 0., 1.]);
    make_box(m.root_mesh_mut(), 100., 50., 30.);
    let f = m.root_mesh().faces.keys().next().unwrap();
    m.root_mesh_mut().faces[f].front = Some(red);
    let ents: Vec<EntityRef> = m.root_mesh().faces.keys().map(EntityRef::Face).chain(m.root_mesh().edges.keys().map(EntityRef::Edge)).collect();
    let a = m.make_component(&[], &ents, "Box").unwrap();
    let mut copy = m.root_mesh().instances[a].clone();
    copy.transform = DMat4::from_translation(glam::DVec3::new(500., 0., 0.));
    m.root_mesh_mut().instances.insert(copy);
    let boxdef = m.root_mesh().instances[a].def;
    // Spare: a box with a nested group, then its only copy deleted.
    let blue = m.add_material("Blue", [0., 0., 1., 1.]);
    m.add_material("Green", [0., 1., 0., 1.]);
    let f = m.root_mesh_mut().add_face(&[v(0., 0., 900.), v(10., 0., 900.), v(10., 10., 900.)]).unwrap();
    m.root_mesh_mut().faces[f].front = Some(blue);
    let g = m.make_group(&[], &[EntityRef::Face(f)]).unwrap();
    let s = m.make_component(&[], &[EntityRef::Instance(g)], "Spare").unwrap();
    let spare = m.root_mesh().instances[s].def;
    m.remove_instance(&[], s);
    (m, boxdef, spare)
}

#[test]
fn statistics_scene_and_stored() {
    let (m, _, _) = model();
    let s = m.statistics();
    assert_eq!((s.scene.faces, s.scene.triangles, s.scene.edges, s.scene.components, s.scene.groups), (12, 24, 24, 2, 0));
    // Stored: Box once (6 faces, 12 edges) + the spare triangle (3 edges) in its group.
    assert_eq!((s.stored.faces, s.stored.triangles, s.stored.edges, s.stored.components, s.stored.groups), (7, 13, 15, 2, 1));
    assert_eq!((s.component_defs, s.component_defs_used, s.group_defs, s.group_defs_used), (2, 1, 1, 0));
    assert_eq!((s.materials, s.materials_used), (3, 1));
}

#[test]
fn purge_unused_definitions_and_materials() {
    let (mut m, boxdef, spare) = model();
    let p = m.purge_unused(true, true);
    assert_eq!(p, stats::Purged { components: 1, groups: 1, materials: 2 });
    assert!(m.defs.contains_key(boxdef) && !m.defs.contains_key(spare));
    assert_eq!(m.materials.values().map(|x| x.name.as_str()).collect::<Vec<_>>(), ["Red"]);
    assert!(m.purge_unused(true, true).is_empty());
    // Materials only: the definitions stay.
    let (mut m, _, spare) = model();
    assert_eq!(m.purge_unused(false, true).materials, 2);
    assert!(m.defs.contains_key(spare));
}

#[test]
fn delete_definition_removes_all_copies() {
    let (mut m, boxdef, _) = model();
    assert_eq!(m.delete_definition(boxdef).unwrap(), 2);
    assert!(!m.defs.contains_key(boxdef));
    assert_eq!(m.statistics().scene.faces, 0);
    assert!(m.delete_definition(m.root).is_err());
    assert!(m.delete_definition(boxdef).is_err());
}
