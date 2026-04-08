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
}
