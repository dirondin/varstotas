mod common;
use common::*;
use varstotas_kernel::glam::DVec3;
use varstotas_kernel::*;

#[test]
fn rectangle_from_four_lines_makes_one_face() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 50., 0.);
    ok(&m);
    assert_eq!(m.faces.len(), 1);
    assert_eq!(m.edges.len(), 4);
    assert_eq!(m.vertices.len(), 4);
    let f = m.faces.keys().next().unwrap();
    assert!((m.face_area(f) - 5000.0).abs() < 1e-9);
}

#[test]
fn ground_face_faces_down() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 50., 0.);
    let f = m.faces.values().next().unwrap();
    assert!(f.normal.dot(DVec3::NEG_Z) > 0.999);
}

#[test]
fn elevated_horizontal_face_faces_up() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 50., 30.);
    let f = m.faces.values().next().unwrap();
    assert!(f.normal.dot(DVec3::Z) > 0.999);
}

#[test]
fn vertical_face_created() {
    let mut m = Mesh::new();
    m.add_line(v(0., 0., 0.), v(10., 0., 0.));
    m.add_line(v(10., 0., 0.), v(10., 0., 10.));
    m.add_line(v(10., 0., 10.), v(0., 0., 10.));
    assert_eq!(m.faces.len(), 0);
    m.add_line(v(0., 0., 10.), v(0., 0., 0.));
    ok(&m);
    assert_eq!(m.faces.len(), 1);
    assert!(m.faces.values().next().unwrap().normal.y.abs() > 0.999);
}

#[test]
fn open_polyline_makes_no_face() {
    let mut m = Mesh::new();
    m.add_line(v(0., 0., 0.), v(10., 0., 0.));
    m.add_line(v(10., 0., 0.), v(10., 10., 0.));
    m.add_line(v(10., 10., 0.), v(0., 10., 0.));
    ok(&m);
    assert_eq!(m.faces.len(), 0);
}

#[test]
fn non_planar_loop_makes_no_face() {
    let mut m = Mesh::new();
    m.add_line(v(0., 0., 0.), v(10., 0., 0.));
    m.add_line(v(10., 0., 0.), v(10., 10., 5.));
    m.add_line(v(10., 10., 5.), v(0., 10., 0.));
    m.add_line(v(0., 10., 0.), v(0., 0., 0.));
    ok(&m);
    assert_eq!(m.faces.len(), 0);
}

#[test]
fn triangle_face() {
    let mut m = Mesh::new();
    m.add_line(v(0., 0., 5.), v(10., 0., 5.));
    m.add_line(v(10., 0., 5.), v(0., 10., 5.));
    m.add_line(v(0., 10., 5.), v(0., 0., 5.));
    ok(&m);
    assert_eq!(m.faces.len(), 1);
    assert!((total_area(&m) - 50.0).abs() < 1e-9);
}

#[test]
fn endpoints_merge_within_tolerance() {
    let mut m = Mesh::new();
    m.add_line(v(0., 0., 0.), v(10., 0., 0.));
    m.add_line(v(10.0004, 0., 0.), v(10., 10., 0.));
    assert_eq!(m.vertices.len(), 3);
    ok(&m);
}

#[test]
fn endpoint_on_edge_splits_it() {
    let mut m = Mesh::new();
    m.add_line(v(0., 0., 0.), v(10., 0., 0.));
    m.add_line(v(5., 0., 0.), v(5., 5., 0.));
    ok(&m);
    assert_eq!(m.edges.len(), 3);
    assert_eq!(m.vertices.len(), 4);
}

#[test]
fn crossing_lines_split_each_other() {
    let mut m = Mesh::new();
    m.add_line(v(0., 0., 0.), v(10., 0., 0.));
    m.add_line(v(5., -5., 0.), v(5., 5., 0.));
    ok(&m);
    assert_eq!(m.edges.len(), 4);
    assert_eq!(m.vertices.len(), 5);
}

#[test]
fn overlapping_collinear_lines_merge() {
    let mut m = Mesh::new();
    m.add_line(v(0., 0., 0.), v(10., 0., 0.));
    let pieces = m.add_line(v(5., 0., 0.), v(15., 0., 0.));
    ok(&m);
    assert_eq!(m.edges.len(), 3); // 0-5, 5-10, 10-15
    assert_eq!(pieces.len(), 2);
    let total: f64 = m.edges.keys().map(|e| m.edge_length(e)).sum();
    assert!((total - 15.0).abs() < 1e-9);
}

#[test]
fn duplicate_line_is_reused() {
    let mut m = Mesh::new();
    let a = m.add_line(v(0., 0., 0.), v(10., 0., 0.));
    let b = m.add_line(v(10., 0., 0.), v(0., 0., 0.));
    assert_eq!(a, b);
    assert_eq!(m.edges.len(), 1);
}

#[test]
fn contained_collinear_line_splits_existing() {
    let mut m = Mesh::new();
    m.add_line(v(0., 0., 0.), v(10., 0., 0.));
    m.add_line(v(2., 0., 0.), v(4., 0., 0.));
    ok(&m);
    assert_eq!(m.edges.len(), 3);
}

#[test]
fn line_across_face_splits_it() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 50., 0.);
    m.add_line(v(40., 0., 0.), v(40., 50., 0.));
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    let mut areas: Vec<f64> = m.faces.keys().map(|f| m.face_area(f)).collect();
    areas.sort_by(|a, b| a.total_cmp(b));
    assert!((areas[0] - 2000.0).abs() < 1e-9 && (areas[1] - 3000.0).abs() < 1e-9);
    // Split faces keep the orientation of the original.
    assert!(m.faces.values().all(|f| f.normal.z < -0.999));
}

#[test]
fn split_keeps_material() {
    let mut m = Mesh::new();
    let f = ground_rect(&mut m, 100., 50.);
    let mut model = Model::new();
    let mat = model.add_material("red", [1., 0., 0., 1.]);
    m.faces[f].front = Some(mat);
    m.add_line(v(40., 0., 0.), v(40., 50., 0.));
    assert_eq!(m.faces.len(), 2);
    assert!(m.faces.values().all(|f| f.front == Some(mat)));
}

#[test]
fn diagonal_across_face_splits_it() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 10., 10., 0.);
    m.add_line(v(0., 0., 0.), v(10., 10., 0.));
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    for f in m.faces.keys() {
        assert!((m.face_area(f) - 50.0).abs() < 1e-9);
    }
}

#[test]
fn erasing_dividing_line_merges_faces() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 50., 0.);
    let cut = m.add_line(v(40., 0., 0.), v(40., 50., 0.));
    assert_eq!(m.faces.len(), 2);
    m.erase(&[EntityRef::Edge(cut[0])]);
    ok(&m);
    assert_eq!(m.faces.len(), 1);
    assert!((total_area(&m) - 5000.0).abs() < 1e-9);
    // The split vertices are healed away.
    assert_eq!(m.edges.len(), 4);
    assert_eq!(m.vertices.len(), 4);
}

#[test]
fn erasing_boundary_edge_removes_face() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 50., 0.);
    let e = m.edges.keys().next().unwrap();
    m.erase(&[EntityRef::Edge(e)]);
    ok(&m);
    assert_eq!(m.faces.len(), 0);
    assert_eq!(m.edges.len(), 3);
}

#[test]
fn erasing_face_keeps_edges() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 50., 0.);
    let f = m.faces.keys().next().unwrap();
    m.erase(&[EntityRef::Face(f)]);
    ok(&m);
    assert_eq!(m.faces.len(), 0);
    assert_eq!(m.edges.len(), 4);
}

#[test]
fn erasing_vertex_removes_its_edges_and_orphans() {
    let mut m = Mesh::new();
    m.add_line(v(0., 0., 0.), v(10., 0., 0.));
    m.add_line(v(10., 0., 0.), v(10., 10., 0.));
    let vid = m.vertices.iter().find(|(_, vx)| vx.pos == v(10., 0., 0.)).unwrap().0;
    m.erase(&[EntityRef::Vertex(vid)]);
    ok(&m);
    assert_eq!(m.edges.len(), 0);
    assert_eq!(m.vertices.len(), 0);
}

#[test]
fn square_inside_square_makes_hole_and_inner_face() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 0.);
    rect_lines(&mut m, 30., 30., 60., 60., 0.);
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    let outer = m.faces.iter().find(|(_, f)| !f.holes.is_empty()).map(|(id, _)| id).unwrap();
    assert_eq!(m.faces[outer].holes.len(), 1);
    assert!((m.face_area(outer) - (10000.0 - 900.0)).abs() < 1e-6);
    assert!((total_area(&m) - 10000.0).abs() < 1e-6);
}

#[test]
fn erasing_inner_face_leaves_hole() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 0.);
    rect_lines(&mut m, 30., 30., 60., 60., 0.);
    let inner = m.faces.iter().find(|(_, f)| f.holes.is_empty()).map(|(id, _)| id).unwrap();
    m.erase(&[EntityRef::Face(inner)]);
    ok(&m);
    assert_eq!(m.faces.len(), 1);
    assert!((total_area(&m) - 9100.0).abs() < 1e-6);
}

#[test]
fn erasing_inner_square_edge_heals_outer_face() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 0.);
    rect_lines(&mut m, 30., 30., 60., 60., 0.);
    let e = m.edges.iter().find(|(_, e)| m.vertices[e.v[0]].pos.x == 30.0 && m.vertices[e.v[1]].pos.x == 30.0).unwrap().0;
    m.erase(&[EntityRef::Edge(e)]);
    ok(&m);
    assert_eq!(m.faces.len(), 1);
    assert!(m.faces.values().next().unwrap().holes.is_empty());
    assert!((total_area(&m) - 10000.0).abs() < 1e-6);
}

#[test]
fn outer_rectangle_drawn_around_existing_face() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 30., 30., 60., 60., 0.);
    rect_lines(&mut m, 0., 0., 100., 100., 0.);
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    assert!((total_area(&m) - 10000.0).abs() < 1e-6);
    assert!(m.faces.values().any(|f| f.holes.len() == 1));
}

#[test]
fn line_touching_boundary_then_closing_inside() {
    // A notch: lines from the boundary into the face and back form a new region.
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 0.);
    m.add_line(v(20., 0., 0.), v(20., 30., 0.));
    assert_eq!(m.faces.len(), 1);
    m.add_line(v(20., 30., 0.), v(50., 30., 0.));
    assert_eq!(m.faces.len(), 1);
    m.add_line(v(50., 30., 0.), v(50., 0., 0.));
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    assert!((total_area(&m) - 10000.0).abs() < 1e-6);
}

#[test]
fn dangling_line_inside_face_does_not_split() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 0.);
    m.add_line(v(20., 20., 0.), v(50., 50., 0.));
    ok(&m);
    assert_eq!(m.faces.len(), 1);
    assert_eq!(m.edges.len(), 5);
}

#[test]
fn add_face_returns_face_and_no_duplicates() {
    let mut m = Mesh::new();
    let pts = [v(0., 0., 10.), v(10., 0., 10.), v(10., 10., 10.), v(0., 10., 10.)];
    let f1 = m.add_face(&pts).unwrap();
    let f2 = m.add_face(&pts).unwrap();
    assert_eq!(f1, f2);
    assert_eq!(m.faces.len(), 1);
    ok(&m);
    // Point order defines the normal off the ground.
    assert!(m.faces[f1].normal.z > 0.999);
    let mut m2 = Mesh::new();
    let rev: Vec<_> = pts.iter().rev().copied().collect();
    let g = m2.add_face(&rev).unwrap();
    assert!(m2.faces[g].normal.z < -0.999);
}

#[test]
fn add_face_on_ground_faces_down() {
    let mut m = Mesh::new();
    let f = m.add_face(&[v(0., 0., 0.), v(0., 10., 0.), v(10., 10., 0.), v(10., 0., 0.)]).unwrap();
    assert!(m.faces[f].normal.z < -0.999);
}

#[test]
fn add_face_rejects_nonplanar_and_degenerate() {
    let mut m = Mesh::new();
    assert!(m.add_face(&[v(0., 0., 0.), v(1., 0., 0.)]).is_none());
    assert!(m.add_face(&[v(0., 0., 0.), v(1., 0., 0.), v(2., 0., 0.)]).is_none());
    assert!(m.add_face(&[v(0., 0., 0.), v(10., 0., 0.), v(10., 10., 5.), v(0., 10., 0.)]).is_none());
    assert_eq!(m.edges.len(), 0);
}

#[test]
fn adjacent_faces_share_edge() {
    let mut m = Mesh::new();
    m.add_face(&[v(0., 0., 0.), v(10., 0., 0.), v(10., 10., 0.), v(0., 10., 0.)]);
    m.add_face(&[v(10., 0., 0.), v(20., 0., 0.), v(20., 10., 0.), v(10., 10., 0.)]);
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    assert_eq!(m.edges.len(), 7);
}

#[test]
fn polygon_with_edge_on_existing_edge_splits_it() {
    let mut m = Mesh::new();
    m.add_face(&[v(0., 0., 0.), v(20., 0., 0.), v(20., 10., 0.), v(0., 10., 0.)]);
    // Smaller square sharing part of the top edge, outside the first face.
    m.add_face(&[v(5., 10., 0.), v(15., 10., 0.), v(15., 20., 0.), v(5., 20., 0.)]);
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    assert!((total_area(&m) - 300.0).abs() < 1e-9);
}

#[test]
fn circle_polyline_is_one_curve_and_face() {
    let mut m = Mesh::new();
    let pts: Vec<DVec3> = (0..24)
        .map(|i| {
            let a = i as f64 / 24.0 * std::f64::consts::TAU;
            v(a.cos() * 50.0, a.sin() * 50.0, 0.0)
        })
        .collect();
    let es = m.add_polyline(&pts, true, true);
    ok(&m);
    assert_eq!(es.len(), 24);
    assert_eq!(m.faces.len(), 1);
    let c = m.edges[es[0]].curve;
    assert!(c.is_some());
    assert!(es.iter().all(|e| m.edges[*e].curve == c));
    // A second curve gets a different id.
    let es2 = m.add_polyline(&[v(100., 0., 0.), v(110., 5., 0.), v(120., 0., 0.)], false, true);
    assert_ne!(m.edges[es2[0]].curve, c);
}

#[test]
fn circle_on_face_makes_hole_and_disc() {
    let mut m = Mesh::new();
    rect_lines(&mut m, -100., -100., 100., 100., 0.);
    let pts: Vec<DVec3> = (0..16)
        .map(|i| {
            let a = i as f64 / 16.0 * std::f64::consts::TAU;
            v(a.cos() * 30.0, a.sin() * 30.0, 0.0)
        })
        .collect();
    m.add_polyline(&pts, true, true);
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    assert!((total_area(&m) - 40000.0).abs() < 1e-6);
}

#[test]
fn two_crossing_rectangles() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 10., 10., 0.);
    rect_lines(&mut m, 5., 5., 15., 15., 0.);
    ok(&m);
    // Union area 175 split into 3 faces: two L-shapes and the overlap.
    assert_eq!(m.faces.len(), 3);
    assert!((total_area(&m) - 175.0).abs() < 1e-6);
}

#[test]
fn connected_returns_component() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 10., 10., 0.);
    rect_lines(&mut m, 50., 0., 60., 10., 0.);
    let f = m.faces.keys().next().unwrap();
    let set = m.connected(EntityRef::Face(f));
    assert_eq!(set.len(), 5); // 4 edges + 1 face
}

#[test]
fn raycast_hits_nearest_face() {
    let mut m = Mesh::new();
    m.add_face(&[v(0., 0., 10.), v(10., 0., 10.), v(10., 10., 10.), v(0., 10., 10.)]);
    m.add_face(&[v(0., 0., 20.), v(10., 0., 20.), v(10., 10., 20.), v(0., 10., 20.)]);
    let r = Ray { origin: v(5., 5., 100.), dir: DVec3::NEG_Z };
    let (_, t) = m.raycast_faces(&r).unwrap();
    assert!((t - 80.0).abs() < 1e-9);
    let miss = Ray { origin: v(50., 5., 100.), dir: DVec3::NEG_Z };
    assert!(m.raycast_faces(&miss).is_none());
}

#[test]
fn reverse_and_orient_faces() {
    let mut m = Mesh::new();
    make_box(&mut m, 10., 10., 10.);
    let f = m.faces.keys().next().unwrap();
    let n0 = m.faces[f].normal;
    m.reverse_face(f);
    assert!((m.faces[f].normal + n0).length() < 1e-9);
    ok(&m);
    // Orient everything like the reversed face: all faces now point inward.
    m.orient_faces_like(f);
    assert_closed_consistent(&m);
    let c = centroid(&m);
    for (id, face) in &m.faces {
        assert!(face.normal.dot(face_centroid(&m, id) - c) < 0.0);
    }
}

#[test]
fn caches_rebuild_after_manual_invalidation() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 10., 10., 0.);
    m.invalidate_caches();
    let e = m.edges.keys().next().unwrap();
    assert_eq!(m.edge_faces(e).len(), 1); // slow path
    m.add_line(v(5., 0., 0.), v(5., 10., 0.));
    ok(&m);
    assert_eq!(m.faces.len(), 2);
}

#[test]
fn many_lines_perf_grid() {
    // 100x100 grid of lines = 20k edges after splitting; must be fast.
    let mut m = Mesh::new();
    let n = 100;
    let t = std::time::Instant::now();
    for i in 0..=n {
        let x = i as f64 * 10.0;
        m.add_line(v(x, 0., 0.), v(x, 1000., 0.));
    }
    for j in 0..=n {
        let y = j as f64 * 10.0;
        m.add_line(v(0., y, 0.), v(1000., y, 0.));
    }
    assert_eq!(m.faces.len(), n * n);
    assert_eq!(m.edges.len(), 2 * n * (n + 1));
    let el = t.elapsed();
    eprintln!("grid build: {el:?}");
    ok(&m);
}

#[test]
fn many_separate_segments_fast() {
    let mut m = Mesh::new();
    let t = std::time::Instant::now();
    for i in 0..20_000 {
        let x = (i % 200) as f64 * 10.0;
        let y = (i / 200) as f64 * 10.0;
        m.add_line(v(x, y, 0.), v(x + 3., y + 2., 0.));
    }
    assert_eq!(m.edges.len(), 20_000);
    let el = t.elapsed();
    eprintln!("20k segments: {el:?}");
    assert!(el.as_secs_f64() < 20.0);
}
