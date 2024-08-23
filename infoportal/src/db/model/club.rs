use crate::http::flags_scraper::ClubFlag;
use aquarius::db::model::utils;
use aquarius::db::tiberius::TiberiusPool;
use aquarius::db::tiberius::{RowColumn, TryRowColumn};
use serde::Serialize;
use tiberius::{numeric::Decimal, Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Club {
    /// The internal ID of the club.
    pub id: i32,

    /// This is the ID used by the external system to identify the club.
    #[serde(skip_serializing_if = "Option::is_none")]
    extern_id: Option<i32>,

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

    /// The number of athletes in this club that are participating.
    #[serde(skip_serializing_if = "Option::is_none")]
    ahtletes_count: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    ahtletes_female_count: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    ahtletes_male_count: Option<i32>,

    /// An optional URL showing the flag of the club.
    #[serde(skip_serializing_if = "Option::is_none")]
    flag_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    latitude: Option<Decimal>,

    #[serde(skip_serializing_if = "Option::is_none")]
    longitude: Option<Decimal>,
}

impl Club {
    /// Query all clubs that are participating in a regatta.
    /// # Arguments
    /// * `regatta_id` - The regatta identifier
    /// * `pool` - The database connection pool
    /// # Returns
    /// A list of clubs that are participating in the regatta
    pub(crate) async fn query_clubs_participating_regatta(regatta_id: i32, pool: &TiberiusPool) -> Vec<Club> {
        let sql = format!(
            "SELECT DISTINCT {0},
                (SELECT COUNT(*) FROM (
                    SELECT DISTINCT Entry_ID FROM Club
                    JOIN Athlet     ON Athlet_Club_ID_FK  = Club_ID
                    JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
                    JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
                    JOIN Event      ON Entry_Event_ID_FK  = Event_ID
                    WHERE Event_ID = e.Event_ID AND c.Club_ID = Club_ID AND Entry_CancelValue = 0
                        AND Crew_RoundTo = 64
                ) AS Participations_Count) AS Participations_Count,
                (SELECT COUNT(*) FROM (
                    SELECT DISTINCT Athlet_ID
                    FROM Club
                    JOIN Athlet     ON Athlet_Club_ID_FK  = Club_ID
                    JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
                    JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
                    JOIN Event      ON Entry_Event_ID_FK  = Event_ID
                    WHERE Event_ID = e.Event_ID AND c.Club_ID = Club_ID AND Entry_CancelValue = 0
                        AND Crew_RoundTo = 64 AND Athlet_Gender = 'W'
                ) AS Athletes_Female_Count) AS Athletes_Female_Count,
                (SELECT COUNT(*) FROM (
                    SELECT DISTINCT Athlet_ID
                    FROM Club
                    JOIN Athlet     ON Athlet_Club_ID_FK  = Club_ID
                    JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
                    JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
                    JOIN Event      ON Entry_Event_ID_FK  = Event_ID
                    WHERE Event_ID = e.Event_ID AND c.Club_ID = Club_ID AND Entry_CancelValue = 0
                        AND Crew_RoundTo = 64 AND Athlet_Gender = 'M'
                ) AS Athletes_Male_Count) AS Athletes_Male_Count
            FROM Club c
            JOIN Athlet ON Athlet_Club_ID_FK      = c.Club_ID
            JOIN Crew   ON Crew_Athlete_ID_FK     = Athlet_ID
            JOIN Entry  ON Crew_Entry_ID_FK       = Entry_ID
            JOIN Event AS e ON Entry_Event_ID_FK  = Event_ID
            WHERE Event_ID = @P1 AND Crew_RoundTo = 64
            ORDER BY Club_City ASC",
            Club::select_columns("c")
        );
        let mut query = Query::new(sql);
        query.bind(regatta_id);

        let mut client = pool.get().await;
        let clubs = utils::get_rows(query.query(&mut client).await.unwrap()).await;
        clubs.into_iter().map(|row| Club::from(&row)).collect()
    }

    /// Query a single club by its identifier
    /// # Arguments
    /// * `club_id` - The club identifier
    /// * `pool` - The database connection pool
    /// # Returns
    /// The club with the given ID
    pub(crate) async fn query_club_by_id(club_id: i32, pool: &TiberiusPool) -> Club {
        let mut query = Query::new(format!(
            "SELECT {0} FROM Club c
            WHERE c.Club_ID = @P1
            ORDER BY c.Club_City ASC",
            Club::select_columns("c")
        ));
        query.bind(club_id);

        let mut client = pool.get().await;
        Club::from(&utils::get_row(query.query(&mut client).await.unwrap()).await)
    }

    pub(crate) async fn query_regatta_club_by_id(regatta_id: i32, club_id: i32, pool: &TiberiusPool) -> Club {
        let mut query = Query::new(format!(
            "SELECT {0},
                (SELECT COUNT(*) FROM (
                    SELECT DISTINCT Entry_ID FROM Club
                    JOIN Athlet     ON Athlet_Club_ID_FK  = Club_ID
                    JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
                    JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
                    JOIN Event      ON Entry_Event_ID_FK  = Event_ID
                    WHERE Event_ID = @P1 AND c.Club_ID = Club_ID AND Entry_CancelValue = 0
                        AND Crew_RoundTo = 64
                ) AS Participations_Count) AS Participations_Count,
                (SELECT COUNT(*) FROM (
                    SELECT DISTINCT Athlet_ID
                    FROM Club
                    JOIN Athlet     ON Athlet_Club_ID_FK  = Club_ID
                    JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
                    JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
                    JOIN Event      ON Entry_Event_ID_FK  = Event_ID
                    WHERE Event_ID = @P1 AND c.Club_ID = Club_ID AND Entry_CancelValue = 0
                        AND Crew_RoundTo = 64 AND Athlet_Gender = 'W'
                ) AS Athletes_Female_Count) AS Athletes_Female_Count,
                (SELECT COUNT(*) FROM (
                    SELECT DISTINCT Athlet_ID
                    FROM Club
                    JOIN Athlet     ON Athlet_Club_ID_FK  = Club_ID
                    JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
                    JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
                    JOIN Event      ON Entry_Event_ID_FK  = Event_ID
                    WHERE Event_ID = @P1 AND c.Club_ID = Club_ID AND Entry_CancelValue = 0
                        AND Crew_RoundTo = 64 AND Athlet_Gender = 'M'
                ) AS Athletes_Male_Count) AS Athletes_Male_Count
            FROM Club c
            WHERE c.Club_ID = @P2
            ORDER BY c.Club_City ASC",
            Club::select_columns("c")
        ));
        query.bind(regatta_id);
        query.bind(club_id);

        let mut client = pool.get().await;
        Club::from(&utils::get_row(query.query(&mut client).await.unwrap()).await)
    }

    pub(crate) fn select_columns(alias: &str) -> String {
        format!(
            " {0}.Club_ID, {0}.Club_Abbr, {0}.Club_Name, {0}.Club_UltraAbbr, {0}.Club_City, {0}.Club_ExternID, {0}.Club_HRV_Latitude, {0}.Club_HRV_Longitude ",
            alias
        )
    }
}

impl From<&Row> for Club {
    fn from(value: &Row) -> Self {
        let mut flag_url = None;
        let club_extern_id = value.try_get_column("Club_ExternID");
        if let Some(extern_id) = club_extern_id {
            if let Some(club_flag) = ClubFlag::get(&extern_id) {
                flag_url = Some(club_flag.flag_url.clone());
            }
        }

        let ahtletes_female_count = value.try_get_column("Athletes_Female_Count");
        let ahtletes_male_count = value.try_get_column("Athletes_Male_Count");
        let ahtletes_count = ahtletes_female_count.zip(ahtletes_male_count).map(|(x, y)| x + y);

        Club {
            id: value.get_column("Club_ID"),
            extern_id: club_extern_id,
            short_name: value.get_column("Club_Abbr"),
            long_name: value.try_get_column("Club_Name"),
            abbreviation: value.try_get_column("Club_UltraAbbr"),
            city: value.get_column("Club_City"),
            participations_count: value.try_get_column("Participations_Count"),
            ahtletes_count,
            ahtletes_female_count,
            ahtletes_male_count,
            flag_url,
            latitude: value.try_get_column("Club_HRV_Latitude"),
            longitude: value.try_get_column("Club_HRV_Longitude"),
        }
    }
}
