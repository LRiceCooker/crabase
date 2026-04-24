use crate::error::AppError;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Column, Row};

use super::{pg_value_to_json, DbState};

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

impl DbState {
    pub async fn execute_query(&self, sql: &str) -> Result<QueryResult, AppError> {
        let pool = self.pool().await?;

        // Use raw_sql (simple query protocol) — returns all values as text,
        // avoiding binary protocol issues with enums, arrays, and custom types.
        let mut stream = sqlx::raw_sql(sql).fetch_many(&pool);
        let mut columns: Vec<String> = Vec::new();
        let mut rows: Vec<Vec<serde_json::Value>> = Vec::new();

        while let Some(either) = stream.try_next().await.map_err(|e| AppError::db("Query failed", e))? {
            if let sqlx::Either::Right(row) = either {
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
        }

        Ok(QueryResult { columns, rows })
    }

    /// Execute multiple SQL statements using the simple query protocol (raw_sql).
    /// Returns a Vec<StatementResult> — one entry per statement.
    /// The simple protocol returns all values as text, avoiding binary issues with enums/arrays.
    pub async fn execute_query_multi(&self, sql: &str) -> Result<Vec<StatementResult>, AppError> {
        let pool = self.pool().await?;

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

        while let Some(either) = stream.try_next().await.map_err(|e| AppError::db("Query failed", e))? {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_query_not_connected() {
        let state = DbState::new();
        let result = state.execute_query("SELECT 1").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotConnected));
    }
}
