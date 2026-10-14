mod common;
use common::*;
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::*;
use std::collections::HashSet;

#[test]
fn push_pull_rectangle_makes_closed_box() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 50., 0.);
    let f = m.faces.keys().next().unwrap();
    // Ground face faces down: pulling "up" is a negative distance along its normal.
    let cap = m.push_pull(f, -30.0, false).unwrap();
    ok(&m);
    assert_eq!(m.faces.len(), 6);
    assert_eq!(m.edges.len(), 12);
    assert_eq!(m.vertices.len(), 8);
    assert_closed_consistent(&m);
    assert_outward(&m);
    assert!(m.faces[cap].normal.z > 0.999);
    assert!((m.vertices[m.faces[cap].outer[0]].pos.z - 30.0).abs() < 1e-9);
    assert!((volume(&m) - 150_000.0).abs() < 1e-6);
}

#[test]
fn push_pull_up_facing_face_positive() {
    let mut m = Mesh::new();
    let f = m.add_face(&[v(0., 0., 10.), v(10., 0., 10.), v(10., 10., 10.), v(0., 10., 10.)]).unwrap();
    m.push_pull(f, 5.0, false).unwrap();
    ok(&m);
    assert_eq!(m.faces.len(), 6);
    assert_closed_consistent(&m);
    assert_outward(&m);
    assert!((volume(&m) - 500.0).abs() < 1e-6);
}

#[test]
fn push_pull_box_top_stretches() {
    let mut m = Mesh::new();
    let top = make_box(&mut m, 100., 50., 30.);
    let (nf, ne, nv) = (m.faces.len(), m.edges.len(), m.vertices.len());
    let cap = m.push_pull(top, 20.0, false).unwrap();
    ok(&m);
    assert_eq!(cap, top);
    assert_eq!((m.faces.len(), m.edges.len(), m.vertices.len()), (nf, ne, nv));
    assert_closed_consistent(&m);
    assert_outward(&m);
    assert!((volume(&m) - 100. * 50. * 50.).abs() < 1e-6);
    // Shrink back down.
    m.push_pull(top, -40.0, false).unwrap();
    ok(&m);
    assert!((volume(&m) - 100. * 50. * 10.).abs() < 1e-6);
    // Pushing all the way to the bottom is refused.
    assert!(m.push_pull(top, -10.0, false).is_err());
    ok(&m);
}

#[test]
fn push_pull_box_side_stretches() {
    let mut m = Mesh::new();
    make_box(&mut m, 100., 50., 30.);
    let side = face_with_normal(&m, DVec3::X)[0];
    m.push_pull(side, 25.0, false).unwrap();
    ok(&m);
    assert_eq!(m.faces.len(), 6);
    assert_closed_consistent(&m);
    assert!((volume(&m) - 125. * 50. * 30.).abs() < 1e-6);
}

#[test]
fn push_pull_inner_rect_on_top_makes_boss() {
    let mut m = Mesh::new();
    make_box(&mut m, 100., 100., 30.);
    rect_lines(&mut m, 30., 30., 60., 60., 30.);
    ok(&m);
    assert_eq!(m.faces.len(), 7);
    let inner = m.faces.iter().find(|(id, f)| f.normal.z > 0.9 && f.holes.is_empty() && (m.face_area(*id) - 900.).abs() < 1e-6).unwrap().0;
    m.push_pull(inner, 10.0, false).unwrap();
    ok(&m);
    // box 6 (top has a hole) + boss 4 sides + cap
    assert_eq!(m.faces.len(), 11);
    assert_closed_consistent(&m);
    assert!((volume(&m) - (100. * 100. * 30. + 900. * 10.)).abs() < 1e-6);
}

#[test]
fn push_pull_inner_rect_pocket() {
    let mut m = Mesh::new();
    make_box(&mut m, 100., 100., 30.);
    rect_lines(&mut m, 30., 30., 60., 60., 30.);
    let inner = m.faces.iter().find(|(id, f)| f.normal.z > 0.9 && f.holes.is_empty() && (m.face_area(*id) - 900.).abs() < 1e-6).unwrap().0;
    let cap = m.push_pull(inner, -10.0, false).unwrap();
    ok(&m);
    assert_eq!(m.faces.len(), 11);
    assert_closed_consistent(&m);
    assert!(m.faces[cap].normal.z > 0.999);
    assert!((volume(&m) - (100. * 100. * 30. - 900. * 10.)).abs() < 1e-6);
}

#[test]
fn push_pull_through_hole() {
    let mut m = Mesh::new();
    make_box(&mut m, 100., 100., 30.);
    rect_lines(&mut m, 30., 30., 60., 60., 30.);
    let inner = m.faces.iter().find(|(id, f)| f.normal.z > 0.9 && f.holes.is_empty() && (m.face_area(*id) - 900.).abs() < 1e-6).unwrap().0;
    m.push_pull(inner, -30.0, false).unwrap();
    ok(&m);
    // 4 outer sides + top ring + bottom ring + 4 hole walls
    assert_eq!(m.faces.len(), 10);
    assert_closed_consistent(&m);
    let rings: Vec<_> = m.faces.values().filter(|f| f.holes.len() == 1).collect();
    assert_eq!(rings.len(), 2);
    assert!((volume(&m) - (100. * 100. * 30. - 900. * 30.)).abs() < 1e-6);
}

#[test]
fn push_pull_keep_original() {
    let mut m = Mesh::new();
    let top = make_box(&mut m, 10., 10., 10.);
    let cap = m.push_pull(top, 5.0, true).unwrap();
    ok(&m);
    assert_ne!(cap, top);
    assert!(m.faces.contains_key(top));
    assert_eq!(m.faces.len(), 6 + 5);
}

#[test]
fn push_pull_notch_on_edge_merges_side_with_front() {
    let mut m = Mesh::new();
    make_box(&mut m, 100., 50., 30.);
    // Rectangle on top touching the front edge (y = 0).
    m.add_line(v(30., 0., 30.), v(30., 20., 30.));
    m.add_line(v(30., 20., 30.), v(60., 20., 30.));
    m.add_line(v(60., 20., 30.), v(60., 0., 30.));
    ok(&m);
    let notch = m.faces.iter().find(|(id, f)| f.normal.z > 0.9 && (m.face_area(*id) - 600.).abs() < 1e-6).unwrap().0;
    m.push_pull(notch, 10.0, false).unwrap();
    ok(&m);
    assert_closed_consistent(&m);
    // Front face (normal -Y) grew a bump and stays one face.
    assert_eq!(face_with_normal(&m, DVec3::NEG_Y).len(), 1);
    assert!((volume(&m) - (100. * 50. * 30. + 600. * 10.)).abs() < 1e-6);
}

#[test]
fn push_pull_half_of_split_top() {
    let mut m = Mesh::new();
    make_box(&mut m, 100., 50., 30.);
    m.add_line(v(50., 0., 30.), v(50., 50., 30.));
    let right = m.faces.iter().find(|(id, f)| f.normal.z > 0.9 && face_centroid(&m, *id).x > 50.).unwrap().0;
    m.push_pull(right, 20.0, false).unwrap();
    ok(&m);
    assert_closed_consistent(&m);
    assert!((volume(&m) - (100. * 50. * 30. + 50. * 50. * 20.)).abs() < 1e-6);
    // Front, back and right side each merged into single faces.
    assert_eq!(face_with_normal(&m, DVec3::NEG_Y).len(), 1);
    assert_eq!(face_with_normal(&m, DVec3::Y).len(), 1);
    assert_eq!(face_with_normal(&m, DVec3::X).len(), 1);
    assert_eq!(m.faces.len(), 8);
}

#[test]
fn push_pull_face_with_hole() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 10.);
    rect_lines(&mut m, 30., 30., 60., 60., 10.);
    let inner = m.faces.iter().find(|(_, f)| f.holes.is_empty()).unwrap().0;
    m.erase(&[EntityRef::Face(inner)]);
    let ring = m.faces.keys().next().unwrap();
    let up = m.faces[ring].normal.z > 0.0;
    m.push_pull(ring, if up { 20.0 } else { -20.0 }, false).unwrap();
    ok(&m);
    assert_closed_consistent(&m);
    assert_eq!(m.faces.len(), 2 + 4 + 4);
    assert!((volume(&m).abs() - 9100. * 20.).abs() < 1e-6);
}

#[test]
fn push_pull_zero_and_missing() {
    let mut m = Mesh::new();
    let f = ground_rect(&mut m, 10., 10.);
    assert_eq!(m.push_pull(f, 0.0, false), Ok(f));
    m.erase(&[EntityRef::Face(f)]);
    assert_eq!(m.push_pull(f, 5.0, false), Err(OpError::NotFound));
}

#[test]
fn offset_inward_creates_inner_face() {
    let mut m = Mesh::new();
    let f = m.add_face(&[v(0., 0., 5.), v(100., 0., 5.), v(100., 50., 5.), v(0., 50., 5.)]).unwrap();
    let inner = m.offset_face(f, -10.0).unwrap();
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    assert!((m.face_area(inner) - 80. * 30.).abs() < 1e-6);
    assert!((total_area(&m) - 5000.).abs() < 1e-6);
    assert!(m.faces[inner].normal.z > 0.999);
    let ring = m.faces.keys().find(|k| *k != inner).unwrap();
    assert_eq!(m.faces[ring].holes.len(), 1);
}

#[test]
fn offset_outward_creates_ring() {
    let mut m = Mesh::new();
    let f = m.add_face(&[v(0., 0., 5.), v(10., 0., 5.), v(10., 10., 5.), v(0., 10., 5.)]).unwrap();
    let ring = m.offset_face(f, 5.0).unwrap();
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    assert!((m.face_area(ring) - (400. - 100.)).abs() < 1e-6);
    assert!(m.faces[ring].normal.z > 0.999);
}

#[test]
fn offset_concave_l_shape() {
    let mut m = Mesh::new();
    let f = m
        .add_face(&[v(0., 0., 1.), v(20., 0., 1.), v(20., 10., 1.), v(10., 10., 1.), v(10., 20., 1.), v(0., 20., 1.)])
        .unwrap();
    let inner = m.offset_face(f, -2.0).unwrap();
    ok(&m);
    // L of width 10 -> 6 after offsetting 2 inward on each side: area = 16*6 + 6*10 = 156
    assert!((m.face_area(inner) - 156.0).abs() < 1e-6, "{}", m.face_area(inner));
}

#[test]
fn offset_collapse_errors() {
    let mut m = Mesh::new();
    let f = m.add_face(&[v(0., 0., 1.), v(10., 0., 1.), v(10., 10., 1.), v(0., 10., 1.)]).unwrap();
    assert_eq!(m.offset_face(f, -6.0), Err(OpError::Degenerate));
    assert_eq!(m.faces.len(), 1);
    ok(&m);
}

#[test]
fn offset_then_push_window() {
    // The "window" workflow: offset a wall face inward, push the inner face in.
    let mut m = Mesh::new();
    make_box(&mut m, 100., 100., 100.);
    let front = face_with_normal(&m, DVec3::NEG_Y)[0];
    let inner = m.offset_face(front, -20.0).unwrap();
    m.push_pull(inner, -10.0, false).unwrap();
    ok(&m);
    assert_closed_consistent(&m);
    assert!((volume(&m) - (1_000_000. - 60. * 60. * 10.)).abs() < 1e-6);
}

#[test]
fn follow_me_rect_along_l_path() {
    let mut m = Mesh::new();
    // Path in the XY plane: (0,0,0) -> (100,0,0) -> (100,100,0)
    let p1 = m.add_line(v(0., 0., 0.), v(100., 0., 0.));
    let p2 = m.add_line(v(100., 0., 0.), v(100., 100., 0.));
    // Profile: 10x10 square in the YZ plane at x = 0, around the path start.
    let prof = m.add_face(&[v(0., -5., 0.), v(0., 5., 0.), v(0., 5., 10.), v(0., -5., 10.)]).unwrap();
    let path: Vec<EdgeId> = p1.into_iter().chain(p2).collect();
    m.follow_me(prof, &path).unwrap();
    ok(&m);
    // 2 caps + 2 outer walls + 2 inner walls + one L-shaped bottom + one L-shaped top
    // (coplanar pieces are merged).
    assert_eq!(m.faces.len(), 8);
    // Volume of the mitred L: centre-line length 200 minus... compute exactly:
    // segment 1 prism from x=0 to the miter, segment 2 from miter to y=100.
    // Cross-section 10x10, path along centreline y=0 -> inner/outer offsets cancel: V = 100 * 200.
    assert!((volume(&m).abs() - 100. * 200.).abs() < 1e-6, "{}", volume(&m));
    // The swept faces are closed except for path edges (dangling).
    for (e, _) in &m.edges {
        let n = m.edge_faces(e).len();
        assert!(n == 2 || n == 0, "edge with {n} faces");
    }
}

#[test]
fn follow_me_closed_path_makes_ring() {
    let mut m = Mesh::new();
    let mut path = Vec::new();
    let sq = [v(0., 0., 0.), v(100., 0., 0.), v(100., 100., 0.), v(0., 100., 0.)];
    for i in 0..4 {
        path.extend(m.add_line(sq[i], sq[(i + 1) % 4]));
    }
    // The path loop itself forms a ground face; remove it, keep edges.
    let ground: Vec<FaceId> = m.faces.keys().collect();
    for f in ground {
        m.erase(&[EntityRef::Face(f)]);
    }
    let prof = m.add_face(&[v(50., -5., 20.), v(50., 5., 20.), v(50., 5., 30.), v(50., -5., 30.)]).unwrap();
    // Profile is not on the path: sweep along the closed path.
    let r = m.follow_me(prof, &path);
    assert!(r.is_ok(), "{r:?}");
    ok(&m);
    // 4 outer + 4 inner walls + top ring + bottom ring (coplanar pieces merged).
    assert_eq!(m.faces.len(), 10);
    assert_eq!(m.faces.values().filter(|f| f.holes.len() == 1).count(), 2);
    let sweep_faces: Vec<FaceId> = m.faces.keys().collect();
    for f in sweep_faces {
        for e in m.face_edges(f) {
            assert_eq!(m.edge_faces(e).len(), 2);
        }
    }
}

#[test]
fn follow_me_errors() {
    let mut m = Mesh::new();
    let prof = m.add_face(&[v(0., 0., 0.), v(10., 0., 0.), v(10., 10., 0.), v(0., 10., 0.)]).unwrap();
    assert!(m.follow_me(prof, &[]).is_err());
    // Path parallel to the profile plane.
    let p = m.add_line(v(20., 0., 0.), v(40., 0., 0.));
    assert!(m.follow_me(prof, &p).is_err());
}

#[test]
fn transform_moves_whole_box() {
    let mut m = Mesh::new();
    make_box(&mut m, 10., 10., 10.);
    let all: HashSet<VertexId> = m.vertices.keys().collect();
    m.transform_vertices(&all, &DMat4::from_translation(v(5., 0., 0.)));
    ok(&m);
    assert_eq!(m.faces.len(), 6);
    let b = m.bounds().unwrap();
    assert!((b.min.x - 5.).abs() < 1e-9 && (b.max.x - 15.).abs() < 1e-9);
}

#[test]
fn transform_rotate_keeps_planarity() {
    let mut m = Mesh::new();
    make_box(&mut m, 10., 10., 10.);
    let all: HashSet<VertexId> = m.vertices.keys().collect();
    m.transform_vertices(&all, &DMat4::from_axis_angle(v(1., 1., 0.).normalize(), 0.7));
    ok(&m);
    assert_eq!(m.faces.len(), 6);
    assert_closed_consistent(&m);
    assert!((volume(&m) - 1000.).abs() < 1e-6);
}

#[test]
fn transform_mirror_keeps_outward_normals() {
    let mut m = Mesh::new();
    make_box(&mut m, 10., 10., 10.);
    let all: HashSet<VertexId> = m.vertices.keys().collect();
    m.transform_vertices(&all, &DMat4::from_scale(v(-1., 1., 1.)));
    ok(&m);
    assert_outward(&m);
    assert!((volume(&m) - 1000.).abs() < 1e-6);
}

#[test]
fn moving_one_vertex_folds_faces() {
    let mut m = Mesh::new();
    make_box(&mut m, 10., 10., 10.);
    let corner = m.vertices.iter().find(|(_, x)| x.pos == v(10., 10., 10.)).unwrap().0;
    let s: HashSet<VertexId> = [corner].into_iter().collect();
    m.transform_vertices(&s, &DMat4::from_translation(v(2., 2., 5.)));
    ok(&m);
    assert_closed_consistent(&m);
    // Top, right (+X) and back (+Y) faces become non-planar and fold into 2 triangles each.
    assert_eq!(m.faces.len(), 3 + 6);
    // Moving straight up only folds the top face (sides stay planar).
    let mut m = Mesh::new();
    make_box(&mut m, 10., 10., 10.);
    let corner = m.vertices.iter().find(|(_, x)| x.pos == v(10., 10., 10.)).unwrap().0;
    let s: HashSet<VertexId> = [corner].into_iter().collect();
    m.transform_vertices(&s, &DMat4::from_translation(v(0., 0., 5.)));
    ok(&m);
    assert_closed_consistent(&m);
    assert_eq!(m.faces.len(), 7);
    assert!(m.validate().is_ok());
}

#[test]
fn moving_top_edge_keeps_side_faces_planar() {
    // Moving a whole top edge up (roof ridge): side faces stay planar quads -> no folding.
    let mut m = Mesh::new();
    make_box(&mut m, 10., 10., 10.);
    let s: HashSet<VertexId> =
        m.vertices.iter().filter(|(_, x)| x.pos.z == 10. && x.pos.x == 10.).map(|(k, _)| k).collect();
    m.transform_vertices(&s, &DMat4::from_translation(v(0., 0., 5.)));
    ok(&m);
    assert_eq!(m.faces.len(), 6);
    assert_closed_consistent(&m);
}

#[test]
fn moving_vertex_onto_another_merges() {
    let mut m = Mesh::new();
    m.add_line(v(0., 0., 0.), v(10., 0., 0.));
    m.add_line(v(20., 0., 0.), v(30., 0., 0.));
    let a = m.vertices.iter().find(|(_, x)| x.pos == v(20., 0., 0.)).unwrap().0;
    let s: HashSet<VertexId> = [a].into_iter().collect();
    m.transform_vertices(&s, &DMat4::from_translation(v(-10., 0., 0.)));
    ok(&m);
    assert_eq!(m.vertices.len(), 3);
}

#[test]
fn vertices_of_and_duplicate() {
    let mut m = Mesh::new();
    let f = m.add_face(&[v(0., 0., 1.), v(10., 0., 1.), v(10., 10., 1.), v(0., 10., 1.)]).unwrap();
    let vs = m.vertices_of(&[EntityRef::Face(f)]);
    assert_eq!(vs.len(), 4);
    let new = m.duplicate(&[EntityRef::Face(f)], &DMat4::from_translation(v(20., 0., 0.)));
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    assert_eq!(m.edges.len(), 8);
    assert!(new.iter().any(|r| matches!(r, EntityRef::Face(_))));
    // Copy adjacent: shares an edge with the original.
    m.duplicate(&[EntityRef::Face(f)], &DMat4::from_translation(v(10., 0., 0.)));
    ok(&m);
    assert_eq!(m.faces.len(), 3);
    assert_eq!(m.edges.len(), 10); // shares x=10 with the original and x=20 with the first copy
}

#[test]
fn erase_box_edge_removes_two_faces() {
    let mut m = Mesh::new();
    make_box(&mut m, 10., 10., 10.);
    let e = m.edges.iter().find(|(_, e)| {
        let (a, b) = (m.vertices[e.v[0]].pos, m.vertices[e.v[1]].pos);
        a.z == 10. && b.z == 10. && a.y == 0. && b.y == 0.
    });
    m.erase(&[EntityRef::Edge(e.unwrap().0)]);
    ok(&m);
    assert_eq!(m.faces.len(), 4);
}

#[test]
fn draw_box_from_lines_has_consistent_normals() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 10., 10., 0.);
    for (x, y) in [(0., 0.), (10., 0.), (10., 10.), (0., 10.)] {
        m.add_line(v(x, y, 0.), v(x, y, 10.));
    }
    rect_lines(&mut m, 0., 0., 10., 10., 10.);
    ok(&m);
    assert_eq!(m.faces.len(), 6);
    assert_closed_consistent(&m);
    assert_outward(&m);
}

#[test]
fn house_scenario_ridge_roof() {
    // Floor -> walls -> ridge line on top -> move ridge up -> gable roof.
    let mut m = Mesh::new();
    make_box(&mut m, 100., 60., 50.);
    let ridge = m.add_line(v(50., 0., 50.), v(50., 60., 50.));
    assert_eq!(ridge.len(), 1);
    assert_eq!(m.faces.len(), 7);
    let vs = m.vertices_of(&[EntityRef::Edge(ridge[0])]);
    m.transform_vertices(&vs, &DMat4::from_translation(v(0., 0., 30.)));
    ok(&m);
    assert_eq!(m.faces.len(), 7);
    assert_closed_consistent(&m);
    // Front gable is one pentagon.
    let front = face_with_normal(&m, DVec3::NEG_Y);
    assert_eq!(front.len(), 1);
    assert_eq!(m.faces[front[0]].outer.len(), 5);
    let expected = 100. * 60. * 50. + 0.5 * 100. * 30. * 60.;
    assert!((volume(&m) - expected).abs() < 1e-6);
}

#[test]
fn circle_pocket_on_box_top_has_soft_walls() {
    let mut m = Mesh::new();
    make_box(&mut m, 100., 100., 50.);
    let pts: Vec<DVec3> = (0..24)
        .map(|i| {
            let a = i as f64 / 24.0 * std::f64::consts::TAU;
            v(50. + a.cos() * 20.0, 50. + a.sin() * 20.0, 50.0)
        })
        .collect();
    m.add_polyline(&pts, true, true);
    ok(&m);
    let disc = m.faces.iter().find(|(_, f)| f.normal.z > 0.9 && f.holes.is_empty() && f.outer.len() == 24).unwrap().0;
    m.push_pull(disc, -20.0, false).unwrap();
    ok(&m);
    assert_closed_consistent(&m);
    let soft = m.edges.values().filter(|e| e.soft && e.smooth).count();
    assert_eq!(soft, 24);
}

#[test]
fn cylinder_through_hole() {
    let mut m = Mesh::new();
    make_box(&mut m, 100., 100., 50.);
    let pts: Vec<DVec3> = (0..12)
        .map(|i| {
            let a = i as f64 / 12.0 * std::f64::consts::TAU;
            v(50. + a.cos() * 20.0, 50. + a.sin() * 20.0, 50.0)
        })
        .collect();
    m.add_polyline(&pts, true, true);
    let disc = m.faces.iter().find(|(_, f)| f.normal.z > 0.9 && f.holes.is_empty() && f.outer.len() == 12).unwrap().0;
    m.push_pull(disc, -50.0, false).unwrap();
    ok(&m);
    assert_closed_consistent(&m);
    assert_eq!(m.faces.values().filter(|f| f.holes.len() == 1).count(), 2);
    assert_eq!(m.faces.len(), 4 + 2 + 12);
}

#[test]
fn corner_cut_splits_into_triangle_and_pentagon() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 10., 10., 0.);
    m.add_line(v(0., 3., 0.), v(3., 0., 0.));
    ok(&m);
    let mut sizes: Vec<usize> = m.faces.values().map(|f| f.outer.len()).collect();
    sizes.sort();
    assert_eq!(sizes, vec![3, 5]);
}

#[test]
fn bowtie_faces_touching_at_vertex() {
    let mut m = Mesh::new();
    m.add_face(&[v(0., 0., 1.), v(10., 0., 1.), v(10., 10., 1.)]);
    m.add_face(&[v(10., 10., 1.), v(20., 10., 1.), v(20., 20., 1.)]);
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    // Erase one face: the other is untouched.
    let f = m.faces.keys().next().unwrap();
    m.erase(&[EntityRef::Face(f)]);
    ok(&m);
    assert_eq!(m.faces.len(), 1);
}

#[test]
fn arc_across_face_splits_it() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 0.);
    let pts: Vec<DVec3> = (0..=8)
        .map(|i| {
            let a = std::f64::consts::PI * i as f64 / 8.0;
            v(50. + 30. * a.cos(), 30. * a.sin(), 0.)
        })
        .collect();
    let es = m.add_polyline(&pts, false, true);
    ok(&m);
    assert_eq!(es.len(), 8);
    assert_eq!(m.faces.len(), 2);
    assert!((total_area(&m) - 10000.).abs() < 1e-6);
    // Erasing one arc piece merges the faces back (healing only collinear vertices).
    m.erase(&[EntityRef::Edge(es[3])]);
    ok(&m);
    assert_eq!(m.faces.len(), 1);
}

#[test]
fn push_ground_face_downward() {
    let mut m = Mesh::new();
    let f = ground_rect(&mut m, 10., 10.);
    m.push_pull(f, 5.0, false).unwrap(); // along -Z normal: below ground
    ok(&m);
    assert_closed_consistent(&m);
    assert_outward(&m);
    assert!((m.bounds().unwrap().min.z + 5.).abs() < 1e-9);
}

#[test]
fn rectangle_on_wall_inherits_orientation() {
    let mut m = Mesh::new();
    make_box(&mut m, 100., 100., 100.);
    // Points given clockwise as seen from the front: the new face still faces outward (-Y).
    let f = m.add_face(&[v(20., 0., 20.), v(20., 0., 60.), v(60., 0., 60.), v(60., 0., 20.)]).unwrap();
    ok(&m);
    assert!(m.faces[f].normal.y < -0.999);
    assert_eq!(face_with_normal(&m, DVec3::NEG_Y).len(), 2);
}

fn split_floor(m: &mut Mesh) -> (FaceId, FaceId) {
    rect_lines(m, 0., 0., 100., 100., 0.);
    m.add_line(v(50., 0., 0.), v(50., 100., 0.));
    let l = m.faces.iter().find(|(id, _)| face_centroid(m, *id).x < 50.).unwrap().0;
    let r = m.faces.iter().find(|(id, _)| face_centroid(m, *id).x > 50.).unwrap().0;
    (l, r)
}

#[test]
fn split_floor_rooms_lower_second_room() {
    let mut m = Mesh::new();
    let (l, r) = split_floor(&mut m);
    let top = m.push_pull(l, -100., false).unwrap();
    assert!(m.faces[top].normal.z > 0.99);
    assert!(m.faces.contains_key(l) && m.faces[l].normal.z < -0.99, "base kept, facing down");
    assert!(m.faces.contains_key(r), "neighbour floor untouched");
    m.push_pull(r, -40., false).unwrap();
    ok(&m);
    assert_closed_consistent(&m);
    assert!((volume(&m) - (50. * 100. * 100. + 50. * 100. * 40.)).abs() < 1e-6);
    // The remaining part of the shared wall (z 40..100) faces +X, outward of the taller room.
    let walls: Vec<FaceId> = face_with_normal(&m, DVec3::X).into_iter().filter(|f| (face_centroid(&m, *f).x - 50.).abs() < 1e-6).collect();
    assert_eq!(walls.len(), 1);
    assert!((m.face_area(walls[0]) - 100. * 60.).abs() < 1e-6);
}

#[test]
fn split_floor_rooms_same_height_become_one_box() {
    let mut m = Mesh::new();
    let (l, r) = split_floor(&mut m);
    m.push_pull(l, -100., false).unwrap();
    m.push_pull(r, -100., false).unwrap();
    ok(&m);
    assert_closed_consistent(&m);
    assert!((volume(&m) - 100. * 100. * 100.).abs() < 1e-6);
    // Floor and roof keep the dividing line; the side walls merge.
    assert_eq!(m.faces.len(), 8);
}

#[test]
fn coordinates_have_no_float_noise() {
    let mut m = Mesh::new();
    let a = DVec3::new(-4.5e-13, 9.1e-13, 0.);
    m.add_face(&[a, v(6000., 0., 0.), v(6000., 4000.000000000001, 0.), v(0., 4000., 0.)]);
    ok(&m);
    for vx in m.vertices.values() {
        for c in vx.pos.to_array() {
            assert!(c == 0. || c == 6000. || c == 4000., "noisy coordinate {c}");
        }
    }
    // Rotations come back clean too.
    let vs: std::collections::HashSet<_> = m.vertices.keys().collect();
    m.transform_vertices(&vs, &DMat4::from_rotation_z(std::f64::consts::FRAC_PI_2));
    assert!(m.vertices.values().all(|v| v.pos.to_array().iter().all(|c| *c == c.round())));
}

/// Do two edges touch anywhere other than at a shared endpoint?
fn edges_touch_without_vertex(m: &Mesh) -> bool {
    let segs: Vec<(DVec3, DVec3)> = m.edges.values().map(|e| (m.vertices[e.v[0]].pos, m.vertices[e.v[1]].pos)).collect();
    let dist = |p: DVec3, a: DVec3, b: DVec3| {
        let d = b - a;
        let t = ((p - a).dot(d) / d.length_squared()).clamp(0.0, 1.0);
        (a + d * t).distance(p)
    };
    let interior = |p: DVec3, a: DVec3, b: DVec3| dist(p, a, b) < 1e-4 && p.distance(a) > 1e-4 && p.distance(b) > 1e-4;
    for (i, &(a, b)) in segs.iter().enumerate() {
        for &(c, d) in &segs[i + 1..] {
            if interior(c, a, b) || interior(d, a, b) || interior(a, c, d) || interior(b, c, d) {
                return true;
            }
            // Proper crossing: sample the closest points.
            let (d1, d2, r) = (b - a, d - c, a - c);
            let (aa, ee, bb, cc, ff) = (d1.dot(d1), d2.dot(d2), d1.dot(d2), d1.dot(r), d2.dot(r));
            let den = aa * ee - bb * bb;
            if den <= 1e-9 * aa * ee {
                continue;
            }
            let s = ((bb * ff - cc * ee) / den).clamp(0.0, 1.0);
            let t = ((bb * s + ff) / ee).clamp(0.0, 1.0);
            let (p, q) = (a + d1 * s, c + d2 * t);
            if p.distance(q) < 1e-4 && interior(p, a, b) && interior(q, c, d) {
                return true;
            }
        }
    }
    false
}

fn no_edge_used_twice_by_a_face(m: &Mesh) -> bool {
    m.faces.values().all(|f| {
        let mut seen = HashSet::new();
        f.loops().all(|l| (0..l.len()).all(|i| {
            let (a, b) = (l[i], l[(i + 1) % l.len()]);
            seen.insert(if a < b { (a, b) } else { (b, a) })
        }))
    })
}

#[test]
fn push_pull_splits_edges_it_crosses() {
    // A line floating at z=30 across the footprint; pulling the rectangle up to z=30 makes the
    // top edges cross it: they must be split at the crossings (and the line splits the top).
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 0.);
    m.add_line(v(50., -50., 30.), v(50., 150., 30.));
    let f = face_with_normal(&m, DVec3::NEG_Z)[0];
    m.push_pull(f, -30.0, false).unwrap();
    ok(&m);
    assert!(!edges_touch_without_vertex(&m));
    let tops = face_with_normal(&m, DVec3::Z);
    assert_eq!(tops.len(), 2, "the line splits the new top");
    assert!(tops.iter().all(|t| (m.face_area(*t) - 5000.).abs() < 1e-6));
}

#[test]
fn push_pull_side_onto_dangling_line_t_junction() {
    // Stretching a box side so its new top edge ends on an existing line (T-junction).
    let mut m = Mesh::new();
    make_box(&mut m, 100., 100., 30.);
    m.add_line(v(150., 20., 30.), v(150., 80., 30.));
    m.add_line(v(150., 50., 30.), v(250., 50., 30.));
    let side = face_with_normal(&m, DVec3::X)[0];
    m.push_pull(side, 100.0, false).unwrap();
    ok(&m);
    assert!(!edges_touch_without_vertex(&m));
}

#[test]
fn push_pull_stretched_face_does_not_fold_onto_itself() {
    // A ground face around a pocket; sliding the pocket wall onto the ground face's outer
    // edge collapses the strip between them: the ground face must become two faces, not one
    // face whose loop runs along an edge twice.
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 300., 200., 0.);
    rect_lines(&mut m, 100., 100., 200., 200., 0.);
    let inner = m.faces.iter().find(|(id, _)| (m.face_area(*id) - 10_000.).abs() < 1e-6).unwrap().0;
    let d = if m.faces[inner].normal.z < 0.0 { 50.0 } else { -50.0 };
    m.push_pull(inner, d, false).unwrap();
    ok(&m);
    let wall = m
        .faces
        .iter()
        .find(|(id, f)| f.normal.y.abs() > 0.999 && (face_centroid(&m, *id).y - 100.).abs() < 1e-6)
        .unwrap();
    let (wall, wn) = (wall.0, wall.1.normal);
    m.push_pull(wall, if wn.y < 0.0 { 100.0 } else { -100.0 }, false).unwrap();
    ok(&m);
    assert!(no_edge_used_twice_by_a_face(&m));
    assert!(!edges_touch_without_vertex(&m));
    // The original ground face (facing down) is left as two strips beside the stretched pocket.
    let ground: Vec<_> = m.faces.iter().filter(|(id, f)| f.normal.z < -0.999 && face_centroid(&m, *id).z.abs() < 1e-6).map(|(id, _)| m.face_area(id)).collect();
    assert_eq!(ground, vec![20_000.0, 20_000.0]);
}
