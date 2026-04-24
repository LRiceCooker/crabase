use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use super::{pg_value_to_json, ColumnInfo, DbState};

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

impl DbState {
    pub async fn get_table_data(
        &self,
        table_name: &str,
        page: u32,
        page_size: u32,
    ) -> Result<TableData, AppError> {
        let columns = self.get_column_info(table_name).await?;

        let pool = self.pool().await?;
        let schema = self.schema().await;

        // Use quoted identifiers to prevent SQL injection
        let quoted_schema = format!("\"{}\"", schema.replace('"', "\"\""));
        let quoted_table = format!("\"{}\"", table_name.replace('"', "\"\""));
        let qualified_table = format!("{quoted_schema}.{quoted_table}");

        // Get total count
        let count_query = format!("SELECT COUNT(*) as cnt FROM {qualified_table}");
        let count_row: (i64,) = sqlx::query_as(&count_query)
            .fetch_one(&pool)
            .await
            .map_err(|e| AppError::db("Failed to get row count", e))?;
        let total_count = count_row.0 as u64;

        // Get paginated rows with smart default ordering
        // Cast enum columns to text so sqlx can decode them properly
        let select_cols = build_select_columns(&columns);
        let order_clause = default_order_clause(&columns);
        let offset = (page.saturating_sub(1)) as i64 * page_size as i64;
        let data_query = format!(
            "SELECT {select_cols} FROM {qualified_table}{order_clause} LIMIT {page_size} OFFSET {offset}"
        );

        let pg_rows = sqlx::query(&data_query)
            .fetch_all(&pool)
            .await
            .map_err(|e| AppError::db("Failed to get table data", e))?;

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
    ) -> Result<TableData, AppError> {
        let columns = self.get_column_info(table_name).await?;

        let pool = self.pool().await?;
        let schema = self.schema().await;

        let quoted_schema = format!("\"{}\"", schema.replace('"', "\"\""));
        let quoted_table = format!("\"{}\"", table_name.replace('"', "\"\""));
        let qualified_table = format!("{quoted_schema}.{quoted_table}");

        // Build WHERE clause from filters
        let where_clause = build_filter_where_clause(&filters);

        // Build ORDER BY clause from sort columns (or apply smart default)
        let order_clause = if sort.is_empty() {
            default_order_clause(&columns)
        } else {
            build_order_clause(&sort)
        };

        // Get filtered count
        let count_query = format!(
            "SELECT COUNT(*) as cnt FROM {}{}",
            qualified_table, where_clause
        );
        let count_row: (i64,) = sqlx::query_as(&count_query)
            .fetch_one(&pool)
            .await
            .map_err(|e| AppError::db("Failed to get row count", e))?;
        let total_count = count_row.0 as u64;

        // Get paginated filtered rows
        // Cast enum columns to text so sqlx can decode them properly
        let select_cols = build_select_columns(&columns);
        let offset = (page.saturating_sub(1)) as i64 * page_size as i64;
        let data_query = format!(
            "SELECT {select_cols} FROM {qualified_table}{where_clause}{order_clause} LIMIT {page_size} OFFSET {offset}"
        );

        let pg_rows = sqlx::query(&data_query)
            .fetch_all(&pool)
            .await
            .map_err(|e| AppError::db("Failed to get table data", e))?;

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
}

/// Build a SELECT column list, casting enum and array columns to text
/// so that sqlx can decode their values as strings without binary protocol issues.
fn build_select_columns(columns: &[ColumnInfo]) -> String {
    columns
        .iter()
        .map(|col| {
            let quoted = format!("\"{}\"", col.name.replace('"', "\"\""));
            if col.is_enum || col.is_array {
                format!("{quoted}::text AS {quoted}")
            } else {
                quoted
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Build a WHERE clause from user-defined filters. Values are embedded as literals
/// with single-quote doubling (safe for PostgreSQL with standard_conforming_strings=on).
/// Column names are identifier-quoted. LIKE patterns escape metacharacters.
fn build_filter_where_clause(filters: &[Filter]) -> String {
    if filters.is_empty() {
        return String::new();
    }
    let mut parts = Vec::new();
    for (i, f) in filters.iter().enumerate() {
        let quoted_col = format!("\"{}\"", f.column.replace('"', "\"\""));
        let escaped_val = f.value.replace('\'', "''");
        // For LIKE patterns, also escape % and _ metacharacters
        let like_escaped = escaped_val.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
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
            "contains" => format!("{} ILIKE '%{}%' ESCAPE '\\'", quoted_col, like_escaped),
            "starts with" => format!("{} ILIKE '{}%' ESCAPE '\\'", quoted_col, like_escaped),
            "ends with" => format!("{} ILIKE '%{}' ESCAPE '\\'", quoted_col, like_escaped),
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
            parts.push(format!("{comb} {condition}"));
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
            format!("{quoted_col} {dir}")
        })
        .collect();
    format!(" ORDER BY {}", parts.join(", "))
}

/// Compute a default ORDER BY clause based on column metadata.
/// Priority: created_at DESC > PK ASC > first column ASC.
fn default_order_clause(columns: &[ColumnInfo]) -> String {
    // 1. Check for created_at column
    if let Some(col) = columns.iter().find(|c| c.name == "created_at") {
        let quoted = format!("\"{}\"", col.name.replace('"', "\"\""));
        return format!(" ORDER BY {} DESC", quoted);
    }
    // 2. Check for primary key column
    if let Some(col) = columns.iter().find(|c| c.is_primary_key) {
        let quoted = format!("\"{}\"", col.name.replace('"', "\"\""));
        return format!(" ORDER BY {} ASC", quoted);
    }
    // 3. Fall back to first column
    if let Some(col) = columns.first() {
        let quoted = format!("\"{}\"", col.name.replace('"', "\"\""));
        return format!(" ORDER BY {} ASC", quoted);
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_table_data_not_connected() {
        let state = DbState::new();
        let result = state.get_table_data("some_table", 1, 25).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotConnected));
    }
}
