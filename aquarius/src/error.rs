use crate::event::AquariusEvent;
use ::std::error::Error;
use ::std::fmt;
use ::std::io;
use ::std::{
    fmt::{Display, Formatter},
    num::ParseIntError,
    sync::mpsc::{RecvError, SendError},
};

/// Error type for the Aquarius crate.
#[derive(Debug)]
pub enum AquariusErr {
    /// Error when parsing a string to a number.
    ParseError(ParseIntError),

    /// Error when I/O operations fail.
    IoError(io::Error),

    /// Error when the message is invalid.
    InvalidMessage(String),

    /// Error when sending a message containing an `AquariusEvent` fails.
    SendError(SendError<AquariusEvent>),

    /// Error when receiving a message containing an `AquariusEvent` fails.
    ReceiveError(RecvError),
}

impl Display for AquariusErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AquariusErr::ParseError(err) => write!(f, "Parse error: {err}"),
            AquariusErr::IoError(err) => write!(f, "I/O error: {err}"),
            AquariusErr::InvalidMessage(msg) => write!(f, "Invalid message: {msg}"),
            AquariusErr::SendError(err) => write!(f, "Send error: {err}"),
            AquariusErr::ReceiveError(err) => write!(f, "Receive error: {err}"),
        }
    }
}

impl Error for AquariusErr {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AquariusErr::ParseError(err) => Some(err),
            AquariusErr::IoError(err) => Some(err),
            AquariusErr::SendError(err) => Some(err),
            AquariusErr::ReceiveError(err) => Some(err),
            AquariusErr::InvalidMessage(_) => None,
        }
    }
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

    #[test]
    fn test_error_trait_implementation() {
        let parse_error = AquariusErr::ParseError("error".parse::<i32>().unwrap_err());
        let io_error = AquariusErr::IoError(io::Error::other("test error"));
        let invalid_message = AquariusErr::InvalidMessage("test message".to_string());

        // Test that errors implement the Error trait
        let _: &dyn Error = &parse_error;
        let _: &dyn Error = &io_error;
        let _: &dyn Error = &invalid_message;

        // Test source() method returns appropriate underlying errors
        assert!(parse_error.source().is_some());
        assert!(io_error.source().is_some());
        assert!(invalid_message.source().is_none());

        // Test that we can chain errors properly
        let chained_error = format!("{}: {}", parse_error, parse_error.source().unwrap());
        assert!(chained_error.contains("Parse error"));
    }
}
