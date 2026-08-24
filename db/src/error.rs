use ::bb8::RunError;
use ::stretto::CacheError;
use ::thiserror::Error;
use ::tiberius::error::Error as TiberiusError;

/// Database error type that wraps various error sources.
#[derive(Debug, Error)]
pub enum DbError {
    /// Tiberius database driver error.
    #[error("Tiberius database error: {0}")]
    Tiberius(#[from] TiberiusError),
    /// Connection pool error.
    #[error("Connection pool error: {0}")]
    Pool(#[from] RunError<TiberiusError>),
    /// Cache error with a descriptive message.
    #[error("Cache error: {0}")]
    CacheMessage(String),
    /// Cache driver error from the underlying stretto cache.
    #[error("Cache driver error: {0}")]
    CacheDriver(#[from] CacheError),
    /// Custom error with message.
    #[error("Database error: {0}")]
    Custom(String),
}
