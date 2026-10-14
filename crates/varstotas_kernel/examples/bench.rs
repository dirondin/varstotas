//! Kernel performance benchmarks (plain `std::time`, no extra deps).
//!
//! Run: `cargo run --release -p varstotas_kernel --example bench [filter]`
//! A filter substring runs only the matching benchmarks (e.g. `add_line`, `import`).

use varstotas_kernel::glam::DVec3;
use varstotas_kernel::*;
use std::fmt::Write as _;
use std::time::{Duration, Instant};

fn v(x: f64, y: f64, z: f64) -> DVec3 {
    DVec3::new(x, y, z)
}

/// N×N grid of unit-size (100 mm) quads on z = 0, built directly (no sticky ops).
fn grid_mesh(n: usize, s: f64) -> Mesh {
    let mut m = Mesh::new();
    let mut vid = Vec::with_capacity((n + 1) * (n + 1));
    for j in 0..=n {
        for i in 0..=n {
            vid.push(m.vertices.insert(Vertex { pos: v(i as f64 * s, j as f64 * s, 0.0) }));
        }
    }
    let at = |i: usize, j: usize| vid[j * (n + 1) + i];
    for j in 0..=n {
        for i in 0..=n {
            if i < n {
                m.edges.insert(Edge::new(at(i, j), at(i + 1, j)));
            }
            if j < n {
                m.edges.insert(Edge::new(at(i, j), at(i, j + 1)));
            }
        }
    }
    for j in 0..n {
        for i in 0..n {
            m.faces.insert(Face {
                outer: vec![at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)],
                holes: Vec::new(),
                normal: DVec3::Z,
                front: None,
                back: None,
                hidden: false,
                tag: None,
            });
        }
    }
    m.rebuild_caches();
    m
}

fn grid_model(n: usize) -> Model {
    let mut model = Model::new();
    *model.root_mesh_mut() = grid_mesh(n, 100.0);
    model
}

/// UV sphere OBJ text with about `2 * seg * seg` triangles.
fn sphere_obj(seg: usize, r: f64) -> String {
    let mut s = String::new();
    let rings = seg;
    let sectors = seg;
    // poles + ring vertices
    let _ = writeln!(s, "v 0 0 {r}");
    for i in 1..rings {
        let th = std::f64::consts::PI * i as f64 / rings as f64;
        for j in 0..sectors {
            let ph = 2.0 * std::f64::consts::PI * j as f64 / sectors as f64;
            let _ = writeln!(s, "v {} {} {}", r * th.sin() * ph.cos(), r * th.sin() * ph.sin(), r * th.cos());
        }
    }
    let _ = writeln!(s, "v 0 0 {}", -r);
    let ring = |i: usize, j: usize| 2 + (i - 1) * sectors + (j % sectors);
    let south = 2 + (rings - 1) * sectors;
    for j in 0..sectors {
        let _ = writeln!(s, "f 1 {} {}", ring(1, j), ring(1, j + 1));
    }
    for i in 1..rings - 1 {
        for j in 0..sectors {
            let (a, b, c, d) = (ring(i, j), ring(i + 1, j), ring(i + 1, j + 1), ring(i, j + 1));
            let _ = writeln!(s, "f {a} {b} {c}");
            let _ = writeln!(s, "f {a} {c} {d}");
        }
    }
    for j in 0..sectors {
        let _ = writeln!(s, "f {south} {} {}", ring(rings - 1, j + 1), ring(rings - 1, j));
    }
    s
}

/// Wavy terrain: N×N quads split into 2 triangles each (non-coplanar).
fn terrain_obj(n: usize) -> String {
    let mut s = String::new();
    for j in 0..=n {
        for i in 0..=n {
            let (x, y) = (i as f64 * 100.0, j as f64 * 100.0);
            let z = 50.0 * (x * 0.013).sin() * (y * 0.017).cos() + 3.0 * ((i * 7 + j * 13) % 5) as f64;
            let _ = writeln!(s, "v {x} {y} {z}");
        }
    }
    let at = |i: usize, j: usize| j * (n + 1) + i + 1;
    for j in 0..n {
        for i in 0..n {
            let _ = writeln!(s, "f {} {} {}", at(i, j), at(i + 1, j), at(i + 1, j + 1));
            let _ = writeln!(s, "f {} {} {}", at(i, j), at(i + 1, j + 1), at(i, j + 1));
        }
    }
    s
}

/// Flat triangulated grid (all coplanar): N×N quads, 2 triangles each.
fn flat_obj(n: usize) -> String {
    let mut s = String::new();
    for j in 0..=n {
        for i in 0..=n {
            let _ = writeln!(s, "v {} {} 0", i as f64 * 100.0, j as f64 * 100.0);
        }
    }
    let at = |i: usize, j: usize| j * (n + 1) + i + 1;
    for j in 0..n {
        for i in 0..n {
            let _ = writeln!(s, "f {} {} {}", at(i, j), at(i + 1, j), at(i + 1, j + 1));
            let _ = writeln!(s, "f {} {} {}", at(i, j), at(i + 1, j + 1), at(i, j + 1));
        }
    }
    s
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

struct Bench {
    filter: Option<String>,
    rows: Vec<(String, f64, String)>,
}

impl Bench {
    fn on(&self, name: &str) -> bool {
        self.filter.as_ref().is_none_or(|f| name.contains(f.as_str()))
    }
    fn report(&mut self, name: &str, t_ms: f64, note: String) {
        println!("{name:<44} {t_ms:>10.3} ms   {note}");
        self.rows.push((name.to_string(), t_ms, note));
    }
}

fn main() {
    let filter = std::env::args().nth(1);
    let mut b = Bench { filter, rows: Vec::new() };
    let tmp = std::env::temp_dir().join(format!("varstotas_bench_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    for n in [100usize, 200] {
        let faces = n * n;
        let tag = format!("{}k", faces / 1000);

        let name = format!("add_line split face ({tag} faces)");
        if b.on(&name) {
            let mut m = grid_mesh(n, 100.0);
            let k = 20;
            let t = Instant::now();
            for r in 0..k {
                // across one cell, midpoint of bottom edge to midpoint of top edge
                let (i, j) = (n / 4 + r * 3 % (n / 2), n / 3 + (r * 7) % (n / 2));
                let x = i as f64 * 100.0 + 50.0;
                let y = j as f64 * 100.0;
                m.add_line(v(x, y, 0.0), v(x, y + 100.0, 0.0));
            }
            let dt = ms(t.elapsed()) / k as f64;
            b.report(&name, dt, format!("avg of {k}, faces now {}", m.faces.len()));
        }

        let name = format!("add_line diagonal ({tag} faces)");
        if b.on(&name) {
            let mut m = grid_mesh(n, 100.0);
            let k = 20;
            let t = Instant::now();
            for r in 0..k {
                let (i, j) = (n / 4 + (r * 5) % (n / 2), n / 5 + (r * 11) % (n / 2));
                let (x, y) = (i as f64 * 100.0, j as f64 * 100.0);
                m.add_line(v(x, y, 0.0), v(x + 100.0, y + 100.0, 0.0));
            }
            let dt = ms(t.elapsed()) / k as f64;
            b.report(&name, dt, format!("avg of {k}, faces now {}", m.faces.len()));
        }

        let name = format!("add_line free (off-plane, {tag} faces)");
        if b.on(&name) {
            let mut m = grid_mesh(n, 100.0);
            let k = 20;
            let t = Instant::now();
            for r in 0..k {
                let x = (r * 37 % n) as f64 * 100.0 + 50.0;
                m.add_line(v(x, 50.0, 0.0), v(x, 50.0, 500.0));
            }
            let dt = ms(t.elapsed()) / k as f64;
            b.report(&name, dt, format!("avg of {k}"));
        }

        let name = format!("push_pull grid cell ({tag} faces)");
        if b.on(&name) {
            let mut m = grid_mesh(n, 100.0);
            let k = 10;
            let mut tot = Duration::ZERO;
            for r in 0..k {
                let x = (n / 4 + r * 4) as f64 * 100.0 + 50.0;
                let y = (n / 3) as f64 * 100.0 + 50.0;
                let ray = Ray { origin: v(x, y, 1000.0), dir: -DVec3::Z };
                let (f, _) = m.raycast_faces(&ray).expect("hit");
                let t = Instant::now();
                m.push_pull(f, 300.0, false).expect("push_pull");
                tot += t.elapsed();
            }
            b.report(&name, ms(tot) / k as f64, format!("avg of {k}, faces now {}", m.faces.len()));
        }

        let name = format!("raycast_faces ({tag} faces)");
        if b.on(&name) {
            let m = grid_mesh(n, 100.0);
            let k = 50;
            let t = Instant::now();
            let mut hits = 0;
            for r in 0..k {
                let x = (r * 13 % n) as f64 * 100.0 + 30.0;
                let ray = Ray { origin: v(x, x * 0.5, 1000.0), dir: v(0.1, 0.2, -1.0).normalize() };
                hits += m.raycast_faces(&ray).is_some() as usize;
            }
            b.report(&name, ms(t.elapsed()) / k as f64, format!("avg of {k}, hits {hits}"));
        }

        let name = format!("tessellate ({tag} faces)");
        if b.on(&name) {
            let model = grid_model(n);
            let _ = tessellate(&model);
            let k = 5;
            let t = Instant::now();
            let mut tris = 0;
            for _ in 0..k {
                tris = tessellate(&model).faces.iter().map(|f| f.positions.len() / 3).sum::<usize>();
            }
            b.report(&name, ms(t.elapsed()) / k as f64, format!("avg of {k}, {tris} triangles"));
        }

        let name = format!("model clone / history snapshot ({tag} faces)");
        if b.on(&name) {
            let model = grid_model(n);
            let k = 10;
            let mut h = History::default();
            let t = Instant::now();
            for i in 0..k {
                h.push(&format!("op{i}"), model.clone());
            }
            b.report(&name, ms(t.elapsed()) / k as f64, format!("avg of {k}"));
        }

        let name = format!("save/load .vrst ({tag} faces)");
        if b.on(&name) {
            let model = grid_model(n);
            let path = tmp.join(format!("grid{n}.vrst"));
            let t = Instant::now();
            io::save(&model, &path).unwrap();
            let ts = ms(t.elapsed());
            let t = Instant::now();
            let m2 = io::load(&path).unwrap();
            let tl = ms(t.elapsed());
            assert_eq!(m2.root_mesh().faces.len(), faces);
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            b.report(&name, ts + tl, format!("save {ts:.1} ms + load {tl:.1} ms, {:.1} MB", size as f64 / 1e6));
        }
    }

    let name = "build 100x100 grid with add_line (202 lines)";
    if b.on(name) {
        let mut m = Mesh::new();
        let n = 100;
        let t = Instant::now();
        for i in 0..=n {
            let x = i as f64 * 10.0;
            m.add_line(v(x, 0., 0.), v(x, 1000., 0.));
        }
        for j in 0..=n {
            let y = j as f64 * 10.0;
            m.add_line(v(0., y, 0.), v(1000., y, 0.));
        }
        let dt = ms(t.elapsed());
        assert_eq!(m.faces.len(), n * n);
        b.report(name, dt, format!("faces {}", m.faces.len()));
    }

    // ---- import --------------------------------------------------------------------
    let name = "import OBJ sphere (~50k tris)";
    if b.on(name) {
        let path = tmp.join("sphere.obj");
        std::fs::write(&path, sphere_obj(158, 5000.0)).unwrap();
        let mut model = Model::new();
        let t = Instant::now();
        io::import_obj(&mut model, &path).unwrap();
        let dt = ms(t.elapsed());
        let m = model.root_mesh();
        b.report(name, dt, format!("faces {}, edges {}, verts {}", m.faces.len(), m.edges.len(), m.vertices.len()));
    }
    let name = "import STL sphere (~50k tris)";
    if b.on(name) {
        let path = tmp.join("sphere.obj");
        std::fs::write(&path, sphere_obj(158, 5000.0)).unwrap();
        let mut src = Model::new();
        io::import_obj(&mut src, &path).unwrap();
        let stl = tmp.join("sphere.stl");
        io::export_stl(&src, &stl).unwrap();
        let mut model = Model::new();
        let t = Instant::now();
        io::import_stl(&mut model, &stl).unwrap();
        let dt = ms(t.elapsed());
        let m = model.root_mesh();
        b.report(name, dt, format!("faces {}, edges {}", m.faces.len(), m.edges.len()));
    }
    let name = "import OBJ terrain (50k tris)";
    if b.on(name) {
        let path = tmp.join("terrain.obj");
        std::fs::write(&path, terrain_obj(158)).unwrap();
        let mut model = Model::new();
        let t = Instant::now();
        io::import_obj(&mut model, &path).unwrap();
        let dt = ms(t.elapsed());
        let m = model.root_mesh();
        b.report(name, dt, format!("faces {}, edges {}", m.faces.len(), m.edges.len()));
    }
    let name = "import OBJ flat coplanar (20k tris)";
    if b.on(name) {
        let path = tmp.join("flat.obj");
        std::fs::write(&path, flat_obj(100)).unwrap();
        let mut model = Model::new();
        let t = Instant::now();
        io::import_obj(&mut model, &path).unwrap();
        let dt = ms(t.elapsed());
        let m = model.root_mesh();
        b.report(name, dt, format!("faces {}, edges {}", m.faces.len(), m.edges.len()));
    }
    let name = "save/load .vrst (sphere 50k)";
    if b.on(name) {
        let path = tmp.join("sphere.obj");
        std::fs::write(&path, sphere_obj(158, 5000.0)).unwrap();
        let mut model = Model::new();
        io::import_obj(&mut model, &path).unwrap();
        let p = tmp.join("sphere.vrst");
        let t = Instant::now();
        io::save(&model, &p).unwrap();
        let ts = ms(t.elapsed());
        let t = Instant::now();
        let _m2 = io::load(&p).unwrap();
        let tl = ms(t.elapsed());
        b.report(name, ts + tl, format!("save {ts:.1} ms + load {tl:.1} ms"));
    }

    let _ = std::fs::remove_dir_all(&tmp);
    println!("\n{} benchmarks", b.rows.len());
}
