use leptos::prelude::*;

use super::{auto_focus_input_ref, INPUT_CLASS};

/// Inline editor for text types (char, varchar, text).
/// Enforces max_length if provided.
#[component]
pub fn TextEditor(
    value: serde_json::Value,
    #[prop(default = 0)] max_length: i32,
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

    let commit_clone = commit;
    let maxlen = if max_length > 0 { max_length.to_string() } else { String::new() };
    let has_maxlen = max_length > 0;

    view! {
        <input
            type="text"
            class=INPUT_CLASS
            prop:value=move || val.get()
            maxlength=move || if has_maxlen { maxlen.clone() } else { String::new() }
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
