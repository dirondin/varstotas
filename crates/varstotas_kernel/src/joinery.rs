//! Joinery (track W, `docs/WORKSHOP-PLAN.md`): joints between two parts that touch, cut into
//! their geometry.
//!
//! A joint names two parts by their instance paths: `a`, the part joined into (a cabinet
//! side), and `b`, the part whose end or edge meets it (a shelf). Where they touch is found
//! from their world bounding boxes ([`contact`]): a face of one lying on a face of the other.
//! The parts are taken as boxes along the world axes there.
//!
//! - dowels: holes drilled across the contact into both parts;
//! - screws: a clearance hole through `a` and a pilot hole into `b`;
//! - housing (dado, or rabbet at `a`'s edge): `b` runs into a groove in `a`;
//! - mortise and tenon: a tenon on `b`'s end, inside shoulders, in a mortise of `a`, blind
//!   or through.
//!
//! Joints stay editable. The first joint on a part keeps the part's geometry from before any
//! joint ([`Model::joint_bases`]); every change restores those and cuts all the joints again
//! in order ([`Model::rebuild_wood_joints`]), and a part left in no joint gets its geometry
//! back. Hand edits of a joined part are lost on the next rebuild.

use crate::model::{ContextPath, Model};
use crate::ops::solid::mesh_boolean;
use crate::primitives::Primitive;
use crate::{DefId, Mesh, OpError, OpResult};
use glam::{DMat4, DQuat, DVec3};
use serde::{Deserialize, Serialize};
use varstotas_i18n::tr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WoodJointKind {
    #[default]
    Dowels,
    Screws,
    /// Dado / groove, or a rabbet at the edge of `a`.
    Housing,
    MortiseTenon,
}

impl WoodJointKind {
    pub const ALL: [WoodJointKind; 4] = [WoodJointKind::Dowels, WoodJointKind::Screws, WoodJointKind::Housing, WoodJointKind::MortiseTenon];
}

/// A joint between two parts (mm).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WoodJoint {
    pub kind: WoodJointKind,
    /// The part joined into.
    pub a: ContextPath,
    /// The part whose end or edge meets `a`.
    pub b: ContextPath,
    /// Dowels, screws: how many, spread along the joint.
    pub count: usize,
    /// Dowel or screw diameter.
    pub diameter: f64,
    /// Into `a`: dowel holes, the housing, the tenon (screws go right through `a`).
    pub depth: f64,
    /// Into `b`: dowel holes, screw pilot holes.
    pub depth_b: f64,
    /// Tenon thickness.
    pub thickness: f64,
    /// Tenon shoulders at its two ends.
    pub shoulder: f64,
    /// The tenon goes right through `a`.
    pub through: bool,
    /// Gap around tenons and in housings.
    pub clearance: f64,
}

/// Where two parts touch: a rectangle on a plane across a world axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contact {
    /// The plane's normal axis (0 = x).
    pub axis: usize,
    /// The plane's coordinate on that axis.
    pub at: f64,
    /// +1 when `a` lies on the positive side of the plane.
    pub into_a: f64,
    /// The touching rectangle (`lo[axis] == hi[axis] == at`).
    pub lo: DVec3,
    pub hi: DVec3,
    /// The rectangle's long side (along the joint) and short side (across it) axes.
    pub along: usize,
    pub across: usize,
    /// The parts' sizes along the axis.
    pub a_size: f64,
    pub b_size: f64,
}

impl Contact {
    pub fn length(&self) -> f64 {
        self.hi[self.along] - self.lo[self.along]
    }

    pub fn width(&self) -> f64 {
        self.hi[self.across] - self.lo[self.across]
    }
}

/// Faces closer than this touch.
const TOUCH: f64 = 0.5;
/// Cutters start this far outside the part they cut, so no face is shared.
const OVER: f64 = 0.5;

fn world_bounds(model: &Model, path: &[crate::InstanceId]) -> Option<(DVec3, DVec3)> {
    let (&id, parent) = path.split_last()?;
    model.instance_world_bounds(parent, id).map(|b| (b.min, b.max))
}

/// Where parts `a` and `b` touch: the largest rectangle where a face of one's bounding box lies
/// on a face of the other's.
pub fn contact(model: &Model, a: &[crate::InstanceId], b: &[crate::InstanceId]) -> Option<Contact> {
    let (a0, a1) = world_bounds(model, a)?;
    let (b0, b1) = world_bounds(model, b)?;
    let mut best: Option<(f64, Contact)> = None;
    for axis in 0..3 {
        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
        let (lo, hi) = (a0.max(b0), a1.min(b1));
        if hi[u] - lo[u] < TOUCH || hi[v] - lo[v] < TOUCH {
            continue;
        }
        for (at, into_a) in [(a0[axis], 1.0), (a1[axis], -1.0)] {
            let b_face = if into_a > 0.0 { b1[axis] } else { b0[axis] };
            if (b_face - at).abs() > TOUCH {
                continue;
            }
            let (mut lo, mut hi) = (lo, hi);
            (lo[axis], hi[axis]) = (at, at);
            let (along, across) = if hi[u] - lo[u] >= hi[v] - lo[v] { (u, v) } else { (v, u) };
            let c = Contact { axis, at, into_a, lo, hi, along, across, a_size: a1[axis] - a0[axis], b_size: b1[axis] - b0[axis] };
            let area = c.length() * c.width();
            if best.as_ref().is_none_or(|(x, _)| area > *x) {
                best = Some((area, c));
            }
        }
    }
    best.map(|b| b.1)
}

/// The face of `p`'s bounding box on the contact plane: (length, width) in the plane.
fn face_area(model: &Model, p: &[crate::InstanceId], axis: usize) -> f64 {
    world_bounds(model, p).map_or(0.0, |(lo, hi)| {
        let s = hi - lo;
        s[(axis + 1) % 3] * s[(axis + 2) % 3]
    })
}

/// A size in mm without trailing zeros.
fn num(v: f64) -> String {
    let s = format!("{v:.1}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn snap(v: f64) -> f64 {
    v.round().max(1.0)
}

impl WoodJoint {
    /// A joint of `kind` with sizes that suit the contact: dowels about half the thickness of
    /// `b`, a housing a third into `a`, a tenon a third of `b`'s thickness.
    pub fn new(kind: WoodJointKind, a: ContextPath, b: ContextPath, c: &Contact) -> Self {
        let (t, len) = (c.width(), c.length());
        let count = if len < 60.0 { 1 } else { ((len / 150.0).round() as usize + 1).clamp(2, 8) };
        let mut j = WoodJoint { kind, a, b, count, ..Default::default() };
        match kind {
            WoodJointKind::Dowels => {
                j.diameter = [5.0, 6.0, 8.0, 10.0, 12.0].into_iter().min_by(|x: &f64, y: &f64| (x - t / 2.0).abs().total_cmp(&(y - t / 2.0).abs())).unwrap_or(8.0);
                j.depth = snap((c.a_size * 2.0 / 3.0).min(15.0));
                j.depth_b = snap((c.b_size / 2.0).min(30.0));
            }
            WoodJointKind::Screws => {
                j.diameter = if t >= 15.0 { 4.0 } else { 3.0 };
                j.depth_b = snap((c.b_size / 2.0).min(30.0));
            }
            WoodJointKind::Housing => j.depth = snap(c.a_size / 3.0),
            WoodJointKind::MortiseTenon => {
                j.thickness = snap(t / 3.0);
                j.shoulder = snap((len * 0.1).max(5.0)).min(len / 3.0);
                j.depth = snap(c.a_size * 2.0 / 3.0);
            }
        }
        j
    }

    /// Hardware it takes: (name, count) for dowels and screws.
    pub fn hardware(&self, c: Option<&Contact>) -> Option<(String, usize)> {
        let d = num(self.diameter);
        match self.kind {
            WoodJointKind::Dowels => Some((tr!("joint-hw-dowel", diameter = d, length = num(self.depth + self.depth_b - 2.0)), self.count)),
            WoodJointKind::Screws => {
                let len = c.map_or(self.depth_b, |c| c.a_size + self.depth_b);
                Some((tr!("joint-hw-screw", diameter = d, length = num((len / 5.0).ceil() * 5.0)), self.count))
            }
            _ => None,
        }
    }

    /// Where dowels and screws go along the joint.
    fn spots(&self, c: &Contact) -> Vec<f64> {
        let (lo, hi) = (c.lo[c.along], c.hi[c.along]);
        if self.count <= 1 {
            return vec![(lo + hi) * 0.5];
        }
        let e = (c.length() / 4.0).min(32.0).max(self.diameter);
        (0..self.count).map(|i| lo + e + (hi - lo - 2.0 * e) * i as f64 / (self.count - 1) as f64).collect()
    }

    /// The solids taken from `a`, taken from `b` and added to `b`, placed in world space.
    #[allow(clippy::type_complexity)]
    fn cuts(&self, c: &Contact) -> OpResult<(Vec<(Mesh, DMat4)>, Vec<(Mesh, DMat4)>, Vec<(Mesh, DMat4)>)> {
        let n = {
            let mut v = DVec3::ZERO;
            v[c.axis] = c.into_a;
            v
        };
        let bad = || OpError::Invalid(tr!("joint-err-size"));
        // a cylinder from `from` along `n` (into a), `len` long
        let cyl = |from: DVec3, len: f64, r: f64| -> OpResult<(Mesh, DMat4)> {
            let m = Primitive::Cylinder { r, h: len, segs: 16 }.mesh().ok_or_else(bad)?;
            Ok((m, DMat4::from_translation(from) * DMat4::from_quat(DQuat::from_rotation_arc(DVec3::Z, n))))
        };
        // a box over `lo..hi` across the plane, from `from` to `to` along `n`
        let cube = |lo: DVec3, hi: DVec3, from: f64, to: f64| -> OpResult<(Mesh, DMat4)> {
            let (mut lo, mut hi) = (lo, hi);
            let (p, q) = (c.at + c.into_a * from, c.at + c.into_a * to);
            (lo[c.axis], hi[c.axis]) = (p.min(q), p.max(q));
            let s = hi - lo;
            let m = Primitive::Box { w: s.x, d: s.y, h: s.z }.mesh().ok_or_else(bad)?;
            Ok((m, DMat4::from_translation(lo)))
        };
        let mid = (c.lo[c.across] + c.hi[c.across]) * 0.5;
        let point = |s: f64| {
            let mut p = c.lo;
            p[c.along] = s;
            p[c.across] = mid;
            p
        };
        let grow = |lo: DVec3, hi: DVec3, axes: &[usize], by: f64| {
            let (mut lo, mut hi) = (lo, hi);
            for &k in axes {
                lo[k] -= by;
                hi[k] += by;
            }
            (lo, hi)
        };
        let (mut from_a, mut from_b, mut to_b) = (Vec::new(), Vec::new(), Vec::new());
        match self.kind {
            WoodJointKind::Dowels => {
                if self.diameter <= 0.0 || self.depth <= 0.0 || self.depth_b <= 0.0 || self.count == 0 || self.diameter >= c.width() {
                    return Err(bad());
                }
                for s in self.spots(c) {
                    let hole = cyl(point(s) - n * self.depth_b, self.depth + self.depth_b, self.diameter / 2.0)?;
                    from_a.push(hole.clone());
                    from_b.push(hole);
                }
            }
            WoodJointKind::Screws => {
                if self.diameter <= 0.0 || self.depth_b <= 0.0 || self.count == 0 || self.diameter >= c.width() {
                    return Err(bad());
                }
                for s in self.spots(c) {
                    from_a.push(cyl(point(s) - n * OVER, c.a_size + 2.0 * OVER, (self.diameter + 0.5) / 2.0)?);
                    from_b.push(cyl(point(s) - n * self.depth_b, self.depth_b + OVER, self.diameter * 0.35)?);
                }
            }
            WoodJointKind::Housing => {
                if self.depth <= 0.0 || self.depth >= c.a_size {
                    return Err(bad());
                }
                to_b.push(cube(c.lo, c.hi, -OVER, self.depth)?);
                let (lo, hi) = grow(c.lo, c.hi, &[c.across], self.clearance);
                from_a.push(cube(lo, hi, -OVER, self.depth)?);
            }
            WoodJointKind::MortiseTenon => {
                let len = if self.through { c.a_size } else { self.depth };
                if self.thickness <= 0.0 || self.thickness >= c.width() || len <= 0.0 || len > c.a_size + 1e-9 || 2.0 * self.shoulder >= c.length() {
                    return Err(bad());
                }
                let (mut lo, mut hi) = (c.lo, c.hi);
                (lo[c.across], hi[c.across]) = (mid - self.thickness / 2.0, mid + self.thickness / 2.0);
                lo[c.along] += self.shoulder;
                hi[c.along] -= self.shoulder;
                to_b.push(cube(lo, hi, -OVER, len)?);
                let (lo, hi) = grow(lo, hi, &[c.across, c.along], self.clearance);
                let end = if self.through { c.a_size + OVER } else { len + self.clearance };
                from_a.push(cube(lo, hi, -OVER, end)?);
            }
        }
        Ok((from_a, from_b, to_b))
    }
}

impl Model {
    /// The definition of the part at `path`.
    fn part_def(&self, path: &[crate::InstanceId]) -> Option<DefId> {
        let (&id, parent) = path.split_last()?;
        Some(self.mesh_at(parent)?.instances.get(id)?.def)
    }

    /// Join parts `p` and `q` (instance paths) with a joint of `kind` sized for them; the one
    /// whose touching face is smaller becomes `b`. Returns the joint's index.
    pub fn add_wood_joint(&mut self, kind: WoodJointKind, p: ContextPath, q: ContextPath) -> OpResult<usize> {
        if p == q || self.part_def(&p).is_none() || self.part_def(&q).is_none() {
            return Err(OpError::Invalid(tr!("joint-err-two-parts")));
        }
        self.restore_joint_bases();
        let c = contact(self, &p, &q).ok_or_else(|| OpError::Invalid(tr!("joint-err-no-contact")));
        let c = match c {
            Ok(c) => c,
            Err(e) => {
                self.rebuild_wood_joints();
                return Err(e);
            }
        };
        let (a, b) = if face_area(self, &q, c.axis) <= face_area(self, &p, c.axis) { (p, q) } else { (q, p) };
        let c = contact(self, &a, &b).unwrap_or(c);
        // a copy of a component would cut every copy: the joined ones get their own
        for path in [&a, &b] {
            if let Some((&id, parent)) = path.split_last() {
                self.make_unique(parent, id)?;
            }
        }
        self.joints.push(WoodJoint::new(kind, a, b, &c));
        let errors = self.rebuild_wood_joints();
        let i = self.joints.len() - 1;
        if let Some((_, e)) = errors.into_iter().find(|(k, _)| *k == i) {
            self.joints.pop();
            self.rebuild_wood_joints();
            return Err(OpError::Invalid(e));
        }
        Ok(i)
    }

    /// Change joint `i` and cut everything again. On failure the joint stays as it was.
    pub fn set_wood_joint(&mut self, i: usize, j: WoodJoint) -> OpResult<()> {
        let old = self.joints.get(i).cloned().ok_or(OpError::NotFound)?;
        self.joints[i] = j;
        if let Some((_, e)) = self.rebuild_wood_joints().into_iter().find(|(k, _)| *k == i) {
            self.joints[i] = old;
            self.rebuild_wood_joints();
            return Err(OpError::Invalid(e));
        }
        Ok(())
    }

    /// Make joint `i` a joint of `kind`, with the sizes [`WoodJoint::new`] picks for it.
    pub fn set_wood_joint_kind(&mut self, i: usize, kind: WoodJointKind) -> OpResult<()> {
        let old = self.joints.get(i).cloned().ok_or(OpError::NotFound)?;
        self.restore_joint_bases();
        let c = contact(self, &old.a, &old.b);
        self.rebuild_wood_joints();
        let c = c.ok_or_else(|| OpError::Invalid(tr!("joint-err-no-contact")))?;
        self.set_wood_joint(i, WoodJoint::new(kind, old.a, old.b, &c))
    }

    /// Delete joint `i`; parts left in no joint get their geometry back.
    pub fn remove_wood_joint(&mut self, i: usize) {
        if i < self.joints.len() {
            self.joints.remove(i);
            self.rebuild_wood_joints();
        }
    }

    fn restore_joint_bases(&mut self) {
        for (d, m) in &self.joint_bases {
            if let Some(def) = self.defs.get_mut(*d) {
                def.mesh = m.clone();
            }
        }
    }

    /// Restore the joined parts and cut every joint again in order. Returns the joints that
    /// could not be cut (index, why); they leave their parts as they were.
    pub fn rebuild_wood_joints(&mut self) -> Vec<(usize, String)> {
        self.restore_joint_bases();
        let mut errors = Vec::new();
        for i in 0..self.joints.len() {
            let j = self.joints[i].clone();
            if let Err(e) = self.cut_wood_joint(&j) {
                errors.push((i, e.to_string()));
            }
        }
        let used: Vec<DefId> = self.joints.iter().flat_map(|j| [self.part_def(&j.a), self.part_def(&j.b)]).flatten().collect();
        self.joint_bases.retain(|(d, _)| used.contains(d) && self.defs.contains_key(*d));
        errors
    }

    fn cut_wood_joint(&mut self, j: &WoodJoint) -> OpResult<()> {
        let missing = || OpError::Invalid(tr!("joint-err-missing"));
        let (da, db) = (self.part_def(&j.a).ok_or_else(missing)?, self.part_def(&j.b).ok_or_else(missing)?);
        let c = contact(self, &j.a, &j.b).ok_or_else(|| OpError::Invalid(tr!("joint-err-no-contact")))?;
        let (from_a, from_b, to_b) = j.cuts(&c)?;
        let mut out = Vec::new();
        for (path, d, ops) in [(&j.a, da, vec![(&from_a, false)]), (&j.b, db, vec![(&to_b, true), (&from_b, false)])] {
            let xf = self.transform_at(path);
            let mut mesh = self.defs[d].mesh.clone();
            if !mesh.is_solid() {
                return Err(OpError::Invalid(tr!("joint-err-not-solid", name = self.defs[d].name.clone())));
            }
            for (tools, add) in ops {
                for (t, txf) in tools {
                    mesh = mesh_boolean(&mesh, &xf, t, txf, add).ok_or_else(|| OpError::Invalid(tr!("joint-err-cut")))?;
                }
            }
            out.push((d, mesh));
        }
        for (d, mesh) in out {
            if !self.joint_bases.iter().any(|(x, _)| *x == d) {
                self.joint_bases.push((d, self.defs[d].mesh.clone()));
            }
            self.defs[d].mesh = mesh;
        }
        Ok(())
    }
}
