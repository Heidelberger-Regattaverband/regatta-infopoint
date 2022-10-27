use async_std::net::TcpStream;
use async_trait::async_trait;
use bb8::Pool;
use log::info;
use std::env;
use tiberius::Config;

#[derive(Clone, Debug)]
pub struct TiberiusConnectionManager {
    config: Config,
}

impl TiberiusConnectionManager {
    /// Create a new `TiberiusConnectionManager`.
    fn new() -> tiberius::Result<TiberiusConnectionManager> {
        let config = create_config();
        Ok(TiberiusConnectionManager { config })
    }
}

#[async_trait]
impl bb8::ManageConnection for TiberiusConnectionManager {
    type Connection = tiberius::Client<TcpStream>;
    type Error = tiberius::error::Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let tcp = TcpStream::connect(&self.config.get_addr()).await?;
        tcp.set_nodelay(true)?;
        tiberius::Client::connect(self.config.clone(), tcp).await
    }

    async fn is_valid(&self, connection: &mut Self::Connection) -> Result<(), Self::Error> {
        //debug!("Checking {:?}", conn);
        connection.simple_query("").await?.into_row().await?;
        Ok(())
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}

pub async fn create_pool() -> Pool<TiberiusConnectionManager> {
    let manager = TiberiusConnectionManager::new().unwrap();
    Pool::builder().max_size(5).build(manager).await.unwrap()
}

fn get_db_password() -> String {
    env::var("DB_PASSWORD").expect("env variable `DB_PASSWORD` should be set")
}

fn create_config() -> tiberius::Config {
    let db_host = env::var("DB_HOST").expect("env variable `DB_HOST` should be set");
    let db_port = env::var("DB_PORT")
        .expect("env variable `DB_PORT` should be set")
        .parse()
        .unwrap();
    let db_name = env::var("DB_NAME").expect("env variable `DB_NAME` should be set");
    let db_user = env::var("DB_USER").expect("env variable `DB_USER` should be set");

    info!(
        "Database configuration: host={}, port={}, name={}, user={}",
        db_host, db_port, db_name, db_user
    );

    let mut config = tiberius::Config::new();
    config.host(db_host);
    config.port(db_port);
    config.database(db_name);
    config.authentication(tiberius::AuthMethod::sql_server(db_user, get_db_password()));
    config.encryption(tiberius::EncryptionLevel::NotSupported);
    config
}
