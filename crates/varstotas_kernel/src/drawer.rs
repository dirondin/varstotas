//! Drawers (track W, batch 5): a drawer box built for the opening between two parts that face
//! each other (cabinet sides), on ball-bearing slides.
//!
//! The opening runs along the axis where the two parts are apart; the drawer goes in from
//! their front (the lower end of the horizontal axis they share) and its depth is the longest
//! standard slide that fits ([`crate::joinery::SLIDE_LENGTHS`]). The box leaves
//! [`crate::joinery::SLIDE_GAP`] on each side for the slides. Its parts are groups next to
//! the left part, joined as a woodworker would: the box front and back on dowels between the
//! sides, the bottom nailed on underneath, a slide on each side, and a front (inset in the
//! opening or overlaying the carcass) screwed on from inside with a handle.

use crate::joinery::{SLIDE_GAP, SLIDE_LENGTHS, WoodJoint, WoodJointKind, joint_contact};
use crate::model::{ContextPath, Model};
use crate::primitives::Primitive;
use crate::{InstanceId, OpError, OpResult};
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};
use varstotas_i18n::tr;

/// The drawer's front panel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawerFront {
    /// The box's own front only.
    None,
    /// In the opening, 2 mm clear of the sides.
    #[default]
    Inset,
    /// Over the carcass's front edges.
    Overlay,
}

/// The drawer's sizes (mm).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawerSpec {
    /// The underside of the box (world height).
    pub z: f64,
    /// The box's height, the bottom included.
    pub height: f64,
    /// Sides, front and back.
    pub wall: f64,
    pub bottom: f64,
    pub front: DrawerFront,
    pub front_thickness: f64,
}

impl Default for DrawerSpec {
    fn default() -> Self {
        DrawerSpec { z: 0.0, height: 150.0, wall: 15.0, bottom: 6.0, front: DrawerFront::Inset, front_thickness: 18.0 }
    }
}

/// Gap round an inset front, and how far an overlay front covers each side.
const REVEAL: f64 = 2.0;
const OVERLAY: f64 = 16.0;

/// The opening between two parts.
struct Opening {
    /// The axis they are apart on, and their inner faces on it.
    k: usize,
    a: f64,
    b: f64,
    /// The depth axis, and the range on it they share.
    d: usize,
    d0: f64,
    d1: f64,
    /// The height range they share.
    z0: f64,
    z1: f64,
}

/// The opening between parts `l` and `r` (assembled world bounds).
fn opening(l: (DVec3, DVec3), r: (DVec3, DVec3)) -> Option<Opening> {
    let ((l0, l1), (r0, r1)) = (l, r);
    for k in [0, 1] {
        let (a, b) = if l1[k] <= r0[k] { (l1[k], r0[k]) } else if r1[k] <= l0[k] { (r1[k], l0[k]) } else { continue };
        let d = 1 - k;
        let (d0, d1) = (l0[d].max(r0[d]), l1[d].min(r1[d]));
        let (z0, z1) = (l0.z.max(r0.z), l1.z.min(r1.z));
        if b - a > 2.0 * SLIDE_GAP && d1 > d0 && z1 > z0 {
            return Some(Opening { k, a, b, d, d0, d1, z0, z1 });
        }
    }
    None
}

impl Model {
    /// The default height for a drawer between `l` and `r`: the bottom of the opening they
    /// share, a little up.
    pub fn drawer_z(&self, l: &[InstanceId], r: &[InstanceId]) -> Option<f64> {
        let o = opening(self.part_box(l)?, self.part_box(r)?)?;
        Some(o.z0 + 30.0)
    }

    fn part_box(&self, path: &[InstanceId]) -> Option<(DVec3, DVec3)> {
        let (&id, parent) = path.split_last()?;
        let off = self.assembly_offset(path);
        self.instance_world_bounds(parent, id).map(|b| (b.min - off, b.max - off))
    }

    /// Build a drawer for the opening between parts `l` and `r` (instance paths in one
    /// parent): its parts and their joints. Returns the new parts.
    pub fn add_drawer(&mut self, l: &[InstanceId], r: &[InstanceId], spec: DrawerSpec) -> OpResult<Vec<InstanceId>> {
        let bad = || OpError::Invalid(tr!("drawer-err-opening"));
        let (Some(lb), Some(rb)) = (self.part_box(l), self.part_box(r)) else { return Err(OpError::Invalid(tr!("joint-err-two-parts"))) };
        let Opening { k, a, b, d, d0, d1, z0, z1 } = opening(lb, rb).ok_or_else(bad)?;
        let parent: ContextPath = l.split_last().map(|(_, p)| p.to_vec()).unwrap_or_default();
        let (w, t) = (spec.wall, spec.bottom);
        let ft = if spec.front == DrawerFront::None { 0.0 } else { spec.front_thickness };
        let inset = if spec.front == DrawerFront::Inset { ft } else { 0.0 };
        let len = SLIDE_LENGTHS.iter().rev().copied().find(|s| *s <= d1 - d0 - inset).ok_or_else(|| OpError::Invalid(tr!("joint-err-slide-length")))?;
        let (x0, x1) = (a + SLIDE_GAP, b - SLIDE_GAP);
        if x1 - x0 < 2.0 * w + 50.0 || spec.height < t + 30.0 || w < 8.0 || t < 3.0 || spec.z < z0 - 1e-6 || spec.z + spec.height > z1 + 1e-6 {
            return Err(bad());
        }
        let (y0, z) = (d0 + inset, spec.z);
        let pxf = self.transform_at(&parent).inverse();
        // a box part from `lo` by `size`, in (opening, depth, up) axes
        let place = |m: &mut Model, name: &str, lo: [f64; 3], size: [f64; 3]| -> OpResult<InstanceId> {
            let (mut p, mut s) = (DVec3::ZERO, DVec3::ZERO);
            (p[k], p[d], p.z) = (lo[0], lo[1], lo[2]);
            (s[k], s[d], s.z) = (size[0], size[1], size[2]);
            let mesh = Primitive::Box { w: s.x, d: s.y, h: s.z }.mesh().ok_or_else(bad)?;
            let id = m.add_group_mesh(&parent, mesh, name, pxf * DMat4::from_translation(p)).ok_or_else(bad)?;
            m.mesh_at_mut(&parent).and_then(|x| x.instances.get_mut(id)).ok_or_else(bad)?.name = name.into();
            Ok(id)
        };
        let h = spec.height - t;
        let side = tr!("drawer-side");
        let left = place(self, &side, [x0, y0, z + t], [w, len, h])?;
        let right = place(self, &side, [x1 - w, y0, z + t], [w, len, h])?;
        let front = place(self, &tr!("drawer-box-front"), [x0 + w, y0, z + t], [x1 - x0 - 2.0 * w, w, h])?;
        let back = place(self, &tr!("drawer-back"), [x0 + w, y0 + len - w, z + t], [x1 - x0 - 2.0 * w, w, h])?;
        let bottom = place(self, &tr!("drawer-bottom"), [x0, y0, z], [x1 - x0, len, t])?;
        let mut parts = vec![left, right, front, back, bottom];
        let panel = match spec.front {
            DrawerFront::None => None,
            DrawerFront::Inset => Some(place(self, &tr!("drawer-front"), [a + REVEAL, d0, z - 10.0], [b - a - 2.0 * REVEAL, ft, spec.height + 20.0])?),
            DrawerFront::Overlay => Some(place(self, &tr!("drawer-front"), [a - OVERLAY, d0 - ft, z - 10.0], [b - a + 2.0 * OVERLAY, ft, spec.height + 20.0])?),
        };
        parts.extend(panel);
        let path = |id: InstanceId| {
            let mut p = parent.clone();
            p.push(id);
            p
        };
        let (pl, pr) = if lb.0[k] <= rb.0[k] { (l.to_vec(), r.to_vec()) } else { (r.to_vec(), l.to_vec()) };
        let mut joints = vec![(WoodJointKind::Slide, pl, path(left)), (WoodJointKind::Slide, pr, path(right))];
        for s in [left, right] {
            joints.push((WoodJointKind::Dowels, path(s), path(front)));
            joints.push((WoodJointKind::Dowels, path(s), path(back)));
            joints.push((WoodJointKind::Nails, path(bottom), path(s)));
        }
        for (kind, p, q) in joints {
            self.add_wood_joint(kind, p, q)?;
        }
        if let Some(panel) = panel {
            // screwed on from inside the box, the handle in the middle
            let (p, q) = (path(front), path(panel));
            let c = joint_contact(self, &WoodJoint { kind: WoodJointKind::Screws, a: p.clone(), b: q.clone(), ..Default::default() }).ok_or_else(bad)?;
            self.push_wood_joint(WoodJoint::new(WoodJointKind::Screws, p, q.clone(), &c))?;
            let _ = self.add_wood_joint(WoodJointKind::Handle, q, Vec::new());
        }
        Ok(parts)
    }
}
