use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crabase::db::{self, DbState};
use crabase::saved_connections::SavedConnection;
use crabase::saved_queries::{self, SavedQuery};
use crabase::settings::Settings;

/// Shared application state for the test server.
struct AppState {
    db: DbState,
    /// In-memory store for saved queries (connection_key -> queries).
    queries: RwLock<HashMap<String, Vec<SavedQuery>>>,
    /// In-memory store for saved connections.
    connections: RwLock<Vec<SavedConnection>>,
    /// In-memory settings.
    settings: RwLock<Settings>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        db: DbState::new(),
        queries: RwLock::new(HashMap::new()),
        connections: RwLock::new(Vec::new()),
        settings: RwLock::new(Settings::default()),
    });

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(
            "http://localhost:8080".parse().unwrap(),
        ))
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/invoke/{command}", post(handle_invoke))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .expect("Failed to bind to port 3001");

    println!("Test server listening on http://127.0.0.1:3001");
    axum::serve(listener, app).await.unwrap();
}

async fn handle_invoke(
    Path(command): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let result = dispatch(&command, &body, &state).await;
    match result {
        Ok(val) => Ok(Json(val)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": err })),
        )),
    }
}

async fn dispatch(command: &str, body: &Value, state: &AppState) -> Result<Value, String> {
    match command {
        "parse_connection_string" => {
            let cs = body
                .get("connectionString")
                .or_else(|| body.get("connection_string"))
                .and_then(|v| v.as_str())
                .ok_or("Missing connectionString")?;
            let info = db::parse_connection_string(cs)?;
            Ok(serde_json::to_value(info).unwrap())
        }

        "connect_db" => {
            let info: db::ConnectionInfo = serde_json::from_value(
                body.get("info")
                    .cloned()
                    .ok_or("Missing info")?,
            )
            .map_err(|e| format!("Invalid info: {e}"))?;
            state.db.connect(info).await?;
            Ok(serde_json::json!("Connected successfully"))
        }

        "disconnect_db" => {
            state.db.disconnect().await?;
            Ok(serde_json::json!("Disconnected successfully"))
        }

        "get_connection_info" => {
            let info = state.db.get_connection_info().await?;
            Ok(serde_json::to_value(info).unwrap())
        }

        "list_schemas" => {
            let cs = body
                .get("connectionString")
                .or_else(|| body.get("connection_string"))
                .and_then(|v| v.as_str())
                .ok_or("Missing connectionString")?;
            let schemas = db::list_schemas(cs).await?;
            Ok(serde_json::to_value(schemas).unwrap())
        }

        "list_tables" => {
            let tables = state.db.list_tables().await?;
            Ok(serde_json::to_value(tables).unwrap())
        }

        "get_column_info" => {
            let table_name = body
                .get("tableName")
                .or_else(|| body.get("table_name"))
                .and_then(|v| v.as_str())
                .ok_or("Missing tableName")?;
            let cols = state.db.get_column_info(table_name).await?;
            Ok(serde_json::to_value(cols).unwrap())
        }

        "get_table_data" => {
            let table_name = body
                .get("tableName")
                .or_else(|| body.get("table_name"))
                .and_then(|v| v.as_str())
                .ok_or("Missing tableName")?;
            let page = body
                .get("page")
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as u32;
            let page_size = body
                .get("pageSize")
                .or_else(|| body.get("page_size"))
                .and_then(|v| v.as_u64())
                .unwrap_or(50) as u32;
            let data = state.db.get_table_data(table_name, page, page_size).await?;
            Ok(serde_json::to_value(data).unwrap())
        }

        "get_table_data_filtered" => {
            let table_name = body
                .get("tableName")
                .or_else(|| body.get("table_name"))
                .and_then(|v| v.as_str())
                .ok_or("Missing tableName")?;
            let page = body
                .get("page")
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as u32;
            let page_size = body
                .get("pageSize")
                .or_else(|| body.get("page_size"))
                .and_then(|v| v.as_u64())
                .unwrap_or(50) as u32;
            let filters: Vec<db::Filter> = serde_json::from_value(
                body.get("filters").cloned().unwrap_or(Value::Array(vec![])),
            )
            .map_err(|e| format!("Invalid filters: {e}"))?;
            let sort: Vec<db::SortCol> = serde_json::from_value(
                body.get("sort").cloned().unwrap_or(Value::Array(vec![])),
            )
            .map_err(|e| format!("Invalid sort: {e}"))?;
            let data = state
                .db
                .get_table_data_filtered(table_name, page, page_size, filters, sort)
                .await?;
            Ok(serde_json::to_value(data).unwrap())
        }

        "save_changes" => {
            let table_name = body
                .get("tableName")
                .or_else(|| body.get("table_name"))
                .and_then(|v| v.as_str())
                .ok_or("Missing tableName")?;
            let changes: db::ChangeSet = serde_json::from_value(
                body.get("changes").cloned().ok_or("Missing changes")?,
            )
            .map_err(|e| format!("Invalid changes: {e}"))?;
            let msg = state.db.save_changes(table_name, changes).await?;
            Ok(serde_json::json!(msg))
        }

        "execute_query" => {
            let sql = body
                .get("sql")
                .and_then(|v| v.as_str())
                .ok_or("Missing sql")?;
            let result = state.db.execute_query(sql).await?;
            Ok(serde_json::to_value(result).unwrap())
        }

        "execute_query_multi" => {
            let sql = body
                .get("sql")
                .and_then(|v| v.as_str())
                .ok_or("Missing sql")?;
            let results = state.db.execute_query_multi(sql).await?;
            Ok(serde_json::to_value(results).unwrap())
        }

        "drop_table" => {
            let table_name = body
                .get("tableName")
                .or_else(|| body.get("table_name"))
                .and_then(|v| v.as_str())
                .ok_or("Missing tableName")?;
            let msg = state.db.drop_table(table_name).await?;
            Ok(serde_json::json!(msg))
        }

        "truncate_table" => {
            let table_name = body
                .get("tableName")
                .or_else(|| body.get("table_name"))
                .and_then(|v| v.as_str())
                .ok_or("Missing tableName")?;
            let msg = state.db.truncate_table(table_name).await?;
            Ok(serde_json::json!(msg))
        }

        "export_table_json" => {
            let table_name = body
                .get("tableName")
                .or_else(|| body.get("table_name"))
                .and_then(|v| v.as_str())
                .ok_or("Missing tableName")?;
            let json_str = state.db.export_table_json(table_name).await?;
            Ok(serde_json::json!(json_str))
        }

        "export_table_sql" => {
            let table_name = body
                .get("tableName")
                .or_else(|| body.get("table_name"))
                .and_then(|v| v.as_str())
                .ok_or("Missing tableName")?;
            let sql_str = state.db.export_table_sql(table_name).await?;
            Ok(serde_json::json!(sql_str))
        }

        "get_columns_for_autocomplete" => {
            let table_names: Vec<String> = serde_json::from_value(
                body.get("tableNames")
                    .or_else(|| body.get("table_names"))
                    .cloned()
                    .ok_or("Missing tableNames")?,
            )
            .map_err(|e| format!("Invalid tableNames: {e}"))?;
            let result = state.db.get_columns_for_autocomplete(&table_names).await?;
            Ok(serde_json::to_value(result).unwrap())
        }

        "get_full_schema_for_chat" => {
            let schema = state.db.get_full_schema_text().await?;
            Ok(serde_json::json!(schema))
        }

        // --- File-based commands (no tauri::AppHandle, use in-memory state) ---

        "load_settings" => {
            let settings = state.settings.read().await;
            Ok(serde_json::to_value(&*settings).unwrap())
        }

        "save_settings" => {
            let new_settings: Settings = serde_json::from_value(
                body.get("settings").cloned().ok_or("Missing settings")?,
            )
            .map_err(|e| format!("Invalid settings: {e}"))?;
            let mut settings = state.settings.write().await;
            *settings = new_settings;
            Ok(Value::Null)
        }

        "save_connection" => {
            let name = body
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing name")?
                .to_string();
            let info: db::ConnectionInfo = serde_json::from_value(
                body.get("info").cloned().ok_or("Missing info")?,
            )
            .map_err(|e| format!("Invalid info: {e}"))?;
            if name.trim().is_empty() {
                return Err("Connection name cannot be empty".to_string());
            }
            let mut connections = state.connections.write().await;
            connections.retain(|c| c.name != name);
            connections.push(SavedConnection { name, info });
            Ok(Value::Null)
        }

        "list_saved_connections" => {
            let connections = state.connections.read().await;
            Ok(serde_json::to_value(&*connections).unwrap())
        }

        "delete_saved_connection" => {
            let name = body
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing name")?;
            let mut connections = state.connections.write().await;
            connections.retain(|c| c.name != name);
            Ok(Value::Null)
        }

        "save_query" | "cmd_save_query" => {
            let name = body
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing name")?
                .to_string();
            let sql = body
                .get("sql")
                .and_then(|v| v.as_str())
                .ok_or("Missing sql")?
                .to_string();
            if name.trim().is_empty() {
                return Err("Query name cannot be empty".to_string());
            }
            let conn_key = get_conn_key(&state.db).await?;
            let mut store = state.queries.write().await;
            let queries = store.entry(conn_key).or_default();
            if queries.iter().any(|q| q.name == name) {
                return Err(format!("A query named '{}' already exists", name));
            }
            queries.push(SavedQuery { name, sql });
            Ok(Value::Null)
        }

        "update_query" | "cmd_update_query" => {
            let name = body
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing name")?;
            let sql = body
                .get("sql")
                .and_then(|v| v.as_str())
                .ok_or("Missing sql")?
                .to_string();
            let conn_key = get_conn_key(&state.db).await?;
            let mut store = state.queries.write().await;
            let queries = store.entry(conn_key).or_default();
            let query = queries
                .iter_mut()
                .find(|q| q.name == name)
                .ok_or_else(|| format!("Query '{}' not found", name))?;
            query.sql = sql;
            Ok(Value::Null)
        }

        "rename_query" | "cmd_rename_query" => {
            let old_name = body
                .get("oldName")
                .or_else(|| body.get("old_name"))
                .and_then(|v| v.as_str())
                .ok_or("Missing oldName")?;
            let new_name = body
                .get("newName")
                .or_else(|| body.get("new_name"))
                .and_then(|v| v.as_str())
                .ok_or("Missing newName")?
                .to_string();
            if new_name.trim().is_empty() {
                return Err("Query name cannot be empty".to_string());
            }
            let conn_key = get_conn_key(&state.db).await?;
            let mut store = state.queries.write().await;
            let queries = store.entry(conn_key).or_default();
            if queries.iter().any(|q| q.name == new_name) {
                return Err(format!("A query named '{}' already exists", new_name));
            }
            let query = queries
                .iter_mut()
                .find(|q| q.name == old_name)
                .ok_or_else(|| format!("Query '{}' not found", old_name))?;
            query.name = new_name;
            Ok(Value::Null)
        }

        "delete_query" | "cmd_delete_query" => {
            let name = body
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing name")?;
            let conn_key = get_conn_key(&state.db).await?;
            let mut store = state.queries.write().await;
            let queries = store.entry(conn_key).or_default();
            let original_len = queries.len();
            queries.retain(|q| q.name != name);
            if queries.len() == original_len {
                return Err(format!("Query '{}' not found", name));
            }
            Ok(Value::Null)
        }

        "list_queries" | "cmd_list_queries" => {
            let conn_key = get_conn_key(&state.db).await?;
            let store = state.queries.read().await;
            let queries = store.get(&conn_key).cloned().unwrap_or_default();
            Ok(serde_json::to_value(queries).unwrap())
        }

        "load_query" | "cmd_load_query" => {
            let name = body
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing name")?;
            let conn_key = get_conn_key(&state.db).await?;
            let store = state.queries.read().await;
            let queries = store
                .get(&conn_key)
                .ok_or("No saved queries for this connection")?;
            let query = queries
                .iter()
                .find(|q| q.name == name)
                .cloned()
                .ok_or_else(|| format!("Query '{}' not found", name))?;
            Ok(serde_json::to_value(query).unwrap())
        }

        "write_file" => {
            let path = body
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing path")?;
            let content = body
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing content")?;
            fs::write(path, content)
                .map_err(|e| format!("Failed to write file: {e}"))?;
            Ok(Value::Null)
        }

        _ => Err(format!("Unknown command: {command}")),
    }
}

/// Derive connection key from current DB state (mirrors lib.rs logic).
async fn get_conn_key(db: &DbState) -> Result<String, String> {
    let info = db.get_connection_info().await?;
    Ok(saved_queries::connection_key(
        &info.host, info.port, &info.dbname, &info.user,
    ))
}
