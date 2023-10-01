use crate::db::{
    aquarius::AquariusClient,
    model::{utils, ToEntity},
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Club {
    pub id: i32,

    /// The short name of the club.
    short_name: String,

    /// The long name of the club.
    #[serde(skip_serializing_if = "Option::is_none")]
    long_name: Option<String>,

    /// A very short abbreviation of the club.
    #[serde(skip_serializing_if = "Option::is_none")]
    abbreviation: Option<String>,

    /// The location of the club.
    city: String,

    /// The number of times this club has been a participant.
    #[serde(skip_serializing_if = "Option::is_none")]
    participations_count: Option<i32>,
}

impl Club {
    pub async fn query_participating(regatta_id: i32, client: &mut AquariusClient<'_>) -> Vec<Club> {
        let mut query = Query::new(
            "SELECT DISTINCT c.*,
              (SELECT COUNT(*) FROM ( 
                SELECT DISTINCT Entry_ID
                FROM Club
                JOIN Athlet     ON Athlet_Club_ID_FK  = Club_ID
                JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
                JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
                JOIN Event      ON Entry_Event_ID_FK  = Event_ID
                WHERE Event_ID = e.Event_ID AND c.Club_ID = Club_ID AND Entry_CancelValue = 0 AND Crew_RoundTo = 64
              ) AS Participations_Count) AS Participations_Count
            FROM Club AS c
            JOIN Athlet ON Athlet_Club_ID_FK      = c.Club_ID
            JOIN Crew   ON Crew_Athlete_ID_FK     = Athlet_ID
            JOIN Entry  ON Crew_Entry_ID_FK       = Entry_ID
            JOIN Event AS e ON Entry_Event_ID_FK  = Event_ID
            WHERE Event_ID = @P1 AND Crew_RoundTo = 64
            ORDER BY Club_City ASC",
        );
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let clubs = utils::get_rows(stream).await;
        clubs.into_iter().map(|row| row.to_entity()).collect()
    }

    pub async fn query_single(club_id: i32, client: &mut AquariusClient<'_>) -> Club {
        let mut query = Query::new(
            "SELECT DISTINCT Club.*
            FROM Club
            WHERE Club_ID = @P1
            ORDER BY Club_City ASC",
        );
        query.bind(club_id);
        let stream = query.query(client).await.unwrap();
        utils::get_row(stream).await.to_entity()
    }
}

impl ToEntity<Club> for Row {
    fn to_entity(&self) -> Club {
        Club {
            id: self.get_column("Club_ID"),
            short_name: self.get_column("Club_Abbr"),
            long_name: self.try_get_column("Club_Name"),
            abbreviation: self.try_get_column("Club_UltraAbbr"),
            city: self.get_column("Club_City"),
            participations_count: self.try_get_column("Participations_Count"),
        }
    }
}
