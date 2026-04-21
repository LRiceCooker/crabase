use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::collections::HashMap;

use super::DbState;

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

impl DbState {
    pub async fn list_tables(&self) -> Result<Vec<String>, String> {
        let pool = self.pool().await?;
        let schema = self.schema().await;

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
        let pool = self.pool().await?;
        let schema = self.schema().await;

        // Extended query: fetch type info, max_length, precision, scale, default, UDT name + schema
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
            Option<String>,  // udt_schema
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
                c.udt_name,
                c.udt_schema
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
        for (name, data_type, is_nullable, constraint_type, max_len, precision, scale, col_default, udt_name, udt_schema) in rows {
            let is_auto = col_default
                .as_deref()
                .map(|d| d.starts_with("nextval("))
                .unwrap_or(false);
            let is_array = data_type == "ARRAY";
            let is_enum = data_type == "USER-DEFINED";

            // Fetch enum values if this is an enum column
            // Use the enum's own schema (udt_schema), not the table's schema
            let enum_values = if is_enum {
                if let Some(ref udt) = udt_name {
                    let enum_schema = udt_schema.as_deref().unwrap_or(&schema);
                    Self::fetch_enum_values(&pool, enum_schema, udt).await.unwrap_or_default()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            // Normalize long-form Postgres type names to short canonical forms
            let normalized_type = if is_enum {
                udt_name.clone().unwrap_or(data_type)
            } else if is_array {
                udt_name.clone().unwrap_or(data_type)
            } else {
                match data_type.as_str() {
                    "timestamp without time zone" => "timestamp".to_string(),
                    "timestamp with time zone" => "timestamptz".to_string(),
                    "time without time zone" => "time".to_string(),
                    "time with time zone" => "timetz".to_string(),
                    "character varying" => "varchar".to_string(),
                    "character" => "char".to_string(),
                    "double precision" => "double".to_string(),
                    "bit varying" => "varbit".to_string(),
                    "boolean" => "boolean".to_string(),
                    other => other.to_string(),
                }
            };

            columns.push(ColumnInfo {
                name,
                data_type: normalized_type,
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
    ) -> Result<HashMap<String, Vec<String>>, String> {
        let mut result = HashMap::new();
        for table_name in table_names {
            let columns = self.get_column_info(table_name).await?;
            result.insert(
                table_name.clone(),
                columns.into_iter().map(|c| c.name).collect(),
            );
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
