use leptos::prelude::*;

use crate::icons::{IconPlay, IconSave};

/// Toolbar for the SQL editor with Run and Save buttons.
#[component]
pub fn SqlToolbar(
    on_run: Callback<()>,
    running: ReadSignal<bool>,
    #[prop(optional)] on_save: Option<Callback<()>>,
    #[prop(optional)] is_dirty: Option<Signal<bool>>,
) -> impl IntoView {
    let dirty = move || is_dirty.map(|s| s.get()).unwrap_or(false);
    let has_save = on_save.is_some();

    view! {
        <div class="h-10 flex items-center justify-end px-3 gap-2 border-b border-gray-200 dark:border-zinc-800 bg-white dark:bg-neutral-950 shrink-0">
            {move || {
                if has_save {
                    let on_save = on_save.unwrap();
                    view! {
                        <button
                            class="text-gray-500 dark:text-zinc-400 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 text-[13px] font-medium px-3 py-1.5 rounded-md transition-colors duration-100 flex items-center gap-1.5 disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=move || !dirty()
                            on:click=move |_| on_save.run(())
                        >
                            <IconSave class="w-4 h-4" />
                            "Save"
                        </button>
                    }.into_any()
                } else {
                    view! { <span /> }.into_any()
                }
            }}
            <button
                class="bg-emerald-500 hover:bg-emerald-600 dark:hover:bg-emerald-400 text-white text-[13px] font-medium px-3 py-1.5 rounded-md transition-colors duration-100 flex items-center gap-1.5 disabled:opacity-50"
                disabled=move || running.get()
                on:click=move |_| on_run.run(())
            >
                <IconPlay class="w-4 h-4" />
                {move || if running.get() { "Running..." } else { "Run" }}
            </button>
        </div>
    }
}
