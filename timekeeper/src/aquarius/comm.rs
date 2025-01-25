use crate::utils;
use encoding_rs::WINDOWS_1252;
use log::{debug, trace};
use std::{
    io::{BufRead, BufReader, BufWriter, Error as IoError, ErrorKind, Result as IoResult, Write},
    net::TcpStream,
};

/// A struct to handle communication with Aquarius.
pub(super) struct Communication {
    /// A buffered reader to read from Aquarius.
    reader: BufReader<TcpStream>,

    /// A buffered writer to write to the server.
    writer: BufWriter<TcpStream>,
}

impl Communication {
    pub(super) fn new(stream: &TcpStream) -> IoResult<Self> {
        let reader = BufReader::new(stream.try_clone()?);
        let writer = BufWriter::new(stream.try_clone()?);
        Ok(Communication { reader, writer })
    }

    pub(super) fn write(&mut self, cmd: &str) -> IoResult<usize> {
        debug!("Writing command: \"{}\"", utils::print_whitespaces(cmd));
        let count = self.writer.write(cmd.as_bytes())?;
        self.writer.flush()?;
        trace!("Written {} bytes", count.to_string());
        Ok(count)
    }

    pub(super) fn receive_line(&mut self) -> IoResult<String> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(count) => {
                if count == 0 {
                    Err(IoError::new(ErrorKind::UnexpectedEof, "Connection closed"))
                } else {
                    debug!(
                        "Received {} bytes: \"{}\"",
                        count.to_string(),
                        utils::print_whitespaces(&line)
                    );
                    Ok(line.trim_end().to_string())
                }
            }
            Err(err) => Err(err),
        }
    }

    pub(super) fn receive_all(&mut self) -> IoResult<String> {
        let mut result = String::new();
        let mut buf = Vec::new();
        loop {
            // Read until a newline character is found.
            let count = self.reader.read_until(b'\n', &mut buf)?;
            let line = WINDOWS_1252.decode(&buf).0;
            trace!(
                "Received {} bytes: \"{}\"",
                count.to_string(),
                utils::print_whitespaces(&line)
            );
            if count <= 2 {
                break;
            }
            // Append the line to the result string.
            result.push_str(&line);
            buf.clear();
        }
        debug!(
            "Received message (len={}): \"{}\"",
            result.len(),
            utils::print_whitespaces(&result)
        );
        Ok(result.trim_end().to_string())
    }
}
