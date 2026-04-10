use leptos::prelude::*;

use crate::icons::{IconChevronLeft, IconChevronRight, IconSearch, IconX};

/// A match position: (row_idx, col_idx).
pub type FindMatch = (usize, usize);

/// Floating find bar overlay for searching visible table cells.
#[component]
pub fn FindOverlay(
    /// Whether the overlay is visible
    visible: ReadSignal<bool>,
    /// Signal to write search query
    search_query: RwSignal<String>,
    /// Current matches (computed externally)
    matches: Memo<Vec<FindMatch>>,
    /// Current match index
    current_match: RwSignal<usize>,
    /// Close callback
    on_close: Callback<()>,
) -> impl IntoView {
    let input_ref = NodeRef::<leptos::html::Input>::new();

    // Auto-focus input when overlay becomes visible
    Effect::new(move |_| {
        if visible.get() {
            if let Some(el) = input_ref.get() {
                let _ = el.focus();
            }
        }
    });

    let go_next = move |_| {
        let count = matches.get().len();
        if count > 0 {
            current_match.update(|i| *i = (*i + 1) % count);
        }
    };

    let go_prev = move |_| {
        let count = matches.get().len();
        if count > 0 {
            current_match.update(|i| {
                if *i == 0 {
                    *i = count - 1;
                } else {
                    *i -= 1;
                }
            });
        }
    };

    view! {
        <div
            class="absolute top-0 right-0 z-30 m-2"
            class:hidden=move || !visible.get()
        >
            <div class="flex items-center gap-1.5 bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-700 dark:ring-1 dark:ring-white/[0.06] rounded-md shadow-xl px-2.5 py-1.5">
                <IconSearch class="w-3.5 h-3.5 text-gray-400 dark:text-zinc-500 shrink-0" />
                <input
                    node_ref=input_ref
                    type="text"
                    class="bg-transparent text-[13px] text-gray-900 dark:text-neutral-50 outline-none w-48 placeholder:text-gray-400 dark:placeholder:text-zinc-500"
                    placeholder="Find in table..."
                    prop:value=move || search_query.get()
                    on:input=move |ev| {
                        search_query.set(event_target_value(&ev));
                        current_match.set(0);
                    }
                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                        if ev.key() == "Escape" {
                            on_close.run(());
                        } else if ev.key() == "Enter" {
                            if ev.shift_key() {
                                go_prev(());
                            } else {
                                go_next(());
                            }
                        }
                    }
                />
                // Match count
                <span class="text-[11px] text-gray-400 dark:text-zinc-500 whitespace-nowrap min-w-[40px] text-right">
                    {move || {
                        let count = matches.get().len();
                        let query = search_query.get();
                        if query.is_empty() {
                            String::new()
                        } else if count == 0 {
                            "0/0".to_string()
                        } else {
                            format!("{}/{}", current_match.get() + 1, count)
                        }
                    }}
                </span>
                // Navigation buttons
                <button
                    class="p-0.5 text-gray-400 dark:text-zinc-500 hover:text-gray-700 dark:hover:text-zinc-300 transition-colors duration-100 rounded"
                    title="Previous (Shift+Enter)"
                    on:click=move |_| go_prev(())
                >
                    <IconChevronLeft class="w-3.5 h-3.5" />
                </button>
                <button
                    class="p-0.5 text-gray-400 dark:text-zinc-500 hover:text-gray-700 dark:hover:text-zinc-300 transition-colors duration-100 rounded"
                    title="Next (Enter)"
                    on:click=move |_| go_next(())
                >
                    <IconChevronRight class="w-3.5 h-3.5" />
                </button>
                // Close button
                <button
                    class="p-0.5 text-gray-400 dark:text-zinc-500 hover:text-gray-700 dark:hover:text-zinc-300 transition-colors duration-100 rounded"
                    title="Close (Escape)"
                    on:click=move |_| on_close.run(())
                >
                    <IconX class="w-3.5 h-3.5" />
                </button>
            </div>
        </div>
    }
}
