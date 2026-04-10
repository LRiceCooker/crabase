mod db;
mod restore;
mod saved_connections;
mod saved_queries;
mod settings;

#[tauri::command]
fn parse_connection_string(connection_string: String) -> Result<db::ConnectionInfo, String> {
    db::parse_connection_string(&connection_string)
}

#[tauri::command]
async fn list_schemas(connection_string: String) -> Result<Vec<String>, String> {
    db::list_schemas(&connection_string).await
}

#[tauri::command]
async fn connect_db(
    info: db::ConnectionInfo,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<String, String> {
    db_state.connect(info).await?;
    Ok("Connected successfully".to_string())
}

#[tauri::command]
fn disconnect_db(db_state: tauri::State<'_, db::DbState>) -> Result<String, String> {
    db_state.disconnect()?;
    Ok("Disconnected successfully".to_string())
}

#[tauri::command]
fn get_connection_info(
    db_state: tauri::State<'_, db::DbState>,
) -> Result<db::ConnectionInfo, String> {
    db_state.get_connection_info()
}

#[tauri::command]
async fn list_tables(
    db_state: tauri::State<'_, db::DbState>,
) -> Result<Vec<String>, String> {
    db_state.list_tables().await
}

#[tauri::command]
async fn get_column_info(
    table_name: String,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<Vec<db::ColumnInfo>, String> {
    db_state.get_column_info(&table_name).await
}

#[tauri::command]
async fn get_table_data(
    table_name: String,
    page: u32,
    page_size: u32,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<db::TableData, String> {
    db_state.get_table_data(&table_name, page, page_size).await
}

#[tauri::command]
async fn execute_query(
    sql: String,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<db::QueryResult, String> {
    db_state.execute_query(&sql).await
}

#[tauri::command]
async fn save_changes(
    table_name: String,
    changes: db::ChangeSet,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<String, String> {
    db_state.save_changes(&table_name, changes).await
}

#[tauri::command]
async fn restore_backup(
    file_path: String,
    app_handle: tauri::AppHandle,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<String, String> {
    let connection_string = db_state.get_connection_string()?;
    // Run blocking I/O (tar extraction + pg_restore subprocess) off the async runtime
    tokio::task::spawn_blocking(move || {
        restore::restore_backup_streaming(&file_path, &connection_string, &app_handle)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
fn save_connection(
    name: String,
    info: db::ConnectionInfo,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    saved_connections::save_connection(&app_handle, name, info)
}

#[tauri::command]
fn list_saved_connections(
    app_handle: tauri::AppHandle,
) -> Result<Vec<saved_connections::SavedConnection>, String> {
    saved_connections::list_saved_connections(&app_handle)
}

#[tauri::command]
fn delete_saved_connection(name: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    saved_connections::delete_saved_connection(&app_handle, name)
}

#[tauri::command]
fn load_settings(app_handle: tauri::AppHandle) -> Result<settings::Settings, String> {
    settings::load_settings(&app_handle)
}

#[tauri::command]
async fn get_columns_for_autocomplete(
    table_names: Vec<String>,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<std::collections::HashMap<String, Vec<String>>, String> {
    db_state.get_columns_for_autocomplete(&table_names).await
}

#[tauri::command]
fn save_settings(
    settings: settings::Settings,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    settings::save_settings(&app_handle, &settings)
}

/// Helper: derive connection key from current DB state.
fn get_conn_key(db_state: &db::DbState) -> Result<String, String> {
    let info = db_state.get_connection_info()?;
    Ok(saved_queries::connection_key(
        &info.host, info.port, &info.dbname, &info.user,
    ))
}

#[tauri::command]
fn cmd_save_query(
    name: String,
    sql: String,
    db_state: tauri::State<'_, db::DbState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let key = get_conn_key(&db_state)?;
    saved_queries::save_query(&app_handle, &key, name, sql)
}

#[tauri::command]
fn cmd_update_query(
    name: String,
    sql: String,
    db_state: tauri::State<'_, db::DbState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let key = get_conn_key(&db_state)?;
    saved_queries::update_query(&app_handle, &key, &name, sql)
}

#[tauri::command]
fn cmd_rename_query(
    old_name: String,
    new_name: String,
    db_state: tauri::State<'_, db::DbState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let key = get_conn_key(&db_state)?;
    saved_queries::rename_query(&app_handle, &key, &old_name, new_name)
}

#[tauri::command]
fn cmd_delete_query(
    name: String,
    db_state: tauri::State<'_, db::DbState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let key = get_conn_key(&db_state)?;
    saved_queries::delete_query(&app_handle, &key, &name)
}

#[tauri::command]
fn cmd_list_queries(
    db_state: tauri::State<'_, db::DbState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<saved_queries::SavedQuery>, String> {
    let key = get_conn_key(&db_state)?;
    saved_queries::list_queries(&app_handle, &key)
}

#[tauri::command]
fn cmd_load_query(
    name: String,
    db_state: tauri::State<'_, db::DbState>,
    app_handle: tauri::AppHandle,
) -> Result<saved_queries::SavedQuery, String> {
    let key = get_conn_key(&db_state)?;
    saved_queries::load_query(&app_handle, &key, &name)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(db::DbState::new())
        .invoke_handler(tauri::generate_handler![parse_connection_string, list_schemas, connect_db, disconnect_db, get_connection_info, list_tables, get_column_info, get_columns_for_autocomplete, get_table_data, execute_query, save_changes, restore_backup, save_connection, list_saved_connections, delete_saved_connection, load_settings, save_settings, cmd_save_query, cmd_update_query, cmd_rename_query, cmd_delete_query, cmd_list_queries, cmd_load_query])
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
