//! Joinery: joints between touching parts cut into their geometry, editable and removable.

use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::joinery::{WoodJointKind, contact};
use varstotas_kernel::primitives::Primitive;
use varstotas_kernel::{InstanceId, Model};

/// A box group `l` × `w` × `t` (x, y, z) named `name` at `at`.
fn board(m: &mut Model, name: &str, s: DVec3, at: DVec3) -> InstanceId {
    let mesh = Primitive::Box { w: s.x, d: s.y, h: s.z }.mesh().unwrap();
    let id = m.add_group_mesh(&[], mesh, "Group", DMat4::from_translation(at)).unwrap();
    m.root_mesh_mut().instances[id].name = name.into();
    id
}

fn volume(m: &Model, id: InstanceId) -> f64 {
    m.defs[m.root_mesh().instances[id].def].mesh.volume()
}

/// A side 18 thick (x) and a shelf whose end meets it.
fn cabinet() -> (Model, InstanceId, InstanceId) {
    let mut m = Model::new();
    let side = board(&mut m, "Side", DVec3::new(18.0, 300.0, 800.0), DVec3::ZERO);
    let shelf = board(&mut m, "Shelf", DVec3::new(500.0, 300.0, 18.0), DVec3::new(18.0, 0.0, 400.0));
    (m, side, shelf)
}

#[test]
fn the_contact_is_the_shelf_end() {
    let (m, side, shelf) = cabinet();
    let c = contact(&m, &[side], &[shelf]).expect("they touch");
    assert_eq!((c.axis, c.at, c.into_a), (0, 18.0, -1.0));
    assert_eq!((c.length(), c.width(), c.a_size, c.b_size), (300.0, 18.0, 18.0, 500.0));
    // apart: no contact
    let mut m2 = m.clone();
    m2.root_mesh_mut().instances[shelf].transform = DMat4::from_translation(DVec3::new(30.0, 0.0, 400.0));
    assert!(contact(&m2, &[side], &[shelf]).is_none());
}

#[test]
fn joints_cut_both_parts_and_come_off_again() {
    let (mut m, side, shelf) = cabinet();
    let (vs, vh) = (volume(&m, side), volume(&m, shelf));
    // picked shelf first: the side is still `a`, the part joined into
    let i = m.add_wood_joint(WoodJointKind::Housing, vec![shelf], vec![side]).unwrap();
    let j = m.joints[i].clone();
    assert_eq!((j.a.clone(), j.b.clone(), j.depth), (vec![side], vec![shelf], 6.0));
    let groove = 300.0 * 18.0 * 6.0;
    assert!((volume(&m, side) - (vs - groove)).abs() < 1e-3, "{}", volume(&m, side));
    assert!((volume(&m, shelf) - (vh + groove)).abs() < 1e-3, "{}", volume(&m, shelf));
    // the cut list sees the longer shelf
    let parts = varstotas_kernel::cutlist::cut_parts(&m);
    assert_eq!(parts.iter().find(|p| p.name == "Shelf").unwrap().size, [506.0, 300.0, 18.0]);

    // as mortise and tenon, through: the tenon 6 thick inside 30 mm shoulders
    let mut t = j.clone();
    t.kind = WoodJointKind::MortiseTenon;
    (t.thickness, t.shoulder, t.through) = (6.0, 30.0, true);
    m.set_wood_joint(i, t).unwrap();
    let tenon = 6.0 * 240.0 * 18.0;
    assert!((volume(&m, side) - (vs - tenon)).abs() < 1e-3);
    assert!((volume(&m, shelf) - (vh + tenon)).abs() < 1e-3);
    // a tenon thicker than the shelf is refused and the joint stays
    let mut bad = m.joints[i].clone();
    bad.thickness = 30.0;
    assert!(m.set_wood_joint(i, bad).is_err());
    assert_eq!(m.joints[i].thickness, 6.0);
    assert!((volume(&m, shelf) - (vh + tenon)).abs() < 1e-3);

    // dowels: three 8 mm holes, 12 into the side and 30 into the shelf
    let mut d = m.joints[i].clone();
    d.kind = WoodJointKind::Dowels;
    (d.count, d.diameter, d.depth, d.depth_b) = (3, 8.0, 12.0, 30.0);
    m.set_wood_joint(i, d).unwrap();
    assert!(volume(&m, side) < vs - 3.0 * 500.0 && volume(&m, side) > vs - 3.0 * 700.0, "{}", volume(&m, side));
    assert!(volume(&m, shelf) < vh - 3.0 * 1300.0, "{}", volume(&m, shelf));
    assert_eq!(m.joints[i].hardware(None), Some(("Dowel Ø8 × 40".to_string(), 3)));
    // on the cut list: three holes in the side's face (length up, width across), none on the
    // shelf, whose holes are in its end
    let parts = varstotas_kernel::cutlist::cut_parts(&m);
    let holes = |n: &str| parts.iter().find(|p| p.name == n).unwrap().holes.clone();
    assert_eq!(holes("Side"), vec![[409.0, 32.0, 8.0], [409.0, 150.0, 8.0], [409.0, 268.0, 8.0]]);
    assert!(holes("Shelf").is_empty());

    // saved and loaded: still rebuilds
    let mut back = varstotas_kernel::io::from_json(&varstotas_kernel::io::to_json(&m).unwrap()).unwrap();
    let errs = back.rebuild_wood_joints();
    assert!(errs.is_empty(), "{errs:?}");
    assert!((volume(&back, side) - volume(&m, side)).abs() < 1e-6);

    // removed: both parts as they were
    m.remove_wood_joint(i);
    assert!(m.joints.is_empty() && m.joint_bases.is_empty());
    assert!((volume(&m, side) - vs).abs() < 1e-6 && (volume(&m, shelf) - vh).abs() < 1e-6);
}

#[test]
fn screws_and_parts_that_do_not_touch() {
    let (mut m, side, shelf) = cabinet();
    let vs = volume(&m, side);
    let i = m.add_wood_joint(WoodJointKind::Screws, vec![side], vec![shelf]).unwrap();
    assert_eq!(m.joints[i].count, 3);
    assert!(volume(&m, side) < vs, "clearance holes through the side");
    let c = contact(&m, &[side], &[shelf]);
    assert_eq!(m.joints[i].hardware(c.as_ref()).unwrap().0, "Screw Ø4 × 50");
    // in the cut list's hardware, priced by name
    m.cut_list.parts.insert("Screw Ø4 × 50".into(), varstotas_kernel::cutlist::PartOptions { price: 0.1, ..Default::default() });
    let hw = varstotas_kernel::cutlist::plan(&m).hardware;
    assert_eq!((hw[0].name.as_str(), hw[0].count), ("Screw Ø4 × 50", 3));
    assert!((hw[0].cost() - 0.3).abs() < 1e-9);
    let far = board(&mut m, "Far", DVec3::new(18.0, 300.0, 800.0), DVec3::new(1000.0, 0.0, 0.0));
    assert!(m.add_wood_joint(WoodJointKind::Dowels, vec![far], vec![shelf]).is_err());
    assert_eq!(m.joints.len(), 1);
}

/// A side and a top meeting at the corner: the side's end flush with the top's upper face.
fn corner() -> (Model, InstanceId, InstanceId) {
    let mut m = Model::new();
    let side = board(&mut m, "Side", DVec3::new(18.0, 300.0, 800.0), DVec3::ZERO);
    let top = board(&mut m, "Top", DVec3::new(500.0, 300.0, 18.0), DVec3::new(18.0, 0.0, 782.0));
    (m, side, top)
}

#[test]
fn box_joints_and_dovetails_at_a_corner() {
    let (mut m, side, top) = corner();
    let (vs, vt) = (volume(&m, side), volume(&m, top));
    // nine fingers about as wide as the boards are thick, right through the side
    let i = m.add_wood_joint(WoodJointKind::BoxJoint, vec![side], vec![top]).unwrap();
    assert_eq!(m.joints[i].count, 9);
    let fingers = 9.0 * (300.0 / 17.0) * 18.0 * 18.0;
    assert!((volume(&m, top) - (vt + fingers)).abs() < 1e-3, "{}", volume(&m, top));
    assert!((volume(&m, side) - (vs - fingers)).abs() < 1.0, "{}", volume(&m, side));
    assert_eq!(m.joints[i].hardware(None), None);

    // dovetails: six tails 1:8 between 6 mm pins
    m.set_wood_joint_kind(i, WoodJointKind::Dovetail).unwrap();
    let j = m.joints[i].clone();
    assert_eq!((j.count, j.pin), (6, 6.0));
    let wide = (300.0 - 7.0 * 6.0) / 6.0;
    let narrow = wide - 2.0 * 18.0 * 7.1f64.to_radians().tan();
    let tails = 6.0 * (wide + narrow) / 2.0 * 18.0 * 18.0;
    assert!((volume(&m, top) - (vt + tails)).abs() < 1e-3, "{}", volume(&m, top));
    assert!((volume(&m, side) - (vs - tails)).abs() < 1.0, "{}", volume(&m, side));
    // too many tails for the pins: refused
    let mut bad = j.clone();
    bad.count = 30;
    assert!(m.set_wood_joint(i, bad).is_err());
    m.remove_wood_joint(i);
    assert!((volume(&m, side) - vs).abs() < 1e-6 && (volume(&m, top) - vt).abs() < 1e-6);
}

#[test]
fn pocket_holes() {
    let (mut m, side, shelf) = cabinet();
    let (vs, vh) = (volume(&m, side), volume(&m, shelf));
    let i = m.add_wood_joint(WoodJointKind::PocketHole, vec![side], vec![shelf]).unwrap();
    let j = m.joints[i].clone();
    assert_eq!((j.count, j.diameter, j.depth), (3, 4.0, 11.0));
    assert_eq!(j.hardware(None), Some(("Pocket screw Ø4 × 25".to_string(), 3)));
    // pockets in the shelf, pilot holes in the side, none of them through the side
    assert!(volume(&m, shelf) < vh - 3.0 * 1000.0, "{}", volume(&m, shelf));
    assert!(volume(&m, side) < vs && volume(&m, side) > vs - 3.0 * 200.0, "{}", volume(&m, side));
    // the pockets show on the shelf's face, the pilot holes in the side's
    let parts = varstotas_kernel::cutlist::cut_parts(&m);
    let shelf_holes = &parts.iter().find(|p| p.name == "Shelf").unwrap().holes;
    assert_eq!(shelf_holes.len(), 3);
    assert!(shelf_holes.iter().all(|h| h[2] == 9.5 && (h[0] - 33.6).abs() < 0.1), "{shelf_holes:?}");
    assert_eq!(parts.iter().find(|p| p.name == "Side").unwrap().holes.len(), 3);
    // from the other face too
    let mut f = j.clone();
    f.flip = true;
    m.set_wood_joint(i, f).unwrap();
    assert!(volume(&m, shelf) < vh - 3.0 * 1000.0);
}
