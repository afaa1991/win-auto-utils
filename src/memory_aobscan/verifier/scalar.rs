//! Scalar pattern verification with prefetching optimization

use crate::memory_aobscan::pattern::Pattern;

/// Scalar pattern verification with prefetching optimization
#[inline]
pub fn verify_pattern_scalar(buffer: &[u8], offset: usize, pattern: &Pattern) -> bool {
    let len = pattern.bytes.len();

    // Check bounds
    if offset + len > buffer.len() {
        return false;
    }

    // Prefetch next cache line (64 bytes ahead) to hide memory latency
    if offset + len + 64 < buffer.len() {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            std::arch::x86_64::_mm_prefetch(
                buffer.as_ptr().add(offset + len + 64) as *const i8,
                std::arch::x86_64::_MM_HINT_T0,
            );
        }
    }

    // Optimized scalar verification with early exit
    for i in 0..len {
        // Early exit on first mismatch
        if pattern.mask[i] && buffer[offset + i] != pattern.bytes[i] {
            return false;
        }
    }

    true
}
