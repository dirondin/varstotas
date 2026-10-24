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
