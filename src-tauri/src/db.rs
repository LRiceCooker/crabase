use sqlx::postgres::PgPool;
use std::sync::Mutex;

pub struct DbState {
    pub pool: Mutex<Option<PgPool>>,
}

impl DbState {
    pub fn new() -> Self {
        Self {
            pool: Mutex::new(None),
        }
    }

    pub async fn connect(&self, connection_string: &str) -> Result<(), String> {
        let pool = PgPool::connect(connection_string)
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        // Verify the connection actually works
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| format!("Connection validation failed: {}", e))?;

        let mut guard = self.pool.lock().map_err(|e| format!("Lock error: {}", e))?;
        *guard = Some(pool);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_state_new() {
        let state = DbState::new();
        let pool = state.pool.lock().unwrap();
        assert!(pool.is_none());
    }

    #[tokio::test]
    async fn test_connect_invalid_string() {
        let state = DbState::new();
        let result = state.connect("postgresql://invalid:invalid@localhost:9999/nope").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Connection failed"));
    }

    #[tokio::test]
    async fn test_connect_empty_string() {
        let state = DbState::new();
        let result = state.connect("").await;
        assert!(result.is_err());
    }
}
