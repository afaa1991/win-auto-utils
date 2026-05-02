//! Scanner module for AOB scanning engine
//!
//! Provides the core scanning logic with parallel processing and early exit support.

pub(crate) mod parallel;
mod strategy;

pub use scan_engine::aob_scan_internal;

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

    /// Calculate optimal chunk size based on pattern length and memory characteristics
    ///
    /// Balances between:
    /// - System call overhead (larger chunks = fewer calls)
    /// - Memory usage (smaller chunks = less RAM per thread)
    /// - CPU cache efficiency (smaller chunks = better L2/L3 cache hit rate)
    ///
    /// # Strategy
    /// - Very short patterns (≤4 bytes): Use very large chunks (4MB) to minimize syscalls
    /// - Short patterns (5-16 bytes): Use large chunks (2MB) for good balance
    /// - Medium patterns (17-64 bytes): Use moderate chunks (512KB) for cache efficiency
    /// - Long patterns (>64 bytes): Use smaller chunks (128KB) for maximum cache hits
    pub fn calculate_optimal_chunk_size(pattern_len: usize) -> usize {
        match pattern_len {
            0..=4 => 4 * 1024 * 1024, // 4MB - Minimize syscall overhead for tiny patterns
            5..=16 => 2 * 1024 * 1024, // 2MB - Good balance for common game patterns
            17..=64 => 512 * 1024,    // 512KB - Better cache locality for medium patterns
            _ => 128 * 1024,          // 128KB - Maximum cache efficiency for long patterns
        }
    }

    /// Internal implementation of the AOB scan logic.
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
