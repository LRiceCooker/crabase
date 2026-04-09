use leptos::prelude::*;

use crate::table_view::change_tracker::ChangeTracker;

#[component]
pub fn DirtyBar(
    changes: ChangeTracker,
    on_discard: Callback<()>,
    on_save: Callback<()>,
) -> impl IntoView {
    view! {
        {move || {
            if !changes.has_changes() {
                return None;
            }
            let count = changes.change_count();
            let label = if count == 1 {
                "1 change pending".to_string()
            } else {
                format!("{} changes pending", count)
            };

            Some(view! {
                <div class="fixed bottom-4 left-1/2 -translate-x-1/2 bg-white border border-gray-200 shadow-lg rounded-lg px-4 py-2 flex items-center gap-3 text-[13px] z-40">
                    <span class="text-gray-700">{label}</span>
                    <button
                        class="text-gray-500 hover:bg-gray-100 hover:text-gray-900 px-2 py-1 rounded-md transition-colors duration-100"
                        on:click=move |_| on_discard.run(())
                    >
                        "Discard"
                    </button>
                    <button
                        class="bg-indigo-500 hover:bg-indigo-600 text-white text-[13px] font-medium px-3 py-1.5 rounded-md transition-colors duration-100"
                        on:click=move |_| on_save.run(())
                    >
                        "Save changes"
                    </button>
                </div>
            })
        }}
    }
}
