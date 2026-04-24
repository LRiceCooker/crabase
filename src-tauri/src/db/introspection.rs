use crate::error::AppError;

use super::DbState;

impl DbState {
    /// Get a text representation of ALL schemas in the database for AI context.
    pub async fn get_full_schema_text(&self) -> Result<String, AppError> {
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
        .map_err(|e| AppError::db("Failed to get schema info", e))?;

        // Build text representation
        let mut output = String::new();
        let mut current_schema = String::new();
        let mut current_table = String::new();

        for (schema, table, column, data_type, constraint) in rows {
            if schema != current_schema {
                if !current_schema.is_empty() {
                    output.push('\n');
                }
                output.push_str(&format!("Schema: {schema}\n"));
                current_schema = schema;
                current_table.clear();
            }
            if table != current_table {
                output.push_str(&format!("  Table: {table}\n"));
                current_table = table;
            }
            let pk = if constraint.as_deref() == Some("PRIMARY KEY") { " [PK]" } else { "" };
            output.push_str(&format!("    {column} {data_type}{pk}\n"));
        }

        Ok(output)
    }
}
