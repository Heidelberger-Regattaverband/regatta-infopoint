use crate::app::AppEvent;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
    num::ParseIntError,
    sync::mpsc::SendError,
};

/// Error type for the timekeeper crate.
#[derive(Debug)]
pub(crate) enum TimekeeperErr {
    /// Error when parsing a string to a number.
    ParseError(ParseIntError),

    /// Error when I/O operations fail.
    IoError(IoError),

    /// Error when the message is invalid.
    InvalidMessage(String),

    /// Error when sending a message containing an `AppEvent` fails.
    SendError(SendError<AppEvent>),
}

impl Display for TimekeeperErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TimekeeperErr::ParseError(err) => write!(f, "Parse error: {}", err),
            TimekeeperErr::IoError(err) => write!(f, "I/O error: {}", err),
            TimekeeperErr::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
            TimekeeperErr::SendError(err) => write!(f, "Send error: {}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn test_message_err() {
        let parse_error = TimekeeperErr::ParseError("error".parse::<i32>().unwrap_err());
        let io_error = TimekeeperErr::IoError(IoError::new(ErrorKind::Other, "error"));
        let invalid_message = TimekeeperErr::InvalidMessage("error".to_string());

        assert!(matches!(parse_error, TimekeeperErr::ParseError(_)));
        assert!(matches!(io_error, TimekeeperErr::IoError(_)));
        assert!(matches!(invalid_message, TimekeeperErr::InvalidMessage { .. }));
    }
}
