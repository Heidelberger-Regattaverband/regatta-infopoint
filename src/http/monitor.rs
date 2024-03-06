use bb8::State;
use serde::Serialize;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use utoipa::ToSchema;

/// The monitor struct contains the state of the database.
#[derive(Serialize, ToSchema)]
pub(crate) struct Monitor {
    /// The database state.
    db: Db,
    /// The system information.
    sys: SysInfo,
}

impl Monitor {
    /// Creates a new monitor with the given state and created connections.
    /// # Arguments
    /// * `state` - The state of the database.
    /// * `created` - The number of created connections.
    /// # Returns
    /// `Monitor` - The monitor.
    pub(crate) fn new(state: State, created: u32) -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        // Wait a bit because CPU usage is based on diff.
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_cpu();

        Monitor {
            db: Db {
                connections: Connections {
                    current: state.connections,
                    idle: state.idle_connections,
                    created,
                },
            },
            sys: SysInfo {
                cpus: sys.cpus().iter().map(Cpu::from).collect(),
                mem: Memory {
                    free: sys.free_memory(),
                    used: sys.used_memory(),
                    available: sys.available_memory(),
                    total: sys.total_memory(),
                },
            },
        }
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SysInfo {
    /// The CPUs information.
    cpus: Vec<Cpu>,
    /// The memory information.
    mem: Memory,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Cpu {
    /// The usage of the CPU.
    usage: f32,
    /// The name of the CPU.
    name: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Memory {
    /// The total memory of the system.
    total: u64,
    /// The used memory of the system.
    used: u64,
    /// The free memory of the system.
    free: u64,
    /// The available memory of the system.
    available: u64,
}

impl From<&sysinfo::Cpu> for Cpu {
    fn from(cpu: &sysinfo::Cpu) -> Self {
        Cpu {
            usage: cpu.cpu_usage(),
            name: cpu.name().to_string(),
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
