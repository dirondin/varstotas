//! Assembly (track W, `docs/WORKSHOP-PLAN.md`): the order a piece of furniture goes together
//! in, and the exploded view.
//!
//! The parts are those of the cut list ([`crate::cutlist::cut_parts`], every copy). Parts touch
//! where a face of one's bounding box lies on a face of another's ([`crate::joinery::contact`]);
//! that contact graph suggests the steps ([`suggest_steps`]): the base first (the lowest part
//! touching another, the one touching most of those), then, step by step, the lowest part touching what is
//! already placed, with its copies. Dowels, shelf pins and cam bolts go into a part before the
//! next one comes onto them, so they get a fastener step of their own ([`Step::joints`],
//! [`with_fastener_steps`]).
//!
//! The view ([`Model::set_assembly_view`]) is document state: the exploded view moves parts
//! apart (the model stretched away from its middle and floor, a drawer out as a whole, no two
//! into each other, [`explode_offsets`]), and a step shown hides the
//! later steps' parts. What it did is kept ([`Assembly::moved`], [`Assembly::hid`]) and undone
//! before the next view, so joints, the cut list and new views work from the assembled model.

use crate::model::{ContextPath, Model};
use crate::InstanceId;
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};

/// One assembly step: the parts it adds (instance paths), the joints whose fasteners go in at
/// it (a fastener step: dowels into the earlier part before the later one comes onto them;
/// other joints' fasteners go in with the later of their parts) and a note for the
/// instructions.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Step {
    pub parts: Vec<ContextPath>,
    /// Indices into [`Model::joints`].
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub joints: Vec<usize>,
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

    /// The step that adds the part at `path`, or the group or component it is in.
    pub fn step_of(&self, path: &[InstanceId]) -> Option<usize> {
        self.steps.iter().position(|s| s.parts.iter().any(|p| path.starts_with(p)))
    }

    /// The step joint `i`'s fasteners are given to, if any.
    pub fn joint_given(&self, i: usize) -> Option<usize> {
        self.steps.iter().position(|s| s.joints.contains(&i))
    }
}

/// The joints whose fasteners could go in at a step of their own before step `k` of `steps`:
/// those that go in first (dowels, shelf pins, cam lock bolts) between a part of step `k` and
/// a part of an earlier step, not given to a step already.
pub fn fasteners_first(model: &Model, steps: &[Step], k: usize) -> Vec<usize> {
    let at = |p: &ContextPath| steps.iter().position(|s| s.parts.contains(p));
    model
        .joints
        .iter()
        .enumerate()
        .filter(|(i, j)| {
            j.kind.goes_in_first()
                && !steps.iter().any(|s| s.joints.contains(i))
                && matches!((at(&j.a), at(&j.b)), (Some(x), Some(y)) if x.max(y) == k && x.min(y) < k)
        })
        .map(|(i, _)| i)
        .collect()
}

/// `steps` with a fastener step ([`fasteners_first`]) before each step that has some.
pub fn with_fastener_steps(model: &Model, steps: Vec<Step>) -> Vec<Step> {
    let mut out = Vec::new();
    for (k, s) in steps.iter().enumerate() {
        let first = fasteners_first(model, &steps, k);
        if !first.is_empty() {
            out.push(Step { joints: first, ..Default::default() });
        }
        out.push(s.clone());
    }
    out
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
/// lowest first. Fasteners that go in first get steps of their own ([`with_fastener_steps`]).
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
        steps.push(Step { parts: vec![paths[b].clone()], ..Default::default() });
    }
    while placed.iter().any(|p| !p) {
        let near: Vec<usize> = (0..n).filter(|&i| !placed[i] && adj[i].iter().any(|&j| placed[j])).collect();
        let pool: Vec<usize> = if near.is_empty() { (0..n).filter(|&i| !placed[i]).collect() } else { near };
        let Some(c) = lowest(&pool) else { break };
        let group: Vec<usize> = pool.iter().copied().filter(|&i| parts[i].1 == parts[c].1).collect();
        for &i in &group {
            placed[i] = true;
        }
        steps.push(Step { parts: group.iter().map(|&i| paths[i].clone()).collect(), ..Default::default() });
    }
    with_fastener_steps(model, steps)
}

/// The exploded view's layout: the parts, their boxes as assembled, which touch which, and the
/// order they go together in (the step of each part, else the suggested one's; parts left out
/// of the steps last).
struct Layout {
    paths: Vec<ContextPath>,
    boxes: Vec<Option<(DVec3, DVec3)>>,
    adj: Vec<Vec<usize>>,
    rank: Vec<usize>,
}

/// Whether two boxes come closer than `margin` on every axis (a negative one: go more than
/// that far into each other).
fn overlap(a: (DVec3, DVec3), b: (DVec3, DVec3), margin: f64) -> bool {
    (0..3).all(|i| a.0[i] < b.1[i] + margin - 1e-6 && b.0[i] < a.1[i] + margin - 1e-6)
}

/// The box around `boxes`.
fn hull(boxes: impl Iterator<Item = (DVec3, DVec3)>) -> Option<(DVec3, DVec3)> {
    boxes.reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
}

/// The six ways along the axes.
const WAYS: [DVec3; 6] = [DVec3::NEG_Y, DVec3::Y, DVec3::Z, DVec3::NEG_Z, DVec3::NEG_X, DVec3::X];

fn layout(model: &Model) -> Layout {
    let model = &*model.unjoined();
    let paths: Vec<ContextPath> = parts(model).into_iter().map(|p| p.0).collect();
    let boxes = paths.iter().map(|p| bounds(model, p)).collect();
    let adj = touching(model, &paths);
    let rank = if model.assembly.steps.is_empty() {
        let steps = suggest_steps(model);
        paths.iter().map(|p| steps.iter().position(|s| s.parts.contains(p)).unwrap_or(usize::MAX)).collect()
    } else {
        paths.iter().map(|p| model.assembly.step_of(p).unwrap_or(usize::MAX)).collect()
    };
    Layout { paths, boxes, adj, rank }
}

impl Layout {
    /// The groups of parts that touch each other, the first going together first.
    fn groups(&self) -> Vec<Vec<usize>> {
        let n = self.paths.len();
        let mut group = vec![usize::MAX; n];
        let mut groups: Vec<Vec<usize>> = Vec::new();
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by_key(|&k| self.rank[k]);
        for s in order {
            if group[s] != usize::MAX {
                continue;
            }
            let (g, mut todo, mut members) = (groups.len(), vec![s], Vec::new());
            group[s] = g;
            while let Some(k) = todo.pop() {
                members.push(k);
                for &j in &self.adj[k] {
                    if group[j] == usize::MAX {
                        group[j] = g;
                        todo.push(j);
                    }
                }
            }
            groups.push(members);
        }
        groups
    }

    /// Whether the box `b` moving `d` for `reach` runs into the parts `blocks`.
    fn swept_clear(&self, b: (DVec3, DVec3), d: DVec3, reach: f64, blocks: &[usize]) -> bool {
        let swept = (b.0.min(b.0 + d * reach), b.1.max(b.1 + d * reach));
        blocks.iter().all(|&j| self.boxes[j].is_none_or(|o| !overlap(swept, o, -0.5)))
    }

    /// The exploded view's offsets for `distance`: the model stretched, each part's middle
    /// (across) and bottom (up) moved away from the model's middle and its floor in proportion,
    /// the outermost parts by `distance`: sides go sideways, a top up, a back backwards, shelves
    /// apart between the sides, the bottom stays on the floor. Parts clear of each other as
    /// assembled (one beside, behind or above the other) only get further apart, so none ends
    /// up in another, and the parts move evenly with `distance`. A group of parts touching
    /// nothing of the first (a drawer) also comes out as a whole by `distance`, the first way
    /// along an axis it can go without meeting another part (the front first), which keeps
    /// them clear too.
    fn offsets(&self, distance: f64) -> Vec<DVec3> {
        let n = self.paths.len();
        let mut off = vec![DVec3::ZERO; n];
        let Some((lo, hi)) = hull(self.boxes.iter().flatten().copied()) else { return off };
        let size = hi - lo;
        let anchor = DVec3::new((lo.x + hi.x) * 0.5, (lo.y + hi.y) * 0.5, lo.z);
        let reach = DVec3::new(size.x * 0.5, size.y * 0.5, size.z);
        let stretch = DVec3::from_array(std::array::from_fn(|a| if reach[a] > 1e-9 { distance / reach[a] } else { 0.0 }));
        for (k, o) in off.iter_mut().enumerate() {
            if let Some(b) = self.boxes[k] {
                let at = DVec3::new((b.0.x + b.1.x) * 0.5, (b.0.y + b.1.y) * 0.5, b.0.z);
                *o = (at - anchor) * stretch;
            }
        }
        for members in self.groups().iter().skip(1) {
            let Some(bounds) = hull(members.iter().filter_map(|&k| self.boxes[k])) else { continue };
            let others: Vec<usize> = (0..n).filter(|k| !members.contains(k)).collect();
            let o = (bounds.0 + bounds.1) * 0.5 - (lo + hi) * 0.5;
            let mut ways = vec![DVec3::NEG_Y];
            let mut axes = [0, 1, 2];
            axes.sort_by(|&x, &y| (o[y].abs() / size[y].max(1e-9)).total_cmp(&(o[x].abs() / size[x].max(1e-9))));
            ways.extend(axes.map(|a| DVec3::from_array(std::array::from_fn(|i| if i == a { if o[a] < 0.0 { -1.0 } else { 1.0 } } else { 0.0 }))));
            ways.extend(WAYS);
            // not down: below the floor
            if let Some(d) = ways.into_iter().filter(|d| d.z >= 0.0).find(|&d| self.swept_clear(bounds, d, size.length() * 2.0, &others)) {
                for &k in members {
                    off[k] += d * distance;
                }
            }
        }
        off
    }
}

/// The way each part leaves the assembled model in the exploded view (unit vectors, world):
/// away from the model's middle and its floor ([`explode_offsets`]).
pub fn explode_dirs(model: &Model) -> Vec<(ContextPath, DVec3)> {
    let l = layout(model);
    let off = l.offsets(1.0);
    l.paths.into_iter().zip(off).map(|(p, o)| (p, o.normalize_or_zero())).collect()
}

/// The exploded view's offsets for `distance` (world, mm), from the assembled model: the model
/// stretched so that the outermost parts move `distance`, a drawer out as a whole too; no part
/// ends up in another or below the floor, and each moves in proportion to `distance`.
pub fn explode_offsets(model: &Model, distance: f64) -> Vec<(ContextPath, DVec3)> {
    let l = layout(model);
    let off = l.offsets(distance);
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

    /// The step joint `i` is made at: the step of the later of its two parts.
    pub fn joint_step(&self, i: usize) -> Option<usize> {
        let j = self.placed_joints.get(i)?;
        let a = self.assembly.step_of(&j.a)?;
        if j.b.is_empty() {
            return Some(a);
        }
        Some(a.max(self.assembly.step_of(&j.b)?))
    }
}
