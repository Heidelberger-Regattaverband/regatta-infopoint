use crate::db::{
    model::{utils, Athlete, Club, ToEntity},
    tiberius::{RowColumn, TiberiusPool},
};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Crew {
    id: i32,

    /// Position in the boat
    pos: u8,

    /// Is cox in the boat
    cox: bool,

    /// the athlete
    athlete: Athlete,

    /// Is crew member from round.
    round_from: i16,

    /// Is crew member until round.
    round_to: i16,
}

impl Crew {
    pub fn select_columns(alias: &str) -> String {
        format!(
            " {0}.Crew_ID, {0}.Crew_Pos, {0}.Crew_IsCox, {0}.Crew_RoundFrom, {0}.Crew_RoundTo ",
            alias
        )
    }

    pub async fn query_all(registration_id: i32, round: i16, pool: &TiberiusPool) -> Vec<Crew> {
        let mut query = Query::new(
            "SELECT".to_string()
                + &Crew::select_columns("cr")
                + ", "
                + &Athlete::select_columns("a")
                + ", "
                + &Club::select_columns("cl")
                + "
            FROM Crew cr
            JOIN Athlet a ON cr.Crew_Athlete_ID_FK = a.Athlet_ID
            JOIN Club cl  ON a.Athlet_Club_ID_FK   = cl.Club_ID
            WHERE Crew_Entry_ID_FK = @P1 AND cr.Crew_RoundFrom <= @P2 AND @P2 <= cr.Crew_RoundTo
            ORDER BY Crew_pos ASC",
        );
        query.bind(registration_id);
        query.bind(round);

        let mut client = pool.get().await;
        let crew = utils::get_rows(query.query(&mut client).await.unwrap()).await;
        crew.into_iter().map(|row| row.to_entity()).collect()
    }
}

impl ToEntity<Crew> for Row {
    fn to_entity(&self) -> Crew {
        Crew {
            id: self.get_column("Crew_ID"),
            pos: self.get_column("Crew_Pos"),
            cox: self.get_column("Crew_IsCox"),
            athlete: self.to_entity(),
            round_from: self.get_column("Crew_RoundFrom"),
            round_to: self.get_column("Crew_RoundTo"),
        }
    }
}
