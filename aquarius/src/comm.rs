use crate::utils;
use ::encoding_rs::WINDOWS_1252;
use ::std::io;
use ::std::{
    io::{BufRead, BufReader, BufWriter, ErrorKind, Write},
    net::TcpStream,
};
use ::tracing::{info, trace};

/// A struct to handle communication with Aquarius.
pub(super) struct Communication {
    /// A buffered reader to read from Aquarius.
    reader: BufReader<TcpStream>,

    /// A buffered writer to write to the server.
    writer: BufWriter<TcpStream>,
}

impl Communication {
    /// Create a new `Communication` struct.
    /// # Arguments
    /// * `stream` - The TCP stream to communicate with Aquarius.
    /// # Returns
    /// A new `Communication` struct.
    /// # Errors
    /// An error if the stream cannot be cloned.
    pub(super) fn new(stream: TcpStream) -> io::Result<Self> {
        let reader = BufReader::new(stream.try_clone()?);
        let writer = BufWriter::new(stream);
        Ok(Communication { reader, writer })
    }

    /// Write a command to Aquarius.
    /// # Arguments
    /// * `cmd` - The command to write.
    /// # Returns
    /// The number of bytes written or an error if the command could not be written.
    pub(super) fn write(&mut self, cmd: &str) -> io::Result<usize> {
        info!("Writing command: \"{}\"", utils::print_whitespaces(cmd));
        let count = self.writer.write(cmd.as_bytes())?;
        self.writer.flush()?;
        trace!("Written {count} bytes");
        Ok(count)
    }

    /// Receive a single line from Aquarius.
    /// # Returns
    /// The line received from Aquarius or an error if the line could not be read.
    /// # Errors
    /// An error if the connection is closed or an error occurs while reading.
    pub(super) fn receive_line(&mut self) -> io::Result<String> {
        let mut line = String::new();

        // Read a line from Aquarius and blocks until data is available.
        match self.reader.read_line(&mut line) {
            Ok(count) => {
                // If no data is read, the connection is closed.
                if count == 0 {
                    Err(io::Error::new(ErrorKind::UnexpectedEof, "Connection closed"))
                } else {
                    trace!("Received line (len={}:) \"{}\"", count, utils::print_whitespaces(&line));
                    Ok(line.trim_end().to_string())
                }
            }
            Err(err) => Err(err),
        }
    }

    /// Receive all data from Aquarius until a newline character is found.
    /// # Returns
    /// The data received from Aquarius or an error if the data could not be read.
    /// # Errors
    /// An error if the connection is closed or an error occurs while reading.
    pub(super) fn receive_all(&mut self) -> io::Result<String> {
        let mut result = String::new();
        let mut buf = Vec::new();
        loop {
            // Read until a newline character is found and blocks until data is available.
            match self.reader.read_until(b'\n', &mut buf) {
                Ok(count) => {
                    if count == 0 {
                        // If no data is read, the connection is closed.
                        return Err(io::Error::new(ErrorKind::UnexpectedEof, "Connection closed"));
                    } else {
                        // Decode the buffer to a string. Aquarius uses Windows-1252 encoding.
                        let line = WINDOWS_1252.decode(&buf).0;
                        trace!("Received line (len={count}:) \"{}\"", utils::print_whitespaces(&line));
                        // If the line is empty, break the loop. Aquarius sends \r\n at the end of the message.
                        if count <= 2 {
                            break;
                        }
                        // Append the line to the result string.
                        result.push_str(&line);
                        buf.clear();
                    }
                }
                Err(err) => return Err(err),
            }
        }
        trace!(
            "Received message (len={}): \"{}\"",
            result.len(),
            utils::print_whitespaces(&result)
        );
        Ok(result.trim_end().to_string())
    }
}
