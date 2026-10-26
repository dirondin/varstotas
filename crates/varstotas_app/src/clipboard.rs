//! Internal clipboard for Copy / Cut / Paste / Paste in Place.
//!
//! The copied entities are stored as a self-contained definition inside a pruned clone of the
//! source model (the selection is "grouped" in the clone, so faces keep their holes, materials,
//! soft edges, nested groups and components). Pasting copies that definition (and the
//! definitions it uses) into the target model, inserts it as a temporary instance in the
//! target context and explodes it, so loose geometry merges with the context.
//! Guides are copied in world space.

use varstotas_i18n::tr;
use crate::document::Document;
use varstotas_kernel::glam::{DMat4, DVec3};
use varstotas_kernel::{
    Aabb, DefId, EntityRef, GuideGeom, GuideLine, GuideRef, Instance, InstanceId, MaterialId, Mesh, Model, OpError, OpResult,
    TagId,
};
use std::collections::{HashMap, HashSet};

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
    /// Selected dimensions (world space).
    dims: Vec<varstotas_kernel::DimGeom>,
    /// World bounds of the copied content.
    pub bounds: Aabb,
    /// World-space edges of the copied content at its original position.
    pub preview: Vec<(DVec3, DVec3)>,
    /// Number of copied entities + guides (for messages).
    pub count: usize,
    /// Library component: the name of its definition; a definition with that name already
    /// in the target model is placed again instead of copied.
    library: Option<String>,
}

/// Source "document" id of library components (never a real document).
const LIBRARY_UID: u64 = u64::MAX;

/// Definitions reachable from `d` (including `d`).
fn reachable_defs(model: &Model, d: DefId, out: &mut HashSet<DefId>) {
    if !out.insert(d) {
        return;
    }
    if let Some(def) = model.defs.get(d) {
        for inst in def.mesh.instances.values() {
            reachable_defs(model, inst.def, out);
        }
        // A parametric group's base is kept even when only its stretched variant is placed.
        if let Some(p) = &def.params {
            reachable_defs(model, p.base, out);
        }
        // A component with parameters: its master (variants and painted parts are posed from it).
        if let Some(r) = &def.rigged {
            reachable_defs(model, r.master, out);
        }
    }
}

/// Edges of definition `d` in its own frame (thumbnails).
pub fn def_edges(model: &Model, d: DefId, max: usize) -> Vec<(DVec3, DVec3)> {
    let mut out = Vec::new();
    collect_edges(model, d, DMat4::IDENTITY, &mut out, 0);
    sample_edges(&out, max)
}

/// Even coverage for small thumbnails; working previews always use the complete mesh.
pub fn sample_edges(edges: &[(DVec3,DVec3)], max: usize) -> Vec<(DVec3,DVec3)> {
    let n=edges.len().min(max);
    (0..n).map(|i|edges[i*edges.len()/n]).collect()
}

fn collect_edges(model: &Model, d: DefId, m: DMat4, out: &mut Vec<(DVec3, DVec3)>, depth: usize) {
    if depth > 64 {
        return;
    }
    let Some(def) = model.defs.get(d) else { return };
    for (id, e) in def.mesh.edges.iter() {
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
    /// The credits of downloaded models in the clipboard (licence warnings on placement).
    pub fn credits(&self) -> Vec<varstotas_kernel::Attribution> {
        self.model.attributions()
    }

    /// Copy the document's selection (entities of the active context + selected guides).
    pub fn from_selection(doc: &Document) -> OpResult<Clipboard> {
        if !doc.has_selection() {
            return Err(OpError::Invalid(tr!("edit-nothing-selected")));
        }
        let dims: Vec<varstotas_kernel::DimGeom> =
            doc.dim_sel.iter().filter_map(|d| doc.model.dimension_in(&doc.active_path, *d)).collect();
        if doc.selection.is_empty() && doc.guide_sel.is_empty() {
            // Dimensions only.
            let pts: Vec<DVec3> = dims.iter().flat_map(|g| [g.a, g.b, g.line().0, g.line().1]).collect();
            let bounds = Aabb::from_points(pts).ok_or(OpError::Invalid(tr!("edit-nothing-to-copy")))?;
            let preview = dims.iter().map(|g| g.line()).collect();
            let mut model = doc.model.clone();
            let root = model.root;
            model.defs.retain(|k, _| k == root);
            model.defs[root].mesh = Mesh::new();
            model.guides.clear();
            model.guide_points.clear();
            let count = dims.len();
            return Ok(Clipboard {
                model,
                def: None,
                src_xf: DMat4::IDENTITY,
                src_doc: doc.uid,
                guides: Vec::new(),
                guide_points: Vec::new(),
                dims,
                bounds,
                preview,
                count,
                library: None,
            });
        }
        let mut c = Self::from_parts(&doc.model, doc.uid, &doc.active_path, &doc.selection, &doc.guide_sel)?;
        c.count += dims.len();
        c.dims = dims;
        Ok(c)
    }

    /// Copy entities `ents` of the context `path` of `src` (document `src_doc`) and `guide_sel`.
    pub fn from_parts(
        src: &Model,
        src_doc: u64,
        path: &[InstanceId],
        ents: &[EntityRef],
        guide_sel: &[GuideRef],
    ) -> OpResult<Clipboard> {
        let mut model = src.clone();
        let src_xf = model.transform_at(path);
        let mut def = None;
        let mut count = guide_sel.len();
        let alive = model.mesh_at(path).ok_or(OpError::NotFound)?;
        let ents: Vec<EntityRef> = ents
            .iter()
            .copied()
            .filter(|e| !matches!(e, EntityRef::Vertex(_)) && crate::document::entity_alive(alive, *e))
            .collect();
        if !ents.is_empty() {
            count += ents.len();
            let inst = model.make_group(path, &ents)?;
            let d = model.mesh_at(path).and_then(|m| m.instances.get(inst)).map(|i| i.def).ok_or(OpError::NotFound)?;
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
        // Guides in world space (the selected ones belong to the context `path`).
        let mut guides = Vec::new();
        let mut guide_points = Vec::new();
        for g in guide_sel {
            match src.guide_geom_in(path, *g) {
                Some(GuideGeom::Line(l)) => guides.push(l),
                Some(GuideGeom::Point(p)) => guide_points.push(p),
                None => {}
            }
        }
        // Model-level guides are not part of the copy (guides inside copied groups are).
        model.guides.clear();
        model.guide_points.clear();

        let mut pts: Vec<DVec3> = guide_points.clone();
        if let Some(b) = def.and_then(|d| model.def_bounds(d)) {
            pts.extend(b.transformed(&src_xf).corners());
        }
        if pts.is_empty() {
            pts.extend(guides.iter().map(|g| g.origin));
        }
        let bounds = Aabb::from_points(pts).ok_or(OpError::Invalid(tr!("edit-nothing-to-copy")))?;
        let mut preview = Vec::new();
        if let Some(d) = def {
            collect_edges(&model, d, src_xf, &mut preview, 0);
        }
        Ok(Clipboard { model, def, src_xf, src_doc, guides, guide_points, dims: Vec::new(), bounds, preview, count, library: None })
    }

    /// One more instance of definition `d` of document `doc_uid` (the Components list's
    /// Document folder): pasted back into that document it shares the definition.
    pub fn of_definition(model: &Model, doc_uid: u64, d: DefId) -> OpResult<Clipboard> {
        let mut m = model.clone();
        let root = m.root;
        m.defs[root].mesh = Mesh::new();
        m.guides.clear();
        m.guide_points.clear();
        let inst = Instance { def: d, transform: DMat4::IDENTITY, name: String::new(), material: None, tag: None, hidden: false, locked: false };
        let i = m.root_mesh_mut().instances.insert(inst);
        Self::from_parts(&m, doc_uid, &[], &[EntityRef::Instance(i)], &[])
    }

    /// A library file's content as one component named `name`, ready to place.
    pub fn from_library(mut model: Model, name: &str) -> OpResult<Clipboard> {
        let root = model.root_mesh();
        let ents: Vec<EntityRef> = root
            .faces
            .keys()
            .map(EntityRef::Face)
            .chain(root.edges.keys().map(EntityRef::Edge))
            .chain(root.instances.keys().map(EntityRef::Instance))
            .collect();
        if ents.is_empty() {
            return Err(OpError::Invalid(tr!("edit-component-file-empty")));
        }
        // A file holding just one component: place that one.
        let inst = match (ents.as_slice(), root.faces.is_empty() && root.edges.is_empty()) {
            ([EntityRef::Instance(i)], true) if model.defs.get(root.instances[*i].def).is_some_and(|d| !d.is_group && d.name == name) => *i,
            // A component with parameters keeps them whatever the file is called.
            ([EntityRef::Instance(i)], true) if model.instance_rigged(&[], *i).is_some() => *i,
            _ => model.make_component(&[], &ents, name)?,
        };
        // The file's picture goes with the component (its tile in the document shows it).
        let d = model.root_mesh().instances[inst].def;
        if let Some(def) = model.defs.get_mut(d)
            && def.preview.is_none()
        {
            def.preview = model.preview.clone();
        }
        model.guides.clear();
        model.guide_points.clear();
        let mut c = Self::from_parts(&model, LIBRARY_UID, &[], &[EntityRef::Instance(inst)], &[])?;
        c.library = Some(name.to_string());
        Ok(c)
    }

    /// The copied content as a model of its own, moved so that its bounding box starts at the
    /// origin (Save to Library). A single copied group or component is exploded into the file.
    pub fn to_model(&self, unit: varstotas_kernel::LengthUnit) -> OpResult<Model> {
        let mut m = Model::new();
        m.unit = unit;
        let t = DMat4::from_translation(-self.bounds.min);
        self.paste_into(&mut m, LIBRARY_UID - 1, &[], &t)?;
        let root = m.root_mesh();
        if root.faces.is_empty() && root.edges.is_empty() && root.instances.len() == 1 {
            let i = root.instances.keys().next().expect("one instance");
            // A component's own picture becomes the file's.
            m.preview = m.defs.get(root.instances[i].def).and_then(|d| d.preview.clone());
            // A component with parameters stays one (its master and variants in the file).
            if m.instance_rigged(&[], i).is_none() {
                m.explode(&[], i)?;
            }
        }
        m.purge_unused_defs();
        Ok(m)
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
            // Explode may report a face/edge more than once (shared or welded), or one that a
            // later weld of the same explode merged away.
            let mut seen = HashSet::new();
            let mesh = model.mesh_at(path).ok_or(OpError::NotFound)?;
            ents.retain(|e| crate::document::entity_alive(mesh, *e) && seen.insert(*e));
        }
        let geoms = self
            .guides
            .iter()
            .map(|g| GuideGeom::Line(*g))
            .chain(self.guide_points.iter().map(|p| GuideGeom::Point(*p)));
        let guides = geoms.filter_map(|g| model.add_guide_in(path, g.transformed(t_world))).collect();
        Ok((ents, guides))
    }

    /// Paste the copied dimensions into the context `path` moved by `t_world` (after
    /// [`Self::paste_into`], so their anchors attach to the pasted vertices).
    pub fn paste_dims_into(&self, model: &mut Model, path: &[InstanceId], t_world: &DMat4) -> Vec<varstotas_kernel::DimId> {
        let mut out = Vec::new();
        for g in &self.dims {
            let (a, b) = (t_world.transform_point3(g.a), t_world.transform_point3(g.b));
            let off = t_world.transform_vector3(g.offset);
            if let Some(id) = model.add_dimension_in(path, a, b, off) {
                if g.text.is_some() {
                    model.set_dimension_text_in(path, id, g.text.clone());
                }
                out.push(id);
            }
        }
        out
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
        // A variant of a component with parameters (or a painted part of one): the master comes
        // along, equal values share the target's variant.
        let mut rig_master = None;
        if let Some(r) = &src.rigged {
            let nm = self.copy_def(model, doc_uid, r.master, maps, false)?;
            let found = model.defs.iter().find(|(_, t)| t.rigged.as_ref().is_some_and(|x| x.master == nm && x.paint == r.paint && x.values == r.values));
            if !top && let Some((id, _)) = found {
                maps.defs.insert(d, id);
                return Ok(id);
            }
            rig_master = Some(nm);
        }
        // A library component placed again: its definition (for a component with parameters,
        // its master) is reused while it still has the library's content — a copy changed in
        // the model leaves it be, and the library's original comes in as a new definition.
        let from_library = !top && !src.is_group && src.rigged.is_none() && src.generated.is_none() && self.src_doc == LIBRARY_UID;
        if from_library && (src.rig.is_some() || self.library.as_deref() == Some(src.name.as_str())) {
            let h = self.model.content_hash(d);
            if let Some((id, _)) =
                model.defs.iter().find(|(id, t)| !t.is_group && t.rigged.is_none() && same_or_numbered(&t.name, &src.name) && model.content_hash(*id) == h)
            {
                maps.defs.insert(d, id);
                return Ok(id);
            }
        }
        // A generated component: its generator comes along. The target's generator with the
        // same script (by that name, or a copy of it "<name> 2") is used, and equal values share
        // its definition; when the target's one was edited, this script comes in beside it as
        // "<name> 2".
        let mut gen_name = None;
        if let Some(g) = &src.generated
            && let Some(gen_src) = self.model.generators.get(&g.generator)
        {
            let same_script = |t: &varstotas_kernel::Generator| t.source == varstotas_kernel::generator::renamed_source(&gen_src.source, &t.name);
            let same = match model.generators.get(&g.generator).filter(|t| same_script(t)) {
                Some(t) => Some(t.name.clone()),
                None => model.generators.values().find(|t| same_or_numbered(&t.name, &g.generator) && same_script(t)).map(|t| t.name.clone()),
            };
            let name = match same {
                Some(n) => n,
                None if !model.generators.contains_key(&g.generator) => {
                    model.add_generator(gen_src.clone());
                    g.generator.clone()
                }
                None => {
                    let n = varstotas_kernel::model::free_name(model.generators.keys().map(|k| k.as_str()).collect(), &g.generator);
                    let copy = varstotas_kernel::Generator::parse(&varstotas_kernel::generator::renamed_source(&gen_src.source, &n)).map_err(OpError::Invalid)?;
                    model.add_generator(copy);
                    n
                }
            };
            if !top && let Some(id) = model.generated_def(&name, &g.values) {
                maps.defs.insert(d, id);
                return Ok(id);
            }
            if name != g.generator {
                gen_name = Some(name);
            }
        }
        let mut def = src.clone();
        if let (Some(x), Some(n)) = (def.generated.as_mut(), gen_name) {
            x.generator = n.clone();
            def.name = n;
        }
        if let (Some(r), Some(nm)) = (def.rigged.as_mut(), rig_master) {
            r.master = nm;
            // a variant is called as its master (renamed when it came in beside a changed copy)
            if r.paint.is_none() {
                def.name = model.defs[nm].name.clone();
            }
        }
        let children: Vec<(InstanceId, DefId)> = def.mesh.instances.iter().map(|(i, x)| (i, x.def)).collect();
        for (i, cd) in children {
            let nd = self.copy_def(model, doc_uid, cd, maps, false)?;
            if let Some(inst) = def.mesh.instances.get_mut(i) {
                inst.def = nd;
                inst.material = inst.material.map(|m| maps.material(&self.model, model, m));
                inst.tag = inst.tag.map(|t| maps.tag(&self.model, model, t));
            }
        }
        // A parametric group brings its base (and stretched variant) along.
        if let Some(mut p) = def.params.take() {
            p.base = self.copy_def(model, doc_uid, p.base, maps, false)?;
            p.variant = match p.variant {
                Some(v) => Some(self.copy_def(model, doc_uid, v, maps, false)?),
                None => None,
            };
            def.params = Some(p);
        }
        for f in def.mesh.faces.values_mut() {
            f.front = f.front.map(|m| maps.material(&self.model, model, m));
            f.back = f.back.map(|m| maps.material(&self.model, model, m));
            f.tag = f.tag.map(|t| maps.tag(&self.model, model, t));
        }
        for e in def.mesh.edges.values_mut() {
            e.tag = e.tag.map(|t| maps.tag(&self.model, model, t));
        }
        // A library definition beside a changed copy of it in the model: "<name> 2".
        if from_library {
            def.name = model.free_def_name(&def.name);
        }
        let nd = model.defs.insert(def);
        maps.defs.insert(d, nd);
        Ok(nd)
    }
}

/// `name` is `base` or `base` with a number ("Chair 2"): a copy [`Model::free_def_name`] made.
fn same_or_numbered(name: &str, base: &str) -> bool {
    name == base || name.strip_prefix(base).and_then(|r| r.strip_prefix(' ')).is_some_and(|n| n.parse::<u32>().is_ok())
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
    fn library_round_trip_places_one_definition() {
        let mut doc = boxed_doc();
        // A group moved away from the origin: the library file starts at the origin.
        doc.select_all();
        let path = doc.active_path.clone();
        let sel = doc.selection.clone();
        let g = doc.commit("group", |m| m.make_group(&path, &sel).unwrap()).unwrap();
        doc.commit("move", |m| m.root_mesh_mut().instances[g].transform = DMat4::from_translation(DVec3::new(500.0, 0.0, 0.0)));
        doc.set_selection(vec![EntityRef::Instance(g)]);
        let file = Clipboard::from_selection(&doc).unwrap().to_model(varstotas_kernel::LengthUnit::Millimeter).unwrap();
        let root = file.root_mesh();
        assert_eq!((root.faces.len(), root.instances.len()), (6, 0), "the group was exploded into the file");
        let b = file.bounds().unwrap();
        assert!(b.min.length() < 1e-9 && (b.max - DVec3::new(1000.0, 1000.0, 500.0)).length() < 1e-9, "{b:?}");
        assert!(file.materials.values().any(|m| m.name == "Red"));

        // Placed twice into another document: one "Crate" definition, two instances.
        let mut target = Document::new();
        let clip = Clipboard::from_library(file, "Crate").unwrap();
        assert_eq!(clip.reference_point(), DVec3::ZERO);
        for x in [0.0, 2000.0] {
            assert!(crate::tools::transform::paste_clipboard(&mut target, &clip, &DMat4::from_translation(DVec3::new(x, 0.0, 0.0))));
        }
        let m = &target.model;
        let crates: Vec<_> = m.defs.iter().filter(|(_, d)| d.name == "Crate" && !d.is_group).map(|(k, _)| k).collect();
        assert_eq!(crates.len(), 1);
        assert_eq!(m.root_mesh().instances.len(), 2);
        assert!(m.root_mesh().instances.values().all(|i| i.def == crates[0]));
        assert!((m.bounds().unwrap().max.x - 3000.0).abs() < 1e-9);

        // The model's Crate changed (a face erased): the library's comes in as "Crate 2", as in
        // the file; placed again, that one is shared.
        let d = crates[0];
        let f = target.model.defs[d].mesh.faces.keys().next().unwrap();
        target.commit("erase", |m| m.defs[d].mesh.erase(&[EntityRef::Face(f)]));
        for x in [4000.0, 6000.0] {
            assert!(crate::tools::transform::paste_clipboard(&mut target, &clip, &DMat4::from_translation(DVec3::new(x, 0.0, 0.0))));
        }
        let m = &target.model;
        let by_name = |n: &str| m.defs.iter().find(|(_, x)| x.name == n).map(|(k, _)| k).unwrap();
        let (c1, c2) = (by_name("Crate"), by_name("Crate 2"));
        assert_eq!((m.defs[c1].mesh.faces.len(), m.defs[c2].mesh.faces.len()), (5, 6));
        assert_eq!((m.instance_count(c1), m.instance_count(c2)), (2, 2));
    }

    #[test]
    fn rig_survives_the_library() {
        use varstotas_kernel::{GenParam, GenValue, Joint, JointKind, Rig};
        let mut doc = Document::new();
        let src = "let body = box(vec3(0, 0, 0), 600, 400, 800);\nlet door = box(vec3(0, -20, 0), 600, 20, 800);\ndoor.name = \"Door\";\ncomponent([body, door], \"Cabinet\");";
        doc.commit("cabinet", |m| assert!(varstotas_script::run(m, &Default::default(), src).error.is_none()));
        let (i, d) = doc.model.root_mesh().instances.iter().map(|(i, x)| (i, x.def)).next().unwrap();
        let rig = Rig {
            params: vec![GenParam::parse("open:angle(0..90/1) = 0").unwrap()],
            joints: vec![Joint { part: "Door".into(), kind: JointKind::Rotate { origin: DVec3::new(0.0, -20.0, 0.0), axis: DVec3::Z }, param: "open".into(), from: 0.0, to: -90.0, expr: String::new() }],
            ..Default::default()
        };
        doc.try_commit("rig", |m| varstotas_script::rig::set_rig(m, d, Some(rig.clone())));
        let open: varstotas_kernel::GenValues = [("open".to_string(), GenValue::Num(90.0))].into();
        doc.try_commit("open", |m| varstotas_script::rig::set_values(m, &[], i, &open, false));
        // Save to Library: the copy stays one component with its master, through a file.
        doc.set_selection(vec![EntityRef::Instance(i)]);
        let file = Clipboard::from_selection(&doc).unwrap().to_model(varstotas_kernel::LengthUnit::Millimeter).unwrap();
        let file: Model = serde_json::from_str(&serde_json::to_string(&file).unwrap()).unwrap();
        assert_eq!(file.root_mesh().instances.len(), 1);
        // Placed twice into another document (the file named otherwise): one master with the
        // parameters, both copies open and sharing one variant.
        let mut target = Document::new();
        let clip = Clipboard::from_library(file.clone(), "Kitchen cabinet").unwrap();
        for x in [0.0, 2000.0] {
            assert!(crate::tools::transform::paste_clipboard(&mut target, &clip, &DMat4::from_translation(DVec3::new(x, 0.0, 0.0))));
        }
        let m = &target.model;
        let masters: Vec<_> = m.defs.iter().filter(|(_, x)| x.rig.is_some()).map(|(k, _)| k).collect();
        assert_eq!(masters.len(), 1, "one master");
        assert_eq!(m.defs[masters[0]].rig.as_ref(), Some(&rig));
        let ids: Vec<InstanceId> = m.root_mesh().instances.keys().collect();
        assert_eq!(ids.len(), 2);
        let (v, r) = m.instance_rigged(&[], ids[0]).expect("a copy with parameters");
        assert_eq!((r.master, r.values["open"].clone()), (masters[0], GenValue::Num(90.0)));
        assert_eq!(m.instance_rigged(&[], ids[1]).unwrap().0, v);
        // Its values still change there.
        let close: varstotas_kernel::GenValues = [("open".to_string(), GenValue::Num(0.0))].into();
        assert!(target.try_commit("close", |m| varstotas_script::rig::set_values(m, &[], ids[1], &close, false)).is_some());
        assert_ne!(target.model.instance_rigged(&[], ids[1]).unwrap().0, v);
        // A third copy from a fresh placement of the same file reuses the master.
        let clip3 = Clipboard::from_library(file, "Kitchen cabinet").unwrap();
        assert!(crate::tools::transform::paste_clipboard(&mut target, &clip3, &DMat4::from_translation(DVec3::new(4000.0, 0.0, 0.0))));
        assert_eq!(target.model.defs.values().filter(|x| x.rig.is_some()).count(), 1);
        // The master changed in the model (a part added): the library's comes in as a master of
        // its own, beside it.
        let mut changed = target.model.clone();
        let edited = changed.add_material("Blue", [0.0, 0.0, 1.0, 1.0]);
        let mf = changed.defs[masters[0]].mesh.instances.keys().next().unwrap();
        changed.defs[masters[0]].mesh.instances[mf].material = Some(edited);
        let mut t2 = Document::new();
        t2.model = changed;
        assert!(crate::tools::transform::paste_clipboard(&mut t2, &clip3, &DMat4::from_translation(DVec3::new(8000.0, 0.0, 0.0))));
        let names: Vec<&str> = t2.model.defs.values().filter(|x| x.rig.is_some()).map(|x| x.name.as_str()).collect();
        assert_eq!(names.len(), 2, "{names:?}");
        assert!(names.iter().any(|n| n.ends_with(" 2")), "{names:?}");
        // The master placed from the Document folder gets the variant for the defaults.
        let clip4 = Clipboard::of_definition(&target.model, target.uid, masters[0]).unwrap();
        assert!(crate::tools::transform::paste_clipboard(&mut target, &clip4, &DMat4::from_translation(DVec3::new(6000.0, 0.0, 0.0))));
        let m = &target.model;
        assert!(m.root_mesh().instances.keys().all(|i| m.instance_rigged(&[], i).is_some()), "every copy on a variant");
    }

    #[test]
    fn document_component_placed_again_shares_its_definition() {
        let mut doc = boxed_doc();
        doc.select_all();
        let sel = doc.selection.clone();
        let c = doc.commit("component", |m| m.make_component(&[], &sel, "Box").unwrap()).unwrap();
        let d = doc.model.root_mesh().instances[c].def;
        let clip = Clipboard::of_definition(&doc.model, doc.uid, d).unwrap();
        assert!(clip.preview.len() >= 12 && def_edges(&doc.model, d, 4).len() == 4);
        assert!(crate::tools::transform::paste_clipboard(&mut doc, &clip, &DMat4::from_translation(DVec3::new(3000.0, 0.0, 0.0))));
        let m = &doc.model;
        assert_eq!(m.defs.values().filter(|x| x.name == "Box").count(), 1, "no copy of the definition");
        assert_eq!(m.root_mesh().instances.len(), 2);
        assert!(m.root_mesh().instances.values().all(|i| i.def == d));
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
    fn parametric_group_pastes_with_its_base_into_another_document() {
        let mut doc = boxed_doc();
        doc.select_all();
        let sel = doc.selection.clone();
        let g = doc.try_commit("Make Group", |m| m.make_group(&[], &sel)).unwrap();
        doc.try_commit("Make Parametric", |m| m.make_parametric(&[], g)).unwrap();
        let mut p = doc.model.instance_params(&[], g).unwrap().clone();
        p.size[0] = Some(1500.0);
        p.count = 2;
        doc.try_commit("Params", |m| m.set_instance_params(&[], g, p)).unwrap();
        doc.set_selection(vec![EntityRef::Instance(g)]);
        let clip = Clipboard::from_selection(&doc).unwrap();
        let mut other = Document::new();
        let ouid = other.uid;
        let (ents, _) = other.commit("Paste", |m| clip.paste_into(m, ouid, &[], &DMat4::IDENTITY)).unwrap().unwrap();
        let [EntityRef::Instance(c)] = ents.as_slice() else { panic!("{ents:?}") };
        let mut p = other.model.instance_params(&[], *c).expect("still parametric").clone();
        assert!(other.model.defs.contains_key(p.base) && other.model.defs.contains_key(p.variant.unwrap()));
        // Editing the pasted copy works on its own base and variant.
        p.count = 1;
        other.try_commit("Params", |m| m.set_instance_params(&[], *c, p)).unwrap();
        let b = other.model.bounds().unwrap();
        assert!((b.max.x - b.min.x - 1500.0).abs() < 1e-6, "{b:?}");
    }

    #[test]
    fn generated_components_bring_their_generator_and_share_by_values() {
        let src = include_str!("../../../examples/generators/shelf.rhai");
        let mut doc = Document::new();
        let g = doc
            .try_commit("Place", |m| {
                let n = varstotas_script::generator::add(m, src)?;
                varstotas_script::generator::place(m, &[], &n, &Default::default(), DMat4::IDENTITY)
            })
            .unwrap();
        doc.set_selection(vec![EntityRef::Instance(g)]);
        let clip = Clipboard::from_selection(&doc).unwrap();
        let mut other = Document::new();
        let ouid = other.uid;
        let mut pasted = Vec::new();
        for k in 0..2 {
            let t = DMat4::from_translation(DVec3::X * 2000.0 * k as f64);
            let (ents, _) = other.commit("Paste", |m| clip.paste_into(m, ouid, &[], &t)).unwrap().unwrap();
            let [EntityRef::Instance(c)] = ents.as_slice() else { panic!("{ents:?}") };
            pasted.push(*c);
        }
        assert_eq!(other.model.generators["Shelf"].source, src);
        let defs: Vec<_> = pasted.iter().map(|c| other.model.root_mesh().instances[*c].def).collect();
        assert_eq!(defs[0], defs[1], "equal values: one definition");
        // The pasted copies are live: a change rebuilds one of them.
        let v = [("width".to_string(), varstotas_kernel::GenValue::Num(1000.0))].into_iter().collect();
        other.try_commit("P", |m| varstotas_script::generator::set_values(m, &[], pasted[1], &v, false)).unwrap();
        let b = other.model.def_bounds(other.model.root_mesh().instances[pasted[1]].def).unwrap();
        assert!((b.max.x - b.min.x - 1000.0).abs() < 1e-6);

        // The target's Shelf script edited: the original comes in beside it as "Shelf 2" (its
        // header renamed), and placed again shares that one.
        let edited = src.replace("fn build(p) {", "fn build(p) {\n    box(vec3(0, 0, -100), 10, 10, 10);");
        other.try_commit("Edit", |m| varstotas_script::generator::add(m, &edited)).unwrap();
        let mut more = Vec::new();
        for k in 2..4 {
            let t = DMat4::from_translation(DVec3::X * 2000.0 * k as f64);
            let (ents, _) = other.commit("Paste", |m| clip.paste_into(m, ouid, &[], &t)).unwrap().unwrap();
            let [EntityRef::Instance(c)] = ents.as_slice() else { panic!("{ents:?}") };
            more.push(other.model.root_mesh().instances[*c].def);
        }
        let m = &other.model;
        assert_eq!(more[0], more[1]);
        assert_eq!(m.defs[more[0]].generated.as_ref().unwrap().generator, "Shelf 2");
        assert_eq!(m.generators["Shelf 2"].source, varstotas_kernel::generator::renamed_source(src, "Shelf 2"));
        assert_eq!(m.generators["Shelf"].source, edited, "the edited one stays");
    }

    #[test]
    fn array_of_touching_copies_welds_and_returns_live_refs() {
        let mut doc = boxed_doc();
        doc.select_all();
        let uid = doc.uid;
        let ents = doc.selection.clone();
        let (out, faces) = doc
            .commit("Copy", |m| {
                let clip = Clipboard::from_parts(m, uid, &[], &ents, &[]).unwrap();
                let mut out = Vec::new();
                for k in 1..=5 {
                    let t = DMat4::from_translation(DVec3::X * 1000.0 * k as f64);
                    out.extend(clip.paste_into(m, uid, &[], &t).unwrap().0);
                }
                (out, m.root_mesh().faces.len())
            })
            .unwrap();
        let mesh = doc.model.root_mesh();
        assert!(out.iter().all(|e| crate::document::entity_alive(mesh, *e)));
        // Every copy landed (the source refs were never welded away mid-array).
        let b = doc.model.bounds().unwrap();
        assert!((b.max.x - 6000.0).abs() < 1e-6, "{b:?}");
        // 6 boxes in a row: 4 outer faces each + 2 ends + 5 shared walls (merged, not doubled).
        assert!((6 * 4 + 2..=6 * 6).contains(&faces), "{faces}");
        // Later copies share (welded) edges with earlier ones; the selection dedupes them.
        let unique: HashSet<EntityRef> = out.iter().copied().collect();
        doc.set_selection(out);
        assert_eq!(doc.selection.len(), unique.len());
    }

    #[test]
    fn copy_guides_only() {
        let mut doc = Document::new();
        let g = doc.commit("g", |m| m.add_guide_in(&[], GuideGeom::Line(GuideLine { origin: DVec3::new(0.0, 500.0, 0.0), dir: DVec3::X })));
        let p = doc.commit("p", |m| m.add_guide_in(&[], GuideGeom::Point(DVec3::new(10.0, 20.0, 30.0))));
        doc.set_selection_with_guides(Vec::new(), vec![g.flatten().unwrap(), p.flatten().unwrap()]);
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
