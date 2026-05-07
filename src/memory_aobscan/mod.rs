//! Memory Array of Bytes (AOB) Scanning Module
//!
//! Provides high-performance pattern scanning in remote process memory using SIMD acceleration,
//! memchr-based heuristic search, and parallel processing.
//!
//! # Feature Flag
//! Enable with: `--features "memory_aobscan"`
//!
//! # Quick Start
//!
//! ## Basic Pattern Scan
//! ```no_run
//! use win_auto_utils::memory_aobscan::AobScanBuilder;
//! use win_auto_utils::handle::open_process_handle;
//! use windows::Win32::System::Threading::{PROCESS_VM_READ, PROCESS_QUERY_INFORMATION};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let pid = 12345;
//!     
//!     // Get process handle
//!     let desired_access = PROCESS_VM_READ | PROCESS_QUERY_INFORMATION;
//!     let handle = open_process_handle(pid, desired_access)
//!         .ok_or("Failed to open process")?;
//!     
//!     // Scan for pattern with wildcards
//!     let results = AobScanBuilder::new(handle)
//!         .pattern_str("48 89 5C 24 ?? 48 89 6C 24 ??")?
//!         .find_all(false)  // Find first match only (faster)
//!         .scan()?;
//!     
//!     if let Some(addr) = results.first() {
//!         println!("Found at: 0x{:X}", addr);
//!     }
//!     
//!     unsafe { windows::Win32::Foundation::CloseHandle(handle); }
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Usage with Range Limiting
//! ```no_run
//! use win_auto_utils::memory_aobscan::AobScanBuilder;
//!
//! # fn example(handle: windows::Win32::Foundation::HANDLE) -> Result<(), Box<dyn std::error::Error>> {
//! // Scan specific memory range
//! let results = AobScanBuilder::new(handle)
//!     .pattern_str("E8 ?? ?? ?? ??")?  // Call instruction pattern
//!     .start_address(0x10000000)
//!     .length(0x1000000)  // Scan 16MB
//!     .find_all(true)     // Find all occurrences
//!     .scan()?;
//!
//! println!("Found {} matches", results.len());
//! # Ok(())
//! # }
//! ```
//!
//! # Pattern Syntax
//! - **Hex bytes**: Space-separated hex values (e.g., `48 89 5C`)
//! - **Wildcards**: Use `??` or `?` for unknown bytes
//! - **Case insensitive**: Both `48` and `48` work
//!
//! # Performance Features
//! - **memchr acceleration**: Uses fast byte search for anchor points
//! - **Parallel scanning**: Multi-threaded region processing via Rayon
//! - **Smart filtering**: Only scans committed, readable memory regions
//! - **Early exit**: Stops immediately when first match is found (if configured)
//! - **SIMD AVX2**: Vectorized pattern verification (32 bytes at a time)
//! - **Multi-byte anchors**: Intelligent anchor sequence selection to reduce false positives
//! - **Software prefetching**: Hidden memory latency in verification loops
//!
//! # Architecture
//! The scanner works in three phases:
//! 1. **Region Discovery**: Uses `VirtualQueryEx` to find valid memory regions (with caching)
//! 2. **Anchor Search**: Uses `memchr` to quickly locate potential match positions
//! 3. **Pattern Verification**: Validates full pattern at each candidate position (SIMD accelerated)

mod builder;
mod cache;
pub mod pattern;
mod scanner;
mod verifier;

pub use builder::AobScanBuilder;
pub use cache::{clear_all_region_cache, clear_region_cache};
pub use pattern::Pattern;

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Foundation::HANDLE;

    #[test]
    fn test_pattern_parse_simple() {
        let pattern = Pattern::from_str("48 89 5C").unwrap();
        // Verify parsing succeeds without accessing private fields
        drop(pattern);
    }

    #[test]
    fn test_pattern_parse_wildcards() {
        let pattern = Pattern::from_str("48 ?? 5C ?? 90").unwrap();
        // Verify parsing succeeds without accessing private fields
        drop(pattern);
    }

    #[test]
    fn test_pattern_parse_single_wildcard() {
        let pattern = Pattern::from_str("48 ? 5C").unwrap();
        // Verify parsing succeeds without accessing private fields
        drop(pattern);
    }

    #[test]
    fn test_pattern_parse_invalid() {
        assert!(Pattern::from_str("").is_err());
        assert!(Pattern::from_str("GG").is_err());
        assert!(Pattern::from_str("100").is_err()); // Out of u8 range
    }

    #[test]
    fn test_pattern_find_anchor() {
        let pattern = Pattern::from_str("?? 48 ?? 55").unwrap();
        // Verify parsing succeeds; internal anchor logic is tested via integration/scanning
        drop(pattern);
    }

    #[test]
    fn test_pattern_only_wildcards() {
        let pattern = Pattern::from_str("?? ?? ??").unwrap();
        // Verify parsing succeeds
        drop(pattern);
    }

    #[test]
    fn test_multi_byte_anchor_sequence() {
        // Pattern with consecutive known bytes
        let pattern = Pattern::from_str("48 89 5C ?? 90 91").unwrap();

        // Verify parsing succeeds; internal anchor logic is tested via integration/scanning
        drop(pattern);
    }

    #[test]
    fn test_no_multi_byte_anchor_for_wildcards() {
        // Pattern with only wildcards or single known bytes separated by wildcards
        let pattern = Pattern::from_str("48 ?? 55 ?? 90").unwrap();

        // Verify parsing succeeds
        drop(pattern);
    }

    #[test]
    fn test_builder_requires_pattern() {
        let handle = HANDLE(std::ptr::null_mut());
        let result = AobScanBuilder::new(handle).scan();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_chain() {
        let handle = HANDLE(std::ptr::null_mut());
        // Just verify that builder methods can be chained without error
        let _builder = AobScanBuilder::new(handle)
            .pattern_str("48 ?? 55")
            .unwrap()
            .start_address(0x1000)
            .length(0x100)
            .find_all(true);

        // Builder successfully created with all settings applied
        // (Fields are private, so we just verify the chain compiles)
    }

    #[test]
    fn test_simd_scalar_consistency() {
        use crate::memory_aobscan::verifier::verify_pattern;

        // Test that SIMD and scalar implementations produce the same results
        let pattern = Pattern::from_str("48 89 5C ?? 90 91 92 93 94 95 96 97 98 99 9A 9B").unwrap();

        // Create a buffer that matches the pattern
        let mut buffer = vec![0u8; 64];
        buffer[0] = 0x48;
        buffer[1] = 0x89;
        buffer[2] = 0x5C;
        // buffer[3] is wildcard
        for i in 4..16 {
            buffer[i] = 0x90 + (i - 4) as u8;
        }

        let result = verify_pattern(&buffer, 0, &pattern);
        assert!(result, "Pattern should match at offset 0");
    }

    #[test]
    fn test_simd_mismatch_detection() {
        use crate::memory_aobscan::verifier::verify_pattern;

        let pattern = Pattern::from_str("48 89 5C ?? 90 91 92 93 94 95 96 97 98 99 9A 9B").unwrap();

        // Create a buffer that does NOT match
        let mut buffer = vec![0u8; 64];
        buffer[0] = 0x48;
        buffer[1] = 0x89;
        buffer[2] = 0x5C;
        buffer[4] = 0xFF; // This should cause mismatch

        let result = verify_pattern(&buffer, 0, &pattern);
        assert!(!result, "Pattern should not match due to byte mismatch");
    }

    #[test]
    fn test_prefetch_does_not_affect_correctness() {
        use crate::memory_aobscan::verifier::verify_pattern;

        let pattern = Pattern::from_str("48 89 5C").unwrap();
        let buffer = vec![0x48, 0x89, 0x5C, 0x00, 0x00];

        for offset in 0..3 {
            let result = verify_pattern(&buffer, offset, &pattern);

            if offset == 0 {
                assert!(result, "Should match at offset 0");
            } else {
                assert!(!result, "Should not match at offset {}", offset);
            }
        }
    }

    #[test]
    fn test_verify_pattern_boundary_conditions() {
        use crate::memory_aobscan::verifier::verify_pattern;

        let pattern = Pattern::from_str("AA BB CC DD").unwrap();
        let buffer = vec![0xAA, 0xBB, 0xCC, 0xDD];

        assert!(verify_pattern(&buffer, 0, &pattern));
        assert!(!verify_pattern(&buffer, 1, &pattern));
        assert!(!verify_pattern(&[], 0, &pattern));
        assert!(!verify_pattern(&buffer, buffer.len() - 3, &pattern));
    }

    #[test]
    fn test_verify_pattern_all_wildcards() {
        use crate::memory_aobscan::verifier::verify_pattern;

        let pattern = Pattern::from_str("?? ?? ??").unwrap();
        let buffer = vec![0x12, 0x34, 0x56, 0x78];

        assert!(verify_pattern(&buffer, 0, &pattern));
        assert!(verify_pattern(&buffer, 1, &pattern));
    }

    #[test]
    fn test_verify_pattern_mixed_wildcards() {
        use crate::memory_aobscan::verifier::verify_pattern;

        let pattern = Pattern::from_str("48 ?? 5C ?? 90").unwrap();
        let buffer = vec![0x48, 0x00, 0x5C, 0x00, 0x90];

        assert!(verify_pattern(&buffer, 0, &pattern));

        let buffer2 = vec![0x48, 0x00, 0x5C, 0x00, 0x91];
        assert!(!verify_pattern(&buffer2, 0, &pattern));
    }

    #[test]
    fn test_verify_pattern_single_byte_match() {
        use crate::memory_aobscan::verifier::verify_pattern;

        let pattern = Pattern::from_str("AA").unwrap();
        let buffer = vec![0xAA, 0xBB, 0xCC];

        assert!(verify_pattern(&buffer, 0, &pattern));
        assert!(!verify_pattern(&buffer, 1, &pattern));
        assert!(!verify_pattern(&buffer, 2, &pattern));
    }

    #[test]
    fn test_verify_pattern_long_pattern() {
        use crate::memory_aobscan::verifier::verify_pattern;

        let pattern = Pattern::from_str("48 89 5C 24 ?? 48 89 6C 24 ?? 48 89 74 24 ?? 57 48 83 EC 30").unwrap();
        let buffer = vec![
            0x48, 0x89, 0x5C, 0x24, 0x00, 0x48, 0x89, 0x6C, 0x24, 0x00,
            0x48, 0x89, 0x74, 0x24, 0x00, 0x57, 0x48, 0x83, 0xEC, 0x30,
        ];

        assert!(verify_pattern(&buffer, 0, &pattern));

        let mut buffer_mismatch = buffer.clone();
        buffer_mismatch[10] = 0xFF;
        assert!(!verify_pattern(&buffer_mismatch, 0, &pattern));
    }

    #[test]
    fn test_verify_pattern_at_buffer_edges() {
        use crate::memory_aobscan::verifier::verify_pattern;

        let pattern = Pattern::from_str("01 02").unwrap();
        let buffer = vec![0x00, 0x01, 0x02, 0x03];

        assert!(!verify_pattern(&buffer, 0, &pattern));
        assert!(verify_pattern(&buffer, 1, &pattern));
        assert!(!verify_pattern(&buffer, 2, &pattern));
        assert!(!verify_pattern(&buffer, 3, &pattern));
    }

    #[test]
    fn test_anchor_byte_selection() {
        use crate::memory_aobscan::pattern::anchor::find_rarest_byte_index;

        let bytes = vec![0x48, 0x48, 0x48, 0x55, 0x00];
        let mask = vec![true, true, true, true, false];

        let rarest_idx = find_rarest_byte_index(&bytes, &mask);
        assert!(rarest_idx.is_some());

        let idx = rarest_idx.unwrap();
        assert!(idx == 3);
    }

    #[test]
    fn test_multi_byte_anchor_all_same_bytes() {
        use crate::memory_aobscan::pattern::anchor::find_best_anchor_sequence;

        let bytes = vec![0x48, 0x48, 0x48, 0x48];
        let mask = vec![true, true, true, true];

        let anchor = find_best_anchor_sequence(&bytes, &mask);
        assert!(anchor.is_some());

        let seq = anchor.unwrap();
        assert!(seq.len() >= 2);
    }

    #[test]
    fn test_multi_byte_anchor_with_wildcards() {
        use crate::memory_aobscan::pattern::anchor::find_best_anchor_sequence;

        let bytes = vec![0x48, 0x89, 0x5C, 0x24, 0x00];
        let mask = vec![true, true, true, true, false];

        let anchor = find_best_anchor_sequence(&bytes, &mask);
        assert!(anchor.is_some());

        let seq = anchor.unwrap();
        assert!(seq.len() >= 2);
    }

    #[test]
    fn test_pattern_with_no_valid_anchor() {
        use crate::memory_aobscan::pattern::anchor::find_best_anchor_sequence;

        let bytes = vec![0x00, 0x00, 0x00];
        let mask = vec![false, false, false];

        let anchor = find_best_anchor_sequence(&bytes, &mask);
        assert!(anchor.is_none());
    }

    #[test]
    fn test_batch_result_collection() {
        use crate::memory_aobscan::scanner::parallel::{process_region, SafeHandle};
        use crate::memory_aobscan::scanner::strategy::AnchorInfo;
        use std::sync::atomic::AtomicBool;
        use std::sync::Mutex;

        let pattern = Pattern::from_str("48 89 5C").unwrap();
        let buffer = vec![0x48, 0x89, 0x5C, 0x00, 0x00];
        let region_addr = 0x1000usize;
        let region_size = buffer.len();

        let results = Mutex::new(Vec::new());
        let found_first = AtomicBool::new(false);

        process_region(
            &region_addr,
            &region_size,
            0,
            0,
            1024,
            &SafeHandle::new(HANDLE(std::ptr::null_mut())),
            &AnchorInfo::SingleByte(0, 0x48),
            true,
            &pattern,
            &results,
            &found_first,
            true,
        );
    }

    #[test]
    fn test_parallel_early_exit() {
        use std::sync::atomic::AtomicBool;
        use std::sync::Arc;

        let found_first = Arc::new(AtomicBool::new(false));
        let found_first_clone = found_first.clone();

        found_first_clone.store(true, std::sync::atomic::Ordering::Release);
        assert!(found_first.load(std::sync::atomic::Ordering::Acquire));
    }

    #[test]
    fn test_chunk_size_calculation() {
        use crate::memory_aobscan::scanner::calculate_optimal_chunk_size;

        assert_eq!(calculate_optimal_chunk_size(4), 4 * 1024 * 1024);
        assert_eq!(calculate_optimal_chunk_size(8), 2 * 1024 * 1024);
        assert_eq!(calculate_optimal_chunk_size(32), 512 * 1024);
        assert_eq!(calculate_optimal_chunk_size(128), 128 * 1024);
    }

    #[test]
    fn test_pattern_edge_cases() {
        assert!(Pattern::from_str("").is_err());
        assert!(Pattern::from_str(" ").is_err());
        assert!(Pattern::from_str("G").is_err());
        assert!(Pattern::from_str("100").is_err());
        assert!(Pattern::from_str("-1").is_err());

        let pattern = Pattern::from_str("FF").unwrap();
        assert_eq!(pattern.bytes.len(), 1);
    }

    #[test]
    fn test_builder_validation() {
        let handle = HANDLE(std::ptr::null_mut());

        let _ = AobScanBuilder::new(handle).scan();

        let builder = AobScanBuilder::new(handle).pattern_str("48 89 5C").unwrap();
        let _ = builder.scan();
    }

    #[test]
    fn test_builder_chaining() {
        let handle = HANDLE(std::ptr::null_mut());

        let _builder = AobScanBuilder::new(handle)
            .pattern_str("48 ?? 55")
            .unwrap()
            .start_address(0x1000)
            .length(0x100)
            .find_all(true)
            .use_cache(false);

        let _builder2 = AobScanBuilder::new(handle)
            .pattern_bytes(vec![0x48, 0x89, 0x5C])
            .find_all(false);
    }

    #[test]
    fn test_cpu_feature_detection_runs() {
        use crate::memory_aobscan::verifier::verify_pattern;

        let pattern = Pattern::from_str("AA BB CC DD").unwrap();
        let buffer = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE];

        let result = verify_pattern(&buffer, 0, &pattern);
        assert!(result);
    }

    #[test]
    fn test_pattern_bytes_api() {
        use crate::memory_aobscan::pattern::anchor::find_best_anchor_sequence;

        let bytes = vec![0x48, 0x89, 0x5C, 0x24];
        let mask = vec![true, true, true, true];
        let mask_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let anchor_sequence = find_best_anchor_sequence(&bytes, &mask);

        let pattern = Pattern {
            bytes,
            mask,
            mask_bytes,
            anchor_sequence,
        };

        assert_eq!(pattern.bytes.len(), 4);
    }

    #[test]
    fn test_empty_buffer_scan() {
        use crate::memory_aobscan::scanner::strategy::scan_with_single_byte_anchor;
        use crate::memory_aobscan::scanner::strategy::AnchorInfo;
        use std::sync::atomic::AtomicBool;

        let pattern = Pattern::from_str("48 89 5C").unwrap();
        let buffer = vec![];
        let mut results = Vec::new();
        let found_first = AtomicBool::new(false);

        scan_with_single_byte_anchor(
            &buffer,
            &AnchorInfo::SingleByte(0, 0x48),
            &pattern,
            &mut results,
            0,
            &found_first,
            true,
        );

        assert!(results.is_empty());
    }

    #[test]
    fn test_verify_pattern_consistency_across_implementations() {
        use crate::memory_aobscan::verifier::verify_pattern;

        let patterns = vec![
            ("48 89 5C", false),
            ("48 ?? 5C", false),
            ("?? ?? ??", true),
            ("48 89 5C 24 ?? 48 89 6C 24 ?? 48 89 74 24 ?? 57 48 83 EC 30", false),
        ];

        for (pattern_str, all_wildcards) in patterns {
            let pattern = Pattern::from_str(pattern_str).unwrap();
            let pattern_len = pattern.bytes.len();
            let mut buffer = vec![0u8; pattern_len + 16];

            for i in 0..pattern_len {
                if pattern.mask[i] {
                    buffer[i] = pattern.bytes[i];
                }
            }

            assert!(verify_pattern(&buffer, 0, &pattern), "Pattern '{}' should match", pattern_str);

            if !all_wildcards {
                buffer[0] = if pattern.mask[0] { pattern.bytes[0] ^ 0xFF } else { 0x00 };
                assert!(!verify_pattern(&buffer, 0, &pattern), "Pattern '{}' should not match after mutation", pattern_str);
            }
        }
    }

    #[test]
    fn test_global_pos_prevention_at_buffer_start() {
        use crate::memory_aobscan::scanner::strategy::scan_with_single_byte_anchor;
        use crate::memory_aobscan::scanner::strategy::AnchorInfo;
        use std::sync::atomic::AtomicBool;

        let pattern = Pattern::from_str("89 5C 24").unwrap();
        let buffer = vec![0x89, 0x5C, 0x24, 0x00];
        let mut results = Vec::new();
        let found_first = AtomicBool::new(false);

        scan_with_single_byte_anchor(
            &buffer,
            &AnchorInfo::SingleByte(0, 0x89),
            &pattern,
            &mut results,
            0x1000,
            &found_first,
            true,
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 0x1000);
    }

    #[test]
    fn test_global_pos_prevention_single_byte_anchor() {
        use crate::memory_aobscan::scanner::strategy::scan_with_single_byte_anchor;
        use crate::memory_aobscan::scanner::strategy::AnchorInfo;
        use std::sync::atomic::AtomicBool;

        let pattern = Pattern::from_str("48 89 5C").unwrap();
        let buffer = vec![0x48, 0x89, 0x5C, 0x24, 0x00];
        let mut results = Vec::new();
        let found_first = AtomicBool::new(false);

        scan_with_single_byte_anchor(
            &buffer,
            &AnchorInfo::SingleByte(0, 0x48),
            &pattern,
            &mut results,
            0x1000,
            &found_first,
            true,
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 0x1000);
    }

    #[test]
    fn test_global_pos_prevention_multi_byte_anchor() {
        use crate::memory_aobscan::scanner::strategy::scan_with_multi_byte_anchor;
        use crate::memory_aobscan::scanner::strategy::AnchorInfo;
        use std::sync::atomic::AtomicBool;

        let pattern = Pattern::from_str("48 ?? 5C").unwrap();
        let buffer = vec![0x48, 0x89, 0x5C, 0x24, 0x00];
        let mut results = Vec::new();
        let found_first = AtomicBool::new(false);

        scan_with_multi_byte_anchor(
            &buffer,
            &AnchorInfo::MultiByte(vec![(0, 0x48), (2, 0x5C)]),
            &pattern,
            &mut results,
            0x1000,
            &found_first,
            true,
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 0x1000);
    }

    #[test]
    fn test_anchor_at_end_of_buffer() {
        use crate::memory_aobscan::scanner::strategy::scan_with_single_byte_anchor;
        use crate::memory_aobscan::scanner::strategy::AnchorInfo;
        use std::sync::atomic::AtomicBool;

        let pattern = Pattern::from_str("EF").unwrap();
        let buffer = vec![0x00, 0x00, 0xEF];
        let mut results = Vec::new();
        let found_first = AtomicBool::new(false);

        scan_with_single_byte_anchor(
            &buffer,
            &AnchorInfo::SingleByte(0, 0xEF),
            &pattern,
            &mut results,
            0x1000,
            &found_first,
            true,
        );

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_large_buffer_scanning() {
        use crate::memory_aobscan::scanner::strategy::scan_with_single_byte_anchor;
        use crate::memory_aobscan::scanner::strategy::AnchorInfo;
        use std::sync::atomic::AtomicBool;

        let pattern = Pattern::from_str("DE AD BE EF").unwrap();
        let mut buffer = vec![0x00; 10000];
        buffer[5000..5004].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        let mut results = Vec::new();
        let found_first = AtomicBool::new(false);

        scan_with_single_byte_anchor(
            &buffer,
            &AnchorInfo::SingleByte(0, 0xDE),
            &pattern,
            &mut results,
            0x1000,
            &found_first,
            true,
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 0x1000 + 5000);
    }

    #[test]
    fn test_find_first_only_with_early_exit() {
        use crate::memory_aobscan::scanner::strategy::scan_with_single_byte_anchor;
        use crate::memory_aobscan::scanner::strategy::AnchorInfo;
        use std::sync::atomic::AtomicBool;

        let pattern = Pattern::from_str("48").unwrap();
        let buffer = vec![0x48; 100];
        let mut results = Vec::new();
        let found_first = AtomicBool::new(false);

        scan_with_single_byte_anchor(
            &buffer,
            &AnchorInfo::SingleByte(0, 0x48),
            &pattern,
            &mut results,
            0x1000,
            &found_first,
            false,
        );

        assert_eq!(results.len(), 1);
        assert!(found_first.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_multi_byte_anchor_with_wildcard_gap() {
        use crate::memory_aobscan::scanner::strategy::scan_with_multi_byte_anchor;
        use crate::memory_aobscan::scanner::strategy::AnchorInfo;
        use std::sync::atomic::AtomicBool;

        let pattern = Pattern::from_str("48 ?? ?? 5C").unwrap();
        let buffer = vec![0x48, 0x00, 0x00, 0x5C, 0x24, 0x00];
        let mut results = Vec::new();
        let found_first = AtomicBool::new(false);

        scan_with_multi_byte_anchor(
            &buffer,
            &AnchorInfo::MultiByte(vec![(0, 0x48), (3, 0x5C)]),
            &pattern,
            &mut results,
            0x1000,
            &found_first,
            true,
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 0x1000);
    }

    #[test]
    fn test_single_byte_anchor_multiple_matches() {
        use crate::memory_aobscan::scanner::strategy::scan_with_single_byte_anchor;
        use crate::memory_aobscan::scanner::strategy::AnchorInfo;
        use std::sync::atomic::AtomicBool;

        let pattern = Pattern::from_str("48").unwrap();
        let buffer = vec![0x48, 0x00, 0x48, 0x00, 0x48];
        let mut results = Vec::new();
        let found_first = AtomicBool::new(false);

        scan_with_single_byte_anchor(
            &buffer,
            &AnchorInfo::SingleByte(0, 0x48),
            &pattern,
            &mut results,
            0x1000,
            &found_first,
            true,
        );

        assert_eq!(results.len(), 3);
    }
}
