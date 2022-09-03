use anyhow::{Ok, Result};
use async_std::net::TcpStream;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel, Row};

const HEATS_QUERY: &str = "SELECT c.*, o.Offer_RaceNumber, o.Offer_ShortLabel, o.Offer_LongLabel \
    FROM Comp AS c \
    INNER JOIN Offer AS o ON o.Offer_ID = c.Comp_Race_ID_FK \
    WHERE c.Comp_Event_ID_FK = @P1";

const HEAT_REGISTRATION_QUERY: &str =
    "SELECT	ce.CE_Lane, e.Entry_Bib, e.Entry_BoatNumber, l.Label_Short, l.Label_Long, r.Result_Rank \
    FROM CompEntries AS ce
    JOIN Entry AS e ON ce.CE_Entry_ID_FK = e.Entry_ID
    JOIN EntryLabel AS el ON el.EL_Entry_ID_FK = e.Entry_ID
    JOIN Label AS l ON el.EL_Label_ID_FK = l.Label_ID
    JOIN Result AS r ON r.Result_CE_ID_FK = ce.CE_ID
    WHERE ce.CE_Comp_ID_FK = @P1 AND r.Result_SplitNr = 64";

const REGATTA_ID: i32 = 12;

pub async fn create_client(config: Config) -> Result<Client<TcpStream>> {
    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let client = Client::connect(config, tcp).await?;
    Ok(client)
}

pub fn create_config() -> Config {
    let mut config = Config::new();
    config.host("8e835d.online-server.cloud");
    config.port(1433);
    config.authentication(AuthMethod::sql_server("SA", "Regatta4HD"));
    config.database("Regatta_2022_Test");
    config.encryption(EncryptionLevel::NotSupported);
    config
}

pub async fn get_heat_registrations(
    client: &mut Client<TcpStream>,
    heat_id: i32,
) -> Result<Vec<HeatRegistration>> {
    let rows = client
        .query(HEAT_REGISTRATION_QUERY, &[&heat_id])
        .await?
        .into_first_result()
        .await?;

    let mut heat_registrations: Vec<HeatRegistration> = Vec::new();

    for row in &rows {
        let heat_registration = create_heat_registration(row).unwrap();
        dbg!(&heat_registration);
        heat_registrations.push(heat_registration);
    }
    Ok(heat_registrations)
}

pub async fn get_heats(client: &mut Client<TcpStream>) -> Result<Vec<Heat>> {
    println!("Query {HEATS_QUERY}");

    let rows = client
        .query(HEATS_QUERY, &[&REGATTA_ID])
        .await?
        .into_first_result()
        .await?;

    let mut heats: Vec<Heat> = Vec::new();

    for row in &rows {
        let heat = create_heat(row).unwrap();
        println!("{:?}", heat);
        heats.push(heat);
    }
    Ok(heats)
}

fn create_heat(row: &Row) -> Result<Heat> {
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
    Ok(heat)
}

fn create_heat_registration(row: &Row) -> Result<HeatRegistration> {
    let heat_registration = HeatRegistration {
        lane: row.try_get("CE_Lane")?.unwrap_or_else(|| 0),
        bib: row.try_get("Entry_Bib")?.unwrap_or_else(|| 0),
        rank: row.try_get("Result_Rank")?.unwrap_or_else(|| 0),
        short_label: row
            .try_get("Label_Short")?
            .unwrap_or_else(|| "")
            .to_string(),
        long_label: row.try_get("Label_Long")?.unwrap_or_else(|| "").to_string(),
    };
    Ok(heat_registration)
}

#[derive(Debug)]pub struct Heat {
    pub id: i32,
    race_number: String,
    race_short_label: String,
    number: i16,
    round_code: String,
    division_number: String,
    state: u8,
    cancelled: bool,
    race_long_label: String,
}

#[derive(Debug)]
pub struct HeatRegistration {
    lane: i16,
    bib: i16,
    rank: u8,
    short_label: String,
    long_label: String,
}
