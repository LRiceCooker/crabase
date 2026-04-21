mod connection;
mod execute;
mod mutations;
mod query;
mod schema;
mod table_ops;

pub use connection::*;
pub use execute::*;
pub use mutations::*;
pub use query::*;
pub use schema::*;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use sqlx::{Column, Row, TypeInfo};

impl DbState {
    /// Get a text representation of ALL schemas in the database for AI context.
    pub async fn get_full_schema_text(&self) -> Result<String, String> {
        let pool = self.pool().await?;

        // Get all schemas with their tables and columns
        let rows: Vec<(String, String, String, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT
                t.table_schema,
                t.table_name,
                c.column_name,
                c.data_type,
                tc.constraint_type
            FROM information_schema.tables t
            JOIN information_schema.columns c
                ON t.table_schema = c.table_schema AND t.table_name = c.table_name
            LEFT JOIN information_schema.key_column_usage kcu
                ON c.table_schema = kcu.table_schema
                AND c.table_name = kcu.table_name
                AND c.column_name = kcu.column_name
            LEFT JOIN information_schema.table_constraints tc
                ON kcu.constraint_name = tc.constraint_name
                AND kcu.table_schema = tc.table_schema
                AND tc.constraint_type = 'PRIMARY KEY'
            WHERE t.table_schema NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
                AND t.table_type = 'BASE TABLE'
            ORDER BY t.table_schema, t.table_name, c.ordinal_position
            "#,
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Failed to get schema info: {}", e))?;

        // Build text representation
        let mut output = String::new();
        let mut current_schema = String::new();
        let mut current_table = String::new();

        for (schema, table, column, data_type, constraint) in rows {
            if schema != current_schema {
                if !current_schema.is_empty() {
                    output.push('\n');
                }
                output.push_str(&format!("Schema: {}\n", schema));
                current_schema = schema;
                current_table.clear();
            }
            if table != current_table {
                output.push_str(&format!("  Table: {}\n", table));
                current_table = table;
            }
            let pk = if constraint.as_deref() == Some("PRIMARY KEY") { " [PK]" } else { "" };
            output.push_str(&format!("    {} {}{}\n", column, data_type, pk));
        }

        Ok(output)
    }

}

/// Build a tagged value: `{ "type": "<pg_type>", "value": <val> }`.
fn tagged(pg_type: &str, value: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "type": pg_type, "value": value })
}

/// Build a tagged value for an unknown type: `{ "type": "unknown", "raw": "<text>" }`.
fn tagged_unknown(raw: &str) -> serde_json::Value {
    serde_json::json!({ "type": "unknown", "raw": raw })
}

/// Normalize a Postgres type name to a canonical frontend type string.
fn normalize_pg_type(type_name: &str) -> &str {
    match type_name {
        "BOOL" => "boolean",
        "INT2" => "smallint",
        "INT4" => "integer",
        "INT8" => "bigint",
        "FLOAT4" => "real",
        "FLOAT8" => "double",
        "NUMERIC" => "numeric",
        "MONEY" => "money",
        "TEXT" => "text",
        "VARCHAR" | "CHAR" | "BPCHAR" | "NAME" => "text",
        "BYTEA" => "bytea",
        "DATE" => "date",
        "TIME" | "TIMETZ" => "time",
        "TIMESTAMP" => "timestamp",
        "TIMESTAMPTZ" => "timestamptz",
        "INTERVAL" => "interval",
        "UUID" => "uuid",
        "JSON" => "json",
        "JSONB" => "jsonb",
        "XML" => "xml",
        "INET" => "inet",
        "CIDR" => "cidr",
        "MACADDR" | "MACADDR8" => "macaddr",
        "BIT" | "VARBIT" => "bit",
        "TSVECTOR" => "tsvector",
        "TSQUERY" => "tsquery",
        "POINT" | "LINE" | "LSEG" | "BOX" | "PATH" | "POLYGON" | "CIRCLE" => "geometry",
        "INT4RANGE" | "INT8RANGE" | "NUMRANGE" | "TSRANGE" | "TSTZRANGE" | "DATERANGE" => "range",
        "OID" => "integer",
        _ => {
            // Array types start with underscore in Postgres internal names
            if type_name.starts_with('_') {
                "array"
            } else {
                "unknown"
            }
        }
    }
}

/// Convert a PostgreSQL column value to a tagged JSON value.
/// Output format: `{ "type": "<pg_type>", "value": <json_val> }`.
/// NULL values are returned as `serde_json::Value::Null` (untagged).
pub(crate) fn pg_value_to_json(row: &sqlx::postgres::PgRow, idx: usize) -> serde_json::Value {
    let col = row.column(idx);
    let type_name = col.type_info().name();
    let canonical = normalize_pg_type(type_name);

    // Check for NULL first via a raw decode attempt
    match type_name {
        "BOOL" => match row.try_get::<Option<bool>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::Bool(v)),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "INT2" => match row.try_get::<Option<i16>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::Number(v.into())),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "INT4" | "OID" => match row.try_get::<Option<i32>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::Number(v.into())),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "INT8" => match row.try_get::<Option<i64>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::Number(v.into())),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "FLOAT4" => match row.try_get::<Option<f32>, _>(idx) {
            Ok(Some(v)) => {
                let n = serde_json::Number::from_f64(v as f64)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::String(v.to_string()));
                tagged(canonical, n)
            }
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "FLOAT8" => match row.try_get::<Option<f64>, _>(idx) {
            Ok(Some(v)) => {
                let n = serde_json::Number::from_f64(v)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::String(v.to_string()));
                tagged(canonical, n)
            }
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "JSON" | "JSONB" => match row.try_get::<Option<serde_json::Value>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, v),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "TIMESTAMP" => match row.try_get::<Option<NaiveDateTime>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::String(v.format("%Y-%m-%d %H:%M:%S").to_string())),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "TIMESTAMPTZ" => match row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::String(v.format("%Y-%m-%dT%H:%M:%SZ").to_string())),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "DATE" => match row.try_get::<Option<NaiveDate>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::String(v.format("%Y-%m-%d").to_string())),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "TIME" | "TIMETZ" => match row.try_get::<Option<NaiveTime>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::String(v.format("%H:%M:%S").to_string())),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        _ => {
            // For all other types, try as String first — covers text, varchar,
            // uuid, inet, cidr, macaddr, interval, xml, numeric, money, bit, range, etc.
            match row.try_get::<Option<String>, _>(idx) {
                Ok(Some(v)) => tagged(canonical, serde_json::Value::String(v)),
                Ok(None) => serde_json::Value::Null,
                Err(_) => tagged_unknown(type_name),
            }
        }
    }
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
