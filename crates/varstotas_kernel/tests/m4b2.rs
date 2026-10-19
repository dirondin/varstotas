//! M4 batch 2: arcs / curve metadata, dimensions, drawing axes, make group with guides.

mod common;
use common::*;
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::*;
use std::collections::HashSet;
use std::f64::consts::{PI, TAU};

fn close(a: DVec3, b: DVec3) -> bool {
    a.distance(b) < 1e-6
}

fn on_circle(info: &CurveInfo, p: DVec3) -> bool {
    let d = p - info.center;
    d.dot(info.normal).abs() < 1e-6 && ((d - info.normal * d.dot(info.normal)).length() - info.radius).abs() < 1e-6
}

fn polygon_area(r: f64, n: u32) -> f64 {
    0.5 * n as f64 * r * r * (TAU / n as f64).sin()
}

fn curve_id(m: &Mesh, es: &[EdgeId]) -> u32 {
    m.edges[es[0]].curve.unwrap()
}

// ---------------------------------------------------------------------------
// Arc construction helpers
// ---------------------------------------------------------------------------

#[test]
fn arc_3pt_passes_through_points() {
    let (a, p, b) = (v(0., 0., 0.), v(500., 300., 0.), v(1000., 0., 0.));
    let arc = CurveInfo::arc_3pt(a, p, b, 12).unwrap();
    assert_eq!(arc.kind, CurveKind::Arc);
    assert!(close(arc.start_point(), a) && close(arc.end_point(), b));
    assert!(on_circle(&arc, p));
    let pts = arc.points();
    assert_eq!(pts.len(), 13);
    assert!(pts.iter().all(|q| on_circle(&arc, *q)));
    assert!(close(pts[0], a) && close(pts[12], b));
    // Apex of a symmetric 3-point arc is the mid point; bulge = sagitta.
    assert!(close(arc.mid_point(), p));
    assert!((arc.bulge() - 300.0).abs() < 1e-6);
    // Other side, and reflex arcs (> 180°).
    let low = CurveInfo::arc_3pt(a, v(500., -300., 0.), b, 12).unwrap();
    assert!(close(low.mid_point(), v(500., -300., 0.)));
    let big = CurveInfo::arc_3pt(a, v(500., 900., 0.), b, 24).unwrap();
    assert!(big.sweep() > PI);
    assert!(on_circle(&big, v(500., 900., 0.)));
    // Collinear / coincident points.
    assert!(CurveInfo::arc_3pt(a, v(500., 0., 0.), b, 12).is_none());
    assert!(CurveInfo::arc_3pt(a, a, b, 12).is_none());
}

#[test]
fn arc_2pt_bulge_and_semicircle() {
    let (a, b) = (v(0., 0., 0.), v(1000., 0., 0.));
    let arc = CurveInfo::arc_2pt(a, b, DVec3::Y, 200.0, 12).unwrap();
    assert!(close(arc.mid_point(), v(500., 200., 0.)));
    assert!((arc.bulge() - 200.0).abs() < 1e-6);
    // r = (c²/4 + h²) / 2h
    assert!((arc.radius - (250_000.0 + 40_000.0) / 400.0).abs() < 1e-6);
    let neg = CurveInfo::arc_2pt(a, b, DVec3::Y, -200.0, 12).unwrap();
    assert!(close(neg.mid_point(), v(500., -200., 0.)));
    // Half circle: bulge = half chord.
    let half = CurveInfo::arc_2pt(a, b, DVec3::new(0.3, 1.0, 0.0), 500.0, 12).unwrap();
    assert!((half.sweep() - PI).abs() < 1e-9 && (half.radius - 500.0).abs() < 1e-9);
    assert!(close(half.center, v(500., 0., 0.)));
    assert!(CurveInfo::arc_2pt(a, b, DVec3::Y, 0.0, 12).is_none());
    // Vertical plane.
    let vert = CurveInfo::arc_2pt(a, b, DVec3::Z, 100.0, 8).unwrap();
    assert!(close(vert.mid_point(), v(500., 0., 100.)));
    assert!(vert.normal.z.abs() < 1e-12);
}

#[test]
fn arc_center_and_tangent_arcs() {
    let c = v(100., 100., 0.);
    let arc = CurveInfo::arc_center(c, DVec3::Z, v(200., 100., 0.), PI / 2.0, 6).unwrap();
    assert!(close(arc.start_point(), v(200., 100., 0.)) && close(arc.end_point(), v(100., 200., 0.)));
    assert!((arc.length() - 100.0 * PI / 2.0).abs() < 1e-9);
    // Clockwise sweep.
    let cw = CurveInfo::arc_center(c, DVec3::Z, v(200., 100., 0.), -PI / 2.0, 6).unwrap();
    assert!(close(cw.end_point(), v(100., 0., 0.)));
    // Start point off the plane is projected (radius in plane).
    let p = CurveInfo::arc_center(c, DVec3::Z, v(200., 100., 50.), PI, 6).unwrap();
    assert!((p.radius - 100.0).abs() < 1e-9);
    assert!(CurveInfo::arc_center(c, DVec3::Z, c, PI, 6).is_none());
    // Tangent arc: leaves `a` along the tangent and ends at `b`.
    let a = v(0., 0., 0.);
    let t = CurveInfo::arc_tangent(a, DVec3::X, v(1000., 1000., 0.), 12).unwrap();
    assert!(close(t.start_point(), a) && close(t.end_point(), v(1000., 1000., 0.)));
    assert!((t.start_tangent_out() + DVec3::X).length() < 1e-9, "{:?}", t.start_tangent_out());
    assert!((t.radius - 1000.0).abs() < 1e-6);
    assert!(close(t.center, v(0., 1000., 0.)));
    assert!(CurveInfo::arc_tangent(a, DVec3::X, v(1000., 0., 0.), 12).is_none());
    // Circle / polygon constructors.
    let circ = CurveInfo::circle(c, DVec3::Z, DVec3::X, 50.0, 24).unwrap();
    assert_eq!(circ.points().len(), 24);
    assert!((circ.length() - TAU * 50.0).abs() < 1e-9);
    assert!(CurveInfo::polygon(c, DVec3::Z, DVec3::X, 50.0, 2).is_none());
}

// ---------------------------------------------------------------------------
// Curves in the mesh
// ---------------------------------------------------------------------------

#[test]
fn add_curve_remembers_metadata_and_saves() {
    let mut m = Model::new();
    let info = CurveInfo::circle(v(0., 0., 0.), DVec3::Z, DVec3::X, 500.0, 24).unwrap();
    let es = m.root_mesh_mut().add_curve(&info);
    assert_eq!(es.len(), 24);
    let mesh = m.root_mesh();
    ok(mesh);
    assert_eq!(mesh.faces.len(), 1);
    let cid = curve_id(mesh, &es);
    assert_eq!(mesh.curve_info(cid), Some(info));
    assert_eq!(mesh.curve_of_edge(es[3]).map(|x| x.0), Some(cid));
    let (chain, closed) = mesh.curve_chain(cid).unwrap();
    assert!(closed && chain.len() == 24);
    // Save / load.
    let json = io::to_json(&m).unwrap();
    assert!(json.contains("\"curves\""));
    let back = io::from_json(&json).unwrap();
    assert_eq!(back.root_mesh().curve_info(cid).map(|i| i.segments), Some(24));
    // Files without curve metadata (and plain polylines) still load; no metadata, no info.
    assert!(!io::to_json(&Model::new()).unwrap().contains("\"curves\""));
    let mut p = Mesh::new();
    let es = p.add_polyline(&[v(0., 0., 0.), v(10., 5., 0.), v(20., 0., 0.)], false, true);
    assert!(p.curve_info(curve_id(&p, &es)).is_none());
    // A plain polyline after a curve does not reuse an id with metadata.
    let mut q = Mesh::new();
    let e1 = q.add_curve(&info);
    let e2 = q.add_polyline(&[v(900., 0., 0.), v(910., 5., 0.)], false, true);
    assert_ne!(curve_id(&q, &e1), curve_id(&q, &e2));
}

#[test]
fn curve_info_invalid_after_edits_and_follows_transforms() {
    let mut m = Mesh::new();
    let info = CurveInfo::circle(v(0., 0., 0.), DVec3::Z, DVec3::X, 100.0, 12).unwrap();
    let es = m.add_curve(&info);
    let cid = curve_id(&m, &es);
    // Moving the whole circle (and uniformly scaling it) keeps the metadata in sync.
    let all: HashSet<VertexId> = m.vertices.keys().collect();
    m.transform_vertices(&all, &DMat4::from_translation(v(10., 20., 30.)));
    let i = m.curve_info(cid).unwrap();
    assert!(close(i.center, v(10., 20., 30.)));
    m.transform_vertices(&all, &DMat4::from_scale(DVec3::splat(2.0)));
    let i = m.curve_info(cid).unwrap();
    assert!((i.radius - 200.0).abs() < 1e-9 && close(i.center, v(20., 40., 60.)));
    // Rotation into a vertical plane.
    m.transform_vertices(&all, &DMat4::from_rotation_x(PI / 2.0));
    let i = m.curve_info(cid).unwrap();
    assert!(i.normal.z.abs() < 1e-9);
    // Non-uniform scale: no longer a circle.
    m.transform_vertices(&all, &DMat4::from_scale(v(2., 1., 1.)));
    assert!(m.curve_info(cid).is_none());

    let mut m = Mesh::new();
    let es = m.add_curve(&info);
    let cid = curve_id(&m, &es);
    // Dragging one vertex invalidates it.
    let v0 = m.edges[es[0]].v[0];
    m.transform_vertices(&HashSet::from([v0]), &DMat4::from_translation(v(5., 0., 0.)));
    assert!(m.curve_info(cid).is_none());
    // Erasing one edge of an arc / circle: no longer that curve.
    let mut m = Mesh::new();
    let es = m.add_curve(&info);
    let cid = curve_id(&m, &es);
    m.erase(&[EntityRef::Edge(es[0])]);
    assert!(m.curve_info(cid).is_none());
}

#[test]
fn circle_split_by_line_through_vertices_stays_valid() {
    let mut m = Mesh::new();
    let info = CurveInfo::circle(v(0., 0., 0.), DVec3::Z, DVec3::X, 100.0, 12).unwrap();
    let es = m.add_curve(&info);
    let cid = curve_id(&m, &es);
    // A chord between two non-vertex points of the circle edges splits them.
    let a = (info.points()[0] + info.points()[1]) * 0.5;
    let b = (info.points()[6] + info.points()[7]) * 0.5;
    m.add_line(a, b);
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    assert!(m.curve_info(cid).is_some());
    // ... but it cannot be regenerated (other edges attached).
    assert!(matches!(m.set_curve(cid, None, Some(24)), Err(OpError::Invalid(_))));
}

#[test]
fn set_curve_regenerates_circle_keeping_face() {
    let mut model = Model::new();
    let red = model.add_material("red", [1., 0., 0., 1.]);
    let m = model.root_mesh_mut();
    let info = CurveInfo::circle(v(0., 0., 100.), DVec3::Z, DVec3::X, 500.0, 24).unwrap();
    let es = m.add_curve(&info);
    let cid = curve_id(m, &es);
    let f = *m.faces.keys().collect::<Vec<_>>().first().unwrap();
    m.faces[f].front = Some(red);
    let n0 = m.faces[f].normal;
    // Fewer segments.
    let es = m.set_curve(cid, None, Some(12)).unwrap();
    ok(m);
    assert_eq!(es.len(), 12);
    assert_eq!(m.edges.len(), 12);
    assert_eq!(m.vertices.len(), 12);
    assert_eq!(m.faces.len(), 1);
    assert!(m.faces.contains_key(f));
    assert_eq!(m.faces[f].front, Some(red));
    assert!((m.faces[f].normal - n0).length() < 1e-9);
    assert!((m.face_area(f) / polygon_area(500.0, 12) - 1.0).abs() < 1e-9);
    // Bigger radius, more segments.
    m.set_curve(cid, Some(800.0), Some(48)).unwrap();
    ok(m);
    assert!((m.face_area(f) / polygon_area(800.0, 48) - 1.0).abs() < 1e-9);
    let i = m.curve_info(cid).unwrap();
    assert_eq!((i.segments, i.radius), (48, 800.0));
    // Edge attributes kept.
    assert!(m.edges.values().all(|e| e.curve == Some(cid)));
    // Bad parameters.
    assert!(m.set_curve(cid, Some(0.0), None).is_err());
    assert!(m.set_curve(cid, None, Some(2)).is_err());
    assert!(m.set_curve(cid, None, Some(100_000)).is_err());
    assert!(m.set_curve(999, None, Some(5)).is_err());
}

#[test]
fn set_curve_circle_hole_in_face() {
    let mut m = Mesh::new();
    m.add_face(&[v(-1000., -1000., 0.), v(1000., -1000., 0.), v(1000., 1000., 0.), v(-1000., 1000., 0.)]).unwrap();
    let info = CurveInfo::circle(v(0., 0., 0.), DVec3::Z, DVec3::X, 300.0, 24).unwrap();
    let es = m.add_curve(&info);
    let cid = curve_id(&m, &es);
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    let total = 2000.0 * 2000.0;
    m.set_curve(cid, Some(400.0), Some(16)).unwrap();
    ok(&m);
    assert_eq!(m.faces.len(), 2);
    let areas: Vec<f64> = m.faces.keys().map(|f| m.face_area(f)).collect();
    let disc = polygon_area(400.0, 16);
    assert!(areas.iter().any(|a| (a / disc - 1.0).abs() < 1e-9), "{areas:?}");
    assert!(areas.iter().any(|a| (a / (total - disc) - 1.0).abs() < 1e-9), "{areas:?}");
    // The disc can still be push/pulled.
    let disc_f = m.faces.keys().find(|f| (m.face_area(*f) / disc - 1.0).abs() < 1e-9).unwrap();
    m.push_pull(disc_f, 200.0, false).unwrap();
    ok(&m);
    // Now the circle bounds side faces: no regeneration.
    assert!(m.curve_info(cid).is_some());
    assert!(matches!(m.set_curve(cid, None, Some(8)), Err(OpError::Invalid(_))));
}

#[test]
fn set_curve_arc_with_attached_lines() {
    // D shape: a straight line closed by an arc -> one face.
    let mut m = Mesh::new();
    let (a, b) = (v(0., 0., 0.), v(1000., 0., 0.));
    m.add_line(a, b);
    let arc = CurveInfo::arc_2pt(a, b, DVec3::Y, 300.0, 6).unwrap();
    let es = m.add_curve(&arc);
    let cid = curve_id(&m, &es);
    ok(&m);
    assert_eq!(m.faces.len(), 1);
    let f = m.faces.keys().next().unwrap();
    m.set_curve(cid, None, Some(20)).unwrap();
    ok(&m);
    assert_eq!(m.faces.len(), 1);
    assert!(m.faces.contains_key(f));
    assert_eq!(m.curve_edges(cid).len(), 20);
    assert_eq!(m.edges.len(), 21);
    // Radius change: the arc ends move (the straight line stretches with them).
    let r = arc.radius * 1.5;
    m.set_curve(cid, Some(r), None).unwrap();
    ok(&m);
    let i = m.curve_info(cid).unwrap();
    assert!((i.radius - r).abs() < 1e-9);
    assert_eq!(m.faces.len(), 1);
    let ends: Vec<DVec3> = m.edges.values().filter(|e| e.curve.is_none()).flat_map(|e| e.v).map(|x| m.vertices[x].pos).collect();
    assert!(ends.iter().any(|p| close(*p, i.start_point())) && ends.iter().any(|p| close(*p, i.end_point())));
    // Rounded-corner shape: arc between two lines in a closed loop.
    let mut m = Mesh::new();
    let fillet = CurveInfo::arc_center(v(800., 800., 0.), DVec3::Z, v(1000., 800., 0.), PI / 2.0, 4).unwrap();
    m.add_line(v(0., 0., 0.), v(1000., 0., 0.));
    m.add_line(v(1000., 0., 0.), v(1000., 800., 0.));
    let es = m.add_curve(&fillet);
    m.add_line(v(800., 1000., 0.), v(0., 1000., 0.));
    m.add_line(v(0., 1000., 0.), v(0., 0., 0.));
    ok(&m);
    assert_eq!(m.faces.len(), 1);
    let cid = curve_id(&m, &es);
    m.set_curve(cid, None, Some(16)).unwrap();
    ok(&m);
    assert_eq!(m.faces.len(), 1);
    let f = m.faces.keys().next().unwrap();
    assert_eq!(m.faces[f].outer.len(), 3 + 17);
}

#[test]
fn pie_makes_face_and_tangent_at_vertex() {
    let mut m = Mesh::new();
    let arc = CurveInfo::arc_center(v(0., 0., 0.), DVec3::Z, v(100., 0., 0.), PI / 3.0, 8).unwrap();
    let es = m.add_pie(&arc);
    ok(&m);
    assert_eq!(m.faces.len(), 1);
    assert_eq!(es.len(), 10);
    let f = m.faces.keys().next().unwrap();
    let expected = 0.5 * 8.0 * 100.0 * 100.0 * (PI / 3.0 / 8.0).sin();
    assert!((m.face_area(f) - expected).abs() < 1e-6);
    let cid = es.iter().find_map(|e| m.edges[*e].curve).unwrap();
    assert!(m.curve_info(cid).is_some());
    // Free arc: tangent at its ends.
    let mut m = Mesh::new();
    let arc = CurveInfo::arc_center(v(0., 0., 0.), DVec3::Z, v(100., 0., 0.), PI / 2.0, 6).unwrap();
    m.add_curve(&arc);
    let at = |m: &Mesh, p: DVec3| m.vertices.iter().find(|(_, x)| close(x.pos, p)).map(|(id, _)| id).unwrap();
    let t_end = m.tangent_at_vertex(at(&m, v(0., 100., 0.))).unwrap();
    assert!((t_end - (-DVec3::X)).length() < 1e-9, "{t_end:?}");
    let t_start = m.tangent_at_vertex(at(&m, v(100., 0., 0.))).unwrap();
    assert!((t_start - (-DVec3::Y)).length() < 1e-9, "{t_start:?}");
    // Straight edge end: continuation of the edge.
    m.add_line(v(500., 0., 0.), v(600., 0., 0.));
    assert!((m.tangent_at_vertex(at(&m, v(600., 0., 0.))).unwrap() - DVec3::X).length() < 1e-12);
    // Arc end with a line attached still reports the arc tangent.
    m.add_line(v(0., 100., 0.), v(-50., 100., 0.));
    let t = m.tangent_at_vertex(at(&m, v(0., 100., 0.))).unwrap();
    assert!((t + DVec3::X).length() < 1e-9);
    // Interior arc vertex: none.
    let mid = arc.points()[3];
    assert!(m.tangent_at_vertex(at(&m, mid)).is_none());
}

#[test]
fn curves_survive_copy_group_explode() {
    let mut model = Model::new();
    let info = CurveInfo::circle(v(0., 0., 0.), DVec3::Z, DVec3::X, 100.0, 12).unwrap();
    let es = model.root_mesh_mut().add_curve(&info);
    let cid = curve_id(model.root_mesh(), &es);
    // Copy (Move + Ctrl): fresh curve id with moved metadata.
    let ents: Vec<EntityRef> = es.iter().map(|e| EntityRef::Edge(*e)).collect();
    let out = model.root_mesh_mut().duplicate(&ents, &DMat4::from_translation(v(1000., 0., 0.)));
    let m = model.root_mesh();
    let c2 = out.iter().find_map(|e| if let EntityRef::Edge(e) = e { m.edges[*e].curve } else { None }).unwrap();
    assert_ne!(c2, cid);
    assert!(close(m.curve_info(c2).unwrap().center, v(1000., 0., 0.)));
    assert!(m.curve_info(cid).is_some());
    // Make group keeps the metadata; explode under a transform gives a fresh, moved curve.
    let all: Vec<EntityRef> = m.edges.keys().filter(|e| m.edges[*e].curve == Some(cid)).map(EntityRef::Edge)
        .chain(m.faces.keys().map(EntityRef::Face))
        .collect();
    let g = model.make_group(&[], &all).unwrap();
    let d = model.def_at(&[g]).unwrap();
    let dm = &model.defs[d].mesh;
    let gc = dm.edges.values().find_map(|e| e.curve).unwrap();
    assert!(dm.curve_info(gc).is_some());
    assert!(!model.root_mesh().curves.contains_key(&cid), "metadata of moved curve pruned");
    model.root_mesh_mut().instances[g].transform = DMat4::from_translation(v(0., 500., 0.)) * DMat4::from_scale(DVec3::splat(2.0));
    model.explode(&[], g).unwrap();
    let m = model.root_mesh();
    ok(m);
    let ids: HashSet<u32> = m.edges.values().filter_map(|e| e.curve).collect();
    assert_eq!(ids.len(), 2);
    let lifted: Vec<CurveInfo> = ids.iter().filter_map(|c| m.curve_info(*c)).collect();
    assert_eq!(lifted.len(), 2);
    assert!(lifted.iter().any(|i| close(i.center, v(0., 500., 0.)) && (i.radius - 200.0).abs() < 1e-9));
}

#[test]
fn random_curve_regeneration_keeps_mesh_valid() {
    // Small LCG: deterministic.
    let mut s: u64 = 0x2545F4914F6CDD1D;
    let mut rnd = || {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((s >> 33) as f64) / (1u64 << 31) as f64
    };
    let mut regenerated = 0;
    for _ in 0..40 {
        let mut m = Mesh::new();
        m.add_face(&[v(-2000., -2000., 0.), v(2000., -2000., 0.), v(2000., 2000., 0.), v(-2000., 2000., 0.)]).unwrap();
        let mut cids = Vec::new();
        for _ in 0..3 {
            let c = v(rnd() * 2000. - 1000., rnd() * 2000. - 1000., 0.);
            let r = 50. + rnd() * 400.;
            let info = if rnd() < 0.5 {
                CurveInfo::circle(c, DVec3::Z, v(rnd() - 0.5, rnd() - 0.5, 0.), r, 3 + (rnd() * 30.) as u32).unwrap()
            } else {
                CurveInfo::arc_center(c, DVec3::Z, c + v(r, 0., 0.), 0.3 + rnd() * 5.5, 1 + (rnd() * 20.) as u32).unwrap()
            };
            let es = m.add_curve(&info);
            ok(&m);
            if let Some(e) = es.first() {
                cids.extend(m.edges[*e].curve);
            }
        }
        for c in cids {
            if m.curve_info(c).is_none() {
                continue;
            }
            let r = if rnd() < 0.5 { Some(30. + rnd() * 300.) } else { None };
            let n = 1 + (rnd() * 40.) as u32;
            let n = if m.curve_info(c).unwrap().kind == CurveKind::Arc { n } else { n.max(3) };
            if m.set_curve(c, r, Some(n)).is_ok() {
                ok(&m);
                regenerated += 1;
            }
        }
    }
    assert!(regenerated >= 40, "{regenerated}");
}

// ---------------------------------------------------------------------------
// Dimensions
// ---------------------------------------------------------------------------

fn box_model() -> Model {
    let mut model = Model::new();
    let m = model.root_mesh_mut();
    let f = m.add_face(&[v(0., 0., 0.), v(1000., 0., 0.), v(1000., 500., 0.), v(0., 500., 0.)]).unwrap();
    m.push_pull(f, -300.0, false).unwrap();
    model
}

#[test]
fn dimension_follows_vertices_resize_undo_save() {
    let mut model = box_model();
    let id = model.add_dimension_in(&[], v(0., 0., 0.), v(1000., 0., 0.), v(0., -200., 0.)).unwrap();
    let d = &model.root_mesh().dims[id];
    assert!(d.a.vertex.is_some() && d.b.vertex.is_some());
    assert_eq!(model.dimension_label_in(&[], id).as_deref(), Some("1000 mm"));
    let g = model.dimension_in(&[], id).unwrap();
    assert!(close(g.line().0, v(0., -200., 0.)));
    // Move the right side: the dimension follows.
    let mut hist = History::default();
    hist.push("Move", model.clone());
    let m = model.root_mesh_mut();
    let right: HashSet<VertexId> = m.vertices.iter().filter(|(_, x)| x.pos.x > 999.0).map(|(k, _)| k).collect();
    m.transform_vertices(&right, &DMat4::from_translation(v(500., 0., 0.)));
    assert_eq!(model.dimension_label_in(&[], id).as_deref(), Some("1500 mm"));
    // Units.
    model.unit = LengthUnit::Meter;
    assert_eq!(model.dimension_label_in(&[], id).as_deref(), Some("1.5 m"));
    model.unit = LengthUnit::Millimeter;
    // Resize (as the app's Tape Measure resize does): attached anchors follow vertices, the
    // offset scales.
    let s = DMat4::from_scale(DVec3::splat(2.0));
    let m = model.root_mesh_mut();
    let all: HashSet<VertexId> = m.vertices.keys().collect();
    m.transform_vertices(&all, &s);
    let root = model.root;
    model.transform_def_guides(root, &s);
    let g = model.dimension_in(&[], id).unwrap();
    assert!((g.length() - 3000.0).abs() < 1e-9);
    assert!((g.offset - v(0., -400., 0.)).length() < 1e-9);
    // Undo restores it.
    hist.undo(&mut model);
    assert_eq!(model.dimension_label_in(&[], id).as_deref(), Some("1000 mm"));
    // Save / load (ids and vertex attachment survive); text override.
    assert!(model.set_dimension_text_in(&[], id, Some("W = <>".into())));
    let json = io::to_json(&model).unwrap();
    let back = io::from_json(&json).unwrap();
    assert_eq!(back.dimension_label_in(&[], id).as_deref(), Some("W = 1000 mm"));
    assert!(back.root_mesh().dims[id].a.vertex.is_some());
    ok(back.root_mesh());
    // Files without dimensions don't mention them.
    assert!(!io::to_json(&Model::new()).unwrap().contains("\"dims\""));
}

#[test]
fn dimension_detaches_on_delete_and_follows_merge() {
    let mut m = Mesh::new();
    m.add_line(v(0., 0., 0.), v(100., 0., 0.));
    m.add_line(v(300., 0., 0.), v(400., 0., 0.));
    let id = m.add_dimension(v(100., 0., 0.), v(300., 0., 0.), v(0., 50., 0.));
    // Move the second line onto the first one's end: (300) merges into (100)... move it by -200
    // so its start lands on (100).
    let second: HashSet<VertexId> = m.vertices.iter().filter(|(_, x)| x.pos.x > 250.0).map(|(k, _)| k).collect();
    m.transform_vertices(&second, &DMat4::from_translation(v(-200., 0., 0.)));
    ok(&m);
    let (a, b) = m.dim_points(id).unwrap();
    assert!(close(a, v(100., 0., 0.)) && close(b, v(100., 0., 0.)));
    assert_eq!(m.dims[id].a.vertex, m.dims[id].b.vertex, "merged into the same vertex");
    // Deleting the geometry: the anchors stay where the vertex was.
    let all: Vec<EntityRef> = m.edges.keys().map(EntityRef::Edge).collect();
    m.erase(&all);
    assert!(m.vertices.is_empty());
    ok(&m);
    let (a, _) = m.dim_points(id).unwrap();
    assert!(close(a, v(100., 0., 0.)));
    assert!(m.dims[id].a.vertex.is_none());
}

#[test]
fn dimension_transform_copy_remove() {
    let mut model = Model::new();
    model.root_mesh_mut().add_line(v(0., 0., 0.), v(100., 0., 0.));
    let free = model.add_dimension_in(&[], v(0., 0., 500.), v(100., 0., 500.), v(0., 0., 10.)).unwrap();
    let att = model.add_dimension_in(&[], v(0., 0., 0.), v(100., 0., 0.), v(0., 10., 0.)).unwrap();
    // Free dimension moves; attached anchors stay on their vertices (offset rotates).
    let t = DMat4::from_translation(v(0., 0., 100.));
    model.transform_dimensions_in(&[], &[free, att], &t);
    assert!(close(model.dimension_in(&[], free).unwrap().a, v(0., 0., 600.)));
    assert!(close(model.dimension_in(&[], att).unwrap().a, v(0., 0., 0.)));
    // Copy geometry then dimension: the copy attaches to the copied vertices.
    let ents: Vec<EntityRef> = model.root_mesh().edges.keys().map(EntityRef::Edge).collect();
    let t2 = DMat4::from_translation(v(0., 1000., 0.));
    model.root_mesh_mut().duplicate(&ents, &t2);
    let copies = model.copy_dimensions_in(&[], &[att], &t2);
    assert_eq!(copies.len(), 1);
    let c = copies[0];
    assert!(model.root_mesh().dims[c].a.vertex.is_some());
    assert!(close(model.dimension_in(&[], c).unwrap().a, v(0., 1000., 0.)));
    assert!(model.set_dimension_offset_in(&[], c, v(0., 0., 77.)));
    assert!(close(model.dimension_in(&[], c).unwrap().offset, v(0., 0., 77.)));
    assert_eq!(model.all_dimensions_in(&[]).len(), 3);
    assert_eq!(model.remove_dimensions_in(&[], &[free, free]), 1);
    assert!(!model.has_dimension_in(&[], free));
    assert!(model.has_any_dimensions());
}

#[test]
fn dimensions_in_groups_make_group_and_explode() {
    let mut model = box_model();
    let top_guide = model.add_guide_in(&[], GuideGeom::Point(v(0., 0., 999.))).unwrap();
    let other_guide = model.add_guide_in(&[], GuideGeom::Point(v(5., 5., 5.))).unwrap();
    let id = model.add_dimension_in(&[], v(0., 0., 0.), v(1000., 0., 0.), v(0., -200., 0.)).unwrap();
    let other = model.add_dimension_in(&[], v(0., 0., 0.), v(0., 500., 0.), v(-200., 0., 0.)).unwrap();
    let ents: Vec<EntityRef> = model.root_mesh().faces.keys().map(EntityRef::Face).collect();
    let g = model.make_group_with(&[], &ents, &[top_guide], &[id]).unwrap();
    // Guide and dimension moved into the group; the others stay.
    assert!(!model.has_guide_in(&[], top_guide));
    assert!(model.has_guide_in(&[], other_guide));
    assert_eq!(model.guides_in(&[g]).len(), 1);
    assert_eq!(model.all_dimensions_in(&[]), vec![other]);
    let gid = model.all_dimensions_in(&[g])[0];
    let d = model.def_at(&[g]).unwrap();
    assert!(model.defs[d].mesh.dims[gid].a.vertex.is_some(), "still attached inside the group");
    ok(&model.defs[d].mesh);
    // The dimension left in the root lost its vertices (moved into the group) but kept place.
    let od = model.dimension_in(&[], other).unwrap();
    assert!(close(od.a, v(0., 0., 0.)) && close(od.b, v(0., 500., 0.)));
    // Scaled group: world length in the label; follows the group.
    model.root_mesh_mut().instances[g].transform = DMat4::from_translation(v(0., 0., 1000.)) * DMat4::from_scale(DVec3::splat(2.0));
    assert_eq!(model.dimension_label_in(&[g], gid).as_deref(), Some("2000 mm"));
    let all = model.all_dimensions_world();
    assert_eq!(all.len(), 2);
    assert!(all.iter().any(|(p, i, g2)| p == &vec![g] && *i == gid && close(g2.a, v(0., 0., 1000.))));
    // Guide points for zoom extents, including the nested one (transformed).
    let gp = model.guide_points_world();
    assert!(gp.iter().any(|p| close(*p, v(0., 0., 1000. + 2.0 * 999.))));
    assert!(gp.iter().any(|p| close(*p, v(5., 5., 5.))));
    assert_eq!(model.dimension_points_world().len(), 8);
    // Hidden instances are skipped.
    model.root_mesh_mut().instances[g].hidden = true;
    assert_eq!(model.all_dimensions_world().len(), 1);
    assert_eq!(model.guide_points_world().len(), 1);
    model.root_mesh_mut().instances[g].hidden = false;
    // Explode lifts the dimension (world position kept, attached to exploded vertices).
    model.explode(&[], g).unwrap();
    ok(model.root_mesh());
    assert_eq!(model.all_dimensions_in(&[]).len(), 2);
    let lifted = model.all_dimensions_in(&[]).into_iter().find(|x| *x != other).unwrap();
    assert!(model.root_mesh().dims[lifted].a.vertex.is_some());
    assert_eq!(model.dimension_label_in(&[], lifted).as_deref(), Some("2000 mm"));
    assert!(close(model.dimension_in(&[], lifted).unwrap().offset, v(0., -400., 0.)));
    // Only guides / dimensions selected: still makes a group.
    let g2 = model.make_group_with(&[], &[], &[other_guide], &[other]).unwrap();
    assert_eq!(model.all_dimensions_in(&[g2]).len(), 1);
    assert!(model.make_group_with(&[], &[], &[], &[]).is_err());
    // Component variant.
    let g3 = model.make_component_with(&[], &[], &[], &[lifted], "C").unwrap();
    assert_eq!(model.all_dimensions_in(&[g3]).len(), 1);
}

// ---------------------------------------------------------------------------
// Axes
// ---------------------------------------------------------------------------

#[test]
fn custom_axes_round_trip() {
    let mut model = Model::new();
    assert_eq!(model.axes_frame(), varstotas_kernel::glam::DAffine3::IDENTITY);
    assert!(close(model.world_to_axes(v(1., 2., 3.)), v(1., 2., 3.)));
    // Rotated 90° about Z, origin at (100, 0, 0); y given non-perpendicular.
    model.set_axes(v(100., 0., 0.), v(0., 2., 0.), v(-1., 1., 0.)).unwrap();
    let [x, y, z] = model.axes_dirs();
    assert!(close(x, DVec3::Y) && close(y, -DVec3::X) && close(z, DVec3::Z));
    assert!(close(model.axes_origin(), v(100., 0., 0.)));
    let p = v(100., 50., 7.);
    assert!(close(model.world_to_axes(p), v(50., 0., 7.)));
    assert!(close(model.axes_to_world(model.world_to_axes(p)), p));
    assert!(close(model.world_dir_to_axes(DVec3::Y), DVec3::X));
    assert!(close(model.axes_dir_to_world(DVec3::X), DVec3::Y));
    // Save / load.
    let json = io::to_json(&model).unwrap();
    assert!(json.contains("\"axes\""));
    let back = io::from_json(&json).unwrap();
    assert_eq!(back.axes, model.axes);
    // Degenerate input.
    assert!(model.set_axes(DVec3::ZERO, DVec3::X, DVec3::X * 3.0).is_err());
    assert!(model.set_axes(DVec3::ZERO, DVec3::ZERO, DVec3::Y).is_err());
    // Reset; world axes are stored as None and not written.
    model.reset_axes();
    assert!(model.axes.is_none());
    model.set_axes(DVec3::ZERO, DVec3::X, DVec3::Y).unwrap();
    assert!(model.axes.is_none());
    assert!(!io::to_json(&model).unwrap().contains("\"axes\""));
}

#[test]
fn pulled_circle_is_smooth_pulled_polygon_is_not() {
    for (info, smooth) in [
        (CurveInfo::circle(v(0., 0., 0.), DVec3::Z, DVec3::X, 500.0, 24).unwrap(), true),
        (CurveInfo::polygon(v(0., 0., 0.), DVec3::Z, DVec3::X, 500.0, 6).unwrap(), false),
    ] {
        let mut m = Model::new();
        m.root_mesh_mut().add_curve(&info);
        let f = *m.root_mesh().faces.keys().collect::<Vec<_>>().first().unwrap();
        m.root_mesh_mut().push_pull(f, 300.0, false).unwrap();
        let mesh = m.root_mesh();
        let side = mesh.faces.iter().find(|(_, fc)| fc.normal.z.abs() < 0.5).map(|(id, _)| id).unwrap();
        let (_, _, normals) = tessellate::face_world_triangles(mesh, side, &DMat4::IDENTITY, false);
        let n = mesh.faces[side].normal;
        assert_eq!(!normals.is_empty(), smooth, "{info:?}");
        if smooth {
            // Corner normals lean toward the neighbours; the face normal is their mean direction.
            assert!(normals.iter().all(|c| c.dot(n) > 0.95 && c.dot(n) < 0.9999));
            let top = mesh.faces.iter().find(|(_, fc)| fc.normal.z > 0.5).map(|(id, _)| id).unwrap();
            assert!(tessellate::face_world_triangles(mesh, top, &DMat4::IDENTITY, false).2.is_empty(), "cap stays flat");
        }
    }
}

#[test]
fn soften_by_angle_follows_fold_angle() {
    let mut m = Mesh::new();
    box_at(&mut m, v(0., 0., 0.), v(1000., 1000., 1000.));
    let all: Vec<EdgeId> = m.edges.keys().collect();
    assert!(all.iter().all(|e| (m.edge_fold_angle(*e).unwrap() - 90.0).abs() < 1e-9));
    assert_eq!(m.soften_by_angle(&all, 20.0, true, false), 0);
    assert_eq!(m.soften_by_angle(&all, 95.0, true, false), 12);
    assert!(m.edges.values().all(|e| e.soft && e.smooth));
    assert_eq!(m.soften_by_angle(&all, 95.0, false, false), 12);
    assert!(m.edges.values().all(|e| e.soft && !e.smooth));
    assert_eq!(m.soften_by_angle(&all, 45.0, true, true), 12);
    assert!(m.edges.values().all(|e| !e.soft && !e.smooth));

    // 6-gon prism sides fold by 60°; a cut across the top face is coplanar.
    let mut m = Model::new();
    let info = CurveInfo::polygon(v(0., 0., 0.), DVec3::Z, DVec3::X, 500.0, 6).unwrap();
    m.root_mesh_mut().add_curve(&info);
    let f = *m.root_mesh().faces.keys().collect::<Vec<_>>().first().unwrap();
    m.root_mesh_mut().push_pull(f, 300.0, false).unwrap();
    let mesh = m.root_mesh_mut();
    let side: Vec<EdgeId> = mesh.edges.iter().filter(|(_, e)| (mesh.vertices[e.v[0]].pos.z - mesh.vertices[e.v[1]].pos.z).abs() > 1.0).map(|(id, _)| id).collect();
    assert_eq!(side.len(), 6);
    assert!(side.iter().all(|e| (mesh.edge_fold_angle(*e).unwrap() - 60.0).abs() < 1e-6));
    assert_eq!(mesh.soften_by_angle(&side, 59.0, true, false), 0);
    assert_eq!(mesh.soften_by_angle(&side, 61.0, true, false), 6);
}

#[test]
fn polyline_lines_close_a_loop_without_a_face() {
    let mut m = Mesh::new();
    let sq = [v(0., 0., 0.), v(10., 0., 0.), v(10., 10., 0.), v(0., 10., 0.)];
    let es = m.add_polyline_lines(&sq, true, true);
    assert_eq!(es.len(), 4);
    assert_eq!(m.faces.len(), 0);
    let mut f = Mesh::new();
    f.add_polyline(&sq, true, true);
    assert_eq!(f.faces.len(), 1);
}
