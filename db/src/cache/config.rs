use ::std::time::Duration;

// Constants based on typical regatta sizes and usage patterns
const MAX_REGATTAS_COUNT: usize = 30;
const MAX_RACES_COUNT: usize = 2000;
const MAX_HEATS_COUNT: usize = 3500;
const MAX_CLUBS_COUNT: usize = 1000;
const MAX_NOTIFICATIONS_COUNT: usize = 100;

/// Configuration for all caches in the system with optimized defaults
#[derive(Debug)]
pub(super) struct CachesConfig {
    pub(super) regattas: CacheConfig,
    pub(super) races: CacheConfig,
    pub(super) heats: CacheConfig,
    pub(super) clubs: CacheConfig,
    pub(super) athletes: CacheConfig,
    pub(super) notifications: CacheConfig,
}

impl CachesConfig {
    /// Creates cache configurations with optimized settings for each data type
    ///
    /// # Arguments
    /// * `base_ttl` - Base time-to-live applied to all caches
    ///
    /// # Returns
    /// Configured cache settings optimized for regatta data patterns
    pub(crate) fn new(base_ttl: Duration) -> Self {
        Self {
            regattas: CacheConfig {
                max_entries: MAX_REGATTAS_COUNT,
                ttl: base_ttl,
            },
            races: CacheConfig {
                max_entries: MAX_RACES_COUNT,
                ttl: base_ttl,
            },
            heats: CacheConfig {
                max_entries: MAX_HEATS_COUNT,
                ttl: base_ttl,
            },
            clubs: CacheConfig {
                max_entries: MAX_CLUBS_COUNT,
                ttl: base_ttl,
            },
            athletes: CacheConfig {
                max_entries: MAX_RACES_COUNT,
                ttl: base_ttl,
            },
            notifications: CacheConfig {
                max_entries: MAX_NOTIFICATIONS_COUNT,
                ttl: base_ttl,
            },
        }
    }
}

/// Cache configuration with builder pattern support
#[derive(Debug, Clone)]
pub(super) struct CacheConfig {
    /// Maximum number of entries in the cache
    pub(super) max_entries: usize,
    /// Time-to-live for cache entries
    pub(super) ttl: Duration,
}
