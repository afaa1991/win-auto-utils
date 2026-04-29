//! Verifier module for pattern matching
//!
//! Provides pattern verification with SIMD acceleration and prefetching.

mod scalar;
#[cfg(target_arch = "x86_64")]
mod simd;

// Note: verify_pattern is used internally by scanner, not exported publicly

pub use verify::verify_pattern;

mod verify {
    use super::scalar::verify_pattern_scalar;
    #[cfg(target_arch = "x86_64")]
    use super::simd::{verify_pattern_avx2, verify_pattern_avx512};
    use crate::memory_aobscan::pattern::Pattern;

    /// Verify pattern match at given buffer offset.
    ///
    /// Automatically selects the best implementation based on:
    /// 1. AVX-512 (64 bytes at once) - Fastest, if CPU supports it
    /// 2. AVX2 (32 bytes at once) - Fast, for patterns >= 16 bytes
    /// 3. Scalar with prefetching - Fallback for short patterns or old CPUs
    ///
    /// # Arguments
    /// * `buffer` - The memory buffer read from target process
    /// * `offset` - Offset within the buffer where pattern should start
    /// * `pattern` - The pattern to verify
    ///
    /// # Returns
    /// * `true` - Pattern matches at this offset
    /// * `false` - Pattern does not match
    #[inline]
    pub fn verify_pattern(buffer: &[u8], offset: usize, pattern: &Pattern) -> bool {
        // Priority 1: Use AVX-512 for patterns >= 32 bytes (avoid overhead for very short patterns)
        #[cfg(target_arch = "x86_64")]
        {
            if pattern.bytes.len() >= 32 && std::is_x86_feature_detected!("avx512f") {
                unsafe {
                    return verify_pattern_avx512(buffer, offset, pattern);
                }
            }
            
            // Priority 2: Use AVX2 for patterns >= 16 bytes
            if pattern.bytes.len() >= 16 && std::is_x86_feature_detected!("avx2") {
                unsafe {
                    return verify_pattern_avx2(buffer, offset, pattern);
                }
            }
        }
        
        // Priority 3: Fallback to optimized scalar implementation
        verify_pattern_scalar(buffer, offset, pattern)
    }
}
