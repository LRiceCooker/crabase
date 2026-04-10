use leptos::prelude::*;

use crate::icons::IconX;

/// State for the JSON editor modal.
#[derive(Clone, Debug)]
pub struct JsonEditRequest {
    pub row: usize,
    pub col: usize,
    pub value: serde_json::Value,
}

#[component]
pub fn JsonEditorModal(
    request: JsonEditRequest,
    on_save: Callback<(usize, usize, serde_json::Value)>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let pretty = serde_json::to_string_pretty(&request.value).unwrap_or_default();
    let (text, set_text) = signal(pretty);
    let (parse_error, set_parse_error) = signal(Option::<String>::None);

    let row = request.row;
    let col = request.col;

    let on_save_click = move |_| {
        let raw = text.get();
        match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(val) => {
                set_parse_error.set(None);
                on_save.run((row, col, val));
            }
            Err(e) => {
                set_parse_error.set(Some(format!("Invalid JSON: {}", e)));
            }
        }
    };

    let on_cancel_click = move |_| {
        on_cancel.run(());
    };

    view! {
        // Overlay
        <div
            class="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center"
            on:click=move |_| on_cancel.run(())
        >
            // Panel
            <div
                class="bg-white dark:bg-zinc-900 rounded-lg shadow-xl dark:shadow-black/40 border border-gray-200 dark:border-white/[0.08] w-[600px] max-h-[80vh] flex flex-col dark:ring-1 dark:ring-white/[0.06]"
                on:click=move |ev| ev.stop_propagation()
            >
                // Header
                <div class="px-4 py-3 border-b border-gray-200 dark:border-zinc-800 flex items-center justify-between shrink-0">
                    <h3 class="text-[13px] font-semibold text-gray-900 dark:text-neutral-50">"Edit JSON"</h3>
                    <button
                        class="text-gray-400 dark:text-zinc-500 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 p-1 rounded-md transition-colors duration-100"
                        on:click=on_cancel_click
                    >
                        <IconX class="w-4 h-4" />
                    </button>
                </div>

                // Body
                <div class="px-4 py-4 flex-1 overflow-hidden flex flex-col gap-3">
                    <textarea
                        class="w-full flex-1 min-h-[200px] bg-gray-50 dark:bg-[#0D0D0F] border border-gray-200 dark:border-zinc-800 rounded-md px-3 py-2 text-[13px] font-mono text-gray-900 dark:text-zinc-200 resize-y focus:outline-none focus:ring-2 focus:ring-indigo-500/20 dark:focus:ring-indigo-500/60 focus:border-indigo-500"
                        prop:value=move || text.get()
                        on:input=move |ev| {
                            let v = event_target_value(&ev);
                            set_text.set(v.clone());
                            // Clear error on edit
                            match serde_json::from_str::<serde_json::Value>(&v) {
                                Ok(_) => set_parse_error.set(None),
                                Err(e) => set_parse_error.set(Some(format!("Invalid JSON: {}", e))),
                            }
                        }
                        spellcheck="false"
                    />

                    // Parse error
                    {move || parse_error.get().map(|err| view! {
                        <p class="text-[12px] text-red-500 dark:text-red-400">{err}</p>
                    })}
                </div>

                // Footer
                <div class="px-4 py-3 border-t border-gray-200 dark:border-zinc-800 flex items-center justify-end gap-2 shrink-0">
                    <button
                        class="text-gray-500 dark:text-zinc-400 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 px-3 py-1.5 rounded-md text-[13px] transition-colors duration-100"
                        on:click=move |_| on_cancel.run(())
                    >
                        "Cancel"
                    </button>
                    <button
                        class="bg-indigo-500 hover:bg-indigo-600 dark:hover:bg-indigo-400 text-white text-[13px] font-medium px-3 py-1.5 rounded-md transition-colors duration-100"
                        disabled=move || parse_error.get().is_some()
                        on:click=on_save_click
                    >
                        "Save"
                    </button>
                </div>
            </div>
        </div>
    }
}
