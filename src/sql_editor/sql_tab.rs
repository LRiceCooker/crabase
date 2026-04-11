use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::shortcuts::use_save_trigger;
use crate::sql_editor::codemirror::{CodeMirrorEditor, CodeMirrorHandle};
use crate::sql_editor::sql_results::SqlResults;
use crate::sql_editor::sql_toolbar::SqlToolbar;
use crate::tauri;

/// Full SQL editor tab: toolbar + editor + results pane.
#[component]
pub fn SqlTab(
    /// Query name for saving. Auto-assigned as "Untitled-N" by the tab system.
    #[prop(default = String::new())]
    query_name: String,
    /// Callback to notify parent of dirty state changes.
    #[prop(optional)]
    on_dirty_change: Option<Callback<bool>>,
) -> impl IntoView {
    let (cm_handle, set_cm_handle) = signal(Option::<CodeMirrorHandle>::None);
    let (running, set_running) = signal(false);
    let (result, set_result) = signal(Option::<Result<Vec<tauri::StatementResult>, String>>::None);
    let (is_dirty, set_is_dirty) = signal(false);
    let name = RwSignal::new(query_name.clone());
    let (is_saved, set_is_saved) = signal(!query_name.is_empty());

    // Track dirty state from editor changes
    let on_change = Callback::new(move |_: String| {
        if !is_dirty.get_untracked() {
            set_is_dirty.set(true);
            if let Some(cb) = on_dirty_change {
                cb.run(true);
            }
        }
    });

    // Auto-focus the editor and load schema for autocomplete once mounted
    Effect::new(move |_| {
        if let Some(handle) = cm_handle.get() {
            // Schedule focus after a micro-delay to ensure DOM is ready
            let handle_clone = handle;
            let cb = wasm_bindgen::closure::Closure::once(move || {
                handle_clone.focus();
            });
            let _ = web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                10,
            );
            cb.forget();

            // Fetch table names and columns for autocomplete (schema-aware)
            spawn_local(async move {
                let Ok(tables) = tauri::list_tables().await else {
                    return;
                };
                if tables.is_empty() {
                    return;
                }
                let Ok(cols) = tauri::get_columns_for_autocomplete(&tables).await else {
                    return;
                };
                // Check if active schema is not public — if so, prefix table names
                let schema_prefix = match tauri::get_connection_info().await {
                    Ok(info) if info.schema != "public" => Some(info.schema),
                    _ => None,
                };
                let autocomplete_schema: std::collections::HashMap<String, Vec<String>> = if let Some(prefix) = schema_prefix {
                    cols.into_iter()
                        .map(|(table, columns)| (format!("{}.{}", prefix, table), columns))
                        .collect()
                } else {
                    cols
                };
                handle.set_schema(&autocomplete_schema);
            });
        }
    });

    let do_save = Callback::new(move |_: ()| {
        let Some(handle) = cm_handle.get_untracked() else {
            return;
        };
        let sql = handle.get_content();
        let query_name = name.get_untracked();
        if query_name.is_empty() {
            return;
        }

        let already_saved = is_saved.get_untracked();
        spawn_local(async move {
            let res = if already_saved {
                tauri::update_query(&query_name, &sql).await
            } else {
                tauri::save_query(&query_name, &sql).await
            };
            match res {
                Ok(()) => {
                    set_is_dirty.set(false);
                    set_is_saved.set(true);
                    if let Some(cb) = on_dirty_change {
                        cb.run(false);
                    }
                }
                Err(_e) => {
                    // Error saving — silently ignore for now
                }
            }
        });
    });

    // Listen for global save trigger (Cmd+S)
    {
        let save_trigger = use_save_trigger();
        let counter = save_trigger.counter();
        Effect::new(move |prev: Option<u64>| {
            let current = counter.get();
            if let Some(prev_val) = prev {
                if current != prev_val && is_dirty.get_untracked() {
                    do_save.run(());
                }
            }
            current
        });
    }

    let on_run = Callback::new(move |_: ()| {
        let Some(handle) = cm_handle.get_untracked() else {
            return;
        };
        let query = handle.get_content();
        if query.trim().is_empty() {
            return;
        }
        set_running.set(true);
        set_result.set(None);

        spawn_local(async move {
            let res = tauri::execute_query_multi(&query).await;
            set_result.set(Some(res));
            set_running.set(false);
        });
    });

    let dirty_signal: Signal<bool> = Signal::derive(move || is_dirty.get());

    // Resizable split: editor fraction (0.0 to 1.0). Default: 60% editor, 40% results.
    let editor_fraction = RwSignal::new(0.6_f64);
    let dragging = RwSignal::new(false);
    let container_ref = NodeRef::<leptos::html::Div>::new();

    // Mouse handlers for drag resize
    {
        let on_move = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |ev: web_sys::MouseEvent| {
                if !dragging.get_untracked() {
                    return;
                }
                ev.prevent_default();
                if let Some(el) = container_ref.get_untracked() {
                    let el: &web_sys::Element = el.as_ref();
                    let rect = el.get_bounding_client_rect();
                    let y = ev.client_y() as f64 - rect.top();
                    let h = rect.height();
                    if h > 0.0 {
                        let frac = (y / h).clamp(0.15, 0.85);
                        editor_fraction.set(frac);
                    }
                }
            },
        );
        let on_up = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |_: web_sys::MouseEvent| {
                dragging.set(false);
            },
        );
        let win = web_sys::window().unwrap();
        let doc = win.document().unwrap();
        doc.add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref())
            .unwrap();
        doc.add_event_listener_with_callback("mouseup", on_up.as_ref().unchecked_ref())
            .unwrap();
        on_move.forget();
        on_up.forget();
    }

    view! {
        <div class="flex flex-col h-full">
            <SqlToolbar
                on_run=on_run
                running=running
                on_save=do_save
                is_dirty=dirty_signal
            />
            <div node_ref=container_ref class="flex flex-col flex-1 overflow-hidden">
                <div style=move || format!("flex: 0 0 {}%; overflow: hidden; display: flex; flex-direction: column;", editor_fraction.get() * 100.0)>
                    <CodeMirrorEditor
                        language="sql".to_string()
                        placeholder="Write your SQL query here...".to_string()
                        on_change=on_change
                        handle=set_cm_handle
                    />
                </div>
                // Drag handle
                <div
                    class="h-1.5 cursor-row-resize bg-gray-100 dark:bg-zinc-800 hover:bg-indigo-200 dark:hover:bg-indigo-500/30 transition-colors duration-100 shrink-0 flex items-center justify-center"
                    on:mousedown=move |ev| {
                        ev.prevent_default();
                        dragging.set(true);
                    }
                >
                    <div class="w-8 h-0.5 rounded-full bg-gray-300 dark:bg-zinc-600"></div>
                </div>
                <div style=move || format!("flex: 0 0 {}%; overflow: hidden; display: flex; flex-direction: column;", (1.0 - editor_fraction.get()) * 100.0)>
                    <SqlResults result=result />
                </div>
            </div>
        </div>
    }
}
