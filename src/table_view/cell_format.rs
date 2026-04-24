use crate::tauri::ColumnInfo;

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
/// Returns (display_text, is_null).
pub fn format_cell(value: &serde_json::Value, data_type: &str) -> (String, bool) {
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
pub fn modal_type(col: &ColumnInfo) -> Option<&'static str> {
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
