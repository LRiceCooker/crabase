use leptos::prelude::*;

use crate::icons::IconX;
use crate::sql_editor::codemirror::CodeMirrorEditor;
use crate::table_view::data_table::unwrap_tagged;
use crate::tauri::QueryResult;

#[component]
pub fn SqlResults(
    result: ReadSignal<Option<Result<QueryResult, String>>>,
) -> impl IntoView {
    // State for read-only JSON viewer modal
    let (json_modal, set_json_modal) = signal(Option::<String>::None);

    view! {
        // Read-only JSON viewer modal
        {move || json_modal.get().map(|content| {
            let on_close = move |_| set_json_modal.set(None);
            view! {
                <div
                    class="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center"
                    on:click=on_close
                >
                    <div
                        class="bg-white dark:bg-zinc-900 rounded-lg shadow-xl dark:shadow-black/40 border border-gray-200 dark:border-white/[0.08] w-[600px] max-h-[80vh] flex flex-col dark:ring-1 dark:ring-white/[0.06]"
                        on:click=move |ev| ev.stop_propagation()
                    >
                        <div class="px-4 py-3 border-b border-gray-200 dark:border-zinc-800 flex items-center justify-between shrink-0">
                            <h3 class="text-[13px] font-semibold text-gray-900 dark:text-neutral-50">"View JSON"</h3>
                            <button
                                class="text-gray-400 dark:text-zinc-500 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 p-1 rounded-md transition-colors duration-100"
                                on:click=on_close
                            >
                                <IconX class="w-4 h-4" />
                            </button>
                        </div>
                        <div class="px-4 py-4 flex-1 overflow-hidden flex flex-col">
                            <div class="flex-1 min-h-[200px] border border-gray-200 dark:border-zinc-800 rounded-md overflow-hidden">
                                <CodeMirrorEditor
                                    initial_content=content
                                    language="json".to_string()
                                    read_only=true
                                />
                            </div>
                        </div>
                        <div class="px-4 py-3 border-t border-gray-200 dark:border-zinc-800 flex items-center justify-end shrink-0">
                            <button
                                class="text-gray-500 dark:text-zinc-400 hover:bg-gray-100 dark:hover:bg-zinc-800 hover:text-gray-900 dark:hover:text-neutral-50 px-3 py-1.5 rounded-md text-[13px] transition-colors duration-100"
                                on:click=on_close
                            >
                                "Close"
                            </button>
                        </div>
                    </div>
                </div>
            }
        })}

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
                                                        let (text, is_null, is_json) = format_value(cell);
                                                        let class = if is_null {
                                                            "px-3 py-1.5 border-b border-gray-100 dark:border-[#1F1F23] border-r border-gray-100 truncate max-w-[300px] text-gray-300 dark:text-zinc-600 italic"
                                                        } else if is_json {
                                                            "px-3 py-1.5 border-b border-gray-100 dark:border-[#1F1F23] border-r border-gray-100 truncate max-w-[300px] text-gray-900 dark:text-neutral-50 cursor-pointer hover:bg-indigo-50 dark:hover:bg-indigo-500/10"
                                                        } else {
                                                            "px-3 py-1.5 border-b border-gray-100 dark:border-[#1F1F23] border-r border-gray-100 truncate max-w-[300px] text-gray-900 dark:text-neutral-50"
                                                        };
                                                        let title = text.clone();
                                                        let cell_clone = cell.clone();
                                                        view! {
                                                            <td
                                                                class=class
                                                                title=title
                                                                on:click=move |_| {
                                                                    if is_json {
                                                                        let inner = unwrap_tagged(&cell_clone);
                                                                        let pretty = serde_json::to_string_pretty(inner).unwrap_or_default();
                                                                        set_json_modal.set(Some(pretty));
                                                                    }
                                                                }
                                                            >
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

/// Format a cell value for display. Returns (display_text, is_null, is_json_clickable).
fn format_value(value: &serde_json::Value) -> (String, bool, bool) {
    let inner = unwrap_tagged(value);

    // Detect tagged type for JSON detection
    let tagged_type = if let serde_json::Value::Object(map) = value {
        map.get("type").and_then(|t| t.as_str()).unwrap_or("")
    } else {
        ""
    };

    match inner {
        serde_json::Value::Null => ("NULL".to_string(), true, false),
        serde_json::Value::Bool(b) => {
            if *b {
                ("\u{2713}".to_string(), false, false) // checkmark
            } else {
                ("\u{2717}".to_string(), false, false) // cross mark
            }
        }
        serde_json::Value::Number(n) => (n.to_string(), false, false),
        serde_json::Value::String(s) => {
            // Check if this is a JSON/JSONB value stored as string
            let is_json = tagged_type == "json" || tagged_type == "jsonb";
            if is_json {
                let display = if s.len() > 50 {
                    format!("{}...", &s[..50])
                } else {
                    s.clone()
                };
                (display, false, true)
            } else {
                (s.clone(), false, false)
            }
        }
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
            (display, false, false)
        }
        serde_json::Value::Object(_) => {
            // JSON objects are clickable
            let s = serde_json::to_string(inner).unwrap_or_default();
            let display = if s.len() > 50 {
                format!("{}...", &s[..50])
            } else {
                s
            };
            (display, false, true)
        }
    }
}
