use crate::cache::cost::CacheCost;

/// Helper trait for estimating heap-allocated memory of individual fields.
/// This is used internally by `CacheCost` implementations to account for
/// `String`, `Option<String>`, `Option<Vec<T>>`, and other heap-owning types.
pub(crate) trait HeapSize {
    /// Returns the estimated heap memory usage in bytes.
    fn heap_size(&self) -> i64;
}

impl HeapSize for String {
    fn heap_size(&self) -> i64 {
        self.capacity() as i64
    }
}

impl<T: HeapSize> HeapSize for Option<T> {
    fn heap_size(&self) -> i64 {
        match self {
            Some(v) => v.heap_size(),
            None => 0,
        }
    }
}

impl<T: CacheCost> HeapSize for Vec<T> {
    fn heap_size(&self) -> i64 {
        self.iter().map(|item| item.cache_cost()).sum()
    }
}
