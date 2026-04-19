use crate::aquarius::model::Athlete;
use crate::aquarius::model::Club;
use crate::aquarius::model::Entry;
use crate::aquarius::model::Filters;
use crate::aquarius::model::Heat;
use crate::aquarius::model::Notification;
use crate::aquarius::model::Race;
use crate::aquarius::model::Regatta;
use crate::aquarius::model::Schedule;
use crate::cache::heap_size::HeapSize;
use ::std::mem;

/// Trait for estimating the memory cost of a cached value.
///
/// Used by the cache to assign a meaningful cost for admission and eviction policies.
/// Implementations should estimate the total memory footprint including heap allocations.
pub(crate) trait CacheCost {
    /// Returns the estimated memory cost in bytes.
    fn cache_cost(&self) -> i64;
}

/// Blanket implementation for `Vec<T>` that accounts for heap-allocated elements.
/// The cost includes the `Vec` stack overhead plus the estimated cost of each element.
impl<T: CacheCost> CacheCost for Vec<T> {
    fn cache_cost(&self) -> i64 {
        let stack = mem::size_of::<Vec<T>>() as i64;
        let heap: i64 = self.iter().map(|item| item.cache_cost()).sum();
        stack + heap
    }
}

impl CacheCost for Regatta {
    fn cache_cost(&self) -> i64 {
        mem::size_of::<Self>() as i64 + self.heap_size()
    }
}

impl CacheCost for Race {
    fn cache_cost(&self) -> i64 {
        mem::size_of::<Self>() as i64 + self.heap_size()
    }
}

impl CacheCost for Heat {
    fn cache_cost(&self) -> i64 {
        mem::size_of::<Self>() as i64 + self.heap_size()
    }
}

impl CacheCost for Club {
    fn cache_cost(&self) -> i64 {
        mem::size_of::<Self>() as i64 + self.heap_size()
    }
}

impl CacheCost for Athlete {
    fn cache_cost(&self) -> i64 {
        mem::size_of::<Self>() as i64 + self.heap_size()
    }
}

impl CacheCost for Entry {
    fn cache_cost(&self) -> i64 {
        mem::size_of::<Self>() as i64 + self.heap_size()
    }
}

impl CacheCost for Notification {
    fn cache_cost(&self) -> i64 {
        mem::size_of::<Self>() as i64 + self.heap_size()
    }
}

impl CacheCost for Filters {
    fn cache_cost(&self) -> i64 {
        mem::size_of::<Self>() as i64 + self.heap_size()
    }
}

impl CacheCost for Schedule {
    fn cache_cost(&self) -> i64 {
        mem::size_of::<Self>() as i64 + self.heap_size()
    }
}
