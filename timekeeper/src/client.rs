use crate::utils;
use colored::Colorize;
use log::{debug, info};
use std::{
    io::{BufRead, BufReader, BufWriter, Result, Write},
    net::TcpStream,
};

/// A client to connect to the Aquarius server.
pub(crate) struct Client {
    /// A buffered reader to read from the server.
    reader: BufReader<TcpStream>,

    /// A buffered writer to write to the server.
    writer: BufWriter<TcpStream>,
}

impl Client {
    /// Create a new client to connect to the server. The client connects to the given host and port.
    /// # Arguments
    /// * `host` - The host to connect to.
    /// * `port` - The port to connect to.
    /// # Returns
    /// A new client to connect to the server.
    /// # Errors
    /// If the client cannot connect to the server.
    pub(crate) fn new(host: String, port: String) -> Result<Self> {
        info!("Connecting to {}:{}", host.bold(), port.bold());
        let stream = TcpStream::connect(format!("{}:{}", host, port))?;
        stream.set_nodelay(true)?;
        let write_stream = stream.try_clone()?;
        let reader = BufReader::new(stream);
        let writer = BufWriter::new(write_stream);
        info!("Connected to {}:{}", host.bold(), port.bold());
        Ok(Client { reader, writer })
    }

    pub(crate) fn write(&mut self, cmd: &str) -> Result<usize> {
        debug!("Writing command: {}", utils::print_whitespaces(cmd).bold());
        let count = self.writer.write(cmd.as_bytes())?;
        self.writer.flush()?;
        debug!("Written {} bytes", count.to_string().bold());
        Ok(count)
    }

    pub(crate) fn receive_line(&mut self) -> Result<String> {
        let mut line = String::new();
        let count = self.reader.read_line(&mut line)?;
        debug!(
            "Received {:2} bytes: \"{}\"",
            count.to_string().bold(),
            utils::print_whitespaces(&line).bold()
        );
        Ok(line.trim_end().to_string())
    }

    pub(crate) fn receive_all(&mut self) -> Result<String> {
        let mut result = String::new();
        let mut buf = Vec::new();
        loop {
            // Read until a newline character is found.
            let count = self.reader.read_until(b'\n', &mut buf)?;
            // Convert the buffer to a string, ignoring invalid UTF-8 sequences.
            let line = String::from_utf8_lossy(&buf);
            debug!(
                "Received {:2} bytes: \"{}\"",
                count.to_string().bold(),
                utils::print_whitespaces(&line).bold()
            );
            if count <= 2 {
                break;
            }
            // Append the line to the result string.
            result.push_str(&line);
            buf.clear();
        }
        debug!("Received message: \"{}\"", result.bold());
        Ok(result.trim_end().to_string())
    }
}
