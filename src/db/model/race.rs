use super::{AgeClass, BoatClass, RowColumn, ToEntity, TryToEntity};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
pub struct Race {
    pub id: i32,
    number: String,
    #[serde(rename = "shortLabel")]
    short_label: String,
    #[serde(rename = "longLabel")]
    long_label: String,
    comment: String,
    distance: i16,
    lightweight: bool,
    cancelled: bool,
    #[serde(rename = "registrationsCount")]
    registrations_count: i32,
    seeded: bool,
    #[serde(rename = "ageClass", skip_serializing_if = "Option::is_none")]
    age_class: Option<AgeClass>,
    #[serde(rename = "boatClass", skip_serializing_if = "Option::is_none")]
    boat_class: Option<BoatClass>,
}

impl ToEntity<Race> for Row {
    fn to_entity(&self) -> Race {
        let short_label: String = self.get_column("Offer_ShortLabel");
        let long_label: String = self.get_column("Offer_LongLabel");
        let comment: String = self.get_column("Offer_Comment");

        Race {
            id: self.get_column("Offer_ID"),
            comment: comment.trim().to_owned(),
            number: self.get_column("Offer_RaceNumber"),
            short_label: short_label.trim().to_owned(),
            long_label: long_label.trim().to_owned(),
            distance: self.get_column("Offer_Distance"),
            lightweight: self.get_column("Offer_IsLightweight"),
            cancelled: self.get_column("Offer_Cancelled"),
            registrations_count: self.get_column("Registrations_Count"),
            seeded: self.get_column("isSet"),
            age_class: self.try_to_entity(),
            boat_class: self.try_to_entity(),
        }
    }
}

impl Race {
    pub fn from_rows(rows: &Vec<Row>) -> Vec<Self> {
        let mut races: Vec<Race> = Vec::with_capacity(rows.len());
        for row in rows {
            races.push(row.to_entity());
        }
        races
    }

    pub fn query_all<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT DISTINCT o.*, ac.*, bc.*, hrv_o.*,
            (SELECT Count(*) FROM Entry e WHERE e.Entry_Race_ID_FK = o.Offer_ID AND e.Entry_CancelValue = 0) as Registrations_Count
            FROM Offer o
            JOIN AgeClass AS ac ON o.Offer_AgeClass_ID_FK = ac.AgeClass_ID
            JOIN BoatClass AS bc ON o.Offer_BoatClass_ID_FK = bc.BoatClass_ID
            FULL OUTER JOIN HRV_Offer AS hrv_o ON o.Offer_ID = hrv_o.id
            WHERE o.Offer_Event_ID_FK = @P1 ORDER BY o.Offer_SortValue ASC");
        query.bind(regatta_id);
        query
    }

    pub fn query_single<'a>(race_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT o.*,
            (SELECT Count(*) FROM Entry e WHERE e.Entry_Race_ID_FK = o.Offer_ID AND e.Entry_CancelValue = 0) as Registrations_Count
            FROM Offer o
            WHERE o.Offer_ID = @P1");
        query.bind(race_id);
        query
    }
}
