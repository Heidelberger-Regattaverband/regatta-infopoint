use std::{io, num};

/// Error type for the timekeeper crate.
enum MessageErr {
    /// Error when parsing a string to a number.
    ParseError(num::ParseIntError),

    /// Error when I/O operations fail.
    IoError(io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_err() {
        let parse_error = MessageErr::ParseError("error".parse::<i32>().unwrap_err());
        let io_error = MessageErr::IoError(std::io::Error::new(std::io::ErrorKind::Other, "error"));

        assert!(matches!(parse_error, MessageErr::ParseError(_)));
        assert!(matches!(io_error, MessageErr::IoError(_)));
    }
}
