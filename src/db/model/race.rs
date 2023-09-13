use crate::db::{
    aquarius::AquariusClient,
    model::{utils, AgeClass, BoatClass, ToEntity, TryToEntity},
    tiberius::{RowColumn, TryRowColumn},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tiberius::{Query, Row};

use super::Registration;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Race {
    pub id: i32,
    number: String,
    short_label: String,
    long_label: String,
    comment: String,
    distance: i16,
    lightweight: bool,
    state: i32,
    cancelled: bool,
    registrations_count: i32,
    seeded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    age_class: Option<AgeClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    boat_class: Option<BoatClass>,
    group_mode: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    date_time: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub registrations: Option<Vec<Registration>>,
}

impl ToEntity<Race> for Row {
    fn to_entity(&self) -> Race {
        let short_label: String = self.get_column("Offer_ShortLabel");
        let long_label: String = self.get_column("Offer_LongLabel");
        let comment: String = self.try_get_column("Offer_Comment").unwrap_or_default();
        let seeded: Option<bool> = self.try_get_column("Offer_HRV_Seeded");

        Race {
            id: self.get_column("Offer_ID"),
            comment: comment.trim().to_owned(),
            number: self.get_column("Offer_RaceNumber"),
            short_label: short_label.trim().to_owned(),
            long_label: long_label.trim().to_owned(),
            distance: self.get_column("Offer_Distance"),
            lightweight: self.get_column("Offer_IsLightweight"),
            cancelled: self.get_column("Offer_Cancelled"),
            registrations_count: self.try_get_column("Registrations_Count").unwrap_or_default(),
            seeded: seeded.unwrap_or_default(),
            age_class: self.try_to_entity(),
            boat_class: self.try_to_entity(),
            state: self.try_get_column("Race_State").unwrap_or_default(),
            group_mode: self.get_column("Offer_GroupMode"),
            date_time: self.try_get_column("Race_DateTime"),
            registrations: None,
        }
    }
}

impl TryToEntity<Race> for Row {
    fn try_to_entity(&self) -> Option<Race> {
        if <Row as TryRowColumn<i32>>::try_get_column(self, "Offer_ID").is_some() {
            Some(self.to_entity())
        } else {
            None
        }
    }
}

impl Race {
    pub async fn query_all<'a>(regatta_id: i32, client: &mut AquariusClient<'_>) -> Vec<Race> {
        let mut query = Query::new("SELECT DISTINCT Offer.*, AgeClass.*, BoatClass.*,
            (SELECT Count(*) FROM Entry WHERE Entry_Race_ID_FK = Offer_ID AND Entry_CancelValue = 0) as Registrations_Count,
            (SELECT AVG(Comp_State) FROM Comp WHERE Comp_Race_ID_FK = Offer_ID AND Comp_Cancelled = 0) as Race_State
            FROM Offer
            JOIN AgeClass  ON Offer_AgeClass_ID_FK  = AgeClass_ID
            JOIN BoatClass ON Offer_BoatClass_ID_FK = BoatClass_ID
            WHERE Offer_Event_ID_FK = @P1 ORDER BY Offer_SortValue ASC");
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let races = utils::get_rows(stream).await;
        races.into_iter().map(|row| row.to_entity()).collect()
    }

    pub async fn query_single<'a>(race_id: i32, client: &mut AquariusClient<'_>) -> Race {
        let mut query = Query::new("SELECT o.*,
            (SELECT Count(*) FROM Entry e WHERE e.Entry_Race_ID_FK = o.Offer_ID AND e.Entry_CancelValue = 0) as Registrations_Count,
            (SELECT AVG(c.Comp_State) FROM Comp c WHERE c.Comp_Race_ID_FK = o.Offer_ID AND c.Comp_Cancelled = 0) as Race_State
            FROM  Offer o
            WHERE o.Offer_ID = @P1");
        query.bind(race_id);
        let stream = query.query(client).await.unwrap();
        utils::get_row(stream).await.to_entity()
    }
}
