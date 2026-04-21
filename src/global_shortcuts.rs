use leptos::prelude::GetUntracked;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::overlay::{self, ActiveOverlay};
use crate::shortcuts::{self, ShortcutAction, use_save_trigger};

/// Registers global keyboard shortcuts on the window.
/// Must be called inside a Leptos reactive context (within a component).
/// Uses capture phase to intercept shortcuts before CodeMirror.
pub fn setup_global_shortcuts() {
    let overlay_ctx = overlay::use_overlay();
    let sc = shortcuts::use_shortcuts();
    let save_trigger = use_save_trigger();

    let closure = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
        move |ev: web_sys::KeyboardEvent| {
            if ev.key() == "Escape"
                && overlay_ctx.active.get_untracked() != ActiveOverlay::None
            {
                ev.prevent_default();
                ev.stop_propagation();
                overlay_ctx.close();
            } else if sc.matches(ShortcutAction::CommandPalette, &ev) {
                ev.prevent_default();
                ev.stop_propagation();
                blur_active_element();
                overlay_ctx.open(ActiveOverlay::CommandPalette);
            } else if sc.matches(ShortcutAction::TableFinder, &ev) {
                ev.prevent_default();
                ev.stop_propagation();
                blur_active_element();
                overlay_ctx.open(ActiveOverlay::TableFinder);
            } else if sc.matches(ShortcutAction::Save, &ev) {
                ev.prevent_default();
                save_trigger.request();
            } else if (ev.meta_key() || ev.ctrl_key()) && ev.shift_key() && ev.code() == "KeyN" {
                ev.prevent_default();
                ev.stop_propagation();
                wasm_bindgen_futures::spawn_local(async {
                    let _ = crate::tauri::open_new_window().await;
                });
            }
        },
    );

    // Use capture phase (true) so we intercept shortcuts BEFORE CodeMirror
    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback_and_bool(
            "keydown",
            closure.as_ref().unchecked_ref(),
            true,
        )
        .unwrap();
    closure.forget(); // App-lifetime: MainLayout lives for the entire session
}

/// Blur the currently focused element (e.g. CodeMirror) so overlay inputs get focus.
fn blur_active_element() {
    if let Some(el) = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .active_element()
    {
        if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
            let _ = html_el.blur();
        }
    }
}
