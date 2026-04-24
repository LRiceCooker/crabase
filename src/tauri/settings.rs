use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::invoke;

/// Application theme options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}

/// Application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: Theme,
}

/// Loads the current application settings from the backend.
pub async fn load_settings() -> Result<Settings, String> {
    let result = invoke("load_settings", JsValue::UNDEFINED)
        .await
        .map_err(|e| {
            e.as_string()
                .unwrap_or_else(|| "Failed to load settings".to_string())
        })?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse settings: {e}"))
}

/// Saves application settings to the backend.
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
