use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::command_palette::CommandPalette;
use crate::tauri;

#[component]
pub fn MainScreen() -> impl IntoView {
    let (connection_info, set_connection_info) =
        signal(Option::<tauri::ConnectionInfo>::None);
    let (tables, set_tables) = signal(Vec::<String>::new());

    // Header editing state
    let (editing, set_editing) = signal(false);
    let (edit_host, set_edit_host) = signal(String::new());
    let (edit_port, set_edit_port) = signal(String::new());
    let (edit_user, set_edit_user) = signal(String::new());
    let (edit_dbname, set_edit_dbname) = signal(String::new());
    let (edit_password, set_edit_password) = signal(String::new());
    let (reconnecting, set_reconnecting) = signal(false);
    let (header_error, set_header_error) = signal(Option::<String>::None);

    // Command palette state
    let (show_palette, set_show_palette) = signal(false);

    // Restore panel state
    let (show_restore, set_show_restore) = signal(false);
    let (restore_file, set_restore_file) = signal(Option::<String>::None);
    let (restore_picking, set_restore_picking) = signal(false);
    let (restore_running, set_restore_running) = signal(false);
    let (restore_logs, set_restore_logs) = signal(Vec::<String>::new());
    let (restore_status, set_restore_status) = signal(Option::<Result<String, String>>::None);

    // Command palette action handler
    let on_command = Callback::new(move |cmd: String| {
        match cmd.as_str() {
            "Restore Backup" => set_show_restore.set(true),
            _ => {}
        }
    });

    // Global keyboard shortcut: Cmd+Shift+P (macOS) / Ctrl+Shift+P
    {
        let closure = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
            move |ev: web_sys::KeyboardEvent| {
                if (ev.meta_key() || ev.ctrl_key()) && ev.shift_key() && ev.code() == "KeyP" {
                    ev.prevent_default();
                    set_show_palette.set(true);
                }
            },
        );
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Fetch connection info on mount
    spawn_local({
        let set_connection_info = set_connection_info.clone();
        async move {
            if let Ok(info) = tauri::get_connection_info().await {
                set_connection_info.set(Some(info));
            }
        }
    });

    // Fetch tables on mount
    spawn_local({
        let set_tables = set_tables.clone();
        async move {
            if let Ok(t) = tauri::list_tables().await {
                set_tables.set(t);
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
            set_edit_password.set(String::new());
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
        let host = edit_host.get();
        let port = edit_port.get();
        let user = edit_user.get();
        let dbname = edit_dbname.get();
        let password = edit_password.get();

        // Build connection string
        let connection_string = if password.is_empty() {
            format!("postgresql://{}@{}:{}/{}", user, host, port, dbname)
        } else {
            format!("postgresql://{}:{}@{}:{}/{}", user, password, host, port, dbname)
        };

        set_reconnecting.set(true);
        set_header_error.set(None);

        spawn_local(async move {
            // Disconnect first (ignore errors — may already be disconnected)
            let _ = tauri::disconnect_db().await;

            match tauri::connect_db(&connection_string).await {
                Ok(_) => {
                    // Refresh connection info and tables
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

    view! {
        <div class="min-h-screen flex flex-col bg-base-200">
            // Command palette overlay
            <CommandPalette show=show_palette set_show=set_show_palette on_command=on_command />

            // Header
            <header class="navbar bg-base-100 shadow-md px-4">
                <div class="flex-1">
                    <span class="text-lg font-bold">"crabase"</span>
                </div>
                <div class="flex-none gap-2 text-sm">
                    {move || {
                        if editing.get() {
                            // Edit mode: input fields
                            view! {
                                <div class="flex items-center gap-2">
                                    <input
                                        type="text"
                                        placeholder="user"
                                        class="input input-bordered input-xs w-24"
                                        prop:value=move || edit_user.get()
                                        on:input=move |ev| set_edit_user.set(event_target_value(&ev))
                                    />
                                    <span class="text-base-content/50">"@"</span>
                                    <input
                                        type="text"
                                        placeholder="host"
                                        class="input input-bordered input-xs w-32"
                                        prop:value=move || edit_host.get()
                                        on:input=move |ev| set_edit_host.set(event_target_value(&ev))
                                    />
                                    <span class="text-base-content/50">":"</span>
                                    <input
                                        type="text"
                                        placeholder="port"
                                        class="input input-bordered input-xs w-16"
                                        prop:value=move || edit_port.get()
                                        on:input=move |ev| set_edit_port.set(event_target_value(&ev))
                                    />
                                    <span class="text-base-content/50">"/"</span>
                                    <input
                                        type="text"
                                        placeholder="dbname"
                                        class="input input-bordered input-xs w-24"
                                        prop:value=move || edit_dbname.get()
                                        on:input=move |ev| set_edit_dbname.set(event_target_value(&ev))
                                    />
                                    <input
                                        type="password"
                                        placeholder="password"
                                        class="input input-bordered input-xs w-24"
                                        prop:value=move || edit_password.get()
                                        on:input=move |ev| set_edit_password.set(event_target_value(&ev))
                                    />
                                    <button
                                        class="btn btn-primary btn-xs"
                                        disabled=move || reconnecting.get()
                                        on:click=on_reconnect
                                    >
                                        {move || if reconnecting.get() {
                                            "Connexion..."
                                        } else {
                                            "Reconnecter"
                                        }}
                                    </button>
                                    <button
                                        class="btn btn-ghost btn-xs"
                                        disabled=move || reconnecting.get()
                                        on:click=on_cancel
                                    >
                                        "Annuler"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            // Read-only mode: badges + edit button
                            view! {
                                <div class="flex items-center gap-3">
                                    {move || connection_info.get().map(|info| view! {
                                        <div class="badge badge-outline">{format!("{}@{}", info.user, info.host)}</div>
                                        <div class="badge badge-outline">{format!(":{}", info.port)}</div>
                                        <div class="badge badge-primary">{info.dbname.clone()}</div>
                                    })}
                                    <button
                                        class="btn btn-ghost btn-xs"
                                        on:click=on_edit
                                    >
                                        "Modifier"
                                    </button>
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </header>

            // Header error message
            {move || header_error.get().map(|msg| view! {
                <div class="alert alert-error shadow-sm mx-4 mt-2">
                    <span>{msg}</span>
                </div>
            })}

            // Body: central area + right panel
            <div class="flex flex-1 overflow-hidden">
                // Central area
                <main class="flex-1 p-4">
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
                                <div class="card bg-base-100 shadow-lg max-w-lg mx-auto mt-8">
                                    <div class="card-body">
                                        <div class="flex items-center justify-between">
                                            <h2 class="card-title">"Restore Backup"</h2>
                                            <button
                                                class="btn btn-ghost btn-sm"
                                                disabled=move || restore_running.get()
                                                on:click=on_close
                                            >
                                                "✕"
                                            </button>
                                        </div>
                                        <p class="text-base-content/60">"Restore a .tar.gz PostgreSQL backup to the connected database."</p>

                                        // File selector
                                        <div class="form-control mt-2">
                                            <label class="label">
                                                <span class="label-text">"Backup file (.tar.gz)"</span>
                                            </label>
                                            <div class="flex items-center gap-2">
                                                <button
                                                    class="btn btn-outline btn-sm"
                                                    disabled=move || restore_picking.get() || restore_running.get()
                                                    on:click=on_pick_file
                                                >
                                                    {move || if restore_picking.get() {
                                                        "Selecting..."
                                                    } else {
                                                        "Choose file..."
                                                    }}
                                                </button>
                                                <span class="text-sm text-base-content/70 truncate max-w-xs">
                                                    {move || restore_file.get().unwrap_or_else(|| "No file selected".to_string())}
                                                </span>
                                            </div>
                                        </div>

                                        // Restore button
                                        <div class="card-actions justify-end mt-4">
                                            <button
                                                class="btn btn-primary"
                                                disabled=move || restore_file.get().is_none() || restore_running.get()
                                                on:click=on_restore
                                            >
                                                {move || if restore_running.get() {
                                                    "Restoring..."
                                                } else {
                                                    "Lancer le restore"
                                                }}
                                            </button>
                                        </div>

                                        // Real-time log display
                                        {move || {
                                            let logs = restore_logs.get();
                                            if !logs.is_empty() {
                                                Some(view! {
                                                    <div class="mt-4">
                                                        <label class="label">
                                                            <span class="label-text font-semibold">"Logs"</span>
                                                        </label>
                                                        <div class="bg-base-300 rounded-lg p-3 max-h-60 overflow-y-auto font-mono text-xs">
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
                                                    <div class="alert alert-success mt-4">
                                                        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                        </svg>
                                                        <span>"Restore completed successfully."</span>
                                                    </div>
                                                }.into_any(),
                                                Some(Err(ref msg)) => view! {
                                                    <div class="alert alert-error mt-4">
                                                        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                        </svg>
                                                        <span>{format!("Restore failed: {}", msg)}</span>
                                                    </div>
                                                }.into_any(),
                                                None => view! { <div></div> }.into_any(),
                                            }
                                        }}
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="flex items-center justify-center h-full text-base-content/30">
                                    <p class="text-lg">"Select a table to get started"</p>
                                </div>
                            }.into_any()
                        }
                    }}
                </main>

                // Right panel: tables list
                <aside class="w-64 bg-base-100 border-l border-base-300 overflow-y-auto">
                    <div class="p-3">
                        <h2 class="text-sm font-semibold text-base-content/50 uppercase tracking-wider mb-2">"Tables"</h2>
                        <ul class="menu menu-sm">
                            {move || tables.get().into_iter().map(|name| view! {
                                <li><a>{name}</a></li>
                            }).collect::<Vec<_>>()}
                        </ul>
                        {move || if tables.get().is_empty() {
                            Some(view! {
                                <p class="text-sm text-base-content/40 italic">"No tables found"</p>
                            })
                        } else {
                            None
                        }}
                    </div>
                </aside>
            </div>
        </div>
    }
}
