use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::command_palette::fuzzy_score;
use crate::icons::{IconSearch, IconTable};

#[component]
pub fn TableFinder(
    show: ReadSignal<bool>,
    set_show: WriteSignal<bool>,
    tables: ReadSignal<Vec<String>>,
    on_select: Callback<String>,
) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (selected_idx, set_selected_idx) = signal(0usize);
    let input_ref = NodeRef::<leptos::html::Input>::new();

    // Focus input when palette opens, clear query when it closes
    Effect::new(move |_| {
        if show.get() {
            if let Some(el) = input_ref.get() {
                let _ = el.focus();
            }
        } else {
            set_query.set(String::new());
            set_selected_idx.set(0);
        }
    });

    // Reset selection when query changes
    Effect::new(move |_| {
        let _ = query.get();
        set_selected_idx.set(0);
    });

    move || {
        if show.get() {
            let q = query.get();
            let all_tables = tables.get();
            let mut scored: Vec<_> = all_tables
                .iter()
                .filter_map(|name| {
                    if q.is_empty() {
                        Some((name.clone(), 0))
                    } else {
                        fuzzy_score(&q, name).map(|s| (name.clone(), s))
                    }
                })
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            let filtered: Vec<String> = scored.into_iter().map(|(name, _)| name).collect();
            let count = filtered.len();

            // Clone for Enter key handler
            let filtered_for_enter = filtered.clone();

            Some(view! {
                <div class="fixed inset-0 z-50 flex justify-center items-start">
                    // Backdrop
                    <div
                        class="absolute inset-0 backdrop-blur-sm bg-black/30"
                        on:click=move |_| set_show.set(false)
                    ></div>
                    // Panel
                    <div class="relative z-10 w-[560px] max-h-[400px] bg-white dark:bg-zinc-900 rounded-xl shadow-2xl dark:shadow-black/40 border border-gray-200 dark:border-white/[0.08] overflow-hidden mt-[20vh] dark:ring-1 dark:ring-white/[0.06]">
                        // Search input
                        <div class="flex items-center gap-2 px-4 py-3 border-b border-gray-100 dark:border-[#1F1F23]">
                            <IconSearch class="w-4 h-4 text-gray-400 dark:text-zinc-500 shrink-0" />
                            <input
                                type="text"
                                node_ref=input_ref
                                placeholder="Search tables..."
                                class="text-base w-full focus:outline-none bg-transparent text-gray-900 dark:text-neutral-50 placeholder-gray-400 dark:placeholder-zinc-500"
                                prop:value=move || query.get()
                                on:input=move |ev| set_query.set(event_target_value(&ev))
                                on:keydown={
                                    let filtered = filtered_for_enter.clone();
                                    move |ev| {
                                        let ev: &web_sys::KeyboardEvent = ev.unchecked_ref();
                                        match ev.key().as_str() {
                                            "Escape" => set_show.set(false),
                                            "Enter" => {
                                                let idx = selected_idx.get();
                                                if let Some(name) = filtered.get(idx) {
                                                    on_select.run(name.clone());
                                                    set_show.set(false);
                                                }
                                            }
                                            "ArrowDown" => {
                                                ev.prevent_default();
                                                let idx = selected_idx.get();
                                                if idx + 1 < count {
                                                    set_selected_idx.set(idx + 1);
                                                }
                                            }
                                            "ArrowUp" => {
                                                ev.prevent_default();
                                                let idx = selected_idx.get();
                                                if idx > 0 {
                                                    set_selected_idx.set(idx - 1);
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            />
                        </div>
                        // Table list
                        <div class="pb-2 max-h-64 overflow-y-auto">
                            {filtered.into_iter().enumerate().map(|(idx, name)| {
                                let table_name = name.clone();
                                let is_selected = selected_idx.get() == idx;
                                let class = if is_selected {
                                    "px-4 py-2 flex items-center gap-3 text-[13px] cursor-pointer bg-indigo-50 dark:bg-indigo-500/25 text-indigo-600 dark:text-indigo-400"
                                } else {
                                    "px-4 py-2 flex items-center gap-3 text-[13px] cursor-pointer hover:bg-indigo-50 dark:hover:bg-indigo-500/25 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors duration-100"
                                };
                                view! {
                                    <div
                                        class=class
                                        on:click=move |_| {
                                            on_select.run(table_name.clone());
                                            set_show.set(false);
                                        }
                                    >
                                        <IconTable class="w-4 h-4 text-gray-400 dark:text-zinc-500 shrink-0" />
                                        <span class="font-medium text-gray-900 dark:text-neutral-50">{name}</span>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                </div>
            })
        } else {
            None
        }
    }
}
