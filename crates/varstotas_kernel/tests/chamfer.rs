//! Chamfer / Fillet of edges.

use varstotas_kernel::glam::DVec3;
use varstotas_kernel::{ChamferKind, EdgeId, Mesh};

fn cube(size: f64) -> Mesh {
    let mut m = Mesh::new();
    let s = size;
    let f = m
        .add_face(&[DVec3::ZERO, DVec3::new(s, 0.0, 0.0), DVec3::new(s, s, 0.0), DVec3::new(0.0, s, 0.0)])
        .unwrap();
    // Ground faces look down: pulling "back" raises the box.
    m.push_pull(f, -s, false).unwrap();
    m
}

fn edge_at(m: &Mesh, a: DVec3, b: DVec3) -> EdgeId {
    m.edges
        .iter()
        .find(|(_, e)| {
            let (p, q) = (m.vertices[e.v[0]].pos, m.vertices[e.v[1]].pos);
            (p.distance(a) < 1e-6 && q.distance(b) < 1e-6) || (p.distance(b) < 1e-6 && q.distance(a) < 1e-6)
        })
        .map(|(id, _)| id)
        .expect("edge")
}

fn volume(m: &Mesh) -> f64 {
    // Divergence theorem over the triangulated faces (outward normals).
    let mut v = 0.0;
    for f in m.faces.keys() {
        for t in varstotas_kernel::tessellate::triangulate_face(m, f) {
            v += t[0].dot(t[1].cross(t[2])) / 6.0;
        }
    }
    v
}

#[test]
fn flat_chamfer_of_a_cube_edge() {
    let mut m = cube(1000.0);
    let z = m.vertices.values().map(|v| v.pos.z).fold(f64::MIN, f64::max);
    assert!((z - 1000.0).abs() < 1e-9);
    let e = edge_at(&m, DVec3::new(0.0, 0.0, 1000.0), DVec3::new(1000.0, 0.0, 1000.0));
    let r = m.chamfer_edge(e, ChamferKind::Flat { distance: 100.0 }).unwrap();
    m.validate().unwrap();
    assert_eq!(r.faces.len(), 1);
    assert_eq!(m.faces.len(), 7);
    assert_eq!(m.edges.len(), 15);
    assert_eq!(m.vertices.len(), 10);
    let n = m.faces[r.faces[0]].normal;
    assert!((n - DVec3::new(0.0, -1.0, 1.0).normalize()).length() < 1e-9, "{n}");
    assert!((volume(&m) - (1e9 - 0.5 * 100.0 * 100.0 * 1000.0)).abs() < 1.0, "{}", volume(&m));
    // The end faces became pentagons.
    assert_eq!(m.faces.values().filter(|f| f.outer.len() == 5).count(), 2);
}

#[test]
fn round_fillet_of_a_cube_edge() {
    let mut m = cube(1000.0);
    let e = edge_at(&m, DVec3::new(0.0, 0.0, 1000.0), DVec3::new(1000.0, 0.0, 1000.0));
    let r = m.chamfer_edge(e, ChamferKind::Round { radius: 200.0, segments: 8 }).unwrap();
    m.validate().unwrap();
    assert_eq!(r.faces.len(), 8);
    // Points on the arc are 200 from the axis (y = 200, z = 800).
    for f in &r.faces {
        for v in &m.faces[*f].outer {
            let p = m.vertices[*v].pos;
            let d = DVec3::new(0.0, p.y - 200.0, p.z - 800.0).length();
            assert!((d - 200.0).abs() < 1e-6, "{p}");
        }
    }
    let quarter = 200.0 * 200.0 * (1.0 - std::f64::consts::FRAC_PI_4);
    // 8 segments are a polygon inside the arc: a bit more material removed than the exact round.
    let removed = 1e9 - volume(&m);
    assert!(removed > quarter * 1000.0 && removed < quarter * 1000.0 * 1.15, "{removed}");
    let soft = m.edges.values().filter(|e| e.soft && e.smooth).count();
    assert_eq!(soft, 9, "arc lines and both tangent lines");
}

#[test]
fn concave_edge_gets_filled_and_bad_sizes_fail() {
    // L-shaped profile extruded along Y: the inner corner edge is concave.
    let mut m = Mesh::new();
    let pts = [
        DVec3::new(0.0, 0.0, 0.0),
        DVec3::new(0.0, 0.0, 1000.0),
        DVec3::new(300.0, 0.0, 1000.0),
        DVec3::new(300.0, 0.0, 300.0),
        DVec3::new(1000.0, 0.0, 300.0),
        DVec3::new(1000.0, 0.0, 0.0),
    ];
    let f = m.add_face(&pts).unwrap();
    let n = m.faces[f].normal;
    m.push_pull(f, if n.y < 0.0 { -500.0 } else { 500.0 }, false).unwrap();
    m.validate().unwrap();
    let before = volume(&m).abs();
    let e = edge_at(&m, DVec3::new(300.0, 0.0, 300.0), DVec3::new(300.0, 500.0, 300.0));
    assert!(m.chamfer_edge(e, ChamferKind::Flat { distance: 800.0 }).is_err(), "longer than the faces");
    assert_eq!(volume(&m).abs(), before, "a failed chamfer changes nothing");
    m.chamfer_edge(e, ChamferKind::Flat { distance: 100.0 }).unwrap();
    m.validate().unwrap();
    let added = volume(&m).abs() - before;
    assert!((added - 0.5 * 100.0 * 100.0 * 500.0).abs() < 1.0, "{added}");
}

#[test]
fn chamfer_follows_into_the_next_edge() {
    let mut m = cube(1000.0);
    let e = edge_at(&m, DVec3::new(0.0, 0.0, 1000.0), DVec3::new(1000.0, 0.0, 1000.0));
    let r = m.chamfer_edge(e, ChamferKind::Flat { distance: 100.0 }).unwrap();
    // The top-right edge was shortened; chamfer its replacement too.
    let next = r
        .replaced
        .iter()
        .map(|(_, n)| *n)
        .find(|n| {
            let ed = &m.edges[*n];
            let (p, q) = (m.vertices[ed.v[0]].pos, m.vertices[ed.v[1]].pos);
            p.z > 999.0 && q.z > 999.0 && p.x > 999.0 && q.x > 999.0
        })
        .expect("shortened top edge on the right");
    m.chamfer_edge(next, ChamferKind::Flat { distance: 100.0 }).unwrap();
    m.validate().unwrap();
}
