use leptos::prelude::*;
use std::collections::{HashMap, HashSet};

/// Tracks dirty state for table edits: modified cells, added rows, deleted rows.
#[derive(Clone, Copy)]
pub struct ChangeTracker {
    /// Map of (row, col) -> original value (before edit).
    pub modified_cells: RwSignal<HashMap<(usize, usize), serde_json::Value>>,
    /// Row indices of newly added rows.
    pub added_rows: RwSignal<HashSet<usize>>,
    /// Row indices marked for deletion.
    pub deleted_rows: RwSignal<HashSet<usize>>,
}

impl ChangeTracker {
    pub fn new() -> Self {
        Self {
            modified_cells: RwSignal::new(HashMap::new()),
            added_rows: RwSignal::new(HashSet::new()),
            deleted_rows: RwSignal::new(HashSet::new()),
        }
    }

    /// Record a cell modification. `original` is the value before editing.
    /// If the cell is edited back to its original value, untrack it.
    pub fn track_cell_edit(
        &self,
        row: usize,
        col: usize,
        original: serde_json::Value,
        new_value: &serde_json::Value,
    ) {
        self.modified_cells.update(|m| {
            let key = (row, col);
            // Only store the first original value (before any edits in this session)
            let orig = m.entry(key).or_insert(original);
            // If edited back to original, remove tracking
            if orig == new_value {
                m.remove(&key);
            }
        });
    }

    /// Mark a row as newly added.
    pub fn mark_row_added(&self, row: usize) {
        self.added_rows.update(|s| {
            s.insert(row);
        });
    }

    /// Mark a row for deletion.
    pub fn mark_row_deleted(&self, row: usize) {
        self.deleted_rows.update(|s| {
            s.insert(row);
        });
        // If an added row is deleted, just remove both markers
        if self.added_rows.get().contains(&row) {
            self.added_rows.update(|s| {
                s.remove(&row);
            });
            self.deleted_rows.update(|s| {
                s.remove(&row);
            });
        }
    }

    /// Unmark a row from deletion.
    pub fn unmark_row_deleted(&self, row: usize) {
        self.deleted_rows.update(|s| {
            s.remove(&row);
        });
    }

    pub fn is_cell_modified(&self, row: usize, col: usize) -> bool {
        self.modified_cells.get().contains_key(&(row, col))
    }

    pub fn is_row_modified(&self, row: usize) -> bool {
        self.modified_cells
            .get()
            .keys()
            .any(|(r, _)| *r == row)
    }

    pub fn is_row_added(&self, row: usize) -> bool {
        self.added_rows.get().contains(&row)
    }

    pub fn is_row_deleted(&self, row: usize) -> bool {
        self.deleted_rows.get().contains(&row)
    }

    /// Total number of pending changes.
    pub fn change_count(&self) -> usize {
        // Count unique modified rows (not added or deleted)
        let modified_rows: HashSet<usize> = self
            .modified_cells
            .get()
            .keys()
            .map(|(r, _)| *r)
            .collect();
        let added = self.added_rows.get();
        let deleted = self.deleted_rows.get();

        let modified_only: usize = modified_rows
            .iter()
            .filter(|r| !added.contains(r) && !deleted.contains(r))
            .count();

        modified_only + added.len() + deleted.len()
    }

    pub fn has_changes(&self) -> bool {
        !self.modified_cells.get().is_empty()
            || !self.added_rows.get().is_empty()
            || !self.deleted_rows.get().is_empty()
    }

    /// Discard all tracked changes.
    pub fn discard(&self) {
        self.modified_cells.set(HashMap::new());
        self.added_rows.set(HashSet::new());
        self.deleted_rows.set(HashSet::new());
    }
}
