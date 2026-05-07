//! Pattern Verifier Module
//!
//! Provides hardware-accelerated pattern verification with automatic dispatching
//! to scalar, AVX2, or AVX-512 implementations based on CPU capabilities and
//! pattern length.
//!
//! # CPU Feature Selection
//! - **AVX-512**: Patterns ≥ 32 bytes (if supported)
//! - **AVX2**: Patterns ≥ 16 bytes (if supported)
//! - **Scalar + Prefetch**: Fallback for short patterns or non-x86_64 systems
//!
//! # Optimization Features
//! - **One-time CPU feature detection**: Cached at module initialization
//! - **Compile-time target arch detection**: Graceful fallback for non-x86_64
//! - **Early SIMD exit**: Vectorized mismatch detection
//! - **Software prefetching**: Improves memory locality in verification loops

mod scalar;
#[cfg(target_arch = "x86_64")]
mod simd;

pub use verify::verify_pattern;

/// x86_64 implementation with SIMD dispatching and CPU feature caching
#[cfg(target_arch = "x86_64")]
mod verify {
    use super::scalar::verify_pattern_scalar;
    #[cfg(target_arch = "x86_64")]
    use super::simd::{verify_pattern_avx2, verify_pattern_avx512};
    use crate::memory_aobscan::pattern::Pattern;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// Cached flag indicating AVX-512 support
    static AVX512_AVAILABLE: AtomicBool = AtomicBool::new(false);
    /// Cached flag indicating AVX2 support
    static AVX2_AVAILABLE: AtomicBool = AtomicBool::new(false);

    /// Initializes CPU feature detection (once per process)
    ///
    /// Uses atomic compare-exchange to guarantee single initialization even
    /// in multi-threaded scenarios.
    fn init_cpu_features() {
        static INIT: AtomicBool = AtomicBool::new(false);
        if INIT.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            if std::is_x86_feature_detected!("avx512f") {
                AVX512_AVAILABLE.store(true, Ordering::Release);
            }
            if std::is_x86_feature_detected!("avx2") {
                AVX2_AVAILABLE.store(true, Ordering::Release);
            }
        }
    }

    /// Verifies if a pattern matches the buffer at the given offset.
    ///
    /// Dispatches to appropriate implementation based on CPU features and pattern length.
    ///
    /// # Arguments
    /// * `buffer` - Memory buffer to verify
    /// * `offset` - Starting position in buffer
    /// * `pattern` - Pattern to match
    ///
    /// # Returns
    /// `true` if pattern matches, `false` otherwise
    #[inline]
    pub fn verify_pattern(buffer: &[u8], offset: usize, pattern: &Pattern) -> bool {
        init_cpu_features();

        let len = pattern.bytes.len();

        if len >= 32 && AVX512_AVAILABLE.load(Ordering::Acquire) {
            unsafe {
                return verify_pattern_avx512(buffer, offset, pattern);
            }
        }

        if len >= 16 && AVX2_AVAILABLE.load(Ordering::Acquire) {
            unsafe {
                return verify_pattern_avx2(buffer, offset, pattern);
            }
        }

        verify_pattern_scalar(buffer, offset, pattern)
    }
}

/// Fallback implementation for non-x86_64 architectures
#[cfg(not(target_arch = "x86_64"))]
mod verify {
    use super::scalar::verify_pattern_scalar;
    use crate::memory_aobscan::pattern::Pattern;

    /// Verifies pattern match (scalar-only fallback)
    #[inline]
    pub fn verify_pattern(buffer: &[u8], offset: usize, pattern: &Pattern) -> bool {
        verify_pattern_scalar(buffer, offset, pattern)
    }
}
