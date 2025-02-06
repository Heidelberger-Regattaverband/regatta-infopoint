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
use log::{debug, error, info, trace, warn};
use std::{
    io::{ErrorKind, Result as IoResult},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    process,
    str::FromStr,
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

/// A client to connect to the Aquarius server.
pub(crate) struct Client {
    comm_main: Option<Communication>,

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
        let mut client = Client {
            comm_main: None,
            address,
            timeout,
            sender,
        };
        client.start_watch_dog();
        client
    }

    /// Connects the client to Aquarius application.
    /// # Errors
    /// If the client could not connect to Aquarius.
    /// # Returns
    /// A result with a unit if the client could connect to Aquarius. Otherwise, an error is returned.
    pub(crate) fn connect(&mut self) -> IoResult<()> {
        let stream = create_stream(&self.address, self.timeout)?;
        self.comm_main = Some(Communication::new(&stream)?);
        Ok(())
    }

    /// Disconnects the client from Aquarius application.
    pub(crate) fn disconnect(&mut self) {
        self.comm_main = None;
    }

    /// Reads the open heats from Aquarius.
    /// # Returns
    /// A vector of open heats or an error if the heats could not be read. The heats contain the boats that are in the heats.
    /// # Errors
    /// If the open heats could not be read from Aquarius.
    pub(crate) fn read_open_heats(&mut self) -> Result<Vec<Heat>, TimekeeperErr> {
        if let Some(comm) = &mut self.comm_main {
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

    /// Starts a thread to watch the thread that receives events from Aquarius.
    /// # Returns
    /// A handle to the thread that watches the thread that receives events from Aquarius.
    /// # Errors
    /// If the thread could not be started.
    /// # Panics
    /// If the sender could not send a message to the application.
    fn start_watch_dog(&mut self) -> JoinHandle<()> {
        let address = self.address;
        let timeout = self.timeout;
        let sender = self.sender.clone();

        // Spawn a thread to watch the thread that receives events from Aquarius
        let watch_dog: JoinHandle<()> = thread::spawn(move || {
            // The interval to retry connecting to Aquarius in case of a failure
            let repeat_interval = Duration::from_secs(timeout as u64);

            loop {
                let start = Instant::now();
                // create a new stream to Aquarius
                match create_stream(&address, timeout) {
                    Ok(stream) => {
                        // Spawn a thread to receive events from Aquarius
                        match spawn_communication_thread(&stream, sender.clone()) {
                            Ok(handle) => {
                                send_connected(&sender);
                                // Wait for the thread to finish
                                let _ = handle.join().is_ok();
                                send_disconnected(&sender);
                            }
                            Err(err) => {
                                send_disconnected(&sender);
                                warn!("Error spawning thread: {}", err);
                            }
                        }
                    }
                    Err(err) => match err.kind() {
                        ErrorKind::HostUnreachable => process::exit(1),
                        ErrorKind::NetworkUnreachable => process::exit(1),
                        _ => {
                            send_disconnected(&sender);
                        }
                    },
                }
                let elapsed = start.elapsed();
                if elapsed < repeat_interval {
                    thread::sleep(repeat_interval - elapsed);
                }
            } // end loop
        });

        watch_dog
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

fn send_connected(sender: &Sender<AppEvent>) {
    // Send a message to the application that the client is connected
    if let Err(err) = sender.send(AppEvent::Client(true)) {
        error!("Error sending message to application: {}", err);
    }
}

fn send_disconnected(sender: &Sender<AppEvent>) {
    // Send a message to the application that the client is disconnected
    if let Err(err) = sender.send(AppEvent::Client(false)) {
        error!("Error sending message to application: {}", err);
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
                                sender.send(AppEvent::Aquarius(event)).unwrap();
                            }
                            Err(err) => log_error(err),
                        }
                    }
                }
                // an error occurred while receiving a line
                Err(err) => {
                    log_error(TimekeeperErr::IoError(err));
                    break;
                }
            }
        }
        debug!("Stopped thread to receive Aquarius events.");
    });
    Ok(handle)
}

/// Logs an TimekeeperErr as a warning.
/// # Arguments
/// * `err` - The TimekeeperErr to log.
fn log_error(err: TimekeeperErr) {
    match err {
        TimekeeperErr::ParseError(parse_err) => warn!("Error parsing number: {}", parse_err),
        TimekeeperErr::IoError(io_err) => warn!("I/O error: {}", io_err),
        TimekeeperErr::InvalidMessage(message) => warn!("Invalid message: {}", message),
        TimekeeperErr::SendError(send_err) => warn!("Send error: {}", send_err),
    }
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
        let result = client.comm_main.unwrap().write(MESSAGE);
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
        let comm = client.comm_main.as_mut().unwrap();
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
        let comm = client.comm_main.as_mut().unwrap();
        comm.write("Hello World!\n").unwrap();
        comm.write("This is a test.\n").unwrap();
        comm.write("\r\n").unwrap();
        let response = comm.receive_all();
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), "Hello World!\nThis is a test.");
    }
}
