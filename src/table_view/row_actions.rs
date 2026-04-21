use std::collections::HashSet;

use leptos::prelude::*;

use crate::table_view::change_tracker::ChangeTracker;
use crate::table_view::context_menu::ContextMenuItem;
use crate::table_view::data_table::unwrap_tagged_owned;
use crate::tauri;

/// Builds context menu items for row actions: delete, duplicate, copy as JSON, copy as SQL.
pub fn build_row_context_menu_items(
    selected_rows: RwSignal<HashSet<usize>>,
    rows: RwSignal<Vec<Vec<serde_json::Value>>>,
    columns: ReadSignal<Vec<tauri::ColumnInfo>>,
    changes: ChangeTracker,
    table_name: Memo<Option<String>>,
) -> Vec<ContextMenuItem> {
    vec![
        delete_action(selected_rows, changes),
        duplicate_action(selected_rows, rows, changes),
        copy_as_json_action(selected_rows, rows, columns),
        copy_as_sql_action(selected_rows, rows, columns, table_name),
    ]
}

/// Marks all selected rows as deleted.
fn delete_action(
    selected_rows: RwSignal<HashSet<usize>>,
    changes: ChangeTracker,
) -> ContextMenuItem {
    ContextMenuItem {
        label: "Delete",
        danger: true,
        action: Callback::new(move |_| {
            let sel = selected_rows.get();
            for &row_idx in &sel {
                changes.mark_row_deleted(row_idx);
            }
        }),
    }
}

/// Duplicates selected rows by cloning their data and appending to the end.
fn duplicate_action(
    selected_rows: RwSignal<HashSet<usize>>,
    rows: RwSignal<Vec<Vec<serde_json::Value>>>,
    changes: ChangeTracker,
) -> ContextMenuItem {
    ContextMenuItem {
        label: "Duplicate",
        danger: false,
        action: Callback::new(move |_| {
            let sel = selected_rows.get();
            let current_rows = rows.get();
            let mut indices: Vec<usize> = sel.into_iter().collect();
            indices.sort();
            let mut new_rows_data: Vec<Vec<serde_json::Value>> = Vec::new();
            for &idx in &indices {
                if let Some(row_data) = current_rows.get(idx) {
                    new_rows_data.push(row_data.clone());
                }
            }
            rows.update(|r| {
                for new_row in &new_rows_data {
                    r.push(new_row.clone());
                }
            });
            let base = current_rows.len();
            for i in 0..new_rows_data.len() {
                changes.mark_row_added(base + i);
            }
        }),
    }
}

/// Copies selected rows as pretty-printed JSON to the clipboard.
fn copy_as_json_action(
    selected_rows: RwSignal<HashSet<usize>>,
    rows: RwSignal<Vec<Vec<serde_json::Value>>>,
    columns: ReadSignal<Vec<tauri::ColumnInfo>>,
) -> ContextMenuItem {
    ContextMenuItem {
        label: "Copy as JSON",
        danger: false,
        action: Callback::new(move |_| {
            let sel = selected_rows.get();
            let current_rows = rows.get();
            let cols = columns.get();
            let mut indices: Vec<usize> = sel.into_iter().collect();
            indices.sort();
            let json_rows: Vec<serde_json::Value> = indices
                .iter()
                .filter_map(|&idx| current_rows.get(idx))
                .map(|row| {
                    let mut obj = serde_json::Map::new();
                    for (col_idx, cell) in row.iter().enumerate() {
                        let col_name = cols
                            .get(col_idx)
                            .map(|c| c.name.clone())
                            .unwrap_or_else(|| format!("col_{col_idx}"));
                        obj.insert(col_name, unwrap_tagged_owned(cell));
                    }
                    serde_json::Value::Object(obj)
                })
                .collect();
            let text = if json_rows.len() == 1 {
                serde_json::to_string_pretty(&json_rows[0]).unwrap_or_default()
            } else {
                serde_json::to_string_pretty(&json_rows).unwrap_or_default()
            };
            let window = web_sys::window().unwrap();
            let clipboard = window.navigator().clipboard();
            let _ = clipboard.write_text(&text);
        }),
    }
}

/// Copies selected rows as SQL INSERT statements to the clipboard.
fn copy_as_sql_action(
    selected_rows: RwSignal<HashSet<usize>>,
    rows: RwSignal<Vec<Vec<serde_json::Value>>>,
    columns: ReadSignal<Vec<tauri::ColumnInfo>>,
    table_name: Memo<Option<String>>,
) -> ContextMenuItem {
    ContextMenuItem {
        label: "Copy as SQL INSERT",
        danger: false,
        action: Callback::new(move |_| {
            let sel = selected_rows.get();
            let current_rows = rows.get();
            let cols = columns.get();
            let tbl = table_name.get().unwrap_or_else(|| "table_name".to_string());
            let col_names: Vec<String> = cols.iter().map(|c| c.name.clone()).collect();
            let col_list = col_names.join(", ");
            let mut indices: Vec<usize> = sel.into_iter().collect();
            indices.sort();
            let stmts: Vec<String> = indices
                .iter()
                .filter_map(|&idx| current_rows.get(idx))
                .map(|row| {
                    let vals: Vec<String> = row
                        .iter()
                        .map(|cell| {
                            let v = unwrap_tagged_owned(cell);
                            match v {
                                serde_json::Value::Null => "NULL".to_string(),
                                serde_json::Value::Bool(b) => {
                                    if b { "TRUE".to_string() } else { "FALSE".to_string() }
                                }
                                serde_json::Value::Number(n) => n.to_string(),
                                serde_json::Value::String(s) => {
                                    format!("'{}'", s.replace('\'', "''"))
                                }
                                _ => {
                                    let s = serde_json::to_string(&v).unwrap_or_default();
                                    format!("'{}'", s.replace('\'', "''"))
                                }
                            }
                        })
                        .collect();
                    format!("INSERT INTO {} ({}) VALUES ({});", tbl, col_list, vals.join(", "))
                })
                .collect();
            let text = stmts.join("\n");
            let window = web_sys::window().unwrap();
            let clipboard = window.navigator().clipboard();
            let _ = clipboard.write_text(&text);
        }),
    }
}
