use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::theme;

// JS bridge bindings to window.__codemirror
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["__codemirror"], js_name = "create")]
    fn cm_create(parent: &web_sys::HtmlElement, opts: &JsValue) -> u32;

    #[wasm_bindgen(js_namespace = ["__codemirror"], js_name = "destroy")]
    fn cm_destroy(id: u32);

    #[wasm_bindgen(js_namespace = ["__codemirror"], js_name = "getContent")]
    fn cm_get_content(id: u32) -> String;

    #[wasm_bindgen(js_namespace = ["__codemirror"], js_name = "setContent")]
    fn cm_set_content(id: u32, content: &str);

    #[wasm_bindgen(js_namespace = ["__codemirror"], js_name = "focus")]
    fn cm_focus(id: u32);

    #[wasm_bindgen(js_namespace = ["__codemirror"], js_name = "isDirty")]
    fn cm_is_dirty(id: u32) -> bool;

    #[wasm_bindgen(js_namespace = ["__codemirror"], js_name = "markClean")]
    fn cm_mark_clean(id: u32);

    #[wasm_bindgen(js_namespace = ["__codemirror"], js_name = "onChange")]
    fn cm_on_change(id: u32, callback: &JsValue);

    #[wasm_bindgen(js_namespace = ["__codemirror"], js_name = "setTheme")]
    fn cm_set_theme(id: u32, is_dark: bool);

    #[wasm_bindgen(js_namespace = ["__codemirror"], js_name = "setSchema")]
    fn cm_set_schema(id: u32, schema: &JsValue);
}

/// Handle to a mounted CodeMirror instance. Provides methods to interact with the editor.
#[derive(Clone, Copy)]
pub struct CodeMirrorHandle {
    id: RwSignal<Option<u32>>,
    dirty: RwSignal<bool>,
}

impl CodeMirrorHandle {
    /// Get the current editor content.
    pub fn get_content(&self) -> String {
        match self.id.get_untracked() {
            Some(id) => cm_get_content(id),
            None => String::new(),
        }
    }

    /// Replace all editor content.
    pub fn set_content(&self, content: &str) {
        if let Some(id) = self.id.get_untracked() {
            cm_set_content(id, content);
        }
    }

    /// Focus the editor.
    pub fn focus(&self) {
        if let Some(id) = self.id.get_untracked() {
            cm_focus(id);
        }
    }

    /// Check if the editor has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty.get()
    }

    /// Mark the current content as the clean baseline.
    pub fn mark_clean(&self) {
        if let Some(id) = self.id.get_untracked() {
            cm_mark_clean(id);
            self.dirty.set(false);
        }
    }

    /// Reactive dirty signal (for UI binding).
    pub fn dirty_signal(&self) -> RwSignal<bool> {
        self.dirty
    }

    /// Set schema for SQL autocomplete (table → columns map).
    pub fn set_schema(&self, schema: &std::collections::HashMap<String, Vec<String>>) {
        if let Some(id) = self.id.get_untracked() {
            let js_obj = js_sys::Object::new();
            for (table, columns) in schema {
                let js_cols = js_sys::Array::new();
                for col in columns {
                    js_cols.push(&JsValue::from_str(col));
                }
                js_sys::Reflect::set(&js_obj, &JsValue::from_str(table), &js_cols).unwrap();
            }
            cm_set_schema(id, &js_obj.into());
        }
    }
}

/// CodeMirror 6 editor component integrated via JS interop.
///
/// Props:
/// - `initial_content`: starting text content
/// - `language`: "sql" (default) or "json"
/// - `read_only`: if true, editor is non-editable
/// - `placeholder`: placeholder text when empty
/// - `on_change`: callback fired on every content change
/// - `handle`: write signal to receive the CodeMirrorHandle for external control
#[component]
pub fn CodeMirrorEditor(
    #[prop(default = String::new())] initial_content: String,
    #[prop(default = "sql".to_string())] language: String,
    #[prop(default = false)] read_only: bool,
    #[prop(default = String::new())] placeholder: String,
    #[prop(optional)] on_change: Option<Callback<String>>,
    #[prop(optional)] handle: Option<WriteSignal<Option<CodeMirrorHandle>>>,
) -> impl IntoView {
    let container_ref = NodeRef::<leptos::html::Div>::new();
    let editor_id: RwSignal<Option<u32>> = RwSignal::new(None);
    let dirty = RwSignal::new(false);

    let theme_ctx = theme::use_theme();

    // Build the handle and expose it
    let cm_handle = CodeMirrorHandle { id: editor_id, dirty };
    if let Some(handle_setter) = handle {
        handle_setter.set(Some(cm_handle));
    }

    // Mount the editor once the container div is in the DOM
    let initial_content_clone = initial_content.clone();
    let language_clone = language.clone();
    let placeholder_clone = placeholder.clone();

    Effect::new(move |_| {
        let Some(el) = container_ref.get() else {
            return;
        };

        // Only mount once
        if editor_id.get_untracked().is_some() {
            return;
        }

        let el: &web_sys::HtmlElement = el.as_ref();
        let is_dark = theme_ctx.is_dark.get_untracked();

        // Build options object for JS bridge
        let opts = js_sys::Object::new();
        js_sys::Reflect::set(&opts, &"content".into(), &JsValue::from_str(&initial_content_clone)).unwrap();
        js_sys::Reflect::set(&opts, &"isDark".into(), &JsValue::from_bool(is_dark)).unwrap();
        js_sys::Reflect::set(&opts, &"language".into(), &JsValue::from_str(&language_clone)).unwrap();
        js_sys::Reflect::set(&opts, &"readOnly".into(), &JsValue::from_bool(read_only)).unwrap();
        js_sys::Reflect::set(&opts, &"placeholder".into(), &JsValue::from_str(&placeholder_clone)).unwrap();

        let id = cm_create(el, &opts);
        editor_id.set(Some(id));

        // Register change callback
        let on_change_clone = on_change;
        let change_closure = Closure::<dyn FnMut(String)>::new(move |new_content: String| {
            dirty.set(true);
            if let Some(ref cb) = on_change_clone {
                cb.run(new_content);
            }
        });
        cm_on_change(id, change_closure.as_ref().unchecked_ref());
        change_closure.forget();
    });

    // React to theme changes
    Effect::new(move |_| {
        let is_dark = theme_ctx.is_dark.get();
        if let Some(id) = editor_id.get_untracked() {
            cm_set_theme(id, is_dark);
        }
    });

    // Cleanup on unmount
    on_cleanup(move || {
        if let Some(id) = editor_id.get_untracked() {
            cm_destroy(id);
        }
    });

    view! {
        <div
            node_ref=container_ref
            class="flex-1 overflow-hidden"
        />
    }
}
