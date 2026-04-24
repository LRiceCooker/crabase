use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::invoke;

/// Column metadata for a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    #[serde(default)]
    pub is_auto_increment: bool,
    #[serde(default)]
    pub is_array: bool,
    #[serde(default)]
    pub is_enum: bool,
    #[serde(default)]
    pub enum_values: Vec<String>,
    #[serde(default)]
    pub max_length: Option<i32>,
    #[serde(default)]
    pub numeric_precision: Option<i32>,
    #[serde(default)]
    pub numeric_scale: Option<i32>,
}

/// Paginated table data with columns, rows, and total count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub total_count: u64,
}

/// A single filter condition for table data queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub column: String,
    pub operator: String,
    pub value: String,
    pub combinator: String,
}

/// A sort column specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortCol {
    pub column: String,
    pub direction: String,
}

/// A single row update with primary key values and changed columns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowUpdate {
    pub pk_values: std::collections::HashMap<String, serde_json::Value>,
    pub changes: std::collections::HashMap<String, serde_json::Value>,
}

/// A single row insertion with column values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowInsert {
    pub values: std::collections::HashMap<String, serde_json::Value>,
}

/// A single row deletion identified by primary key values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowDelete {
    pub pk_values: std::collections::HashMap<String, serde_json::Value>,
}

/// A batch of table mutations (updates, inserts, deletes).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub updates: Vec<RowUpdate>,
    pub inserts: Vec<RowInsert>,
    pub deletes: Vec<RowDelete>,
}

/// Lists all table names in the current schema.
pub async fn list_tables() -> Result<Vec<String>, String> {
    let result = invoke("list_tables", JsValue::UNDEFINED)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to list tables".to_string()))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse tables list: {e}"))
}

/// Gets column names for each table, used for SQL editor autocomplete.
pub async fn get_columns_for_autocomplete(
    table_names: &[String],
) -> Result<std::collections::HashMap<String, Vec<String>>, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        table_names: &'a [String],
    }

    let args = serde_wasm_bindgen::to_value(&Args { table_names })
        .map_err(|e| e.to_string())?;

    let result = invoke("get_columns_for_autocomplete", args)
        .await
        .map_err(|e| {
            e.as_string()
                .unwrap_or_else(|| "Failed to get columns for autocomplete".to_string())
        })?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse autocomplete columns: {e}"))
}

/// Fetches paginated table data (columns, rows, total count).
pub async fn get_table_data(
    table_name: &str,
    page: u32,
    page_size: u32,
) -> Result<TableData, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        table_name: &'a str,
        page: u32,
        page_size: u32,
    }

    let args = serde_wasm_bindgen::to_value(&Args {
        table_name,
        page,
        page_size,
    })
    .map_err(|e| e.to_string())?;

    let result = invoke("get_table_data", args)
        .await
        .map_err(|e| {
            e.as_string()
                .unwrap_or_else(|| "Failed to get table data".to_string())
        })?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse table data: {e}"))
}

/// Fetches paginated table data with filters and sort columns applied.
pub async fn get_table_data_filtered(
    table_name: &str,
    page: u32,
    page_size: u32,
    filters: Vec<Filter>,
    sort: Vec<SortCol>,
) -> Result<TableData, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        table_name: &'a str,
        page: u32,
        page_size: u32,
        filters: &'a Vec<Filter>,
        sort: &'a Vec<SortCol>,
    }

    let args = serde_wasm_bindgen::to_value(&Args {
        table_name,
        page,
        page_size,
        filters: &filters,
        sort: &sort,
    })
    .map_err(|e| e.to_string())?;

    let result = invoke("get_table_data_filtered", args)
        .await
        .map_err(|e| {
            e.as_string()
                .unwrap_or_else(|| "Failed to get table data".to_string())
        })?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse table data: {e}"))
}

/// Applies a batch of changes (inserts, updates, deletes) to a table.
pub async fn save_changes(table_name: &str, changes: &ChangeSet) -> Result<String, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        table_name: &'a str,
        changes: &'a ChangeSet,
    }

    let args = serde_wasm_bindgen::to_value(&Args {
        table_name,
        changes,
    })
    .map_err(|e| e.to_string())?;

    let result = invoke("save_changes", args)
        .await
        .map_err(|e| {
            e.as_string()
                .unwrap_or_else(|| "Failed to save changes".to_string())
        })?;

    result
        .as_string()
        .ok_or_else(|| "Invalid response from backend".to_string())
}

/// Drops an entire table.
pub async fn drop_table(table_name: &str) -> Result<String, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> { table_name: &'a str }
    let args = serde_wasm_bindgen::to_value(&Args { table_name }).map_err(|e| e.to_string())?;
    let result = invoke("drop_table", args).await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Drop failed".to_string()))?;
    Ok(result.as_string().unwrap_or_default())
}

/// Truncates all rows from a table.
pub async fn truncate_table(table_name: &str) -> Result<String, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> { table_name: &'a str }
    let args = serde_wasm_bindgen::to_value(&Args { table_name }).map_err(|e| e.to_string())?;
    let result = invoke("truncate_table", args).await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Truncate failed".to_string()))?;
    Ok(result.as_string().unwrap_or_default())
}

/// Exports a table as JSON.
pub async fn export_table_json(table_name: &str) -> Result<String, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> { table_name: &'a str }
    let args = serde_wasm_bindgen::to_value(&Args { table_name }).map_err(|e| e.to_string())?;
    let result = invoke("export_table_json", args).await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Export failed".to_string()))?;
    Ok(result.as_string().unwrap_or_default())
}

/// Exports a table as SQL INSERT statements.
pub async fn export_table_sql(table_name: &str) -> Result<String, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> { table_name: &'a str }
    let args = serde_wasm_bindgen::to_value(&Args { table_name }).map_err(|e| e.to_string())?;
    let result = invoke("export_table_sql", args).await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Export failed".to_string()))?;
    Ok(result.as_string().unwrap_or_default())
}
