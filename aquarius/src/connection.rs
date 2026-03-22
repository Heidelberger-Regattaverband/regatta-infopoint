use crate::utils;
use ::encoding_rs::WINDOWS_1252;
use ::std::io;
use ::std::io::{BufRead, BufReader, BufWriter, ErrorKind, Write};
use ::std::net::Shutdown;
use ::std::net::TcpStream;
use ::tracing::trace;
use ::tracing::warn;

/// A struct to handle a connection to the Aquarius application.
pub(super) struct Connection {
    /// A buffered reader to read from the Aquarius application.
    reader: BufReader<TcpStream>,

    /// A buffered writer to write to the Aquarius application.
    writer: BufWriter<TcpStream>,

    /// The underlying TCP stream for the connection.
    stream: TcpStream,
}

impl Connection {
    /// Create a new `Connection` struct.
    /// # Arguments
    /// * `stream` - The TCP stream to communicate with Aquarius.
    /// # Returns
    /// A new `Connection` struct.
    /// # Errors
    /// An error if the stream cannot be cloned.
    pub(super) fn new(stream: TcpStream) -> io::Result<Self> {
        let reader = BufReader::new(stream.try_clone()?);
        let writer = BufWriter::new(stream.try_clone()?);
        Ok(Connection { reader, writer, stream })
    }

    /// Closes the connection to Aquarius.
    /// # Returns
    /// An empty result or an error if the connection could not be closed.
    pub(super) fn disconnect(&mut self) -> io::Result<()> {
        trace!("Disconnecting from Aquarius");
        self.writer.flush()?;
        self.stream.shutdown(Shutdown::Both)
    }

    /// Write a command to Aquarius.
    /// # Arguments
    /// * `cmd` - The command to write.
    /// # Returns
    /// The number of bytes written or an error if the command could not be written.
    pub(super) fn write(&mut self, cmd: &str) -> io::Result<usize> {
        trace!(cmd = utils::print_whitespaces(cmd), "Writing command:");
        let count = self.writer.write(cmd.as_bytes())?;
        self.writer.flush()?;
        trace!(count, "Written bytes:");
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
                    trace!(line = utils::print_whitespaces(&line), count, "Received line:");
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
                        trace!(line = utils::print_whitespaces(&line), count, "Received line:");
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
            msg = utils::print_whitespaces(&result),
            len = result.len(),
            "Received message:",
        );
        Ok(result.trim_end().to_string())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if let Err(err) = self.disconnect() {
            warn!(error = ?err, "Error while disconnecting from Aquarius");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// Helper: start a TcpListener on a random port and return it together with
    /// a `Connection` that is connected to it.
    fn setup() -> (TcpListener, Connection) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let stream = TcpStream::connect(addr).unwrap();
        let conn = Connection::new(stream).unwrap();
        (listener, conn)
    }

    // ── new ──────────────────────────────────────────────────────────────

    #[test]
    fn new_creates_connection() {
        let (listener, _conn) = setup();
        // Accept the client to prove the connection was established.
        let (_server_stream, _addr) = listener.accept().unwrap();
    }

    // ── write ────────────────────────────────────────────────────────────

    #[test]
    fn write_sends_bytes_to_server() {
        let (listener, mut conn) = setup();
        let (mut server, _) = listener.accept().unwrap();

        let msg = "HELLO\r\n";
        let count = conn.write(msg).unwrap();
        assert_eq!(count, msg.len());

        let mut buf = vec![0u8; 64];
        let n = server.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], msg.as_bytes());
    }

    #[test]
    fn write_returns_correct_byte_count() {
        let (listener, mut conn) = setup();
        let (_server, _) = listener.accept().unwrap();

        let msg = "CMD";
        let count = conn.write(msg).unwrap();
        assert_eq!(count, 3);
    }

    // ── receive_line ─────────────────────────────────────────────────────

    #[test]
    fn receive_line_reads_single_line() {
        let (listener, mut conn) = setup();
        let (mut server, _) = listener.accept().unwrap();

        server.write_all(b"Hello World\n").unwrap();
        server.flush().unwrap();

        let line = conn.receive_line().unwrap();
        assert_eq!(line, "Hello World");
    }

    #[test]
    fn receive_line_trims_crlf() {
        let (listener, mut conn) = setup();
        let (mut server, _) = listener.accept().unwrap();

        server.write_all(b"data\r\n").unwrap();
        server.flush().unwrap();

        let line = conn.receive_line().unwrap();
        assert_eq!(line, "data");
    }

    #[test]
    fn receive_line_returns_error_on_closed_connection() {
        let (listener, mut conn) = setup();
        let (server, _) = listener.accept().unwrap();

        // Close the server side so the client sees EOF.
        drop(server);

        let err = conn.receive_line().unwrap_err();
        assert_eq!(err.kind(), ErrorKind::UnexpectedEof);
    }

    // ── receive_all ──────────────────────────────────────────────────────

    #[test]
    fn receive_all_reads_multi_line_message() {
        let (listener, mut conn) = setup();
        let (mut server, _) = listener.accept().unwrap();

        // Simulate Aquarius protocol: content lines followed by an empty \r\n terminator.
        server.write_all(b"line1\r\n").unwrap();
        server.write_all(b"line2\r\n").unwrap();
        server.write_all(b"\r\n").unwrap(); // terminator (<=2 bytes after \n split)
        server.flush().unwrap();

        let result = conn.receive_all().unwrap();
        // Both content lines are concatenated (with their \r\n), then trim_end is applied.
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
    }

    #[test]
    fn receive_all_returns_error_on_closed_connection() {
        let (listener, mut conn) = setup();
        let (server, _) = listener.accept().unwrap();

        // Close server so client gets EOF immediately.
        drop(server);

        let err = conn.receive_all().unwrap_err();
        assert_eq!(err.kind(), ErrorKind::UnexpectedEof);
    }

    #[test]
    fn receive_all_handles_single_line_before_terminator() {
        let (listener, mut conn) = setup();
        let (mut server, _) = listener.accept().unwrap();

        server.write_all(b"only-line\r\n").unwrap();
        server.write_all(b"\r\n").unwrap();
        server.flush().unwrap();

        let result = conn.receive_all().unwrap();
        assert_eq!(result, "only-line");
    }

    #[test]
    fn receive_all_decodes_windows_1252() {
        let (listener, mut conn) = setup();
        let (mut server, _) = listener.accept().unwrap();

        // ä in Windows-1252 is 0xE4, ö is 0xF6, ü is 0xFC
        let mut payload: Vec<u8> = vec![0xE4, 0xF6, 0xFC]; // äöü
        payload.extend_from_slice(b"\r\n");
        server.write_all(&payload).unwrap();
        server.write_all(b"\r\n").unwrap();
        server.flush().unwrap();

        let result = conn.receive_all().unwrap();
        assert_eq!(result, "äöü");
    }

    // ── disconnect ───────────────────────────────────────────────────────

    #[test]
    fn disconnect_shuts_down_connection() {
        let (listener, mut conn) = setup();
        let (mut server, _) = listener.accept().unwrap();

        conn.disconnect().unwrap();

        // After disconnect the server should see EOF.
        let mut buf = vec![0u8; 64];
        let n = server.read(&mut buf).unwrap();
        assert_eq!(n, 0);
    }

    // ── round-trip ───────────────────────────────────────────────────────

    #[test]
    fn write_then_receive_line_round_trip() {
        let (listener, mut conn) = setup();
        let (mut server, _) = listener.accept().unwrap();

        // Client writes a command.
        conn.write("REQUEST\r\n").unwrap();

        // Server reads the command.
        let mut cmd_buf = vec![0u8; 64];
        let n = server.read(&mut cmd_buf).unwrap();
        assert_eq!(&cmd_buf[..n], b"REQUEST\r\n");

        // Server sends a response.
        server.write_all(b"RESPONSE\r\n").unwrap();
        server.flush().unwrap();

        // Client reads the response.
        let response = conn.receive_line().unwrap();
        assert_eq!(response, "RESPONSE");
    }
}
