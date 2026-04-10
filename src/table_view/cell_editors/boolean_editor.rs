use leptos::prelude::*;

/// Inline editor for boolean columns. Renders a checkbox.
#[component]
pub fn BooleanEditor(
    value: serde_json::Value,
    on_commit: Callback<serde_json::Value>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let checked = value.as_bool().unwrap_or(false);

    view! {
        <input
            type="checkbox"
            class="accent-indigo-500 dark:accent-indigo-400"
            prop:checked=checked
            on:change=move |ev| {
                let target = event_target::<web_sys::HtmlInputElement>(&ev);
                on_commit.run(serde_json::Value::Bool(target.checked()));
            }
            on:keydown=move |ev| {
                if ev.key() == "Escape" {
                    on_cancel.run(());
                }
            }
        />
    }
}
