//! Parametric components (a light form of dynamic components): a group whose
//! content is generated from a base definition by a few attributes — size along the base's
//! own X / Y / Z and a number of copies in a row with a gap.
//!
//! The parametric group's definition holds `count` instances of either the base itself
//! (resize by scaling) or of a *variant*: a copy of the base stretched by moving everything
//! beyond its centre plane (so ends, holes and thicknesses near the ends keep their size).
//! The base stays a definition of its own (kept alive by [`Parametric::base`]); it is edited
//! by opening one of the copies when stretching is off.

use crate::model::{Definition, Model};
use crate::{Aabb, DefId, Instance, InstanceId, OpError, OpResult, TOL, VertexId};
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use varstotas_i18n::tr;

/// Attributes of a parametric group (stored on its definition).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parametric {
    /// The definition the content is generated from.
    pub base: DefId,
    /// Size along the base's own X / Y / Z (mm); `None` = as modelled.
    pub size: [Option<f64>; 3],
    /// Stretch (move the geometry beyond the centre plane) instead of scaling.
    pub stretch: bool,
    /// Number of copies (≥ 1).
    pub count: u32,
    /// Axis of the row of copies (0 = X, 1 = Y, 2 = Z of the base).
    pub axis: u8,
    /// Gap between neighbouring copies (mm, may be negative for overlap).
    pub gap: f64,
    /// Stretched copy of the base used by the copies (generated).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<DefId>,
}

impl Parametric {
    pub fn new(base: DefId) -> Self {
        Self { base, size: [None; 3], stretch: true, count: 1, axis: 0, gap: 0.0, variant: None }
    }
}

fn axis(k: usize) -> DVec3 {
    [DVec3::X, DVec3::Y, DVec3::Z][k]
}

impl Model {
    /// Parametric attributes of the definition of instance `inst` in the context `path`.
    pub fn instance_params(&self, path: &[InstanceId], inst: InstanceId) -> Option<&Parametric> {
        let d = self.mesh_at(path)?.instances.get(inst)?.def;
        self.defs.get(d)?.params.as_ref()
    }

    /// Turn a group / component instance into a parametric group whose content is generated
    /// from the instance's definition (one copy, original size). Returns the new definition.
    pub fn make_parametric(&mut self, path: &[InstanceId], inst: InstanceId) -> OpResult<DefId> {
        let pd = self.def_at(path).ok_or(OpError::NotFound)?;
        let base = self.defs[pd].mesh.instances.get(inst).ok_or(OpError::NotFound)?.def;
        if self.defs.get(base).is_none_or(|d| d.params.is_some()) {
            return Err(OpError::Invalid(tr!("kernel-err-already-parametric")));
        }
        if self.def_bounds(base).is_none() {
            return Err(OpError::Invalid(tr!("kernel-err-empty-group")));
        }
        let name = self.defs[base].name.clone();
        let p = self.defs.insert(Definition {
            name,
            mesh: Default::default(),
            is_group: true,
            guides: Default::default(),
            params: Some(Parametric::new(base)),
            generated: None,
            source: None,
            rig: None,
            rigged: None,
            preview: None,
        });
        self.defs[pd].mesh.instances[inst].def = p;
        self.regenerate_params(p)?;
        Ok(p)
    }

    /// Set the attributes of the parametric instance `inst` (its definition is made unique
    /// first) and regenerate its content.
    pub fn set_instance_params(&mut self, path: &[InstanceId], inst: InstanceId, mut params: Parametric) -> OpResult<()> {
        self.make_unique(path, inst)?;
        let d = self.mesh_at(path).and_then(|m| m.instances.get(inst)).ok_or(OpError::NotFound)?.def;
        let old = self.defs[d].params.as_ref().ok_or(OpError::Invalid(tr!("kernel-err-not-parametric")))?;
        params.base = old.base;
        params.variant = old.variant;
        params.count = params.count.clamp(1, 1000);
        params.axis = params.axis.min(2);
        if params.size.iter().flatten().any(|s| !s.is_finite() || *s <= TOL) || !params.gap.is_finite() {
            return Err(OpError::Invalid(tr!("kernel-err-size-must-be-positive")));
        }
        self.defs[d].params = Some(params);
        self.regenerate_params(d)
    }

    /// Rebuild the content of parametric definition `d` from its base and attributes.
    pub fn regenerate_params(&mut self, d: DefId) -> OpResult<()> {
        let mut params = self.defs.get(d).and_then(|x| x.params.clone()).ok_or(OpError::NotFound)?;
        let bb = self.def_bounds(params.base).ok_or(OpError::Invalid(tr!("kernel-err-empty-base")))?;
        let ext = bb.max - bb.min;
        let wants_stretch = params.stretch
            && (0..3).any(|k| params.size[k].is_some_and(|s| (s - ext[k]).abs() > TOL && ext[k] > TOL));
        // Old generated copies go; other content of the group stays.
        let generated: Vec<InstanceId> = self.defs[d]
            .mesh
            .instances
            .iter()
            .filter(|(_, i)| i.def == params.base || Some(i.def) == params.variant)
            .map(|(id, _)| id)
            .collect();
        for id in generated {
            self.defs[d].mesh.instances.remove(id);
        }
        let (child, xf) = if wants_stretch {
            // Reuse the variant unless another definition still uses it (a copied group).
            let v = match params.variant.filter(|v| self.defs.contains_key(*v) && self.instance_count(*v) == 0) {
                Some(v) => v,
                None => {
                    let b = &self.defs[params.base];
                    self.defs.insert(Definition { name: format!("{} (stretched)", b.name), is_group: true, params: None, generated: None, rig: None, rigged: None, ..b.clone() })
                }
            };
            let mut def = self.defs[params.base].clone();
            def.name = format!("{} (stretched)", def.name);
            def.is_group = true;
            def.params = None;
            def.generated = None;
            def.rig = None;
            def.rigged = None;
            self.defs[v] = def;
            for k in 0..3 {
                if let Some(s) = params.size[k] {
                    self.stretch_def(v, k, s);
                }
            }
            params.variant = Some(v);
            (v, DMat4::IDENTITY)
        } else {
            if let Some(v) = params.variant.take()
                && self.instance_count(v) == 0
            {
                self.defs.remove(v);
            }
            let s = DVec3::from_array(std::array::from_fn(|k| match params.size[k] {
                Some(s) if ext[k] > TOL => s / ext[k],
                _ => 1.0,
            }));
            let xf = DMat4::from_translation(bb.min) * DMat4::from_scale(s) * DMat4::from_translation(-bb.min);
            (params.base, xf)
        };
        let size = self.def_bounds(child).map_or(DVec3::ZERO, |b| b.transformed(&xf).max - b.transformed(&xf).min);
        let k = params.axis as usize;
        let step = axis(k) * (size[k] + params.gap);
        let mesh = &mut self.defs[d].mesh;
        for i in 0..params.count.max(1) {
            mesh.instances.insert(Instance {
                def: child,
                transform: DMat4::from_translation(step * i as f64) * xf,
                name: String::new(),
                material: None,
                tag: None,
                hidden: false,
                locked: false,
            });
        }
        self.defs[d].params = Some(params);
        Ok(())
    }

    /// Stretch definition `d` along local axis `k` to the size `s` by moving every vertex and
    /// nested instance beyond the centre plane. Shrinking stops before moved parts reach the
    /// ones that stay.
    fn stretch_def(&mut self, d: DefId, k: usize, s: f64) {
        let Some(bb) = self.def_bounds(d) else { return };
        let mut delta = s - (bb.max[k] - bb.min[k]);
        if delta.abs() <= TOL {
            return;
        }
        let mid = (bb.min[k] + bb.max[k]) * 0.5;
        let child_bounds: Vec<(InstanceId, Aabb)> = self.defs[d]
            .mesh
            .instances
            .iter()
            .filter_map(|(id, i)| Some((id, self.def_bounds(i.def)?.transformed(&i.transform))))
            .collect();
        let mesh = &self.defs[d].mesh;
        let moved: HashSet<VertexId> = mesh.vertices.iter().filter(|(_, v)| v.pos[k] > mid + TOL).map(|(id, _)| id).collect();
        let moved_inst: Vec<InstanceId> =
            child_bounds.iter().filter(|(_, b)| (b.min[k] + b.max[k]) * 0.5 > mid + TOL).map(|(id, _)| *id).collect();
        // Clamp a shrink: the closest moved part must stay beyond the farthest fixed one.
        let mut moved_lo = f64::INFINITY;
        let mut fixed_hi = f64::NEG_INFINITY;
        for (id, v) in &mesh.vertices {
            if moved.contains(&id) {
                moved_lo = moved_lo.min(v.pos[k]);
            } else {
                fixed_hi = fixed_hi.max(v.pos[k]);
            }
        }
        for (id, b) in &child_bounds {
            if moved_inst.contains(id) {
                moved_lo = moved_lo.min(b.min[k]);
            } else {
                fixed_hi = fixed_hi.max(b.max[k]);
            }
        }
        if fixed_hi.is_finite() && moved_lo.is_finite() {
            delta = delta.max(-(moved_lo - fixed_hi) + 0.01 * (bb.max[k] - bb.min[k]).max(TOL * 10.0));
        }
        let t = DMat4::from_translation(axis(k) * delta);
        let mesh = &mut self.defs[d].mesh;
        mesh.transform_vertices(&moved, &t);
        for id in moved_inst {
            if let Some(i) = mesh.instances.get_mut(id) {
                i.transform = t * i.transform;
            }
        }
    }

    /// Definitions kept alive by parametric groups (their bases), besides instances.
    pub(crate) fn param_bases(&self) -> HashSet<DefId> {
        let mut s: HashSet<DefId> = self.defs.values().filter_map(|d| d.params.as_ref().map(|p| p.base)).collect();
        s.extend(self.rig_masters());
        s
    }
}
