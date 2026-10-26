//! Guides (construction geometry): infinite guide lines and guide points. Owner: app agent
//! (M4 "guides as first-class entities").
//!
//! As usual, guides belong to an editing context: model-level guides live in
//! [`Model::guides`] / [`Model::guide_points`] (world space); guides created inside a group or
//! component live in its [`Definition::guides`](crate::model::Definition) in the definition's
//! local coordinates, so they move / scale with it and are shared by all instances of a
//! component. A [`GuideRef`] (like an `EntityRef`) is relative to a context path; the `*_in`
//! methods take that path and work in world space.
//!
//! Guides have stable slotmap ids so they can be selected, moved and deleted like other
//! entities. In `.vrst` files they are still written as plain arrays (`"guides": [..]`,
//! `"guide_points": [..]`), so older files load unchanged and ids are re-issued on load.

use crate::Model;
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;

slotmap::new_key_type! {
    pub struct GuideId;
    pub struct GuidePointId;
}

/// An infinite construction line through `origin` along `dir`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GuideLine {
    pub origin: DVec3,
    pub dir: DVec3,
}

impl GuideLine {
    /// The guide after an affine transform (direction re-normalized).
    pub fn transformed(&self, m: &DMat4) -> GuideLine {
        let dir = m.transform_vector3(self.dir).normalize_or(self.dir);
        GuideLine { origin: m.transform_point3(self.origin), dir }
    }
}

/// Reference to a guide line or a guide point of a [`Model`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuideRef {
    Line(GuideId),
    Point(GuidePointId),
}

/// Guides stored in a group / component definition (local coordinates).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DefGuides {
    #[serde(with = "slotmap_vec", default)]
    pub lines: GuideLines,
    #[serde(with = "slotmap_vec", default)]
    pub points: GuidePoints,
}

impl DefGuides {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() && self.points.is_empty()
    }
}

/// A guide's geometry (world or local space).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GuideGeom {
    Line(GuideLine),
    Point(DVec3),
}

impl GuideGeom {
    pub fn transformed(&self, m: &DMat4) -> GuideGeom {
        match self {
            GuideGeom::Line(l) => GuideGeom::Line(l.transformed(m)),
            GuideGeom::Point(p) => GuideGeom::Point(m.transform_point3(*p)),
        }
    }
}

impl Model {
    /// Guide storage of the context `path` (empty = model level).
    pub fn guide_store(&self, path: &[crate::InstanceId]) -> Option<(&GuideLines, &GuidePoints)> {
        if path.is_empty() {
            return Some((&self.guides, &self.guide_points));
        }
        let d = self.defs.get(self.def_at(path)?)?;
        Some((&d.guides.lines, &d.guides.points))
    }

    pub fn guide_store_mut(&mut self, path: &[crate::InstanceId]) -> Option<(&mut GuideLines, &mut GuidePoints)> {
        if path.is_empty() {
            return Some((&mut self.guides, &mut self.guide_points));
        }
        let id = self.def_at(path)?;
        let d = self.defs.get_mut(id)?;
        Some((&mut d.guides.lines, &mut d.guides.points))
    }

    /// Guides of the context `path` with their world geometry.
    pub fn guides_in(&self, path: &[crate::InstanceId]) -> Vec<(GuideRef, GuideGeom)> {
        let Some((lines, points)) = self.guide_store(path) else { return Vec::new() };
        let xf = self.transform_at(path);
        let lines = lines.iter().map(|(id, l)| (GuideRef::Line(id), GuideGeom::Line(l.transformed(&xf))));
        let points = points.iter().map(|(id, p)| (GuideRef::Point(id), GuideGeom::Point(xf.transform_point3(*p))));
        lines.chain(points).collect()
    }

    /// World geometry of guide `g` of the context `path`.
    pub fn guide_geom_in(&self, path: &[crate::InstanceId], g: GuideRef) -> Option<GuideGeom> {
        let (lines, points) = self.guide_store(path)?;
        let local = match g {
            GuideRef::Line(id) => GuideGeom::Line(*lines.get(id)?),
            GuideRef::Point(id) => GuideGeom::Point(*points.get(id)?),
        };
        Some(local.transformed(&self.transform_at(path)))
    }

    /// Guides of `path` and of every enclosing context (what is drawn and inferred while
    /// `path` is open): (context depth, guide, world geometry). Depth `path.len()` = the
    /// active context (selectable); smaller depths are outer contexts.
    pub fn visible_guides(&self, path: &[crate::InstanceId]) -> Vec<(usize, GuideRef, GuideGeom)> {
        (0..=path.len())
            .flat_map(|k| self.guides_in(&path[..k]).into_iter().map(move |(g, geom)| (k, g, geom)))
            .collect()
    }

    /// Any guide anywhere in the model.
    pub fn has_any_guides(&self) -> bool {
        !self.guides.is_empty() || !self.guide_points.is_empty() || self.defs.values().any(|d| !d.guides.is_empty())
    }

    /// Add a guide given in world space to the context `path`.
    pub fn add_guide_in(&mut self, path: &[crate::InstanceId], world: GuideGeom) -> Option<GuideRef> {
        let inv = self.transform_at(path).inverse();
        let (lines, points) = self.guide_store_mut(path)?;
        Some(match world.transformed(&inv) {
            GuideGeom::Line(l) => GuideRef::Line(lines.insert(l)),
            GuideGeom::Point(p) => GuideRef::Point(points.insert(p)),
        })
    }

    pub fn has_guide_in(&self, path: &[crate::InstanceId], g: GuideRef) -> bool {
        self.guide_store(path).is_some_and(|(lines, points)| match g {
            GuideRef::Line(id) => lines.contains_key(id),
            GuideRef::Point(id) => points.contains_key(id),
        })
    }

    /// Remove guides of the context `path`. Returns how many were removed.
    pub fn remove_guides_in(&mut self, path: &[crate::InstanceId], gs: &[GuideRef]) -> usize {
        let Some((lines, points)) = self.guide_store_mut(path) else { return 0 };
        gs.iter()
            .filter(|g| match **g {
                GuideRef::Line(id) => lines.remove(id).is_some(),
                GuideRef::Point(id) => points.remove(id).is_some(),
            })
            .count()
    }

    /// Transform guides of the context `path` by the world transform `m`.
    pub fn transform_guides_in(&mut self, path: &[crate::InstanceId], gs: &[GuideRef], m: &DMat4) {
        let xf = self.transform_at(path);
        let local = xf.inverse() * *m * xf;
        if let Some((lines, points)) = self.guide_store_mut(path) {
            transform_in(lines, points, gs, &local);
        }
    }

    /// Insert copies of guides of the context `path` moved by the world transform `m`.
    pub fn copy_guides_in(&mut self, path: &[crate::InstanceId], gs: &[GuideRef], m: &DMat4) -> Vec<GuideRef> {
        let xf = self.transform_at(path);
        let local = xf.inverse() * *m * xf;
        let Some((lines, points)) = self.guide_store_mut(path) else { return Vec::new() };
        copy_in(lines, points, gs, &local)
    }

    /// All guides of the context `path`.
    pub fn all_guides_in(&self, path: &[crate::InstanceId]) -> Vec<GuideRef> {
        self.guide_store(path)
            .map(|(l, p)| l.keys().map(GuideRef::Line).chain(p.keys().map(GuideRef::Point)).collect())
            .unwrap_or_default()
    }

    /// Scale / transform every guide stored in definition `d` (local transform `m`), and its
    /// dimensions (free anchors and offsets; attached anchors follow their vertices).
    pub fn transform_def_guides(&mut self, d: crate::DefId, m: &DMat4) {
        self.transform_def_dims(d, m);
        let (lines, points) = if d == self.root {
            (&mut self.guides, &mut self.guide_points)
        } else if let Some(def) = self.defs.get_mut(d) {
            (&mut def.guides.lines, &mut def.guides.points)
        } else {
            return;
        };
        for l in lines.values_mut() {
            *l = l.transformed(m);
        }
        for p in points.values_mut() {
            *p = m.transform_point3(*p);
        }
    }

    /// Copy the guides of definition `d` into the context `path`, mapped by `t` (the
    /// instance transform; used when exploding).
    pub(crate) fn lift_guides(&mut self, path: &[crate::InstanceId], d: crate::DefId, t: &DMat4) {
        let Some(g) = self.defs.get(d).map(|x| x.guides.clone()).filter(|g| !g.is_empty()) else { return };
        if let Some((lines, points)) = self.guide_store_mut(path) {
            for l in g.lines.values() {
                lines.insert(l.transformed(t));
            }
            for p in g.points.values() {
                points.insert(t.transform_point3(*p));
            }
        }
    }

    /// Remove every guide line and guide point (model level and inside all definitions).
    pub fn delete_all_guides(&mut self) {
        self.guides.clear();
        self.guide_points.clear();
        for d in self.defs.values_mut() {
            d.guides = DefGuides::default();
        }
    }

}

fn transform_in(lines: &mut GuideLines, points: &mut GuidePoints, gs: &[GuideRef], m: &DMat4) {
    for g in gs {
        match *g {
            GuideRef::Line(id) => {
                if let Some(l) = lines.get_mut(id) {
                    *l = l.transformed(m);
                }
            }
            GuideRef::Point(id) => {
                if let Some(p) = points.get_mut(id) {
                    *p = m.transform_point3(*p);
                }
            }
        }
    }
}

fn copy_in(lines: &mut GuideLines, points: &mut GuidePoints, gs: &[GuideRef], m: &DMat4) -> Vec<GuideRef> {
    let mut out = Vec::with_capacity(gs.len());
    for g in gs {
        match *g {
            GuideRef::Line(id) => {
                if let Some(l) = lines.get(id).copied() {
                    out.push(GuideRef::Line(lines.insert(l.transformed(m))));
                }
            }
            GuideRef::Point(id) => {
                if let Some(p) = points.get(id).copied() {
                    out.push(GuideRef::Point(points.insert(m.transform_point3(p))));
                }
            }
        }
    }
    out
}

/// Serde adapter: a slotmap written as a plain array of values (ids are not persisted).
pub(crate) mod slotmap_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use slotmap::{Key, SlotMap};

    pub fn serialize<K: Key, V: Serialize, S: Serializer>(m: &SlotMap<K, V>, s: S) -> Result<S::Ok, S::Error> {
        s.collect_seq(m.values())
    }

    pub fn deserialize<'de, K: Key, V: Deserialize<'de>, D: Deserializer<'de>>(d: D) -> Result<SlotMap<K, V>, D::Error> {
        let v = Vec::<V>::deserialize(d)?;
        let mut m = SlotMap::with_capacity_and_key(v.len());
        for x in v {
            m.insert(x);
        }
        Ok(m)
    }
}

/// Construction-line storage type of [`Model::guides`].
pub type GuideLines = SlotMap<GuideId, GuideLine>;
/// Guide-point storage type of [`Model::guide_points`].
pub type GuidePoints = SlotMap<GuidePointId, DVec3>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guides_roundtrip_as_arrays_and_load_old_files() {
        let mut m = Model::new();
        let a = m.add_guide_in(&[], GuideGeom::Line(GuideLine { origin: DVec3::new(1.0, 2.0, 3.0), dir: DVec3::X })).unwrap();
        let p = m.add_guide_in(&[], GuideGeom::Point(DVec3::new(5.0, 6.0, 7.0))).unwrap();
        let json = crate::io::to_json(&m).unwrap();
        // Plain arrays on disk (backward compatible with pre-M4 files).
        assert!(json.contains(r#""guides":[{"origin":[1.0,2.0,3.0],"dir":[1.0,0.0,0.0]}]"#), "{json}");
        assert!(json.contains(r#""guide_points":[[5.0,6.0,7.0]]"#), "{json}");
        let back = crate::io::from_json(&json).unwrap();
        assert_eq!(back.guides.len(), 1);
        assert_eq!(back.guide_points.values().next().copied(), Some(DVec3::new(5.0, 6.0, 7.0)));
        // Moving / copying / removing by id.
        let t = DMat4::from_translation(DVec3::new(0.0, 0.0, 10.0));
        let copies = m.copy_guides_in(&[], &[a, p], &t);
        assert_eq!(copies.len(), 2);
        assert_eq!(m.guides.len(), 2);
        m.transform_guides_in(&[], &[p], &t);
        assert_eq!(m.guide_geom_in(&[], p), Some(GuideGeom::Point(DVec3::new(5.0, 6.0, 17.0))));
        assert_eq!(m.remove_guides_in(&[], &[a, a]), 1);
        assert!(!m.has_guide_in(&[], a));
        assert_eq!(m.all_guides_in(&[]).len(), 3);
        // A file without guide fields / without the visibility flag still loads (also checked
        // below for definitions without a `guides` field).
        let old = json.replace(r#","guides_hidden":false"#, "");
        assert!(crate::io::from_json(&old).is_ok());
    }

    #[test]
    fn guides_inside_a_group_follow_it() {
        use crate::mesh::Mesh;
        let mut m = Model::new();
        let mut mesh = Mesh::new();
        mesh.add_face(&[DVec3::ZERO, DVec3::X * 10.0, DVec3::new(10.0, 10.0, 0.0), DVec3::Y * 10.0]).unwrap();
        let ents: Vec<crate::EntityRef> = mesh.faces.keys().map(crate::EntityRef::Face).collect();
        *m.root_mesh_mut() = mesh;
        let g = m.make_group(&[], &ents).unwrap();
        m.root_mesh_mut().instances[g].transform = DMat4::from_translation(DVec3::X * 100.0);
        let path = [g];
        // Added in world space, stored locally in the group's definition.
        let p = m.add_guide_in(&path, GuideGeom::Point(DVec3::new(105.0, 0.0, 0.0))).unwrap();
        assert!(m.guide_points.is_empty());
        let d = m.def_at(&path).unwrap();
        assert_eq!(m.defs[d].guides.points.values().next().copied(), Some(DVec3::new(5.0, 0.0, 0.0)));
        assert_eq!(m.guide_geom_in(&path, p), Some(GuideGeom::Point(DVec3::new(105.0, 0.0, 0.0))));
        // Visible from inside (depth 1) but not a model-level guide.
        assert_eq!(m.visible_guides(&path).len(), 1);
        assert!(m.guides_in(&[]).is_empty());
        // Moving the group moves the guide.
        m.root_mesh_mut().instances[g].transform = DMat4::from_translation(DVec3::X * 200.0);
        assert_eq!(m.guide_geom_in(&path, p), Some(GuideGeom::Point(DVec3::new(205.0, 0.0, 0.0))));
        // World-space move inside the context.
        m.transform_guides_in(&path, &[p], &DMat4::from_translation(DVec3::Z * 3.0));
        assert_eq!(m.guide_geom_in(&path, p), Some(GuideGeom::Point(DVec3::new(205.0, 0.0, 3.0))));
        // Saved with the definition; files without definition guides still load.
        let json = crate::io::to_json(&m).unwrap();
        let back = crate::io::from_json(&json).unwrap();
        assert!(back.defs.values().any(|d| d.guides.points.len() == 1));
        assert!(!crate::io::to_json(&Model::new()).unwrap().contains("\"points\""));
        // Exploding lifts the guide to the parent context (world position kept).
        m.explode(&[], g).unwrap();
        assert_eq!(m.guide_points.values().next().copied(), Some(DVec3::new(205.0, 0.0, 3.0)));
        assert!(m.has_any_guides());
        m.delete_all_guides();
        assert!(!m.has_any_guides());
    }
}
