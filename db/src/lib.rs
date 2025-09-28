//! Database layer for regatta management system.
//!
//! This crate provides database connectivity and data models for the regatta
//! information portal. It supports both Aquarius (regatta management) and
//! timekeeper data sources.

pub mod aquarius;
pub mod tiberius;
pub mod timekeeper;

/// Common error types for the database layer.
mod error {
    use bb8::RunError;
    use std::{
        error::Error,
        fmt::{Display, Formatter, Result},
    };
    use tiberius::error::Error as TiberiusError;

    /// Database error type that wraps various error sources.
    #[derive(Debug)]
    pub enum DbError {
        /// Tiberius database driver error.
        Tiberius(TiberiusError),
        /// Connection pool error.
        Pool(RunError<TiberiusError>),
        /// Custom error with message.
        Custom(String),
    }

    impl Display for DbError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                DbError::Tiberius(e) => write!(f, "Database error: {}", e),
                DbError::Pool(e) => write!(f, "Connection pool error: {}", e),
                DbError::Custom(msg) => write!(f, "Database error: {}", msg),
            }
        }
    }

    impl Error for DbError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                DbError::Tiberius(e) => Some(e),
                DbError::Pool(e) => Some(e),
                DbError::Custom(_) => None,
            }
        }
    }

    impl From<TiberiusError> for DbError {
        fn from(err: TiberiusError) -> Self {
            DbError::Tiberius(err)
        }
    }

    impl From<RunError<TiberiusError>> for DbError {
        fn from(err: RunError<TiberiusError>) -> Self {
            DbError::Pool(err)
        }
    }

    impl From<String> for DbError {
        fn from(err: String) -> Self {
            DbError::Custom(err)
        }
    }
}
