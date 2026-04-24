use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::invoke;

/// Result of a single SQL query execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

/// Result of a single statement in a multi-statement execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StatementResult {
    Rows { columns: Vec<String>, rows: Vec<Vec<serde_json::Value>>, sql_preview: String },
    Affected { command: String, rows_affected: u64, sql_preview: String },
    Error { message: String, sql_preview: String },
}

/// A saved SQL query (name + SQL text).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedQuery {
    pub name: String,
    pub sql: String,
}

/// Executes multiple SQL statements and returns per-statement results.
pub async fn execute_query_multi(sql: &str) -> Result<Vec<StatementResult>, String> {
    #[derive(Serialize)]
    struct Args<'a> {
        sql: &'a str,
    }

    let args = serde_wasm_bindgen::to_value(&Args { sql })
        .map_err(|e| e.to_string())?;

    let result = invoke("execute_query_multi", args)
        .await
        .map_err(|e| {
            e.as_string()
                .unwrap_or_else(|| "Multi-statement execution failed".to_string())
        })?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse multi-statement result: {e}"))
}

/// Saves a new named query.
pub async fn save_query(name: &str, sql: &str) -> Result<(), String> {
    #[derive(Serialize)]
    struct Args<'a> {
        name: &'a str,
        sql: &'a str,
    }
    let args = serde_wasm_bindgen::to_value(&Args { name, sql }).map_err(|e| e.to_string())?;
    invoke("cmd_save_query", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to save query".to_string()))?;
    Ok(())
}

/// Updates the SQL of an existing saved query.
pub async fn update_query(name: &str, sql: &str) -> Result<(), String> {
    #[derive(Serialize)]
    struct Args<'a> {
        name: &'a str,
        sql: &'a str,
    }
    let args = serde_wasm_bindgen::to_value(&Args { name, sql }).map_err(|e| e.to_string())?;
    invoke("cmd_update_query", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to update query".to_string()))?;
    Ok(())
}

/// Renames a saved query.
pub async fn rename_query(old_name: &str, new_name: &str) -> Result<(), String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        old_name: &'a str,
        new_name: &'a str,
    }
    let args = serde_wasm_bindgen::to_value(&Args { old_name, new_name }).map_err(|e| e.to_string())?;
    invoke("cmd_rename_query", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to rename query".to_string()))?;
    Ok(())
}

/// Deletes a saved query by name.
pub async fn delete_query(name: &str) -> Result<(), String> {
    #[derive(Serialize)]
    struct Args<'a> {
        name: &'a str,
    }
    let args = serde_wasm_bindgen::to_value(&Args { name }).map_err(|e| e.to_string())?;
    invoke("cmd_delete_query", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to delete query".to_string()))?;
    Ok(())
}

/// Lists all saved queries.
pub async fn list_queries() -> Result<Vec<SavedQuery>, String> {
    let result = invoke("cmd_list_queries", JsValue::UNDEFINED)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to list queries".to_string()))?;
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse queries: {e}"))
}

/// Loads a single saved query by name.
pub async fn load_query(name: &str) -> Result<SavedQuery, String> {
    #[derive(Serialize)]
    struct Args<'a> {
        name: &'a str,
    }
    let args = serde_wasm_bindgen::to_value(&Args { name }).map_err(|e| e.to_string())?;
    let result = invoke("cmd_load_query", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to load query".to_string()))?;
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse query: {e}"))
}
