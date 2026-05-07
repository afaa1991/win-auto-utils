//! AOB Scan Engine
//!
//! Core scanning module that orchestrates region discovery, anchor searching,
//! and pattern verification with parallel processing support.
//!
//! # Architecture
//! - **Anchor-based heuristic search**: Uses memchr to locate candidate positions
//! - **Multi-threaded region processing**: Rayon-based parallel scanning
//! - **Dynamic chunk sizing**: Adaptive based on pattern length
//! - **Early-exit support**: Stop immediately when first match found
//! - **Batch result collection**: Reduces mutex contention

pub(crate) mod parallel;
pub mod strategy;

pub use scan_engine::aob_scan_internal;

#[cfg(test)]
pub use scan_engine::calculate_optimal_chunk_size;

mod scan_engine {
    use super::parallel;
    use super::strategy::AnchorInfo;
    use crate::memory::MemoryError;
    use crate::memory_aobscan::cache::region::get_valid_memory_regions;
    use crate::memory_aobscan::pattern::anchor::find_rarest_byte_index;
    use crate::memory_aobscan::pattern::Pattern;
    use rayon::prelude::*;
    use std::sync::atomic::AtomicBool;
    use windows::Win32::Foundation::HANDLE;

    /// Calculates optimal chunk size for memory reading based on pattern length.
    ///
    /// Balances system call overhead, memory usage, and CPU cache efficiency:
    /// - **Very short patterns** (≤4 bytes): Use very large chunks (4MB) to minimize syscalls
    /// - **Short patterns** (5-16 bytes): Use large chunks (2MB) for good balance
    /// - **Medium patterns** (17-64 bytes): Use moderate chunks (512KB) for cache efficiency
    /// - **Long patterns** (>64 bytes): Use smaller chunks (128KB) for maximum cache hits
    ///
    /// # Arguments
    /// * `pattern_len` - Length of search pattern in bytes
    ///
    /// # Returns
    /// Optimal chunk size in bytes
    pub fn calculate_optimal_chunk_size(pattern_len: usize) -> usize {
        match pattern_len {
            0..=4 => 4 * 1024 * 1024,  // 4MB - Minimize syscall overhead for tiny patterns
            5..=16 => 2 * 1024 * 1024, // 2MB - Good balance for common game patterns
            17..=64 => 512 * 1024,    // 512KB - Better cache locality for medium patterns
            _ => 128 * 1024,          // 128KB - Maximum cache efficiency for long patterns
        }
    }

    /// Internal implementation of AOB pattern scanning.
    ///
    /// Orchestrates the entire scan workflow:
    /// 1. Discovers valid memory regions (with optional caching)
    /// 2. Selects optimal anchor strategy (multi-byte or single rarest byte)
    /// 3. Parallel processes regions with batch result collection
    ///
    /// # Arguments
    /// * `handle` - Target process handle with PROCESS_VM_READ access
    /// * `pattern` - Pattern to search for
    /// * `start_address` - Memory address to start scan (0 = entire address space)
    /// * `length` - Bytes to scan (0 = scan all available)
    /// * `find_all` - `true` to find all matches, `false` to stop at first
    /// * `use_cache` - `true` to cache region information, `false` to re-query
    ///
    /// # Returns
    /// Vector of matching addresses, sorted ascending
    pub fn aob_scan_internal(
        handle: HANDLE,
        pattern: &Pattern,
        start_address: usize,
        length: usize,
        find_all: bool,
        use_cache: bool,
    ) -> Result<Vec<usize>, MemoryError> {
        let regions = get_valid_memory_regions(handle, use_cache);

        if regions.is_empty() {
            return Ok(vec![]);
        }

        // Try multi-byte anchor sequence first, fallback to single byte
        let (use_multi_byte, anchor_info) = if let Some(ref seq) = pattern.anchor_sequence {
            (true, AnchorInfo::MultiByte(seq.clone()))
        } else {
            // Use rarest byte as anchor for better performance (fewer false positives)
            let anchor_idx =
                find_rarest_byte_index(&pattern.bytes, &pattern.mask).ok_or_else(|| {
                    MemoryError::InvalidAddress("Pattern contains only wildcards".to_string())
                })?;
            let anchor_byte = pattern.bytes[anchor_idx];

            (false, AnchorInfo::SingleByte(anchor_idx, anchor_byte))
        };

        // Calculate optimal chunk size based on pattern length
        let chunk_size = calculate_optimal_chunk_size(pattern.bytes.len());

        let found_first = AtomicBool::new(false);
        let results = std::sync::Mutex::new(Vec::new());

        let safe_handle = parallel::SafeHandle::new(handle);

        // Use par_iter for parallel scanning with early exit support
        regions.par_iter().for_each(|(region_addr, region_size)| {
            parallel::process_region(
                region_addr,
                region_size,
                start_address,
                length,
                chunk_size,
                &safe_handle,
                &anchor_info,
                use_multi_byte,
                pattern,
                &results,
                &found_first,
                find_all,
            );
        });

        let mut final_results = results.lock().unwrap();
        if !find_all && !final_results.is_empty() {
            // If we only wanted the first one, sort to ensure we return the lowest address
            final_results.sort();
            final_results.truncate(1);
        } else {
            final_results.sort();
        }

        Ok(final_results.drain(..).collect())
    }
}
