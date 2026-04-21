use leptos::prelude::*;

use crate::icons::{IconPlus, IconRefreshCw, IconTable};

/// Toolbar showing the table name, row count, and action buttons (add row, refresh).
#[component]
pub fn Toolbar(
    table_name: String,
    total_count: u64,
    on_add_row: Callback<()>,
    on_refresh: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="h-10 flex items-center justify-between px-3 border-b border-gray-200 dark:border-zinc-800 bg-white dark:bg-neutral-950 shrink-0">
            <div class="flex items-center gap-2">
                <IconTable class="w-4 h-4 text-gray-400 dark:text-zinc-500" />
                <span class="text-[13px] font-semibold text-gray-900 dark:text-neutral-50">{table_name}</span>
                <span class="text-[11px] text-gray-400 dark:text-zinc-500">{format!("{total_count} rows")}</span>
            </div>
            <div class="flex items-center gap-1">
                <button
                    class="text-gray-500 dark:text-zinc-400 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 px-2 py-1 rounded-md transition-colors duration-100 flex items-center gap-1 text-[13px]"
                    title="Add row"
                    on:click=move |_| on_add_row.run(())
                >
                    <IconPlus class="w-4 h-4" />
                </button>
                <button
                    class="text-gray-500 dark:text-zinc-400 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 px-2 py-1 rounded-md transition-colors duration-100"
                    title="Refresh"
                    on:click=move |_| on_refresh.run(())
                >
                    <IconRefreshCw class="w-4 h-4" />
                </button>
            </div>
        </div>
    }
}
