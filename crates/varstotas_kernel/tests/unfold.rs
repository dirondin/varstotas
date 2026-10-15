//! Unfold (flat patterns).

mod common;
use common::*;
use varstotas_kernel::geom::{PointLoc, point_in_region, signed_area};
use varstotas_kernel::glam::{DMat4, DVec2, DVec3};
use varstotas_kernel::triangulate::triangulate;
use varstotas_kernel::unfold::{FlatLine, FlatPiece, UnfoldInput, UnfoldOptions, Unfolding, unfold, to_dxf};
use varstotas_kernel::*;

fn area(u: &Unfolding) -> f64 {
    u.sheets
        .iter()
        .flat_map(|s| &s.pieces)
        .flat_map(|p| &p.faces)
        .map(|(o, h)| signed_area(o).abs() - h.iter().map(|h| signed_area(h).abs()).sum::<f64>())
        .sum()
}

fn count(u: &Unfolding, kind: FlatLine) -> usize {
    u.sheets.iter().flat_map(|s| &s.pieces).flat_map(|p| &p.lines).filter(|l| l.2 == kind).count()
}

fn pieces(u: &Unfolding) -> Vec<&FlatPiece> {
    u.sheets.iter().flat_map(|s| &s.pieces).collect()
}

/// No two faces (or tabs) of all sheets overlap: triangle centroids of each lie outside the others.
fn no_overlap(u: &Unfolding) {
    for s in &u.sheets {
        let mut regions: Vec<(Vec<DVec2>, Vec<Vec<DVec2>>)> = Vec::new();
        for p in &s.pieces {
            regions.extend(p.faces.iter().cloned());
            regions.extend(p.tabs.iter().map(|t| (t.clone(), Vec::new())));
        }
        for (i, (o, h)) in regions.iter().enumerate() {
            let holes: Vec<&[DVec2]> = h.iter().map(|h| h.as_slice()).collect();
            let all: Vec<DVec2> = o.iter().chain(h.iter().flatten()).copied().collect();
            for t in triangulate(o, &holes) {
                let c = (all[t[0]] + all[t[1]] + all[t[2]]) / 3.0;
                for (j, (o2, h2)) in regions.iter().enumerate() {
                    if i != j {
                        assert_ne!(point_in_region(c, o2, h2, 1e-6), PointLoc::Inside, "regions {i} and {j} overlap at {c}");
                    }
                }
            }
        }
    }
}

fn cube(size: f64) -> Model {
    let mut model = Model::new();
    make_box(model.root_mesh_mut(), size, size, size);
    model
}

#[test]
fn cube_unfolds_to_one_cross_with_tabs_and_numbers() {
    let model = cube(100.0);
    let input = UnfoldInput::from_model(&model, &[], &[]);
    assert_eq!(input.faces.len(), 6);
    let u = unfold(&input, &UnfoldOptions::default());
    assert_eq!((u.faces, u.pieces, u.sheets.len()), (6, 1, 1));
    assert!((area(&u) - 60_000.0).abs() < 1e-6, "{}", area(&u));
    // 12 edges: 5 folds of the net + 7 tabs fold the other way round onto the faces.
    let p = pieces(&u)[0];
    assert_eq!(p.faces.len(), 6);
    assert_eq!(p.tabs.len(), 7);
    assert_eq!(count(&u, FlatLine::Mountain), 12);
    assert_eq!(count(&u, FlatLine::Valley), 0);
    // Each cut edge: the plain side + three tab sides.
    assert_eq!(count(&u, FlatLine::Cut), 7 + 7 * 3);
    let mut nums: Vec<String> = p.labels.iter().map(|l| l.text.clone()).collect();
    nums.sort();
    nums.dedup();
    assert_eq!(nums.len(), 7);
    assert_eq!(p.labels.len(), 14);
    no_overlap(&u);
    // On the sheet: positive coordinates, within the sheet.
    let (lo, hi) = p.bounds();
    assert!(lo.min_element() >= -1e-9 && hi.x <= u.sheets[0].size.x + 1e-9 && hi.y <= u.sheets[0].size.y + 1e-9);
}

#[test]
fn scale_tabs_and_numbers_options() {
    let model = cube(100.0);
    let input = UnfoldInput::from_model(&model, &[], &[]);
    let opts = UnfoldOptions { scale: 0.5, tabs: false, numbers: false, ..Default::default() };
    let u = unfold(&input, &opts);
    assert!((area(&u) - 15_000.0).abs() < 1e-6);
    assert!(pieces(&u)[0].tabs.is_empty() && pieces(&u)[0].labels.is_empty());
    assert_eq!(count(&u, FlatLine::Mountain), 5);
    assert_eq!(count(&u, FlatLine::Cut), 14);
}

#[test]
fn concave_edges_fold_as_valleys() {
    // Floor and wall at x = 0 (z 10..110), x ≥ 0: fronts up and +X form an inside corner
    // (valley), fronts down and -X an outside one (mountain). Mixed orientations are unfolded
    // too (one face mirrored), with a single fold.
    for flip in [false, true] {
        let mut model = Model::new();
        let m = model.root_mesh_mut();
        let mut floor = vec![v(0., 0., 10.), v(100., 0., 10.), v(100., 100., 10.), v(0., 100., 10.)];
        let mut wall = vec![v(0., 0., 10.), v(0., 100., 10.), v(0., 100., 110.), v(0., 0., 110.)];
        if flip {
            floor.reverse();
            wall.reverse();
        }
        m.add_face(&floor).unwrap();
        m.add_face(&wall).unwrap();
        let input = UnfoldInput::from_model(&model, &[], &[]);
        let fz = input.faces.iter().find(|f| f.normal.z.abs() > 0.5).unwrap().normal.z;
        let wx = input.faces.iter().find(|f| f.normal.x.abs() > 0.5).unwrap().normal.x;
        let u = unfold(&input, &UnfoldOptions { tabs: false, ..Default::default() });
        assert_eq!(u.pieces, 1);
        assert_eq!(count(&u, FlatLine::Cut), 6);
        let (mt, vl) = (count(&u, FlatLine::Mountain), count(&u, FlatLine::Valley));
        match (fz > 0.0, wx > 0.0) {
            (true, true) => assert_eq!((mt, vl), (0, 1)),
            (false, false) => assert_eq!((mt, vl), (1, 0)),
            _ => assert_eq!(mt + vl, 1),
        }
        no_overlap(&u);
    }
    // L-shaped prism: area kept, no overlaps.
    let mut model = Model::new();
    let m = model.root_mesh_mut();
    let f = m
        .add_face(&[v(0., 0., 0.), v(200., 0., 0.), v(200., 100., 0.), v(100., 100., 0.), v(100., 200., 0.), v(0., 200., 0.)])
        .unwrap();
    m.push_pull(f, -100.0, false).unwrap();
    ok(m);
    let u = unfold(&UnfoldInput::from_model(&model, &[], &[]), &UnfoldOptions::default());
    let total = total_area(model.root_mesh());
    assert!((area(&u) - total).abs() < 1e-6 * total);
    no_overlap(&u);
}

#[test]
fn cylinder_soft_sides_bend_without_lines() {
    let mut model = Model::new();
    let m = model.root_mesh_mut();
    m.add_curve(&CurveInfo::circle(v(0., 0., 0.), DVec3::Z, DVec3::X, 100.0, 24).unwrap());
    let f = m.faces.keys().next().unwrap();
    m.push_pull(f, 300.0, false).unwrap();
    let input = UnfoldInput::from_model(&model, &[], &[]);
    assert_eq!(input.faces.len(), 26);
    assert!(input.soft.len() >= 24, "soft side edges: {}", input.soft.len());
    let u = unfold(&input, &UnfoldOptions { tabs: false, ..Default::default() });
    no_overlap(&u);
    let total = total_area(model.root_mesh());
    assert!((area(&u) - total).abs() < 1e-6 * total);
    // Only the two cap folds (and no lines along the side strip).
    let folds = count(&u, FlatLine::Mountain) + count(&u, FlatLine::Valley);
    assert_eq!(folds, 2, "{} pieces", u.pieces);
    let with = unfold(&input, &UnfoldOptions { tabs: false, soft_folds: true, ..Default::default() });
    assert!(count(&with, FlatLine::Mountain) > folds);
}

#[test]
fn sheets_pack_pieces_and_flag_oversize() {
    // Two separate cubes (groups) → two pieces.
    let mut model = Model::new();
    for x in [0.0, 500.0] {
        let m = model.root_mesh_mut();
        box_at(m, v(x, 0., 0.), v(x + 25., 25., 25.));
    }
    let ents: Vec<EntityRef> = model.root_mesh().faces.keys().map(EntityRef::Face).collect();
    let input = UnfoldInput::from_model(&model, &[], &ents);
    let a4 = DVec2::new(277.0, 190.0);
    let u = unfold(&input, &UnfoldOptions { sheet: Some(a4), ..Default::default() });
    assert_eq!((u.pieces, u.sheets.len(), u.oversize), (2, 1, 0));
    assert_eq!(u.sheets[0].size, a4);
    no_overlap(&u);
    // 1:1 cube of 300 mm cannot fit on A4.
    let big = cube(300.0);
    let u = unfold(&UnfoldInput::from_model(&big, &[], &[]), &UnfoldOptions { sheet: Some(a4), ..Default::default() });
    assert_eq!(u.oversize, 1);
    // Many small pieces spill over to more sheets.
    let mut many = Model::new();
    for i in 0..12 {
        let x = i as f64 * 200.0;
        box_at(many.root_mesh_mut(), v(x, 0., 0.), v(x + 60., 60., 60.));
    }
    let u = unfold(&UnfoldInput::from_model(&many, &[], &[]), &UnfoldOptions { sheet: Some(a4), ..Default::default() });
    assert_eq!(u.pieces, 12);
    assert!(u.sheets.len() > 1);
    for s in &u.sheets {
        for p in &s.pieces {
            let (lo, hi) = p.bounds();
            assert!(lo.min_element() >= -1e-9 && hi.x <= a4.x + 1e-9 && hi.y <= a4.y + 1e-9, "{lo} {hi}");
        }
    }
    no_overlap(&u);
}

#[test]
fn instances_use_their_world_transform() {
    let mut model = cube(100.0);
    let ents: Vec<EntityRef> = model.root_mesh().faces.keys().map(EntityRef::Face).collect();
    let g = model.make_group(&[], &ents).unwrap();
    model.root_mesh_mut().instances[g].transform = DMat4::from_scale(DVec3::splat(2.0)) * DMat4::from_rotation_z(0.4);
    let u = unfold(&UnfoldInput::from_model(&model, &[], &[EntityRef::Instance(g)]), &UnfoldOptions::default());
    assert!((area(&u) - 240_000.0).abs() < 1e-6, "{}", area(&u));
    assert_eq!(u.pieces, 1);
}

#[test]
fn dxf_has_layers_lines_and_labels() {
    let u = unfold(&UnfoldInput::from_model(&cube(100.0), &[], &[]), &UnfoldOptions::default());
    let d = to_dxf(&u);
    for layer in ["CUT", "MOUNTAIN", "LABELS"] {
        assert!(d.contains(&format!("8\n{layer}\n")), "{layer}");
    }
    assert_eq!(d.matches("0\nLINE\n").count(), 12 + 7 * 4);
    assert_eq!(d.matches("0\nTEXT\n").count(), 14);
    assert!(d.ends_with("0\nEOF\n"));
}

#[test]
fn a_reversed_face_still_unfolds_in_one_piece() {
    let mut model = cube(100.0);
    let f = model.root_mesh().faces.keys().next().unwrap();
    model.root_mesh_mut().reverse_face(f);
    let u = unfold(&UnfoldInput::from_model(&model, &[], &[]), &UnfoldOptions::default());
    assert_eq!(u.pieces, 1);
    assert!((area(&u) - 60_000.0).abs() < 1e-6);
    assert_eq!(pieces(&u)[0].tabs.len(), 7);
    no_overlap(&u);
}

#[test]
fn sphere_unfolds_without_overlaps() {
    let mut model = Model::new();
    let (nu, nv, r) = (16, 10, 300.0);
    let p = |i: usize, j: usize| {
        let (th, ph) = (i as f64 / nu as f64 * std::f64::consts::TAU, j as f64 / nv as f64 * std::f64::consts::PI);
        v(r * ph.sin() * th.cos(), r * ph.sin() * th.sin(), 1000.0 + r * ph.cos())
    };
    let m = model.root_mesh_mut();
    for j in 0..nv {
        for i in 0..nu {
            let (a, b, c, d) = (p(i, j), p(i + 1, j), p(i + 1, j + 1), p(i, j + 1));
            if j == 0 {
                m.add_face(&[a, d, c]).unwrap();
            } else if j == nv - 1 {
                m.add_face(&[a, d, b]).unwrap();
            } else {
                m.add_face(&[a, d, c, b]).unwrap();
            }
        }
    }
    let total = total_area(model.root_mesh());
    let input = UnfoldInput::from_model(&model, &[], &[]);
    assert_eq!(input.faces.len(), nu * nv);
    for opts in [UnfoldOptions::default(), UnfoldOptions { scale: 0.2, sheet: Some(DVec2::new(190.0, 277.0)), ..Default::default() }] {
        let u = unfold(&input, &opts);
        assert!((area(&u) - total * opts.scale * opts.scale).abs() < 1e-6 * total, "{} vs {total}", area(&u));
        assert_eq!(u.faces, nu * nv);
        no_overlap(&u);
    }
}
