use leptos::prelude::*;

use crate::icons::{IconFilter, IconPlus, IconX};
use crate::tauri::{ColumnInfo, Filter, SortCol};

/// Inline filter & sort bar, always visible below the toolbar.
#[component]
pub fn FilterBar(
    columns: Memo<Vec<ColumnInfo>>,
    filters: RwSignal<Vec<Filter>>,
    sort: RwSignal<Vec<SortCol>>,
    on_change: Callback<()>,
) -> impl IntoView {
    let add_filter = move |_| {
        let cols = columns.get();
        if let Some(first_col) = cols.first() {
            filters.update(|f| {
                let combinator = if f.is_empty() {
                    "AND".to_string()
                } else {
                    "AND".to_string()
                };
                f.push(Filter {
                    column: first_col.name.clone(),
                    operator: "=".to_string(),
                    value: String::new(),
                    combinator,
                });
            });
        }
    };

    let remove_filter = move |idx: usize| {
        filters.update(|f| {
            if idx < f.len() {
                f.remove(idx);
            }
        });
        on_change.run(());
    };

    let remove_sort = move |idx: usize| {
        sort.update(|s| {
            if idx < s.len() {
                s.remove(idx);
            }
        });
        on_change.run(());
    };

    view! {
        <div class="flex items-center gap-1.5 px-3 py-1.5 border-b border-gray-100 dark:border-[#1F1F23] bg-white dark:bg-neutral-950 shrink-0 min-h-[32px] flex-wrap">
            <IconFilter class="w-3.5 h-3.5 text-gray-400 dark:text-zinc-500 shrink-0" />

            // Filter chips
            {move || {
                let current_filters = filters.get();
                let cols = columns.get();
                current_filters.into_iter().enumerate().map(|(idx, f)| {
                    let cols_for_select = cols.clone();
                    let cols_for_select2 = cols.clone();
                    view! {
                        <FilterChip
                            idx=idx
                            filter=f
                            columns=cols_for_select
                            columns_for_ops=cols_for_select2
                            filters=filters
                            on_change=on_change
                            on_remove=Callback::new(move |_| remove_filter(idx))
                        />
                    }
                }).collect::<Vec<_>>()
            }}

            // Sort chips
            {move || {
                let current_sort = sort.get();
                current_sort.into_iter().enumerate().map(|(idx, s)| {
                    let label = format!(
                        "{} {}",
                        s.column,
                        if s.direction == "desc" { "\u{2193}" } else { "\u{2191}" }
                    );
                    view! {
                        <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-violet-50 dark:bg-violet-500/20 text-[11px] text-violet-700 dark:text-violet-300 border border-violet-200 dark:border-violet-500/30">
                            {label}
                            <button
                                class="hover:text-red-500 dark:hover:text-red-400 transition-colors duration-100"
                                on:click=move |_| remove_sort(idx)
                            >
                                <IconX class="w-3 h-3" />
                            </button>
                        </span>
                    }
                }).collect::<Vec<_>>()
            }}

            // Add filter button
            <button
                class="inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded text-[11px] text-gray-400 dark:text-zinc-500 hover:bg-gray-100 dark:hover:bg-white/[0.06] hover:text-gray-600 dark:hover:text-zinc-300 transition-colors duration-100"
                title="Add filter"
                on:click=add_filter
            >
                <IconPlus class="w-3 h-3" />
                "Filter"
            </button>
        </div>
    }
}

/// Operators available for filtering.
const OPERATORS: &[&str] = &[
    "=", "!=", "<", ">", "<=", ">=", "LIKE", "NOT LIKE", "IN", "NOT IN", "IS NULL", "IS NOT NULL",
    "contains", "starts with", "ends with",
];

/// Combinators for chaining filters.
const COMBINATORS: &[&str] = &["AND", "OR", "XOR"];

/// A single filter chip: column select + operator select + value input + delete.
#[component]
fn FilterChip(
    idx: usize,
    filter: Filter,
    columns: Vec<ColumnInfo>,
    columns_for_ops: Vec<ColumnInfo>,
    filters: RwSignal<Vec<Filter>>,
    on_change: Callback<()>,
    on_remove: Callback<()>,
) -> impl IntoView {
    let is_first = idx == 0;
    let needs_value = !matches!(filter.operator.as_str(), "IS NULL" | "IS NOT NULL");

    let update_field = move |field: &str, value: String| {
        let field = field.to_string();
        filters.update(|f| {
            if let Some(flt) = f.get_mut(idx) {
                match field.as_str() {
                    "column" => flt.column = value,
                    "operator" => flt.operator = value,
                    "value" => flt.value = value,
                    "combinator" => flt.combinator = value,
                    _ => {}
                }
            }
        });
        on_change.run(());
    };

    let chip_class = "inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded bg-indigo-50 dark:bg-indigo-500/20 text-[11px] text-indigo-700 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-500/30";

    let select_class = "bg-transparent text-[11px] outline-none cursor-pointer text-indigo-700 dark:text-indigo-300";
    let input_class = "bg-transparent text-[11px] outline-none w-16 placeholder:text-indigo-300 dark:placeholder:text-indigo-500 text-indigo-700 dark:text-indigo-300";

    view! {
        {if !is_first {
            let combinator = filter.combinator.clone();
            Some(view! {
                <select
                    class="bg-transparent text-[10px] font-medium outline-none cursor-pointer text-gray-400 dark:text-zinc-500 uppercase"
                    on:change=move |ev| {
                        let val = leptos::prelude::event_target_value(&ev);
                        update_field("combinator", val);
                    }
                >
                    {COMBINATORS.iter().map(|c| {
                        let selected = *c == combinator.as_str();
                        view! { <option value=*c selected=selected>{*c}</option> }
                    }).collect::<Vec<_>>()}
                </select>
            })
        } else {
            None
        }}

        <span class=chip_class>
            // Column select
            <select
                class=select_class
                on:change=move |ev| {
                    let val = leptos::prelude::event_target_value(&ev);
                    update_field("column", val);
                }
            >
                {columns.iter().map(|col| {
                    let selected = col.name == filter.column;
                    let name = col.name.clone();
                    let name2 = col.name.clone();
                    view! { <option value=name selected=selected>{name2}</option> }
                }).collect::<Vec<_>>()}
            </select>

            // Operator select
            <select
                class=select_class
                on:change=move |ev| {
                    let val = leptos::prelude::event_target_value(&ev);
                    update_field("operator", val);
                }
            >
                {OPERATORS.iter().map(|op| {
                    let selected = *op == filter.operator.as_str();
                    view! { <option value=*op selected=selected>{*op}</option> }
                }).collect::<Vec<_>>()}
            </select>

            // Value input (hidden for IS NULL / IS NOT NULL)
            {if needs_value {
                let val = filter.value.clone();
                Some(view! {
                    <input
                        type="text"
                        class=input_class
                        placeholder="value"
                        prop:value=val
                        on:change=move |ev| {
                            let val = leptos::prelude::event_target_value(&ev);
                            update_field("value", val);
                        }
                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                            if ev.key() == "Enter" {
                                let val = leptos::prelude::event_target_value(&ev);
                                update_field("value", val);
                            }
                        }
                    />
                })
            } else {
                None
            }}

            // Remove button
            <button
                class="hover:text-red-500 dark:hover:text-red-400 transition-colors duration-100"
                on:click=move |_| on_remove.run(())
            >
                <IconX class="w-3 h-3" />
            </button>
        </span>
    }
}
