use super::{pg_value_to_json, DbState};

impl DbState {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_drop_table_not_connected() {
        let state = DbState::new();
        let result = state.drop_table("test").await;
        assert!(result.is_err());
    }
}
