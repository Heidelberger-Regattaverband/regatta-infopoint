use super::{HeatResult, Registration, ToEntity, TryToEntity};
use crate::db::tiberius::RowColumn;
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
    // pub fn from_rows(rows: &Vec<Row>) -> Vec<HeatRegistration> {
    //     let mut heat_regs: Vec<HeatRegistration> = Vec::with_capacity(rows.len());
    //     for row in rows {
    //         heat_regs.push(HeatRegistration::from_row(row));
    //     }
    //     heat_regs
    // }

    pub(crate) fn query_all<'a>(heat_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT DISTINCT CompEntries.*, Entry.*, Label_Short, Result_Rank, Result_DisplayValue, Result_Delta, BoatClass_NumRowers, Club.*, Offer.*
            FROM CompEntries
            JOIN Comp                  ON CE_Comp_ID_FK = Comp_ID
            JOIN Offer                 ON Offer_ID      = Comp_Race_ID_FK
            JOIN BoatClass             ON Offer_BoatClass_ID_FK = BoatClass_ID
            FULL OUTER JOIN Entry      ON CE_Entry_ID_FK = Entry_ID
            FULL OUTER JOIN EntryLabel ON EL_Entry_ID_FK = Entry_ID
            FULL OUTER JOIN Label      ON EL_Label_ID_FK = Label_ID
            FULL OUTER JOIN Result     ON Result_CE_ID_FK = CE_ID
            JOIN Club                  ON Club_ID = Entry_OwnerClub_ID_FK
            WHERE CE_Comp_ID_FK = @P1 AND (Result_SplitNr = 64 OR Result_SplitNr IS NULL)
            AND EL_RoundFrom <= Comp_Round AND Comp_Round <= EL_RoundTo");
        query.bind(heat_id);
        query
    }
}
