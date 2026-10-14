//! Internal clipboard for Copy / Cut / Paste / Paste in Place.
//!
//! The copied entities are stored as a self-contained definition inside a pruned clone of the
//! source model (the selection is "grouped" in the clone, so faces keep their holes, materials,
//! soft edges, nested groups and components). Pasting copies that definition (and the
//! definitions it uses) into the target model, inserts it as a temporary instance in the
//! target context and explodes it, so loose geometry merges with the context.
//! Guides are copied in world space.

use crate::document::Document;
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::{
    Aabb, DefId, EntityRef, GuideLine, GuideRef, Instance, InstanceId, MaterialId, Mesh, Model, OpError, OpResult,
    TagId,
};
use std::collections::{HashMap, HashSet};

/// Max preview segments drawn while a paste follows the cursor.
const PREVIEW_LIMIT: usize = 20_000;

pub struct Clipboard {
    /// Pruned clone of the source model holding the copied definition.
    model: Model,
    /// Definition with the copied entities (in the source context's local frame); None when
    /// only guides were copied.
    def: Option<DefId>,
    /// World transform of the source context.
    src_xf: DMat4,
    /// Identity of the source document (component definitions are shared when pasting back
    /// into the same document).
    src_doc: u64,
    guides: Vec<GuideLine>,
    guide_points: Vec<DVec3>,
    /// World bounds of the copied content.
    pub bounds: Aabb,
    /// World-space edges of the copied content at its original position (capped).
    pub preview: Vec<(DVec3, DVec3)>,
    /// Number of copied entities + guides (for messages).
    pub count: usize,
}

/// Definitions reachable from `d` (including `d`).
fn reachable_defs(model: &Model, d: DefId, out: &mut HashSet<DefId>) {
    if !out.insert(d) {
        return;
    }
    if let Some(def) = model.defs.get(d) {
        for inst in def.mesh.instances.values() {
            reachable_defs(model, inst.def, out);
        }
    }
}

fn collect_edges(model: &Model, d: DefId, m: DMat4, out: &mut Vec<(DVec3, DVec3)>, depth: usize) {
    if depth > 32 || out.len() >= PREVIEW_LIMIT {
        return;
    }
    let Some(def) = model.defs.get(d) else { return };
    for (id, e) in def.mesh.edges.iter() {
        if out.len() >= PREVIEW_LIMIT {
            return;
        }
        if e.hidden {
            continue;
        }
        let (a, b) = def.mesh.edge_segment(id);
        out.push((m.transform_point3(a), m.transform_point3(b)));
    }
    for inst in def.mesh.instances.values() {
        if !inst.hidden {
            collect_edges(model, inst.def, m * inst.transform, out, depth + 1);
        }
    }
}

impl Clipboard {
    /// Copy the document's selection (entities of the active context + selected guides).
    pub fn from_selection(doc: &Document) -> OpResult<Clipboard> {
        if !doc.has_selection() {
            return Err(OpError::Invalid("nothing selected".into()));
        }
        let mut model = doc.model.clone();
        let path = doc.active_path.clone();
        let src_xf = model.transform_at(&path);
        let mut def = None;
        let mut count = doc.guide_sel.len();
        let ents: Vec<EntityRef> =
            doc.selection.iter().copied().filter(|e| !matches!(e, EntityRef::Vertex(_))).collect();
        if !ents.is_empty() {
            count += ents.len();
            let inst = model.make_group(&path, &ents)?;
            let d = model.mesh_at(&path).and_then(|m| m.instances.get(inst)).map(|i| i.def).ok_or(OpError::NotFound)?;
            def = Some(d);
        }
        // Keep only what the copied definition needs (the root stays, emptied).
        let mut keep = HashSet::new();
        if let Some(d) = def {
            reachable_defs(&model, d, &mut keep);
        }
        let root = model.root;
        let drop: Vec<DefId> = model.defs.keys().filter(|k| *k != root && !keep.contains(k)).collect();
        for k in drop {
            model.defs.remove(k);
        }
        if !keep.contains(&root) {
            model.defs[root].mesh = Mesh::new();
        }
        let guides: Vec<GuideLine> =
            doc.guide_sel.iter().filter_map(|g| if let GuideRef::Line(id) = g { model.guides.get(*id).copied() } else { None }).collect();
        let guide_points: Vec<DVec3> = doc
            .guide_sel
            .iter()
            .filter_map(|g| if let GuideRef::Point(id) = g { model.guide_points.get(*id).copied() } else { None })
            .collect();
        model.delete_all_guides();

        let mut pts: Vec<DVec3> = guide_points.clone();
        if let Some(b) = def.and_then(|d| model.def_bounds(d)) {
            pts.extend(b.transformed(&src_xf).corners());
        }
        if pts.is_empty() {
            pts.extend(guides.iter().map(|g| g.origin));
        }
        let bounds = Aabb::from_points(pts).ok_or(OpError::Invalid("nothing to copy".into()))?;
        let mut preview = Vec::new();
        if let Some(d) = def {
            collect_edges(&model, d, src_xf, &mut preview, 0);
        }
        Ok(Clipboard { model, def, src_xf, src_doc: doc.uid, guides, guide_points, bounds, preview, count })
    }

    /// Point of the content that follows the cursor while pasting (bounding-box min corner).
    pub fn reference_point(&self) -> DVec3 {
        self.bounds.min
    }

    pub fn guide_lines(&self) -> &[GuideLine] {
        &self.guides
    }

    pub fn guide_points(&self) -> &[DVec3] {
        &self.guide_points
    }

    /// Paste into the context `path` of `model` (from document `doc_uid`), moved by the world
    /// transform `t_world` (identity = Paste in Place). Returns the new entities and guides.
    pub fn paste_into(
        &self,
        model: &mut Model,
        doc_uid: u64,
        path: &[InstanceId],
        t_world: &DMat4,
    ) -> OpResult<(Vec<EntityRef>, Vec<GuideRef>)> {
        let mut ents = Vec::new();
        if let Some(d) = self.def {
            let mut maps = Maps::default();
            let nd = self.copy_def(model, doc_uid, d, &mut maps, true)?;
            let ctx_xf = model.transform_at(path);
            let transform = ctx_xf.inverse() * *t_world * self.src_xf;
            let inst = Instance {
                def: nd,
                transform,
                name: String::new(),
                material: None,
                tag: None,
                hidden: false,
                locked: false,
            };
            let iid = model.mesh_at_mut(path).ok_or(OpError::NotFound)?.instances.insert(inst);
            ents = model.explode(path, iid)?;
            // Explode may report a face/edge more than once (shared or welded).
            let mut seen = HashSet::new();
            ents.retain(|e| seen.insert(*e));
        }
        let mut guides = Vec::new();
        for g in &self.guides {
            guides.push(GuideRef::Line(model.add_guide_line(g.transformed(t_world))));
        }
        for p in &self.guide_points {
            guides.push(GuideRef::Point(model.add_guide_point(t_world.transform_point3(*p))));
        }
        Ok((ents, guides))
    }

    /// Copy definition `d` of the clipboard model into `model`; returns the new id.
    /// `top` = the copied selection itself (always copied; it is exploded after insertion).
    fn copy_def(&self, model: &mut Model, doc_uid: u64, d: DefId, maps: &mut Maps, top: bool) -> OpResult<DefId> {
        if let Some(n) = maps.defs.get(&d) {
            return Ok(*n);
        }
        let src = self.model.defs.get(d).ok_or(OpError::NotFound)?;
        // Same document: paste another instance of an existing component definition.
        if !top
            && doc_uid == self.src_doc
            && !src.is_group
            && model.defs.get(d).is_some_and(|t| !t.is_group && t.name == src.name)
        {
            maps.defs.insert(d, d);
            return Ok(d);
        }
        let mut def = src.clone();
        let children: Vec<(InstanceId, DefId)> = def.mesh.instances.iter().map(|(i, x)| (i, x.def)).collect();
        for (i, cd) in children {
            let nd = self.copy_def(model, doc_uid, cd, maps, false)?;
            if let Some(inst) = def.mesh.instances.get_mut(i) {
                inst.def = nd;
                inst.material = inst.material.map(|m| maps.material(&self.model, model, m));
                inst.tag = inst.tag.map(|t| maps.tag(&self.model, model, t));
            }
        }
        for f in def.mesh.faces.values_mut() {
            f.front = f.front.map(|m| maps.material(&self.model, model, m));
            f.back = f.back.map(|m| maps.material(&self.model, model, m));
            f.tag = f.tag.map(|t| maps.tag(&self.model, model, t));
        }
        for e in def.mesh.edges.values_mut() {
            e.tag = e.tag.map(|t| maps.tag(&self.model, model, t));
        }
        let nd = model.defs.insert(def);
        maps.defs.insert(d, nd);
        Ok(nd)
    }
}

/// Clipboard-model id → target-model id maps for one paste.
#[derive(Default)]
struct Maps {
    defs: HashMap<DefId, DefId>,
    materials: HashMap<MaterialId, MaterialId>,
    tags: HashMap<TagId, TagId>,
}

impl Maps {
    /// Same-named material of the target (added if missing).
    fn material(&mut self, src: &Model, dst: &mut Model, m: MaterialId) -> MaterialId {
        if let Some(x) = self.materials.get(&m) {
            return *x;
        }
        let Some(mat) = src.materials.get(m) else { return m };
        let id = match dst.materials.iter().find(|(_, x)| x.name == mat.name) {
            Some((id, _)) => id,
            None => dst.materials.insert(mat.clone()),
        };
        self.materials.insert(m, id);
        id
    }

    fn tag(&mut self, src: &Model, dst: &mut Model, t: TagId) -> TagId {
        if let Some(x) = self.tags.get(&t) {
            return *x;
        }
        let Some(tag) = src.tags.get(t) else { return t };
        let id = match dst.tags.iter().find(|(_, x)| x.name == tag.name) {
            Some((id, _)) => id,
            None => dst.tags.insert(tag.clone()),
        };
        self.tags.insert(t, id);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boxed_doc() -> Document {
        let mut doc = Document::new();
        doc.commit("box", |m| {
            let red = m.add_material("Red", [1.0, 0.0, 0.0, 1.0]);
            let mesh = m.root_mesh_mut();
            let f = mesh
                .add_face(&[
                    DVec3::new(0.0, 0.0, 0.0),
                    DVec3::new(1000.0, 0.0, 0.0),
                    DVec3::new(1000.0, 1000.0, 0.0),
                    DVec3::new(0.0, 1000.0, 0.0),
                ])
                .unwrap();
            let _ = mesh.push_pull(f, -500.0, false);
            for f in mesh.faces.values_mut() {
                f.front = Some(red);
            }
        });
        doc
    }

    #[test]
    fn copy_paste_loose_geometry_and_in_place() {
        let mut doc = boxed_doc();
        doc.select_all();
        let n_faces = doc.model.root_mesh().faces.len();
        let clip = Clipboard::from_selection(&doc).unwrap();
        assert_eq!(clip.bounds.min, DVec3::ZERO);
        assert!(clip.preview.len() >= 12);
        // The source document is untouched by copying.
        assert_eq!(doc.model.root_mesh().faces.len(), n_faces);

        // Paste 3 m to the right: a second, separate box.
        let t = DMat4::from_translation(DVec3::new(3000.0, 0.0, 0.0));
        let uid = doc.uid;
        let (ents, _) = doc.commit("Paste", |m| clip.paste_into(m, uid, &[], &t)).unwrap().unwrap();
        let faces = doc.model.root_mesh().faces.len();
        assert_eq!(faces, n_faces * 2);
        assert_eq!(ents.iter().filter(|e| matches!(e, EntityRef::Face(_))).count(), n_faces);
        let b = doc.model.bounds().unwrap();
        assert!((b.max.x - 4000.0).abs() < 1e-6, "{b:?}");
        // Material kept (same name, no duplicate material).
        assert_eq!(doc.model.materials.len(), 1);
        assert!(doc.model.root_mesh().faces.values().all(|f| f.front.is_some()));

        // Paste into a fresh document keeps the material by name.
        let mut other = Document::new();
        let ouid = other.uid;
        other.commit("Paste", |m| clip.paste_into(m, ouid, &[], &DMat4::IDENTITY)).unwrap().unwrap();
        assert_eq!(other.model.root_mesh().faces.len(), n_faces);
        assert_eq!(other.model.materials.values().next().map(|m| m.name.as_str()), Some("Red"));
    }

    #[test]
    fn copy_group_pastes_a_group_and_into_open_context() {
        let mut doc = boxed_doc();
        doc.select_all();
        let sel = doc.selection.clone();
        let g = doc.try_commit("Make Group", |m| m.make_group(&[], &sel)).unwrap();
        doc.set_selection(vec![EntityRef::Instance(g)]);
        let clip = Clipboard::from_selection(&doc).unwrap();
        let uid = doc.uid;
        let (ents, _) = doc
            .commit("Paste", |m| clip.paste_into(m, uid, &[], &DMat4::from_translation(DVec3::Y * 2000.0)))
            .unwrap()
            .unwrap();
        assert_eq!(ents.len(), 1);
        assert!(matches!(ents[0], EntityRef::Instance(_)));
        assert_eq!(doc.model.root_mesh().instances.len(), 2);
        // Paste in place inside the open group: loose geometry lands in the group's mesh.
        doc.set_active_path(vec![g]);
        let mut d2 = boxed_doc();
        d2.select_all();
        let clip2 = Clipboard::from_selection(&d2).unwrap();
        let before = doc.model.mesh_at(&[g]).unwrap().faces.len();
        doc.commit("Paste", |m| clip2.paste_into(m, uid, &[g], &DMat4::from_translation(DVec3::Z * 5000.0)))
            .unwrap()
            .unwrap();
        assert_eq!(doc.model.mesh_at(&[g]).unwrap().faces.len(), before * 2);
    }

    #[test]
    fn copy_guides_only() {
        let mut doc = Document::new();
        let gid = doc.commit("g", |m| m.add_guide_line(GuideLine { origin: DVec3::new(0.0, 500.0, 0.0), dir: DVec3::X }));
        let pid = doc.commit("p", |m| m.add_guide_point(DVec3::new(10.0, 20.0, 30.0)));
        doc.set_selection_with_guides(Vec::new(), vec![GuideRef::Line(gid.unwrap()), GuideRef::Point(pid.unwrap())]);
        let clip = Clipboard::from_selection(&doc).unwrap();
        assert_eq!(clip.count, 2);
        let uid = doc.uid;
        let (_, gs) = doc
            .commit("Paste", |m| clip.paste_into(m, uid, &[], &DMat4::from_translation(DVec3::Z * 100.0)))
            .unwrap()
            .unwrap();
        assert_eq!(gs.len(), 2);
        assert_eq!(doc.model.guides.len(), 2);
        assert!(doc.model.guide_points.values().any(|p| (*p - DVec3::new(10.0, 20.0, 130.0)).length() < 1e-9));
    }
}
