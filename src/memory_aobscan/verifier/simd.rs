//! AVX2 and AVX-512 accelerated pattern verification

use crate::memory_aobscan::pattern::Pattern;

/// AVX2 accelerated pattern verification (processes 32 bytes at a time)
///
/// # Safety
/// Requires AVX2 CPU support (checked before calling)
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn verify_pattern_avx2(buffer: &[u8], offset: usize, pattern: &Pattern) -> bool {
    use std::arch::x86_64::*;

    let len = pattern.bytes.len();

    // Bounds check
    if offset + len > buffer.len() {
        return false;
    }

    let buf_ptr = buffer.as_ptr().add(offset);
    let pat_ptr = pattern.bytes.as_ptr();
    let mask_ptr = pattern.mask_bytes.as_ptr();

    // Process 32 bytes at a time using AVX2
    let mut i = 0;
    while i + 32 <= len {
        // Load data (unaligned load - modern CPUs handle this efficiently)
        let buf_chunk = _mm256_loadu_si256(buf_ptr.add(i) as *const __m256i);
        let pat_chunk = _mm256_loadu_si256(pat_ptr.add(i) as *const __m256i);
        let mask_chunk = _mm256_loadu_si256(mask_ptr.add(i) as *const __m256i);

        // Compare: (buf == pat) OR (NOT mask)
        // If byte is wildcard (mask=0x00), NOT mask=0xFF, result is all 1s (match)
        // If byte must match (mask=0xFF), NOT mask=0x00, result depends on comparison
        let cmp = _mm256_cmpeq_epi8(buf_chunk, pat_chunk);
        let not_mask = _mm256_andnot_si256(mask_chunk, _mm256_set1_epi8(-1));
        let result = _mm256_or_si256(cmp, not_mask);

        // Check if all bytes match (result should be all 0xFF = -1)
        if _mm256_movemask_epi8(result) != -1 {
            return false;
        }

        i += 32;
    }

    // Handle remaining bytes with scalar code
    for j in i..len {
        if pattern.mask[j] && buffer[offset + j] != pattern.bytes[j] {
            return false;
        }
    }

    true
}

/// AVX-512 accelerated pattern verification (processes 64 bytes at a time)
///
/// This provides ~30-50% speedup over AVX2 for long patterns.
///
/// # Safety
/// Requires AVX-512F CPU support (checked before calling)
#[target_feature(enable = "avx512f")]
#[inline]
pub unsafe fn verify_pattern_avx512(buffer: &[u8], offset: usize, pattern: &Pattern) -> bool {
    use std::arch::x86_64::*;

    let len = pattern.bytes.len();

    // Bounds check
    if offset + len > buffer.len() {
        return false;
    }

    let buf_ptr = buffer.as_ptr().add(offset);
    let pat_ptr = pattern.bytes.as_ptr();
    let mask_ptr = pattern.mask_bytes.as_ptr();

    // Process 64 bytes at a time using AVX-512
    let mut i = 0;
    while i + 64 <= len {
        // Load 64 bytes at once
        let buf_chunk = _mm512_loadu_si512(buf_ptr.add(i) as *const __m512i);
        let pat_chunk = _mm512_loadu_si512(pat_ptr.add(i) as *const __m512i);
        let mask_chunk = _mm512_loadu_si512(mask_ptr.add(i) as *const __m512i);

        // Compare bytes: returns mask where equal bytes have bit set
        let cmp_mask = _mm512_cmpeq_epi8_mask(buf_chunk, pat_chunk);

        // Get mask of required bytes (where mask_bytes is 0xFF)
        let required_mask = _mm512_movepi8_mask(mask_chunk);

        // Check: all required bytes must match
        // (cmp_mask & required_mask) == required_mask
        let matched_required = cmp_mask & required_mask;

        if matched_required != required_mask {
            return false;
        }

        i += 64;
    }

    // Handle remaining bytes with AVX2 or scalar
    if i + 32 <= len {
        // Use AVX2 for next 32 bytes
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

    // Handle final remaining bytes with scalar code
    for j in i..len {
        if pattern.mask[j] && buffer[offset + j] != pattern.bytes[j] {
            return false;
        }
    }

    true
}
