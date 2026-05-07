//! Parallel Region Processing
//!
//! Provides multi-threaded memory region scanning with Rayon, including
//! thread-safe handle wrapping and batch result collection to reduce
//! mutex contention.
//!
//! # Optimization Features
//! - **SafeHandle wrapper**: Makes Windows HANDLE Send+Sync safe
//! - **Local batch collection**: Reduces lock contention by 64x on average
//! - **Early-exit flag checking**: Aborts processing if first match already found
//! - **Range filtering**: Only scans memory within requested address bounds

use crate::memory_aobscan::pattern::Pattern;
use crate::memory_aobscan::scanner::strategy::{
    scan_with_multi_byte_anchor, scan_with_single_byte_anchor, AnchorInfo,
};
use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::Foundation::HANDLE;

/// Thread-safe wrapper for Windows process HANDLE.
///
/// Windows HANDLE types are not Send/Sync by default. This wrapper safely
/// converts the pointer-sized HANDLE to an integer for thread safety.
pub struct SafeHandle(usize);

// Mark SafeHandle as thread-safe (HANDLE is safe to Send/Sync across threads)
unsafe impl Send for SafeHandle {}
unsafe impl Sync for SafeHandle {}

impl SafeHandle {
    /// Creates a new thread-safe handle wrapper.
    ///
    /// # Arguments
    /// * `handle` - Raw Windows HANDLE
    pub fn new(handle: HANDLE) -> Self {
        SafeHandle(handle.0 as usize)
    }

    /// Retrieves the raw HANDLE.
    pub fn get(&self) -> HANDLE {
        HANDLE(self.0 as *mut _)
    }
}

/// Number of matches to collect locally before locking global results.
const BATCH_SIZE: usize = 64;

/// Processes a single memory region, reading in chunks and scanning for pattern.
///
/// Uses local batch collection to minimize mutex contention in highly parallel
/// scans. Collects up to BATCH_SIZE matches locally before locking the global
/// result vector.
///
/// # Arguments
/// * `region_addr` - Base address of memory region
/// * `region_size` - Size of memory region in bytes
/// * `start_address` - User-specified scan start address
/// * `length` - User-specified scan length (0 = unlimited)
/// * `chunk_size` - Memory read chunk size (bytes)
/// * `safe_handle` - Thread-safe process handle
/// * `anchor_info` - Anchor search strategy
/// * `use_multi_byte` - `true` for multi-byte anchor, `false` for single byte
/// * `pattern` - Search pattern
/// * `results` - Global result vector (mutex protected)
/// * `found_first` - Atomic flag indicating first match found
/// * `find_all` - `true` to collect all matches, `false` to stop at first
pub fn process_region(
    region_addr: &usize,
    region_size: &usize,
    start_address: usize,
    length: usize,
    chunk_size: usize,
    safe_handle: &SafeHandle,
    anchor_info: &AnchorInfo,
    use_multi_byte: bool,
    pattern: &Pattern,
    results: &std::sync::Mutex<Vec<usize>>,
    found_first: &AtomicBool,
    find_all: bool,
) {
    if !find_all && found_first.load(Ordering::Relaxed) {
        return;
    }

    let region_end = region_addr + region_size;
    if region_end <= start_address {
        return;
    }

    // Calculate effective scan window within region
    let effective_start = if *region_addr >= start_address {
        *region_addr
    } else {
        start_address
    };
    let effective_end = if length > 0 {
        std::cmp::min(region_end, start_address + length)
    } else {
        region_end
    };

    if effective_start >= effective_end {
        return;
    }

    let h = safe_handle.get();
    let mut current_offset = effective_start - region_addr;
    let mut local_batch: Vec<usize> = Vec::with_capacity(BATCH_SIZE);

    while current_offset < (effective_end - region_addr) {
        // Early-exit if first match already found by another thread
        if !find_all && found_first.load(Ordering::Relaxed) {
            if !local_batch.is_empty() {
                let mut guard = results.lock().unwrap();
                guard.extend(local_batch.drain(..));
            }
            return;
        }

        let bytes_to_read =
            std::cmp::min(chunk_size, (effective_end - region_addr) - current_offset);
        let actual_addr = region_addr + current_offset;

        match crate::memory::read_memory_bytes(h, actual_addr, bytes_to_read) {
            Ok(buffer) => {
                if use_multi_byte {
                    scan_with_multi_byte_anchor(
                        &buffer,
                        anchor_info,
                        pattern,
                        &mut local_batch,
                        actual_addr,
                        found_first,
                        find_all,
                    );
                } else {
                    scan_with_single_byte_anchor(
                        &buffer,
                        anchor_info,
                        pattern,
                        &mut local_batch,
                        actual_addr,
                        found_first,
                        find_all,
                    );
                }

                // Flush batch when full
                if local_batch.len() >= BATCH_SIZE {
                    let mut guard = results.lock().unwrap();
                    guard.extend(local_batch.drain(..));
                }
            }
            Err(_) => {}
        }

        current_offset += chunk_size;
    }

    // Flush any remaining matches
    if !local_batch.is_empty() {
        let mut guard = results.lock().unwrap();
        guard.extend(local_batch);
    }
}
