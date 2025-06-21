use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// This atomic counter monitors the amount of memory (in bytes) that is
/// currently allocated for this process.
static CURRENT: AtomicUsize = AtomicUsize::new(0);
/// This atomic counter monitors the maximum amount of memory (in bytes) that
/// has been allocated for this process over the course of its life.
static PEAK: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Default, Copy, Clone)]
pub struct PeakAlloc;

impl PeakAlloc {
    /// Returns the number of bytes that are currently allocated to the process
    pub fn current_usage(&self) -> usize {
        CURRENT.load(Ordering::Relaxed)
    }
    /// Returns the maximum number of bytes that have been allocated to the
    /// process over the course of its life.
    pub fn peak_usage(&self) -> usize {
        PEAK.load(Ordering::Relaxed)
    }
}

/// PeakAlloc only implements the minimum required set of methods to make it
/// useable as a global allocator (with `#[global_allocator]` attribute).
unsafe impl GlobalAlloc for PeakAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe { System.alloc(layout) };
        if !ret.is_null() {
            // as pointed out by @luxalpa, fetch_add returns the PREVIOUS value.
            let prev = CURRENT.fetch_add(layout.size(), Ordering::Relaxed);
            PEAK.fetch_max(prev + layout.size(), Ordering::Relaxed);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        CURRENT.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}
