use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::icons::{
    IconAlertTriangle, IconDatabase, IconEdit, IconLogOut, IconPlus,
};
use crate::tauri;

/// Top header bar showing the app title, connection badges, schema selector,
/// edit/reconnect form, and disconnect button.
#[component]
pub fn HeaderBar(
    /// Current connection info (read-only from parent).
    connection_info: ReadSignal<Option<tauri::ConnectionInfo>>,
    /// Writer for updating connection info after reconnect or schema change.
    set_connection_info: WriteSignal<Option<tauri::ConnectionInfo>>,
    /// Available schemas for the schema dropdown.
    available_schemas: ReadSignal<Vec<String>>,
    /// Writer for refreshing the table list after reconnect or schema change.
    set_tables: WriteSignal<Vec<String>>,
    /// Called when the user clicks the disconnect button.
    on_disconnect: Callback<()>,
    /// Called when the user clicks the "+" button to open a new SQL editor.
    on_new_sql_editor: Callback<()>,
) -> impl IntoView {
    // Header editing state (internal)
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
            sslmode: if edit_ssl.get() {
                "require".to_string()
            } else {
                "disable".to_string()
            },
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
        <header class="h-10 flex items-center justify-between px-4 border-b border-gray-200 dark:border-zinc-800 bg-white dark:bg-neutral-950 shrink-0">
            <div class="flex items-center gap-2">
                <IconDatabase class="w-4 h-4 text-indigo-500 dark:text-indigo-400" />
                <span class="text-base font-semibold text-gray-900 dark:text-neutral-50">"crabase"</span>
                <button
                    class="text-gray-400 dark:text-zinc-500 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 p-1 rounded-md transition-colors duration-100"
                    title="New SQL Editor"
                    on:click=move |_| on_new_sql_editor.run(())
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
                                                            if tauri::connect_db(&new_info).await.is_ok() {
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
    }
}
