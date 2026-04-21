/// Row selection logic for the data table.
///
/// Handles single click, Cmd/Ctrl+click (toggle), Shift+click (range),
/// and Shift+Cmd/Ctrl+click (add range) selection patterns.
use std::collections::HashSet;

use leptos::prelude::*;

/// Handle a left-click on a row index cell.
///
/// Supports three selection modes based on modifier keys:
/// - **Plain click**: selects only the clicked row
/// - **Cmd/Ctrl+click**: toggles the clicked row in/out of the current selection
/// - **Shift+click**: selects a contiguous range from the anchor to the clicked row
/// - **Shift+Cmd/Ctrl+click**: adds the range to the existing selection
pub fn handle_row_click(
    ev: &web_sys::MouseEvent,
    row_idx: usize,
    selected_rows: RwSignal<HashSet<usize>>,
    selection_anchor: RwSignal<Option<usize>>,
) {
    if ev.shift_key() {
        // Shift+click: range select from anchor to clicked row
        let anchor = selection_anchor.get().unwrap_or(row_idx);
        let start = anchor.min(row_idx);
        let end = anchor.max(row_idx);
        let mut set = if ev.meta_key() || ev.ctrl_key() {
            // Shift+Cmd: add range to existing selection
            selected_rows.get()
        } else {
            HashSet::new()
        };
        for i in start..=end {
            set.insert(i);
        }
        selected_rows.set(set);
        // Don't update anchor on shift+click
    } else if ev.meta_key() || ev.ctrl_key() {
        // Cmd+click: toggle row in selection
        let mut set = selected_rows.get();
        if set.contains(&row_idx) {
            set.remove(&row_idx);
        } else {
            set.insert(row_idx);
        }
        selected_rows.set(set);
        selection_anchor.set(Some(row_idx));
    } else {
        // Plain click: select single row
        let mut new_set = HashSet::new();
        new_set.insert(row_idx);
        selected_rows.set(new_set);
        selection_anchor.set(Some(row_idx));
    }
}

/// Handle selection adjustment on right-click (context menu).
///
/// If the right-clicked row is already in the current selection, the selection
/// is preserved. Otherwise, the selection is replaced with just the clicked row.
pub fn handle_context_menu_selection(
    row_idx: usize,
    selected_rows: RwSignal<HashSet<usize>>,
    selection_anchor: RwSignal<Option<usize>>,
) {
    let sel = selected_rows.get();
    if !sel.contains(&row_idx) {
        let mut new_set = HashSet::new();
        new_set.insert(row_idx);
        selected_rows.set(new_set);
        selection_anchor.set(Some(row_idx));
    }
}
