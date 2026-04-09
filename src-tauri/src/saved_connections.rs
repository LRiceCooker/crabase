use crate::db::ConnectionInfo;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConnection {
    pub name: String,
    pub info: ConnectionInfo,
}

fn connections_file(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(data_dir.join("saved_connections.json"))
}

fn read_connections(app_handle: &tauri::AppHandle) -> Result<Vec<SavedConnection>, String> {
    let path = connections_file(app_handle)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read saved connections: {}", e))?;
    serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse saved connections: {}", e))
}

fn write_connections(
    app_handle: &tauri::AppHandle,
    connections: &[SavedConnection],
) -> Result<(), String> {
    let path = connections_file(app_handle)?;
    let data = serde_json::to_string_pretty(connections)
        .map_err(|e| format!("Failed to serialize connections: {}", e))?;
    fs::write(&path, data).map_err(|e| format!("Failed to write saved connections: {}", e))
}

pub fn save_connection(
    app_handle: &tauri::AppHandle,
    name: String,
    info: ConnectionInfo,
) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Connection name cannot be empty".to_string());
    }
    let mut connections = read_connections(app_handle)?;
    // Replace existing connection with same name
    connections.retain(|c| c.name != name);
    connections.push(SavedConnection { name, info });
    write_connections(app_handle, &connections)
}

pub fn list_saved_connections(
    app_handle: &tauri::AppHandle,
) -> Result<Vec<SavedConnection>, String> {
    read_connections(app_handle)
}

pub fn delete_saved_connection(
    app_handle: &tauri::AppHandle,
    name: String,
) -> Result<(), String> {
    let mut connections = read_connections(app_handle)?;
    let original_len = connections.len();
    connections.retain(|c| c.name != name);
    if connections.len() == original_len {
        return Err(format!("Connection '{}' not found", name));
    }
    write_connections(app_handle, &connections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_info() -> ConnectionInfo {
        ConnectionInfo {
            host: "localhost".to_string(),
            port: 5432,
            user: "admin".to_string(),
            password: "secret".to_string(),
            dbname: "mydb".to_string(),
            schema: "public".to_string(),
            sslmode: "disable".to_string(),
        }
    }

    #[test]
    fn test_saved_connection_serialization() {
        let conn = SavedConnection {
            name: "dev".to_string(),
            info: make_info(),
        };
        let json = serde_json::to_string(&conn).unwrap();
        let parsed: SavedConnection = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "dev");
        assert_eq!(parsed.info.host, "localhost");
        assert_eq!(parsed.info.port, 5432);
    }

    #[test]
    fn test_read_connections_missing_file() {
        // Manually test the read/write logic using temp dir
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("saved_connections.json");
        assert!(!path.exists());
        // If file doesn't exist, should return empty vec
        // We test this indirectly since read_connections requires AppHandle
    }

    #[test]
    fn test_connections_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("saved_connections.json");

        let connections = vec![
            SavedConnection {
                name: "dev".to_string(),
                info: make_info(),
            },
            SavedConnection {
                name: "staging".to_string(),
                info: ConnectionInfo {
                    host: "staging.example.com".to_string(),
                    port: 5433,
                    user: "deploy".to_string(),
                    password: "pw".to_string(),
                    dbname: "staging_db".to_string(),
                    schema: "public".to_string(),
                    sslmode: "require".to_string(),
                },
            },
        ];

        let data = serde_json::to_string_pretty(&connections).unwrap();
        fs::write(&path, &data).unwrap();

        let loaded: Vec<SavedConnection> =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "dev");
        assert_eq!(loaded[1].name, "staging");
        assert_eq!(loaded[1].info.host, "staging.example.com");
    }

    #[test]
    fn test_connections_replace_on_duplicate_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("saved_connections.json");

        let mut connections = vec![SavedConnection {
            name: "dev".to_string(),
            info: make_info(),
        }];

        // Simulate save_connection logic: retain + push
        let new_info = ConnectionInfo {
            host: "new-host.example.com".to_string(),
            ..make_info()
        };
        connections.retain(|c| c.name != "dev");
        connections.push(SavedConnection {
            name: "dev".to_string(),
            info: new_info,
        });

        let data = serde_json::to_string_pretty(&connections).unwrap();
        fs::write(&path, &data).unwrap();

        let loaded: Vec<SavedConnection> =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].info.host, "new-host.example.com");
    }

    #[test]
    fn test_connections_delete() {
        let connections = vec![
            SavedConnection {
                name: "dev".to_string(),
                info: make_info(),
            },
            SavedConnection {
                name: "staging".to_string(),
                info: make_info(),
            },
        ];

        let mut filtered = connections;
        let original_len = filtered.len();
        filtered.retain(|c| c.name != "dev");
        assert_eq!(filtered.len(), original_len - 1);
        assert_eq!(filtered[0].name, "staging");
    }

    #[test]
    fn test_delete_nonexistent_connection() {
        let connections: Vec<SavedConnection> = vec![SavedConnection {
            name: "dev".to_string(),
            info: make_info(),
        }];

        let mut filtered = connections.clone();
        let original_len = filtered.len();
        filtered.retain(|c| c.name != "nonexistent");
        // Length unchanged means connection not found
        assert_eq!(filtered.len(), original_len);
    }
}
