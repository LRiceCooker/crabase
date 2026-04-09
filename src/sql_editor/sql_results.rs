use leptos::prelude::*;

use crate::tauri::QueryResult;

#[component]
pub fn SqlResults(
    result: ReadSignal<Option<Result<QueryResult, String>>>,
) -> impl IntoView {
    view! {
        {move || {
            match result.get() {
                None => view! {
                    <div class="flex-1"></div>
                }.into_any(),
                Some(Ok(qr)) => {
                    if qr.columns.is_empty() && qr.rows.is_empty() {
                        view! {
                            <div class="flex-1 flex items-center justify-center text-gray-400 text-[13px]">
                                "Query executed successfully (no results)"
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="flex-1 overflow-auto border-t border-gray-200">
                                <table class="w-full text-xs font-mono">
                                    <thead>
                                        <tr class="bg-gray-50 border-b border-gray-200 sticky top-0 z-10">
                                            {qr.columns.iter().map(|col| {
                                                let name = col.clone();
                                                view! {
                                                    <th class="px-3 py-2 text-left text-[11px] font-medium uppercase tracking-wider text-gray-500 border-r border-gray-100 select-none whitespace-nowrap">
                                                        {name}
                                                    </th>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {qr.rows.iter().map(|row| {
                                            view! {
                                                <tr class="hover:bg-gray-50">
                                                    {row.iter().map(|cell| {
                                                        let (text, is_null) = format_value(cell);
                                                        let class = if is_null {
                                                            "px-3 py-1.5 border-b border-gray-100 border-r border-gray-100 truncate max-w-[300px] text-gray-300 italic"
                                                        } else {
                                                            "px-3 py-1.5 border-b border-gray-100 border-r border-gray-100 truncate max-w-[300px]"
                                                        };
                                                        let title = text.clone();
                                                        view! {
                                                            <td class=class title=title>{text}</td>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </tr>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </tbody>
                                </table>
                            </div>
                        }.into_any()
                    }
                }
                Some(Err(err)) => view! {
                    <div class="flex-1 bg-gray-900 text-red-400 font-mono text-xs p-3 overflow-y-auto border-t border-gray-200">
                        <pre class="whitespace-pre-wrap">{err}</pre>
                    </div>
                }.into_any(),
            }
        }}
    }
}

fn format_value(value: &serde_json::Value) -> (String, bool) {
    match value {
        serde_json::Value::Null => ("NULL".to_string(), true),
        serde_json::Value::Bool(b) => (b.to_string(), false),
        serde_json::Value::Number(n) => (n.to_string(), false),
        serde_json::Value::String(s) => (s.clone(), false),
        _ => (serde_json::to_string(value).unwrap_or_default(), false),
    }
}
