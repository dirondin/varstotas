//! Generators (track К): header parsing, building, shared definitions, scaling, the script API.

use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::{GenKind, GenValue, GenValues, Generator, InstanceId, Model};
use varstotas_script::{ScriptCtx, generator, run};

const SHELF: &str = include_str!("../../../examples/generators/shelf.rhai");
const DRAWER: &str = include_str!("../../../examples/generators/drawer_box.rhai");
const STAIR: &str = include_str!("../../../examples/generators/stair.rhai");
const CABINET: &str = include_str!("../../../examples/generators/cabinet.rhai");
const RAILING: &str = include_str!("../../../examples/generators/railing.rhai");
const TABLE: &str = include_str!("../../../examples/generators/table.rhai");

fn vals(kv: &[(&str, GenValue)]) -> GenValues {
    kv.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}

fn def_of(m: &Model, i: InstanceId) -> varstotas_kernel::DefId {
    m.root_mesh().instances[i].def
}

fn size_of(m: &Model, i: InstanceId) -> DVec3 {
    let b = m.def_bounds(def_of(m, i)).unwrap();
    b.max - b.min
}

#[test]
fn header_parsing() {
    let g = Generator::parse(SHELF).unwrap();
    assert_eq!(g.name, "Shelf");
    let w = g.param("width").unwrap();
    assert_eq!((w.kind.clone(), w.min, w.max, w.step), (GenKind::Length, Some(200.0), Some(3000.0), Some(10.0)));
    assert_eq!(w.default, GenValue::Num(800.0));
    assert_eq!(w.label, "Width");
    assert_eq!(g.param("shelves").unwrap().default, GenValue::Num(3.0));
    assert_eq!(g.param("back").unwrap().default, GenValue::Bool(true));
    assert_eq!(g.param("finish").unwrap().kind, GenKind::Choice(vec!["Oak".into(), "Walnut".into(), "White".into()]));
    assert_eq!(g.param("paint_with").unwrap().default, GenValue::Text(String::new()));
    let d = Generator::parse(DRAWER).unwrap();
    let fill = d.param("fill").unwrap();
    assert_eq!((fill.min, fill.max), (Some(0.0), Some(100.0)));
    assert_eq!(d.param("lid").unwrap().kind, GenKind::Angle);
    // Normalizing: clamped, rounded to the step, units, choices by name or index.
    let n = g
        .normalize(&vals(&[
            ("width", GenValue::Text("1.234m".into())),
            ("shelves", GenValue::Num(40.0)),
            ("finish", GenValue::Text("walnut".into())),
        ]))
        .unwrap();
    assert_eq!(n["width"], GenValue::Num(1230.0));
    assert_eq!(n["shelves"], GenValue::Num(12.0));
    assert_eq!(n["finish"], GenValue::Text("Walnut".into()));
    assert_eq!(g.normalize(&vals(&[("finish", GenValue::Num(2.0))])).unwrap()["finish"], GenValue::Text("White".into()));
    assert!(g.normalize(&vals(&[("finish", GenValue::Text("Pine".into()))])).is_err());
    assert!(g.normalize(&vals(&[("nope", GenValue::Num(1.0))])).is_err());
    // Errors in headers.
    assert!(Generator::parse("// @name X\nfn build(p) {}").is_err());
    assert!(Generator::parse("// @generator X\n// @param a:choice() = x\nfn build(p) {}").is_err());
    assert!(Generator::parse("// @generator X\n// @param a:length(5..1) = 3\nfn build(p) {}").is_err());
    assert!(Generator::parse("// @generator X\nlet a = 1;").is_err());
}

#[test]
fn equal_values_share_a_definition() {
    let mut m = Model::new();
    generator::add(&mut m, SHELF).unwrap();
    let none = GenValues::new();
    let a = generator::place(&mut m, &[], "Shelf", &none, DMat4::IDENTITY).unwrap();
    let b = generator::place(&mut m, &[], "Shelf", &none, DMat4::from_translation(DVec3::X * 2000.0)).unwrap();
    assert_eq!(def_of(&m, a), def_of(&m, b));
    let s = size_of(&m, a);
    assert!((s - DVec3::new(800.0, 300.0, 1200.0)).length() < 1e-6, "{s}");
    // Shelves: 2 sides + bottom + top + 3 shelves + back.
    assert_eq!(m.defs[def_of(&m, a)].mesh.instances.len(), 8);
    let wood = m.materials.values().filter(|x| x.name == "Oak").count();
    assert_eq!(wood, 1);

    // Changing one moves it to a new definition; the other keeps the old one.
    let old = def_of(&m, a);
    generator::set_values(&mut m, &[], b, &vals(&[("width", GenValue::Num(1000.0))]), false).unwrap();
    assert_ne!(def_of(&m, b), old);
    assert_eq!(def_of(&m, a), old);
    assert!((size_of(&m, b).x - 1000.0).abs() < 1e-6);
    // Back to the same values: shared again, the unused definition is gone.
    let defs = m.defs.len();
    generator::set_values(&mut m, &[], b, &vals(&[("width", GenValue::Num(800.0))]), false).unwrap();
    assert_eq!(def_of(&m, b), old);
    assert!(m.defs.len() < defs);
    // A single user is rebuilt in place.
    generator::set_values(&mut m, &[], a, &vals(&[("shelves", GenValue::Num(1.0)), ("back", GenValue::Bool(false))]), false).unwrap();
    let d = def_of(&m, a);
    assert_eq!(m.defs[d].mesh.instances.len(), 5);
    generator::set_values(&mut m, &[], a, &vals(&[("shelves", GenValue::Num(2.0))]), false).unwrap();
    assert_eq!(def_of(&m, a), d, "the only user: rebuilt in place");
    assert_eq!(m.defs[d].mesh.instances.len(), 6);
    // Reset: the defaults, shared with b again.
    generator::set_values(&mut m, &[], a, &GenValues::new(), true).unwrap();
    assert_eq!(def_of(&m, a), def_of(&m, b));
    // Bad values change nothing.
    assert!(generator::set_values(&mut m, &[], a, &vals(&[("finish", GenValue::Text("Pine".into()))]), false).is_err());
    assert_eq!(def_of(&m, a), def_of(&m, b));
}

#[test]
fn script_errors_are_reported() {
    let mut m = Model::new();
    generator::add(&mut m, "// @generator Bad\n// @param n:int = 1\nfn build(p) { box(ORIGIN, 1, 1, 1); throw \"boom\"; }").unwrap();
    let e = generator::place(&mut m, &[], "Bad", &GenValues::new(), DMat4::IDENTITY).unwrap_err();
    assert!(e.contains("boom"), "{e}");
    assert!(m.root_mesh().instances.is_empty());
}

#[test]
fn scale_changes_size_parameters() {
    let mut m = Model::new();
    generator::add(&mut m, SHELF).unwrap();
    let at = DMat4::from_translation(DVec3::new(100.0, 0.0, 0.0));
    let i = generator::place(&mut m, &[], "Shelf", &GenValues::new(), at).unwrap();
    // Twice as wide about x = 100 (the left side stays), height × 0.5.
    let xf = DMat4::from_translation(DVec3::new(100.0, 0.0, 0.0))
        * DMat4::from_scale(DVec3::new(2.0, 1.0, 0.5))
        * DMat4::from_translation(DVec3::new(-100.0, 0.0, 0.0));
    generator::scale(&mut m, &[], i, &xf).unwrap().unwrap();
    let g = m.instance_generated(&[], i).unwrap();
    assert_eq!(g.values["width"], GenValue::Num(1600.0));
    assert_eq!(g.values["height"], GenValue::Num(600.0));
    assert_eq!(g.values["board"], GenValue::Num(18.0), "a board is not a size");
    let t = m.root_mesh().instances[i].transform;
    assert!((t.w_axis.truncate() - DVec3::new(100.0, 0.0, 0.0)).length() < 1e-9);
    assert!((t.x_axis.truncate() - DVec3::X).length() < 1e-9, "no stretch in the transform");
    // A rotated instance: its width runs along world Y.
    let r = DMat4::from_translation(DVec3::new(0.0, 5000.0, 0.0)) * DMat4::from_rotation_z(std::f64::consts::FRAC_PI_2);
    let j = generator::place(&mut m, &[], "Shelf", &GenValues::new(), r).unwrap();
    let ys = DMat4::from_scale(DVec3::new(1.0, 1.5, 1.0));
    generator::scale(&mut m, &[], j, &ys).unwrap().unwrap();
    let g = m.instance_generated(&[], j).unwrap();
    assert_eq!((g.values["width"].clone(), g.values["depth"].clone()), (GenValue::Num(1200.0), GenValue::Num(300.0)));
    // A generator without size parameters is scaled as usual.
    generator::add(&mut m, "// @generator Plain\n// @param n:int = 1\nfn build(p) { box(ORIGIN, 10, 10, 10); }").unwrap();
    let k = generator::place(&mut m, &[], "Plain", &GenValues::new(), DMat4::IDENTITY).unwrap();
    assert!(generator::scale(&mut m, &[], k, &ys).is_none());
}

#[test]
fn script_api_and_file_round_trip() {
    let mut m = Model::new();
    let src = format!(
        "let n = add_generator({SHELF:?});\n\
         let a = generate(n, vec3(0, 0, 0));\n\
         let b = generate(n, vec3(2000, 0, 0), #{{ width: \"1m\", finish: \"White\" }});\n\
         set_params(a, #{{ shelves: 5 }});\n\
         print(params(a).shelves); print(params(b).width); print(params(b).finish); print(generators());\n\
         reset_params(a); print(params(a).shelves);"
    );
    let r = run(&mut m, &ScriptCtx::default(), &src);
    assert!(r.error.is_none(), "{:?}", r.error);
    assert_eq!(r.output, "5\n1000.0\nWhite\n[\"Shelf\"]\n3\n");
    let ids: Vec<InstanceId> = m.root_mesh().instances.keys().collect();
    assert_eq!(ids.len(), 2);
    // Saved and loaded: the source and the values come back.
    let path = std::env::temp_dir().join(format!("varstotas_gen_{}.vrst", std::process::id()));
    varstotas_kernel::io::save(&m, &path).unwrap();
    let mut l = varstotas_kernel::io::load(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(l.generators["Shelf"].source, SHELF);
    let b = ids.iter().copied().find(|i| l.instance_generated(&[], *i).is_some_and(|g| g.values["finish"] == GenValue::Text("White".into())));
    let b = b.expect("b survives");
    generator::set_values(&mut l, &[], b, &vals(&[("finish", GenValue::Text("Oak".into()))]), false).unwrap();
    assert!(l.root_mesh().instances.contains_key(b));
}

#[test]
fn every_parameter_type_builds() {
    let mut m = Model::new();
    m.add_material("Red", [1.0, 0.0, 0.0, 1.0]);
    generator::add(&mut m, DRAWER).unwrap();
    for (hinge, lid) in [("back", 30.0), ("left", 90.0), ("right", 0.0)] {
        let v = vals(&[
            ("hinge", GenValue::Text(hinge.into())),
            ("lid", GenValue::Num(lid)),
            ("fill", GenValue::Num(50.0)),
            ("color", GenValue::Text("Red".into())),
        ]);
        let i = generator::place(&mut m, &[], "Drawer Box", &v, DMat4::IDENTITY).unwrap();
        let s = size_of(&m, i);
        assert!(s.z > 200.0, "{hinge}: the lid stands up: {s}");
        assert_eq!(m.defs[def_of(&m, i)].mesh.instances.len(), 7);
    }
    assert_eq!(m.materials.values().filter(|x| x.name == "Red").count(), 1);
}

#[test]
fn formulas_and_checks() {
    let g = Generator::parse(STAIR).unwrap();
    assert_eq!(g.formulas.iter().map(|f| f.key.as_str()).collect::<Vec<_>>(), ["count", "riser", "first", "run"]);
    assert_eq!(g.formulas[0].kind, GenKind::Int);
    assert_eq!(g.formulas[1].label, "Riser height");
    assert_eq!(g.checks.len(), 2);
    assert!(g.checks[0].expr.ends_with("count >= 6"), "{:?}", g.checks[0]);
    // 2800 / 180 → 16 risers of 175 mm.
    let f = generator::formula_values(&g, &g.defaults()).unwrap();
    assert_eq!(f[0].1, GenValue::Num(16.0));
    assert_eq!(f[1].1, GenValue::Num(175.0));
    assert_eq!(f[3].1, GenValue::Num(15.0 * 270.0));
    // A label is not taken from a compared string; formulas see earlier formulas.
    let src = "// @generator F\n// @param kind:choice(a|b) = a\n// @formula is_a:bool = kind == \"a\"\n// @formula twice = if is_a { 2 } else { 3 } \"Twice\"\nfn build(p) { box(ORIGIN, p.twice, 1, 1); }";
    let h = Generator::parse(src).unwrap();
    assert_eq!(h.formulas[0].expr, "kind == \"a\"");
    assert_eq!(h.formulas[1].label, "Twice");
    let v = generator::formula_values(&h, &h.defaults()).unwrap();
    assert_eq!((v[0].1.clone(), v[1].1.clone()), (GenValue::Bool(true), GenValue::Num(2.0)));
    // Names are shared with parameters; bad types are refused.
    assert!(Generator::parse("// @generator X\n// @param a = 1\n// @formula a = 2\nfn build(p) {}").is_err());
    assert!(Generator::parse("// @generator X\n// @formula a:choice = 2\nfn build(p) {}").is_err());

    // A failing check refuses the values and changes nothing.
    let mut m = Model::new();
    generator::add(&mut m, STAIR).unwrap();
    let i = generator::place(&mut m, &[], "Stair", &GenValues::new(), DMat4::IDENTITY).unwrap();
    let s = size_of(&m, i);
    assert!((s.z - 2800.0).abs() < 60.0 && s.x > 3900.0, "{s}");
    let e = generator::set_values(&mut m, &[], i, &vals(&[("going", GenValue::Num(350.0)), ("max_riser", GenValue::Num(220.0))]), false).unwrap_err();
    assert!(e.contains("Uncomfortable"), "{e}");
    assert_eq!(m.instance_generated(&[], i).unwrap().values["going"], GenValue::Num(270.0));
    let e = generator::set_values(&mut m, &[], i, &vals(&[("layout", GenValue::Text("landing".into())), ("height", GenValue::Num(800.0))]), false)
        .unwrap_err();
    assert!(e.contains("at least 6"), "{e}");
    // The formula reaches build: a landing stair is longer by the landing.
    generator::set_values(&mut m, &[], i, &vals(&[("layout", GenValue::Text("landing".into()))]), false).unwrap();
    let l = size_of(&m, i);
    assert!(l.x > s.x + 500.0, "{l} vs {s}");
}

#[test]
fn bundled_generators_build() {
    let cases: &[(&str, &[(&str, GenValue)])] = &[
        (SHELF, &[("shelves", GenValue::Num(6.0))]),
        (STAIR, &[("risers", GenValue::Bool(false)), ("height", GenValue::Num(1200.0))]),
        (CABINET, &[("doors", GenValue::Num(1.0)), ("hinge", GenValue::Text("right".into())), ("open", GenValue::Num(60.0)), ("drawers", GenValue::Num(2.0))]),
        (CABINET, &[("doors", GenValue::Num(3.0)), ("open", GenValue::Num(90.0)), ("handles", GenValue::Bool(false))]),
        (RAILING, &[("infill", GenValue::Text("bars".into())), ("length", GenValue::Num(5200.0))]),
        (RAILING, &[("infill", GenValue::Text("none".into())), ("bottom_rail", GenValue::Bool(false))]),
        (TABLE, &[("legs", GenValue::Text("round".into())), ("apron", GenValue::Bool(false))]),
    ];
    for (src, v) in cases {
        let mut m = Model::new();
        let name = generator::add(&mut m, src).unwrap();
        let a = generator::place(&mut m, &[], &name, &GenValues::new(), DMat4::IDENTITY).unwrap_or_else(|e| panic!("{name}: {e}"));
        let b = generator::place(&mut m, &[], &name, &vals(v), DMat4::IDENTITY).unwrap_or_else(|e| panic!("{name} {v:?}: {e}"));
        for i in [a, b] {
            let s = size_of(&m, i);
            assert!(s.x > 100.0 && s.y > 30.0 && s.z > 100.0, "{name}: {s}");
        }
    }
    // Sizes follow the parameters.
    let mut m = Model::new();
    generator::add(&mut m, TABLE).unwrap();
    let t = generator::place(&mut m, &[], "Table", &GenValues::new(), DMat4::IDENTITY).unwrap();
    assert!((size_of(&m, t) - DVec3::new(1600.0, 900.0, 750.0)).length() < 1e-6);
    generator::add(&mut m, RAILING).unwrap();
    let r = generator::place(&mut m, &[], "Railing", &GenValues::new(), DMat4::IDENTITY).unwrap();
    let g = m.instance_generated(&[], r).unwrap().clone();
    let f = generator::formula_values(&m.generators["Railing"], &g.values).unwrap();
    assert_eq!(f[0].1, GenValue::Num(3.0), "3000 / 1500 → 3 posts");
}
