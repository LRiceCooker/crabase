use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::icons::IconX;
use crate::shortcuts::{use_shortcuts, KeyBinding, ShortcutAction};

/// A clickable input that listens for a key combination to bind to a shortcut action.
///
/// Displays the current binding. When clicked, enters "listening" mode and captures
/// the next key combination (with at least one modifier). Press Escape to cancel,
/// Backspace to clear the binding.
#[component]
pub fn ShortcutInput(action: ShortcutAction) -> impl IntoView {
    let sc = use_shortcuts();
    let (listening, set_listening) = signal(false);

    // Reactive display of the current binding
    let display = move || {
        let bindings = sc.bindings().get();
        bindings
            .get(&action)
            .map(|b| b.display())
            .unwrap_or_else(|| "None".to_string())
    };

    let has_binding = move || {
        let bindings = sc.bindings().get();
        bindings.contains_key(&action)
    };

    // When the user clicks the input, start listening for a key combo
    let on_click = move |_| {
        set_listening.set(true);
    };

    // Clear binding via the X button
    let on_clear = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        sc.clear_binding(action);
    };

    // Attach a global keydown listener once (matches main_layout.rs pattern).
    // It checks the `listening` signal each time and only acts when active.
    {
        let closure =
            Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |ev: web_sys::KeyboardEvent| {
                if !listening.get_untracked() {
                    return;
                }

                ev.prevent_default();
                ev.stop_propagation();

                let code = ev.code();

                // Escape cancels listening
                if code == "Escape" {
                    set_listening.set(false);
                    return;
                }

                // Backspace/Delete clears the binding
                if code == "Backspace" || code == "Delete" {
                    sc.clear_binding(action);
                    set_listening.set(false);
                    return;
                }

                // Ignore bare modifier keys (wait for the actual key)
                if is_modifier_code(&code) {
                    return;
                }

                let cmd_or_ctrl = ev.meta_key() || ev.ctrl_key();
                let shift = ev.shift_key();
                let alt = ev.alt_key();

                // Require at least one modifier for a valid shortcut
                if !cmd_or_ctrl && !shift && !alt {
                    return;
                }

                let binding = KeyBinding::new(cmd_or_ctrl, shift, alt, &code);
                sc.set_binding(action, binding);
                set_listening.set(false);
            });

        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    view! {
        <div
            class=move || {
                let base = "relative inline-flex items-center gap-1.5 min-w-[120px] h-[30px] px-2.5 rounded-md text-[13px] font-mono cursor-pointer select-none transition-colors duration-100 border";
                if listening.get() {
                    format!("{base} bg-indigo-50 dark:bg-indigo-500/10 border-indigo-500 dark:border-indigo-400 ring-2 ring-indigo-500/20 dark:ring-indigo-500/40 text-indigo-600 dark:text-indigo-300")
                } else {
                    format!("{base} bg-white dark:bg-zinc-900 border-gray-200 dark:border-zinc-800 text-gray-900 dark:text-neutral-50 hover:border-gray-300 dark:hover:border-zinc-700")
                }
            }
            on:click=on_click
            tabindex=0
        >
            <span class="flex-1 text-center">
                {move || {
                    if listening.get() {
                        "Press keys...".to_string()
                    } else {
                        display()
                    }
                }}
            </span>
            // Clear button (only when not listening and has a binding)
            <Show when=move || !listening.get() && has_binding()>
                <button
                    class="text-gray-400 dark:text-zinc-500 hover:text-gray-700 dark:hover:text-zinc-300 p-0.5 rounded transition-colors duration-100"
                    on:click=on_clear
                    title="Clear shortcut"
                >
                    <IconX class="w-3 h-3" />
                </button>
            </Show>
        </div>
    }
}

/// Returns true if the code represents a modifier-only key.
fn is_modifier_code(code: &str) -> bool {
    matches!(
        code,
        "MetaLeft"
            | "MetaRight"
            | "ControlLeft"
            | "ControlRight"
            | "ShiftLeft"
            | "ShiftRight"
            | "AltLeft"
            | "AltRight"
    )
}
