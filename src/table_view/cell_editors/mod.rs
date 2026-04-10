pub mod array_editor_modal;
pub mod bit_editor;
pub mod boolean_editor;
pub mod bytea_editor;
pub mod date_editor;
pub mod datetime_editor;
pub mod enum_editor;
pub mod inet_editor;
pub mod interval_editor;
pub mod number_editor;
pub mod range_editor;
pub mod text_editor;
pub mod time_editor;
pub mod unknown_editor;
pub mod uuid_editor;
pub mod xml_editor_modal;

use leptos::prelude::*;

/// Shared input CSS class for all inline cell editors.
pub const INPUT_CLASS: &str = "w-full bg-white dark:bg-zinc-900 text-xs font-mono text-gray-900 dark:text-neutral-50 px-1 py-0 border-0 outline-none";

/// Creates a NodeRef that auto-focuses and selects an input element on mount.
pub fn auto_focus_input_ref() -> NodeRef<leptos::html::Input> {
    let node_ref = NodeRef::<leptos::html::Input>::new();
    Effect::new(move |_| {
        if let Some(el) = node_ref.get() {
            let _ = el.focus();
            let _ = el.select();
        }
    });
    node_ref
}

/// Creates a NodeRef that auto-focuses a select element on mount.
pub fn auto_focus_select_ref() -> NodeRef<leptos::html::Select> {
    let node_ref = NodeRef::<leptos::html::Select>::new();
    Effect::new(move |_| {
        if let Some(el) = node_ref.get() {
            let _ = el.focus();
        }
    });
    node_ref
}

/// Small "×" button that sets the cell value to NULL. Used for nullable columns.
/// Uses mousedown + preventDefault to avoid triggering the input's blur handler.
#[component]
pub fn NullButton(on_commit: Callback<serde_json::Value>) -> impl IntoView {
    view! {
        <button
            class="shrink-0 text-[10px] text-gray-400 dark:text-zinc-500 hover:text-red-500 dark:hover:text-red-400 px-1 py-0.5 rounded hover:bg-red-50 dark:hover:bg-red-500/10 transition-colors duration-100"
            title="Set NULL"
            tabindex=-1
            on:mousedown=move |ev| {
                ev.prevent_default();
                on_commit.run(serde_json::Value::Null);
            }
        >
            "×"
        </button>
    }
}
