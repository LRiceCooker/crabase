use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["__TAURI__", "dialog"], js_name = "open", catch)]
    async fn dialog_open(options: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["__TAURI__", "event"], catch)]
    async fn listen(event: &str, handler: &JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,
    pub schema: String,
    pub sslmode: String,
}

pub fn build_connection_string_js(info: &ConnectionInfo) -> String {
    let password_part = if info.password.is_empty() {
        String::new()
    } else {
        format!(":{}", info.password)
    };
    format!(
        "postgresql://{}{}@{}:{}/{}?sslmode={}",
        info.user, password_part, info.host, info.port, info.dbname, info.sslmode
    )
}

pub async fn parse_connection_string(connection_string: &str) -> Result<ConnectionInfo, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        connection_string: &'a str,
    }

    let args = serde_wasm_bindgen::to_value(&Args { connection_string })
        .map_err(|e| e.to_string())?;

    let result = invoke("parse_connection_string", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to parse connection string".to_string()))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse response: {}", e))
}

pub async fn list_schemas(connection_string: &str) -> Result<Vec<String>, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        connection_string: &'a str,
    }

    let args = serde_wasm_bindgen::to_value(&Args { connection_string })
        .map_err(|e| e.to_string())?;

    let result = invoke("list_schemas", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to list schemas".to_string()))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse schemas: {}", e))
}

pub async fn connect_db(info: &ConnectionInfo) -> Result<String, String> {
    #[derive(Serialize)]
    struct Args<'a> {
        info: &'a ConnectionInfo,
    }

    let args = serde_wasm_bindgen::to_value(&Args { info })
        .map_err(|e| e.to_string())?;

    let result = invoke("connect_db", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Connection failed".to_string()))?;

    result
        .as_string()
        .ok_or_else(|| "Invalid response from backend".to_string())
}

pub async fn get_connection_info() -> Result<ConnectionInfo, String> {
    let result = invoke("get_connection_info", JsValue::UNDEFINED)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to get connection info".to_string()))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse connection info: {}", e))
}

pub async fn disconnect_db() -> Result<String, String> {
    let result = invoke("disconnect_db", JsValue::UNDEFINED)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to disconnect".to_string()))?;

    result
        .as_string()
        .ok_or_else(|| "Invalid response from backend".to_string())
}

pub async fn list_tables() -> Result<Vec<String>, String> {
    let result = invoke("list_tables", JsValue::UNDEFINED)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to list tables".to_string()))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse tables list: {}", e))
}

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
        .map_err(|e| format!("Failed to parse autocomplete columns: {}", e))
}

/// Opens a native file picker dialog filtered on .tar.gz files.
/// Returns the selected file path, or None if cancelled.
pub async fn pick_backup_file() -> Result<Option<String>, String> {
    #[derive(Serialize)]
    struct DialogFilter {
        name: String,
        extensions: Vec<String>,
    }

    #[derive(Serialize)]
    struct OpenDialogOptions {
        title: String,
        filters: Vec<DialogFilter>,
        multiple: bool,
        directory: bool,
    }

    let options = OpenDialogOptions {
        title: "Select a .tar.gz backup file".to_string(),
        filters: vec![DialogFilter {
            name: "PostgreSQL Backup".to_string(),
            extensions: vec!["gz".to_string()],
        }],
        multiple: false,
        directory: false,
    };

    let args = serde_wasm_bindgen::to_value(&options).map_err(|e| e.to_string())?;

    let result = dialog_open(args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to open file dialog".to_string()))?;

    if result.is_null() || result.is_undefined() {
        return Ok(None);
    }

    // The result is a file path string
    Ok(result.as_string())
}

pub async fn restore_backup(file_path: &str) -> Result<String, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        file_path: &'a str,
    }

    let args =
        serde_wasm_bindgen::to_value(&Args { file_path }).map_err(|e| e.to_string())?;

    let result = invoke("restore_backup", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Restore failed".to_string()))?;

    result
        .as_string()
        .ok_or_else(|| "Invalid response from backend".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConnection {
    pub name: String,
    pub info: ConnectionInfo,
}

pub async fn save_connection(name: &str, info: &ConnectionInfo) -> Result<(), String> {
    #[derive(Serialize)]
    struct Args<'a> {
        name: &'a str,
        info: &'a ConnectionInfo,
    }

    let args = serde_wasm_bindgen::to_value(&Args { name, info })
        .map_err(|e| e.to_string())?;

    invoke("save_connection", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to save connection".to_string()))?;

    Ok(())
}

pub async fn list_saved_connections() -> Result<Vec<SavedConnection>, String> {
    let result = invoke("list_saved_connections", JsValue::UNDEFINED)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to list saved connections".to_string()))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse saved connections: {}", e))
}

pub async fn delete_saved_connection(name: &str) -> Result<(), String> {
    #[derive(Serialize)]
    struct Args<'a> {
        name: &'a str,
    }

    let args = serde_wasm_bindgen::to_value(&Args { name })
        .map_err(|e| e.to_string())?;

    invoke("delete_saved_connection", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to delete saved connection".to_string()))?;

    Ok(())
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub total_count: u64,
}

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
        .map_err(|e| format!("Failed to parse table data: {}", e))
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
        .map_err(|e| format!("Failed to parse table data: {}", e))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowUpdate {
    pub pk_values: std::collections::HashMap<String, serde_json::Value>,
    pub changes: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowInsert {
    pub values: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowDelete {
    pub pk_values: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub updates: Vec<RowUpdate>,
    pub inserts: Vec<RowInsert>,
    pub deletes: Vec<RowDelete>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StatementResult {
    Rows { columns: Vec<String>, rows: Vec<Vec<serde_json::Value>>, sql_preview: String },
    Affected { command: String, rows_affected: u64, sql_preview: String },
    Error { message: String, sql_preview: String },
}

pub async fn execute_query(sql: &str) -> Result<QueryResult, String> {
    #[derive(Serialize)]
    struct Args<'a> {
        sql: &'a str,
    }

    let args = serde_wasm_bindgen::to_value(&Args { sql })
        .map_err(|e| e.to_string())?;

    let result = invoke("execute_query", args)
        .await
        .map_err(|e| {
            e.as_string()
                .unwrap_or_else(|| "Query execution failed".to_string())
        })?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse query result: {}", e))
}

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
        .map_err(|e| format!("Failed to parse multi-statement result: {}", e))
}

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

/// Listen to restore-log events. Returns a JS function to call to unlisten.
// --- Settings ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: Theme,
}

pub async fn load_settings() -> Result<Settings, String> {
    let result = invoke("load_settings", JsValue::UNDEFINED)
        .await
        .map_err(|e| {
            e.as_string()
                .unwrap_or_else(|| "Failed to load settings".to_string())
        })?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse settings: {}", e))
}

pub async fn save_settings(settings: &Settings) -> Result<(), String> {
    #[derive(Serialize)]
    struct Args<'a> {
        settings: &'a Settings,
    }

    let args = serde_wasm_bindgen::to_value(&Args { settings })
        .map_err(|e| e.to_string())?;

    invoke("save_settings", args)
        .await
        .map_err(|e| {
            e.as_string()
                .unwrap_or_else(|| "Failed to save settings".to_string())
        })?;

    Ok(())
}

// --- Saved Queries ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedQuery {
    pub name: String,
    pub sql: String,
}

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

pub async fn list_queries() -> Result<Vec<SavedQuery>, String> {
    let result = invoke("cmd_list_queries", JsValue::UNDEFINED)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to list queries".to_string()))?;
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse queries: {}", e))
}

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
        .map_err(|e| format!("Failed to parse query: {}", e))
}

pub async fn open_new_window() -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).map_err(|e| e.to_string())?;
    invoke("open_new_window", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to open window".to_string()))?;
    Ok(())
}

pub async fn listen_restore_logs(
    callback: impl Fn(String) + 'static,
) -> Result<js_sys::Function, String> {
    let closure = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
        if let Ok(payload) = js_sys::Reflect::get(&event, &JsValue::from_str("payload")) {
            if let Some(line) = payload.as_string() {
                callback(line);
            }
        }
    });

    let unlisten = listen("restore-log", closure.as_ref())
        .await
        .map_err(|e| {
            e.as_string()
                .unwrap_or_else(|| "Failed to listen for events".to_string())
        })?;

    closure.forget();

    Ok(unlisten.unchecked_into::<js_sys::Function>())
}
