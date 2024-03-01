use bb8::State;
use serde::Serialize;
use utoipa::ToSchema;

/// The monitor struct contains the state of the database.
#[derive(Serialize, ToSchema)]
pub(crate) struct Monitor {
    /// The database state.
    db: Db,
}

impl Monitor {
    /// Creates a new monitor with the given state and created connections.
    /// # Arguments
    /// * `state` - The state of the database.
    /// * `created` - The number of created connections.
    /// # Returns
    /// `Monitor` - The monitor.
    pub fn new(state: State, created: u32) -> Self {
        Monitor {
            db: Db {
                connections: Connections {
                    current: state.connections,
                    idle: state.idle_connections,
                    created,
                },
            },
        }
    }
}

/// The Db struct contains the connections of the database.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Db {
    /// The connections of the database.
    connections: Connections,
}

/// The connections struct contains the current, idle and created connections.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Connections {
    /// The current connections are the number of connections that are currently in use.
    current: u32,
    /// The idle connections are the number of connections that are currently not in use.
    idle: u32,
    /// The created connections are the number of connections that have been created.
    created: u32,
}
