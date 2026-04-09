use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::icons::IconLoader;
use crate::table_view::data_table::DataTable;
use crate::tauri;

#[component]
pub fn TableView(table_name: Memo<Option<String>>) -> impl IntoView {
    let (data, set_data) = signal(Option::<tauri::TableData>::None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (loaded_table, set_loaded_table) = signal(Option::<String>::None);

    // Reactively fetch data when table_name changes
    Effect::new(move |_| {
        let name = table_name.get();
        let current = loaded_table.get();

        if name == current {
            return;
        }

        set_loaded_table.set(name.clone());

        if let Some(name) = name {
            set_loading.set(true);
            set_error.set(None);

            spawn_local(async move {
                match tauri::get_table_data(&name, 1, 50).await {
                    Ok(td) => {
                        set_data.set(Some(td));
                    }
                    Err(e) => {
                        set_error.set(Some(e));
                        set_data.set(None);
                    }
                }
                set_loading.set(false);
            });
        } else {
            set_data.set(None);
            set_loading.set(false);
        }
    });

    view! {
        <div class="flex flex-col h-full">
            {move || {
                if loading.get() {
                    view! {
                        <div class="flex items-center justify-center h-full text-gray-400">
                            <IconLoader class="w-5 h-5 animate-spin" />
                        </div>
                    }.into_any()
                } else if let Some(err) = error.get() {
                    view! {
                        <div class="flex items-center justify-center h-full">
                            <p class="text-[13px] text-red-500">{err}</p>
                        </div>
                    }.into_any()
                } else if let Some(td) = data.get() {
                    view! {
                        <DataTable columns=td.columns rows=td.rows />
                    }.into_any()
                } else {
                    view! {
                        <div class="flex items-center justify-center h-full text-gray-400">
                            <p class="text-[13px]">"Select a table to get started"</p>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
