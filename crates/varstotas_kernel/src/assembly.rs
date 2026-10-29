//! Assembly (track W, `docs/WORKSHOP-PLAN.md`): the order a piece of furniture goes together
//! in, and the exploded view.
//!
//! The parts are those of the cut list ([`crate::cutlist::cut_parts`], every copy). Parts touch
//! where a face of one's bounding box lies on a face of another's ([`crate::joinery::contact`]);
//! that contact graph suggests the steps ([`suggest_steps`]): the base first (the lowest part
//! touching another, the one touching most of those), then, step by step, the lowest part touching what is
//! already placed, with its copies.
//!
//! The view ([`Model::set_assembly_view`]) is document state: the exploded view moves parts
//! apart (each away from the parts it touches, [`explode_offsets`]), and a step shown hides the
//! later steps' parts. What it did is kept ([`Assembly::moved`], [`Assembly::hid`]) and undone
//! before the next view, so joints, the cut list and new views work from the assembled model.

use crate::model::{ContextPath, Model};
use crate::InstanceId;
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};

/// One assembly step: the parts it adds (instance paths) and a note for the instructions.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Step {
    pub parts: Vec<ContextPath>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub note: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Assembly {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<Step>,
    /// How far the exploded view moves parts apart (mm); 0: assembled.
    pub explode: f64,
    /// The step shown (from 0): the parts of later steps are hidden.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<usize>,
    /// The offsets the exploded view gave parts (world, mm).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub moved: Vec<(ContextPath, DVec3)>,
    /// The parts hidden for the step shown.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hid: Vec<ContextPath>,
}

impl Assembly {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// The view: (explode, step).
    pub fn view(&self) -> SceneAssembly {
        SceneAssembly { explode: self.explode, step: self.step }
    }

    /// The step adding part `path`.
    pub fn step_of(&self, path: &[InstanceId]) -> Option<usize> {
        self.steps.iter().position(|s| s.parts.iter().any(|p| p == path))
    }
}

/// The assembly view a scene keeps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SceneAssembly {
    pub explode: f64,
    pub step: Option<usize>,
}

/// Every copy of every part, with the index of its part in [`crate::cutlist::cut_parts`].
pub fn parts(model: &Model) -> Vec<(ContextPath, usize)> {
    crate::cutlist::cut_parts(model).iter().enumerate().flat_map(|(i, p)| p.paths.iter().map(move |path| (path.clone(), i))).collect()
}

fn bounds(model: &Model, path: &[InstanceId]) -> Option<(DVec3, DVec3)> {
    let (&id, parent) = path.split_last()?;
    let b = model.instance_world_bounds(parent, id)?;
    let off = model.assembly_offset(path);
    Some((b.min - off, b.max - off))
}

/// Which parts touch which (indices into `paths`).
fn touching(model: &Model, paths: &[ContextPath]) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::new(); paths.len()];
    for i in 0..paths.len() {
        for j in i + 1..paths.len() {
            if crate::joinery::contact(model, &paths[i], &paths[j]).is_some() {
                adj[i].push(j);
                adj[j].push(i);
            }
        }
    }
    adj
}

/// The steps the contact graph suggests: the base, then the lowest part touching what is
/// placed with the other copies of its part that touch it too; parts touching nothing last,
/// lowest first.
pub fn suggest_steps(model: &Model) -> Vec<Step> {
    let model = &*model.unjoined();
    let parts = parts(model);
    let paths: Vec<ContextPath> = parts.iter().map(|p| p.0.clone()).collect();
    let low: Vec<f64> = paths.iter().map(|p| bounds(model, p).map_or(0.0, |b| b.0.z)).collect();
    let adj = touching(model, &paths);
    let n = paths.len();
    let mut placed = vec![false; n];
    let mut steps = Vec::new();
    let lowest = |c: &[usize]| c.iter().copied().min_by(|&a, &b| low[a].total_cmp(&low[b]).then(parts[a].1.cmp(&parts[b].1)));
    let joined: Vec<usize> = (0..n).filter(|&i| !adj[i].is_empty()).collect();
    let base = (if joined.is_empty() { 0..n } else { 0..0 }).chain(joined).min_by(|&a, &b| low[a].total_cmp(&low[b]).then(adj[b].len().cmp(&adj[a].len())));
    if let Some(b) = base {
        placed[b] = true;
        steps.push(Step { parts: vec![paths[b].clone()], note: String::new() });
    }
    while placed.iter().any(|p| !p) {
        let near: Vec<usize> = (0..n).filter(|&i| !placed[i] && adj[i].iter().any(|&j| placed[j])).collect();
        let pool: Vec<usize> = if near.is_empty() { (0..n).filter(|&i| !placed[i]).collect() } else { near };
        let Some(c) = lowest(&pool) else { break };
        let group: Vec<usize> = pool.iter().copied().filter(|&i| parts[i].1 == parts[c].1).collect();
        for &i in &group {
            placed[i] = true;
        }
        steps.push(Step { parts: group.iter().map(|&i| paths[i].clone()).collect(), note: String::new() });
    }
    steps
}

/// Which parts touch which, with the axis and the side of the plane the first one lies on.
type Contacts = Vec<(usize, usize, usize, f64)>;

/// The exploded view's layout: the parts, their boxes, the contacts, the step of each part
/// (`None`: no steps, or a part left out of them).
struct Layout {
    paths: Vec<ContextPath>,
    boxes: Vec<Option<(DVec3, DVec3)>>,
    contacts: Contacts,
    step: Vec<Option<usize>>,
}

fn layout(model: &Model) -> Layout {
    let model = &*model.unjoined();
    let paths: Vec<ContextPath> = parts(model).into_iter().map(|p| p.0).collect();
    let boxes = paths.iter().map(|p| bounds(model, p)).collect();
    let mut contacts = Vec::new();
    for i in 0..paths.len() {
        for j in i + 1..paths.len() {
            if let Some(c) = crate::joinery::contact(model, &paths[i], &paths[j]) {
                contacts.push((i, j, c.axis, c.into_a));
            }
        }
    }
    let step = paths.iter().map(|p| model.assembly.step_of(p)).collect();
    Layout { paths, boxes, contacts, step }
}

impl Layout {
    /// The parts `k` touches that `earlier` keeps, each with the way away from it.
    fn away(&self, k: usize, earlier: impl Fn(usize) -> bool) -> Vec<(usize, DVec3)> {
        self.contacts
            .iter()
            .filter_map(|&(i, j, axis, into_a)| {
                let (other, sign) = if i == k { (j, into_a) } else if j == k { (i, -into_a) } else { return None };
                earlier(other).then(|| (other, DVec3::from_array(std::array::from_fn(|a| if a == axis { sign } else { 0.0 }))))
            })
            .collect()
    }

    /// Away from the parts in `from`; held on opposite sides, out along a free axis: away from
    /// the middle when well off it (a back panel backwards), else to the front.
    fn dir(&self, k: usize, from: &[(usize, DVec3)]) -> DVec3 {
        let dir: DVec3 = from.iter().map(|f| f.1).sum();
        if dir.length() >= 0.5 {
            return dir.normalize();
        }
        let blocked = from.iter().fold(DVec3::ZERO, |b, f| b.max(f.1.abs()));
        let (lo, hi) = self.boxes.iter().flatten().fold((DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY)), |(a, b), (l, h)| (a.min(*l), b.max(*h)));
        let centre = (lo + hi) * 0.5;
        let mid = self.boxes[k].map_or(centre, |(l, h)| (l + h) * 0.5);
        let off = mid - centre;
        let away = DVec3::from_array(std::array::from_fn(|a| if blocked[a] == 0.0 && off[a].abs() > 0.15 * (hi - lo)[a] { off[a] } else { 0.0 }));
        let d = if away.length() > 1e-6 {
            away
        } else if blocked.y == 0.0 {
            DVec3::NEG_Y
        } else if blocked == DVec3::ONE {
            off
        } else {
            DVec3::ONE - blocked
        };
        d.normalize_or_zero()
    }

    /// Each part's way out (unit) and the earlier parts it follows (with steps).
    fn plan(&self) -> Vec<(DVec3, Vec<usize>)> {
        (0..self.paths.len())
            .map(|k| match self.step[k] {
                // the base stays; a later part leaves the parts of earlier steps it touches
                Some(0) => (DVec3::ZERO, Vec::new()),
                Some(s) => {
                    let from = self.away(k, |o| self.step[o].is_some_and(|t| t < s));
                    let follow = from.iter().map(|f| f.0).collect();
                    (if from.is_empty() { self.dir(k, &self.away(k, |_| true)) } else { self.dir(k, &from) }, follow)
                }
                None => (self.dir(k, &self.away(k, |_| true)), Vec::new()),
            })
            .collect()
    }
}

/// The way each part leaves the assembled model in the exploded view (unit vectors, world).
/// Without steps: away from the parts it touches (a side away from the shelf, the top and
/// bottom away from the sides); a part held on opposite sides (a shelf between two sides)
/// slides out along a free axis, away from the middle (a back panel backwards) or else to the
/// front. With steps: the base stays, and a part leaves the parts of earlier steps it touches.
pub fn explode_dirs(model: &Model) -> Vec<(ContextPath, DVec3)> {
    let l = layout(model);
    let plan = l.plan();
    l.paths.into_iter().zip(plan).map(|(p, (d, _))| (p, d)).collect()
}

/// The exploded view's offsets for `distance` (world, mm), from the assembled model: each part
/// `distance` along [`explode_dirs`], with steps on top of where the earlier parts it touches
/// went (so each step stands further out); all then lifted so that nothing goes below the
/// model's bottom (the floor it stands on).
pub fn explode_offsets(model: &Model, distance: f64) -> Vec<(ContextPath, DVec3)> {
    let l = layout(model);
    let plan = l.plan();
    let mut off = vec![DVec3::ZERO; l.paths.len()];
    let mut order: Vec<usize> = (0..l.paths.len()).collect();
    order.sort_by_key(|&k| l.step[k].unwrap_or(usize::MAX));
    for k in order {
        let (dir, follow) = &plan[k];
        let base = if follow.is_empty() { DVec3::ZERO } else { follow.iter().map(|&j| off[j]).sum::<DVec3>() / follow.len() as f64 };
        off[k] = base + *dir * distance;
    }
    let floor = l.boxes.iter().flatten().map(|b| b.0.z).fold(f64::INFINITY, f64::min);
    let low = l.boxes.iter().zip(&off).filter_map(|(b, o)| b.map(|b| b.0.z + o.z)).fold(f64::INFINITY, f64::min);
    if low < floor {
        for o in &mut off {
            o.z += floor - low;
        }
    }
    l.paths.into_iter().zip(off).collect()
}

impl Model {
    /// How far the exploded view moved the part at `path` (world).
    pub fn assembly_offset(&self, path: &[InstanceId]) -> DVec3 {
        self.assembly.moved.iter().find(|(p, _)| p == path).map_or(DVec3::ZERO, |m| m.1)
    }

    /// The world transform of the part at `path` as assembled.
    pub fn assembled_transform(&self, path: &[InstanceId]) -> DMat4 {
        DMat4::from_translation(-self.assembly_offset(path)) * self.transform_at(path)
    }

    fn shift_part(&mut self, path: &[InstanceId], by: DVec3) {
        let Some((&id, parent)) = path.split_last() else { return };
        let local = self.transform_at(parent).inverse().transform_vector3(by);
        if let Some(inst) = self.mesh_at_mut(parent).and_then(|m| m.instances.get_mut(id)) {
            inst.transform = DMat4::from_translation(local) * inst.transform;
        }
    }

    fn set_part_hidden(&mut self, path: &[InstanceId], hidden: bool) -> bool {
        let Some((&id, parent)) = path.split_last() else { return false };
        match self.mesh_at_mut(parent).and_then(|m| m.instances.get_mut(id)) {
            Some(inst) if inst.hidden != hidden => {
                inst.hidden = hidden;
                true
            }
            _ => false,
        }
    }

    /// Put the parts back where they belong and show the ones a step hid.
    pub fn assemble(&mut self) {
        for (p, off) in std::mem::take(&mut self.assembly.moved) {
            self.shift_part(&p, -off);
        }
        for p in std::mem::take(&mut self.assembly.hid) {
            self.set_part_hidden(&p, false);
        }
        self.assembly.explode = 0.0;
        self.assembly.step = None;
    }

    /// Show the model exploded by `v.explode` mm and up to step `v.step`.
    pub fn set_assembly_view(&mut self, v: SceneAssembly) {
        self.assemble();
        let step = v.step.filter(|&s| s < self.assembly.steps.len());
        if v.explode > 0.0 {
            let offsets = explode_offsets(self, v.explode);
            for (p, off) in &offsets {
                self.shift_part(p, *off);
            }
            self.assembly.moved = offsets.into_iter().filter(|(_, o)| *o != DVec3::ZERO).collect();
        }
        if let Some(s) = step {
            let later: Vec<ContextPath> = self.assembly.steps[s + 1..].iter().flat_map(|st| st.parts.iter().cloned()).collect();
            for p in later {
                if self.set_part_hidden(&p, true) {
                    self.assembly.hid.push(p);
                }
            }
        }
        self.assembly.explode = v.explode.max(0.0);
        self.assembly.step = step;
    }

    /// The hardware joint `i` adds at its step: the step of the later of its two parts.
    pub fn joint_step(&self, i: usize) -> Option<usize> {
        let j = self.joints.get(i)?;
        Some(self.assembly.step_of(&j.a)?.max(self.assembly.step_of(&j.b)?))
    }
}
