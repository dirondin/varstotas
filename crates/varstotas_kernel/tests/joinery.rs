//! Joinery: joints between touching parts cut into their geometry, editable and removable.

use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::joinery::{WoodJoint, WoodJointKind, contact};
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

#[test]
fn fasteners_follow_their_part() {
    use varstotas_kernel::assembly::SceneAssembly;
    use varstotas_kernel::fasteners::Solid;
    let (mut m, side, shelf) = cabinet();
    m.add_wood_joint(WoodJointKind::Dowels, vec![side], vec![shelf]).unwrap();
    // three dowels across the contact plane x = 18, 12 into the side and 30 into the shelf
    assert_eq!(m.fasteners.len(), 3);
    for f in &m.fasteners {
        let Solid::Round { at, dir, len, .. } = f.solids[0] else { panic!("a dowel is round") };
        assert_eq!((at.x, dir, len), (47.0, DVec3::NEG_X, 40.0));
        assert_eq!(f.name, "Dowel Ø8 × 40");
    }
    let faces = m.fastener_faces();
    assert!(!faces.is_empty() && faces.iter().all(|f| f.color.is_some() && f.face.is_none()));
    // exploded: without steps they stay in the shelf (b) and move with it
    m.set_assembly_view(SceneAssembly { explode: 100.0, step: None });
    let shift = m.assembly_offset(&[shelf]);
    assert!(shift.length() > 1.0);
    let moved = m.fastener_transform(&m.fasteners[0]).unwrap();
    assert!(moved.transform_point3(DVec3::ZERO).distance(shift) < 1e-9);
    // hidden from the view, and gone with the joint
    m.assemble();
    m.fasteners_hidden = true;
    assert!(m.fastener_faces().is_empty());
    m.remove_wood_joint(0);
    assert!(m.fasteners.is_empty() && m.joint_contacts.is_empty());
}

#[test]
fn layouts_along_and_across_the_joint() {
    let (mut m, side, shelf) = cabinet();
    let i = m.add_wood_joint(WoodJointKind::Dowels, vec![side], vec![shelf]).unwrap();
    let pos = |m: &Model| {
        let c = m.joint_contacts[i].unwrap();
        m.joints[i].positions(&c)
    };
    // evenly, 32 mm from the ends of the 300 mm joint
    assert_eq!(pos(&m), vec![32.0, 150.0, 268.0]);
    // the 32 mm system: a margin and a pitch
    let mut j = m.joints[i].clone();
    (j.count, j.margin, j.pitch) = (4, 37.0, 32.0);
    m.set_wood_joint(i, j.clone()).unwrap();
    assert_eq!(pos(&m), vec![37.0, 69.0, 101.0, 133.0]);
    // one fastener with a margin sits at the margin, without one in the middle
    (j.count, j.pitch) = (1, 0.0);
    m.set_wood_joint(i, j.clone()).unwrap();
    assert_eq!(pos(&m), vec![37.0]);
    // at distances, which set the count; off centre across the joint
    (j.at, j.offset) = (vec![20.0, 200.0], 3.0);
    m.set_wood_joint(i, j.clone()).unwrap();
    assert_eq!((pos(&m), m.joints[i].fastener_count()), (vec![20.0, 200.0], 2));
    let c = m.joint_contacts[i].unwrap();
    assert_eq!(m.joints[i].point(&c, c.lo[c.along] + 20.0).z, 409.0 + 3.0);
    assert_eq!(m.fasteners.len(), 2);
    // a point off the joint's end or a hole past its side is refused and the joint kept
    for bad in [WoodJoint { at: vec![20.0, 298.0], ..j.clone() }, WoodJoint { offset: 6.0, ..j.clone() }] {
        assert!(m.set_wood_joint(i, bad).is_err());
        assert_eq!(pos(&m), vec![20.0, 200.0]);
    }
}

#[test]
fn furniture_fittings() {
    use WoodJointKind as K;
    let hw = |m: &Model, i: usize| {
        let c = varstotas_kernel::joinery::joint_contact(m, &m.joints[i]);
        m.joints[i].hardware(c.as_ref()).unwrap()
    };
    let bodies = |m: &Model, i: usize| m.fasteners.iter().filter(|f| f.joint == i).count();
    // confirmats and cam locks between the side and the shelf
    let (mut m, side, shelf) = cabinet();
    let (vs, vh) = (volume(&m, side), volume(&m, shelf));
    let i = m.add_wood_joint(K::Confirmat, vec![side], vec![shelf]).unwrap();
    assert_eq!(hw(&m, i), ("Confirmat Ø7 × 50".into(), 3));
    assert!(volume(&m, side) < vs && volume(&m, shelf) < vh);
    m.set_wood_joint_kind(i, K::CamLock).unwrap();
    assert_eq!((hw(&m, i).0.as_str(), bodies(&m, i)), ("Cam lock Ø15 with bolt", 6), "a bolt and a cam each");
    // shelf pins: two under the shelf, two rows above and below; the shelf is not cut
    m.set_wood_joint_kind(i, K::ShelfPins).unwrap();
    let mut j = m.joints[i].clone();
    j.rows = 2;
    m.set_wood_joint(i, j).unwrap();
    let c = m.joint_contacts[i].unwrap();
    let (a, b) = m.joints[i].holes(&c);
    assert_eq!((a.len(), b.len(), bodies(&m, i)), (10, 0, 2));
    assert_eq!(a[0].0.z, 400.0 - 2.5, "just under the shelf");
    assert_eq!(volume(&m, shelf), vh);
    // an overlay door on the side's front edge: two cups in the door, 22.5 from its edge
    let door = board(&mut m, "Door", DVec3::new(600.0, 18.0, 800.0), DVec3::new(0.0, -18.0, 0.0));
    let vd = volume(&m, door);
    let h = m.add_wood_joint(K::Hinge, vec![side], vec![door]).unwrap();
    assert_eq!(m.joints[h].a, vec![door], "the door's face is the larger");
    assert_eq!(hw(&m, h), ("Cup hinge Ø35, overlay".into(), 2));
    let c = m.joint_contacts[h].unwrap();
    assert_eq!(m.joints[h].holes(&c).0[0].0.x, 22.5);
    assert!(volume(&m, door) < vd);
    // opening right back, said in its name
    let mut j = m.joints[h].clone();
    j.angle = 165.0;
    m.set_wood_joint(h, j).unwrap();
    assert_eq!(hw(&m, h).0, "Cup hinge Ø35, overlay, 165°");
    // a gas strut from inside the side to the door's back, about three times its weight;
    // the tube with the side, the rod with the door, two screws in each
    let (vs, vd) = (volume(&m, side), volume(&m, door));
    let g = m.add_wood_joint(K::GasStrut, vec![door], vec![side]).unwrap();
    assert_eq!(m.joints[g].a, vec![door]);
    assert_eq!((hw(&m, g), bodies(&m, g)), (("Gas strut 180 N".into(), 1), 2));
    let (ha, hb) = m.joints[g].holes(&m.joint_contacts[g].unwrap());
    assert_eq!((ha.len(), hb.len()), (2, 2));
    assert!(hb.iter().all(|h| h.0.x == 18.0 && h.1 == DVec3::NEG_X), "into the side's inner face");
    assert!(volume(&m, side) < vs && volume(&m, door) < vd);
    m.remove_wood_joint(g);
    // a door over half the side's edge: half overlay hinges
    let (mut m2, side2, _) = cabinet();
    let door2 = board(&mut m2, "Door", DVec3::new(591.0, 18.0, 800.0), DVec3::new(9.0, -18.0, 0.0));
    let h2 = m2.add_wood_joint(K::Hinge, vec![side2], vec![door2]).unwrap();
    assert_eq!(hw(&m2, h2).0, "Cup hinge Ø35, half overlay");
    // a handle on the door's front, the longest that fits; legs under the shelf
    let k = m.add_wood_joint(K::Handle, vec![door], vec![]).unwrap();
    assert_eq!((hw(&m, k), m.joints[k].b.len()), (("Handle, centres 192".into(), 1), 0));
    assert_eq!(m.joint_contacts[k].unwrap().at, -18.0, "on the front face");
    let l = m.add_wood_joint(K::Leg, vec![shelf], vec![]).unwrap();
    assert_eq!((hw(&m, l), bodies(&m, l)), (("Leg, height 100".into(), 4), 4));
    // or casters instead: a plate and fork, and a wheel, each; four screws per plate
    m.remove_wood_joint(l);
    let vs = volume(&m, shelf);
    let w = m.add_wood_joint(K::Caster, vec![shelf], vec![]).unwrap();
    assert_eq!((hw(&m, w), bodies(&m, w)), (("Swivel caster Ø50, height 70".into(), 4), 8));
    assert_eq!(m.joints[w].holes(&m.joint_contacts[w].unwrap()).0.len(), 16);
    assert!(volume(&m, shelf) < vs);
    assert!(m.add_wood_joint(K::Knob, vec![], vec![]).is_err());
    // a back panel nailed onto the side's back edge: nothing cut
    let back = board(&mut m, "Back", DVec3::new(600.0, 6.0, 800.0), DVec3::new(0.0, 300.0, 0.0));
    let vb = volume(&m, back);
    let n = m.add_wood_joint(K::Nails, vec![side], vec![back]).unwrap();
    assert_eq!(hw(&m, n), ("Nail 1.6 × 25".into(), 9));
    assert_eq!(volume(&m, back), vb);
    // a top on figure 8 fasteners recessed into the side's top edge
    let top = board(&mut m, "Top", DVec3::new(600.0, 300.0, 18.0), DVec3::new(0.0, 0.0, 800.0));
    let t = m.add_wood_joint(K::TableTop, vec![side], vec![top]).unwrap();
    assert_eq!(hw(&m, t).0, "Table-top fastener (figure 8)");
    // a ball-bearing slide: 12.7 mm between the sides, the longest that fits
    let (mut m, side, _) = cabinet();
    let drawer = board(&mut m, "Drawer side", DVec3::new(12.0, 450.0, 150.0), DVec3::new(18.0 + 12.7, 0.0, 100.0));
    let s = m.add_wood_joint(K::Slide, vec![side], vec![drawer]).unwrap();
    assert_eq!(hw(&m, s), ("Ball-bearing slide 300".into(), 1));
    assert_eq!(bodies(&m, s), 2);
    m.root_mesh_mut().instances[drawer].transform = DMat4::from_translation(DVec3::new(38.0, 0.0, 100.0));
    assert!(m.add_wood_joint(K::Slide, vec![side], vec![drawer]).is_err_and(|e| e.to_string().contains("12.7")));
}

#[test]
fn a_drawer_for_an_opening() {
    use varstotas_kernel::drawer::{DrawerFront, DrawerSpec};
    let mut m = Model::new();
    let l = board(&mut m, "Left", DVec3::new(18.0, 500.0, 700.0), DVec3::ZERO);
    let r = board(&mut m, "Right", DVec3::new(18.0, 500.0, 700.0), DVec3::new(518.0, 0.0, 0.0));
    let z = m.drawer_z(&[l], &[r]).unwrap();
    let parts = m.add_drawer(&[l], &[r], DrawerSpec { z, ..Default::default() }).unwrap();
    assert_eq!(parts.len(), 6, "sides, box front and back, bottom, front");
    let size = |m: &Model, id| {
        let b = m.instance_world_bounds(&[], id).unwrap();
        (b.min, b.max - b.min)
    };
    // 12.7 mm each side, 450 slides behind an 18 mm inset front
    let (min, s) = size(&m, parts[0]);
    assert!((min.x - 30.7).abs() < 1e-9 && (s.y - 450.0).abs() < 1e-9 && (min.y - 18.0).abs() < 1e-9, "{min} {s}");
    let names: Vec<String> = m.joints.iter().map(|j| format!("{:?}", j.kind)).collect();
    assert_eq!(names, ["Slide", "Slide", "Dowels", "Dowels", "Nails", "Dowels", "Dowels", "Nails", "Screws", "Handle"]);
    let hw: Vec<String> = varstotas_kernel::cutlist::plan(&m).hardware.iter().map(|h| format!("{} × {}", h.count, h.name)).collect();
    assert!(hw.contains(&"2 × Ball-bearing slide 450".to_string()), "{hw:?}");
    // an overlay front stands in front of the carcass; no room: refused, nothing added
    let mut m2 = m.clone();
    let p2 = m2.add_drawer(&[l], &[r], DrawerSpec { z: 400.0, front: DrawerFront::Overlay, ..Default::default() }).unwrap();
    assert_eq!(size(&m2, p2[5]).0.y, -18.0);
    assert!(m.add_drawer(&[l], &[r], DrawerSpec { z: 650.0, ..Default::default() }).is_err());
}

#[test]
fn a_chest_of_drawers() {
    use varstotas_kernel::drawer::{DrawerFront, DrawerSpec};
    use varstotas_kernel::joinery::{SlideKind, WoodJointKind as K};
    // sides with a bottom and a top between them, a back behind: the free height is between
    // the bottom and the top
    let mut m = Model::new();
    let l = board(&mut m, "Left", DVec3::new(18.0, 500.0, 800.0), DVec3::ZERO);
    let r = board(&mut m, "Right", DVec3::new(18.0, 500.0, 800.0), DVec3::new(518.0, 0.0, 0.0));
    board(&mut m, "Bottom", DVec3::new(500.0, 500.0, 18.0), DVec3::new(18.0, 0.0, 50.0));
    board(&mut m, "Top", DVec3::new(500.0, 500.0, 18.0), DVec3::new(18.0, 0.0, 782.0));
    board(&mut m, "Back", DVec3::new(500.0, 4.0, 800.0), DVec3::new(18.0, 496.0, 0.0));
    let space = m.drawer_space(&[l], &[r]).unwrap();
    assert_eq!(space, (68.0, 782.0));
    let size = |m: &Model, id| {
        let b = m.instance_world_bounds(&[], id).unwrap();
        (b.min, b.max)
    };
    // three inset fronts of equal height 2 mm apart, roller slides 12.5 mm each side
    let mut m1 = m.clone();
    let spec = DrawerSpec { slide: SlideKind::Roller, ..Default::default() };
    let ds = m1.add_drawers(&[l], &[r], 3, space, spec).unwrap();
    assert_eq!(ds.len(), 3);
    let fronts: Vec<(f64, f64)> = ds.iter().map(|d| size(&m1, d[5])).map(|(a, b)| (a.z, b.z)).collect();
    let fh = (714.0 - 4.0 * 2.0) / 3.0;
    for (i, (f0, f1)) in fronts.iter().enumerate() {
        assert!((f0 - (70.0 + i as f64 * (fh + 2.0))).abs() < 1e-6 && (f1 - f0 - fh).abs() < 1e-6, "{fronts:?}");
    }
    // each box within its front and clear of the bottom and the top
    for d in &ds {
        let (f0, f1) = (size(&m1, d[5]).0.z, size(&m1, d[5]).1.z);
        for &p in &d[..5] {
            let (a, b) = size(&m1, p);
            assert!(a.z >= f0.max(68.0) + 9.9 && b.z <= f1.min(782.0) - 9.9 && a.x >= 30.5 - 1e-6, "{a} {b}");
        }
    }
    let hw: Vec<String> = varstotas_kernel::cutlist::plan(&m1).hardware.iter().map(|h| format!("{} × {}", h.count, h.name)).collect();
    assert!(hw.contains(&"6 × Roller slide 450".to_string()) && hw.contains(&"3 × Handle, centres 192".to_string()), "{hw:?}");
    // a drawer is in the way now: what is left is above it
    let mut m2 = m.clone();
    m2.add_drawers(&[l], &[r], 1, (68.0, 300.0), DrawerSpec::default()).unwrap();
    assert_eq!(m2.drawer_space(&[l], &[r]).unwrap(), (298.0, 782.0), "above its front");
    // overlay fronts push to open on undermount slides, the slides on the bottom's edges, 5 mm
    // each side, room under the box; no handles
    let spec = DrawerSpec { slide: SlideKind::Undermount, push: true, front: DrawerFront::Overlay, ..Default::default() };
    let ds = m.add_drawers(&[l], &[r], 2, space, spec).unwrap();
    let (f0, f1) = (size(&m, ds[0][5]).0.z, size(&m, ds[1][5]).1.z);
    assert!((f0 - 52.0).abs() < 1e-6 && (f1 - 798.0).abs() < 1e-6, "{f0} {f1}");
    assert!((size(&m, ds[0][4]).0.z - 92.0).abs() < 1e-6 && (size(&m, ds[0][0]).0.x - 23.0).abs() < 1e-6);
    let slides: Vec<_> = m.joints.iter().filter(|j| j.kind == K::Slide).collect();
    assert_eq!(slides.len(), 4);
    assert!(slides.iter().all(|j| j.slide == SlideKind::Undermount && j.push && (j.b == vec![ds[0][4]] || j.b == vec![ds[1][4]])));
    assert!(!m.joints.iter().any(|j| j.kind == K::Handle));
    let hw: Vec<String> = varstotas_kernel::cutlist::plan(&m).hardware.iter().map(|h| format!("{} × {}", h.count, h.name)).collect();
    assert!(hw.contains(&"4 × Undermount slide 450, push to open".to_string()), "{hw:?}");
    // too many for the height: refused, nothing added
    let n = m.joints.len();
    assert!(m.add_drawers(&[l], &[r], 16, space, DrawerSpec::default()).is_err());
    assert_eq!(m.joints.len(), n);
}
