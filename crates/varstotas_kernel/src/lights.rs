//! Light objects and render settings of the path tracer (track Р batch 3): point, spot and
//! rectangular area lights placed in the model (world space, mm), and the Render window's
//! settings saved with the document.

use glam::DVec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LightKind {
    /// Shines in all directions from a small sphere (`radius`).
    #[default]
    Point,
    /// A cone along `direction` (`spot_angle`, soft edge `spot_blend`).
    Spot,
    /// A rectangle (`size`) facing `direction`, lit on that side.
    Area,
}

impl LightKind {
    pub const ALL: [LightKind; 3] = [Self::Point, Self::Spot, Self::Area];

    pub fn key(self) -> &'static str {
        match self {
            Self::Point => "point",
            Self::Spot => "spot",
            Self::Area => "area",
        }
    }

    pub fn from_key(k: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|l| l.key() == k)
    }
}

/// A light in the model. Power is in watts of a white bulb-like emitter: 100 W lights a
/// surface 2 m away about as brightly as a dim overcast sky.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Light {
    pub name: String,
    pub kind: LightKind,
    pub position: DVec3,
    /// Where a spot points / the side an area light shines to (unit).
    pub direction: DVec3,
    /// sRGB colour.
    pub color: [f32; 3],
    pub power: f32,
    /// Size of a point or spot light (mm): bigger = softer shadows.
    pub radius: f32,
    /// Full cone angle of a spot, degrees.
    pub spot_angle: f32,
    /// Soft part of the spot's edge (0 = hard, 1 = all soft).
    pub spot_blend: f32,
    /// Width and height of an area light (mm); its width runs across `direction` and Z.
    pub size: [f32; 2],
    pub enabled: bool,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            name: "Light".into(),
            kind: LightKind::Point,
            position: DVec3::new(0.0, 0.0, 2500.0),
            direction: DVec3::NEG_Z,
            color: [1.0, 0.95, 0.88],
            power: 100.0,
            radius: 50.0,
            spot_angle: 45.0,
            spot_blend: 0.3,
            size: [1000.0, 1000.0],
            enabled: true,
        }
    }
}

impl Light {
    /// A light of `kind` at `position` with its defaults.
    pub fn new(kind: LightKind, position: DVec3) -> Self {
        let name = match kind {
            LightKind::Point => "Point Light",
            LightKind::Spot => "Spot Light",
            LightKind::Area => "Area Light",
        };
        Self { name: name.into(), kind, position, ..Default::default() }
    }

    /// Clamp the values to their ranges (NaN → default), unit direction.
    pub fn sanitized(mut self) -> Self {
        let d = Self::default();
        let c = |v: f32, lo: f32, hi: f32, def: f32| if v.is_finite() { v.clamp(lo, hi) } else { def };
        if !self.position.is_finite() {
            self.position = d.position;
        }
        self.direction = self.direction.try_normalize().unwrap_or(d.direction);
        for k in 0..3 {
            self.color[k] = c(self.color[k], 0.0, 1.0, d.color[k]);
        }
        self.power = c(self.power, 0.0, 1.0e6, d.power);
        self.radius = c(self.radius, 0.0, 1.0e5, d.radius);
        self.spot_angle = c(self.spot_angle, 1.0, 179.0, d.spot_angle);
        self.spot_blend = c(self.spot_blend, 0.0, 1.0, d.spot_blend);
        for k in 0..2 {
            self.size[k] = c(self.size[k], 1.0, 1.0e6, d.size[k]);
        }
        self
    }
}

/// Settings of the Render window (path tracer).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderSettings {
    pub width: u32,
    pub height: u32,
    /// Stop after this many samples per pixel.
    pub samples: u32,
    /// Stop after this many seconds (0 = no limit).
    pub time_limit: f32,
    /// Light bounces per path.
    pub bounces: u32,
    /// Smooth the noise out when the render stops.
    pub denoise: bool,
    /// The image follows the viewport's aspect ratio (the height is derived from the width).
    pub match_view: bool,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self { width: 1280, height: 720, samples: 256, time_limit: 0.0, bounces: 6, denoise: true, match_view: true }
    }
}

impl RenderSettings {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    pub fn sanitized(mut self) -> Self {
        self.width = self.width.clamp(16, 8192);
        self.height = self.height.clamp(16, 8192);
        self.samples = self.samples.clamp(1, 1 << 16);
        self.time_limit = if self.time_limit.is_finite() { self.time_limit.clamp(0.0, 86400.0) } else { 0.0 };
        self.bounces = self.bounces.clamp(1, 32);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lights_are_sanitized() {
        let l = Light { direction: DVec3::ZERO, power: f32::NAN, spot_angle: 500.0, ..Light::new(LightKind::Spot, DVec3::ONE) }.sanitized();
        assert_eq!(l.direction, DVec3::NEG_Z);
        assert_eq!(l.power, 100.0);
        assert_eq!(l.spot_angle, 179.0);
        assert_eq!(l.name, "Spot Light");
        assert_eq!(LightKind::from_key("area"), Some(LightKind::Area));
        let r = RenderSettings { width: 0, bounces: 100, ..Default::default() }.sanitized();
        assert_eq!((r.width, r.bounces), (16, 32));
    }
}
