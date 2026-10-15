//! Layout sheets: stored with the document, viewports following scenes and their state.

use glam::{DVec2, DVec3};
use varstotas_kernel::Model;
use varstotas_kernel::Scene;
use varstotas_kernel::io::{from_json, to_json};
use varstotas_kernel::layout::{LayoutSheet, SheetText, SheetViewport};

fn view(name: &str) -> Scene {
    Scene::camera(name, DVec3::new(0., 0., 5000.), DVec3::ZERO, DVec3::Y, 35.0, true)
}

#[test]
fn sheets_round_trip_and_old_files_have_none() {
    let mut m = Model::new();
    let plain = to_json(&m).unwrap();
    assert!(!plain.contains("sheets"));
    let mut s = LayoutSheet::new("Plans", DVec2::new(420., 297.));
    let mut vp = SheetViewport::new(DVec2::new(20., 20.), DVec2::new(200., 150.), view("Top"));
    vp.scale = Some(50.0);
    vp.scene = Some("Plan".into());
    s.viewports.push(vp);
    s.texts.push(SheetText { at: DVec2::new(30., 250.), size: 5.0, text: "A\nB".into() });
    s.title.as_mut().unwrap().author = "Me".into();
    m.sheets.push(s.clone());
    let back = from_json(&to_json(&m).unwrap()).unwrap();
    assert_eq!(back.sheets, vec![s]);
    assert!(from_json(&plain).unwrap().sheets.is_empty());
}

#[test]
fn viewport_follows_scene_and_its_hidden_tags() {
    let mut m = Model::new();
    let tag = m.add_tag("Furniture");
    let vp = SheetViewport::new(DVec2::ZERO, DVec2::splat(100.), view("Own"));
    assert_eq!(m.viewport_scene(&vp).name, "Own");
    let mut plan = view("Plan");
    plan.eye = DVec3::new(0., 0., 9000.);
    plan.hidden_tags = Some(vec![tag]);
    m.scenes.push(plan);
    let mut vp2 = vp.clone();
    vp2.scene = Some("Plan".into());
    let s = m.viewport_scene(&vp2).clone();
    assert_eq!(s.eye.z, 9000.0);
    let shown = m.with_scene_state(&s);
    assert!(!shown.tags[tag].visible);
    assert!(m.tags[tag].visible);
    // A missing scene falls back to the viewport's own view.
    vp2.scene = Some("Gone".into());
    assert_eq!(m.viewport_scene(&vp2).name, "Own");
    assert!(vp.contains(DVec2::new(50., 99.)) && !vp.contains(DVec2::new(101., 50.)));
}
