use bb8::ManageConnection;
use tiberius::{Client, Config, error::Error};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

/// The type of a Tiberius connection.
pub type TiberiusConnection = Client<Compat<TcpStream>>;

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

impl ManageConnection for TiberiusConnectionManager {
    type Connection = TiberiusConnection;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let tcp = TcpStream::connect(self.config.get_addr()).await?;
        tcp.set_nodelay(true)?;
        Client::connect(self.config.clone(), tcp.compat_write()).await
    }

    async fn is_valid(&self, connection: &mut Self::Connection) -> Result<(), Self::Error> {
        //debug!("Checking {:?}", conn);
        connection.simple_query("").await?.into_row().await?;
        Ok(())
    }

    fn has_broken(&self, _connection: &mut Self::Connection) -> bool {
        false
    }
}
