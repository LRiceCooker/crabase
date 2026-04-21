use leptos::prelude::*;

use crate::tauri;

/// Inline connection editing form shown in the header bar.
/// Collects host/port/user/password/dbname fields and emits a `ConnectionInfo` on submit.
#[component]
pub fn HeaderEditForm(
    /// Initial connection info to populate the fields.
    initial_info: tauri::ConnectionInfo,
    /// Whether a reconnection is in progress (disables buttons).
    reconnecting: ReadSignal<bool>,
    /// Called with the new `ConnectionInfo` when the user clicks "Reconnect".
    on_submit: Callback<tauri::ConnectionInfo>,
    /// Called when the user clicks "Cancel".
    on_cancel: Callback<()>,
) -> impl IntoView {
    let (edit_host, set_edit_host) = signal(initial_info.host);
    let (edit_port, set_edit_port) = signal(initial_info.port.to_string());
    let (edit_user, set_edit_user) = signal(initial_info.user);
    let (edit_dbname, set_edit_dbname) = signal(initial_info.dbname);
    let (edit_password, set_edit_password) = signal(initial_info.password);
    let (edit_schema, _set_edit_schema) = signal(initial_info.schema);
    let (edit_ssl, _set_edit_ssl) = signal(initial_info.sslmode == "require");

    let on_reconnect_click = move |_| {
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
        on_submit.run(info);
    };

    let on_cancel_click = move |_| {
        on_cancel.run(());
    };

    let input_class = "bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-800 rounded-md px-2 py-1 text-[13px] text-gray-900 dark:text-neutral-50 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 dark:focus:ring-indigo-500/60 focus:border-indigo-500 transition-colors duration-100";

    view! {
        <div class="flex items-center gap-1.5">
            <input
                type="text"
                placeholder="user"
                class=format!("{input_class} w-20")
                prop:value=move || edit_user.get()
                on:input=move |ev| set_edit_user.set(event_target_value(&ev))
            />
            <span class="text-gray-400 dark:text-zinc-500">"@"</span>
            <input
                type="text"
                placeholder="host"
                class=format!("{input_class} w-28")
                prop:value=move || edit_host.get()
                on:input=move |ev| set_edit_host.set(event_target_value(&ev))
            />
            <span class="text-gray-400 dark:text-zinc-500">":"</span>
            <input
                type="text"
                placeholder="port"
                class=format!("{input_class} w-14")
                prop:value=move || edit_port.get()
                on:input=move |ev| set_edit_port.set(event_target_value(&ev))
            />
            <span class="text-gray-400 dark:text-zinc-500">"/"</span>
            <input
                type="text"
                placeholder="dbname"
                class=format!("{input_class} w-20")
                prop:value=move || edit_dbname.get()
                on:input=move |ev| set_edit_dbname.set(event_target_value(&ev))
            />
            <input
                type="password"
                placeholder="password"
                class=format!("{input_class} w-20")
                prop:value=move || edit_password.get()
                on:input=move |ev| set_edit_password.set(event_target_value(&ev))
            />
            <button
                class="bg-indigo-500 hover:bg-indigo-600 dark:hover:bg-indigo-400 text-white text-[13px] font-medium px-2 py-1 rounded-md transition-colors duration-100 disabled:opacity-50"
                disabled=move || reconnecting.get()
                on:click=on_reconnect_click
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
                on:click=on_cancel_click
            >
                "Cancel"
            </button>
        </div>
    }
}
