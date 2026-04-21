use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::icons::{
    IconCheckCircle, IconFile, IconLoader, IconUpload, IconX, IconXCircle,
};
use crate::tauri;

/// Panel for restoring a PostgreSQL `.tar.gz` backup to the connected database.
/// Handles file selection, restore execution with real-time logs, and status display.
#[component]
pub fn RestorePanel(
    /// Called when the user closes the panel (X button or after completion).
    on_close: Callback<()>,
) -> impl IntoView {
    let (restore_file, set_restore_file) = signal(Option::<String>::None);
    let (restore_picking, set_restore_picking) = signal(false);
    let (restore_running, set_restore_running) = signal(false);
    let (restore_logs, set_restore_logs) = signal(Vec::<String>::new());
    let (restore_status, set_restore_status) = signal(Option::<Result<String, String>>::None);

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

    let on_close_click = move |_| {
        on_close.run(());
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
                })
                .await;

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
                        set_restore_logs.update(|logs| logs.push(format!("ERROR: {msg}")));
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
                    on:click=on_close_click
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
                            <IconUpload class="w-4 h-4 text-gray-400 dark:text-zinc-500" />
                            {move || if restore_picking.get() {
                                "Selecting..."
                            } else {
                                "Choose file..."
                            }}
                        </button>
                        <span class="text-[13px] text-gray-500 dark:text-zinc-400 truncate max-w-xs flex items-center gap-1.5">
                            {move || restore_file.get().map(|f| view! {
                                <IconFile class="w-4 h-4 text-gray-400 dark:text-zinc-500 shrink-0" />
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
                                <span class="text-[13px] text-red-700 dark:text-red-400">{format!("Restore failed: {msg}")}</span>
                            </div>
                        }.into_any(),
                        None => view! { <div></div> }.into_any(),
                    }
                }}
            </div>
        </div>
    }
}
