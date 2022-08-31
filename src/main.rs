use anyhow::Result;
use async_std::net::TcpStream;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel};

#[async_std::main]
async fn main() -> Result<()> {
    let config = create_config();

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let mut client = Client::connect(config, tcp).await?;

    let stream = client.query("SELECT @P1", &[&1i32]).await?;
    let row = stream.into_row().await?.unwrap();

    println!("{:?}", row);
    assert_eq!(Some(1), row.get(0));

    Ok(())
}

fn create_config() -> Config {
    let mut config = Config::new();
    config.host("8e835d.online-server.cloud");
    config.port(1433);
    config.authentication(AuthMethod::sql_server("SA", "Regatta4HD"));
    config.database("Regatta_2022");
    config.encryption(EncryptionLevel::NotSupported);
    config
}
