use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use super::{dialog_open, dialog_save, invoke, listen};

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

/// Restores a PostgreSQL database from a .tar.gz backup file.
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

/// Opens a save file dialog and writes content to the chosen path.
pub async fn save_file_dialog(default_name: &str, content: &str) -> Result<(), String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct SaveDialogOptions<'a> {
        default_path: &'a str,
    }

    let options = SaveDialogOptions { default_path: default_name };
    let args = serde_wasm_bindgen::to_value(&options).map_err(|e| e.to_string())?;

    let result = dialog_save(args).await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Save dialog failed".to_string()))?;

    if result.is_null() || result.is_undefined() {
        return Ok(()); // User cancelled
    }

    let path = result.as_string().ok_or("Invalid path")?;

    // Write file via backend
    #[derive(Serialize)]
    struct WriteArgs<'a> {
        path: &'a str,
        content: &'a str,
    }
    let write_args = serde_wasm_bindgen::to_value(&WriteArgs { path: &path, content }).map_err(|e| e.to_string())?;
    invoke("write_file", write_args).await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Write failed".to_string()))?;
    Ok(())
}

/// Listens for restore-log events. Returns a JS function to call to unlisten.
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

/// Sets the application tray icon (dark or light variant).
pub async fn set_app_icon(is_dark: bool) -> Result<(), String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args { is_dark: bool }
    let args = serde_wasm_bindgen::to_value(&Args { is_dark }).map_err(|e| e.to_string())?;
    invoke("set_app_icon", args).await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Failed to set icon".to_string()))?;
    Ok(())
}
