use super::{AgeClass, BoatClass, Column, RowToEntity, TryRowToEntity};
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

impl RowToEntity<Race> for Row {
    fn to_entity(&self) -> Race {
        let short_label: String = Column::get(self, "Offer_ShortLabel");
        let long_label: String = Column::get(self, "Offer_LongLabel");
        let comment: String = Column::get(self, "Offer_Comment");

        Race {
            id: Column::get(self, "Offer_ID"),
            comment: comment.trim().to_owned(),
            number: Column::get(self, "Offer_RaceNumber"),
            short_label: short_label.trim().to_owned(),
            long_label: long_label.trim().to_owned(),
            distance: Column::get(self, "Offer_Distance"),
            lightweight: Column::get(self, "Offer_IsLightweight"),
            cancelled: Column::get(self, "Offer_Cancelled"),
            registrations_count: Column::get(self, "Registrations_Count"),
            seeded: Column::get(self, "isSet"),
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
