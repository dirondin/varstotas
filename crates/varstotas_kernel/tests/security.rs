use varstotas_kernel::{Instance, Model, glam::DMat4, io};

fn dir(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("varstotas-security-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn cyclic_native_file_is_rejected_and_bounds_terminate() {
    let mut m = Model::new();
    let root = m.root;
    for _ in 0..2 {
        m.root_mesh_mut().instances.insert(Instance {
            def: root,
            transform: DMat4::IDENTITY,
            name: String::new(),
            material: None,
            tag: None,
            hidden: false,
            locked: false,
        });
    }
    assert!(io::from_json(&io::to_json(&m).unwrap()).is_err());
    assert!(m.bounds().is_none());
}

#[test]
fn resources_stay_inside_model_directory() {
    let d = dir("resources");
    let root = d.join("model");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("ok.png"), "ok").unwrap();
    std::fs::write(d.join("secret"), "secret").unwrap();
    assert!(io::resource_path(&root, "ok.png").is_ok());
    for name in [
        "../secret",
        "sub/../../secret",
        "/etc/hosts",
        "C:\\secret",
        "http://example.org/x",
    ] {
        assert!(io::resource_path(&root, name).is_err());
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(d.join("secret"), root.join("link")).unwrap();
        assert!(io::resource_path(&root, "link").is_err());
    }
    assert!(io::read_limited(&d.join("secret"), 2).is_err());
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn sidecar_collision_is_checked_before_any_export_output() {
    let d = dir("export");
    let path = d.join("part.obj");
    std::fs::write(path.with_extension("mtl"), "keep").unwrap();
    assert!(io::export_obj(&Model::new(), &path).is_err());
    assert!(!path.exists());
    assert_eq!(
        std::fs::read_to_string(path.with_extension("mtl")).unwrap(),
        "keep"
    );
    let files = io::export_bundle(&Model::new(), &path).unwrap();
    io::write_bundle(&files, true).unwrap();
    assert!(path.exists());
    std::fs::remove_dir_all(d).unwrap();
}

#[cfg(unix)]
#[test]
fn native_save_does_not_follow_destination_or_old_temp_symlinks() {
    let d = dir("save");
    let marker = d.join("marker");
    std::fs::write(&marker, "keep").unwrap();
    let path = d.join("part.vrst");
    std::os::unix::fs::symlink(&marker, path.with_extension("vrst.tmp")).unwrap();
    io::save(&Model::new(), &path).unwrap();
    assert_eq!(std::fs::read_to_string(&marker).unwrap(), "keep");
    std::fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink(&marker, &path).unwrap();
    assert!(io::save(&Model::new(), &path).is_err());
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "keep");
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn zip_rejects_declared_bombs_and_actual_size_mismatch() {
    fn zip(declared: u32) -> Vec<u8> {
        let mut b = vec![0; 30];
        b[..4].copy_from_slice(b"PK\x03\x04");
        b.extend_from_slice(b"x");
        let mut c = vec![0; 47];
        c[..4].copy_from_slice(b"PK\x01\x02");
        c[20..24].copy_from_slice(&1u32.to_le_bytes());
        c[24..28].copy_from_slice(&declared.to_le_bytes());
        c[28..30].copy_from_slice(&1u16.to_le_bytes());
        c[46] = b'a';
        b.extend(c);
        let mut e = vec![0; 22];
        e[..4].copy_from_slice(b"PK\x05\x06");
        e[10..12].copy_from_slice(&1u16.to_le_bytes());
        e[16..20].copy_from_slice(&31u32.to_le_bytes());
        b.extend(e);
        b
    }
    assert!(io::unzip(&zip(u32::MAX), |_| true).is_err());
    assert!(io::unzip(&zip(2), |_| true).is_err());
    assert_eq!(io::unzip(&zip(1), |_| true).unwrap()[0].1, b"x");
}

#[test]
fn acyclic_expansion_and_depth_are_bounded() {
    fn instance(def: varstotas_kernel::DefId) -> Instance {
        Instance { def, transform: DMat4::IDENTITY, name: String::new(), material: None, tag: None, hidden: false, locked: false }
    }
    let mut m = Model::new();
    let empty = m.defs[m.root].clone();
    let mut child = m.root;
    for _ in 0..22 {
        let mut d = empty.clone();
        d.mesh.instances.insert(instance(child));
        d.mesh.instances.insert(instance(child));
        child = m.defs.insert(d);
    }
    m.root = child;
    assert!(io::validate_model(&m).is_err());
    // Bounds remains cheap even for an exponentially expanding DAG.
    assert!(m.bounds().is_none());
    let mut m = Model::new(); let empty = m.defs[m.root].clone();
    let mut child = m.root;
    for _ in 0..66 {
        let mut d = empty.clone(); d.mesh.instances.insert(instance(child)); child = m.defs.insert(d);
    }
    m.root = child;
    assert!(io::validate_model(&m).is_err());
}

#[test]
fn obj_import_rejects_outside_texture_but_accepts_sibling() {
    let d = dir("obj-assets"); let model_dir = d.join("model");
    std::fs::create_dir(&model_dir).unwrap();
    let mut png = vec![0; 24]; png[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
    png[12..16].copy_from_slice(b"IHDR"); png[19] = 1; png[23] = 1;
    std::fs::write(d.join("secret.png"), &png).unwrap();
    std::fs::write(model_dir.join("allowed.png"), &png).unwrap();
    let path = model_dir.join("part.obj");
    std::fs::write(&path, "mtllib part.mtl\nv 0 0 0\nv 10 0 0\nv 0 10 0\nusemtl Red\nf 1 2 3\n").unwrap();
    for (asset, expected) in [("../secret.png", false), ("allowed.png", true)] {
        std::fs::write(model_dir.join("part.mtl"), format!("newmtl Red\nKd 1 0 0\nmap_Kd {asset}\n")).unwrap();
        let mut m = Model::new(); io::import_obj(&mut m, &path).unwrap();
        assert_eq!(m.materials.values().any(|m| m.image.is_some()), expected);
    }
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn late_export_collision_requires_its_own_approval() {
    let d = dir("late-export"); let path = d.join("part.obj");
    let files = io::export_bundle(&Model::new(), &path).unwrap();
    let approved = std::collections::HashSet::new();
    // The sidecar appears after the destination list was approved.
    std::fs::write(path.with_extension("mtl"), "keep").unwrap();
    assert!(io::write_bundle_approved(&files, &approved).is_err());
    assert!(!path.exists());
    assert_eq!(std::fs::read_to_string(path.with_extension("mtl")).unwrap(), "keep");
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn missing_sidecar_files_are_reported() {
    let d = dir("missing");
    let obj = d.join("chair.obj");
    std::fs::write(&obj, "mtllib chair.mtl\nv 0 0 0\nv 1 0 0\nv 0 1 0\nusemtl wood\nf 1 2 3\n").unwrap();
    io::take_missing_resources();
    io::import_obj(&mut Model::new(), &obj).unwrap();
    assert_eq!(io::take_missing_resources(), vec!["chair.mtl".to_string()]);
    // found: nothing reported, and the list was emptied by the call above
    std::fs::write(d.join("chair.mtl"), "newmtl wood\nKd 0.5 0.3 0.1\n").unwrap();
    io::import_obj(&mut Model::new(), &obj).unwrap();
    assert!(io::take_missing_resources().is_empty());
    let _ = std::fs::remove_dir_all(d);
}
