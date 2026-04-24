use leptos::prelude::*;

use crate::icons::IconX;
use crate::sql_editor::codemirror::{CodeMirrorEditor, CodeMirrorHandle};

/// State for the JSON editor modal.
#[derive(Clone, Debug)]
pub struct JsonEditRequest {
    pub row: usize,
    pub col: usize,
    pub value: serde_json::Value,
}

/// Modal editor for JSON/JSONB columns with CodeMirror syntax highlighting.
#[component]
pub fn JsonEditorModal(
    request: JsonEditRequest,
    on_save: Callback<(usize, usize, serde_json::Value)>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let pretty = serde_json::to_string_pretty(&request.value).unwrap_or_default();
    let (parse_error, set_parse_error) = signal(Option::<String>::None);
    let (cm_handle, set_cm_handle) = signal(Option::<CodeMirrorHandle>::None);

    let row = request.row;
    let col = request.col;

    // Validate JSON on every change
    let on_change = Callback::new(move |content: String| {
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(_) => set_parse_error.set(None),
            Err(e) => set_parse_error.set(Some(format!("Invalid JSON: {e}"))),
        }
    });

    let on_save_click = move |_| {
        let Some(handle) = cm_handle.get_untracked() else {
            return;
        };
        let raw = handle.get_content();
        match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(val) => {
                set_parse_error.set(None);
                on_save.run((row, col, val));
            }
            Err(e) => {
                set_parse_error.set(Some(format!("Invalid JSON: {e}")));
            }
        }
    };

    let _on_cancel_click = move |_: web_sys::MouseEvent| {
        on_cancel.run(());
    };

    view! {
        // Overlay — use mousedown to close, fires before CodeMirror can intercept
        <div
            class="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center"
            on:mousedown=move |_| on_cancel.run(())
        >
            // Panel — stop mousedown propagation so clicks inside don't close
            <div
                class="bg-white dark:bg-zinc-900 rounded-lg shadow-xl dark:shadow-black/40 border border-gray-200 dark:border-white/[0.08] w-[600px] max-h-[80vh] flex flex-col dark:ring-1 dark:ring-white/[0.06]"
                on:mousedown=move |ev: web_sys::MouseEvent| ev.stop_propagation()
            >
                // Header
                <div class="px-4 py-3 border-b border-gray-200 dark:border-zinc-800 flex items-center justify-between shrink-0">
                    <h3 class="text-[13px] font-semibold text-gray-900 dark:text-neutral-50">"Edit JSON"</h3>
                    <button
                        class="text-gray-400 dark:text-zinc-500 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 p-1 rounded-md transition-colors duration-100"
                        on:mousedown=move |ev: web_sys::MouseEvent| { ev.stop_propagation(); on_cancel.run(()); }
                    >
                        <IconX class="w-4 h-4" />
                    </button>
                </div>

                // Body
                <div class="px-4 py-4 flex-1 min-h-0 flex flex-col gap-3">
                    <div class="flex-1 min-h-[200px] border border-gray-200 dark:border-zinc-800 rounded-md overflow-auto">
                        <CodeMirrorEditor
                            initial_content=pretty
                            language="json".to_string()
                            placeholder="Enter JSON...".to_string()
                            on_change=on_change
                            handle=set_cm_handle
                        />
                    </div>

                    // Parse error
                    {move || parse_error.get().map(|err| view! {
                        <p class="text-[12px] text-red-500 dark:text-red-400">{err}</p>
                    })}
                </div>

                // Footer
                <div class="px-4 py-3 border-t border-gray-200 dark:border-zinc-800 flex items-center justify-end gap-2 shrink-0">
                    <button
                        class="text-gray-500 dark:text-zinc-400 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 px-3 py-1.5 rounded-md text-[13px] transition-colors duration-100"
                        on:mousedown=move |ev: web_sys::MouseEvent| { ev.stop_propagation(); on_cancel.run(()); }
                    >
                        "Cancel"
                    </button>
                    <button
                        class="bg-indigo-500 hover:bg-indigo-600 dark:hover:bg-indigo-400 text-white text-[13px] font-medium px-3 py-1.5 rounded-md transition-colors duration-100"
                        disabled=move || parse_error.get().is_some()
                        on:mousedown=move |ev: web_sys::MouseEvent| { ev.stop_propagation(); on_save_click(ev); }
                    >
                        "Save"
                    </button>
                </div>
            </div>
        </div>
    }
}
