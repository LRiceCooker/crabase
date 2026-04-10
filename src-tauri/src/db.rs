use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::{Column, Executor, Row, TypeInfo};
use std::collections::HashMap;
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

        // Extended query: fetch type info, max_length, precision, scale, default, UDT name
        let rows: Vec<(
            String,          // column_name
            String,          // data_type
            String,          // is_nullable
            Option<String>,  // constraint_type
            Option<i32>,     // character_maximum_length
            Option<i32>,     // numeric_precision
            Option<i32>,     // numeric_scale
            Option<String>,  // column_default
            Option<String>,  // udt_name
        )> = sqlx::query_as(
            r#"
            SELECT
                c.column_name,
                c.data_type,
                c.is_nullable,
                tc.constraint_type,
                c.character_maximum_length::int4,
                c.numeric_precision::int4,
                c.numeric_scale::int4,
                c.column_default,
                c.udt_name
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

        let mut columns = Vec::new();
        for (name, data_type, is_nullable, constraint_type, max_len, precision, scale, col_default, udt_name) in rows {
            let is_auto = col_default
                .as_deref()
                .map(|d| d.starts_with("nextval("))
                .unwrap_or(false);
            let is_array = data_type == "ARRAY";
            let is_enum = data_type == "USER-DEFINED";

            // Fetch enum values if this is an enum column
            let enum_values = if is_enum {
                if let Some(ref udt) = udt_name {
                    Self::fetch_enum_values(&pool, &schema, udt).await.unwrap_or_default()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            columns.push(ColumnInfo {
                name,
                data_type: if is_enum {
                    udt_name.clone().unwrap_or(data_type)
                } else if is_array {
                    // Use udt_name (e.g. "_int4") to derive element type
                    udt_name.clone().unwrap_or(data_type)
                } else {
                    data_type
                },
                is_nullable: is_nullable == "YES",
                is_primary_key: constraint_type.as_deref() == Some("PRIMARY KEY"),
                is_auto_increment: is_auto,
                is_array,
                is_enum,
                enum_values,
                max_length: max_len,
                numeric_precision: precision,
                numeric_scale: scale,
            });
        }

        Ok(columns)
    }

    /// Fetch enum allowed values from pg_enum for a given UDT name.
    async fn fetch_enum_values(
        pool: &PgPool,
        schema: &str,
        enum_name: &str,
    ) -> Result<Vec<String>, String> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT e.enumlabel
            FROM pg_enum e
            JOIN pg_type t ON e.enumtypid = t.oid
            JOIN pg_namespace n ON t.typnamespace = n.oid
            WHERE n.nspname = $1 AND t.typname = $2
            ORDER BY e.enumsortorder
            "#,
        )
        .bind(schema)
        .bind(enum_name)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch enum values: {}", e))?;

        Ok(rows.into_iter().map(|(label,)| label).collect())
    }

    pub async fn get_columns_for_autocomplete(
        &self,
        table_names: &[String],
    ) -> Result<std::collections::HashMap<String, Vec<String>>, String> {
        let mut result = std::collections::HashMap::new();
        for table_name in table_names {
            let columns = self.get_column_info(table_name).await?;
            result.insert(
                table_name.clone(),
                columns.into_iter().map(|c| c.name).collect(),
            );
        }
        Ok(result)
    }

    pub async fn get_table_data(
        &self,
        table_name: &str,
        page: u32,
        page_size: u32,
    ) -> Result<TableData, String> {
        let columns = self.get_column_info(table_name).await?;

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

        // Use quoted identifiers to prevent SQL injection
        let quoted_schema = format!("\"{}\"", schema.replace('"', "\"\""));
        let quoted_table = format!("\"{}\"", table_name.replace('"', "\"\""));
        let qualified_table = format!("{}.{}", quoted_schema, quoted_table);

        // Get total count
        let count_query = format!("SELECT COUNT(*) as cnt FROM {}", qualified_table);
        let count_row: (i64,) = sqlx::query_as(&count_query)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to get row count: {}", e))?;
        let total_count = count_row.0 as u64;

        // Get paginated rows
        let offset = (page.saturating_sub(1)) as i64 * page_size as i64;
        let data_query = format!(
            "SELECT * FROM {} LIMIT {} OFFSET {}",
            qualified_table, page_size, offset
        );

        let pg_rows = sqlx::query(&data_query)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to get table data: {}", e))?;

        let rows: Vec<Vec<serde_json::Value>> = pg_rows
            .iter()
            .map(|row| {
                (0..row.len())
                    .map(|i| pg_value_to_json(row, i))
                    .collect()
            })
            .collect();

        Ok(TableData {
            columns,
            rows,
            total_count,
        })
    }

    pub async fn get_table_data_filtered(
        &self,
        table_name: &str,
        page: u32,
        page_size: u32,
        filters: Vec<Filter>,
        sort: Vec<SortCol>,
    ) -> Result<TableData, String> {
        let columns = self.get_column_info(table_name).await?;

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

        let quoted_schema = format!("\"{}\"", schema.replace('"', "\"\""));
        let quoted_table = format!("\"{}\"", table_name.replace('"', "\"\""));
        let qualified_table = format!("{}.{}", quoted_schema, quoted_table);

        // Build WHERE clause from filters
        let where_clause = build_filter_where_clause(&filters);

        // Build ORDER BY clause from sort columns
        let order_clause = build_order_clause(&sort);

        // Get filtered count
        let count_query = format!(
            "SELECT COUNT(*) as cnt FROM {}{}",
            qualified_table, where_clause
        );
        let count_row: (i64,) = sqlx::query_as(&count_query)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to get row count: {}", e))?;
        let total_count = count_row.0 as u64;

        // Get paginated filtered rows
        let offset = (page.saturating_sub(1)) as i64 * page_size as i64;
        let data_query = format!(
            "SELECT * FROM {}{}{} LIMIT {} OFFSET {}",
            qualified_table, where_clause, order_clause, page_size, offset
        );

        let pg_rows = sqlx::query(&data_query)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to get table data: {}", e))?;

        let rows: Vec<Vec<serde_json::Value>> = pg_rows
            .iter()
            .map(|row| {
                (0..row.len())
                    .map(|i| pg_value_to_json(row, i))
                    .collect()
            })
            .collect();

        Ok(TableData {
            columns,
            rows,
            total_count,
        })
    }

    pub async fn save_changes(
        &self,
        table_name: &str,
        change_set: ChangeSet,
    ) -> Result<String, String> {
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
        let pool = {
            let pool_guard = self.pool.lock().map_err(|e| format!("Lock error: {}", e))?;
            pool_guard
                .clone()
                .ok_or_else(|| "Not connected to any database".to_string())?
        };

        let pg_rows = sqlx::query(sql)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("{}", e))?;

        // Extract column names from the first row (or empty if no rows)
        let columns: Vec<String> = if let Some(first) = pg_rows.first() {
            (0..first.len())
                .map(|i| first.column(i).name().to_string())
                .collect()
        } else {
            Vec::new()
        };

        let rows: Vec<Vec<serde_json::Value>> = pg_rows
            .iter()
            .map(|row| {
                (0..row.len())
                    .map(|i| pg_value_to_json(row, i))
                    .collect()
            })
            .collect();

        Ok(QueryResult { columns, rows })
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

/// Build a WHERE clause from user-defined filters. Values are embedded as literals
/// with proper quoting to prevent SQL injection via column names.
fn build_filter_where_clause(filters: &[Filter]) -> String {
    if filters.is_empty() {
        return String::new();
    }
    let mut parts = Vec::new();
    for (i, f) in filters.iter().enumerate() {
        let quoted_col = format!("\"{}\"", f.column.replace('"', "\"\""));
        let escaped_val = f.value.replace('\'', "''");
        let condition = match f.operator.as_str() {
            "=" => format!("{} = '{}'", quoted_col, escaped_val),
            "!=" => format!("{} != '{}'", quoted_col, escaped_val),
            "<" => format!("{} < '{}'", quoted_col, escaped_val),
            ">" => format!("{} > '{}'", quoted_col, escaped_val),
            "<=" => format!("{} <= '{}'", quoted_col, escaped_val),
            ">=" => format!("{} >= '{}'", quoted_col, escaped_val),
            "LIKE" => format!("{} LIKE '{}'", quoted_col, escaped_val),
            "NOT LIKE" => format!("{} NOT LIKE '{}'", quoted_col, escaped_val),
            "IN" => {
                // value is comma-separated
                let items: Vec<String> = f
                    .value
                    .split(',')
                    .map(|s| format!("'{}'", s.trim().replace('\'', "''")))
                    .collect();
                format!("{} IN ({})", quoted_col, items.join(", "))
            }
            "NOT IN" => {
                let items: Vec<String> = f
                    .value
                    .split(',')
                    .map(|s| format!("'{}'", s.trim().replace('\'', "''")))
                    .collect();
                format!("{} NOT IN ({})", quoted_col, items.join(", "))
            }
            "IS NULL" => format!("{} IS NULL", quoted_col),
            "IS NOT NULL" => format!("{} IS NOT NULL", quoted_col),
            "contains" => format!("{} ILIKE '%{}%'", quoted_col, escaped_val),
            "starts with" => format!("{} ILIKE '{}%'", quoted_col, escaped_val),
            "ends with" => format!("{} ILIKE '%{}'", quoted_col, escaped_val),
            _ => format!("{} = '{}'", quoted_col, escaped_val),
        };
        if i == 0 {
            parts.push(condition);
        } else {
            let comb = match f.combinator.to_uppercase().as_str() {
                "OR" => "OR",
                "XOR" => "XOR",
                _ => "AND",
            };
            parts.push(format!("{} {}", comb, condition));
        }
    }
    format!(" WHERE {}", parts.join(" "))
}

/// Build an ORDER BY clause from sort column specifications.
fn build_order_clause(sort: &[SortCol]) -> String {
    if sort.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = sort
        .iter()
        .map(|s| {
            let quoted_col = format!("\"{}\"", s.column.replace('"', "\"\""));
            let dir = if s.direction.to_lowercase() == "desc" {
                "DESC"
            } else {
                "ASC"
            };
            format!("{} {}", quoted_col, dir)
        })
        .collect();
    format!(" ORDER BY {}", parts.join(", "))
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
fn pg_value_to_json(row: &sqlx::postgres::PgRow, idx: usize) -> serde_json::Value {
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
        _ => {
            // For all other types, try as String — covers text, varchar, timestamp,
            // uuid, inet, cidr, macaddr, interval, date, time, xml, bytea, numeric,
            // money, bit, range, geometry, arrays, enums, etc.
            match row.try_get::<Option<String>, _>(idx) {
                Ok(Some(v)) => tagged(canonical, serde_json::Value::String(v)),
                Ok(None) => serde_json::Value::Null,
                Err(_) => tagged_unknown(type_name),
            }
        }
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
    #[serde(default)]
    pub is_auto_increment: bool,
    #[serde(default)]
    pub is_array: bool,
    #[serde(default)]
    pub is_enum: bool,
    #[serde(default)]
    pub enum_values: Vec<String>,
    #[serde(default)]
    pub max_length: Option<i32>,
    #[serde(default)]
    pub numeric_precision: Option<i32>,
    #[serde(default)]
    pub numeric_scale: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub total_count: u64,
}

/// A single filter condition for table data queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub column: String,
    pub operator: String,
    pub value: String,
    /// Combinator with previous filter: "AND", "OR", or "XOR". Ignored for the first filter.
    pub combinator: String,
}

/// A sort column specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortCol {
    pub column: String,
    /// "asc" or "desc"
    pub direction: String,
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
