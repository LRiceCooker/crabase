use std::collections::HashSet;

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::icons::{IconLoader, IconPlus, IconRefreshCw, IconTable};
use crate::overlay::{self, ActiveOverlay};
use crate::shortcuts::use_save_trigger;
use crate::table_view::cell_editor::CellEdit;
use crate::table_view::cell_editors::array_editor_modal::{ArrayEditRequest, ArrayEditorModal};
use crate::table_view::cell_editors::xml_editor_modal::{XmlEditRequest, XmlEditorModal};
use crate::table_view::change_tracker::ChangeTracker;
use crate::table_view::context_menu::ContextMenu;
use crate::table_view::data_table::{DataTable, RowContextMenuEvent};
use crate::table_view::row_actions;
use crate::table_view::save_handler;
use crate::table_view::filter_bar::FilterBar;
use crate::table_view::find_overlay::FindOverlay;
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
    let (array_edit, set_array_edit) = signal(Option::<ArrayEditRequest>::None);
    let (xml_edit, set_xml_edit) = signal(Option::<XmlEditRequest>::None);
    let changes = ChangeTracker::new();
    let selected_rows = RwSignal::new(HashSet::<usize>::new());
    let selection_anchor = RwSignal::new(Option::<usize>::None);
    // Context menu state: (x, y) position when open
    let (ctx_menu, set_ctx_menu) = signal(Option::<(i32, i32)>::None);
    // Filter & sort state
    let active_filters = RwSignal::new(Vec::<tauri::Filter>::new());
    let active_sort = RwSignal::new(Vec::<tauri::SortCol>::new());
    // Find overlay state (uses centralized overlay ctx)
    let overlay_ctx = overlay::use_overlay();
    let find_query = RwSignal::new(String::new());
    let find_current = RwSignal::new(0usize);
    let find_matches = Memo::new(move |_| {
        let query = find_query.get();
        if query.is_empty() {
            return Vec::new();
        }
        let query_lower = query.to_lowercase();
        let current_rows = rows.get();
        let _cols = columns.get();
        let mut matches = Vec::new();
        for (row_idx, row) in current_rows.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                let inner = crate::table_view::data_table::unwrap_tagged(cell);
                let text = match inner {
                    serde_json::Value::Null => "null".to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    _ => serde_json::to_string(inner).unwrap_or_default(),
                };
                if text.to_lowercase().contains(&query_lower) {
                    matches.push((row_idx, col_idx));
                }
            }
        }
        matches
    });
    let highlighted_cells = Memo::new(move |_| {
        let m = find_matches.get();
        let set: std::collections::HashSet<(usize, usize)> = m.into_iter().collect();
        set
    });

    // Fetch data helper (called when table, page, or page_size change)
    let fetch_data = move |name: String, pg: u32, ps: u32| {
        set_loading.set(true);
        set_error.set(None);
        changes.discard();
        selected_rows.set(HashSet::new());
        let filters = active_filters.get();
        let sort_cols = active_sort.get();

        spawn_local(async move {
            let result = if filters.is_empty() && sort_cols.is_empty() {
                tauri::get_table_data(&name, pg, ps).await
            } else {
                tauri::get_table_data_filtered(&name, pg, ps, filters, sort_cols).await
            };
            match result {
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
            active_filters.set(Vec::new());
            active_sort.set(Vec::new());
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

        // Compare the unwrapped original value with the new value to avoid phantom edits
        let original_inner = crate::table_view::data_table::unwrap_tagged_owned(&original);
        if original_inner == edit.value {
            return; // No actual change — skip tracking
        }

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

    // Array editor callbacks
    let on_array_edit = Callback::new(move |req: ArrayEditRequest| {
        set_array_edit.set(Some(req));
    });

    let on_array_save = Callback::new(move |(row, col, val): (usize, usize, serde_json::Value)| {
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
        set_array_edit.set(None);
    });

    let on_array_cancel = Callback::new(move |_| {
        set_array_edit.set(None);
    });

    // XML editor callbacks
    let on_xml_edit = Callback::new(move |req: XmlEditRequest| {
        set_xml_edit.set(Some(req));
    });

    let on_xml_save = Callback::new(move |(row, col, val): (usize, usize, serde_json::Value)| {
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
        set_xml_edit.set(None);
    });

    let on_xml_cancel = Callback::new(move |_| {
        set_xml_edit.set(None);
    });

    // Delete row callback
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
        let refetch = Callback::new(move |_: ()| {
            if let Some(name) = loaded_table.get() {
                fetch_data(name, page.get(), page_size.get());
            }
        });
        save_handler::execute_save(loaded_table, changes, rows, columns, refetch);
    });

    // Listen for global save trigger (Cmd+S)
    {
        let save_trigger = use_save_trigger();
        let counter = save_trigger.counter();
        Effect::new(move |prev: Option<u64>| {
            let current = counter.get();
            // Only trigger save if this isn't the initial run and there are changes
            if let Some(prev_val) = prev {
                if current != prev_val && changes.has_changes() {
                    on_save.run(());
                }
            }
            current
        });
    }

    view! {
        <div
            class="flex flex-col h-full"
            on:keydown=move |ev: web_sys::KeyboardEvent| {
                // Cmd+F / Ctrl+F → open find overlay
                if (ev.meta_key() || ev.ctrl_key()) && !ev.shift_key() && ev.code() == "KeyF"
                    && loaded_table.get().is_some() {
                        ev.prevent_default();
                        overlay_ctx.open(ActiveOverlay::FindBar);
                    }
            }
            tabindex="-1"
        >
            // Toolbar
            {move || {
                loaded_table.get().map(|name| {
                    let count = total_count.get();
                    view! {
                        <div class="h-10 flex items-center justify-between px-3 border-b border-gray-200 dark:border-zinc-800 bg-white dark:bg-neutral-950 shrink-0">
                            <div class="flex items-center gap-2">
                                <IconTable class="w-4 h-4 text-gray-400 dark:text-zinc-500" />
                                <span class="text-[13px] font-semibold text-gray-900 dark:text-neutral-50">{name}</span>
                                <span class="text-[11px] text-gray-400 dark:text-zinc-500">{format!("{count} rows")}</span>
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

            // Filter & sort bar
            {move || {
                loaded_table.get().map(|_| {
                    let columns_memo = Memo::new(move |_| columns.get());
                    let on_filter_change = Callback::new(move |_| {
                        if let Some(name) = loaded_table.get() {
                            fetch_data(name, page.get(), page_size.get());
                        }
                    });
                    view! {
                        <FilterBar
                            columns=columns_memo
                            filters=active_filters
                            sort=active_sort
                            on_change=on_filter_change
                        />
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
                        <div class="relative flex-1 flex flex-col overflow-hidden">
                            <FindOverlay
                                search_query=find_query
                                matches=find_matches
                                current_match=find_current
                                on_close=Callback::new(move |_| {
                                    overlay_ctx.close();
                                    find_query.set(String::new());
                                })
                            />
                            <DataTable
                                columns=columns.get()
                                rows=rows
                                changes=changes
                                selected_rows=selected_rows
                                selection_anchor=selection_anchor
                                page=page.get()
                                page_size=page_size.get()
                                on_cell_edit=on_cell_edit
                                on_json_edit=on_json_edit
                                on_array_edit=on_array_edit
                                on_xml_edit=on_xml_edit
                                on_row_context_menu=Callback::new(move |ev: RowContextMenuEvent| {
                                    set_ctx_menu.set(Some((ev.x, ev.y)));
                                })
                                active_sort=active_sort
                                on_sort_change=Callback::new(move |_| {
                                    if let Some(name) = loaded_table.get() {
                                        fetch_data(name, page.get(), page_size.get());
                                    }
                                })
                                highlighted_cells=highlighted_cells
                                find_current_match=(find_current, find_matches)
                            />
                        </div>
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

            // Array editor modal
            {move || {
                array_edit.get().map(|req| {
                    view! {
                        <ArrayEditorModal
                            request=req
                            on_save=on_array_save
                            on_cancel=on_array_cancel
                        />
                    }
                })
            }}

            // XML editor modal
            {move || {
                xml_edit.get().map(|req| {
                    view! {
                        <XmlEditorModal
                            request=req
                            on_save=on_xml_save
                            on_cancel=on_xml_cancel
                        />
                    }
                })
            }}

            // Context menu
            {move || {
                ctx_menu.get().map(|(x, y)| {
                    let items = row_actions::build_row_context_menu_items(
                        selected_rows,
                        rows,
                        columns,
                        changes,
                        table_name,
                    );
                    view! {
                        <ContextMenu
                            x=x
                            y=y
                            items=items
                            on_close=Callback::new(move |_| set_ctx_menu.set(None))
                        />
                    }
                })
            }}
        </div>
    }
}
