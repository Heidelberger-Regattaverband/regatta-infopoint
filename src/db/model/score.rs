use crate::db::{
    aquarius::AquariusClient,
    model::{utils, Club, ToEntity},
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    rank: Option<i16>,
    points: f64,
    club: Club,
}

impl ToEntity<Score> for Row {
    fn to_entity(&self) -> Score {
        Score {
            rank: self.try_get_column("rank"),
            points: self.get_column("points"),
            club: self.to_entity(),
        }
    }
}

impl Score {
    pub async fn query_all<'a>(regatta_id: i32, client: &mut AquariusClient<'_>) -> Vec<Self> {
        let mut query = Query::new(
            "SELECT s.rank, s.points, c.Club_ID, c.Club_Name, c.Club_Abbr, c.Club_City
            FROM HRV_Score AS s
            JOIN Club      AS c ON s.club_id = c.Club_ID
            WHERE s.event_id = @P1
            ORDER BY s.rank ASC",
        );
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let scores = utils::get_rows(stream).await;
        scores.into_iter().map(|row| row.to_entity()).collect()
    }

    pub async fn calculate<'a>(regatta_id: i32, client: &mut AquariusClient<'_>) -> Vec<Self> {
        let mut query = Query::new(
            "SELECT Club_ID, SUM(Points_Crew) as points, Club_Name, Club_City, Club_Abbr, Club_UltraAbbr FROM
              (SELECT Club_ID, Club_Name, Club_City, Club_Abbr, Club_UltraAbbr,
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
            GROUP BY Club_ID, Club_City, Club_Name, Club_Abbr, Club_UltraAbbr
            ORDER BY points DESC",
        );
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let scores = utils::get_rows(stream).await;
        let mut index = 0;
        scores
            .into_iter()
            .map(|row| {
                index += 1;
                let mut score: Self = row.to_entity();
                score.rank = Some(index);
                score
            })
            .collect()
    }
}
