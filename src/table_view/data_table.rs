use leptos::prelude::*;

use crate::table_view::cell_editor::{CellEdit, CellEditor};
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

#[component]
pub fn DataTable(
    columns: Vec<ColumnInfo>,
    rows: RwSignal<Vec<Vec<serde_json::Value>>>,
    on_cell_edit: Callback<CellEdit>,
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
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let current_rows = rows.get();
                        let active = editing_cell.get();
                        let col_types = columns_for_types.clone();

                        current_rows.into_iter().enumerate().map(|(row_idx, row)| {
                            let col_types = col_types.clone();
                            view! {
                                <tr class="hover:bg-gray-50">
                                    {row.into_iter().enumerate().map(|(col_idx, cell)| {
                                        let is_editing = active == Some((row_idx, col_idx));
                                        let data_type = col_types.get(col_idx).map(|c| c.data_type.clone()).unwrap_or_default();

                                        if is_editing {
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
                                            let class = if is_null {
                                                "px-3 py-1.5 border-b border-gray-100 border-r border-gray-100 truncate max-w-[300px] text-gray-300 italic cursor-pointer"
                                            } else {
                                                "px-3 py-1.5 border-b border-gray-100 border-r border-gray-100 truncate max-w-[300px] cursor-pointer"
                                            };
                                            let title = text.clone();
                                            view! {
                                                <td
                                                    class=class
                                                    title=title
                                                    on:click=move |_| {
                                                        set_editing_cell.set(Some((row_idx, col_idx)));
                                                    }
                                                >
                                                    {text}
                                                </td>
                                            }.into_any()
                                        }
                                    }).collect::<Vec<_>>()}
                                </tr>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </tbody>
            </table>
        </div>
    }
}
