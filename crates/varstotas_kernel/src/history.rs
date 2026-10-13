//! Snapshot-based undo/redo.

use crate::Model;

/// Snapshot-based history. Usage: `history.push("Push/Pull", model_before)` after a successful
/// mutation (or before mutating with a clone), then `undo`/`redo`.
#[derive(Default)]
pub struct History {
    undo: Vec<(String, Model)>,
    redo: Vec<(String, Model)>,
}

impl History {
    /// Record `before` as the state to return to on undo. Clears redo stack.
    pub fn push(&mut self, label: &str, before: Model) {
        self.undo.push((label.to_string(), before));
        self.redo.clear();
    }
    pub fn undo(&mut self, current: &mut Model) -> Option<String> {
        let (label, prev) = self.undo.pop()?;
        let cur = std::mem::replace(current, prev);
        self.redo.push((label.clone(), cur));
        Some(label)
    }
    pub fn redo(&mut self, current: &mut Model) -> Option<String> {
        let (label, next) = self.redo.pop()?;
        let cur = std::mem::replace(current, next);
        self.undo.push((label.clone(), cur));
        Some(label)
    }
    pub fn undo_label(&self) -> Option<&str> {
        self.undo.last().map(|(l, _)| l.as_str())
    }
    pub fn redo_label(&self) -> Option<&str> {
        self.redo.last().map(|(l, _)| l.as_str())
    }
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}
