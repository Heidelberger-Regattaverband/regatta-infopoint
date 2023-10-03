use crate::db::{
    model::{utils, HeatResult, Registration, ToEntity, TryToEntity},
    tiberius::{RowColumn, TiberiusPool},
};
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
    pub async fn query_all(heat_id: i32, pool: &TiberiusPool) -> Vec<HeatRegistration> {
        let mut client = pool.get().await;
        let mut query = Query::new("SELECT DISTINCT
                ce.CE_ID, ce.CE_Lane,
                e.Entry_ID, e.Entry_Bib, e.Entry_Comment, e.Entry_BoatNumber, e.Entry_GroupValue, e.Entry_CancelValue,
                Label_Short, Result_Rank, Result_DisplayValue, Result_Delta, BoatClass_NumRowers, Club.*, Offer.*
            FROM CompEntries ce
            JOIN Comp                  ON CE_Comp_ID_FK = Comp_ID
            JOIN Offer                 ON Offer_ID      = Comp_Race_ID_FK
            JOIN BoatClass             ON Offer_BoatClass_ID_FK = BoatClass_ID
            FULL OUTER JOIN Entry e    ON CE_Entry_ID_FK = e.Entry_ID
            FULL OUTER JOIN EntryLabel ON EL_Entry_ID_FK = e.Entry_ID
            FULL OUTER JOIN Label      ON EL_Label_ID_FK = Label_ID
            FULL OUTER JOIN Result     ON Result_CE_ID_FK = CE_ID
            JOIN Club                  ON Club_ID = Entry_OwnerClub_ID_FK
            WHERE CE_Comp_ID_FK = @P1 AND ((Result_SplitNr = 64 AND Comp_State >=4) OR (Result_SplitNr = 0 AND Comp_State < 3) OR (Comp_State < 2 AND Result_SplitNr IS NULL))
            AND EL_RoundFrom <= Comp_Round AND Comp_Round <= EL_RoundTo");
        query.bind(heat_id);
        let stream = query.query(&mut client).await.unwrap();
        let crew = utils::get_rows(stream).await;
        crew.into_iter().map(|row| row.to_entity()).collect()
    }
}
