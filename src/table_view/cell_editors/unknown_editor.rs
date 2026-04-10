use leptos::prelude::*;

/// Read-only editor for unknown/unsupported Postgres types.
/// Displays the raw text with a tooltip explaining the type isn't fully supported.
#[component]
pub fn UnknownEditor(
    value: serde_json::Value,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let display = match &value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => "NULL".to_string(),
        _ => value.to_string(),
    };

    view! {
        <div
            class="w-full text-xs font-mono text-gray-500 dark:text-zinc-400 px-1 py-0 italic cursor-not-allowed"
            title="This Postgres type is not fully supported for editing"
            tabindex=0
            on:keydown=move |ev| {
                if ev.key() == "Escape" {
                    on_cancel.run(());
                }
            }
        >
            {display}
        </div>
    }
}
