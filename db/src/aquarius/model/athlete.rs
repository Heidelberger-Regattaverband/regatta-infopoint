use super::TryToEntity;
use super::club::Club;
use super::get_row;
use super::get_rows;
use crate::tiberius::TiberiusClient;
use crate::{
    error::DbError,
    tiberius::{RowColumn, TryRowColumn},
};
use ::serde::Serialize;
use ::tiberius::{Query, Row, time::chrono::NaiveDateTime};
use ::utoipa::ToSchema;

const ID: &str = "Athlet_ID";
const FIRST_NAME: &str = "Athlet_FirstName";
const LAST_NAME: &str = "Athlet_LastName";
const GENDER: &str = "Athlet_Gender";
const DOB: &str = "Athlet_DOB";

/// An athlete is a person who participates in a regatta.
#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Athlete {
    /// The internal ID of the athlete.
    id: i32,

    /// First name of the athlete.
    first_name: String,

    /// Last name of the athlete.
    last_name: String,

    /// The athlete's gender.
    gender: String,

    /// Year of birth.
    year: String,

    /// The athlete's club.
    club: Club,

    /// The number of entries the athlete has.
    #[serde(skip_serializing_if = "Option::is_none")]
    entries_count: Option<i32>,
}

impl Athlete {
    pub async fn query_participating_athletes(
        regatta_id: i32,
        client: &mut TiberiusClient,
    ) -> Result<Vec<Athlete>, DbError> {
        let round = 64;
        let mut query = Query::new(format!(
            "SELECT DISTINCT {0}, {1},
                (SELECT COUNT(*) FROM (
                    SELECT {ID} FROM Athlet
                    JOIN Crew  ON Crew_Athlete_ID_FK = {ID}
                    JOIN Entry ON Crew_Entry_ID_FK   = Entry_ID
                    WHERE {ID} = a.{ID} AND Crew_RoundTo = @P2
                ) AS Athlet_Entries_Count ) AS Athlet_Entries_Count
                FROM Athlet a
                JOIN Club  cl ON a.Athlet_Club_ID_FK = cl.Club_ID
                JOIN Crew  cr ON a.{ID}              = cr.Crew_Athlete_ID_FK
                JOIN Entry  e ON cr.Crew_Entry_ID_FK = e.Entry_ID
                WHERE e.Entry_Event_ID_FK = @P1 AND e.Entry_CancelValue = 0 AND cr.Crew_RoundTo = @P2",
            Athlete::select_columns("a"),
            Club::select_min_columns("cl")
        ));
        query.bind(regatta_id);
        query.bind(round);

        let stream = query.query(client).await?;
        let athletes = get_rows(stream).await?;
        Ok(athletes.into_iter().map(|row| Athlete::from(&row)).collect())
    }

    pub async fn query_athlete(
        regatta_id: i32,
        athlete_id: i32,
        client: &mut TiberiusClient,
    ) -> Result<Athlete, DbError> {
        let round = 64;
        let mut query = Query::new(format!(
            "SELECT {0}, {1},
                (SELECT COUNT(*) FROM (
                    SELECT {ID} FROM Athlet
                    JOIN Crew  ON Crew_Athlete_ID_FK = {ID}
                    JOIN Entry ON Crew_Entry_ID_FK   = Entry_ID
                    WHERE e.Entry_Event_ID_FK = @P1 AND {ID} = a.{ID} AND Crew_RoundTo = @P3
                ) AS Athlet_Entries_Count ) AS Athlet_Entries_Count
                FROM Athlet  a
                JOIN Club   cl ON a.Athlet_Club_ID_FK = cl.Club_ID
                JOIN Crew   cr ON a.{ID}              = cr.Crew_Athlete_ID_FK
                JOIN Entry   e ON cr.Crew_Entry_ID_FK = e.Entry_ID
                WHERE e.Entry_Event_ID_FK = @P1 AND a.{ID} = @P2 AND cr.Crew_RoundTo = @P3",
            Athlete::select_columns("a"),
            Club::select_all_columns("cl")
        ));
        query.bind(regatta_id);
        query.bind(athlete_id);
        query.bind(round);

        let stream = query.query(client).await?;
        let row = get_row(stream).await?;
        Ok(Athlete::from(&row))
    }

    pub(crate) fn select_columns(alias: &str) -> String {
        format!("{alias}.{ID}, {alias}.{FIRST_NAME}, {alias}.{LAST_NAME}, {alias}.{GENDER}, {alias}.{DOB}")
    }
}

impl From<&Row> for Athlete {
    fn from(row: &Row) -> Self {
        Athlete {
            id: row.get_column(ID),
            first_name: row.get_column(FIRST_NAME),
            last_name: row.get_column(LAST_NAME),
            gender: row.get_column(GENDER),
            year: <Row as RowColumn<NaiveDateTime>>::get_column(row, DOB)
                .date()
                .format("%Y")
                .to_string(),
            club: Club::from(row),
            entries_count: row.try_get_column("Athlet_Entries_Count"),
        }
    }
}

impl TryToEntity<Athlete> for Row {
    fn try_to_entity(&self) -> Option<Athlete> {
        <Row as TryRowColumn<i32>>::try_get_column(self, ID).map(|_id| Athlete::from(self))
    }
}
