//! DXF export / import.

use varstotas_kernel::glam::DVec3;
use varstotas_kernel::io::{ImportOptions, add_dxf, parse_dxf, write_dxf};
use varstotas_kernel::Model;

fn import(text: &str) -> Model {
    let mut m = Model::default();
    add_dxf(&mut m, &parse_dxf(text).unwrap(), &ImportOptions::default()).unwrap();
    m
}

/// DXF text from (code, value) pairs.
fn dxf(header: &[(i32, &str)], ents: &[(i32, &str)]) -> String {
    let mut s = String::new();
    let mut put = |c: i32, v: &str| s.push_str(&format!("{c}\n{v}\n"));
    put(0, "SECTION");
    put(2, "HEADER");
    header.iter().for_each(|(c, v)| put(*c, v));
    put(0, "ENDSEC");
    put(0, "SECTION");
    put(2, "ENTITIES");
    ents.iter().for_each(|(c, v)| put(*c, v));
    put(0, "ENDSEC");
    put(0, "EOF");
    s
}

#[test]
fn box_round_trip() {
    let mut m = Model::default();
    let mesh = m.root_mesh_mut();
    let f = mesh
        .add_face(&[DVec3::ZERO, DVec3::new(2000.0, 0.0, 0.0), DVec3::new(2000.0, 1000.0, 0.0), DVec3::new(0.0, 1000.0, 0.0)])
        .unwrap();
    let n = mesh.faces[f].normal;
    mesh.push_pull(f, if n.z > 0.0 { 500.0 } else { -500.0 }, false).unwrap();
    let text = write_dxf(&m);
    assert!(text.contains("AC1009") && text.contains("3DFACE") && text.contains("LINE"));
    let data = parse_dxf(&text).unwrap();
    assert_eq!(data.faces.len(), 12, "two triangles per face");
    assert_eq!(data.polylines.len(), 12, "12 edges");
    let back = import(&text);
    let mesh = back.root_mesh();
    assert_eq!(mesh.faces.len(), 6, "coplanar triangles merged back");
    assert_eq!(mesh.edges.len(), 12);
    assert!(mesh.is_solid());
    assert!((mesh.volume().abs() - 2000.0 * 1000.0 * 500.0).abs() < 1.0);
}

#[test]
fn closed_lwpolyline_makes_a_face_in_inches() {
    let ents = [
        (0, "LWPOLYLINE"),
        (8, "0"),
        (90, "4"),
        (70, "1"),
        (10, "0"),
        (20, "0"),
        (10, "10"),
        (20, "0"),
        (10, "10"),
        (20, "5"),
        (10, "0"),
        (20, "5"),
    ];
    let m = import(&dxf(&[(9, "$INSUNITS"), (70, "1")], &ents));
    let mesh = m.root_mesh();
    assert_eq!(mesh.edges.len(), 4);
    assert_eq!(mesh.faces.len(), 1);
    let f = mesh.faces.keys().next().unwrap();
    assert!((mesh.face_area(f) - 254.0 * 127.0).abs() < 1e-3, "inches → mm");
}

#[test]
fn bulge_circle_and_arc() {
    // Two half-circle bulges (b = 1): a closed circle of radius 50 around (50, 0).
    let ents = [
        (0, "LWPOLYLINE"),
        (90, "2"),
        (70, "1"),
        (10, "0"),
        (20, "0"),
        (42, "1"),
        (10, "100"),
        (20, "0"),
        (42, "1"),
        (0, "CIRCLE"),
        (10, "500"),
        (20, "0"),
        (30, "0"),
        (40, "100"),
        (0, "ARC"),
        (10, "1000"),
        (20, "0"),
        (30, "0"),
        (40, "100"),
        (50, "0"),
        (51, "90"),
    ];
    let data = parse_dxf(&dxf(&[], &ents)).unwrap();
    assert_eq!(data.polylines.len(), 3);
    let (circle, closed) = &data.polylines[0];
    assert!(*closed);
    for p in circle {
        assert!((p.distance(DVec3::new(50.0, 0.0, 0.0)) - 50.0).abs() < 1e-6, "on the circle: {p}");
    }
    assert!(circle.iter().any(|p| p.y < -40.0), "the first half bulges below (counter-clockwise)");
    let (arc, closed) = &data.polylines[2];
    assert!(!*closed);
    assert!(arc[0].distance(DVec3::new(1100.0, 0.0, 0.0)) < 1e-9);
    assert!(arc.last().unwrap().distance(DVec3::new(1000.0, 100.0, 0.0)) < 1e-9);
    let m = import(&dxf(&[], &ents));
    assert_eq!(m.root_mesh().faces.len(), 2, "the two circles fill");
}

#[test]
fn polyface_mesh() {
    // A unit square as a polyface mesh: 4 position vertices, one 4-sided face.
    let mut ents = vec![(0, "POLYLINE"), (66, "1"), (70, "64")];
    for (x, y) in [("0", "0"), ("100", "0"), ("100", "100"), ("0", "100")] {
        ents.extend([(0, "VERTEX"), (10, x), (20, y), (30, "0"), (70, "192")]);
    }
    ents.extend([(0, "VERTEX"), (10, "0"), (20, "0"), (30, "0"), (70, "128"), (71, "1"), (72, "2"), (73, "3"), (74, "4")]);
    ents.push((0, "SEQEND"));
    let data = parse_dxf(&dxf(&[], &ents)).unwrap();
    assert_eq!(data.faces.len(), 1);
    assert_eq!(data.faces[0].len(), 4);
    let m = import(&dxf(&[], &ents));
    assert_eq!(m.root_mesh().faces.len(), 1);
}

#[test]
fn not_a_dxf() {
    assert!(parse_dxf("hello\nworld\n").is_err());
    assert!(parse_dxf("solid x\nfacet normal 0 0 1\n").is_err());
}
