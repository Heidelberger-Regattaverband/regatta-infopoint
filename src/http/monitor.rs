use bb8::State;
use serde::Serialize;

#[derive(Serialize)]
pub struct Monitor {
    db: Db,
}

impl Monitor {
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Db {
    connections: Connections,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Connections {
    current: u32,
    idle: u32,
    created: u32,
}
