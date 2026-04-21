use std::collections::HashSet;

use leptos::prelude::*;

use crate::table_view::data_table::unwrap_tagged;
use crate::tauri;

/// Reactive state for the find overlay (Cmd+F search within the table).
#[derive(Clone, Copy)]
pub struct FindState {
    pub query: RwSignal<String>,
    pub current: RwSignal<usize>,
    pub matches: Memo<Vec<(usize, usize)>>,
    pub highlighted_cells: Memo<HashSet<(usize, usize)>>,
}

impl FindState {
    /// Create a new find state derived from the given rows and columns.
    pub fn new(
        rows: RwSignal<Vec<Vec<serde_json::Value>>>,
        columns: ReadSignal<Vec<tauri::ColumnInfo>>,
    ) -> Self {
        let query = RwSignal::new(String::new());
        let current = RwSignal::new(0usize);

        let matches = Memo::new(move |_| {
            let q = query.get();
            if q.is_empty() {
                return Vec::new();
            }
            let query_lower = q.to_lowercase();
            let current_rows = rows.get();
            let _cols = columns.get();
            let mut result = Vec::new();
            for (row_idx, row) in current_rows.iter().enumerate() {
                for (col_idx, cell) in row.iter().enumerate() {
                    let inner = unwrap_tagged(cell);
                    let text = match inner {
                        serde_json::Value::Null => "null".to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::String(s) => s.clone(),
                        _ => serde_json::to_string(inner).unwrap_or_default(),
                    };
                    if text.to_lowercase().contains(&query_lower) {
                        result.push((row_idx, col_idx));
                    }
                }
            }
            result
        });

        let highlighted_cells = Memo::new(move |_| {
            let m = matches.get();
            m.into_iter().collect::<HashSet<(usize, usize)>>()
        });

        Self {
            query,
            current,
            matches,
            highlighted_cells,
        }
    }
}
