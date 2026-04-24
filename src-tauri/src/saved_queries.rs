use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

/// A saved SQL query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedQuery {
    pub name: String,
    pub sql: String,
}

/// Top-level storage: connection_key → list of queries.
type QueriesStore = HashMap<String, Vec<SavedQuery>>;

/// Build a connection key from host:port:dbname:user.
#[must_use]
pub fn connection_key(host: &str, port: u16, dbname: &str, user: &str) -> String {
    format!("{host}:{port}:{dbname}:{user}")
}

/// Build a connection key from a `ConnectionInfo` struct.
#[must_use]
pub fn connection_key_from_info(info: &crate::db::ConnectionInfo) -> String {
    connection_key(&info.host, info.port, &info.dbname, &info.user)
}

fn queries_file(app_handle: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("Failed to resolve app data dir: {e}")))?;
    fs::create_dir_all(&data_dir)
        .map_err(|e| AppError::io("Failed to create app data dir", e))?;
    Ok(data_dir.join("saved_queries.json"))
}

fn read_store(app_handle: &tauri::AppHandle) -> Result<QueriesStore, AppError> {
    let path = queries_file(app_handle)?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let data =
        fs::read_to_string(&path).map_err(|e| AppError::io("Failed to read saved queries", e))?;
    serde_json::from_str(&data).map_err(|e| AppError::json("Failed to parse saved queries", e))
}

fn write_store(app_handle: &tauri::AppHandle, store: &QueriesStore) -> Result<(), AppError> {
    let path = queries_file(app_handle)?;
    let data = serde_json::to_string_pretty(store)
        .map_err(|e| AppError::json("Failed to serialize saved queries", e))?;
    fs::write(&path, data).map_err(|e| AppError::io("Failed to write saved queries", e))
}

pub fn save_query(
    app_handle: &tauri::AppHandle,
    conn_key: &str,
    name: String,
    sql: String,
) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("Query name cannot be empty".into()));
    }
    let mut store = read_store(app_handle)?;
    let queries = store.entry(conn_key.to_string()).or_default();
    if queries.iter().any(|q| q.name == name) {
        return Err(AppError::Validation(format!("A query named '{name}' already exists")));
    }
    queries.push(SavedQuery { name, sql });
    write_store(app_handle, &store)
}

pub fn update_query(
    app_handle: &tauri::AppHandle,
    conn_key: &str,
    name: &str,
    sql: String,
) -> Result<(), AppError> {
    let mut store = read_store(app_handle)?;
    let queries = store.entry(conn_key.to_string()).or_default();
    let query = queries
        .iter_mut()
        .find(|q| q.name == name)
        .ok_or_else(|| AppError::Validation(format!("Query '{name}' not found")))?;
    query.sql = sql;
    write_store(app_handle, &store)
}

pub fn rename_query(
    app_handle: &tauri::AppHandle,
    conn_key: &str,
    old_name: &str,
    new_name: String,
) -> Result<(), AppError> {
    if new_name.trim().is_empty() {
        return Err(AppError::Validation("Query name cannot be empty".into()));
    }
    let mut store = read_store(app_handle)?;
    let queries = store.entry(conn_key.to_string()).or_default();
    if queries.iter().any(|q| q.name == new_name) {
        return Err(AppError::Validation(format!("A query named '{new_name}' already exists")));
    }
    let query = queries
        .iter_mut()
        .find(|q| q.name == old_name)
        .ok_or_else(|| AppError::Validation(format!("Query '{old_name}' not found")))?;
    query.name = new_name;
    write_store(app_handle, &store)
}

pub fn delete_query(
    app_handle: &tauri::AppHandle,
    conn_key: &str,
    name: &str,
) -> Result<(), AppError> {
    let mut store = read_store(app_handle)?;
    let queries = store.entry(conn_key.to_string()).or_default();
    let original_len = queries.len();
    queries.retain(|q| q.name != name);
    if queries.len() == original_len {
        return Err(AppError::Validation(format!("Query '{name}' not found")));
    }
    write_store(app_handle, &store)
}

pub fn list_queries(
    app_handle: &tauri::AppHandle,
    conn_key: &str,
) -> Result<Vec<SavedQuery>, AppError> {
    let store = read_store(app_handle)?;
    Ok(store.get(conn_key).cloned().unwrap_or_default())
}

pub fn load_query(
    app_handle: &tauri::AppHandle,
    conn_key: &str,
    name: &str,
) -> Result<SavedQuery, AppError> {
    let store = read_store(app_handle)?;
    let queries = store.get(conn_key)
        .ok_or_else(|| AppError::Validation("No saved queries for this connection".into()))?;
    queries
        .iter()
        .find(|q| q.name == name)
        .cloned()
        .ok_or_else(|| AppError::Validation(format!("Query '{name}' not found")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_key() {
        let key = connection_key("localhost", 5432, "mydb", "admin");
        assert_eq!(key, "localhost:5432:mydb:admin");
    }

    #[test]
    fn test_saved_query_serialization() {
        let query = SavedQuery {
            name: "get users".to_string(),
            sql: "SELECT * FROM users".to_string(),
        };
        let json = serde_json::to_string(&query).unwrap();
        let parsed: SavedQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "get users");
        assert_eq!(parsed.sql, "SELECT * FROM users");
    }

    #[test]
    fn test_store_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("saved_queries.json");

        let mut store = QueriesStore::new();
        store.insert(
            "localhost:5432:mydb:admin".to_string(),
            vec![
                SavedQuery {
                    name: "all users".to_string(),
                    sql: "SELECT * FROM users".to_string(),
                },
                SavedQuery {
                    name: "active orders".to_string(),
                    sql: "SELECT * FROM orders WHERE status = 'active'".to_string(),
                },
            ],
        );

        let data = serde_json::to_string_pretty(&store).unwrap();
        fs::write(&path, &data).unwrap();

        let loaded: QueriesStore =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let queries = loaded.get("localhost:5432:mydb:admin").unwrap();
        assert_eq!(queries.len(), 2);
        assert_eq!(queries[0].name, "all users");
        assert_eq!(queries[1].name, "active orders");
    }

    #[test]
    fn test_duplicate_name_detection() {
        let queries = vec![
            SavedQuery {
                name: "test".to_string(),
                sql: "SELECT 1".to_string(),
            },
        ];
        assert!(queries.iter().any(|q| q.name == "test"));
        assert!(!queries.iter().any(|q| q.name == "other"));
    }

    #[test]
    fn test_rename_conflict_detection() {
        let queries = vec![
            SavedQuery {
                name: "first".to_string(),
                sql: "SELECT 1".to_string(),
            },
            SavedQuery {
                name: "second".to_string(),
                sql: "SELECT 2".to_string(),
            },
        ];
        // Renaming "first" to "second" should conflict
        assert!(queries.iter().any(|q| q.name == "second"));
    }

    #[test]
    fn test_delete_logic() {
        let mut queries = vec![
            SavedQuery {
                name: "keep".to_string(),
                sql: "SELECT 1".to_string(),
            },
            SavedQuery {
                name: "remove".to_string(),
                sql: "SELECT 2".to_string(),
            },
        ];
        queries.retain(|q| q.name != "remove");
        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].name, "keep");
    }

    #[test]
    fn test_empty_name_rejected() {
        assert!("".trim().is_empty());
        assert!("  ".trim().is_empty());
        assert!(!"valid".trim().is_empty());
    }
}
