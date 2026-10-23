use varstotas_kernel::{Model, TexPlacement, io, glam::{DVec2, DVec3}, texture::face_uv_mm};

#[test]
fn arbitrary_triangle_atlas_uvs_fit_without_losing_shear() {
    let points = [DVec3::new(20.0, 30.0, 0.0), DVec3::new(730.0, 140.0, 0.0), DVec3::new(85.0, 410.0, 0.0)];
    let uvs = [DVec2::new(0.12, 0.21), DVec2::new(0.43, -0.14), DVec2::new(0.66, 0.35)];
    let tile = DVec2::new(1000.0, 800.0);
    let placement = TexPlacement::fit(&points, &uvs, DVec3::Z, tile.to_array()).unwrap();
    assert!(placement.shear.abs() > 0.1);
    for (p, uv) in points.iter().zip(uvs) {
        let mapped = face_uv_mm(*p, DVec3::Z, Some(&placement)) / tile;
        assert!(mapped.distance(uv) < 1e-10, "{mapped:?} != {uv:?}");
        assert!(placement.to_planar(mapped * tile).distance(DVec2::new(p.x, p.y)) < 1e-8);
    }
}

#[test]
fn shear_survives_native_io_and_reverse_preserves_texture_coordinates() {
    let mut model = Model::new();
    let face = model.root_mesh_mut().add_face_exact(&[DVec3::ZERO, DVec3::X * 1000.0, DVec3::Y * 1000.0]).unwrap();
    model.root_mesh_mut().faces[face].uv = Some(TexPlacement { offset: [30.0, 40.0], angle_deg: 27.0, scale: [0.5, -1.2], shear: 0.8 });
    let sample = DVec3::new(100.0, 300.0, 0.0);
    let uv = |model: &Model| { let f = &model.root_mesh().faces[face]; face_uv_mm(sample, f.normal, f.uv.as_ref()) };
    let before = uv(&model);
    let loaded = io::from_json(&io::to_json(&model).unwrap()).unwrap();
    assert!(uv(&loaded).distance(before) < 1e-8);
    model.root_mesh_mut().reverse_face(face);
    assert!(uv(&model).distance(before) < 1e-8);
    model.root_mesh_mut().reverse_face(face);
    assert!(uv(&model).distance(before) < 1e-8);
    let old: TexPlacement = serde_json::from_str(r#"{"offset":[1,2],"angle_deg":20,"scale":[2,3]}"#).unwrap();
    assert_eq!(old.shear, 0.0);
}
