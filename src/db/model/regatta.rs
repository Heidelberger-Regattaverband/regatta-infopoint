use super::column::Column;
use serde::Serialize;
use tiberius::{time::chrono::NaiveDateTime, Query, Row};

#[derive(Debug, Serialize, Clone)]
pub struct Regatta {
    pub id: i32,
    title: String,
    sub_title: String,
    venue: String,
    #[serde(rename = "startDate")]
    start_date: String,
    #[serde(rename = "endDate")]
    end_date: String,
}

impl Regatta {
    pub fn from_rows(rows: &Vec<Row>) -> Vec<Self> {
        let mut regattas: Vec<Regatta> = Vec::with_capacity(rows.len());
        for row in rows {
            let regatta = Regatta::from_row(row);
            regattas.push(regatta);
        }
        regattas
    }

    pub fn from_row(row: &Row) -> Self {
        let start_date: NaiveDateTime = Column::get(row, "Event_StartDate");
        let end_date: NaiveDateTime = Column::get(row, "Event_EndDate");

        Regatta {
            id: Column::get(row, "Event_ID"),
            title: Column::get(row, "Event_Title"),
            sub_title: Column::get(row, "Event_SubTitle"),
            venue: Column::get(row, "Event_Venue"),
            start_date: start_date.date().to_string(),
            end_date: end_date.date().to_string(),
        }
    }

    pub fn query_all<'a>() -> Query<'a> {
        Query::new("SELECT * FROM Event e")
    }

    pub fn query_single<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT * FROM Event e WHERE e.Event_ID = @P1");
        query.bind(regatta_id);
        query
    }
}
