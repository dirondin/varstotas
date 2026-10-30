//! Carcass parts (track W): a back in grooves in the parts round it or nailed over their back
//! edges, and a plinth frame under a carcass.
//!
//! As for drawers ([`crate::drawer`]), the opening runs along the axis where the two parts are
//! apart, its front at the lower end of the horizontal axis they share and its back at the
//! upper end.

use crate::drawer::opening;
use crate::joinery::{WoodJoint, WoodJointKind, joint_contact};
use crate::model::{ContextPath, Model};
use crate::primitives::Primitive;
use crate::{InstanceId, OpError, OpResult};
use glam::{DMat4, DVec3};
use varstotas_i18n::tr;

/// A back panel's sizes (mm).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackSpec {
    /// In grooves in the parts round the opening, else nailed over their back edges.
    pub groove: bool,
    pub thickness: f64,
    /// A grooved back's face from the parts' back edges, and how deep it goes into them.
    pub inset: f64,
    pub depth: f64,
}

impl Default for BackSpec {
    fn default() -> Self {
        BackSpec { groove: true, thickness: 4.0, inset: 10.0, depth: 8.0 }
    }
}

/// A plinth frame's sizes (mm).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlinthSpec {
    pub height: f64,
    /// How far its front stands back from the carcass's front (a toe kick).
    pub setback: f64,
    pub thickness: f64,
}

/// The screws into a plinth's rails stay this far from their ends, clear of the confirmats.
const PLINTH_SCREW_MARGIN: f64 = 60.0;

impl Default for PlinthSpec {
    fn default() -> Self {
        PlinthSpec { height: 100.0, setback: 40.0, thickness: 18.0 }
    }
}

impl Model {
    /// A box part named `name` in `parent` from `lo` by `size` (world).
    fn add_board(&mut self, parent: &ContextPath, name: &str, lo: DVec3, size: DVec3) -> Option<ContextPath> {
        let mesh = Primitive::Box { w: size.x, d: size.y, h: size.z }.mesh()?;
        let xf = self.transform_at(parent).inverse() * DMat4::from_translation(lo);
        let id = self.add_group_mesh(parent, mesh, name, xf)?;
        self.mesh_at_mut(parent)?.instances.get_mut(id)?.name = name.into();
        let mut p = parent.clone();
        p.push(id);
        Some(p)
    }

    /// A back for the opening between parts `l` and `r` (instance paths in one parent).
    /// Grooved: between them `spec.inset` in from their back edges, from the part across the
    /// back at the bottom to the one at the top, `spec.depth` into each of them in a groove
    /// (housings). Else over their back edges and those of the parts at the bottom and the
    /// top, nailed to each. Returns the back.
    pub fn add_back(&mut self, l: &[InstanceId], r: &[InstanceId], spec: BackSpec) -> OpResult<InstanceId> {
        let bad = || OpError::Invalid(tr!("back-err-opening"));
        let (Some(lb), Some(rb)) = (self.part_box(l), self.part_box(r)) else { return Err(OpError::Invalid(tr!("joint-err-two-parts"))) };
        let o = opening(lb, rb).ok_or_else(bad)?;
        let (k, d) = (o.k, o.d);
        let t = spec.thickness;
        if t < 2.0 || (spec.groove && (spec.depth <= 0.0 || spec.inset < 0.0 || o.d1 - spec.inset - t <= o.d0)) {
            return Err(bad());
        }
        let parent: ContextPath = l.split_last().map(|(_, p)| p.to_vec()).unwrap_or_default();
        // the back's depth range, and the parts across it at the bottom and the top
        let (y0, y1) = if spec.groove { (o.d1 - spec.inset - t, o.d1 - spec.inset) } else { (lb.1[d].max(rb.1[d]), lb.1[d].max(rb.1[d]) + t) };
        let reach = if spec.groove { (y0, y1) } else { (y0 - 1.0, y0 - 1.0) };
        let mid = (o.z0 + o.z1) * 0.5;
        let across: Vec<(ContextPath, (DVec3, DVec3))> =
            self.parts_in_opening(l, r, &o).into_iter().filter(|(_, (m0, m1))| m0[d] <= reach.0 + 1e-6 && m1[d] >= reach.1 - 1e-6).collect();
        let bottom = across.iter().filter(|(_, (_, m1))| m1.z < mid).max_by(|x, y| x.1.1.z.total_cmp(&y.1.1.z)).cloned();
        let top = across.iter().filter(|(_, (m0, _))| m0.z > mid).min_by(|x, y| x.1.0.z.total_cmp(&y.1.0.z)).cloned();
        let (lo, hi, s0, s1) = if spec.groove {
            (bottom.as_ref().map_or(o.z0, |b| b.1.1.z), top.as_ref().map_or(o.z1, |b| b.1.0.z), o.a, o.b)
        } else {
            let z0 = bottom.as_ref().map_or(o.z0, |b| b.1.0.z.min(o.z0));
            let z1 = top.as_ref().map_or(o.z1, |b| b.1.1.z.max(o.z1));
            (z0, z1, lb.0[k].min(rb.0[k]), lb.1[k].max(rb.1[k]))
        };
        if hi - lo < 50.0 {
            return Err(bad());
        }
        let (mut p, mut s) = (DVec3::ZERO, DVec3::ZERO);
        (p[k], p[d], p.z) = (s0, y0, lo);
        (s[k], s[d], s.z) = (s1 - s0, t, hi - lo);
        let back = self.add_board(&parent, &tr!("back-name"), p, s).ok_or_else(bad)?;
        let id = *back.last().ok_or_else(bad)?;
        let round: Vec<ContextPath> = [Some(l.to_vec()), Some(r.to_vec()), bottom.map(|b| b.0), top.map(|b| b.0)].into_iter().flatten().collect();
        for part in round {
            if spec.groove {
                // the back's edge runs into a groove in the part
                let probe = WoodJoint { kind: WoodJointKind::Housing, a: part.clone(), b: back.clone(), ..Default::default() };
                let c = joint_contact(self, &probe).ok_or_else(bad)?;
                let mut j = WoodJoint::new(WoodJointKind::Housing, part, back.clone(), &c);
                j.depth = spec.depth;
                self.push_wood_joint(j)?;
            } else {
                self.add_wood_joint(WoodJointKind::Nails, back.clone(), part)?;
            }
        }
        Ok(id)
    }

    /// A plinth frame under `parts` (instance paths in one parent, a carcass's): as wide and
    /// deep as they are, `spec.height` tall under their underside, its front `spec.setback`
    /// back; front and back rails between two side rails on confirmats, the parts lying on it
    /// (thinner than 40 mm, a bottom) screwed down into the rails. Returns the rails: front,
    /// back, sides.
    pub fn add_plinth(&mut self, parts: &[ContextPath], spec: PlinthSpec) -> OpResult<Vec<InstanceId>> {
        let bad = || OpError::Invalid(tr!("plinth-err-size"));
        let boxes: Vec<(DVec3, DVec3)> = parts.iter().filter_map(|p| self.part_box(p)).collect();
        if boxes.is_empty() || boxes.len() != parts.len() {
            return Err(OpError::Invalid(tr!("joint-err-one-part")));
        }
        let lo = boxes.iter().fold(DVec3::splat(f64::MAX), |a, b| a.min(b.0));
        let hi = boxes.iter().fold(DVec3::splat(f64::MIN), |a, b| a.max(b.1));
        let (t, h) = (spec.thickness, spec.height);
        let y0 = lo.y + spec.setback;
        if h < 30.0 || t < 6.0 || spec.setback < 0.0 || hi.x - lo.x < 2.0 * t + 100.0 || hi.y - y0 < 2.0 * t + 100.0 {
            return Err(bad());
        }
        let parent: ContextPath = parts[0].split_last().map(|(_, p)| p.to_vec()).unwrap_or_default();
        let z = lo.z - h;
        let inner = hi.x - lo.x - 2.0 * t;
        let rails = [
            (tr!("plinth-front"), DVec3::new(lo.x + t, y0, z), DVec3::new(inner, t, h)),
            (tr!("plinth-back"), DVec3::new(lo.x + t, hi.y - t, z), DVec3::new(inner, t, h)),
            (tr!("plinth-side"), DVec3::new(lo.x, y0, z), DVec3::new(t, hi.y - y0, h)),
            (tr!("plinth-side"), DVec3::new(hi.x - t, y0, z), DVec3::new(t, hi.y - y0, h)),
        ];
        let mut paths = Vec::new();
        for (name, p, s) in rails {
            paths.push(self.add_board(&parent, &name, p, s).ok_or_else(bad)?);
        }
        for side in &paths[2..] {
            for rail in &paths[..2] {
                self.add_wood_joint(WoodJointKind::Confirmat, side.clone(), rail.clone())?;
            }
        }
        // a bottom lying on the rails screwed down into them
        for (part, b) in parts.iter().zip(&boxes) {
            if b.0.z > lo.z + 1.0 || b.1.z - b.0.z >= 40.0 {
                continue;
            }
            for rail in &paths {
                let probe = WoodJoint { kind: WoodJointKind::Screws, a: part.clone(), b: rail.clone(), ..Default::default() };
                if let Some(c) = joint_contact(self, &probe).filter(|c| c.axis == 2) {
                    // clear of the confirmats in the rails' ends
                    let mut j = WoodJoint::new(WoodJointKind::Screws, part.clone(), rail.clone(), &c);
                    j.margin = j.margin.max(PLINTH_SCREW_MARGIN);
                    self.push_wood_joint(j)?;
                }
            }
        }
        Ok(paths.iter().filter_map(|p| p.last().copied()).collect())
    }
}
