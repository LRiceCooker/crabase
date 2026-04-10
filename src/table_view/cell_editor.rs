use leptos::prelude::*;

/// Represents a cell edit completion.
#[derive(Clone, Debug)]
pub struct CellEdit {
    pub row: usize,
    pub col: usize,
    pub value: serde_json::Value,
}

/// Inline cell editor. Renders the appropriate input based on data_type.
/// Calls on_commit with the new value on Enter/Tab/blur, or on_cancel on Escape.
#[component]
pub fn CellEditor(
    data_type: String,
    value: serde_json::Value,
    on_commit: Callback<serde_json::Value>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let dt = data_type.to_uppercase();

    match dt.as_str() {
        "boolean" | "bool" => {
            let checked = value.as_bool().unwrap_or(false);
            view! {
                <input
                    type="checkbox"
                    class="accent-indigo-500"
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
            .into_any()
        }
        s if s.starts_with("INT") || s == "SERIAL" || s == "BIGSERIAL" || s == "SMALLINT"
            || s == "BIGINT" || s == "INTEGER" || s.starts_with("SMALLSERIAL") =>
        {
            let initial = match &value {
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => String::new(),
                _ => value.to_string(),
            };
            let (val, set_val) = signal(initial);
            view! {
                <input
                    type="number"
                    class="w-full bg-white dark:bg-zinc-900 text-xs font-mono text-gray-900 dark:text-neutral-50 px-1 py-0 border-0 outline-none"
                    prop:value=move || val.get()
                    on:input=move |ev| set_val.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        match ev.key().as_str() {
                            "Enter" | "Tab" => {
                                ev.prevent_default();
                                let v = val.get();
                                if v.is_empty() {
                                    on_commit.run(serde_json::Value::Null);
                                } else if let Ok(n) = v.parse::<i64>() {
                                    on_commit.run(serde_json::Value::Number(n.into()));
                                }
                            }
                            "Escape" => on_cancel.run(()),
                            _ => {}
                        }
                    }
                    on:blur=move |_| {
                        let v = val.get();
                        if v.is_empty() {
                            on_commit.run(serde_json::Value::Null);
                        } else if let Ok(n) = v.parse::<i64>() {
                            on_commit.run(serde_json::Value::Number(n.into()));
                        }
                    }
                    node_ref=auto_focus_ref()
                />
            }
            .into_any()
        }
        s if s.starts_with("FLOAT") || s == "REAL" || s == "DOUBLE PRECISION"
            || s.starts_with("NUMERIC") || s == "DECIMAL" =>
        {
            let initial = match &value {
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => String::new(),
                _ => value.to_string(),
            };
            let (val, set_val) = signal(initial);
            view! {
                <input
                    type="number"
                    step="any"
                    class="w-full bg-white dark:bg-zinc-900 text-xs font-mono text-gray-900 dark:text-neutral-50 px-1 py-0 border-0 outline-none"
                    prop:value=move || val.get()
                    on:input=move |ev| set_val.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        match ev.key().as_str() {
                            "Enter" | "Tab" => {
                                ev.prevent_default();
                                let v = val.get();
                                if v.is_empty() {
                                    on_commit.run(serde_json::Value::Null);
                                } else if let Some(n) = v.parse::<f64>().ok().and_then(serde_json::Number::from_f64) {
                                    on_commit.run(serde_json::Value::Number(n));
                                }
                            }
                            "Escape" => on_cancel.run(()),
                            _ => {}
                        }
                    }
                    on:blur=move |_| {
                        let v = val.get();
                        if v.is_empty() {
                            on_commit.run(serde_json::Value::Null);
                        } else if let Some(n) = v.parse::<f64>().ok().and_then(serde_json::Number::from_f64) {
                            on_commit.run(serde_json::Value::Number(n));
                        }
                    }
                    node_ref=auto_focus_ref()
                />
            }
            .into_any()
        }
        s if s.contains("TIMESTAMP") || s == "DATE" || s == "TIME" => {
            let input_type = if s == "DATE" {
                "date"
            } else if s == "TIME" || s == "TIME WITHOUT TIME ZONE" || s == "TIME WITH TIME ZONE" {
                "time"
            } else {
                "datetime-local"
            };
            let initial = match &value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => String::new(),
                _ => value.to_string(),
            };
            let (val, set_val) = signal(initial);
            view! {
                <input
                    type=input_type
                    class="w-full bg-white dark:bg-zinc-900 text-xs font-mono text-gray-900 dark:text-neutral-50 px-1 py-0 border-0 outline-none"
                    prop:value=move || val.get()
                    on:input=move |ev| set_val.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        match ev.key().as_str() {
                            "Enter" | "Tab" => {
                                ev.prevent_default();
                                let v = val.get();
                                if v.is_empty() {
                                    on_commit.run(serde_json::Value::Null);
                                } else {
                                    on_commit.run(serde_json::Value::String(v));
                                }
                            }
                            "Escape" => on_cancel.run(()),
                            _ => {}
                        }
                    }
                    on:blur=move |_| {
                        let v = val.get();
                        if v.is_empty() {
                            on_commit.run(serde_json::Value::Null);
                        } else {
                            on_commit.run(serde_json::Value::String(v));
                        }
                    }
                    node_ref=auto_focus_ref()
                />
            }
            .into_any()
        }
        // Default: text input for varchar, text, and all other types
        _ => {
            let initial = match &value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => String::new(),
                _ => value.to_string(),
            };
            let (val, set_val) = signal(initial);
            view! {
                <input
                    type="text"
                    class="w-full bg-white dark:bg-zinc-900 text-xs font-mono text-gray-900 dark:text-neutral-50 px-1 py-0 border-0 outline-none"
                    prop:value=move || val.get()
                    on:input=move |ev| set_val.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        match ev.key().as_str() {
                            "Enter" | "Tab" => {
                                ev.prevent_default();
                                let v = val.get();
                                if v.is_empty() {
                                    on_commit.run(serde_json::Value::Null);
                                } else {
                                    on_commit.run(serde_json::Value::String(v));
                                }
                            }
                            "Escape" => on_cancel.run(()),
                            _ => {}
                        }
                    }
                    on:blur=move |_| {
                        let v = val.get();
                        if v.is_empty() {
                            on_commit.run(serde_json::Value::Null);
                        } else {
                            on_commit.run(serde_json::Value::String(v));
                        }
                    }
                    node_ref=auto_focus_ref()
                />
            }
            .into_any()
        }
    }
}

/// Creates a NodeRef that auto-focuses the input element on mount.
fn auto_focus_ref() -> NodeRef<leptos::html::Input> {
    let node_ref = NodeRef::<leptos::html::Input>::new();
    Effect::new(move |_| {
        if let Some(el) = node_ref.get() {
            let _ = el.focus();
            let _ = el.select();
        }
    });
    node_ref
}
