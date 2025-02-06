use crate::{
    app::AppEvent,
    aquarius::{
        comm::Communication,
        messages::{
            EventHeatChanged, Heat, RequestListOpenHeats, RequestStartList, ResponseListOpenHeats, ResponseStartList,
        },
    },
    error::TimekeeperErr,
    utils,
};
use log::{debug, info, trace, warn};
use std::{
    io::Result as IoResult,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    str::FromStr,
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
    time::Duration,
};

/// A client to connect to the Aquarius server.
pub(crate) struct Client {
    /// The communication struct to handle events from Aquarius.
    comm_events: Option<Communication>,

    address: SocketAddr,

    timeout: u16,

    sender: Sender<AppEvent>,
}

impl Client {
    /// Connects the client to Aquarius application. The client connects to the given host and port.
    /// # Arguments
    /// * `host` - The host to connect to.
    /// * `port` - The port to connect to.
    /// * `timeout` - The timeout in seconds to connect to Aquarius.
    /// * `sender` - The sender to send events to the application.
    /// # Returns
    /// A client to communicate with Aquarius.
    pub(crate) fn new(host: &str, port: u16, timeout: u16, sender: Sender<AppEvent>) -> Self {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str(host).unwrap()), port);
        Client {
            comm_events: None,
            address,
            timeout,
            sender,
        }
    }

    /// Start receiving events from Aquarius.
    /// # Arguments
    /// * `sender` - The sender to send events to the application.
    /// # Returns
    /// A handle to the thread that receives events or an error if the thread could not be started.
    pub(crate) fn connect(&mut self) -> IoResult<JoinHandle<()>> {
        let address = self.address;
        let timeout = self.timeout;
        let sender = self.sender.clone();

        let stream = create_stream(&address, timeout)?;
        self.comm_events = Some(Communication::new(&stream)?);

        // Spawn a thread to watch the thread that receives events from Aquarius
        let watch_dog: JoinHandle<()> = thread::spawn(move || {
            loop {
                // create a new stream to Aquarius
                if let Ok(stream) = create_stream(&address, timeout) {
                    // Spawn a thread to receive events from Aquarius
                    match spawn_communication_thread(&stream, sender.clone()) {
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

    /// Reads the open heats from Aquarius.
    /// # Returns
    /// A vector of open heats or an error if the heats could not be read. The heats contain the boats that are in the heats.
    /// # Errors
    /// If the open heats could not be read from Aquarius.
    pub(crate) fn read_open_heats(&mut self) -> Result<Vec<Heat>, TimekeeperErr> {
        if let Some(comm) = &mut self.comm_events {
            comm.write(&RequestListOpenHeats::new().to_string())
                .map_err(TimekeeperErr::IoError)?;
            let response = comm.receive_all().map_err(TimekeeperErr::IoError)?;
            let mut heats = ResponseListOpenHeats::parse(&response)?;
            for heat in heats.heats.iter_mut() {
                Client::read_start_list(comm, heat)?;
            }
            Ok(heats.heats)
        } else {
            Err(TimekeeperErr::InvalidMessage(
                "Communication is not initialized.".to_string(),
            ))
        }
    }

    fn read_start_list(comm: &mut Communication, heat: &mut Heat) -> Result<(), TimekeeperErr> {
        comm.write(&RequestStartList::new(heat.id).to_string())
            .map_err(TimekeeperErr::IoError)?;
        let response = comm.receive_all().map_err(TimekeeperErr::IoError)?;
        let start_list = ResponseStartList::parse(response)?;
        heat.boats = Some(start_list.boats);
        Ok(())
    }
}

fn handle_error(err: TimekeeperErr) {
    match err {
        TimekeeperErr::ParseError(parse_err) => {
            warn!("Error parsing number: {}", parse_err);
        }
        TimekeeperErr::IoError(io_err) => {
            warn!("I/O error: {}", io_err);
        }
        TimekeeperErr::InvalidMessage(message) => {
            warn!("Invalid message: {}", message);
        }
        TimekeeperErr::SendError(send_err) => {
            warn!("Send error: {}", send_err);
        }
    }
}

fn create_stream(addr: &SocketAddr, timeout: u16) -> IoResult<TcpStream> {
    trace!("Connecting to {} with a timeout {}", addr.to_string(), timeout);
    let stream = TcpStream::connect_timeout(addr, Duration::new(timeout as u64, 0))?;
    stream.set_nodelay(true)?;
    info!("Connected to {}", addr.to_string());
    Ok(stream)
}

fn spawn_communication_thread(stream: &TcpStream, sender: Sender<AppEvent>) -> IoResult<JoinHandle<()>> {
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
                                sender.send(AppEvent::AquariusEvent(event)).unwrap();
                            }
                            Err(err) => handle_error(err),
                        }
                    }
                }
                // an error occurred while receiving a line
                Err(err) => {
                    handle_error(TimekeeperErr::IoError(err));
                    break;
                }
            }
        }
        debug!("Stopped thread to receive Aquarius events.");
    });
    Ok(handle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;
    use std::{
        io::{BufRead, BufReader, Write},
        net::{SocketAddr, TcpListener},
        sync::mpsc::{self, Receiver},
        thread,
    };

    fn init() -> (Sender<AppEvent>, Receiver<AppEvent>) {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Trace)
            .try_init();
        mpsc::channel()
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
        let (sender, _) = init();

        let addr = start_test_server();
        let mut client = Client::new(&addr.ip().to_string(), addr.port(), 1, sender);
        let result = client.connect();
        assert!(result.is_ok());
    }

    #[test]
    fn test_client_write() {
        let (sender, _) = init();

        let addr = start_test_server();
        let mut client = Client::new(&addr.ip().to_string(), addr.port(), 1, sender);
        client.connect().unwrap();
        const MESSAGE: &str = "Hello World!";
        let result = client.comm_events.unwrap().write(MESSAGE);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), MESSAGE.len());
    }

    #[test]
    fn test_client_receive_line() {
        let (sender, _) = init();

        let addr = start_test_server();
        let mut client = Client::new(&addr.ip().to_string(), addr.port(), 1, sender);
        client.connect().unwrap();
        const MESSAGE: &str = "Hello World!";
        let comm = client.comm_events.as_mut().unwrap();
        comm.write(MESSAGE).unwrap();
        comm.write("\r\n").unwrap();
        let response = comm.receive_line();
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), MESSAGE);
    }

    #[test]
    fn test_client_receive_all() {
        let (sender, _) = init();

        let addr = start_test_server();
        let mut client = Client::new(&addr.ip().to_string(), addr.port(), 1, sender);
        client.connect().unwrap();
        let comm = client.comm_events.as_mut().unwrap();
        comm.write("Hello World!\n").unwrap();
        comm.write("This is a test.\n").unwrap();
        comm.write("\r\n").unwrap();
        let response = comm.receive_all();
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), "Hello World!\nThis is a test.");
    }
}
