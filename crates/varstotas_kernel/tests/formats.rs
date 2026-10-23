//! Native importers: 3MF, COLLADA, FBX, PLY (and its export), CSV points, faces as JSON.

use varstotas_kernel::glam::DVec3;
use varstotas_kernel::io::{self, ImportOptions};
use varstotas_kernel::{EntityRef, Model};

/// A 2000 × 1000 × 500 box group, painted red.
fn box_model() -> Model {
    let mut m = Model::default();
    let red = m.add_material("Red", [0.8, 0.1, 0.1, 1.0]);
    let mesh = m.root_mesh_mut();
    let f = mesh
        .add_face(&[DVec3::ZERO, DVec3::new(2000.0, 0.0, 0.0), DVec3::new(2000.0, 1000.0, 0.0), DVec3::new(0.0, 1000.0, 0.0)])
        .unwrap();
    let n = mesh.faces[f].normal;
    mesh.push_pull(f, if n.z > 0.0 { 500.0 } else { -500.0 }, false).unwrap();
    for face in mesh.faces.values_mut() {
        face.front = Some(red);
    }
    let ents: Vec<EntityRef> = mesh.faces.keys().map(EntityRef::Face).chain(mesh.edges.keys().map(EntityRef::Edge)).collect();
    let g = m.make_group(&[], &ents).unwrap();
    m.root_mesh_mut().instances[g].name = "Box".into();
    m
}

fn faces(m: &Model) -> usize {
    m.defs.values().map(|d| d.mesh.faces.len()).sum()
}

fn close(a: DVec3, b: DVec3) -> bool {
    a.distance(b) < 1e-3
}

fn tmp(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("varstotas_formats_{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d.join(name)
}

#[test]
fn three_mf_round_trip_and_components() {
    let src = box_model();
    let mut m = Model::default();
    io::add_3mf(&mut m, &io::write_3mf(&src), &ImportOptions::default()).unwrap();
    assert_eq!(faces(&m), 6);
    let b = m.bounds().unwrap();
    assert!(close(b.min, DVec3::ZERO) && close(b.max, DVec3::new(2000.0, 1000.0, 500.0)), "{b:?}");
    assert!(m.materials.values().any(|x| (x.color[0] - 0.8).abs() < 0.01));
    // A hand-made package in centimetres: an object used twice through components.
    let model = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="centimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:m="http://schemas.microsoft.com/3dmanufacturing/material/2015/02">
 <resources>
  <m:colorgroup id="5"><m:color color="#00FF00"/></m:colorgroup>
  <object id="1" type="model" pid="5" pindex="0"><mesh>
   <vertices><vertex x="0" y="0" z="0"/><vertex x="10" y="0" z="0"/><vertex x="0" y="10" z="0"/></vertices>
   <triangles><triangle v1="0" v2="1" v3="2"/></triangles>
  </mesh></object>
  <object id="2" type="model" name="Pair"><components>
   <component objectid="1"/>
   <component objectid="1" transform="1 0 0 0 1 0 0 0 1 0 0 5"/>
  </components></object>
 </resources>
 <build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 100 0 0"/></build>
</model>"##;
    let pkg = zip_stored(&[("_rels/.rels", r#"<Relationships><Relationship Target="/3D/3dmodel.model" Id="r" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#), ("3D/3dmodel.model", model)]);
    let mut m = Model::default();
    io::add_3mf(&mut m, &pkg, &ImportOptions::default()).unwrap();
    assert_eq!(faces(&m), 2);
    let b = m.bounds().unwrap();
    assert!(close(b.min, DVec3::new(1000.0, 0.0, 0.0)) && close(b.max, DVec3::new(1100.0, 100.0, 50.0)), "{b:?}");
    assert_eq!(m.root_mesh().instances.values().next().unwrap().name, "Pair");
    assert!(m.materials.values().any(|x| x.color == [0.0, 1.0, 0.0, 1.0]));
}

/// A zip with stored entries.
fn zip_stored(files: &[(&str, &str)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut dir = Vec::new();
    for (name, body) in files {
        let off = out.len() as u32;
        let (n, b) = (name.as_bytes(), body.as_bytes());
        let mut h = Vec::new();
        h.extend_from_slice(b"PK\x03\x04");
        h.extend_from_slice(&[20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        h.extend_from_slice(&0u32.to_le_bytes());
        h.extend_from_slice(&(b.len() as u32).to_le_bytes());
        h.extend_from_slice(&(b.len() as u32).to_le_bytes());
        h.extend_from_slice(&(n.len() as u16).to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&h);
        out.extend_from_slice(n);
        out.extend_from_slice(b);
        let mut c = Vec::new();
        c.extend_from_slice(b"PK\x01\x02");
        c.extend_from_slice(&[20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        c.extend_from_slice(&0u32.to_le_bytes());
        c.extend_from_slice(&(b.len() as u32).to_le_bytes());
        c.extend_from_slice(&(b.len() as u32).to_le_bytes());
        c.extend_from_slice(&(n.len() as u16).to_le_bytes());
        c.extend_from_slice(&[0; 12]);
        c.extend_from_slice(&off.to_le_bytes());
        c.extend_from_slice(n);
        dir.extend_from_slice(&c);
    }
    let at = out.len() as u32;
    out.extend_from_slice(&dir);
    out.extend_from_slice(b"PK\x05\x06\0\0\0\0");
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(dir.len() as u32).to_le_bytes());
    out.extend_from_slice(&at.to_le_bytes());
    out.extend_from_slice(&[0, 0]);
    out
}

#[test]
fn collada_y_up_centimetres() {
    let dae = r##"<?xml version="1.0"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
 <asset><unit meter="0.01"/><up_axis>Y_UP</up_axis></asset>
 <library_effects><effect id="fx"><profile_COMMON><technique sid="t"><lambert>
  <diffuse><color>0 0 1 1</color></diffuse></lambert></technique></profile_COMMON></effect></library_effects>
 <library_materials><material id="blue" name="Blue"><instance_effect url="#fx"/></material></library_materials>
 <library_geometries><geometry id="g" name="Quad"><mesh>
  <source id="pos"><float_array id="pa" count="12">0 0 0 100 0 0 100 0 -100 0 0 -100</float_array>
   <technique_common><accessor source="#pa" count="4" stride="3"/></technique_common></source>
  <vertices id="v"><input semantic="POSITION" source="#pos"/></vertices>
  <polylist material="m" count="1"><input semantic="VERTEX" source="#v" offset="0"/><vcount>4</vcount><p>0 1 2 3</p></polylist>
 </mesh></geometry></library_geometries>
 <library_visual_scenes><visual_scene id="s"><node name="Floor"><translate>10 0 0</translate>
  <instance_geometry url="#g"><bind_material><technique_common><instance_material symbol="m" target="#blue"/></technique_common></bind_material></instance_geometry>
 </node></visual_scene></library_visual_scenes>
 <scene><instance_visual_scene url="#s"/></scene>
</COLLADA>"##;
    let mut m = Model::default();
    io::add_dae(&mut m, dae, None, &ImportOptions::default()).unwrap();
    assert_eq!(faces(&m), 1);
    // Y up → Z up: -z (depth) becomes +y; centimetres → mm.
    let b = m.bounds().unwrap();
    assert!(close(b.min, DVec3::new(100.0, 0.0, 0.0)) && close(b.max, DVec3::new(1100.0, 1000.0, 0.0)), "{b:?}");
    let f = m.defs.values().flat_map(|d| d.mesh.faces.values()).next().unwrap();
    assert_eq!(m.materials[f.front.unwrap()].name, "Blue");
    assert_eq!(m.root_mesh().instances.values().next().unwrap().name, "Floor");
}

const FBX: &str = r#"; FBX 7.4.0 project file
FBXHeaderExtension:  {
	FBXHeaderVersion: 1003
	FBXVersion: 7400
}
GlobalSettings:  {
	Version: 1000
	Properties70:  {
		P: "UpAxis", "int", "Integer", "",1
		P: "UpAxisSign", "int", "Integer", "",1
		P: "FrontAxis", "int", "Integer", "",2
		P: "FrontAxisSign", "int", "Integer", "",1
		P: "CoordAxis", "int", "Integer", "",0
		P: "CoordAxisSign", "int", "Integer", "",1
		P: "UnitScaleFactor", "double", "Number", "",1
	}
}
Objects:  {
	Geometry: 1000, "Geometry::Quad", "Mesh" {
		Vertices: *12 {
			a: 0,0,0,100,0,0,100,0,-100,0,0,-100
		}
		PolygonVertexIndex: *4 {
			a: 0,1,2,-4
		}
		GeometryVersion: 124
		LayerElementMaterial: 0 {
			Version: 101
			Name: ""
			MappingInformationType: "AllSame"
			ReferenceInformationType: "IndexToDirect"
			Materials: *1 {
				a: 0
			}
		}
		Layer: 0 {
			Version: 100
			LayerElement:  {
				Type: "LayerElementMaterial"
				TypedIndex: 0
			}
		}
	}
	Model: 2000, "Model::Floor", "Mesh" {
		Version: 232
		Properties70:  {
			P: "Lcl Translation", "Lcl Translation", "", "A",10,0,0
		}
	}
	Material: 3000, "Material::Red", "" {
		Version: 102
		ShadingModel: "lambert"
		Properties70:  {
			P: "DiffuseColor", "Color", "", "A",1,0,0
		}
	}
}
Connections:  {
	C: "OO",2000,0
	C: "OO",1000,2000
	C: "OO",3000,2000
}
"#;

#[test]
fn fbx_ascii_y_up_centimetres() {
    let mut m = Model::default();
    io::add_fbx(&mut m, FBX.as_bytes(), None, &ImportOptions::default()).unwrap();
    assert_eq!(faces(&m), 1);
    let b = m.bounds().unwrap();
    assert!(close(b.min, DVec3::new(100.0, 0.0, 0.0)) && close(b.max, DVec3::new(1100.0, 1000.0, 0.0)), "{b:?}");
    let f = m.defs.values().flat_map(|d| d.mesh.faces.values()).next().unwrap();
    let mat = &m.materials[f.front.unwrap()];
    assert_eq!(mat.name, "Red");
    assert_eq!(&mat.color[..3], &[1.0, 0.0, 0.0]);
    assert_eq!(m.root_mesh().instances.values().next().unwrap().name, "Floor");
    // Not an FBX.
    assert!(io::add_fbx(&mut Model::default(), b"hello", None, &ImportOptions::default()).is_err());
}

#[test]
fn ply_ascii_colours_and_round_trip() {
    let ply = "ply\nformat ascii 1.0\ncomment a unit square in metres\nelement vertex 4\nproperty float x\nproperty float y\nproperty float z\nelement face 1\nproperty list uchar int vertex_indices\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nend_header\n0 0 0\n1 0 0\n1 1 0\n0 1 0\n4 0 1 2 3 255 0 0\n";
    let mut m = Model::default();
    io::add_ply(&mut m, ply.as_bytes(), "Square", &ImportOptions::default()).unwrap();
    assert_eq!(faces(&m), 1);
    assert!(close(m.bounds().unwrap().max, DVec3::new(1000.0, 1000.0, 0.0)));
    assert!(m.materials.values().any(|x| x.color == [1.0, 0.0, 0.0, 1.0]));
    // Binary export (mm) and back: the same box.
    let src = box_model();
    let mut back = Model::default();
    io::add_ply(&mut back, &io::write_ply(&src), "Box", &ImportOptions::default()).unwrap();
    assert_eq!(faces(&back), 6);
    assert!(close(back.bounds().unwrap().max, DVec3::new(2000.0, 1000.0, 500.0)));
    assert!(back.materials.values().any(|x| (x.color[0] - 0.8).abs() < 0.01));
    assert!(io::add_ply(&mut Model::default(), b"solid x", "x", &ImportOptions::default()).is_err());
}

#[test]
fn csv_points_and_faces_json() {
    let p = tmp("outline.csv");
    std::fs::write(&p, "x,y,z\n0,0,0\n1000,0,0\n1000,500,0\n0,500,0\n0,0,0\n").unwrap();
    let mut m = Model::default();
    assert_eq!(io::import_points_csv(&mut m, &p).unwrap(), 4);
    assert_eq!(m.root_mesh().faces.len(), 1);
    std::fs::write(&p, "1;2\n3;4;5\n").unwrap();
    let mut m = Model::default();
    io::import_points_csv(&mut m, &p).unwrap();
    assert_eq!((m.root_mesh().faces.len(), m.root_mesh().edges.len()), (0, 1));
    let v = io::faces_json(&box_model());
    let fs = v["faces"].as_array().unwrap();
    assert_eq!(fs.len(), 6);
    let area: f64 = fs.iter().map(|f| f["area"].as_f64().unwrap()).sum();
    assert!((area - 2.0 * (2000.0 * 1000.0 + 2000.0 * 500.0 + 1000.0 * 500.0)).abs() < 1e-3);
    assert_eq!(fs[0]["material"], "Red");
    assert_eq!(v["units"], "mm");
}

#[test]
fn fbx_preserves_nearby_layers_and_authored_corner_normals() {
    let fbx=FBX.replace("Vertices: *12","Vertices: *18")
        .replace("a: 0,0,0,100,0,0,100,0,-100,0,0,-100","a: 0,0,0,100,0,0,0,0,-100,0,0.00005,0,100,0.00005,0,0,0.00005,-100")
        .replace("PolygonVertexIndex: *4","PolygonVertexIndex: *6")
        .replace("a: 0,1,2,-4","a: 0,1,-3,3,4,-6")
        .replace("GeometryVersion: 124",r#"GeometryVersion: 124
        LayerElementNormal: 0 {
            Version: 101
            Name: ""
            MappingInformationType: "ByPolygonVertex"
            ReferenceInformationType: "Direct"
            Normals: *18 { a: 0.1,1,0,0,1,0.2,0,1,0,0.1,1,0,0,1,0.2,0,1,0 }
        }"#)
        .replace("Layer: 0 {",r#"Layer: 0 {
            LayerElement: { Type: "LayerElementNormal"
                TypedIndex: 0 }
        "#);
    let mut model=Model::new();
    io::add_fbx(&mut model,fbx.as_bytes(),None,&ImportOptions{merge_coplanar:false,..Default::default()}).unwrap();
    assert_eq!(faces(&model),2);
    let mesh=&model.defs.values().find(|d|!d.mesh.faces.is_empty()).unwrap().mesh;
    let z:Vec<_>=mesh.faces.values().map(|f|mesh.vertices[f.outer[0]].pos.z).collect();
    assert!((z[0]-z[1]).abs()>0.0004,"FBX layers welded: {z:?}");
    for f in mesh.faces.values() {
        assert_eq!(f.normals.len(),3);
        assert!(f.normals[0].distance(f.normals[1])>0.1,"FBX normals flattened");
        assert!(f.normals.iter().all(|n|n.z>0.9));
    }
    io::validate_model(&model).unwrap();
}

#[test]
fn ply_preserves_authored_vertex_normals() {
    let ply="ply\nformat ascii 1.0\ncomment units mm\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\nproperty float nx\nproperty float ny\nproperty float nz\nelement face 1\nproperty list uchar int vertex_indices\nend_header\n0 0 0 0.2 0 1\n100 0 0 0 0.2 1\n0 100 0 0 0 1\n3 0 1 2\n";
    let path=std::env::temp_dir().join(format!("varstotas_ply_normals_{}.ply",std::process::id()));
    std::fs::write(&path,ply).unwrap();let mut model=Model::new();io::import_ply(&mut model,&path).unwrap();
    let face=model.defs.values().flat_map(|d|d.mesh.faces.values()).next().unwrap();
    assert_eq!(face.normals.len(),3);
    assert!(face.normals[0].distance(DVec3::new(0.2,0.,1.).normalize())<1e-9);
    io::validate_model(&model).unwrap();std::fs::remove_file(path).unwrap();
}
