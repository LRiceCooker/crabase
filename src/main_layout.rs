use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::command_palette::CommandPalette;
use crate::icons::{
    IconAlertTriangle, IconCheckCircle, IconDatabase, IconEdit, IconFile, IconLoader, IconLogOut,
    IconPlus, IconUpload, IconX, IconXCircle,
};
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

    // Header editing state
    let (editing, set_editing) = signal(false);
    let (edit_host, set_edit_host) = signal(String::new());
    let (edit_port, set_edit_port) = signal(String::new());
    let (edit_user, set_edit_user) = signal(String::new());
    let (edit_dbname, set_edit_dbname) = signal(String::new());
    let (edit_password, set_edit_password) = signal(String::new());
    let (edit_schema, set_edit_schema) = signal(String::new());
    let (edit_ssl, set_edit_ssl) = signal(false);
    let (reconnecting, set_reconnecting) = signal(false);
    let (header_error, set_header_error) = signal(Option::<String>::None);

    // Command palette state
    let (show_palette, set_show_palette) = signal(false);

    // Table finder state (Cmd+P)
    let (show_finder, set_show_finder) = signal(false);

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

    // Settings panel state
    let (show_settings, set_show_settings) = signal(false);

    // Restore panel state
    let (show_restore, set_show_restore) = signal(false);
    let (restore_file, set_restore_file) = signal(Option::<String>::None);
    let (restore_picking, set_restore_picking) = signal(false);
    let (restore_running, set_restore_running) = signal(false);
    let (restore_logs, set_restore_logs) = signal(Vec::<String>::new());
    let (restore_status, set_restore_status) = signal(Option::<Result<String, String>>::None);

    // Command palette action handler
    let on_command = {
        let ts = tab_state.clone();
        Callback::new(move |cmd: String| {
            match cmd.as_str() {
                "New SQL Editor" => { ts.open(TabKind::SqlEditor); },
                "Restore Backup" => set_show_restore.set(true),
                "Settings" => set_show_settings.set(true),
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
                if sc.matches(ShortcutAction::CommandPalette, &ev) {
                    ev.prevent_default();
                    set_show_palette.set(true);
                } else if sc.matches(ShortcutAction::TableFinder, &ev) {
                    ev.prevent_default();
                    set_show_finder.set(true);
                } else if sc.matches(ShortcutAction::Save, &ev) {
                    ev.prevent_default();
                    save_trigger.request();
                }
            },
        );
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Fetch connection info, schemas, and tables on mount
    spawn_local({
        let set_connection_info = set_connection_info.clone();
        let set_tables = set_tables.clone();
        let set_available_schemas = set_available_schemas.clone();
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

    // Enter edit mode: populate fields from current connection info
    let on_edit = move |_| {
        if let Some(info) = connection_info.get() {
            set_edit_host.set(info.host.clone());
            set_edit_port.set(info.port.to_string());
            set_edit_user.set(info.user.clone());
            set_edit_dbname.set(info.dbname.clone());
            set_edit_password.set(info.password.clone());
            set_edit_schema.set(info.schema.clone());
            set_edit_ssl.set(info.sslmode == "require");
            set_header_error.set(None);
            set_editing.set(true);
        }
    };

    // Cancel edit mode
    let on_cancel = move |_| {
        set_editing.set(false);
        set_header_error.set(None);
    };

    // Reconnect with edited fields
    let on_reconnect = move |_| {
        let info = tauri::ConnectionInfo {
            host: edit_host.get(),
            port: edit_port.get().parse().unwrap_or(5432),
            user: edit_user.get(),
            password: edit_password.get(),
            dbname: edit_dbname.get(),
            schema: edit_schema.get(),
            sslmode: if edit_ssl.get() { "require".to_string() } else { "disable".to_string() },
        };

        set_reconnecting.set(true);
        set_header_error.set(None);

        spawn_local(async move {
            let _ = tauri::disconnect_db().await;

            match tauri::connect_db(&info).await {
                Ok(_) => {
                    if let Ok(info) = tauri::get_connection_info().await {
                        set_connection_info.set(Some(info));
                    }
                    if let Ok(t) = tauri::list_tables().await {
                        set_tables.set(t);
                    }
                    set_editing.set(false);
                }
                Err(e) => {
                    set_header_error.set(Some(e));
                }
            }
            set_reconnecting.set(false);
        });
    };

    let header_input_class = "bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-800 rounded-md px-2 py-1 text-[13px] text-gray-900 dark:text-neutral-50 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 dark:focus:ring-indigo-500/60 focus:border-indigo-500 transition-colors duration-100";

    view! {
        <div class="h-screen flex flex-col bg-white dark:bg-neutral-950 overflow-hidden">
            // Command palette overlay
            <CommandPalette show=show_palette set_show=set_show_palette on_command=on_command />

            // Table finder overlay (Cmd+P)
            <TableFinder
                show=show_finder
                set_show=set_show_finder
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
            <header class="h-10 flex items-center justify-between px-4 border-b border-gray-200 dark:border-zinc-800 bg-white dark:bg-neutral-950 shrink-0">
                <div class="flex items-center gap-2">
                    <IconDatabase class="w-4 h-4 text-indigo-500 dark:text-indigo-400" />
                    <span class="text-base font-semibold text-gray-900 dark:text-neutral-50">"crabase"</span>
                    <button
                        class="text-gray-400 dark:text-zinc-500 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 p-1 rounded-md transition-colors duration-100"
                        title="New SQL Editor"
                        on:click={
                            let ts = tab_state.clone();
                            move |_| { ts.open(TabKind::SqlEditor); }
                        }
                    >
                        <IconPlus class="w-4 h-4" />
                    </button>
                </div>
                <div class="flex items-center gap-2 text-[13px]">
                    {move || {
                        if editing.get() {
                            // Edit mode: input fields
                            view! {
                                <div class="flex items-center gap-1.5">
                                    <input
                                        type="text"
                                        placeholder="user"
                                        class=format!("{} w-20", header_input_class)
                                        prop:value=move || edit_user.get()
                                        on:input=move |ev| set_edit_user.set(event_target_value(&ev))
                                    />
                                    <span class="text-gray-400 dark:text-zinc-500">"@"</span>
                                    <input
                                        type="text"
                                        placeholder="host"
                                        class=format!("{} w-28", header_input_class)
                                        prop:value=move || edit_host.get()
                                        on:input=move |ev| set_edit_host.set(event_target_value(&ev))
                                    />
                                    <span class="text-gray-400 dark:text-zinc-500">":"</span>
                                    <input
                                        type="text"
                                        placeholder="port"
                                        class=format!("{} w-14", header_input_class)
                                        prop:value=move || edit_port.get()
                                        on:input=move |ev| set_edit_port.set(event_target_value(&ev))
                                    />
                                    <span class="text-gray-400 dark:text-zinc-500">"/"</span>
                                    <input
                                        type="text"
                                        placeholder="dbname"
                                        class=format!("{} w-20", header_input_class)
                                        prop:value=move || edit_dbname.get()
                                        on:input=move |ev| set_edit_dbname.set(event_target_value(&ev))
                                    />
                                    <input
                                        type="password"
                                        placeholder="password"
                                        class=format!("{} w-20", header_input_class)
                                        prop:value=move || edit_password.get()
                                        on:input=move |ev| set_edit_password.set(event_target_value(&ev))
                                    />
                                    <button
                                        class="bg-indigo-500 hover:bg-indigo-600 dark:hover:bg-indigo-400 text-white text-[13px] font-medium px-2 py-1 rounded-md transition-colors duration-100 disabled:opacity-50"
                                        disabled=move || reconnecting.get()
                                        on:click=on_reconnect
                                    >
                                        {move || if reconnecting.get() {
                                            "Reconnecting..."
                                        } else {
                                            "Reconnect"
                                        }}
                                    </button>
                                    <button
                                        class="text-gray-500 dark:text-zinc-400 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 px-2 py-1 rounded-md text-[13px] transition-colors duration-100"
                                        disabled=move || reconnecting.get()
                                        on:click=on_cancel
                                    >
                                        "Cancel"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            // Read-only mode: badges + schema select + edit button
                            view! {
                                <div class="flex items-center gap-2">
                                    {move || connection_info.get().map(|info| view! {
                                        <span class="text-[11px] font-medium text-gray-500 dark:text-zinc-400 bg-gray-50 dark:bg-[#111113] border border-gray-200 dark:border-zinc-800 rounded px-1.5 py-0.5">
                                            {format!("{}@{}", info.user, info.host)}
                                        </span>
                                        <span class="text-[11px] font-medium text-gray-500 dark:text-zinc-400 bg-gray-50 dark:bg-[#111113] border border-gray-200 dark:border-zinc-800 rounded px-1.5 py-0.5">
                                            {format!(":{}", info.port)}
                                        </span>
                                        <span class="text-[11px] font-medium text-white bg-indigo-500 rounded px-1.5 py-0.5">
                                            {info.dbname.clone()}
                                        </span>
                                    })}
                                    // Schema select
                                    {move || {
                                        let schemas = available_schemas.get();
                                        let current = connection_info.get().map(|i| i.schema.clone()).unwrap_or_default();
                                        if schemas.is_empty() {
                                            view! {
                                                <select class="bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-800 rounded-md px-2 py-0.5 text-[11px] text-gray-900 dark:text-neutral-50 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 dark:focus:ring-indigo-500/60 focus:border-indigo-500">
                                                    <option>{current}</option>
                                                </select>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <select
                                                    class="bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-800 rounded-md px-2 py-0.5 text-[11px] text-gray-900 dark:text-neutral-50 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 dark:focus:ring-indigo-500/60 focus:border-indigo-500"
                                                    on:change=move |ev| {
                                                        let new_schema = event_target_value(&ev);
                                                        // Reconnect with new schema
                                                        if let Some(mut info) = connection_info.get() {
                                                            info.schema = new_schema;
                                                            let new_info = info.clone();
                                                            spawn_local(async move {
                                                                let _ = tauri::disconnect_db().await;
                                                                if let Ok(_) = tauri::connect_db(&new_info).await {
                                                                    if let Ok(updated) = tauri::get_connection_info().await {
                                                                        set_connection_info.set(Some(updated));
                                                                    }
                                                                    if let Ok(t) = tauri::list_tables().await {
                                                                        set_tables.set(t);
                                                                    }
                                                                }
                                                            });
                                                        }
                                                    }
                                                >
                                                    {schemas.into_iter().map(|s| {
                                                        let selected = s == current;
                                                        let s2 = s.clone();
                                                        view! { <option value={s} selected=selected>{s2}</option> }
                                                    }).collect::<Vec<_>>()}
                                                </select>
                                            }.into_any()
                                        }
                                    }}
                                    <button
                                        class="text-gray-400 dark:text-zinc-500 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 p-1 rounded-md transition-colors duration-100"
                                        title="Edit connection"
                                        on:click=on_edit
                                    >
                                        <IconEdit class="w-4 h-4" />
                                    </button>
                                    <div class="w-px h-4 bg-gray-200 dark:bg-zinc-800"></div>
                                    <button
                                        class="text-gray-400 dark:text-zinc-500 hover:bg-red-50 dark:hover:bg-red-950/60 hover:text-red-600 dark:hover:text-red-400 p-1 rounded-md transition-colors duration-100"
                                        title="Disconnect"
                                        on:click=move |_| {
                                            spawn_local(async move {
                                                let _ = tauri::disconnect_db().await;
                                                on_disconnect.run(());
                                            });
                                        }
                                    >
                                        <IconLogOut class="w-4 h-4" />
                                    </button>
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </header>

            // Header error message
            {move || header_error.get().map(|msg| view! {
                <div class="flex items-center gap-2 mx-4 mt-2 px-3 py-2 bg-red-50 dark:bg-red-950/60 border border-red-200 dark:border-red-800 rounded-md">
                    <IconAlertTriangle class="w-4 h-4 text-red-500 dark:text-red-400 shrink-0" />
                    <span class="text-[13px] text-red-700 dark:text-red-400">{msg}</span>
                </div>
            })}

            // Body: sidebar (left) + tab bar + content area
            <div class="flex flex-1 overflow-hidden">
                // Left sidebar — scrolls independently
                <aside class="w-56 bg-gray-50 dark:bg-[#111113] border-r border-gray-200 dark:border-zinc-800 overflow-y-auto shrink-0">
                    <SavedQueriesList
                        queries=saved_query_names
                        on_select=Callback::new({
                            let ts = tab_state.clone();
                            move |name: String| {
                                // Check if a tab with this name already exists
                                let existing = ts.tabs.get().iter().find(|t| {
                                    matches!(&t.kind, TabKind::SqlEditor) && t.title == name
                                }).map(|t| t.id);

                                if let Some(id) = existing {
                                    ts.switch(id);
                                } else {
                                    // Open a new SQL tab and rename it
                                    let id = ts.open(TabKind::SqlEditor);
                                    ts.rename_tab(id, name);
                                }
                            }
                        })
                    />
                    <TablesList tables=tables active_table=active_table on_select=on_table_select />
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
                            if show_restore.get() {
                            let on_pick_file = move |_| {
                                set_restore_picking.set(true);
                                spawn_local(async move {
                                    match tauri::pick_backup_file().await {
                                        Ok(Some(path)) => set_restore_file.set(Some(path)),
                                        Ok(None) => {} // User cancelled
                                        Err(_) => {}
                                    }
                                    set_restore_picking.set(false);
                                });
                            };

                            let on_close = move |_| {
                                set_show_restore.set(false);
                                set_restore_file.set(None);
                                set_restore_logs.set(Vec::new());
                                set_restore_status.set(None);
                            };

                            let on_restore = move |_| {
                                if let Some(file_path) = restore_file.get() {
                                    set_restore_running.set(true);
                                    set_restore_logs.set(Vec::new());
                                    set_restore_status.set(None);
                                    spawn_local(async move {
                                        // Set up event listener for real-time logs
                                        let unlisten = tauri::listen_restore_logs(move |line| {
                                            set_restore_logs.update(|logs| logs.push(line));
                                        }).await;

                                        // Run the restore
                                        let result = tauri::restore_backup(&file_path).await;

                                        // Clean up event listener
                                        if let Ok(unlisten_fn) = &unlisten {
                                            let _ = unlisten_fn.call0(&wasm_bindgen::JsValue::NULL);
                                        }

                                        // Log the final result and set status
                                        match &result {
                                            Ok(msg) => {
                                                set_restore_logs.update(|logs| logs.push(msg.clone()));
                                            }
                                            Err(msg) => {
                                                set_restore_logs.update(|logs| logs.push(format!("ERROR: {}", msg)));
                                            }
                                        }
                                        set_restore_status.set(Some(result));
                                        set_restore_running.set(false);
                                    });
                                }
                            };

                            view! {
                                <div class="bg-white dark:bg-zinc-900 rounded-lg border border-gray-200 dark:border-zinc-800 shadow-lg dark:shadow-black/40 max-w-lg mx-auto mt-8 dark:ring-1 dark:ring-white/[0.06]">
                                    // Header
                                    <div class="px-4 py-3 border-b border-gray-200 dark:border-zinc-800 flex items-center justify-between">
                                        <h2 class="text-[13px] font-semibold text-gray-900 dark:text-neutral-50">"Restore Backup"</h2>
                                        <button
                                            class="text-gray-400 dark:text-zinc-500 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 p-1 rounded-md transition-colors duration-100"
                                            disabled=move || restore_running.get()
                                            on:click=on_close
                                        >
                                            <IconX class="w-4 h-4" />
                                        </button>
                                    </div>
                                    // Body
                                    <div class="px-4 py-4">
                                        <p class="text-[13px] text-gray-500 dark:text-zinc-400 mb-4">"Restore a .tar.gz PostgreSQL backup to the connected database."</p>

                                        // File selector
                                        <div class="flex flex-col gap-1.5">
                                            <label class="text-[13px] font-normal text-gray-700 dark:text-zinc-300">"Backup file (.tar.gz)"</label>
                                            <div class="flex items-center gap-2">
                                                <button
                                                    class="bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-800 text-gray-700 dark:text-zinc-300 hover:bg-gray-50 dark:hover:bg-white/[0.03] text-[13px] px-3 py-1.5 rounded-md transition-colors duration-100 flex items-center gap-1.5 disabled:opacity-50"
                                                    disabled=move || restore_picking.get() || restore_running.get()
                                                    on:click=on_pick_file
                                                >
                                                    <IconUpload class="w-4 h-4 text-gray-400" />
                                                    {move || if restore_picking.get() {
                                                        "Selecting..."
                                                    } else {
                                                        "Choose file..."
                                                    }}
                                                </button>
                                                <span class="text-[13px] text-gray-500 dark:text-zinc-400 truncate max-w-xs flex items-center gap-1.5">
                                                    {move || restore_file.get().map(|f| view! {
                                                        <IconFile class="w-4 h-4 text-gray-400 shrink-0" />
                                                        <span class="truncate">{f}</span>
                                                    })}
                                                    {move || if restore_file.get().is_none() {
                                                        Some(view! { <span class="text-gray-400 dark:text-zinc-500 italic">"No file selected"</span> })
                                                    } else {
                                                        None
                                                    }}
                                                </span>
                                            </div>
                                        </div>

                                        // Restore button
                                        <div class="flex justify-end mt-4">
                                            <button
                                                class="bg-indigo-500 hover:bg-indigo-600 dark:hover:bg-indigo-400 text-white text-[13px] font-medium px-3 py-1.5 rounded-md transition-colors duration-100 disabled:opacity-50 disabled:cursor-not-allowed"
                                                disabled=move || restore_file.get().is_none() || restore_running.get()
                                                on:click=on_restore
                                            >
                                                {move || if restore_running.get() {
                                                    view! {
                                                        <span class="flex items-center gap-2">
                                                            <IconLoader class="w-4 h-4 animate-spin" />
                                                            "Restoring..."
                                                        </span>
                                                    }.into_any()
                                                } else {
                                                    view! { <span>"Start restore"</span> }.into_any()
                                                }}
                                            </button>
                                        </div>

                                        // Real-time log display
                                        {move || {
                                            let logs = restore_logs.get();
                                            if !logs.is_empty() {
                                                Some(view! {
                                                    <div class="mt-4">
                                                        <label class="text-[13px] font-semibold text-gray-700 dark:text-zinc-300 mb-1.5 block">"Logs"</label>
                                                        <div class="bg-gray-900 dark:bg-[#0D0D0F] text-gray-300 dark:text-zinc-200 rounded-md p-3 max-h-60 overflow-y-auto font-mono text-xs">
                                                            {logs.into_iter().map(|line| view! {
                                                                <div class="whitespace-pre-wrap">{line}</div>
                                                            }).collect::<Vec<_>>()}
                                                        </div>
                                                    </div>
                                                })
                                            } else {
                                                None
                                            }
                                        }}

                                        // Success/failure indicator
                                        {move || {
                                            let status = restore_status.get();
                                            match status {
                                                Some(Ok(_)) => view! {
                                                    <div class="flex items-center gap-2 mt-4 px-3 py-2 bg-emerald-50 dark:bg-emerald-950/60 border border-emerald-200 dark:border-emerald-800 rounded-md">
                                                        <IconCheckCircle class="w-4 h-4 text-emerald-500 dark:text-emerald-400 shrink-0" />
                                                        <span class="text-[13px] text-emerald-700 dark:text-emerald-400">"Restore completed successfully."</span>
                                                    </div>
                                                }.into_any(),
                                                Some(Err(ref msg)) => view! {
                                                    <div class="flex items-center gap-2 mt-4 px-3 py-2 bg-red-50 dark:bg-red-950/60 border border-red-200 dark:border-red-800 rounded-md">
                                                        <IconXCircle class="w-4 h-4 text-red-500 dark:text-red-400 shrink-0" />
                                                        <span class="text-[13px] text-red-700 dark:text-red-400">{format!("Restore failed: {}", msg)}</span>
                                                    </div>
                                                }.into_any(),
                                                None => view! { <div></div> }.into_any(),
                                            }
                                        }}
                                    </div>
                                </div>
                            }.into_any()
                        } else if show_settings.get() {
                            view! {
                                <SettingsView set_show=set_show_settings />
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
                            // Get the tab title for query_name
                            let query_name = {
                                let tabs = tab_state.tabs.get();
                                tabs.iter()
                                    .find(|t| t.id == tab_id)
                                    .map(|t| t.title.clone())
                                    .unwrap_or_default()
                            };
                            view! {
                                <div class="h-full">
                                    <SqlTab query_name=query_name on_dirty_change=on_dirty />
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
