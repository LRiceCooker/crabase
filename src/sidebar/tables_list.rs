use leptos::prelude::*;

use crate::icons::IconTable;

#[component]
pub fn TablesList(
    tables: ReadSignal<Vec<String>>,
    active_table: Memo<Option<String>>,
    on_select: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="p-3">
            <h2 class="text-[11px] font-medium text-gray-400 uppercase tracking-wider mb-2">"Tables"</h2>
            <div class="flex flex-col gap-0.5">
                {move || {
                    let current_active = active_table.get();
                    tables.get().into_iter().map(|name| {
                        let is_active = current_active.as_ref() == Some(&name);
                        let item_class = if is_active {
                            "flex items-center gap-2 px-3 py-1 text-[13px] rounded-md cursor-pointer transition-colors duration-100 bg-indigo-50 text-indigo-600"
                        } else {
                            "flex items-center gap-2 px-3 py-1 text-[13px] text-gray-700 rounded-md hover:bg-gray-100 cursor-pointer transition-colors duration-100"
                        };
                        let icon_class = if is_active {
                            "w-4 h-4 text-indigo-400 shrink-0"
                        } else {
                            "w-4 h-4 text-gray-400 shrink-0"
                        };
                        let click_name = name.clone();
                        view! {
                            <div
                                class=item_class
                                on:click=move |_| on_select.run(click_name.clone())
                            >
                                <IconTable class=icon_class />
                                <span class="truncate">{name}</span>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
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
