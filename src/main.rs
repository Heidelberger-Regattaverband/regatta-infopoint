use anyhow::Result;
use async_std::net::TcpStream;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel, Row};

const HEATS_QUERY: &str = "SELECT c.Comp_ID, c.Comp_Event_ID_FK, c.Comp_Number, c.Comp_RoundCode, c.Comp_Label, c.Comp_State, c.Comp_Cancelled, o.Offer_RaceNumber, o.Offer_ShortLabel, o.Offer_LongLabel \
    FROM Comp AS c \
    INNER JOIN Offer AS o ON o.Offer_ID = c.Comp_Race_ID_FK \
    WHERE c.Comp_Event_ID_FK = 12";

#[async_std::main]
async fn main() -> Result<()> {
    let config = create_config();

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let mut client = Client::connect(config, tcp).await?;

    println!("Query {HEATS_QUERY}");

    let rows = client
        .query(HEATS_QUERY, &[])
        .await?
        .into_first_result()
        .await?;

    println!("Row count: {}", rows.len());

    let mut heats: Vec<Heat> = Vec::new();

    for row in rows {
        let heat = Heat {
            id: row.try_get("Comp_ID")?.unwrap_or_else(|| 0),
            race_number: row
                .try_get("Offer_RaceNumber")?
                .unwrap_or_else(|| "")
                .to_string(),
            race_short_label: row
                .try_get("Offer_ShortLabel")?
                .unwrap_or_else(|| "")
                .to_string(),
            race_long_label: row
                .try_get("Offer_LongLabel")?
                .unwrap_or_else(|| "")
                .to_string(),
            number: row.try_get("Comp_Number")?.unwrap_or_else(|| 0),
            round_code: row
                .try_get("Comp_RoundCode")?
                .unwrap_or_else(|| "")
                .to_string(),
            division_number: row.try_get("Comp_Label")?.unwrap_or_else(|| "").to_string(),
            state: row.try_get("Comp_State")?.unwrap_or_else(|| 0),
            cancelled: row.try_get("Comp_Cancelled")?.unwrap_or_else(|| false),
        };
        println!(
            "Heat: id={}, race_number={}, number={}, round_code={}, division_number={}, race_short_label={}, state={}, cancelled={}, race_long_label={}", 
            heat.id,
            heat.race_number,
            heat.number,
            heat.round_code,
            heat.division_number,
            heat.race_short_label,
            heat.state,
            heat.cancelled,
            heat.race_long_label
        );
        heats.push(heat);
    }
    // println!("{:?}", row);
    // assert_eq!(Some(1), row.get(0));

    Ok(())
}

fn create_config() -> Config {
    let mut config = Config::new();
    config.host("8e835d.online-server.cloud");
    config.port(1433);
    config.authentication(AuthMethod::sql_server("SA", "Regatta4HD"));
    config.database("Regatta_2022_Test");
    config.encryption(EncryptionLevel::NotSupported);
    config
}

struct Heat {
    id: i32,
    race_number: String,
    race_short_label: String,
    number: i16,
    round_code: String,
    division_number: String,
    state: u8,
    cancelled: bool,
    race_long_label: String,
}
