use varstotas_kernel::{
    Edge, EntityRef, Face, InstanceId, Model, Plane, Vertex,
    glam::{DMat4, DVec3},
    primitives::Primitive,
    printing::{
        PrintSelection,
        analysis::AnalysisOptions,
        preparation::{BedProfile, ConnectorOptions, Placement},
    },
};
fn part(m: &mut Model, size: DVec3, at: DVec3) -> InstanceId {
    let mesh = Primitive::Box {
        w: size.x,
        d: size.y,
        h: size.z,
    }
    .mesh()
    .unwrap();
    let mut definition = m.defs[m.root].clone();
    definition.mesh = mesh;
    definition.is_group = true;
    definition.name = "Print box".into();
    let d = m.defs.insert(definition);
    m.root_mesh_mut()
        .instances
        .insert(varstotas_kernel::Instance {
            def: d,
            transform: DMat4::from_translation(at),
            name: "Print box".into(),
            material: None,
            tag: None,
            hidden: false,
            locked: false,
        })
}
fn selected(i: InstanceId) -> PrintSelection {
    PrintSelection {
        path: vec![],
        entities: vec![EntityRef::Instance(i)],
    }
}
fn cube() -> (Model, InstanceId) {
    let mut m = Model::new();
    let i = part(&mut m, DVec3::splat(20.), DVec3::ZERO);
    (m, i)
}
fn json(m: &Model) -> String {
    varstotas_kernel::io::to_json(m).unwrap()
}
#[test]
fn thickness_is_world_space_sampled_and_excludes_bed_contact_overhangs() {
    let (mut m, i) = cube();
    let o = AnalysisOptions::default();
    let r = m.print_analyze(None, 1e-5, &o);
    assert_eq!(r.intersection_pairs, 0);
    assert!(r.intersections_complete && r.reliable_surface);
    assert_eq!(r.samples, r.possible_samples);
    assert_eq!(r.unknown_samples, 0);
    assert!((r.minimum_mm.unwrap() - 20.).abs() < 1e-6);
    assert_eq!(r.overhang_area_mm2, 0.);
    m.root_mesh_mut().instances[i].transform = DMat4::from_translation(DVec3::Z * 5.);
    let r = m.print_analyze(None, 1e-5, &o);
    assert!((r.overhang_area_mm2 - 400.).abs() < 1e-6);
    let r = m.print_analyze(
        None,
        1e-5,
        &AnalysisOptions {
            max_samples: 3,
            ..o
        },
    );
    assert_eq!(r.samples, 3);
    assert!(r.sampled_triangles < r.triangles);
    let mut thin = Model::new();
    let i = part(&mut thin, DVec3::new(20., 20., 0.4), DVec3::ZERO);
    let r = thin.print_analyze(None, 1e-5, &Default::default());
    assert!((r.minimum_mm.unwrap() - 0.4).abs() < 1e-6);
    assert!(r.thin_samples > 0);
    thin.root_mesh_mut().instances[i].transform = DMat4::from_scale(DVec3::new(1., 1., 3.));
    let r = thin.print_analyze(None, 1e-5, &Default::default());
    assert!((r.minimum_mm.unwrap() - 1.2).abs() < 1e-6);
    assert_eq!(r.thin_samples, 0);
}
#[test]
fn intersections_detect_overlaps_and_do_not_hide_crossings_at_shared_vertices() {
    let (mut m, _) = cube();
    part(&mut m, DVec3::splat(20.), DVec3::X * 10.);
    let r = m.print_analyze(None, 1e-5, &Default::default());
    assert!(r.intersection_pairs > 0);
    assert!(!r.reliable_surface);
    let r = m.print_analyze(
        None,
        1e-5,
        &AnalysisOptions {
            max_pairs: 1,
            ..Default::default()
        },
    );
    assert!(!r.intersections_complete);
    assert_eq!(r.candidate_pairs, 1);
    let mut m = Model::new();
    for ps in [
        [DVec3::ZERO, DVec3::X * 4., DVec3::Y * 4.],
        [DVec3::ZERO, DVec3::new(2., 2., -2.), DVec3::new(2., 2., 2.)],
    ] {
        let mesh = m.root_mesh_mut();
        let v = ps.map(|pos| mesh.vertices.insert(Vertex { pos }));
        for k in 0..3 {
            mesh.edges.insert(Edge::new(v[k], v[(k + 1) % 3]));
        }
        let mut f = Face::template();
        f.outer = v.to_vec();
        f.normal = (ps[1] - ps[0]).cross(ps[2] - ps[0]).normalize();
        mesh.faces.insert(f);
    }
    m.rebuild_caches();
    let r = m.print_analyze(None, 1e-5, &Default::default());
    assert_eq!(r.intersection_pairs, 1);
    assert!(!r.reliable_surface);
    assert!(r.unknown_samples > 0);
}
#[test]
fn scoped_export_keeps_world_placement_materials_and_separate_parts() {
    let (mut m, i) = cube();
    part(&mut m, DVec3::splat(20.), DVec3::X * 300.);
    m.root_mesh_mut().instances[i].transform = DMat4::from_translation(DVec3::new(100., 50., 25.))
        * DMat4::from_scale(DVec3::new(2., 1., 1.));
    let material = m.add_material("Red & blue", [1., 0., 0., 1.]);
    m.root_mesh_mut().instances[i].material = Some(material);
    m.metadata.text = "Print <part>".into();
    m.metadata.license = "CC0".into();
    let before = json(&m);
    let exported = m.print_export_model(Some(&selected(i))).unwrap();
    assert_eq!(exported.root_mesh().instances.len(), 1);
    let b = exported.bounds().unwrap();
    assert_eq!(b.min, DVec3::new(100., 50., 25.));
    assert_eq!(b.max, DVec3::new(140., 70., 45.));
    assert!(
        varstotas_kernel::tessellate(&exported)
            .faces
            .iter()
            .all(|f| f.front == Some(material))
    );
    assert_eq!(exported.metadata, m.metadata);
    assert_eq!(before, json(&m));
    assert_eq!(
        m.print_export_model(None)
            .unwrap()
            .root_mesh()
            .instances
            .len(),
        2
    );
}
#[test]
fn bed_layout_preserves_scale_and_fails_atomically() {
    let (mut m, _) = cube();
    part(
        &mut m,
        DVec3::new(50., 30., 10.),
        DVec3::new(100., 100., 25.),
    );
    let bed = BedProfile {
        width_mm: 100.,
        depth_mm: 100.,
        height_mm: 100.,
        ..Default::default()
    };
    let volume: f64 = m
        .print_check(None, 1e-5)
        .shells
        .iter()
        .map(|s| s.volume_mm3)
        .sum();
    m.print_place(None, &bed, Placement::Arrange).unwrap();
    let r = m.print_bed_report(None, &bed).unwrap();
    assert!(
        r.parts
            .iter()
            .all(|p| p.fits && p.bounds.min.z.abs() < 1e-6)
    );
    assert_eq!(r.bounds_overlaps, 0);
    assert!(
        (m.print_check(None, 1e-5)
            .shells
            .iter()
            .map(|s| s.volume_mm3)
            .sum::<f64>()
            - volume)
            .abs()
            < 1e-5
    );
    let before = json(&m);
    assert!(
        m.print_place(
            None,
            &BedProfile {
                width_mm: 15.,
                depth_mm: 15.,
                ..bed.clone()
            },
            Placement::Arrange
        )
        .is_err()
    );
    assert_eq!(before, json(&m));
    let i = m.root_mesh().instances.keys().next().unwrap();
    m.root_mesh_mut().instances[i].locked = true;
    let before = json(&m);
    assert!(m.print_place(None, &bed, Placement::Drop).is_err());
    assert_eq!(before, json(&m));
}
#[test]
fn picked_face_lays_its_whole_part_on_bed() {
    let (mut m, i) = cube();
    let d = m.root_mesh().instances[i].def;
    let face = m.defs[d]
        .mesh
        .faces
        .iter()
        .find(|(_, f)| f.normal.x > 0.99)
        .unwrap()
        .0;
    let before_volume = m.print_check(None, 1e-5).shells[0].volume_mm3;
    m.print_lay_face(&[i], face, &Default::default()).unwrap();
    let f = varstotas_kernel::tessellate(&m)
        .faces
        .into_iter()
        .find(|f| f.face == Some(face))
        .unwrap();
    assert!(f.normal.distance(-DVec3::Z) < 1e-6);
    assert!(f.positions.iter().all(|p| p.z.abs() < 1e-6));
    assert!((m.print_check(None, 1e-5).shells[0].volume_mm3 - before_volume).abs() < 1e-6);
}
#[test]
fn capped_splits_preserve_closed_volume_including_cavities() {
    for diagonal in [false, true] {
        let (mut m, i) = cube();
        let n = if diagonal {
            DVec3::ONE.normalize()
        } else {
            DVec3::Z
        };
        let r = m
            .print_split(
                &selected(i),
                Plane::new(DVec3::splat(10.), n),
                &Default::default(),
            )
            .unwrap();
        assert_eq!(r.parts.len(), 2);
        let report = m.print_check(None, 1e-5);
        assert!(
            report.shells.iter().all(|s| s.closed && s.consistent),
            "{:?}",
            report
                .issues
                .iter()
                .map(|i| (i.kind, i.count))
                .collect::<Vec<_>>()
        );
        assert!((report.shells.iter().map(|s| s.volume_mm3).sum::<f64>() - 8000.).abs() < 1e-5);
        assert!(report.shells.iter().all(|s| s.closed));
    }
    let (mut m, outer) = cube();
    let inner = part(&mut m, DVec3::splat(10.), DVec3::splat(5.));
    let d = m.root_mesh().instances[inner].def;
    let fs: Vec<_> = m.defs[d].mesh.faces.keys().collect();
    m.defs[d].mesh.reverse_faces(&fs);
    let group = m
        .make_group(
            &[],
            &[EntityRef::Instance(outer), EntityRef::Instance(inner)],
        )
        .unwrap();
    m.print_split(
        &selected(group),
        Plane::new(DVec3::Z * 10., DVec3::Z),
        &Default::default(),
    )
    .unwrap();
    let report = m.print_check(None, 1e-5);
    assert!(
        report
            .issues
            .iter()
            .filter(|i| i.kind != varstotas_kernel::printing::PrintIssueKind::DuplicateTriangles)
            .all(|i| i.count == 0)
    );
    assert!((report.shells.iter().map(|s| s.volume_mm3).sum::<f64>() - 7000.).abs() < 1e-5);
}
#[test]
fn sockets_and_separate_dowels_are_closed_and_rollback_when_they_do_not_fit() {
    let (mut m, i) = cube();
    let o = ConnectorOptions {
        enabled: true,
        diameter_mm: 2.,
        length_mm: 6.,
        segments: 12,
        ..Default::default()
    };
    let before = json(&m);
    assert!(
        m.print_split(
            &selected(i),
            Plane::new(DVec3::Z * 10., DVec3::Z),
            &ConnectorOptions {
                diameter_mm: 30.,
                ..o.clone()
            }
        )
        .is_err()
    );
    assert_eq!(before, json(&m));
    let r = m
        .print_split(&selected(i), Plane::new(DVec3::Z * 10., DVec3::Z), &o)
        .unwrap();
    assert_eq!((r.parts.len(), r.dowels), (4, 2));
    for i in r.parts {
        let d = m.root_mesh().instances[i].def;
        assert!(m.defs[d].mesh.is_solid());
        assert!(m.defs[d].mesh.volume() > 0.);
    }
    varstotas_kernel::io::validate_model(&m).unwrap();
    m.print_place(None, &BedProfile::default(), Placement::Arrange)
        .unwrap();
    assert_eq!(
        m.print_bed_report(None, &BedProfile::default())
            .unwrap()
            .bounds_overlaps,
        0
    );
}
#[test]
fn automatic_grid_cut_produces_bed_sized_closed_parts() {
    let mut m = Model::new();
    let i = part(&mut m, DVec3::new(450., 120., 100.), DVec3::ZERO);
    let bed = BedProfile::default();
    let r = m.print_split_to_bed(&selected(i), &bed).unwrap();
    assert_eq!(r.parts.len(), 3);
    for i in r.parts {
        let s = selected(i);
        let b = m.print_check(Some(&s), 1e-5).bounds.unwrap();
        assert!(
            (b.max - b.min)
                .cmple(bed.usable() + DVec3::splat(1e-5))
                .all()
        );
    }
    let r = m.print_check(None, 1e-5);
    assert!(
        r.issues
            .iter()
            .filter(|i| i.kind != varstotas_kernel::printing::PrintIssueKind::DuplicateTriangles)
            .all(|i| i.count == 0)
    );
    assert!((r.shells.iter().map(|s| s.volume_mm3).sum::<f64>() - 450. * 120. * 100.).abs() < 1e-3);
}
#[test]
fn print_3mf_is_valid_xml_and_preserves_precision_metadata_and_parts() {
    let (mut m, i) = cube();
    part(&mut m, DVec3::splat(2.), DVec3::new(25., 0., 0.));
    m.root_mesh_mut().instances[i].transform =
        DMat4::from_translation(DVec3::new(0.0000123456789, 0., 0.));
    m.metadata.text = "A <part> & its description".into();
    m.metadata.license = "CC BY 4.0".into();
    let mat = m.add_material("red & blue", [1., 0., 0., 1.]);
    m.root_mesh_mut().instances[i].material = Some(mat);
    let snapshot = m.print_export_model(None).unwrap();
    let xml = varstotas_kernel::io::write_3mf_model(&snapshot);
    assert!(xml.contains("x=\"0.0000123456789\""));
    assert!(xml.contains("A &lt;part&gt; &amp; its description"));
    assert!(xml.contains("<metadata name=\"LicenseTerms\">CC BY 4.0</metadata>"));
    assert!(xml.contains("red &amp; blue"));
    assert_eq!(xml.matches("<object ").count(), 2);
    assert!(!xml.contains("// i18n"));
    let mut reader = quick_xml::Reader::from_str(&xml);
    let mut objects = 0;
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Start(e)) => {
                for a in e.attributes() {
                    a.unwrap();
                }
                if e.name().as_ref() == b"object" {
                    objects += 1;
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => panic!("Invalid 3MF XML: {e}"),
            _ => {}
        }
    }
    assert_eq!(objects, 2);
}

#[test]
fn cutting_a_nested_shared_copy_preserves_other_copies_and_respects_locks() {
    let (mut m, i) = cube();
    let source_def = m.root_mesh().instances[i].def;
    let mut other = m.root_mesh().instances[i].clone();
    other.transform = DMat4::from_translation(DVec3::X * 50.);
    let other = m.root_mesh_mut().instances.insert(other);
    m.root_mesh_mut().instances[i].transform = DMat4::from_translation(DVec3::new(100., 50., 30.))
        * DMat4::from_scale(DVec3::new(2., 1., 1.));
    let parent = m.make_group(&[], &[EntityRef::Instance(i)]).unwrap();
    let inside = m
        .mesh_at(&[parent])
        .unwrap()
        .instances
        .keys()
        .next()
        .unwrap();
    let scope = PrintSelection {
        path: vec![parent],
        entities: vec![EntityRef::Instance(inside)],
    };
    m.root_mesh_mut().instances[parent].locked = true;
    let before = json(&m);
    assert!(
        m.print_split(
            &scope,
            Plane::new(DVec3::Z * 40., DVec3::Z),
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(before, json(&m));
    m.root_mesh_mut().instances[parent].locked = false;
    let result = m
        .print_split(
            &scope,
            Plane::new(DVec3::Z * 40., DVec3::Z),
            &Default::default(),
        )
        .unwrap();
    assert_eq!(result.parts.len(), 2);
    assert_eq!(m.root_mesh().instances[other].def, source_def);
    assert_eq!(m.defs[source_def].mesh.faces.len(), 6);
    let scope = PrintSelection {
        path: vec![parent],
        entities: result.parts.into_iter().map(EntityRef::Instance).collect(),
    };
    let report = m.print_check(Some(&scope), 1e-5);
    assert!(report.shells.iter().all(|s| s.closed && s.consistent));
    assert!((report.shells.iter().map(|s| s.volume_mm3).sum::<f64>() - 16000.).abs() < 1e-5);
}

#[test]
fn drop_on_bed_is_a_true_noop_and_failed_cut_preserves_the_model() {
    let (mut m, i) = cube();
    let before = json(&m);
    let result = m
        .print_place(Some(&selected(i)), &Default::default(), Placement::Drop)
        .unwrap();
    assert!(!result.changed);
    assert_eq!(before, json(&m));
    assert!(
        m.print_split(
            &selected(i),
            Plane::new(DVec3::Z * 100., DVec3::Z),
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(before, json(&m));
}
