use bb8::State;
use serde::Serialize;

#[derive(Serialize)]
pub struct Monitor {
    #[serde(rename = "dbConnections")]
    db_connections: DBConnections,
}

impl Monitor {
    pub fn new(state: State) -> Self {
        Monitor {
            db_connections: DBConnections {
                current: state.connections,
                idle: state.idle_connections,
            },
        }
    }
}

#[derive(Serialize)]
struct DBConnections {
    current: u32,
    idle: u32,
}
