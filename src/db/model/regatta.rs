use crate::db::{
    model::utils,
    sql::builder::SqlBuilder,
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use serde::Serialize;
use tiberius::{time::chrono::NaiveDateTime, Query, Row};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Regatta {
    pub id: i32,
    title: String,
    sub_title: String,
    venue: String,
    start_date: String,
    end_date: String,
    url: String,
}

impl From<&Row> for Regatta {
    fn from(value: &Row) -> Self {
        let start_date: NaiveDateTime = value.get_column("Event_StartDate");
        let end_date: NaiveDateTime = value.get_column("Event_EndDate");

        Regatta {
            id: value.get_column("Event_ID"),
            title: value.get_column("Event_Title"),
            sub_title: value.try_get_column("Event_SubTitle").unwrap_or_default(),
            venue: value.try_get_column("Event_Venue").unwrap_or_default(),
            start_date: start_date.date().to_string(),
            end_date: end_date.date().to_string(),
            url: value.try_get_column("Event_Url").unwrap_or_default(),
        }
    }
}

impl Regatta {
    pub async fn query_active_regatta(pool: &TiberiusPool) -> Regatta {
        let mut client = pool.get().await;
        let sql = SqlBuilder::select_from("Event")
            .limit(1)
            .columns(&["*"])
            .order_by(&[("Event_StartDate", false), ("Event_ID", false)])
            .build()
            .unwrap();
        let stream = Query::new(sql).query(&mut client).await.unwrap();
        Regatta::from(&utils::get_row(stream).await)
    }

    pub async fn query_all(pool: &TiberiusPool) -> Vec<Regatta> {
        let mut client = pool.get().await;
        let stream = Query::new("SELECT * FROM Event").query(&mut client).await.unwrap();
        let regattas = utils::get_rows(stream).await;
        regattas.into_iter().map(|row| Regatta::from(&row)).collect()
    }

    pub async fn query(regatta_id: i32, pool: &TiberiusPool) -> Regatta {
        let sql = SqlBuilder::select_from("Event")
            .columns(&["*"])
            .where_eq("Event_ID", "@P1")
            .build()
            .unwrap();
        let mut query = Query::new(sql);
        query.bind(regatta_id);

        let mut client = pool.get().await;
        Regatta::from(&utils::get_row(query.query(&mut client).await.unwrap()).await)
    }
}
