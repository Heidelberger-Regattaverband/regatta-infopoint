use crate::db::utils::Column;
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
pub struct AgeClass {
    id: i32,
    caption: String,
    abbreviation: String,
    suffix: String,
    gender: String,
    #[serde(rename = "numSubClasses")]
    num_sub_classes: u8,
}

impl AgeClass {
    pub fn from_row(row: &Row) -> Option<Self> {
        if let Some(id) = Column::get(row, "AgeClass_ID") {
            let caption = Column::get(row, "AgeClass_Caption");
            let abbreviation = Column::get(row, "AgeClass_Abbr");
            let suffix = Column::get(row, "AgeClass_Suffix");
            let gender = Column::get(row, "AgeClass_Gender");
            let num_sub_classes = Column::get(row, "AgeClass_NumSubClasses");
            Some(AgeClass {
                id,
                caption,
                abbreviation,
                suffix,
                gender,
                num_sub_classes,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct BoatClass {
    id: i32,
    caption: String,
    abbreviation: String,
    #[serde(rename = "numRowers")]
    num_rowers: i32,
    coxed: bool,
}

impl BoatClass {
    pub fn from_row(row: &Row) -> Option<Self> {
        if let Some(id) = Column::get(row, "BoatClass_ID") {
            let caption = Column::get(row, "BoatClass_Caption");
            let abbreviation = Column::get(row, "BoatClass_Abbr");
            let num_rowers = Column::get(row, "BoatClass_NumRowers");
            let coxed: u8 = Column::get(row, "BoatClass_Coxed");
            Some(BoatClass {
                id,
                caption,
                abbreviation,
                num_rowers,
                coxed: coxed > 0,
            })
        } else {
            None
        }
    }
}

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
    #[serde(rename = "raceMode", skip_serializing_if = "Option::is_none")]
    race_mode: Option<String>,
    cancelled: bool,
    #[serde(rename = "registrationsCount")]
    registrations_count: i32,
    seeded: bool,
    #[serde(rename = "ageClass", skip_serializing_if = "Option::is_none")]
    age_class: Option<AgeClass>,
    #[serde(rename = "boatClass", skip_serializing_if = "Option::is_none")]
    boat_class: Option<BoatClass>,
}

impl Race {
    pub fn from_rows(rows: &Vec<Row>) -> Vec<Self> {
        let mut races: Vec<Race> = Vec::with_capacity(rows.len());
        for row in rows {
            races.push(Race::from_row(row));
        }
        races
    }

    pub fn from_row(row: &Row) -> Self {
        let short_label: String = Column::get(row, "Offer_ShortLabel");
        let long_label: String = Column::get(row, "Offer_LongLabel");
        let comment: String = Column::get(row, "Offer_Comment");

        Race {
            id: Column::get(row, "Offer_ID"),
            comment: comment.trim().to_owned(),
            number: Column::get(row, "Offer_RaceNumber"),
            short_label: short_label.trim().to_owned(),
            long_label: long_label.trim().to_owned(),
            distance: Column::get(row, "Offer_Distance"),
            lightweight: Column::get(row, "Offer_IsLightweight"),
            race_mode: Column::get(row, "RaceMode_Title"),
            cancelled: Column::get(row, "Offer_Cancelled"),
            registrations_count: Column::get(row, "Registrations_Count"),
            seeded: Column::get(row, "isSet"),
            age_class: AgeClass::from_row(row),
            boat_class: BoatClass::from_row(row),
        }
    }

    pub fn query_all<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT DISTINCT o.*, ac.*, bc.*, rm.*, hrv_o.*,
            (SELECT Count(*) FROM Entry e WHERE e.Entry_Race_ID_FK = o.Offer_ID AND e.Entry_CancelValue = 0) as Registrations_Count
            FROM Offer o
            JOIN RaceMode AS rm ON o.Offer_RaceMode_ID_FK = rm.RaceMode_ID
            JOIN AgeClass AS ac ON o.Offer_AgeClass_ID_FK = ac.AgeClass_ID
            JOIN BoatClass AS bc ON o.Offer_BoatClass_ID_FK = bc.BoatClass_ID
            FULL OUTER JOIN HRV_Offer AS hrv_o ON o.Offer_ID = hrv_o.id
            WHERE o.Offer_Event_ID_FK = @P1 ORDER BY o.Offer_SortValue ASC");
        query.bind(regatta_id);
        query
    }

    pub fn query_single<'a>(race_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT o.*, rm.*,
            (SELECT Count(*) FROM Entry e WHERE e.Entry_Race_ID_FK = o.Offer_ID AND e.Entry_CancelValue = 0) as Registrations_Count
            FROM Offer o
            JOIN RaceMode AS rm ON o.Offer_RaceMode_ID_FK = rm.RaceMode_ID
            WHERE o.Offer_ID = @P1");
        query.bind(race_id);
        query
    }
}
