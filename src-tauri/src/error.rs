use thiserror::Error;

/// Unified error type for all backend operations.
#[derive(Error, Debug)]
#[must_use]
pub enum AppError {
    /// Database pool is not available (not connected).
    #[error("Not connected to any database")]
    NotConnected,

    /// A database operation failed.
    #[error("{context}: {source}")]
    Database {
        context: String,
        #[source]
        source: sqlx::Error,
    },

    /// A file I/O operation failed.
    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },

    /// JSON serialization or deserialization failed.
    #[error("{context}: {source}")]
    Json {
        context: String,
        #[source]
        source: serde_json::Error,
    },

    /// URL parsing failed.
    #[error("Invalid connection string: {0}")]
    UrlParse(#[from] url::ParseError),

    /// A validation check failed (empty name, not found, etc.).
    #[error("{0}")]
    Validation(String),

    /// Catch-all for other errors.
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    /// Create a database error with context.
    pub fn db(context: impl Into<String>, source: sqlx::Error) -> Self {
        Self::Database {
            context: context.into(),
            source,
        }
    }

    /// Create an I/O error with context.
    pub fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    /// Create a JSON error with context.
    pub fn json(context: impl Into<String>, source: serde_json::Error) -> Self {
        Self::Json {
            context: context.into(),
            source,
        }
    }
}

/// Allow `?` to convert `AppError` to `String` in functions returning `Result<T, String>`.
impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}
