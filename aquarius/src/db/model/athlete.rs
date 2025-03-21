use crate::db::{
    model::{Club, TryToEntity, utils},
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use serde::Serialize;
use tiberius::{Query, Row, error::Error as DbError, time::chrono::NaiveDateTime};

/// An athlete is a person who participates in a regatta.
#[derive(Debug, Serialize, Clone)]
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
}

impl Athlete {
    pub async fn query_participating_athletes(regatta_id: i32, pool: &TiberiusPool) -> Result<Vec<Athlete>, DbError> {
        let mut query = Query::new(format!(
            "SELECT DISTINCT {0}, {1} FROM Athlet a
                JOIN Club  cl ON a.Athlet_Club_ID_FK = cl.Club_ID
                JOIN Crew   c ON a.Athlet_ID         = c.Crew_Athlete_ID_FK
                JOIN Entry  e ON c.Crew_Entry_ID_FK  = e.Entry_ID
                WHERE e.Entry_Event_ID_FK = @P1 AND e.Entry_CancelValue = 0",
            Athlete::select_columns("a"),
            Club::select_columns("cl")
        ));
        query.bind(regatta_id);

        let mut client = pool.get().await;
        let stream = query.query(&mut client).await?;
        let athletes = utils::get_rows(stream).await?;
        Ok(athletes.into_iter().map(|row| Athlete::from(&row)).collect())
    }

    pub fn select_columns(alias: &str) -> String {
        format!(
            " {0}.Athlet_ID, {0}.Athlet_FirstName, {0}.Athlet_LastName, {0}.Athlet_Gender, {0}.Athlet_DOB ",
            alias
        )
    }
}

impl From<&Row> for Athlete {
    fn from(row: &Row) -> Self {
        Athlete {
            id: row.get_column("Athlet_ID"),
            first_name: row.get_column("Athlet_FirstName"),
            last_name: row.get_column("Athlet_LastName"),
            gender: row.get_column("Athlet_Gender"),
            year: <Row as RowColumn<NaiveDateTime>>::get_column(row, "Athlet_DOB")
                .date()
                .format("%Y")
                .to_string(),
            club: Club::from(row),
        }
    }
}

impl TryToEntity<Athlete> for Row {
    fn try_to_entity(&self) -> Option<Athlete> {
        <Row as TryRowColumn<i32>>::try_get_column(self, "Athlet_ID").map(|_id| Athlete::from(self))
    }
}
