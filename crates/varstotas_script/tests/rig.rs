//! Rigs: parameters of ordinary components (joints, formulas, the script, shared variants,
//! edits written back to the master, removing the rig).

use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{DefId, GenFormula, GenParam, GenValue, GenValues, InstanceId, Joint, JointKind, Model, Rig};
use varstotas_script::{ScriptCtx, rig, run};

fn vals(kv: &[(&str, GenValue)]) -> GenValues {
    kv.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}

/// A cabinet component (body + door, the door's hinge on the left edge at x = 0, y = -20) and
/// two copies of it at the top level.
fn cabinet() -> (Model, DefId, InstanceId, InstanceId) {
    let mut m = Model::new();
    let src = r#"
        let body = box(vec3(0, 0, 0), 600, 400, 800);
        let door = box(vec3(0, -20, 0), 600, 20, 800);
        door.name = "Door";
        let c = component([body, door], "Cabinet");
        copy(c, vec3(1000, 0, 0));
    "#;
    let r = run(&mut m, &ScriptCtx::default(), src);
    assert!(r.error.is_none(), "{:?}", r.error);
    material_named(&mut m, "Red");
    let ids: Vec<InstanceId> = m.root_mesh().instances.keys().collect();
    let d = m.root_mesh().instances[ids[0]].def;
    (m, d, ids[0], ids[1])
}

fn material_named(m: &mut Model, name: &str) {
    let r = run(m, &ScriptCtx::default(), &format!("material(\"{name}\", \"#cc2222\")"));
    assert!(r.error.is_none(), "{:?}", r.error);
}

fn door_rig() -> Rig {
    Rig {
        params: vec![GenParam::parse("open:angle(0..90/1) = 0 \"Opening\"").unwrap(), GenParam::parse("handle:bool = true").unwrap()],
        formulas: vec![GenFormula::parse("half = open / 2").unwrap()],
        joints: vec![Joint {
            part: "Door".into(),
            kind: JointKind::Rotate { origin: DVec3::new(0.0, -20.0, 0.0), axis: DVec3::Z },
            param: "open".into(),
            from: 0.0,
            to: -90.0,
            expr: String::new(),
        }],
        script: String::new(),
    }
}

fn def_of(m: &Model, i: InstanceId) -> DefId {
    m.root_mesh().instances[i].def
}

/// World-ish bounds (in the component's axes) of the part named `name` of definition `d`.
fn part_bounds(m: &Model, d: DefId, name: &str) -> varstotas_kernel::Aabb {
    let i = m.defs[d].mesh.instances.values().find(|i| i.name == name).expect("the part");
    m.def_bounds(i.def).unwrap().transformed(&i.transform)
}

fn close(a: DVec3, b: DVec3) -> bool {
    a.distance(b) < 1e-6
}

#[test]
fn joints_pose_shared_variants() {
    let (mut m, d, a, b) = cabinet();
    let master = rig::set_rig(&mut m, d, Some(door_rig())).unwrap();
    assert_eq!(master, d);
    // Unnamed parts got names, the copies moved to the variant for the defaults.
    let names: Vec<String> = m.rig_parts(d).into_iter().map(|(_, n)| n).collect();
    assert_eq!(names, vec!["Door".to_string(), "Group".to_string()]);
    let v0 = def_of(&m, a);
    assert_ne!(v0, d);
    assert_eq!(def_of(&m, b), v0);
    assert_eq!(m.defs[v0].rigged.as_ref().unwrap().values["open"], GenValue::Num(0.0));
    // Open the first copy: its own variant, the door turned about the hinge (-90° about Z:
    // the door's far end swings to -y).
    rig::set_values(&mut m, &[], a, &vals(&[("open", GenValue::Num(90.0))]), false).unwrap();
    let v1 = def_of(&m, a);
    assert_ne!(v1, v0);
    let bb = part_bounds(&m, v1, "Door");
    assert!(close(bb.min, DVec3::new(0.0, -620.0, 0.0)) && close(bb.max, DVec3::new(20.0, -20.0, 800.0)), "{bb:?}");
    // The master keeps the rest pose; the other copy is closed.
    assert!(close(part_bounds(&m, d, "Door").max, DVec3::new(600.0, 0.0, 800.0)));
    assert!(close(part_bounds(&m, def_of(&m, b), "Door").max, DVec3::new(600.0, 0.0, 800.0)));
    // Half open (mapped linearly onto 0..-90).
    rig::set_values(&mut m, &[], b, &vals(&[("open", GenValue::Num(45.0))]), false).unwrap();
    // Both open: one variant shared, the 45° one posed in place before.
    rig::set_values(&mut m, &[], b, &vals(&[("open", GenValue::Num(90.0))]), false).unwrap();
    assert_eq!(def_of(&m, b), v1);
    assert_eq!(m.rig_variants(d).len(), 1);
    // Values are clamped to the range; formulas derived.
    rig::set_values(&mut m, &[], a, &vals(&[("open", GenValue::Num(300.0))]), false).unwrap();
    assert_eq!(m.instance_rigged(&[], a).unwrap().1.values["open"], GenValue::Num(90.0));
    let g = rig::generator(&m, d).unwrap();
    let f = varstotas_script::generator::formula_values(&g, &m.instance_rigged(&[], a).unwrap().1.values).unwrap();
    assert_eq!(f[0].1, GenValue::Num(45.0));
    // Saved and loaded: the rig and the variants come back.
    let json = serde_json::to_string(&m).unwrap();
    let back: Model = serde_json::from_str(&json).unwrap();
    assert_eq!(back.defs[d].rig, m.defs[d].rig);
    assert_eq!(back.instance_rigged(&[], a).unwrap().1.values, m.instance_rigged(&[], a).unwrap().1.values);
}

#[test]
fn edits_in_a_variant_go_to_the_master() {
    let (mut m, d, a, b) = cabinet();
    rig::set_rig(&mut m, d, Some(door_rig())).unwrap();
    rig::set_values(&mut m, &[], a, &vals(&[("open", GenValue::Num(90.0))]), false).unwrap();
    let va = def_of(&m, a);
    // Inside the open copy: move the door 100 up (in the variant's axes), then the edit hook.
    let door = m.defs[va].mesh.instances.iter().find(|(_, i)| i.name == "Door").map(|(id, _)| id).unwrap();
    m.defs[va].mesh.instances[door].transform = varstotas_kernel::glam::DMat4::from_translation(DVec3::new(0.0, 0.0, 100.0)) * m.defs[va].mesh.instances[door].transform;
    // And add a new part (a shelf) there.
    let r = run(&mut m, &ScriptCtx { path: vec![a], ..Default::default() }, "let s = box(vec3(0, 0, 400), 600, 400, 18); s.name = \"Shelf\";");
    assert!(r.error.is_none(), "{:?}", r.error);
    rig::after_edit(&mut m, &[a]).unwrap();
    // The master: the door closed again but 100 higher, the shelf there.
    let md = part_bounds(&m, d, "Door");
    assert!(close(md.min, DVec3::new(0.0, -20.0, 100.0)), "{md:?}");
    assert!(m.rig_parts(d).iter().any(|(_, n)| n == "Shelf"));
    // The closed copy shows both too; the open one keeps its pose.
    let vb = def_of(&m, b);
    assert!(close(part_bounds(&m, vb, "Door").min, DVec3::new(0.0, -20.0, 100.0)));
    assert!(m.rig_parts(vb).iter().any(|(_, n)| n == "Shelf"));
    assert!(close(part_bounds(&m, def_of(&m, a), "Door").min, DVec3::new(0.0, -620.0, 100.0)));
}

#[test]
fn script_visibility_and_paint() {
    let (mut m, d, a, _) = cabinet();
    let mut r = door_rig();
    r.params.push(GenParam::parse("color:material = \"\" \"Colour\"").unwrap());
    r.joints.push(Joint { part: "Door".into(), kind: JointKind::Material, param: "color".into(), from: 0.0, to: 0.0, expr: String::new() });
    r.joints.push(Joint { part: "Group".into(), kind: JointKind::Visible, param: String::new(), from: 0.0, to: 0.0, expr: "open < 80".into() });
    r.script = "fn update(p, parts) { if p.handle { move(\"Door\", vec3(0, 0, parts[\"Door\"].size.z / 8)); } }".into();
    rig::set_rig(&mut m, d, Some(r)).unwrap();
    // Defaults: handle on → the script lifts the door by 100.
    assert!(close(part_bounds(&m, def_of(&m, a), "Door").min, DVec3::new(0.0, -20.0, 100.0)));
    rig::set_values(&mut m, &[], a, &vals(&[("handle", GenValue::Bool(false)), ("color", GenValue::Text("Red".into())), ("open", GenValue::Num(85.0))]), false).unwrap();
    let v = def_of(&m, a);
    let door = m.defs[v].mesh.instances.values().find(|i| i.name == "Door").unwrap();
    let painted = &m.defs[door.def];
    let red = m.materials.iter().find(|(_, x)| x.name == "Red").map(|(id, _)| id);
    assert!(painted.rigged.as_ref().is_some_and(|r| r.paint.as_deref() == Some("Red")));
    assert!(painted.mesh.faces.values().all(|f| f.front == red));
    assert!(m.defs[v].mesh.instances.values().find(|i| i.name == "Group").unwrap().hidden);
    // The master's door is untouched.
    let md = m.defs[d].mesh.instances.values().find(|i| i.name == "Door").unwrap();
    assert!(m.defs[md.def].rigged.is_none());
    // A broken script is refused and the rig stays.
    let mut bad = m.defs[d].rig.clone().unwrap();
    bad.script = "fn update(p, parts) { rotate(\"Door\", 1); }".into();
    assert!(rig::set_rig(&mut m, d, Some(bad)).is_err());
    assert!(m.defs[d].rig.as_ref().unwrap().script.contains("handle"));
}

#[test]
fn remove_rig_and_attach() {
    let (mut m, d, a, b) = cabinet();
    rig::set_rig(&mut m, d, Some(door_rig())).unwrap();
    rig::set_values(&mut m, &[], a, &vals(&[("open", GenValue::Num(30.0))]), false).unwrap();
    // A copy placed with the master itself (paste, Components card) gets the default variant.
    let t = m.root_mesh().instances[a].transform;
    let c = m.root_mesh_mut().instances.insert(varstotas_kernel::Instance { def: d, transform: t, name: String::new(), material: None, tag: None, hidden: false, locked: false });
    rig::attach(&mut m);
    assert_eq!(def_of(&m, c), def_of(&m, b));
    // Removing the rig: the copies use the master again, the variants are gone.
    rig::set_rig(&mut m, d, None).unwrap();
    assert!(m.defs[d].rig.is_none());
    assert!([a, b, c].iter().all(|i| def_of(&m, *i) == d));
    assert!(m.defs.values().all(|x| x.rigged.is_none()));
    // Groups can't have a rig.
    let r = run(&mut m, &ScriptCtx::default(), "box(vec3(0, 0, 0), 10, 10, 10);");
    assert!(r.error.is_none());
    let g = m.root_mesh().instances.values().find(|i| m.defs[i.def].is_group).unwrap().def;
    assert!(rig::set_rig(&mut m, g, Some(door_rig())).is_err());
}
