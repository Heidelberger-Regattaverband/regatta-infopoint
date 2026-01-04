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

/// Represents a message with a severity level and text content.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// The unique ID of the message.
    pub id: i32,

    /// The severity level of the message. Higher values indicate more severe messages.
    pub severity: u8,

    /// The text of the message.
    pub text: String,

    /// The timestamp when the message was modified.
    pub modified_at: DateTime<Utc>,
}

impl Message {
    pub async fn query_messages_for_regatta(
        regatta_id: i32,
        client: &mut TiberiusClient,
    ) -> Result<Vec<Message>, DbError> {
        let sql = format!(
            "SELECT {ID}, {SEVERITY}, {TEXT}, {MODIFIED_AT} FROM HRV_Message WHERE {EVENT_ID} = @P1 ORDER BY {ID}"
        );
        let mut query = Query::new(&sql);
        query.bind(regatta_id);

        let results = utils::get_rows(query.query(client).await?)
            .await?
            .into_iter()
            .map(|row| Message::from(&row))
            .collect();
        Ok(results)
    }
}

impl From<&Row> for Message {
    fn from(row: &Row) -> Self {
        Message {
            id: row.get_column(ID),
            severity: row.get_column(SEVERITY),
            text: row.get_column(TEXT),
            modified_at: row.get_column(MODIFIED_AT),
        }
    }
}
