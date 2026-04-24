use leptos::prelude::*;

use crate::icons::IconX;
use crate::sql_editor::codemirror::CodeMirrorEditor;
use crate::table_view::data_table::unwrap_tagged;
use crate::tauri::StatementResult;

/// Results pane for the SQL editor. Renders query result tables, affected-row counts, and errors.
#[component]
pub fn SqlResults(
    result: ReadSignal<Option<Result<Vec<StatementResult>, String>>>,
) -> impl IntoView {
    // State for read-only JSON viewer modal
    let (json_modal, set_json_modal) = signal(Option::<String>::None);
    // Active statement index for multi-statement results
    let (active_idx, set_active_idx) = signal(0usize);

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
                Some(Err(err)) => view! {
                    <div class="flex-1 bg-gray-900 dark:bg-[#0D0D0F] text-red-400 font-mono text-xs p-3 overflow-y-auto border-t border-gray-200 dark:border-zinc-800">
                        <pre class="whitespace-pre-wrap">{err}</pre>
                    </div>
                }.into_any(),
                Some(Ok(results)) => {
                    if results.is_empty() {
                        return view! {
                            <div class="flex-1 flex items-center justify-center text-gray-400 dark:text-zinc-500 text-[13px]">
                                "Query executed successfully (no results)"
                            </div>
                        }.into_any();
                    }

                    // Reset active index if out of bounds
                    let idx = active_idx.get().min(results.len().saturating_sub(1));
                    let active_result = results.get(idx).cloned();
                    let has_multiple = results.len() > 1;
                    let results_for_tabs = results.clone();

                    view! {
                        <div class="flex-1 flex flex-col overflow-hidden border-t border-gray-200 dark:border-zinc-800">
                            // Result content area
                            <div class="flex-1 overflow-auto">
                                {match active_result {
                                    Some(StatementResult::Rows { columns, rows, .. }) => {
                                        render_rows_table(columns, rows, set_json_modal).into_any()
                                    }
                                    Some(StatementResult::Affected { command, rows_affected, .. }) => {
                                        view! {
                                            <div class="flex items-center justify-center h-full text-[13px] text-gray-500 dark:text-zinc-400">
                                                <span class="font-mono">{format!("{command} — {rows_affected} row(s) affected")}</span>
                                            </div>
                                        }.into_any()
                                    }
                                    Some(StatementResult::Error { message, .. }) => {
                                        view! {
                                            <div class="bg-gray-900 dark:bg-[#0D0D0F] text-red-400 font-mono text-xs p-3 h-full overflow-y-auto">
                                                <pre class="whitespace-pre-wrap">{message}</pre>
                                            </div>
                                        }.into_any()
                                    }
                                    None => view! { <div></div> }.into_any(),
                                }}
                            </div>

                            // Statement selector (below the result table) — only shown for multi-statement
                            {if has_multiple {
                                Some(view! {
                                    <div class="shrink-0 border-t border-gray-200 dark:border-zinc-800 bg-gray-50 dark:bg-[#0F0F11] px-2 py-1.5 flex items-center gap-1 overflow-x-auto">
                                        {results_for_tabs.iter().enumerate().map(|(i, stmt)| {
                                            let is_active = i == idx;
                                            let preview = match stmt {
                                                StatementResult::Rows { sql_preview, .. } => sql_preview.clone(),
                                                StatementResult::Affected { sql_preview, .. } => sql_preview.clone(),
                                                StatementResult::Error { sql_preview, .. } => sql_preview.clone(),
                                            };
                                            let label = format!("#{} {}", i + 1, if preview.len() > 30 { &preview[..30] } else { &preview });
                                            let bg_class = if is_active {
                                                "px-2 py-1 text-[11px] font-mono rounded cursor-pointer bg-indigo-50 dark:bg-indigo-500/25 text-indigo-600 dark:text-indigo-400 whitespace-nowrap shrink-0"
                                            } else {
                                                "px-2 py-1 text-[11px] font-mono rounded cursor-pointer text-gray-500 dark:text-zinc-400 hover:bg-gray-100 dark:hover:bg-zinc-800 whitespace-nowrap shrink-0"
                                            };
                                            // Badge for result type
                                            let badge = match stmt {
                                                StatementResult::Rows { rows, .. } => format!("{} rows", rows.len()),
                                                StatementResult::Affected { rows_affected, .. } => format!("{rows_affected} affected"),
                                                StatementResult::Error { .. } => "error".to_string(),
                                            };
                                            view! {
                                                <button
                                                    class=bg_class
                                                    on:click=move |_| set_active_idx.set(i)
                                                >
                                                    {label}
                                                    <span class="ml-1 text-[10px] opacity-60">{format!("({badge})")}</span>
                                                </button>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                })
                            } else {
                                None
                            }}
                        </div>
                    }.into_any()
                }
            }
        }}
    }
}

/// Render a rows table for SELECT results.
fn render_rows_table(
    columns: Vec<String>,
    rows: Vec<Vec<serde_json::Value>>,
    set_json_modal: WriteSignal<Option<String>>,
) -> impl IntoView {
    if columns.is_empty() && rows.is_empty() {
        return view! {
            <div class="flex items-center justify-center h-full text-gray-400 dark:text-zinc-500 text-[13px]">
                "Query returned no rows"
            </div>
        }.into_any();
    }

    view! {
        <table class="w-full text-xs font-mono text-gray-900 dark:text-neutral-50">
            <thead>
                <tr class="bg-gray-50 dark:bg-[#0F0F11] border-b border-gray-200 dark:border-zinc-800 sticky top-0 z-10">
                    {columns.iter().map(|col| {
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
                {rows.iter().map(|row| {
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
    }.into_any()
}

/// Format a cell value for display. Returns (display_text, is_null, is_json_clickable).
fn format_value(value: &serde_json::Value) -> (String, bool, bool) {
    let inner = unwrap_tagged(value);

    let tagged_type = if let serde_json::Value::Object(map) = value {
        map.get("type").and_then(|t| t.as_str()).unwrap_or("")
    } else {
        ""
    };

    match inner {
        serde_json::Value::Null => ("NULL".to_string(), true, false),
        serde_json::Value::Bool(b) => {
            if *b {
                ("\u{2713}".to_string(), false, false)
            } else {
                ("\u{2717}".to_string(), false, false)
            }
        }
        serde_json::Value::Number(n) => (n.to_string(), false, false),
        serde_json::Value::String(s) => {
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
