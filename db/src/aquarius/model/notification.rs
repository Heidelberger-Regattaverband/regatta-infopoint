use crate::aquarius::model::utils;
use crate::error::DbError;
use crate::tiberius::RowColumn;
use crate::tiberius::TiberiusClient;
use ::chrono::DateTime;
use ::chrono::Utc;
use ::serde::Serialize;
use ::tiberius::Query;
use ::tiberius::Row;
use ::utoipa::ToSchema;

const ID: &str = "id";
const EVENT_ID: &str = "eventId";
const SEVERITY: &str = "severity";
const TEXT: &str = "text";
const MODIFIED_AT: &str = "modifiedAt";

/// Represents a notification with a severity level and text content.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    /// The unique ID of the notification.
    pub id: i32,

    /// The severity level of the notification. Higher values indicate more severe notifications.
    pub severity: u8,

    /// The text of the notification.
    pub text: String,

    /// The timestamp when the notification was modified.
    pub modified_at: DateTime<Utc>,
}

impl Notification {
    pub async fn query_notifications_for_regatta(
        regatta_id: i32,
        client: &mut TiberiusClient,
    ) -> Result<Vec<Notification>, DbError> {
        let sql = format!(
            "SELECT {ID}, {SEVERITY}, {TEXT}, {MODIFIED_AT} FROM HRV_Message WHERE {EVENT_ID} = @P1 ORDER BY {ID}"
        );
        let mut query = Query::new(&sql);
        query.bind(regatta_id);

        let results = utils::get_rows(query.query(client).await?)
            .await?
            .into_iter()
            .map(|row| Notification::from(&row))
            .collect();
        Ok(results)
    }
}

impl From<&Row> for Notification {
    fn from(row: &Row) -> Self {
        Notification {
            id: row.get_column(ID),
            severity: row.get_column(SEVERITY),
            text: row.get_column(TEXT),
            modified_at: row.get_column(MODIFIED_AT),
        }
    }
}
