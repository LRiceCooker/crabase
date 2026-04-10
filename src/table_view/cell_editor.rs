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

/// Represents a cell edit completion.
#[derive(Clone, Debug)]
pub struct CellEdit {
    pub row: usize,
    pub col: usize,
    pub value: serde_json::Value,
}

/// Inline cell editor. Dispatches to the appropriate specialized editor based on column metadata.
/// Modal editors (JSON, XML, array) are handled separately by DataTable.
#[component]
pub fn CellEditor(
    column: ColumnInfo,
    value: serde_json::Value,
    on_commit: Callback<serde_json::Value>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    // Enum columns get a select dropdown regardless of data_type string
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

    match dt.as_str() {
        "boolean" | "bool" => {
            view! {
                <BooleanEditor value=value on_commit=on_commit on_cancel=on_cancel />
            }
            .into_any()
        }
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
                    format!("{}", step_val)
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
        "unknown" | "tsvector" | "tsquery" | "geometry" => {
            view! {
                <UnknownEditor value=value on_cancel=on_cancel />
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
    }
}
