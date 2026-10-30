//! Drawers (track W, batch 5) and doors: a drawer box built for the opening between two parts that face
//! each other (cabinet sides), on slides (ball-bearing, roller or undermount, see
//! [`SlideKind`]); a chest of them one above another in the free height of the opening.
//!
//! The opening runs along the axis where the two parts are apart; the drawer goes in from
//! their front (the lower end of the horizontal axis they share) and its depth is the longest
//! standard slide that fits ([`crate::joinery::SLIDE_LENGTHS`]) with 10 mm to spare behind.
//! The box leaves the slide's gap ([`SlideKind::gap`]) on each side. Its parts are groups next
//! to the left part, joined as a woodworker would: the box front and back on dowels between the
//! sides, the bottom nailed on underneath, a slide on each side (undermount slides on the
//! bottom's edges), and a front (inset in the opening or overlaying the carcass) screwed on
//! from inside with a handle, or none when the slides push to open. Doors, one or a pair, hang
//! on cup hinges on the parts round the opening ([`Model::add_doors`]).

use crate::joinery::{SLIDE_GAP, SLIDE_LENGTHS, SlideKind, WoodJoint, WoodJointKind, joint_contact};
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
    pub slide: SlideKind,
    /// Push-to-open slides: no handle on the front.
    pub push: bool,
}

impl Default for DrawerSpec {
    fn default() -> Self {
        DrawerSpec { z: 0.0, height: 150.0, wall: 15.0, bottom: 6.0, front: DrawerFront::Inset, front_thickness: 18.0, slide: SlideKind::BallBearing, push: false }
    }
}

/// Gap round an inset front (and between fronts), and how far an overlay front covers each
/// side.
const REVEAL: f64 = 2.0;
const OVERLAY: f64 = 16.0;
/// A box stands this far inside its front and the free height; undermount slides take
/// `UNDER` under the box.
const CLEAR: f64 = 10.0;
const UNDER: f64 = 14.0;
/// Room left behind a box (to a back, or the opening's end) for the slides' ends.
const BEHIND: f64 = 10.0;

/// The opening between two parts.
pub(crate) struct Opening {
    /// The axis they are apart on, and their inner faces on it.
    pub k: usize,
    pub a: f64,
    pub b: f64,
    /// The depth axis, and the range on it they share.
    pub d: usize,
    pub d0: f64,
    pub d1: f64,
    /// The height range they share.
    pub z0: f64,
    pub z1: f64,
}

/// The opening between parts `l` and `r` (assembled world bounds).
pub(crate) fn opening(l: (DVec3, DVec3), r: (DVec3, DVec3)) -> Option<Opening> {
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

    pub(crate) fn part_box(&self, path: &[InstanceId]) -> Option<(DVec3, DVec3)> {
        let (&id, parent) = path.split_last()?;
        let off = self.assembly_offset(path);
        self.instance_world_bounds(parent, id).map(|b| (b.min - off, b.max - off))
    }

    /// The free height of the opening between `l` and `r`: the tallest stretch of the height
    /// they share that no other part at its front crosses (a bottom, a top, a rail, another
    /// drawer).
    pub fn drawer_space(&self, l: &[InstanceId], r: &[InstanceId]) -> Option<(f64, f64)> {
        let o = opening(self.part_box(l)?, self.part_box(r)?)?;
        let mut cuts: Vec<(f64, f64)> = Vec::new();
        for (m0, m1) in self.in_opening(l, r, &o) {
            if m1[o.d] > o.d0 + 1.0 && m0[o.d] < o.d0 + 100.0 && m1.z > o.z0 && m0.z < o.z1 {
                cuts.push((m0.z, m1.z));
            }
        }
        cuts.sort_by(|x, y| x.0.total_cmp(&y.0));
        let (mut best, mut at) = (None::<(f64, f64)>, o.z0);
        for (c0, c1) in cuts.into_iter().chain([(o.z1, o.z1)]) {
            let c0 = c0.min(o.z1);
            if c0 > at && best.is_none_or(|(b0, b1)| c0 - at > b1 - b0) {
                best = Some((at, c0));
            }
            at = at.max(c1);
        }
        best
    }

    /// The other parts in `l`'s parent that reach into the opening between `l` and `r`.
    pub(crate) fn in_opening(&self, l: &[InstanceId], r: &[InstanceId], o: &Opening) -> Vec<(DVec3, DVec3)> {
        self.parts_in_opening(l, r, o).into_iter().map(|(_, b)| b).collect()
    }

    /// [`Model::in_opening`] with the parts' paths.
    pub(crate) fn parts_in_opening(&self, l: &[InstanceId], r: &[InstanceId], o: &Opening) -> Vec<(ContextPath, (DVec3, DVec3))> {
        let Some((_, parent)) = l.split_last() else { return Vec::new() };
        let Some(mesh) = self.mesh_at(parent) else { return Vec::new() };
        let ids: Vec<InstanceId> = mesh.instances.iter().map(|(id, _)| id).collect();
        ids.into_iter()
            .filter_map(|id| {
                let mut p = parent.to_vec();
                p.push(id);
                if p == l || p == r { None } else { self.part_box(&p).map(|b| (p, b)) }
            })
            .filter(|(_, (m0, m1))| m1[o.k] > o.a + 1.0 && m0[o.k] < o.b - 1.0)
            .collect()
    }

    /// `count` drawers one above another filling the height `space` (world, as from
    /// [`Model::drawer_space`]) of the opening between `l` and `r`: fronts of equal height
    /// [`REVEAL`] apart (inset: in the space; overlay: over the parts above and below it), each
    /// box `CLEAR` inside its front and the space. `spec.z` and `spec.height` are not used.
    /// Returns each drawer's parts, the lowest first.
    pub fn add_drawers(&mut self, l: &[InstanceId], r: &[InstanceId], count: usize, space: (f64, f64), spec: DrawerSpec) -> OpResult<Vec<Vec<InstanceId>>> {
        let (lo, hi) = space;
        let overlay = spec.front == DrawerFront::Overlay;
        let (s0, s1, gaps) = if overlay { (lo - OVERLAY, hi + OVERLAY, count as f64 - 1.0) } else { (lo, hi, count as f64 + 1.0) };
        let fh = (s1 - s0 - gaps * REVEAL) / count.max(1) as f64;
        let first = if overlay { s0 } else { s0 + REVEAL };
        let below = if spec.slide == SlideKind::Undermount { CLEAR + UNDER } else { CLEAR };
        let slots: Vec<(f64, f64, f64, f64)> = (0..count)
            .map(|i| {
                let f0 = first + i as f64 * (fh + REVEAL);
                let f1 = f0 + fh;
                let z = f0.max(lo) + below;
                (f0, f1, z, f1.min(hi) - CLEAR - z)
            })
            .collect();
        if count == 0 || slots.iter().any(|s| s.3 < spec.bottom + 30.0) {
            return Err(OpError::Invalid(tr!("drawer-err-opening")));
        }
        let mut out = Vec::new();
        for (f0, f1, z, height) in slots {
            out.push(self.build_drawer(l, r, DrawerSpec { z, height, ..spec }, (f0, f1))?);
        }
        Ok(out)
    }

    /// Build a drawer for the opening between parts `l` and `r` (instance paths in one
    /// parent): its parts and their joints, the front `CLEAR` beyond the box above and below.
    /// Returns the new parts.
    pub fn add_drawer(&mut self, l: &[InstanceId], r: &[InstanceId], spec: DrawerSpec) -> OpResult<Vec<InstanceId>> {
        self.build_drawer(l, r, spec, (spec.z - CLEAR, spec.z + spec.height + CLEAR))
    }

    /// A drawer with its front panel over the height range `fz` (world).
    fn build_drawer(&mut self, l: &[InstanceId], r: &[InstanceId], spec: DrawerSpec, fz: (f64, f64)) -> OpResult<Vec<InstanceId>> {
        let bad = || OpError::Invalid(tr!("drawer-err-opening"));
        let (Some(lb), Some(rb)) = (self.part_box(l), self.part_box(r)) else { return Err(OpError::Invalid(tr!("joint-err-two-parts"))) };
        let o = opening(lb, rb).ok_or_else(bad)?;
        // the depth ends at a part behind (a back between the sides)
        let behind = self.in_opening(l, r, &o).into_iter().filter(|(m0, m1)| m0[o.d] >= o.d0 + 100.0 && m1.z > spec.z && m0.z < spec.z + spec.height).map(|(m0, _)| m0[o.d]);
        let d1 = behind.fold(o.d1, f64::min);
        let Opening { k, a, b, d, d0, z0, z1, .. } = o;
        let parent: ContextPath = l.split_last().map(|(_, p)| p.to_vec()).unwrap_or_default();
        let (w, t) = (spec.wall, spec.bottom);
        let ft = if spec.front == DrawerFront::None { 0.0 } else { spec.front_thickness };
        let inset = if spec.front == DrawerFront::Inset { ft } else { 0.0 };
        let len = SLIDE_LENGTHS.iter().rev().copied().find(|s| *s <= d1 - d0 - inset - BEHIND).ok_or_else(|| OpError::Invalid(tr!("joint-err-slide-length")))?;
        let (x0, x1) = (a + spec.slide.gap(), b - spec.slide.gap());
        let under = spec.slide == SlideKind::Undermount;
        let room = if under { UNDER } else { 0.0 };
        if x1 - x0 < 2.0 * w + 50.0 || spec.height < t + 30.0 || w < 8.0 || t < 3.0 || spec.z - room < z0 - 1e-6 || spec.z + spec.height > z1 + 1e-6 {
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
            DrawerFront::Inset => Some(place(self, &tr!("drawer-front"), [a + REVEAL, d0, fz.0], [b - a - 2.0 * REVEAL, ft, fz.1 - fz.0])?),
            DrawerFront::Overlay => Some(place(self, &tr!("drawer-front"), [a - OVERLAY, d0 - ft, fz.0], [b - a + 2.0 * OVERLAY, ft, fz.1 - fz.0])?),
        };
        parts.extend(panel);
        let path = |id: InstanceId| {
            let mut p = parent.clone();
            p.push(id);
            p
        };
        let (pl, pr) = if lb.0[k] <= rb.0[k] { (l.to_vec(), r.to_vec()) } else { (r.to_vec(), l.to_vec()) };
        // the slides on the box's sides, undermount ones on the bottom's edges
        let (sl, sr) = if under { (bottom, bottom) } else { (left, right) };
        for (p, q) in [(pl, path(sl)), (pr, path(sr))] {
            let c = joint_contact(self, &WoodJoint { kind: WoodJointKind::Slide, a: p.clone(), b: q.clone(), ..Default::default() }).ok_or_else(bad)?;
            let mut j = WoodJoint::new(WoodJointKind::Slide, p, q, &c);
            (j.slide, j.push) = (spec.slide, spec.push);
            self.push_wood_joint(j)?;
        }
        let mut joints = Vec::new();
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
            if !spec.push {
                let _ = self.add_wood_joint(WoodJointKind::Handle, q, Vec::new());
            }
        }
        Ok(parts)
    }
}

/// Doors for an opening (track W): one or two, inset in it or overlaying the parts round it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DoorSpec {
    /// One door or a pair meeting in the middle.
    pub count: usize,
    /// In the opening (between the parts, [`REVEAL`] clear) or over their front edges.
    pub inset: bool,
    pub thickness: f64,
    /// Push latches: no handles.
    pub push: bool,
}

impl Default for DoorSpec {
    fn default() -> Self {
        DoorSpec { count: 1, inset: false, thickness: 18.0, push: false }
    }
}

impl Model {
    /// The height of the opening between `l` and `r` for doors: from the part across its front
    /// at the bottom (a bottom) to the one at the top (a top), else the height the two share;
    /// the parts between (shelves) are behind the doors.
    pub fn door_space(&self, l: &[InstanceId], r: &[InstanceId]) -> Option<(f64, f64)> {
        let o = opening(self.part_box(l)?, self.part_box(r)?)?;
        let (mut lo, mut hi) = (o.z0, o.z1);
        for (m0, m1) in self.in_opening(l, r, &o) {
            if m1[o.d] > o.d0 + 1.0 && m0[o.d] < o.d0 + 100.0 {
                if m0.z <= o.z0 + 1.0 && m1.z > lo {
                    lo = m1.z;
                }
                if m1.z >= o.z1 - 1.0 && m0.z < hi {
                    hi = m0.z;
                }
            }
        }
        (hi - lo > 100.0).then_some((lo, hi))
    }

    /// Doors over the height `space` (world, as from [`Model::door_space`]) of the opening
    /// between `l` and `r`: inset in it [`REVEAL`] clear of its edges, or over the parts round
    /// it by [`OVERLAY`]; a pair [`REVEAL`] apart in the middle. Each hangs on cup hinges on its
    /// side (one door on `l`), as many as its height wants, with a handle upright by its free
    /// edge unless `spec.push`. Returns the doors, `l`'s first.
    pub fn add_doors(&mut self, l: &[InstanceId], r: &[InstanceId], space: (f64, f64), spec: DoorSpec) -> OpResult<Vec<InstanceId>> {
        let bad = || OpError::Invalid(tr!("door-err-opening"));
        let (Some(lb), Some(rb)) = (self.part_box(l), self.part_box(r)) else { return Err(OpError::Invalid(tr!("joint-err-two-parts"))) };
        let o = opening(lb, rb).ok_or_else(bad)?;
        let Opening { k, a, b, d, d0, .. } = o;
        let (lo, hi) = space;
        let count = spec.count;
        let ft = spec.thickness;
        if !(1..=2).contains(&count) || hi - lo < 100.0 || ft < 6.0 {
            return Err(bad());
        }
        // across the opening and up; the door's depth range
        let (s0, s1, z0, z1, y0) = if spec.inset { (a + REVEAL, b - REVEAL, lo + REVEAL, hi - REVEAL, d0) } else { (a - OVERLAY, b + OVERLAY, lo - OVERLAY, hi + OVERLAY, d0 - ft) };
        let w = (s1 - s0 - (count as f64 - 1.0) * REVEAL) / count as f64;
        if w < 100.0 {
            return Err(bad());
        }
        // `l` on the low side of the opening hangs the first door
        let l_low = lb.0[k] <= rb.0[k];
        let sides: Vec<&[InstanceId]> = if count == 1 { vec![l] } else if l_low { vec![l, r] } else { vec![r, l] };
        let parent: ContextPath = l.split_last().map(|(_, p)| p.to_vec()).unwrap_or_default();
        let pxf = self.transform_at(&parent).inverse();
        let mut doors = Vec::new();
        for (i, side) in sides.into_iter().enumerate() {
            let x = s0 + i as f64 * (w + REVEAL);
            let (mut p, mut s) = (DVec3::ZERO, DVec3::ZERO);
            (p[k], p[d], p.z) = (x, y0, z0);
            (s[k], s[d], s.z) = (w, ft, z1 - z0);
            let mesh = Primitive::Box { w: s.x, d: s.y, h: s.z }.mesh().ok_or_else(bad)?;
            let name = tr!("door-name");
            let id = self.add_group_mesh(&parent, mesh, &name, pxf * DMat4::from_translation(p)).ok_or_else(bad)?;
            self.mesh_at_mut(&parent).and_then(|m| m.instances.get_mut(id)).ok_or_else(bad)?.name = name;
            let mut door = parent.clone();
            door.push(id);
            if spec.inset {
                // the door is b, between the sides
                let probe = WoodJoint { kind: WoodJointKind::Hinge, a: side.to_vec(), b: door.clone(), inset: true, ..Default::default() };
                let c = joint_contact(self, &probe).ok_or_else(bad)?;
                let mut j = WoodJoint::new(WoodJointKind::Hinge, side.to_vec(), door.clone(), &c);
                j.inset = true;
                self.push_wood_joint(j)?;
            } else {
                self.add_wood_joint(WoodJointKind::Hinge, door.clone(), side.to_vec())?;
            }
            if !spec.push && let Ok(h) = self.add_wood_joint(WoodJointKind::Handle, door.clone(), Vec::new()) {
                // upright by the free edge, away from the hinged side
                let hinged_low = self.part_box(side).is_some_and(|b| b.0[k] < x);
                let mut j = self.joints[h].clone();
                if s.z > w {
                    j.offset = (w / 2.0 - 50.0) * if hinged_low { 1.0 } else { -1.0 };
                    let _ = self.set_wood_joint(h, j);
                }
            }
            doors.push(id);
        }
        Ok(doors)
    }
}
