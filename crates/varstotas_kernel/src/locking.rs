//! Lock queries and atomic validation shared by the UI, scripts, plugins and bridge.
//! A caller must validate against the ORIGINAL model before accepting a mutation.
use crate::{DefId, EntityRef, InstanceId, Model, TagId};
use serde::Serialize;
use std::collections::HashSet;
use varstotas_i18n::tr;

fn value<T: Serialize>(v: &T) -> serde_json::Value {
    let mut v = serde_json::to_value(v).expect("serializable model data");
    fn clean(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
                // Visibility is a display setting, independent of editing locks.
                m.remove("hidden");
                for v in m.values_mut() {
                    clean(v);
                }
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    clean(v);
                }
            }
            _ => {}
        }
    }
    clean(&mut v);
    v
}

impl Model {
    pub fn tag_locked(&self, d: DefId, tag: Option<TagId>) -> bool {
        match tag {
            Some(t) => self.tags.get(t).is_some_and(|t| t.locked),
            None => d == self.root && self.untagged_locked,
        }
    }

    pub fn entity_locked_in(&self, d: DefId, e: EntityRef) -> bool {
        let Some(def) = self.defs.get(d) else {
            return true;
        };
        let m = &def.mesh;
        match e {
            EntityRef::Instance(i) => m
                .instances
                .get(i)
                .is_none_or(|i| i.locked || self.tag_locked(d, i.tag)),
            EntityRef::Face(f) => m
                .faces
                .get(f)
                .is_none_or(|f| f.locked || self.tag_locked(d, f.tag)),
            EntityRef::Edge(e) => {
                m.edges
                    .get(e)
                    .is_none_or(|e| e.locked || self.tag_locked(d, e.tag))
                    || m.edge_faces(e)
                        .iter()
                        .any(|f| self.entity_locked_in(d, EntityRef::Face(*f)))
            }
            EntityRef::Vertex(v) => m
                .edges
                .iter()
                .any(|(e, edge)| edge.has(v) && self.entity_locked_in(d, EntityRef::Edge(e))),
        }
    }

    /// Includes ancestor locks and shared definitions reached through another locked copy.
    pub fn context_locked(&self, path: &[InstanceId]) -> bool {
        let mut d = self.root;
        for id in path {
            if self.entity_locked_in(d, EntityRef::Instance(*id)) {
                return true;
            }
            let Some(i) = self.defs.get(d).and_then(|d| d.mesh.instances.get(*id)) else {
                return true;
            };
            d = i.def;
        }
        if d == self.root {
            return false;
        }
        self.locked_definitions().contains(&d)
    }

    pub fn entity_locked(&self, path: &[InstanceId], e: EntityRef) -> bool {
        self.context_locked(path)
            || self
                .def_at(path)
                .is_none_or(|d| self.entity_locked_in(d, e))
    }

    fn locked_definitions(&self) -> HashSet<DefId> {
        let mut out = HashSet::new();
        let mut pending = Vec::new();
        for (d, def) in &self.defs {
            for (id, i) in &def.mesh.instances {
                if self.entity_locked_in(d, EntityRef::Instance(id)) {
                    pending.push(i.def);
                }
            }
        }
        while let Some(d) = pending.pop() {
            if out.insert(d) && let Some(def) = self.defs.get(d) {
                pending.extend(def.mesh.instances.values().map(|i| i.def));
                pending.extend(def.params.as_ref().map(|p| p.base));
                pending.extend(def.rigged.as_ref().map(|r| r.master));
            }
        }
        out
    }

    /// Reject ANY change to protected content, including indirect changes through shared
    /// definitions/materials, ancestor transforms, tag reassignment, deletion and regeneration.
    /// Unlocking is deliberately outside this API: only explicit UI lock controls may do it.
    pub fn has_locks(&self) -> bool {
        self.untagged_locked
            || self.tags.values().any(|t| t.locked)
            || self.defs.values().any(|d| {
                d.mesh.instances.values().any(|i| i.locked)
                    || d.mesh.faces.values().any(|f| f.locked)
                    || d.mesh.edges.values().any(|e| e.locked)
            })
    }

    pub fn validate_locked_changes(&self, after: &Model) -> Result<(), String> {
        if !self.has_locks() {
            return Ok(());
        }
        let fail = || Err(tr!("edit-locked"));
        if self.root != after.root || self.untagged_locked != after.untagged_locked {
            return fail();
        }
        for (id, tag) in &self.tags {
            if after.tags.get(id).is_none_or(|t| t.locked != tag.locked) && tag.locked {
                return fail();
            }
        }
        let protected = self.locked_definitions();
        let mut contains = HashSet::new();
        // An ancestor cannot move/disappear while it contains protected geometry.
        for (d, def) in &self.defs {
            if def.mesh.instances.iter().any(|(i, _)| self.entity_locked_in(d, EntityRef::Instance(i)))
                || def
                .mesh
                .faces
                .iter()
                .any(|(f, _)| self.entity_locked_in(d, EntityRef::Face(f)))
                || def
                    .mesh
                    .edges
                    .iter()
                    .any(|(e, _)| self.entity_locked_in(d, EntityRef::Edge(e)))
            {
                contains.insert(d);
            }
        }
        loop {
            let n = contains.len();
            for (d, def) in &self.defs {
                if def
                    .mesh
                    .instances
                    .values()
                    .any(|i| contains.contains(&i.def))
                {
                    contains.insert(d);
                }
            }
            if n == contains.len() {
                break;
            }
        }
        let mut materials = HashSet::new();
        for (d, def) in &self.defs {
            let full = protected.contains(&d);
            let other = after.defs.get(d);
            if full && other.is_none_or(|v| value(def) != value(v)) {
                return fail();
            }
            if full && let Some(g) = &def.generated
                && value(&self.generators.get(&g.generator))
                    != value(&after.generators.get(&g.generator))
            {
                return fail();
            }
            if full {
                for i in def.mesh.instances.values() {
                    materials.extend(i.material);
                }
                for f in def.mesh.faces.values() {
                    materials.extend(f.front);
                    materials.extend(f.back);
                }
                continue;
            }
            for (id, inst) in &def.mesh.instances {
                if full
                    || self.entity_locked_in(d, EntityRef::Instance(id))
                    || contains.contains(&inst.def)
                {
                    if other
                        .and_then(|v| v.mesh.instances.get(id))
                        .is_none_or(|v| value(inst) != value(v))
                    {
                        return fail();
                    }
                    materials.extend(inst.material);
                }
            }
            for (id, face) in &def.mesh.faces {
                if full || self.entity_locked_in(d, EntityRef::Face(id)) {
                    if other
                        .and_then(|v| v.mesh.faces.get(id))
                        .is_none_or(|v| value(face) != value(v))
                    {
                        return fail();
                    }
                    materials.extend(face.front);
                    materials.extend(face.back);
                    for v in face.loops().flatten() {
                        if value(&def.mesh.vertices.get(*v))
                            != value(&other.and_then(|d| d.mesh.vertices.get(*v)))
                        {
                            return fail();
                        }
                    }
                }
            }
            for (id, edge) in &def.mesh.edges {
                if full || self.entity_locked_in(d, EntityRef::Edge(id)) {
                    if other
                        .and_then(|v| v.mesh.edges.get(id))
                        .is_none_or(|v| value(edge) != value(v))
                    {
                        return fail();
                    }
                    for v in edge.v {
                        if value(&def.mesh.vertices.get(v))
                            != value(&other.and_then(|d| d.mesh.vertices.get(v)))
                        {
                            return fail();
                        }
                    }
                    if let Some(c) = edge.curve
                        && (value(&def.mesh.curves.get(&c))
                            != value(&other.and_then(|d| d.mesh.curves.get(&c)))
                            || value(&def.mesh.splines.get(&c))
                                != value(&other.and_then(|d| d.mesh.splines.get(&c))))
                    {
                        return fail();
                    }
                }
            }
        }
        for m in materials {
            if value(&self.materials.get(m)) != value(&after.materials.get(m)) {
                return fail();
            }
        }
        // Existing locked tags cannot receive newly inserted/reassigned geometry either.
        for (d, def) in &after.defs {
            let old = self.defs.get(d);
            for (id, f) in &def.mesh.faces {
                if self.tag_locked(d, f.tag)
                    && value(&old.and_then(|d| d.mesh.faces.get(id))) != value(&Some(f))
                {
                    return fail();
                }
            }
            for (id, e) in &def.mesh.edges {
                if self.tag_locked(d, e.tag)
                    && value(&old.and_then(|d| d.mesh.edges.get(id))) != value(&Some(e))
                {
                    return fail();
                }
            }
            for (id, i) in &def.mesh.instances {
                if self.tag_locked(d, i.tag)
                    && value(&old.and_then(|d| d.mesh.instances.get(id))) != value(&Some(i))
                {
                    return fail();
                }
            }
        }
        Ok(())
    }

    pub fn set_entities_locked(
        &mut self,
        path: &[InstanceId],
        entities: &[EntityRef],
        locked: bool,
    ) {
        if let Some(m) = self.mesh_at_mut(path) {
            for e in entities {
                match *e {
                    EntityRef::Face(id) => {
                        if let Some(f) = m.faces.get_mut(id) {
                            f.locked = locked;
                        }
                    }
                    EntityRef::Edge(id) => {
                        if let Some(e) = m.edges.get_mut(id) {
                            e.locked = locked;
                        }
                    }
                    EntityRef::Instance(id) => {
                        if let Some(i) = m.instances.get_mut(id) {
                            i.locked = locked;
                        }
                    }
                    EntityRef::Vertex(_) => {}
                }
            }
        }
    }

    pub fn unlock_all(&mut self) {
        self.untagged_locked = false;
        for t in self.tags.values_mut() {
            t.locked = false;
        }
        for d in self.defs.values_mut() {
            for f in d.mesh.faces.values_mut() {
                f.locked = false;
            }
            for e in d.mesh.edges.values_mut() {
                e.locked = false;
            }
            for i in d.mesh.instances.values_mut() {
                i.locked = false;
            }
        }
    }
}
