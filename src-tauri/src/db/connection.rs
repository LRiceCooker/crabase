use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use tokio::sync::RwLock;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub dbname: String,
    pub schema: String,
    pub sslmode: String,
    pub password: String,
}

pub struct DbState {
    pub(crate) pool: RwLock<Option<PgPool>>,
    pub(crate) connection_info: RwLock<Option<ConnectionInfo>>,
    connection_string: RwLock<Option<String>>,
}

impl Default for DbState {
    fn default() -> Self {
        Self {
            pool: RwLock::new(None),
            connection_info: RwLock::new(None),
            connection_string: RwLock::new(None),
        }
    }
}

impl DbState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a clone of the connection pool, or error if not connected.
    pub(crate) async fn pool(&self) -> Result<PgPool, String> {
        self.pool
            .read()
            .await
            .clone()
            .ok_or_else(|| "Not connected to any database".to_string())
    }

    /// Get the current schema name.
    pub(crate) async fn schema(&self) -> String {
        self.connection_info
            .read()
            .await
            .as_ref()
            .map(|i| i.schema.clone())
            .unwrap_or_else(|| "public".to_string())
    }

    pub async fn connect(&self, info: ConnectionInfo) -> Result<(), String> {
        let connection_string = build_connection_string(&info);

        let pool = PgPool::connect(&connection_string)
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        // Verify the connection actually works
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| format!("Connection validation failed: {}", e))?;

        *self.pool.write().await = Some(pool);
        *self.connection_info.write().await = Some(info);
        *self.connection_string.write().await = Some(connection_string);

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), String> {
        *self.pool.write().await = None;
        *self.connection_info.write().await = None;
        *self.connection_string.write().await = None;
        Ok(())
    }

    pub async fn get_connection_string(&self) -> Result<String, String> {
        self.connection_string
            .read()
            .await
            .clone()
            .ok_or_else(|| "Not connected to any database".to_string())
    }

    pub async fn get_connection_info(&self) -> Result<ConnectionInfo, String> {
        self.connection_info
            .read()
            .await
            .clone()
            .ok_or_else(|| "Not connected to any database".to_string())
    }
}

pub fn build_connection_string(info: &ConnectionInfo) -> String {
    let password_part = if info.password.is_empty() {
        String::new()
    } else {
        format!(":{}", info.password)
    };
    format!(
        "postgresql://{}{}@{}:{}/{}?sslmode={}",
        info.user, password_part, info.host, info.port, info.dbname, info.sslmode
    )
}

pub fn parse_connection_string(connection_string: &str) -> Result<ConnectionInfo, String> {
    let url =
        Url::parse(connection_string).map_err(|e| format!("Invalid connection string: {}", e))?;

    let sslmode = url
        .query_pairs()
        .find(|(k, _)| k == "sslmode")
        .map(|(_, v)| v.to_string())
        .unwrap_or_else(|| "disable".to_string());

    Ok(ConnectionInfo {
        host: url.host_str().unwrap_or("localhost").to_string(),
        port: url.port().unwrap_or(5432),
        user: url.username().to_string(),
        password: url.password().unwrap_or("").to_string(),
        dbname: url.path().trim_start_matches('/').to_string(),
        schema: "public".to_string(),
        sslmode,
    })
}

pub async fn list_schemas(connection_string: &str) -> Result<Vec<String>, String> {
    let pool = PgPool::connect(connection_string)
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT schema_name FROM information_schema.schemata WHERE schema_name NOT IN ('pg_catalog', 'pg_toast', 'information_schema') ORDER BY schema_name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to list schemas: {}", e))?;

    pool.close().await;

    Ok(rows.into_iter().map(|(name,)| name).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_db_state_new() {
        let state = DbState::new();
        let pool = state.pool.read().await;
        assert!(pool.is_none());
        let info = state.connection_info.read().await;
        assert!(info.is_none());
    }

    #[tokio::test]
    async fn test_get_connection_info_not_connected() {
        let state = DbState::new();
        let result = state.get_connection_info().await;
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
        assert_eq!(info.password, "secret");
        assert_eq!(info.dbname, "mydb");
        assert_eq!(info.schema, "public");
        assert_eq!(info.sslmode, "disable");
    }

    #[test]
    fn test_parse_connection_string_with_ssl() {
        let info =
            parse_connection_string("postgresql://admin@localhost/testdb?sslmode=require").unwrap();
        assert_eq!(info.sslmode, "require");
    }

    #[test]
    fn test_parse_connection_string_defaults() {
        let info = parse_connection_string("postgresql://admin@localhost/testdb").unwrap();
        assert_eq!(info.host, "localhost");
        assert_eq!(info.port, 5432);
        assert_eq!(info.user, "admin");
        assert_eq!(info.dbname, "testdb");
        assert_eq!(info.password, "");
        assert_eq!(info.schema, "public");
    }

    #[test]
    fn test_build_connection_string() {
        let info = ConnectionInfo {
            host: "localhost".to_string(),
            port: 5432,
            user: "admin".to_string(),
            password: "secret".to_string(),
            dbname: "mydb".to_string(),
            schema: "public".to_string(),
            sslmode: "require".to_string(),
        };
        assert_eq!(
            build_connection_string(&info),
            "postgresql://admin:secret@localhost:5432/mydb?sslmode=require"
        );
    }

    #[test]
    fn test_build_connection_string_no_password() {
        let info = ConnectionInfo {
            host: "localhost".to_string(),
            port: 5432,
            user: "admin".to_string(),
            password: "".to_string(),
            dbname: "mydb".to_string(),
            schema: "public".to_string(),
            sslmode: "disable".to_string(),
        };
        assert_eq!(
            build_connection_string(&info),
            "postgresql://admin@localhost:5432/mydb?sslmode=disable"
        );
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
    async fn test_disconnect_when_not_connected() {
        let state = DbState::new();
        let result = state.disconnect().await;
        assert!(result.is_ok());
        assert!(state.pool.read().await.is_none());
        assert!(state.connection_info.read().await.is_none());
    }

    #[tokio::test]
    async fn test_disconnect_clears_connection_info() {
        let state = DbState::new();
        {
            let mut info_guard = state.connection_info.write().await;
            *info_guard = Some(ConnectionInfo {
                host: "localhost".to_string(),
                port: 5432,
                user: "test".to_string(),
                password: "".to_string(),
                dbname: "testdb".to_string(),
                schema: "public".to_string(),
                sslmode: "disable".to_string(),
            });
        }
        assert!(state.get_connection_info().await.is_ok());

        let result = state.disconnect().await;
        assert!(result.is_ok());
        assert!(state.get_connection_info().await.is_err());
    }

    #[tokio::test]
    async fn test_connect_invalid_string() {
        let state = DbState::new();
        let info = ConnectionInfo {
            host: "localhost".to_string(),
            port: 9999,
            user: "invalid".to_string(),
            password: "invalid".to_string(),
            dbname: "nope".to_string(),
            schema: "public".to_string(),
            sslmode: "disable".to_string(),
        };
        let result = state.connect(info).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Connection failed"));
    }
}
