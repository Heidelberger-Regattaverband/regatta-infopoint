use ::std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
    num::ParseIntError,
    sync::mpsc::{RecvError, SendError},
};

use crate::event::AquariusEvent;

/// Error type for the timekeeper crate.
#[derive(Debug)]
pub enum TimekeeperErr {
    /// Error when parsing a string to a number.
    ParseError(ParseIntError),

    /// Error when I/O operations fail.
    IoError(IoError),

    /// Error when the message is invalid.
    InvalidMessage(String),

    /// Error when sending a message containing an `AquariusEvent` fails.
    SendError(SendError<AquariusEvent>),

    /// Error when receiving a message containing an `AquariusEvent` fails.
    ReceiveError(RecvError),
}

impl Display for TimekeeperErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TimekeeperErr::ParseError(err) => write!(f, "Parse error: {err}"),
            TimekeeperErr::IoError(err) => write!(f, "I/O error: {err}"),
            TimekeeperErr::InvalidMessage(msg) => write!(f, "Invalid message: {msg}"),
            TimekeeperErr::SendError(err) => write!(f, "Send error: {err}"),
            TimekeeperErr::ReceiveError(err) => write!(f, "Receive error: {err}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_err() {
        let parse_error = TimekeeperErr::ParseError("error".parse::<i32>().unwrap_err());
        let io_error = TimekeeperErr::IoError(IoError::other("error"));
        let invalid_message = TimekeeperErr::InvalidMessage("error".to_string());

        assert!(matches!(parse_error, TimekeeperErr::ParseError(_)));
        assert!(matches!(io_error, TimekeeperErr::IoError(_)));
        assert!(matches!(invalid_message, TimekeeperErr::InvalidMessage { .. }));
    }
}
