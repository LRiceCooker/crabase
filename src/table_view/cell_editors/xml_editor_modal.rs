use leptos::prelude::*;

use crate::icons::IconX;
use crate::sql_editor::codemirror::{CodeMirrorEditor, CodeMirrorHandle};

/// Modal edit request for XML values.
#[derive(Clone, Debug)]
pub struct XmlEditRequest {
    pub row: usize,
    pub col: usize,
    pub value: serde_json::Value,
}

/// Modal editor for XML columns. Uses CodeMirror (plain text mode — no XML syntax highlighting).
#[component]
pub fn XmlEditorModal(
    request: XmlEditRequest,
    on_save: Callback<(usize, usize, serde_json::Value)>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let content = match &request.value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        _ => request.value.to_string(),
    };
    let (cm_handle, set_cm_handle) = signal(Option::<CodeMirrorHandle>::None);

    let row = request.row;
    let col = request.col;

    let on_save_click = move |_| {
        let Some(handle) = cm_handle.get_untracked() else {
            return;
        };
        let raw = handle.get_content();
        if raw.is_empty() {
            on_save.run((row, col, serde_json::Value::Null));
        } else {
            on_save.run((row, col, serde_json::Value::String(raw)));
        }
    };

    view! {
        <div
            class="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center"
            on:click=move |_| on_cancel.run(())
        >
            <div
                class="bg-white dark:bg-zinc-900 rounded-lg shadow-xl dark:shadow-black/40 border border-gray-200 dark:border-white/[0.08] w-[600px] max-h-[80vh] flex flex-col dark:ring-1 dark:ring-white/[0.06]"
                on:click=move |ev| ev.stop_propagation()
            >
                // Header
                <div class="px-4 py-3 border-b border-gray-200 dark:border-zinc-800 flex items-center justify-between shrink-0">
                    <h3 class="text-[13px] font-semibold text-gray-900 dark:text-neutral-50">"Edit XML"</h3>
                    <button
                        class="text-gray-400 dark:text-zinc-500 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 p-1 rounded-md transition-colors duration-100"
                        on:click=move |_| on_cancel.run(())
                    >
                        <IconX class="w-4 h-4" />
                    </button>
                </div>

                // Body
                <div class="px-4 py-4 flex-1 overflow-hidden flex flex-col gap-3">
                    <div class="flex-1 min-h-[200px] border border-gray-200 dark:border-zinc-800 rounded-md overflow-hidden">
                        <CodeMirrorEditor
                            initial_content=content
                            language="text".to_string()
                            placeholder="Enter XML...".to_string()
                            handle=set_cm_handle
                        />
                    </div>
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
                        on:click=on_save_click
                    >
                        "Save"
                    </button>
                </div>
            </div>
        </div>
    }
}
