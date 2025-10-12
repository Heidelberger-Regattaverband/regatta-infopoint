use crate::{
    aquarius::model::utils,
    error::DbError,
    tiberius::{RowColumn, TiberiusPool, TryRowColumn},
};
use serde::Serialize;
use tiberius::{Query, Row, time::chrono::NaiveDateTime};
use utoipa::ToSchema;

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
    pub async fn query_active_regatta(pool: &TiberiusPool) -> Result<Regatta, DbError> {
        let mut client = pool.get().await?;
        let stream = Query::new("SELECT TOP 1 e.* FROM Event e ORDER BY e.Event_StartDate DESC, e.Event_ID DESC")
            .query(&mut client)
            .await?;
        Ok(Regatta::from(&utils::get_row(stream).await?))
    }

    pub async fn query_by_id(regatta_id: i32, pool: &TiberiusPool) -> Result<Option<Regatta>, DbError> {
        let mut query = Query::new("SELECT * FROM Event WHERE Event_ID = @P1");
        query.bind(regatta_id);

        let mut client = pool.get().await?;

        let row = utils::try_get_row(query.query(&mut client).await?).await?;
        if let Some(row) = row {
            Ok(Some(Regatta::from(&row)))
        } else {
            Ok(None)
        }
    }
}
