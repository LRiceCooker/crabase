mod db;
mod restore;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[tauri::command]
async fn connect_db(
    connection_string: String,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<String, String> {
    db_state.connect(&connection_string).await?;
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

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(db::DbState::new())
        .invoke_handler(tauri::generate_handler![greet, connect_db, disconnect_db, get_connection_info, list_tables, restore_backup])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greet("crabase"), "Hello, crabase!");
    }
}
