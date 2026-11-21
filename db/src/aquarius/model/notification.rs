use super::get_row;
use super::get_rows;
use crate::error::DbError;
use crate::tiberius::RowColumn;
use crate::tiberius::TiberiusClient;
use crate::tiberius::TryRowColumn;
use ::chrono::DateTime;
use ::chrono::Utc;
use ::serde::Deserialize;
use ::serde::Serialize;
use ::tiberius::ColumnData;
use ::tiberius::IntoSql;
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

/// An omitted field is unchanged; a supplied field replaces its value.
/// Use `UpdateField<Option<T>>` when null is a valid replacement.
#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum UpdateField<T> {
    /// Supplied by the field default, never by an explicit JSON value.
    #[default]
    #[serde(skip)]
    Unchanged,

    /// Explicitly set the field to the given value. Can be null if the type allows it.
    Set(T),
}

/// Partial update: omitted fields are unchanged. Explicit null clears priority
/// and text, but is rejected for the non-nullable title and visible fields.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNotificationRequest {
    /// Omit to keep the priority, use null to clear it, or supply a new value.
    #[serde(default)]
    #[schema(value_type = Option<u8>)]
    pub priority: UpdateField<Option<u8>>,

    /// Omit to keep the title. A supplied title must be non-null and non-empty.
    #[serde(default)]
    #[schema(value_type = String, nullable = false)]
    pub title: UpdateField<String>,

    /// Omit to keep the text, use null to clear it, or supply a new value.
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub text: UpdateField<Option<String>>,

    /// Omit to keep visibility. A supplied value must be a non-null boolean.
    #[serde(default)]
    #[schema(value_type = bool, nullable = false)]
    pub visible: UpdateField<bool>,
}

impl UpdateNotificationRequest {
    /// Pair each SET clause with its typed SQL value, including typed NULLs.
    /// None represents a no-op request and avoids changing modifiedAt.
    fn update_statement<'a>(&'a self) -> Option<(String, Vec<ColumnData<'a>>)> {
        let mut fields = Vec::new();
        let mut push = |column: &str, value: ColumnData<'a>| {
            fields.push((format!("{column} = @P{}", fields.len() + 1), value));
        };
        if let UpdateField::Set(value) = &self.priority {
            push(PRIORITY, (*value).into_sql());
        }
        if let UpdateField::Set(value) = &self.title {
            push(TITLE, value.as_str().into_sql());
        }
        if let UpdateField::Set(value) = &self.text {
            push(TEXT, value.as_deref().into_sql());
        }
        if let UpdateField::Set(value) = &self.visible {
            push(VISIBLE, (*value).into_sql());
        }
        if fields.is_empty() {
            return None;
        }
        let set_sql = fields
            .iter()
            .map(|(clause, _)| clause.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let modified_param = fields.len() + 1;
        let id_param = modified_param + 1;
        let sql = format!(
            "UPDATE HRV_Notification SET {set_sql}, {MODIFIED_AT} = @P{modified_param} \
             OUTPUT INSERTED.{ID}, INSERTED.{PRIORITY}, INSERTED.{TITLE}, INSERTED.{TEXT}, INSERTED.{VISIBLE}, INSERTED.{MODIFIED_AT}, INSERTED.{EVENT_ID} \
             WHERE {ID} = @P{id_param}"
        );
        Some((sql, fields.into_iter().map(|(_, value)| value).collect()))
    }
}

fn bind_update_query(
    sql: String,
    values: Vec<ColumnData<'_>>,
    notification_id: i32,
    modified_at: DateTime<Utc>,
) -> Query<'_> {
    let mut query = Query::new(sql);
    query.bind_iter(values);
    query.bind(modified_at);
    query.bind(notification_id);
    query
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
        let Some((sql, values)) = request.update_statement() else {
            return Self::query_notification_by_id(notification_id, client).await;
        };
        let query = bind_update_query(sql, values, notification_id, Utc::now());

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

#[cfg(test)]
mod tests {
    use super::*;
    use ::serde_json::{Map, Value, json};
    use ::tiberius::ColumnType;

    struct UpdateCase {
        body: Value,
        columns: Vec<&'static str>,
        values: Vec<ColumnData<'static>>,
    }

    fn update_cases() -> Vec<UpdateCase> {
        let priority = [
            (None, None),
            (Some(Value::Null), Some(ColumnData::U8(None))),
            (Some(json!(0)), Some(ColumnData::U8(Some(0)))),
            (Some(json!(255)), Some(ColumnData::U8(Some(255)))),
        ];
        let text = [
            (None, None),
            (Some(Value::Null), Some(ColumnData::String(None))),
            (Some(json!("")), Some(ColumnData::String(Some("".into())))),
            (
                Some(json!("Updated text")),
                Some(ColumnData::String(Some("Updated text".into()))),
            ),
        ];
        let title = [
            (None, None),
            (
                Some(json!("Updated title")),
                Some(ColumnData::String(Some("Updated title".into()))),
            ),
        ];
        let visible = [(None, None), (Some(json!(false)), Some(ColumnData::Bit(Some(false))))];
        let mut cases = Vec::new();
        for p in &priority {
            for t in &title {
                for x in &text {
                    for v in &visible {
                        let mut body = Map::new();
                        let mut columns = Vec::new();
                        let mut values = Vec::new();
                        for (column, (json, sql)) in [(PRIORITY, p), (TITLE, t), (TEXT, x), (VISIBLE, v)] {
                            if let Some(json) = json {
                                body.insert(column.to_owned(), json.clone());
                                columns.push(column);
                                values.push(sql.clone().unwrap());
                            }
                        }
                        cases.push(UpdateCase {
                            body: Value::Object(body),
                            columns,
                            values,
                        });
                    }
                }
            }
        }
        cases
    }

    #[test]
    fn nullable_updates_distinguish_omission_null_and_value() {
        let omitted: UpdateNotificationRequest = serde_json::from_value(json!({})).unwrap();
        assert_eq!(omitted.priority, UpdateField::Unchanged);
        assert_eq!(omitted.text, UpdateField::Unchanged);
        assert_eq!(omitted.title, UpdateField::Unchanged);
        assert_eq!(omitted.visible, UpdateField::Unchanged);
        assert!(omitted.update_statement().is_none());

        let cleared: UpdateNotificationRequest =
            serde_json::from_value(json!({"priority": null, "text": null})).unwrap();
        assert_eq!(cleared.priority, UpdateField::Set(None));
        assert_eq!(cleared.text, UpdateField::Set(None));

        let set: UpdateNotificationRequest =
            serde_json::from_value(json!({"priority": 0, "text": "", "visible": false})).unwrap();
        assert_eq!(set.priority, UpdateField::Set(Some(0)));
        assert_eq!(set.text, UpdateField::Set(Some(String::new())));
        assert_eq!(set.visible, UpdateField::Set(false));
    }

    #[test]
    fn updates_reject_null_for_non_nullable_fields_and_invalid_types() {
        for body in [
            json!({"title": null}),
            json!({"visible": null}),
            json!({"title": 1}),
            json!({"visible": "false"}),
            json!({"text": false}),
            json!({"priority": "1"}),
            json!({"priority": -1}),
            json!({"priority": 256}),
        ] {
            assert!(
                serde_json::from_value::<UpdateNotificationRequest>(body.clone()).is_err(),
                "accepted {body}"
            );
        }
    }

    #[test]
    fn all_update_combinations_preserve_sql_parameter_order_and_typed_nulls() {
        let cases = update_cases();
        assert_eq!(cases.len(), 64);
        for case in cases {
            let request: UpdateNotificationRequest = serde_json::from_value(case.body.clone()).unwrap();
            let statement = request.update_statement();
            if case.columns.is_empty() {
                assert!(statement.is_none());
                continue;
            }
            let (sql, values) = statement.unwrap();
            let clauses = case
                .columns
                .iter()
                .enumerate()
                .map(|(index, column)| format!("{column} = @P{}", index + 1))
                .collect::<Vec<_>>()
                .join(", ");
            assert!(
                sql.starts_with(&format!(
                    "UPDATE HRV_Notification SET {clauses}, modifiedAt = @P{} ",
                    case.values.len() + 1
                )),
                "{sql}"
            );
            assert!(
                sql.ends_with(&format!("WHERE id = @P{}", case.values.len() + 2)),
                "{sql}"
            );
            assert_eq!(values, case.values, "{}", case.body);
            let query = bind_update_query(sql, values, 17, Utc::now());
            assert_eq!(query.param_count(), case.columns.len() + 2);
        }
    }

    #[test]
    fn update_schema_matches_optional_and_nullable_fields() {
        let schema = <UpdateNotificationRequest as utoipa::PartialSchema>::schema();
        let schema = serde_json::to_value(schema).unwrap();
        let has_type = |field: &str, expected: &str| {
            let value = &schema["properties"][field]["type"];
            value.as_str() == Some(expected)
                || value
                    .as_array()
                    .is_some_and(|types| types.iter().any(|value| value.as_str() == Some(expected)))
        };
        assert!(has_type(PRIORITY, "integer"));
        assert!(has_type(PRIORITY, "null"));
        assert!(has_type(TEXT, "string"));
        assert!(has_type(TEXT, "null"));
        assert!(has_type(TITLE, "string"));
        assert!(!has_type(TITLE, "null"));
        assert!(has_type(VISIBLE, "boolean"));
        assert!(!has_type(VISIBLE, "null"));
        assert!(
            schema
                .get("required")
                .and_then(Value::as_array)
                .is_none_or(Vec::is_empty)
        );
    }

    #[test]
    fn null_output_fields_decode_and_are_absent_from_the_response() {
        let modified_at = DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let row = Row::builder()
            .column(ID, ColumnType::Int4, 17_i32.into_sql())
            .column(EVENT_ID, ColumnType::Int4, 19_i32.into_sql())
            .column(PRIORITY, ColumnType::Int1, ColumnData::U8(None))
            .column(TITLE, ColumnType::NVarchar, "Original title".into_sql())
            .column(TEXT, ColumnType::NVarchar, ColumnData::String(None))
            .column(VISIBLE, ColumnType::Bit, true.into_sql())
            .column(MODIFIED_AT, ColumnType::Datetime2, modified_at.naive_utc().into_sql())
            .build();
        let notification = Notification::from(&row);
        assert_eq!(notification.priority, None);
        assert_eq!(notification.text, None);
        assert_eq!(notification.title, "Original title");
        assert!(notification.visible);
        assert_eq!(notification.modified_at, modified_at);
        let response = serde_json::to_value(notification).unwrap();
        assert!(response.get(PRIORITY).is_none());
        assert!(response.get(TEXT).is_none());
    }

    /// Explicit opt-in: executes the production SQL and bindings against only
    /// a connection-local temporary table. No Aquarius tables are changed.
    #[tokio::test]
    #[ignore = "Requires an approved NOTIFICATION_TEST_CONNECTION_STRING; writes only to a temporary table"]
    async fn notification_update_sql_roundtrip() {
        let config = tiberius::Config::from_ado_string(
            &std::env::var("NOTIFICATION_TEST_CONNECTION_STRING").expect("Set an approved test connection string"),
        )
        .unwrap();
        let mut client = crate::tiberius::create_client(&config).await.unwrap();
        client.simple_query(
            "CREATE TABLE #NotificationUpdateFixture (id int NOT NULL PRIMARY KEY, eventId int NOT NULL, priority tinyint NULL, title nvarchar(200) NOT NULL, text nvarchar(max) NULL, visible bit NOT NULL, modifiedAt datetime2(7) NOT NULL)"
        ).await.unwrap().into_results().await.unwrap();
        let now = DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        for case in update_cases() {
            client.simple_query(
                "DELETE FROM #NotificationUpdateFixture; INSERT INTO #NotificationUpdateFixture VALUES (17, 19, 2, N'Original title', N'Original text', 1, '2000-01-01T00:00:00')"
            ).await.unwrap().into_results().await.unwrap();
            let request: UpdateNotificationRequest = serde_json::from_value(case.body).unwrap();
            let expected_priority = match &request.priority {
                UpdateField::Unchanged => Some(2),
                UpdateField::Set(value) => *value,
            };
            let expected_text = match &request.text {
                UpdateField::Unchanged => Some("Original text"),
                UpdateField::Set(value) => value.as_deref(),
            };
            let expected_title = match &request.title {
                UpdateField::Unchanged => "Original title",
                UpdateField::Set(value) => value.as_str(),
            };
            let expected_visible = match &request.visible {
                UpdateField::Unchanged => true,
                UpdateField::Set(value) => *value,
            };
            if let Some((sql, values)) = request.update_statement() {
                let query = bind_update_query(
                    sql.replace("HRV_Notification", "#NotificationUpdateFixture"),
                    values,
                    17,
                    now,
                );
                let row = get_row(query.query(&mut client).await.unwrap()).await.unwrap();
                let response = Notification::from(&row);
                assert_eq!(response.priority, expected_priority);
                assert_eq!(response.text.as_deref(), expected_text);
                assert_eq!(response.title, expected_title);
                assert_eq!(response.visible, expected_visible);
                assert_eq!(response.modified_at, now);
            }
            // Verify stored state, not only OUTPUT rows, including the empty-body no-op.
            let row = client
                .simple_query("SELECT * FROM #NotificationUpdateFixture WHERE id = 17")
                .await
                .unwrap()
                .into_row()
                .await
                .unwrap()
                .unwrap();
            let stored = Notification::from(&row);
            assert_eq!(stored.priority, expected_priority);
            assert_eq!(stored.text.as_deref(), expected_text);
            assert_eq!(stored.title, expected_title);
            assert_eq!(stored.visible, expected_visible);
            assert_eq!(
                stored.modified_at,
                if case.columns.is_empty() {
                    DateTime::from_timestamp(946_684_800, 0).unwrap()
                } else {
                    now
                }
            );
        }
    }
}
