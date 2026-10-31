//! Fittings (track W): handles, knobs, legs, casters, and any other bought object put on a
//! part.
//!
//! A fitting is a group or component whose definition has a [`Fitting`]: what it is called on
//! the shopping list and the holes it needs, in its own coordinates — its origin on the face of
//! the part it goes on, +Z out of that face, so its holes go down along −Z. Any model can be one
//! (a handle downloaded or drawn); [`Builtin`] makes the usual ones.
//!
//! A copy of it mounted on a part ([`crate::Instance::mount`], [`Mount`]) stays on the same
//! face at the same place when the part moves or changes size: by a corner, the middle of an
//! edge or the middle of the face ([`Mount::at`]), and an offset from there. Moved by hand, it
//! keeps the face and its mount follows ([`Model::place_fittings`]). It drills its holes into
//! the part ([`Model::rebuild_wood_joints`]) and is an object of the model like any other:
//! picked, copied, painted, added in an assembly step; in the cut list it is a line of hardware.
//!
//! The mount names the part by its path from the fitting's own container, so a fitting and its
//! part go together in groups and components (a door with its handle copied as one).

use crate::model::{ContextPath, Definition, Model};
use crate::primitives::Primitive;
use crate::{DefId, Instance, InstanceId, Mesh, OpError, OpResult};
use glam::{DMat4, DQuat, DVec3, DVec4};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use varstotas_i18n::tr;

/// What makes a definition a fitting.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Fitting {
    /// What it is on the shopping list ("Handle, centres 128"); empty: the definition's name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub item: String,
    /// The holes it drills into the part it is mounted on.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub holes: Vec<FittingHole>,
}

/// A hole a fitting needs, drilled along −Z (into the part).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FittingHole {
    /// Where it starts, in the fitting's coordinates (on the face: z = 0).
    pub at: DVec3,
    pub diameter: f64,
    /// How deep it goes into the part; 0: right through it.
    #[serde(default)]
    pub depth: f64,
}

/// Where a fitting is on a part.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Mount {
    /// The part: its instance path from the definition the fitting is in.
    pub on: ContextPath,
    /// The face: across the part's own axis `axis` (0 = x), on its high side or its low one.
    pub axis: usize,
    pub high: bool,
    /// From the face's corner (0), the middle (½) or the far end (1), along its two directions…
    pub at: [f64; 2],
    /// …by this much (mm).
    pub offset: [f64; 2],
    /// Off the face (mm).
    #[serde(skip_serializing_if = "is_zero")]
    pub lift: f64,
    /// Turned about the face's normal from its first direction (degrees).
    #[serde(skip_serializing_if = "is_zero")]
    pub turn: f64,
    /// The face as last placed from ([`Face::key`]), to tell the part moving from the fitting
    /// moving; not saved.
    #[serde(skip)]
    pub seen: Option<DMat4>,
}

/// A hole's cutter in a part definition's coordinates: where it starts, its direction (into
/// the part), its diameter and depth.
pub type Cutter = (DVec3, DVec3, f64, f64);

fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

/// A face of a part's box, in the coordinates of the fitting's container: its corner, its two
/// directions (unit, `u × v = n`), its outward normal and its size along them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Face {
    pub origin: DVec3,
    pub u: DVec3,
    pub v: DVec3,
    pub n: DVec3,
    pub size: [f64; 2],
}

impl Face {
    /// Face (`axis`, `high`) of the box `lo..hi` placed by `xf`.
    pub fn of_box(lo: DVec3, hi: DVec3, xf: &DMat4, axis: usize, high: bool) -> Face {
        let (a, b) = ((axis + 1) % 3, (axis + 2) % 3);
        let (ua, va) = if high { (a, b) } else { (b, a) };
        let mut o = lo;
        o[axis] = if high { hi[axis] } else { lo[axis] };
        let (mut pu, mut pv) = (o, o);
        pu[ua] = hi[ua];
        pv[va] = hi[va];
        let w = |p: DVec3| xf.transform_point3(p);
        let (wo, du, dv) = (w(o), w(pu) - w(o), w(pv) - w(o));
        let mut out = DVec3::ZERO;
        out[axis] = if high { 1.0 } else { -1.0 };
        // the normal through the inverse transpose: right for a mirrored part too
        let n = xf.inverse().transpose().transform_vector3(out).normalize_or_zero();
        let u = (du - n * du.dot(n)).normalize_or_zero();
        let v = n.cross(u);
        Face { origin: wo, u, v, n, size: [du.dot(u).abs(), dv.dot(v).abs()] }
    }

    /// The face as one matrix, for telling whether it changed.
    pub fn key(&self) -> DMat4 {
        DMat4::from_cols((self.u * self.size[0]).extend(0.0), (self.v * self.size[1]).extend(0.0), self.n.extend(0.0), self.origin.extend(1.0))
    }
}

fn close(a: &DMat4, b: &DMat4) -> bool {
    a.abs_diff_eq(*b, 1e-6)
}

/// The scale along a transform's own axes.
fn scale_of(m: &DMat4) -> DVec3 {
    DVec3::new(m.x_axis.truncate().length(), m.y_axis.truncate().length(), m.z_axis.truncate().length())
}

impl Mount {
    /// Where the fitting goes on `face` (in its container), at `scale` along its own axes.
    pub fn place(&self, face: &Face, scale: DVec3) -> DMat4 {
        let p = face.origin + face.u * (self.at[0] * face.size[0] + self.offset[0]) + face.v * (self.at[1] * face.size[1] + self.offset[1]) + face.n * self.lift;
        let (s, c) = self.turn.to_radians().sin_cos();
        let x = face.u * c + face.v * s;
        let y = face.n.cross(x);
        DMat4::from_cols(x.extend(0.0), y.extend(0.0), face.n.extend(0.0), DVec4::from((p, 1.0))) * DMat4::from_scale(scale)
    }

    /// Take the place of a fitting at `xf` (in its container) on `face`: by the nearest of the
    /// face's corner, middle and far end along each direction.
    pub fn follow(&mut self, face: &Face, xf: &DMat4) {
        let p = xf.w_axis.truncate() - face.origin;
        for k in 0..2 {
            let (dir, size) = ([face.u, face.v][k], face.size[k]);
            let t = p.dot(dir);
            let a = if size > 1e-9 { [0.0, 0.5, 1.0].into_iter().min_by(|x: &f64, y: &f64| (t - x * size).abs().total_cmp(&(t - y * size).abs())).unwrap_or(0.0) } else { 0.0 };
            self.at[k] = a;
            self.offset[k] = round(t - a * size);
        }
        self.lift = round(p.dot(face.n));
        let x = xf.x_axis.truncate();
        self.turn = round(x.dot(face.v).atan2(x.dot(face.u)).to_degrees());
        if self.turn == -180.0 {
            self.turn = 180.0;
        }
    }
}

/// To a thousandth: no noise from transforms in the numbers shown.
fn round(v: f64) -> f64 {
    let r = (v * 1e3).round() / 1e3;
    if r == 0.0 { 0.0 } else { r }
}

/// The usual fittings, made on demand ([`Model::builtin_fitting`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Builtin {
    /// A bar handle on two posts `centres` apart.
    Handle { centres: f64 },
    Knob { diameter: f64 },
    Leg { height: f64, diameter: f64 },
    /// A swivel caster: the wheel's diameter, from the floor to its plate.
    Caster { wheel: f64, height: f64 },
}

/// A size in mm without trailing zeros.
fn num(v: f64) -> String {
    let s = format!("{v:.1}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// What a built-in fitting is made of: a shape, where it is, its metal.
type Piece = (Primitive, DMat4, Metal);

#[derive(Clone, Copy, Debug, PartialEq)]
enum Metal {
    Steel,
    Brass,
    Zinc,
    Dark,
}

impl Metal {
    fn material(self) -> (String, [f32; 4]) {
        let (key, c) = match self {
            Metal::Steel => ("fitting-mat-steel", crate::fasteners::STEEL),
            Metal::Brass => ("fitting-mat-brass", crate::fasteners::BRASS),
            Metal::Zinc => ("fitting-mat-zinc", crate::fasteners::ZINC),
            Metal::Dark => ("fitting-mat-dark", crate::fasteners::DARK),
        };
        (varstotas_i18n::tr(key), c)
    }
}

impl Builtin {
    /// Its sizes are sensible.
    pub fn is_valid(&self) -> bool {
        let ok = |v: f64| v.is_finite() && v > 0.0 && v < 10_000.0;
        match *self {
            Builtin::Handle { centres } => ok(centres) && centres >= 16.0,
            Builtin::Knob { diameter } => ok(diameter),
            Builtin::Leg { height, diameter } => ok(height) && ok(diameter),
            Builtin::Caster { wheel, height } => ok(wheel) && ok(height) && height > wheel,
        }
    }

    /// Its line on the shopping list, also its definition's name.
    pub fn item(&self) -> String {
        match *self {
            Builtin::Handle { centres } => tr!("joint-hw-handle", length = num(centres)),
            Builtin::Knob { diameter } => tr!("joint-hw-knob", diameter = num(diameter)),
            Builtin::Leg { height, .. } => tr!("joint-hw-leg", length = num(height)),
            Builtin::Caster { wheel, height } => tr!("joint-hw-caster", diameter = num(wheel), length = num(height)),
        }
    }

    pub fn holes(&self) -> Vec<FittingHole> {
        let hole = |x: f64, y: f64, diameter: f64, depth: f64| FittingHole { at: DVec3::new(x, y, 0.0), diameter, depth };
        match *self {
            Builtin::Handle { centres } => vec![hole(-centres / 2.0, 0.0, 5.0, 0.0), hole(centres / 2.0, 0.0, 5.0, 0.0)],
            Builtin::Knob { .. } => vec![hole(0.0, 0.0, 5.0, 0.0)],
            Builtin::Leg { .. } => vec![hole(0.0, 0.0, 8.0, 10.0)],
            Builtin::Caster { .. } => [(-18.0, -18.0), (18.0, -18.0), (-18.0, 18.0), (18.0, 18.0)].map(|(x, y)| hole(x, y, 3.0, 12.0)).to_vec(),
        }
    }

    fn pieces(&self) -> Vec<Piece> {
        let at = |x: f64, y: f64, z: f64| DMat4::from_translation(DVec3::new(x, y, z));
        let along_x = DMat4::from_quat(DQuat::from_rotation_arc(DVec3::Z, DVec3::X));
        let cyl = |r: f64, h: f64| Primitive::Cylinder { r, h, segs: 24 };
        match *self {
            Builtin::Handle { centres } => {
                let (r, len) = (6.0, centres + 30.0);
                vec![
                    (cyl(4.5, 28.0), at(-centres / 2.0, 0.0, 0.0), Metal::Steel),
                    (cyl(4.5, 28.0), at(centres / 2.0, 0.0, 0.0), Metal::Steel),
                    (cyl(r, len), at(-len / 2.0, 0.0, 28.0 + r) * along_x, Metal::Steel),
                ]
            }
            Builtin::Knob { diameter } => vec![(cyl(4.5, 18.0), DMat4::IDENTITY, Metal::Brass), (cyl(diameter / 2.0, 14.0), at(0.0, 0.0, 18.0), Metal::Brass)],
            Builtin::Leg { height, diameter } => vec![(cyl(diameter / 2.0, height), DMat4::IDENTITY, Metal::Dark)],
            Builtin::Caster { wheel, height } => {
                let r = wheel / 2.0;
                let fork = (height - r - 3.0).max(1.0);
                vec![
                    (Primitive::Box { w: 50.0, d: 50.0, h: 3.0 }, at(-25.0, -25.0, 0.0), Metal::Zinc),
                    (Primitive::Box { w: 28.0, d: r * 0.9, h: fork }, at(-14.0, -r * 0.45, 3.0), Metal::Zinc),
                    (cyl(r, 22.0), at(-11.0, 0.0, height - r) * along_x, Metal::Dark),
                ]
            }
        }
    }
}

impl Model {
    /// The fitting of definition `d`, if it is one.
    pub fn fitting_of(&self, d: DefId) -> Option<&Fitting> {
        self.defs.get(d)?.fitting.as_ref()
    }

    /// What fitting definition `d` is on the shopping list.
    pub fn fitting_item(&self, d: DefId) -> Option<String> {
        let def = self.defs.get(d)?;
        let f = def.fitting.as_ref()?;
        Some(if f.item.trim().is_empty() { def.name.clone() } else { f.item.clone() })
    }

    /// The definition of built-in fitting `b`: the model's own when there is one (by its name),
    /// else made — a component of plain metal pieces.
    pub fn builtin_fitting(&mut self, b: Builtin) -> OpResult<DefId> {
        if !b.is_valid() {
            return Err(OpError::Invalid(tr!("joint-err-size")));
        }
        let item = b.item();
        if let Some((d, _)) = self.defs.iter().find(|(_, x)| !x.is_group && x.name == item && x.fitting.is_some()) {
            return Ok(d);
        }
        let mut mesh = Mesh::new();
        for (shape, xf, metal) in b.pieces() {
            let piece = shape.mesh().ok_or_else(|| OpError::Invalid(tr!("joint-err-size")))?;
            let (name, color) = metal.material();
            let mat = match self.materials.iter().find(|(_, m)| m.name == name) {
                Some((id, _)) => id,
                None => self.add_material(&name, color),
            };
            let pd = self.defs.insert(Definition { name: item.clone(), mesh: piece, is_group: true, ..Definition::empty() });
            mesh.instances.insert(Instance { def: pd, transform: xf, name: String::new(), material: Some(mat), tag: None, hidden: false, locked: false, mount: None });
        }
        let name = self.free_def_name(&item);
        Ok(self.defs.insert(Definition { name, mesh, is_group: false, fitting: Some(Fitting { item, holes: b.holes() }), ..Definition::empty() }))
    }

    /// The first place each definition is used at (the root at the empty path).
    pub(crate) fn first_paths(&self) -> HashMap<DefId, ContextPath> {
        fn walk(m: &Model, d: DefId, path: &mut ContextPath, out: &mut HashMap<DefId, ContextPath>, depth: usize) {
            let Some(def) = m.defs.get(d).filter(|_| depth < 64) else { return };
            for (id, inst) in &def.mesh.instances {
                path.push(id);
                if let std::collections::hash_map::Entry::Vacant(e) = out.entry(inst.def) {
                    e.insert(path.clone());
                    walk(m, inst.def, path, out, depth + 1);
                }
                path.pop();
            }
        }
        let mut out = HashMap::new();
        out.insert(self.root, Vec::new());
        walk(self, self.root, &mut Vec::new(), &mut out, 0);
        out
    }

    /// The box of part definition `d` from before its joints and holes.
    fn part_def_box(&self, d: DefId) -> Option<(DVec3, DVec3)> {
        let b = match self.joint_bases.iter().find(|(x, _)| *x == d) {
            Some((_, m)) => m.bounds(),
            None => self.def_bounds(d),
        }?;
        Some((b.min, b.max))
    }

    /// Transform of `path` (from the container placed at `prefix`) relative to that container,
    /// as assembled.
    fn rel_assembled(&self, prefix: &[InstanceId], path: &[InstanceId]) -> DMat4 {
        let full: ContextPath = prefix.iter().chain(path).copied().collect();
        self.assembled_transform(prefix).inverse() * self.assembled_transform(&full)
    }

    /// Face (`axis`, `high`) of the part at `on` from the container placed at `prefix`, in that
    /// container's coordinates, as assembled.
    pub fn mount_face(&self, prefix: &[InstanceId], on: &[InstanceId], axis: usize, high: bool) -> Option<Face> {
        let full: ContextPath = prefix.iter().chain(on).copied().collect();
        let (lo, hi) = self.part_def_box(self.def_at(&full)?)?;
        Some(Face::of_box(lo, hi, &self.rel_assembled(prefix, on), axis, high))
    }

    /// Mount the fitting at `path` on the part at `part` (both from the root, in one container
    /// or the part deeper in it), on the face of the part facing the way the fitting's +Z does
    /// (the nearest one when none does), where it is now.
    pub fn mount_fitting(&mut self, path: &[InstanceId], part: &[InstanceId]) -> OpResult<()> {
        self.attach_fitting(path, part)?;
        self.sync_joints();
        Ok(())
    }

    /// [`Model::mount_fitting`] without placing it or drilling its holes yet.
    pub(crate) fn attach_fitting(&mut self, path: &[InstanceId], part: &[InstanceId]) -> OpResult<()> {
        let bad = || OpError::Invalid(tr!("fitting-err-mount"));
        let (&id, prefix) = path.split_last().ok_or_else(bad)?;
        if self.def_at(path).and_then(|d| self.fitting_of(d)).is_none() {
            return Err(OpError::Invalid(tr!("fitting-err-not-fitting")));
        }
        let on = part.strip_prefix(prefix).filter(|r| !r.is_empty() && r[0] != id).ok_or_else(bad)?.to_vec();
        let xf = self.rel_assembled(prefix, &[id]);
        let (z, p) = (xf.z_axis.truncate().normalize_or_zero(), xf.w_axis.truncate());
        let faces: Vec<(usize, bool, Face)> = (0..3).flat_map(|axis| [false, true].map(|high| (axis, high))).filter_map(|(axis, high)| Some((axis, high, self.mount_face(prefix, &on, axis, high)?))).collect();
        // facing its way, then the nearest
        let facing = |f: &Face| f.n.dot(z) > 0.9;
        let dist = |f: &Face| (p - f.origin).dot(f.n).abs();
        let (axis, high, face) = faces.iter().filter(|(_, _, f)| facing(f)).min_by(|a, b| dist(&a.2).total_cmp(&dist(&b.2))).or_else(|| faces.iter().min_by(|a, b| dist(&a.2).total_cmp(&dist(&b.2)))).copied().ok_or_else(bad)?;
        let mut m = Mount { on, axis, high, ..Default::default() };
        if facing(&face) {
            m.follow(&face, &xf);
        } else {
            // turned to face out of it, where it is above the face
            let q = p - face.n * (p - face.origin).dot(face.n);
            m.follow(&face, &DMat4::from_cols(face.u.extend(0.0), face.v.extend(0.0), face.n.extend(0.0), q.extend(1.0)));
            m.lift = 0.0;
        }
        let inst = self.mesh_at_mut(prefix).and_then(|x| x.instances.get_mut(id)).ok_or_else(bad)?;
        inst.mount = Some(m);
        Ok(())
    }

    /// The fitting at `path` comes off its part (stays where it is, no longer drills it).
    pub fn unmount_fitting(&mut self, path: &[InstanceId]) {
        let Some((&id, prefix)) = path.split_last() else { return };
        if let Some(inst) = self.mesh_at_mut(prefix).and_then(|x| x.instances.get_mut(id)) {
            inst.mount = None;
        }
        self.sync_joints();
    }

    /// Put fitting definition `d` on the part at `part` (from the root), beside it in its
    /// container, mounted on face (`axis`, `high`) at `at` + `offset`, turned `turn`. Returns
    /// its path.
    #[allow(clippy::too_many_arguments)]
    pub fn add_fitting(&mut self, d: DefId, part: &[InstanceId], axis: usize, high: bool, at: [f64; 2], offset: [f64; 2], turn: f64) -> OpResult<ContextPath> {
        let bad = || OpError::Invalid(tr!("fitting-err-mount"));
        if self.fitting_of(d).is_none() {
            return Err(OpError::Invalid(tr!("fitting-err-not-fitting")));
        }
        let (&pid, prefix) = part.split_last().ok_or_else(bad)?;
        let face = self.mount_face(prefix, &[pid], axis, high).ok_or_else(bad)?;
        let m = Mount { on: vec![pid], axis, high, at, offset, lift: 0.0, turn, seen: None };
        let rel = m.place(&face, DVec3::ONE);
        // the container's transform as assembled, and its parts' exploded offsets
        let transform = self.transform_at(prefix).inverse() * self.assembled_transform(prefix) * rel;
        let mesh = self.mesh_at_mut(prefix).ok_or_else(bad)?;
        let id = mesh.instances.insert(Instance { def: d, transform, name: String::new(), material: None, tag: None, hidden: false, locked: false, mount: Some(m) });
        self.sync_joints();
        Ok(prefix.iter().copied().chain(std::iter::once(id)).collect())
    }

    /// Bring the mounted fittings to their parts: a fitting whose part moved or changed size
    /// goes with it; one moved itself while its part stayed keeps its face, its mount following
    /// it. True when one moved.
    pub fn place_fittings(&mut self) -> bool {
        let firsts = self.first_paths();
        let mut set: Vec<(DefId, InstanceId, DMat4, Mount)> = Vec::new();
        for (d, def) in &self.defs {
            for (id, inst) in &def.mesh.instances {
                let Some(m) = &inst.mount else { continue };
                let Some(prefix) = firsts.get(&d) else { continue };
                let Some(face) = self.mount_face(prefix, &m.on, m.axis, m.high) else { continue };
                let now = self.rel_assembled(prefix, &[id]);
                let scale = scale_of(&inst.transform);
                let mut nm = m.clone();
                if m.seen.is_some_and(|s| close(&s, &face.key())) && !close(&now, &m.place(&face, scale)) {
                    nm.follow(&face, &now);
                }
                nm.seen = Some(face.key());
                let rel = nm.place(&face, scale);
                let full: ContextPath = prefix.iter().copied().chain(std::iter::once(id)).collect();
                // the instance's own transform: its exploded offset kept
                let transform = self.transform_at(prefix).inverse() * DMat4::from_translation(self.assembly_offset(&full)) * self.assembled_transform(prefix) * rel;
                if !close(&transform, &inst.transform) || nm != *m {
                    set.push((d, id, transform, nm));
                }
            }
        }
        let moved = set.iter().any(|s| !close(&s.2, &self.defs[s.0].mesh.instances[s.1].transform));
        for (d, id, transform, m) in set {
            if let Some(inst) = self.defs.get_mut(d).and_then(|x| x.mesh.instances.get_mut(id)) {
                inst.transform = transform;
                inst.mount = Some(m);
            }
        }
        moved
    }

    /// Every mounted fitting as placed: (its path, its part's path), from the root.
    pub fn placed_fittings(&self) -> Vec<(ContextPath, ContextPath)> {
        fn walk(m: &Model, d: DefId, path: &mut ContextPath, out: &mut Vec<(ContextPath, ContextPath)>, depth: usize) {
            let Some(def) = m.defs.get(d).filter(|_| depth < 64) else { return };
            for (id, inst) in &def.mesh.instances {
                if let Some(mt) = &inst.mount {
                    out.push((path.iter().copied().chain(std::iter::once(id)).collect(), path.iter().chain(&mt.on).copied().collect()));
                }
                path.push(id);
                walk(m, inst.def, path, out, depth + 1);
                path.pop();
            }
        }
        let mut out = Vec::new();
        walk(self, self.root, &mut Vec::new(), &mut out, 0);
        out
    }

    /// The holes of the mounted fittings by part, in world space as assembled: (start,
    /// direction, diameter, depth; 0 right through).
    pub fn fitting_holes(&self) -> Vec<(ContextPath, Vec<crate::joinery::Bore>)> {
        let mut out: Vec<(ContextPath, Vec<crate::joinery::Bore>)> = Vec::new();
        for (f, part) in self.placed_fittings() {
            let Some(fit) = self.def_at(&f).and_then(|d| self.fitting_of(d)) else { continue };
            if fit.holes.is_empty() || self.def_at(&part).is_none() {
                continue;
            }
            let xf = self.assembled_transform(&f);
            let down = xf.transform_vector3(DVec3::NEG_Z).normalize_or_zero();
            let holes = fit.holes.iter().map(|h| (xf.transform_point3(h.at), down, h.diameter, h.depth));
            match out.iter_mut().find(|(p, _)| *p == part) {
                Some(e) => e.1.extend(holes),
                None => out.push((part, holes.collect())),
            }
        }
        out
    }

    /// The cutters of the fittings' holes by part definition, in its own coordinates, once each
    /// (copies of a part share its definition).
    pub(crate) fn fitting_cutters(&self) -> Vec<(DefId, Vec<Cutter>)> {
        let mut out: Vec<(DefId, Vec<Cutter>)> = Vec::new();
        for (part, holes) in self.fitting_holes() {
            let Some(d) = self.def_at(&part) else { continue };
            let Some((lo, hi)) = self.part_def_box(d) else { continue };
            let inv = self.assembled_transform(&part).inverse();
            let through = (hi - lo).length() + 2.0;
            let e = match out.iter().position(|(x, _)| *x == d) {
                Some(k) => k,
                None => {
                    out.push((d, Vec::new()));
                    out.len() - 1
                }
            };
            for (p, dir, dia, depth) in holes {
                let (p, dir) = (inv.transform_point3(p), inv.transform_vector3(dir).normalize_or_zero());
                let len = if depth > 0.0 { depth } else { through };
                if !out[e].1.iter().any(|h| h.0.distance(p) < 1e-3 && h.1.dot(dir) > 0.999 && (h.2 - dia).abs() < 1e-6) {
                    out[e].1.push((p, dir, dia, len));
                }
            }
        }
        out
    }
}
