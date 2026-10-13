//! QA M3 regression/edge-case tests (owner: QA). Each test documents an expected classic
//! behaviour. Tests that currently fail are reported in docs/qa/M3-report.md.
mod common;
use common::*;
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::units::{parse_length, parse_lengths};
use varstotas_kernel::*;

fn circle_pts(c: DVec3, r: f64, n: usize) -> Vec<DVec3> {
    (0..n)
        .map(|i| {
            let a = i as f64 / n as f64 * std::f64::consts::TAU;
            c + DVec3::new(a.cos() * r, a.sin() * r, 0.0)
        })
        .collect()
}

fn face_center_in_void_check(m: &Mesh) {
    // For every face of a closed solid, a point slightly behind the face (against the normal)
    // should be inside, a point slightly in front should be outside. Uses ray parity.
    let inside = |p: DVec3| -> bool {
        let dir = DVec3::new(0.3713, 0.5571, 0.7428).normalize();
        let mut n = 0;
        let mut o = p;
        for _ in 0..64 {
            match m.raycast_faces(&Ray { origin: o, dir }) {
                Some((_, t)) => {
                    n += 1;
                    o += dir * (t + 1e-6);
                }
                None => break,
            }
        }
        n % 2 == 1
    };
    for (f, face) in &m.faces {
        let tris = tessellate::triangulate_face(m, f);
        let t = tris[0];
        let c = (t[0] + t[1] + t[2]) / 3.0;
        let eps = 0.01;
        assert!(inside(c - face.normal * eps), "face {f:?} normal {:?}: point behind face is not inside", face.normal);
        assert!(!inside(c + face.normal * eps), "face {f:?} normal {:?}: point in front of face is inside", face.normal);
    }
}

// ---------------------------------------------------------------------------
// Vision scenario (kernel level)
// ---------------------------------------------------------------------------

#[test]
fn qa_vision_house_kernel_roundtrip_and_obj() {
    let mut model = Model::new();
    let brick = model.add_material("Brick", [0.7, 0.3, 0.2, 1.0]);
    let roof = model.add_material("Roof", [0.3, 0.3, 0.35, 1.0]);
    {
        let m = model.root_mesh_mut();
        let floor = ground_rect(m, 6000., 4000.);
        m.push_pull(floor, -2700., false).unwrap();
        let ridge = m.add_line(v(0., 2000., 2700.), v(6000., 2000., 2700.));
        let vs = m.vertices_of(&[EntityRef::Edge(ridge[0])]);
        m.transform_vertices(&vs, &DMat4::from_translation(v(0., 0., 1500.)));
        // Window: rectangle on the front wall, offset, push through the house.
        m.add_polyline(&[v(1000., 0., 800.), v(2200., 0., 800.), v(2200., 0., 2000.), v(1000., 0., 2000.)], true, false);
        let win = m
            .faces
            .iter()
            .find(|(_, f)| f.normal.y < -0.99 && f.holes.is_empty() && f.outer.len() == 4)
            .map(|(id, _)| id)
            .unwrap();
        let inner = m.offset_face(win, -100.).unwrap();
        m.push_pull(inner, -4000., false).unwrap();
        ok(m);
        assert_closed_consistent(m);
        face_center_in_void_check(m);
        for (_, f) in m.faces.iter_mut() {
            if f.normal.z > 0.1 && f.normal.z < 0.99 {
                f.front = Some(roof);
            } else if f.normal.z.abs() < 0.01 {
                f.front = Some(brick);
            }
        }
    }
    let before = model.root_mesh().faces.len();
    let dir = std::env::temp_dir().join("varstotas_qa_m3");
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("house.vrst");
    io::save(&model, &p).unwrap();
    let back = io::load(&p).unwrap();
    assert_eq!(back.root_mesh().faces.len(), before);
    ok(back.root_mesh());
    assert_eq!(back.materials.len(), 2);
    let painted = back.root_mesh().faces.values().filter(|f| f.front.is_some()).count();
    assert!(painted > 0);
    let (obj, mtl) = io::write_obj(&back, "house.mtl");
    assert!(obj.contains("usemtl") && mtl.contains("Kd 0.7 0.3 0.2"));
    // Every OBJ face index must be in range.
    let nv = obj.lines().filter(|l| l.starts_with("v ")).count();
    for l in obj.lines().filter(|l| l.starts_with("f ")) {
        for t in l.split_whitespace().skip(1) {
            let i: usize = t.split('/').next().unwrap().parse().unwrap();
            assert!(i >= 1 && i <= nv);
        }
    }
}

// ---------------------------------------------------------------------------
// Sticky geometry edge cases
// ---------------------------------------------------------------------------

#[test]
fn qa_line_across_vertical_wall_splits_it() {
    let mut m = Mesh::new();
    make_box(&mut m, 100., 100., 100.);
    m.add_line(v(0., 0., 50.), v(100., 0., 50.));
    ok(&m);
    assert_eq!(m.faces.len(), 7);
    assert_closed_consistent(&m);
}

#[test]
fn qa_two_segment_line_through_interior_splits_face() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 0.);
    m.add_line(v(50., 0., 0.), v(50., 50., 0.));
    assert_eq!(m.faces.len(), 1, "dangling line must not split");
    m.add_line(v(50., 50., 0.), v(100., 50., 0.));
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    let mut a: Vec<f64> = m.faces.keys().map(|f| m.face_area(f)).collect();
    a.sort_by(f64::total_cmp);
    assert!((a[0] - 2500.).abs() < 1e-6 && (a[1] - 7500.).abs() < 1e-6, "{a:?}");
}

#[test]
fn qa_overlapping_rectangles_sharing_edge_segment() {
    // Second rectangle overlaps the first and shares part of the bottom edge.
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 0.);
    rect_lines(&mut m, 50., 0., 150., 50., 0.);
    ok(&m);
    // Regions: L-shaped part of A, overlap (50..100 x 0..50), right part of B.
    assert_eq!(m.faces.len(), 3);
    assert!((total_area(&m) - (10000. + 2500.)).abs() < 1e-6, "area {}", total_area(&m));
}

#[test]
fn qa_same_rectangle_twice_is_idempotent() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 0.);
    rect_lines(&mut m, 0., 0., 100., 100., 0.);
    ok(&m);
    assert_eq!(m.faces.len(), 1);
    assert_eq!(m.edges.len(), 4);
}

#[test]
fn qa_rectangle_inside_touching_one_edge() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 0.);
    rect_lines(&mut m, 20., 0., 60., 40., 0.);
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    assert!((total_area(&m) - 10000.).abs() < 1e-6);
    assert!(m.faces.values().all(|f| f.holes.is_empty()), "touching rect must split, not make a hole");
}

#[test]
fn qa_rect_on_wall_push_through_exact_depth() {
    let mut m = Mesh::new();
    make_box(&mut m, 300., 200., 250.);
    m.add_polyline(&[v(100., 0., 50.), v(200., 0., 50.), v(200., 0., 150.), v(100., 0., 150.)], true, false);
    let win = m.faces.iter().filter(|(_, f)| f.normal.y < -0.99).min_by(|a, b| m.face_area(a.0).total_cmp(&m.face_area(b.0))).unwrap().0;
    m.push_pull(win, -200., false).unwrap();
    ok(&m);
    assert_closed_consistent(&m);
    assert_eq!(m.faces.values().filter(|f| f.holes.len() == 1).count(), 2, "through-hole: front & back get holes");
    assert!((volume(&m) - (300. * 200. * 250. - 100. * 100. * 200.)).abs() < 1e-3);
    face_center_in_void_check(&m);
}

#[test]
fn qa_rect_on_wall_push_beyond_depth_does_not_corrupt() {
    // it stops at the back face; at minimum the mesh must stay valid and closed.
    let mut m = Mesh::new();
    make_box(&mut m, 300., 200., 250.);
    m.add_polyline(&[v(100., 0., 50.), v(200., 0., 50.), v(200., 0., 150.), v(100., 0., 150.)], true, false);
    let win = m.faces.iter().filter(|(_, f)| f.normal.y < -0.99).min_by(|a, b| m.face_area(a.0).total_cmp(&m.face_area(b.0))).unwrap().0;
    let _ = m.push_pull(win, -350., false);
    ok(&m);
    assert_closed_consistent(&m);
}

#[test]
fn qa_circle_on_top_push_through_bottom() {
    let mut m = Mesh::new();
    make_box(&mut m, 100., 100., 50.);
    m.add_polyline(&circle_pts(v(50., 50., 50.), 20., 24), true, true);
    let disc = m.faces.iter().find(|(_, f)| f.normal.z > 0.9 && f.holes.is_empty() && f.outer.len() == 24).unwrap().0;
    m.push_pull(disc, -50.0, false).unwrap();
    ok(&m);
    assert_closed_consistent(&m);
    face_center_in_void_check(&m);
    // 4 sides + top/bottom with holes + 24 cylinder walls.
    assert_eq!(m.faces.len(), 6 + 24);
}

#[test]
fn qa_circle_hole_walls_are_soft() {
    let mut m = Mesh::new();
    make_box(&mut m, 100., 100., 50.);
    m.add_polyline(&circle_pts(v(50., 50., 50.), 20., 24), true, true);
    let disc = m.faces.iter().find(|(_, f)| f.normal.z > 0.9 && f.holes.is_empty() && f.outer.len() == 24).unwrap().0;
    m.push_pull(disc, -50.0, false).unwrap();
    let soft = m.edges.values().filter(|e| e.soft || e.smooth).count();
    assert_eq!(soft, 24, "the 24 vertical edges of a cylindrical hole should be soft/smooth");
}

#[test]
fn qa_erase_ridge_merges_top_faces() {
    let mut m = Mesh::new();
    make_box(&mut m, 100., 100., 100.);
    let l = m.add_line(v(0., 50., 100.), v(100., 50., 100.));
    assert_eq!(m.faces.len(), 7);
    m.erase(&[EntityRef::Edge(l[0])]);
    ok(&m);
    assert_eq!(m.faces.len(), 6);
    assert_closed_consistent(&m);
    // The split vertices on the side edges should heal (side faces back to 4 verts).
    assert!(m.faces.values().all(|f| f.outer.len() == 4), "collinear split vertices left behind");
    assert_eq!(m.vertices.len(), 8);
}

#[test]
fn qa_erase_edge_between_non_coplanar_faces_removes_both() {
    let mut m = Mesh::new();
    make_box(&mut m, 100., 100., 100.);
    let e = m.edges.iter().find(|(id, _)| {
        let (a, b) = m.edge_segment(*id);
        a.z == 100. && b.z == 100. && a.y == 0. && b.y == 0.
    }).unwrap().0;
    m.erase(&[EntityRef::Edge(e)]);
    ok(&m);
    assert_eq!(m.faces.len(), 4);
}

#[test]
fn qa_push_pull_one_of_two_adjacent_ground_faces() {
    // Two coplanar ground rectangles sharing an edge; pull one up, then the other higher.
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 0.);
    m.add_line(v(50., 0., 0.), v(50., 100., 0.));
    assert_eq!(m.faces.len(), 2);
    let left = m.faces.iter().find(|(id, _)| face_centroid(&m, *id).x < 50.).unwrap().0;
    let top_l = m.push_pull(left, -100., false).unwrap();
    ok(&m);
    let right = m.faces.iter().find(|(id, f)| f.normal.z < -0.99 && face_centroid(&m, *id).x > 50.).unwrap().0;
    m.push_pull(right, -150., false).unwrap();
    ok(&m);
    assert_closed_consistent(&m);
    face_center_in_void_check(&m);
    assert!((volume(&m) - (50. * 100. * 100. + 50. * 100. * 150.)).abs() < 1e-3, "vol {}", volume(&m));
    // Now push the lower top down to the ground: should not leave a zero-volume sliver.
    assert!(m.faces.contains_key(top_l));
}

#[test]
fn qa_push_pull_side_face_of_l_shape_adjacent_to_coplanar_face() {
    // Pull a box side flush with a neighbour: coplanar faces should merge (behaviour).
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 0.);
    m.add_line(v(50., 0., 0.), v(50., 100., 0.));
    let left = m.faces.iter().find(|(id, _)| face_centroid(&m, *id).x < 50.).unwrap().0;
    let right = m.faces.iter().find(|(id, _)| face_centroid(&m, *id).x > 50.).unwrap().0;
    m.push_pull(left, -100., false).unwrap();
    m.push_pull(right, -100., false).unwrap();
    ok(&m);
    assert_closed_consistent(&m);
    // Equal heights: tops are coplanar and share the x=50 edge. Faces may or may not merge, but
    // no internal wall may remain between the two halves.
    let internal = m.faces.values().filter(|f| f.normal.x.abs() > 0.99).filter(|f| {
        f.outer.iter().all(|vv| (m.vertices[*vv].pos.x - 50.).abs() < 1e-6)
    }).count();
    assert_eq!(internal, 0, "internal wall at x=50 left inside the solid");
}

#[test]
fn qa_push_pull_tiny_and_huge_distances() {
    let mut m = Mesh::new();
    let f = ground_rect(&mut m, 100., 100.);
    let r = m.push_pull(f, -0.0001, false);
    ok(&m);
    // A sub-tolerance push is either rejected or a no-op; it must not create slivers.
    assert!(r.is_err() || m.faces.len() == 1 || m.faces.len() == 6, "faces {}", m.faces.len());
    let mut m = Mesh::new();
    let f = ground_rect(&mut m, 100., 100.);
    m.push_pull(f, -1.0e9, false).unwrap();
    ok(&m);
    assert_closed_consistent(&m);
}

#[test]
fn qa_degenerate_inputs() {
    let mut m = Mesh::new();
    assert!(m.add_line(v(1., 1., 1.), v(1., 1., 1.)).is_empty(), "zero-length line must create nothing");
    assert!(m.add_face(&[v(0., 0., 0.), v(10., 0., 0.), v(20., 0., 0.)]).is_none());
    assert!(m.add_face(&[v(0., 0., 0.), v(0.0005, 0., 0.), v(0.0005, 0.0005, 0.)]).is_none());
    ok(&m);
    assert_eq!(m.edges.len(), 0, "degenerate inputs left {} edges", m.edges.len());
}

#[test]
fn qa_offset_face_with_hole() {
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 10.);
    rect_lines(&mut m, 40., 40., 60., 60., 10.);
    let outer = m.faces.iter().find(|(_, f)| f.holes.len() == 1).unwrap().0;
    let r = m.offset_face(outer, -5.);
    ok(&m);
    assert!(r.is_ok(), "offset of a face with a hole failed: {r:?}");
}

#[test]
fn qa_offset_too_far_inward_errors_cleanly() {
    let mut m = Mesh::new();
    let f = ground_rect(&mut m, 100., 100.);
    let before = (m.faces.len(), m.edges.len());
    assert!(m.offset_face(f, -60.).is_err());
    ok(&m);
    assert_eq!((m.faces.len(), m.edges.len()), before, "failed offset must not leave geometry");
}

#[test]
fn qa_follow_me_circle_along_straight_path_makes_pipe() {
    let mut m = Mesh::new();
    let path = m.add_line(v(0., 0., 0.), v(0., 0., 200.));
    let prof = m.add_face(&circle_pts(v(0., 0., 0.), 20., 16)).unwrap();
    m.follow_me(prof, &path).unwrap();
    ok(&m);
    assert_eq!(m.faces.len(), 2 + 16);
}

#[test]
fn qa_follow_me_profile_away_from_path() {
    // Profile not touching the path start: a classic modeller still sweeps (path relative). Must not corrupt.
    let mut m = Mesh::new();
    let p1 = m.add_line(v(0., 0., 0.), v(100., 0., 0.));
    let p2 = m.add_line(v(100., 0., 0.), v(100., 100., 0.));
    let prof = m.add_face(&[v(-50., -5., 0.), v(-50., 5., 0.), v(-50., 5., 10.), v(-50., -5., 10.)]).unwrap();
    let path: Vec<EdgeId> = p1.into_iter().chain(p2).collect();
    let _ = m.follow_me(prof, &path);
    ok(&m);
}

#[test]
fn qa_move_face_copy_array_merges_with_existing() {
    // 5x copy array of a unit box along X with spacing equal to the width: adjacent copies
    // share faces; the mesh must remain valid.
    let mut m = Mesh::new();
    make_box(&mut m, 100., 100., 100.);
    let ents: Vec<EntityRef> = m.faces.keys().map(EntityRef::Face).collect();
    for i in 1..=3 {
        m.duplicate(&ents, &DMat4::from_translation(v(100. * i as f64, 0., 0.)));
    }
    ok(&m);
}

#[test]
fn qa_group_isolation_kernel() {
    let mut model = Model::new();
    let f = ground_rect(model.root_mesh_mut(), 100., 100.);
    model.root_mesh_mut().push_pull(f, -100., false).unwrap();
    let all: Vec<EntityRef> = model.root_mesh().faces.keys().map(EntityRef::Face).collect();
    let g = model.make_group(&[], &all).unwrap();
    assert_eq!(model.root_mesh().faces.len(), 0);
    assert_eq!(model.root_mesh().edges.len(), 0, "edges left behind in root after grouping");
    // Draw in root overlapping the group: must not touch the group's mesh.
    rect_lines(model.root_mesh_mut(), 50., 50., 150., 150., 0.);
    let gm = model.mesh_at(&[g]).unwrap();
    assert_eq!(gm.faces.len(), 6);
    assert_eq!(gm.vertices.len(), 8);
    // Draw inside the group overlapping root geometry.
    model.mesh_at_mut(&[g]).unwrap().add_line(v(0., 50., 100.), v(100., 50., 100.));
    assert_eq!(model.root_mesh().faces.len(), 1);
    assert_eq!(model.mesh_at(&[g]).unwrap().faces.len(), 7);
    let out = model.explode(&[], g).unwrap();
    assert!(!out.is_empty());
    ok(model.root_mesh());
}

#[test]
fn qa_history_rapid_undo_redo() {
    let mut model = Model::new();
    let mut h = History::default();
    for i in 0..50 {
        let before = model.clone();
        model.root_mesh_mut().add_line(v(i as f64 * 10., 0., 0.), v(i as f64 * 10., 100., 0.));
        h.push("Line", before);
    }
    for _ in 0..60 {
        h.undo(&mut model);
    }
    assert_eq!(model.root_mesh().edges.len(), 0);
    for _ in 0..60 {
        h.redo(&mut model);
    }
    assert_eq!(model.root_mesh().edges.len(), 50);
}

// ---------------------------------------------------------------------------
// Units
// ---------------------------------------------------------------------------

#[test]
fn qa_units_parsing() {
    let mm = LengthUnit::Millimeter;
    let close = |a: Option<f64>, b: f64| a.is_some_and(|x| (x - b).abs() < 1e-6);
    assert!(close(parse_length("2.5m", mm), 2500.));
    assert!(close(parse_length("250cm", mm), 2500.));
    assert!(close(parse_length("1'6\"", mm), 457.2));
    assert!(close(parse_length("10'", mm), 3048.));
    assert!(close(parse_length("6\"", mm), 152.4));
    assert!(close(parse_length("2500", mm), 2500.));
    assert!(close(parse_length(" 2500 mm ", mm), 2500.));
    assert!(close(parse_length("2,5m", mm), 2500.));
    assert!(close(parse_length("2500", LengthUnit::Meter), 2_500_000.));
    assert_eq!(parse_lengths("300,200", mm), Some(vec![300., 200.]));
    assert_eq!(parse_lengths("300;200", mm), Some(vec![300., 200.]));
    assert_eq!(parse_lengths("2.5m,1.2m", mm), Some(vec![2500., 1200.]));
    assert!(parse_length("abc", mm).is_none());
    assert!(parse_length("", mm).is_none());
    assert!(parse_length("1e400", mm).is_none());
}

#[test]
fn qa_units_feet_inches_with_space() {
    let mm = LengthUnit::Millimeter;
    assert!(parse_length("1' 6\"", mm).is_some_and(|x| (x - 457.2).abs() < 1e-6));
    assert!(parse_length("5'-6\"", mm).is_some_and(|x| (x - 1676.4).abs() < 1e-6));
}

#[test]
fn qa_push_pull_one_of_two_adjacent_elevated_faces() {
    // Same as the ground case, but the split rectangle floats at z=10 (normals up).
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 100., 100., 10.);
    m.add_line(v(50., 0., 10.), v(50., 100., 10.));
    let left = m.faces.iter().find(|(id, _)| face_centroid(&m, *id).x < 50.).unwrap().0;
    m.push_pull(left, 100., false).unwrap();
    ok(&m);
    // Left block must be closed: its bottom (z=10, x<50) must still exist.
    let bottoms = m.faces.iter().filter(|(id, f)| f.normal.z.abs() > 0.99 && (face_centroid(&m, *id).z - 10.).abs() < 1e-6).count();
    assert_eq!(bottoms, 2, "both z=10 faces (left block bottom + right face) must exist");
    let tops = m.faces.iter().filter(|(id, f)| f.normal.z > 0.99 && (face_centroid(&m, *id).z - 110.).abs() < 1e-6).count();
    assert_eq!(tops, 1, "top of pulled block must face up");
}

#[test]
fn qa_push_pull_single_ground_face_reference() {
    // Control: an isolated ground face pulls into a closed box.
    let mut m = Mesh::new();
    rect_lines(&mut m, 0., 0., 50., 100., 0.);
    let f = m.faces.keys().next().unwrap();
    m.push_pull(f, -100., false).unwrap();
    ok(&m);
    assert_closed_consistent(&m);
}
