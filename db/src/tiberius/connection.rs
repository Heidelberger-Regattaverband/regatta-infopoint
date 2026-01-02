use bb8::ManageConnection;
use tiberius::{Client, Config, error::Error as TiberiusError};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

/// The type of a Tiberius connection.
pub type TiberiusClient = Client<Compat<TcpStream>>;

/// A connection manager for Tiberius connections.
#[derive(Debug)]
pub struct TiberiusConnectionManager {
    /// The database configuration.
    config: Config,
}

impl TiberiusConnectionManager {
    /// Creates a new `TiberiusConnectionManager`.
    ///
    /// # Arguments
    /// * `config` - The configuration for the Tiberius connection manager.
    /// # Returns
    /// A new instance of `TiberiusConnectionManager`.
    pub fn new(config: Config) -> TiberiusConnectionManager {
        TiberiusConnectionManager { config }
    }
}

/// Implements the `ManageConnection` trait for `TiberiusConnectionManager`.
/// This allows the connection manager to create, validate, and manage Tiberius connections.
impl ManageConnection for TiberiusConnectionManager {
    type Connection = TiberiusClient;
    type Error = TiberiusError;

    /// Establishes a new connection to the database. This implementation creates a new TCP stream and connects to
    /// the database using the provided configuration.
    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        create_client(&self.config).await
    }

    /// Checks if the connection is valid. This implementation sends a simple query to the database.
    async fn is_valid(&self, connection: &mut Self::Connection) -> Result<(), Self::Error> {
        connection.simple_query("").await?.into_row().await?;
        Ok(())
    }

    /// Checks if the connection is broken. This implementation always returns false.
    fn has_broken(&self, _connection: &mut Self::Connection) -> bool {
        false
    }
}

/// Creates a new Tiberius client connection using the provided configuration.
/// # Arguments
/// * `config` - The configuration for the Tiberius connection.
/// # Returns
/// A result containing the Tiberius client connection or an error if the connection fails.
/// # Errors
/// This function will return an error if the TCP connection or Tiberius client connection fails.
pub async fn create_client(config: &Config) -> Result<TiberiusClient, TiberiusError> {
    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;
    Client::connect(config.clone(), tcp.compat_write()).await
}
