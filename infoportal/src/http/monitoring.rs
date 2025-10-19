use crate::peak_alloc::PeakAlloc;
use db::{cache::CacheStats, tiberius::TiberiusPool};
use prometheus::Registry;
use serde::Serialize;
use serde_json::{Map, Number, Value};
use std::time::Duration;
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};
use utoipa::ToSchema;

/// The monitoring struct contains the database state, system information and metrics.
#[derive(Serialize, ToSchema)]
pub(crate) struct Monitoring {
    /// The database state.
    db: Db,
    /// The system information.
    sys: SysInfo,
    /// The metrics of the system.
    metrics: Map<String, Value>,
    /// The application information.
    app: AppInfo,
}

impl Monitoring {
    /// Creates a new monitoring struct.
    /// # Arguments
    /// * `pool` - The tiberius pool.
    /// * `registry` - The prometheus registry.
    /// # Returns
    /// `Monitoring` - The monitoring struct.
    pub(crate) fn new(pool: &TiberiusPool, registry: &Registry, caches: &CacheStats) -> Self {
        let sys = get_system();
        let metrics = get_metrics(registry);
        let state = pool.state();
        let stats = state.statistics;
        Monitoring {
            db: Db {
                connections: Connections {
                    total: state.connections,
                    idle: state.idle_connections,
                    used: state.connections - state.idle_connections,
                    created: stats.connections_created,
                    closed_idle_timeout: stats.connections_closed_idle_timeout,
                    closed_max_lifetime: stats.connections_closed_max_lifetime,
                    closed_error: stats.connections_closed_broken + stats.connections_closed_invalid,
                },
                caches: Caches::from(caches.clone()),
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
                uptime: uptime_lib::get().unwrap_or_default(),
            },
            metrics,
            app: AppInfo {
                mem_current: PeakAlloc.current_usage(),
                mem_max: PeakAlloc.peak_usage(),
            },
        }
    }
}

fn get_system() -> System {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );
    // Wait a bit because CPU usage is based on diff.
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_all();
    sys
}

fn get_metrics(registry: &Registry) -> Map<String, Value> {
    let mut all_metrics = Map::new();
    registry.gather().iter().for_each(|f| {
        let mut family_entries = Vec::new();
        f.get_metric().iter().for_each(|m| {
            let mut labels: Map<String, Value> = Map::new();
            m.get_label().iter().for_each(|l| {
                labels.insert(l.name().to_string(), Value::String(l.value().to_string()));
            });
            labels.insert(
                "counter".to_string(),
                Value::Number(Number::from_f64(m.get_counter().get_value()).unwrap()),
            );
            labels.insert(
                "gauge".to_string(),
                Value::Number(Number::from_f64(m.get_gauge().get_value()).unwrap()),
            );
            family_entries.push(Value::Object(labels));
        });
        all_metrics.insert(f.name().to_string(), Value::Array(family_entries));
    });
    all_metrics
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
    /// The system uptime
    uptime: Duration,
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
    /// The cache statistics.
    caches: Caches,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Caches {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
    pub hit_rate: f64,
}

impl From<CacheStats> for Caches {
    fn from(stats: CacheStats) -> Self {
        Caches {
            hits: stats.hits,
            misses: stats.misses,
            entries: stats.entries,
            hit_rate: stats.hit_rate,
        }
    }
}

/// The connections struct contains the current, idle and created connections.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Connections {
    /// The total number of connections.
    total: u32,
    /// The number of connections that are currently not in use.
    idle: u32,
    /// The number of connections that are currently activly being used.
    used: u32,
    /// The number of connections that have been created.
    created: u64,
    /// The number of connections that have been closed due to a timeout.
    closed_idle_timeout: u64,
    /// The number of connections that have been closed due to max lifetime.
    closed_max_lifetime: u64,
    /// The number of connections that have been closed due to an error.
    closed_error: u64,
}

/// The AppInfo struct contains the current and peak memory usage.
#[derive(Serialize, ToSchema)]
pub(crate) struct AppInfo {
    /// The current memory usage.
    mem_current: usize,
    /// The peak memory usage.
    mem_max: usize,
}
