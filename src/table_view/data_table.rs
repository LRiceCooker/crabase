use leptos::prelude::*;

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
    rows: Vec<Vec<serde_json::Value>>,
) -> impl IntoView {
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
                    {rows.into_iter().map(|row| {
                        view! {
                            <tr class="hover:bg-gray-50">
                                {row.into_iter().map(|cell| {
                                    let (text, is_null) = format_cell(&cell);
                                    let class = if is_null {
                                        "px-3 py-1.5 border-b border-gray-100 border-r border-gray-100 truncate max-w-[300px] text-gray-300 italic"
                                    } else {
                                        "px-3 py-1.5 border-b border-gray-100 border-r border-gray-100 truncate max-w-[300px]"
                                    };
                                    let title = text.clone();
                                    view! {
                                        <td class=class title=title>
                                            {text}
                                        </td>
                                    }
                                }).collect::<Vec<_>>()}
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    }
}
