//! Common error types for the database layer.

use ::stretto::CacheError;
use bb8::RunError;
use thiserror::Error;
use tiberius::error::Error as TiberiusError;

/// Database error type that wraps various error sources.
#[derive(Debug, Error)]
pub enum DbError {
    /// Tiberius database driver error.
    #[error("Tiberius database error: {0}")]
    Tiberius(#[from] TiberiusError),
    /// Connection pool error.
    #[error("Connection pool error: {0}")]
    Pool(#[from] RunError<TiberiusError>),
    /// Cache-related error.
    #[error("Cache error: {0}")]
    Cache(String),
    /// Cache-related error.
    #[error("Cache error: {0}")]
    CacheError(#[from] CacheError),
    /// Custom error with message.
    #[error("Database error: {0}")]
    Custom(String),
}
