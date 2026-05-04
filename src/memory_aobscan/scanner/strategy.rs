//! Scanning strategy module
//!
//! Implements different anchor-based scanning strategies.

use crate::memory_aobscan::pattern::Pattern;
use crate::memory_aobscan::verifier::verify_pattern;
use std::sync::atomic::{AtomicBool, Ordering};

/// Enum to represent different anchor strategies
pub enum AnchorInfo {
    SingleByte(usize, u8),       // (offset, byte)
    MultiByte(Vec<(usize, u8)>), // [(offset, byte), ...]
}

/// Scan using multi-byte anchor sequence
pub fn scan_with_multi_byte_anchor(
    buffer: &[u8],
    anchor_info: &AnchorInfo,
    pattern: &Pattern,
    results: &mut Vec<usize>,
    base_addr: usize,
    found_first: &AtomicBool,
    find_all: bool,
) {
    if let AnchorInfo::MultiByte(ref seq) = anchor_info {
        if seq.is_empty() {
            return;
        }

        // Use first byte as primary anchor
        let (first_offset, first_byte) = seq[0];

        let mut search_start = 0;
        while let Some(pos) = memchr::memchr(first_byte, &buffer[search_start..]) {
            // Check stop flag inside inner loop
            if !find_all && found_first.load(Ordering::Relaxed) {
                break;
            }

            let global_pos = search_start + pos;

            // Verify subsequent bytes in the anchor sequence
            let mut all_match = true;
            for &(offset, byte) in &seq[1..] {
                let offset_diff = offset as isize - first_offset as isize;
                let check_pos = global_pos as isize + offset_diff;

                // FIX: Separate boundary checks to prevent unsigned integer overflow
                // When check_pos is negative, check_pos as usize becomes a very large positive number
                // This could bypass the boundary check and cause memory access violations
                if check_pos < 0 || check_pos as usize >= buffer.len() {
                    all_match = false;
                    break;
                }
                if buffer[check_pos as usize] != byte {
                    all_match = false;
                    break;
                }
            }

            if all_match {
                
                // FIX: Prevent negative pattern_start calculation
                // When global_pos < first_offset, pattern_start would be negative
                // Converting negative usize causes integer overflow and invalid memory access
                // This occurs when anchor is found in buffer's beginning but pattern can't fit
                if global_pos < first_offset {
                    search_start = global_pos + 1;
                    continue;
                }

                let pattern_start = global_pos - first_offset;

                if pattern_start + pattern.bytes.len() <= buffer.len() {
                    if verify_pattern(buffer, pattern_start, pattern) {
                        let result_addr = base_addr + pattern_start;

                        results.push(result_addr);

                        if !find_all {
                            found_first.store(true, Ordering::Release);
                            break;
                        }
                    }
                }
            }

            search_start = global_pos + 1;
        }
    }
}

/// Scan using traditional single-byte memchr
pub fn scan_with_single_byte_anchor(
    buffer: &[u8],
    anchor_info: &AnchorInfo,
    pattern: &Pattern,
    results: &mut Vec<usize>,
    base_addr: usize,
    found_first: &AtomicBool,
    find_all: bool,
) {
    if let AnchorInfo::SingleByte(anchor_idx, anchor_byte) = anchor_info {
        let mut search_start = 0;
        while let Some(pos) = memchr::memchr(*anchor_byte, &buffer[search_start..]) {
            // Check stop flag inside inner loop
            if !find_all && found_first.load(Ordering::Relaxed) {
                break;
            }

            let global_pos = search_start + pos;

            if global_pos >= *anchor_idx {
                let pattern_start = global_pos - anchor_idx;
                let buffer_offset = pattern_start;

                if verify_pattern(buffer, buffer_offset, pattern) {
                    let result_addr = base_addr + buffer_offset;

                    results.push(result_addr);

                    if !find_all {
                        found_first.store(true, Ordering::Release);
                        break;
                    }
                }
            }

            search_start = global_pos + 1;
        }
    }
}
