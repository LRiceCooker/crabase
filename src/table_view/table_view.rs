use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::icons::{IconLoader, IconRefreshCw, IconTable};
use crate::table_view::cell_editor::CellEdit;
use crate::table_view::change_tracker::ChangeTracker;
use crate::table_view::data_table::DataTable;
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

    view! {
        <div class="flex flex-col h-full">
            // Toolbar
            {move || {
                loaded_table.get().map(|name| {
                    let count = total_count.get();
                    view! {
                        <div class="h-10 flex items-center justify-between px-3 border-b border-gray-200 bg-white shrink-0">
                            <div class="flex items-center gap-2">
                                <IconTable class="w-4 h-4 text-gray-400" />
                                <span class="text-[13px] font-semibold text-gray-900">{name}</span>
                                <span class="text-[11px] text-gray-400">{format!("{} rows", count)}</span>
                            </div>
                            <button
                                class="text-gray-500 hover:bg-gray-100 hover:text-gray-900 px-2 py-1 rounded-md transition-colors duration-100"
                                title="Refresh"
                                on:click=move |_| on_refresh.run(())
                            >
                                <IconRefreshCw class="w-4 h-4" />
                            </button>
                        </div>
                    }
                })
            }}

            // Data content
            {move || {
                if loading.get() {
                    view! {
                        <div class="flex items-center justify-center flex-1 text-gray-400">
                            <IconLoader class="w-5 h-5 animate-spin" />
                        </div>
                    }.into_any()
                } else if let Some(err) = error.get() {
                    view! {
                        <div class="flex items-center justify-center flex-1">
                            <p class="text-[13px] text-red-500">{err}</p>
                        </div>
                    }.into_any()
                } else if has_data.get() {
                    view! {
                        <DataTable
                            columns=columns.get()
                            rows=rows
                            changes=changes.clone()
                            on_cell_edit=on_cell_edit
                            on_json_edit=on_json_edit
                        />
                    }.into_any()
                } else {
                    view! {
                        <div class="flex items-center justify-center flex-1 text-gray-400">
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
