use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::tauri;

#[component]
pub fn MainScreen() -> impl IntoView {
    let (connection_info, set_connection_info) =
        signal(Option::<tauri::ConnectionInfo>::None);
    let (tables, set_tables) = signal(Vec::<String>::new());

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

    view! {
        <div class="min-h-screen flex flex-col bg-base-200">
            // Header
            <header class="navbar bg-base-100 shadow-md px-4">
                <div class="flex-1">
                    <span class="text-lg font-bold">"crabase"</span>
                </div>
                <div class="flex-none gap-4 text-sm">
                    {move || connection_info.get().map(|info| view! {
                        <div class="flex items-center gap-3">
                            <div class="badge badge-outline">{format!("{}@{}", info.user, info.host)}</div>
                            <div class="badge badge-outline">{format!(":{}", info.port)}</div>
                            <div class="badge badge-primary">{info.dbname.clone()}</div>
                        </div>
                    })}
                </div>
            </header>

            // Body: central area + right panel
            <div class="flex flex-1 overflow-hidden">
                // Central area (empty for now)
                <main class="flex-1 p-4">
                    <div class="flex items-center justify-center h-full text-base-content/30">
                        <p class="text-lg">"Select a table to get started"</p>
                    </div>
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
