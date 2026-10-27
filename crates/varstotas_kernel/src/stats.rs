//! Document statistics (Window → Document Info) and cleaning: what the scene shows (every placed
//! copy counted) next to what the file stores (each definition once), definitions and
//! materials nothing in the scene uses, [`Model::purge_unused`],
//! [`Model::delete_definition`], [`Model::rename_definition`] and [`Model::delete_material`].

use crate::model::Model;
use crate::{DefId, MaterialId, OpError, OpResult};
use std::collections::{HashMap, HashSet};
use varstotas_i18n::tr;

/// Geometry and instance counts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub faces: usize,
    /// Triangles the faces tessellate into (a face of n vertices with h holes: n − 2 + 2h).
    pub triangles: usize,
    pub edges: usize,
    /// Placed groups / components (instances).
    pub groups: usize,
    pub components: usize,
}

impl Counts {
    fn add(&mut self, o: &Counts) {
        self.faces += o.faces;
        self.triangles += o.triangles;
        self.edges += o.edges;
        self.groups += o.groups;
        self.components += o.components;
    }
}

/// Statistics of a model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelStats {
    /// What the scene shows: nested content counted once per placed copy.
    pub scene: Counts,
    /// What the file stores: every definition once (unused ones too).
    pub stored: Counts,
    /// Component definitions: all / used in the scene.
    pub component_defs: usize,
    pub component_defs_used: usize,
    /// Group definitions: all / used in the scene.
    pub group_defs: usize,
    pub group_defs_used: usize,
    /// Materials: all / used in the scene; textured ones and their image bytes.
    pub materials: usize,
    pub materials_used: usize,
    pub textures: usize,
    pub texture_bytes: usize,
    pub tags: usize,
    pub scenes: usize,
    pub sheets: usize,
    pub generators: usize,
    pub lights: usize,
    /// Guide lines and points (all contexts), dimensions.
    pub guides: usize,
    pub dims: usize,
}

/// What [`Model::purge_unused`] removed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Purged {
    pub components: usize,
    pub groups: usize,
    pub materials: usize,
}

impl Purged {
    pub fn is_empty(&self) -> bool {
        self.components + self.groups + self.materials == 0
    }
}

impl Model {
    /// Definitions the scene uses: reachable from the root through instances (and the base /
    /// variant of parametric groups). The root itself included.
    pub fn used_defs(&self) -> HashSet<DefId> {
        let mut seen = HashSet::new();
        let mut stack = vec![self.root];
        while let Some(d) = stack.pop() {
            let Some(def) = self.defs.get(d) else { continue };
            if !seen.insert(d) {
                continue;
            }
            stack.extend(def.mesh.instances.values().map(|i| i.def));
            if let Some(p) = &def.params {
                stack.push(p.base);
                stack.extend(p.variant);
            }
            stack.extend(def.rigged.as_ref().map(|r| r.master));
        }
        seen
    }

    /// Materials the scene uses: on faces and instances of the used definitions, and named by
    /// the values of generated components (a generator rebuilds with them).
    pub fn used_materials(&self) -> HashSet<MaterialId> {
        self.materials_of(&self.used_defs())
    }

    fn materials_of(&self, defs: &HashSet<DefId>) -> HashSet<MaterialId> {
        let mut out = HashSet::new();
        let mut names = HashSet::new();
        for d in defs.iter().filter_map(|d| self.defs.get(*d)) {
            for f in d.mesh.faces.values() {
                out.extend(f.front);
                out.extend(f.back);
            }
            out.extend(d.mesh.instances.values().filter_map(|i| i.material));
            if let Some(g) = &d.generated {
                names.extend(g.values.values().filter_map(|v| v.text()));
            }
        }
        out.extend(self.materials.iter().filter(|(_, m)| names.contains(m.name.as_str())).map(|(id, _)| id));
        out
    }

    /// Counts of definition `d`'s own content (instances not expanded).
    fn own_counts(&self, d: DefId) -> Counts {
        let Some(def) = self.defs.get(d) else { return Counts::default() };
        let m = &def.mesh;
        let mut c = Counts { faces: m.faces.len(), edges: m.edges.len(), ..Default::default() };
        c.triangles = m.faces.values().map(|f| f.loops().map(Vec::len).sum::<usize>().saturating_sub(2) + 2 * f.holes.len()).sum();
        for i in m.instances.values() {
            if self.defs.get(i.def).is_some_and(|x| x.is_group) {
                c.groups += 1;
            } else {
                c.components += 1;
            }
        }
        c
    }

    /// Counts of `d` with its instances expanded (memoised in `memo`).
    fn scene_counts(&self, d: DefId, memo: &mut HashMap<DefId, Counts>, depth: usize) -> Counts {
        if let Some(c) = memo.get(&d) {
            return *c;
        }
        let mut c = self.own_counts(d);
        if depth < 64
            && let Some(def) = self.defs.get(d)
        {
            for i in def.mesh.instances.values() {
                let k = self.scene_counts(i.def, memo, depth + 1);
                c.add(&k);
            }
        }
        memo.insert(d, c);
        c
    }

    /// Statistics of the model (see [`ModelStats`]).
    pub fn statistics(&self) -> ModelStats {
        let used = self.used_defs();
        let used_mats = self.materials_of(&used);
        let mut s = ModelStats { scene: self.scene_counts(self.root, &mut HashMap::new(), 0), ..Default::default() };
        for (id, d) in &self.defs {
            s.stored.add(&self.own_counts(id));
            // Variants and painted parts of rigs are counted with their masters.
            if id == self.root || d.rigged.is_some() {
                continue;
            }
            let u = used.contains(&id) as usize;
            if d.is_group {
                s.group_defs += 1;
                s.group_defs_used += u;
            } else {
                s.component_defs += 1;
                s.component_defs_used += u;
            }
            s.guides += d.guides.lines.len() + d.guides.points.len();
        }
        s.dims = self.defs.values().map(|d| d.mesh.dims.len()).sum();
        s.materials = self.materials.len();
        s.materials_used = used_mats.len();
        for m in self.materials.values() {
            if let Some(img) = &m.image {
                s.textures += 1;
                s.texture_bytes += img.bytes.len();
            }
        }
        s.tags = self.tags.len();
        s.scenes = self.scenes.len();
        s.sheets = self.sheets.len();
        s.generators = self.generators.len();
        s.lights = self.lights.len();
        s.guides += self.guides.len() + self.guide_points.len();
        s
    }

    /// Remove the definitions and materials the scene doesn't use (Purge Unused). Groups
    /// can't be placed again, so unused ones go too.
    pub fn purge_unused(&mut self, defs: bool, materials: bool) -> Purged {
        let mut p = Purged::default();
        if defs {
            let used = self.used_defs();
            let gone: Vec<DefId> = self.defs.keys().filter(|d| !used.contains(d)).collect();
            for d in gone {
                if self.defs.remove(d).is_some_and(|x| x.is_group) {
                    p.groups += 1;
                } else {
                    p.components += 1;
                }
            }
        }
        if materials {
            let used = self.used_materials();
            let before = self.materials.len();
            self.materials.retain(|id, _| used.contains(&id));
            p.materials = before - self.materials.len();
        }
        p
    }

    /// Remove definition `d` from the document: every placed copy (in any context), then the
    /// definition and the nested ones nothing else uses. Returns the number of copies removed.
    pub fn delete_definition(&mut self, d: DefId) -> OpResult<usize> {
        if d == self.root || !self.defs.contains_key(d) {
            return Err(OpError::NotFound);
        }
        // A component with parameters: its copies use its variants.
        let mut gone: HashSet<DefId> = self.rig_variants(d).into_iter().map(|(v, _)| v).collect();
        gone.insert(d);
        let mut removed = 0;
        for def in self.defs.values_mut() {
            let n = def.mesh.instances.len();
            def.mesh.instances.retain(|_, i| !gone.contains(&i.def));
            removed += n - def.mesh.instances.len();
        }
        self.remove_unused(gone);
        Ok(removed)
    }

    /// Remove material `m` from the document: faces and groups / components painted with it
    /// get the default material. Returns how many faces and objects used it.
    pub fn delete_material(&mut self, m: MaterialId) -> OpResult<usize> {
        if self.materials.remove(m).is_none() {
            return Err(OpError::NotFound);
        }
        let mut uses = 0;
        let clear = |slot: &mut Option<MaterialId>| {
            if *slot == Some(m) {
                *slot = None;
                1
            } else {
                0
            }
        };
        for def in self.defs.values_mut() {
            for f in def.mesh.faces.values_mut() {
                let n = clear(&mut f.front) + clear(&mut f.back);
                uses += n.min(1);
            }
            for i in def.mesh.instances.values_mut() {
                uses += clear(&mut i.material);
            }
        }
        Ok(uses)
    }

    /// Paint everything painted with `old` (faces, objects, every definition) with `new`
    /// instead and remove `old`. Returns the number of faces and objects repainted.
    pub fn replace_material(&mut self, old: MaterialId, new: MaterialId) -> OpResult<usize> {
        if old == new || !self.materials.contains_key(new) || self.materials.remove(old).is_none() {
            return Err(OpError::NotFound);
        }
        let mut uses = 0;
        let swap = |slot: &mut Option<MaterialId>| {
            if *slot == Some(old) {
                *slot = Some(new);
                1
            } else {
                0
            }
        };
        for def in self.defs.values_mut() {
            for f in def.mesh.faces.values_mut() {
                let n = swap(&mut f.front) + swap(&mut f.back);
                uses += n.min(1);
            }
            for i in def.mesh.instances.values_mut() {
                uses += swap(&mut i.material);
            }
        }
        Ok(uses)
    }

    /// Rename component `d` (a variant of a component with parameters renames its master);
    /// the variants of a master take the name too. Groups and the root can't be renamed.
    pub fn rename_definition(&mut self, d: DefId, name: &str) -> OpResult<()> {
        let name = name.trim();
        if name.is_empty() {
            return Err(OpError::Invalid(tr!("kernel-err-an-empty-name")));
        }
        let d = self.defs.get(d).and_then(|x| x.rigged.as_ref()).filter(|r| r.paint.is_none()).map_or(d, |r| r.master);
        if d == self.root || self.defs.get(d).is_none_or(|x| x.is_group) {
            return Err(OpError::NotFound);
        }
        for v in std::iter::once(d).chain(self.rig_variants(d).into_iter().map(|(v, _)| v)) {
            self.defs[v].name = name.to_string();
        }
        Ok(())
    }
}
