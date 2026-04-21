use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::command_palette::CommandPalette;
use crate::content_area::ContentArea;
use crate::global_shortcuts;
use crate::header_bar::HeaderBar;
use crate::overlay::{self, ActiveOverlay};
use crate::sidebar::saved_queries_list::SavedQueriesList;
use crate::sidebar::tables_list::TablesList;
use crate::table_finder::TableFinder;
use crate::tabs::tab_bar::{TabBar, TabKind, TabState};
use crate::tauri;

/// Root layout component composing HeaderBar, Sidebar, TabBar, and ContentArea.
#[component]
pub fn MainLayout(on_disconnect: Callback<()>) -> impl IntoView {
    let (connection_info, set_connection_info) =
        signal(Option::<tauri::ConnectionInfo>::None);
    let (tables, set_tables) = signal(Vec::<String>::new());
    let (available_schemas, set_available_schemas) = signal(Vec::<String>::new());
    let (saved_query_names, set_saved_query_names) = signal(Vec::<String>::new());

    let overlay_ctx = overlay::use_overlay();
    let tab_state = TabState::new();

    // Derived signals for active tab state
    let active_table = {
        let ts = tab_state.clone();
        Memo::new(move |_| {
            let active = ts.active_id.get();
            let tabs = ts.tabs.get();
            active.and_then(|id| {
                tabs.iter().find(|t| t.id == id).and_then(|t| match &t.kind {
                    TabKind::TableView(name) => Some(name.clone()),
                    _ => None,
                })
            })
        })
    };

    let is_sql_tab = {
        let ts = tab_state.clone();
        Memo::new(move |_| {
            let active = ts.active_id.get();
            let tabs = ts.tabs.get();
            active
                .map(|id| tabs.iter().any(|t| t.id == id && matches!(t.kind, TabKind::SqlEditor)))
                .unwrap_or(false)
        })
    };

    let active_tab_id = {
        let ts = tab_state.clone();
        Memo::new(move |_| ts.active_id.get().unwrap_or(0))
    };

    // Shared callbacks
    let on_table_select = {
        let ts = tab_state.clone();
        Callback::new(move |table_name: String| {
            ts.open(TabKind::TableView(table_name));
        })
    };

    let on_query_select = {
        let ts = tab_state.clone();
        Callback::new(move |name: String| {
            let existing = ts
                .tabs
                .get()
                .iter()
                .find(|t| matches!(&t.kind, TabKind::SqlEditor) && t.title == name)
                .map(|t| t.id);
            if let Some(id) = existing {
                ts.switch(id);
            } else {
                let id = ts.open(TabKind::SqlEditor);
                ts.rename_tab(id, name);
            }
        })
    };

    let on_queries_refresh = Callback::new(move |_: ()| {
        spawn_local(async move {
            if let Ok(queries) = tauri::list_queries().await {
                set_saved_query_names.set(queries.into_iter().map(|q| q.name).collect());
            }
        });
    });

    let on_command = {
        let ts = tab_state.clone();
        Callback::new(move |cmd: String| {
            match cmd.as_str() {
                "New SQL Editor" => {
                    ts.open(TabKind::SqlEditor);
                }
                "Restore Backup" => overlay_ctx.open(ActiveOverlay::Restore),
                "Settings" => overlay_ctx.open(ActiveOverlay::Settings),
                _ => {}
            }
        })
    };

    // Global keyboard shortcuts (Escape, Cmd+K, Cmd+P, Cmd+S, Cmd+Shift+N)
    global_shortcuts::setup_global_shortcuts();

    // Fetch connection info, schemas, and tables on mount
    spawn_local(async move {
        if let Ok(info) = tauri::get_connection_info().await {
            let cs = tauri::build_connection_string_js(&info);
            if let Ok(schemas) = tauri::list_schemas(&cs).await {
                set_available_schemas.set(schemas);
            }
            set_connection_info.set(Some(info));
        }
        if let Ok(t) = tauri::list_tables().await {
            set_tables.set(t);
        }
        if let Ok(queries) = tauri::list_queries().await {
            set_saved_query_names.set(queries.into_iter().map(|q| q.name).collect());
        }
    });

    view! {
        <div class="h-screen flex flex-col bg-white dark:bg-neutral-950 overflow-hidden">
            <CommandPalette on_command=on_command />
            <TableFinder
                tables=tables
                saved_queries=saved_query_names
                on_select=on_table_select
                on_query_select=on_query_select
            />

            <HeaderBar
                connection_info=connection_info
                set_connection_info=set_connection_info
                available_schemas=available_schemas
                set_tables=set_tables
                on_disconnect=on_disconnect
                on_new_sql_editor=Callback::new({
                    let ts = tab_state.clone();
                    move |_: ()| { ts.open(TabKind::SqlEditor); }
                })
            />

            // Body: sidebar + tab bar + content
            <div class="flex flex-1 overflow-hidden">
                <aside class="w-56 bg-gray-50 dark:bg-[#111113] border-r border-gray-200 dark:border-zinc-800 flex flex-col overflow-hidden shrink-0">
                    <SavedQueriesList
                        queries=saved_query_names
                        on_select=on_query_select
                        on_queries_changed=on_queries_refresh
                    />
                    <TablesList
                        tables=tables
                        active_table=active_table
                        on_select=on_table_select
                        on_tables_changed=Callback::new(move |_: ()| {
                            spawn_local(async move {
                                if let Ok(t) = tauri::list_tables().await {
                                    set_tables.set(t);
                                }
                            });
                        })
                    />
                </aside>

                <div class="flex-1 flex flex-col overflow-hidden">
                    <TabBar
                        state=tab_state.clone()
                        on_tab_rename=Callback::new(move |(_tab_id, old_name, new_name): (usize, String, String)| {
                            spawn_local(async move {
                                let _ = tauri::rename_query(&old_name, &new_name).await;
                            });
                        })
                    />
                    <ContentArea
                        active_table=active_table
                        is_sql_tab=is_sql_tab
                        active_tab_id=active_tab_id
                        tab_state=tab_state.clone()
                        on_query_saved=on_queries_refresh
                    />
                </div>
            </div>
        </div>
    }
}
