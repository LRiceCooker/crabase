use std::collections::HashSet;

use leptos::prelude::*;

use crate::icons::IconLoader;
use crate::overlay::{self, ActiveOverlay};
use crate::shortcuts::use_save_trigger;
use crate::table_view::cell_editor::CellEdit;
use crate::table_view::change_tracker::ChangeTracker;
use crate::table_view::context_menu::ContextMenu;
use crate::table_view::data_fetcher::DataFetcher;
use crate::table_view::data_table::{DataTable, RowContextMenuEvent};
use crate::table_view::dirty_bar::DirtyBar;
use crate::table_view::filter_bar::FilterBar;
use crate::table_view::find_overlay::FindOverlay;
use crate::table_view::find_state::FindState;
use crate::table_view::modal_editors::ModalEditors;
use crate::table_view::pagination::Pagination;
use crate::table_view::row_actions;
use crate::table_view::save_handler;
use crate::table_view::toolbar::Toolbar;

/// Main table view component — composes toolbar, filter bar, data table,
/// pagination, dirty bar, find overlay, modal editors, and context menu.
#[component]
pub fn TableView(table_name: Memo<Option<String>>) -> impl IntoView {
    let changes = ChangeTracker::new();
    let selected_rows = RwSignal::new(HashSet::<usize>::new());
    let selection_anchor = RwSignal::new(Option::<usize>::None);
    let (ctx_menu, set_ctx_menu) = signal(Option::<(i32, i32)>::None);
    let overlay_ctx = overlay::use_overlay();

    // Data fetching state
    let fetcher = DataFetcher::new(changes, selected_rows);
    fetcher.watch_table_name(table_name);

    // Find overlay state
    let find = FindState::new(fetcher.rows, fetcher.columns);

    // Modal editor state (JSON, Array, XML)
    let modals = ModalEditors::new(fetcher.rows, changes);

    let rows = fetcher.rows;
    let columns = fetcher.columns;
    let loaded_table = fetcher.loaded_table;

    // Cell edit callback
    let on_cell_edit = Callback::new(move |edit: CellEdit| {
        let original = rows.get()
            .get(edit.row)
            .and_then(|r| r.get(edit.col))
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        if crate::table_view::data_table::unwrap_tagged_owned(&original) == edit.value {
            return;
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

    // Add row callback
    let on_add_row = Callback::new(move |_: ()| {
        let col_count = columns.get().len();
        rows.update(|r| r.push(vec![serde_json::Value::Null; col_count]));
        changes.mark_row_added(rows.get().len() - 1);
    });

    // Save / discard
    let on_discard = fetcher.on_refresh();
    let on_save = {
        let refresh = fetcher.on_refresh();
        Callback::new(move |_: ()| {
            save_handler::execute_save(
                loaded_table, changes, rows, columns,
                Callback::new(move |_: ()| refresh.run(())),
            );
        })
    };

    // Listen for global Cmd+S save trigger
    {
        let save_trigger = use_save_trigger();
        let counter = save_trigger.counter();
        Effect::new(move |prev: Option<u64>| {
            let current = counter.get();
            if let Some(prev_val) = prev {
                if current != prev_val && changes.has_changes() {
                    on_save.run(());
                }
            }
            current
        });
    }

    // Pre-build callbacks for the view
    let on_page_change = fetcher.on_page_change();
    let on_page_size_change = fetcher.on_page_size_change();
    let on_refresh = fetcher.on_refresh();
    let on_filter_change = fetcher.on_filter_change();
    let on_sort_change = fetcher.on_filter_change();
    let on_json_edit = modals.on_json_edit();
    let on_array_edit = modals.on_array_edit();
    let on_xml_edit = modals.on_xml_edit();
    let (total_count, loading, error, has_data) =
        (fetcher.total_count, fetcher.loading, fetcher.error, fetcher.has_data);
    let (page, page_size) = (fetcher.page, fetcher.page_size);
    let (active_filters, active_sort) = (fetcher.active_filters, fetcher.active_sort);

    view! {
        <div
            class="flex flex-col h-full"
            on:keydown=move |ev: web_sys::KeyboardEvent| {
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
                    view! { <Toolbar table_name=name total_count=count on_add_row=on_add_row on_refresh=on_refresh /> }
                })
            }}

            // Filter & sort bar
            {move || {
                loaded_table.get().map(|_| {
                    let columns_memo = Memo::new(move |_| columns.get());
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
                                search_query=find.query
                                matches=find.matches
                                current_match=find.current
                                on_close=Callback::new(move |_| {
                                    overlay_ctx.close();
                                    find.query.set(String::new());
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
                                on_sort_change=on_sort_change
                                highlighted_cells=find.highlighted_cells
                                find_current_match=(find.current, find.matches)
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

            <DirtyBar changes=changes on_discard=on_discard on_save=on_save />

            {modals.view()}

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
