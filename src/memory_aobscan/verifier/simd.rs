//! SIMD Pattern Verifiers (AVX2 / AVX-512)
//!
//! Provides vectorized pattern verification using Intel SIMD extensions.
//!
//! # Performance Features
//! - **Vectorized mismatch detection**: Compares 32/64 bytes per iteration
//! - **Early-exit on chunk mismatch**: Aborts immediately if any chunk doesn't match
//! - **Hierarchical verification**: 64b → 32b → scalar for tail handling
//! - **Software prefetching**: In long patterns, hints next chunks
//!
//! # Safety
//! Functions use `unsafe` blocks to call CPU intrinsics directly. Callers must
//! ensure CPU features are available before invocation.

use crate::memory_aobscan::pattern::Pattern;

/// Verifies pattern using AVX2 instructions (32 bytes per iteration).
///
/// # Arguments
/// * `buffer` - Memory buffer to verify
/// * `offset` - Starting position in buffer
/// * `pattern` - Pattern to match
///
/// # Returns
/// `true` if pattern matches, `false` otherwise
///
/// # Safety
/// Requires CPU with AVX2 support.
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn verify_pattern_avx2(buffer: &[u8], offset: usize, pattern: &Pattern) -> bool {
    use std::arch::x86_64::*;

    let len = pattern.bytes.len();

    if offset + len > buffer.len() {
        return false;
    }

    let buf_ptr = buffer.as_ptr().add(offset);
    let pat_ptr = pattern.bytes.as_ptr();
    let mask_ptr = pattern.mask_bytes.as_ptr();

    // Process 32 bytes at a time
    let mut i = 0;
    while i + 32 <= len {
        let buf_chunk = _mm256_loadu_si256(buf_ptr.add(i) as *const __m256i);
        let pat_chunk = _mm256_loadu_si256(pat_ptr.add(i) as *const __m256i);
        let mask_chunk = _mm256_loadu_si256(mask_ptr.add(i) as *const __m256i);

        let cmp = _mm256_cmpeq_epi8(buf_chunk, pat_chunk);
        let not_mask = _mm256_andnot_si256(mask_chunk, _mm256_set1_epi8(-1));
        let result = _mm256_or_si256(cmp, not_mask);

        if _mm256_movemask_epi8(result) != -1 {
            return false;
        }

        i += 32;
    }

    // Handle remaining bytes with scalar
    for j in i..len {
        if pattern.mask[j] && buffer[offset + j] != pattern.bytes[j] {
            return false;
        }
    }

    true
}

/// Verifies pattern using AVX-512 instructions (64 bytes per iteration).
///
/// Automatically falls back to AVX2 for 32-byte tail and scalar for remainder.
///
/// # Arguments
/// * `buffer` - Memory buffer to verify
/// * `offset` - Starting position in buffer
/// * `pattern` - Pattern to match
///
/// # Returns
/// `true` if pattern matches, `false` otherwise
///
/// # Safety
/// Requires CPU with AVX-512F support.
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn verify_pattern_avx512(buffer: &[u8], offset: usize, pattern: &Pattern) -> bool {
    use std::arch::x86_64::*;

    let len = pattern.bytes.len();

    if offset + len > buffer.len() {
        return false;
    }

    let buf_ptr = buffer.as_ptr().add(offset);
    let pat_ptr = pattern.bytes.as_ptr();
    let mask_ptr = pattern.mask_bytes.as_ptr();

    // Process 64 bytes at a time with prefetch
    let mut i = 0;
    while i + 64 <= len {
        let buf_chunk = _mm512_loadu_si512(buf_ptr.add(i) as *const __m512i);
        let pat_chunk = _mm512_loadu_si512(pat_ptr.add(i) as *const __m512i);
        let mask_chunk = _mm512_loadu_si512(mask_ptr.add(i) as *const __m512i);

        let cmp_mask = _mm512_cmpeq_epi8_mask(buf_chunk, pat_chunk);
        let required_mask = _mm512_movepi8_mask(mask_chunk);

        let matched_required = cmp_mask & required_mask;

        if matched_required != required_mask {
            return false;
        }

        if i + 128 <= len {
            _mm_prefetch(buf_ptr.add(i + 64) as *const i8, _MM_HINT_T0);
            _mm_prefetch(buf_ptr.add(i + 128) as *const i8, _MM_HINT_T0);
        }

        i += 64;
    }

    // Fallback to AVX2 for 32-byte tail
    if i + 32 <= len {
        let buf_chunk = _mm256_loadu_si256(buf_ptr.add(i) as *const __m256i);
        let pat_chunk = _mm256_loadu_si256(pat_ptr.add(i) as *const __m256i);
        let mask_chunk = _mm256_loadu_si256(mask_ptr.add(i) as *const __m256i);

        let cmp = _mm256_cmpeq_epi8(buf_chunk, pat_chunk);
        let not_mask = _mm256_andnot_si256(mask_chunk, _mm256_set1_epi8(-1));
        let result = _mm256_or_si256(cmp, not_mask);

        if _mm256_movemask_epi8(result) != -1 {
            return false;
        }

        i += 32;
    }

    // Scalar for final remainder
    for j in i..len {
        if pattern.mask[j] && buffer[offset + j] != pattern.bytes[j] {
            return false;
        }
    }

    true
}
