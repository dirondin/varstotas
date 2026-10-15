//! Shadow settings (the classic Shadows panel) and the sun position for a date, time and place.

use glam::DVec3;
use serde::{Deserialize, Serialize};
use std::f64::consts::{PI, TAU};

/// Days before the first of each month (non-leap year).
const MONTH_START: [u32; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
const MONTH_DAYS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
pub const MONTH_NAMES: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShadowSettings {
    /// View → Shadows.
    pub enabled: bool,
    /// Day of the year, 1..=365 (no leap years).
    pub day: u32,
    /// Local clock time in minutes after midnight, 0..1440.
    pub minutes: u32,
    /// Degrees, north positive.
    pub latitude: f64,
    /// Degrees, east positive.
    pub longitude: f64,
    /// Time zone, hours from UTC.
    pub utc_offset: f64,
    /// Direction of north, degrees clockwise from the green (+Y) axis seen from above.
    pub north_deg: f64,
    /// Shade of sunlit faces, 0..1.
    pub light: f32,
    /// Shade of faces in shadow, 0..1.
    pub dark: f32,
    pub on_faces: bool,
    pub on_ground: bool,
}

impl Default for ShadowSettings {
    /// Defaults: 8 November, 13:30; the place is Moscow.
    fn default() -> Self {
        Self {
            enabled: false,
            day: day_of_year(11, 8),
            minutes: 13 * 60 + 30,
            latitude: 55.75,
            longitude: 37.62,
            utc_offset: 3.0,
            north_deg: 0.0,
            light: 0.8,
            dark: 0.45,
            on_faces: true,
            on_ground: true,
        }
    }
}

/// Day of the year (1-based) for a month (1..=12) and day of month (clamped).
pub fn day_of_year(month: u32, day: u32) -> u32 {
    let m = month.clamp(1, 12) as usize - 1;
    MONTH_START[m] + day.clamp(1, MONTH_DAYS[m])
}

/// `(month 1..=12, day of month)` of a day of the year.
pub fn month_day(day: u32) -> (u32, u32) {
    let d = day.clamp(1, 365);
    let m = MONTH_START.iter().rposition(|s| *s < d).unwrap_or(0);
    (m as u32 + 1, d - MONTH_START[m])
}

impl ShadowSettings {
    /// "8 Nov".
    pub fn date_label(&self) -> String {
        let (m, d) = month_day(self.day);
        format!("{d} {}", MONTH_NAMES[m as usize - 1])
    }

    /// "13:30".
    pub fn time_label(&self) -> String {
        let m = self.minutes.min(24 * 60 - 1);
        format!("{:02}:{:02}", m / 60, m % 60)
    }

    /// Unit direction toward the sun in model coordinates (z < 0: below the horizon).
    pub fn sun_direction(&self) -> DVec3 {
        let n = self.day.clamp(1, 365) as f64;
        // Solar declination and the equation of time (minutes), standard approximations.
        let decl = (23.44f64).to_radians() * (TAU * (284.0 + n) / 365.0).sin();
        let b = TAU * (n - 81.0) / 364.0;
        let eot = 9.87 * (2.0 * b).sin() - 7.53 * b.cos() - 1.5 * b.sin();
        let solar_min = self.minutes as f64 + 4.0 * self.longitude - 60.0 * self.utc_offset + eot;
        let h = (solar_min / 60.0 - 12.0) * 15.0f64.to_radians();
        let lat = self.latitude.clamp(-90.0, 90.0).to_radians();
        // East / north / up components.
        let e = -decl.cos() * h.sin();
        let no = decl.sin() * lat.cos() - decl.cos() * lat.sin() * h.cos();
        let up = decl.sin() * lat.sin() + decl.cos() * lat.cos() * h.cos();
        let t = self.north_deg.to_radians();
        let north = DVec3::new(t.sin(), t.cos(), 0.0);
        let east = DVec3::new(t.cos(), -t.sin(), 0.0);
        (east * e + north * no + DVec3::Z * up).normalize_or(DVec3::Z)
    }

    /// Altitude of the sun above the horizon, degrees.
    pub fn sun_altitude_deg(&self) -> f64 {
        self.sun_direction().z.clamp(-1.0, 1.0).asin() * 180.0 / PI
    }
}
