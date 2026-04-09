use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::icons::IconTrash2;
use crate::tauri::{self, SavedConnection};

#[component]
pub fn SavedConnections(
    on_select: Callback<SavedConnection>,
) -> impl IntoView {
    let (connections, set_connections) = signal(Vec::<SavedConnection>::new());
    let (loaded, set_loaded) = signal(false);

    // Load saved connections on mount
    Effect::new(move || {
        spawn_local(async move {
            if let Ok(conns) = tauri::list_saved_connections().await {
                set_connections.set(conns);
            }
            set_loaded.set(true);
        });
    });

    let delete_connection = Callback::new(move |name: String| {
        spawn_local(async move {
            if tauri::delete_saved_connection(&name).await.is_ok() {
                // Refresh the list
                if let Ok(conns) = tauri::list_saved_connections().await {
                    set_connections.set(conns);
                }
            }
        });
    });

    move || {
        let conns = connections.get();
        if !loaded.get() || conns.is_empty() {
            return view! { <div class="hidden"></div> }.into_any();
        }

        let items = conns.into_iter().map(|conn| {
            let conn_for_click = conn.clone();
            let name_for_delete = conn.name.clone();
            let display_name = conn.name.clone();
            let host = conn.info.host.clone();
            let port = conn.info.port;
            let dbname = conn.info.dbname.clone();

            view! {
                <button
                    type="button"
                    class="group border border-gray-200 rounded-lg px-4 py-3 hover:bg-gray-50 cursor-pointer flex items-center justify-between w-full text-left transition-colors duration-100"
                    on:click={
                        let conn_for_click = conn_for_click.clone();
                        move |_| on_select.run(conn_for_click.clone())
                    }
                >
                    <div class="flex flex-col gap-0.5 min-w-0">
                        <span class="text-[13px] font-medium text-gray-900 truncate">{display_name}</span>
                        <span class="text-[11px] text-gray-400 truncate">
                            {format!("{}:{} / {}", host, port, dbname)}
                        </span>
                    </div>
                    <button
                        type="button"
                        class="opacity-0 group-hover:opacity-100 text-gray-400 hover:text-red-500 p-1 rounded-md hover:bg-red-50 transition-all duration-100 shrink-0 ml-2"
                        on:click={
                            let name_for_delete = name_for_delete.clone();
                            move |ev: web_sys::MouseEvent| {
                                ev.stop_propagation();
                                delete_connection.run(name_for_delete.clone());
                            }
                        }
                    >
                        <IconTrash2 class="w-3.5 h-3.5" />
                    </button>
                </button>
            }
        }).collect::<Vec<_>>();

        view! {
            <div class="flex flex-col gap-2 mb-4">
                <span class="text-[11px] font-medium text-gray-400 uppercase tracking-wider">"Saved Connections"</span>
                <div class="flex flex-col gap-1.5">
                    {items}
                </div>
            </div>
        }.into_any()
    }
}
