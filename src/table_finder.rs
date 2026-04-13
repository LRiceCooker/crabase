use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::command_palette::fuzzy_score;
use crate::icons::{IconSearch, IconTable, IconTerminal};
use crate::overlay::{self, ActiveOverlay};

#[derive(Clone, Debug)]
enum FinderItemKind {
    Table,
    Query,
}

#[derive(Clone, Debug)]
struct FinderItem {
    name: String,
    kind: FinderItemKind,
}

#[component]
pub fn TableFinder(
    tables: ReadSignal<Vec<String>>,
    #[prop(optional)] saved_queries: Option<ReadSignal<Vec<String>>>,
    on_select: Callback<String>,
    #[prop(optional)] on_query_select: Option<Callback<String>>,
) -> impl IntoView {
    let overlay_ctx = overlay::use_overlay();
    let (query, set_query) = signal(String::new());
    let (selected_idx, set_selected_idx) = signal(0usize);
    let input_ref = NodeRef::<leptos::html::Input>::new();

    // Focus input when finder opens, clear query when it closes
    Effect::new(move |_| {
        if overlay_ctx.is_open(ActiveOverlay::TableFinder) {
            // Delay focus to next frame so CodeMirror releases focus first
            let input = input_ref;
            let cb = wasm_bindgen::closure::Closure::once(move || {
                if let Some(el) = input.get() {
                    let _ = el.focus();
                }
            });
            let _ = web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                20,
            );
            cb.forget();
        } else {
            // Defer signal writes to avoid re-entrant borrow panics
            let cb = wasm_bindgen::closure::Closure::once(move || {
                set_query.set(String::new());
                set_selected_idx.set(0);
            });
            let _ = web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                0,
            );
            cb.forget();
        }
    });

    // Reset selection when query changes
    Effect::new(move |_| {
        let _ = query.get();
        set_selected_idx.set(0);
    });

    move || {
        if overlay_ctx.is_open(ActiveOverlay::TableFinder) {
            let q = query.get();
            let all_tables = tables.get();
            let all_queries = saved_queries.map(|s| s.get()).unwrap_or_default();

            // Score and filter tables
            let mut table_items: Vec<(FinderItem, i32)> = all_tables
                .iter()
                .filter_map(|name| {
                    let score = if q.is_empty() { Some(0) } else { fuzzy_score(&q, name) };
                    score.map(|s| (FinderItem { name: name.clone(), kind: FinderItemKind::Table }, s))
                })
                .collect();
            table_items.sort_by(|a, b| b.1.cmp(&a.1));

            // Score and filter queries
            let mut query_items: Vec<(FinderItem, i32)> = all_queries
                .iter()
                .filter_map(|name| {
                    let score = if q.is_empty() { Some(0) } else { fuzzy_score(&q, name) };
                    score.map(|s| (FinderItem { name: name.clone(), kind: FinderItemKind::Query }, s))
                })
                .collect();
            query_items.sort_by(|a, b| b.1.cmp(&a.1));

            // Build grouped results: queries first, then tables
            let mut items: Vec<FinderItem> = Vec::new();
            let has_queries = !query_items.is_empty();
            let has_tables = !table_items.is_empty();

            for (item, _) in query_items {
                items.push(item);
            }
            for (item, _) in table_items {
                items.push(item);
            }

            let count = items.len();
            let items_for_enter = items.clone();

            Some(view! {
                <div class="fixed inset-0 z-50 flex justify-center items-start">
                    // Backdrop
                    <div
                        class="absolute inset-0 backdrop-blur-sm bg-black/30"
                        on:click=move |_| overlay_ctx.close()
                    ></div>
                    // Panel
                    <div class="relative z-10 w-[560px] max-h-[400px] bg-white dark:bg-zinc-900 rounded-xl shadow-2xl dark:shadow-black/40 border border-gray-200 dark:border-white/[0.08] overflow-hidden mt-[20vh] dark:ring-1 dark:ring-white/[0.06]">
                        // Search input
                        <div class="flex items-center gap-2 px-4 py-3 border-b border-gray-100 dark:border-[#1F1F23]">
                            <IconSearch class="w-4 h-4 text-gray-400 dark:text-zinc-500 shrink-0" />
                            <input
                                type="text"
                                node_ref=input_ref
                                placeholder="Search tables and queries..."
                                class="text-base w-full focus:outline-none bg-transparent text-gray-900 dark:text-neutral-50 placeholder-gray-400 dark:placeholder-zinc-500"
                                prop:value=move || query.get()
                                on:input=move |ev| set_query.set(event_target_value(&ev))
                                on:keydown={
                                    let items = items_for_enter.clone();
                                    move |ev| {
                                        let ev: &web_sys::KeyboardEvent = ev.unchecked_ref();
                                        match ev.key().as_str() {
                                            "Escape" => overlay_ctx.close(),
                                            "Enter" => {
                                                let idx = selected_idx.get();
                                                if let Some(item) = items.get(idx) {
                                                    let name = item.name.clone();
                                                    let kind = item.kind.clone();
                                                    // Close FIRST so handlers can open another overlay if needed
                                                    overlay_ctx.close();
                                                    match kind {
                                                        FinderItemKind::Table => on_select.run(name),
                                                        FinderItemKind::Query => {
                                                            if let Some(cb) = on_query_select {
                                                                cb.run(name);
                                                            }
                                                        }
                                                    }
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
                        // Results list
                        <div class="pb-2 max-h-64 overflow-y-auto">
                            {
                                let mut global_idx = 0usize;
                                let mut views = Vec::new();

                                // Queries group header
                                if has_queries {
                                    views.push(view! {
                                        <div class="px-4 pt-2 pb-1 text-[11px] font-medium text-gray-400 dark:text-zinc-500 uppercase tracking-wider">"Queries"</div>
                                    }.into_any());
                                }

                                for item in items.iter().filter(|i| matches!(i.kind, FinderItemKind::Query)) {
                                    let idx = global_idx;
                                    global_idx += 1;
                                    let item_name = item.name.clone();
                                    let click_name = item.name.clone();
                                    let is_selected = selected_idx.get() == idx;
                                    let class = if is_selected {
                                        "px-4 py-2 flex items-center gap-3 text-[13px] cursor-pointer bg-indigo-50 dark:bg-indigo-500/25 text-indigo-600 dark:text-indigo-400"
                                    } else {
                                        "px-4 py-2 flex items-center gap-3 text-[13px] cursor-pointer hover:bg-indigo-50 dark:hover:bg-indigo-500/25 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors duration-100"
                                    };
                                    views.push(view! {
                                        <div
                                            class=class
                                            on:click=move |_| {
                                                // Close FIRST so handler can open another overlay if needed
                                                overlay_ctx.close();
                                                if let Some(cb) = on_query_select {
                                                    cb.run(click_name.clone());
                                                }
                                            }
                                        >
                                            <IconTerminal class="w-4 h-4 text-gray-400 dark:text-zinc-500 shrink-0" />
                                            <span class="font-medium text-gray-900 dark:text-neutral-50">{item_name}</span>
                                        </div>
                                    }.into_any());
                                }

                                // Tables group header
                                if has_tables {
                                    views.push(view! {
                                        <div class="px-4 pt-2 pb-1 text-[11px] font-medium text-gray-400 dark:text-zinc-500 uppercase tracking-wider">"Tables"</div>
                                    }.into_any());
                                }

                                for item in items.iter().filter(|i| matches!(i.kind, FinderItemKind::Table)) {
                                    let idx = global_idx;
                                    global_idx += 1;
                                    let item_name = item.name.clone();
                                    let click_name = item.name.clone();
                                    let is_selected = selected_idx.get() == idx;
                                    let class = if is_selected {
                                        "px-4 py-2 flex items-center gap-3 text-[13px] cursor-pointer bg-indigo-50 dark:bg-indigo-500/25 text-indigo-600 dark:text-indigo-400"
                                    } else {
                                        "px-4 py-2 flex items-center gap-3 text-[13px] cursor-pointer hover:bg-indigo-50 dark:hover:bg-indigo-500/25 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors duration-100"
                                    };
                                    views.push(view! {
                                        <div
                                            class=class
                                            on:click=move |_| {
                                                // Close FIRST so handler can open another overlay if needed
                                                overlay_ctx.close();
                                                on_select.run(click_name.clone());
                                            }
                                        >
                                            <IconTable class="w-4 h-4 text-gray-400 dark:text-zinc-500 shrink-0" />
                                            <span class="font-medium text-gray-900 dark:text-neutral-50">{item_name}</span>
                                        </div>
                                    }.into_any());
                                }

                                views
                            }
                        </div>
                    </div>
                </div>
            })
        } else {
            None
        }
    }
}
