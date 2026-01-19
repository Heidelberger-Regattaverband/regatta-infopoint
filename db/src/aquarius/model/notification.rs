use crate::aquarius::model::utils;
use crate::error::DbError;
use crate::tiberius::RowColumn;
use crate::tiberius::TiberiusClient;
use crate::tiberius::TryRowColumn;
use ::chrono::DateTime;
use ::chrono::Utc;
use ::serde::Serialize;
use ::tiberius::Query;
use ::tiberius::Row;
use ::utoipa::ToSchema;

const ID: &str = "id";
const EVENT_ID: &str = "eventId";
const PRIORITY: &str = "priority";
const TEXT: &str = "text";
const TITLE: &str = "title";
const VISIBLE: &str = "visible";
const MODIFIED_AT: &str = "modifiedAt";

/// Represents a notification with a priority level and text content.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    /// The unique identifier of the notification.
    pub id: i32,

    /// The priority level of the notification. Higher values indicate more severe notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<u8>,

    /// The title of the notification.
    title: String,

    /// The text of the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,

    /// Whether the notification is visible.
    visible: bool,

    /// The timestamp when the notification was modified.
    pub modified_at: DateTime<Utc>,
}

impl Notification {
    pub async fn query_notifications_for_regatta(
        regatta_id: i32,
        client: &mut TiberiusClient,
    ) -> Result<Vec<Notification>, DbError> {
        let sql = format!(
            "SELECT {ID}, {PRIORITY}, {TITLE}, {TEXT}, {VISIBLE}, {MODIFIED_AT} FROM HRV_Notification \
            WHERE {EVENT_ID} = @P1 AND {VISIBLE} = 1 ORDER BY {ID}"
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
            priority: row.try_get_column(PRIORITY),
            title: row.get_column(TITLE),
            text: row.try_get_column(TEXT),
            visible: row.get_column(VISIBLE),
            modified_at: row.get_column(MODIFIED_AT),
        }
    }
}
