//! Sweep along curves (Bezier / smooth splines, circles).

mod common;
use common::*;
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::*;

/// A vertical w × h rectangle from z = 0, centred on `at`, facing `dir` (horizontal).
fn profile(m: &mut Mesh, at: DVec3, dir: DVec3, w: f64, h: f64) -> FaceId {
    let side = DVec3::Z.cross(dir).normalize() * (w / 2.0);
    m.add_polyline(&[at - side, at + side, at + side + DVec3::Z * h, at - side + DVec3::Z * h], true, false);
    m.faces.keys().find(|f| m.faces[*f].normal.z.abs() < 0.5).expect("profile face")
}

/// The enclosed volume (faces' outer loops; the sweeps here have no holes).
fn volume(m: &Mesh) -> f64 {
    let mut v = 0.0;
    for f in m.faces.values() {
        let p: Vec<DVec3> = f.outer.iter().map(|x| m.vertices[*x].pos).collect();
        let area: DVec3 = (0..p.len()).map(|i| p[i].cross(p[(i + 1) % p.len()])).sum::<DVec3>() * 0.5;
        v += p[0].dot(area) / 3.0;
    }
    v.abs()
}

fn path_len(m: &Mesh, path: &[EdgeId]) -> f64 {
    path.iter().map(|e| { let [a, b] = m.edges[*e].v; m.vertices[a].pos.distance(m.vertices[b].pos) }).sum()
}

/// A Bezier path's sweep: a closed solid (only the path's own edges may lack two faces), as big
/// as the profile times the path, smooth along the curve.
fn check_sweep(m: &Mesh, path_curve: u32, area: f64, len: f64) {
    ok(m);
    for e in m.edges.keys() {
        let n = m.edge_faces(e).len();
        assert!(n == 2 || m.edges[e].curve == Some(path_curve), "edge with {n} faces off the path");
    }
    let v = volume(m);
    assert!((v / (area * len) - 1.0).abs() < 0.03, "volume {v} vs {}", area * len);
    assert!(m.edges.values().filter(|e| e.soft && e.smooth).count() >= 20);
}

fn smooth_path(pts: &[DVec3]) -> Spline {
    Spline { anchors: spline::smooth(&pts.iter().copied().map(Anchor::corner).collect::<Vec<_>>(), false), closed: false, segs: 12, normal: DVec3::Z }
}

#[test]
fn profile_perpendicular_to_the_curve() {
    let sp = smooth_path(&[v(0., 0., 0.), v(1000., 500., 0.), v(2000., 0., 0.)]);
    let mut m = Mesh::default();
    let es = m.add_spline(&sp);
    let cid = m.edges[es[0]].curve.unwrap();
    let path = m.curve_edges(cid);
    let len = path_len(&m, &path);
    let p = sp.points();
    let f = profile(&mut m, p[0], p[1] - p[0], 100., 100.);
    m.follow_me(f, &path).unwrap();
    check_sweep(&m, cid, 100. * 100., len);
}

#[test]
fn axis_profile_at_a_tilted_curve_end_does_not_fold() {
    // The curve leaves its start at ~30° with short end segments: a 400 wide profile drawn
    // along the axes used to fold its first sections back inside out.
    let sp = smooth_path(&[v(0., 0., 0.), v(1500., 800., 0.), v(3000., 0., 0.)]);
    let mut m = Mesh::default();
    let es = m.add_spline(&sp);
    let cid = m.edges[es[0]].curve.unwrap();
    let path = m.curve_edges(cid);
    let len = path_len(&m, &path);
    let f = profile(&mut m, v(0., 0., 0.), DVec3::X, 400., 300.);
    m.follow_me(f, &path).unwrap();
    ok(&m);
    // The section leans against the path: the volume is the profile's area seen along it.
    let v0 = volume(&m);
    assert!(v0 > 400. * 300. * len * 0.75 && v0 < 400. * 300. * len * 1.05, "volume {} of {}", v0, 400. * 300. * len);
    for e in m.edges.keys() {
        assert!(m.edge_faces(e).len() == 2 || m.edges[e].curve == Some(cid));
    }
}

#[test]
fn wide_profile_on_a_tight_bend_does_not_fold() {
    // 600 wide on a bend of a few hundred: the inner side would cross itself.
    let sp = smooth_path(&[v(0., 0., 0.), v(500., 0., 0.), v(600., 300., 0.), v(500., 600., 0.), v(0., 600., 0.)]);
    let mut m = Mesh::default();
    let es = m.add_spline(&sp);
    let cid = m.edges[es[0]].curve.unwrap();
    let path = m.curve_edges(cid);
    let p = sp.points();
    let f = profile(&mut m, p[0], p[1] - p[0], 600., 100.);
    m.follow_me(f, &path).unwrap();
    ok(&m);
    for f in m.faces.keys() {
        assert!(m.face_area(f) > 0.0);
    }
    for e in m.edges.keys() {
        assert!(m.edge_faces(e).len() == 2 || m.edges[e].curve == Some(cid));
    }
}

#[test]
fn circle_profile_along_a_curve_is_smooth_both_ways() {
    let sp = smooth_path(&[v(0., 0., 0.), v(1000., 400., 0.), v(2000., -400., 0.), v(3000., 0., 0.)]);
    let mut m = Mesh::default();
    let es = m.add_spline(&sp);
    let cid = m.edges[es[0]].curve.unwrap();
    let path = m.curve_edges(cid);
    let p = sp.points();
    let t = (p[1] - p[0]).normalize();
    let side = DVec3::Z.cross(t);
    m.add_curve(&CurveInfo::circle(p[0] + DVec3::Z * 200., t, side, 100., 24).unwrap());
    let f = m.faces.keys().next().unwrap();
    m.follow_me(f, &path).unwrap();
    ok(&m);
    // Around the tube (profile vertices) and along it (path vertices) the edges are smooth;
    // only the end rings stay hard.
    let hard: Vec<_> = m.edges.values().filter(|e| e.curve != Some(cid) && !e.soft).collect();
    assert_eq!(hard.len(), 2 * 24, "only the end rings");
}
