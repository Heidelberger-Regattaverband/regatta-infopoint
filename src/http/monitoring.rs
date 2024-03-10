use bb8::State;
use serde::Serialize;
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};
use utoipa::ToSchema;

/// The monitor struct contains the state of the database.
#[derive(Serialize, ToSchema)]
pub(crate) struct Monitoring {
    /// The database state.
    db: Db,
    /// The system information.
    sys: SysInfo,
}

impl Monitoring {
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

        Monitoring {
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
                disks: Disks::new_with_refreshed_list().iter().map(Disk::from).collect(),
            },
        }
    }
}

/// The sysinfo struct contains the cpus and memory information.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SysInfo {
    /// The CPUs information.
    cpus: Vec<Cpu>,
    /// The memory information.
    mem: Memory,
    /// The disks information.
    disks: Vec<Disk>,
}

/// The cpu struct contains the usage, name and frequency of the CPU.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Cpu {
    /// The usage of the CPU.
    usage: f32,
    /// The name of the CPU.
    name: String,
    /// The frequency of the CPU.
    frequency: u64,
    /// The brand of the CPU.
    brand: String,
    /// The vendor of the CPU.
    vendor: String,
}

/// The memory struct contains the total, used, free and available memory of the system.
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

/// Converts a sysinfo::Cpu to a Cpu.
/// # Arguments
/// * `cpu` - The sysinfo::Cpu.
/// # Returns
/// `Cpu` - The Cpu.
impl From<&sysinfo::Cpu> for Cpu {
    fn from(cpu: &sysinfo::Cpu) -> Self {
        Cpu {
            usage: cpu.cpu_usage(),
            name: cpu.name().to_string(),
            frequency: cpu.frequency(),
            brand: cpu.brand().to_string(),
            vendor: cpu.vendor_id().to_string(),
        }
    }
}

/// The disk struct contains the name, mount point and file system of the disk.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Disk {
    /// The name of the disk.
    name: String,
    /// The mount point of the disk.
    mount_point: String,
    /// The file system of the disk.
    fs: String,
    /// The total space of the disk.
    total: u64,
    /// The available space of the disk.
    available: u64,
    /// The kind of the disk. It can be "HDD", "SSD" or "Unknown".
    kind: String,
}

/// Converts a sysinfo::Disk to a Disk.
impl From<&sysinfo::Disk> for Disk {
    /// # Arguments
    /// * `disk` - The sysinfo::Disk.
    /// # Returns
    /// `Disk` - The Disk.
    fn from(disk: &sysinfo::Disk) -> Self {
        Disk {
            name: disk.name().to_owned().into_string().unwrap_or_default(),
            mount_point: disk
                .mount_point()
                .to_owned()
                .into_os_string()
                .into_string()
                .unwrap_or_default(),
            fs: disk.file_system().to_owned().into_string().unwrap_or_default(),
            total: disk.total_space(),
            available: disk.available_space(),
            kind: disk.kind().to_owned().to_string(),
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
