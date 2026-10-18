//! Parameters of ordinary components (rigs): a component drawn or imported as usual gets
//! parameters (`@param` kinds of [`crate::generator`]), formulas and *joints* that move, show /
//! hide or paint its parts — the direct child groups / components, found by name — and
//! optionally a Rhai script `update(p, parts)` doing the same in code. Turning the values into
//! part operations ([`PartOp`]) is `varstotas_script::rig`; the kernel keeps the data and poses.
//!
//! The definition with the [`Rig`] is the *master*: its content is the rest pose, as drawn.
//! It is not placed itself: every copy uses a *variant* — a definition with the master's
//! content posed for its values ([`Rigged`]). Copies with equal values share one variant, as
//! generated components do. Edits made inside a variant are written back to the master (the
//! poses undone) and the other variants posed again.
//!
//! A painted part is a copy of the part's definition with its faces in the material (also
//! [`Rigged`], with `paint`), shared by all variants painting it the same.

use crate::generator::{GenFormula, GenParam, GenValues};
use crate::mesh::Mesh;
use crate::model::{Definition, Model};
use crate::{Aabb, DefId, InstanceId};
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// The parameters of a master definition.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<GenParam>,
    /// Derived values, in order (as a generator's `@formula`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formulas: Vec<GenFormula>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub joints: Vec<Joint>,
    /// Rhai code with `fn update(p, parts)` (run after the joints; "" = none).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub script: String,
}

/// A part driven by a parameter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Joint {
    /// The part: name of a direct child instance (its definition's name when unnamed).
    pub part: String,
    pub kind: JointKind,
    /// The driving parameter or formula ("" when `expr` drives it).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub param: String,
    /// Motion (degrees / mm) at the parameter's minimum and maximum; a parameter without a
    /// range is the motion itself.
    #[serde(default)]
    pub from: f64,
    #[serde(default)]
    pub to: f64,
    /// A Rhai expression over the parameters and formulas giving the motion (or the flag /
    /// the material name) instead of the parameter.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub expr: String,
}

/// What a joint does to its part (vectors in the master's local axes).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum JointKind {
    /// Turn about the axis through `origin` (degrees, right hand about `axis`).
    Rotate { origin: DVec3, axis: DVec3 },
    /// Move along `dir` (mm).
    Slide { dir: DVec3 },
    /// Shown while the value is true (a number: not zero).
    Visible,
    /// Painted with the material named by the value ("" = as drawn).
    Material,
}

impl JointKind {
    pub fn name(&self) -> &'static str {
        match self {
            JointKind::Rotate { .. } => "rotate",
            JointKind::Slide { .. } => "slide",
            JointKind::Visible => "visible",
            JointKind::Material => "material",
        }
    }
}

/// An operation on a part (by name) for a set of values.
#[derive(Clone, Debug, PartialEq)]
pub enum PartOp {
    /// Applied after the part's own transform (in the master's axes): `m * transform`.
    Transform(String, DMat4),
    Visible(String, bool),
    /// Material name ("" = as drawn).
    Paint(String, String),
}

/// On a variant (the master's content posed for `values`) or, with `paint`, on a painted copy
/// of a part (`master` is then the part's own definition).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rigged {
    pub master: DefId,
    #[serde(default, skip_serializing_if = "GenValues::is_empty")]
    pub values: GenValues,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paint: Option<String>,
}

/// The values of one copy (the instance path from the root, itself last) saved in a scene.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneRig {
    pub path: Vec<InstanceId>,
    pub values: GenValues,
}

/// What the operations do to one part instance.
#[derive(Clone, Debug, Default)]
struct Effect {
    m: Option<DMat4>,
    hidden: Option<bool>,
    paint: Option<String>,
}

/// A part's name: the instance's, else its definition's.
fn part_name(model: &Model, i: &crate::Instance) -> String {
    if i.name.is_empty() { model.defs.get(i.def).map(|d| d.name.clone()).unwrap_or_default() } else { i.name.clone() }
}

impl Model {
    /// The master of definition `d`: `d` when it has a rig, its master when it is a variant.
    pub fn rig_master(&self, d: DefId) -> Option<DefId> {
        let def = self.defs.get(d)?;
        if def.rig.is_some() {
            return Some(d);
        }
        def.rigged.as_ref().filter(|r| r.paint.is_none()).map(|r| r.master).filter(|m| self.defs.get(*m).is_some_and(|x| x.rig.is_some()))
    }

    /// The variant and its values of instance `inst` in the context `path`.
    pub fn instance_rigged(&self, path: &[InstanceId], inst: InstanceId) -> Option<(DefId, &Rigged)> {
        let d = self.mesh_at(path)?.instances.get(inst)?.def;
        let r = self.defs.get(d)?.rigged.as_ref().filter(|r| r.paint.is_none())?;
        Some((d, r))
    }

    /// The parts of master `d` (its direct child instances) with their names.
    pub fn rig_parts(&self, d: DefId) -> Vec<(InstanceId, String)> {
        let Some(def) = self.defs.get(d) else { return Vec::new() };
        let mut v: Vec<_> = def.mesh.instances.iter().map(|(id, i)| (id, part_name(self, i))).collect();
        v.sort_by_key(|a| a.1.to_lowercase());
        v
    }

    /// Rest-pose bounds of each part of `d` (in `d`'s axes), by name.
    pub fn rig_part_bounds(&self, d: DefId) -> Vec<(String, Aabb)> {
        let Some(def) = self.defs.get(d) else { return Vec::new() };
        def.mesh
            .instances
            .values()
            .filter_map(|i| Some((part_name(self, i), self.def_bounds(i.def)?.transformed(&i.transform))))
            .collect()
    }

    /// Give the unnamed parts of `d` unique names (their definitions' names, numbered when
    /// repeated), so joints find each one. Returns how many were named.
    pub fn name_rig_parts(&mut self, d: DefId) -> usize {
        let Some(def) = self.defs.get(d) else { return 0 };
        let mut taken: HashSet<String> = def.mesh.instances.values().filter(|i| !i.name.is_empty()).map(|i| i.name.clone()).collect();
        let mut ids: Vec<(InstanceId, String)> =
            def.mesh.instances.iter().filter(|(_, i)| i.name.is_empty()).map(|(id, i)| (id, self.defs.get(i.def).map(|x| x.name.clone()).unwrap_or_default())).collect();
        ids.sort_by_key(|a| a.0);
        let mut n = 0;
        for (id, base) in ids {
            let base = if base.trim().is_empty() { "Part".to_string() } else { base };
            let name = if taken.contains(&base) { (2..).map(|k| format!("{base} {k}")).find(|x| !taken.contains(x)).expect("a free name") } else { base };
            taken.insert(name.clone());
            self.defs[d].mesh.instances[id].name = name;
            n += 1;
        }
        n
    }

    /// The variant of master `master` with exactly `values`.
    pub fn rig_variant(&self, master: DefId, values: &GenValues) -> Option<DefId> {
        self.defs
            .iter()
            .find(|(_, d)| d.rigged.as_ref().is_some_and(|r| r.master == master && r.paint.is_none() && r.values == *values))
            .map(|(id, _)| id)
    }

    /// Variants of `master` with their values.
    pub fn rig_variants(&self, master: DefId) -> Vec<(DefId, GenValues)> {
        self.defs
            .iter()
            .filter_map(|(id, d)| d.rigged.as_ref().filter(|r| r.master == master && r.paint.is_none()).map(|r| (id, r.values.clone())))
            .collect()
    }

    /// Every placed copy of a component with parameters (not inside variants) with its values.
    pub fn rig_instances(&self) -> Vec<SceneRig> {
        fn walk(m: &Model, d: DefId, path: &mut Vec<InstanceId>, out: &mut Vec<SceneRig>) {
            if path.len() > 32 {
                return;
            }
            let Some(def) = m.defs.get(d) else { return };
            for (id, i) in &def.mesh.instances {
                path.push(id);
                match m.defs.get(i.def).and_then(|x| x.rigged.as_ref()) {
                    Some(r) if r.paint.is_none() => out.push(SceneRig { path: path.clone(), values: r.values.clone() }),
                    Some(_) => {}
                    None => walk(m, i.def, path, out),
                }
                path.pop();
            }
        }
        let mut out = Vec::new();
        walk(self, self.root, &mut Vec::new(), &mut out);
        out
    }

    /// Placed copies of component `d`, those of its variants included.
    pub fn placed_count(&self, d: DefId) -> usize {
        self.instance_count(d) + self.rig_variants(d).iter().map(|(v, _)| self.instance_count(*v)).sum::<usize>()
    }

    /// Masters, painted parts' originals: kept while their variants / copies live.
    pub(crate) fn rig_masters(&self) -> HashSet<DefId> {
        self.defs.values().filter_map(|d| d.rigged.as_ref().map(|r| r.master)).collect()
    }

    /// The effect of `ops` on each part instance of `mesh`.
    fn effects(&self, mesh: &Mesh, ops: &[PartOp]) -> HashMap<InstanceId, Effect> {
        let names: Vec<(InstanceId, String)> = mesh.instances.iter().map(|(id, i)| (id, part_name(self, i))).collect();
        let mut out: HashMap<InstanceId, Effect> = HashMap::new();
        for op in ops {
            let part = match op {
                PartOp::Transform(p, _) | PartOp::Visible(p, _) | PartOp::Paint(p, _) => p,
            };
            for (id, _) in names.iter().filter(|(_, n)| n == part) {
                let e = out.entry(*id).or_default();
                match op {
                    PartOp::Transform(_, m) => e.m = Some(*m * e.m.unwrap_or(DMat4::IDENTITY)),
                    PartOp::Visible(_, v) => e.hidden = Some(!v),
                    PartOp::Paint(_, name) => e.paint = (!name.is_empty()).then(|| name.clone()),
                }
            }
        }
        out
    }

    /// The painted copy of part definition `d` (shared), `None` when there is no such material.
    fn painted(&mut self, d: DefId, material: &str) -> Option<DefId> {
        let mat = self.materials.iter().find(|(_, m)| m.name == material).map(|(id, _)| id)?;
        let found = self.defs.iter().find(|(_, x)| x.rigged.as_ref().is_some_and(|r| r.master == d && r.paint.as_deref() == Some(material)));
        if let Some((id, _)) = found {
            return Some(id);
        }
        let mut def = self.defs.get(d)?.clone();
        for f in def.mesh.faces.values_mut() {
            f.front = Some(mat);
            if f.back.is_some() {
                f.back = Some(mat);
            }
        }
        for i in def.mesh.instances.values_mut() {
            i.material = Some(mat);
        }
        def.rig = None;
        def.rigged = Some(Rigged { master: d, values: GenValues::new(), paint: Some(material.to_string()) });
        Some(self.defs.insert(def))
    }

    /// Pose definition `target` as master `master` with `ops` (`target` takes the master's
    /// content, the parts moved, hidden and painted). Painted copies nothing uses any more go.
    pub fn pose_rig(&mut self, target: DefId, master: DefId, ops: &[PartOp]) {
        let Some(mut mesh) = self.defs.get(master).map(|d| d.mesh.clone()) else { return };
        let fx = self.effects(&mesh, ops);
        for (id, e) in fx {
            let Some(i) = mesh.instances.get_mut(id) else { continue };
            if let Some(m) = e.m {
                i.transform = m * i.transform;
            }
            if let Some(h) = e.hidden {
                i.hidden = h;
            }
            if let Some(p) = e.paint {
                let d = i.def;
                if let Some(c) = self.painted(d, &p) {
                    mesh.instances[id].def = c;
                }
            }
        }
        let old: HashSet<DefId> = self.defs.get(target).map(|d| d.mesh.instances.values().map(|i| i.def).collect()).unwrap_or_default();
        if let Some(t) = self.defs.get_mut(target) {
            t.mesh = mesh;
        }
        self.remove_unused(old);
    }

    /// A new variant of `master` for `values` posed with `ops`.
    pub fn new_rig_variant(&mut self, master: DefId, values: GenValues, ops: &[PartOp]) -> Option<DefId> {
        let m = self.defs.get(master)?;
        let def = Definition {
            name: m.name.clone(),
            mesh: Mesh::new(),
            is_group: false,
            guides: m.guides.clone(),
            params: None,
            generated: None,
            source: m.source.clone(),
            rig: None,
            rigged: Some(Rigged { master, values, paint: None }),
        };
        let v = self.defs.insert(def);
        self.pose_rig(v, master, ops);
        Some(v)
    }

    /// Write the content of variant `v` (edited, posed with `ops` of its values) back to its
    /// master: the poses undone, the parts' visibility as in the master, painted parts back to
    /// their own definitions. Parts added in the variant stay where they are.
    pub fn rig_write_back(&mut self, v: DefId, ops: &[PartOp]) -> bool {
        let Some(master) = self.defs.get(v).and_then(|d| d.rigged.as_ref()).filter(|r| r.paint.is_none()).map(|r| r.master) else { return false };
        let Some(old) = self.defs.get(master).map(|d| d.mesh.clone()) else { return false };
        let fx = self.effects(&old, ops);
        let mut mesh = self.defs[v].mesh.clone();
        let ids: Vec<InstanceId> = mesh.instances.keys().collect();
        for id in ids {
            let i = &mut mesh.instances[id];
            if let Some(r) = self.defs.get(i.def).and_then(|d| d.rigged.as_ref()).filter(|r| r.paint.is_some()) {
                i.def = r.master;
            }
            let Some(e) = fx.get(&id) else { continue };
            if let Some(m) = e.m {
                i.transform = m.inverse() * i.transform;
            }
            if e.hidden.is_some() {
                i.hidden = old.instances.get(id).is_some_and(|o| o.hidden);
            }
        }
        self.defs[master].mesh = mesh;
        self.defs[master].guides = self.defs[v].guides.clone();
        true
    }
}
