use crate::{
    error::MessageErr,
    messages::{
        EventHeatChanged, Heat, RequestListOpenHeats, RequestStartList, ResponseListOpenHeats, ResponseStartList,
    },
    utils,
};
use colored::Colorize;
use encoding_rs::WINDOWS_1252;
use log::{debug, info, trace, warn};
use std::{
    io::{BufRead, BufReader, BufWriter, Result as IoResult, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    str::FromStr,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

/// A trait to receive heat events from Aquarius.
pub(crate) trait HeatEventReceiver: Send + Sync + 'static {
    /// Handle an event from Aquarius.
    /// # Arguments
    /// * `event` - The event to handle.
    fn on_event(&mut self, event: &EventHeatChanged);
}

/// A client to connect to the Aquarius server.
pub(crate) struct Client {
    /// The TCP stream to the server.
    stream: TcpStream,

    /// The communication struct to handle communication with Aquarius.
    communication: Communication,
}

/// A struct to handle communication with Aquarius.
struct Communication {
    /// A buffered reader to read from Aquarius.
    reader: BufReader<TcpStream>,

    /// A buffered writer to write to the server.
    writer: BufWriter<TcpStream>,
}

impl Communication {
    fn new(stream: &TcpStream) -> IoResult<Self> {
        let reader = BufReader::new(stream.try_clone()?);
        let writer = BufWriter::new(stream.try_clone()?);
        Ok(Communication { reader, writer })
    }

    fn write(&mut self, cmd: &str) -> IoResult<usize> {
        debug!("Writing command: \"{}\"", utils::print_whitespaces(cmd).bold());
        let count = self.writer.write(cmd.as_bytes())?;
        self.writer.flush()?;
        trace!("Written {} bytes", count.to_string().bold());
        Ok(count)
    }

    fn receive_line(&mut self) -> IoResult<String> {
        let mut line = String::new();
        let count = self.reader.read_line(&mut line)?;
        debug!(
            "Received {} bytes: \"{}\"",
            count.to_string().bold(),
            utils::print_whitespaces(&line).bold()
        );
        Ok(line.trim_end().to_string())
    }

    fn receive_all(&mut self) -> IoResult<String> {
        let mut result = String::new();
        let mut buf = Vec::new();
        loop {
            // Read until a newline character is found.
            let count = self.reader.read_until(b'\n', &mut buf)?;
            let line = WINDOWS_1252.decode(&buf).0;
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

impl Client {
    /// Connects the client to Aquarius application. The client connects to the given host and port.
    /// # Arguments
    /// * `host` - The host to connect to.
    /// * `port` - The port to connect to.
    /// * `timeout` - The timeout in seconds to connect to Aquarius.
    /// # Returns
    /// A new client connected to Aquarius application or an error if the client cannot connect.
    pub(crate) fn connect(host: String, port: u16, timeout: u16) -> IoResult<Self> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str(&host).unwrap()), port);
        info!("Connecting to {}", addr.to_string().bold());
        let stream = TcpStream::connect_timeout(&addr, Duration::new(timeout as u64, 0))?;
        stream.set_nodelay(true)?;
        info!("Connected to {}", addr.to_string().bold());

        Ok(Client {
            communication: Communication::new(&stream)?,
            stream,
        })
    }

    /// Start receiving events from Aquarius.
    /// # Arguments
    /// * `receiver` - The receiver to handle the events.
    /// # Returns
    /// A handle to the thread that receives events or an error if the thread could not be started.
    pub(crate) fn start_receiving_events(
        &mut self,
        receiver: Arc<Mutex<impl HeatEventReceiver>>,
    ) -> IoResult<JoinHandle<()>> {
        let mut comm = Communication::new(&self.stream)?;

        debug!("Starting thread to receive events");
        let handle = thread::spawn(move || loop {
            let received = comm.receive_line().unwrap();
            if !received.is_empty() {
                debug!("Received: \"{}\"", utils::print_whitespaces(&received).bold());
                let event_opt = EventHeatChanged::parse(&received);
                match event_opt {
                    Ok(mut event) => {
                        if event.opened {
                            Client::read_start_list(&mut comm, &mut event.heat).unwrap();
                        }
                        receiver.lock().unwrap().on_event(&event);
                    }
                    Err(err) => handle_error(err),
                }
            }
        });
        Ok(handle)
    }

    pub(crate) fn read_open_heats(&mut self) -> Result<Vec<Heat>, MessageErr> {
        self.communication
            .write(&RequestListOpenHeats::new().to_string())
            .map_err(MessageErr::IoError)?;
        let response = self.communication.receive_all().map_err(MessageErr::IoError)?;
        let mut heats = ResponseListOpenHeats::parse(&response)?;
        for heat in heats.heats.iter_mut() {
            Client::read_start_list(&mut self.communication, heat)?;
        }
        Ok(heats.heats)
    }

    fn read_start_list(comm: &mut Communication, heat: &mut Heat) -> Result<(), MessageErr> {
        comm.write(&RequestStartList::new(heat.id).to_string())
            .map_err(MessageErr::IoError)?;
        let response = comm.receive_all().map_err(MessageErr::IoError)?;
        let start_list = ResponseStartList::parse(response)?;
        heat.boats = Some(start_list.boats);
        Ok(())
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
        let client = Client::connect(addr.ip().to_string(), addr.port(), 1);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_write() {
        init();

        let addr = start_test_server();
        let mut client = Client::connect(addr.ip().to_string(), addr.port(), 1).unwrap();
        const MESSAGE: &str = "Hello World!";
        let result = client.communication.write(MESSAGE);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), MESSAGE.len());
    }

    #[test]
    fn test_client_receive_line() {
        init();

        let addr = start_test_server();
        let mut client = Client::connect(addr.ip().to_string(), addr.port(), 1).unwrap();
        const MESSAGE: &str = "Hello World!";
        client.communication.write(MESSAGE).unwrap();
        client.communication.write("\r\n").unwrap();
        let response = client.communication.receive_line();
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), MESSAGE);
    }

    #[test]
    fn test_client_receive_all() {
        let addr = start_test_server();
        let mut client = Client::connect(addr.ip().to_string(), addr.port(), 1).unwrap();
        client.communication.write("Hello World!\n").unwrap();
        client.communication.write("This is a test.\n").unwrap();
        client.communication.write("\r\n").unwrap();
        let response = client.communication.receive_all();
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), "Hello World!\nThis is a test.");
    }
}
