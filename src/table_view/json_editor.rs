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
                class="bg-white rounded-lg shadow-xl border border-gray-200 w-[600px] max-h-[80vh] flex flex-col"
                on:click=move |ev| ev.stop_propagation()
            >
                // Header
                <div class="px-4 py-3 border-b border-gray-200 flex items-center justify-between shrink-0">
                    <h3 class="text-[13px] font-semibold text-gray-900">"Edit JSON"</h3>
                    <button
                        class="text-gray-400 hover:bg-gray-100 hover:text-gray-900 p-1 rounded-md transition-colors duration-100"
                        on:click=on_cancel_click
                    >
                        <IconX class="w-4 h-4" />
                    </button>
                </div>

                // Body
                <div class="px-4 py-4 flex-1 overflow-hidden flex flex-col gap-3">
                    <textarea
                        class="w-full flex-1 min-h-[200px] bg-gray-50 border border-gray-200 rounded-md px-3 py-2 text-[13px] font-mono resize-y focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500"
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
                        <p class="text-[12px] text-red-500">{err}</p>
                    })}
                </div>

                // Footer
                <div class="px-4 py-3 border-t border-gray-200 flex items-center justify-end gap-2 shrink-0">
                    <button
                        class="text-gray-500 hover:bg-gray-100 hover:text-gray-900 px-3 py-1.5 rounded-md text-[13px] transition-colors duration-100"
                        on:click=move |_| on_cancel.run(())
                    >
                        "Cancel"
                    </button>
                    <button
                        class="bg-indigo-500 hover:bg-indigo-600 text-white text-[13px] font-medium px-3 py-1.5 rounded-md transition-colors duration-100"
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
