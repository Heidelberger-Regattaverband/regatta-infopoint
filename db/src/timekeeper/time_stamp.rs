use crate::tiberius::TiberiusClient;
use crate::{
    aquarius::model::utils,
    error::DbError,
    tiberius::{RowColumn, TryRowColumn},
};
use chrono::{DateTime, Local, Utc};
use serde::Serialize;
use strum_macros::Display;
use tiberius::{Query, Row};
use utoipa::ToSchema;

/// A time stamp of an event, such as a start or finish time stamp in a race.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TimeStamp {
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

impl TimeStamp {
    /// Create a new time stamp with the current time in UTC.
    ///
    /// # Arguments
    /// * `split` - The type of the time stamp.
    /// # Returns
    /// A new time stamp with the current time.
    pub(crate) fn now(split: Split) -> TimeStamp {
        TimeStamp {
            time: Local::now().to_utc(),
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

    pub fn set_heat_nr(&mut self, heat_nr: i16) {
        self.heat_nr = Some(heat_nr);
        self.persisted = false;
    }

    pub fn heat_nr(&self) -> Option<i16> {
        self.heat_nr
    }

    pub fn set_bib(&mut self, bib: u8) {
        self.bib = Some(bib);
        self.persisted = false;
    }

    pub fn bib_opt(&self) -> Option<u8> {
        self.bib
    }

    pub(crate) async fn query_all_for_regatta(
        regatta_id: i32,
        client: &mut TiberiusClient,
    ) -> Result<Vec<TimeStamp>, DbError> {
        let mut query = Query::new(
            "SELECT timestamp, event_id, split_nr, heat_nr, bib FROM HRV_Timestamp WHERE event_id = @P1 ORDER BY timestamp ASC"
                .to_string(),
        );
        query.bind(regatta_id);

        let stream = query.query(client).await?;
        let time_stamps = utils::get_rows(stream).await?;
        Ok(time_stamps.into_iter().map(|row| TimeStamp::from(&row)).collect())
    }

    pub(crate) async fn delete(&self, client: &mut TiberiusClient) -> Result<(), DbError> {
        let mut query = Query::new("DELETE FROM HRV_Timestamp WHERE timestamp = @P1".to_string());
        query.bind(self.time);

        query.execute(client).await?;
        Ok(())
    }

    pub(crate) async fn persist(&mut self, regatta_id: i32, client: &mut TiberiusClient) -> Result<(), DbError> {
        if !self.persisted {
            let mut query = Query::new(
            "INSERT INTO HRV_Timestamp (timestamp, event_id, split_nr, heat_nr, bib) VALUES (@P1, @P2, @P3, @P4, @P5)"
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
            let mut query =
                Query::new("UPDATE HRV_Timestamp SET heat_nr = @P2, bib = @P3 WHERE timestamp = @P1".to_string());
            query.bind(self.time);
            query.bind(self.heat_nr);
            query.bind(self.bib);
            query.execute(client).await?;
            self.persisted = true;
        }
        Ok(())
    }
}

impl From<&Row> for TimeStamp {
    fn from(row: &Row) -> Self {
        let split_nr: u8 = row.get_column("split_nr");
        TimeStamp {
            time: row.get_column("timestamp"),
            split: Split::from(split_nr),
            heat_nr: row.try_get_column("heat_nr"),
            bib: row.try_get_column("bib"),
            persisted: true,
        }
    }
}

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
            0 => Split::Start,
            64 => Split::Finish,
            _ => Split::Start,
        }
    }
}

impl From<&Split> for u8 {
    fn from(split: &Split) -> Self {
        match split {
            Split::Start => 0,
            Split::Finish => 64,
        }
    }
}

impl From<&Split> for String {
    fn from(split: &Split) -> Self {
        split.to_string()
    }
}
