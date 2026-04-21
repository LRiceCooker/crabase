use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::command_palette::CommandPalette;
use crate::header_bar::HeaderBar;
use crate::overlay::{self, ActiveOverlay};
use crate::restore_panel::RestorePanel;
use crate::settings::settings_view::SettingsView;
use crate::shortcuts::{self, ShortcutAction, use_save_trigger};
use crate::sidebar::saved_queries_list::SavedQueriesList;
use crate::sidebar::tables_list::TablesList;
use crate::sql_editor::sql_tab::SqlTab;
use crate::table_finder::TableFinder;
use crate::table_view::table_view::TableView;
use crate::tabs::tab_bar::{TabBar, TabKind, TabState};
use crate::tauri;

#[component]
pub fn MainLayout(on_disconnect: Callback<()>) -> impl IntoView {
    let (connection_info, set_connection_info) =
        signal(Option::<tauri::ConnectionInfo>::None);
    let (tables, set_tables) = signal(Vec::<String>::new());
    let (available_schemas, set_available_schemas) = signal(Vec::<String>::new());
    let (saved_query_names, set_saved_query_names) = signal(Vec::<String>::new());

    // Centralized overlay state (only one overlay open at a time)
    let overlay_ctx = overlay::use_overlay();

    // Tab state
    let tab_state = TabState::new();

    // Derived signal: name of the table in the active tab (if any)
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

    // Derived signal: is the active tab an SQL editor?
    let is_sql_tab = {
        let ts = tab_state.clone();
        Memo::new(move |_| {
            let active = ts.active_id.get();
            let tabs = ts.tabs.get();
            active.map(|id| {
                tabs.iter().any(|t| t.id == id && matches!(t.kind, TabKind::SqlEditor))
            }).unwrap_or(false)
        })
    };

    // Derived signal: active tab ID (for passing to child components)
    let active_tab_id = {
        let ts = tab_state.clone();
        Memo::new(move |_| ts.active_id.get().unwrap_or(0))
    };

    // Callback for when a table is clicked in the sidebar
    let on_table_select = {
        let ts = tab_state.clone();
        Callback::new(move |table_name: String| {
            ts.open(TabKind::TableView(table_name));
        })
    };

    // Command palette action handler
    let on_command = {
        let ts = tab_state.clone();
        Callback::new(move |cmd: String| {
            match cmd.as_str() {
                "New SQL Editor" => { ts.open(TabKind::SqlEditor); },
                "Restore Backup" => overlay_ctx.open(ActiveOverlay::Restore),
                "Settings" => overlay_ctx.open(ActiveOverlay::Settings),
                _ => {}
            }
        })
    };

    // Global keyboard shortcuts (dispatched via ShortcutsCtx)
    {
        let sc = shortcuts::use_shortcuts();
        let save_trigger = use_save_trigger();
        let closure = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
            move |ev: web_sys::KeyboardEvent| {
                if ev.key() == "Escape" && overlay_ctx.active.get_untracked() != ActiveOverlay::None {
                    ev.prevent_default();
                    ev.stop_propagation(); // Prevent CodeMirror from seeing Escape
                    overlay_ctx.close();
                } else if sc.matches(ShortcutAction::CommandPalette, &ev) {
                    ev.prevent_default();
                    ev.stop_propagation(); // Prevent CodeMirror from capturing the event
                    // Blur active element (e.g. CodeMirror) so the overlay input gets focus
                    if let Some(el) = web_sys::window().unwrap().document().unwrap().active_element() {
                        if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                            let _ = html_el.blur();
                        }
                    }
                    overlay_ctx.open(ActiveOverlay::CommandPalette);
                } else if sc.matches(ShortcutAction::TableFinder, &ev) {
                    ev.prevent_default();
                    ev.stop_propagation();
                    if let Some(el) = web_sys::window().unwrap().document().unwrap().active_element() {
                        if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                            let _ = html_el.blur();
                        }
                    }
                    overlay_ctx.open(ActiveOverlay::TableFinder);
                } else if sc.matches(ShortcutAction::Save, &ev) {
                    ev.prevent_default();
                    save_trigger.request();
                } else if (ev.meta_key() || ev.ctrl_key()) && ev.shift_key() && ev.code() == "KeyN" {
                    ev.prevent_default();
                    ev.stop_propagation();
                    wasm_bindgen_futures::spawn_local(async {
                        let _ = crate::tauri::open_new_window().await;
                    });
                }
            },
        );
        // Use capture phase (true) so we intercept shortcuts BEFORE CodeMirror
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback_and_bool("keydown", closure.as_ref().unchecked_ref(), true)
            .unwrap();
        closure.forget(); // App-lifetime: MainLayout lives for the entire session
    }

    // Fetch connection info, schemas, and tables on mount
    spawn_local({
        async move {
            if let Ok(info) = tauri::get_connection_info().await {
                // Fetch schemas using current connection string
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
        }
    });

    view! {
        <div class="h-screen flex flex-col bg-white dark:bg-neutral-950 overflow-hidden">
            // Command palette overlay
            <CommandPalette on_command=on_command />

            // Table finder overlay (Cmd+P)
            <TableFinder
                tables=tables
                saved_queries=saved_query_names
                on_select=on_table_select
                on_query_select=Callback::new({
                    let ts = tab_state.clone();
                    move |name: String| {
                        let existing = ts.tabs.get().iter().find(|t| {
                            matches!(&t.kind, TabKind::SqlEditor) && t.title == name
                        }).map(|t| t.id);
                        if let Some(id) = existing {
                            ts.switch(id);
                        } else {
                            let id = ts.open(TabKind::SqlEditor);
                            ts.rename_tab(id, name);
                        }
                    }
                })
            />

            // Header — h-10 with border-b
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

            // Body: sidebar (left) + tab bar + content area
            <div class="flex flex-1 overflow-hidden">
                // Left sidebar — scrolls independently
                <aside class="w-56 bg-gray-50 dark:bg-[#111113] border-r border-gray-200 dark:border-zinc-800 flex flex-col overflow-hidden shrink-0">
                    <SavedQueriesList
                        queries=saved_query_names
                        on_select=Callback::new({
                            let ts = tab_state.clone();
                            move |name: String| {
                                let existing = ts.tabs.get().iter().find(|t| {
                                    matches!(&t.kind, TabKind::SqlEditor) && t.title == name
                                }).map(|t| t.id);

                                if let Some(id) = existing {
                                    ts.switch(id);
                                } else {
                                    let id = ts.open(TabKind::SqlEditor);
                                    ts.rename_tab(id, name);
                                }
                            }
                        })
                        on_queries_changed=Callback::new(move |_: ()| {
                            spawn_local(async move {
                                if let Ok(queries) = tauri::list_queries().await {
                                    set_saved_query_names.set(queries.into_iter().map(|q| q.name).collect());
                                }
                            });
                        })
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

                // Right panel: tab bar + content
                <div class="flex-1 flex flex-col overflow-hidden">
                    // Tab bar — h-10
                    <TabBar
                        state=tab_state.clone()
                        on_tab_rename=Callback::new(move |(_tab_id, old_name, new_name): (usize, String, String)| {
                            spawn_local(async move {
                                let _ = tauri::rename_query(&old_name, &new_name).await;
                            });
                        })
                    />

                    // Content area — scrolls independently
                    <main class="flex-1 overflow-y-auto">
                        {move || {
                            if overlay_ctx.is_open(ActiveOverlay::Restore) {
                            view! {
                                <RestorePanel on_close=Callback::new(move |_: ()| {
                                    overlay_ctx.close();
                                }) />
                            }.into_any()
                        } else if overlay_ctx.is_open(ActiveOverlay::Settings) {
                            view! {
                                <SettingsView />
                            }.into_any()
                        } else if active_table.get().is_some() {
                            view! {
                                <div class="h-full">
                                    <TableView table_name=active_table />
                                </div>
                            }.into_any()
                        } else if is_sql_tab.get() {
                            let ts = tab_state.clone();
                            let tab_id = active_tab_id.get();
                            let on_dirty = Callback::new(move |dirty: bool| {
                                ts.set_dirty(tab_id, dirty);
                            });
                            // Get the tab title for query_name — use get_untracked to avoid
                            // re-creating the SqlTab (and losing editor content) on tab rename
                            let query_name = {
                                let tabs = tab_state.tabs.get_untracked();
                                tabs.iter()
                                    .find(|t| t.id == tab_id)
                                    .map(|t| t.title.clone())
                                    .unwrap_or_default()
                            };
                            view! {
                                <div class="h-full">
                                    <SqlTab
                                        query_name=query_name
                                        on_dirty_change=on_dirty
                                        on_query_saved=Callback::new(move |_: ()| {
                                            spawn_local(async move {
                                                if let Ok(queries) = tauri::list_queries().await {
                                                    set_saved_query_names.set(queries.into_iter().map(|q| q.name).collect());
                                                }
                                            });
                                        })
                                    />
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="flex items-center justify-center h-full text-gray-400 dark:text-zinc-500">
                                    <p class="text-[13px]">"Select a table to get started"</p>
                                </div>
                            }.into_any()
                        }
                    }}
                    </main>
                </div>
            </div>
        </div>
    }
}
