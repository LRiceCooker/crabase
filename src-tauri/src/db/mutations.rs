use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::DbState;

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

impl DbState {
    pub async fn save_changes(
        &self,
        table_name: &str,
        change_set: ChangeSet,
    ) -> Result<String, AppError> {
        let pool = self.pool().await?;
        let schema = self.schema().await;

        let quoted_schema = format!("\"{}\"", schema.replace('"', "\"\""));
        let quoted_table = format!("\"{}\"", table_name.replace('"', "\"\""));
        let qualified_table = format!("{quoted_schema}.{quoted_table}");

        let mut tx = pool
            .begin()
            .await
            .map_err(|e| AppError::db("Failed to begin transaction", e))?;

        let mut total_affected = 0u64;

        // Apply deletes
        for delete in &change_set.deletes {
            if delete.pk_values.is_empty() {
                continue;
            }
            let (where_clause, values) = build_where_clause(&delete.pk_values, 1);
            let sql = format!("DELETE FROM {qualified_table} WHERE {where_clause}");
            let mut query = sqlx::query(&sql);
            for v in &values {
                query = bind_json_value(query, v);
            }
            let result = query
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::db("Delete failed", e))?;
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
                .map_err(|e| AppError::db("Update failed", e))?;
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
            let placeholders: Vec<String> = (1..=cols.len()).map(|i| format!("${i}")).collect();
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
                .map_err(|e| AppError::db("Insert failed", e))?;
            total_affected += result.rows_affected();
        }

        tx.commit()
            .await
            .map_err(|e| AppError::db("Failed to commit transaction", e))?;

        Ok(format!("{total_affected} rows affected"))
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
        parts.push(format!("{quoted_col} = ${idx}"));
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
        parts.push(format!("{quoted_col} = ${idx}"));
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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(matches!(result.unwrap_err(), AppError::NotConnected));
    }
}
