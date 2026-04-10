use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::icons::{IconLoader, IconPlus, IconRefreshCw, IconTable};
use crate::table_view::cell_editor::CellEdit;
use crate::table_view::change_tracker::ChangeTracker;
use crate::table_view::data_table::DataTable;
use crate::table_view::dirty_bar::DirtyBar;
use crate::table_view::json_editor::{JsonEditRequest, JsonEditorModal};
use crate::table_view::pagination::Pagination;
use crate::tauri;

#[component]
pub fn TableView(table_name: Memo<Option<String>>) -> impl IntoView {
    let (columns, set_columns) = signal(Vec::<tauri::ColumnInfo>::new());
    let rows = RwSignal::new(Vec::<Vec<serde_json::Value>>::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (loaded_table, set_loaded_table) = signal(Option::<String>::None);
    let (page, set_page) = signal(1u32);
    let (page_size, set_page_size) = signal(50u32);
    let (total_count, set_total_count) = signal(0u64);
    let (has_data, set_has_data) = signal(false);
    let (json_edit, set_json_edit) = signal(Option::<JsonEditRequest>::None);
    let changes = ChangeTracker::new();

    // Fetch data helper (called when table, page, or page_size change)
    let fetch_data = move |name: String, pg: u32, ps: u32| {
        set_loading.set(true);
        set_error.set(None);
        changes.discard();

        spawn_local(async move {
            match tauri::get_table_data(&name, pg, ps).await {
                Ok(td) => {
                    set_total_count.set(td.total_count);
                    set_columns.set(td.columns);
                    rows.set(td.rows);
                    set_has_data.set(true);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_has_data.set(false);
                }
            }
            set_loading.set(false);
        });
    };

    // Reactively fetch data when table_name changes
    Effect::new(move |_| {
        let name = table_name.get();
        let current = loaded_table.get();

        if name == current {
            return;
        }

        set_loaded_table.set(name.clone());

        if let Some(name) = name {
            set_page.set(1);
            fetch_data(name, 1, page_size.get());
        } else {
            set_has_data.set(false);
            set_total_count.set(0);
            set_loading.set(false);
        }
    });

    let on_page_change = Callback::new(move |new_page: u32| {
        set_page.set(new_page);
        if let Some(name) = loaded_table.get() {
            fetch_data(name, new_page, page_size.get());
        }
    });

    let on_page_size_change = Callback::new(move |new_size: u32| {
        set_page_size.set(new_size);
        set_page.set(1);
        if let Some(name) = loaded_table.get() {
            fetch_data(name, 1, new_size);
        }
    });

    let on_refresh = Callback::new(move |_: ()| {
        if let Some(name) = loaded_table.get() {
            fetch_data(name, page.get(), page_size.get());
        }
    });

    let on_cell_edit = Callback::new(move |edit: CellEdit| {
        // Get the original value before updating
        let original = rows.get()
            .get(edit.row)
            .and_then(|r| r.get(edit.col))
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        changes.track_cell_edit(edit.row, edit.col, original, &edit.value);

        rows.update(|r| {
            if let Some(row) = r.get_mut(edit.row) {
                if let Some(cell) = row.get_mut(edit.col) {
                    *cell = edit.value;
                }
            }
        });
    });

    let on_json_edit = Callback::new(move |req: JsonEditRequest| {
        set_json_edit.set(Some(req));
    });

    let on_json_save = Callback::new(move |(row, col, val): (usize, usize, serde_json::Value)| {
        let original = rows.get()
            .get(row)
            .and_then(|r| r.get(col))
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        changes.track_cell_edit(row, col, original, &val);

        rows.update(|r| {
            if let Some(row_data) = r.get_mut(row) {
                if let Some(cell) = row_data.get_mut(col) {
                    *cell = val;
                }
            }
        });
        set_json_edit.set(None);
    });

    let on_json_cancel = Callback::new(move |_| {
        set_json_edit.set(None);
    });

    // Delete row callback
    let on_delete_row = Callback::new(move |row_idx: usize| {
        changes.mark_row_deleted(row_idx);
    });

    // Add row callback
    let on_add_row = Callback::new(move |_: ()| {
        let col_count = columns.get().len();
        let new_row = vec![serde_json::Value::Null; col_count];
        rows.update(|r| {
            r.push(new_row);
        });
        let row_idx = rows.get().len() - 1;
        changes.mark_row_added(row_idx);
    });

    // Dirty bar callbacks
    let on_discard = Callback::new(move |_: ()| {
        // Re-fetch to restore original data
        if let Some(name) = loaded_table.get() {
            fetch_data(name, page.get(), page_size.get());
        }
    });

    let on_save = Callback::new(move |_: ()| {
        let table = match loaded_table.get() {
            Some(name) => name,
            None => return,
        };

        // Build the change set from tracked changes
        let modified = changes.modified_cells.get();
        let added_set = changes.added_rows.get();
        let deleted_set = changes.deleted_rows.get();
        let current_rows = rows.get();
        let cols = columns.get();

        // Find primary key columns
        let pk_cols: Vec<(usize, String)> = cols
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_primary_key)
            .map(|(i, c)| (i, c.name.clone()))
            .collect();

        // Build updates: group modified cells by row, exclude added/deleted rows
        let mut update_rows: std::collections::HashMap<usize, std::collections::HashMap<String, serde_json::Value>> =
            std::collections::HashMap::new();
        for ((row_idx, col_idx), _original) in &modified {
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
                let mut pk_values = std::collections::HashMap::new();
                for (pk_idx, pk_name) in &pk_cols {
                    pk_values.insert(pk_name.clone(), row.get(*pk_idx)?.clone());
                }
                Some(tauri::RowUpdate {
                    pk_values,
                    changes: change_map,
                })
            })
            .collect();

        // Build inserts
        let inserts: Vec<tauri::RowInsert> = added_set
            .iter()
            .filter_map(|row_idx| {
                let row = current_rows.get(*row_idx)?;
                let mut values = std::collections::HashMap::new();
                for (i, col) in cols.iter().enumerate() {
                    if let Some(val) = row.get(i) {
                        if !val.is_null() {
                            values.insert(col.name.clone(), val.clone());
                        }
                    }
                }
                Some(tauri::RowInsert { values })
            })
            .collect();

        // Build deletes
        let deletes: Vec<tauri::RowDelete> = deleted_set
            .iter()
            .filter_map(|row_idx| {
                let row = current_rows.get(*row_idx)?;
                let mut pk_values = std::collections::HashMap::new();
                for (pk_idx, pk_name) in &pk_cols {
                    pk_values.insert(pk_name.clone(), row.get(*pk_idx)?.clone());
                }
                Some(tauri::RowDelete { pk_values })
            })
            .collect();

        let change_set = tauri::ChangeSet {
            updates,
            inserts,
            deletes,
        };

        spawn_local(async move {
            match tauri::save_changes(&table, &change_set).await {
                Ok(_) => {
                    // Re-fetch data to show committed state
                    // (signals are Copy, so fetch_data closure works here)
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Save failed: {}", e).into());
                }
            }
        });

        // Re-fetch after save to show committed state
        if let Some(name) = loaded_table.get() {
            fetch_data(name, page.get(), page_size.get());
        }
    });

    view! {
        <div class="flex flex-col h-full">
            // Toolbar
            {move || {
                loaded_table.get().map(|name| {
                    let count = total_count.get();
                    view! {
                        <div class="h-10 flex items-center justify-between px-3 border-b border-gray-200 dark:border-zinc-800 bg-white dark:bg-neutral-950 shrink-0">
                            <div class="flex items-center gap-2">
                                <IconTable class="w-4 h-4 text-gray-400 dark:text-zinc-500" />
                                <span class="text-[13px] font-semibold text-gray-900 dark:text-neutral-50">{name}</span>
                                <span class="text-[11px] text-gray-400 dark:text-zinc-500">{format!("{} rows", count)}</span>
                            </div>
                            <div class="flex items-center gap-1">
                                <button
                                    class="text-gray-500 dark:text-zinc-400 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 px-2 py-1 rounded-md transition-colors duration-100 flex items-center gap-1 text-[13px]"
                                    title="Add row"
                                    on:click=move |_| on_add_row.run(())
                                >
                                    <IconPlus class="w-4 h-4" />
                                </button>
                                <button
                                    class="text-gray-500 dark:text-zinc-400 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 px-2 py-1 rounded-md transition-colors duration-100"
                                    title="Refresh"
                                    on:click=move |_| on_refresh.run(())
                                >
                                    <IconRefreshCw class="w-4 h-4" />
                                </button>
                            </div>
                        </div>
                    }
                })
            }}

            // Data content
            {move || {
                if loading.get() {
                    view! {
                        <div class="flex items-center justify-center flex-1 text-gray-400 dark:text-zinc-500">
                            <IconLoader class="w-5 h-5 animate-spin" />
                        </div>
                    }.into_any()
                } else if let Some(err) = error.get() {
                    view! {
                        <div class="flex items-center justify-center flex-1">
                            <p class="text-[13px] text-red-500 dark:text-red-400">{err}</p>
                        </div>
                    }.into_any()
                } else if has_data.get() {
                    view! {
                        <DataTable
                            columns=columns.get()
                            rows=rows
                            changes=changes
                            on_cell_edit=on_cell_edit
                            on_json_edit=on_json_edit
                            on_delete_row=on_delete_row
                        />
                    }.into_any()
                } else {
                    view! {
                        <div class="flex items-center justify-center flex-1 text-gray-400 dark:text-zinc-500">
                            <p class="text-[13px]">"Select a table to get started"</p>
                        </div>
                    }.into_any()
                }
            }}
            <Pagination
                page=page
                page_size=page_size
                total_count=total_count
                on_page_change=on_page_change
                on_page_size_change=on_page_size_change
            />

            // Dirty bar (floating)
            <DirtyBar changes=changes on_discard=on_discard on_save=on_save />

            // JSON editor modal
            {move || {
                json_edit.get().map(|req| {
                    view! {
                        <JsonEditorModal
                            request=req
                            on_save=on_json_save
                            on_cancel=on_json_cancel
                        />
                    }
                })
            }}
        </div>
    }
}
