// Clippy pedantic: suppress documentation and minor style lints for internal desktop app
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::doc_markdown,
    clippy::struct_excessive_bools,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::items_after_statements,
    clippy::uninlined_format_args,
    clippy::semicolon_if_nothing_returned,
    clippy::needless_pass_by_value,
    clippy::redundant_closure_for_method_calls,
    clippy::match_same_arms,
    clippy::if_not_else,
    clippy::needless_raw_string_hashes,
    clippy::inefficient_to_string,
    clippy::format_push_string,
    clippy::type_complexity,
    clippy::map_unwrap_or,
    clippy::if_same_then_else,
    clippy::lines_filter_map_ok,
    clippy::derivable_impls
)]

mod app_icon;
mod claude;
pub mod db;
mod restore;
pub mod saved_connections;
pub mod saved_queries;
pub mod settings;

// ── Connection commands ─────────────────────────────────────────────

/// Parse a PostgreSQL connection string into its component parts.
#[tauri::command]
fn parse_connection_string(connection_string: String) -> Result<db::ConnectionInfo, String> {
    db::parse_connection_string(&connection_string)
}

/// List all schemas available on the PostgreSQL server at the given connection string.
#[tauri::command]
async fn list_schemas(connection_string: String) -> Result<Vec<String>, String> {
    db::list_schemas(&connection_string).await
}

/// Connect to a PostgreSQL database using the provided connection info.
#[tauri::command]
async fn connect_db(
    info: db::ConnectionInfo,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<String, String> {
    db_state.connect(info).await?;
    Ok("Connected successfully".to_string())
}

/// Disconnect from the current PostgreSQL database.
#[tauri::command]
async fn disconnect_db(db_state: tauri::State<'_, db::DbState>) -> Result<String, String> {
    db_state.disconnect().await?;
    Ok("Disconnected successfully".to_string())
}

/// Return the connection info for the currently connected database.
#[tauri::command]
async fn get_connection_info(
    db_state: tauri::State<'_, db::DbState>,
) -> Result<db::ConnectionInfo, String> {
    db_state.get_connection_info().await
}

// ── Schema & column commands ────────────────────────────────────────

/// List all table names in the current schema.
#[tauri::command]
async fn list_tables(
    db_state: tauri::State<'_, db::DbState>,
) -> Result<Vec<String>, String> {
    db_state.list_tables().await
}

/// Return column metadata (name, type, nullable, default, PK) for a table.
#[tauri::command]
async fn get_column_info(
    table_name: String,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<Vec<db::ColumnInfo>, String> {
    db_state.get_column_info(&table_name).await
}

/// Return column names for each given table, used for SQL editor autocomplete.
#[tauri::command]
async fn get_columns_for_autocomplete(
    table_names: Vec<String>,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<std::collections::HashMap<String, Vec<String>>, String> {
    db_state.get_columns_for_autocomplete(&table_names).await
}

/// Return the full schema DDL text for all tables, used as context for AI chat.
#[tauri::command]
async fn get_full_schema_for_chat(
    db_state: tauri::State<'_, db::DbState>,
) -> Result<String, String> {
    db_state.get_full_schema_text().await
}

// ── Table data & query commands ─────────────────────────────────────

/// Fetch paginated table data. Returns rows as JSON values plus total row count.
#[tauri::command]
async fn get_table_data(
    table_name: String,
    page: u32,
    page_size: u32,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<db::TableData, String> {
    db_state.get_table_data(&table_name, page, page_size).await
}

/// Fetch paginated table data with column filters and sort ordering.
#[tauri::command]
async fn get_table_data_filtered(
    table_name: String,
    page: u32,
    page_size: u32,
    filters: Vec<db::Filter>,
    sort: Vec<db::SortCol>,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<db::TableData, String> {
    db_state
        .get_table_data_filtered(&table_name, page, page_size, filters, sort)
        .await
}

/// Execute a single SQL statement and return the result set.
#[tauri::command]
async fn execute_query(
    sql: String,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<db::QueryResult, String> {
    db_state.execute_query(&sql).await
}

/// Execute multiple SQL statements (semicolon-separated) and return each result.
#[tauri::command]
async fn execute_query_multi(
    sql: String,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<Vec<db::StatementResult>, String> {
    db_state.execute_query_multi(&sql).await
}

/// Apply a set of row inserts, updates, and deletes to a table.
#[tauri::command]
async fn save_changes(
    table_name: String,
    changes: db::ChangeSet,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<String, String> {
    db_state.save_changes(&table_name, changes).await
}

// ── Table operations (drop, truncate, export) ───────────────────────

/// Drop a table from the database.
#[tauri::command]
async fn drop_table(table_name: String, db_state: tauri::State<'_, db::DbState>) -> Result<String, String> {
    db_state.drop_table(&table_name).await
}

/// Truncate (delete all rows from) a table.
#[tauri::command]
async fn truncate_table(table_name: String, db_state: tauri::State<'_, db::DbState>) -> Result<String, String> {
    db_state.truncate_table(&table_name).await
}

/// Export all rows of a table as a JSON string.
#[tauri::command]
async fn export_table_json(table_name: String, db_state: tauri::State<'_, db::DbState>) -> Result<String, String> {
    db_state.export_table_json(&table_name).await
}

/// Export a table as SQL INSERT statements.
#[tauri::command]
async fn export_table_sql(table_name: String, db_state: tauri::State<'_, db::DbState>) -> Result<String, String> {
    db_state.export_table_sql(&table_name).await
}

// ── Backup restore ──────────────────────────────────────────────────

/// Restore a .tar.gz backup file using pg_restore, streaming progress via events.
#[tauri::command]
async fn restore_backup(
    file_path: String,
    app_handle: tauri::AppHandle,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<String, String> {
    let connection_string = db_state.get_connection_string().await?;
    tokio::task::spawn_blocking(move || {
        restore::restore_backup_streaming(&file_path, &connection_string, &app_handle)
    })
    .await
    .map_err(|e| format!("Task failed: {e}"))?
}

// ── Saved connections ───────────────────────────────────────────────

/// Persist a named connection to the app data directory.
#[tauri::command]
fn save_connection(
    name: String,
    info: db::ConnectionInfo,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    saved_connections::save_connection(&app_handle, name, info)
}

/// List all saved connections from the app data directory.
#[tauri::command]
fn list_saved_connections(
    app_handle: tauri::AppHandle,
) -> Result<Vec<saved_connections::SavedConnection>, String> {
    saved_connections::list_saved_connections(&app_handle)
}

/// Delete a saved connection by name.
#[tauri::command]
fn delete_saved_connection(name: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    saved_connections::delete_saved_connection(&app_handle, &name)
}

// ── Saved queries ───────────────────────────────────────────────────

/// Derive a connection key (host:port:dbname:user) from the current DB state.
async fn get_conn_key(db_state: &db::DbState) -> Result<String, String> {
    let info = db_state.get_connection_info().await?;
    Ok(saved_queries::connection_key_from_info(&info))
}

/// Save a named SQL query for the current connection.
#[tauri::command]
async fn cmd_save_query(
    name: String,
    sql: String,
    db_state: tauri::State<'_, db::DbState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let key = get_conn_key(&db_state).await?;
    saved_queries::save_query(&app_handle, &key, name, sql)
}

/// Update the SQL of an existing saved query by name.
#[tauri::command]
async fn cmd_update_query(
    name: String,
    sql: String,
    db_state: tauri::State<'_, db::DbState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let key = get_conn_key(&db_state).await?;
    saved_queries::update_query(&app_handle, &key, &name, sql)
}

/// Rename a saved query.
#[tauri::command]
async fn cmd_rename_query(
    old_name: String,
    new_name: String,
    db_state: tauri::State<'_, db::DbState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let key = get_conn_key(&db_state).await?;
    saved_queries::rename_query(&app_handle, &key, &old_name, new_name)
}

/// Delete a saved query by name.
#[tauri::command]
async fn cmd_delete_query(
    name: String,
    db_state: tauri::State<'_, db::DbState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let key = get_conn_key(&db_state).await?;
    saved_queries::delete_query(&app_handle, &key, &name)
}

/// List all saved queries for the current connection.
#[tauri::command]
async fn cmd_list_queries(
    db_state: tauri::State<'_, db::DbState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<saved_queries::SavedQuery>, String> {
    let key = get_conn_key(&db_state).await?;
    saved_queries::list_queries(&app_handle, &key)
}

/// Load a single saved query by name for the current connection.
#[tauri::command]
async fn cmd_load_query(
    name: String,
    db_state: tauri::State<'_, db::DbState>,
    app_handle: tauri::AppHandle,
) -> Result<saved_queries::SavedQuery, String> {
    let key = get_conn_key(&db_state).await?;
    saved_queries::load_query(&app_handle, &key, &name)
}

// ── Settings ────────────────────────────────────────────────────────

/// Load app settings from the data directory, returning defaults if not found.
#[tauri::command]
fn load_settings(app_handle: tauri::AppHandle) -> Result<settings::Settings, String> {
    settings::load_settings(&app_handle)
}

/// Save app settings to the data directory.
#[tauri::command]
fn save_settings(
    settings: settings::Settings,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    settings::save_settings(&app_handle, &settings)
}

// ── Claude AI chat ──────────────────────────────────────────────────

/// Check whether the Claude CLI is installed and available on PATH.
#[tauri::command]
fn check_claude_installed() -> bool {
    claude::is_installed()
}

/// Send a prompt to Claude CLI and stream the response via Tauri events.
#[tauri::command]
async fn chat_with_claude(prompt: String, app: tauri::AppHandle) -> Result<(), String> {
    if !claude::is_installed() {
        return Err("Claude Code not found. Install it from claude.ai/code".to_string());
    }

    tokio::task::spawn_blocking(move || claude::run_streaming(&prompt, &app))
        .await
        .map_err(|e| format!("Task failed: {e}"))?
}

// ── Window & file commands ──────────────────────────────────────────

/// Set the app dock/taskbar icon to the light or dark variant.
#[tauri::command]
fn set_app_icon(is_dark: bool, app_handle: tauri::AppHandle) -> Result<(), String> {
    app_icon::set_icon(is_dark, &app_handle)
}

/// Open a new application window with the same content.
#[tauri::command]
async fn open_new_window(app: tauri::AppHandle) -> Result<(), String> {
    use std::sync::atomic::{AtomicU32, Ordering};
    static WINDOW_COUNTER: AtomicU32 = AtomicU32::new(2);
    let id = WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed);
    let label = format!("main-{id}");
    tauri::WebviewWindowBuilder::new(&app, &label, tauri::WebviewUrl::App("index.html".into()))
        .title("crabase")
        .inner_size(1200.0, 800.0)
        .background_color(tauri::webview::Color(10, 10, 10, 255))
        .build()
        .map_err(|e| format!("Failed to create window: {e}"))?;
    Ok(())
}

/// Write content to a file at the given path (used for table exports).
#[tauri::command]
fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, &content).map_err(|e| format!("Failed to write file: {e}"))
}

// ── App entry point ─────────────────────────────────────────────────

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(db::DbState::new())
        .invoke_handler(tauri::generate_handler![
            // Connection
            parse_connection_string, list_schemas, connect_db, disconnect_db, get_connection_info,
            // Schema & columns
            list_tables, get_column_info, get_columns_for_autocomplete, get_full_schema_for_chat,
            // Table data & queries
            get_table_data, get_table_data_filtered, execute_query, execute_query_multi, save_changes,
            // Table operations
            drop_table, truncate_table, export_table_json, export_table_sql,
            // Backup restore
            restore_backup,
            // Saved connections
            save_connection, list_saved_connections, delete_saved_connection,
            // Saved queries
            cmd_save_query, cmd_update_query, cmd_rename_query, cmd_delete_query, cmd_list_queries, cmd_load_query,
            // Settings
            load_settings, save_settings,
            // Claude AI
            check_claude_installed, chat_with_claude,
            // Window & file
            set_app_icon, open_new_window, write_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_connection_string_command() {
        let info = parse_connection_string("postgresql://user:pass@localhost:5432/mydb".to_string()).unwrap();
        assert_eq!(info.host, "localhost");
        assert_eq!(info.user, "user");
        assert_eq!(info.dbname, "mydb");
    }
}
