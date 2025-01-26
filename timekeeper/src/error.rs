use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
    num,
};

/// Error type for the timekeeper crate.
#[derive(Debug)]
pub(crate) enum MessageErr {
    /// Error when parsing a string to a number.
    ParseError(num::ParseIntError),

    /// Error when I/O operations fail.
    IoError(IoError),

    /// Error when the message is invalid.
    InvalidMessage(String),
}

impl Display for MessageErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            MessageErr::ParseError(err) => write!(f, "Parse error: {}", err),
            MessageErr::IoError(err) => write!(f, "I/O error: {}", err),
            MessageErr::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn test_message_err() {
        let parse_error = MessageErr::ParseError("error".parse::<i32>().unwrap_err());
        let io_error = MessageErr::IoError(IoError::new(ErrorKind::Other, "error"));
        let invalid_message = MessageErr::InvalidMessage("error".to_string());

        assert!(matches!(parse_error, MessageErr::ParseError(_)));
        assert!(matches!(io_error, MessageErr::IoError(_)));
        assert!(matches!(invalid_message, MessageErr::InvalidMessage { .. }));
    }
}
