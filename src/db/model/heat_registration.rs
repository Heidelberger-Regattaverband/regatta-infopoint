use crate::db::{
    model::{utils, Club, Crew, Heat, HeatResult, Race, Registration, ToEntity, TryToEntity},
    tiberius::{RowColumn, TiberiusPool},
};
use futures::future::{join_all, BoxFuture};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeatRegistration {
    pub id: i32,
    lane: i16,
    pub registration: Registration,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<HeatResult>,
}

impl ToEntity<HeatRegistration> for Row {
    fn to_entity(&self) -> HeatRegistration {
        HeatRegistration {
            id: self.get_column("CE_ID"),
            lane: self.get_column("CE_Lane"),
            registration: self.to_entity(),
            result: self.try_to_entity(),
        }
    }
}

impl HeatRegistration {
    pub async fn query_all(heat: &Heat, pool: &TiberiusPool) -> Vec<HeatRegistration> {
        let mut query = Query::new("SELECT DISTINCT
                ce.CE_ID, ce.CE_Lane, ".to_string() + &Registration::select_columns("e") + ", Label_Short, BoatClass_NumRowers,"
                + &Club::select_columns("c") + ", " + &Race::select_columns("o") + ", " + &HeatResult::select_columns("r")
            + "
            FROM CompEntries ce
            JOIN Comp                  ON CE_Comp_ID_FK     = Comp_ID
            JOIN Offer o               ON o.Offer_ID        = Comp_Race_ID_FK
            JOIN BoatClass             ON o.Offer_BoatClass_ID_FK = BoatClass_ID
            FULL OUTER JOIN Entry e    ON CE_Entry_ID_FK    = e.Entry_ID
            FULL OUTER JOIN EntryLabel ON EL_Entry_ID_FK    = e.Entry_ID
            FULL OUTER JOIN Label      ON EL_Label_ID_FK    = Label_ID
            FULL OUTER JOIN Result r   ON r.Result_CE_ID_FK = ce.CE_ID
            JOIN Club c                ON c.Club_ID = Entry_OwnerClub_ID_FK
            WHERE CE_Comp_ID_FK = @P1 AND ((Result_SplitNr = 64 AND Comp_State >=4) OR (Result_SplitNr = 0 AND Comp_State < 3) OR (Comp_State < 2 AND Result_SplitNr IS NULL))
                AND EL_RoundFrom <= Comp_Round AND Comp_Round <= EL_RoundTo");
        query.bind(heat.id);

        let mut client = pool.get().await;
        let crew = utils::get_rows(query.query(&mut client).await.unwrap()).await;

        let mut crew_futures: Vec<BoxFuture<Vec<Crew>>> = Vec::new();
        let mut heat_registrations: Vec<HeatRegistration> = crew
            .into_iter()
            .map(|row| {
                let mut heat_registration: HeatRegistration = row.to_entity();
                crew_futures.push(Box::pin(Crew::query_all(
                    heat_registration.registration.id,
                    heat.round,
                    pool,
                )));
                // if a result is available, the registration isn't cancelled yet
                if heat_registration.result.is_some() {
                    heat_registration.registration.cancelled = false;
                }
                heat_registration
            })
            .collect();

        let crews = join_all(crew_futures).await;

        // if let Some(registration) = heat_registrations.get(0) {
        //     if registration.result.is_some() {
        //         let netto_time = registration.result.as_ref().unwrap().net_time;
        //     }
        // }

        for (pos, heat_registration) in heat_registrations.iter_mut().enumerate() {
            let crew = crews.get(pos).unwrap();
            heat_registration.registration.crew = Some(crew.to_vec());
        }

        heat_registrations
    }
}
