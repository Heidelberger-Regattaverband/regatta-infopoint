use super::{HeatResult, Registration, RowColumn, ToEntity};
use serde::Serialize;
use tiberius::{Query, Row};

#[derive(Debug, Serialize, Clone)]
pub struct HeatRegistration {
    pub id: i32,
    lane: i16,
    result: HeatResult,
    pub registration: Registration,
}

impl ToEntity<HeatRegistration> for Row {
    fn to_entity(&self) -> HeatRegistration {
        HeatRegistration {
            id: self.get_column("CE_ID"),
            lane: self.get_column("CE_Lane"),
            registration: self.to_entity(),
            result: self.to_entity(),
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
        let mut query = Query::new("SELECT DISTINCT ce.*, e.Entry_Bib, e.Entry_ID, e.Entry_BoatNumber, e.Entry_Comment, e.Entry_CancelValue, l.Label_Short, r.Result_Rank, r.Result_DisplayValue, r.Result_Delta, bc.BoatClass_NumRowers, cl.Club_ID, cl.Club_Abbr, cl.Club_City
            FROM CompEntries AS ce
            JOIN Comp AS c ON ce.CE_Comp_ID_FK = c.Comp_ID
            JOIN Offer AS o ON o.Offer_ID = c.Comp_Race_ID_FK
            JOIN BoatClass AS bc ON o.Offer_BoatClass_ID_FK = bc.BoatClass_ID
            FULL OUTER JOIN Entry AS e ON ce.CE_Entry_ID_FK = e.Entry_ID
            FULL OUTER JOIN EntryLabel AS el ON el.EL_Entry_ID_FK = e.Entry_ID
            FULL OUTER JOIN Label AS l ON el.EL_Label_ID_FK = l.Label_ID
            FULL OUTER JOIN Result AS r ON r.Result_CE_ID_FK = ce.CE_ID
            JOIN Club AS cl ON cl.Club_ID = e.Entry_OwnerClub_ID_FK
            WHERE ce.CE_Comp_ID_FK = @P1 AND (r.Result_SplitNr = 64 OR r.Result_SplitNr IS NULL)
            AND el.EL_RoundFrom <= c.Comp_Round AND c.Comp_Round <= el.EL_RoundTo");
        query.bind(heat_id);
        query
    }
}
