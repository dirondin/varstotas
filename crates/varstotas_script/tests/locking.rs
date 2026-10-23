use varstotas_kernel::{Model, io};
use varstotas_script::{ScriptCtx, run};

fn fixture() -> Model {
    let mut m = Model::new();
    let r = run(
        &mut m,
        &ScriptCtx::default(),
        "let b = box(ORIGIN, 100, 50, 30); b.name = \"Protected\"; paint(b, material(\"Oak\", [0.5,0.3,0.2]));",
    );
    assert!(r.error.is_none(), "{:?}", r.error);
    m.root_mesh_mut()
        .instances
        .values_mut()
        .next()
        .unwrap()
        .locked = true;
    m
}

#[test]
fn trusted_plugins_and_untrusted_bridge_cannot_change_locked_objects() {
    let original = fixture();
    for trusted_io in [false, true] {
        for src in [
            "move(groups()[0], X * 100);",
            "erase(groups()[0]);",
            "enter(groups()[0]); pushpull(faces()[0], 10);",
            "enter(groups()[0]); rect(vec3(500,500,0), 20, 20);",
            "let b = groups()[0]; b.name = \"Changed\";",
            "tag(groups()[0], \"Other\");",
            "paint(groups()[0], ());",
            "material(\"Oak\", [1.0,0.0,0.0]);",
            "box(vec3(500,500,0), 10, 10, 10); erase(groups()[0]);",
        ] {
            let mut m = original.clone();
            let r = run(
                &mut m,
                &ScriptCtx {
                    trusted_io,
                    ..Default::default()
                },
                src,
            );
            assert!(
                r.error.as_deref().is_some_and(|e| e.contains("locked")),
                "unexpected result {src} trusted={trusted_io}: {:?}",
                r.error
            );
            assert!(!r.changed);
            assert_eq!(
                io::to_json(&m).unwrap(),
                io::to_json(&original).unwrap(),
                "{src}"
            );
        }
    }
}

#[test]
fn locked_objects_are_readable_but_not_selectable_and_unrelated_geometry_is_editable() {
    let mut m = fixture();
    let r = run(
        &mut m,
        &ScriptCtx::default(),
        "let b = groups()[0]; if !b.locked { throw \"lock query\"; } print(bounds(b).size); select(b);",
    );
    assert!(r.error.is_none(), "{:?}", r.error);
    assert!(r.selection.unwrap().is_empty());
    let r = run(
        &mut m,
        &ScriptCtx::default(),
        "box(vec3(500,500,0), 10, 10, 10);",
    );
    assert!(r.error.is_none(), "{:?}", r.error);
    assert!(r.changed);
    assert_eq!(m.root_mesh().instances.len(), 2);
}
