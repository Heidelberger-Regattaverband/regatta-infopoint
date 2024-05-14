use crate::db::{
    model::{utils, Club, Crew, Heat, HeatResult, Race, Registration, TryToEntity},
    tiberius::{RowColumn, TiberiusPool},
};
use futures::future::{join_all, BoxFuture};
use serde::Serialize;
use std::{cmp::Ordering, time::Duration};
use tiberius::{Query, Row};

/// A registration of a boat in a heat.
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeatRegistration {
    /// The unique id of the registration.
    pub(crate) id: i32,

    /// The lane in which the boat is supposed to start.
    lane: i16,

    /// The registration of the boat.
    pub(crate) registration: Registration,

    /// The result of the boat in the heat
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<HeatResult>,
}

impl From<&Row> for HeatRegistration {
    fn from(value: &Row) -> Self {
        HeatRegistration {
            id: value.get_column("CE_ID"),
            lane: value.get_column("CE_Lane"),
            registration: Registration::from(value),
            result: value.try_to_entity(),
        }
    }
}

impl HeatRegistration {
    pub(crate) async fn query_all(heat: &Heat, pool: &TiberiusPool) -> Vec<HeatRegistration> {
        let mut query = Query::new("SELECT DISTINCT ce.CE_ID, ce.CE_Lane, ".to_string() +
            &Registration::select_columns("e") + ", Label_Short, BoatClass_NumRowers," +
            &Club::select_columns("c") + ", " + 
            &Race::select_columns("o") + ", " + 
            &HeatResult::select_columns("r") +
            " FROM CompEntries ce
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
        let rows = utils::get_rows(query.query(&mut client).await.unwrap()).await;

        // convert rows into HeatRegistrations
        let mut heat_registrations: Vec<HeatRegistration> = rows
            .into_iter()
            .map(|row| {
                let mut heat_registration = HeatRegistration::from(&row);
                // if a result is available, the registration isn't cancelled yet
                if heat_registration.result.is_some() {
                    heat_registration.registration.cancelled = false;
                }
                heat_registration
            })
            .collect();

        // sort heat registrations by rank
        heat_registrations.sort_by(|a, b| {
            if let (Some(result_a), Some(result_b)) = (a.result.as_ref(), b.result.as_ref()) {
                if result_a.rank_sort > result_b.rank_sort {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            } else {
                Ordering::Equal
            }
        });

        let mut first_net_time: i32 = 0;

        let mut crew_futures: Vec<BoxFuture<Vec<Crew>>> = Vec::new();
        for (pos, heat_registration) in heat_registrations.iter_mut().enumerate() {
            crew_futures.push(Box::pin(Crew::query_crew_of_registration(
                heat_registration.registration.id,
                heat.round,
                pool,
            )));

            if let Some(result) = &mut heat_registration.result {
                if pos == 0 {
                    first_net_time = result.net_time;
                } else if result.rank_sort > 1 && result.rank_sort < u8::MAX {
                    let delta = result.net_time - first_net_time;
                    let duration = Duration::from_millis(delta as u64);
                    let millis = duration.subsec_millis() / 10;
                    result.delta = Some(format!("+{}.{millis:02}", duration.as_secs()));
                }
            }
        }

        // query the crews of all registrations in parallel
        let crews = join_all(crew_futures).await;

        for (pos, heat_registration) in heat_registrations.iter_mut().enumerate() {
            let crew = crews.get(pos).unwrap();
            heat_registration.registration.crew = Some(crew.to_vec());
        }

        heat_registrations
    }
}
