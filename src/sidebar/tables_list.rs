use leptos::prelude::*;

use crate::icons::IconTable;

#[component]
pub fn TablesList(
    tables: ReadSignal<Vec<String>>,
) -> impl IntoView {
    view! {
        <div class="p-3">
            <h2 class="text-[11px] font-medium text-gray-400 uppercase tracking-wider mb-2">"Tables"</h2>
            <div class="flex flex-col gap-0.5">
                {move || tables.get().into_iter().map(|name| view! {
                    <div class="flex items-center gap-2 px-3 py-1 text-[13px] text-gray-700 rounded-md hover:bg-gray-100 cursor-pointer transition-colors duration-100">
                        <IconTable class="w-4 h-4 text-gray-400 shrink-0" />
                        <span class="truncate">{name}</span>
                    </div>
                }).collect::<Vec<_>>()}
            </div>
            {move || if tables.get().is_empty() {
                Some(view! {
                    <p class="text-[13px] text-gray-400 italic px-3">"No tables found"</p>
                })
            } else {
                None
            }}
        </div>
    }
}
