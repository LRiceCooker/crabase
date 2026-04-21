mod connection;
mod query;
mod schema;

pub use connection::*;
pub use query::*;
pub use schema::*;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use sqlx::{Column, Row, TypeInfo};
use std::collections::HashMap;

impl DbState {
    pub async fn save_changes(
        &self,
        table_name: &str,
        change_set: ChangeSet,
    ) -> Result<String, String> {
        let pool = self.pool().await?;
        let schema = self.schema().await;

        let quoted_schema = format!("\"{}\"", schema.replace('"', "\"\""));
        let quoted_table = format!("\"{}\"", table_name.replace('"', "\"\""));
        let qualified_table = format!("{}.{}", quoted_schema, quoted_table);

        let mut tx = pool
            .begin()
            .await
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        let mut total_affected = 0u64;

        // Apply deletes
        for delete in &change_set.deletes {
            if delete.pk_values.is_empty() {
                continue;
            }
            let (where_clause, values) = build_where_clause(&delete.pk_values, 1);
            let sql = format!("DELETE FROM {} WHERE {}", qualified_table, where_clause);
            let mut query = sqlx::query(&sql);
            for v in &values {
                query = bind_json_value(query, v);
            }
            let result = query
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Delete failed: {}", e))?;
            total_affected += result.rows_affected();
        }

        // Apply updates
        for update in &change_set.updates {
            if update.pk_values.is_empty() || update.changes.is_empty() {
                continue;
            }
            let (set_clause, set_values, next_idx) = build_set_clause(&update.changes);
            let (where_clause, where_values) = build_where_clause(&update.pk_values, next_idx);
            let sql = format!(
                "UPDATE {} SET {} WHERE {}",
                qualified_table, set_clause, where_clause
            );
            let mut query = sqlx::query(&sql);
            for v in &set_values {
                query = bind_json_value(query, v);
            }
            for v in &where_values {
                query = bind_json_value(query, v);
            }
            let result = query
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Update failed: {}", e))?;
            total_affected += result.rows_affected();
        }

        // Apply inserts
        for insert in &change_set.inserts {
            if insert.values.is_empty() {
                continue;
            }
            let cols: Vec<&str> = insert.values.keys().map(|k| k.as_str()).collect();
            let quoted_cols: Vec<String> = cols
                .iter()
                .map(|c| format!("\"{}\"", c.replace('"', "\"\"")))
                .collect();
            let placeholders: Vec<String> = (1..=cols.len()).map(|i| format!("${}", i)).collect();
            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                qualified_table,
                quoted_cols.join(", "),
                placeholders.join(", ")
            );
            let values: Vec<&serde_json::Value> =
                insert.values.values().collect();
            let mut query = sqlx::query(&sql);
            for v in &values {
                query = bind_json_value(query, v);
            }
            let result = query
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Insert failed: {}", e))?;
            total_affected += result.rows_affected();
        }

        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(format!("{} rows affected", total_affected))
    }

    pub async fn execute_query(&self, sql: &str) -> Result<QueryResult, String> {
        let pool = self.pool().await?;

        // Use raw_sql (simple query protocol) — returns all values as text,
        // avoiding binary protocol issues with enums, arrays, and custom types.
        use futures::TryStreamExt;
        let mut stream = sqlx::raw_sql(sql).fetch_many(&pool);
        let mut columns: Vec<String> = Vec::new();
        let mut rows: Vec<Vec<serde_json::Value>> = Vec::new();

        while let Some(either) = stream.try_next().await.map_err(|e| format!("{}", e))? {
            match either {
                sqlx::Either::Right(row) => {
                    if columns.is_empty() {
                        columns = (0..row.len())
                            .map(|i| row.column(i).name().to_string())
                            .collect();
                    }
                    let row_values: Vec<serde_json::Value> = (0..row.len())
                        .map(|i| pg_value_to_json(&row, i))
                        .collect();
                    rows.push(row_values);
                }
                sqlx::Either::Left(_) => {}
            }
        }

        Ok(QueryResult { columns, rows })
    }

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

    /// Execute multiple SQL statements using the simple query protocol (raw_sql).
    /// Returns a Vec<StatementResult> — one entry per statement.
    /// The simple protocol returns all values as text, avoiding binary issues with enums/arrays.
    pub async fn execute_query_multi(&self, sql: &str) -> Result<Vec<StatementResult>, String> {
        let pool = self.pool().await?;

        use futures::TryStreamExt;

        // raw_sql sends everything via the simple query protocol.
        // It returns a stream of Either<QueryResult, Row> — Left for commands, Right for data rows.
        let mut stream = sqlx::raw_sql(sql).fetch_many(&pool);

        let mut results: Vec<StatementResult> = Vec::new();
        let mut current_columns: Vec<String> = Vec::new();
        let mut current_rows: Vec<Vec<serde_json::Value>> = Vec::new();
        let mut current_preview = String::new();

        // Track which statement we're on for previews
        let statements: Vec<&str> = sql.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let mut stmt_idx = 0usize;

        while let Some(either) = stream.try_next().await.map_err(|e| format!("{}", e))? {
            match either {
                sqlx::Either::Right(row) => {
                    // Data row — accumulate into current result
                    if current_columns.is_empty() {
                        current_columns = (0..row.len())
                            .map(|i| row.column(i).name().to_string())
                            .collect();
                        current_preview = statements.get(stmt_idx).map(|s| {
                            if s.len() > 60 { format!("{}...", &s[..60]) } else { s.to_string() }
                        }).unwrap_or_default();
                    }
                    let row_values: Vec<serde_json::Value> = (0..row.len())
                        .map(|i| pg_value_to_json(&row, i))
                        .collect();
                    current_rows.push(row_values);
                }
                sqlx::Either::Left(result) => {
                    // Command completed — flush any accumulated rows first
                    if !current_columns.is_empty() {
                        results.push(StatementResult::Rows {
                            columns: std::mem::take(&mut current_columns),
                            rows: std::mem::take(&mut current_rows),
                            sql_preview: std::mem::take(&mut current_preview),
                        });
                        stmt_idx += 1;
                    }

                    // Record the command result
                    let preview = statements.get(stmt_idx).map(|s| {
                        if s.len() > 60 { format!("{}...", &s[..60]) } else { s.to_string() }
                    }).unwrap_or_default();
                    let affected = result.rows_affected();

                    // Only record if it actually did something (skip the "no rows" result from SELECT)
                    if affected > 0 || current_rows.is_empty() {
                        let upper = preview.trim_start().to_uppercase();
                        let cmd = upper.split_whitespace().next().unwrap_or("OK").to_string();
                        // Don't add Affected for SELECT statements (they come with Rows)
                        if !cmd.starts_with("SELECT") && !cmd.starts_with("WITH") && !cmd.starts_with("TABLE") {
                            results.push(StatementResult::Affected {
                                command: cmd,
                                rows_affected: affected,
                                sql_preview: preview,
                            });
                        }
                    }
                    stmt_idx += 1;
                }
            }
        }

        // Flush any remaining rows
        if !current_columns.is_empty() {
            results.push(StatementResult::Rows {
                columns: current_columns,
                rows: current_rows,
                sql_preview: current_preview,
            });
        }

        if results.is_empty() {
            results.push(StatementResult::Affected {
                command: "OK".to_string(),
                rows_affected: 0,
                sql_preview: String::new(),
            });
        }

        Ok(results)
    }

    pub async fn drop_table(&self, table_name: &str) -> Result<String, String> {
        let pool = self.pool().await?;
        let schema = self.schema().await;
        let qualified = format!("\"{}\".\"{}\"", schema.replace('"', "\"\""), table_name.replace('"', "\"\""));
        let sql = format!("DROP TABLE {} CASCADE", qualified);
        sqlx::query(&sql).execute(&pool).await.map_err(|e| format!("DROP TABLE failed: {}", e))?;
        Ok(format!("Table {} dropped", table_name))
    }

    pub async fn truncate_table(&self, table_name: &str) -> Result<String, String> {
        let pool = self.pool().await?;
        let schema = self.schema().await;
        let qualified = format!("\"{}\".\"{}\"", schema.replace('"', "\"\""), table_name.replace('"', "\"\""));
        let sql = format!("TRUNCATE TABLE {} CASCADE", qualified);
        sqlx::query(&sql).execute(&pool).await.map_err(|e| format!("TRUNCATE failed: {}", e))?;
        Ok(format!("Table {} truncated", table_name))
    }

    pub async fn export_table_json(&self, table_name: &str) -> Result<String, String> {
        let pool = self.pool().await?;
        let schema = self.schema().await;
        let qualified = format!("\"{}\".\"{}\"", schema.replace('"', "\"\""), table_name.replace('"', "\"\""));
        let query = format!("SELECT row_to_json(t) FROM {} t", qualified);
        let rows: Vec<(serde_json::Value,)> = sqlx::query_as(&query)
            .fetch_all(&pool).await
            .map_err(|e| format!("Export failed: {}", e))?;
        let arr: Vec<serde_json::Value> = rows.into_iter().map(|(v,)| v).collect();
        serde_json::to_string_pretty(&arr).map_err(|e| format!("JSON serialization failed: {}", e))
    }

    pub async fn export_table_sql(&self, table_name: &str) -> Result<String, String> {
        let columns = self.get_column_info(table_name).await?;
        let pool = self.pool().await?;
        let schema = self.schema().await;
        let qualified = format!("\"{}\".\"{}\"", schema.replace('"', "\"\""), table_name.replace('"', "\"\""));
        let query = format!("SELECT * FROM {}", qualified);
        let rows = sqlx::query(&query).fetch_all(&pool).await
            .map_err(|e| format!("Export failed: {}", e))?;

        let col_names: Vec<String> = columns.iter().map(|c| format!("\"{}\"", c.name.replace('"', "\"\""))).collect();
        let header = format!("-- Export of {}\n", qualified);
        let mut inserts = Vec::new();
        for row in &rows {
            let values: Vec<String> = (0..columns.len()).map(|i| {
                let val = pg_value_to_json(row, i);
                let inner = if let Some(v) = val.get("value") { v.clone() } else if let Some(r) = val.get("raw") { r.clone() } else { val };
                match inner {
                    serde_json::Value::Null => "NULL".to_string(),
                    serde_json::Value::Bool(b) => if b { "TRUE".to_string() } else { "FALSE".to_string() },
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                    other => format!("'{}'", serde_json::to_string(&other).unwrap_or_default().replace('\'', "''")),
                }
            }).collect();
            inserts.push(format!("INSERT INTO {} ({}) VALUES ({});", qualified, col_names.join(", "), values.join(", ")));
        }
        Ok(format!("{}{}", header, inserts.join("\n")))
    }
}

/// Build a WHERE clause from primary key values starting at parameter index `start_idx`.
/// Returns (clause, values).
fn build_where_clause(pk_values: &HashMap<String, serde_json::Value>, start_idx: usize) -> (String, Vec<&serde_json::Value>) {
    let mut parts = Vec::new();
    let mut values = Vec::new();
    let mut idx = start_idx;
    for (col, val) in pk_values {
        let quoted_col = format!("\"{}\"", col.replace('"', "\"\""));
        parts.push(format!("{} = ${}", quoted_col, idx));
        values.push(val);
        idx += 1;
    }
    (parts.join(" AND "), values)
}

/// Build a SET clause from changed values starting at parameter index 1.
/// Returns (clause, values, next_idx).
fn build_set_clause(changes: &HashMap<String, serde_json::Value>) -> (String, Vec<&serde_json::Value>, usize) {
    let mut parts = Vec::new();
    let mut values = Vec::new();
    let mut idx = 1;
    for (col, val) in changes {
        let quoted_col = format!("\"{}\"", col.replace('"', "\"\""));
        parts.push(format!("{} = ${}", quoted_col, idx));
        values.push(val);
        idx += 1;
    }
    (parts.join(", "), values, idx)
}

/// Bind a serde_json::Value to a sqlx query.
fn bind_json_value<'q>(
    query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    value: &'q serde_json::Value,
) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    match value {
        serde_json::Value::Null => query.bind(None::<String>),
        serde_json::Value::Bool(b) => query.bind(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                query.bind(i)
            } else if let Some(f) = n.as_f64() {
                query.bind(f)
            } else {
                query.bind(n.to_string())
            }
        }
        serde_json::Value::String(s) => query.bind(s.as_str()),
        // For JSON/JSONB or complex types, bind as JSON
        _ => query.bind(value),
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

/// A row update: identified by primary key values, with changed column values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowUpdate {
    pub pk_values: HashMap<String, serde_json::Value>,
    pub changes: HashMap<String, serde_json::Value>,
}

/// A new row to insert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowInsert {
    pub values: HashMap<String, serde_json::Value>,
}

/// A row to delete, identified by primary key values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowDelete {
    pub pk_values: HashMap<String, serde_json::Value>,
}

/// A set of changes to apply to a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub updates: Vec<RowUpdate>,
    pub inserts: Vec<RowInsert>,
    pub deletes: Vec<RowDelete>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

/// Result of a single statement in a multi-statement execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StatementResult {
    Rows { columns: Vec<String>, rows: Vec<Vec<serde_json::Value>>, sql_preview: String },
    Affected { command: String, rows_affected: u64, sql_preview: String },
    Error { message: String, sql_preview: String },
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
