use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::sync::Mutex;
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
    pub pool: Mutex<Option<PgPool>>,
    pub connection_info: Mutex<Option<ConnectionInfo>>,
    pub connection_string: Mutex<Option<String>>,
}

impl DbState {
    pub fn new() -> Self {
        Self {
            pool: Mutex::new(None),
            connection_info: Mutex::new(None),
            connection_string: Mutex::new(None),
        }
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

        let mut pool_guard = self.pool.lock().map_err(|e| format!("Lock error: {}", e))?;
        *pool_guard = Some(pool);

        let mut info_guard = self
            .connection_info
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        *info_guard = Some(info);

        let mut cs_guard = self
            .connection_string
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        *cs_guard = Some(connection_string);

        Ok(())
    }

    pub fn disconnect(&self) -> Result<(), String> {
        let mut pool_guard = self.pool.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(pool) = pool_guard.take() {
            drop(pool);
        }

        let mut info_guard = self
            .connection_info
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        *info_guard = None;

        let mut cs_guard = self
            .connection_string
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        *cs_guard = None;

        Ok(())
    }

    pub fn get_connection_string(&self) -> Result<String, String> {
        let guard = self
            .connection_string
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        guard
            .clone()
            .ok_or_else(|| "Not connected to any database".to_string())
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

        let schema = {
            let info_guard = self
                .connection_info
                .lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            info_guard
                .as_ref()
                .map(|i| i.schema.clone())
                .unwrap_or_else(|| "public".to_string())
        };

        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = $1 ORDER BY table_name",
        )
        .bind(&schema)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Failed to list tables: {}", e))?;

        Ok(rows.into_iter().map(|(name,)| name).collect())
    }

    pub async fn get_column_info(&self, table_name: &str) -> Result<Vec<ColumnInfo>, String> {
        let pool = {
            let pool_guard = self.pool.lock().map_err(|e| format!("Lock error: {}", e))?;
            pool_guard
                .clone()
                .ok_or_else(|| "Not connected to any database".to_string())?
        };

        let schema = {
            let info_guard = self
                .connection_info
                .lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            info_guard
                .as_ref()
                .map(|i| i.schema.clone())
                .unwrap_or_else(|| "public".to_string())
        };

        let rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT
                c.column_name,
                c.data_type,
                c.is_nullable,
                tc.constraint_type
            FROM information_schema.columns c
            LEFT JOIN information_schema.key_column_usage kcu
                ON c.table_schema = kcu.table_schema
                AND c.table_name = kcu.table_name
                AND c.column_name = kcu.column_name
            LEFT JOIN information_schema.table_constraints tc
                ON kcu.constraint_name = tc.constraint_name
                AND kcu.table_schema = tc.table_schema
                AND tc.constraint_type = 'PRIMARY KEY'
            WHERE c.table_schema = $1
                AND c.table_name = $2
            ORDER BY c.ordinal_position
            "#,
        )
        .bind(&schema)
        .bind(table_name)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Failed to get column info: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|(name, data_type, is_nullable, constraint_type)| ColumnInfo {
                name,
                data_type,
                is_nullable: is_nullable == "YES",
                is_primary_key: constraint_type.as_deref() == Some("PRIMARY KEY"),
            })
            .collect())
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
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

    #[test]
    fn test_disconnect_when_not_connected() {
        let state = DbState::new();
        let result = state.disconnect();
        assert!(result.is_ok());
        // State should still be empty
        assert!(state.pool.lock().unwrap().is_none());
        assert!(state.connection_info.lock().unwrap().is_none());
    }

    #[test]
    fn test_disconnect_clears_connection_info() {
        let state = DbState::new();
        // Manually set connection info
        {
            let mut info_guard = state.connection_info.lock().unwrap();
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
        assert!(state.get_connection_info().is_ok());

        let result = state.disconnect();
        assert!(result.is_ok());
        assert!(state.get_connection_info().is_err());
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

    #[tokio::test]
    async fn test_list_tables_not_connected() {
        let state = DbState::new();
        let result = state.list_tables().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not connected to any database");
    }

    #[tokio::test]
    async fn test_get_column_info_not_connected() {
        let state = DbState::new();
        let result = state.get_column_info("some_table").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not connected to any database");
    }
}
