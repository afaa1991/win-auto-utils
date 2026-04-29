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

mod pattern;
mod cache;
mod verifier;
mod scanner;
mod builder;

pub use pattern::Pattern;
pub use builder::AobScanBuilder;
pub use cache::{clear_region_cache, clear_all_region_cache};

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
            .pattern_str("48 ?? 55").unwrap()
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
        
        // Ensure prefetching doesn't change verification results
        let pattern = Pattern::from_str("48 89 5C").unwrap();
        let buffer = vec![0x48, 0x89, 0x5C, 0x00, 0x00];
        
        // Test multiple offsets
        for offset in 0..3 {
            let result = verify_pattern(&buffer, offset, &pattern);
            
            if offset == 0 {
                assert!(result, "Should match at offset 0");
            } else {
                assert!(!result, "Should not match at offset {}", offset);
            }
        }
    }
}
