//! Fasteners (track W, batch 5): the bodies a joint puts between its parts (dowels, screws,
//! hinges, slides, handles…), drawn with the model.
//!
//! They are not parts: they come from the joints' sizes ([`crate::joinery::WoodJoint::bodies`])
//! whenever the joints are cut again or a file is read ([`Model::refresh_fasteners`]), are
//! left out of the cut list, picking and exports, and are not saved. Each one moves with the part
//! it goes into first, so it follows the exploded view; screws are pulled out along their axis
//! as well.

use crate::model::{ContextPath, Model};
use crate::tessellate::TessFace;
use glam::{DMat4, DVec3};

/// A simple solid of a fastener, in world space as assembled.
#[derive(Clone, Debug, PartialEq)]
pub enum Solid {
    /// A round body from `at` along `dir` (unit) for `len`: radius `r0` at `at`, `r1` at the
    /// far end (a cone or a countersunk head when they differ).
    Round { at: DVec3, dir: DVec3, len: f64, r0: f64, r1: f64 },
    /// A box along the world axes.
    Block { min: DVec3, max: DVec3 },
}

/// The part a fastener moves with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    A,
    B,
    /// The part placed in the earlier assembly step (`b` without steps): dowels go into it
    /// first and the other part comes onto them.
    Earlier,
}

/// One fastener body of joint `joint`.
#[derive(Clone, Debug, PartialEq)]
pub struct Fastener {
    pub joint: usize,
    pub anchor: Anchor,
    /// The joint's parts' inverse transforms as assembled when the body was placed, so that
    /// the body follows its part when that moves.
    pub local_a: DMat4,
    pub local_b: DMat4,
    /// The exploded view pulls the body out this way (world, unit), by `pull` × the distance.
    pub out: DVec3,
    pub pull: f64,
    pub color: [f32; 4],
    pub solids: Vec<Solid>,
    /// What it is, for the shopping list and the instructions ("Dowel Ø8 × 40").
    pub name: String,
}

pub const WOOD: [f32; 4] = [0.86, 0.71, 0.48, 1.0];
pub const STEEL: [f32; 4] = [0.6, 0.62, 0.66, 1.0];
pub const ZINC: [f32; 4] = [0.74, 0.75, 0.78, 1.0];
pub const BRASS: [f32; 4] = [0.82, 0.66, 0.3, 1.0];
pub const DARK: [f32; 4] = [0.2, 0.2, 0.22, 1.0];
pub const PLASTIC: [f32; 4] = [0.93, 0.93, 0.9, 1.0];

const SEGS: usize = 16;

impl Fastener {
    /// How far the exploded view pulls it out for `distance`: `pull` × the distance, but no
    /// further than just clear of its hole (its own length along `out` and 10 mm), so that it
    /// stays by the hole it goes into.
    pub fn pulled(&self, distance: f64) -> f64 {
        let along = |q: DVec3| q.dot(self.out);
        let (lo, hi) = self.solids.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), s| {
            let (a, b) = match *s {
                Solid::Round { at, dir, len, r0, r1 } => {
                    let r = r0.max(r1) * dir.cross(self.out).length();
                    let (a, b) = (along(at), along(at + dir * len));
                    (a.min(b) - r, a.max(b) + r)
                }
                Solid::Block { min, max } => {
                    let c = along((min + max) * 0.5);
                    let h = ((max - min) * 0.5).dot(self.out.abs());
                    (c - h, c + h)
                }
            };
            (lo.min(a), hi.max(b))
        });
        let clear = if hi >= lo { hi - lo + 10.0 } else { f64::INFINITY };
        (self.pull * distance).min(clear)
    }
}

/// Two unit vectors across `d`, with `u × v = d`.
fn across(d: DVec3) -> (DVec3, DVec3) {
    let u = d.any_orthonormal_vector();
    (u, d.cross(u))
}

impl Solid {
    /// Triangles (positions CCW about the outward normal, smoothed normals) with `xf` applied.
    pub fn faces(&self, xf: &DMat4) -> Vec<(Vec<DVec3>, Vec<DVec3>, DVec3)> {
        let p = |q: DVec3| xf.transform_point3(q);
        let n = |q: DVec3| xf.transform_vector3(q).normalize_or_zero();
        match *self {
            Solid::Round { at, dir, len, r0, r1 } => {
                let (u, v) = across(dir);
                let ring = |k: usize| {
                    let a = std::f64::consts::TAU * k as f64 / SEGS as f64;
                    u * a.cos() + v * a.sin()
                };
                let end = at + dir * len;
                let (mut side, mut normals) = (Vec::new(), Vec::new());
                let (mut cap0, mut cap1) = (Vec::new(), Vec::new());
                // the side's normal leans by the cone's slope
                let lean = (r0 - r1) / len.max(1e-9);
                for k in 0..SEGS {
                    let (ra, rb) = (ring(k), ring(k + 1));
                    let (a0, b0, a1, b1) = (at + ra * r0, at + rb * r0, end + ra * r1, end + rb * r1);
                    side.extend([a0, b0, b1, a0, b1, a1].map(p));
                    let (na, nb) = (n(ra + dir * lean), n(rb + dir * lean));
                    normals.extend([na, nb, nb, na, nb, na]);
                    if r0 > 1e-9 {
                        cap0.extend([at, b0, a0].map(p));
                    }
                    if r1 > 1e-9 {
                        cap1.extend([end, a1, b1].map(p));
                    }
                }
                let mut out = vec![(side, normals, n(dir.any_orthonormal_vector()))];
                for (tris, normal) in [(cap0, -dir), (cap1, dir)] {
                    if !tris.is_empty() {
                        out.push((tris, Vec::new(), n(normal)));
                    }
                }
                out
            }
            Solid::Block { min, max } => {
                let c = |i: usize, j: usize, k: usize| DVec3::new([min.x, max.x][i], [min.y, max.y][j], [min.z, max.z][k]);
                let quads = [
                    ([c(0, 0, 0), c(0, 1, 0), c(1, 1, 0), c(1, 0, 0)], DVec3::NEG_Z),
                    ([c(0, 0, 1), c(1, 0, 1), c(1, 1, 1), c(0, 1, 1)], DVec3::Z),
                    ([c(0, 0, 0), c(1, 0, 0), c(1, 0, 1), c(0, 0, 1)], DVec3::NEG_Y),
                    ([c(1, 1, 0), c(0, 1, 0), c(0, 1, 1), c(1, 1, 1)], DVec3::Y),
                    ([c(0, 1, 0), c(0, 0, 0), c(0, 0, 1), c(0, 1, 1)], DVec3::NEG_X),
                    ([c(1, 0, 0), c(1, 1, 0), c(1, 1, 1), c(1, 0, 1)], DVec3::X),
                ];
                quads.into_iter().map(|(q, normal)| ([q[0], q[1], q[2], q[0], q[2], q[3]].map(p).to_vec(), Vec::new(), n(normal))).collect()
            }
        }
    }

    /// The solid's corners or the points round its ends (world, as placed).
    pub fn points(&self) -> Vec<DVec3> {
        match *self {
            Solid::Round { at, dir, len, r0, r1 } => {
                let (u, v) = across(dir);
                let end = at + dir * len;
                [(at, r0), (end, r1)].into_iter().flat_map(|(c, r)| [c + u * r, c - u * r, c + v * r, c - v * r]).collect()
            }
            Solid::Block { min, max } => (0..8).map(|k| DVec3::new([min.x, max.x][k & 1], [min.y, max.y][(k >> 1) & 1], [min.z, max.z][(k >> 2) & 1])).collect(),
        }
    }
}

impl Model {
    /// Place the fasteners of every joint again (from the parts as they were before any joint
    /// was cut, as assembled).
    pub fn refresh_fasteners(&mut self) {
        let (fasteners, contacts) = {
            let m = self.unjoined();
            let mut fasteners = Vec::new();
            let mut contacts = Vec::new();
            for (i, j) in m.joints.iter().enumerate() {
                let c = crate::joinery::joint_contact(&m, j);
                if let Some(c) = &c {
                    let (la, lb) = (m.assembled_transform(&j.a).inverse(), if j.b.is_empty() { DMat4::IDENTITY } else { m.assembled_transform(&j.b).inverse() });
                    for b in j.bodies(c) {
                        fasteners.push(Fastener { joint: i, local_a: la, local_b: lb, ..b });
                    }
                }
                contacts.push(c);
            }
            (fasteners, contacts)
        };
        self.fasteners = fasteners;
        self.joint_contacts = contacts;
    }

    /// The part fastener `f` moves with.
    pub fn fastener_anchor(&self, f: &Fastener) -> Option<(&ContextPath, DMat4)> {
        let j = self.joints.get(f.joint)?;
        let use_a = match f.anchor {
            Anchor::A => true,
            Anchor::B => j.b.is_empty(),
            Anchor::Earlier => match (self.assembly.step_of(&j.a), self.assembly.step_of(&j.b)) {
                (Some(x), Some(y)) => x <= y,
                _ => j.b.is_empty(),
            },
        };
        Some(if use_a { (&j.a, f.local_a) } else { (&j.b, f.local_b) })
    }

    /// Where fastener `f` is drawn: its part's transform now (exploded too) and the pull out of
    /// the exploded view. `None` when hidden: fasteners off, its part hidden, or a step shown
    /// before the one it goes in at.
    pub fn fastener_transform(&self, f: &Fastener) -> Option<DMat4> {
        if self.fasteners_hidden {
            return None;
        }
        if let (Some(shown), Some(at)) = (self.assembly.step, self.fastener_step(f.joint))
            && at > shown
        {
            return None;
        }
        let (path, local) = self.fastener_anchor(f)?;
        if !self.path_shown(path) {
            return None;
        }
        let pull = DMat4::from_translation(f.out * f.pulled(self.assembly.explode));
        Some(self.transform_at(path) * local * pull)
    }

    /// The step joint `i`'s fasteners go in at: the fastener step given them, else the joint's.
    pub fn fastener_step(&self, i: usize) -> Option<usize> {
        self.assembly.joint_given(i).or_else(|| self.joint_step(i))
    }

    /// Every instance on `path` is shown.
    fn path_shown(&self, path: &[crate::InstanceId]) -> bool {
        let mut d = self.root;
        for &id in path {
            let Some(inst) = self.defs.get(d).and_then(|def| def.mesh.instances.get(id)) else { return false };
            if inst.hidden || !self.tag_visible(d, inst.tag) {
                return false;
            }
            d = inst.def;
        }
        true
    }

    /// The fasteners' triangles to draw with the model, in their colours.
    pub fn fastener_faces(&self) -> Vec<TessFace> {
        let mut out = Vec::new();
        for f in &self.fasteners {
            let Some(xf) = self.fastener_transform(f) else { continue };
            let Some((path, _)) = self.fastener_anchor(f) else { continue };
            for s in &f.solids {
                for (positions, normals, normal) in s.faces(&xf) {
                    out.push(TessFace { owner_path: path.clone(), face: None, positions, normal, normals, color: Some(f.color), ..Default::default() });
                }
            }
        }
        out
    }
}
