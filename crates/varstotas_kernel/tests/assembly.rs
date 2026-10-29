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
    m.assembly.steps = suggest_steps(&m);
    let before: Vec<DVec3> = [bottom, left, right, top, shelf].iter().map(|&i| at(&m, i)).collect();
    m.set_assembly_view(SceneAssembly { explode: 100.0, step: None });
    // sides apart, bottom down, top up, the shelf out to the front; all lifted by 100 so
    // that the bottom stays on the floor
    assert_eq!(at(&m, left) - before[1], DVec3::new(-100.0, 0.0, 100.0));
    assert_eq!(at(&m, right) - before[2], DVec3::new(100.0, 0.0, 100.0));
    assert_eq!(at(&m, bottom) - before[0], DVec3::ZERO);
    assert_eq!(at(&m, top) - before[3], DVec3::new(0.0, 0.0, 200.0));
    assert_eq!(at(&m, shelf) - before[4], DVec3::new(0.0, -100.0, 100.0));
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
