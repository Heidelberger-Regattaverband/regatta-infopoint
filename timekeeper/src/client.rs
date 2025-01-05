use crate::{
    error::MessageErr,
    messages::{
        EventHeatChanged, Heat, RequestListOpenHeats, RequestStartList, ResponseListOpenHeats, ResponseStartList,
    },
    utils,
};
use colored::Colorize;
use log::{debug, info, trace, warn};
use std::{
    io::{BufRead, BufReader, BufWriter, Result as IoResult, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    str::FromStr,
    thread::{self, JoinHandle},
    time::Duration,
};

/// A client to connect to the Aquarius server.
pub(crate) struct Client {
    /// A buffered reader to read from the server.
    reader: BufReader<TcpStream>,

    /// A buffered writer to write to the server.
    writer: BufWriter<TcpStream>,

    /// The TCP stream to the server.
    stream: TcpStream,
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
    pub(crate) fn new(host: String, port: u16, timeout: u16) -> IoResult<Self> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str(&host).unwrap()), port);
        info!("Connecting to {}", addr.to_string().bold());
        let stream = TcpStream::connect_timeout(&addr, Duration::new(timeout as u64, 0))?;
        stream.set_nodelay(true)?;
        // let write_stream = stream.try_clone()?;
        let reader = BufReader::new(stream.try_clone()?);
        let writer = BufWriter::new(stream.try_clone()?);
        info!("Connected to {}", addr.to_string().bold());
        Ok(Client { reader, writer, stream })
    }

    pub(crate) fn start_receiving_events(&mut self) -> IoResult<JoinHandle<()>> {
        let mut reader = BufReader::new(self.stream.try_clone()?);
        let mut writer = BufWriter::new(self.stream.try_clone()?);

        debug!("Starting thread to receive events");
        let handle = thread::spawn(move || loop {
            debug!("Waiting for events");
            let received = Client::receive_line(&mut reader).unwrap();
            if !received.is_empty() {
                debug!("Received: \"{}\"", utils::print_whitespaces(&received).bold());
                let event_opt = EventHeatChanged::parse(&received);
                match event_opt {
                    Ok(mut event) => {
                        if event.opened {
                            Client::read_start_list(&mut reader, &mut writer, &mut event.heat).unwrap();
                        }
                    }
                    Err(err) => handle_error(err),
                }
            }
        });
        Ok(handle)
    }

    pub(crate) fn read_open_heats(&mut self) -> Result<Vec<Heat>, MessageErr> {
        Client::write(&mut self.writer, &RequestListOpenHeats::new().to_string()).map_err(MessageErr::IoError)?;
        let response = Client::receive_all(&mut self.reader).map_err(MessageErr::IoError)?;
        let mut open_heats = ResponseListOpenHeats::parse(&response)?;

        for heat in open_heats.heats.iter_mut() {
            Client::read_start_list(&mut self.reader, &mut self.writer, heat)?;
        }

        Ok(open_heats.heats)
    }

    fn read_start_list(
        reader: &mut BufReader<TcpStream>,
        writer: &mut BufWriter<TcpStream>,
        heat: &mut Heat,
    ) -> Result<(), MessageErr> {
        Client::write(writer, &RequestStartList::new(heat.id).to_string()).map_err(MessageErr::IoError)?;
        let response = Client::receive_all(reader).map_err(MessageErr::IoError)?;
        let start_list = ResponseStartList::parse(response)?;
        heat.boats = Some(start_list.boats);
        Ok(())
    }

    fn write(writer: &mut BufWriter<TcpStream>, cmd: &str) -> IoResult<usize> {
        debug!("Writing command: \"{}\"", utils::print_whitespaces(cmd).bold());
        let count = writer.write(cmd.as_bytes())?;
        writer.flush()?;
        trace!("Written {} bytes", count.to_string().bold());
        Ok(count)
    }

    fn receive_line(reader: &mut BufReader<TcpStream>) -> IoResult<String> {
        let mut line = String::new();
        let count = reader.read_line(&mut line)?;
        trace!(
            "Received {} bytes: \"{}\"",
            count.to_string().bold(),
            utils::print_whitespaces(&line).bold()
        );
        Ok(line.trim_end().to_string())
    }

    fn receive_all(reader: &mut BufReader<TcpStream>) -> IoResult<String> {
        let mut result = String::new();
        let mut buf = Vec::new();
        loop {
            // Read until a newline character is found.
            let count = reader.read_until(b'\n', &mut buf)?;
            // Convert the buffer to a string, ignoring invalid UTF-8 sequences.
            let line = String::from_utf8_lossy(&buf);
            trace!(
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
        debug!(
            "Received message (len={}): \"{}\"",
            result.len(),
            utils::print_whitespaces(&result).bold()
        );
        Ok(result.trim_end().to_string())
    }
}

fn handle_error(err: MessageErr) {
    match err {
        MessageErr::ParseError(parse_err) => {
            warn!("Error parsing number: {}", parse_err);
        }
        MessageErr::IoError(io_err) => {
            warn!("I/O error: {}", io_err);
        }
        MessageErr::InvalidMessage(message) => {
            warn!("Invalid message: {}", message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;
    use std::{
        net::{SocketAddr, TcpListener},
        thread,
    };

    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Trace)
            .try_init();
    }

    fn start_test_server() -> SocketAddr {
        init();

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
        init();

        let addr = start_test_server();
        let client = Client::new(addr.ip().to_string(), addr.port(), 1);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_write() {
        init();

        let addr = start_test_server();
        let mut client = Client::new(addr.ip().to_string(), addr.port(), 1).unwrap();
        const MESSAGE: &str = "Hello World!";
        let result = Client::write(&mut client.writer, MESSAGE);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), MESSAGE.len());
    }

    #[test]
    fn test_client_receive_line() {
        init();

        let addr = start_test_server();
        let mut client = Client::new(addr.ip().to_string(), addr.port(), 1).unwrap();
        const MESSAGE: &str = "Hello World!";
        Client::write(&mut client.writer, MESSAGE).unwrap();
        Client::write(&mut client.writer, "\r\n").unwrap();
        let response = Client::receive_line(&mut client.reader);
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), MESSAGE);
    }

    #[test]
    fn test_client_receive_all() {
        let addr = start_test_server();
        let mut client = Client::new(addr.ip().to_string(), addr.port(), 1).unwrap();
        Client::write(&mut client.writer, "Hello World!\n").unwrap();
        Client::write(&mut client.writer, "This is a test.\n").unwrap();
        Client::write(&mut client.writer, "\r\n").unwrap();
        let response = Client::receive_all(&mut client.reader);
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), "Hello World!\nThis is a test.");
    }
}
