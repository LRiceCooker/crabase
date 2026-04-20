use leptos::prelude::*;

use super::{auto_focus_input_ref, INPUT_CLASS};

/// Inline editor for UUID columns. Text input with a generate-new-UUID button.
#[component]
pub fn UuidEditor(
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

    let commit_clone = commit;
    view! {
        <div class="flex items-center gap-1 w-full">
            <input
                type="text"
                class=INPUT_CLASS
                placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
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
            <button
                class="shrink-0 text-[10px] text-indigo-500 dark:text-indigo-400 hover:text-indigo-600 dark:hover:text-indigo-300 px-1 py-0.5 rounded hover:bg-indigo-50 dark:hover:bg-indigo-500/10 transition-colors duration-100"
                title="Generate UUID"
                on:click=move |ev| {
                    ev.prevent_default();
                    ev.stop_propagation();
                    let uuid = generate_uuid_v4();
                    set_val.set(uuid);
                }
            >
                "Gen"
            </button>
        </div>
    }
}

/// Generate a v4 UUID using crypto.getRandomValues.
fn generate_uuid_v4() -> String {
    let window = web_sys::window().expect("no window");
    let crypto = window.crypto().expect("no crypto");
    let mut buf = [0u8; 16];
    crypto
        .get_random_values_with_u8_array(&mut buf)
        .expect("get_random_values failed");
    // Set version (4) and variant (RFC 4122)
    buf[6] = (buf[6] & 0x0f) | 0x40;
    buf[8] = (buf[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        buf[0], buf[1], buf[2], buf[3],
        buf[4], buf[5],
        buf[6], buf[7],
        buf[8], buf[9],
        buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
    )
}
