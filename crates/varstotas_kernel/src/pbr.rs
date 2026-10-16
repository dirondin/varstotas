//! Physically based materials and the environment light (track Р, the Realistic style).
//!
//! A material's base colour stays in [`crate::Material::color`] (sRGB + opacity) and its
//! base colour map in [`crate::Material::image`]; [`Pbr`] adds the metallic-roughness
//! parameters of glTF 2.0: metalness, roughness, emission, transmission (glass) with an index
//! of refraction, and the maps (normal, roughness, metallic, ambient occlusion, emission). A
//! map multiplies its factor, like glTF. Maps use the planar texture coordinates of faces
//! (see [`crate::texture`]) with their own tile size.
//!
//! [`Environment`] is the light around the model: a sky built from the style's colours, a
//! plain colour or an HDRI panorama ([`crate::Model::hdri`]), turned by `rotation_deg`, plus
//! the sun of the Shadows panel and the camera's exposure and tone mapping.

use crate::texture::TextureImage;
use serde::{Deserialize, Serialize};

/// Metallic-roughness parameters of a material (glTF 2.0 core plus emission strength,
/// transmission and IOR from `KHR_materials_*`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Pbr {
    /// 0 = dielectric, 1 = metal.
    pub metallic: f32,
    /// 0 = mirror, 1 = fully rough.
    pub roughness: f32,
    /// Emitted colour, sRGB 0..1.
    pub emission: [f32; 3],
    /// Emission multiplier (1 = the colour's brightness).
    pub emission_strength: f32,
    /// 0 = opaque, 1 = clear glass (light passes through).
    pub transmission: f32,
    /// Index of refraction (glass 1.5, water 1.33).
    pub ior: f32,
    /// Strength of the normal map (1 = as authored).
    pub normal_scale: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal_map: Option<TextureImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness_map: Option<TextureImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metallic_map: Option<TextureImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ao_map: Option<TextureImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emission_map: Option<TextureImage>,
}

impl Default for Pbr {
    fn default() -> Self {
        Self {
            metallic: 0.0,
            roughness: 0.6,
            emission: [0.0; 3],
            emission_strength: 1.0,
            transmission: 0.0,
            ior: 1.5,
            normal_scale: 1.0,
            normal_map: None,
            roughness_map: None,
            metallic_map: None,
            ao_map: None,
            emission_map: None,
        }
    }
}

/// The maps of a [`Pbr`] by name (`normal`, `roughness`, `metallic`, `ao`, `emission`).
pub const PBR_MAPS: [&str; 5] = ["normal", "roughness", "metallic", "ao", "emission"];

impl Pbr {
    /// Clamp the factors to their ranges (NaN → default).
    pub fn sanitized(mut self) -> Self {
        let d = Self::default();
        let c = |v: f32, lo: f32, hi: f32, def: f32| if v.is_finite() { v.clamp(lo, hi) } else { def };
        self.metallic = c(self.metallic, 0.0, 1.0, d.metallic);
        self.roughness = c(self.roughness, 0.0, 1.0, d.roughness);
        for k in 0..3 {
            self.emission[k] = c(self.emission[k], 0.0, 1.0, 0.0);
        }
        self.emission_strength = c(self.emission_strength, 0.0, 1000.0, 1.0);
        self.transmission = c(self.transmission, 0.0, 1.0, 0.0);
        self.ior = c(self.ior, 1.0, 3.0, d.ior);
        self.normal_scale = c(self.normal_scale, 0.0, 4.0, 1.0);
        self
    }

    pub fn map(&self, name: &str) -> Option<&TextureImage> {
        self.map_slot(name).and_then(|m| m.as_ref())
    }

    pub fn map_mut(&mut self, name: &str) -> Option<&mut Option<TextureImage>> {
        Some(match name {
            "normal" => &mut self.normal_map,
            "roughness" => &mut self.roughness_map,
            "metallic" => &mut self.metallic_map,
            "ao" => &mut self.ao_map,
            "emission" => &mut self.emission_map,
            _ => return None,
        })
    }

    fn map_slot(&self, name: &str) -> Option<&Option<TextureImage>> {
        Some(match name {
            "normal" => &self.normal_map,
            "roughness" => &self.roughness_map,
            "metallic" => &self.metallic_map,
            "ao" => &self.ao_map,
            "emission" => &self.emission_map,
            _ => return None,
        })
    }

    /// Emits light (a colour and a strength above zero).
    pub fn emissive(&self) -> bool {
        self.emission_strength > 0.0 && (self.emission.iter().any(|c| *c > 0.0) || self.emission_map.is_some())
    }
}

/// What lights the model and shows behind it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkyKind {
    /// A sky from the style's sky and ground colours.
    #[default]
    Sky,
    /// One colour all around.
    Color,
    /// The document's HDRI panorama ([`crate::Model::hdri`]); the sky when there is none.
    Hdri,
}

/// Tone mapping of the Realistic style: scene light → screen colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tonemap {
    /// Filmic, contrasty (ACES fit).
    #[default]
    Aces,
    /// Filmic, keeps bright colours from going yellow / white (AgX).
    Agx,
    /// None: colours above 1 are clipped.
    Clamp,
}

impl Tonemap {
    pub fn key(self) -> &'static str {
        match self {
            Self::Aces => "aces",
            Self::Agx => "agx",
            Self::Clamp => "clamp",
        }
    }

    pub fn from_key(k: &str) -> Option<Self> {
        [Self::Aces, Self::Agx, Self::Clamp].into_iter().find(|t| t.key() == k)
    }
}

/// Environment light and camera settings of the Realistic style (saved in the document and
/// in scenes).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Environment {
    pub sky: SkyKind,
    /// The colour of [`SkyKind::Color`], sRGB.
    pub color: [f32; 3],
    /// Turns the panorama around the vertical axis, degrees.
    pub rotation_deg: f32,
    /// Brightness of the environment light (1 = as is).
    pub intensity: f32,
    /// The environment is drawn behind the model (off: the sky colours of the style).
    pub background: bool,
    /// Sun strength (0 = no sun); its direction comes from the Shadows panel.
    pub sun: f32,
    /// Camera exposure in stops (EV, 0 = as is).
    pub exposure: f32,
    pub tonemap: Tonemap,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            sky: SkyKind::Sky,
            color: [0.8, 0.82, 0.85],
            rotation_deg: 0.0,
            intensity: 1.0,
            background: true,
            sun: 3.0,
            exposure: 0.0,
            tonemap: Tonemap::Aces,
        }
    }
}

impl Environment {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// Clamp the values to their ranges (NaN → default).
    pub fn sanitized(mut self) -> Self {
        let d = Self::default();
        let c = |v: f32, lo: f32, hi: f32, def: f32| if v.is_finite() { v.clamp(lo, hi) } else { def };
        for k in 0..3 {
            self.color[k] = c(self.color[k], 0.0, 1.0, d.color[k]);
        }
        self.rotation_deg = c(self.rotation_deg, -360.0, 360.0, 0.0);
        self.intensity = c(self.intensity, 0.0, 100.0, 1.0);
        self.sun = c(self.sun, 0.0, 100.0, d.sun);
        self.exposure = c(self.exposure, -10.0, 10.0, 0.0);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_ranges() {
        let p = Pbr { metallic: 2.0, roughness: f32::NAN, ior: 0.5, ..Default::default() }.sanitized();
        assert_eq!((p.metallic, p.roughness, p.ior), (1.0, 0.6, 1.0));
        assert!(!p.emissive());
        let mut q = Pbr { emission: [1.0, 0.5, 0.0], ..Default::default() };
        assert!(q.emissive());
        q.emission_strength = 0.0;
        assert!(!q.emissive());
        assert!(q.map_mut("normal").is_some() && q.map_mut("bump").is_none());
        let e = Environment { exposure: 50.0, ..Default::default() }.sanitized();
        assert_eq!(e.exposure, 10.0);
        assert!(Environment::default().is_default());
        assert_eq!(Tonemap::from_key("agx"), Some(Tonemap::Agx));
        let json = serde_json::to_string(&Pbr::default()).unwrap();
        assert!(!json.contains("normal_map"));
        let back: Pbr = serde_json::from_str("{\"metallic\":1.0}").unwrap();
        assert_eq!((back.metallic, back.roughness), (1.0, 0.6));
    }
}
