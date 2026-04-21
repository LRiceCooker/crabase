use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::header_edit_form::HeaderEditForm;
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
    let (editing, set_editing) = signal(false);
    let (reconnecting, set_reconnecting) = signal(false);
    let (header_error, set_header_error) = signal(Option::<String>::None);

    // Enter edit mode
    let on_edit = move |_| {
        if connection_info.get().is_some() {
            set_header_error.set(None);
            set_editing.set(true);
        }
    };

    // Handle reconnect submission from the edit form
    let on_form_submit = Callback::new(move |info: tauri::ConnectionInfo| {
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
    });

    let on_form_cancel = Callback::new(move |_: ()| {
        set_editing.set(false);
        set_header_error.set(None);
    });

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
                        // Edit mode: show the edit form
                        let info = connection_info.get().unwrap_or(tauri::ConnectionInfo {
                            host: String::new(),
                            port: 5432,
                            user: String::new(),
                            password: String::new(),
                            dbname: String::new(),
                            schema: String::new(),
                            sslmode: "disable".to_string(),
                        });
                        view! {
                            <HeaderEditForm
                                initial_info=info
                                reconnecting=reconnecting
                                on_submit=on_form_submit
                                on_cancel=on_form_cancel
                            />
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
