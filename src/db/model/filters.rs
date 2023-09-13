use crate::db::{aquarius::AquariusClient, model::utils, tiberius::RowColumn};
use chrono::NaiveDate;
use serde::Serialize;
use tiberius::Query;

use super::{AgeClass, BoatClass, ToEntity};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Filters {
    /// Available distances of all races.
    distances: Vec<i16>,

    /// All dates of the races.
    dates: Vec<NaiveDate>,

    /// All age classes used in races
    age_classes: Vec<AgeClass>,

    /// All boat classes used in races
    boat_classes: Vec<BoatClass>,
}

impl Filters {
    pub async fn query(regatta_id: i32, client: &mut AquariusClient<'_>) -> Self {
        // get all available distances
        let mut query = Query::new("SELECT DISTINCT Offer_Distance FROM Offer WHERE Offer_Event_ID_FK = @P1");
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let rows = utils::get_rows(stream).await;
        let distances = rows.into_iter().map(|row| row.get_column("Offer_Distance")).collect();

        // get all available dates
        let mut query = Query::new(
            "SELECT DISTINCT CAST (c.Comp_Datetime as date) AS Comp_Date FROM Comp c WHERE c.Comp_Event_ID_FK = @P1",
        );
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let rows = utils::get_rows(stream).await;
        let dates = rows.into_iter().map(|row| row.get_column("Comp_Date")).collect();

        let mut query = Query::new(
            "SELECT DISTINCT a.* FROM AgeClass a JOIN Offer o ON o.Offer_AgeClass_ID_FK = a.AgeClass_ID
            WHERE o.Offer_Event_ID_FK = @P1 ORDER BY AgeClass_MinAge ASC",
        );
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let rows = utils::get_rows(stream).await;
        let age_classes = rows.into_iter().map(|row| row.to_entity()).collect();

        let mut query = Query::new(
            "SELECT DISTINCT b.* FROM BoatClass b JOIN Offer o ON o.Offer_BoatClass_ID_FK = b.BoatClass_ID 
            WHERE o.Offer_Event_ID_FK = @P1 ORDER BY b.BoatClass_NumRowers ASC, b.BoatClass_Coxed ASC",
        );
        query.bind(regatta_id);
        let stream = query.query(client).await.unwrap();
        let rows = utils::get_rows(stream).await;
        let boat_classes = rows.into_iter().map(|row| row.to_entity()).collect();

        Filters {
            distances,
            dates,
            age_classes,
            boat_classes,
        }
    }
}
