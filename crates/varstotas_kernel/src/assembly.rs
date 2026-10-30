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
//! apart (each square to its face, out of the way of the parts in before it, no two into each
//! other, a drawer out as a whole, [`explode_offsets`]), and a step shown hides the
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

    /// The step adding part `path`.
    pub fn step_of(&self, path: &[InstanceId]) -> Option<usize> {
        self.steps.iter().position(|s| s.parts.iter().any(|p| p == path))
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

/// Where the exploded view takes each part: its way out (unit) within its group and the
/// group's, and its offset.
struct Plan {
    dir: Vec<DVec3>,
    group_dir: Vec<DVec3>,
    off: Vec<DVec3>,
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
            members.sort_by_key(|&k| (self.rank[k], k));
            groups.push(members);
        }
        groups
    }

    /// Whether the box `b` moving `d` for `reach` runs into the parts `blocks`.
    fn swept_clear(&self, b: (DVec3, DVec3), d: DVec3, reach: f64, blocks: &[usize]) -> bool {
        let swept = (b.0.min(b.0 + d * reach), b.1.max(b.1 + d * reach));
        blocks.iter().all(|&j| self.boxes[j].is_none_or(|o| !overlap(swept, o, -0.5)))
    }

    /// Part `k`'s way out of its group: square to its face (its thinnest size), away from the
    /// group's middle, so that sides go sideways, a top up, a back backwards and doors to the
    /// front; when the parts in before it are in that way (a shelf between the top and the
    /// bottom), the first way clear of them: to the front, then away from the middle, then any.
    fn part_dir(&self, k: usize, group: &[usize], (lo, hi): (DVec3, DVec3)) -> DVec3 {
        let Some(b) = self.boxes[k] else { return DVec3::ZERO };
        let size = b.1 - b.0;
        let off = (b.0 + b.1) * 0.5 - (lo + hi) * 0.5;
        let thin = (0..3).min_by(|&x, &y| size[x].total_cmp(&size[y])).unwrap_or(2);
        let along = |a: usize| DVec3::from_array(std::array::from_fn(|i| if i == a { if off[a] < -1e-6 { -1.0 } else { 1.0 } } else { 0.0 }));
        let mut ways = vec![along(thin), DVec3::NEG_Y];
        let mut axes = [0, 1, 2];
        axes.sort_by(|&x, &y| (off[y].abs() / (hi - lo)[y].max(1e-9)).total_cmp(&(off[x].abs() / (hi - lo)[x].max(1e-9))));
        ways.extend(axes.map(along));
        ways.extend(WAYS);
        let blocks: Vec<usize> = group.iter().copied().filter(|&j| j != k && self.rank[j] <= self.rank[k]).collect();
        let reach = (hi - lo).length() * 2.0;
        ways.iter().copied().find(|&d| self.swept_clear(b, d, reach, &blocks)).unwrap_or(ways[0])
    }

    /// Moves `parts` (with offsets `off`) further along `dir` while one comes closer than
    /// `margin` to one of `placed`, or than they stand apart assembled when closer (parts that
    /// touch: while they go into each other).
    fn clear(&self, parts: &[usize], dir: DVec3, placed: &[usize], off: &mut [DVec3], margin: f64) {
        if dir == DVec3::ZERO {
            return;
        }
        let at = |off: &[DVec3], j: usize| self.boxes[j].map(|b| (b.0 + off[j], b.1 + off[j]));
        for _ in 0..4 * (placed.len() + 1) {
            let hit = parts.iter().find_map(|&k| {
                let a = at(off, k)?;
                placed.iter().find_map(|&j| {
                    let o = at(off, j)?;
                    // no closer than they stand assembled (doors side by side)
                    let apart = match (self.boxes[k], self.boxes[j]) {
                        (Some(x), Some(y)) => (0..3).map(|i| (x.0[i] - y.1[i]).max(y.0[i] - x.1[i])).fold(0.0, f64::max),
                        _ => margin,
                    };
                    let m = if self.adj[k].contains(&j) { 0.0 } else { margin.min(apart) };
                    overlap(a, o, m).then_some((a, o, m))
                })
            });
            let Some(((lo, hi), o, m)) = hit else { return };
            // the least way along `dir` that clears it on one axis
            let t = (0..3)
                .filter(|&a| dir[a].abs() > 1e-9)
                .map(|a| if dir[a] > 0.0 { (o.1[a] + m - lo[a]) / dir[a] } else { (hi[a] + m - o.0[a]) / -dir[a] })
                .fold(f64::INFINITY, f64::min);
            for &k in parts {
                off[k] += dir * t.max(1e-6);
            }
        }
    }

    /// The exploded view for `distance`: within each group the parts that go together first
    /// stay, every other part goes `distance` its way ([`Layout::part_dir`]) out from where the
    /// parts before it that it touches went along that way, and further while it would run into
    /// a part placed before it; each later group (a drawer) then moves out of the first as a
    /// whole, to the front when it can, `distance` and further until clear of all placed.
    fn plan(&self, distance: f64) -> Plan {
        let n = self.paths.len();
        let (mut dir, mut group_dir, mut off) = (vec![DVec3::ZERO; n], vec![DVec3::ZERO; n], vec![DVec3::ZERO; n]);
        let margin = distance * 0.1;
        let groups = self.groups();
        let mut placed: Vec<usize> = Vec::new();
        for (g, members) in groups.iter().enumerate() {
            let Some(bounds) = hull(members.iter().filter_map(|&k| self.boxes[k])) else { continue };
            let first = self.rank[members[0]];
            let mut inside: Vec<usize> = Vec::new();
            for &k in members {
                if self.rank[k] != first {
                    dir[k] = self.part_dir(k, members, bounds);
                    let out = self.adj[k].iter().filter(|&&j| self.rank[j] < self.rank[k]).map(|&j| off[j].dot(dir[k])).fold(0.0, f64::max);
                    off[k] = dir[k] * (out + distance);
                    self.clear(&[k], dir[k], &inside, &mut off, margin);
                }
                inside.push(k);
            }
            if g > 0 {
                // the group out of the ones before it: the front first, else the way it is off
                // their middle, then any
                let Some(before) = hull(placed.iter().filter_map(|&k| self.boxes[k])) else { continue };
                let o = (bounds.0 + bounds.1) * 0.5 - (before.0 + before.1) * 0.5;
                let mut ways = vec![DVec3::NEG_Y];
                let mut axes = [0, 1, 2];
                axes.sort_by(|&x, &y| (o[y].abs() / (before.1 - before.0)[y].max(1e-9)).total_cmp(&(o[x].abs() / (before.1 - before.0)[x].max(1e-9))));
                ways.extend(axes.map(|a| DVec3::from_array(std::array::from_fn(|i| if i == a { if o[a] < 0.0 { -1.0 } else { 1.0 } } else { 0.0 }))));
                ways.extend(WAYS);
                let reach = (before.1 - before.0).length() * 2.0;
                let d = ways.iter().copied().find(|&d| self.swept_clear(bounds, d, reach, &placed)).unwrap_or(DVec3::NEG_Y);
                for &k in members {
                    off[k] += d * distance;
                    group_dir[k] = d;
                }
                self.clear(members, d, &placed, &mut off, margin);
            }
            placed.extend(members);
        }
        // lifted so that nothing goes below the bottom (the floor it stands on)
        let floor = self.boxes.iter().flatten().map(|b| b.0.z).fold(f64::INFINITY, f64::min);
        let low = self.boxes.iter().zip(&off).filter_map(|(b, o)| b.map(|b| b.0.z + o.z)).fold(f64::INFINITY, f64::min);
        if low < floor {
            for o in &mut off {
                o.z += floor - low;
            }
        }
        Plan { dir, group_dir, off }
    }
}

/// The way each part leaves the assembled model in the exploded view (unit vectors, world):
/// square to its face away from the middle of the parts it touches (sides sideways, a top up,
/// a back backwards, doors to the front), or out of the way of the parts in before it (a shelf
/// between a top and a bottom to the front); a part that stays in a group moved out as a whole
/// (a drawer's) the group's way.
pub fn explode_dirs(model: &Model) -> Vec<(ContextPath, DVec3)> {
    let l = layout(model);
    let p = l.plan(1.0);
    l.paths.into_iter().enumerate().map(|(k, path)| (path, if p.dir[k] == DVec3::ZERO { p.group_dir[k] } else { p.dir[k] })).collect()
}

/// The exploded view's offsets for `distance` (world, mm), from the assembled model: no two
/// parts end up in each other (a tenth of `distance` apart unless closer as assembled), and
/// nothing goes below the model's bottom ([`Layout::plan`]).
pub fn explode_offsets(model: &Model, distance: f64) -> Vec<(ContextPath, DVec3)> {
    let l = layout(model);
    let p = l.plan(distance);
    l.paths.into_iter().zip(p.off).collect()
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
        let j = self.joints.get(i)?;
        let a = self.assembly.step_of(&j.a)?;
        if j.b.is_empty() {
            return Some(a);
        }
        Some(a.max(self.assembly.step_of(&j.b)?))
    }
}
