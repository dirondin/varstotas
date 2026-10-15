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
