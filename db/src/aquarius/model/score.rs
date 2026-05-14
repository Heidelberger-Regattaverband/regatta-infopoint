use super::Club;
use super::athlete::ID as ATHLETE_ID;
use super::boat_class::ID as BOAT_CLASS_ID;
use super::boat_class::NUM_ROWERS;
use super::club::ID as CLUB_ID;
use super::entry::ID as ENTRY_ID;
use super::get_rows;
use super::heat::ID as HEAT_ID;
use super::race::ID as RACE_ID;
use crate::tiberius::TiberiusClient;
use crate::{
    error::DbError,
    tiberius::{RowColumn, TryRowColumn},
};
use ::serde::Serialize;
use ::tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    rank: Option<i16>,
    points: f64,
    club: Club,
}

impl From<&Row> for Score {
    fn from(value: &Row) -> Self {
        Score {
            rank: value.try_get_column("rank"),
            points: value.get_column("points"),
            club: Club::from(value),
        }
    }
}

impl Score {
    pub async fn calculate(regatta_id: i32, client: &mut TiberiusClient) -> Result<Vec<Self>, DbError> {
        let mut query = Query::new(format!(
            "SELECT {CLUB_ID}, SUM(Points_Crew) as points, Club_Name, Club_City, Club_Abbr, Club_UltraAbbr, Club_ExternID FROM
              (SELECT {CLUB_ID}, Club_Name, Club_City, Club_Abbr, Club_UltraAbbr, Club_ExternID,
                (SELECT CASE WHEN Offer_HRV_Seeded = 1 AND Comp_HeatNumber = 1
                  THEN
                    ((RaceMode_LaneCount + 1 - CAST(Result_Rank AS float) + {NUM_ROWERS}) / {NUM_ROWERS}) * 2
                  ELSE 
                    (RaceMode_LaneCount + 1 - CAST(Result_Rank AS float) + {NUM_ROWERS}) / {NUM_ROWERS}
                  END
                ) as Points_Crew
              FROM Result
              JOIN CompEntries ON           CE_ID = Result_CE_ID_FK
              JOIN Comp        ON       {HEAT_ID} = CE_Comp_ID_FK
              JOIN Entry       ON      {ENTRY_ID} = CE_Entry_ID_FK
              JOIN Crew        ON      {ENTRY_ID} = Crew_Entry_ID_FK
              JOIN Athlet      ON    {ATHLETE_ID} = Crew_Athlete_ID_FK
              JOIN Club        ON       {CLUB_ID} = Athlet_Club_ID_FK
              JOIN Offer       ON       {RACE_ID} = Comp_Race_ID_FK
              JOIN BoatClass   ON {BOAT_CLASS_ID} = Offer_BoatClass_ID_FK
              JOIN RaceMode    ON     RaceMode_ID = Offer_RaceMode_ID_FK
              WHERE Offer_Event_ID_FK = @P1 AND Crew_IsCox = 0 AND Result_SplitNr = 64 AND Crew_RoundTo = 64 AND Result_Rank > 0 AND Comp_Round = 64 AND Comp_State = 4
            ) as t
            GROUP BY {CLUB_ID}, Club_City, Club_Name, Club_Abbr, Club_UltraAbbr, Club_ExternID
            ORDER BY points DESC",
        ));
        query.bind(regatta_id);

        let scores = get_rows(query.query(client).await?).await?;
        Ok(scores
            .into_iter()
            .enumerate()
            .map(|(index, row)| {
                let mut score = Score::from(&row);
                score.rank = Some((index + 1) as i16);
                score
            })
            .collect())
    }
}
