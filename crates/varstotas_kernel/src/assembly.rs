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
//! A group or component with steps of its own ([`crate::model::Definition::steps`]) is a
//! sub-assembly: a drawer put together first, then added to the cabinet as one unit. In the
//! steps it is placed in it is one of the [`units`], its copies like a part's; its own joints
//! belong to its steps, not to those ([`Model::joint_step`]); [`Model::sub_model`] is it alone,
//! for its steps and its pages of the booklet. [`suggest_all`] gives every group or component
//! with joints inside steps of its own.
//!
//! The view ([`Model::set_assembly_view`]) is document state: the exploded view moves parts
//! apart (the model stretched away from its middle and floor, a drawer out as a whole, no two
//! into each other, [`explode_offsets`]), and a step shown hides the
//! later steps' parts. What it did is kept ([`Assembly::moved`], [`Assembly::hid`]) and undone
//! before the next view, so joints, the cut list and new views work from the assembled model.

use crate::model::{ContextPath, Model};
use crate::{DefId, InstanceId};
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
    /// The sub-assembly whose steps the view shows (its instance path; empty: the root's).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub within: ContextPath,
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

/// The joints whose fasteners could go in at a step of their own before step `k` of `steps`
/// (those of [`Assembly::within`]; indices into [`Model::placed_joints`]): those that go in
/// first (dowels, shelf pins, cam lock bolts) of the sub-assembly's own between a part of step
/// `k` and a part of an earlier step, not given to a step already.
pub fn fasteners_first(model: &Model, steps: &[Step], k: usize) -> Vec<usize> {
    let at = |p: &ContextPath| steps.iter().position(|s| s.parts.iter().any(|q| p.starts_with(q)));
    let within = &model.assembly.within;
    let owner = model.def_at(within);
    let own = |i: usize| model.joint_places.get(i).is_some_and(|p| p.prefix == *within && Some(p.def) == owner);
    model
        .placed_joints
        .iter()
        .enumerate()
        .filter(|(i, j)| {
            own(*i)
                && j.kind.goes_in_first()
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

/// What a step adds: a part (its index in [`crate::cutlist::cut_parts`]) or a sub-assembly (its
/// definition). Units alike are copies of one another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Part(usize),
    Sub(DefId),
}

/// The units the steps add: each part, or the outermost sub-assembly it is in, once.
pub fn units(model: &Model) -> Vec<(ContextPath, Unit)> {
    units_within(model, &[])
}

/// The units the steps of the sub-assembly at `within` add: each of its parts, or the outermost
/// sub-assembly inside it the part is in, once.
pub fn units_within(model: &Model, within: &[InstanceId]) -> Vec<(ContextPath, Unit)> {
    let mut out: Vec<(ContextPath, Unit)> = Vec::new();
    for (path, i) in parts(model).into_iter().filter(|(p, _)| p.len() > within.len() && p.starts_with(within)) {
        let sub = (within.len() + 1..path.len()).find_map(|k| model.def_at(&path[..k]).filter(|&d| model.defs.get(d).is_some_and(|x| !x.steps.is_empty())).map(|d| (k, d)));
        let unit = match sub {
            Some((k, d)) => (path[..k].to_vec(), Unit::Sub(d)),
            None => (path, Unit::Part(i)),
        };
        if !out.iter().any(|(p, _)| *p == unit.0) {
            out.push(unit);
        }
    }
    out
}

/// Every group or component with joints between two or more parts inside and no steps yet
/// gets steps of its own (inner ones first), unless it is the whole model; then the root's
/// steps, those sub-assemblies one unit each. Returns the root's steps and each definition's.
pub fn suggest_all(model: &Model) -> (Vec<Step>, Vec<(DefId, Vec<Step>)>) {
    fn walk(m: &Model, d: DefId, path: &mut ContextPath, out: &mut Vec<(DefId, ContextPath)>) {
        let Some(def) = m.defs.get(d) else { return };
        for (id, inst) in &def.mesh.instances {
            path.push(id);
            out.push((inst.def, path.clone()));
            walk(m, inst.def, path, out);
            path.pop();
        }
    }
    let mut placed = Vec::new();
    walk(model, model.root, &mut Vec::new(), &mut placed);
    let all: Vec<ContextPath> = parts(model).into_iter().map(|p| p.0).collect();
    let mut todo: Vec<(DefId, usize)> = Vec::new();
    for (d, path) in &placed {
        let Some(def) = model.defs.get(*d) else { continue };
        if def.joints.is_empty() || !def.steps.is_empty() || todo.iter().any(|t| t.0 == *d) {
            continue;
        }
        let at: Vec<&ContextPath> = placed.iter().filter(|(x, _)| x == d).map(|(_, p)| p).collect();
        let inside = all.iter().filter(|p| p.starts_with(path)).count();
        let whole = at.len() == 1 && all.iter().all(|p| p.starts_with(path));
        if inside >= 2 && !whole {
            todo.push((*d, path.len()));
        }
    }
    todo.sort_by_key(|t| std::cmp::Reverse(t.1));
    let mut w = model.clone();
    let mut subs = Vec::new();
    for (d, _) in todo {
        let steps = suggest_steps(&w.sub_model(d));
        w.defs[d].steps = steps.clone();
        subs.push((d, steps));
    }
    (suggest_steps(&w), subs)
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
    let parts = units(model);
    let paths: Vec<ContextPath> = parts.iter().map(|p| p.0.clone()).collect();
    let low: Vec<f64> = paths.iter().map(|p| bounds(model, p).map_or(0.0, |b| b.0.z)).collect();
    let adj = touching(model, &paths);
    let n = paths.len();
    let mut placed = vec![false; n];
    let mut steps = Vec::new();
    let lowest = |c: &[usize]| c.iter().copied().min_by(|&a, &b| low[a].total_cmp(&low[b]).then(a.cmp(&b)));
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
    let paths: Vec<ContextPath> = units(model).into_iter().map(|p| p.0).collect();
    let boxes = paths.iter().map(|p| bounds(model, p)).collect();
    let adj = touching(model, &paths);
    let rank = if model.assembly.steps.is_empty() {
        let steps = suggest_steps(model);
        paths.iter().map(|p| steps.iter().position(|s| s.parts.iter().any(|q| p.starts_with(q))).unwrap_or(usize::MAX)).collect()
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
    /// How far the exploded view moved the part at `path`, or the sub-assembly it is in (world).
    pub fn assembly_offset(&self, path: &[InstanceId]) -> DVec3 {
        self.assembly.moved.iter().filter(|(p, _)| path.starts_with(p)).map(|m| m.1).sum()
    }

    /// The sub-assembly at `path` alone, as a model of its own: its content the root's, its
    /// joints and steps the root's, put together.
    pub fn sub_model(&self, d: DefId) -> Model {
        let mut m = self.clone();
        m.assemble();
        m.assembly = Assembly { steps: m.defs.get(d).map(|x| x.steps.clone()).unwrap_or_default(), ..Default::default() };
        m.joints = m.defs.get(d).map(|x| x.joints.clone()).unwrap_or_default();
        m.root = d;
        m.joint_context = Vec::new();
        m.refresh_fasteners();
        m
    }

    /// The steps of the sub-assembly at `within` (an instance path; empty: the root's), their
    /// parts' paths from the root, their joints indices into [`Model::placed_joints`].
    pub fn steps_within(&self, within: &[InstanceId]) -> std::borrow::Cow<'_, [Step]> {
        if within.is_empty() {
            return std::borrow::Cow::Borrowed(&self.assembly.steps);
        }
        let Some(d) = self.def_at(within) else { return std::borrow::Cow::Owned(Vec::new()) };
        let placed = |i: usize| self.joint_places.iter().position(|p| p.def == d && p.index == i && p.prefix == within);
        let steps = self.defs.get(d).map_or(&[][..], |x| &x.steps[..]);
        std::borrow::Cow::Owned(
            steps
                .iter()
                .map(|s| Step {
                    parts: s.parts.iter().map(|p| within.iter().chain(p).copied().collect()).collect(),
                    joints: s.joints.iter().filter_map(|&i| placed(i)).collect(),
                    note: s.note.clone(),
                })
                .collect(),
        )
    }

    /// The steps the view shows ([`Assembly::within`]).
    pub fn shown_steps(&self) -> std::borrow::Cow<'_, [Step]> {
        self.steps_within(&self.assembly.within)
    }

    /// Show the steps of the sub-assembly at `within` (empty: the root's), put together.
    pub fn set_assembly_within(&mut self, within: ContextPath) {
        if within != self.assembly.within {
            self.assemble();
            self.assembly.within = if self.def_at(&within).is_some() { within } else { Vec::new() };
        }
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

    /// Show the model exploded by `v.explode` mm and up to step `v.step`, of the steps of
    /// [`Assembly::within`]: a sub-assembly's view moves and hides its own parts only.
    pub fn set_assembly_view(&mut self, v: SceneAssembly) {
        self.assemble();
        let within = self.assembly.within.clone();
        let steps = self.shown_steps().into_owned();
        let step = v.step.filter(|&s| s < steps.len());
        if v.explode > 0.0 {
            let offsets = match self.def_at(&within).filter(|_| !within.is_empty()) {
                None => explode_offsets(self, v.explode),
                Some(d) => {
                    let xf = self.transform_at(&within);
                    let sub = self.sub_model(d);
                    explode_offsets(&sub, v.explode).into_iter().map(|(p, o)| (within.iter().chain(&p).copied().collect(), xf.transform_vector3(o))).collect()
                }
            };
            for (p, off) in &offsets {
                self.shift_part(p, *off);
            }
            self.assembly.moved = offsets.into_iter().filter(|(_, o)| *o != DVec3::ZERO).collect();
        }
        if let Some(s) = step {
            let later: Vec<ContextPath> = steps[s + 1..].iter().flat_map(|st| st.parts.iter().cloned()).collect();
            for p in later {
                if self.set_part_hidden(&p, true) {
                    self.assembly.hid.push(p);
                }
            }
        }
        self.assembly.explode = v.explode.max(0.0);
        self.assembly.step = step;
    }

    /// The step joint `i` is made at, of the steps shown: the step of the later of its two
    /// parts; none for a joint inside a sub-assembly those steps add whole (made in its own).
    pub fn joint_step(&self, i: usize) -> Option<usize> {
        let j = self.placed_joints.get(i)?;
        let steps = self.shown_steps();
        let at = |p: &ContextPath| steps.iter().enumerate().find_map(|(k, s)| s.parts.iter().find(|q| p.starts_with(q)).map(|q| (k, q.len())));
        let (a, unit) = at(&j.a)?;
        if unit < j.a.len() && (j.b.is_empty() || j.b.starts_with(&j.a[..unit])) {
            return None;
        }
        if j.b.is_empty() {
            return Some(a);
        }
        Some(a.max(at(&j.b)?.0))
    }
}
