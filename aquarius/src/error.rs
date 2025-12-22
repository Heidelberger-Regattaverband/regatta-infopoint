use crate::event::AquariusEvent;
use ::std::io;
use ::std::{
    num::ParseIntError,
    sync::mpsc::{RecvError, SendError},
};
use ::thiserror::Error;

/// Error type for the Aquarius crate.
#[derive(Debug, Error)]
pub enum AquariusErr {
    /// Error when parsing a string to a number.
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseIntError),

    /// Error when I/O operations fail.
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// Error when the message is invalid.
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// Error when sending a message containing an `AquariusEvent` fails.
    #[error("Send error: {0}")]
    SendError(#[from] SendError<AquariusEvent>),

    /// Error when receiving a message containing an `AquariusEvent` fails.
    #[error("Receive error: {0}")]
    ReceiveError(#[from] RecvError),

    /// Error when a mutex is poisoned.
    #[error("Mutex poisoned error: {0}")]
    MutexPoisonError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_err() {
        let parse_error = AquariusErr::ParseError("error".parse::<i32>().unwrap_err());
        let io_error = AquariusErr::IoError(io::Error::other("error"));
        let invalid_message = AquariusErr::InvalidMessage("error".to_string());

        assert!(matches!(parse_error, AquariusErr::ParseError(_)));
        assert!(matches!(io_error, AquariusErr::IoError(_)));
        assert!(matches!(invalid_message, AquariusErr::InvalidMessage { .. }));
    }
}
