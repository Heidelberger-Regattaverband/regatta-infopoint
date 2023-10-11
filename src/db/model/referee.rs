use crate::db::{
    model::{utils, ToEntity, TryToEntity},
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use serde::Serialize;
use tiberius::{Query, Row};

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
    pub async fn query(heat_id: i32, pool: &TiberiusPool) -> Vec<Referee> {
        let mut query = Query::new(
            "SELECT r.* FROM Referee r
            JOIN CompReferee cr ON cr.CompReferee_Referee_ID_FK = r.Referee_ID
            WHERE cr.CompReferee_Comp_ID_FK = @P1",
        );
        query.bind(heat_id);

        let mut client = pool.get().await;
        let heats = utils::get_rows(query.query(&mut client).await.unwrap()).await;
        heats.into_iter().map(|row| row.to_entity()).collect()
    }
}

impl TryToEntity<Referee> for Row {
    fn try_to_entity(&self) -> Option<Referee> {
        <Row as TryRowColumn<i32>>::try_get_column(self, "Referee_ID").map(|_id| self.to_entity())
    }
}

impl ToEntity<Referee> for Row {
    fn to_entity(&self) -> Referee {
        let id = self.get_column("Referee_ID");
        let last_name: String = self.get_column("Referee_LastName");
        let first_name: String = self.get_column("Referee_FirstName");
        let city: String = self.get_column("Referee_City");
        Referee {
            id,
            last_name,
            first_name,
            city,
        }
    }
}
