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
//! the sun of the Shadows panel and the camera's exposure and tone mapping, and [`Effects`]
//! (soft shadows, ambient occlusion, reflections, bloom, depth of field, fog, vignette, clay and
//! stylized looks).
//!
//! [`CustomShader`] replaces a material's surface with a WGSL function (procedural wood,
//! water, …) whose `// @param` header lines become sliders.

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
    /// A clear varnish layer on top (car paint, lacquered wood): 0 = none, 1 = full.
    pub clearcoat: f32,
    /// Roughness of the varnish layer.
    pub clearcoat_roughness: f32,
    /// Soft rim of fabrics (velvet, cotton): 0 = none, 1 = strong.
    pub sheen: f32,
    /// 0 = a white sheen, 1 = the base colour's.
    pub sheen_tint: f32,
    /// Brushed metal: highlights stretched along the texture's u direction (+) or across it
    /// (−); 0 = round highlights.
    pub anisotropy: f32,
    /// Hide the repetition of the maps (a shifted copy blended in by low-frequency noise):
    /// 0 = plain tiling, 1 = strong variation.
    pub anti_tiling: f32,
    /// Rounded edges (mm): the shading normal bends near the model's edges within this radius
    /// as if they were filleted (the path-traced render; 0 = sharp).
    pub bevel: f32,
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
    /// A WGSL surface function replacing the factors and maps (Realistic style).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shader: Option<CustomShader>,
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
            clearcoat: 0.0,
            clearcoat_roughness: 0.05,
            sheen: 0.0,
            sheen_tint: 0.5,
            anisotropy: 0.0,
            anti_tiling: 0.0,
            bevel: 0.0,
            normal_map: None,
            roughness_map: None,
            metallic_map: None,
            ao_map: None,
            emission_map: None,
            shader: None,
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
        self.clearcoat = c(self.clearcoat, 0.0, 1.0, 0.0);
        self.clearcoat_roughness = c(self.clearcoat_roughness, 0.0, 1.0, d.clearcoat_roughness);
        self.sheen = c(self.sheen, 0.0, 1.0, 0.0);
        self.sheen_tint = c(self.sheen_tint, 0.0, 1.0, d.sheen_tint);
        self.anisotropy = c(self.anisotropy, -1.0, 1.0, 0.0);
        self.anti_tiling = c(self.anti_tiling, 0.0, 1.0, 0.0);
        self.bevel = c(self.bevel, 0.0, 1000.0, 0.0);
        if let Some(sh) = &mut self.shader {
            sh.params.truncate(MAX_SHADER_PARAMS);
            sh.params.iter_mut().for_each(|v| *v = if v.is_finite() { *v } else { 0.0 });
        }
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

/// Most `@param` values of a [`CustomShader`] (two vec4 in the renderer's material table).
pub const MAX_SHADER_PARAMS: usize = 8;

/// A material's surface written in WGSL (see `docs/SHADERS.md`): the source defines
/// `fn surface(s: Surface) -> Surface`; `params` are the values of its `// @param` lines in order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CustomShader {
    pub source: String,
    pub params: Vec<f32>,
}

/// One `// @param name = default (min..max) "Label"` line of a [`CustomShader`].
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderParam {
    pub name: String,
    pub label: String,
    pub default: f32,
    pub min: f32,
    pub max: f32,
}

impl CustomShader {
    /// A shader with its parameters at their defaults.
    pub fn new(source: impl Into<String>) -> Self {
        let source = source.into();
        let params = shader_params(&source).iter().map(|p| p.default).collect();
        Self { source, params }
    }

    /// The value of parameter `i` (its default when the list is shorter).
    pub fn param(&self, i: usize) -> f32 {
        self.params.get(i).copied().unwrap_or_else(|| shader_params(&self.source).get(i).map_or(0.0, |p| p.default))
    }
}

/// The `// @param` lines of a WGSL surface (at most [`MAX_SHADER_PARAMS`]). The range is
/// optional (default 0..1 or around the default), so is the label (default: the name).
pub fn shader_params(source: &str) -> Vec<ShaderParam> {
    let mut out = Vec::new();
    for line in source.lines() {
        let Some(rest) = line.trim().strip_prefix("//").map(str::trim).and_then(|l| l.strip_prefix("@param")) else {
            continue;
        };
        let (head, label) = match rest.split_once('"') {
            Some((h, l)) => (h, l.trim_end().trim_end_matches('"').to_string()),
            None => (rest, String::new()),
        };
        let (name, tail) = head.split_once('=').unwrap_or((head, "0"));
        let name = name.trim().to_string();
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        let (def, range) = match tail.split_once('(') {
            Some((d, r)) => (d.trim(), Some(r.trim_end().trim_end_matches(')').trim())),
            None => (tail.trim(), None),
        };
        let default: f32 = def.parse().unwrap_or(0.0);
        let (min, max) = range
            .and_then(|r| r.split_once(".."))
            .and_then(|(a, b)| Some((a.trim().parse().ok()?, b.trim().parse().ok()?)))
            .unwrap_or(if (0.0..=1.0).contains(&default) { (0.0, 1.0) } else { (0.0f32.min(default * 2.0), default.abs() * 2.0) });
        let label = if label.is_empty() { name.clone() } else { label };
        out.push(ShaderParam { name, label, default, min, max });
        if out.len() == MAX_SHADER_PARAMS {
            break;
        }
    }
    out
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
    /// A physical daylight sky for the sun's position (Shadows card) and the haze
    /// ([`Environment::turbidity`]).
    Physical,
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
    pub effects: Effects,
    /// Haze of the physical sky (2 = clear, 10 = hazy).
    pub turbidity: f32,
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
            effects: Effects::default(),
            turbidity: 3.0,
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
        self.effects = self.effects.sanitized();
        self.turbidity = c(self.turbidity, 1.7, 12.0, d.turbidity);
        self
    }
}

/// A stylized (non-photorealistic) look of the Realistic style, drawn over the shaded image.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stylize {
    #[default]
    None,
    /// Pencil sketch: outlines and hatching on paper.
    Pencil,
    /// Watercolour: soft washes, darker pigment at the edges, paper grain.
    Watercolor,
    /// Technical drawing: thick outlines, thin creases, flat light fills.
    Technical,
}

impl Stylize {
    pub const ALL: [Stylize; 4] = [Self::None, Self::Pencil, Self::Watercolor, Self::Technical];

    pub fn key(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Pencil => "pencil",
            Self::Watercolor => "watercolor",
            Self::Technical => "technical",
        }
    }

    pub fn from_key(k: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.key() == k)
    }
}

/// Screen effects of the Realistic style; each one is switched on separately. Lengths in mm.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Effects {
    /// Soft shadows: the penumbra widens away from the occluder (PCSS).
    pub soft_shadows: bool,
    /// Apparent size of the sun (1 = a soft daylight sun).
    pub shadow_softness: f32,
    /// Ambient occlusion: darker creases and contact shadows.
    pub ao: bool,
    pub ao_radius: f32,
    pub ao_strength: f32,
    /// Screen-space reflections of the model in glossy surfaces.
    pub reflections: bool,
    pub reflection_strength: f32,
    /// Glow around bright light.
    pub bloom: bool,
    pub bloom_strength: f32,
    /// Light above this (scene units, 1 = white) glows.
    pub bloom_threshold: f32,
    /// Depth of field: objects away from the focus are blurred.
    pub dof: bool,
    /// Focus distance from the camera; 0 = what is in the middle of the view.
    pub focus_distance: f32,
    /// Blur strength (like a wide-open lens).
    pub aperture: f32,
    /// Fog / haze by distance.
    pub fog: bool,
    /// Distance at which the fog hides half of the light.
    pub fog_distance: f32,
    /// sRGB.
    pub fog_color: [f32; 3],
    /// Fog thins out upward (0 = even fog; 1 = haze near the ground).
    pub fog_height: f32,
    /// Darker corners, 0 = off.
    pub vignette: f32,
    /// All materials as one matte clay (shapes and light only).
    pub clay: bool,
    pub stylize: Stylize,
    /// More samples (slower, smoother).
    pub high_quality: bool,
}

impl Default for Effects {
    fn default() -> Self {
        Self {
            soft_shadows: false,
            shadow_softness: 1.0,
            ao: false,
            ao_radius: 600.0,
            ao_strength: 1.0,
            reflections: false,
            reflection_strength: 1.0,
            bloom: false,
            bloom_strength: 0.3,
            bloom_threshold: 1.0,
            dof: false,
            focus_distance: 0.0,
            aperture: 0.5,
            fog: false,
            fog_distance: 50_000.0,
            fog_color: [0.8, 0.84, 0.9],
            fog_height: 0.0,
            vignette: 0.0,
            clay: false,
            stylize: Stylize::None,
            high_quality: false,
        }
    }
}

/// Named sets of [`Effects`] (the toggles only; strengths keep their values).
pub const EFFECT_PRESETS: [&str; 4] = ["off", "fast", "balanced", "quality"];

impl Effects {
    /// Any screen pass needed (soft shadows and clay work without one).
    pub fn needs_post(&self) -> bool {
        self.ao || self.reflections || self.bloom || self.dof || self.fog || self.vignette > 0.0 || self.stylize != Stylize::None
    }

    /// Switch the effects of preset `name` (one of [`EFFECT_PRESETS`]) on, the others off.
    pub fn with_preset(mut self, name: &str) -> Option<Self> {
        let (soft, ao, refl, bloom, hq) = match name {
            "off" => (false, false, false, false, false),
            "fast" => (false, true, false, false, false),
            "balanced" => (true, true, true, true, false),
            "quality" => (true, true, true, true, true),
            _ => return None,
        };
        self.soft_shadows = soft;
        self.ao = ao;
        self.reflections = refl;
        self.bloom = bloom;
        self.high_quality = hq;
        if name == "off" {
            self.dof = false;
            self.fog = false;
            self.vignette = 0.0;
        } else if name == "quality" && self.vignette == 0.0 {
            self.vignette = 0.25;
        }
        Some(self)
    }

    /// Clamp the values to their ranges (NaN → default).
    pub fn sanitized(mut self) -> Self {
        let d = Self::default();
        let c = |v: f32, lo: f32, hi: f32, def: f32| if v.is_finite() { v.clamp(lo, hi) } else { def };
        self.shadow_softness = c(self.shadow_softness, 0.0, 20.0, d.shadow_softness);
        self.ao_radius = c(self.ao_radius, 1.0, 100_000.0, d.ao_radius);
        self.ao_strength = c(self.ao_strength, 0.0, 4.0, d.ao_strength);
        self.reflection_strength = c(self.reflection_strength, 0.0, 2.0, d.reflection_strength);
        self.bloom_strength = c(self.bloom_strength, 0.0, 4.0, d.bloom_strength);
        self.bloom_threshold = c(self.bloom_threshold, 0.0, 100.0, d.bloom_threshold);
        self.focus_distance = c(self.focus_distance, 0.0, 1e9, 0.0);
        self.aperture = c(self.aperture, 0.0, 4.0, d.aperture);
        self.fog_distance = c(self.fog_distance, 1.0, 1e9, d.fog_distance);
        for k in 0..3 {
            self.fog_color[k] = c(self.fog_color[k], 0.0, 1.0, d.fog_color[k]);
        }
        self.fog_height = c(self.fog_height, 0.0, 1.0, 0.0);
        self.vignette = c(self.vignette, 0.0, 1.0, 0.0);
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
        // Layers and details: off by default (old files), clamped when out of range.
        assert_eq!((back.clearcoat, back.clearcoat_roughness, back.sheen, back.sheen_tint), (0.0, 0.05, 0.0, 0.5));
        assert_eq!((back.anisotropy, back.anti_tiling, back.bevel), (0.0, 0.0, 0.0));
        let x = Pbr { clearcoat: 3.0, sheen: -1.0, anisotropy: -5.0, anti_tiling: f32::NAN, bevel: 1e6, ..Default::default() }.sanitized();
        assert_eq!((x.clearcoat, x.sheen, x.anisotropy, x.anti_tiling, x.bevel), (1.0, 0.0, -1.0, 0.0, 1000.0));
    }

    #[test]
    fn shader_param_header() {
        let src = "// @param scale = 4 (0.5..20) \"Ring scale\"\n// @param dark = 0.3\n//@param big = 50\n// @param bad name = 1\nfn surface(s: Surface) -> Surface { return s; }";
        let ps = shader_params(src);
        assert_eq!(ps.len(), 3);
        assert_eq!((ps[0].name.as_str(), ps[0].label.as_str(), ps[0].default, ps[0].min, ps[0].max), ("scale", "Ring scale", 4.0, 0.5, 20.0));
        assert_eq!((ps[1].label.as_str(), ps[1].min, ps[1].max), ("dark", 0.0, 1.0));
        assert_eq!((ps[2].min, ps[2].max), (0.0, 100.0));
        let sh = CustomShader::new(src);
        assert_eq!(sh.params, vec![4.0, 0.3, 50.0]);
        assert_eq!(CustomShader { params: vec![], ..sh.clone() }.param(1), 0.3);
    }

    #[test]
    fn effect_presets() {
        let e = Effects { fog: true, ..Default::default() };
        assert!(e.needs_post() && !Effects::default().needs_post());
        assert!(!Effects { soft_shadows: true, clay: true, ..Default::default() }.needs_post());
        let q = e.with_preset("quality").unwrap();
        assert!(q.ao && q.reflections && q.bloom && q.soft_shadows && q.high_quality && q.fog);
        let off = q.with_preset("off").unwrap();
        assert!(!off.needs_post() && !off.soft_shadows);
        assert!(e.with_preset("ultra").is_none());
        assert_eq!(Stylize::from_key("watercolor"), Some(Stylize::Watercolor));
        let env: Environment = serde_json::from_str("{\"exposure\":1.0}").unwrap();
        assert_eq!(env.effects, Effects::default());
    }
}
