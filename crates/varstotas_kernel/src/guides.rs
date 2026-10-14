//! Guides (construction geometry): infinite guide lines and guide points, stored in world
//! coordinates at the model level. Owner: app agent (M4 "guides as first-class entities").
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

impl Model {
    pub fn add_guide_line(&mut self, g: GuideLine) -> GuideId {
        self.guides.insert(g)
    }

    pub fn add_guide_point(&mut self, p: DVec3) -> GuidePointId {
        self.guide_points.insert(p)
    }

    pub fn has_guide(&self, g: GuideRef) -> bool {
        match g {
            GuideRef::Line(id) => self.guides.contains_key(id),
            GuideRef::Point(id) => self.guide_points.contains_key(id),
        }
    }

    /// Remove the given guides (missing ids are ignored). Returns how many were removed.
    pub fn remove_guides(&mut self, gs: &[GuideRef]) -> usize {
        let mut n = 0;
        for g in gs {
            let removed = match *g {
                GuideRef::Line(id) => self.guides.remove(id).is_some(),
                GuideRef::Point(id) => self.guide_points.remove(id).is_some(),
            };
            n += removed as usize;
        }
        n
    }

    /// Remove every guide line and guide point.
    pub fn delete_all_guides(&mut self) {
        self.guides.clear();
        self.guide_points.clear();
    }

    /// Transform guides in place (world transform `m`).
    pub fn transform_guides(&mut self, gs: &[GuideRef], m: &DMat4) {
        for g in gs {
            match *g {
                GuideRef::Line(id) => {
                    if let Some(l) = self.guides.get_mut(id) {
                        *l = l.transformed(m);
                    }
                }
                GuideRef::Point(id) => {
                    if let Some(p) = self.guide_points.get_mut(id) {
                        *p = m.transform_point3(*p);
                    }
                }
            }
        }
    }

    /// Insert transformed copies of guides; returns the new guides (same order).
    pub fn copy_guides(&mut self, gs: &[GuideRef], m: &DMat4) -> Vec<GuideRef> {
        let mut out = Vec::with_capacity(gs.len());
        for g in gs {
            match *g {
                GuideRef::Line(id) => {
                    if let Some(l) = self.guides.get(id).copied() {
                        out.push(GuideRef::Line(self.guides.insert(l.transformed(m))));
                    }
                }
                GuideRef::Point(id) => {
                    if let Some(p) = self.guide_points.get(id).copied() {
                        out.push(GuideRef::Point(self.guide_points.insert(m.transform_point3(p))));
                    }
                }
            }
        }
        out
    }

    /// All guides of the model.
    pub fn all_guides(&self) -> Vec<GuideRef> {
        self.guides.keys().map(GuideRef::Line).chain(self.guide_points.keys().map(GuideRef::Point)).collect()
    }
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
        let a = m.add_guide_line(GuideLine { origin: DVec3::new(1.0, 2.0, 3.0), dir: DVec3::X });
        let p = m.add_guide_point(DVec3::new(5.0, 6.0, 7.0));
        let json = crate::io::to_json(&m).unwrap();
        // Plain arrays on disk (backward compatible with pre-M4 files).
        assert!(json.contains(r#""guides":[{"origin":[1.0,2.0,3.0],"dir":[1.0,0.0,0.0]}]"#), "{json}");
        assert!(json.contains(r#""guide_points":[[5.0,6.0,7.0]]"#), "{json}");
        let back = crate::io::from_json(&json).unwrap();
        assert_eq!(back.guides.len(), 1);
        assert_eq!(back.guide_points.values().next().copied(), Some(DVec3::new(5.0, 6.0, 7.0)));
        // Moving / copying / removing by id.
        let t = DMat4::from_translation(DVec3::new(0.0, 0.0, 10.0));
        let copies = m.copy_guides(&[GuideRef::Line(a), GuideRef::Point(p)], &t);
        assert_eq!(copies.len(), 2);
        assert_eq!(m.guides.len(), 2);
        m.transform_guides(&[GuideRef::Point(p)], &t);
        assert_eq!(m.guide_points[p], DVec3::new(5.0, 6.0, 17.0));
        assert_eq!(m.remove_guides(&[GuideRef::Line(a), GuideRef::Line(a)]), 1);
        assert!(!m.has_guide(GuideRef::Line(a)));
        assert_eq!(m.all_guides().len(), 3);
        // A file without guide fields / without the visibility flag still loads.
        let old = json.replace(r#","guides_hidden":false"#, "");
        assert!(crate::io::from_json(&old).is_ok());
    }
}
