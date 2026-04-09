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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
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

/// Listen to restore-log events. Returns a JS function to call to unlisten.
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
