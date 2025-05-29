use crate::db::{
    model::{TryToEntity, utils},
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use serde::Serialize;
use tiberius::{Query, Row, error::Error as DbError};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Referee {
    id: i32,

    /// First name of the referee.
    first_name: String,

    /// Last name of the referee.
    last_name: String,

    /// City of the referee.
    city: String,
}

impl Referee {
    /// Query all referees for a specific heat.
    /// # Arguments
    /// `heat_id`: The unique identifier of the heat.
    /// `pool`: The database connection pool.
    /// # Returns
    /// A list of referees.
    pub async fn query_referees_for_heat(heat_id: i32, pool: &TiberiusPool) -> Result<Vec<Self>, DbError> {
        let mut query = Query::new(
            "SELECT r.* FROM Referee r
            JOIN CompReferee cr ON cr.CompReferee_Referee_ID_FK = r.Referee_ID
            WHERE cr.CompReferee_Comp_ID_FK = @P1",
        );
        query.bind(heat_id);

        let mut client = pool.get().await;
        let heats = utils::get_rows(query.query(&mut client).await?).await?;
        Ok(heats.into_iter().map(|row| Referee::from(&row)).collect())
    }
}

impl TryToEntity<Referee> for Row {
    fn try_to_entity(&self) -> Option<Referee> {
        <Row as TryRowColumn<i32>>::try_get_column(self, "Referee_ID").map(|_id| Referee::from(self))
    }
}

/// Convert a database Row into a Referee.
impl From<&Row> for Referee {
    /// Create a new Referee from a database Row.
    /// # Arguments
    /// `row`: The database Row.
    /// # Returns
    /// A new Referee.
    fn from(value: &Row) -> Self {
        Referee {
            id: value.get_column("Referee_ID"),
            last_name: value.get_column("Referee_LastName"),
            first_name: value.get_column("Referee_FirstName"),
            city: value.get_column("Referee_City"),
        }
    }
}
