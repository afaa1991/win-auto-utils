//! Memory region query module
//!
//! Handles querying and filtering valid memory regions for scanning.

use std::sync::OnceLock;
use windows::{
    Win32::Foundation::HANDLE,
    Win32::System::Memory::{VirtualQueryEx, MEMORY_BASIC_INFORMATION},
};

/// Global cache for memory regions per process handle
/// Key: Process handle (as usize), Value: Cached memory regions
static REGION_CACHE: OnceLock<
    std::sync::Mutex<std::collections::HashMap<usize, Vec<(usize, usize)>>>,
> = OnceLock::new();

/// Get or initialize the region cache
pub(crate) fn get_region_cache(
) -> &'static std::sync::Mutex<std::collections::HashMap<usize, Vec<(usize, usize)>>> {
    REGION_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

/// Retrieves all valid memory regions for scanning with intelligent filtering and optional caching.
///
/// Optimizes scanning by:
/// 1. Skipping NULL pointer guard page (0x0 - 0xFFFF)
/// 2. Starting from minimum valid user-space address (0x10000)
/// 3. Only including committed, readable regions
/// 4. Caching results for repeated scans on the same process
///
/// # Arguments
/// * `handle` - Process handle with PROCESS_QUERY_INFORMATION and PROCESS_VM_READ
/// * `use_cache` - Whether to use cached regions (true) or force fresh query (false)
///
/// # Returns
/// Vector of (base_address, size) tuples for scannable regions
pub fn get_valid_memory_regions(handle: HANDLE, use_cache: bool) -> Vec<(usize, usize)> {
    let handle_key = handle.0 as usize;

    // Try to get from cache if enabled
    if use_cache {
        if let Some(cache) = get_region_cache().lock().ok().as_ref() {
            if let Some(cached_regions) = cache.get(&handle_key) {
                return cached_regions.clone();
            }
        }
    }

    // Perform full scan
    let mut regions = Vec::new();

    // Start from 0x10000 to skip NULL pointer guard page (first 64KB)
    // User-space processes never have valid code/data in 0x0-0xFFFF
    let mut address = 0x10000usize;
    let mut mbi = MEMORY_BASIC_INFORMATION::default();

    unsafe {
        while VirtualQueryEx(
            handle,
            Some(address as *const _),
            &mut mbi,
            std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        ) != 0
        {
            let protect = mbi.Protect;

            // Filter criteria:
            // 1. Must be MEM_COMMIT (allocated and accessible)
            // 2. Must be readable (READWRITE, EXECUTE_READ, or EXECUTE_READWRITE)
            // 3. Must NOT be PAGE_GUARD (would trigger exception on access)
            if mbi.State == windows::Win32::System::Memory::MEM_COMMIT
                && (protect.0
                    & (windows::Win32::System::Memory::PAGE_READWRITE.0
                        | windows::Win32::System::Memory::PAGE_EXECUTE_READ.0
                        | windows::Win32::System::Memory::PAGE_EXECUTE_READWRITE.0))
                    != 0
                && (protect.0 & windows::Win32::System::Memory::PAGE_GUARD.0) == 0
            {
                regions.push((mbi.BaseAddress as usize, mbi.RegionSize));
            }

            // Move to next region
            address += mbi.RegionSize;
        }
    }

    // Cache the results for future use (only if caching is enabled)
    if use_cache {
        if let Ok(mut cache) = get_region_cache().lock() {
            cache.insert(handle_key, regions.clone());
        }
    }

    regions
}
