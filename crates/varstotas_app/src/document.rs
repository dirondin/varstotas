//! The open document: model + undo history + file state + selection + editing context.

use varstotas_kernel::{ContextPath, EntityRef, History, MaterialId, Mesh, Model, ModelHit, Ray};
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
}

#[allow(dead_code)] // used by the Paint tool (Tools agent)
impl PaletteMaterial {
    /// Find a material with this name in `model`, or add it. Returns its id.
    pub fn ensure_in(&self, model: &mut Model) -> MaterialId {
        if let Some((id, _)) = model.materials.iter().find(|(_, m)| m.name == self.name) {
            return id;
        }
        model.add_material(&self.name, self.color)
    }
}

pub struct Document {
    pub model: Model,
    pub history: History,
    pub path: Option<PathBuf>,
    pub dirty: bool,
    /// Bumped on every model change; the viewport rebuilds GPU geometry when it changes.
    pub model_version: u64,
    /// Bumped on selection / context change (also triggers a geometry rebuild).
    pub selection_version: u64,
    /// Selected entities, all living in the mesh at `active_path`.
    pub selection: Vec<EntityRef>,
    /// Editing context (empty = model root).
    pub active_path: ContextPath,
    /// Current paint material (None = "Default", i.e. remove material).
    pub current_material: Option<PaletteMaterial>,
    /// Last error from a guarded kernel call, shown in the status bar.
    pub last_error: Option<String>,
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
            "internal error".to_string()
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
            selection_version: 1,
            selection: Vec::new(),
            active_path: Vec::new(),
            current_material: None,
            last_error: None,
        }
    }

    /// File name for the window title.
    pub fn display_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".to_string())
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
                self.history.push(label, before);
                self.after_change();
                self.last_error = None;
                Some(r)
            }
            Err(msg) => {
                self.model = before;
                self.last_error = Some(format!("{label} failed: {msg}"));
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
        let before = self.model.clone();
        match guarded(|| f(&mut self.model)) {
            Ok(Ok(r)) => {
                self.history.push(label, before);
                self.after_change();
                self.last_error = None;
                Some(r)
            }
            Ok(Err(e)) => {
                self.model = before;
                self.last_error = Some(format!("{label}: {e}"));
                None
            }
            Err(msg) => {
                self.model = before;
                self.last_error = Some(format!("{label} failed: {msg}"));
                log::warn!("{label} failed: {msg}");
                None
            }
        }
    }

    /// Mutate document-level state that should not create an undo step
    /// (e.g. tag visibility). Still marks the document dirty.
    pub fn mutate_untracked(&mut self, f: impl FnOnce(&mut Model)) {
        if let Err(msg) = guarded(|| f(&mut self.model)) {
            self.last_error = Some(msg);
        }
        self.after_change();
    }

    fn after_change(&mut self) {
        self.dirty = true;
        self.model_version = next_model_version();
        self.prune_selection();
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

    // ---- selection -------------------------------------------------------

    pub fn set_selection(&mut self, sel: Vec<EntityRef>) {
        if sel != self.selection {
            self.selection = sel;
            self.selection_version += 1;
        }
    }

    pub fn clear_selection(&mut self) {
        self.set_selection(Vec::new());
    }

    #[allow(dead_code)]
    pub fn is_selected(&self, e: &EntityRef) -> bool {
        self.selection.contains(e)
    }

    pub fn toggle_selected(&mut self, e: EntityRef) {
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
            if have.insert(e) {
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
            if !had.contains(e) && seen.insert(*e) {
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
        self.set_selection(sel);
    }

    /// Enter an editing context (double-click on group).
    pub fn set_active_path(&mut self, path: ContextPath) {
        if path != self.active_path {
            self.active_path = path;
            self.selection.clear();
            self.selection_version += 1;
        }
    }

    /// Leave the current editing context; the group we were in becomes selected.
    pub fn exit_context(&mut self) -> bool {
        let Some(last) = self.active_path.pop() else {
            return false;
        };
        self.selection = vec![EntityRef::Instance(last)];
        self.selection_version += 1;
        true
    }

    /// Drop selected refs that no longer exist (after undo/erase), and reset an invalid context.
    pub fn prune_selection(&mut self) {
        if guarded(|| self.model.mesh_at(&self.active_path).is_none()).unwrap_or(true) {
            self.active_path.clear();
        }
        let Ok(Some(mesh)) = guarded(|| self.model.mesh_at(&self.active_path)) else {
            self.selection.clear();
            self.selection_version += 1;
            return;
        };
        let n = self.selection.len();
        self.selection.retain(|e| match *e {
            EntityRef::Vertex(id) => mesh.vertices.contains_key(id),
            EntityRef::Edge(id) => mesh.edges.contains_key(id),
            EntityRef::Face(id) => mesh.faces.contains_key(id),
            EntityRef::Instance(id) => mesh.instances.contains_key(id),
        });
        if n != self.selection.len() {
            self.selection_version += 1;
        }
    }

    /// Erase the current selection from the active context (one undo step).
    pub fn erase_selection(&mut self) {
        if self.selection.is_empty() {
            return;
        }
        let sel = std::mem::take(&mut self.selection);
        let path = self.active_path.clone();
        self.commit("Erase", |m| {
            if let Some(mesh) = m.mesh_at_mut(&path) {
                mesh.erase(&sel);
            }
        });
        self.selection_version += 1;
    }
}
