use crate::connection::Connection;
use crate::error::AquariusErr;
use crate::event::AquariusEvent;
use crate::messages::Bib;
use crate::messages::EventHeatChanged;
use crate::messages::Heat;
use crate::messages::RequestListOpenHeats;
use crate::messages::RequestSetTime;
use crate::messages::RequestStartList;
use crate::messages::ResponseListOpenHeats;
use crate::messages::ResponseStartList;
use ::db::timekeeper::TimeStamp;
use ::std::io;
use ::std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, ToSocketAddrs},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering::Relaxed},
        mpsc::Sender,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use ::tracing::{debug, error, info, trace, warn};

/// A client to connect to the Aquarius server.
pub struct Client {
    /// The connection to the Aquarius server.
    connection: Arc<Mutex<Option<Connection>>>,

    /// A flag to stop the watch dog thread.
    stop_watch_dog: Arc<AtomicBool>,
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
    pub fn new(host: &str, port: u16, timeout: u16, sender: Sender<AquariusEvent>) -> Result<Self, AquariusErr> {
        let mut addrs_iter = format!("{host}:{port}").to_socket_addrs()?;
        let address = addrs_iter
            .next()
            .unwrap_or_else(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port));
        let mut client = Client {
            connection: Arc::new(Mutex::new(None)),
            stop_watch_dog: Arc::new(AtomicBool::new(false)),
        };
        client.start_watch_dog(address, timeout, sender);
        Ok(client)
    }

    /// Reads the open heats from Aquarius.
    /// # Returns
    /// A vector of open heats or an error if the heats could not be read. The heats contain the boats that are in the heats.
    /// # Errors
    /// If the open heats could not be read from Aquarius.
    pub fn read_open_heats(&mut self) -> Result<Vec<Heat>, AquariusErr> {
        match self.connection.lock() {
            Ok(mut guard) => match guard.as_mut() {
                Some(connection) => {
                    connection.write(&RequestListOpenHeats::default().to_string())?;
                    let response = connection.receive_all()?;
                    let mut heats = ResponseListOpenHeats::parse(&response)?;
                    for heat in heats.heats.iter_mut() {
                        Client::read_start_list(connection, heat)?;
                    }
                    Ok(heats.heats)
                }
                None => Err(AquariusErr::NotConnectedError()),
            },
            Err(_) => Err(AquariusErr::MutexPoisonError()),
        }
    }

    /// Sends a time stamp to Aquarius.
    /// # Arguments
    /// * `time_stamp` - The time stamp to send to Aquarius.
    /// * `bib` - The bib number of the boat to send the time stamp to.
    pub fn send_time(&mut self, time_stamp: &TimeStamp, bib: Option<Bib>) -> Result<(), AquariusErr> {
        match self.connection.lock() {
            Ok(mut guard) => match guard.as_mut() {
                Some(connection) => {
                    let request = RequestSetTime {
                        time: time_stamp.time.into(),
                        split: time_stamp.split().clone(),
                        heat_nr: time_stamp.heat_nr().unwrap_or_default(),
                        bib,
                    };
                    connection.write(&request.to_string())?;
                    Ok(())
                }
                None => Err(AquariusErr::NotConnectedError()),
            },
            Err(_) => Err(AquariusErr::MutexPoisonError()),
        }
    }

    /// Starts a thread to watch the thread that receives events from Aquarius.
    /// # Returns
    /// A handle to the thread that watches the thread that receives events from Aquarius.
    /// # Errors
    /// If the thread could not be started.
    /// # Panics
    /// If the sender could not send a message to the application.
    fn start_watch_dog(&mut self, address: SocketAddr, timeout: u16, sender: Sender<AquariusEvent>) -> JoinHandle<()> {
        let connection_mutex = self.connection.clone();
        let stop_watch_dog = self.stop_watch_dog.clone();

        // Spawn a thread to watch the thread that receives events from Aquarius
        let watch_dog: JoinHandle<()> = thread::spawn(move || {
            // The interval to retry connecting to Aquarius in case of a failure
            let repeat_interval = Duration::from_millis(timeout as u64);

            // Loop until the stop flag is set
            while !stop_watch_dog.load(Relaxed) {
                let start = Instant::now();
                // create a new connection to Aquarius
                match connect(&address, timeout) {
                    Ok(connection) => {
                        // Spawn a thread to receive events from Aquarius
                        match spawn_event_thread(connection, sender.clone()) {
                            Ok(handle) => {
                                match connect(&address, timeout) {
                                    Ok(connection) => {
                                        *connection_mutex.lock().unwrap() = Some(connection);
                                        send_connection_status(&sender, true);
                                        // Wait for the thread to finish
                                        let _ = handle.join().is_ok();
                                    }
                                    Err(err) => {
                                        warn!(%err, "Error connecting to Aquarius:");
                                    }
                                }
                            }
                            Err(err) => {
                                warn!(%err, "Error spawning event thread:");
                            }
                        }
                    }
                    Err(err) => {
                        trace!(%err, "Error connecting to Aquarius:");
                    }
                }
                let mut previous_connection = connection_mutex.lock().unwrap();
                if previous_connection.is_some() {
                    *previous_connection = None;
                    send_connection_status(&sender, false);
                    info!("Disconnected from Aquarius");
                }

                let elapsed = start.elapsed();
                if elapsed < repeat_interval {
                    thread::sleep(repeat_interval - elapsed);
                }
            } // end while
            debug!("Stopped watch dog thread.");
        });

        watch_dog
    }

    fn stop_watch_dog(&mut self) {
        self.stop_watch_dog.store(true, Relaxed);
    }

    fn read_start_list(comm: &mut Connection, heat: &mut Heat) -> Result<(), AquariusErr> {
        comm.write(&RequestStartList::new(heat.id).to_string())?;
        let response = comm.receive_all()?;
        let start_list = ResponseStartList::parse(response)?;
        heat.boats = Some(start_list.boats);
        Ok(())
    }
}

fn connect(addr: &SocketAddr, timeout: u16) -> io::Result<Connection> {
    debug!(%addr, timeout, "Connecting to:");
    let stream = TcpStream::connect_timeout(addr, Duration::from_millis(timeout as u64))?;
    stream.set_nodelay(true)?;
    info!(%addr, "Connected to:");
    Connection::new(stream)
}

fn send_connection_status(sender: &Sender<AquariusEvent>, status: bool) {
    // Send a message to the application that the client is connected
    if let Err(err) = sender.send(AquariusEvent::Client(status)) {
        error!(%err, "Error sending message to application:");
    }
}

fn spawn_event_thread(mut connection: Connection, sender: Sender<AquariusEvent>) -> io::Result<JoinHandle<()>> {
    debug!("Starting thread to receive Aquarius events");
    let handle = thread::spawn(move || {
        loop {
            // Read a line from the server and blocks until a line is received.
            match connection.receive_line() {
                // successfully received a line
                Ok(received) => {
                    if !received.is_empty() {
                        // Parse the received line and handle the event
                        match EventHeatChanged::parse(&received) {
                            Ok(mut event) => {
                                if event.opened {
                                    Client::read_start_list(&mut connection, &mut event.heat).unwrap();
                                }
                                sender.send(AquariusEvent::HeatListChanged(event)).unwrap();
                            }
                            Err(err) => warn!(%err),
                        }
                    }
                }
                // an error occurred while receiving a line
                Err(err) => {
                    warn!(%err);
                    break;
                }
            }
        }
        debug!("Stopped thread to receive Aquarius events");
    });
    Ok(handle)
}

impl Drop for Client {
    fn drop(&mut self) {
        self.stop_watch_dog();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{BufRead, BufReader, Write},
        net::{SocketAddr, TcpListener},
        sync::mpsc::{self, Receiver},
        thread,
    };
    use tracing::Level;
    const TEST_MESSAGE: &str = "Hello World!";
    const EXIT_COMMAND: &str = "exit";
    const MESSAGE_END: &str = "\r\n";

    fn init_client() -> (Client, Receiver<AquariusEvent>) {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_test_writer()
            .try_init();
        let (sender, receiver) = mpsc::channel();
        let addr = start_test_server();
        let client = Client::new(&addr.ip().to_string(), addr.port(), 1, sender).unwrap();
        (client, receiver)
    }

    fn start_test_server() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            for incoming in listener.incoming() {
                let mut stream = incoming.unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut buffer = String::new();
                while reader.read_line(&mut buffer).unwrap() > 0 {
                    if buffer.trim() == EXIT_COMMAND {
                        break;
                    }
                    stream.write_all(buffer.as_bytes()).unwrap();
                    buffer.clear();
                }
            }
            info!("Exiting test server.");
        });
        addr
    }

    #[test]
    fn test_client_connection() {
        let (client, receiver) = init_client();
        receiver.recv().unwrap(); // wait until connected
        assert!(client.connection.lock().unwrap().is_some());
    }

    #[test]
    fn test_client_write() {
        let (client, receiver) = init_client();
        receiver.recv().unwrap(); // wait until connected
        let mut binding = client.connection.lock().unwrap();
        let connection = binding.as_mut().unwrap();
        let result = connection.write(TEST_MESSAGE);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TEST_MESSAGE.len());
    }

    #[test]
    #[ignore]
    fn test_client_receive_line() {
        let (client, receiver) = init_client();
        receiver.recv().unwrap(); // wait until connected
        let mut binding = client.connection.lock().unwrap();
        let connection = binding.as_mut().unwrap();
        connection.write(TEST_MESSAGE).unwrap();
        connection.write(MESSAGE_END).unwrap();
        let response = connection.receive_line();
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), TEST_MESSAGE);

        connection.write(EXIT_COMMAND).unwrap();
    }

    #[test]
    #[ignore]
    fn test_client_receive_all() {
        let (client, receiver) = init_client();
        receiver.recv().unwrap(); // wait until connected
        let mut binding = client.connection.lock().unwrap();
        let comm = binding.as_mut().unwrap();
        comm.write("Hello World!\n").unwrap();
        comm.write("This is a test.\n").unwrap();
        comm.write(MESSAGE_END).unwrap();
        let response = comm.receive_all();
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), "Hello World!\nThis is a test.");

        comm.write(EXIT_COMMAND).unwrap();
    }
}
