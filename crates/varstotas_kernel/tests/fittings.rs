//! Fittings: handles, legs and the like as objects mounted on a part, following it, drilling it.

use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::joinery::{WoodJoint, WoodJointKind as K};
use varstotas_kernel::primitives::Primitive;
use varstotas_kernel::{EntityRef, InstanceId, Model};

/// A box group `l` × `w` × `t` (x, y, z) named `name` at `at`.
fn board(m: &mut Model, name: &str, s: DVec3, at: DVec3) -> InstanceId {
    let mesh = Primitive::Box { w: s.x, d: s.y, h: s.z }.mesh().unwrap();
    let id = m.add_group_mesh(&[], mesh, "Group", DMat4::from_translation(at)).unwrap();
    m.root_mesh_mut().instances[id].name = name.into();
    id
}

fn at(m: &Model, path: &[InstanceId]) -> DVec3 {
    m.transform_at(path).w_axis.truncate()
}

fn close(a: DVec3, b: DVec3) -> bool {
    a.abs_diff_eq(b, 1e-6)
}

/// A door 600 × 800 standing on the x–z plane, its front at y = −18, a handle in its middle.
fn door_with_handle() -> (Model, InstanceId, Vec<InstanceId>) {
    let mut m = Model::new();
    let door = board(&mut m, "Door", DVec3::new(600.0, 18.0, 800.0), DVec3::new(0.0, -18.0, 0.0));
    let j = m.fitting_joint(K::Handle, vec![door]).unwrap();
    let h = m.put_fitting_joint(&j).unwrap().remove(0);
    (m, door, h)
}

#[test]
fn a_handle_follows_its_door() {
    let (mut m, door, h) = door_with_handle();
    assert!(close(at(&m, &h), DVec3::new(300.0, -18.0, 400.0)), "{}", at(&m, &h));
    let mount = m.root_mesh().instances[h[0]].mount.clone().unwrap();
    assert_eq!((mount.on.as_slice(), mount.at), (&[door][..], [0.5, 0.5]), "in the middle of the front");
    // a wider door: still in its middle
    m.root_mesh_mut().instances[door].transform = DMat4::from_translation(DVec3::new(0.0, -18.0, 0.0)) * DMat4::from_scale(DVec3::new(1.5, 1.0, 1.0));
    m.sync_joints();
    assert!(close(at(&m, &h), DVec3::new(450.0, -18.0, 400.0)), "{}", at(&m, &h));
    // moved by hand 100 up: its mount follows, the holes with it
    let holes = m.fitting_holes()[0].1.clone();
    m.root_mesh_mut().instances[h[0]].transform = DMat4::from_translation(DVec3::new(0.0, 0.0, 100.0)) * m.root_mesh().instances[h[0]].transform;
    m.sync_joints();
    assert!(close(at(&m, &h), DVec3::new(450.0, -18.0, 500.0)));
    let after = &m.fitting_holes()[0].1;
    assert!(holes.iter().zip(after).all(|(a, b)| (b.0.z - a.0.z - 100.0).abs() < 1e-6));
    // pulled off the face: back onto it
    m.root_mesh_mut().instances[h[0]].transform = DMat4::from_translation(DVec3::new(0.0, -40.0, 0.0)) * m.root_mesh().instances[h[0]].transform;
    m.sync_joints();
    assert!((at(&m, &h).y + 58.0).abs() < 1e-6, "lifted 40 off it");
    let mut mt = m.root_mesh().instances[h[0]].mount.clone().unwrap();
    mt.lift = 0.0;
    mt.seen = None;
    m.root_mesh_mut().instances[h[0]].mount = Some(mt);
    m.sync_joints();
    assert!((at(&m, &h).y + 18.0).abs() < 1e-6);
    // the door moved: the handle goes with it
    m.root_mesh_mut().instances[door].transform = DMat4::from_translation(DVec3::new(1000.0, -18.0, 0.0)) * DMat4::from_scale(DVec3::new(1.5, 1.0, 1.0));
    m.sync_joints();
    assert!(close(at(&m, &h), DVec3::new(1450.0, -18.0, 500.0)), "{}", at(&m, &h));
    // a hardware line, not a part
    let plan = varstotas_kernel::cutlist::plan(&m);
    assert_eq!(plan.parts.iter().filter(|p| !p.fitting).count(), 1);
    assert_eq!(plan.hardware.iter().map(|r| (r.name.as_str(), r.count)).collect::<Vec<_>>(), [("Handle, centres 192", 1)]);
}

#[test]
fn a_handle_goes_with_its_door_into_groups() {
    // the door and its handle grouped: the handle is mounted on the door inside the group
    let (mut m, door, h) = door_with_handle();
    let g = m.make_group(&[], &[EntityRef::Instance(door), EntityRef::Instance(h[0])]).unwrap();
    m.sync_joints();
    let gd = m.root_mesh().instances[g].def;
    let inside: Vec<_> = m.defs[gd].mesh.instances.iter().filter_map(|(id, i)| i.mount.as_ref().map(|mt| (id, mt.on.clone()))).collect();
    assert_eq!(inside.len(), 1);
    assert!(m.defs[gd].mesh.instances.contains_key(inside[0].1[0]), "its door, inside the group");
    assert_eq!(m.placed_fittings().len(), 1);
    // copied with it: each copy has its handle
    let copy = varstotas_kernel::Instance { transform: DMat4::from_translation(DVec3::new(2000.0, 0.0, 0.0)), ..m.root_mesh().instances[g].clone() };
    m.root_mesh_mut().instances.insert(copy);
    m.sync_joints();
    assert_eq!(m.placed_fittings().len(), 2);
    // exploded again: still mounted on the door
    let ents = m.explode(&[], g).unwrap();
    m.sync_joints();
    assert!(ents.len() >= 2 && m.placed_fittings().len() == 2);
    // the door alone grouped: the handle's mount goes through the group
    let (mut m, door, h) = door_with_handle();
    let g = m.make_group(&[], &[EntityRef::Instance(door)]).unwrap();
    m.sync_joints();
    let on = m.root_mesh().instances[h[0]].mount.as_ref().unwrap().on.clone();
    assert_eq!((on.len(), on[0]), (2, g));
    // the handle alone grouped: it leaves its door
    let (mut m, _, h) = door_with_handle();
    m.make_group(&[], &[EntityRef::Instance(h[0])]).unwrap();
    m.sync_joints();
    assert!(m.placed_fittings().is_empty());
}

#[test]
fn fittings_are_saved_and_old_handles_become_fittings() {
    let (m, _, h) = door_with_handle();
    let json = varstotas_kernel::io::to_json(&m).unwrap();
    let mut back = varstotas_kernel::io::from_json(&json).unwrap();
    back.rebuild_caches();
    back.sync_joints();
    assert_eq!(back.placed_fittings(), m.placed_fittings());
    assert!(close(at(&back, &h), at(&m, &h)));
    assert_eq!(back.fitting_holes(), m.fitting_holes());
    // a file whose handle and legs are joints: they become fittings where they were
    let mut old = Model::new();
    let door = board(&mut old, "Door", DVec3::new(600.0, 18.0, 800.0), DVec3::new(0.0, -18.0, 0.0));
    let table = board(&mut old, "Top", DVec3::new(800.0, 500.0, 18.0), DVec3::new(0.0, 0.0, 700.0));
    for (kind, p) in [(K::Handle, door), (K::Leg, table)] {
        let c = varstotas_kernel::joinery::joint_contact(&old, &WoodJoint { kind, a: vec![p], ..Default::default() }).unwrap();
        old.joints.push(WoodJoint::new(kind, vec![p], Vec::new(), &c));
    }
    old.rebuild_caches();
    assert!(old.joints.is_empty());
    let placed = old.placed_fittings();
    assert_eq!((placed.iter().filter(|p| p.1 == vec![door]).count(), placed.iter().filter(|p| p.1 == vec![table]).count()), (1, 4));
    let handle = &placed.iter().find(|p| p.1 == vec![door]).unwrap().0;
    assert!(close(at(&old, handle), DVec3::new(300.0, -18.0, 400.0)));
    // the legs at the corners of the underside, 50 in
    let mut legs: Vec<DVec3> = placed.iter().filter(|p| p.1 == vec![table]).map(|p| at(&old, &p.0)).collect();
    legs.sort_by(|a, b| (a.x, a.y).partial_cmp(&(b.x, b.y)).unwrap());
    assert!(close(legs[0], DVec3::new(50.0, 50.0, 700.0)) && close(legs[3], DVec3::new(750.0, 450.0, 700.0)), "{legs:?}");
    // a bigger top: the legs stay 50 from its corners
    old.root_mesh_mut().instances[table].transform = DMat4::from_translation(DVec3::new(0.0, 0.0, 700.0)) * DMat4::from_scale(DVec3::new(1.5, 1.0, 1.0));
    old.sync_joints();
    let far = old.placed_fittings().iter().filter(|p| p.1 == vec![table]).map(|p| at(&old, &p.0).x).fold(0.0, f64::max);
    assert!((far - 1150.0).abs() < 1e-6, "{far}");
}

#[test]
fn a_point_on_a_part_snaps_to_its_middle_and_edges() {
    let (m, door, _) = door_with_handle();
    let n = DVec3::new(0.0, -1.0, 0.0);
    // the door's front at y = −18, 600 wide and 800 high: near its middle, pulled onto it
    let (p, hit) = m.mount_snap(&[door], DVec3::new(296.0, -18.0, 405.0), n, 10.0).unwrap();
    assert!(close(p, DVec3::new(300.0, -18.0, 400.0)) && hit == [true, true], "{p} {hit:?}");
    // near its left edge only
    let (p, hit) = m.mount_snap(&[door], DVec3::new(6.0, -18.0, 250.0), n, 10.0).unwrap();
    assert!(close(p, DVec3::new(0.0, -18.0, 250.0)) && hit.iter().filter(|h| **h).count() == 1, "{p} {hit:?}");
    // nowhere near: as it is
    let (p, hit) = m.mount_snap(&[door], DVec3::new(150.0, -18.0, 600.0), n, 10.0).unwrap();
    assert!(close(p, DVec3::new(150.0, -18.0, 600.0)) && hit == [false, false]);
    // the handle's definition is named by its item
    let d = m.defs.values().find(|d| d.fitting.is_some()).unwrap();
    assert_eq!(d.name, "Handle, centres 192");
}

#[test]
fn handles_knobs_and_legs_come_in_shapes() {
    use varstotas_kernel::fitting::{Builtin, Style};
    for style in Style::ALL {
        for b in [Builtin::Handle { centres: 128.0, style }, Builtin::Knob { diameter: 30.0, style }, Builtin::Leg { height: 400.0, diameter: 40.0, style }] {
            assert!(Model::new().builtin_fitting(b).is_ok(), "{b:?}");
        }
    }
    // hairpin legs on a top: four, each its own item, screwed on at the corners of its plate
    let mut m = Model::new();
    let top = board(&mut m, "Top", DVec3::new(800.0, 500.0, 18.0), DVec3::new(0.0, 0.0, 700.0));
    let j = m.fitting_joint(K::Leg, vec![top]).unwrap();
    let legs = m.put_fitting_joint_styled(&j, Style::Shaped).unwrap();
    assert_eq!(legs.len(), 4);
    let d = m.defs.values().find(|d| d.fitting.is_some()).unwrap();
    assert_eq!(d.fitting.as_ref().unwrap().item, "Hairpin leg, height 100");
    assert_eq!(d.fitting.as_ref().unwrap().holes.len(), 4);
    // a round one beside them is another item
    let round = m.put_fitting_joint(&j).unwrap();
    assert_eq!(round.len(), 4);
    let mut items: Vec<String> = m.defs.values().filter_map(|d| d.fitting.as_ref().map(|f| f.item.clone())).collect();
    items.sort();
    assert_eq!(items, ["Hairpin leg, height 100", "Leg, height 100"]);
}

#[test]
fn fittings_are_made_again_in_other_sizes() {
    use varstotas_kernel::fitting::{Builtin, Style};
    let mut m = Model::new();
    let top = board(&mut m, "Top", DVec3::new(800.0, 500.0, 18.0), DVec3::new(0.0, 0.0, 700.0));
    let j = m.fitting_joint(K::Leg, vec![top]).unwrap();
    let legs = m.put_fitting_joint(&j).unwrap();
    let defs = m.defs.len();
    let old = m.def_at(&legs[0]).unwrap();
    let b = m.fitting_of(old).unwrap().builtin.unwrap();
    assert!(matches!(b, Builtin::Leg { height: 100.0, style: Style::Round, .. }), "{b:?}");
    // kept in the file
    let back = varstotas_kernel::io::from_json(&varstotas_kernel::io::to_json(&m).unwrap()).unwrap();
    assert_eq!(back.fitting_of(back.def_at(&legs[0]).unwrap()).unwrap().builtin, Some(b));
    let before: Vec<DVec3> = legs.iter().map(|p| at(&m, p)).collect();
    m.refit(&legs, Builtin::Leg { height: 300.0, diameter: 40.0, style: Style::Square }).unwrap();
    let d = m.def_at(&legs[0]).unwrap();
    assert_eq!(m.fitting_item(d).unwrap(), "Square leg, height 300");
    assert!(legs.iter().all(|p| m.def_at(p) == Some(d)));
    assert!(legs.iter().zip(&before).all(|(p, b)| close(at(&m, p), *b)), "where they were");
    assert!(m.defs.get(old).is_none(), "the old legs' definition went");
    assert_eq!(m.defs.len(), defs, "a leg for a leg, a piece for a piece");
    assert_eq!(m.placed_fittings().len(), 4);
}
