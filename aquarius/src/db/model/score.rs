use crate::{
    db::model::{Club, utils},
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use serde::Serialize;
use tiberius::{Query, Row, error::Error as DbError};

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
    pub async fn calculate(regatta_id: i32, pool: &TiberiusPool) -> Result<Vec<Self>, DbError> {
        let mut query = Query::new(
            "SELECT Club_ID, SUM(Points_Crew) as points, Club_Name, Club_City, Club_Abbr, Club_UltraAbbr, Club_ExternID FROM
              (SELECT Club_ID, Club_Name, Club_City, Club_Abbr, Club_UltraAbbr, Club_ExternID,
                (SELECT CASE WHEN Offer_HRV_Seeded = 1 AND Comp_HeatNumber = 1
                  THEN
                    ((RaceMode_LaneCount + 1 - CAST(Result_Rank AS float) + BoatClass_NumRowers) / BoatClass_NumRowers) * 2
                  ELSE 
                    (RaceMode_LaneCount + 1 - CAST(Result_Rank AS float) + BoatClass_NumRowers) / BoatClass_NumRowers
                  END
                ) as Points_Crew
              FROM Result
              JOIN CompEntries ON CE_ID        = Result_CE_ID_FK
              JOIN Comp        ON Comp_ID      = CE_Comp_ID_FK
              JOIN Entry       ON Entry_ID     = CE_Entry_ID_FK
              JOIN Crew        ON Entry_ID     = Crew_Entry_ID_FK
              JOIN Athlet      ON Athlet_ID    = Crew_Athlete_ID_FK
              JOIN Club        ON Club_ID      = Athlet_Club_ID_FK
              JOIN Offer       ON Offer_ID     = Comp_Race_ID_FK
              JOIN BoatClass   ON BoatClass_ID = Offer_BoatClass_ID_FK
              JOIN RaceMode    ON RaceMode_ID  = Offer_RaceMode_ID_FK
              WHERE Offer_Event_ID_FK = @P1 AND Crew_IsCox = 0 AND Result_SplitNr = 64 AND Crew_RoundTo = 64 AND Result_Rank > 0 AND Comp_Round = 64 AND Comp_State = 4
            ) as t
            GROUP BY Club_ID, Club_City, Club_Name, Club_Abbr, Club_UltraAbbr, Club_ExternID
            ORDER BY points DESC",
        );
        query.bind(regatta_id);

        let mut client = pool.get().await;
        let scores = utils::get_rows(query.query(&mut client).await?).await?;
        let mut index = 0;
        Ok(scores
            .into_iter()
            .map(|row| {
                index += 1;
                let mut score = Score::from(&row);
                score.rank = Some(index);
                score
            })
            .collect())
    }
}
