use leptos::prelude::*;

use super::{auto_focus_input_ref, INPUT_CLASS};

/// Inline editor for numeric types (smallint, integer, bigint, decimal, real, double, money).
/// `is_integer` controls parsing (i64 vs f64). `step` controls the HTML step attribute.
#[component]
pub fn NumberEditor(
    value: serde_json::Value,
    #[prop(default = true)] is_integer: bool,
    #[prop(default = "any".to_string())] step: String,
    on_commit: Callback<serde_json::Value>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let initial = match &value {
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        _ => value.to_string(),
    };
    let (val, set_val) = signal(initial);
    let step_attr = if step == "any" && is_integer { "1".to_string() } else { step };

    let commit = move |v: String| {
        if v.is_empty() {
            on_commit.run(serde_json::Value::Null);
        } else if is_integer {
            if let Ok(n) = v.parse::<i64>() {
                on_commit.run(serde_json::Value::Number(n.into()));
            }
        } else if let Some(n) = v.parse::<f64>().ok().and_then(serde_json::Number::from_f64) {
            on_commit.run(serde_json::Value::Number(n));
        }
    };

    let commit_clone = commit.clone();
    view! {
        <input
            type="number"
            step=step_attr
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
