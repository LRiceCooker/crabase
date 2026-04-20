use leptos::prelude::*;

use super::{auto_focus_input_ref, INPUT_CLASS};

/// Inline editor for inet, cidr, and macaddr columns. Text input with format-appropriate placeholder.
#[component]
pub fn InetEditor(
    value: serde_json::Value,
    #[prop(into)] editor_type: String,
    on_commit: Callback<serde_json::Value>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let initial = match &value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        _ => value.to_string(),
    };
    let (val, set_val) = signal(initial);

    let placeholder = match editor_type.as_str() {
        "cidr" => "e.g. 192.168.0.0/24",
        "macaddr" => "e.g. 08:00:2b:01:02:03",
        _ => "e.g. 192.168.0.1",
    };

    let commit = move |v: String| {
        if v.is_empty() {
            on_commit.run(serde_json::Value::Null);
        } else {
            on_commit.run(serde_json::Value::String(v));
        }
    };

    let commit_clone = commit;
    view! {
        <input
            type="text"
            class=INPUT_CLASS
            placeholder=placeholder
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
