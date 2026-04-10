use leptos::prelude::*;

use super::INPUT_CLASS;

/// Inline editor for range types (int4range, int8range, numrange, tsrange, etc.).
/// Shows two inputs (lower/upper) with inclusivity toggles.
#[component]
pub fn RangeEditor(
    value: serde_json::Value,
    on_commit: Callback<serde_json::Value>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    // Parse range string like "[1,10)" or "(,5]" or "empty"
    let raw = match &value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        _ => value.to_string(),
    };

    let (lower_inc, upper_inc, lower_val, upper_val) = parse_range(&raw);
    let (lower, set_lower) = signal(lower_val);
    let (upper, set_upper) = signal(upper_val);
    let (lower_inclusive, set_lower_inclusive) = signal(lower_inc);
    let (upper_inclusive, set_upper_inclusive) = signal(upper_inc);

    let build_range = move || -> String {
        let lb = if lower_inclusive.get() { "[" } else { "(" };
        let ub = if upper_inclusive.get() { "]" } else { ")" };
        format!("{}{},{}{}", lb, lower.get(), upper.get(), ub)
    };

    let commit = move || {
        let l = lower.get();
        let u = upper.get();
        if l.is_empty() && u.is_empty() {
            on_commit.run(serde_json::Value::Null);
        } else {
            on_commit.run(serde_json::Value::String(build_range()));
        }
    };

    let commit_clone = commit.clone();
    let commit_clone2 = commit.clone();
    let commit_clone3 = commit.clone();
    let commit_clone4 = commit.clone();
    view! {
        <div class="flex items-center gap-0.5 w-full text-xs font-mono">
            <button
                class="shrink-0 px-1 py-0 text-[10px] rounded border border-gray-200 dark:border-zinc-700 bg-gray-50 dark:bg-zinc-800 text-gray-600 dark:text-zinc-300 hover:bg-gray-100 dark:hover:bg-zinc-700"
                title=move || if lower_inclusive.get() { "Inclusive [" } else { "Exclusive (" }
                on:click=move |ev| {
                    ev.prevent_default();
                    ev.stop_propagation();
                    set_lower_inclusive.set(!lower_inclusive.get());
                }
            >
                {move || if lower_inclusive.get() { "[" } else { "(" }}
            </button>
            <input
                type="text"
                class=INPUT_CLASS
                placeholder="lower"
                prop:value=move || lower.get()
                on:input=move |ev| set_lower.set(event_target_value(&ev))
                on:keydown=move |ev| {
                    match ev.key().as_str() {
                        "Enter" | "Tab" => { ev.prevent_default(); commit(); }
                        "Escape" => on_cancel.run(()),
                        _ => {}
                    }
                }
                on:blur=move |_| commit_clone()
            />
            <span class="text-gray-400 dark:text-zinc-500">","</span>
            <input
                type="text"
                class=INPUT_CLASS
                placeholder="upper"
                prop:value=move || upper.get()
                on:input=move |ev| set_upper.set(event_target_value(&ev))
                on:keydown=move |ev| {
                    match ev.key().as_str() {
                        "Enter" | "Tab" => { ev.prevent_default(); commit_clone2(); }
                        "Escape" => on_cancel.run(()),
                        _ => {}
                    }
                }
                on:blur=move |_| commit_clone3()
            />
            <button
                class="shrink-0 px-1 py-0 text-[10px] rounded border border-gray-200 dark:border-zinc-700 bg-gray-50 dark:bg-zinc-800 text-gray-600 dark:text-zinc-300 hover:bg-gray-100 dark:hover:bg-zinc-700"
                title=move || if upper_inclusive.get() { "Inclusive ]" } else { "Exclusive )" }
                on:click=move |ev| {
                    ev.prevent_default();
                    ev.stop_propagation();
                    set_upper_inclusive.set(!upper_inclusive.get());
                    commit_clone4();
                }
            >
                {move || if upper_inclusive.get() { "]" } else { ")" }}
            </button>
        </div>
    }
}

/// Parse a Postgres range string like "[1,10)" into (lower_inclusive, upper_inclusive, lower, upper).
fn parse_range(s: &str) -> (bool, bool, String, String) {
    let s = s.trim();
    if s.is_empty() || s == "empty" {
        return (true, false, String::new(), String::new());
    }

    let lower_inc = s.starts_with('[');
    let upper_inc = s.ends_with(']');

    // Strip the brackets
    let inner = &s[1..s.len().saturating_sub(1)];
    let parts: Vec<&str> = inner.splitn(2, ',').collect();
    let lower = parts.first().unwrap_or(&"").trim().to_string();
    let upper = parts.get(1).unwrap_or(&"").trim().to_string();

    (lower_inc, upper_inc, lower, upper)
}
