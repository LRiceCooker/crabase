use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::icons::IconTable;
use crate::tauri;

/// Sidebar list of database tables with search filter and right-click context menu.
#[component]
pub fn TablesList(
    tables: ReadSignal<Vec<String>>,
    active_table: Memo<Option<String>>,
    on_select: Callback<String>,
    /// Called after a table is dropped or truncated so the parent can refresh the table list.
    #[prop(optional)]
    on_tables_changed: Option<Callback<()>>,
) -> impl IntoView {
    // Context menu state
    let (ctx_menu, set_ctx_menu) = signal(Option::<(i32, i32, String)>::None);
    // Confirmation dialog state
    let (confirm, set_confirm) = signal(Option::<(String, String)>::None); // (action, table_name)
    let (action_error, set_action_error) = signal(Option::<String>::None);

    // Close context menu on any click
    {
        let closure = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |_: web_sys::MouseEvent| {
                if ctx_menu.get_untracked().is_some() {
                    set_ctx_menu.set(None);
                }
            },
        );
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    let do_action = move |action: String, table: String| {
        set_action_error.set(None);
        match action.as_str() {
            "drop" | "truncate" => {
                // Show confirmation dialog
                set_confirm.set(Some((action, table)));
            }
            "export_json" => {
                let t = table.clone();
                spawn_local(async move {
                    match tauri::export_table_json(&t).await {
                        Ok(json) => {
                            let _ = tauri::save_file_dialog(&format!("{t}.json"), &json).await;
                        }
                        Err(e) => set_action_error.set(Some(e)),
                    }
                });
            }
            "export_sql" => {
                let t = table.clone();
                spawn_local(async move {
                    match tauri::export_table_sql(&t).await {
                        Ok(sql) => {
                            let _ = tauri::save_file_dialog(&format!("{t}.sql"), &sql).await;
                        }
                        Err(e) => set_action_error.set(Some(e)),
                    }
                });
            }
            _ => {}
        }
    };

    let confirm_action = move || {
        if let Some((action, table)) = confirm.get_untracked() {
            set_confirm.set(None);
            let t = table.clone();
            spawn_local(async move {
                let result = match action.as_str() {
                    "drop" => tauri::drop_table(&t).await,
                    "truncate" => tauri::truncate_table(&t).await,
                    _ => Ok(String::new()),
                };
                match result {
                    Ok(_) => {
                        if let Some(cb) = on_tables_changed {
                            cb.run(());
                        }
                    }
                    Err(e) => set_action_error.set(Some(e)),
                }
            });
        }
    };

    view! {
        <div class="flex-1 overflow-y-auto p-3">
            <h2 class="text-[11px] font-medium text-gray-400 dark:text-zinc-500 uppercase tracking-wider mb-2">"Tables"</h2>
            <div class="flex flex-col gap-0.5">
                {move || {
                    let current_active = active_table.get();
                    tables.get().into_iter().map(|name| {
                        let is_active = current_active.as_ref() == Some(&name);
                        let item_class = if is_active {
                            "flex items-center gap-2 px-3 py-1 text-[13px] rounded-md cursor-pointer transition-colors duration-100 bg-indigo-50 dark:bg-indigo-500/25 text-indigo-600 dark:text-indigo-400"
                        } else {
                            "flex items-center gap-2 px-3 py-1 text-[13px] text-gray-700 dark:text-zinc-300 rounded-md hover:bg-gray-100 dark:hover:bg-zinc-800 cursor-pointer transition-colors duration-100"
                        };
                        let icon_class = if is_active {
                            "w-4 h-4 text-indigo-600 dark:text-indigo-400 shrink-0"
                        } else {
                            "w-4 h-4 text-gray-400 dark:text-zinc-500 shrink-0"
                        };
                        let click_name = name.clone();
                        let ctx_name = name.clone();
                        view! {
                            <div
                                class=item_class
                                on:click=move |_| on_select.run(click_name.clone())
                                on:contextmenu=move |ev: web_sys::MouseEvent| {
                                    ev.prevent_default();
                                    set_ctx_menu.set(Some((ev.client_x(), ev.client_y(), ctx_name.clone())));
                                }
                            >
                                <IconTable class=icon_class />
                                <span class="truncate">{name}</span>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>
            {move || if tables.with(|t| t.is_empty()) {
                Some(view! {
                    <p class="text-[13px] text-gray-400 dark:text-zinc-500 italic px-3">"No tables found"</p>
                })
            } else {
                None
            }}

            // Context menu
            {move || {
                ctx_menu.get().map(|(x, y, table_name)| {
                    let menu_items = vec![
                        ("Export as JSON", "export_json", false),
                        ("Export as SQL", "export_sql", false),
                        ("Truncate", "truncate", true),
                        ("Drop", "drop", true),
                    ];
                    view! {
                        <div
                            class="fixed z-50 bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-700 rounded-lg shadow-xl dark:shadow-black/40 py-1 w-[140px]"
                            style=format!("left: {}px; top: {}px;", x, y)
                        >
                            {menu_items.into_iter().map(|(label, action, danger)| {
                                let table = table_name.clone();
                                let action_str = action.to_string();
                                let class = if danger {
                                    "w-full text-left px-3 py-1.5 text-[13px] text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/60 cursor-pointer transition-colors duration-100"
                                } else {
                                    "w-full text-left px-3 py-1.5 text-[13px] text-gray-700 dark:text-zinc-300 hover:bg-gray-100 dark:hover:bg-zinc-800 cursor-pointer transition-colors duration-100"
                                };
                                view! {
                                    <button
                                        class=class
                                        on:click=move |ev: web_sys::MouseEvent| {
                                            ev.stop_propagation();
                                            set_ctx_menu.set(None);
                                            do_action(action_str.clone(), table.clone());
                                        }
                                    >
                                        {label}
                                    </button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }
                })
            }}

            // Confirmation dialog
            {move || {
                confirm.get().map(|(action, table_name)| {
                    let action_label = match action.as_str() {
                        "drop" => "DROP",
                        "truncate" => "TRUNCATE",
                        _ => "EXECUTE",
                    };
                    view! {
                        <div class="fixed inset-0 z-50 flex items-center justify-center">
                            <div class="absolute inset-0 bg-black/40 backdrop-blur-sm" on:click=move |_| set_confirm.set(None) />
                            <div class="relative bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-700 rounded-lg shadow-xl dark:shadow-black/40 p-4 max-w-sm w-full">
                                <h3 class="text-[13px] font-semibold text-gray-900 dark:text-neutral-50 mb-2">
                                    {format!("{action_label} table?")}
                                </h3>
                                <p class="text-[13px] text-gray-500 dark:text-zinc-400 mb-4">
                                    {format!("Are you sure you want to {action_label} \"{table_name}\"? This action cannot be undone.")}
                                </p>
                                <div class="flex justify-end gap-2">
                                    <button
                                        class="text-gray-500 dark:text-zinc-400 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 px-3 py-1.5 rounded-md text-[13px] transition-colors duration-100"
                                        on:click=move |_| set_confirm.set(None)
                                    >"Cancel"</button>
                                    <button
                                        class="bg-red-500 hover:bg-red-600 text-white text-[13px] font-medium px-3 py-1.5 rounded-md transition-colors duration-100"
                                        on:click=move |_| confirm_action()
                                    >{action_label}</button>
                                </div>
                            </div>
                        </div>
                    }
                })
            }}

            // Error toast
            {move || {
                action_error.get().map(|err| {
                    view! {
                        <div class="fixed bottom-4 right-4 z-50 bg-red-50 dark:bg-red-950/60 border border-red-200 dark:border-red-800 rounded-lg shadow-lg px-4 py-3 max-w-sm">
                            <p class="text-[13px] text-red-700 dark:text-red-400">{err}</p>
                            <button
                                class="text-[11px] text-red-500 underline mt-1"
                                on:click=move |_| set_action_error.set(None)
                            >"Dismiss"</button>
                        </div>
                    }
                })
            }}
        </div>
    }
}
