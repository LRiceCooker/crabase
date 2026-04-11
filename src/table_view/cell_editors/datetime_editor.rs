use leptos::prelude::*;

use super::{auto_focus_input_ref, INPUT_CLASS};

/// Inline editor for timestamp/timestamptz columns. Uses native `<input type="datetime-local">`.
#[component]
pub fn DatetimeEditor(
    value: serde_json::Value,
    on_commit: Callback<serde_json::Value>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    // datetime-local input expects YYYY-MM-DDTHH:MM:SS format
    let initial = value.as_str().unwrap_or("").replace(' ', "T");
    // Trim trailing Z for timestamptz values
    let initial = initial.trim_end_matches('Z').to_string();
    let (val, set_val) = signal(initial);

    let commit = move |v: String| {
        on_commit.run(if v.is_empty() {
            serde_json::Value::Null
        } else {
            // Reformat from datetime-local (YYYY-MM-DDTHH:MM:SS) to Postgres-compatible
            // timestamp format (YYYY-MM-DD HH:MM:SS)
            serde_json::Value::String(v.replace('T', " "))
        });
    };

    let commit_clone = commit.clone();
    view! {
        <input
            type="datetime-local"
            step="1"
            class=INPUT_CLASS
            prop:value=move || val.get()
            on:input=move |ev| set_val.set(event_target_value(&ev))
            on:keydown=move |ev| {
                match ev.key().as_str() {
                    "Enter" | "Tab" => {
                        ev.prevent_default();
                        commit(val.get());
                    }
                    "Escape" => on_cancel.run(()),
                    _ => {}
                }
            }
            on:blur=move |_| commit_clone(val.get())
            node_ref=auto_focus_input_ref()
        />
    }
}
