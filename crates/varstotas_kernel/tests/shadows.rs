//! M5 batch 2: sun position for shadows.

use varstotas_kernel::ShadowSettings;
use varstotas_kernel::shadows::{day_of_year, month_day};

/// Settings at solar time (longitude on the time zone meridian; EoT-free days are not needed:
/// tolerances cover the equation of time).
fn at(lat: f64, month: u32, day: u32, hh: u32) -> ShadowSettings {
    ShadowSettings { latitude: lat, longitude: 0.0, utc_offset: 0.0, day: day_of_year(month, day), minutes: hh * 60, ..Default::default() }
}

#[test]
fn dates_round_trip() {
    assert_eq!(day_of_year(1, 1), 1);
    assert_eq!(day_of_year(11, 8), 312);
    assert_eq!(day_of_year(12, 31), 365);
    for d in 1..=365 {
        let (m, dd) = month_day(d);
        assert_eq!(day_of_year(m, dd), d);
    }
    let s = ShadowSettings::default();
    assert_eq!(s.date_label(), "8 Nov");
    assert_eq!(s.time_label(), "13:30");
}

#[test]
fn sun_position() {
    // Equinox noon on the equator: overhead.
    let s = at(0.0, 3, 21, 12);
    assert!(s.sun_altitude_deg() > 85.0, "{}", s.sun_altitude_deg());
    // Moscow-ish latitude, summer solstice noon: 90 - 55.75 + 23.44 ≈ 57.7°, to the south (-Y).
    let s = at(55.75, 6, 21, 12);
    let d = s.sun_direction();
    assert!((s.sun_altitude_deg() - 57.7).abs() < 1.5, "{}", s.sun_altitude_deg());
    assert!(d.y < -0.4 && d.x.abs() < 0.05, "{d}");
    // Morning sun is in the east (+X), evening in the west.
    assert!(at(55.75, 6, 21, 8).sun_direction().x > 0.5);
    assert!(at(55.75, 6, 21, 16).sun_direction().x < -0.5);
    // Midnight in winter: below the horizon.
    assert!(at(55.75, 12, 21, 0).sun_direction().z < 0.0);
    // North angle 90° (north = +X): the noon sun moves to -X.
    let s = ShadowSettings { north_deg: 90.0, ..at(55.75, 6, 21, 12) };
    assert!(s.sun_direction().x < -0.4);
    // Southern hemisphere: the noon sun is to the north.
    assert!(at(-33.9, 6, 21, 12).sun_direction().y > 0.4);
}

#[test]
fn physical_materials_and_environment_are_saved() {
    use varstotas_kernel::io::{from_json, to_json};
    use varstotas_kernel::{CustomShader, Effects, Environment, Model, Pbr, Scene, SkyKind, Stylize, TextureImage, Tonemap};
    let mut m = Model::new();
    let plain = m.add_material("Plain", [1.0; 4]);
    let gold = m.add_material("Gold", [1.0, 0.8, 0.3, 1.0]);
    m.materials[gold].pbr = Some(Pbr {
        metallic: 1.0,
        roughness: 0.2,
        normal_map: Some(TextureImage::new(b"\x89PNG-not-really".to_vec(), "n.png", 300.0)),
        shader: Some(CustomShader { source: "// @param k = 2\nfn surface(s: Surface, p: Params) -> Surface { return s; }".into(), params: vec![3.0] }),
        ..Default::default()
    });
    let plain_json = to_json(&m).unwrap();
    assert!(!plain_json.contains("\"environment\""), "the default environment is not written");
    let effects = Effects { ao: true, fog: true, stylize: Stylize::Pencil, ..Effects::default().with_preset("balanced").unwrap() };
    let env = Environment { sky: SkyKind::Hdri, rotation_deg: 90.0, exposure: 1.5, tonemap: Tonemap::Agx, effects, ..Default::default() };
    m.environment = env;
    m.hdri = Some(TextureImage::new(b"#?RADIANCE".to_vec(), "sky.hdr", 1000.0));
    let mut sc = Scene::camera("Dusk", glam::DVec3::new(1.0, 2.0, 3.0), glam::DVec3::ZERO, glam::DVec3::Z, 35.0, false);
    sc.environment = Some(Environment { exposure: -1.0, ..env });
    m.scenes.push(sc);
    let back = from_json(&to_json(&m).unwrap()).unwrap();
    assert!(back.materials[plain].pbr.is_none());
    let p = back.materials[gold].pbr.as_ref().unwrap();
    assert_eq!((p.metallic, p.roughness), (1.0, 0.2));
    assert_eq!(p.normal_map.as_ref().map(|t| t.size[0]), Some(300.0));
    assert_eq!(p.shader.as_ref().map(|sh| sh.param(0)), Some(3.0));
    assert_eq!(back.environment, env);
    assert_eq!(back.hdri.as_ref().map(|h| h.file_name.as_str()), Some("sky.hdr"));
    assert_eq!(back.scenes[0].environment.map(|e| e.exposure), Some(-1.0));
}

#[test]
fn lights_render_settings_and_physical_sky_are_saved() {
    use varstotas_kernel::io::{from_json, to_json};
    use varstotas_kernel::{Environment, Light, LightKind, Model, RenderSettings, SkyKind};
    let mut m = Model::new();
    let plain = to_json(&m).unwrap();
    assert!(!plain.contains("\"lights\"") && !plain.contains("\"render\""), "defaults are not written");
    let mut spot = Light::new(LightKind::Spot, glam::DVec3::new(100.0, 200.0, 3000.0));
    spot.power = 250.0;
    spot.spot_angle = 30.0;
    m.lights.push(spot.clone());
    m.lights.push(Light { enabled: false, ..Light::new(LightKind::Area, glam::DVec3::ZERO) });
    m.render = RenderSettings { width: 640, height: 480, samples: 64, denoise: false, ..Default::default() };
    m.environment = Environment { sky: SkyKind::Physical, turbidity: 6.0, ..Default::default() };
    let back = from_json(&to_json(&m).unwrap()).unwrap();
    assert_eq!(back.lights.len(), 2);
    assert_eq!(back.lights[0], spot);
    assert!(!back.lights[1].enabled && back.lights[1].kind == LightKind::Area);
    assert_eq!(back.render, m.render);
    assert_eq!((back.environment.sky, back.environment.turbidity), (SkyKind::Physical, 6.0));
}
