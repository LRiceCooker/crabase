use leptos::prelude::*;

use crate::icons::IconX;
use super::theme_setting::ThemeSetting;

#[component]
pub fn SettingsView(
    set_show: WriteSignal<bool>,
) -> impl IntoView {
    let on_close = move |_| {
        set_show.set(false);
    };

    view! {
        <div class="bg-white dark:bg-zinc-900 rounded-lg border border-gray-200 dark:border-zinc-800 shadow-lg dark:shadow-black/40 max-w-lg mx-auto mt-8 dark:ring-1 dark:ring-white/[0.06]">
            // Header
            <div class="px-4 py-3 border-b border-gray-200 dark:border-zinc-800 flex items-center justify-between">
                <h2 class="text-[13px] font-semibold text-gray-900 dark:text-neutral-50">"Settings"</h2>
                <button
                    class="text-gray-400 dark:text-zinc-500 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 p-1 rounded-md transition-colors duration-100"
                    on:click=on_close
                >
                    <IconX class="w-4 h-4" />
                </button>
            </div>
            // Body
            <div class="px-4 py-4">
                <ThemeSetting />

                // Divider
                <div class="border-t border-gray-200 dark:border-zinc-800 my-4"></div>

                // Keyboard shortcuts placeholder
                <div class="flex flex-col gap-1.5">
                    <label class="text-[13px] font-semibold text-gray-700 dark:text-zinc-300">"Keyboard Shortcuts"</label>
                    <p class="text-[13px] text-gray-500 dark:text-zinc-400">"Shortcut customization coming soon."</p>
                </div>
            </div>
        </div>
    }
}
