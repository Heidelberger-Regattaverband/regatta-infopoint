use crate::{
    aquarius::model::{TryToEntity, utils},
    error::DbError,
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use serde::Serialize;
use tiberius::{Query, Row};
use utoipa::ToSchema;

const ID: &str = "Referee_ID";
const FIRST_NAME: &str = "Referee_FirstName";
const LAST_NAME: &str = "Referee_LastName";
const CITY: &str = "Referee_City";

#[derive(Debug, Serialize, Clone, ToSchema)]
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
    pub(crate) async fn query_referees_for_heat(heat_id: i32, pool: &TiberiusPool) -> Result<Vec<Self>, DbError> {
        let mut query = Query::new(format!(
            "SELECT {} FROM Referee r
            JOIN CompReferee cr ON cr.CompReferee_Referee_ID_FK = r.{ID}
            WHERE cr.CompReferee_Comp_ID_FK = @P1",
            Referee::select_columns("r")
        ));
        query.bind(heat_id);

        let mut client = pool.get().await?;
        let heats = utils::get_rows(query.query(&mut client).await?).await?;
        Ok(heats.into_iter().map(|row| Referee::from(&row)).collect())
    }

    fn select_columns(alias: &str) -> String {
        format!("{alias}.{ID}, {alias}.{FIRST_NAME}, {alias}.{LAST_NAME}, {alias}.{CITY}")
    }
}

impl TryToEntity<Referee> for Row {
    fn try_to_entity(&self) -> Option<Referee> {
        <Row as TryRowColumn<i32>>::try_get_column(self, ID).map(|_id| Referee::from(self))
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
            id: value.get_column(ID),
            last_name: value.get_column(LAST_NAME),
            first_name: value.get_column(FIRST_NAME),
            city: value.get_column(CITY),
        }
    }
}
