use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub theme: Theme,
}

fn settings_file(app_handle: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to resolve app data dir: {e}")))?;
    fs::create_dir_all(&data_dir)
        .map_err(|e| AppError::io("Failed to create app data dir", e))?;
    Ok(data_dir.join("settings.json"))
}

pub fn load_settings(app_handle: &tauri::AppHandle) -> Result<Settings, AppError> {
    let path = settings_file(app_handle)?;
    if !path.exists() {
        return Ok(Settings::default());
    }
    let data = fs::read_to_string(&path)
        .map_err(|e| AppError::io("Failed to read settings", e))?;
    serde_json::from_str(&data).map_err(|e| AppError::json("Failed to parse settings", e))
}

pub fn save_settings(app_handle: &tauri::AppHandle, settings: &Settings) -> Result<(), AppError> {
    let path = settings_file(app_handle)?;
    let data = serde_json::to_string_pretty(settings)
        .map_err(|e| AppError::json("Failed to serialize settings", e))?;
    fs::write(&path, data).map_err(|e| AppError::io("Failed to write settings", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        matches!(settings.theme, Theme::Light);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = Settings {
            theme: Theme::Dark,
        };
        let json = serde_json::to_string(&settings).unwrap();
        let parsed: Settings = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed.theme, Theme::Dark));
    }

    #[test]
    fn test_settings_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        let settings = Settings {
            theme: Theme::System,
        };

        let data = serde_json::to_string_pretty(&settings).unwrap();
        fs::write(&path, &data).unwrap();

        let loaded: Settings =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(matches!(loaded.theme, Theme::System));
    }

    #[test]
    fn test_settings_deserialize_missing_fields() {
        // Empty JSON object should use defaults
        let json = "{}";
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert!(matches!(settings.theme, Theme::Light));
    }

    #[test]
    fn test_theme_rename_all() {
        let settings = Settings {
            theme: Theme::Dark,
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"dark\""));

        let settings = Settings {
            theme: Theme::Light,
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"light\""));

        let settings = Settings {
            theme: Theme::System,
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"system\""));
    }
}
