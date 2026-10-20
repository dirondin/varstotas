//! Bezier curves: anchors kept with the edges, reshaping, copies.

mod common;
use common::*;
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::*;
use std::collections::HashSet;

fn smooth_pt(p: DVec3, t: DVec3) -> Anchor {
    Anchor { p, h_in: Some(p - t), h_out: Some(p + t) }
}

/// A closed drop shape (a face) on z = 0.
fn drop_shape() -> Spline {
    Spline {
        anchors: vec![
            Anchor::corner(v(0., 0., 0.)),
            smooth_pt(v(500., 300., 0.), v(0., 200., 0.)),
            smooth_pt(v(0., 600., 0.), v(-200., 0., 0.)),
        ],
        closed: true,
        segs: 8,
        normal: DVec3::Z,
    }
}

fn open_wave() -> Spline {
    Spline {
        anchors: vec![Anchor::corner(v(0., 0., 0.)), smooth_pt(v(500., 200., 0.), v(200., 0., 0.)), Anchor::corner(v(1000., 0., 0.))],
        closed: false,
        segs: 6,
        normal: DVec3::Z,
    }
}

fn cid_of(m: &Mesh, es: &[EdgeId]) -> u32 {
    m.edges[es[0]].curve.unwrap()
}

#[test]
fn closed_spline_makes_a_face_and_stays_editable() {
    let mut m = Mesh::default();
    let es = m.add_spline(&drop_shape());
    ok(&m);
    assert_eq!(m.faces.len(), 1);
    let cid = cid_of(&m, &es);
    assert_eq!(m.spline(cid), Some(drop_shape()));
    let f = m.faces.keys().next().unwrap();
    // Pull the middle anchor out: the same face, bigger.
    let mut sp = drop_shape();
    sp.anchors[1].p = v(800., 300., 0.);
    sp.anchors[1].h_in = Some(v(800., 100., 0.));
    sp.anchors[1].h_out = Some(v(800., 500., 0.));
    let a0 = m.face_area(f);
    m.set_spline(cid, &sp).unwrap();
    ok(&m);
    assert_eq!(m.faces.len(), 1);
    assert!(m.faces.contains_key(f));
    assert!(m.face_area(f) > a0 * 1.2);
    assert_eq!(m.spline(cid), Some(sp.clone()));
    // More segments.
    sp.segs = 16;
    m.set_spline(cid, &sp).unwrap();
    ok(&m);
    assert_eq!(m.curve_edges(cid).len(), 3 * 16);
    assert!(m.faces.contains_key(f));
}

#[test]
fn crossing_itself_is_refused() {
    let mut m = Mesh::default();
    let es = m.add_spline(&drop_shape());
    let cid = cid_of(&m, &es);
    let mut sp = drop_shape();
    // A bow tie: the outline crosses itself.
    sp.anchors = [v(0., 0., 0.), v(600., 600., 0.), v(600., 0., 0.), v(0., 600., 0.)].map(Anchor::corner).to_vec();
    assert!(m.set_spline(cid, &sp).is_err());
    assert_eq!(m.spline(cid), Some(drop_shape()));
    ok(&m);
}

#[test]
fn open_spline_end_drags_attached_edges() {
    let mut m = Mesh::default();
    let es = m.add_spline(&open_wave());
    let cid = cid_of(&m, &es);
    m.add_line(v(1000., 0., 0.), v(1000., -500., 0.));
    let mut sp = open_wave();
    sp.anchors[2].p = v(1200., 100., 0.);
    m.set_spline(cid, &sp).unwrap();
    ok(&m);
    let ends: HashSet<_> = m.edges.values().filter(|e| e.curve.is_none()).flat_map(|e| e.v).map(|x| m.vertices[x].pos).map(|p| (p.x as i64, p.y as i64)).collect();
    assert_eq!(ends, HashSet::from([(1200, 100), (1000, -500)]));
    assert!(m.spline(cid).is_some());
}

#[test]
fn spline_metadata_lapses_when_the_curve_is_changed() {
    let mut m = Mesh::default();
    let es = m.add_spline(&open_wave());
    let cid = cid_of(&m, &es);
    // Moving one inner vertex by itself: no longer the curve.
    let (chain, _) = m.spline_chain(cid).unwrap();
    m.transform_vertices(&HashSet::from([chain[3]]), &DMat4::from_translation(v(0., 50., 0.)));
    assert!(m.spline(cid).is_none());
    assert!(m.set_spline(cid, &open_wave()).is_err());
    // Erasing one edge of another curve.
    let es = m.add_spline(&Spline { anchors: open_wave().anchors.iter().map(|a| Anchor { p: a.p + v(0., 2000., 0.), h_in: a.h_in.map(|h| h + v(0., 2000., 0.)), h_out: a.h_out.map(|h| h + v(0., 2000., 0.)) }).collect(), ..open_wave() });
    let c2 = cid_of(&m, &es);
    assert!(m.spline(c2).is_some());
    m.erase(&[EntityRef::Edge(es[2])]);
    assert!(m.spline(c2).is_none());
}

#[test]
fn moved_scaled_copied_and_saved_splines_stay_editable() {
    let mut model = Model::new();
    let es = model.root_mesh_mut().add_spline(&drop_shape());
    let cid = cid_of(model.root_mesh(), &es);
    let m = model.root_mesh_mut();
    let all: HashSet<VertexId> = m.vertices.keys().collect();
    // Non-uniform scale: still exact for Bezier curves.
    m.transform_vertices(&all, &(DMat4::from_translation(v(100., 0., 0.)) * DMat4::from_scale(v(2., 1., 1.))));
    let sp = m.spline(cid).expect("still a spline");
    assert!(sp.anchors[1].p.distance(v(1100., 300., 0.)) < 1e-9);
    // Copy: fresh id.
    let ents: Vec<EntityRef> = m.edges.keys().map(EntityRef::Edge).collect();
    let out = m.duplicate(&ents, &DMat4::from_translation(v(0., 2000., 0.)));
    let c2 = out.iter().find_map(|e| if let EntityRef::Edge(e) = e { m.edges[*e].curve } else { None }).unwrap();
    assert_ne!(c2, cid);
    assert!(m.spline(c2).is_some() && m.spline(cid).is_some());
    // Save / load.
    let back = io::from_json(&io::to_json(&model).unwrap()).unwrap();
    assert_eq!(back.root_mesh().splines, model.root_mesh().splines);
    assert!(back.root_mesh().spline(cid).is_some());
    // Group and explode.
    let ents: Vec<EntityRef> = model.root_mesh().edges.keys().map(EntityRef::Edge).chain(model.root_mesh().faces.keys().map(EntityRef::Face)).collect();
    let g = model.make_group(&[], &ents).unwrap();
    let d = model.def_at(&[g]).unwrap();
    let dm = &model.defs[d].mesh;
    assert_eq!(dm.edges.values().filter_map(|e| e.curve).filter(|c| dm.spline(*c).is_some()).collect::<HashSet<_>>().len(), 2);
    model.root_mesh_mut().instances[g].transform = DMat4::from_translation(v(0., 0., 300.));
    model.explode(&[], g).unwrap();
    let m = model.root_mesh();
    ok(m);
    let ids: HashSet<u32> = m.edges.values().filter_map(|e| e.curve).collect();
    assert_eq!(ids.len(), 2);
    assert!(ids.iter().all(|c| m.spline(*c).is_some_and(|s| (s.anchors[0].p.z - 300.).abs() < 1e-9)));
}

#[test]
fn push_pulled_spline_face_keeps_both_outlines() {
    let mut m = Mesh::default();
    m.add_spline(&drop_shape());
    let f = m.faces.keys().next().unwrap();
    m.push_pull(f, 400.0, false).unwrap();
    ok(&m);
    let ids: HashSet<u32> = m.edges.values().filter_map(|e| e.curve).collect();
    assert_eq!(ids.len(), 2);
    for c in &ids {
        let sp = m.spline(*c).expect("outline still a spline");
        // A solid's outline cannot be reshaped (its faces leave the plane).
        assert!(m.clone().set_spline(*c, &sp).is_err());
    }
}
