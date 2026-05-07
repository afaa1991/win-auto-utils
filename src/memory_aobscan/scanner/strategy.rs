//! Anchor-Based Scanning Strategies
//!
//! Implements heuristic scanning strategies using memchr to quickly locate candidate
//! positions before verifying full pattern.
//!
//! # Strategies
//! - **Single-byte anchor**: Uses rarest non-wildcard byte for minimal false positives
//! - **Multi-byte anchor sequence**: Uses multiple non-wildcard bytes at known offsets for better precision
//!
//! Both strategies both use `memchr` crate for extremely fast byte searching, which uses
//! CPU-specific vectorized implementations internally.

use crate::memory_aobscan::pattern::Pattern;
use crate::memory_aobscan::verifier::verify_pattern;
use std::sync::atomic::{AtomicBool, Ordering};

/// Anchor search strategy descriptor.
pub enum AnchorInfo {
    /// Single-byte anchor with offset and value.
    SingleByte(usize, u8),
    /// Multi-byte anchor sequence with [(offset, byte) pairs.
    MultiByte(Vec<(usize, u8)>),
}

/// Scans buffer using multi-byte anchor sequence.
///
/// Uses first byte of anchor for fast memchr search, then verifies remaining
/// anchor bytes at their offsets, then full pattern.
///
/// # Arguments
/// * `buffer` - Memory buffer to scan
/// * `anchor_info` - Multi-byte anchor sequence
/// * `pattern` - Full pattern to verify
/// * `results` - Vector to collect matches
/// * `base_addr` - Base address of buffer in target process
/// * `found_first` - Atomic flag to signal match found
/// * `find_all` - Continue scanning or stop at first match
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

                // Separate boundary checks to prevent unsigned overflow
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

                // Prevent negative pattern_start calculation
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

/// Scans buffer using traditional single-byte anchor + memchr.
///
/// Uses memchr finds positions of anchor byte, then verifies full pattern.
///
/// # Arguments
/// * `buffer` - Memory buffer to scan
/// * `anchor_info` - Single-byte anchor (offset, byte)
/// * `pattern` - Full pattern to verify
/// * `results` - Vector to collect matches
/// * `base_addr` - Base address of buffer in target process
/// * `found_first` - Atomic flag to signal match found
/// * `find_all` - Continue scanning or stop at first match
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
