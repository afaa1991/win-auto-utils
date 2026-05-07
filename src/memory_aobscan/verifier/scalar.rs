//! Scalar Pattern Verifier with Prefetching
//!
//! Provides fall-back pattern verification with software prefetching
//! optimization for x86_64 targets. Used when SIMD support is unavailable
//! or for short patterns (< 16 bytes).
//!
//! # Optimization Features
//! - **Multi-cacheline prefetching**: Hints CPU to load next 128 bytes
//! - **Early-exit on mismatch**: Returns immediately when non-matching byte found
//! - **Zero-overhead for non-x86_64**: Prefetch code conditionally compiled

use crate::memory_aobscan::pattern::Pattern;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Verifies pattern match using scalar operations with prefetch optimization.
///
/// # Arguments
/// * `buffer` - Memory buffer to verify
/// * `offset` - Starting position in buffer
/// * `pattern` - Pattern to match
///
/// # Returns
/// `true` if pattern matches at offset, `false` otherwise
///
/// # Performance Note
/// Prefetches two cache lines after the pattern end to improve memory locality
/// for subsequent verification calls.
#[inline]
pub fn verify_pattern_scalar(buffer: &[u8], offset: usize, pattern: &Pattern) -> bool {
    let len = pattern.bytes.len();

    if offset + len > buffer.len() {
        return false;
    }

    // Prefetch upcoming memory for next verification (x86_64 only)
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let prefetch_dist = 128;
        if offset + len + prefetch_dist < buffer.len() {
            _mm_prefetch(
                buffer.as_ptr().add(offset + len) as *const i8,
                _MM_HINT_T0,
            );
            _mm_prefetch(
                buffer.as_ptr().add(offset + len + 64) as *const i8,
                _MM_HINT_T0,
            );
        }
    }

    // Early-exit on first mismatch
    for i in 0..len {
        if pattern.mask[i] && buffer[offset + i] != pattern.bytes[i] {
            return false;
        }
    }

    true
}
