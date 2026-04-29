//! Cache module for memory region caching
//!
//! Provides caching mechanisms to avoid repeated VirtualQueryEx calls.

pub(crate) mod region;

// Note: get_valid_memory_regions is used internally by scanner, not exported publicly
pub use cache_api::{clear_region_cache, clear_all_region_cache};

mod cache_api {
    use super::region::get_region_cache;
    use windows::Win32::Foundation::HANDLE;

    /// Clear the cached memory regions for a specific process handle
    ///
    /// Call this when the target process's memory layout changes significantly
    /// (e.g., after loading/unloading modules, major allocations)
    ///
    /// # Arguments
    /// * `handle` - The process handle whose cache should be cleared
    pub fn clear_region_cache(handle: HANDLE) {
        let handle_key = handle.0 as usize;
        if let Ok(mut cache) = get_region_cache().lock() {
            cache.remove(&handle_key);
        }
    }

    /// Clear all cached memory regions
    ///
    /// Useful when switching between different processes or before program shutdown
    pub fn clear_all_region_cache() {
        if let Ok(mut cache) = get_region_cache().lock() {
            let _count = cache.len();
            cache.clear();
        }
    }
}
