//! 3D Text geometry from outlines.

use varstotas_kernel::glam::{DMat4, DVec2, DVec3};
use varstotas_kernel::text3d::text_mesh;
use varstotas_kernel::{Model, Text3dOptions};

fn ring(c: DVec2, r: f64, n: usize, cw: bool) -> Vec<DVec2> {
    let mut v: Vec<DVec2> = (0..n)
        .map(|i| {
            let a = i as f64 / n as f64 * std::f64::consts::TAU;
            c + DVec2::new(a.cos(), a.sin()) * r
        })
        .collect();
    if cw {
        v.reverse();
    }
    v
}

/// "O" with a dot in its counter: outer ring, counter, dot (three levels).
fn letters() -> Vec<Vec<DVec2>> {
    vec![ring(DVec2::ZERO, 100.0, 24, true), ring(DVec2::ZERO, 60.0, 24, false), ring(DVec2::ZERO, 20.0, 12, true)]
}

#[test]
fn flat_text_fills_odd_regions() {
    let m = text_mesh(&letters(), Text3dOptions { filled: true, extrude: None });
    assert_eq!(m.faces.len(), 2, "ring and dot, no counter face");
    for f in m.faces.values() {
        let n = varstotas_kernel::geom::newell(&f.outer.iter().map(|v| m.vertices[*v].pos).collect::<Vec<_>>());
        assert!(n.z > 0.0, "faces look up");
    }
    let ring_face = m.faces.values().find(|f| !f.holes.is_empty()).expect("ring has a hole");
    assert_eq!(ring_face.holes.len(), 1);
}

#[test]
fn outlines_only_and_extruded() {
    let m = text_mesh(&letters(), Text3dOptions { filled: false, extrude: Some(50.0) });
    assert_eq!(m.faces.len(), 0);
    assert_eq!(m.edges.len(), 24 + 24 + 12);
    let m = text_mesh(&letters(), Text3dOptions { filled: true, extrude: Some(50.0) });
    // Ring: top, bottom, 24 + 24 sides; dot: top, bottom, 12 sides.
    assert_eq!(m.faces.len(), 2 + 48 + 2 + 12);
    let zmax = m.vertices.values().map(|v| v.pos.z).fold(f64::MIN, f64::max);
    assert!((zmax - 50.0).abs() < 1e-9);
}

#[test]
fn model_group_placed_by_transform() {
    let mut model = Model::new();
    let xf = DMat4::from_translation(DVec3::new(1000.0, 0.0, 0.0));
    let inst = model
        .add_text3d_in(&[], &letters(), Text3dOptions { filled: true, extrude: Some(10.0) }, &xf)
        .unwrap();
    let b = model.bounds().unwrap();
    assert!((b.min.x - 900.0).abs() < 1e-6 && (b.max.z - 10.0).abs() < 1e-6, "{b:?}");
    assert!(model.root_mesh().instances.contains_key(inst));
}
