use std::collections::HashMap;

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::table_view::change_tracker::ChangeTracker;
use crate::table_view::cell_format::unwrap_tagged_owned;
use crate::tauri;

/// Build a `ChangeSet` from the current tracked changes and persist it via the backend.
///
/// After a successful save the provided `refetch` callback is invoked so the
/// caller can reload the table data.
pub fn execute_save(
    loaded_table: ReadSignal<Option<String>>,
    changes: ChangeTracker,
    rows: RwSignal<Vec<Vec<serde_json::Value>>>,
    columns: ReadSignal<Vec<tauri::ColumnInfo>>,
    refetch: Callback<()>,
) {
    let table = match loaded_table.get() {
        Some(name) => name,
        None => return,
    };

    let change_set = build_change_set(&changes, &rows.get(), &columns.get());

    spawn_local(async move {
        match tauri::save_changes(&table, &change_set).await {
            Ok(_) => {
                refetch.run(());
            }
            Err(e) => {
                crate::log::log_error(&format!("Save failed: {e}"));
            }
        }
    });
}

/// Assemble a [`tauri::ChangeSet`] from the current tracker state, row data, and column metadata.
fn build_change_set(
    changes: &ChangeTracker,
    current_rows: &[Vec<serde_json::Value>],
    cols: &[tauri::ColumnInfo],
) -> tauri::ChangeSet {
    let modified = changes.modified_cells.get();
    let added_set = changes.added_rows.get();
    let deleted_set = changes.deleted_rows.get();

    // Find primary key columns
    let pk_cols: Vec<(usize, String)> = cols
        .iter()
        .enumerate()
        .filter(|(_, c)| c.is_primary_key)
        .map(|(i, c)| (i, c.name.clone()))
        .collect();

    // Build updates: group modified cells by row, exclude added/deleted rows
    let mut update_rows: HashMap<usize, HashMap<String, serde_json::Value>> = HashMap::new();
    for (row_idx, col_idx) in modified.keys() {
        if added_set.contains(row_idx) || deleted_set.contains(row_idx) {
            continue;
        }
        if let Some(row) = current_rows.get(*row_idx) {
            if let Some(col) = cols.get(*col_idx) {
                if let Some(val) = row.get(*col_idx) {
                    update_rows
                        .entry(*row_idx)
                        .or_default()
                        .insert(col.name.clone(), val.clone());
                }
            }
        }
    }

    let updates: Vec<tauri::RowUpdate> = update_rows
        .into_iter()
        .filter_map(|(row_idx, change_map)| {
            let row = current_rows.get(row_idx)?;
            let pk_values: HashMap<String, serde_json::Value> = pk_cols
                .iter()
                .map(|(pk_idx, pk_name)| {
                    row.get(*pk_idx)
                        .map(|v| (pk_name.clone(), unwrap_tagged_owned(v)))
                })
                .collect::<Option<HashMap<_, _>>>()?;
            let unwrapped_changes: HashMap<String, serde_json::Value> = change_map
                .into_iter()
                .map(|(k, v)| (k, unwrap_tagged_owned(&v)))
                .collect();
            Some(tauri::RowUpdate {
                pk_values,
                changes: unwrapped_changes,
            })
        })
        .collect();

    // Build inserts
    let inserts: Vec<tauri::RowInsert> = added_set
        .iter()
        .filter_map(|row_idx| {
            let row = current_rows.get(*row_idx)?;
            let values: HashMap<String, serde_json::Value> = cols
                .iter()
                .enumerate()
                .filter_map(|(i, col)| {
                    let raw = unwrap_tagged_owned(row.get(i)?);
                    (!raw.is_null()).then(|| (col.name.clone(), raw))
                })
                .collect();
            Some(tauri::RowInsert { values })
        })
        .collect();

    // Build deletes
    let deletes: Vec<tauri::RowDelete> = deleted_set
        .iter()
        .filter_map(|row_idx| {
            let row = current_rows.get(*row_idx)?;
            let pk_values: HashMap<String, serde_json::Value> = pk_cols
                .iter()
                .map(|(pk_idx, pk_name)| {
                    row.get(*pk_idx)
                        .map(|v| (pk_name.clone(), unwrap_tagged_owned(v)))
                })
                .collect::<Option<HashMap<_, _>>>()?;
            Some(tauri::RowDelete { pk_values })
        })
        .collect();

    tauri::ChangeSet {
        updates,
        inserts,
        deletes,
    }
}
