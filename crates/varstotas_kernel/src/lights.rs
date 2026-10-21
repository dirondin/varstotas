//! Light objects and render settings of the path tracer (track R batch 3): point, spot and
//! rectangular area lights placed in the model (world space, mm), and the Render window's
//! settings saved with the document.

use glam::{DMat4, DVec3};
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
    /// The light moved / turned by a world transform (Move, Rotate): position and aim.
    pub fn transformed(&self, t: &DMat4) -> Light {
        let d = t.transform_vector3(self.direction);
        Light {
            position: t.transform_point3(self.position),
            direction: d.try_normalize().unwrap_or(self.direction),
            ..self.clone()
        }
    }

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

/// Depth of field of a render.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderDof {
    /// As the view's Effects (Depth of field when it is on).
    #[default]
    View,
    Off,
    /// The render's own f-stop and focus distance.
    On,
}

impl RenderDof {
    pub const ALL: [RenderDof; 3] = [RenderDof::View, RenderDof::Off, RenderDof::On];

    pub fn label(self) -> String {
        varstotas_i18n::tr(match self {
            RenderDof::View => "render-dof-view",
            RenderDof::Off => "render-dof-off",
            RenderDof::On => "render-dof-on",
        })
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
    /// Denoise with Open Image Denoise when it is installed (else the built-in filter).
    pub ai_denoise: bool,
    /// Brightest indirect light of a sample (kills fireflies); 0 = no limit.
    pub clamp: f32,
    /// The image follows the viewport's aspect ratio (the height is derived from the width).
    pub match_view: bool,
    pub dof: RenderDof,
    /// Lens opening of `RenderDof::On` (f-number; the focal length follows the field of view
    /// on a 35 mm camera).
    pub f_stop: f32,
    /// Focus distance (mm) of `RenderDof::On`; 0 = the view's target.
    pub focus: f64,
    /// Nothing behind the model: the sky and the ground plane are transparent in the PNG (they
    /// still light the scene).
    pub transparent: bool,
    /// Colour grading of the image (applied live, no new samples): exposure offset (EV),
    /// tone curve (`None` = the environment's), white balance (K) and tint (green −1 .. magenta 1),
    /// saturation and contrast (1 = unchanged), vignette (0..1).
    pub exposure: f32,
    pub tonemap: Option<crate::Tonemap>,
    pub temperature: f32,
    pub tint: f32,
    pub saturation: f32,
    pub contrast: f32,
    pub vignette: f32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            samples: 256,
            time_limit: 0.0,
            bounces: 6,
            denoise: true,
            ai_denoise: true,
            clamp: 20.0,
            match_view: true,
            dof: RenderDof::View,
            f_stop: 2.8,
            focus: 0.0,
            transparent: false,
            exposure: 0.0,
            tonemap: None,
            temperature: 6500.0,
            tint: 0.0,
            saturation: 1.0,
            contrast: 1.0,
            vignette: 0.0,
        }
    }
}

/// Quality presets of the Render window: name, samples, bounces, firefly clamp.
pub const RENDER_PRESETS: [(&str, u32, u32, f32); 4] =
    [("Draft", 32, 3, 10.0), ("Preview", 128, 5, 20.0), ("Final", 512, 8, 30.0), ("Best", 2048, 12, 60.0)];

/// Image sizes of the Render window: name, width, height.
pub const RENDER_SIZES: [(&str, u32, u32); 6] = [
    ("HD 1280 × 720", 1280, 720),
    ("Full HD 1920 × 1080", 1920, 1080),
    ("QHD 2560 × 1440", 2560, 1440),
    ("4K 3840 × 2160", 3840, 2160),
    ("Square 1080", 1080, 1080),
    ("Portrait 1080 × 1350", 1080, 1350),
];

impl RenderSettings {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// Apply a quality preset (`RENDER_PRESETS` by name); false for an unknown name.
    pub fn apply_preset(&mut self, name: &str) -> bool {
        let Some(&(_, samples, bounces, clamp)) = RENDER_PRESETS.iter().find(|p| p.0.eq_ignore_ascii_case(name)) else { return false };
        self.samples = samples;
        self.bounces = bounces;
        self.clamp = clamp;
        self.time_limit = 0.0;
        self.denoise = true;
        true
    }

    /// The quality preset these settings match.
    pub fn preset(&self) -> Option<&'static str> {
        RENDER_PRESETS.iter().find(|p| (p.1, p.2, p.3) == (self.samples, self.bounces, self.clamp)).map(|p| p.0)
    }

    /// The grading is the default (the image is the environment's tone curve and exposure only).
    pub fn neutral_grade(&self) -> bool {
        let d = Self::default();
        (self.exposure, self.tonemap, self.temperature, self.tint, self.saturation, self.contrast, self.vignette)
            == (d.exposure, d.tonemap, d.temperature, d.tint, d.saturation, d.contrast, d.vignette)
    }

    /// Back to the default grading.
    pub fn reset_grade(&mut self) {
        let d = Self::default();
        (self.exposure, self.tonemap, self.temperature, self.tint, self.saturation, self.contrast, self.vignette) =
            (d.exposure, d.tonemap, d.temperature, d.tint, d.saturation, d.contrast, d.vignette);
    }

    pub fn sanitized(mut self) -> Self {
        let d = Self::default();
        let c = |v: f32, lo: f32, hi: f32, def: f32| if v.is_finite() { v.clamp(lo, hi) } else { def };
        self.width = self.width.clamp(16, 8192);
        self.height = self.height.clamp(16, 8192);
        self.samples = self.samples.clamp(1, 1 << 16);
        self.time_limit = c(self.time_limit, 0.0, 86400.0, 0.0);
        self.bounces = self.bounces.clamp(1, 32);
        self.clamp = c(self.clamp, 0.0, 1000.0, d.clamp);
        self.f_stop = c(self.f_stop, 0.5, 64.0, d.f_stop);
        self.focus = if self.focus.is_finite() { self.focus.clamp(0.0, 1.0e7) } else { 0.0 };
        self.exposure = c(self.exposure, -10.0, 10.0, 0.0);
        self.temperature = c(self.temperature, 1500.0, 15000.0, d.temperature);
        self.tint = c(self.tint, -1.0, 1.0, 0.0);
        self.saturation = c(self.saturation, 0.0, 2.0, 1.0);
        self.contrast = c(self.contrast, 0.5, 2.0, 1.0);
        self.vignette = c(self.vignette, 0.0, 1.0, 0.0);
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
        let r = RenderSettings { temperature: 0.0, f_stop: f32::NAN, saturation: 9.0, ..Default::default() }.sanitized();
        assert_eq!((r.temperature, r.f_stop, r.saturation), (1500.0, 2.8, 2.0));
    }

    #[test]
    fn lights_follow_transforms() {
        let l = Light { direction: DVec3::X, ..Light::new(LightKind::Spot, DVec3::new(1000.0, 0.0, 0.0)) };
        let t = DMat4::from_translation(DVec3::new(0.0, 0.0, 500.0)) * DMat4::from_rotation_z(std::f64::consts::FRAC_PI_2);
        let m = l.transformed(&t);
        assert!(m.position.distance(DVec3::new(0.0, 1000.0, 500.0)) < 1e-9);
        assert!(m.direction.distance(DVec3::Y) < 1e-9);
        assert_eq!((m.name, m.power), (l.name, l.power));
    }

    #[test]
    fn render_presets_and_grading() {
        let mut r = RenderSettings::default();
        assert_eq!(r.preset(), None);
        assert!(r.apply_preset("final"));
        assert_eq!((r.samples, r.bounces, r.preset()), (512, 8, Some("Final")));
        assert!(!r.apply_preset("nope"));
        assert!(r.neutral_grade());
        r.saturation = 1.4;
        r.tonemap = Some(crate::Tonemap::Agx);
        assert!(!r.neutral_grade());
        r.reset_grade();
        assert!(r.neutral_grade());
        // Old documents (no new fields) load with the defaults.
        let old: RenderSettings = serde_json::from_str(r#"{"width":800,"height":600,"samples":64}"#).unwrap();
        assert_eq!((old.width, old.clamp, old.temperature, old.dof), (800, 20.0, 6500.0, RenderDof::View));
    }
}
