//! The scripting API end to end.

use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{EntityRef, Model};
use varstotas_script::{ScriptCtx, ScriptRun, run};
use std::time::Duration;

fn go(m: &mut Model, src: &str) -> ScriptRun {
    let r = run(m, &ScriptCtx {
            trusted_io: true,
            ..Default::default()
        }, src,
    );
    assert!(r.error.is_none(), "script failed: {:?}\noutput: {}", r.error, r.output);
    r
}

fn size(m: &Model) -> DVec3 {
    let b = m.bounds().unwrap();
    b.max - b.min
}

#[test]
fn vectors_print_and_last_value() {
    let mut m = Model::new();
    let r = go(&mut m, "let a = vec3(1, 2, 3) + X * 2; print(a); a.x * 10 + length(\"1.5m\")");
    assert_eq!(r.output, "vec3(3, 2, 3)\n1530.0\n");
    assert!(!r.changed);
}

#[test]
fn face_pushpull_and_queries() {
    let mut m = Model::new();
    let r = go(
        &mut m,
        r#"
        let f = rect(ORIGIN, 1000, 500);
        let top = pushpull(f, 300);
        print(top.kind + " " + top.area + " " + top.normal);
        print(faces().len() + " " + edges().len());
        bounds(all()).size
    "#,
    );
    assert!(r.changed);
    assert_eq!(r.output.lines().collect::<Vec<_>>(), ["face 500000.0 vec3(0, 0, 1)", "6 12", "vec3(1000, 500, 300)"]);
    assert_eq!(m.root_mesh().faces.len(), 6);
}

#[test]
fn groups_cylinders_transforms_and_context() {
    let mut m = Model::new();
    go(
        &mut m,
        r#"
        let b = box(vec3(0, 0, 0), 600, 400, 18);
        b.name = "Shelf";
        let c = copy(b, vec3(0, 0, 300));
        rotate(c, vec3(0, 0, 309), Z, 90);
        let leg = cylinder(vec3(1000, 0, 0), 20, 700, 16);
        enter(leg);
        let tip = box(vec3(990, -10, 700), 20, 20, 20);
        exit();
        print(groups().len());
    "#,
    );
    let root = m.root_mesh();
    assert_eq!(root.instances.len(), 3);
    assert!(root.instances.values().any(|i| i.name == "Shelf"));
    let s = size(&m);
    assert!((s.z - 720.0).abs() < 1e-6, "{s}");
    // The copy turned 90° about the Z axis through x = y = 0 covers x = -400..0.
    let b = m.bounds().unwrap();
    assert!((b.min.x - -400.0).abs() < 1e-6, "{b:?}");
}

#[test]
fn errors_roll_back_and_report_the_line() {
    let mut m = Model::new();
    let r = run(&mut m, &ScriptCtx::default(), "box(ORIGIN, 1, 1, 1);\nprint(\"before\");\npushpull(42, 10);");
    assert!(r.error.as_deref().unwrap().contains("line 3"), "{:?}", r.error);
    assert_eq!(r.output, "before\n");
    assert!(!r.changed);
    assert!(m.root_mesh().instances.is_empty(), "nothing applied");
    // A face merged away is reported, not a crash.
    let r = run(&mut m, &ScriptCtx::default(), "let f = rect(ORIGIN, 10, 10); erase(f); f.area");
    assert!(r.error.unwrap().contains("no longer exists"));
}

#[test]
fn time_limit_stops_endless_loops() {
    let mut m = Model::new();
    let ctx = ScriptCtx { time_limit: Some(Duration::from_millis(200)), ..Default::default() };
    let r = run(&mut m, &ctx, "loop { }");
    let error = r.error.unwrap();
    assert!(error.contains("time limit") || error.contains("operations"), "{error}");
}

#[test]
fn materials_tags_selection_and_params() {
    let mut m = Model::new();
    let r = go(
        &mut m,
        r##"
        material("Oak", "#C8A064");
        let b = box(ORIGIN, 800, 300, 18);
        paint(b, "Oak");
        tag(b, "Shelves");
        make_parametric(b);
        set_params(b, #{ x: 1200, count: 3, axis: "y", gap: 100 });
        select(b);
        print(b.kind + " " + b.material + " " + b.tag);
        params(b).count
    "##,
    );
    assert_eq!(r.output, "parametric Oak Shelves\n3\n");
    let sel = r.selection.unwrap();
    assert!(matches!(sel.as_slice(), [EntityRef::Instance(_)]));
    let s = size(&m);
    assert!((s - DVec3::new(1200.0, 1100.0, 18.0)).length() < 1e-6, "{s}");
}

#[test]
fn solid_subtract_is_a_minus_b() {
    let mut m = Model::new();
    let r = go(
        &mut m,
        r"
        let a = box(ORIGIN, 1000, 1000, 1000);
        let b = box(vec3(500, 500, 500), 1000, 1000, 1000);
        let out = subtract(a, b);
        bounds(out).size
    ",
    );
    assert_eq!(r.output, "vec3(1000, 1000, 1000)\n");
    assert_eq!(m.root_mesh().instances.len(), 1);
}

#[test]
fn circle_face_and_follow_me() {
    let mut m = Model::new();
    let r = go(
        &mut m,
        r"
        let path = line(vec3(0, 0, 0), vec3(0, 0, 1000));
        let prof = circle(ORIGIN, 50, 12);
        print(prof.kind);
        let top = pushpull(prof, 200);
        top.normal
    ",
    );
    assert_eq!(r.output, "face\nvec3(0, 0, 1)\n");
}

#[test]
fn example_bookshelf_runs() {
    let mut m = Model::new();
    let r = go(&mut m, include_str!("../../../examples/scripts/bookshelf.rhai"));
    assert_eq!(r.output, "Bookshelf 800 x 300 x 1800, 9 parts\n");
    assert_eq!(m.root_mesh().instances.len(), 1);
    let s = size(&m);
    assert!((s - DVec3::new(800.0, 304.0, 1800.0)).length() < 1e-6, "{s}");
}

#[test]
fn docs_example_and_polyline_offset() {
    let mut m = Model::new();
    let r = go(
        &mut m,
        r##"
        material("Oak", "#C8A064");
        let top = box(vec3(0, 0, 720), 1200, 800, 30);
        top.name = "Table top";
        for p in [[50, 50], [1110, 50], [50, 710], [1110, 710]] {
            let leg = box(vec3(p[0], p[1], 0), 40, 40, 720);
            leg.name = "Leg";
            paint(leg, "Oak");
        }
        paint(top, "Oak");
        let hole = cylinder(vec3(600, 400, 700), 30, 60);
        subtract(top, hole);
        print(bounds(groups()).size);
        let pl = polyline([[2000, 0], [2500, 0], [2500, 400], [2000, 400]], true);
        let f = faces()[0];
        print(f.area);
        let inner = offset(f, -50);
        inner.area
    "##,
    );
    assert_eq!(r.output, "vec3(1200, 800, 750)\n200000.0\n120000.0\n");
}

#[test]
fn json_vars_values_and_entity_ids_round_trip() {
    use serde_json::json;
    let mut m = Model::new();
    let vars = |v: serde_json::Value| v.as_object().unwrap().clone();
    let ctx = ScriptCtx::default();
    let r = varstotas_script::eval_json(&mut m, &ctx, "box(a0, a1, a2, a3)", &vars(json!({"a0": [0, 0, 0], "a1": 600, "a2": "40cm", "a3": 18.5})));
    assert!(r.run.error.is_none() && r.run.changed, "{:?}", r.run.error);
    assert_eq!(r.value["kind"], "group");
    let g = r.value;
    // The id from the first call names the same group in the next one.
    let r = varstotas_script::eval_json(&mut m, &ctx, "g.name = \"Shelf\"; [g.name, bounds(g).size, #{ n: 1, ok: true }]", &vars(json!({"g": g})));
    assert!(r.run.error.is_none(), "{:?}", r.run.error);
    assert_eq!(r.value, json!(["Shelf", {"$vec": [600.0, 400.0, 18.5]}, {"n": 1, "ok": true}]));
    let (path, e) = varstotas_script::parse_entity_id(g["$entity"].as_str().unwrap()).unwrap();
    assert!(path.is_empty() && matches!(e, EntityRef::Instance(_)));
    assert_eq!(varstotas_script::entity_id(&path, e), g["$entity"].as_str().unwrap());
    let r = varstotas_script::eval_json(&mut m, &ctx, "x", &vars(json!({"x": {"$entity": "nonsense"}})));
    assert!(r.run.error.unwrap().contains("bad entity id"));
}

#[test]
fn all_faces_and_text_files() {
    let mut m = Model::new();
    let dir = std::env::temp_dir().join(format!("varstotas_api_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("pts.csv");
    std::fs::write(&f, "0,0\n100,0\n100,50\n").unwrap();
    let code = format!(
        r#"
        let pts = [];
        for l in read_text("{p}").split("\n") {{
            l.trim();
            if l == "" {{ continue; }}
            let c = l.split(",");
            pts.push(vec3(parse_float(c[0]), parse_float(c[1]), 0));
        }}
        let f = face(pts);
        box(vec3(500, 0, 0), 10, 10, 10);
        write_text("{p}.out", "" + all_faces().len());
        all_faces().len()
    "#,
        // forward slashes: a Windows path with backslashes is not a valid Rhai string literal
        p = f.to_string_lossy().replace('\\', "/")
    );
    let r = go(&mut m, &code);
    assert_eq!(r.output, "7\n");
    assert_eq!(std::fs::read_to_string(dir.join("pts.csv.out")).unwrap(), "7");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn example_plugins_run() {
    use serde_json::json;
    let ctx = ScriptCtx { trusted_io: true, ..Default::default() };
    let run = |src: &str, vars: serde_json::Value| {
        let mut m = Model::new();
        let r = varstotas_script::eval_json(&mut m, &ctx, src, vars.as_object().unwrap());
        assert!(r.run.error.is_none(), "{:?}", r.run.error);
        (m, r.run.output)
    };
    let p = |x: f64, y: f64, z: f64| json!({"$vec": [x, y, z]});
    let (m, out) = run(
        include_str!("../../../examples/plugins/stairs.rhai"),
        json!({"width": 900.0, "rise": 2700.0, "going": 250.0, "steps": 15, "tread": 40.0, "points": [p(100.0, 0.0, 0.0)]}),
    );
    assert_eq!(out, "Stairs: 15 steps of 180.0 mm\n");
    let s = size(&m);
    assert!((s - DVec3::new(900.0, 14.0 * 250.0 + 270.0, 2700.0 - 180.0 + 40.0)).length() < 1e-6, "{s}");
    // A beam rising 45° towards -X: its bounding box spans both points.
    let (m, _) = run(
        include_str!("../../../examples/plugins/beam.rhai"),
        json!({"w": 100.0, "h": 100.0, "points": [p(1000.0, 0.0, 0.0), p(0.0, 0.0, 1000.0)]}),
    );
    let b = m.bounds().unwrap();
    let r = 50.0 / 2f64.sqrt();
    assert!((b.min - DVec3::new(-r, -50.0, -r)).length() < 1e-5 && (b.max - DVec3::new(1000.0 + r, 50.0, 1000.0 + r)).length() < 1e-5, "{b:?}");
    let f = std::env::temp_dir().join(format!("varstotas_pts_{}.csv", std::process::id()));
    std::fs::write(&f, "x,y\n0,0\n1000, 0\n1000,500\n0,500\n0,0\n").unwrap();
    let (m, out) = run(include_str!("../../../examples/plugins/import_points_csv.rhai"), json!({"path": f.to_string_lossy()}));
    assert_eq!(out, "Imported 4 points\n");
    assert_eq!(m.root_mesh().faces.len(), 1);
    assert!(m.root_mesh().faces.values().all(|f| f.normal.z > 0.99), "the outline faces up");
    let _ = std::fs::remove_file(f);
}

#[test]
fn files_and_json() {
    let d = std::env::temp_dir().join(format!("varstotas_script_files_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("a").join("b")).unwrap();
    std::fs::write(d.join("a").join("x.meta"), "guid: 1").unwrap();
    std::fs::write(d.join("a").join("b").join("y.META"), "").unwrap();
    std::fs::write(d.join("z.txt"), "").unwrap();
    let dir = d.to_string_lossy().replace('\\', "/");
    let mut m = Model::new();
    let src = format!(
        r#"let j = parse_json("{{\"n\": 2, \"list\": [1, \"a\"], \"o\": {{\"k\": true}}}}");
print(j.n + j.list.len());
print(j.o.k);
print(list_files("{dir}", "meta").len());
print(list_files("{dir}", "").len());
print(list_dirs("{dir}").len());
print(file_exists("{dir}/z.txt"));
print(file_exists("{dir}/none"));"#
    );
    let r = go(&mut m, &src);
    assert_eq!(r.output, "4\ntrue\n2\n3\n1\ntrue\nfalse\n");
    assert!(run(&mut m, &ScriptCtx::default(), "parse_json(\"{oops\")").error.is_some());
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn children_by_name_and_object_fields() {
    let mut m = Model::new();
    let r = go(
        &mut m,
        r#"
        let a = box(vec3(0, 0, 0), 100, 200, 300);
        a.name = "Leg";
        let b = box(vec3(500, 0, 0), 10, 10, 10);
        let t = group([a, b]);
        t.name = "Table";
        let out = [];
        out.push(child("Leg") == ());
        out.push(child("Leg", true).size);
        out.push(child(t, "Leg").min);
        out.push(children(t).len());
        out.push(children(true).len());
        hide("Table");
        out.push(child("Table").hidden);
        show(child("Table"), true);
        out.push(t.hidden);
        out.push(child(t, "Nothing", true) == ());
        out
        "#,
    );
    assert!(r.error.is_none(), "{:?}", r.error);
    assert_eq!(r.output.trim(), "[true, vec3(100, 200, 300), vec3(0, 0, 0), 2, 3, true, false, true]");
}

#[test]
fn cut_list_stock_and_options() {
    let mut m = Model::new();
    let r = go(
        &mut m,
        r##"
        material("Plywood", "#d8b98a");
        set_stock("Plywood", #{ kind: "sheet", length: 2440, width: 1220, thickness: 18, grain: true, price: 60 });
        material("Pine", "#e5cf9e");
        set_stock("Pine", [#{ kind: "board", width: 95, thickness: 45, lengths: ["2.4 m", 3000], price: 5 }]);
        for i in 0..3 {
            let s = box(vec3(0, i * 500, 0), 800, 400, 18);
            s.name = "Shelf";
            paint(s, "Plywood");
        }
        let r = box(vec3(0, 2000, 0), 1400, 95, 45);
        r.name = "Rail";
        paint(r, "Pine");
        cut_settings(#{ kerf: 4 });
        cut_part("Rail", #{ grain: "length" });
        let c = cut_list();
        for p in c.parts { print(p.name + " " + p.count + " " + p.length + "x" + p.width + "x" + p.thickness + " " + p.material); }
        for s in c.stock { print(s.material + " " + s.kind + " " + s.count + " " + s.buy[0].length + " " + s.cost); }
        print(c.issues.len() + " " + c.cost);
        "##,
    );
    assert_eq!(
        r.output,
        "Shelf 3 800.0x400.0x18.0 Plywood\nRail 1 1400.0x95.0x45.0 Pine\nPlywood sheet 1 2440.0 60.0\nPine board 1 2400.0 12.0\n0 72.0\n",
        "{}",
        r.output
    );
    assert_eq!(m.cut_list.kerf, 4.0);
    assert!(m.cut_list.parts.is_empty(), "the default options are not stored");
    let ply = m.materials.values().find(|x| x.name == "Plywood").unwrap();
    assert_eq!(ply.stock.len(), 1);
    assert!(ply.stock[0].grain);
    // boards per piece: a 3 m board at 9 holds the 1400 mm rail
    let r = go(&mut m, r##"set_stock("Pine", #{ kind: "board", width: 95, thickness: 45, lengths: [3000, 6000], prices: [9, 16] }); let s = cut_list().stock.filter(|x| x.kind == "board")[0]; print(s.buy[0].length + " " + s.cost + " " + s.prices.len());"##);
    assert_eq!(r.output, "3000.0 9.0 2\n", "{}", r.output);
    // clearing the stock: the parts report it
    go(&mut m, r#"set_stock("Plywood", ()); print(cut_list().issues.len());"#);
    assert!(m.materials.values().find(|x| x.name == "Plywood").unwrap().stock.is_empty());
}
