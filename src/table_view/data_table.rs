use std::collections::HashSet;

use leptos::prelude::*;

use crate::table_view::cell_editor::{CellEdit, CellEditor};
use crate::table_view::cell_editors::array_editor_modal::ArrayEditRequest;
use crate::table_view::cell_editors::xml_editor_modal::XmlEditRequest;
use crate::table_view::change_tracker::ChangeTracker;
use crate::table_view::json_editor::JsonEditRequest;
use crate::tauri::{ColumnInfo, SortCol};

/// Info about a right-click event on a row.
pub struct RowContextMenuEvent {
    pub row_idx: usize,
    pub x: i32,
    pub y: i32,
}

/// Extract the inner value from a tagged cell `{ "type": "...", "value": ... }`.
/// Falls through to the raw value if not tagged.
pub fn unwrap_tagged(value: &serde_json::Value) -> &serde_json::Value {
    if let serde_json::Value::Object(map) = value {
        if map.contains_key("type") {
            if let Some(inner) = map.get("value") {
                return inner;
            }
            // Unknown type: { "type": "unknown", "raw": "..." }
            if let Some(raw) = map.get("raw") {
                return raw;
            }
        }
    }
    value
}

/// Extract the inner value as an owned clone.
pub fn unwrap_tagged_owned(value: &serde_json::Value) -> serde_json::Value {
    unwrap_tagged(value).clone()
}

/// Format a cell value for display. Handles tagged values.
fn format_cell(value: &serde_json::Value, data_type: &str) -> (String, bool) {
    let inner = unwrap_tagged(value);
    match inner {
        serde_json::Value::Null => ("NULL".to_string(), true),
        serde_json::Value::Bool(b) => {
            if *b {
                ("\u{2713}".to_string(), false) // checkmark
            } else {
                ("\u{2717}".to_string(), false) // cross mark
            }
        }
        serde_json::Value::Number(n) => (n.to_string(), false),
        serde_json::Value::String(s) => {
            let dt = data_type.to_lowercase();
            match dt.as_str() {
                "json" | "jsonb" => {
                    // Truncate JSON string display
                    let display = if s.len() > 50 {
                        format!("{}...", &s[..50])
                    } else {
                        s.clone()
                    };
                    (display, false)
                }
                _ => (s.clone(), false),
            }
        }
        serde_json::Value::Array(arr) => {
            // Array display: show first 3 items + "..."
            let items: Vec<String> = arr
                .iter()
                .take(3)
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Null => "NULL".to_string(),
                    _ => v.to_string(),
                })
                .collect();
            let mut display = format!("[{}]", items.join(", "));
            if arr.len() > 3 {
                display = format!("[{}, ...]", items.join(", "));
            }
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

/// Check if a type should open a modal editor (JSON, XML, or Array).
fn modal_type(col: &ColumnInfo) -> Option<&'static str> {
    if col.is_array {
        return Some("array");
    }
    let dt = col.data_type.to_lowercase();
    match dt.as_str() {
        "json" | "jsonb" => Some("json"),
        "xml" => Some("xml"),
        _ => None,
    }
}

#[component]
pub fn DataTable(
    columns: Vec<ColumnInfo>,
    rows: RwSignal<Vec<Vec<serde_json::Value>>>,
    changes: ChangeTracker,
    selected_rows: RwSignal<HashSet<usize>>,
    selection_anchor: RwSignal<Option<usize>>,
    page: u32,
    page_size: u32,
    on_cell_edit: Callback<CellEdit>,
    on_json_edit: Callback<JsonEditRequest>,
    on_array_edit: Callback<ArrayEditRequest>,
    on_xml_edit: Callback<XmlEditRequest>,
    on_row_context_menu: Callback<RowContextMenuEvent>,
    active_sort: RwSignal<Vec<SortCol>>,
    on_sort_change: Callback<()>,
) -> impl IntoView {
    // Track which cell is being edited: (row_idx, col_idx)
    let (editing_cell, set_editing_cell) = signal(Option::<(usize, usize)>::None);

    // Clone columns for use in the view closures
    let columns_for_types = columns.clone();

    view! {
        <div class="overflow-auto flex-1">
            <table class="w-full text-xs font-mono">
                <thead>
                    <tr class="bg-gray-50 dark:bg-[#0F0F11] border-b border-gray-200 dark:border-zinc-800 sticky top-0 z-10">
                        <th class="sticky left-0 z-20 bg-gray-50 dark:bg-[#0F0F11] px-2 py-2 text-left text-[11px] font-medium text-gray-500 dark:text-zinc-400 border-r border-gray-100 dark:border-[#1F1F23] select-none w-10"></th>
                        {columns.iter().map(|col| {
                            let name = col.name.clone();
                            let name_for_click = col.name.clone();
                            let data_type = col.data_type.clone();
                            view! {
                                <th
                                    class="px-3 py-2 text-left text-[11px] font-medium uppercase tracking-wider text-gray-500 dark:text-zinc-400 border-r border-gray-100 dark:border-[#1F1F23] select-none whitespace-nowrap cursor-pointer hover:bg-gray-100 dark:hover:bg-white/[0.06] transition-colors duration-100"
                                    on:click={
                                        let col_name = name_for_click.clone();
                                        move |_| {
                                            let current = active_sort.get();
                                            // Find existing sort for this column
                                            let existing = current.iter().position(|s| s.column == col_name);
                                            let mut new_sort: Vec<SortCol> = current.into_iter().filter(|s| s.column != col_name).collect();
                                            match existing.map(|i| active_sort.get()[i].direction.as_str().to_string()) {
                                                None => {
                                                    // No sort → asc
                                                    new_sort.insert(0, SortCol { column: col_name.clone(), direction: "asc".to_string() });
                                                }
                                                Some(ref d) if d == "asc" => {
                                                    // asc → desc
                                                    new_sort.insert(0, SortCol { column: col_name.clone(), direction: "desc".to_string() });
                                                }
                                                _ => {
                                                    // desc → none (already removed)
                                                }
                                            }
                                            active_sort.set(new_sort);
                                            on_sort_change.run(());
                                        }
                                    }
                                >
                                    <div class="flex flex-col gap-0.5">
                                        <div class="flex items-center gap-1">
                                            <span>{name.clone()}</span>
                                            {move || {
                                                let sort = active_sort.get();
                                                sort.iter().find(|s| s.column == name).map(|s| {
                                                    let arrow = if s.direction == "asc" { "\u{2191}" } else { "\u{2193}" };
                                                    view! { <span class="text-indigo-500 dark:text-indigo-400 text-[10px]">{arrow}</span> }
                                                })
                                            }}
                                        </div>
                                        <span class="text-[10px] font-normal text-gray-400 dark:text-zinc-500 normal-case tracking-normal">{data_type}</span>
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
                        let selection = selected_rows.get();
                        let col_types = columns_for_types.clone();

                        current_rows.into_iter().enumerate().map(|(row_idx, row)| {
                            let col_types = col_types.clone();
                            let is_selected = selection.contains(&row_idx);
                            let (row_class, index_bg, index_border_l) = if changes.is_row_deleted(row_idx) {
                                (
                                    "bg-red-50 dark:bg-red-950/60 line-through opacity-60",
                                    "bg-red-50 dark:bg-red-950/60",
                                    " border-l-2 border-l-red-500 dark:border-l-red-400",
                                )
                            } else if changes.is_row_added(row_idx) {
                                (
                                    "bg-emerald-50 dark:bg-emerald-950/60",
                                    "bg-emerald-50 dark:bg-emerald-950/60",
                                    " border-l-2 border-emerald-500 dark:border-emerald-400",
                                )
                            } else if changes.is_row_modified(row_idx) {
                                (
                                    "bg-amber-50 dark:bg-amber-950/60",
                                    "bg-amber-50 dark:bg-amber-950/60",
                                    " border-l-2 border-amber-500 dark:border-amber-400",
                                )
                            } else if is_selected {
                                (
                                    "bg-indigo-50 dark:bg-indigo-500/25",
                                    "bg-indigo-50 dark:bg-indigo-500/25",
                                    "",
                                )
                            } else {
                                (
                                    "hover:bg-gray-50 dark:hover:bg-white/[0.03]",
                                    "bg-gray-50 dark:bg-[#0F0F11]",
                                    "",
                                )
                            };
                            let global_idx = (page - 1) * page_size + (row_idx as u32) + 1;
                            let index_td_class = format!(
                                "sticky left-0 z-[5] {} px-2 py-1.5 border-b border-gray-100 dark:border-[#1F1F23] border-r border-gray-100 dark:border-[#1F1F23] text-[11px] text-gray-400 dark:text-zinc-500 text-right select-none w-10 font-mono cursor-pointer{}",
                                index_bg, index_border_l
                            );
                            view! {
                                <tr
                                    class=row_class
                                    on:contextmenu=move |ev: web_sys::MouseEvent| {
                                        ev.prevent_default();
                                        // If right-clicked row is already in selection, keep selection;
                                        // otherwise select just this row.
                                        let sel = selected_rows.get();
                                        if !sel.contains(&row_idx) {
                                            let mut new_set = HashSet::new();
                                            new_set.insert(row_idx);
                                            selected_rows.set(new_set);
                                            selection_anchor.set(Some(row_idx));
                                        }
                                        on_row_context_menu.run(RowContextMenuEvent {
                                            row_idx,
                                            x: ev.client_x(),
                                            y: ev.client_y(),
                                        });
                                    }
                                >
                                    <td
                                        class=index_td_class
                                        on:click=move |ev: web_sys::MouseEvent| {
                                            if ev.shift_key() {
                                                // Shift+click: range select from anchor to clicked row
                                                let anchor = selection_anchor.get().unwrap_or(row_idx);
                                                let start = anchor.min(row_idx);
                                                let end = anchor.max(row_idx);
                                                let mut set = if ev.meta_key() || ev.ctrl_key() {
                                                    // Shift+Cmd: add range to existing selection
                                                    selected_rows.get()
                                                } else {
                                                    HashSet::new()
                                                };
                                                for i in start..=end {
                                                    set.insert(i);
                                                }
                                                selected_rows.set(set);
                                                // Don't update anchor on shift+click
                                            } else if ev.meta_key() || ev.ctrl_key() {
                                                // Cmd+click: toggle row in selection
                                                let mut set = selected_rows.get();
                                                if set.contains(&row_idx) {
                                                    set.remove(&row_idx);
                                                } else {
                                                    set.insert(row_idx);
                                                }
                                                selected_rows.set(set);
                                                selection_anchor.set(Some(row_idx));
                                            } else {
                                                // Plain click: select single row
                                                let mut new_set = HashSet::new();
                                                new_set.insert(row_idx);
                                                selected_rows.set(new_set);
                                                selection_anchor.set(Some(row_idx));
                                            }
                                        }
                                    >
                                        {global_idx}
                                    </td>
                                    {row.into_iter().enumerate().map(|(col_idx, cell)| {
                                        let is_editing = active == Some((row_idx, col_idx));
                                        let col_info = col_types.get(col_idx).cloned().unwrap_or_else(|| ColumnInfo {
                                            name: String::new(),
                                            data_type: String::new(),
                                            is_nullable: true,
                                            is_primary_key: false,
                                            is_auto_increment: false,
                                            is_array: false,
                                            is_enum: false,
                                            enum_values: vec![],
                                            max_length: None,
                                            numeric_precision: None,
                                            numeric_scale: None,
                                        });
                                        let modal = modal_type(&col_info);
                                        let data_type_display = col_info.data_type.clone();
                                        // PK and auto-increment columns are read-only on existing rows
                                        let is_new_row = changes.is_row_added(row_idx);
                                        let is_readonly = (col_info.is_primary_key || col_info.is_auto_increment) && !is_new_row;

                                        if is_editing && modal.is_none() && !is_readonly {
                                            let cell_val = unwrap_tagged_owned(&cell);
                                            view! {
                                                <td class="px-3 py-1.5 border-b border-gray-100 dark:border-[#1F1F23] border-r border-gray-100 ring-2 ring-indigo-500/30 dark:ring-indigo-500/60 bg-white dark:bg-zinc-900 max-w-[300px]">
                                                    <CellEditor
                                                        column=col_info
                                                        value=cell_val
                                                        is_new_row=is_new_row
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
                                            let (text, is_null) = format_cell(&cell, &data_type_display);
                                            let cell_modified = changes.is_cell_modified(row_idx, col_idx);
                                            let cursor = if is_readonly { "cursor-default" } else { "cursor-pointer" };
                                            let class = if is_null && cell_modified {
                                                format!("px-3 py-1.5 border-b border-gray-100 dark:border-[#1F1F23] border-r border-gray-100 truncate max-w-[300px] text-gray-300 dark:text-zinc-600 italic {} bg-amber-100/50 dark:bg-amber-900/40", cursor)
                                            } else if is_null {
                                                format!("px-3 py-1.5 border-b border-gray-100 dark:border-[#1F1F23] border-r border-gray-100 truncate max-w-[300px] text-gray-300 dark:text-zinc-600 italic {}", cursor)
                                            } else if cell_modified {
                                                format!("px-3 py-1.5 border-b border-gray-100 dark:border-[#1F1F23] border-r border-gray-100 truncate max-w-[300px] {} bg-amber-100/50 dark:bg-amber-900/40", cursor)
                                            } else {
                                                format!("px-3 py-1.5 border-b border-gray-100 dark:border-[#1F1F23] border-r border-gray-100 truncate max-w-[300px] {}", cursor)
                                            };
                                            let title = if is_readonly {
                                                format!("{} (read-only)", text)
                                            } else {
                                                text.clone()
                                            };
                                            let cell_for_modal = unwrap_tagged_owned(&cell);
                                            view! {
                                                <td
                                                    class=class
                                                    title=title
                                                    on:click=move |_| {
                                                        if is_readonly {
                                                            return;
                                                        }
                                                        match modal {
                                                            Some("json") => {
                                                                on_json_edit.run(JsonEditRequest {
                                                                    row: row_idx,
                                                                    col: col_idx,
                                                                    value: cell_for_modal.clone(),
                                                                });
                                                            }
                                                            Some("array") => {
                                                                on_array_edit.run(ArrayEditRequest {
                                                                    row: row_idx,
                                                                    col: col_idx,
                                                                    value: cell_for_modal.clone(),
                                                                });
                                                            }
                                                            Some("xml") => {
                                                                on_xml_edit.run(XmlEditRequest {
                                                                    row: row_idx,
                                                                    col: col_idx,
                                                                    value: cell_for_modal.clone(),
                                                                });
                                                            }
                                                            _ => {
                                                                set_editing_cell.set(Some((row_idx, col_idx)));
                                                            }
                                                        }
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
