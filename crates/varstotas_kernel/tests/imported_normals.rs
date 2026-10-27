use std::collections::HashSet;
use varstotas_kernel::{Model, io, glam::{DMat4, DVec3}, tessellate::face_world_triangles};

fn authored_face() -> (Model, varstotas_kernel::FaceId) {
    let mut model = Model::new();
    let face = model.root_mesh_mut().add_face_exact(&[DVec3::ZERO, DVec3::X * 100.0, DVec3::Y * 100.0]).unwrap();
    model.root_mesh_mut().faces[face].normals = vec![
        DVec3::new(0.1, 0.2, 1.0).normalize(), DVec3::new(-0.2, 0.1, 1.0).normalize(), DVec3::new(0.3, -0.2, 1.0).normalize(),
    ];
    (model, face)
}

#[test]
fn authored_normals_survive_native_save_and_reversal() {
    let (mut model, face) = authored_face();
    let original = model.root_mesh().faces[face].normals.clone();
    let json = io::to_json(&model).unwrap();
    let loaded = io::from_json(&json).unwrap();
    for (actual, expected) in loaded.root_mesh().faces[face].normals.iter().zip(&original) {
        assert!(actual.distance(*expected) < 1e-12);
    }
    let (_, _, normals) = face_world_triangles(loaded.root_mesh(), face, &DMat4::IDENTITY, false);
    for (actual, expected) in normals.iter().zip(&original) { assert!(actual.distance(*expected) < 1e-12); }
    model.root_mesh_mut().reverse_face(face);
    assert_eq!(model.root_mesh().faces[face].normals, original.iter().rev().map(|n| -*n).collect::<Vec<_>>());
    model.root_mesh_mut().reverse_face(face);
    assert_eq!(model.root_mesh().faces[face].normals, original);
}

#[test]
fn whole_face_transforms_normals_and_partial_edits_invalidate_them() {
    let (mut model, face) = authored_face();
    let original = model.root_mesh().faces[face].normals.clone();
    let vertices: HashSet<_> = model.root_mesh().faces[face].outer.iter().copied().collect();
    let matrix = DMat4::from_rotation_x(0.6) * DMat4::from_scale(DVec3::new(-2.0, 3.0, 1.0));
    model.root_mesh_mut().transform_vertices(&vertices, &matrix);
    let expected: Vec<_> = original.iter().rev().map(|n| matrix.inverse().transpose().transform_vector3(*n).normalize()).collect();
    assert_eq!(model.root_mesh().faces[face].normals.len(), expected.len());
    for (actual, expected) in model.root_mesh().faces[face].normals.iter().zip(expected) {
        assert!(actual.distance(expected) < 1e-9);
    }
    let vertex = *vertices.iter().next().unwrap();
    model.root_mesh_mut().transform_vertices(&HashSet::from([vertex]), &DMat4::from_translation(DVec3::Z));
    assert!(model.root_mesh().faces.values().all(|f| f.normals.is_empty()));
}

#[test]
fn malformed_native_corner_normals_are_rejected() {
    let (mut model, face) = authored_face();
    model.root_mesh_mut().faces[face].normals.pop();
    assert!(io::from_json(&io::to_json(&model).unwrap()).is_err());
    model.root_mesh_mut().faces[face].normals = vec![DVec3::ZERO; 3];
    assert!(io::from_json(&io::to_json(&model).unwrap()).is_err());
}

#[test]
fn glb_preserves_authored_smooth_normals_instead_of_flattening_them() {
    let (model, _) = authored_face();
    let glb = io::write_glb(&model);
    let mut imported = Model::new();
    io::add_gltf(&mut imported, &glb, None, &Default::default()).unwrap();
    let original = model.root_mesh().faces.values().next().unwrap();
    let face = imported.defs.values().flat_map(|d| d.mesh.faces.values()).next().unwrap();
    assert_eq!(face.normals.len(), 3);
    for (actual, expected) in face.normals.iter().zip(&original.normals) {
        assert!(actual.distance(*expected) < 1e-6, "{actual:?} != {expected:?}");
    }
}

#[test]
fn inserting_a_vertex_on_an_authored_face_edge_interpolates_its_normal() {
    let (mut model, face) = authored_face();
    let original = model.root_mesh().faces[face].normals.clone();
    model.root_mesh_mut().add_line(DVec3::new(50.0, 0.0, 0.0), DVec3::new(50.0, -50.0, 0.0));
    let mesh = model.root_mesh();
    let f = &mesh.faces[face];
    assert_eq!(f.outer.len(), 4);
    assert_eq!(f.normals.len(), 4);
    let at = f.outer.iter().position(|v| mesh.vertices[*v].pos.distance(DVec3::new(50.0, 0.0, 0.0)) < 1e-8).unwrap();
    assert!(f.normals[at].distance((original[0] + original[1]).normalize()) < 1e-9);
    io::validate_model(&model).unwrap();
    io::from_json(&io::to_json(&model).unwrap()).unwrap();
}

#[test]
fn native_save_rejects_invalid_shading_before_overwriting_or_creating_a_backup() {
    let (mut model, face) = authored_face();
    let dir = std::env::temp_dir().join(format!("varstotas-normal-save-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("model.vrst");
    let backup = dir.join("model.bak.vrst");
    io::save(&model, &path).unwrap();
    let original = std::fs::read(&path).unwrap();
    model.root_mesh_mut().faces[face].normals.pop();
    assert!(io::save(&model, &path).is_err());
    assert!(io::save_with_backup(&model, &path, &backup).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert!(!&backup.exists());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn obj_and_collada_round_trip_authored_corner_shading() {
    let (model,face)=authored_face();
    let expected=model.root_mesh().faces[face].normals.clone();
    let dir=std::env::temp_dir().join(format!("varstotas_corner_exchange_{}",std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path=dir.join("normals.obj");io::export_obj(&model,&path).unwrap();
    let mut obj=Model::new();io::import_obj(&mut obj,&path).unwrap();
    let (dae,_)=io::write_dae(&model);let mut collada=Model::new();
    io::add_dae(&mut collada,&dae,None,&Default::default()).unwrap();
    for (imported,tolerance) in [(&obj,1e-9),(&collada,1e-4)] {
        let mesh=&imported.defs.values().find(|d|!d.mesh.faces.is_empty()).unwrap().mesh;
        let f=mesh.faces.values().next().unwrap();assert_eq!(f.normals.len(),3);
        for (actual,expected) in f.normals.iter().zip(&expected) {assert!(actual.distance(*expected)<tolerance,"{actual:?} != {expected:?}");}
        io::validate_model(imported).unwrap();
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn mesh_formats_preserve_overlaps_and_thin_layers() {
    use varstotas_kernel::{Face,Vertex,Edge};
    let mut source=Model::new();
    for z in [0.0,0.0005,0.0] {
        let mesh=source.root_mesh_mut();let points=[DVec3::new(0.,0.,z),DVec3::new(100.,0.,z),DVec3::new(0.,100.,z)];
        let vs=points.map(|pos|mesh.vertices.insert(Vertex{pos}));
        for i in 0..3 {mesh.edges.insert(Edge::new(vs[i],vs[(i+1)%3]));}
        let mut f=Face::template();f.outer=vs.to_vec();f.normal=DVec3::Z;mesh.faces.insert(f);
    }
    source.rebuild_caches();
    let dir=std::env::temp_dir().join(format!("varstotas_surface_exchange_{}",std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let opts=io::ImportOptions{merge_coplanar:false,..Default::default()};
    for ext in ["obj","stl","ply","dae","3mf","dxf","glb"] {
        let path=dir.join(format!("layers.{ext}"));
        let mut imported=Model::new();
        match ext {
            "obj"=>{io::export_obj(&source,&path).unwrap();io::import_obj_with(&mut imported,&path,&opts).unwrap();}
            "stl"=>{std::fs::write(&path,io::write_stl_binary(&source)).unwrap();io::import_stl_with(&mut imported,&path,&opts).unwrap();}
            "ply"=>{std::fs::write(&path,io::write_ply(&source)).unwrap();io::import_ply_with(&mut imported,&path,&opts).unwrap();}
            "dae"=>{let (text,_)=io::write_dae(&source);io::add_dae(&mut imported,&text,None,&opts).unwrap();}
            "3mf"=>{io::add_3mf(&mut imported,&io::write_3mf(&source),&opts).unwrap();}
            "dxf"=>{let data=io::parse_dxf(&io::write_dxf(&source)).unwrap();io::add_dxf(&mut imported,&data,&opts).unwrap();}
            "glb"=>{io::add_gltf(&mut imported,&io::write_glb(&source),None,&opts).unwrap();}
            _=>unreachable!()
        }
        let faces:Vec<_>=imported.defs.values().flat_map(|d|d.mesh.faces.values()).collect();
        assert_eq!(faces.len(),3,"{ext} discarded an overlapping or close layer");
        assert!(faces.iter().all(|f|f.outer.len()==3),"{ext} changed triangle loops");
        let tess=varstotas_kernel::tessellate(&imported);
        let min_nonzero=tess.faces.iter().flat_map(|f|&f.positions).map(|p|p.z.abs()).filter(|z|*z>1e-8).fold(f64::INFINITY,f64::min);
        assert!((min_nonzero-0.0005).abs()<1e-7,"{ext} welded layers: {min_nonzero}");
        io::validate_model(&imported).unwrap();
    }
    std::fs::remove_dir_all(dir).unwrap();
}
