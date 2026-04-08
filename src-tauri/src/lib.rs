mod db;

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
fn get_connection_info(
    db_state: tauri::State<'_, db::DbState>,
) -> Result<db::ConnectionInfo, String> {
    db_state.get_connection_info()
}

pub fn run() {
    tauri::Builder::default()
        .manage(db::DbState::new())
        .invoke_handler(tauri::generate_handler![greet, connect_db, get_connection_info])
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
