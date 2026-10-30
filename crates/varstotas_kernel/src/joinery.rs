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
//!   or through;
//! - box joint: fingers on `b`'s end right through `a`, which keeps the pins between them;
//! - dovetail (through): tails on `b`'s end widening into `a`, pins left in `a`;
//! - pocket hole: a screw from a pocket in `b`'s face, at 15° through `b`'s end into `a`.
//!
//! Furniture fittings (batch 5, D) are joints too: they drill their own holes, price themselves
//! in the cut list and show their bodies.
//!
//! - confirmat: a stepped hole through `a` into `b`'s end, a 7 mm screw;
//! - cam lock (minifix): a bolt screwed into `a`'s face, through a hole in `b`'s end to a
//!   15 mm cam in `b`'s face;
//! - shelf pins: 5 mm holes in `a` just under `b`, optionally rows 32 mm apart above and below;
//! - cup hinge: a 35 mm cup in the door, a plate on the side; overlay (the door is `a`, the
//!   side's front edge on its back; half overlay when it covers only part of that edge, two
//!   doors on a middle partition) or inset (the door is `b`, between the sides); the angle it
//!   opens to (110° unless set);
//! - gas strut: holds a lift-up flap (an overlay door `a`, hinged along its top) open, from a
//!   bracket low on the inside of the side `b` to one high on the door's back; its force in
//!   newtons, from the door's weight unless set;
//! - ball-bearing slide: between a cabinet side `a` and a drawer side `b` 12.7 mm apart, the
//!   longest standard length that fits;
//! - handle, knob, legs, casters: on one part (`b` empty), on its front face (min y; `flip`: the back)
//!   or under it;
//! - table-top fastener (figure 8): recessed into an apron's edge `b`, screwed to the top `a`;
//! - nails: through a back panel `a` into the carcass edge `b`.
//!
//! Box joints and dovetails are corner joints: `a`'s end lies flush with `b`'s outer face, so
//! the sockets cut in `a` are open at its end.
//!
//! Joints stay editable. The first joint on a part keeps the part's geometry from before any
//! joint ([`Model::joint_bases`]); every change restores those and cuts all the joints again
//! in order ([`Model::rebuild_wood_joints`]), and a part left in no joint gets its geometry
//! back. Hand edits of a joined part are lost on the next rebuild.

use crate::fasteners::{Anchor, BRASS, DARK, Fastener, STEEL, Solid, WOOD, ZINC};
use crate::model::{ContextPath, Model};
use crate::ops::solid::{mesh_boolean, merge_tools};
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
    BoxJoint,
    Dovetail,
    PocketHole,
    Confirmat,
    CamLock,
    ShelfPins,
    Hinge,
    /// A gas strut holding a lift-up flap open.
    GasStrut,
    Slide,
    Handle,
    Knob,
    Leg,
    /// Swivel casters, four under the part like legs.
    Caster,
    TableTop,
    Nails,
}

impl WoodJointKind {
    pub const ALL: [WoodJointKind; 19] = [
        WoodJointKind::Dowels,
        WoodJointKind::Screws,
        WoodJointKind::PocketHole,
        WoodJointKind::Housing,
        WoodJointKind::MortiseTenon,
        WoodJointKind::BoxJoint,
        WoodJointKind::Dovetail,
        WoodJointKind::Confirmat,
        WoodJointKind::CamLock,
        WoodJointKind::ShelfPins,
        WoodJointKind::Hinge,
        WoodJointKind::GasStrut,
        WoodJointKind::Slide,
        WoodJointKind::TableTop,
        WoodJointKind::Nails,
        WoodJointKind::Handle,
        WoodJointKind::Knob,
        WoodJointKind::Leg,
        WoodJointKind::Caster,
    ];

    /// On one part (`b` empty): handles, knobs, legs, casters.
    pub fn single(self) -> bool {
        matches!(self, WoodJointKind::Handle | WoodJointKind::Knob | WoodJointKind::Leg | WoodJointKind::Caster)
    }

    /// Its fasteners go into the earlier part first, in a step of their own, and the later
    /// part comes onto them: dowels, shelf pins, cam lock bolts.
    pub fn goes_in_first(self) -> bool {
        matches!(self, WoodJointKind::Dowels | WoodJointKind::ShelfPins | WoodJointKind::CamLock)
    }

    /// Four under the part (legs, casters).
    pub fn under(self) -> bool {
        matches!(self, WoodJointKind::Leg | WoodJointKind::Caster)
    }

    /// Furniture hardware rather than a wood joint.
    pub fn fitting(self) -> bool {
        WoodJointKind::ALL.iter().position(|k| *k == self).is_some_and(|i| i >= 7)
    }

    /// Its fasteners are laid out along the joint (count, margin, pitch, distances).
    pub fn spaced(self) -> bool {
        !matches!(self, WoodJointKind::Housing | WoodJointKind::MortiseTenon | WoodJointKind::BoxJoint | WoodJointKind::Dovetail | WoodJointKind::Slide | WoodJointKind::GasStrut | WoodJointKind::Leg | WoodJointKind::Caster)
    }
}

/// What a drawer slide is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlideKind {
    /// Full-extension ball-bearing slides on the drawer's sides, 12.7 mm each side.
    #[default]
    BallBearing,
    /// Roller slides along the bottom of the drawer's sides, 12.5 mm each side.
    Roller,
    /// Concealed slides under the drawer's bottom (the joint is on the bottom's edge), 5 mm
    /// each side.
    Undermount,
}

impl SlideKind {
    pub const ALL: [SlideKind; 3] = [SlideKind::BallBearing, SlideKind::Roller, SlideKind::Undermount];

    /// The gap it takes between the carcass and the drawer.
    pub fn gap(self) -> f64 {
        match self {
            SlideKind::BallBearing => SLIDE_GAP,
            SlideKind::Roller => 12.5,
            SlideKind::Undermount => 5.0,
        }
    }

    fn is_default(&self) -> bool {
        *self == SlideKind::BallBearing
    }
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
    /// Dowels, screws, pocket holes: how many, spread along the joint; box joints: fingers on
    /// `b`; dovetails: tails.
    pub count: usize,
    /// Dowel or screw diameter.
    pub diameter: f64,
    /// Into `a`: dowel holes, the housing, the tenon, the pocket screw (screws go right through
    /// `a`).
    pub depth: f64,
    /// Into `b`: dowel holes, screw pilot holes.
    pub depth_b: f64,
    /// Tenon thickness.
    pub thickness: f64,
    /// Tenon shoulders at its two ends.
    pub shoulder: f64,
    /// The tenon goes right through `a`.
    pub through: bool,
    /// Gap around tenons, fingers and tails, and in housings.
    pub clearance: f64,
    /// Dovetail slope, degrees from the joint's direction; the angle a cup hinge opens to
    /// (0: 110°).
    pub angle: f64,
    /// Dovetail pins at their narrow end (on `a`'s outer face).
    pub pin: f64,
    /// Pocket holes from `b`'s other face.
    pub flip: bool,
    /// Fasteners this far from the joint's ends (0: a quarter of its length, at most 32 mm).
    pub margin: f64,
    /// Fasteners this far apart from the first one (0: spread evenly between the margins; 32:
    /// the 32 mm system).
    pub pitch: f64,
    /// Fasteners at these distances from the joint's start, its end with the lower coordinate
    /// (empty: laid out by `count`, `margin` and `pitch`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub at: Vec<f64>,
    /// Fasteners this far off the joint's middle, across it.
    pub offset: f64,
    /// Confirmat and nail length, cam lock bolt (`a`'s face to the cam's centre), handle
    /// centres, slide length (0: the longest that fits), leg or caster height, gas strut force
    /// (N).
    pub size: f64,
    /// Cup hinge on an inset door (`b`); else an overlay door (`a`).
    pub inset: bool,
    /// Shelf pins: rows of holes 32 mm apart above and below the shelf's.
    pub rows: usize,
    /// Slides: what kind.
    #[serde(skip_serializing_if = "SlideKind::is_default")]
    pub slide: SlideKind,
    /// Slides: push to open (a front without a handle).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub push: bool,
}

/// A drilled hole: where it starts, its direction, its diameter.
pub type Hole = (DVec3, DVec3, f64);
/// A hole and how deep it goes.
pub type Bore = (DVec3, DVec3, f64, f64);

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
    /// `a`'s and `b`'s bounding boxes (`b`'s is `a`'s for a fitting on one part).
    pub a_min: DVec3,
    pub a_max: DVec3,
    pub b_min: DVec3,
    pub b_max: DVec3,
    /// How far apart the faces are (slides; 0 where they touch).
    pub gap: f64,
}

impl Contact {
    /// Unit vector across the plane, into `a`.
    pub fn normal(&self) -> DVec3 {
        let mut n = DVec3::ZERO;
        n[self.axis] = self.into_a;
        n
    }

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
/// Pocket holes: their angle to `b`'s face, and how much of `b` the screw head keeps.
const POCKET_ANGLE: f64 = 15.0;
const POCKET_KEEP: f64 = 12.0;

/// A solid with eight corners `p(i, j, k)` (0 or 1 each): a box, or a box sheared into a
/// prism. Its faces are turned outwards whatever the corners' handedness.
fn hexahedron(p: impl Fn(usize, usize, usize) -> DVec3) -> Option<Mesh> {
    let c = |i, j, k| p(i, j, k);
    let mut quads = [
        [c(0, 0, 0), c(0, 1, 0), c(1, 1, 0), c(1, 0, 0)],
        [c(0, 0, 1), c(1, 0, 1), c(1, 1, 1), c(0, 1, 1)],
        [c(0, 0, 0), c(1, 0, 0), c(1, 0, 1), c(0, 0, 1)],
        [c(1, 0, 0), c(1, 1, 0), c(1, 1, 1), c(1, 0, 1)],
        [c(1, 1, 0), c(0, 1, 0), c(0, 1, 1), c(1, 1, 1)],
        [c(0, 1, 0), c(0, 0, 0), c(0, 0, 1), c(0, 1, 1)],
    ];
    let volume: f64 = quads.iter().map(|q| q[0].dot(q[1].cross(q[2])) + q[0].dot(q[2].cross(q[3]))).sum();
    if volume.abs() < 1e-9 {
        return None;
    }
    if volume < 0.0 {
        for q in &mut quads {
            q.reverse();
        }
    }
    let mut m = Mesh::new();
    for q in &quads {
        m.add_face_exact(q)?;
    }
    Some(m)
}

fn world_bounds(model: &Model, path: &[crate::InstanceId]) -> Option<(DVec3, DVec3)> {
    // as assembled: the exploded view's offsets taken off
    let (&id, parent) = path.split_last()?;
    let off = model.assembly_offset(path);
    model.instance_world_bounds(parent, id).map(|b| (b.min - off, b.max - off))
}

/// Where parts `a` and `b` touch: the largest rectangle where a face of one's bounding box lies
/// on a face of the other's.
pub fn contact(model: &Model, a: &[crate::InstanceId], b: &[crate::InstanceId]) -> Option<Contact> {
    contact_within(model, a, b, 0.0)
}

/// Like [`contact`], the faces up to `gap` apart (the plane on `a`'s face).
pub fn contact_within(model: &Model, a: &[crate::InstanceId], b: &[crate::InstanceId], gap: f64) -> Option<Contact> {
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
            let apart = (at - b_face) * into_a;
            if apart < -TOUCH || apart > gap + TOUCH {
                continue;
            }
            let (mut lo, mut hi) = (lo, hi);
            (lo[axis], hi[axis]) = (at, at);
            let (along, across) = if hi[u] - lo[u] >= hi[v] - lo[v] { (u, v) } else { (v, u) };
            let c = Contact { axis, at, into_a, lo, hi, along, across, a_size: a1[axis] - a0[axis], b_size: b1[axis] - b0[axis], a_min: a0, a_max: a1, b_min: b0, b_max: b1, gap: apart.max(0.0) };
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

/// A countersunk screw: its head at `head`, going `dir` for `len` (head included).
fn screw(head: DVec3, dir: DVec3, len: f64, d: f64) -> Vec<Solid> {
    let h = d * 0.5;
    vec![
        Solid::Round { at: head, dir, len: h, r0: d, r1: d / 2.0 },
        Solid::Round { at: head + dir * h, dir, len: (len - h - d).max(0.5), r0: d / 2.0, r1: d / 2.0 },
        Solid::Round { at: head + dir * (len - d), dir, len: d, r0: d / 2.0, r1: 0.0 },
    ]
}

/// A face of part `a` as a contact (fittings on one part): its front face (the thinnest
/// axis, the lower side; `back`: the upper), or for legs its underside.
fn face_contact(model: &Model, a: &[crate::InstanceId], leg: bool, back: bool) -> Option<Contact> {
    let (a0, a1) = world_bounds(model, a)?;
    let s = a1 - a0;
    let axis = if leg { 2 } else { (0..3).min_by(|&i, &k| s[i].total_cmp(&s[k])).unwrap_or(1) };
    let (at, into_a) = if back && !leg { (a1[axis], -1.0) } else { (a0[axis], 1.0) };
    let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
    let (mut lo, mut hi) = (a0, a1);
    (lo[axis], hi[axis]) = (at, at);
    let (along, across) = if s[u] >= s[v] { (u, v) } else { (v, u) };
    Some(Contact { axis, at, into_a, lo, hi, along, across, a_size: s[axis], b_size: 0.0, a_min: a0, a_max: a1, b_min: a0, b_max: a1, gap: 0.0 })
}

/// Slides are found between faces this far apart.
const SLIDE_GAP_MAX: f64 = 30.0;
/// The gap a ball-bearing slide takes, and what is tolerated for any slide.
pub const SLIDE_GAP: f64 = 12.7;
const SLIDE_GAP_TOL: f64 = 0.8;
/// Slide lengths on sale.
pub const SLIDE_LENGTHS: [f64; 9] = [250.0, 300.0, 350.0, 400.0, 450.0, 500.0, 550.0, 600.0, 650.0];
/// Cup hinges: the cup's centre from the door's edge; the angle they open to unless set.
const CUP_FROM_EDGE: f64 = 22.5;
pub const HINGE_ANGLE: f64 = 110.0;
/// Gas struts: the tube's and the rod's radii, how far in from the side's face the strut runs.
const STRUT_TUBE: f64 = 9.0;
const STRUT_ROD: f64 = 4.0;
const STRUT_IN: f64 = 14.0;

/// Where joint `j`'s parts touch (fittings on one part: the face they go on).
pub fn joint_contact(model: &Model, j: &WoodJoint) -> Option<Contact> {
    kind_contact(model, j.kind, &j.a, &j.b, j.flip)
}

fn kind_contact(model: &Model, kind: WoodJointKind, a: &[crate::InstanceId], b: &[crate::InstanceId], flip: bool) -> Option<Contact> {
    match kind {
        k if k.single() => face_contact(model, a, k.under(), flip),
        WoodJointKind::Slide => contact_within(model, a, b, SLIDE_GAP_MAX),
        _ => contact(model, a, b),
    }
}

/// The shortest distance between segments `p0`–`p1` and `q0`–`q1`.
fn segment_distance(p0: DVec3, p1: DVec3, q0: DVec3, q1: DVec3) -> f64 {
    let (d1, d2, r) = (p1 - p0, q1 - q0, p0 - q0);
    let (a, e, f) = (d1.length_squared(), d2.length_squared(), d2.dot(r));
    let (s, t) = if a < 1e-12 && e < 1e-12 {
        (0.0, 0.0)
    } else if a < 1e-12 {
        (0.0, (f / e).clamp(0.0, 1.0))
    } else {
        let c = d1.dot(r);
        if e < 1e-12 {
            ((-c / a).clamp(0.0, 1.0), 0.0)
        } else {
            let b = d1.dot(d2);
            let den = a * e - b * b;
            let mut s = if den > 1e-12 { ((b * f - c * e) / den).clamp(0.0, 1.0) } else { 0.0 };
            let mut t = (b * s + f) / e;
            if t < 0.0 {
                t = 0.0;
                s = (-c / a).clamp(0.0, 1.0);
            } else if t > 1.0 {
                t = 1.0;
                s = ((b - c) / a).clamp(0.0, 1.0);
            }
            (s, t)
        }
    };
    ((p0 + d1 * s) - (q0 + d2 * t)).length()
}

/// A box solid between two corners.
fn block(p: DVec3, q: DVec3) -> Solid {
    Solid::Block { min: p.min(q), max: p.max(q) }
}

/// Unit vector along axis `k`, times `s`.
fn unit(k: usize, s: f64) -> DVec3 {
    let mut v = DVec3::ZERO;
    v[k] = s;
    v
}

/// A caster plate's four screws about its centre `p` (on the underside, axis z).
fn caster_screws(p: DVec3) -> [DVec3; 4] {
    [(-18.0, -18.0), (18.0, -18.0), (-18.0, 18.0), (18.0, 18.0)].map(|(x, y)| p + DVec3::new(x, y, 0.0))
}

/// Rounded up to 5 mm.
fn ceil5(v: f64) -> f64 {
    (v / 5.0).ceil() * 5.0
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
            // fingers about as wide as the board is thick
            WoodJointKind::BoxJoint => j.count = (((len / t.max(1.0)) + 1.0) / 2.0).round().max(2.0) as usize,
            WoodJointKind::Dovetail => {
                j.count = (len / (3.0 * t.max(1.0))).round().max(1.0) as usize;
                j.angle = 7.1; // 1:8
                j.pin = snap((t / 3.0).max(5.0));
            }
            WoodJointKind::PocketHole => {
                j.diameter = 4.0;
                j.depth = snap((c.a_size * 0.6).min(25.0));
                j.count = if len < 100.0 { 1 } else { ((len / 200.0).round() as usize + 1).clamp(2, 8) };
            }
            WoodJointKind::Confirmat => {
                j.diameter = 7.0;
                j.size = if c.a_size + 30.0 <= 50.0 { 50.0 } else { ceil5(c.a_size + 30.0) };
            }
            WoodJointKind::CamLock => {
                j.diameter = 15.0;
                j.size = 34.0;
            }
            WoodJointKind::ShelfPins => {
                j.diameter = 5.0;
                j.depth = 10.0;
                j.count = 2;
                j.margin = 37.0_f64.min(len / 4.0);
            }
            WoodJointKind::Hinge => {
                j.diameter = 35.0;
                j.depth = 12.5;
                j.count = match len {
                    l if l <= 900.0 => 2,
                    l if l <= 1600.0 => 3,
                    l if l <= 2000.0 => 4,
                    _ => 5,
                };
                j.margin = 100.0_f64.min(len / 5.0);
                j.angle = HINGE_ANGLE;
            }
            WoodJointKind::GasStrut => {
                // one each side: about three times the door's weight, in tens of newtons
                let s = c.a_max - c.a_min;
                let kg = s.x * s.y * s.z * 0.7e-6;
                j.count = 1;
                j.size = ((kg * 9.81 * 3.0 / 10.0).ceil() * 10.0).clamp(50.0, 250.0);
            }
            WoodJointKind::Slide => j.count = 1,
            WoodJointKind::Handle => {
                j.count = 1;
                j.diameter = 12.0;
                j.size = [192.0, 160.0, 128.0, 96.0, 64.0].into_iter().find(|s| s + 60.0 <= len).unwrap_or(32.0);
            }
            WoodJointKind::Knob => {
                j.count = 1;
                j.diameter = 30.0;
            }
            WoodJointKind::Leg => {
                j.count = 4;
                j.diameter = 40.0;
                j.size = 100.0;
                j.margin = 50.0_f64.min(t / 4.0).max(20.0);
            }
            WoodJointKind::Caster => {
                // a Ø50 wheel, 70 mm from the floor to the plate
                j.count = 4;
                j.diameter = 50.0;
                j.size = 70.0;
                j.margin = 40.0_f64.min(t / 4.0).max(25.0);
            }
            WoodJointKind::TableTop => {
                j.diameter = 19.0;
                j.count = ((len / 300.0).round() as usize + 1).clamp(2, 12);
            }
            WoodJointKind::Nails => {
                j.diameter = 1.6;
                j.size = ceil5(c.a_size + 15.0).max(20.0);
                j.count = ((len / 100.0).round() as usize + 1).clamp(2, 40);
                j.margin = 30.0_f64.min(len / 4.0);
            }
        }
        j
    }

    /// Hardware it takes: (name, count) for dowels, screws and pocket screws.
    pub fn hardware(&self, c: Option<&Contact>) -> Option<(String, usize)> {
        let d = num(self.diameter);
        match self.kind {
            WoodJointKind::Dowels => Some((tr!("joint-hw-dowel", diameter = d, length = num(self.depth + self.depth_b - 2.0)), self.fastener_count())),
            WoodJointKind::Screws => {
                let len = c.map_or(self.depth_b, |c| c.a_size + self.depth_b);
                Some((tr!("joint-hw-screw", diameter = d, length = num((len / 5.0).ceil() * 5.0)), self.fastener_count()))
            }
            WoodJointKind::PocketHole => {
                Some((tr!("joint-hw-pocket-screw", diameter = d, length = num(((POCKET_KEEP + self.depth) / 5.0).ceil() * 5.0)), self.fastener_count()))
            }
            WoodJointKind::Confirmat => Some((tr!("joint-hw-confirmat", diameter = d, length = num(self.size)), self.fastener_count())),
            WoodJointKind::CamLock => Some((tr!("joint-hw-cam-lock", diameter = d), self.fastener_count())),
            WoodJointKind::ShelfPins => Some((tr!("joint-hw-shelf-pin", diameter = d), self.fastener_count())),
            WoodJointKind::Hinge => {
                let half = !self.inset && c.is_some_and(|c| c.width() + 1.0 < c.b_max[c.across] - c.b_min[c.across]);
                let name = match (self.inset, half) {
                    (true, _) => tr!("joint-hw-hinge-inset", diameter = d),
                    (false, true) => tr!("joint-hw-hinge-half", diameter = d),
                    (false, false) => tr!("joint-hw-hinge-overlay", diameter = d),
                };
                let angle = self.hinge_angle();
                let name = if (angle - HINGE_ANGLE).abs() < 0.5 { name } else { tr!("joint-hw-hinge-angle", hinge = name, angle = num(angle)) };
                Some((name, self.fastener_count()))
            }
            WoodJointKind::GasStrut => Some((tr!("joint-hw-gas-strut", force = num(self.size)), 1)),
            WoodJointKind::Slide => {
                let length = num(c.and_then(|c| self.slide_length(c)).unwrap_or(self.size));
                let name = match self.slide {
                    SlideKind::BallBearing => tr!("joint-hw-slide", length = length),
                    SlideKind::Roller => tr!("joint-hw-slide-roller", length = length),
                    SlideKind::Undermount => tr!("joint-hw-slide-undermount", length = length),
                };
                Some((if self.push { tr!("joint-hw-slide-push", slide = name) } else { name }, 1))
            }
            WoodJointKind::Handle => Some((tr!("joint-hw-handle", length = num(self.size)), self.fastener_count())),
            WoodJointKind::Knob => Some((tr!("joint-hw-knob", diameter = d), self.fastener_count())),
            WoodJointKind::Leg => Some((tr!("joint-hw-leg", length = num(self.size)), 4)),
            WoodJointKind::Caster => Some((tr!("joint-hw-caster", diameter = d, length = num(self.size)), 4)),
            WoodJointKind::TableTop => Some((tr!("joint-hw-table-top"), self.fastener_count())),
            WoodJointKind::Nails => Some((tr!("joint-hw-nail", diameter = d, length = num(self.size)), self.fastener_count())),
            _ => None,
        }
    }

    /// The slide's length: its own, else the longest on sale that fits along the joint.
    pub fn slide_length(&self, c: &Contact) -> Option<f64> {
        if self.size > 0.0 {
            return (self.size <= c.length() + 1e-6).then_some(self.size);
        }
        SLIDE_LENGTHS.iter().rev().copied().find(|l| *l <= c.length() + 1e-6)
    }

    /// The angle a cup hinge opens to.
    pub fn hinge_angle(&self) -> f64 {
        if self.angle > 0.0 { self.angle } else { HINGE_ANGLE }
    }

    /// A gas strut's ends, its bracket on the side `b` and on the door `a` (on the strut's
    /// line, `STRUT_IN` in from the side's inner face), and the way from the side into the
    /// carcass. The contact is the door's back on the side's front edge: the door's height
    /// along it, the side's thickness across; the side bracket a third up the edge and about
    /// a third of its height back (shelves set back clear it), the door's a third down from
    /// its top.
    fn strut(&self, c: &Contact) -> (DVec3, DVec3, DVec3) {
        let n = c.normal();
        let door_mid = (c.a_min[c.across] + c.a_max[c.across]) * 0.5;
        let inward = if door_mid >= (c.lo[c.across] + c.hi[c.across]) * 0.5 { 1.0 } else { -1.0 };
        let face = if inward > 0.0 { c.hi[c.across] } else { c.lo[c.across] };
        let (lo, hi) = (c.lo[c.along], c.hi[c.along]);
        let l = hi - lo;
        let side_depth = (c.b_max - c.b_min)[c.axis];
        let mut p = c.lo;
        p[c.across] = face + inward * STRUT_IN;
        let (mut side, mut door) = (p, p);
        side[c.along] = lo + l * 0.3;
        door[c.along] = hi - l * 0.3;
        let back = (l * 0.35).min(side_depth - 40.0).max(40.0);
        (side - n * back, door - n * 12.0, unit(c.across, inward))
    }

    /// The door of a cup hinge is `a` (overlay) or `b` (inset).
    fn door_is_a(&self) -> bool {
        !self.inset
    }

    /// A cup hinge's cup at `s` along the joint: its centre on the door's back face and the
    /// way into the door. Overlay: the door's back on the plane, the cup in from the door's
    /// edge by the side; inset: in from the door's edge on the side, from its back face (the
    /// side the carcass goes on).
    fn hinge_cup(&self, c: &Contact, s: f64) -> (DVec3, DVec3) {
        let mut p = self.point(c, s);
        let n = c.normal();
        if self.door_is_a() {
            let (edge, inward) = self.door_edge(c);
            p[c.across] = edge + inward * CUP_FROM_EDGE + self.offset;
            (p, n)
        } else {
            let (back, sb) = self.door_back(c);
            p[c.across] = back;
            (p - n * CUP_FROM_EDGE, unit(c.across, -sb))
        }
    }

    /// Overlay door: its edge by the side (across) and the way into the door from there.
    fn door_edge(&self, c: &Contact) -> (f64, f64) {
        let m = (c.lo[c.across] + c.hi[c.across]) * 0.5;
        if (m - c.a_min[c.across]).abs() <= (c.a_max[c.across] - m).abs() { (c.a_min[c.across], 1.0) } else { (c.a_max[c.across], -1.0) }
    }

    /// Inset door: its back face (across) and the way back from it, where the side goes on.
    fn door_back(&self, c: &Contact) -> (f64, f64) {
        if c.a_max[c.across] - c.hi[c.across] >= c.lo[c.across] - c.a_min[c.across] { (c.hi[c.across], 1.0) } else { (c.lo[c.across], -1.0) }
    }

    /// A cam lock's cam: its centre on `b`'s face and the way into `b`.
    fn cam(&self, c: &Contact, s: f64) -> (DVec3, DVec3) {
        let mut p = self.point(c, s) - c.normal() * self.size;
        let (face, dir) = if self.flip { (c.hi[c.across], -1.0) } else { (c.lo[c.across], 1.0) };
        p[c.across] = face;
        (p, unit(c.across, dir))
    }

    /// Shelf pin holes: under the shelf (`flip`: over it), with the rows above and below that
    /// lie on `a`; the first `fastener_count` are the pins'.
    fn pin_holes(&self, c: &Contact) -> Vec<DVec3> {
        let r = self.diameter / 2.0;
        let (lo, hi) = (c.a_min[c.across] + r, c.a_max[c.across] - r);
        let y = if self.flip { c.hi[c.across] + r } else { c.lo[c.across] - r };
        let mut out: Vec<DVec3> = self.spots(c).into_iter().map(|s| {
            let mut p = self.point(c, s);
            p[c.across] = y;
            p
        }).collect();
        let pins = out.clone();
        for k in 1..=self.rows {
            for d in [-32.0 * k as f64, 32.0 * k as f64] {
                if (lo..=hi).contains(&(y + d)) {
                    out.extend(pins.iter().map(|p| *p + unit(c.across, d)));
                }
            }
        }
        out
    }

    /// Legs: in from the corners of the underside by `margin`.
    fn leg_points(&self, c: &Contact) -> Vec<DVec3> {
        let m = self.margin;
        let mut out = Vec::new();
        for u in [c.lo[c.along] + m, c.hi[c.along] - m] {
            for v in [c.lo[c.across] + m, c.hi[c.across] - m] {
                let mut p = c.lo;
                (p[c.along], p[c.across]) = (u, v);
                out.push(p);
            }
        }
        out
    }

    /// A gas strut's bracket screws: two in the door's back, two in the side's inner face.
    fn strut_screws(&self, c: &Contact) -> (Vec<DVec3>, Vec<DVec3>) {
        let (side, door, inward) = self.strut(c);
        let n = c.normal();
        let up = unit(c.along, 1.0);
        let on_door = |p: DVec3| {
            let mut q = p;
            q[c.axis] = c.at;
            q
        };
        let on_side = |p: DVec3| p - inward * STRUT_IN;
        (vec![on_door(door - up * 12.0), on_door(door + up * 12.0)], vec![on_side(side - n * 12.0), on_side(side + n * 12.0)])
    }

    /// A handle's two screw points about `e`, or a knob's one.
    fn handle_points(&self, c: &Contact, e: DVec3) -> Vec<DVec3> {
        if self.kind == WoodJointKind::Knob {
            return vec![e];
        }
        let u = unit(c.along, self.size / 2.0);
        vec![e - u, e + u]
    }

    /// The holes drilled into `a` and into `b`: where each starts on the part's surface, its
    /// direction (a pocket's: into the face it is drilled from) and diameter, in world space.
    pub fn holes(&self, c: &Contact) -> (Vec<Hole>, Vec<Hole>) {
        let (a, b) = self.bores(c);
        let hole = |v: Vec<Bore>| v.into_iter().map(|(p, d, r, _)| (p, d, r)).collect();
        (hole(a), hole(b))
    }

    /// [`WoodJoint::holes`] with how deep each goes (0: not known, a pocket's counterbore, a
    /// leg's bolt).
    pub fn bores(&self, c: &Contact) -> (Vec<Bore>, Vec<Bore>) {
        let mut n = DVec3::ZERO;
        n[c.axis] = c.into_a;
        let point = |s: f64| self.point(c, s);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        for s in self.spots(c) {
            let e = point(s);
            match self.kind {
                WoodJointKind::Dowels => {
                    a.push((e, n, self.diameter, self.depth));
                    b.push((e, -n, self.diameter, self.depth_b));
                }
                WoodJointKind::Screws => {
                    a.push((e, n, self.diameter + 0.5, c.a_size));
                    b.push((e, -n, self.diameter * 0.7, self.depth_b));
                }
                WoodJointKind::PocketHole => {
                    let (sin, cos) = POCKET_ANGLE.to_radians().sin_cos();
                    let mut m = DVec3::ZERO;
                    m[c.across] = if self.flip { -1.0 } else { 1.0 };
                    let dir = n * cos + m * sin;
                    let entry = e - dir * (c.width() / 2.0 / sin);
                    b.push((entry, m, (2.0 * self.diameter + 1.5).max(9.5), 0.0));
                    a.push((e, dir, self.diameter * 0.7, self.depth));
                }
                WoodJointKind::Confirmat => {
                    a.push((e, n, self.diameter, c.a_size));
                    b.push((e, -n, 5.0, (self.size - c.a_size).max(0.0)));
                }
                WoodJointKind::CamLock => {
                    a.push((e, n, 5.0, 11.0));
                    b.push((e, -n, 8.0, self.size));
                    let (p, dir) = self.cam(c, s);
                    b.push((p, dir, self.diameter, 13.0));
                }
                WoodJointKind::Hinge => {
                    let (p, dir) = self.hinge_cup(c, s);
                    if self.door_is_a() { &mut a } else { &mut b }.push((p, dir, self.diameter, self.depth));
                }
                WoodJointKind::Handle | WoodJointKind::Knob => a.extend(self.handle_points(c, e).into_iter().map(|p| (p, n, 5.0, c.a_size))),
                _ => break,
            }
        }
        match self.kind {
            WoodJointKind::GasStrut => {
                let (sa, sb) = self.strut_screws(c);
                a.extend(sa.into_iter().map(|p| (p, n, 3.0, 10.0)));
                let (_, _, inward) = self.strut(c);
                b.extend(sb.into_iter().map(|p| (p, -inward, 3.0, 10.0)));
            }
            WoodJointKind::ShelfPins => a.extend(self.pin_holes(c).into_iter().map(|p| (p, n, self.diameter, self.depth))),
            WoodJointKind::Leg => a.extend(self.leg_points(c).into_iter().map(|p| (p, n, 8.0, 0.0))),
            WoodJointKind::Caster => a.extend(self.leg_points(c).into_iter().flat_map(caster_screws).map(|p| (p, n, 3.0, 10.0))),
            _ => {}
        }
        (a, b)
    }

    /// What about it works but is likely a mistake: fasteners too close to the joint's ends or
    /// to each other, holes too thick for the board they go into or coming out of its other
    /// face, too few hinges for the door's height. Holes of two joints meeting are found by
    /// [`Model::joint_warnings`].
    pub fn warnings(&self, c: &Contact) -> Vec<String> {
        let mut out = Vec::new();
        let d = num(self.diameter);
        let fastened = matches!(self.kind, WoodJointKind::Dowels | WoodJointKind::Screws | WoodJointKind::PocketHole | WoodJointKind::Confirmat | WoodJointKind::CamLock);
        if fastened {
            // the hole's centre at least its diameter and a millimetre in from the end
            let (lo, hi) = (c.lo[c.along], c.hi[c.along]);
            let min = self.diameter + 1.0;
            let spots = self.spots(c);
            if let Some(at) = spots.iter().map(|&s| (s - lo).min(hi - s)).min_by(f64::total_cmp).filter(|&e| e < min - 1e-6) {
                out.push(tr!("joint-warn-end", at = num(at), min = num(min)));
            }
            if let Some(gap) = spots.windows(2).map(|w| w[1] - w[0]).min_by(f64::total_cmp).filter(|&g| g < self.diameter + 2.0 - 1e-6) {
                out.push(tr!("joint-warn-spacing", gap = num(gap), min = num(self.diameter + 2.0)));
            }
        }
        let width = c.width();
        match self.kind {
            WoodJointKind::Dowels if self.diameter > width / 2.0 + 0.5 => out.push(tr!("joint-warn-dowel-thick", diameter = d, thickness = num(width))),
            WoodJointKind::Confirmat if width < 16.0 - 1e-6 => out.push(tr!("joint-warn-confirmat-thin", thickness = num(width))),
            WoodJointKind::Screws if (width - self.diameter) / 2.0 - self.offset.abs() < 3.0 => out.push(tr!("joint-warn-screw-thick", diameter = d, thickness = num(width))),
            _ => {}
        }
        // blind holes into a's face
        let blind = match self.kind {
            WoodJointKind::Dowels | WoodJointKind::ShelfPins => Some((self.depth, c.a_size)),
            WoodJointKind::Hinge => Some((self.depth, if self.door_is_a() { c.a_size } else { c.b_max[c.axis] - c.b_min[c.axis] })),
            _ => None,
        };
        if let Some((depth, thick)) = blind.filter(|(depth, thick)| depth + 3.0 > thick + 1e-6) {
            out.push(tr!("joint-warn-through", depth = num(depth), thickness = num(thick)));
        }
        if self.kind == WoodJointKind::Hinge {
            let want = match c.length() {
                l if l <= 900.0 => 2,
                l if l <= 1600.0 => 3,
                l if l <= 2000.0 => 4,
                _ => 5,
            };
            if self.fastener_count() < want {
                out.push(tr!("joint-warn-hinges", height = num(c.length()), count = want.to_string()));
            }
        }
        out
    }

    /// The bodies of its fasteners, in world space as assembled (their joint and parts' frames
    /// are filled in by [`Model::refresh_fasteners`]).
    pub fn bodies(&self, c: &Contact) -> Vec<Fastener> {
        let n = c.normal();
        let name = self.hardware(Some(c)).map(|h| h.0).unwrap_or_default();
        let body = |anchor, out: DVec3, pull, color, solids| Fastener {
            joint: 0,
            anchor,
            local_a: DMat4::IDENTITY,
            local_b: DMat4::IDENTITY,
            out,
            pull,
            color,
            solids,
            name: name.clone(),
        };
        let mut out = Vec::new();
        for s in self.spots(c) {
            let e = self.point(c, s);
            match self.kind {
                WoodJointKind::Dowels => {
                    // a little short of both hole bottoms
                    let r = self.diameter / 2.0 * 0.97;
                    let len = (self.depth + self.depth_b - 2.0).max(1.0);
                    out.push(body(Anchor::Earlier, n, 0.0, WOOD, vec![Solid::Round { at: e - n * (self.depth_b - 1.0), dir: n, len, r0: r, r1: r }]));
                }
                WoodJointKind::Screws => {
                    // from a's far face through a into b, countersunk
                    let d = self.diameter;
                    let head = e + n * c.a_size;
                    let len = ((c.a_size + self.depth_b) / 5.0).ceil() * 5.0;
                    out.push(body(Anchor::A, n, 0.6, STEEL, screw(head, -n, len, d)));
                }
                WoodJointKind::PocketHole => {
                    let (dir, _) = self.pocket_dir(c);
                    let from = e - dir * POCKET_KEEP;
                    let len = ((POCKET_KEEP + self.depth) / 5.0).ceil() * 5.0;
                    let d = self.diameter;
                    let mut solids = vec![Solid::Round { at: from - dir * 2.5, dir, len: 2.5, r0: d * 1.1, r1: d * 1.1 }];
                    solids.push(Solid::Round { at: from, dir, len: len - d, r0: d / 2.0, r1: d / 2.0 });
                    solids.push(Solid::Round { at: from + dir * (len - d), dir, len: d, r0: d / 2.0, r1: 0.0 });
                    out.push(body(Anchor::B, -dir, 0.6, STEEL, solids));
                }
                WoodJointKind::Confirmat => {
                    // a countersunk head on a's far face, the shank through a into b
                    let (head, r) = (e + n * c.a_size, self.diameter / 2.0);
                    let solids = vec![
                        Solid::Round { at: head, dir: -n, len: 3.0, r0: r + 1.5, r1: r },
                        Solid::Round { at: head - n * 3.0, dir: -n, len: (self.size - 7.0).max(1.0), r0: r, r1: r },
                        Solid::Round { at: head - n * (self.size - 4.0), dir: -n, len: 4.0, r0: r, r1: 0.0 },
                    ];
                    out.push(body(Anchor::A, n, 0.6, STEEL, solids));
                }
                WoodJointKind::CamLock => {
                    // the bolt screwed into a, its head where the cam takes it
                    let solids = vec![
                        Solid::Round { at: e + n * 11.0, dir: -n, len: 11.0, r0: 2.5, r1: 2.5 },
                        Solid::Round { at: e, dir: -n, len: (self.size - 4.0).max(1.0), r0: 3.0, r1: 3.0 },
                        Solid::Round { at: e - n * (self.size - 4.0), dir: -n, len: 4.0, r0: 3.5, r1: 3.5 },
                    ];
                    out.push(body(Anchor::A, n, 0.0, STEEL, solids));
                    let (p, dir) = self.cam(c, s);
                    let r = self.diameter / 2.0 - 0.3;
                    out.push(body(Anchor::B, -dir, 0.6, ZINC, vec![Solid::Round { at: p + dir * 0.5, dir, len: 12.0, r0: r, r1: r }]));
                }
                WoodJointKind::Hinge => {
                    let (p, dir) = self.hinge_cup(c, s);
                    let r = self.diameter / 2.0;
                    let cup = Solid::Round { at: p, dir, len: self.depth, r0: r, r1: r };
                    // the arm from the cup to the plate on the side, the plate on the side's face
                    let along = |p: DVec3, h: f64| (p - unit(c.along, h), p + unit(c.along, h));
                    let (arm, plate) = if self.door_is_a() {
                        let (_, inward) = self.door_edge(c);
                        let mut q = p;
                        q[c.across] = if inward > 0.0 { c.hi[c.across] } else { c.lo[c.across] };
                        let ((q0, q1), (r0, r1)) = (along(q, 9.0), along(q, 18.0));
                        let w = |k: f64| unit(c.across, inward * k);
                        (block(q0 + w(3.0), q1 + w(15.0) - n * 60.0), block(r0 - n * 20.0, r1 + w(3.0) - n * 60.0))
                    } else {
                        let (_, sb) = self.door_back(c);
                        let w = |k: f64| unit(c.across, sb * k);
                        let ((q0, mut q1), (mut r0, mut r1)) = (along(p, 9.0), along(p, 18.0));
                        q1[c.axis] = c.at;
                        (r0[c.axis], r1[c.axis]) = (c.at, c.at);
                        (block(q0 + w(3.0), q1 + w(25.0) - n * 3.0), block(r0 + w(20.0), r1 + w(60.0) - n * 3.0))
                    };
                    let (door, side) = if self.door_is_a() { (Anchor::A, Anchor::B) } else { (Anchor::B, Anchor::A) };
                    out.push(body(door, DVec3::ZERO, 0.0, STEEL, vec![cup, arm]));
                    out.push(body(side, DVec3::ZERO, 0.0, STEEL, vec![plate]));
                }
                WoodJointKind::Handle | WoodJointKind::Knob => {
                    // standing off the face, out of a
                    let mut solids = Vec::new();
                    for p in self.handle_points(c, e) {
                        let stem = if self.kind == WoodJointKind::Knob { 18.0 } else { 28.0 };
                        solids.push(Solid::Round { at: p, dir: -n, len: stem, r0: 4.5, r1: 4.5 });
                    }
                    if self.kind == WoodJointKind::Knob {
                        let r = self.diameter / 2.0;
                        solids.push(Solid::Round { at: e - n * 18.0, dir: -n, len: 14.0, r0: r * 0.6, r1: r });
                        solids.push(Solid::Round { at: e - n * 32.0, dir: -n, len: 3.0, r0: r, r1: r * 0.7 });
                    } else {
                        let r = self.diameter / 2.0;
                        let u = unit(c.along, 1.0);
                        let half = self.size / 2.0 + 15.0;
                        solids.push(Solid::Round { at: e - n * (28.0 + r) - u * half, dir: u, len: 2.0 * half, r0: r, r1: r });
                    }
                    out.push(body(Anchor::A, -n, 0.0, if self.kind == WoodJointKind::Knob { BRASS } else { STEEL }, solids));
                }
                WoodJointKind::TableTop => {
                    // two lobes flush with the apron's edge, one in it and one under the top
                    let m = unit(c.across, if self.flip { -1.0 } else { 1.0 });
                    let r = self.diameter / 2.0;
                    let solids = [e, e + m * (r * 1.7)].map(|p| Solid::Round { at: p - n * 3.0, dir: n, len: 3.0, r0: r, r1: r }).to_vec();
                    out.push(body(Anchor::B, DVec3::ZERO, 0.0, STEEL, solids));
                }
                WoodJointKind::Nails => {
                    let (head, r) = (e + n * c.a_size, self.diameter / 2.0);
                    let solids = vec![
                        Solid::Round { at: head, dir: -n, len: 0.8, r0: r * 2.2, r1: r * 2.2 },
                        Solid::Round { at: head - n * 0.8, dir: -n, len: (self.size - 2.8).max(0.5), r0: r, r1: r },
                        Solid::Round { at: head - n * (self.size - 2.0), dir: -n, len: 2.0, r0: r, r1: 0.0 },
                    ];
                    out.push(body(Anchor::A, n, 0.6, STEEL, solids));
                }
                WoodJointKind::ShelfPins => {
                    let p = self.pin_holes(c)[out.len()];
                    let r = self.diameter / 2.0 * 0.95;
                    let solids = vec![
                        Solid::Round { at: p + n * self.depth, dir: -n, len: self.depth + 12.0, r0: r, r1: r },
                        Solid::Round { at: p - n * 2.0, dir: -n, len: 1.5, r0: r * 1.8, r1: r * 1.8 },
                    ];
                    out.push(body(Anchor::A, DVec3::ZERO, 0.0, BRASS, solids));
                }
                _ => break,
            }
        }
        match self.kind {
            WoodJointKind::GasStrut => {
                // the tube from the side's bracket, the rod on to the door's; brackets on both
                let (side, door, inward) = self.strut(c);
                let d = door - side;
                let (len, dir) = (d.length(), d.normalize_or_zero());
                let tube = Solid::Round { at: side, dir, len: len * 0.55, r0: STRUT_TUBE, r1: STRUT_TUBE };
                let rod = Solid::Round { at: side + dir * len * 0.55, dir, len: len * 0.45, r0: STRUT_ROD, r1: STRUT_ROD };
                let (n, up) = (c.normal(), unit(c.along, 1.0));
                let side_bracket = block(side - inward * STRUT_IN - n * 20.0 - up * 10.0, side + n * 20.0 + up * 10.0 - inward * 4.0);
                let mut q = door;
                q[c.axis] = c.at;
                let door_bracket = block(q - up * 20.0 - inward * 10.0, door + up * 20.0 + inward * 10.0 + n * 0.0);
                out.push(body(Anchor::B, DVec3::ZERO, 0.0, DARK, vec![tube, side_bracket]));
                out.push(body(Anchor::A, DVec3::ZERO, 0.0, STEEL, vec![rod, door_bracket]));
            }
            WoodJointKind::Slide => {
                if let Some(len) = self.slide_length(c) {
                    // the cabinet member on a, the drawer member on b, the front flush
                    let f = if self.flip { c.hi[c.along] - len } else { c.lo[c.along] };
                    let mut p = self.point(c, f);
                    p[c.along] = f;
                    let (g, u, up) = (c.gap, unit(c.along, len), unit(c.across, 1.0));
                    let mut q = p;
                    q[c.across] = c.lo[c.across];
                    match self.slide {
                        SlideKind::BallBearing => {
                            let h = 45.0_f64.min(c.width());
                            let w = up * (h / 2.0);
                            out.push(body(Anchor::A, DVec3::ZERO, 0.0, ZINC, vec![block(p - w, p + w + u - n * (g / 2.0))]));
                            let w = up * (h / 2.0 - 4.0);
                            out.push(body(Anchor::B, DVec3::ZERO, 0.0, STEEL, vec![block(p - w - n * (g / 2.0), p + w + u - n * g)]));
                        }
                        SlideKind::Roller => {
                            // along the bottom of b, a wheel at the cabinet member's front and
                            // at the drawer member's back
                            let h = 30.0_f64.min(c.width());
                            let wheel = |at: DVec3| Solid::Round { at: at + up * (h / 2.0) - n * (g * 0.2), dir: -n, len: g * 0.6, r0: h * 0.35, r1: h * 0.35 };
                            let back = u - unit(c.along, 20.0);
                            out.push(body(Anchor::A, DVec3::ZERO, 0.0, ZINC, vec![block(q, q + up * h + u - n * (g * 0.2))]));
                            out.push(body(Anchor::A, DVec3::ZERO, 0.0, DARK, vec![wheel(q + unit(c.along, 20.0))]));
                            out.push(body(Anchor::B, DVec3::ZERO, 0.0, STEEL, vec![block(q - n * (g * 0.8), q + up * h + u - n * g)]));
                            out.push(body(Anchor::B, DVec3::ZERO, 0.0, DARK, vec![wheel(q + back)]));
                        }
                        SlideKind::Undermount => {
                            // under b (the drawer's bottom): the cabinet member on a's face
                            // and reaching under b, the drawer member under b's edge
                            let reach = 30.0_f64.min(c.b_max[c.axis] - c.b_min[c.axis]);
                            let (lo, mid) = (q - up * 14.0, q - up * 5.0);
                            out.push(body(Anchor::A, DVec3::ZERO, 0.0, ZINC, vec![block(lo, mid - n * (g + reach) + u), block(lo, q + up * 10.0 - n * 2.0 + u)]));
                            out.push(body(Anchor::B, DVec3::ZERO, 0.0, STEEL, vec![block(mid - n * (g + 2.0), q - n * (g + reach - 4.0) + u)]));
                        }
                    }
                }
            }
            WoodJointKind::Leg => {
                let r = self.diameter / 2.0;
                for p in self.leg_points(c) {
                    out.push(body(Anchor::A, -n, 0.0, DARK, vec![Solid::Round { at: p, dir: -n, len: self.size, r0: r, r1: r * 0.75 }]));
                }
            }
            WoodJointKind::Caster => {
                // a plate on the underside, the fork down to the wheel's axle, the wheel across it
                let (r, down) = (self.diameter / 2.0, -n);
                let (along, across) = (unit(c.along, 1.0), unit(c.across, 1.0));
                for p in self.leg_points(c) {
                    let axle = p + down * (self.size - r);
                    let plate = block(p - along * 25.0 - across * 25.0, p + along * 25.0 + across * 25.0 + down * 3.0);
                    let fork = block(p + down * 3.0 - along * 14.0 - across * (r * 0.45), axle + along * 14.0 + across * (r * 0.45));
                    let wheel = Solid::Round { at: axle - along * 11.0, dir: along, len: 22.0, r0: r, r1: r };
                    out.push(body(Anchor::A, -n, 0.0, ZINC, vec![plate, fork]));
                    out.push(body(Anchor::A, -n, 0.0, DARK, vec![wheel]));
                }
            }
            _ => {}
        }
        out
    }

    /// A pocket screw's direction (from `b`'s face towards its end and on into `a`) and the
    /// way across the joint it leans.
    fn pocket_dir(&self, c: &Contact) -> (DVec3, DVec3) {
        let (sin, cos) = POCKET_ANGLE.to_radians().sin_cos();
        let mut m = DVec3::ZERO;
        m[c.across] = if self.flip { -1.0 } else { 1.0 };
        (c.normal() * cos + m * sin, m)
    }

    /// Where its fasteners go on the contact (world, as assembled).
    pub fn points(&self, c: &Contact) -> Vec<DVec3> {
        self.spots(c).into_iter().map(|s| self.point(c, s)).collect()
    }

    /// The point on the contact at `s` along the joint, across it at `offset` from the middle.
    pub fn point(&self, c: &Contact, s: f64) -> DVec3 {
        let mut p = c.lo;
        p[c.along] = s;
        p[c.across] = (c.lo[c.across] + c.hi[c.across]) * 0.5 + self.offset;
        p
    }

    /// How many fasteners it has.
    pub fn fastener_count(&self) -> usize {
        if self.at.is_empty() { self.count } else { self.at.len() }
    }

    /// Where its fasteners go along the joint, from its start.
    pub fn positions(&self, c: &Contact) -> Vec<f64> {
        let lo = c.lo[c.along];
        self.spots(c).into_iter().map(|s| s - lo).collect()
    }

    /// Where dowels and screws go along the joint (world coordinate on its long axis).
    fn spots(&self, c: &Contact) -> Vec<f64> {
        let (lo, hi) = (c.lo[c.along], c.hi[c.along]);
        if !self.at.is_empty() {
            return self.at.iter().map(|s| lo + s).collect();
        }
        let e = if self.margin > 0.0 { self.margin } else { (c.length() / 4.0).min(32.0).max(self.diameter) };
        if self.pitch > 0.0 {
            return (0..self.count).map(|i| lo + e + self.pitch * i as f64).collect();
        }
        if self.count <= 1 {
            return vec![if self.margin > 0.0 { lo + e } else { (lo + hi) * 0.5 }];
        }
        (0..self.count).map(|i| lo + e + (hi - lo - 2.0 * e) * i as f64 / (self.count - 1) as f64).collect()
    }

    /// The fasteners lie on the joint, their holes clear of its ends and sides.
    fn spots_fit(&self, c: &Contact, r: f64) -> bool {
        let (lo, hi) = (c.lo[c.along], c.hi[c.along]);
        let half = c.width() / 2.0;
        self.spots(c).iter().all(|&s| s - r >= lo - 1e-6 && s + r <= hi + 1e-6) && self.offset.abs() + r <= half + 1e-6
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
        // a cylinder from `from` along `dir`, `len` long
        let cyl_to = |from: DVec3, dir: DVec3, len: f64, r: f64| -> OpResult<(Mesh, DMat4)> {
            let m = Primitive::Cylinder { r, h: len, segs: 16 }.mesh().ok_or_else(bad)?;
            Ok((m, DMat4::from_translation(from) * DMat4::from_quat(DQuat::from_rotation_arc(DVec3::Z, dir))))
        };
        let cyl = |from: DVec3, len: f64, r: f64| cyl_to(from, n, len, r);
        // a box over `lo..hi` across the plane, from `from` to `to` along `n`
        let cube = |lo: DVec3, hi: DVec3, from: f64, to: f64| -> OpResult<(Mesh, DMat4)> {
            let (mut lo, mut hi) = (lo, hi);
            let (p, q) = (c.at + c.into_a * from, c.at + c.into_a * to);
            (lo[c.axis], hi[c.axis]) = (p.min(q), p.max(q));
            let s = hi - lo;
            let m = Primitive::Box { w: s.x, d: s.y, h: s.z }.mesh().ok_or_else(bad)?;
            Ok((m, DMat4::from_translation(lo)))
        };
        let point = |s: f64| self.point(c, s);
        let grow = |lo: DVec3, hi: DVec3, axes: &[usize], by: f64| {
            let (mut lo, mut hi) = (lo, hi);
            for &k in axes {
                lo[k] -= by;
                hi[k] += by;
            }
            (lo, hi)
        };
        // across the joint, out past `a`'s end where it stops at `b`'s face (a corner joint)
        let open = |lo: DVec3, hi: DVec3| {
            let (mut lo, mut hi) = (lo, hi);
            if c.a_min[c.across] > lo[c.across] - TOUCH {
                lo[c.across] -= OVER;
            }
            if c.a_max[c.across] < hi[c.across] + TOUCH {
                hi[c.across] += OVER;
            }
            (lo, hi)
        };
        let (mut from_a, mut from_b, mut to_b) = (Vec::new(), Vec::new(), Vec::new());
        match self.kind {
            WoodJointKind::Dowels => {
                if self.diameter <= 0.0 || self.depth <= 0.0 || self.depth_b <= 0.0 || self.fastener_count() == 0 || !self.spots_fit(c, self.diameter / 2.0) {
                    return Err(bad());
                }
                for s in self.spots(c) {
                    let hole = cyl(point(s) - n * self.depth_b, self.depth + self.depth_b, self.diameter / 2.0)?;
                    from_a.push(hole.clone());
                    from_b.push(hole);
                }
            }
            WoodJointKind::Screws => {
                if self.diameter <= 0.0 || self.depth_b <= 0.0 || self.fastener_count() == 0 || !self.spots_fit(c, self.diameter / 2.0) {
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
                if self.thickness <= 0.0 || self.thickness / 2.0 + self.offset.abs() >= c.width() / 2.0 || len <= 0.0 || len > c.a_size + 1e-9 || 2.0 * self.shoulder >= c.length() {
                    return Err(bad());
                }
                let (mut lo, mut hi) = (c.lo, c.hi);
                let mid = (c.lo[c.across] + c.hi[c.across]) * 0.5 + self.offset;
                (lo[c.across], hi[c.across]) = (mid - self.thickness / 2.0, mid + self.thickness / 2.0);
                lo[c.along] += self.shoulder;
                hi[c.along] -= self.shoulder;
                to_b.push(cube(lo, hi, -OVER, len)?);
                let (lo, hi) = grow(lo, hi, &[c.across, c.along], self.clearance);
                let end = if self.through { c.a_size + OVER } else { len + self.clearance };
                from_a.push(cube(lo, hi, -OVER, end)?);
            }
            WoodJointKind::BoxJoint => {
                let slots = 2 * self.count.max(1) - 1;
                let w = c.length() / slots as f64;
                if self.count == 0 || w <= 2.0 * self.clearance + 1.0 {
                    return Err(bad());
                }
                for i in (0..slots).step_by(2) {
                    let (mut lo, mut hi) = (c.lo, c.hi);
                    (lo[c.along], hi[c.along]) = (c.lo[c.along] + w * i as f64, c.lo[c.along] + w * (i + 1) as f64);
                    to_b.push(cube(lo, hi, -OVER, c.a_size)?);
                    let (lo, hi) = grow(lo, hi, &[c.along], self.clearance);
                    let (lo, hi) = open(lo, hi);
                    from_a.push(cube(lo, hi, -OVER, c.a_size + OVER)?);
                }
            }
            WoodJointKind::Dovetail => {
                let n = self.count;
                let slope = self.angle.to_radians().tan();
                // tails' width on a's outer face, and at the shoulder line
                let wide = (c.length() - (n + 1) as f64 * self.pin) / n.max(1) as f64;
                let narrow = wide - 2.0 * c.a_size * slope;
                if n == 0 || !(0.5..=30.0).contains(&self.angle) || self.pin < 1.0 || narrow < 2.0 * self.clearance + 1.0 {
                    return Err(bad());
                }
                // a tail from depth `d0` to `d1` (along n), grown along the joint by `g`, across
                // it to `lo..hi`
                let tail = |i: usize, d0: f64, d1: f64, g: f64, (lo, hi): (DVec3, DVec3)| {
                    let mid = c.lo[c.along] + self.pin + wide / 2.0 + i as f64 * (wide + self.pin);
                    hexahedron(|u, v, k| {
                        let d = if v == 0 { d0 } else { d1 };
                        let half = (narrow / 2.0 + d * slope + g) * if u == 0 { -1.0 } else { 1.0 };
                        let mut p = c.lo;
                        p[c.axis] = c.at + c.into_a * d;
                        p[c.along] = mid + half;
                        p[c.across] = if k == 0 { lo[c.across] } else { hi[c.across] };
                        p
                    })
                    .map(|m| (m, DMat4::IDENTITY))
                    .ok_or_else(bad)
                };
                for i in 0..n {
                    to_b.push(tail(i, -OVER, c.a_size, 0.0, (c.lo, c.hi))?);
                    from_a.push(tail(i, -OVER, c.a_size + OVER, self.clearance, open(c.lo, c.hi))?);
                }
            }
            WoodJointKind::PocketHole => {
                let t = c.width();
                let (sin, cos) = POCKET_ANGLE.to_radians().sin_cos();
                // the hole runs from b's face towards its end and on into a
                let mut m = DVec3::ZERO;
                m[c.across] = if self.flip { -1.0 } else { 1.0 };
                let dir = n * cos + m * sin;
                let enter = t / 2.0 / sin;
                let pocket_r = (2.0 * self.diameter + 1.5).max(9.5) / 2.0;
                if self.diameter <= 0.0 || self.depth <= 0.0 || self.fastener_count() == 0 || 2.0 * pocket_r >= c.length() || enter < POCKET_KEEP + 3.0 || self.depth >= c.a_size || !self.spots_fit(c, 0.0) {
                    return Err(bad());
                }
                for s in self.spots(c) {
                    let e = point(s);
                    let drill = |from: f64, to: f64, r: f64| cyl_to(e + dir * from, dir, to - from, r);
                    from_b.push(drill(-enter - 10.0, -POCKET_KEEP, pocket_r)?);
                    from_b.push(drill(-POCKET_KEEP - OVER, OVER, (self.diameter + 0.5) / 2.0)?);
                    from_a.push(drill(-OVER, self.depth, self.diameter * 0.35)?);
                }
            }
            WoodJointKind::Confirmat => {
                let into_b = self.size - c.a_size;
                if self.diameter <= 0.0 || into_b < 10.0 || self.fastener_count() == 0 || !self.spots_fit(c, self.diameter / 2.0) {
                    return Err(bad());
                }
                for s in self.spots(c) {
                    from_a.push(cyl(point(s) - n * OVER, c.a_size + 2.0 * OVER, (self.diameter + 0.5) / 2.0)?);
                    from_b.push(cyl(point(s) - n * into_b, into_b + OVER, 2.5)?);
                }
            }
            WoodJointKind::CamLock => {
                if self.diameter <= 0.0 || c.width() < 16.0 || self.size < self.diameter || c.b_size < self.size + self.diameter / 2.0 + 3.0 || self.fastener_count() == 0 || !self.spots_fit(c, self.diameter / 2.0) {
                    return Err(bad());
                }
                for s in self.spots(c) {
                    from_a.push(cyl(point(s) - n * OVER, 11.0 + OVER, 2.5)?);
                    from_b.push(cyl_to(point(s) + n * OVER, -n, self.size + OVER, 4.0)?);
                    let (p, dir) = self.cam(c, s);
                    from_b.push(cyl_to(p - dir * OVER, dir, 13.0 + OVER, self.diameter / 2.0)?);
                }
            }
            WoodJointKind::ShelfPins => {
                let holes = self.pin_holes(c);
                let r = self.diameter / 2.0;
                let inside = |p: &DVec3| (c.a_min[c.across] + r - 1e-6..=c.a_max[c.across] - r + 1e-6).contains(&p[c.across]);
                if r <= 0.0 || self.depth <= 0.0 || self.depth >= c.a_size || self.fastener_count() == 0 || !self.spots_fit(c, 0.0) || !holes.iter().all(inside) {
                    return Err(bad());
                }
                for p in holes {
                    from_a.push(cyl(p - n * OVER, self.depth + OVER, r)?);
                }
            }
            WoodJointKind::Hinge => {
                let door = if self.door_is_a() { c.a_size } else { c.width() };
                let r = self.diameter / 2.0;
                if r <= 0.0 || self.depth <= 0.0 || door < self.depth + 3.0 || self.fastener_count() == 0 || !self.spots_fit(c, 0.0) || self.spots(c).iter().any(|&s| s - r < c.lo[c.along] || s + r > c.hi[c.along]) {
                    return Err(bad());
                }
                for s in self.spots(c) {
                    let (p, dir) = self.hinge_cup(c, s);
                    let cup = cyl_to(p - dir * OVER, dir, self.depth + OVER, r)?;
                    if self.door_is_a() { &mut from_a } else { &mut from_b }.push(cup);
                }
            }
            WoodJointKind::GasStrut => {
                // an overlay door (its back on the side's edge) tall enough, a side deep enough
                let door = c.a_max - c.a_min;
                let thin = door[c.axis] <= door[c.along].min(door[c.across]);
                if self.size <= 0.0 || !thin || c.length() < 150.0 || (c.b_max - c.b_min)[c.axis] < 100.0 {
                    return Err(OpError::Invalid(tr!("joint-err-gas-strut")));
                }
                let (sa, sb) = self.strut_screws(c);
                let (_, _, inward) = self.strut(c);
                for p in sa {
                    from_a.push(cyl(p - n * OVER, 10.0 + OVER, 1.5)?);
                }
                for p in sb {
                    from_b.push(cyl_to(p + inward * OVER, -inward, 10.0 + OVER, 1.5)?);
                }
            }
            WoodJointKind::Slide => {
                if (c.gap - self.slide.gap()).abs() > SLIDE_GAP_TOL {
                    return Err(OpError::Invalid(tr!("joint-err-slide-gap", gap = num(self.slide.gap()), now = num(c.gap))));
                }
                if self.slide_length(c).is_none() {
                    return Err(OpError::Invalid(tr!("joint-err-slide-length")));
                }
            }
            WoodJointKind::Handle | WoodJointKind::Knob => {
                if self.fastener_count() == 0 || !self.spots_fit(c, self.diameter / 2.0) || (self.kind == WoodJointKind::Handle && (self.size < 16.0 || self.spots(c).iter().any(|&s| s - self.size / 2.0 - 15.0 < c.lo[c.along] || s + self.size / 2.0 + 15.0 > c.hi[c.along]))) {
                    return Err(bad());
                }
                for s in self.spots(c) {
                    for p in self.handle_points(c, point(s)) {
                        from_a.push(cyl(p - n * OVER, c.a_size + 2.0 * OVER, 2.5)?);
                    }
                }
            }
            WoodJointKind::Leg => {
                let r = self.diameter / 2.0;
                if r <= 0.0 || self.size <= 0.0 || c.a_size <= 12.0 || self.margin < r || 2.0 * self.margin >= c.length().min(c.width()) {
                    return Err(bad());
                }
                for p in self.leg_points(c) {
                    from_a.push(cyl(p - n * OVER, 10.0 + OVER, 4.0)?);
                }
            }
            WoodJointKind::Caster => {
                // its plate's four screws
                let r = self.diameter / 2.0;
                if r <= 0.0 || self.size <= self.diameter || c.a_size <= 12.0 || self.margin < 25.0 || 2.0 * self.margin >= c.length().min(c.width()) {
                    return Err(bad());
                }
                for p in self.leg_points(c).into_iter().flat_map(caster_screws) {
                    from_a.push(cyl(p - n * OVER, 12.0 + OVER, 1.5)?);
                }
            }
            WoodJointKind::TableTop => {
                let r = self.diameter / 2.0;
                if r <= 0.0 || c.width() < 16.0 || self.fastener_count() == 0 || !self.spots_fit(c, 0.0) || self.spots(c).iter().any(|&s| s - r < c.lo[c.along] || s + r > c.hi[c.along]) {
                    return Err(bad());
                }
                let m = unit(c.across, if self.flip { -1.0 } else { 1.0 });
                for s in self.spots(c) {
                    for p in [point(s), point(s) + m * (r * 1.7)] {
                        from_b.push(cyl_to(p + n * OVER, -n, 3.0 + OVER, r + 0.5)?);
                    }
                }
            }
            WoodJointKind::Nails => {
                if self.diameter <= 0.0 || self.size < c.a_size + 5.0 || self.fastener_count() == 0 || !self.spots_fit(c, self.diameter / 2.0) {
                    return Err(bad());
                }
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

    /// Each joint's warnings ([`WoodJoint::warnings`]), and holes of two joints that meet in a
    /// part (two shelves' dowels from either side of a partition, a confirmat across a dowel).
    pub fn joint_warnings(&self) -> Vec<Vec<String>> {
        let mut out: Vec<Vec<String>> = self.joints.iter().enumerate().map(|(i, j)| self.joint_contacts.get(i).copied().flatten().map(|c| j.warnings(&c)).unwrap_or_default()).collect();
        // the holes in each part: (joint, start, end, radius)
        type Segments = Vec<(usize, DVec3, DVec3, f64)>;
        let mut parts: Vec<(&ContextPath, Segments)> = Vec::new();
        for (i, j) in self.joints.iter().enumerate() {
            let Some(c) = self.joint_contacts.get(i).copied().flatten() else { continue };
            let (a, b) = j.bores(&c);
            for (path, bores) in [(&j.a, a), (&j.b, b)] {
                let k = parts.iter().position(|(p, _)| *p == path).unwrap_or_else(|| {
                    parts.push((path, Vec::new()));
                    parts.len() - 1
                });
                parts[k].1.extend(bores.into_iter().filter(|b| b.3 > 0.0).map(|(p, dir, d, depth)| (i, p, p + dir.normalize_or_zero() * depth, d / 2.0)));
            }
        }
        let mut met: Vec<(usize, usize)> = Vec::new();
        for (_, holes) in &parts {
            for (x, &(i, p0, p1, r0)) in holes.iter().enumerate() {
                for &(k, q0, q1, r1) in &holes[x + 1..] {
                    if i != k && !met.contains(&(i.min(k), i.max(k))) && segment_distance(p0, p1, q0, q1) < r0 + r1 - 0.5 {
                        met.push((i.min(k), i.max(k)));
                    }
                }
            }
        }
        for (i, k) in met {
            out[i].push(tr!("joint-warn-meet", other = (k + 1).to_string()));
            out[k].push(tr!("joint-warn-meet", other = (i + 1).to_string()));
        }
        out
    }

    /// Join parts `p` and `q` (instance paths) with a joint of `kind` sized for them; the one
    /// whose touching face is smaller becomes `b`. A fitting on one part (`kind.single()`)
    /// takes `p` only. Returns the joint's index.
    pub fn add_wood_joint(&mut self, kind: WoodJointKind, p: ContextPath, q: ContextPath) -> OpResult<usize> {
        if kind.single() {
            if self.part_def(&p).is_none() {
                return Err(OpError::Invalid(tr!("joint-err-one-part")));
            }
            self.unique_parts(&[&p])?;
            let c = face_contact(&self.unjoined(), &p, kind.under(), false).ok_or_else(|| OpError::Invalid(tr!("joint-err-one-part")))?;
            return self.push_wood_joint(WoodJoint::new(kind, p, Vec::new(), &c));
        }
        if p == q || self.part_def(&p).is_none() || self.part_def(&q).is_none() {
            return Err(OpError::Invalid(tr!("joint-err-two-parts")));
        }
        // sized from the parts before any joint was cut
        let (a, b, c) = {
            let m = self.unjoined();
            let c = kind_contact(&m, kind, &p, &q, false).ok_or_else(|| OpError::Invalid(tr!("joint-err-no-contact")))?;
            let (a, b) = if face_area(&m, &q, c.axis) <= face_area(&m, &p, c.axis) { (p, q) } else { (q, p) };
            let c = kind_contact(&m, kind, &a, &b, false).unwrap_or(c);
            (a, b, c)
        };
        self.unique_parts(&[&a, &b])?;
        self.push_wood_joint(WoodJoint::new(kind, a, b, &c))
    }

    /// A copy of a component would cut every copy: the parts to join get their own definition,
    /// from their shape before any joint, and the joints are cut again.
    fn unique_parts(&mut self, paths: &[&ContextPath]) -> OpResult<()> {
        let shared = |m: &Model, path: &ContextPath| path.split_last().is_some_and(|(&id, parent)| m.can_make_unique(parent, id));
        if !paths.iter().any(|p| shared(self, p)) {
            return Ok(());
        }
        self.restore_joint_bases();
        let mut r = Ok(());
        for path in paths {
            if let Some((&id, parent)) = path.split_last() {
                r = r.and_then(|_| self.make_unique(parent, id));
            }
        }
        self.rebuild_wood_joints();
        r
    }

    /// Add joint `j` and cut everything again; taken back if it cannot be cut.
    pub(crate) fn push_wood_joint(&mut self, j: WoodJoint) -> OpResult<usize> {
        // the joints before it are cut already, in order: cutting only the new one gives what
        // a rebuild would (a joint that fails changes nothing)
        self.cut_wood_joint(&j)?;
        self.joints.push(j);
        self.refresh_fasteners();
        Ok(self.joints.len() - 1)
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
        if kind.single() != old.kind.single() {
            return Err(OpError::Invalid(tr!(if kind.single() { "joint-err-one-part" } else { "joint-err-two-parts" })));
        }
        self.restore_joint_bases();
        let c = kind_contact(self, kind, &old.a, &old.b, old.flip && kind.single());
        self.rebuild_wood_joints();
        let c = c.ok_or_else(|| OpError::Invalid(tr!("joint-err-no-contact")))?;
        self.set_wood_joint(i, WoodJoint::new(kind, old.a, old.b, &c))
    }

    /// Delete joint `i`; parts left in no joint get their geometry back, assembly steps forget
    /// it (a fastener step left empty goes).
    pub fn remove_wood_joint(&mut self, i: usize) {
        if i < self.joints.len() {
            self.joints.remove(i);
            for s in &mut self.assembly.steps {
                s.joints.retain(|&j| j != i);
                s.joints.iter_mut().filter(|j| **j > i).for_each(|j| *j -= 1);
            }
            self.assembly.steps.retain(|s| !s.parts.is_empty() || !s.joints.is_empty());
            self.rebuild_wood_joints();
        }
    }

    /// The model with its parts as they were before any joint was cut (tenons and rabbets make
    /// bounding boxes overlap, so parts no longer seem to touch); borrowed when there are none.
    pub fn unjoined(&self) -> std::borrow::Cow<'_, Model> {
        if self.joint_bases.is_empty() {
            return std::borrow::Cow::Borrowed(self);
        }
        let mut m = self.clone();
        m.restore_joint_bases();
        std::borrow::Cow::Owned(m)
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
        self.refresh_fasteners();
        errors
    }

    fn cut_wood_joint(&mut self, j: &WoodJoint) -> OpResult<()> {
        let missing = || OpError::Invalid(tr!("joint-err-missing"));
        let da = self.part_def(&j.a).ok_or_else(missing)?;
        let db = if j.kind.single() { None } else { Some(self.part_def(&j.b).ok_or_else(missing)?) };
        let c = joint_contact(self, j).ok_or_else(|| OpError::Invalid(tr!("joint-err-no-contact")))?;
        let (from_a, from_b, to_b) = j.cuts(&c)?;
        let mut out = Vec::new();
        for (path, d, ops) in [(&j.a, Some(da), vec![(&from_a, false)]), (&j.b, db, vec![(&to_b, true), (&from_b, false)])] {
            // fittings that drill nothing into a part leave it as it is
            let Some(d) = d.filter(|_| ops.iter().any(|(t, _)| !t.is_empty())) else { continue };
            let xf = self.assembled_transform(path);
            let mut mesh = self.defs[d].mesh.clone();
            if !mesh.is_solid() {
                return Err(OpError::Invalid(tr!("joint-err-not-solid", name = self.defs[d].name.clone())));
            }
            for (tools, add) in ops {
                for (t, txf) in &merge_tools(tools) {
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
