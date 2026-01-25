use crate::aquarius::model::utils;
use crate::error::DbError;
use crate::tiberius::RowColumn;
use crate::tiberius::TiberiusClient;
use crate::tiberius::TryRowColumn;
use ::chrono::DateTime;
use ::chrono::Utc;
use ::serde::{Deserialize, Serialize};
use ::tiberius::Query;
use ::tiberius::Row;
use ::tracing::info;
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

/// Request structure for creating a new notification.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateNotificationRequest {
    /// The priority level of the notification. Higher values indicate more severe notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,

    /// The title of the notification.
    pub title: String,

    /// The text of the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,

    /// The title of the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The text of the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Whether the notification is visible.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
}

fn default_visible() -> bool {
    true
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

    pub async fn create_notification(
        regatta_id: i32,
        request: &CreateNotificationRequest,
        client: &mut TiberiusClient,
    ) -> Result<Notification, DbError> {
        let now = Utc::now();
        let sql = format!(
            "INSERT INTO HRV_Notification ({EVENT_ID}, {PRIORITY}, {TITLE}, {TEXT}, {VISIBLE}, {MODIFIED_AT}) \
            OUTPUT INSERTED.{ID}, INSERTED.{PRIORITY}, INSERTED.{TITLE}, INSERTED.{TEXT}, INSERTED.{VISIBLE}, INSERTED.{MODIFIED_AT} \
            VALUES (@P1, @P2, @P3, @P4, @P5, @P6)"
        );
        let mut query = Query::new(&sql);
        query.bind(regatta_id);
        query.bind(request.priority);
        query.bind(&request.title);
        query.bind(request.text.as_deref());
        query.bind(request.visible);
        query.bind(now);

        let row = utils::get_row(query.query(client).await?).await?;
        Ok(Notification::from(&row))
    }

    pub async fn update_notification(
        notification_id: i32,
        request: &UpdateNotificationRequest,
        client: &mut TiberiusClient,
    ) -> Result<Option<Notification>, DbError> {
        let now = Utc::now();
        // Build dynamic SQL based on provided fields
        let mut set_clauses = Vec::new();
        let mut param_count = 1;

        if request.priority.is_some() {
            set_clauses.push(format!("{PRIORITY} = @P{param_count}"));
            param_count += 1;
        }
        if request.title.is_some() {
            set_clauses.push(format!("{TITLE} = @P{param_count}"));
            param_count += 1;
        }
        if request.text.is_some() {
            set_clauses.push(format!("{TEXT} = @P{param_count}"));
            param_count += 1;
        }
        if request.visible.is_some() {
            set_clauses.push(format!("{VISIBLE} = @P{param_count}"));
            param_count += 1;
        }
        set_clauses.push(format!("{MODIFIED_AT} = @P{param_count}"));

        if set_clauses.is_empty() {
            return Self::query_notification_by_id(notification_id, client).await;
        }

        let sql = format!(
            "UPDATE HRV_Notification SET {} \
            OUTPUT INSERTED.{ID}, INSERTED.{PRIORITY}, INSERTED.{TITLE}, INSERTED.{TEXT}, INSERTED.{VISIBLE}, INSERTED.{MODIFIED_AT} \
            WHERE {ID} = @P{}",
            set_clauses.join(", "),
            param_count + 1
        );
        let mut query = Query::new(&sql);
        // Bind parameters in the same order as set_clauses
        if let Some(priority) = request.priority {
            query.bind(priority);
        }
        if let Some(ref title) = request.title {
            query.bind(title);
        }
        if let Some(ref text) = request.text {
            query.bind(text);
        }
        if let Some(visible) = request.visible {
            query.bind(visible);
        }
        query.bind(now);
        query.bind(notification_id);

        let rows = utils::get_rows(query.query(client).await?).await?;
        Ok(rows.into_iter().map(|row| Notification::from(&row)).next())
    }

    pub async fn delete_notification(notification_id: i32, client: &mut TiberiusClient) -> Result<bool, DbError> {
        let sql = format!("DELETE FROM HRV_Notification WHERE {ID} = @P1");
        let mut query = Query::new(&sql);
        query.bind(notification_id);

        let result = query.execute(client).await?;
        info!(notification_id, rows_affected = ?result.rows_affected(), "Deleted notification");
        Ok(!result.rows_affected().is_empty())
    }

    async fn query_notification_by_id(
        notification_id: i32,
        client: &mut TiberiusClient,
    ) -> Result<Option<Notification>, DbError> {
        let sql = format!(
            "SELECT {ID}, {PRIORITY}, {TITLE}, {TEXT}, {VISIBLE}, {MODIFIED_AT} FROM HRV_Notification \
            WHERE {ID} = @P1"
        );
        let mut query = Query::new(&sql);
        query.bind(notification_id);

        let rows = utils::get_rows(query.query(client).await?).await?;
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
        }
    }
}
