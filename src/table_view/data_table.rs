use leptos::prelude::*;

use crate::icons::IconTrash2;
use crate::table_view::cell_editor::{CellEdit, CellEditor};
use crate::table_view::change_tracker::ChangeTracker;
use crate::table_view::json_editor::JsonEditRequest;
use crate::tauri::ColumnInfo;

/// Format a cell value for display.
fn format_cell(value: &serde_json::Value) -> (String, bool) {
    match value {
        serde_json::Value::Null => ("NULL".to_string(), true),
        serde_json::Value::Bool(b) => (b.to_string(), false),
        serde_json::Value::Number(n) => (n.to_string(), false),
        serde_json::Value::String(s) => (s.clone(), false),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            (serde_json::to_string(value).unwrap_or_default(), false)
        }
    }
}

fn is_json_type(data_type: &str) -> bool {
    let dt = data_type.to_uppercase();
    dt == "JSON" || dt == "JSONB"
}

#[component]
pub fn DataTable(
    columns: Vec<ColumnInfo>,
    rows: RwSignal<Vec<Vec<serde_json::Value>>>,
    changes: ChangeTracker,
    on_cell_edit: Callback<CellEdit>,
    on_json_edit: Callback<JsonEditRequest>,
    on_delete_row: Callback<usize>,
) -> impl IntoView {
    // Track which cell is being edited: (row_idx, col_idx)
    let (editing_cell, set_editing_cell) = signal(Option::<(usize, usize)>::None);

    // Clone columns for use in the view closures
    let columns_for_types = columns.clone();

    view! {
        <div class="overflow-auto flex-1">
            <table class="w-full text-xs font-mono">
                <thead>
                    <tr class="bg-gray-50 border-b border-gray-200 sticky top-0 z-10">
                        {columns.iter().map(|col| {
                            let name = col.name.clone();
                            let data_type = col.data_type.clone();
                            view! {
                                <th class="px-3 py-2 text-left text-[11px] font-medium uppercase tracking-wider text-gray-500 border-r border-gray-100 select-none whitespace-nowrap">
                                    <div class="flex flex-col gap-0.5">
                                        <span>{name}</span>
                                        <span class="text-[10px] font-normal text-gray-400 normal-case tracking-normal">{data_type}</span>
                                    </div>
                                </th>
                            }
                        }).collect::<Vec<_>>()}
                        <th class="px-2 py-2 text-left text-[11px] font-medium text-gray-500 border-r border-gray-100 select-none w-8"></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let current_rows = rows.get();
                        let active = editing_cell.get();
                        let col_types = columns_for_types.clone();

                        current_rows.into_iter().enumerate().map(|(row_idx, row)| {
                            let col_types = col_types.clone();
                            let row_class = if changes.is_row_deleted(row_idx) {
                                "bg-red-50 border-l-2 border-l-red-500 line-through opacity-60"
                            } else if changes.is_row_added(row_idx) {
                                "bg-emerald-50 border-l-2 border-emerald-500"
                            } else if changes.is_row_modified(row_idx) {
                                "bg-amber-50 border-l-2 border-amber-500"
                            } else {
                                "hover:bg-gray-50"
                            };
                            view! {
                                <tr class=row_class>
                                    {row.into_iter().enumerate().map(|(col_idx, cell)| {
                                        let is_editing = active == Some((row_idx, col_idx));
                                        let data_type = col_types.get(col_idx).map(|c| c.data_type.clone()).unwrap_or_default();
                                        let is_json = is_json_type(&data_type);

                                        if is_editing && !is_json {
                                            let dt = data_type.clone();
                                            let cell_val = cell.clone();
                                            view! {
                                                <td class="px-3 py-1.5 border-b border-gray-100 border-r border-gray-100 ring-2 ring-indigo-500/30 bg-white max-w-[300px]">
                                                    <CellEditor
                                                        data_type=dt
                                                        value=cell_val
                                                        on_commit=Callback::new(move |new_val: serde_json::Value| {
                                                            set_editing_cell.set(None);
                                                            on_cell_edit.run(CellEdit {
                                                                row: row_idx,
                                                                col: col_idx,
                                                                value: new_val,
                                                            });
                                                        })
                                                        on_cancel=Callback::new(move |_| {
                                                            set_editing_cell.set(None);
                                                        })
                                                    />
                                                </td>
                                            }.into_any()
                                        } else {
                                            let (text, is_null) = format_cell(&cell);
                                            let cell_modified = changes.is_cell_modified(row_idx, col_idx);
                                            let class = if is_null && cell_modified {
                                                "px-3 py-1.5 border-b border-gray-100 border-r border-gray-100 truncate max-w-[300px] text-gray-300 italic cursor-pointer bg-amber-100/50"
                                            } else if is_null {
                                                "px-3 py-1.5 border-b border-gray-100 border-r border-gray-100 truncate max-w-[300px] text-gray-300 italic cursor-pointer"
                                            } else if cell_modified {
                                                "px-3 py-1.5 border-b border-gray-100 border-r border-gray-100 truncate max-w-[300px] cursor-pointer bg-amber-100/50"
                                            } else {
                                                "px-3 py-1.5 border-b border-gray-100 border-r border-gray-100 truncate max-w-[300px] cursor-pointer"
                                            };
                                            let title = text.clone();
                                            let cell_for_json = cell.clone();
                                            view! {
                                                <td
                                                    class=class
                                                    title=title
                                                    on:click=move |_| {
                                                        if is_json {
                                                            on_json_edit.run(JsonEditRequest {
                                                                row: row_idx,
                                                                col: col_idx,
                                                                value: cell_for_json.clone(),
                                                            });
                                                        } else {
                                                            set_editing_cell.set(Some((row_idx, col_idx)));
                                                        }
                                                    }
                                                >
                                                    {text}
                                                </td>
                                            }.into_any()
                                        }
                                    }).collect::<Vec<_>>()}
                                    <td class="px-2 py-1.5 border-b border-gray-100 text-center w-8">
                                        {if !changes.is_row_deleted(row_idx) {
                                            Some(view! {
                                                <button
                                                    class="text-gray-300 hover:text-red-500 p-0.5 rounded transition-colors duration-100"
                                                    title="Delete row"
                                                    on:click=move |_| on_delete_row.run(row_idx)
                                                >
                                                    <IconTrash2 class="w-3.5 h-3.5" />
                                                </button>
                                            })
                                        } else {
                                            None
                                        }}
                                    </td>
                                </tr>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </tbody>
            </table>
        </div>
    }
}
