//! Frontend Tauri invoke wrappers, organized by domain.
//!
//! Re-exports everything so callers can use `crate::tauri::*` unchanged.

mod chat;
mod connection;
mod files;
mod queries;
mod settings;
mod tables;

// Re-export all public items from submodules
pub use chat::*;
pub use connection::*;
pub use files::*;
pub use queries::*;
pub use settings::*;
pub use tables::*;

use wasm_bindgen::prelude::*;

// --- FFI bindings shared by all submodules ---

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["__TAURI__", "dialog"], js_name = "open", catch)]
    async fn dialog_open(options: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["__TAURI__", "dialog"], js_name = "save", catch)]
    async fn dialog_save(options: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["__TAURI__", "event"], catch)]
    async fn listen(event: &str, handler: &JsValue) -> Result<JsValue, JsValue>;
}

// --- Misc functions that don't belong to a specific domain ---

/// Opens a new application window.
pub async fn open_new_window() -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).map_err(|e| e.to_string())?;
    invoke("open_new_window", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to open window".to_string()))?;
    Ok(())
}
