use leptos::prelude::*;

use crate::icons::IconTerminal;

#[component]
pub fn SavedQueriesList(
    queries: ReadSignal<Vec<String>>,
    on_select: Callback<String>,
) -> impl IntoView {
    view! {
        {move || {
            let items = queries.get();
            if items.is_empty() {
                return None;
            }
            Some(view! {
                <div class="p-3 pb-2 border-b border-gray-200 dark:border-zinc-800">
                    <h2 class="text-[11px] font-medium text-gray-400 dark:text-zinc-500 uppercase tracking-wider mb-2">"Saved Queries"</h2>
                    <div class="flex flex-col gap-0.5">
                        {items.into_iter().map(|name| {
                            let click_name = name.clone();
                            view! {
                                <div
                                    class="flex items-center gap-2 px-3 py-1 text-[13px] text-gray-700 dark:text-zinc-300 rounded-md hover:bg-gray-100 dark:hover:bg-zinc-800 cursor-pointer transition-colors duration-100"
                                    on:click=move |_| on_select.run(click_name.clone())
                                >
                                    <IconTerminal class="w-4 h-4 text-gray-400 dark:text-zinc-500 shrink-0" />
                                    <span class="truncate">{name}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            })
        }}
    }
}
