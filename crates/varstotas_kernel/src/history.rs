//! Snapshot-based undo/redo.

use crate::Model;

/// Snapshot-based history. Usage: `history.push("Push/Pull", model_before)` after a successful
/// mutation (or before mutating with a clone), then `undo`/`redo`.
#[derive(Default)]
pub struct History {
    undo: Vec<(String, Model)>,
    redo: Vec<(String, Model)>,
    /// Most undo steps kept (0 = no limit); the oldest go first.
    limit: usize,
}

impl History {
    /// Record `before` as the state to return to on undo. Clears redo stack.
    pub fn push(&mut self, label: &str, before: Model) {
        self.undo.push((label.to_string(), before));
        self.redo.clear();
        self.trim();
    }

    /// Keep at most `n` undo steps (0 = no limit).
    pub fn set_limit(&mut self, n: usize) {
        if self.limit != n {
            self.limit = n;
            self.trim();
        }
    }

    fn trim(&mut self) {
        if self.limit > 0 && self.undo.len() > self.limit {
            let extra = self.undo.len() - self.limit;
            self.undo.drain(..extra);
        }
    }

    pub fn undo_steps(&self) -> usize {
        self.undo.len()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_drops_the_oldest_steps() {
        let mut h = History::default();
        let mut m = Model::default();
        for i in 0..5 {
            h.push(&format!("step {i}"), m.clone());
        }
        h.set_limit(3);
        assert_eq!(h.undo_steps(), 3);
        assert_eq!(h.undo(&mut m).as_deref(), Some("step 4"));
        h.push("step 5", m.clone());
        h.push("step 6", m.clone());
        assert_eq!(h.undo_steps(), 3);
        let labels: Vec<String> = std::iter::from_fn(|| h.undo(&mut m)).collect();
        assert_eq!(labels, ["step 6", "step 5", "step 3"]);
    }
}
