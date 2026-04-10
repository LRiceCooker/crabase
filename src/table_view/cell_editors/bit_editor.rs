use leptos::prelude::*;

use super::{auto_focus_input_ref, INPUT_CLASS};

/// Inline editor for bit/bit varying columns. Text input restricted to 0 and 1 characters.
#[component]
pub fn BitEditor(
    value: serde_json::Value,
    on_commit: Callback<serde_json::Value>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let initial = match &value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        _ => value.to_string(),
    };
    let (val, set_val) = signal(initial);

    let commit = move |v: String| {
        if v.is_empty() {
            on_commit.run(serde_json::Value::Null);
        } else {
            on_commit.run(serde_json::Value::String(v));
        }
    };

    let commit_clone = commit.clone();
    view! {
        <input
            type="text"
            class=INPUT_CLASS
            pattern="[01]*"
            placeholder="e.g. 10110"
            prop:value=move || val.get()
            on:input=move |ev| {
                let v = event_target_value(&ev);
                // Filter to only 0 and 1 characters
                let filtered: String = v.chars().filter(|c| *c == '0' || *c == '1').collect();
                set_val.set(filtered);
            }
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
