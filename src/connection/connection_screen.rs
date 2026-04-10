use leptos::prelude::*;

use crate::connection::saved_connections::SavedConnections;
use crate::icons::{IconDatabase, IconAlertTriangle, IconLoader};
use crate::tauri::SavedConnection;

#[component]
pub fn ConnectionScreen(
    connection_string: ReadSignal<String>,
    set_connection_string: WriteSignal<String>,
    error_message: ReadSignal<Option<String>>,
    parsing: ReadSignal<bool>,
    on_parse: Callback<()>,
    on_select_saved: Callback<SavedConnection>,
) -> impl IntoView {
    view! {
        <main class="min-h-screen bg-gray-50 dark:bg-neutral-950 flex items-center justify-center p-4">
            <div class="bg-white dark:bg-zinc-900 rounded-lg shadow-xl dark:shadow-black/40 border border-gray-200 dark:border-zinc-800 w-full max-w-md dark:ring-1 dark:ring-white/[0.06]">
                <div class="px-6 py-8">
                    <div class="flex flex-col items-center gap-1 mb-6">
                        <div class="flex items-center gap-2">
                            <IconDatabase class="w-5 h-5 text-indigo-500 dark:text-indigo-400" />
                            <h1 class="text-base font-semibold text-gray-900 dark:text-neutral-50">"crabase"</h1>
                        </div>
                        <p class="text-[13px] text-gray-500 dark:text-zinc-400">"PostgreSQL Desktop Client"</p>
                    </div>

                    <SavedConnections on_select=on_select_saved />

                    <div class="flex flex-col gap-1.5">
                        <label class="text-[13px] font-normal text-gray-700 dark:text-zinc-300">"Connection string"</label>
                        <input
                            type="text"
                            placeholder="postgresql://user:password@host:port/dbname"
                            class="bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-800 rounded-md px-3 py-1.5 text-[13px] text-gray-900 dark:text-neutral-50 w-full focus:outline-none focus:ring-2 focus:ring-indigo-500/20 dark:focus:ring-indigo-500/60 focus:border-indigo-500 transition-colors duration-100 placeholder-gray-400 dark:placeholder-zinc-500"
                            prop:value=move || connection_string.get()
                            on:input=move |ev| {
                                set_connection_string.set(event_target_value(&ev));
                            }
                        />
                    </div>

                    {move || error_message.get().map(|msg| view! {
                        <div class="flex items-center gap-2 mt-4 px-3 py-2 bg-red-50 dark:bg-red-950/60 border border-red-200 dark:border-red-800 rounded-md">
                            <IconAlertTriangle class="w-4 h-4 text-red-500 dark:text-red-400 shrink-0" />
                            <span class="text-[13px] text-red-700 dark:text-red-400">{msg}</span>
                        </div>
                    })}

                    <div class="mt-4">
                        <button
                            class="bg-indigo-500 hover:bg-indigo-600 dark:hover:bg-indigo-400 text-white text-[13px] font-medium px-3 py-1.5 rounded-md w-full transition-colors duration-100 disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=move || connection_string.with(|s| s.is_empty()) || parsing.get()
                            on:click=move |_| on_parse.run(())
                        >
                            {move || if parsing.get() {
                                view! {
                                    <span class="flex items-center justify-center gap-2">
                                        <IconLoader class="w-4 h-4 animate-spin" />
                                        "Parsing..."
                                    </span>
                                }.into_any()
                            } else {
                                view! { <span>"Next"</span> }.into_any()
                            }}
                        </button>
                    </div>
                </div>
            </div>
        </main>
    }
}
