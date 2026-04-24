use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::invoke;

/// Connection metadata for a PostgreSQL database.
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

/// A named saved connection (name + connection details).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConnection {
    pub name: String,
    pub info: ConnectionInfo,
}

/// Builds a PostgreSQL connection URL from structured connection info.
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

/// Parses a PostgreSQL connection string into structured `ConnectionInfo`.
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
        .map_err(|e| format!("Failed to parse response: {e}"))
}

/// Lists available schemas for a given connection string.
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
        .map_err(|e| format!("Failed to parse schemas: {e}"))
}

/// Connects to a PostgreSQL database using the given connection info.
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

/// Retrieves the current active connection info.
pub async fn get_connection_info() -> Result<ConnectionInfo, String> {
    let result = invoke("get_connection_info", JsValue::UNDEFINED)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to get connection info".to_string()))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse connection info: {e}"))
}

/// Disconnects from the current database.
pub async fn disconnect_db() -> Result<String, String> {
    let result = invoke("disconnect_db", JsValue::UNDEFINED)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to disconnect".to_string()))?;

    result
        .as_string()
        .ok_or_else(|| "Invalid response from backend".to_string())
}

/// Saves a named connection for later reuse.
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

/// Lists all saved connections.
pub async fn list_saved_connections() -> Result<Vec<SavedConnection>, String> {
    let result = invoke("list_saved_connections", JsValue::UNDEFINED)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to list saved connections".to_string()))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse saved connections: {e}"))
}

/// Deletes a saved connection by name.
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
