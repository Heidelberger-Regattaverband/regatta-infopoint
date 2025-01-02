use crate::utils;
use colored::Colorize;
use log::{debug, info, trace};
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
        debug!("Writing command: \"{}\"", utils::print_whitespaces(cmd).bold());
        let count = self.writer.write(cmd.as_bytes())?;
        self.writer.flush()?;
        trace!("Written {} bytes", count.to_string().bold());
        Ok(count)
    }

    pub(crate) fn receive_line(&mut self) -> Result<String> {
        let mut line = String::new();
        let count = self.reader.read_line(&mut line)?;
        trace!(
            "Received {} bytes: \"{}\"",
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
                "Received {} bytes: \"{}\"",
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        net::{SocketAddr, TcpListener},
        thread,
    };

    fn start_test_server() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut buffer = String::new();
                while reader.read_line(&mut buffer).unwrap() > 0 {
                    stream.write_all(buffer.as_bytes()).unwrap();
                    buffer.clear();
                }
            }
        });
        addr
    }

    #[test]
    fn test_client_connection() {
        let addr = start_test_server();
        let client = Client::new(addr.ip().to_string(), addr.port().to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_write() {
        let addr = start_test_server();
        let mut client = Client::new(addr.ip().to_string(), addr.port().to_string()).unwrap();
        const MESSAGE: &str = "Hello World!";
        let result = client.write(MESSAGE);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), MESSAGE.len());
    }

    #[test]
    fn test_client_receive_line() {
        let addr = start_test_server();
        let mut client = Client::new(addr.ip().to_string(), addr.port().to_string()).unwrap();
        const MESSAGE: &str = "Hello World!";
        client.write(MESSAGE).unwrap();
        client.write("\r\n").unwrap();
        let response = client.receive_line();
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), MESSAGE);
    }

    #[test]
    fn test_client_receive_all() {
        let addr = start_test_server();
        let mut client = Client::new(addr.ip().to_string(), addr.port().to_string()).unwrap();
        client.write("Hello World!\n").unwrap();
        client.write("This is a test.\n").unwrap();
        client.write("\r\n").unwrap();
        let response = client.receive_all();
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), "Hello World!\nThis is a test.");
    }
}
