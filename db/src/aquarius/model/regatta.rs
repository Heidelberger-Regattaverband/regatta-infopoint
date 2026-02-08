use crate::tiberius::TiberiusClient;
use crate::{
    aquarius::model::utils,
    error::DbError,
    tiberius::{RowColumn, TryRowColumn},
};
use serde::Serialize;
use tiberius::{Query, Row, time::chrono::NaiveDateTime};
use utoipa::ToSchema;

const ID: &str = "Event_ID";
const TITLE: &str = "Event_Title";
const SUB_TITLE: &str = "Event_SubTitle";
const VENUE: &str = "Event_Venue";
const START_DATE: &str = "Event_StartDate";
const END_DATE: &str = "Event_EndDate";
const URL: &str = "Event_Url";

#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Regatta {
    /// The internal ID of the regatta.
    pub id: i32,

    /// The title of the regatta.
    title: String,

    /// The subtitle of the regatta.
    sub_title: String,

    /// The venue of the regatta.
    venue: String,

    /// The start date of the regatta.
    start_date: String,

    /// The end date of the regatta.
    end_date: String,

    /// The URL of the homepage of the regatta.
    url: String,
}

impl From<&Row> for Regatta {
    fn from(value: &Row) -> Self {
        let start_date: NaiveDateTime = value.get_column(START_DATE);
        let end_date: NaiveDateTime = value.get_column(END_DATE);

        Regatta {
            id: value.get_column(ID),
            title: value.get_column(TITLE),
            sub_title: value.try_get_column(SUB_TITLE).unwrap_or_default(),
            venue: value.try_get_column(VENUE).unwrap_or_default(),
            start_date: start_date.date().to_string(),
            end_date: end_date.date().to_string(),
            url: value.try_get_column(URL).unwrap_or_default(),
        }
    }
}

impl Regatta {
    pub async fn query_active_regatta(client: &mut TiberiusClient) -> Result<Regatta, DbError> {
        let stream = Query::new(format!(
            "SELECT TOP 1 {} FROM Event e ORDER BY e.{START_DATE} DESC, e.{ID} DESC",
            Regatta::select_columns("e")
        ))
        .query(client)
        .await?;
        Ok(Regatta::from(&utils::get_row(stream).await?))
    }

    pub async fn query_by_id(regatta_id: i32, client: &mut TiberiusClient) -> Result<Option<Regatta>, DbError> {
        let mut query = Query::new(format!(
            "SELECT {} FROM Event e WHERE e.{ID} = @P1",
            Regatta::select_columns("e")
        ));
        query.bind(regatta_id);

        let row = utils::try_get_row(query.query(client).await?).await?;
        if let Some(row) = row {
            Ok(Some(Regatta::from(&row)))
        } else {
            Ok(None)
        }
    }

    fn select_columns(alias: &str) -> String {
        format!(
            "{alias}.{ID}, {alias}.{TITLE}, {alias}.{SUB_TITLE}, {alias}.{VENUE}, {alias}.{START_DATE}, {alias}.{END_DATE}, {alias}.{URL}"
        )
    }
}
