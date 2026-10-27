//! QA regression tests for M4 batch 2 (curves, dimensions).

mod common;
use common::*;
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::*;
use std::f64::consts::PI;

fn curve_id(m: &Mesh, es: &[EdgeId]) -> u32 {
    m.edges[es[0]].curve.unwrap()
}

/// Circle hole in a square face; growing the radius past the square must not leave a hole
/// outside its outer loop (either refuse, or produce sane topology).
#[test]
fn set_curve_hole_cannot_escape_face() {
    let mut m = Mesh::new();
    m.add_face(&[v(-500., -500., 0.), v(500., -500., 0.), v(500., 500., 0.), v(-500., 500., 0.)]).unwrap();
    let info = CurveInfo::circle(v(0., 0., 0.), DVec3::Z, DVec3::X, 200.0, 24).unwrap();
    let es = m.add_curve(&info);
    let cid = curve_id(&m, &es);
    sane(&m);
    if m.set_curve(cid, Some(800.0), None).is_ok() {
        sane(&m);
    }
}

/// Two circle holes in one face: growing one into the other.
#[test]
fn set_curve_hole_cannot_cross_other_hole() {
    let mut m = Mesh::new();
    m.add_face(&[v(-2000., -1000., 0.), v(2000., -1000., 0.), v(2000., 1000., 0.), v(-2000., 1000., 0.)]).unwrap();
    let a = m.add_curve(&CurveInfo::circle(v(-500., 0., 0.), DVec3::Z, DVec3::X, 300.0, 24).unwrap());
    m.add_curve(&CurveInfo::circle(v(500., 0., 0.), DVec3::Z, DVec3::X, 300.0, 24).unwrap());
    let cid = curve_id(&m, &a);
    sane(&m);
    if m.set_curve(cid, Some(700.0), None).is_ok() {
        sane(&m);
    }
}

/// Random: circles/arcs in a face, random regeneration; every successful regeneration must
/// leave sane topology (valid, no crossing edges, no overlapping faces).
#[test]
fn random_set_curve_is_sane() {
    let mut s: u64 = 0x9E3779B97F4A7C15;
    let mut rnd = || {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((s >> 33) as f64) / (1u64 << 31) as f64
    };
    let mut bad = Vec::new();
    for iter in 0..60 {
        let mut m = Mesh::new();
        m.add_face(&[v(-2000., -2000., 0.), v(2000., -2000., 0.), v(2000., 2000., 0.), v(-2000., 2000., 0.)]).unwrap();
        let mut cids = Vec::new();
        for _ in 0..3 {
            let c = v(rnd() * 3000. - 1500., rnd() * 3000. - 1500., 0.);
            let r = 50. + rnd() * 400.;
            let info = CurveInfo::circle(c, DVec3::Z, DVec3::X, r, 6 + (rnd() * 20.) as u32).unwrap();
            let es = m.add_curve(&info);
            if let Some(e) = es.first() {
                cids.extend(m.edges[*e].curve);
            }
        }
        if m.validate().is_err() || edges_touch(&m).is_some() {
            continue;
        }
        for c in cids {
            if m.curve_info(c).is_none() {
                continue;
            }
            let r = 30. + rnd() * 1500.;
            let before = m.clone();
            if m.set_curve(c, Some(r), None).is_ok() {
                let valid = m.validate().is_ok();
                let touch = edges_touch(&m).is_some();
                let overlap = coplanar_overlap(&m).is_some();
                if !valid || touch || overlap {
                    bad.push((iter, r, valid, touch, overlap));
                    m = before;
                }
            }
        }
    }
    assert!(bad.is_empty(), "{} insane regenerations, e.g. {:?}", bad.len(), &bad[..bad.len().min(5)]);
}

/// Regenerating a Pie's arc keeps one planar face whose radii follow the arc ends.
#[test]
fn set_curve_pie_radius_follows() {
    let mut m = Mesh::new();
    let arc = CurveInfo::arc_center(v(0., 0., 0.), DVec3::Z, v(500., 0., 0.), PI / 2.0, 6).unwrap();
    let es = m.add_pie(&arc);
    let cid = es.iter().find_map(|e| m.edges[*e].curve).unwrap();
    m.set_curve(cid, Some(800.0), Some(10)).unwrap();
    sane(&m);
    assert_eq!(m.faces.len(), 1);
    let f = m.faces.keys().next().unwrap();
    let expected = 0.5 * 10.0 * 800.0 * 800.0 * (PI / 2.0 / 10.0).sin();
    assert!((m.face_area(f) / expected - 1.0).abs() < 1e-9, "{} vs {expected}", m.face_area(f));
}

/// Dimension attached to an inner vertex of a circle: regeneration removes that vertex; the
/// dimension must keep a sane (free) anchor and not vanish.
#[test]
fn dimension_on_regenerated_circle_vertex_detaches() {
    let mut model = Model::new();
    let info = CurveInfo::circle(v(0., 0., 0.), DVec3::Z, DVec3::X, 500.0, 12).unwrap();
    let es = model.root_mesh_mut().add_curve(&info);
    let cid = curve_id(model.root_mesh(), &es);
    let id = model.add_dimension_in(&[], v(500., 0., 0.), v(-500., 0., 0.), v(0., -300., 0.)).unwrap();
    model.root_mesh_mut().set_curve(cid, None, Some(8)).unwrap();
    let g = model.dimension_in(&[], id).unwrap();
    assert!((g.length() - 1000.0).abs() < 1e-6, "{}", g.length());
}

/// Dimension to a circle point, then the radius changes: classic behaviour would keep
/// the dimension on the curve. Varstotas detaches it (anchors become free at old positions).
/// Documented here as current behaviour.
#[test]
fn dimension_on_circle_radius_change_current_behaviour() {
    let mut model = Model::new();
    let info = CurveInfo::circle(v(0., 0., 0.), DVec3::Z, DVec3::X, 500.0, 12).unwrap();
    let es = model.root_mesh_mut().add_curve(&info);
    let cid = curve_id(model.root_mesh(), &es);
    let id = model.add_dimension_in(&[], v(500., 0., 0.), v(-500., 0., 0.), v(0., -300., 0.)).unwrap();
    model.root_mesh_mut().set_curve(cid, Some(800.0), None).unwrap();
    let g = model.dimension_in(&[], id).unwrap();
    // Stale: still 1000 although the diameter is now 1600.
    assert!((g.length() - 1000.0).abs() < 1e-6);
}

/// Curve metadata after a uniform-scaled group is exploded keeps the curve editable.
#[test]
fn curve_editable_after_rotated_group_explode() {
    let mut model = Model::new();
    let info = CurveInfo::arc_center(v(0., 0., 0.), DVec3::Z, v(300., 0., 0.), PI * 0.75, 9).unwrap();
    let es = model.root_mesh_mut().add_curve(&info);
    let ents: Vec<EntityRef> = es.iter().map(|e| EntityRef::Edge(*e)).collect();
    let g = model.make_group(&[], &ents).unwrap();
    model.root_mesh_mut().instances[g].transform =
        DMat4::from_rotation_z(0.7) * DMat4::from_rotation_x(0.3) * DMat4::from_scale(DVec3::splat(1.5));
    model.explode(&[], g).unwrap();
    let m = model.root_mesh_mut();
    let cid = m.edges.values().find_map(|e| e.curve).unwrap();
    let i = m.curve_info(cid).expect("metadata survives rotated explode");
    assert!((i.radius - 450.0).abs() < 1e-6);
    m.set_curve(cid, Some(600.0), Some(4)).unwrap();
    sane(m);
}

/// Non-uniform scale drops the metadata (documented), so Properties can't edit it any more.
#[test]
fn non_uniform_scale_drops_curve_metadata() {
    let mut m = Mesh::new();
    let es = m.add_curve(&CurveInfo::circle(v(0., 0., 0.), DVec3::Z, DVec3::X, 100.0, 12).unwrap());
    let cid = curve_id(&m, &es);
    let vs: std::collections::HashSet<VertexId> = m.vertices.keys().collect();
    m.transform_vertices(&vs, &DMat4::from_scale(DVec3::new(2.0, 1.0, 1.0)));
    assert!(m.curve_info(cid).is_none());
}

/// Circle A grows until it encloses circle B (no edge crossing): B's disc must become a hole
/// of A's disc (QA4B-K1).
#[test]
fn set_curve_enclosing_other_circle_no_overlap() {
    let mut m = Mesh::new();
    m.add_face(&[v(-2000., -2000., 0.), v(2000., -2000., 0.), v(2000., 2000., 0.), v(-2000., 2000., 0.)]).unwrap();
    let a = m.add_curve(&CurveInfo::circle(v(-800., 0., 0.), DVec3::Z, DVec3::X, 200.0, 24).unwrap());
    m.add_curve(&CurveInfo::circle(v(0., 0., 0.), DVec3::Z, DVec3::X, 100.0, 24).unwrap());
    let cid = curve_id(&m, &a);
    sane(&m);
    // r1100 stays inside the square and encloses B.
    m.set_curve(cid, Some(1100.0), None).unwrap();
    sane(&m);
    // Square with A's hole, A's disc with B's hole, B's disc.
    assert_eq!(m.faces.len(), 3);
    let mut holes: Vec<usize> = m.faces.values().map(|f| f.holes.len()).collect();
    holes.sort_unstable();
    assert_eq!(holes, [0, 1, 1]);
    let total: f64 = m.faces.keys().map(|f| m.face_area(f)).sum();
    assert!((total / 16e6 - 1.0).abs() < 1e-9, "{total}");
    // Shrinking A again hands B's hole back to the square.
    m.set_curve(cid, Some(200.0), None).unwrap();
    sane(&m);
    let mut holes: Vec<usize> = m.faces.values().map(|f| f.holes.len()).collect();
    holes.sort_unstable();
    assert_eq!(holes, [0, 0, 2]);
    let total: f64 = m.faces.keys().map(|f| m.face_area(f)).sum();
    assert!((total / 16e6 - 1.0).abs() < 1e-9, "{total}");
}

/// Push/pull of a disc: keep both the top and the bottom circle as circles.
#[test]
fn push_pull_disc_keeps_both_circles() {
    let mut m = Mesh::new();
    m.add_curve(&CurveInfo::circle(v(0., 0., 0.), DVec3::Z, DVec3::X, 500.0, 24).unwrap());
    let f = m.faces.keys().next().unwrap();
    m.push_pull(f, 1000.0, false).unwrap();
    ok(&m);
    let ids: std::collections::HashSet<u32> = m.edges.values().filter_map(|e| e.curve).collect();
    let valid = ids.iter().filter(|c| m.curve_info(**c).is_some()).count();
    assert_eq!(valid, 2, "curve ids {ids:?}, valid {valid}");
}
