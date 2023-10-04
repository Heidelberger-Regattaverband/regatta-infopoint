use crate::db::{
    model::{utils, AgeClass, BoatClass, Registration, ToEntity, TryToEntity},
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tiberius::{Query, Row};

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

impl Race {
    pub fn select_columns(alias: &str) -> String {
        format!(" {0}.Offer_ID, {0}.Offer_RaceNumber, {0}.Offer_Distance, {0}.Offer_IsLightweight, {0}.Offer_Cancelled, {0}.Offer_ShortLabel, {0}.Offer_LongLabel, {0}.Offer_Comment, {0}.Offer_GroupMode, {0}.Offer_SortValue, {0}.Offer_HRV_Seeded ", alias)
    }
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
        <Row as TryRowColumn<i32>>::try_get_column(self, "Offer_ID").map(|_id| self.to_entity())
    }
}

impl Race {
    pub async fn query_all(regatta_id: i32, pool: &TiberiusPool) -> Vec<Race> {
        let mut client = pool.get().await;
        let mut query = Query::new("SELECT DISTINCT".to_string() + &Race::select_columns("o")
                + ","
                + &AgeClass::select_columns("a")
                + ","
                + &BoatClass::select_columns("b")
                + ", (SELECT Count(*) FROM Entry WHERE Entry_Race_ID_FK = Offer_ID AND Entry_CancelValue = 0) as Registrations_Count,
                (SELECT AVG(Comp_State) FROM Comp WHERE Comp_Race_ID_FK = Offer_ID AND Comp_Cancelled = 0) as Race_State
            FROM Offer o
            JOIN AgeClass a  ON o.Offer_AgeClass_ID_FK  = a.AgeClass_ID
            JOIN BoatClass b ON o.Offer_BoatClass_ID_FK = b.BoatClass_ID
            WHERE o.Offer_Event_ID_FK = @P1 ORDER BY o.Offer_SortValue ASC");
        query.bind(regatta_id);
        let stream = query.query(&mut client).await.unwrap();
        let races = utils::get_rows(stream).await;
        races.into_iter().map(|row| row.to_entity()).collect()
    }

    pub async fn query_single(race_id: i32, pool: &TiberiusPool) -> Race {
        let mut client = pool.get().await;
        let mut query = Query::new("SELECT".to_string() + &Race::select_columns("o")
                + ", (SELECT Count(*) FROM Entry e WHERE e.Entry_Race_ID_FK = o.Offer_ID AND e.Entry_CancelValue = 0) as Registrations_Count,
                (SELECT AVG(c.Comp_State) FROM Comp c WHERE c.Comp_Race_ID_FK = o.Offer_ID AND c.Comp_Cancelled = 0) as Race_State
            FROM  Offer o
            WHERE o.Offer_ID = @P1");
        query.bind(race_id);
        let stream = query.query(&mut client).await.unwrap();
        utils::get_row(stream).await.to_entity()
    }
}
