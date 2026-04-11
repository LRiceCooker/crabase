use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Inline-editable tab title for SQL editor tabs.
/// Double-click to enter edit mode. Enter or blur to save. Escape to revert.
#[component]
pub fn TabTitle(
    title: String,
    /// If true, double-clicking the title enters edit mode.
    editable: bool,
    /// Called with the new title when the user confirms the edit.
    #[prop(optional)]
    on_rename: Option<Callback<String>>,
) -> impl IntoView {
    let (editing, set_editing) = signal(false);
    let (edit_value, set_edit_value) = signal(title.clone());
    let original = RwSignal::new(title.clone());
    let input_ref = NodeRef::<leptos::html::Input>::new();

    let start_editing = move |ev: web_sys::MouseEvent| {
        if !editable {
            return;
        }
        ev.stop_propagation();
        set_edit_value.set(original.get_untracked());
        set_editing.set(true);
        request_animation_frame(move || {
            if let Some(el) = input_ref.get() {
                let el: &web_sys::HtmlInputElement = el.as_ref();
                let _ = el.focus();
                let _ = el.select();
            }
        });
    };

    let do_confirm = move || {
        let new_name = edit_value.get_untracked();
        set_editing.set(false);
        let orig = original.get_untracked();
        if !new_name.trim().is_empty() && new_name != orig {
            original.set(new_name.clone());
            if let Some(cb) = on_rename {
                cb.run(new_name);
            }
        }
    };

    let do_revert = move || {
        set_editing.set(false);
        set_edit_value.set(original.get_untracked());
    };

    view! {
        {move || {
            if editing.get() {
                view! {
                    <input
                        class="bg-transparent text-[13px] text-gray-900 dark:text-neutral-50 border border-indigo-500 rounded px-1 py-0 w-24 focus:outline-none"
                        prop:value=move || edit_value.get()
                        on:input=move |ev| set_edit_value.set(event_target_value(&ev))
                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                            match ev.key().as_str() {
                                "Enter" => { ev.prevent_default(); do_confirm(); }
                                "Escape" => { ev.prevent_default(); do_revert(); }
                                _ => {}
                            }
                        }
                        on:blur=move |_| do_confirm()
                        on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                        node_ref=input_ref
                    />
                }.into_any()
            } else {
                let title_display = original.get();
                let cursor_class = if editable { "cursor-text" } else { "" };
                view! {
                    <span
                        class=format!("truncate max-w-[120px] {}", cursor_class)
                        on:click=start_editing
                    >
                        {title_display}
                    </span>
                }.into_any()
            }
        }}
    }
}

fn request_animation_frame(f: impl FnOnce() + 'static) {
    let closure = wasm_bindgen::closure::Closure::once(f);
    web_sys::window()
        .unwrap()
        .request_animation_frame(closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();
}
