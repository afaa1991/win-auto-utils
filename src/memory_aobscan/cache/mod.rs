//! Memory Region Cache
//!
//! Provides caching mechanisms to avoid repeated VirtualQueryEx calls
//! during multiple scans on the same process, significantly improving
//! performance.
//!
//! # Cache Management
//! - **Per-process caching**: Separate caches per process handle
//! - **Explicit invalidation**: Call clear_region_cache when memory layout changes
//! - **Thread-safe**: Uses Mutex for multi-threaded access

pub(crate) mod region;

// Note: get_valid_memory_regions is used internally by scanner, not exported publicly
pub use cache_api::{clear_all_region_cache, clear_region_cache};

mod cache_api {
    use super::region::get_region_cache;
    use windows::Win32::Foundation::HANDLE;

    /// Clears the cached memory regions for a specific process handle.
    ///
    /// Call when target process's memory layout changes significantly
    /// (e.g., after loading/unloading modules, major allocations/frees).
    ///
    /// # Arguments
    /// * `handle` - Process handle whose cache should be cleared
    pub fn clear_region_cache(handle: HANDLE) {
        let handle_key = handle.0 as usize;
        if let Ok(mut cache) = get_region_cache().lock() {
            cache.remove(&handle_key);
        }
    }

    /// Clears all cached memory regions.
    ///
    /// Useful when switching between different processes or before program shutdown.
    pub fn clear_all_region_cache() {
        if let Ok(mut cache) = get_region_cache().lock() {
            let _count = cache.len();
            cache.clear();
        }
    }
}
