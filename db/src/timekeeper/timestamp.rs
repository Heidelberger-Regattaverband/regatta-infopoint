use crate::aquarius::model::get_rows;
use crate::tiberius::TiberiusClient;
use crate::{
    error::DbError,
    tiberius::{RowColumn, TryRowColumn},
};
use ::chrono::{DateTime, Utc};
use ::serde::Serialize;
use ::strum_macros::Display;
use ::tiberius::{Query, Row};
use ::utoipa::ToSchema;

const TIMESTAMP: &str = "timestamp";
const EVENT_ID: &str = "eventId";
const SPLIT_NR: &str = "splitNr";
const HEAT_NR: &str = "heatNr";
const BIB: &str = "bib";

/// A time stamp of an event, such as a start or finish time stamp in a race.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Timestamp {
    /// The time of the event.
    pub time: DateTime<Utc>,

    /// The split of the time stamp. Either start or finish.
    split: Split,

    /// The optional heat number.
    heat_nr: Option<i16>,

    /// The optional bib number.
    bib: Option<u8>,

    /// Whether the time stamp is persisted in DB or not.
    persisted: bool,
}

impl Timestamp {
    pub(crate) fn from_time(time: DateTime<Utc>, split: Split) -> Timestamp {
        Timestamp {
            time,
            split,
            heat_nr: None,
            bib: None,
            persisted: false,
        }
    }

    pub fn split(&self) -> &Split {
        &self.split
    }

    pub fn is_persisted(&self) -> bool {
        self.persisted
    }

    pub(crate) fn set_heat_nr(&mut self, heat_nr: i16) {
        self.heat_nr = Some(heat_nr);
        self.persisted = false;
    }

    pub fn heat_nr(&self) -> Option<i16> {
        self.heat_nr
    }

    pub(crate) fn set_bib(&mut self, bib: u8) {
        self.bib = Some(bib);
        self.persisted = false;
    }

    pub fn bib(&self) -> Option<u8> {
        self.bib
    }

    pub(crate) async fn query_all_for_regatta(
        regatta_id: i32,
        offset: Option<i32>,
        top: Option<i32>,
        client: &mut TiberiusClient,
    ) -> Result<Vec<Timestamp>, DbError> {
        let mut query = Query::new(format!(
            "SELECT {TIMESTAMP}, {EVENT_ID}, {SPLIT_NR}, {HEAT_NR}, {BIB} FROM HRV_Timestamp \
            WHERE {EVENT_ID} = @P1 ORDER BY {TIMESTAMP} DESC \
            OFFSET @P2 ROWS FETCH NEXT @P3 ROWS ONLY"
        ));
        query.bind(regatta_id);
        query.bind(offset.unwrap_or(0)); // OFFSET
        query.bind(top.unwrap_or(30)); // FETCH NEXT

        let stream = query.query(client).await?;
        let time_stamps = get_rows(stream).await?;
        Ok(time_stamps.into_iter().map(|row| Timestamp::from(&row)).collect())
    }

    pub(crate) async fn delete(&self, client: &mut TiberiusClient) -> Result<(), DbError> {
        let mut query = Query::new(format!("DELETE FROM HRV_Timestamp WHERE {TIMESTAMP} = @P1"));
        query.bind(self.time);

        query.execute(client).await?;
        Ok(())
    }

    pub(crate) async fn persist(&mut self, regatta_id: i32, client: &mut TiberiusClient) -> Result<(), DbError> {
        if !self.persisted {
            let mut query = Query::new(
                format!("INSERT INTO HRV_Timestamp ({TIMESTAMP}, {EVENT_ID}, {SPLIT_NR}, {HEAT_NR}, {BIB}) VALUES (@P1, @P2, @P3, @P4, @P5)")
                    .to_string(),
            );
            query.bind(self.time);
            query.bind(regatta_id);
            query.bind(u8::from(&self.split));
            query.bind(self.heat_nr);
            query.bind(self.bib);

            query.execute(client).await?;
            self.persisted = true;
        }
        Ok(())
    }

    pub(crate) async fn update(&mut self, client: &mut TiberiusClient) -> Result<(), DbError> {
        if !self.persisted {
            let mut query = Query::new(format!(
                "UPDATE HRV_Timestamp SET {HEAT_NR} = @P2, {BIB} = @P3 WHERE {TIMESTAMP} = @P1"
            ));
            query.bind(self.time);
            query.bind(self.heat_nr);
            query.bind(self.bib);
            query.execute(client).await?;
            self.persisted = true;
        }
        Ok(())
    }
}

impl From<&Row> for Timestamp {
    fn from(row: &Row) -> Self {
        let split_nr: u8 = row.get_column(SPLIT_NR);
        Timestamp {
            time: row.get_column(TIMESTAMP),
            split: Split::from(split_nr),
            heat_nr: row.try_get_column(HEAT_NR),
            bib: row.try_get_column(BIB),
            persisted: true,
        }
    }
}

const SPLIT_START: u8 = 0;
const SPLIT_FINISH: u8 = 64;

/// The type of a time stamp.
#[derive(Debug, Clone, Display, Serialize, ToSchema)]
pub enum Split {
    /// A start time stamp.
    #[strum(to_string = "Start")]
    Start,

    /// A finish time stamp.
    #[strum(to_string = "Ziel")]
    Finish,
}

impl From<u8> for Split {
    fn from(value: u8) -> Self {
        match value {
            SPLIT_START => Self::Start,
            SPLIT_FINISH => Self::Finish,
            _ => Self::Start,
        }
    }
}

impl From<&Split> for u8 {
    fn from(split: &Split) -> Self {
        match split {
            Split::Start => SPLIT_START,
            Split::Finish => SPLIT_FINISH,
        }
    }
}

impl From<&Split> for String {
    fn from(split: &Split) -> Self {
        split.to_string()
    }
}
