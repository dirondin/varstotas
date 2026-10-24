//! Textures stay on the geometry: moving, rotating, mirroring, stretching, copying and
//! exploding faces keep each corner's texture coordinates.

mod common;
use common::*;
use std::collections::HashSet;
use varstotas_kernel::glam::{DMat4, DVec2, DVec3};
use varstotas_kernel::texture::face_uv_mm;
use varstotas_kernel::*;

/// A painted square with a non-trivial placement.
fn painted_square(model: &mut Model) -> FaceId {
    let mat = model.add_material("Brick", [0.8, 0.4, 0.3, 1.0]);
    let m = model.root_mesh_mut();
    let f = m.add_face(&[v(0., 0., 0.), v(1000., 0., 0.), v(1000., 600., 0.), v(0., 600., 0.)]).unwrap();
    let face = &mut m.faces[f];
    face.front = Some(mat);
    face.uv = Some(TexPlacement { offset: [120.0, -40.0], angle_deg: 17.0, scale: [1.5, 0.8], shear: 0.0 });
    f
}

/// Texture coordinates of each corner, keyed by vertex.
fn corner_uvs(m: &Mesh, f: FaceId) -> Vec<(VertexId, DVec2)> {
    let face = &m.faces[f];
    face.outer.iter().map(|v| (*v, face_uv_mm(m.vertices[*v].pos, face.normal, face.uv.as_ref()))).collect()
}

fn assert_same(a: &[(VertexId, DVec2)], b: &[(VertexId, DVec2)]) {
    for (v, uv) in a {
        let other = b.iter().find(|(w, _)| w == v).map(|x| x.1).expect("corner kept");
        assert!((other - *uv).length() < 1e-6, "{uv} became {other}");
    }
}

fn transform_all(m: &mut Mesh, xf: &DMat4) {
    let vs: HashSet<VertexId> = m.vertices.keys().collect();
    m.transform_vertices(&vs, xf);
}

#[test]
fn texture_follows_move_rotate_and_mirror() {
    for xf in [
        DMat4::from_translation(DVec3::new(250.0, -80.0, 0.0)),
        DMat4::from_rotation_z(0.6),
        // stood up: the face turns vertical, so its projection axes change
        DMat4::from_rotation_x(std::f64::consts::FRAC_PI_2) * DMat4::from_translation(DVec3::new(10.0, 0.0, 5.0)),
        DMat4::from_scale(DVec3::new(-1.0, 1.0, 1.0)),
        DMat4::from_scale(DVec3::splat(2.0)),
    ] {
        let mut model = Model::new();
        let f = painted_square(&mut model);
        let before = corner_uvs(model.root_mesh(), f);
        let m = model.root_mesh_mut();
        transform_all(m, &xf);
        ok(m);
        let g = m.faces.keys().next().unwrap();
        assert_same(&before, &corner_uvs(m, g));
    }
}

#[test]
fn texture_stretches_when_part_of_a_face_moves() {
    let mut model = Model::new();
    let f = painted_square(&mut model);
    let before = corner_uvs(model.root_mesh(), f);
    let m = model.root_mesh_mut();
    let right: HashSet<VertexId> = m.vertices.iter().filter(|(_, x)| x.pos.x > 500.0).map(|(k, _)| k).collect();
    m.transform_vertices(&right, &DMat4::from_translation(DVec3::new(400.0, 0.0, 0.0)));
    ok(m);
    assert_same(&before, &corner_uvs(m, f));
}

#[test]
fn unpainted_faces_keep_the_default_projection() {
    let mut m = Mesh::new();
    m.add_face(&[v(0., 0., 0.), v(100., 0., 0.), v(100., 100., 0.), v(0., 100., 0.)]).unwrap();
    transform_all(&mut m, &DMat4::from_translation(DVec3::new(30.0, 0.0, 0.0)));
    assert!(m.faces.values().all(|f| f.uv.is_none()));
}

#[test]
fn copies_carry_the_texture() {
    let mut model = Model::new();
    let f = painted_square(&mut model);
    let before = corner_uvs(model.root_mesh(), f);
    let m = model.root_mesh_mut();
    let out = m.duplicate(&[EntityRef::Face(f)], &DMat4::from_translation(DVec3::new(0.0, 0.0, 700.0)));
    let copy = out.iter().find_map(|e| if let EntityRef::Face(g) = e { Some(*g) } else { None }).unwrap();
    // the copy's corners show what the original's corners show
    let cu = corner_uvs(m, copy);
    let mut a: Vec<DVec2> = before.iter().map(|x| x.1).collect();
    let mut b: Vec<DVec2> = cu.iter().map(|x| x.1).collect();
    let key = |p: &DVec2| (p.x * 1e3).round() as i64 * 1_000_000 + (p.y * 1e3).round() as i64;
    a.sort_by_key(key);
    b.sort_by_key(key);
    for (x, y) in a.iter().zip(&b) {
        assert!((*x - *y).length() < 1e-6, "{x} vs {y}");
    }
}

#[test]
fn explode_keeps_the_texture_where_it_was() {
    let mut model = Model::new();
    let f = painted_square(&mut model);
    let g = model.make_group(&[], &[EntityRef::Face(f)]).unwrap();
    let xf = DMat4::from_translation(DVec3::new(500.0, 200.0, 0.0)) * DMat4::from_rotation_z(0.4);
    model.root_mesh_mut().instances[g].transform = xf;
    let def = model.root_mesh().instances[g].def;
    let inner = &model.defs[def].mesh;
    let (gf, face) = inner.faces.iter().next().unwrap();
    let _ = gf;
    // texture coordinate at each world corner before the explode
    let world: Vec<(DVec3, DVec2)> = face
        .outer
        .iter()
        .map(|v| {
            let p = inner.vertices[*v].pos;
            (xf.transform_point3(p), face_uv_mm(p, face.normal, face.uv.as_ref()))
        })
        .collect();
    model.explode(&[], g).unwrap();
    let m = model.root_mesh();
    let (_, face) = m.faces.iter().next().unwrap();
    for (p, uv) in world {
        let got = face_uv_mm(p, face.normal, face.uv.as_ref());
        assert!((got - uv).length() < 1e-6, "{uv} became {got}");
    }
}

#[test]
fn texture_wraps_round_a_corner_and_across_coplanar_faces() {
    let mut model = Model::new();
    let mat = model.add_material("Brick", [0.8, 0.4, 0.3, 1.0]);
    let m = model.root_mesh_mut();
    // an L of two walls sharing the vertical edge at x = 1000, plus a separate panel in the
    // first wall's plane
    let front = m.add_face(&[v(0., 0., 0.), v(1000., 0., 0.), v(1000., 0., 800.), v(0., 0., 800.)]).unwrap();
    let side = m.add_face(&[v(1000., 0., 0.), v(1000., 600., 0.), v(1000., 600., 800.), v(1000., 0., 800.)]).unwrap();
    let panel = m.add_face(&[v(1500., 0., 0.), v(2000., 0., 0.), v(2000., 0., 300.), v(1500., 0., 300.)]).unwrap();
    for f in [front, side, panel] {
        m.faces[f].front = Some(mat);
    }
    let uv = TexPlacement { offset: [40.0, 10.0], angle_deg: 12.0, scale: [1.3, 0.9], shear: 0.0 };
    let out = texture::wrap_placements(m, &[front, side, panel], front, Some(uv));
    assert_eq!(out.len(), 2);
    m.faces[front].uv = Some(uv);
    for (f, p) in &out {
        m.faces[*f].uv = *p;
    }
    let fu = |m: &Mesh, f: FaceId, p: DVec3| face_uv_mm(p, m.faces[f].normal, m.faces[f].uv.as_ref());
    // the shared edge shows the same texture on both walls
    for p in [v(1000., 0., 0.), v(1000., 0., 800.)] {
        assert!((fu(m, front, p) - fu(m, side, p)).length() < 1e-6);
    }
    // the side wall continues the front's texture as if unfolded: a point 600 along the side
    // matches a point 600 further along the front's u direction
    let unfolded = v(1600., 0., 0.);
    assert!((fu(m, side, v(1000., 600., 0.)) - face_uv_mm(unfolded, m.faces[front].normal, Some(&uv))).length() < 1e-6);
    // the coplanar panel shares the front's mapping
    assert!((fu(m, panel, v(1700., 0., 100.)) - face_uv_mm(v(1700., 0., 100.), m.faces[front].normal, Some(&uv))).length() < 1e-6);
}

#[test]
fn fit_rotate_flip_and_align() {
    use texture::{TexOp, align_to_edge, placement_op};
    let n = DVec3::NEG_Y;
    let pts = [v(0., 0., 0.), v(1000., 0., 0.), v(1000., 0., 500.), v(0., 0., 500.)];
    let tile = [400.0, 200.0];
    let t = |uv: Option<&TexPlacement>| -> Vec<DVec2> { pts.iter().map(|p| face_uv_mm(*p, n, uv)).collect() };
    // fit: the face's corners land on the tile's corners
    let fit = placement_op(TexOp::Fit, &pts, n, None, tile).unwrap().unwrap();
    let got = t(Some(&fit));
    let lo = got.iter().copied().reduce(DVec2::min).unwrap();
    let hi = got.iter().copied().reduce(DVec2::max).unwrap();
    assert!(lo.length() < 1e-6 && (hi - DVec2::new(400.0, 200.0)).length() < 1e-6, "{lo} {hi}");
    // flipping twice is the identity; a flip mirrors u about the centre
    let f1 = placement_op(TexOp::FlipU, &pts, n, Some(&fit), tile).unwrap();
    let a = t(f1.as_ref());
    assert!((a[0].x - 400.0).abs() < 1e-6 && (a[1].x).abs() < 1e-6);
    let f2 = placement_op(TexOp::FlipU, &pts, n, f1.as_ref(), tile).unwrap();
    assert!(t(f2.as_ref()).iter().zip(&got).all(|(x, y)| (*x - *y).length() < 1e-6));
    // four quarter turns come back
    let mut p = Some(fit);
    for _ in 0..4 {
        p = placement_op(TexOp::Rotate90, &pts, n, p.as_ref(), tile).unwrap();
    }
    assert!(t(p.as_ref()).iter().zip(&got).all(|(x, y)| (*x - *y).length() < 1e-6));
    // align to the diagonal: its start is the origin, its end lies on the u axis
    let al = align_to_edge(n, None, pts[0], pts[2]);
    let e = face_uv_mm(pts[2], n, Some(&al));
    assert!(face_uv_mm(pts[0], n, Some(&al)).length() < 1e-6 && e.y.abs() < 1e-6 && e.x > 0.0);
}
