//! The document: definitions (root, groups, components), instances, materials, tags.

use crate::dims::{DimAnchor, DimId};
use crate::guides::GuideRef;
use crate::mesh::Mesh;
use crate::{
    Aabb, DefId, EdgeId, EntityRef, FaceId, Instance, InstanceId, MaterialId, OpError, OpResult, Ray, TagId, VertexId,
};
use glam::{DAffine3, DMat3, DMat4, DVec3};
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use std::collections::{HashMap, HashSet};
use varstotas_i18n::tr;

/// `name` when not `taken`, else "<name> 2", "<name> 3", … (a trailing number of `name` is
/// replaced).
pub fn free_name(taken: HashSet<&str>, name: &str) -> String {
    if !taken.contains(name) {
        return name.to_string();
    }
    let base = match name.rsplit_once(' ') {
        Some((b, n)) if !b.is_empty() && n.parse::<u32>().is_ok() => b,
        _ => name,
    };
    (2..).map(|n| format!("{base} {n}")).find(|n| !taken.contains(n.as_str())).expect("a free name")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Definition {
    pub name: String,
    pub mesh: Mesh,
    /// Groups are definitions with exactly one instance and `is_group = true`.
    pub is_group: bool,
    /// Guides created inside this group / component (local coordinates; see
    /// [`crate::guides`]). Omitted from files when empty.
    #[serde(default, skip_serializing_if = "crate::guides::DefGuides::is_empty")]
    pub guides: crate::guides::DefGuides,
    /// Parametric group attributes (see [`crate::params`]). Omitted from files when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<crate::params::Parametric>,
    /// Built by a generator with these values (see [`crate::generator`]). Omitted when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated: Option<crate::generator::Generated>,
    /// Where a downloaded component came from: author, licence, link (File → Export →
    /// Attributions). Omitted when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Attribution>,
    /// Parameters of an ordinary component: this is the master (see [`crate::rig`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rig: Option<crate::rig::Rig>,
    /// A posed variant of a rig master, or a painted copy of a part. Omitted when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rigged: Option<crate::rig::Rigged>,
    /// A picture shown on the component's tile instead of a rendered thumbnail (PNG / JPEG,
    /// like [`Model::preview`]). Omitted when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<crate::texture::TextureImage>,
}

/// The origin of a downloaded model (3D Library): needed to credit CC-BY authors.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Attribution {
    /// The library / site ("Poly Haven", "Smithsonian", a plugin's name).
    pub source: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub author: String,
    /// "CC0", "CC-BY 4.0", … ("" = unknown).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub license: String,
    /// The model's web page (or the file's address).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
}

/// What a licence allows, read from its text ([`Attribution::license_kind`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LicenseKind {
    /// CC0 / public domain: no conditions.
    Cc0,
    /// CC-BY: credit the author.
    CcBy,
    /// CC-BY-SA: credit, share changes under the same licence.
    CcBySa,
    /// Non-commercial (CC-BY-NC, -NC-SA, -NC-ND).
    CcByNc,
    /// No derivatives (CC-BY-ND).
    CcByNd,
    /// A site's standard / royalty-free / personal-use licence.
    Standard,
    Unknown,
}

impl LicenseKind {
    /// The choices of the licence fields.
    pub const ALL: [LicenseKind; 7] = [LicenseKind::Cc0, LicenseKind::CcBy, LicenseKind::CcBySa, LicenseKind::CcByNc, LicenseKind::CcByNd, LicenseKind::Standard, LicenseKind::Unknown];

    /// Its short name (also the text stored when chosen from a list).
    pub fn label(self) -> String {
        varstotas_i18n::tr(match self {
            LicenseKind::Cc0 => "licence-cc0",
            LicenseKind::CcBy => "licence-cc-by",
            LicenseKind::CcBySa => "licence-cc-by-sa",
            LicenseKind::CcByNc => "licence-cc-by-nc",
            LicenseKind::CcByNd => "licence-cc-by-nd",
            LicenseKind::Standard => "licence-standard",
            LicenseKind::Unknown => "licence-unknown",
        })
    }

    /// The reminder for models under it (empty for CC0 / CC-BY).
    pub fn note(self) -> String {
        varstotas_i18n::tr(match self {
            LicenseKind::CcByNc => "licence-cc-by-nc-note",
            LicenseKind::CcByNd => "licence-cc-by-nd-note",
            LicenseKind::CcBySa => "licence-cc-by-sa-note",
            LicenseKind::Standard => "licence-standard-note",
            LicenseKind::Unknown => "licence-unknown-note",
            _ => return String::new(),
        })
    }

    /// Read a licence text ("CC BY-NC 4.0", "Creative Commons Attribution-ShareAlike",
    /// "Public Domain", "Royalty Free"…).
    pub fn parse(s: &str) -> LicenseKind {
        let t = s.to_lowercase().replace(['_', '-', '–'], " ");
        let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
        let has = |w: &str| t.split(|c: char| !c.is_alphanumeric()).any(|x| x == w);
        if t.is_empty() || t == "unknown" {
            return LicenseKind::Unknown;
        }
        if has("cc0") || t.contains("public domain") || t.contains("publicdomain") || t.contains("zero") {
            return LicenseKind::Cc0;
        }
        let cc = has("cc") || t.contains("creative commons") || t.contains("attribution");
        if cc {
            if has("nc") || t.contains("noncommercial") || t.contains("non commercial") {
                return LicenseKind::CcByNc;
            }
            if has("nd") || t.contains("noderivatives") || t.contains("no derivatives") || t.contains("noderivs") {
                return LicenseKind::CcByNd;
            }
            if has("sa") || t.contains("sharealike") || t.contains("share alike") {
                return LicenseKind::CcBySa;
            }
            return LicenseKind::CcBy;
        }
        if t.contains("royalty") || t.contains("standard") || t.contains("personal") || t.contains("editorial") || t.contains("guideline") || t.contains("local files") || t.contains("usage") {
            return LicenseKind::Standard;
        }
        LicenseKind::Unknown
    }
}

impl Attribution {
    /// What its licence allows.
    pub fn license_kind(&self) -> LicenseKind {
        LicenseKind::parse(&self.license)
    }

    /// One line: `Title by Author (Source, licence) — url`.
    pub fn line(&self) -> String {
        let mut s = self.title.clone();
        if !self.author.is_empty() {
            s += &format!(" by {}", self.author);
        }
        // i18n-ignore-next
        let lic = if self.license.is_empty() { "licence unknown" } else { &self.license };
        s += &format!(" ({}, {lic})", self.source);
        if !self.url.is_empty() {
            s += &format!(" — {}", self.url);
        }
        s
    }
}

impl Model {
    /// Credits of the downloaded components placed in the model (one per definition, sorted).
    pub fn attributions(&self) -> Vec<Attribution> {
        let mut used = std::collections::HashSet::new();
        for d in self.defs.values() {
            for i in d.mesh.instances.values() {
                used.insert(i.def);
            }
        }
        let mut out: Vec<Attribution> = self.defs.iter().filter(|(id, _)| used.contains(id)).filter_map(|(_, d)| d.source.clone()).collect();
        out.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()).then_with(|| a.source.cmp(&b.source)));
        out.dedup();
        out
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    /// Linear-ish sRGB 0..1 + alpha (opacity).
    pub color: [f32; 4],
    /// Texture file path from an imported MTL (`map_Kd`); informational.
    pub texture: Option<String>,
    /// Embedded texture image (see [`crate::texture`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<crate::texture::TextureImage>,
    /// Metallic-roughness parameters and maps (Realistic style, see [`crate::pbr`]); `None` =
    /// the defaults (a matte dielectric).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pbr: Option<crate::pbr::Pbr>,
}

impl Material {
    /// The material's PBR parameters (the defaults when unset).
    pub fn pbr_or_default(&self) -> crate::pbr::Pbr {
        self.pbr.clone().unwrap_or_default()
    }

    /// The image in texture slot `slot` ("base" or a PBR map name).
    pub fn slot_image(&self, slot: &str) -> Option<&crate::TextureImage> {
        match slot {
            "base" => self.image.as_ref(),
            _ => self.pbr.as_ref().and_then(|p| p.map(slot)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tag {
    #[serde(default)]
    pub locked: bool,
    pub name: String,
    pub visible: bool,
    pub color: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LengthUnit {
    Millimeter,
    Centimeter,
    Meter,
    Inch,
    Foot,
}

impl LengthUnit {
    /// Millimetres in one unit.
    pub fn mm(self) -> f64 {
        match self {
            LengthUnit::Millimeter => 1.0,
            LengthUnit::Centimeter => 10.0,
            LengthUnit::Meter => 1000.0,
            LengthUnit::Inch => 25.4,
            LengthUnit::Foot => 304.8,
        }
    }

    /// The short unit name (`mm`, `cm`, `m`, `in`, `ft`).
    pub fn suffix(self) -> &'static str {
        match self {
            LengthUnit::Millimeter => "mm",
            LengthUnit::Centimeter => "cm",
            LengthUnit::Meter => "m",
            LengthUnit::Inch => "in",
            LengthUnit::Foot => "ft",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub eye: DVec3,
    pub target: DVec3,
    pub up: DVec3,
    pub fov_y_deg: f64,
    pub ortho: bool,
    /// Display style saved with the scene (`None` in old files: the style is left as is).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<SceneStyle>,
    /// Tags hidden in this scene (`None` in old files: tag visibility is left as is).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden_tags: Option<Vec<TagId>>,
    /// Environment of the Realistic style in this scene (`None` in old files: left as is).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<crate::pbr::Environment>,
    /// Untagged entities hidden in this scene (`None` in old files: left as is).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub untagged_hidden: Option<bool>,
    /// Active section plane and section display saved with the scene (`None` in old files).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sections: Option<crate::section::SceneSections>,
    /// Shadow settings saved with the scene (`None` in old files).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadows: Option<crate::ShadowSettings>,
    /// Played by View → Animation (the classic "Include in animation").
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub in_animation: bool,
    /// Values of the components with parameters in this scene (`None`: left as they are).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rigs: Option<Vec<crate::rig::SceneRig>>,
}

fn yes() -> bool {
    true
}

fn is_true(b: &bool) -> bool {
    *b
}

/// Display style remembered by a scene. `face_style` is one of `shaded`, `shadedcolor`,
/// `realistic`, `monochrome`, `hiddenline`, `wireframe` (unknown values leave the style
/// unchanged).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneStyle {
    pub face_style: String,
    pub xray: bool,
    pub edges: bool,
    pub profiles: bool,
    #[serde(default)]
    pub triangles: bool,
}

/// Document Info → Animation: scene transitions and playback.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SceneSettings {
    /// Smooth camera transitions between scenes (off = cut).
    pub transitions: bool,
    /// Transition length, seconds.
    pub transition_secs: f64,
    /// Pause on each scene while playing, seconds.
    pub delay_secs: f64,
}

impl Default for SceneSettings {
    fn default() -> Self {
        Self { transitions: true, transition_secs: 2.0, delay_secs: 1.0 }
    }
}

impl Scene {
    /// A camera-only scene (no saved style or tags), included in the animation.
    pub fn camera(name: impl Into<String>, eye: DVec3, target: DVec3, up: DVec3, fov_y_deg: f64, ortho: bool) -> Self {
        Self { name: name.into(), eye, target, up, fov_y_deg, ortho, style: None, hidden_tags: None, environment: None, untagged_hidden: None, sections: None, shadows: None, in_animation: true, rigs: None }
    }
}

pub use crate::guides::GuideLine;

/// A path from the root into nested instances, identifying an editing context.
pub type ContextPath = Vec<InstanceId>;

/// Free-form document information; an empty licence makes no licensing claim.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelMetadata {
    pub text: String,
    pub license: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    #[serde(default)]
    pub metadata: ModelMetadata,
    pub defs: SlotMap<DefId, Definition>,
    pub root: DefId,
    pub materials: SlotMap<MaterialId, Material>,
    pub tags: SlotMap<TagId, Tag>,
    /// Environment light, exposure and tone mapping of the Realistic style.
    #[serde(default, skip_serializing_if = "crate::pbr::Environment::is_default")]
    pub environment: crate::pbr::Environment,
    /// HDRI panorama (`.hdr`, equirectangular) lighting the model when the environment's sky
    /// is [`crate::pbr::SkyKind::Hdri`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hdri: Option<crate::texture::TextureImage>,
    /// Light objects (path tracer and the Realistic style), world space.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lights: Vec<crate::lights::Light>,
    /// Settings of the Render window.
    #[serde(default, skip_serializing_if = "crate::lights::RenderSettings::is_default")]
    pub render: crate::lights::RenderSettings,
    /// Untagged top-level entities hidden (the eye of "Untagged" in the Tags card).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub untagged_hidden: bool,
    #[serde(default)]
    pub untagged_locked: bool,
    pub scenes: Vec<Scene>,
    #[serde(default)]
    pub scene_settings: SceneSettings,
    /// Guide lines (world space). Serialized as a plain array; see [`crate::guides`].
    #[serde(with = "crate::guides::slotmap_vec", default)]
    pub guides: crate::guides::GuideLines,
    /// Guide points (world space). Serialized as a plain array.
    #[serde(with = "crate::guides::slotmap_vec", default)]
    pub guide_points: crate::guides::GuidePoints,
    /// View > Guides off: guides are not drawn, picked or inferred.
    #[serde(default)]
    pub guides_hidden: bool,
    pub unit: LengthUnit,
    /// Custom drawing axes (Tools > Axes): origin = translation, columns of the matrix = unit
    /// red / green / blue directions (orthonormal, right-handed). `None` = world axes.
    /// Omitted from files when unset. See [`Model::set_axes`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub axes: Option<DAffine3>,
    /// Section planes (world space), see [`crate::section`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<crate::SectionPlane>,
    #[serde(default)]
    pub section_display: crate::SectionDisplay,
    #[serde(default)]
    pub shadows: crate::ShadowSettings,
    /// Layout sheets (paper with viewports, texts, title block).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sheets: Vec<crate::layout::LayoutSheet>,
    /// Work planes (world space), see [`crate::workplane`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub work_planes: Vec<crate::WorkPlane>,
    /// Index of the active work plane (drawing happens on it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_plane: Option<usize>,
    /// Generator sources by name (see [`crate::generator`]).
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub generators: std::collections::BTreeMap<String, crate::generator::Generator>,
    /// Preview picture of the file (PNG or JPEG), shown by the Components library instead of
    /// the drawn edges; `None` = the library draws the model. See [`Model::set_preview`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<crate::texture::TextureImage>,
}

/// Result of a model-wide ray pick.
#[derive(Clone, Debug)]
pub struct ModelHit {
    /// Instance chain from the root to the mesh containing `face`.
    pub path: ContextPath,
    pub face: FaceId,
    pub point: DVec3,
    pub t: f64,
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

impl Model {
    /// Empty model: a root definition, no materials/tags, millimetres.
    pub fn new() -> Self {
        let mut defs = SlotMap::with_key();
        // i18n-ignore-next
        let root = defs.insert(Definition { name: "Model".into(), mesh: Mesh::new(), is_group: false, guides: Default::default(), params: None, generated: None, source: None, rig: None, rigged: None, preview: None });
        Self {
            metadata: ModelMetadata::default(),
            defs,
            root,
            materials: SlotMap::with_key(),
            tags: SlotMap::with_key(),
            untagged_hidden: false,
            untagged_locked: false,
            environment: Default::default(),
            hdri: None,
            lights: Vec::new(),
            render: Default::default(),
            scenes: Vec::new(),
            scene_settings: SceneSettings::default(),
            guides: SlotMap::with_key(),
            guide_points: SlotMap::with_key(),
            guides_hidden: false,
            unit: LengthUnit::Millimeter,
            axes: None,
            sections: Vec::new(),
            section_display: Default::default(),
            shadows: Default::default(),
            sheets: Vec::new(),
            work_planes: Vec::new(),
            active_plane: None,
            generators: Default::default(),
            preview: None,
        }
    }

    /// Set (PNG / JPEG bytes) or clear the file's preview picture.
    pub fn set_preview(&mut self, image: Option<Vec<u8>>) {
        self.preview = image.map(|b| crate::texture::TextureImage::new(b, "preview", 0.0));
    }
    pub fn root_mesh(&self) -> &Mesh {
        &self.defs[self.root].mesh
    }
    pub fn root_mesh_mut(&mut self) -> &mut Mesh {
        let r = self.root;
        &mut self.defs[r].mesh
    }

    /// Definition reached through `path` (empty path = root).
    pub fn def_at(&self, path: &[InstanceId]) -> Option<DefId> {
        let mut d = self.root;
        for i in path {
            d = self.defs.get(d)?.mesh.instances.get(*i)?.def;
        }
        self.defs.contains_key(d).then_some(d)
    }

    /// Mesh of the definition reached through `path` (empty path = root).
    pub fn mesh_at(&self, path: &[InstanceId]) -> Option<&Mesh> {
        let d = self.def_at(path)?;
        Some(&self.defs[d].mesh)
    }
    pub fn mesh_at_mut(&mut self, path: &[InstanceId]) -> Option<&mut Mesh> {
        let d = self.def_at(path)?;
        Some(&mut self.defs[d].mesh)
    }

    /// Definition `d` contains `target` (itself or nested at any depth).
    pub fn def_contains(&self, d: DefId, target: DefId) -> bool {
        fn walk(m: &Model, d: DefId, target: DefId, depth: usize) -> bool {
            d == target || (depth < 64 && m.defs.get(d).is_some_and(|x| x.mesh.instances.values().any(|i| walk(m, i.def, target, depth + 1))))
        }
        walk(self, d, target, 0)
    }

    /// Whether instance `id` of the context `from` can move into the context `to`: both exist,
    /// `to` is another context and not inside the instance (no definition would contain itself).
    pub fn can_move_instance(&self, from: &[InstanceId], id: InstanceId, to: &[InstanceId]) -> bool {
        let (Some(inst), Some(dest)) = (self.mesh_at(from).and_then(|m| m.instances.get(id)), self.def_at(to)) else { return false };
        from != to && !self.def_contains(inst.def, dest)
    }

    /// Move instance `id` from the context `from` into the context `to` (paths from the root),
    /// keeping where it is in the world. Returns its id there.
    pub fn move_instance(&mut self, from: &[InstanceId], id: InstanceId, to: &[InstanceId]) -> Option<InstanceId> {
        if !self.can_move_instance(from, id, to) {
            return None;
        }
        let world = self.transform_at(from);
        let dest = self.transform_at(to);
        let mut inst = self.mesh_at_mut(from)?.instances.remove(id)?;
        inst.transform = dest.inverse() * world * inst.transform;
        Some(self.mesh_at_mut(to)?.instances.insert(inst))
    }

    /// Remove instance `id` from the context `path`.
    pub fn remove_instance(&mut self, path: &[InstanceId], id: InstanceId) -> bool {
        self.mesh_at_mut(path).and_then(|m| m.instances.remove(id)).is_some()
    }

    /// World bounds of instance `id` of the context `path`.
    pub fn instance_world_bounds(&self, path: &[InstanceId], id: InstanceId) -> Option<Aabb> {
        let inst = self.mesh_at(path)?.instances.get(id)?;
        let b = self.def_bounds(inst.def)?;
        let xf = self.transform_at(path) * inst.transform;
        let corners = (0..8).map(|i| {
            let p = DVec3::new(if i & 1 == 0 { b.min.x } else { b.max.x }, if i & 2 == 0 { b.min.y } else { b.max.y }, if i & 4 == 0 { b.min.z } else { b.max.z });
            xf.transform_point3(p)
        });
        Aabb::from_points(corners)
    }

    /// World transform of the context reached through `path`.
    pub fn transform_at(&self, path: &[InstanceId]) -> DMat4 {
        let mut m = DMat4::IDENTITY;
        let mut d = self.root;
        for i in path {
            let Some(inst) = self.defs.get(d).and_then(|def| def.mesh.instances.get(*i)) else { break };
            m *= inst.transform;
            d = inst.def;
        }
        m
    }

    /// Whether entities of definition `d` with tag `t` are shown. Untagged entities hide with
    /// [`Model::untagged_hidden`] only at the top level: inside a group or component they
    /// belong to it (its own tag decides).
    pub fn tag_visible(&self, d: DefId, t: Option<TagId>) -> bool {
        match t {
            Some(t) => self.tags.get(t).is_none_or(|t| t.visible),
            None => !(self.untagged_hidden && d == self.root),
        }
    }

    /// Local bounds of a definition including nested instances.
    pub fn def_bounds(&self, d: DefId) -> Option<Aabb> {
        fn visit(
            m: &Model, d: DefId, depth: usize,
            cache: &mut HashMap<DefId, Option<Aabb>>,
            active: &mut HashSet<DefId>,
        ) -> Option<Aabb> {
        if depth > 64 || active.contains(&d) {
            return None;
            }
            if let Some(b) = cache.get(&d) {
                return *b;
        }
        let mesh = &m.defs.get(d)?.mesh;
            active.insert(d);
        let mut b = mesh.bounds();
        for inst in mesh.instances.values() {
            if let Some(cb) = visit(m, inst.def, depth + 1, cache, active) {
                let wb = cb.transformed(&inst.transform);
                b = Some(b.map_or(wb, |x| x.union(&wb)));
            }
        }
            active.remove(&d);
            cache.insert(d, b);
            b
        }
        visit(self, d, 0, &mut HashMap::new(), &mut HashSet::new())
    }

    /// Closest visible face hit, descending into visible instances.
    pub fn raycast(&self, ray: &Ray) -> Option<ModelHit> {
        // The active section plane cuts the ray to its visible part [t0, t1].
        let (mut t0, mut t1) = (0.0, f64::INFINITY);
        if let Some(s) = self.active_section() {
            let (d0, dd) = (s.dist(ray.origin), s.normal.dot(ray.dir));
            if dd.abs() < 1e-15 {
                if d0 < 0.0 {
                    return None;
                }
            } else if dd > 0.0 {
                t0 = (-d0 / dd).max(0.0);
            } else {
                t1 = -d0 / dd;
                if t1 < 0.0 {
                    return None;
                }
            }
        }
        let shifted = Ray { origin: ray.at(t0), dir: ray.dir };
        let mut best: Option<ModelHit> = None;
        self.raycast_rec(self.root, DMat4::IDENTITY, &mut Vec::new(), &shifted, &mut best, 0);
        best.map(|mut h| {
            h.t += t0;
            h
        })
        .filter(|h| h.t <= t1 + crate::TOL)
    }

    fn raycast_rec(
        &self,
        d: DefId,
        m: DMat4,
        path: &mut Vec<InstanceId>,
        ray: &Ray,
        best: &mut Option<ModelHit>,
        depth: usize,
    ) {
        if depth > 64 {
            return;
        }
        let Some(def) = self.defs.get(d) else { return };
        let inv = m.inverse();
        let lo = inv.transform_point3(ray.origin);
        let ld = inv.transform_vector3(ray.dir);
        if let Some((f, t)) = def.mesh.raycast_faces_filtered(lo, ld, |face| !face.hidden && self.tag_visible(d, face.tag))
            && best.as_ref().is_none_or(|b| t < b.t) {
                *best = Some(ModelHit { path: path.clone(), face: f, point: ray.at(t), t });
            }
        for (iid, inst) in &def.mesh.instances {
            if inst.hidden || !self.tag_visible(d, inst.tag) {
                continue;
            }
            let cm = m * inst.transform;
            if let Some(b) = self.def_bounds(inst.def) {
                let wb = b.transformed(&cm).inflate(crate::TOL);
                match wb.ray_hit(ray.origin, ray.dir) {
                    None => continue,
                    Some(t) if best.as_ref().is_some_and(|h| t > h.t) => continue,
                    _ => {}
                }
            } else {
                continue;
            }
            path.push(iid);
            self.raycast_rec(inst.def, cm, path, ray, best, depth + 1);
            path.pop();
        }
    }

    /// World bounds of the whole model.
    pub fn bounds(&self) -> Option<Aabb> {
        self.def_bounds(self.root)
    }

    fn new_def_from(
        &mut self,
        path: &[InstanceId],
        ents: &[EntityRef],
        guides: &[GuideRef],
        dims: &[DimId],
        name: &str,
        is_group: bool,
    ) -> OpResult<InstanceId> {
        let pd = self.def_at(path).ok_or(OpError::NotFound)?;
        let guides: Vec<GuideRef> = guides.iter().copied().filter(|g| self.has_guide_in(path, *g)).collect();
        let dims: Vec<DimId> = dims.iter().copied().filter(|d| self.defs[pd].mesh.dims.contains_key(*d)).collect();
        if ents.is_empty() && guides.is_empty() && dims.is_empty() {
            return Err(OpError::Invalid(tr!("kernel-err-nothing-selected")));
        }
        let parent = &mut self.defs[pd].mesh;
        parent.ensure();
        let mut faces: Vec<FaceId> = Vec::new();
        let mut edges: Vec<EdgeId> = Vec::new();
        let mut verts: Vec<VertexId> = Vec::new();
        let mut insts: Vec<InstanceId> = Vec::new();
        for e in ents {
            match *e {
                EntityRef::Face(f) if parent.faces.contains_key(f) => {
                    if !faces.contains(&f) {
                        faces.push(f);
                    }
                    for e in parent.face_edges(f) {
                        if !edges.contains(&e) {
                            edges.push(e);
                        }
                    }
                }
                EntityRef::Edge(e) if parent.edges.contains_key(e) => {
                    if !edges.contains(&e) {
                        edges.push(e);
                    }
                }
                EntityRef::Vertex(v) if parent.vertices.contains_key(v) => verts.push(v),
                EntityRef::Instance(i) if parent.instances.contains_key(i)
                    && !insts.contains(&i) => {
                        insts.push(i);
                    }
                _ => {}
            }
        }
        // Build the new mesh.
        let mut mesh = Mesh::new();
        let mut vmap: HashMap<VertexId, VertexId> = HashMap::new();
        let mut touched: Vec<VertexId> = verts.clone();
        for e in &edges {
            let ed = parent.edges[*e].clone();
            touched.extend(ed.v);
            let mut ids = [ed.v[0], ed.v[1]];
            for x in ids.iter_mut() {
                *x = *vmap.entry(*x).or_insert_with(|| mesh.v_new(parent.vertices[*x].pos));
            }
            mesh.e_new(ids[0], ids[1], Some(&ed));
        }
        for v in &verts {
            vmap.entry(*v).or_insert_with(|| mesh.v_new(parent.vertices[*v].pos));
        }
        // Curve metadata of the moved curves (same ids: the new mesh is fresh).
        for ed in mesh.edges.values() {
            if let Some(c) = ed.curve
                && let Some(info) = parent.curves.get(&c)
            {
                mesh.curves.insert(c, *info);
            }
            if let Some(c) = ed.curve
                && let Some(sp) = parent.splines.get(&c)
            {
                mesh.splines.insert(c, sp.clone());
            }
        }
        // Selected dimensions move along; anchors on moved vertices stay attached.
        for id in &dims {
            let Some(mut d) = parent.dims.remove(*id) else { continue };
            for a in [&mut d.a, &mut d.b] {
                let p = parent.anchor_pos(a);
                *a = match a.vertex.and_then(|v| vmap.get(&v)) {
                    Some(nv) => DimAnchor { vertex: Some(*nv), point: p },
                    None => DimAnchor::free(p),
                };
            }
            mesh.dims.insert(d);
        }
        for f in &faces {
            let face = parent.faces[*f].clone();
            let outer = face.outer.iter().map(|v| vmap[v]).collect();
            let holes = face.holes.iter().map(|h| h.iter().map(|v| vmap[v]).collect()).collect();
            mesh.f_new(outer, holes, &face);
        }
        for i in &insts {
            let inst = parent.instances.remove(*i).unwrap();
            mesh.instances.insert(inst);
        }
        // Remove from parent: faces, then edges no longer used by remaining faces.
        for f in &faces {
            parent.f_remove(*f);
        }
        for e in &edges {
            if parent.edges.contains_key(*e) && parent.edge_faces(*e).is_empty() {
                parent.e_remove(*e);
            }
        }
        for v in touched {
            parent.remove_if_orphan(v);
        }
        parent.prune_curves();
        // Selected guides move into the new definition (identity transform: same coordinates).
        let mut def_guides = crate::guides::DefGuides::default();
        if let Some((lines, points)) = self.guide_store_mut(path) {
            for g in &guides {
                match *g {
                    GuideRef::Line(id) => {
                        if let Some(l) = lines.remove(id) {
                            def_guides.lines.insert(l);
                        }
                    }
                    GuideRef::Point(id) => {
                        if let Some(p) = points.remove(id) {
                            def_guides.points.insert(p);
                        }
                    }
                }
            }
        }
        let def = self.defs.insert(Definition { name: name.to_string(), mesh, is_group, guides: def_guides, params: None, generated: None, source: None, rig: None, rigged: None, preview: None });
        let inst = Instance {
            def,
            transform: DMat4::IDENTITY,
            name: String::new(),
            material: None,
            tag: None,
            hidden: false,
            locked: false,
        };
        Ok(self.defs[pd].mesh.instances.insert(inst))
    }

    /// Move `ents` from the mesh at `path` into a new group; returns the new instance.
    /// Edges still used by faces that stay in the parent are copied (they remain in both).
    pub fn make_group(&mut self, path: &[InstanceId], ents: &[EntityRef]) -> OpResult<InstanceId> {
        // i18n-ignore-next
        self.new_def_from(path, ents, &[], &[], "Group", true)
    }

    /// A new group of the context `path` holding `mesh` (in the group's own axes) at `transform`
    /// (made geometry, e.g. a draped cloth; no welding with the context).
    pub fn add_group_mesh(&mut self, path: &[InstanceId], mesh: Mesh, name: &str, transform: DMat4) -> Option<InstanceId> {
        self.mesh_at(path)?;
        let def = self.defs.insert(Definition { name: name.into(), mesh, is_group: true, guides: Default::default(), params: None, generated: None, source: None, rig: None, rigged: None, preview: None });
        let inst = Instance { def, transform, name: String::new(), material: None, tag: None, hidden: false, locked: false };
        self.mesh_at_mut(path).map(|m| m.instances.insert(inst))
    }

    /// Like make_group but the definition is a reusable component.
    pub fn make_component(&mut self, path: &[InstanceId], ents: &[EntityRef], name: &str) -> OpResult<InstanceId> {
        self.new_def_from(path, ents, &[], &[], name, false)
    }

    /// [`Model::make_group`] that also moves the selected guides and dimensions of the context
    /// `path` into the new group. Works with only guides / dimensions selected too.
    pub fn make_group_with(
        &mut self,
        path: &[InstanceId],
        ents: &[EntityRef],
        guides: &[GuideRef],
        dims: &[DimId],
    ) -> OpResult<InstanceId> {
        // i18n-ignore-next
        self.new_def_from(path, ents, guides, dims, "Group", true)
    }

    /// [`Model::make_component`] that also moves the selected guides and dimensions.
    pub fn make_component_with(
        &mut self,
        path: &[InstanceId],
        ents: &[EntityRef],
        guides: &[GuideRef],
        dims: &[DimId],
        name: &str,
    ) -> OpResult<InstanceId> {
        self.new_def_from(path, ents, guides, dims, name, false)
    }

    /// Number of instances (anywhere in the model) that use definition `d`.
    pub fn instance_count(&self, d: DefId) -> usize {
        self.defs.values().map(|def| def.mesh.instances.values().filter(|i| i.def == d).count()).sum()
    }

    /// Remove definitions (other than the root) that are no longer instanced.
    pub fn purge_unused_defs(&mut self) {
        loop {
            let bases = self.param_bases();
            let unused: Vec<DefId> = self
                .defs
                .keys()
                .filter(|d| *d != self.root && self.instance_count(*d) == 0 && !bases.contains(d))
                .collect();
            if unused.is_empty() {
                break;
            }
            for d in unused {
                self.defs.remove(d);
            }
        }
    }

    /// Explode an instance: copy its transformed geometry into the parent mesh (vertices merge
    /// with coincident parent vertices; existing edges/faces are reused). The instance material
    /// is applied to faces without a material. Returns the new entities in the parent.
    pub fn explode(&mut self, path: &[InstanceId], inst: InstanceId) -> OpResult<Vec<EntityRef>> {
        let pd = self.def_at(path).ok_or(OpError::NotFound)?;
        let instance = self.defs[pd].mesh.instances.get(inst).ok_or(OpError::NotFound)?.clone();
        let child = self.defs.get(instance.def).ok_or(OpError::NotFound)?.mesh.clone();
        let t = instance.transform;
        // Guides inside the exploded group / component move to the parent context.
        self.lift_guides(path, instance.def, &t);
        let mirror = t.determinant() < 0.0;
        let parent = &mut self.defs[pd].mesh;
        parent.ensure();
        parent.instances.remove(inst);
        let mut out = Vec::new();
        let mut vmap: HashMap<VertexId, VertexId> = HashMap::new();
        for (v, vd) in &child.vertices {
            let nv = parent.v_get_or_new(t.transform_point3(vd.pos));
            vmap.insert(v, nv);
        }
        // Dimensions are lifted to the parent; anchors stay on the (exploded) vertices.
        for d in child.dims.values() {
            let mut d = d.clone();
            for a in [&mut d.a, &mut d.b] {
                let p = t.transform_point3(child.anchor_pos(a));
                *a = match a.vertex.and_then(|v| vmap.get(&v)) {
                    Some(nv) => DimAnchor { vertex: Some(*nv), point: p },
                    None => DimAnchor::free(p),
                };
            }
            d.offset = t.transform_vector3(d.offset);
            parent.dims.insert(d);
        }
        // Curves get fresh ids in the parent (metadata transformed with the instance).
        let cmap = parent.remap_curves(&child.curves, &child.splines, child.edges.values().filter_map(|e| e.curve), &t);
        for (_, ed) in &child.edges {
            let (a, b) = (vmap[&ed.v[0]], vmap[&ed.v[1]]);
            if a == b {
                continue;
            }
            let e = match parent.edge_between(a, b) {
                Some(e) => e,
                None => {
                    let mut tmpl = ed.clone();
                    tmpl.curve = tmpl.curve.and_then(|c| cmap.get(&c).copied());
                    parent.e_new(a, b, Some(&tmpl))
                }
            };
            out.push(EntityRef::Edge(e));
        }
        for (_, face) in &child.faces {
            let mut outer: Vec<VertexId> = face.outer.iter().map(|v| vmap[v]).collect();
            let mut holes: Vec<Vec<VertexId>> = face.holes.iter().map(|h| h.iter().map(|v| vmap[v]).collect()).collect();
            if mirror {
                outer.reverse();
                for h in &mut holes {
                    h.reverse();
                }
            }
            if let Some(ex) = parent.find_face_with_outer(&outer) {
                out.push(EntityRef::Face(ex));
                continue;
            }
            let mut tmpl = face.clone();
            if tmpl.front.is_none() {
                tmpl.front = instance.material;
            }
            let f = parent.f_new(outer, holes, &tmpl);
            // Keep the texture where it was on the instance.
            let pts: Vec<(VertexId, DVec3)> = face.loops().flatten().map(|v| (vmap[v], child.vertices[*v].pos)).collect();
            parent.carry_texture(f, face.normal, face.uv.as_ref(), &pts);
            out.push(EntityRef::Face(f));
        }
        for (_, ci) in &child.instances {
            let mut c = ci.clone();
            c.transform = t * c.transform;
            if c.material.is_none() {
                c.material = instance.material;
            }
            out.push(EntityRef::Instance(parent.instances.insert(c)));
        }
        // Weld to the parent's geometry: T-junctions, crossings, faces split by edges lying in
        // them and coplanar overlaps merged.
        let mut verts: Vec<VertexId> = vmap.values().copied().collect();
        verts.sort(); // deterministic welding order
        verts.dedup();
        let out = parent.absorb_refs(&verts, out);
        if self.instance_count(instance.def) == 0 && instance.def != self.root && !self.param_bases().contains(&instance.def) {
            self.defs.remove(instance.def);
        }
        Ok(out)
    }

    /// Whether Make Unique applies to instance `inst` (context `path`): other copies share its
    /// content — the same definition, or for a component with parameters the same master
    /// (edits inside any copy go to the master).
    pub fn can_make_unique(&self, path: &[InstanceId], inst: InstanceId) -> bool {
        let Some(d) = self.mesh_at(path).and_then(|m| m.instances.get(inst)).map(|i| i.def) else { return false };
        if let Some(g) = self.defs.get(d).and_then(|x| x.generated.as_ref()) {
            return self.generator_uses(&g.generator) > 1;
        }
        match self.rig_master(d) {
            Some(m) if m != d => self.placed_count(m) > 1,
            _ => self.instance_count(d) > 1,
        }
    }

    /// Give an instance its own copy of the definition, named "<name> 2" (the next free number).
    /// A copy of a component with parameters gets its own master too (later edits inside it
    /// leave the other copies alone); a generated one its own generator — a copy of the script
    /// named "<generator> 2", edited apart from the first. Nested components stay shared.
    pub fn make_unique(&mut self, path: &[InstanceId], inst: InstanceId) -> OpResult<()> {
        let pd = self.def_at(path).ok_or(OpError::NotFound)?;
        let d = self.defs[pd].mesh.instances.get(inst).ok_or(OpError::NotFound)?.def;
        if !self.can_make_unique(path, inst) {
            return Ok(());
        }
        let mut copy = self.defs[d].clone();
        if let Some(g) = copy.generated.as_mut() {
            let new = self.fork_generator(&g.generator).ok_or(OpError::Invalid(tr!("kernel-err-the-generator-s-script-can-t")))?;
            g.generator = new.clone();
            copy.name = new;
            let nd = self.defs.insert(copy);
            self.defs[pd].mesh.instances[inst].def = nd;
            return Ok(());
        }
        match self.rig_master(d).filter(|m| *m != d) {
            Some(master) => {
                let mut mc = self.defs[master].clone();
                mc.name = self.free_def_name(&mc.name);
                copy.name = mc.name.clone();
                let nm = self.defs.insert(mc);
                if let Some(r) = copy.rigged.as_mut() {
                    r.master = nm;
                }
            }
            None => copy.name = self.free_def_name(&copy.name),
        }
        let nd = self.defs.insert(copy);
        self.defs[pd].mesh.instances[inst].def = nd;
        Ok(())
    }

    /// `name` when no definition has it, else "<name> 2", "<name> 3", … (a trailing number of
    /// `name` is replaced).
    pub fn free_def_name(&self, name: &str) -> String {
        free_name(self.defs.values().map(|d| d.name.as_str()).collect(), name)
    }

    /// A hash of definition `d`'s content: geometry (to 0.001 mm), materials and tags by name,
    /// edge flags, nested copies (their content and placement), parameters. Equal for copies
    /// of one content in different models (a library component and its placed definition).
    pub fn content_hash(&self, d: DefId) -> u64 {
        self.content_hash_in(d, 0)
    }

    fn content_hash_in(&self, d: DefId, depth: usize) -> u64 {
        use std::hash::{Hash, Hasher};
        fn h<T: Hash>(x: T) -> u64 {
            let mut s = std::collections::hash_map::DefaultHasher::new();
            x.hash(&mut s);
            s.finish()
        }
        let q = |p: DVec3| [(p.x * 1000.0).round() as i64, (p.y * 1000.0).round() as i64, (p.z * 1000.0).round() as i64];
        let Some(def) = self.defs.get(d) else { return 0 };
        if depth > 32 {
            return 0;
        }
        let m = &def.mesh;
        let mat = |x: Option<MaterialId>| x.and_then(|x| self.materials.get(x)).map(|x| x.name.as_str());
        let tag = |x: Option<TagId>| x.and_then(|x| self.tags.get(x)).map(|x| x.name.as_str());
        let pos = |v: VertexId| m.vertices.get(v).map(|x| q(x.pos));
        let mut faces: Vec<u64> = m
            .faces
            .values()
            .map(|f| {
                let loops: Vec<Vec<_>> = f.loops().map(|l| l.iter().map(|v| pos(*v)).collect()).collect();
                h((loops, mat(f.front), mat(f.back), f.hidden, f.locked, tag(f.tag)))
            })
            .collect();
        let mut edges: Vec<u64> = m
            .edges
            .values()
            .map(|e| {
                let mut ends = [pos(e.v[0]), pos(e.v[1])];
                ends.sort();
                h((ends, e.soft, e.smooth, e.hidden, e.locked, tag(e.tag)))
            })
            .collect();
        let mut insts: Vec<u64> = m
            .instances
            .values()
            .map(|i| {
                let xf: Vec<i64> = i.transform.to_cols_array().iter().map(|x| (x * 1000.0).round() as i64).collect();
                h((self.content_hash_in(i.def, depth + 1), xf, &i.name, mat(i.material), tag(i.tag), i.hidden, i.locked))
            })
            .collect();
        faces.sort_unstable();
        edges.sort_unstable();
        insts.sort_unstable();
        let extra = format!("{:?}{:?}{}", def.rig, def.generated, def.is_group);
        h((faces, edges, insts, extra))
    }

    pub fn add_material(&mut self, name: &str, color: [f32; 4]) -> MaterialId {
        self.materials.insert(Material { name: name.to_string(), color, texture: None, image: None, pbr: None })
    }

    pub fn add_tag(&mut self, name: &str) -> TagId {
        self.tags.insert(Tag { locked: false, name: name.to_string(), visible: true, color: [0.5, 0.5, 0.5, 1.0] })
    }

    /// Delete tag `t`: its faces, edges, groups and components become untagged (the classic
    /// "Assign to Untagged"), scenes forget it. False when there is no such tag.
    pub fn delete_tag(&mut self, t: TagId) -> bool {
        if self.tags.remove(t).is_none() {
            return false;
        }
        let un = |x: &mut Option<TagId>| {
            if *x == Some(t) {
                *x = None;
            }
        };
        for d in self.defs.values_mut() {
            d.mesh.faces.values_mut().for_each(|f| un(&mut f.tag));
            d.mesh.edges.values_mut().for_each(|e| un(&mut e.tag));
            d.mesh.instances.values_mut().for_each(|i| un(&mut i.tag));
        }
        for s in &mut self.scenes {
            if let Some(h) = &mut s.hidden_tags {
                h.retain(|x| *x != t);
            }
        }
        true
    }

    /// Entities (faces, edges, instances) tagged `t`, in all definitions.
    pub fn tag_uses(&self, t: TagId) -> usize {
        self.defs
            .values()
            .map(|d| {
                d.mesh.faces.values().filter(|f| f.tag == Some(t)).count()
                    + d.mesh.edges.values().filter(|e| e.tag == Some(t)).count()
                    + d.mesh.instances.values().filter(|i| i.tag == Some(t)).count()
            })
            .sum()
    }

    /// Set custom drawing axes: origin plus red (`x`) and green (`y`) directions. `y` is made
    /// perpendicular to `x`; blue = `x × y`. `Err(Degenerate)` for zero / parallel directions.
    pub fn set_axes(&mut self, origin: DVec3, x: DVec3, y: DVec3) -> OpResult<()> {
        let xa = x.try_normalize().ok_or(OpError::Degenerate)?;
        let ya = (y - xa * y.dot(xa)).try_normalize().ok_or(OpError::Degenerate)?;
        if (y - xa * y.dot(xa)).length() < 1e-9 * y.length().max(1.0) {
            return Err(OpError::Degenerate);
        }
        let za = xa.cross(ya);
        let frame = DAffine3::from_mat3_translation(DMat3::from_cols(xa, ya, za), origin);
        self.axes = if frame == DAffine3::IDENTITY { None } else { Some(frame) };
        Ok(())
    }

    /// Back to the world axes (Reset in the Axes context menu).
    pub fn reset_axes(&mut self) {
        self.axes = None;
    }

    /// The drawing-axes frame (identity when unset): maps axes coordinates to world.
    pub fn axes_frame(&self) -> DAffine3 {
        self.axes.unwrap_or(DAffine3::IDENTITY)
    }

    /// Rebuild caches of every mesh (call after deserializing a model by hand).
    pub fn rebuild_caches(&mut self) {
        for d in self.defs.values_mut() {
            d.mesh.rebuild_caches();
        }
    }
}

#[cfg(test)]
mod move_tests {
    use super::*;

    fn group(m: &mut Model, parent: &[InstanceId], name: &str, at: DVec3) -> (DefId, InstanceId) {
        let d = m.defs.insert(Definition { name: name.into(), mesh: Mesh::new(), is_group: true, guides: Default::default(), params: None, generated: None, source: None, rig: None, rigged: None, preview: None });
        m.defs[d].mesh.add_face(&[DVec3::ZERO, DVec3::X * 100.0, DVec3::new(100.0, 100.0, 0.0), DVec3::Y * 100.0]);
        let inst = Instance { def: d, transform: DMat4::from_translation(at), name: name.into(), material: None, tag: None, hidden: false, locked: false };
        let id = m.mesh_at_mut(parent).unwrap().instances.insert(inst);
        (d, id)
    }

    #[test]
    fn an_instance_moves_between_contexts_in_place() {
        let mut m = Model::new();
        let (_, a) = group(&mut m, &[], "A", DVec3::new(1000.0, 0.0, 0.0));
        let (_, b) = group(&mut m, &[], "B", DVec3::new(0.0, 500.0, 0.0));
        let before = m.instance_world_bounds(&[], b).unwrap();
        // B into A: same place in the world.
        let nb = m.move_instance(&[], b, &[a]).unwrap();
        assert_eq!(m.root_mesh().instances.len(), 1);
        let after = m.instance_world_bounds(&[a], nb).unwrap();
        assert!((after.min - before.min).length() < 1e-9 && (after.max - before.max).length() < 1e-9, "{after:?} {before:?}");
        // A can't go into B (now inside A), nor into its own context.
        assert!(!m.can_move_instance(&[], a, &[a, nb]));
        assert!(!m.can_move_instance(&[], a, &[]));
        // Back out to the top level.
        let back = m.move_instance(&[a], nb, &[]).unwrap();
        assert_eq!(m.root_mesh().instances.len(), 2);
        let out = m.instance_world_bounds(&[], back).unwrap();
        assert!((out.min - before.min).length() < 1e-9);
        assert!(m.remove_instance(&[], back));
        assert_eq!(m.root_mesh().instances.len(), 1);
    }
}
