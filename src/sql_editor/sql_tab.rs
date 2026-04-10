use leptos::prelude::*;
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
    let (result, set_result) = signal(Option::<Result<tauri::QueryResult, String>>::None);
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
            handle.focus();
            // Fetch table names and columns for autocomplete
            spawn_local(async move {
                let Ok(tables) = tauri::list_tables().await else {
                    return;
                };
                if tables.is_empty() {
                    return;
                }
                let Ok(schema) = tauri::get_columns_for_autocomplete(&tables).await else {
                    return;
                };
                handle.set_schema(&schema);
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
            let res = tauri::execute_query(&query).await;
            set_result.set(Some(res));
            set_running.set(false);
        });
    });

    let dirty_signal: Signal<bool> = Signal::derive(move || is_dirty.get());

    view! {
        <div class="flex flex-col h-full">
            <SqlToolbar
                on_run=on_run
                running=running
                on_save=do_save
                is_dirty=dirty_signal
            />
            <div class="flex flex-col flex-1 overflow-hidden">
                <CodeMirrorEditor
                    language="sql".to_string()
                    placeholder="Write your SQL query here...".to_string()
                    on_change=on_change
                    handle=set_cm_handle
                />
                <SqlResults result=result />
            </div>
        </div>
    }
}
