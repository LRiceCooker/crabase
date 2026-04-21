mod connection;
mod execute;
mod introspection;
mod mutations;
mod query;
mod schema;
mod table_ops;
mod types;

pub use connection::*;
pub use execute::*;
pub use mutations::*;
pub use query::*;
pub use schema::*;

// Re-export for use by sibling submodules via super::pg_value_to_json
pub(crate) use types::pg_value_to_json;

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
        // State should still be empty
        assert!(state.pool.read().await.is_none());
        assert!(state.connection_info.read().await.is_none());
    }

    #[tokio::test]
    async fn test_disconnect_clears_connection_info() {
        let state = DbState::new();
        // Manually set connection info
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

    #[tokio::test]
    async fn test_get_table_data_not_connected() {
        let state = DbState::new();
        let result = state.get_table_data("some_table", 1, 25).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not connected to any database");
    }

    #[tokio::test]
    async fn test_execute_query_not_connected() {
        let state = DbState::new();
        let result = state.execute_query("SELECT 1").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not connected to any database");
    }

    #[tokio::test]
    async fn test_drop_table_not_connected() {
        let state = DbState::new();
        let result = state.drop_table("test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_save_changes_not_connected() {
        let state = DbState::new();
        let cs = ChangeSet {
            updates: vec![],
            inserts: vec![],
            deletes: vec![],
        };
        let result = state.save_changes("some_table", cs).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not connected to any database");
    }
}
