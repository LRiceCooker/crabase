use leptos::prelude::*;

use crate::icons::IconPlay;

#[component]
pub fn SqlToolbar(
    on_run: Callback<()>,
    running: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="h-10 flex items-center justify-end px-3 gap-2 border-b border-gray-200 bg-white shrink-0">
            <button
                class="bg-emerald-500 hover:bg-emerald-600 text-white text-[13px] font-medium px-3 py-1.5 rounded-md transition-colors duration-100 flex items-center gap-1.5 disabled:opacity-50"
                disabled=move || running.get()
                on:click=move |_| on_run.run(())
            >
                <IconPlay class="w-4 h-4" />
                {move || if running.get() { "Running..." } else { "Run" }}
            </button>
        </div>
    }
}
