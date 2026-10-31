//! Assembly: steps from the contact graph, the exploded view and steps shown.

use varstotas_kernel::assembly::{SceneAssembly, suggest_steps};
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::joinery::WoodJointKind;
use varstotas_kernel::primitives::Primitive;
use varstotas_kernel::{InstanceId, Model};

fn board(m: &mut Model, name: &str, s: DVec3, at: DVec3) -> InstanceId {
    let mesh = Primitive::Box { w: s.x, d: s.y, h: s.z }.mesh().unwrap();
    let id = m.add_group_mesh(&[], mesh, "Group", DMat4::from_translation(at)).unwrap();
    m.root_mesh_mut().instances[id].name = name.into();
    id
}

/// Two sides standing on a bottom, a top on them and a shelf between them.
fn cabinet() -> (Model, [InstanceId; 5]) {
    let mut m = Model::new();
    let bottom = board(&mut m, "Bottom", DVec3::new(600.0, 300.0, 18.0), DVec3::ZERO);
    let left = board(&mut m, "Side", DVec3::new(18.0, 300.0, 700.0), DVec3::new(0.0, 0.0, 18.0));
    let right = board(&mut m, "Side", DVec3::new(18.0, 300.0, 700.0), DVec3::new(582.0, 0.0, 18.0));
    let top = board(&mut m, "Top", DVec3::new(600.0, 300.0, 18.0), DVec3::new(0.0, 0.0, 718.0));
    let shelf = board(&mut m, "Shelf", DVec3::new(564.0, 300.0, 18.0), DVec3::new(18.0, 0.0, 350.0));
    (m, [bottom, left, right, top, shelf])
}

fn at(m: &Model, id: InstanceId) -> DVec3 {
    m.root_mesh().instances[id].transform.w_axis.truncate()
}

#[test]
fn steps_follow_the_contacts() {
    let (m, [bottom, left, right, top, shelf]) = cabinet();
    let steps: Vec<Vec<InstanceId>> = suggest_steps(&m).into_iter().map(|s| s.parts.into_iter().map(|p| p[0]).collect()).collect();
    // the bottom is the base; both sides on it, then the shelf between them, the top
    assert_eq!(steps, vec![vec![bottom], vec![left, right], vec![shelf], vec![top]]);
}

#[test]
fn exploded_and_step_views_come_back() {
    let (mut m, [bottom, left, right, top, shelf]) = cabinet();
    let before: Vec<DVec3> = [bottom, left, right, top, shelf].iter().map(|&i| at(&m, i)).collect();
    m.set_assembly_view(SceneAssembly { explode: 100.0, step: None });
    // stretched from the middle and the floor, the outermost parts by 100: the sides sideways,
    // the top up, the shelf up between them, the bottom on the floor; steps change nothing
    let moved = |m: &Model| -> Vec<DVec3> { [bottom, left, right, top, shelf].iter().enumerate().map(|(k, &i)| at(m, i) - before[k]).collect() };
    let z = |h: f64| h * 100.0 / 736.0;
    let expected = vec![DVec3::ZERO, DVec3::new(-97.0, 0.0, z(18.0)), DVec3::new(97.0, 0.0, z(18.0)), DVec3::new(0.0, 0.0, z(718.0)), DVec3::new(0.0, 0.0, z(350.0))];
    let close = |a: &[DVec3], b: &[DVec3]| a.iter().zip(b).all(|(x, y)| x.distance(*y) < 1e-9);
    assert!(close(&moved(&m), &expected), "{:?}", moved(&m));
    m.assembly.steps = suggest_steps(&m);
    m.set_assembly_view(SceneAssembly { explode: 100.0, step: None });
    assert!(close(&moved(&m), &expected), "{:?}", moved(&m));
    // the cut list and joints still see the assembled cabinet
    assert_eq!(varstotas_kernel::cutlist::cut_parts(&m).len(), 4);
    let i = m.add_wood_joint(WoodJointKind::Dowels, vec![left], vec![shelf]).unwrap();
    assert_eq!(m.joints[i].a, vec![left]);
    // step 2 of 4: the later parts hidden, still in the cut list
    m.set_assembly_view(SceneAssembly { explode: 0.0, step: Some(1) });
    let hidden = |m: &Model, id| m.root_mesh().instances[id].hidden;
    assert!(!hidden(&m, bottom) && hidden(&m, top) && hidden(&m, shelf));
    assert_eq!(varstotas_kernel::cutlist::cut_parts(&m).iter().map(|p| p.count).sum::<usize>(), 5);
    assert_eq!(m.joint_step(i), Some(2));
    // assembled again: everything back where it was
    m.assemble();
    assert!(!hidden(&m, top) && !hidden(&m, shelf));
    for (k, id) in [bottom, left, right, top, shelf].into_iter().enumerate() {
        assert_eq!(at(&m, id), before[k]);
    }
    assert!(m.assembly.moved.is_empty() && m.assembly.hid.is_empty());
}

#[test]
fn fasteners_get_steps_of_their_own() {
    let (mut m, [bottom, left, right, top, shelf]) = cabinet();
    let screws = m.add_wood_joint(WoodJointKind::Screws, vec![bottom], vec![left]).unwrap();
    let l = m.add_wood_joint(WoodJointKind::Dowels, vec![left], vec![shelf]).unwrap();
    let r = m.add_wood_joint(WoodJointKind::Dowels, vec![right], vec![shelf]).unwrap();
    m.assembly.steps = suggest_steps(&m);
    let parts: Vec<Vec<InstanceId>> = m.assembly.steps.iter().map(|s| s.parts.iter().map(|p| p[0]).collect()).collect();
    // the dowels go into the sides in a step before the shelf comes onto them; screws go in
    // with their later part
    assert_eq!(parts, vec![vec![bottom], vec![left, right], vec![], vec![shelf], vec![top]]);
    assert_eq!(m.assembly.steps[2].joints, vec![l, r]);
    assert_eq!((m.fastener_step(l), m.fastener_step(r), m.fastener_step(screws)), (Some(2), Some(2), Some(1)));
    // shown from their step on, in the sides
    let shown = |m: &Model, j: usize| m.fasteners.iter().filter(|f| f.joint == j).all(|f| m.fastener_transform(f).is_some());
    m.set_assembly_view(SceneAssembly { explode: 0.0, step: Some(1) });
    assert!(!shown(&m, l) && shown(&m, screws));
    m.set_assembly_view(SceneAssembly { explode: 0.0, step: Some(2) });
    assert!(shown(&m, l) && shown(&m, r) && m.root_mesh().instances[shelf].hidden);
    let f = m.fasteners.iter().find(|f| f.joint == l).unwrap();
    assert_eq!(m.fastener_anchor(f).unwrap().0, &vec![left]);
    m.assemble();
    // saved with the steps; a joint deleted is renumbered, the step goes with its last joint
    let json = serde_json::to_string(&m.assembly).unwrap();
    assert!(json.contains("\"joints\":[1,2]"));
    m.remove_wood_joint(l);
    assert_eq!(m.assembly.steps[2].joints, vec![r - 1]);
    m.remove_wood_joint(r - 1);
    assert_eq!(m.assembly.steps.len(), 4);
}

#[test]
fn exploded_parts_stay_clear_of_each_other() {
    let (mut m, [bottom, left, right, top, shelf]) = cabinet();
    // a drawer under the shelf, clear of the sides: a box and its front, touching each other only
    let drawer = board(&mut m, "Drawer", DVec3::new(540.0, 280.0, 150.0), DVec3::new(30.0, 10.0, 60.0));
    let front = board(&mut m, "Front", DVec3::new(540.0, 10.0, 150.0), DVec3::new(30.0, 0.0, 60.0));
    let ids = [bottom, left, right, top, shelf, drawer, front];
    let before: Vec<DVec3> = ids.iter().map(|&i| at(&m, i)).collect();
    // at any distance, moving evenly with it
    for d in [10.0, 60.0, 200.0, 900.0] {
        m.set_assembly_view(SceneAssembly { explode: d, step: None });
        let boxes: Vec<_> = ids.iter().map(|&i| m.instance_world_bounds(&[], i).unwrap()).collect();
        for i in 0..boxes.len() {
            for j in i + 1..boxes.len() {
                let (a, b) = (boxes[i], boxes[j]);
                let into = (0..3).all(|x| a.min[x] < b.max[x] - 1e-6 && b.min[x] < a.max[x] - 1e-6);
                assert!(!into, "parts {i} and {j} overlap at {d}");
            }
        }
        assert!(boxes.iter().all(|b| b.min.z >= -1e-9));
    }
    m.set_assembly_view(SceneAssembly { explode: 200.0, step: None });
    let half: Vec<DVec3> = {
        let mut h = m.clone();
        h.set_assembly_view(SceneAssembly { explode: 100.0, step: None });
        ids.iter().enumerate().map(|(k, &i)| at(&h, i) - before[k]).collect()
    };
    let moved: Vec<DVec3> = ids.iter().enumerate().map(|(k, &i)| at(&m, i) - before[k]).collect();
    // the drawer comes out to the front as a whole, its front further
    assert!(moved[5].y <= -200.0 && moved[5].x.abs() < 1e-9 && moved[5].z >= 0.0, "{:?}", moved[5]);
    assert!(moved[6].y < moved[5].y);
    assert!(moved.iter().zip(&half).all(|(a, b)| a.distance(*b * 2.0) < 1e-9));
}

#[test]
fn a_component_with_steps_is_a_sub_assembly() {
    use varstotas_kernel::EntityRef;
    use varstotas_kernel::assembly::suggest_all;
    let (mut m, [bottom, left, right, top, shelf]) = cabinet();
    // a tray on the shelf: its bottom and a front screwed onto it, made a component
    let base = board(&mut m, "Tray bottom", DVec3::new(400.0, 250.0, 12.0), DVec3::new(50.0, 20.0, 368.0));
    let front = board(&mut m, "Tray front", DVec3::new(400.0, 12.0, 80.0), DVec3::new(50.0, 20.0, 380.0));
    m.add_wood_joint(WoodJointKind::Screws, vec![base], vec![front]).unwrap();
    let tray = m.make_component(&[], &[EntityRef::Instance(base), EntityRef::Instance(front)], "Tray").unwrap();
    let td = m.root_mesh().instances[tray].def;
    assert!(m.joints.is_empty() && m.defs[td].joints.len() == 1);
    // the tray gets steps of its own and goes into the cabinet whole, onto the shelf before
    // the top
    let (steps, subs) = suggest_all(&m);
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].0, td);
    assert_eq!(subs[0].1.len(), 2);
    let ids: Vec<Vec<Vec<InstanceId>>> = steps.iter().map(|s| s.parts.clone()).collect();
    assert_eq!(ids, vec![vec![vec![bottom]], vec![vec![left], vec![right]], vec![vec![shelf]], vec![vec![tray]], vec![vec![top]]]);
    m.assembly.steps = steps;
    m.defs[td].steps = subs[0].1.clone();
    m.refresh_fasteners();
    // its screws go in at its own steps, not at the cabinet's
    assert_eq!(m.placed_joints.len(), 1);
    assert_eq!(m.fastener_step(0), None);
    let sub = m.sub_model(td);
    assert_eq!(sub.joints.len(), 1);
    assert_eq!(sub.fastener_step(0), Some(1));
    // exploded, the tray comes out whole
    m.set_assembly_view(SceneAssembly { explode: 100.0, step: None });
    let off = |m: &Model, rel: InstanceId| m.assembly_offset(&[tray, rel]);
    let (b0, f0) = {
        let inner = &m.defs[td].mesh.instances;
        let mut ids = inner.iter().map(|(id, _)| id);
        (ids.next().unwrap(), ids.next().unwrap())
    };
    assert!(off(&m, b0).length() > 1.0 && off(&m, b0) == off(&m, f0));
    // its own view: its front hidden at its first step, the cabinet as it is
    m.set_assembly_within(vec![tray]);
    assert!(m.assembly.moved.is_empty());
    m.set_assembly_view(SceneAssembly { explode: 50.0, step: Some(0) });
    let first = &m.defs[td].steps[0].parts[0];
    let later = if first[0] == b0 { f0 } else { b0 };
    assert!(m.defs[td].mesh.instances[later].hidden);
    assert!(m.assembly.moved.iter().all(|(p, _)| p[0] == tray));
    assert_eq!(at(&m, top), DVec3::new(0.0, 0.0, 718.0));
    m.assemble();
    assert!(!m.defs[td].mesh.instances[later].hidden);
}
