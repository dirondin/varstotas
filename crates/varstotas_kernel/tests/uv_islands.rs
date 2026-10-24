//! UV islands: seams split faces into islands, unfolding lays an island flat with true
//! lengths, projections and island transforms, packing into one tile.

mod common;
use common::*;
use varstotas_kernel::glam::{DVec2, DVec3};
use varstotas_kernel::uv::{self, IslandOp, Projection};
use varstotas_kernel::*;

fn boxed() -> Mesh {
    let mut m = Mesh::default();
    box_at(&mut m, v(0., 0., 0.), v(1000., 600., 400.));
    m
}

fn apply(m: &mut Mesh, p: uv::Placements) {
    for (f, pl) in p {
        m.faces[f].uv = pl;
    }
}

/// Texture coordinates of point `p` on face `f`.
fn tex(m: &Mesh, f: FaceId, p: DVec3) -> DVec2 {
    texture::face_uv_mm(p, m.faces[f].normal, m.faces[f].uv.as_ref())
}

#[test]
fn seams_split_islands_and_unfold_keeps_lengths() {
    let mut m = boxed();
    let faces: Vec<FaceId> = m.faces.keys().collect();
    assert_eq!(uv::islands(&m, &faces).len(), 1);
    // every hard edge a seam: six islands
    let hard = uv::hard_edges(&m, &faces);
    assert_eq!(hard.len(), 12);
    for e in &hard {
        m.edges[*e].seam = true;
    }
    assert_eq!(uv::islands(&m, &faces).len(), 6);
    // open the four vertical edges' seams except one: the walls are one island of four faces
    let vertical: Vec<EdgeId> = hard
        .iter()
        .copied()
        .filter(|e| {
            let ed = &m.edges[*e];
            (m.vertices[ed.v[0]].pos.z - m.vertices[ed.v[1]].pos.z).abs() > 1.0
        })
        .collect();
    assert_eq!(vertical.len(), 4);
    for e in &vertical[1..] {
        m.edges[*e].seam = false;
    }
    let isl = uv::islands(&m, &faces);
    let walls = isl.iter().find(|i| i.len() == 4).expect("wall island").clone();
    let root = walls[0];
    let out = uv::unfold(&m, &walls, root);
    assert_eq!(out.len(), 4);
    apply(&mut m, out);
    // along a shared (non-seam) vertical edge both walls show the same texture
    for e in &vertical[1..] {
        let ed = m.edges[*e].clone();
        let fs = m.edge_faces(*e);
        for vtx in ed.v {
            let p = m.vertices[vtx].pos;
            assert!((tex(&m, fs[0], p) - tex(&m, fs[1], p)).length() < 1e-6);
        }
    }
    // the unfolded strip is 2 × (1000 + 600) long and 400 high: true lengths
    let (lo, hi) = uv::bounds(&m, &walls).unwrap();
    let size = hi - lo;
    let (long, short) = (size.x.max(size.y), size.x.min(size.y));
    assert!((long - 3200.0).abs() < 1e-6 && (short - 400.0).abs() < 1e-6, "{size}");
    assert!((uv::tex_per_world(&m, &walls) - 1.0).abs() < 1e-9);
}

#[test]
fn projections_and_island_transforms() {
    let mut m = boxed();
    let faces: Vec<FaceId> = m.faces.keys().collect();
    // box projection = the default projection everywhere
    let out = uv::project(&m, &faces, Projection::Box);
    assert!(out.iter().all(|(_, p)| p.is_none()), "{out:?}");
    // cylinder round the box: the walls wrap, u is continuous across the shared edges
    let walls: Vec<FaceId> = faces.iter().copied().filter(|f| m.faces[*f].normal.z.abs() < 0.5).collect();
    let cyl = Projection::cylinder_for(&m, &walls);
    let Projection::Cylinder { axis, .. } = cyl else { panic!() };
    assert!((axis - DVec3::Z).length() < 1e-9);
    let p = uv::project(&m, &walls, cyl);
    apply(&mut m, p);
    // the ends are projected along the axis: no degenerate (flattened) placement
    let ends: Vec<FaceId> = faces.iter().copied().filter(|f| m.faces[*f].normal.z.abs() > 0.5).collect();
    let out = uv::project(&m, &ends, cyl);
    assert!(out.iter().all(|(_, p)| p.is_none_or(|p| p.scale[0].abs() < 10.0 && p.scale[1].abs() < 10.0)), "{out:?}");
    let p = v(1000., 0., 0.);
    let at: Vec<FaceId> = walls.iter().copied().filter(|f| uv::corners(&m, *f).iter().any(|c| (*c - p).length() < 1e-9)).collect();
    assert_eq!(at.len(), 2);
    assert!((tex(&m, at[0], p) - tex(&m, at[1], p)).length() < 1e-6);
    // planar from above onto the top face = the default too
    let top = faces.iter().copied().find(|f| m.faces[*f].normal.z > 0.5).unwrap();
    let out = uv::project(&m, &[top], Projection::Planar(DVec3::Z));
    assert_eq!(out, vec![(top, None)]);
    // rotate the top island 90° about its centre; flip; scale ×2; move
    let (lo, hi) = uv::bounds(&m, &[top]).unwrap();
    let p = uv::transform(&m, &[top], IslandOp::Rotate(90.0));
    apply(&mut m, p);
    let (lo2, hi2) = uv::bounds(&m, &[top]).unwrap();
    assert!(((hi2 - lo2) - DVec2::new(hi.y - lo.y, hi.x - lo.x)).length() < 1e-6);
    assert!((((lo2 + hi2) - (lo + hi)) / 2.0).length() < 1e-6);
    let p = uv::transform(&m, &[top], IslandOp::Scale(2.0));
    apply(&mut m, p);
    assert!((uv::tex_per_world(&m, &[top]) - 2.0).abs() < 1e-9);
    let p = uv::transform(&m, &[top], IslandOp::Move(DVec2::new(10.0, 20.0)));
    apply(&mut m, p);
    let (lo3, _) = uv::bounds(&m, &[top]).unwrap();
    let c = (lo2 + hi2) / 2.0;
    assert!((lo3 - (c + (lo2 - c) * 2.0 + DVec2::new(10.0, 20.0))).length() < 1e-6);
    // fit: one tile
    let p = uv::fit_to_tile(&m, &[top], [500.0, 500.0]);
    apply(&mut m, p);
    let (lo4, hi4) = uv::bounds(&m, &[top]).unwrap();
    assert!(lo4.length() < 1e-6 && ((hi4.x - 500.0).abs() < 1e-6 || (hi4.y - 500.0).abs() < 1e-6), "{lo4} {hi4}");
    // align: a 30°-turned island comes back straight
    let p = uv::transform(&m, &[top], IslandOp::Rotate(30.0));
    apply(&mut m, p);
    let p = uv::align(&m, &[top]);
    apply(&mut m, p);
    let q = uv::face_tex(&m, top);
    for i in 0..q.len() {
        let d = q[(i + 1) % q.len()] - q[i];
        assert!(d.x.abs() < 1e-6 || d.y.abs() < 1e-6, "{d}");
    }
}

#[test]
fn pack_fits_every_island_in_one_tile_at_one_scale() {
    let mut m = boxed();
    let faces: Vec<FaceId> = m.faces.keys().collect();
    for e in uv::hard_edges(&m, &faces) {
        m.edges[e].seam = true;
    }
    let isl = uv::islands(&m, &faces);
    assert_eq!(isl.len(), 6);
    let p = uv::pack(&m, &isl, [1000.0, 1000.0], 0.02);
    apply(&mut m, p);
    let k = uv::tex_per_world(&m, &isl[0]);
    let mut boxes = Vec::new();
    for i in &isl {
        assert!((uv::tex_per_world(&m, i) - k).abs() < 1e-6, "one texel density");
        let (lo, hi) = uv::bounds(&m, i).unwrap();
        assert!(lo.x >= -1e-6 && lo.y >= -1e-6 && hi.x <= 1000.0 + 1e-6 && hi.y <= 1000.0 + 1e-6, "{lo} {hi}");
        boxes.push((lo, hi));
    }
    for a in 0..boxes.len() {
        for b in a + 1..boxes.len() {
            let (p, q) = (boxes[a], boxes[b]);
            let overlap = p.0.x < q.1.x - 1e-6 && q.0.x < p.1.x - 1e-6 && p.0.y < q.1.y - 1e-6 && q.0.y < p.1.y - 1e-6;
            assert!(!overlap, "islands {a} and {b} overlap");
        }
    }
    assert!(k > 0.15, "the tile is used well: {k}");
}

/// Randomize: the same seed repeats, another seed gives another variant; angles stay multiples of 90°.
#[test]
fn randomize_is_seeded() {
    let m = boxed();
    let f = m.faces.keys().next().unwrap();
    let j = uv::Jitter::default();
    let a = uv::randomize(&m, &[f], [800.0, 400.0], 7, j);
    assert_eq!(a, uv::randomize(&m, &[f], [800.0, 400.0], 7, j));
    let angles: std::collections::HashSet<i64> = (0..40)
        .flat_map(|s| uv::randomize(&m, &[f], [800.0, 400.0], s, j))
        .filter_map(|(_, p)| p.map(|p| p.angle_deg.rem_euclid(360.0).round() as i64 % 360))
        .collect();
    assert!(angles.len() > 1 && angles.iter().all(|a| a % 90 == 0), "{angles:?}");
}

/// Randomize with any angle and a scale range: angles off the 90° steps, scales inside the range.
#[test]
fn randomize_any_angle_and_scale() {
    let m = boxed();
    let f = m.faces.keys().next().unwrap();
    let j = uv::Jitter { any_angle: true, scale: [0.5, 2.0] };
    let all: Vec<_> = (0..40).flat_map(|s| uv::randomize(&m, &[f], [800.0, 400.0], s, j)).filter_map(|(_, p)| p).collect();
    assert!(all.iter().any(|p| p.angle_deg.rem_euclid(90.0) > 1.0), "some angle off the 90° steps");
    assert!(all.iter().all(|p| (0.5 - 1e-9..=2.0 + 1e-9).contains(&p.scale[0])), "{:?}", all.iter().map(|p| p.scale).collect::<Vec<_>>());
    assert!(all.iter().any(|p| p.scale[0] < 0.9) && all.iter().any(|p| p.scale[0] > 1.1));
}
