use crate::{
    db::{
        model::{utils, ToEntity},
        tiberius::{RowColumn, TiberiusPool, TryRowColumn},
    },
    http::crawler::load_club_flags,
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

    /// An optional URL showing the flag of the club.
    flag_url: Option<String>,
}

impl Club {
    pub async fn query_participating(regatta_id: i32, pool: &TiberiusPool) -> Vec<Club> {
        let mut query = Query::new(
            "SELECT DISTINCT".to_string()
                + &Club::select_columns("c")
                + ", (SELECT COUNT(*) FROM ( 
                    SELECT DISTINCT Entry_ID
                    FROM Club
                    JOIN Athlet     ON Athlet_Club_ID_FK  = Club_ID
                    JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
                    JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
                    JOIN Event      ON Entry_Event_ID_FK  = Event_ID
                    WHERE Event_ID = e.Event_ID AND c.Club_ID = Club_ID AND Entry_CancelValue = 0 AND Crew_RoundTo = 64
                ) AS Participations_Count) AS Participations_Count
            FROM Club c
            JOIN Athlet ON Athlet_Club_ID_FK      = c.Club_ID
            JOIN Crew   ON Crew_Athlete_ID_FK     = Athlet_ID
            JOIN Entry  ON Crew_Entry_ID_FK       = Entry_ID
            JOIN Event AS e ON Entry_Event_ID_FK  = Event_ID
            WHERE Event_ID = @P1 AND Crew_RoundTo = 64
            ORDER BY Club_City ASC",
        );
        query.bind(regatta_id);

        let mut client = pool.get().await;
        let clubs = utils::get_rows(query.query(&mut client).await.unwrap()).await;
        clubs.into_iter().map(|row| row.to_entity()).collect()
    }

    pub async fn query_single(club_id: i32, pool: &TiberiusPool) -> Club {
        let mut query = Query::new(
            "SELECT".to_string()
                + &Club::select_columns("c")
                + "FROM Club c
            WHERE c.Club_ID = @P1
            ORDER BY c.Club_City ASC",
        );
        query.bind(club_id);

        let mut client = pool.get().await;
        utils::get_row(query.query(&mut client).await.unwrap())
            .await
            .to_entity()
    }

    pub fn select_columns(alias: &str) -> String {
        format!(
            " {0}.Club_ID, {0}.Club_Abbr, {0}.Club_Name, {0}.Club_UltraAbbr, {0}.Club_City ",
            alias
        )
    }
}

impl ToEntity<Club> for Row {
    fn to_entity(&self) -> Club {
        let club_id: i32 = self.get_column("Club_ID");
        let binding = load_club_flags();
        let mut flag_url = None;
        if let Some(club_flag) = binding.get(&club_id) {
            flag_url = Some(club_flag.flag_url.clone());
        }
        Club {
            id: club_id,
            short_name: self.get_column("Club_Abbr"),
            long_name: self.try_get_column("Club_Name"),
            abbreviation: self.try_get_column("Club_UltraAbbr"),
            city: self.get_column("Club_City"),
            participations_count: self.try_get_column("Participations_Count"),
            flag_url,
        }
    }
}
