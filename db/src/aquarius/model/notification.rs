use super::get_row;
use super::get_rows;
use crate::error::DbError;
use crate::tiberius::RowColumn;
use crate::tiberius::TiberiusClient;
use crate::tiberius::TryRowColumn;
use ::chrono::DateTime;
use ::chrono::Utc;
use ::serde::{Deserialize, Serialize};
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

    /// The identifier of the associated event.
    pub event_id: i32,
}

/// Request structure for creating a new notification.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateNotificationRequest {
    /// The priority level of the notification. Higher values indicate more severe notifications.
    pub priority: Option<u8>,

    /// The title of the notification.
    pub title: String,

    /// The text of the notification.
    pub text: Option<String>,

    /// Whether the notification is visible. Defaults to true if not provided.
    #[serde(default = "default_visible")]
    pub visible: bool,
}

/// Request structure for updating an existing notification.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNotificationRequest {
    /// The priority level of the notification. Higher values indicate more severe notifications.
    pub priority: Option<u8>,

    /// The title of the notification.
    pub title: Option<String>,

    /// The text of the notification.
    pub text: Option<String>,

    /// Whether the notification is visible.
    pub visible: Option<bool>,
}

fn default_visible() -> bool {
    true
}

impl Notification {
    pub async fn query_visible_notifications_for_regatta(
        regatta_id: i32,
        client: &mut TiberiusClient,
    ) -> Result<Vec<Notification>, DbError> {
        let sql = format!(
            "SELECT {ID}, {PRIORITY}, {TITLE}, {TEXT}, {VISIBLE}, {MODIFIED_AT}, {EVENT_ID} FROM HRV_Notification \
            WHERE {EVENT_ID} = @P1 AND {VISIBLE} = 1 ORDER BY {ID}"
        );
        let mut query = Query::new(&sql);
        query.bind(regatta_id);

        let notifications = get_rows(query.query(client).await?)
            .await?
            .into_iter()
            .map(|row| Notification::from(&row))
            .collect();
        Ok(notifications)
    }

    pub async fn query_all_notifications_for_regatta(
        regatta_id: i32,
        client: &mut TiberiusClient,
    ) -> Result<Vec<Notification>, DbError> {
        let sql = format!(
            "SELECT {ID}, {PRIORITY}, {TITLE}, {TEXT}, {VISIBLE}, {MODIFIED_AT}, {EVENT_ID} FROM HRV_Notification \
            WHERE {EVENT_ID} = @P1 ORDER BY {ID} DESC"
        );
        let mut query = Query::new(&sql);
        query.bind(regatta_id);

        let notifications = get_rows(query.query(client).await?)
            .await?
            .into_iter()
            .map(|row| Notification::from(&row))
            .collect();
        Ok(notifications)
    }

    pub async fn create_notification(
        regatta_id: i32,
        request: &CreateNotificationRequest,
        client: &mut TiberiusClient,
    ) -> Result<Notification, DbError> {
        let now = Utc::now();
        let sql = format!(
            "INSERT INTO HRV_Notification ({EVENT_ID}, {PRIORITY}, {TITLE}, {TEXT}, {VISIBLE}, {MODIFIED_AT}) \
            OUTPUT INSERTED.{ID}, INSERTED.{PRIORITY}, INSERTED.{TITLE}, INSERTED.{TEXT}, INSERTED.{VISIBLE}, INSERTED.{MODIFIED_AT}, INSERTED.{EVENT_ID} \
            VALUES (@P1, @P2, @P3, @P4, @P5, @P6)"
        );
        let mut query = Query::new(&sql);
        query.bind(regatta_id);
        query.bind(request.priority);
        query.bind(&request.title);
        query.bind(request.text.as_deref());
        query.bind(request.visible);
        query.bind(now);

        let row = get_row(query.query(client).await?).await?;
        Ok(Notification::from(&row))
    }

    pub async fn update_notification(
        notification_id: i32,
        request: &UpdateNotificationRequest,
        client: &mut TiberiusClient,
    ) -> Result<Option<Notification>, DbError> {
        let now = Utc::now();

        // Each entry atomically pairs its SQL SET clause with its bound value,
        // preventing the ordering mismatch that two separate if-blocks could introduce.
        enum FieldParam<'a> {
            U8(u8),
            Bool(bool),
            Str(&'a str),
        }
        let mut fields: Vec<(String, FieldParam<'_>)> = Vec::new();
        let mut p = 1u8;

        if let Some(v) = request.priority {
            fields.push((format!("{PRIORITY} = @P{p}"), FieldParam::U8(v)));
            p += 1;
        }
        if let Some(v) = request.title.as_deref() {
            fields.push((format!("{TITLE} = @P{p}"), FieldParam::Str(v)));
            p += 1;
        }
        if let Some(v) = request.text.as_deref() {
            fields.push((format!("{TEXT} = @P{p}"), FieldParam::Str(v)));
            p += 1;
        }
        if let Some(v) = request.visible {
            fields.push((format!("{VISIBLE} = @P{p}"), FieldParam::Bool(v)));
            p += 1;
        }

        if fields.is_empty() {
            return Self::query_notification_by_id(notification_id, client).await;
        }

        let set_sql = fields.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>().join(", ");
        let id_param = p + 1;
        let sql = format!(
            "UPDATE HRV_Notification SET {set_sql}, {MODIFIED_AT} = @P{p} \
            OUTPUT INSERTED.{ID}, INSERTED.{PRIORITY}, INSERTED.{TITLE}, INSERTED.{TEXT}, INSERTED.{VISIBLE}, INSERTED.{MODIFIED_AT}, INSERTED.{EVENT_ID} \
            WHERE {ID} = @P{id_param}"
        );
        let mut query = Query::new(&sql);
        for (_, param) in &fields {
            match param {
                FieldParam::U8(v) => query.bind(*v),
                FieldParam::Bool(v) => query.bind(*v),
                FieldParam::Str(v) => query.bind(*v),
            }
        }
        query.bind(now);
        query.bind(notification_id);

        let rows = get_rows(query.query(client).await?).await?;
        Ok(rows.into_iter().map(|row| Notification::from(&row)).next())
    }

    pub async fn delete_notification(
        notification_id: i32,
        client: &mut TiberiusClient,
    ) -> Result<Option<Notification>, DbError> {
        let sql = format!(
            "DELETE FROM HRV_Notification \
            OUTPUT DELETED.{ID}, DELETED.{PRIORITY}, DELETED.{TITLE}, DELETED.{TEXT}, DELETED.{VISIBLE}, DELETED.{MODIFIED_AT}, DELETED.{EVENT_ID} \
            WHERE {ID} = @P1"
        );
        let mut query = Query::new(&sql);
        query.bind(notification_id);

        let result = query.query(client).await?;
        let rows = get_rows(result).await?;
        Ok(rows.into_iter().map(|row| Notification::from(&row)).next())
    }

    async fn query_notification_by_id(
        notification_id: i32,
        client: &mut TiberiusClient,
    ) -> Result<Option<Notification>, DbError> {
        let sql = format!(
            "SELECT {ID}, {PRIORITY}, {TITLE}, {TEXT}, {VISIBLE}, {MODIFIED_AT}, {EVENT_ID} FROM HRV_Notification \
            WHERE {ID} = @P1"
        );
        let mut query = Query::new(&sql);
        query.bind(notification_id);

        let rows = get_rows(query.query(client).await?).await?;
        Ok(rows.into_iter().map(|row| Notification::from(&row)).next())
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
            event_id: row.get_column(EVENT_ID),
        }
    }
}
