use crate::{
    aquarius::{
        comm::Communication,
        messages::{
            EventHeatChanged, Heat, RequestListOpenHeats, RequestStartList, ResponseListOpenHeats, ResponseStartList,
        },
    },
    error::MessageErr,
    utils,
};
use log::{debug, info, warn};
use std::{
    io::Result as IoResult,
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

    address: SocketAddr,

    timeout: u16,
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
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str(&host).unwrap()), port);
        let stream = Self::create_stream(&address, timeout)?;
        Ok(Client {
            communication: Communication::new(&stream)?,
            stream,
            address,
            timeout,
        })
    }

    fn create_stream(addr: &SocketAddr, timeout: u16) -> IoResult<TcpStream> {
        info!("Connecting to {} with a timeout {}", addr.to_string(), timeout);
        let stream = TcpStream::connect_timeout(addr, Duration::new(timeout as u64, 0))?;
        stream.set_nodelay(true)?;
        info!("Connected to {}", addr.to_string());
        Ok(stream)
    }

    /// Start receiving events from Aquarius.
    /// # Arguments
    /// * `receiver` - The receiver to handle the events.
    /// # Returns
    /// A handle to the thread that receives events or an error if the thread could not be started.
    pub(crate) fn start_receiving_events(
        &self,
        receiver: Arc<Mutex<impl HeatEventReceiver>>,
    ) -> IoResult<JoinHandle<()>> {
        let address = self.address;
        let timeout = self.timeout;

        // Spawn a thread to watch the thread that receives events from Aquarius
        let watch_dog: JoinHandle<()> = thread::spawn(move || {
            loop {
                // create a new stream to Aquarius
                if let Ok(stream) = Self::create_stream(&address, timeout) {
                    // Spawn a thread to receive events from Aquarius
                    match Self::spawn_communication_thread(&stream, receiver.clone()) {
                        Ok(handle) => {
                            // Wait for the thread to finish
                            let _ = handle.join().is_ok();
                        }
                        Err(err) => {
                            warn!("Error spawning thread: {}", err);
                        }
                    }
                }
            }
        });

        Ok(watch_dog)
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

    fn spawn_communication_thread(
        stream: &TcpStream,
        event_receiver: Arc<Mutex<impl HeatEventReceiver>>,
    ) -> IoResult<JoinHandle<()>> {
        let mut comm = Communication::new(stream)?;

        debug!("Starting thread to receive Aquarius events.");
        let handle = thread::spawn(move || {
            loop {
                // Read a line from the server and blocks until a line is received.
                match comm.receive_line() {
                    // successfully received a line
                    Ok(received) => {
                        if !received.is_empty() {
                            debug!("Received: \"{}\"", utils::print_whitespaces(&received));
                            // Parse the received line and handle the event
                            match EventHeatChanged::parse(&received) {
                                Ok(mut event) => {
                                    if event.opened {
                                        Client::read_start_list(&mut comm, &mut event.heat).unwrap();
                                    }
                                    event_receiver.lock().unwrap().on_event(&event);
                                }
                                Err(err) => handle_error(err),
                            }
                        }
                    }
                    // an error occurred while receiving a line
                    Err(err) => {
                        handle_error(MessageErr::IoError(err));
                        break;
                    }
                }
            }
            debug!("Stopped thread to receive Aquarius events.");
        });
        Ok(handle)
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
        io::{BufRead, BufReader, Write},
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
