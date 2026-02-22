use super::get_row;
use super::get_rows;
use crate::tiberius::TiberiusClient;
use crate::{
    aquarius::flags_scraper::ClubFlag,
    error::DbError,
    tiberius::{RowColumn, TryRowColumn},
};
use ::serde::Serialize;
use ::tiberius::{Query, Row, numeric::Decimal};
use ::utoipa::ToSchema;

const ID: &str = "Club_ID";
const EXTERN_ID: &str = "Club_ExternID";
const SHORT_NAME: &str = "Club_Abbr";
const LONG_NAME: &str = "Club_Name";
const ABBREVIATION: &str = "Club_UltraAbbr";
const CITY: &str = "Club_City";
const LATITUDE: &str = "Club_HRV_Latitude";
const LONGITUDE: &str = "Club_HRV_Longitude";

#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Club {
    /// The internal ID of the club.
    pub id: i32,

    /// This is the ID used by the external system to identify the club.
    #[serde(skip_serializing_if = "Option::is_none")]
    extern_id: Option<i32>,

    /// The short name of the club.
    #[serde(skip_serializing_if = "Option::is_none")]
    short_name: Option<String>,

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
    athletes_count: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    athletes_female_count: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    athletes_male_count: Option<i32>,

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
    /// * `client` - The database connection
    /// # Returns
    /// A list of clubs that are participating in the regatta
    pub async fn query_clubs_participating_regatta(
        regatta_id: i32,
        client: &mut TiberiusClient,
    ) -> Result<Vec<Self>, DbError> {
        let sql = format!(
            "SELECT DISTINCT {0},
                (SELECT COUNT(*) FROM (
                    SELECT DISTINCT Entry_ID FROM Club
                    JOIN Athlet     ON Athlet_Club_ID_FK  = {ID}
                    JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
                    JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
                    WHERE Entry_Event_ID_FK = @P1 AND c.{ID} = {ID} AND Entry_CancelValue = 0
                        AND Crew_RoundTo = 64
                ) AS Participations_Count) AS Participations_Count,
                (SELECT COUNT(*) FROM (
                    SELECT DISTINCT Athlet_ID
                    FROM Club
                    JOIN Athlet     ON Athlet_Club_ID_FK  = {ID}
                    JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
                    JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
                    WHERE Entry_Event_ID_FK = @P1 AND c.{ID} = {ID} AND Entry_CancelValue = 0
                        AND Crew_RoundTo = 64 AND Athlet_Gender = 'W'
                ) AS Athletes_Female_Count) AS Athletes_Female_Count,
                (SELECT COUNT(*) FROM (
                    SELECT DISTINCT Athlet_ID
                    FROM Club
                    JOIN Athlet     ON Athlet_Club_ID_FK  = {ID}
                    JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
                    JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
                    WHERE Entry_Event_ID_FK = @P1 AND c.{ID} = {ID} AND Entry_CancelValue = 0
                        AND Crew_RoundTo = 64 AND Athlet_Gender = 'M'
                ) AS Athletes_Male_Count) AS Athletes_Male_Count
            FROM Club c
            JOIN Athlet  a ON a.Athlet_Club_ID_FK   = c.{ID}
            JOIN Crew   cr ON cr.Crew_Athlete_ID_FK = a.Athlet_ID
            JOIN Entry   e ON cr.Crew_Entry_ID_FK   = e.Entry_ID
            WHERE Entry_Event_ID_FK = @P1 AND Crew_RoundTo = 64
            ORDER BY {CITY} ASC",
            Club::select_all_columns("c")
        );
        let mut query = Query::new(sql);
        query.bind(regatta_id);

        let clubs = get_rows(query.query(client).await?).await?;
        Ok(clubs.into_iter().map(|row| Club::from(&row)).collect())
    }

    /// Query a single club by its identifier with additional aggregations such as the number of participations and athletes
    /// # Arguments
    /// * `regatta_id` - The regatta identifier
    /// * `club_id` - The club identifier
    /// * `pool` - The database connection pool
    /// # Returns
    /// The club with the given ID and additional aggregations such as the number of participations and athletes
    pub async fn query_club_with_aggregations(
        regatta_id: i32,
        club_id: i32,
        client: &mut TiberiusClient,
    ) -> Result<Self, DbError> {
        let mut query = Query::new(format!(
            "SELECT {0},
                (SELECT COUNT(*) FROM (
                    SELECT DISTINCT Entry_ID FROM Club
                    JOIN Athlet     ON Athlet_Club_ID_FK  = {ID}
                    JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
                    JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
                    WHERE Entry_Event_ID_FK = @P1 AND c.{ID} = {ID} AND Entry_CancelValue = 0
                        AND Crew_RoundTo = 64
                ) AS Participations_Count) AS Participations_Count,
                (SELECT COUNT(*) FROM (
                    SELECT DISTINCT Athlet_ID FROM Club
                    JOIN Athlet     ON Athlet_Club_ID_FK  = {ID}
                    JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
                    JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
                    WHERE Entry_Event_ID_FK = @P1 AND c.{ID} = {ID} AND Entry_CancelValue = 0
                        AND Crew_RoundTo = 64 AND Athlet_Gender = 'W'
                ) AS Athletes_Female_Count) AS Athletes_Female_Count,
                (SELECT COUNT(*) FROM (
                    SELECT DISTINCT Athlet_ID FROM Club
                    JOIN Athlet     ON Athlet_Club_ID_FK  = {ID}
                    JOIN Crew       ON Crew_Athlete_ID_FK = Athlet_ID
                    JOIN Entry      ON Crew_Entry_ID_FK   = Entry_ID
                    WHERE Entry_Event_ID_FK = @P1 AND c.{ID} = {ID} AND Entry_CancelValue = 0
                        AND Crew_RoundTo = 64 AND Athlet_Gender = 'M'
                ) AS Athletes_Male_Count) AS Athletes_Male_Count
            FROM Club c
            WHERE c.{ID} = @P2",
            Club::select_all_columns("c")
        ));
        query.bind(regatta_id);
        query.bind(club_id);

        Ok(Club::from(&get_row(query.query(client).await?).await?))
    }

    pub(crate) fn select_all_columns(alias: &str) -> String {
        format!(
            "{alias}.{ID}, {alias}.{SHORT_NAME}, {alias}.{LONG_NAME}, {alias}.{ABBREVIATION}, {alias}.{CITY}, {alias}.{EXTERN_ID}, {alias}.{LATITUDE}, {alias}.{LONGITUDE}"
        )
    }

    pub(crate) fn select_min_columns(alias: &str) -> String {
        format!("{alias}.{ID}, {alias}.{SHORT_NAME}, {alias}.{ABBREVIATION}, {alias}.{CITY}, {alias}.{EXTERN_ID}")
    }
}

impl From<&Row> for Club {
    fn from(value: &Row) -> Self {
        let mut flag_url = None;
        let club_extern_id = value.try_get_column(EXTERN_ID);
        if let Some(extern_id) = club_extern_id {
            if let Some(club_flag) = ClubFlag::get(&extern_id) {
                flag_url = Some(club_flag.flag_url.clone());
            } else {
                flag_url = Some(format!("images/flags/{extern_id}.png"));
            }
        }

        let athletes_female_count = value.try_get_column("Athletes_Female_Count");
        let athletes_male_count = value.try_get_column("Athletes_Male_Count");
        let athletes_count = athletes_female_count.zip(athletes_male_count).map(|(x, y)| x + y);

        Club {
            id: value.get_column(ID),
            extern_id: club_extern_id,
            short_name: value.try_get_column(SHORT_NAME),
            long_name: value.try_get_column(LONG_NAME),
            abbreviation: value.try_get_column(ABBREVIATION),
            city: value.get_column(CITY),
            participations_count: value.try_get_column("Participations_Count"),
            athletes_count,
            athletes_female_count,
            athletes_male_count,
            flag_url,
            latitude: value.try_get_column(LATITUDE),
            longitude: value.try_get_column(LONGITUDE),
        }
    }
}
