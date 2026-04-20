use leptos::prelude::*;

use crate::tauri::ColumnInfo;
use crate::table_view::cell_editors::bit_editor::BitEditor;
use crate::table_view::cell_editors::boolean_editor::BooleanEditor;
use crate::table_view::cell_editors::bytea_editor::ByteaEditor;
use crate::table_view::cell_editors::date_editor::DateEditor;
use crate::table_view::cell_editors::datetime_editor::DatetimeEditor;
use crate::table_view::cell_editors::enum_editor::EnumEditor;
use crate::table_view::cell_editors::inet_editor::InetEditor;
use crate::table_view::cell_editors::interval_editor::IntervalEditor;
use crate::table_view::cell_editors::number_editor::NumberEditor;
use crate::table_view::cell_editors::range_editor::RangeEditor;
use crate::table_view::cell_editors::text_editor::TextEditor;
use crate::table_view::cell_editors::time_editor::TimeEditor;
use crate::table_view::cell_editors::unknown_editor::UnknownEditor;
use crate::table_view::cell_editors::uuid_editor::UuidEditor;
use crate::table_view::cell_editors::NullButton;

/// Represents a cell edit completion.
#[derive(Clone, Debug)]
pub struct CellEdit {
    pub row: usize,
    pub col: usize,
    pub value: serde_json::Value,
}

/// Inline cell editor. Dispatches to the appropriate specialized editor based on column metadata.
/// Modal editors (JSON, XML, array) are handled separately by DataTable.
///
/// For nullable columns, a "×" (Set NULL) button is shown next to the editor.
/// BooleanEditor and EnumEditor handle nullable internally via a select dropdown.
#[component]
pub fn CellEditor(
    column: ColumnInfo,
    value: serde_json::Value,
    #[prop(default = false)] is_new_row: bool,
    on_commit: Callback<serde_json::Value>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    // Primary key and auto-increment columns are read-only on existing rows
    if (column.is_primary_key || column.is_auto_increment) && !is_new_row {
        return view! {
            <UnknownEditor value=value on_cancel=on_cancel />
        }
        .into_any();
    }

    let is_nullable = column.is_nullable;

    // Enum editor handles nullable internally with a NULL option in the dropdown
    if column.is_enum {
        return view! {
            <EnumEditor
                value=value
                enum_values=column.enum_values.clone()
                is_nullable=column.is_nullable
                on_commit=on_commit
                on_cancel=on_cancel
            />
        }
        .into_any();
    }

    let dt = column.data_type.to_lowercase();

    // Boolean and unknown editors handle nullable specially (or are read-only)
    match dt.as_str() {
        "boolean" | "bool" => {
            // BooleanEditor handles nullable internally with a tri-state select
            return view! {
                <BooleanEditor value=value is_nullable=is_nullable on_commit=on_commit on_cancel=on_cancel />
            }
            .into_any();
        }
        "unknown" | "tsvector" | "tsquery" | "geometry" => {
            // Read-only editor, no NULL toggle needed
            return view! {
                <UnknownEditor value=value on_cancel=on_cancel />
            }
            .into_any();
        }
        _ => {}
    }

    // Build the editor view for all other types
    let editor = match dt.as_str() {
        "smallint" | "integer" | "bigint" | "int2" | "int4" | "int8" | "serial"
        | "smallserial" | "bigserial" => {
            view! {
                <NumberEditor value=value is_integer=true on_commit=on_commit on_cancel=on_cancel />
            }
            .into_any()
        }
        "real" | "double" | "numeric" | "money" | "float4" | "float8" | "decimal"
        | "double precision" => {
            let step = if let (Some(_prec), Some(scale)) =
                (column.numeric_precision, column.numeric_scale)
            {
                if scale > 0 {
                    let step_val = 10f64.powi(-scale);
                    format!("{step_val}")
                } else {
                    "1".to_string()
                }
            } else {
                "any".to_string()
            };
            view! {
                <NumberEditor
                    value=value
                    is_integer=false
                    step=step
                    on_commit=on_commit
                    on_cancel=on_cancel
                />
            }
            .into_any()
        }
        "date" => {
            view! {
                <DateEditor value=value on_commit=on_commit on_cancel=on_cancel />
            }
            .into_any()
        }
        "time" => {
            view! {
                <TimeEditor value=value on_commit=on_commit on_cancel=on_cancel />
            }
            .into_any()
        }
        "timestamp" | "timestamptz" => {
            view! {
                <DatetimeEditor value=value on_commit=on_commit on_cancel=on_cancel />
            }
            .into_any()
        }
        "interval" => {
            view! {
                <IntervalEditor value=value on_commit=on_commit on_cancel=on_cancel />
            }
            .into_any()
        }
        "uuid" => {
            view! {
                <UuidEditor value=value on_commit=on_commit on_cancel=on_cancel />
            }
            .into_any()
        }
        "inet" | "cidr" | "macaddr" => {
            let editor_type = dt.clone();
            view! {
                <InetEditor
                    value=value
                    editor_type=editor_type
                    on_commit=on_commit
                    on_cancel=on_cancel
                />
            }
            .into_any()
        }
        "bit" => {
            view! {
                <BitEditor value=value on_commit=on_commit on_cancel=on_cancel />
            }
            .into_any()
        }
        "range" => {
            view! {
                <RangeEditor value=value on_commit=on_commit on_cancel=on_cancel />
            }
            .into_any()
        }
        "bytea" => {
            view! {
                <ByteaEditor value=value on_commit=on_commit on_cancel=on_cancel />
            }
            .into_any()
        }
        // Default: text editor for text, varchar, char, and anything else
        _ => {
            let max_length = column.max_length.unwrap_or(0);
            view! {
                <TextEditor
                    value=value
                    max_length=max_length
                    on_commit=on_commit
                    on_cancel=on_cancel
                />
            }
            .into_any()
        }
    };

    // Wrap with "×" NULL button for nullable columns
    if is_nullable {
        view! {
            <div class="flex items-center gap-0.5 w-full">
                {editor}
                <NullButton on_commit=on_commit />
            </div>
        }
        .into_any()
    } else {
        editor
    }
}
