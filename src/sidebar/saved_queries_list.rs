use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::icons::IconTerminal;
use crate::tauri;

/// Sidebar section listing saved SQL queries with right-click context menu for rename/delete.
#[component]
pub fn SavedQueriesList(
    queries: ReadSignal<Vec<String>>,
    on_select: Callback<String>,
    /// Called after a query is deleted/duplicated/renamed so the parent can refresh the list.
    #[prop(optional)]
    on_queries_changed: Option<Callback<()>>,
) -> impl IntoView {
    // Context menu state: (x, y, query_name)
    let (ctx_menu, set_ctx_menu) = signal(Option::<(i32, i32, String)>::None);
    // Rename state: (original_name)
    let (renaming, set_renaming) = signal(Option::<String>::None);
    let (rename_value, set_rename_value) = signal(String::new());
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
        closure.forget(); // OK since this component lives for the app lifetime
    }

    let refresh = move || {
        if let Some(cb) = on_queries_changed {
            cb.run(());
        }
    };

    let do_delete = move |name: String| {
        set_action_error.set(None);
        spawn_local(async move {
            match tauri::delete_query(&name).await {
                Ok(()) => refresh(),
                Err(e) => set_action_error.set(Some(e)),
            }
        });
    };

    let do_duplicate = move |name: String| {
        set_action_error.set(None);
        spawn_local(async move {
            // Load the original content
            match tauri::load_query(&name).await {
                Ok(saved) => {
                    let new_name = format!("{name} (copy)");
                    match tauri::save_query(&new_name, &saved.sql).await {
                        Ok(()) => refresh(),
                        Err(e) => set_action_error.set(Some(e)),
                    }
                }
                Err(e) => set_action_error.set(Some(e)),
            }
        });
    };

    let start_rename = move |name: String| {
        set_rename_value.set(name.clone());
        set_renaming.set(Some(name));
    };

    let commit_rename = move || {
        if let Some(old_name) = renaming.get_untracked() {
            let new_name = rename_value.get_untracked().trim().to_string();
            set_renaming.set(None);
            if new_name.is_empty() || new_name == old_name {
                return;
            }
            set_action_error.set(None);
            spawn_local(async move {
                match tauri::rename_query(&old_name, &new_name).await {
                    Ok(()) => refresh(),
                    Err(e) => set_action_error.set(Some(e)),
                }
            });
        }
    };

    let rename_ref = NodeRef::<leptos::html::Input>::new();

    // Auto-focus the rename input when it appears
    Effect::new(move |_| {
        if renaming.get().is_some() {
            let input = rename_ref;
            let cb = wasm_bindgen::closure::Closure::once(move || {
                if let Some(el) = input.get() {
                    let _ = el.focus();
                    el.select();
                }
            });
            let _ = web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                20,
            );
            cb.forget();
        }
    });

    view! {
        {move || {
            let items = queries.get();
            if items.is_empty() {
                return None;
            }
            Some(view! {
                <div class="shrink-0 max-h-[20%] overflow-y-auto border-b border-gray-200 dark:border-zinc-800">
                    <div class="p-3 pb-2">
                    <h2 class="text-[11px] font-medium text-gray-400 dark:text-zinc-500 uppercase tracking-wider mb-2">"Saved Queries"</h2>
                    <div class="flex flex-col gap-0.5">
                        {items.into_iter().map(|name| {
                            let click_name = name.clone();
                            let ctx_name = name.clone();
                            let is_renaming = renaming.get() == Some(name.clone());

                            if is_renaming {
                                view! {
                                    <div class="flex items-center gap-2 px-3 py-1">
                                        <IconTerminal class="w-4 h-4 text-gray-400 dark:text-zinc-500 shrink-0" />
                                        <input
                                            type="text"
                                            node_ref=rename_ref
                                            class="flex-1 bg-white dark:bg-zinc-900 border border-indigo-500 rounded px-1 py-0 text-[13px] text-gray-900 dark:text-neutral-50 outline-none"
                                            prop:value=move || rename_value.get()
                                            on:input=move |ev| set_rename_value.set(event_target_value(&ev))
                                            on:blur=move |_| commit_rename()
                                            on:keydown=move |ev: web_sys::KeyboardEvent| {
                                                match ev.key().as_str() {
                                                    "Enter" => commit_rename(),
                                                    "Escape" => set_renaming.set(None),
                                                    _ => {}
                                                }
                                            }
                                        />
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div
                                        class="flex items-center gap-2 px-3 py-1 text-[13px] text-gray-700 dark:text-zinc-300 rounded-md hover:bg-gray-100 dark:hover:bg-zinc-800 cursor-pointer transition-colors duration-100"
                                        on:click=move |_| on_select.run(click_name.clone())
                                        on:contextmenu=move |ev: web_sys::MouseEvent| {
                                            ev.prevent_default();
                                            set_ctx_menu.set(Some((ev.client_x(), ev.client_y(), ctx_name.clone())));
                                        }
                                    >
                                        <IconTerminal class="w-4 h-4 text-gray-400 dark:text-zinc-500 shrink-0" />
                                        <span class="truncate">{name}</span>
                                    </div>
                                }.into_any()
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    </div>
                </div>
            })
        }}

        // Context menu
        {move || {
            ctx_menu.get().map(|(x, y, query_name)| {
                let dup_name = query_name.clone();
                let del_name = query_name.clone();
                let ren_name = query_name.clone();
                view! {
                    <div
                        class="fixed z-50 bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-700 rounded-lg shadow-xl dark:shadow-black/40 py-1 w-[140px]"
                        style=format!("left: {}px; top: {}px;", x, y)
                    >
                        <button
                            class="w-full text-left px-3 py-1.5 text-[13px] text-gray-700 dark:text-zinc-300 hover:bg-gray-100 dark:hover:bg-zinc-800 cursor-pointer transition-colors duration-100"
                            on:click=move |ev: web_sys::MouseEvent| {
                                ev.stop_propagation();
                                set_ctx_menu.set(None);
                                start_rename(ren_name.clone());
                            }
                        >"Rename"</button>
                        <button
                            class="w-full text-left px-3 py-1.5 text-[13px] text-gray-700 dark:text-zinc-300 hover:bg-gray-100 dark:hover:bg-zinc-800 cursor-pointer transition-colors duration-100"
                            on:click=move |ev: web_sys::MouseEvent| {
                                ev.stop_propagation();
                                set_ctx_menu.set(None);
                                do_duplicate(dup_name.clone());
                            }
                        >"Duplicate"</button>
                        <button
                            class="w-full text-left px-3 py-1.5 text-[13px] text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/60 cursor-pointer transition-colors duration-100"
                            on:click=move |ev: web_sys::MouseEvent| {
                                ev.stop_propagation();
                                set_ctx_menu.set(None);
                                do_delete(del_name.clone());
                            }
                        >"Delete"</button>
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
    }
}
