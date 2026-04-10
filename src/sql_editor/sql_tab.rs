use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::sql_editor::codemirror::{CodeMirrorEditor, CodeMirrorHandle};
use crate::sql_editor::sql_results::SqlResults;
use crate::sql_editor::sql_toolbar::SqlToolbar;
use crate::tauri;

/// Full SQL editor tab: toolbar + editor + results pane.
#[component]
pub fn SqlTab() -> impl IntoView {
    let (cm_handle, set_cm_handle) = signal(Option::<CodeMirrorHandle>::None);
    let (running, set_running) = signal(false);
    let (result, set_result) = signal(Option::<Result<tauri::QueryResult, String>>::None);

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

    view! {
        <div class="flex flex-col h-full">
            <SqlToolbar on_run=on_run running=running />
            <div class="flex flex-col flex-1 overflow-hidden">
                <CodeMirrorEditor
                    language="sql".to_string()
                    placeholder="Write your SQL query here...".to_string()
                    handle=set_cm_handle
                />
                <SqlResults result=result />
            </div>
        </div>
    }
}
