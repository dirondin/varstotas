//! The open document: model + undo history + file state + selection + editing context.

use varstotas_i18n::tr;
use varstotas_kernel::{ContextPath, DimId, EntityRef, GuideRef, History, MaterialId, Mesh, Model, ModelHit, Ray};
use std::cell::RefCell;
use std::collections::HashSet;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_MODEL_VERSION: AtomicU64 = AtomicU64::new(1);

/// A model version never used before in this process, so caches keyed by version can't
/// confuse two documents (a fresh document does not restart at version 1).
pub fn next_model_version() -> u64 {
    NEXT_MODEL_VERSION.fetch_add(1, Ordering::Relaxed)
}

type RayKey = (usize, u64, [u64; 6]);

thread_local! {
    /// The last few model raycasts. One pointer event typically raycasts the same pick ray
    /// several times (inference, hover picking, edge occlusion); on big models each raycast
    /// is a full scan, so repeats are served from here.
    static RAYCASTS: RefCell<Vec<(RayKey, Option<ModelHit>)>> = const { RefCell::new(Vec::new()) };
}

/// `model.raycast(ray)` (panic-guarded), memoized per (`version`, exact ray).
/// `version` must change whenever the model does (use [`Document::model_version`]).
pub fn cached_raycast(model: &Model, version: u64, ray: &Ray) -> Option<ModelHit> {
    let (o, d) = (ray.origin, ray.dir);
    let key = (model as *const Model as usize, version, [o.x, o.y, o.z, d.x, d.y, d.z].map(f64::to_bits));
    if let Some(hit) = RAYCASTS.with_borrow(|c| c.iter().find(|(k, _)| *k == key).map(|(_, h)| h.clone())) {
        return hit;
    }
    let hit = guarded(|| model.raycast(ray)).ok().flatten();
    RAYCASTS.with_borrow_mut(|c| {
        c.retain(|(k, _)| k.0 == key.0 && k.1 == version);
        if c.len() >= 8 {
            c.remove(0);
        }
        c.push((key, hit.clone()));
    });
    hit
}

/// Material chosen in the Materials panel. Stored by name/colour (not id) so it survives
/// undo/redo; the Paint tool resolves it with [`PaletteMaterial::ensure_in`] inside a commit.
#[derive(Clone, Debug, PartialEq)]
pub struct PaletteMaterial {
    pub name: String,
    pub color: [f32; 4],
    /// Texture of a textured material (bytes shared, cheap to clone).
    pub image: Option<varstotas_kernel::TextureImage>,
    /// PBR parameters, maps and shader (a library material brings its own).
    pub pbr: Option<varstotas_kernel::Pbr>,
}

#[allow(dead_code)] // used by the Paint tool (Tools agent)
impl PaletteMaterial {
    /// Find a material with this name in `model`, or add it. Returns its id.
    pub fn ensure_in(&self, model: &mut Model) -> MaterialId {
        if let Some((id, _)) = model.materials.iter().find(|(_, m)| m.name == self.name) {
            return id;
        }
        let id = model.add_material(&self.name, self.color);
        model.materials[id].image = self.image.clone();
        model.materials[id].pbr = self.pbr.clone();
        id
    }

    /// A plain colour.
    pub fn color(name: impl Into<String>, color: [f32; 4]) -> Self {
        Self { name: name.into(), color, image: None, pbr: None }
    }

    /// The palette entry for a model material.
    pub fn of(m: &varstotas_kernel::Material) -> Self {
        Self { name: m.name.clone(), color: m.color, image: m.image.clone(), pbr: m.pbr.clone() }
    }
}

pub struct Document {
    pub model: Model,
    pub history: History,
    pub path: Option<PathBuf>,
    pub dirty: bool,
    /// Bumped on every model change; the viewport rebuilds GPU geometry when it changes.
    pub model_version: u64,
    /// `model_version` right after the last recorded undo step; "edit last operation" (VCB)
    /// may undo only while nothing (undo / redo / other change) happened since.
    pub last_commit_version: u64,
    /// Bumped on selection / context change (also triggers a geometry rebuild).
    pub selection_version: u64,
    /// Selected entities, all living in the mesh at `active_path`.
    pub selection: Vec<EntityRef>,
    /// Selected guides of the active context (like `selection`).
    pub guide_sel: Vec<GuideRef>,
    /// Selected dimensions of the active context (like `selection`).
    pub dim_sel: Vec<DimId>,
    /// Selected section plane (index into `model.sections`); exclusive with other selections.
    pub section_sel: Option<usize>,
    /// Selected light (index into `model.lights`); exclusive with other selections.
    pub light_sel: Option<usize>,
    /// Editing context (empty = model root).
    pub active_path: ContextPath,
    /// Current paint material (None = "Default", i.e. remove material).
    pub current_material: Option<PaletteMaterial>,
    /// Last error from a guarded kernel call, shown in the status bar.
    pub last_error: Option<String>,
    /// Process-unique id of this document (clipboard: share component definitions only when
    /// pasting back into the same document).
    pub uid: u64,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// Run a closure, converting a panic (e.g. a not-yet-implemented kernel op) into an error.
pub fn guarded<R>(f: impl FnOnce() -> R) -> Result<R, String> {
    catch_unwind(AssertUnwindSafe(f)).map_err(|e| {
        if let Some(s) = e.downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = e.downcast_ref::<String>() {
            s.clone()
        } else {
            tr!("edit-internal-error")
        }
    })
}

impl Document {
    pub fn new() -> Self {
        Self::from_model(Model::new(), None)
    }

    pub fn from_model(model: Model, path: Option<PathBuf>) -> Self {
        Self {
            model,
            history: History::default(),
            path,
            dirty: false,
            model_version: next_model_version(),
            last_commit_version: 0,
            selection_version: 1,
            selection: Vec::new(),
            guide_sel: Vec::new(),
            dim_sel: Vec::new(),
            section_sel: None,
            light_sel: None,
            active_path: Vec::new(),
            current_material: None,
            last_error: None,
            uid: {
                static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            },
        }
    }

    /// File name for the window title.
    pub fn display_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| tr!("edit-untitled"))
    }

    pub fn active_mesh(&self) -> Option<&Mesh> {
        self.model.mesh_at(&self.active_path)
    }

    #[allow(dead_code)]
    pub fn active_mesh_mut(&mut self) -> Option<&mut Mesh> {
        let path = self.active_path.clone();
        self.model.mesh_at_mut(&path)
    }

    /// Mutate the model as one undoable operation. The model is snapshotted before `f` runs;
    /// if `f` panics the model is restored, nothing is recorded and `None` is returned.
    pub fn commit<R>(&mut self, label: &str, f: impl FnOnce(&mut Model) -> R) -> Option<R> {
        let before = self.model.clone();
        match guarded(|| f(&mut self.model)) {
            Ok(r) => {
                self.rig_hook();
                if let Err(msg) = before.validate_locked_changes(&self.model) {
                    self.model = before;
                    self.last_error = Some(msg);
                    return None;
                }
                self.history.push(label, before);
                self.after_change();
                self.last_commit_version = self.model_version;
                self.last_error = None;
                Some(r)
            }
            Err(msg) => {
                self.model = before;
                self.last_error = Some(tr!("edit-failed", label = label, error = msg.to_string()));
                log::warn!("{label} failed: {msg}");
                None
            }
        }
    }

    /// Like [`Self::commit`] but for fallible kernel ops: on `Err` the model is restored and
    /// no undo step is recorded.
    pub fn try_commit<R, E: std::fmt::Display>(
        &mut self,
        label: &str,
        f: impl FnOnce(&mut Model) -> Result<R, E>,
    ) -> Option<R> {
        self.try_commit_if(label, f, |_| true)
    }

    /// [`Self::try_commit`] that records the step only when `changed(&result)`; otherwise the
    /// model is restored untouched (no empty undo step, document not dirtied) and the result
    /// is still returned.
    pub fn try_commit_if<R, E: std::fmt::Display>(
        &mut self,
        label: &str,
        f: impl FnOnce(&mut Model) -> Result<R, E>,
        changed: impl FnOnce(&R) -> bool,
    ) -> Option<R> {
        let before = self.model.clone();
        match guarded(|| f(&mut self.model)) {
            Ok(Ok(r)) if !changed(&r) => {
                self.model = before;
                self.last_error = None;
                Some(r)
            }
            Ok(Ok(r)) => {
                self.rig_hook();
                if let Err(msg) = before.validate_locked_changes(&self.model) {
                    self.model = before;
                    self.last_error = Some(msg);
                    return None;
                }
                self.history.push(label, before);
                self.after_change();
                self.last_commit_version = self.model_version;
                self.last_error = None;
                Some(r)
            }
            Ok(Err(e)) => {
                self.model = before;
                self.last_error = Some(tr!("edit-op-error", label = label, error = e.to_string()));
                None
            }
            Err(msg) => {
                self.model = before;
                self.last_error = Some(tr!("edit-failed", label = label, error = msg.to_string()));
                log::warn!("{label} failed: {msg}");
                None
            }
        }
    }

    /// Mutate document-level state that should not create an undo step
    /// (e.g. tag visibility). Still marks the document dirty.
    pub fn mutate_untracked(&mut self, f: impl FnOnce(&mut Model)) {
        let before = self.model.has_locks().then(|| self.model.clone());
        let result = guarded(|| f(&mut self.model)).and_then(|_| {
            before.as_ref().map_or(Ok(()), |m| m.validate_locked_changes(&self.model))
        });
        if let Err(msg) = result {
            if let Some(before) = before { self.model = before; }
            self.last_error = Some(msg);
            return;
        }
        self.after_change();
    }

    /// Record an undo step for changes already made in place since `before`.
    pub fn record_step(&mut self, label: &str, before: Model) {
        self.rig_hook();
        if let Err(msg) = before.validate_locked_changes(&self.model) {
            self.model = before;
            self.last_error = Some(msg);
            self.after_change();
            return;
        }
        self.history.push(label, before);
        self.after_change();
        self.last_commit_version = self.model_version;
    }

    /// Explicit user lock controls; scripts and plugins cannot call this entry point.
    pub fn change_locks(&mut self, label: &str, f: impl FnOnce(&mut Model)) {
        let before = self.model.clone();
        f(&mut self.model);
        self.history.push(label, before);
        self.after_change();
        self.last_commit_version = self.model_version;
        self.last_error = None;
    }

    /// Components with parameters (rigs) after an edit: an edited variant goes back to its
    /// master (the other copies follow), copies placed with a master get a variant.
    fn rig_hook(&mut self) {
        if let Err(e) = varstotas_script::rig::after_edit(&mut self.model, &self.active_path) {
            log::warn!("parameters: {e}");
            self.last_error = Some(tr!("edit-parameters-error", error = e.to_string()));
        }
    }

    fn after_change(&mut self) {
        self.dirty = true;
        self.model_version = next_model_version();
        self.prune_selection();
    }

    /// Undo the step just committed, for re-typing a value in the Measurements box. Refused
    /// when the top undo step is not the latest commit (e.g. the user pressed Undo since).
    pub fn undo_last_commit(&mut self) -> Option<String> {
        if self.model_version != self.last_commit_version {
            return None;
        }
        self.undo()
    }

    pub fn undo(&mut self) -> Option<String> {
        let l = self.history.undo(&mut self.model)?;
        self.after_change();
        Some(l)
    }

    pub fn redo(&mut self) -> Option<String> {
        let l = self.history.redo(&mut self.model)?;
        self.after_change();
        Some(l)
    }

    pub fn agent_history(&mut self, redo: bool) -> Result<Option<String>, String> {
        let label = if redo { self.history.redo_preserving_locks(&mut self.model) }
            else { self.history.undo_preserving_locks(&mut self.model) }?;
        if label.is_some() { self.after_change(); }
        Ok(label)
    }

    // ---- selection -------------------------------------------------------

    /// Replace the selection with `sel` (entities of the active context; no guides).
    pub fn set_selection(&mut self, sel: Vec<EntityRef>) {
        self.set_selection_with_guides(sel, Vec::new());
    }

    /// Replace the selection with entities and guides.
    /// Dead refs (e.g. entities merged away by a kernel weld) and duplicates are dropped.
    pub fn set_selection_with_guides(&mut self, sel: Vec<EntityRef>, guides: Vec<GuideRef>) {
        self.set_selection_full(sel, guides, Vec::new());
    }

    /// Replace the selection with entities, guides and dimensions.
    pub fn set_selection_full(&mut self, mut sel: Vec<EntityRef>, mut guides: Vec<GuideRef>, mut dims: Vec<DimId>) {
        if let Some(mesh) = self.model.mesh_at(&self.active_path) {
            let mut seen = std::collections::HashSet::new();
            sel.retain(|e| entity_alive(mesh, *e) && !self.model.entity_locked(&self.active_path, *e) && seen.insert(*e));
        }
        let mut seen = std::collections::HashSet::new();
        guides.retain(|g| self.model.has_guide_in(&self.active_path, *g) && seen.insert(*g));
        let mut seen = std::collections::HashSet::new();
        dims.retain(|d| self.model.has_dimension_in(&self.active_path, *d) && seen.insert(*d));
        if sel != self.selection || guides != self.guide_sel || dims != self.dim_sel || self.section_sel.is_some() || self.light_sel.is_some() {
            self.section_sel = None;
            self.light_sel = None;
            self.selection = sel;
            self.guide_sel = guides;
            self.dim_sel = dims;
            self.selection_version += 1;
        }
    }

    /// Select section plane `i` alone.
    pub fn select_section(&mut self, i: usize) {
        self.set_selection(Vec::new());
        if i < self.model.sections.len() && self.section_sel != Some(i) {
            self.section_sel = Some(i);
            self.selection_version += 1;
        }
    }

    /// Select light `i` alone.
    pub fn select_light(&mut self, i: usize) {
        self.set_selection(Vec::new());
        if i < self.model.lights.len() && self.light_sel != Some(i) {
            self.light_sel = Some(i);
            self.selection_version += 1;
        }
    }

    pub fn clear_selection(&mut self) {
        self.set_selection(Vec::new());
    }

    /// One component is selected whose content other copies share (Make Unique applies).
    pub fn can_make_unique(&self) -> bool {
        match self.selection.as_slice() {
            [EntityRef::Instance(i)] => {
                let m = &self.model;
                let group = m.mesh_at(&self.active_path).and_then(|x| x.instances.get(*i)).and_then(|x| m.defs.get(x.def)).is_none_or(|d| d.is_group);
                !group && m.can_make_unique(&self.active_path, *i)
            }
            _ => false,
        }
    }

    /// Anything (entities or guides) selected.
    pub fn has_selection(&self) -> bool {
        !self.selection.is_empty() || !self.guide_sel.is_empty() || !self.dim_sel.is_empty() || self.section_sel.is_some() || self.light_sel.is_some()
    }

    /// Apply a selection operation to dimensions (mirrors the entity ops).
    pub fn add_dims(&mut self, ds: &[DimId]) {
        let n = self.dim_sel.len();
        for d in ds {
            if !self.dim_sel.contains(d) {
                self.dim_sel.push(*d);
            }
        }
        if n != self.dim_sel.len() {
            self.selection_version += 1;
        }
    }

    pub fn remove_dims_from_selection(&mut self, ds: &[DimId]) {
        let n = self.dim_sel.len();
        self.dim_sel.retain(|d| !ds.contains(d));
        if n != self.dim_sel.len() {
            self.selection_version += 1;
        }
    }

    pub fn toggle_dims(&mut self, ds: &[DimId]) {
        if ds.is_empty() {
            return;
        }
        let had = self.dim_sel.clone();
        self.dim_sel.retain(|d| !ds.contains(d));
        for d in ds {
            if !had.contains(d) && !self.dim_sel.contains(d) {
                self.dim_sel.push(*d);
            }
        }
        self.selection_version += 1;
    }

    /// Apply a selection operation to guides (mirrors the entity ops).
    pub fn add_guides(&mut self, gs: &[GuideRef]) {
        let n = self.guide_sel.len();
        for g in gs {
            if !self.guide_sel.contains(g) {
                self.guide_sel.push(*g);
            }
        }
        if n != self.guide_sel.len() {
            self.selection_version += 1;
        }
    }

    pub fn remove_guides_from_selection(&mut self, gs: &[GuideRef]) {
        let n = self.guide_sel.len();
        self.guide_sel.retain(|g| !gs.contains(g));
        if n != self.guide_sel.len() {
            self.selection_version += 1;
        }
    }

    pub fn toggle_guides(&mut self, gs: &[GuideRef]) {
        if gs.is_empty() {
            return;
        }
        let had = self.guide_sel.clone();
        self.guide_sel.retain(|g| !gs.contains(g));
        for g in gs {
            if !had.contains(g) && !self.guide_sel.contains(g) {
                self.guide_sel.push(*g);
            }
        }
        self.selection_version += 1;
    }

    #[allow(dead_code)]
    pub fn is_selected(&self, e: &EntityRef) -> bool {
        self.selection.contains(e)
    }

    #[allow(dead_code)]
    pub fn toggle_selected(&mut self, e: EntityRef) {
        if self.model.entity_locked(&self.active_path, e) { return; }
        if let Some(i) = self.selection.iter().position(|x| *x == e) {
            self.selection.remove(i);
        } else {
            self.selection.push(e);
        }
        self.selection_version += 1;
    }

    pub fn add_to_selection(&mut self, ents: impl IntoIterator<Item = EntityRef>) {
        let mut have: HashSet<EntityRef> = self.selection.iter().copied().collect();
        let n = self.selection.len();
        for e in ents {
            if !self.model.entity_locked(&self.active_path, e) && have.insert(e) {
                self.selection.push(e);
            }
        }
        if n != self.selection.len() {
            self.selection_version += 1;
        }
    }

    pub fn remove_from_selection(&mut self, ents: &[EntityRef]) {
        let n = self.selection.len();
        let gone: HashSet<EntityRef> = ents.iter().copied().collect();
        self.selection.retain(|e| !gone.contains(e));
        if n != self.selection.len() {
            self.selection_version += 1;
        }
    }

    /// Toggle each entity (Shift + selection box): selected ones are removed, others added.
    pub fn toggle_many(&mut self, ents: &[EntityRef]) {
        if ents.is_empty() {
            return;
        }
        let toggled: HashSet<EntityRef> = ents.iter().copied().collect();
        let had: HashSet<EntityRef> = self.selection.iter().copied().collect();
        self.selection.retain(|e| !toggled.contains(e));
        let mut seen = HashSet::new();
        for e in ents {
            if !self.model.entity_locked(&self.active_path, *e) && !had.contains(e) && seen.insert(*e) {
                self.selection.push(*e);
            }
        }
        self.selection_version += 1;
    }

    /// Select every visible entity of the active context.
    pub fn select_all(&mut self) {
        let Some(mesh) = self.active_mesh() else {
            return;
        };
        let mut sel = Vec::new();
        sel.extend(mesh.faces.iter().filter(|(_, f)| !f.hidden).map(|(id, _)| EntityRef::Face(id)));
        sel.extend(mesh.edges.iter().filter(|(_, e)| !e.hidden).map(|(id, _)| EntityRef::Edge(id)));
        sel.extend(
            mesh.instances
                .iter()
                .filter(|(_, i)| !i.hidden)
                .map(|(id, _)| EntityRef::Instance(id)),
        );
        let guides = if self.model.guides_hidden { Vec::new() } else { self.model.all_guides_in(&self.active_path) };
        let dims = self.model.all_dimensions_in(&self.active_path);
        self.set_selection_full(sel, guides, dims);
    }

    /// Enter an editing context (double-click on group).
    pub fn set_active_path(&mut self, path: ContextPath) {
        if self.model.context_locked(&path) { return; }
        if path != self.active_path {
            self.active_path = path;
            self.selection.clear();
            self.guide_sel.clear();
            self.dim_sel.clear();
            self.selection_version += 1;
        }
    }

    /// Leave the current editing context; the group we were in becomes selected.
    pub fn exit_context(&mut self) -> bool {
        let Some(last) = self.active_path.pop() else {
            return false;
        };
        self.selection = vec![EntityRef::Instance(last)];
        self.guide_sel.clear();
        self.dim_sel.clear();
        self.selection_version += 1;
        true
    }

    /// Drop selected refs that no longer exist (after undo/erase), and reset an invalid context.
    pub fn prune_selection(&mut self) {
        if self.model.context_locked(&self.active_path) || guarded(|| self.model.mesh_at(&self.active_path).is_none()).unwrap_or(true) {
            self.active_path.clear();
        }
        let ng = self.guide_sel.len();
        let model = &self.model;
        let path = &self.active_path;
        self.guide_sel.retain(|g| !model.guides_hidden && model.has_guide_in(path, *g));
        let nd = self.dim_sel.len();
        self.dim_sel.retain(|d| model.has_dimension_in(path, *d));
        if ng != self.guide_sel.len() || nd != self.dim_sel.len() {
            self.selection_version += 1;
        }
        if self.section_sel.is_some_and(|i| i >= model.sections.len() || !model.section_display.planes) {
            self.section_sel = None;
            self.selection_version += 1;
        }
        if self.light_sel.is_some_and(|i| i >= model.lights.len()) {
            self.light_sel = None;
            self.selection_version += 1;
        }
        let Ok(Some(mesh)) = guarded(|| self.model.mesh_at(&self.active_path)) else {
            self.selection.clear();
            self.selection_version += 1;
            return;
        };
        let n = self.selection.len();
        self.selection.retain(|e| entity_alive(mesh, *e) && !model.entity_locked(path, *e));
        if n != self.selection.len() {
            self.selection_version += 1;
        }
    }

    /// Erase the current selection from the active context (one undo step).
    pub fn erase_selection(&mut self) {
        self.erase_selection_labeled(&tr!("edit-undo-erase"));
    }

    /// [`Self::erase_selection`] with an undo label (e.g. "Cut").
    pub fn erase_selection_labeled(&mut self, label: &str) {
        if !self.has_selection() {
            return;
        }
        let sel = std::mem::take(&mut self.selection);
        let guides = std::mem::take(&mut self.guide_sel);
        let dims = std::mem::take(&mut self.dim_sel);
        let section = self.section_sel.take();
        let light = self.light_sel.take();
        let path = self.active_path.clone();
        self.commit(label, |m| {
            if let Some(i) = section.filter(|i| *i < m.sections.len()) {
                m.sections.remove(i);
            }
            if let Some(i) = light.filter(|i| *i < m.lights.len()) {
                m.lights.remove(i);
            }
            m.remove_guides_in(&path, &guides);
            m.remove_dimensions_in(&path, &dims);
            if !sel.is_empty()
                && let Some(mesh) = m.mesh_at_mut(&path)
            {
                mesh.erase(&sel);
            }
        });
        self.selection_version += 1;
    }

    /// Edit > Delete Guides: remove every guide, in all contexts (one undo step).
    pub fn delete_all_guides(&mut self) -> bool {
        if !self.model.has_any_guides() {
            return false;
        }
        self.commit(&tr!("edit-undo-delete-guides"), |m| m.delete_all_guides());
        true
    }
}

/// `e` still exists in `mesh`.
pub fn entity_alive(mesh: &varstotas_kernel::Mesh, e: EntityRef) -> bool {
    match e {
        EntityRef::Vertex(v) => mesh.vertices.contains_key(v),
        EntityRef::Edge(x) => mesh.edges.contains_key(x),
        EntityRef::Face(f) => mesh.faces.contains_key(f),
        EntityRef::Instance(i) => mesh.instances.contains_key(i),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_commit_if_skips_unchanged_results() {
        let mut doc = Document::new();
        // Op mutates the model but reports "nothing happened": restored, no undo step, not dirty.
        let r = doc.try_commit_if(
            "Noop",
            |m| {
                m.unit = varstotas_kernel::LengthUnit::Inch;
                Ok::<_, String>(0usize)
            },
            |n| *n > 0,
        );
        assert_eq!(r, Some(0));
        assert_eq!(doc.history.undo_label(), None);
        assert!(!doc.dirty);
        assert_ne!(doc.model.unit, varstotas_kernel::LengthUnit::Inch);
        // Same op reporting a change records the step.
        let r = doc.try_commit_if(
            "Real",
            |m| {
                m.unit = varstotas_kernel::LengthUnit::Inch;
                Ok::<_, String>(1usize)
            },
            |n| *n > 0,
        );
        assert_eq!(r, Some(1));
        assert_eq!(doc.history.undo_label(), Some("Real"));
        assert!(doc.dirty);
    }
}

#[cfg(test)]
mod locking_tests {
    use super::*;
    use varstotas_kernel::{FaceId, glam::DVec3};

    fn fixture() -> (Document, FaceId) {
        let mut d = Document::new();
        let f = d.model.root_mesh_mut().add_face(&[DVec3::ZERO, DVec3::X * 100., DVec3::Y * 100.]).unwrap();
        (d, f)
    }

    #[test]
    fn lock_controls_clear_selection_and_support_undo_redo() {
        let (mut d, f) = fixture();
        d.set_selection(vec![EntityRef::Face(f)]);
        d.change_locks("Lock", |m| m.set_entities_locked(&[], &[EntityRef::Face(f)], true));
        assert!(d.selection.is_empty());
        assert!(d.agent_history(false).is_err());
        assert_eq!(d.history.undo_label(), Some("Lock"));
        d.select_all(); assert!(d.selection.is_empty());
        d.toggle_selected(EntityRef::Face(f)); assert!(d.selection.is_empty());
        d.add_to_selection([EntityRef::Face(f)]); assert!(d.selection.is_empty());
        d.toggle_many(&[EntityRef::Face(f)]); assert!(d.selection.is_empty());
        d.undo().unwrap(); assert!(!d.model.root_mesh().faces[f].locked);
        d.select_all(); assert!(!d.selection.is_empty());
        d.redo().unwrap(); assert!(d.selection.is_empty());
        d.change_locks("Unlock All", |m| m.unlock_all());
        d.select_all(); assert!(!d.selection.is_empty());
    }

    #[test]
    fn every_document_write_path_rolls_back_locked_mutations_without_undo_steps() {
        let (mut d, f) = fixture();
        d.model.root_mesh_mut().faces[f].locked = true;
        let v = d.model.root_mesh().faces[f].outer[0];
        let before = varstotas_kernel::io::to_json(&d.model).unwrap();
        assert!(d.commit("Move", |m| m.root_mesh_mut().vertices[v].pos += DVec3::Z).is_none());
        assert!(d.try_commit("Delete", |m| Ok::<_, String>(m.root_mesh_mut().faces.remove(f))).is_none());
        d.mutate_untracked(|m| m.root_mesh_mut().vertices[v].pos += DVec3::Z);
        let original = d.model.clone();
        d.model.root_mesh_mut().vertices[v].pos += DVec3::Z;
        d.record_step("In place", original);
        assert_eq!(varstotas_kernel::io::to_json(&d.model).unwrap(), before);
        assert!(d.history.undo_label().is_none());
        assert!(d.last_error.is_some());
    }
}
