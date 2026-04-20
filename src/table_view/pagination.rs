use leptos::prelude::*;

use crate::icons::{IconChevronLeft, IconChevronRight};

#[component]
pub fn Pagination(
    page: ReadSignal<u32>,
    page_size: ReadSignal<u32>,
    total_count: ReadSignal<u64>,
    on_page_change: Callback<u32>,
    on_page_size_change: Callback<u32>,
) -> impl IntoView {
    let total_pages = move || {
        let count = total_count.get();
        let size = page_size.get() as u64;
        if size == 0 {
            1
        } else {
            count.div_ceil(size).max(1) as u32
        }
    };

    let can_prev = move || page.get() > 1;
    let can_next = move || page.get() < total_pages();

    let on_prev = move |_| {
        let p = page.get();
        if p > 1 {
            on_page_change.run(p - 1);
        }
    };

    let on_next = move |_| {
        let p = page.get();
        if p < total_pages() {
            on_page_change.run(p + 1);
        }
    };

    view! {
        <div class="flex items-center justify-between px-3 py-2 border-t border-gray-200 dark:border-zinc-800 bg-gray-50 dark:bg-[#0F0F11] text-[12px] text-gray-500 dark:text-zinc-400 shrink-0">
            <div class="flex items-center gap-2">
                <span>"Rows per page:"</span>
                <select
                    class="bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-800 rounded px-1.5 py-0.5 text-[12px] text-gray-900 dark:text-neutral-50 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 dark:focus:ring-indigo-500/60 focus:border-indigo-500"
                    on:change=move |ev| {
                        if let Ok(v) = event_target_value(&ev).parse::<u32>() {
                            on_page_size_change.run(v);
                        }
                    }
                >
                    <option value="25" selected=move || page_size.get() == 25>"25"</option>
                    <option value="50" selected=move || page_size.get() == 50>"50"</option>
                    <option value="100" selected=move || page_size.get() == 100>"100"</option>
                </select>
                <span class="text-gray-400 dark:text-zinc-500">
                    {move || format!("{} rows", total_count.get())}
                </span>
            </div>
            <div class="flex items-center gap-2">
                <span>
                    {move || format!("Page {} of {}", page.get(), total_pages())}
                </span>
                <button
                    class="p-1 rounded hover:bg-gray-200 dark:hover:bg-zinc-700 disabled:opacity-30 disabled:cursor-not-allowed transition-colors duration-100"
                    disabled=move || !can_prev()
                    on:click=on_prev
                >
                    <IconChevronLeft class="w-4 h-4" />
                </button>
                <button
                    class="p-1 rounded hover:bg-gray-200 dark:hover:bg-zinc-700 disabled:opacity-30 disabled:cursor-not-allowed transition-colors duration-100"
                    disabled=move || !can_next()
                    on:click=on_next
                >
                    <IconChevronRight class="w-4 h-4" />
                </button>
            </div>
        </div>
    }
}
