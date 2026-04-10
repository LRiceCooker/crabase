use leptos::prelude::*;

use crate::icons::{IconPlus, IconTrash2, IconX};

/// Modal edit request for array values.
#[derive(Clone, Debug)]
pub struct ArrayEditRequest {
    pub row: usize,
    pub col: usize,
    pub value: serde_json::Value,
}

/// Modal editor for Postgres array columns.
/// Displays a list of items with add/remove controls. Each item is a text input.
#[component]
pub fn ArrayEditorModal(
    request: ArrayEditRequest,
    on_save: Callback<(usize, usize, serde_json::Value)>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let row = request.row;
    let col = request.col;

    // Parse initial array items
    let initial_items: Vec<String> = match &request.value {
        serde_json::Value::Array(arr) => arr
            .iter()
            .map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => String::new(),
                _ => v.to_string(),
            })
            .collect(),
        serde_json::Value::String(s) => {
            // Try to parse from Postgres array literal like {a,b,c}
            parse_pg_array_literal(s)
        }
        _ => vec![],
    };

    let items = RwSignal::new(initial_items);

    let on_save_click = move |_| {
        let arr: Vec<serde_json::Value> = items
            .get()
            .into_iter()
            .map(|s| {
                if s.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::String(s)
                }
            })
            .collect();
        on_save.run((row, col, serde_json::Value::Array(arr)));
    };

    let on_add = move |_| {
        items.update(|list| list.push(String::new()));
    };

    view! {
        <div
            class="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center"
            on:click=move |_| on_cancel.run(())
        >
            <div
                class="bg-white dark:bg-zinc-900 rounded-lg shadow-xl dark:shadow-black/40 border border-gray-200 dark:border-white/[0.08] w-[500px] max-h-[80vh] flex flex-col dark:ring-1 dark:ring-white/[0.06]"
                on:click=move |ev| ev.stop_propagation()
            >
                // Header
                <div class="px-4 py-3 border-b border-gray-200 dark:border-zinc-800 flex items-center justify-between shrink-0">
                    <h3 class="text-[13px] font-semibold text-gray-900 dark:text-neutral-50">"Edit Array"</h3>
                    <button
                        class="text-gray-400 dark:text-zinc-500 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 p-1 rounded-md transition-colors duration-100"
                        on:click=move |_| on_cancel.run(())
                    >
                        <IconX class="w-4 h-4" />
                    </button>
                </div>

                // Body — list of items
                <div class="px-4 py-4 flex-1 overflow-y-auto flex flex-col gap-2">
                    {move || {
                        let current = items.get();
                        current.into_iter().enumerate().map(|(idx, item)| {
                            view! {
                                <div class="flex items-center gap-2">
                                    <span class="text-[10px] text-gray-400 dark:text-zinc-500 w-5 shrink-0 text-right">{idx}</span>
                                    <input
                                        type="text"
                                        class="flex-1 bg-white dark:bg-zinc-800 border border-gray-200 dark:border-zinc-700 rounded-md px-2 py-1 text-xs font-mono text-gray-900 dark:text-neutral-50 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500"
                                        prop:value=item
                                        on:input=move |ev| {
                                            let v = event_target_value(&ev);
                                            items.update(|list| {
                                                if let Some(elem) = list.get_mut(idx) {
                                                    *elem = v;
                                                }
                                            });
                                        }
                                    />
                                    <button
                                        class="text-gray-300 dark:text-zinc-600 hover:text-red-500 dark:hover:text-red-400 p-0.5 rounded transition-colors duration-100"
                                        title="Remove item"
                                        on:click=move |_| {
                                            items.update(|list| { list.remove(idx); });
                                        }
                                    >
                                        <IconTrash2 class="w-3.5 h-3.5" />
                                    </button>
                                </div>
                            }
                        }).collect::<Vec<_>>()
                    }}
                    <button
                        class="flex items-center gap-1 text-[12px] text-indigo-500 dark:text-indigo-400 hover:text-indigo-600 dark:hover:text-indigo-300 px-2 py-1 rounded-md hover:bg-indigo-50 dark:hover:bg-indigo-500/10 transition-colors duration-100 self-start"
                        on:click=on_add
                    >
                        <IconPlus class="w-3.5 h-3.5" />
                        "Add item"
                    </button>
                </div>

                // Footer
                <div class="px-4 py-3 border-t border-gray-200 dark:border-zinc-800 flex items-center justify-end gap-2 shrink-0">
                    <button
                        class="text-gray-500 dark:text-zinc-400 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 px-3 py-1.5 rounded-md text-[13px] transition-colors duration-100"
                        on:click=move |_| on_cancel.run(())
                    >
                        "Cancel"
                    </button>
                    <button
                        class="bg-indigo-500 hover:bg-indigo-600 dark:hover:bg-indigo-400 text-white text-[13px] font-medium px-3 py-1.5 rounded-md transition-colors duration-100"
                        on:click=on_save_click
                    >
                        "Save"
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Parse a Postgres array literal like `{a,b,c}` or `{1,2,3}` into a Vec<String>.
fn parse_pg_array_literal(s: &str) -> Vec<String> {
    let s = s.trim();
    if s.starts_with('{') && s.ends_with('}') {
        let inner = &s[1..s.len() - 1];
        if inner.is_empty() {
            return vec![];
        }
        inner.split(',').map(|item| item.trim().to_string()).collect()
    } else {
        vec![s.to_string()]
    }
}
