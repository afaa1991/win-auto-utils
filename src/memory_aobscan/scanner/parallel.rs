//! Parallel scanning module
//!
//! Handles parallel region processing with Rayon.

use crate::memory_aobscan::pattern::Pattern;
use crate::memory_aobscan::scanner::strategy::{
    scan_with_multi_byte_anchor, scan_with_single_byte_anchor, AnchorInfo,
};
use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::Foundation::HANDLE;

/// Wrapper to make HANDLE Send + Sync for Rayon
#[derive(Clone, Copy)]
pub struct SafeHandle(usize);
unsafe impl Send for SafeHandle {}
unsafe impl Sync for SafeHandle {}

impl SafeHandle {
    pub fn new(handle: HANDLE) -> Self {
        SafeHandle(handle.0 as usize)
    }

    pub fn get(&self) -> HANDLE {
        HANDLE(self.0 as *mut _)
    }
}

/// Process a single memory region in parallel
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
    // Check global stop flag immediately upon entering thread
    if !find_all && found_first.load(Ordering::Relaxed) {
        return;
    }

    // Apply start_address and length filters
    let region_end = region_addr + region_size;
    if region_end <= start_address {
        return;
    }

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

    while current_offset < (effective_end - region_addr) {
        // Check stop flag before every chunk read
        if !find_all && found_first.load(Ordering::Relaxed) {
            break;
        }

        let bytes_to_read =
            std::cmp::min(chunk_size, (effective_end - region_addr) - current_offset);
        let actual_addr = region_addr + current_offset;

        match crate::memory::read_memory_bytes(h, actual_addr, bytes_to_read) {
            Ok(buffer) => {
                if use_multi_byte {
                    // Use multi-byte anchor sequence scanning
                    scan_with_multi_byte_anchor(
                        &buffer,
                        anchor_info,
                        pattern,
                        &mut results.lock().unwrap(),
                        actual_addr,
                        found_first,
                        find_all,
                    );
                } else {
                    // Use traditional single-byte memchr scanning
                    scan_with_single_byte_anchor(
                        &buffer,
                        anchor_info,
                        pattern,
                        &mut results.lock().unwrap(),
                        actual_addr,
                        found_first,
                        find_all,
                    );
                }
            }
            Err(_) => {}
        }

        current_offset += chunk_size;
    }
}
