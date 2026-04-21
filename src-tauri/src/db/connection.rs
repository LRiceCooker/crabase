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
