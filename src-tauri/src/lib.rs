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

pub mod db;
mod restore;
pub mod saved_connections;
pub mod saved_queries;
pub mod settings;

use tauri::Emitter;

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
async fn disconnect_db(db_state: tauri::State<'_, db::DbState>) -> Result<String, String> {
    db_state.disconnect().await?;
    Ok("Disconnected successfully".to_string())
}

#[tauri::command]
async fn get_connection_info(
    db_state: tauri::State<'_, db::DbState>,
) -> Result<db::ConnectionInfo, String> {
    db_state.get_connection_info().await
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

#[tauri::command]
async fn execute_query(
    sql: String,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<db::QueryResult, String> {
    db_state.execute_query(&sql).await
}

#[tauri::command]
async fn execute_query_multi(
    sql: String,
    db_state: tauri::State<'_, db::DbState>,
) -> Result<Vec<db::StatementResult>, String> {
    db_state.execute_query_multi(&sql).await
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
    let connection_string = db_state.get_connection_string().await?;
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

#[tauri::command]
fn set_app_icon(is_dark: bool, app_handle: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;

    let png_bytes: &[u8] = if is_dark {
        include_bytes!("../icons/dark/icon.png")
    } else {
        include_bytes!("../icons/light/icon.png")
    };

    // Decode PNG to RGBA using the png crate
    let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
    let mut reader = decoder.read_info().map_err(|e| format!("PNG decode error: {}", e))?;
    let info = reader.info().clone();
    let mut buf = vec![0u8; info.raw_bytes()];
    reader.next_frame(&mut buf).map_err(|e| format!("PNG frame error: {}", e))?;

    let icon = tauri::image::Image::new_owned(buf, info.width, info.height);

    // Set icon on the main window
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.set_icon(icon);
    }
    Ok(())
}

/// Helper: derive connection key from current DB state.
async fn get_conn_key(db_state: &db::DbState) -> Result<String, String> {
    let info = db_state.get_connection_info().await?;
    Ok(saved_queries::connection_key(
        &info.host, info.port, &info.dbname, &info.user,
    ))
}

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

#[tauri::command]
async fn cmd_delete_query(
    name: String,
    db_state: tauri::State<'_, db::DbState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let key = get_conn_key(&db_state).await?;
    saved_queries::delete_query(&app_handle, &key, &name)
}

#[tauri::command]
async fn cmd_list_queries(
    db_state: tauri::State<'_, db::DbState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<saved_queries::SavedQuery>, String> {
    let key = get_conn_key(&db_state).await?;
    saved_queries::list_queries(&app_handle, &key)
}

#[tauri::command]
async fn cmd_load_query(
    name: String,
    db_state: tauri::State<'_, db::DbState>,
    app_handle: tauri::AppHandle,
) -> Result<saved_queries::SavedQuery, String> {
    let key = get_conn_key(&db_state).await?;
    saved_queries::load_query(&app_handle, &key, &name)
}

/// Get the user's default shell, falling back to /bin/zsh.
fn user_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string())
}

/// Check if claude is available via user's login shell.
/// Uses -l (login) AND -i (interactive) to ensure ~/.zshrc is sourced,
/// which is where tools like mise/nvm/volta configure PATH.
fn check_claude_via_shell() -> bool {
    let shell = user_shell();
    std::process::Command::new(&shell)
        .args(["-l", "-i", "-c", "command -v claude"])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[tauri::command]
fn check_claude_installed() -> bool {
    check_claude_via_shell()
}

#[tauri::command]
async fn chat_with_claude(prompt: String, app: tauri::AppHandle) -> Result<(), String> {
    if !check_claude_via_shell() {
        return Err("Claude Code not found. Install it from claude.ai/code".to_string());
    }

    // Run the blocking subprocess on a background thread
    // Use login shell so claude is in PATH with all user env (nvm, mise, volta, etc.)
    tokio::task::spawn_blocking(move || {
        use std::io::BufRead;

        // Escape single quotes in the prompt for shell
        let escaped_prompt = prompt.replace('\'', "'\\''");
        let shell_cmd = format!("claude --print -p '{}' --dangerously-skip-permissions", escaped_prompt);
        let shell = user_shell();

        let mut child = std::process::Command::new(&shell)
            .args(["-l", "-i", "-c", &shell_cmd])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn claude: {}", e))?;

        if let Some(stdout) = child.stdout.take() {
            let reader = std::io::BufReader::new(stdout);
            for line in reader.lines() {
                let Ok(line) = line else { continue };
                // --print outputs plain text, stream line by line
                let _ = app.emit("chat-response", format!("{}\n", line));
            }
        }

        let _ = child.wait();
        let _ = app.emit("chat-done", ());
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
async fn get_full_schema_for_chat(
    db_state: tauri::State<'_, db::DbState>,
) -> Result<String, String> {
    db_state.get_full_schema_text().await
}

#[tauri::command]
async fn open_new_window(app: tauri::AppHandle) -> Result<(), String> {
    use std::sync::atomic::{AtomicU32, Ordering};
    static WINDOW_COUNTER: AtomicU32 = AtomicU32::new(2);
    let id = WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed);
    let label = format!("main-{}", id);
    tauri::WebviewWindowBuilder::new(&app, &label, tauri::WebviewUrl::App("index.html".into()))
        .title("crabase")
        .inner_size(1200.0, 800.0)
        .background_color(tauri::webview::Color(10, 10, 10, 255))
        .build()
        .map_err(|e| format!("Failed to create window: {}", e))?;
    Ok(())
}

#[tauri::command]
fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, &content).map_err(|e| format!("Failed to write file: {}", e))
}

#[tauri::command]
async fn drop_table(table_name: String, db_state: tauri::State<'_, db::DbState>) -> Result<String, String> {
    db_state.drop_table(&table_name).await
}

#[tauri::command]
async fn truncate_table(table_name: String, db_state: tauri::State<'_, db::DbState>) -> Result<String, String> {
    db_state.truncate_table(&table_name).await
}

#[tauri::command]
async fn export_table_json(table_name: String, db_state: tauri::State<'_, db::DbState>) -> Result<String, String> {
    db_state.export_table_json(&table_name).await
}

#[tauri::command]
async fn export_table_sql(table_name: String, db_state: tauri::State<'_, db::DbState>) -> Result<String, String> {
    db_state.export_table_sql(&table_name).await
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(db::DbState::new())
        .invoke_handler(tauri::generate_handler![parse_connection_string, list_schemas, connect_db, disconnect_db, get_connection_info, list_tables, get_column_info, get_columns_for_autocomplete, get_table_data, get_table_data_filtered, execute_query, execute_query_multi, save_changes, restore_backup, save_connection, list_saved_connections, delete_saved_connection, load_settings, save_settings, cmd_save_query, cmd_update_query, cmd_rename_query, cmd_delete_query, cmd_list_queries, cmd_load_query, open_new_window, check_claude_installed, chat_with_claude, get_full_schema_for_chat, drop_table, truncate_table, export_table_json, export_table_sql, write_file, set_app_icon])
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
