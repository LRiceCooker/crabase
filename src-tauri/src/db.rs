use serde::Serialize;
use sqlx::postgres::PgPool;
use std::sync::Mutex;
use url::Url;

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionInfo {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub dbname: String,
}

pub struct DbState {
    pub pool: Mutex<Option<PgPool>>,
    pub connection_info: Mutex<Option<ConnectionInfo>>,
}

impl DbState {
    pub fn new() -> Self {
        Self {
            pool: Mutex::new(None),
            connection_info: Mutex::new(None),
        }
    }

    pub async fn connect(&self, connection_string: &str) -> Result<(), String> {
        let info = parse_connection_string(connection_string)?;

        let pool = PgPool::connect(connection_string)
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        // Verify the connection actually works
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| format!("Connection validation failed: {}", e))?;

        let mut pool_guard = self.pool.lock().map_err(|e| format!("Lock error: {}", e))?;
        *pool_guard = Some(pool);

        let mut info_guard = self
            .connection_info
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        *info_guard = Some(info);

        Ok(())
    }

    pub fn get_connection_info(&self) -> Result<ConnectionInfo, String> {
        let guard = self
            .connection_info
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        guard
            .clone()
            .ok_or_else(|| "Not connected to any database".to_string())
    }

    pub async fn list_tables(&self) -> Result<Vec<String>, String> {
        let pool = {
            let pool_guard = self.pool.lock().map_err(|e| format!("Lock error: {}", e))?;
            pool_guard
                .clone()
                .ok_or_else(|| "Not connected to any database".to_string())?
        };

        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' ORDER BY table_name",
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Failed to list tables: {}", e))?;

        Ok(rows.into_iter().map(|(name,)| name).collect())
    }
}

pub fn parse_connection_string(connection_string: &str) -> Result<ConnectionInfo, String> {
    let url =
        Url::parse(connection_string).map_err(|e| format!("Invalid connection string: {}", e))?;

    Ok(ConnectionInfo {
        host: url.host_str().unwrap_or("localhost").to_string(),
        port: url.port().unwrap_or(5432),
        user: url.username().to_string(),
        dbname: url.path().trim_start_matches('/').to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_state_new() {
        let state = DbState::new();
        let pool = state.pool.lock().unwrap();
        assert!(pool.is_none());
        let info = state.connection_info.lock().unwrap();
        assert!(info.is_none());
    }

    #[test]
    fn test_get_connection_info_not_connected() {
        let state = DbState::new();
        let result = state.get_connection_info();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not connected to any database");
    }

    #[test]
    fn test_parse_connection_string_full() {
        let info =
            parse_connection_string("postgresql://myuser:secret@db.example.com:5433/mydb").unwrap();
        assert_eq!(info.host, "db.example.com");
        assert_eq!(info.port, 5433);
        assert_eq!(info.user, "myuser");
        assert_eq!(info.dbname, "mydb");
    }

    #[test]
    fn test_parse_connection_string_defaults() {
        let info = parse_connection_string("postgresql://admin@localhost/testdb").unwrap();
        assert_eq!(info.host, "localhost");
        assert_eq!(info.port, 5432);
        assert_eq!(info.user, "admin");
        assert_eq!(info.dbname, "testdb");
    }

    #[test]
    fn test_parse_connection_string_invalid() {
        let result = parse_connection_string("not-a-url");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid connection string"));
    }

    #[test]
    fn test_parse_connection_string_empty() {
        let result = parse_connection_string("");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connect_invalid_string() {
        let state = DbState::new();
        let result = state
            .connect("postgresql://invalid:invalid@localhost:9999/nope")
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Connection failed"));
    }

    #[tokio::test]
    async fn test_connect_empty_string() {
        let state = DbState::new();
        let result = state.connect("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_tables_not_connected() {
        let state = DbState::new();
        let result = state.list_tables().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not connected to any database");
    }
}
