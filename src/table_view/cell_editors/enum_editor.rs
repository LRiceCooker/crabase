use leptos::prelude::*;

use super::auto_focus_select_ref;

/// Inline editor for enum columns. Renders a `<select>` with allowed values.
#[component]
pub fn EnumEditor(
    value: serde_json::Value,
    enum_values: Vec<String>,
    is_nullable: bool,
    on_commit: Callback<serde_json::Value>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let current = match &value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        _ => value.to_string(),
    };
    let (val, set_val) = signal(current);

    view! {
        <select
            class="w-full bg-white dark:bg-zinc-900 text-xs font-mono text-gray-900 dark:text-neutral-50 px-1 py-0 border-0 outline-none cursor-pointer"
            prop:value=move || val.get()
            on:change=move |ev| {
                let v = event_target_value(&ev);
                set_val.set(v.clone());
                if v.is_empty() {
                    on_commit.run(serde_json::Value::Null);
                } else {
                    on_commit.run(serde_json::Value::String(v));
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
            {enum_values.iter().map(|ev| {
                let v = ev.clone();
                let label = ev.clone();
                view! {
                    <option value=v>{label}</option>
                }
            }).collect::<Vec<_>>()}
        </select>
    }
}
