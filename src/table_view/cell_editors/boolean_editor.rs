use leptos::prelude::*;

use super::auto_focus_select_ref;

/// Inline editor for boolean columns.
/// When nullable, renders a `<select>` with NULL/True/False options.
/// When non-nullable, renders a `<select>` with True/False options.
#[component]
pub fn BooleanEditor(
    value: serde_json::Value,
    #[prop(default = false)] is_nullable: bool,
    on_commit: Callback<serde_json::Value>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let current = match &value {
        serde_json::Value::Bool(true) => "true".to_string(),
        serde_json::Value::Bool(false) => "false".to_string(),
        serde_json::Value::Null => String::new(),
        _ => String::new(),
    };
    let (val, set_val) = signal(current);

    view! {
        <select
            class="w-full bg-white dark:bg-zinc-900 text-xs font-mono text-gray-900 dark:text-neutral-50 px-1 py-0 border-0 outline-none cursor-pointer"
            prop:value=move || val.get()
            on:change=move |ev| {
                let v = event_target_value(&ev);
                set_val.set(v.clone());
                match v.as_str() {
                    "true" => on_commit.run(serde_json::Value::Bool(true)),
                    "false" => on_commit.run(serde_json::Value::Bool(false)),
                    _ => on_commit.run(serde_json::Value::Null),
                }
            }
            on:keydown=move |ev| {
                if ev.key() == "Escape" {
                    on_cancel.run(());
                }
            }
            node_ref=auto_focus_select_ref()
        >
            {if is_nullable {
                Some(view! {
                    <option value="">"NULL"</option>
                })
            } else {
                None
            }}
            <option value="true">"true"</option>
            <option value="false">"false"</option>
        </select>
    }
}
