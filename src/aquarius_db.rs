use anyhow::{Ok, Result};
use async_std::net::TcpStream;
use log::debug;
use serde::Serialize;
use std::time::Duration;
use tiberius::{time::chrono::NaiveDateTime, Client, Row};

const REGATTAS_QUERY: &str = "SELECT * FROM Event e";

const REGATTA_QUERY: &str = "SELECT * FROM Event e WHERE e.Event_ID = @P1";

const HEATS_QUERY: &str =
    "SELECT c.*, o.Offer_RaceNumber, o.Offer_ShortLabel, o.Offer_LongLabel, o.Offer_Comment, o.Offer_Distance, ag.* \
    FROM Comp AS c \
    JOIN Offer AS o ON o.Offer_ID = c.Comp_Race_ID_FK \
    JOIN AgeClass AS ag ON o.Offer_AgeClass_ID_FK = ag.AgeClass_ID \
    WHERE c.Comp_Event_ID_FK = @P1 \
    ORDER BY Comp_DateTime ASC";

const HEAT_REGISTRATION_QUERY: &str =
    "SELECT	ce.*, e.Entry_Bib, e.Entry_BoatNumber, l.Label_Short, l.Label_Long, r.Result_Rank, r.Result_DisplayValue, r.Result_Delta \
    FROM CompEntries AS ce
    JOIN Comp AS c ON ce.CE_Comp_ID_FK = c.Comp_ID
    JOIN Entry AS e ON ce.CE_Entry_ID_FK = e.Entry_ID
    JOIN EntryLabel AS el ON el.EL_Entry_ID_FK = e.Entry_ID
    JOIN Label AS l ON el.EL_Label_ID_FK = l.Label_ID
    JOIN Result AS r ON r.Result_CE_ID_FK = ce.CE_ID
    WHERE ce.CE_Comp_ID_FK = @P1 AND r.Result_SplitNr = 64 \
      AND el.EL_RoundFrom <= c.Comp_Round AND c.Comp_Round <= el.EL_RoundTo";

pub async fn get_regattas(client: &mut Client<TcpStream>) -> Result<Vec<Regatta>> {
    debug!("Query {HEATS_QUERY}");

    let rows = client
        .query(REGATTAS_QUERY, &[])
        .await?
        .into_first_result()
        .await?;

    let mut regattas: Vec<Regatta> = Vec::new();

    for row in &rows {
        let regatta = create_regatta(row);
        debug!("{:?}", regatta);
        regattas.push(regatta);
    }
    Ok(regattas)
}

pub async fn get_regatta(client: &mut Client<TcpStream>, regatta_id: i32) -> Result<Regatta> {
    debug!("Query {REGATTA_QUERY}");

    let row = client
        .query(REGATTA_QUERY, &[&regatta_id])
        .await?
        .into_row()
        .await?
        .unwrap();

    let regatta = create_regatta(&row);
    Ok(regatta)
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

    let mut heat_registrations: Vec<HeatRegistration> = Vec::with_capacity(rows.len());

    for row in &rows {
        let heat_registration = create_heat_registration(row);
        debug!("{:?}", heat_registration);
        heat_registrations.push(heat_registration);
    }
    Ok(heat_registrations)
}

pub async fn get_heats(client: &mut Client<TcpStream>, regatta_id: i32) -> Result<Vec<Heat>> {
    debug!("Query {HEATS_QUERY}");

    let rows = client
        .query(HEATS_QUERY, &[&regatta_id])
        .await?
        .into_first_result()
        .await?;

    let mut heats: Vec<Heat> = Vec::new();

    for row in &rows {
        let heat = create_heat(row);
        debug!("{:?}", heat);
        heats.push(heat);
    }
    Ok(heats)
}

fn create_regatta(row: &Row) -> Regatta {
    let start_date: NaiveDateTime = Column::get(row, "Event_StartDate");
    let end_date: NaiveDateTime = Column::get(row, "Event_EndDate");

    Regatta {
        id: Column::get(row, "Event_ID"),
        title: Column::get(row, "Event_Title"),
        sub_title: Column::get(row, "Event_SubTitle"),
        venue: Column::get(row, "Event_Venue"),
        start_date: start_date.date().to_string(),
        end_date: end_date.date().to_string(),
    }
}

fn create_heat(row: &Row) -> Heat {
    let date_time: NaiveDateTime = Column::get(row, "Comp_DateTime");

    Heat {
        id: Column::get(row, "Comp_ID"),
        race_number: Column::get(row, "Offer_RaceNumber"),
        race_short_label: Column::get(row, "Offer_ShortLabel"),
        race_comment: Column::get(row, "Offer_Comment"),
        number: Column::get(row, "Comp_Number"),
        round_code: Column::get(row, "Comp_RoundCode"),
        label: Column::get(row, "Comp_Label"),
        group_value: Column::get(row, "Comp_GroupValue"),
        state: Column::get(row, "Comp_State"),
        cancelled: Column::get(row, "Comp_Cancelled"),
        date: date_time.date().to_string(),
        time: date_time.time().to_string(),
        ac_num_sub_classes: Column::get(row, "AgeClass_NumSubClasses"),
        distance: Column::get(row, "Offer_Distance"),
    }
}

fn create_heat_registration(row: &Row) -> HeatRegistration {
    let delta: i32 = Column::get(row, "Result_Delta");
    let duration = Duration::from_millis(delta as u64);

    let seconds = duration.as_secs();
    let millis = duration.subsec_millis() / 10;

    HeatRegistration {
        id: Column::get(row, "CE_ID"),
        lane: Column::get(row, "CE_Lane"),
        bib: Column::get(row, "Entry_Bib"),
        rank: Column::get(row, "Result_Rank"),
        short_label: Column::get(row, "Label_Short"),
        long_label: Column::get(row, "Label_Long"),
        result: Column::get(row, "Result_DisplayValue"),
        boat_number: Column::get(row, "Entry_BoatNumber"),
        delta: format!("{}.{}", seconds, millis),
    }
}

#[derive(Debug, Serialize)]
pub struct Regatta {
    id: i32,
    title: String,
    sub_title: String,
    venue: String,
    start_date: String,
    end_date: String,
}

#[derive(Debug, Serialize)]
pub struct Heat {
    pub id: i32,
    number: i16,
    race_short_label: String,
    race_comment: String,
    race_number: String,
    round_code: String,
    label: String,
    group_value: i16,
    state: u8,
    cancelled: bool,
    date: String,
    time: String,
    ac_num_sub_classes: u8,
    distance: i16,
}

#[derive(Debug, Serialize)]
pub struct HeatRegistration {
    id: i32,
    lane: i16,
    bib: i16,
    rank: u8,
    short_label: String,
    long_label: String,
    boat_number: i16,
    result: String,
    delta: String,
}

// see: https://github.com/prisma/tiberius/issues/101#issuecomment-978144867
trait Column {
    fn get(row: &Row, col_name: &str) -> Self;
}

impl Column for bool {
    fn get(row: &Row, col_name: &str) -> bool {
        row.try_get::<bool, _>(col_name).unwrap().unwrap()
    }
}

impl Column for u8 {
    fn get(row: &Row, col_name: &str) -> u8 {
        row.try_get::<u8, _>(col_name).unwrap().unwrap()
    }
}

impl Column for i16 {
    fn get(row: &Row, col_name: &str) -> i16 {
        row.try_get::<i16, _>(col_name).unwrap().unwrap_or_default()
    }
}

impl Column for i32 {
    fn get(row: &Row, col_name: &str) -> i32 {
        row.try_get::<i32, _>(col_name).unwrap().unwrap()
    }
}

impl Column for NaiveDateTime {
    fn get(row: &Row, col_name: &str) -> NaiveDateTime {
        row.try_get::<NaiveDateTime, _>(col_name).unwrap().unwrap()
    }
}

impl Column for String {
    fn get(row: &Row, col_name: &str) -> String {
        row.try_get::<&str, _>(col_name)
            .unwrap()
            .unwrap_or("")
            .to_string()
    }
}
