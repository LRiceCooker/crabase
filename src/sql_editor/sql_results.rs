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
                            <div class="flex-1 flex items-center justify-center text-gray-400 dark:text-zinc-500 text-[13px]">
                                "Query executed successfully (no results)"
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="flex-1 overflow-auto border-t border-gray-200 dark:border-zinc-800">
                                <table class="w-full text-xs font-mono text-gray-900 dark:text-neutral-50">
                                    <thead>
                                        <tr class="bg-gray-50 dark:bg-[#0F0F11] border-b border-gray-200 dark:border-zinc-800 sticky top-0 z-10">
                                            {qr.columns.iter().map(|col| {
                                                let name = col.clone();
                                                view! {
                                                    <th class="px-3 py-2 text-left text-[11px] font-medium uppercase tracking-wider text-gray-500 dark:text-zinc-400 border-r border-gray-100 dark:border-[#1F1F23] select-none whitespace-nowrap">
                                                        {name}
                                                    </th>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {qr.rows.iter().map(|row| {
                                            view! {
                                                <tr class="hover:bg-gray-50 dark:hover:bg-white/[0.03]">
                                                    {row.iter().map(|cell| {
                                                        let (text, is_null) = format_value(cell);
                                                        let class = if is_null {
                                                            "px-3 py-1.5 border-b border-gray-100 dark:border-[#1F1F23] border-r border-gray-100 truncate max-w-[300px] text-gray-300 dark:text-zinc-600 italic"
                                                        } else {
                                                            "px-3 py-1.5 border-b border-gray-100 dark:border-[#1F1F23] border-r border-gray-100 truncate max-w-[300px] text-gray-900 dark:text-neutral-50"
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
                    <div class="flex-1 bg-gray-900 dark:bg-[#0D0D0F] text-red-400 font-mono text-xs p-3 overflow-y-auto border-t border-gray-200 dark:border-zinc-800">
                        <pre class="whitespace-pre-wrap">{err}</pre>
                    </div>
                }.into_any(),
            }
        }}
    }
}

fn format_value(value: &serde_json::Value) -> (String, bool) {
    // Unwrap tagged values from backend (e.g. {"type":"unknown","raw":"..."})
    let inner = crate::table_view::data_table::unwrap_tagged(value);
    match inner {
        serde_json::Value::Null => ("NULL".to_string(), true),
        serde_json::Value::Bool(b) => (b.to_string(), false),
        serde_json::Value::Number(n) => (n.to_string(), false),
        serde_json::Value::String(s) => (s.clone(), false),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().take(3).map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => "NULL".to_string(),
                _ => v.to_string(),
            }).collect();
            let display = if arr.len() > 3 {
                format!("[{}, ...]", items.join(", "))
            } else {
                format!("[{}]", items.join(", "))
            };
            (display, false)
        }
        serde_json::Value::Object(_) => {
            let s = serde_json::to_string(inner).unwrap_or_default();
            let display = if s.len() > 50 {
                format!("{}...", &s[..50])
            } else {
                s
            };
            (display, false)
        }
    }
}
