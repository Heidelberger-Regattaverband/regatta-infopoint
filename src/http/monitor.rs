use bb8::State;
use serde::Serialize;

#[derive(Serialize)]
pub struct Monitor {
    db: Db,
}

impl Monitor {
    pub fn new(state: State) -> Self {
        Monitor {
            db: Db {
                connections: Connections {
                    current: state.connections,
                    idle: state.idle_connections,
                },
            },
        }
    }
}

#[derive(Serialize)]
struct Db {
    connections: Connections,
}

#[derive(Serialize)]
struct Connections {
    current: u32,
    idle: u32,
}
