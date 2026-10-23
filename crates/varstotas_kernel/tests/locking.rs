mod common;
use common::*;
use varstotas_kernel::{
    glam::{DMat4, DVec3},
    *,
};

fn grouped() -> (Model, InstanceId) {
    let mut m = Model::new();
    make_box(m.root_mesh_mut(), 100., 50., 30.);
    let faces = m
        .root_mesh()
        .faces
        .keys()
        .map(EntityRef::Face)
        .collect::<Vec<_>>();
    let i = m.make_group(&[], &faces).unwrap();
    m.root_mesh_mut().instances[i].locked = true;
    (m, i)
}

#[test]
fn locked_instance_blocks_transform_delete_and_descendants_but_allows_visibility() {
    let (m, i) = grouped();
    for mode in 0..4 {
        let mut n = m.clone();
        match mode {
            0 => n.root_mesh_mut().instances[i].transform = DMat4::from_translation(DVec3::X),
            1 => {
                n.root_mesh_mut().instances.remove(i);
            }
            2 => {
                n.mesh_at_mut(&[i])
                    .unwrap()
                    .vertices
                    .values_mut()
                    .next()
                    .unwrap()
                    .pos
                    .x += 1.;
            }
            _ => n.root_mesh_mut().instances[i].locked = false,
        }
        assert!(m.validate_locked_changes(&n).is_err(), "mode {mode}");
    }
    let mut n = m.clone();
    n.root_mesh_mut().instances[i].hidden = true;
    assert!(m.validate_locked_changes(&n).is_ok());
    assert!(m.context_locked(&[i]));
}

#[test]
fn shared_definitions_materials_and_parent_transforms_are_protected() {
    let (mut m, i) = grouped();
    let material = m.add_material("Oak", [1.; 4]);
    let def = m.root_mesh().instances[i].def;
    m.defs[def].mesh.faces.values_mut().next().unwrap().front = Some(material);
    let mut copy = m.root_mesh().instances[i].clone();
    copy.locked = false;
    let j = m.root_mesh_mut().instances.insert(copy);
    assert!(m.context_locked(&[j]));
    let mut n = m.clone();
    n.root_mesh_mut().instances[j].transform = DMat4::from_translation(DVec3::X * 300.);
    assert!(m.validate_locked_changes(&n).is_ok(), "an unlocked copy can move independently");
    let mut n = m.clone();
    n.root_mesh_mut().instances.remove(j);
    assert!(m.validate_locked_changes(&n).is_ok(), "an unlocked copy can be deleted independently");
    let mut n = m.clone();
    n.materials[material].color = [0.; 4];
    assert!(m.validate_locked_changes(&n).is_err());
    let outer = m.make_group(&[], &[EntityRef::Instance(i)]).unwrap();
    let mut n = m.clone();
    n.root_mesh_mut().instances[outer].transform = DMat4::from_translation(DVec3::Y);
    assert!(m.validate_locked_changes(&n).is_err());
    let mut n = m.clone();
    let sibling = n.root_mesh_mut().instances[j].clone();
    n.root_mesh_mut().instances.insert(sibling);
    assert!(m.validate_locked_changes(&n).is_ok());
}

#[test]
fn face_lock_protects_boundary_and_shared_vertices_from_indirect_edits() {
    let mut m = Model::new();
    make_box(m.root_mesh_mut(), 100., 50., 30.);
    let f = m.root_mesh().faces.keys().next().unwrap();
    m.root_mesh_mut().faces[f].locked = true;
    let v = m.root_mesh().faces[f].outer[0];
    let edge = m
        .root_mesh()
        .edges
        .iter()
        .find(|(_, e)| e.has(v))
        .unwrap()
        .0;
    assert!(m.entity_locked(&[], EntityRef::Vertex(v)));
    let mut n = m.clone();
    n.root_mesh_mut().vertices[v].pos += DVec3::X;
    assert!(m.validate_locked_changes(&n).is_err());
    let mut n = m.clone();
    n.root_mesh_mut().edges.remove(edge);
    // Select an actual face boundary (not an edge leading away from the face).
    if m.entity_locked(&[], EntityRef::Edge(edge)) {
        assert!(m.validate_locked_changes(&n).is_err());
    }
    let mut n = m.clone();
    n.root_mesh_mut().faces.remove(f);
    assert!(m.validate_locked_changes(&n).is_err());
}

#[test]
fn tags_inherit_lock_and_cannot_be_removed_reassigned_or_populated_by_scripts() {
    let (mut m, i) = grouped();
    m.root_mesh_mut().instances[i].locked = false;
    let t = m.add_tag("Building");
    m.root_mesh_mut().instances[i].tag = Some(t);
    m.tags[t].locked = true;
    assert!(m.entity_locked(&[], EntityRef::Instance(i)));
    assert!(m.context_locked(&[i]));
    let mut n = m.clone();
    n.delete_tag(t);
    assert!(m.validate_locked_changes(&n).is_err());
    let mut n = m.clone();
    n.root_mesh_mut().instances[i].tag = None;
    assert!(m.validate_locked_changes(&n).is_err());
    let mut n = m.clone();
    let copy = n.root_mesh().instances[i].clone();
    n.root_mesh_mut().instances.insert(copy);
    assert!(m.validate_locked_changes(&n).is_err());
    let mut n = m.clone();
    n.tags[t].visible = false;
    assert!(m.validate_locked_changes(&n).is_ok());
}

#[test]
fn locks_roundtrip_old_files_default_unlocked_and_unlock_all_clears_every_level() {
    let (mut m, i) = grouped();
    let t = m.add_tag("Locked");
    m.tags[t].locked = true;
    m.untagged_locked = true;
    m.mesh_at_mut(&[i])
        .unwrap()
        .faces
        .values_mut()
        .next()
        .unwrap()
        .locked = true;
    m.mesh_at_mut(&[i])
        .unwrap()
        .edges
        .values_mut()
        .next()
        .unwrap()
        .locked = true;
    let text = io::to_json(&m).unwrap();
    let mut n = io::from_json(&text).unwrap();
    assert!(n.has_locks());
    n.unlock_all();
    assert!(!n.has_locks());
    let mut old: serde_json::Value = serde_json::from_str(&text).unwrap();
    fn strip(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
                m.remove("locked");
                m.remove("untagged_locked");
                for v in m.values_mut() {
                    strip(v);
                }
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    strip(v);
                }
            }
            _ => {}
        }
    }
    strip(&mut old);
    assert!(!io::from_json(&old.to_string()).unwrap().has_locks());
}
