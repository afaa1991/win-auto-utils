//! Color finding algorithms - Pure Rust implementation
//!
//! Provides high-performance color searching in pixel arrays.
//! This module is **completely independent** with zero external dependencies.
//!
//! # Features
//! - Scalar implementation (fallback for all architectures)
//! - AVX2 SIMD optimization (x86_64 only, ~4-8x faster)
//! - SWAR technique for remainder handling
//! - Automatic runtime CPU feature detection
//!
//! # Data Format
//! All color data should be in BGRA format:
//! - Byte 0: Blue channel
//! - Byte 1: Green channel
//! - Byte 2: Red channel
//! - Byte 3: Alpha channel (unused for matching)
//!
//! # Example
//! ```
//! use win_auto_utils::color_finder::{find_color_in_buffer, FindResult};
//!
//! // Create a sample BGRA buffer (100x100 pixels, all red)
//! let width = 100i32;
//! let height = 100i32;
//! let mut buffer = vec![0u8; (width * height * 4) as usize];
//!
//! // Set a blue pixel at position (50, 50)
//! let idx = ((50 * width + 50) * 4) as usize;
//! buffer[idx] = 255;     // B
//! buffer[idx + 1] = 0;   // G
//! buffer[idx + 2] = 0;   // R
//! buffer[idx + 3] = 255; // A
//!
//! // Search for blue color
//! let result = find_color_in_buffer(&buffer, width, height, (255, 0, 0));
//! assert!(result.matched);
//! assert_eq!(result.x, 50);
//! assert_eq!(result.y, 50);
//! ```

/// Result structure for color finding operations
///
/// Contains the match status and absolute screen coordinates if found.
#[derive(Debug, Default, Clone, Copy)]
pub struct FindResult {
    /// Whether the target color was found
    pub matched: bool,

    /// X coordinate within the buffer (if matched)
    pub x: i32,

    /// Y coordinate within the buffer (if matched)
    pub y: i32,
}

/// Scalar version: pixel-by-pixel comparison (used when SIMD is unavailable)
///
/// Iterates through each pixel sequentially, checking BGR channels.
/// This is the fallback implementation for non-x86_64 architectures or
/// when AVX2 is not supported.
///
/// # Arguments
/// * `colors` - Raw BGRA byte array
/// * `w` - Width of the region in pixels
/// * `target` - Target color as (B, G, R) tuple
///
/// # Returns
/// * `Some((x, y))` - Local coordinates within the region if found
/// * `None` - If color not found
#[inline]
fn find_color_scalar(colors: &[u8], w: i32, target: (u8, u8, u8)) -> Option<(i32, i32)> {
    let pixel_count = colors.len() / 4;

    for i in 0..pixel_count {
        let offset = i * 4;
        // BGRA: Blue, Green, Red, Alpha
        if colors[offset] == target.0
            && colors[offset + 1] == target.1
            && colors[offset + 2] == target.2
        {
            let x = (i as i32) % w;
            let y = (i as i32) / w;
            return Some((x, y));
        }
    }

    None
}

/// SWAR-based pixel comparison for remaining pixels (less than 8 pixels)
///
/// Uses SIMD Within A Register technique to process 2 pixels simultaneously
/// by packing them into a u64 and using bitwise operations for parallel comparison.
///
/// # Arguments
/// * `colors` - Raw BGRA byte array
/// * `start_offset` - Starting byte offset in the colors array
/// * `remaining_bytes` - Number of bytes to process
/// * `w` - Width of the region for coordinate calculation
/// * `target` - Target color as (B, G, R) tuple
///
/// # Returns
/// * `Some((x, y))` - Local coordinates if color found
/// * `None` - If color not found in remaining pixels
#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn find_color_swar(
    colors: &[u8],
    start_offset: usize,
    remaining_bytes: usize,
    w: i32,
    target: (u8, u8, u8),
) -> Option<(i32, i32)> {
    if remaining_bytes < 4 {
        return None;
    }

    // Process pairs of pixels using u64 SWAR (8 bytes = 2 pixels)
    if remaining_bytes >= 8 {
        let swar_pairs = remaining_bytes / 8;

        for i in 0..swar_pairs {
            let offset = start_offset + i * 8;

            // Load 8 bytes (2 pixels) as u64
            let chunk = std::ptr::read_unaligned(colors.as_ptr().add(offset) as *const u64);

            // Extract and compare both pixels
            if let Some(pixel_idx) = compare_two_pixels_swar(chunk, target) {
                let global_idx = start_offset / 4 + i * 2 + pixel_idx;
                let x = (global_idx as i32) % w;
                let y = (global_idx as i32) / w;
                return Some((x, y));
            }
        }

        // Handle odd pixel if remaining_bytes is not divisible by 8
        let processed_bytes = swar_pairs * 8;
        let leftover_bytes = remaining_bytes - processed_bytes;

        if leftover_bytes >= 4 {
            let offset = start_offset + processed_bytes;
            if colors[offset] == target.0
                && colors[offset + 1] == target.1
                && colors[offset + 2] == target.2
            {
                let global_idx = start_offset / 4 + swar_pairs * 2;
                let x = (global_idx as i32) % w;
                let y = (global_idx as i32) / w;
                return Some((x, y));
            }
        }
    } else if remaining_bytes >= 4 {
        // Less than 8 bytes but at least 4: use simple scalar comparison
        let offset = start_offset;
        if colors[offset] == target.0
            && colors[offset + 1] == target.1
            && colors[offset + 2] == target.2
        {
            let global_idx = start_offset / 4;
            let x = (global_idx as i32) % w;
            let y = (global_idx as i32) / w;
            return Some((x, y));
        }
    }

    None
}

/// Compare two pixels packed in a u64 using SWAR technique
///
/// Pixel layout in u64 (little-endian): [B0, G0, R0, A0, B1, G1, R1, A1]
///
/// # Arguments
/// * `chunk` - 8 bytes containing 2 BGRA pixels
/// * `target` - Target color as (B, G, R) tuple
///
/// # Returns
/// * `Some(0)` - First pixel matches
/// * `Some(1)` - Second pixel matches
/// * `None` - Neither pixel matches
#[inline]
fn compare_two_pixels_swar(chunk: u64, target: (u8, u8, u8)) -> Option<usize> {
    // Extract B, G, R channels using bit masks
    let b_mask: u64 = 0x000000FF_000000FF;
    let g_mask: u64 = 0x0000FF00_0000FF00;
    let r_mask: u64 = 0x00FF0000_00FF0000;

    let b_vals = chunk & b_mask;
    let g_vals = (chunk & g_mask) >> 8;
    let r_vals = (chunk & r_mask) >> 16;

    // Check first pixel (lower 32 bits)
    let first_pixel_match = (b_vals & 0xFF) == (target.0 as u64)
        && (g_vals & 0xFF) == (target.1 as u64)
        && (r_vals & 0xFF) == (target.2 as u64);

    if first_pixel_match {
        return Some(0);
    }

    // Check second pixel (upper 32 bits)
    let second_pixel_match = ((b_vals >> 32) & 0xFF) == (target.0 as u64)
        && ((g_vals >> 32) & 0xFF) == (target.1 as u64)
        && ((r_vals >> 32) & 0xFF) == (target.2 as u64);

    if second_pixel_match {
        return Some(1);
    }

    None
}

/// AVX2 optimized version: compares 8 pixels simultaneously
///
/// Uses SIMD instructions to process multiple pixels in parallel,
/// providing significant performance improvements on supported hardware.
/// Includes handling for remaining pixels using SWAR technique.
///
/// # Safety
/// This function requires AVX2 support. Callers must ensure:
/// - Running on x86_64 architecture
/// - AVX2 feature is detected via `std::is_x86_feature_detected!("avx2")`
/// - Function is called with `#[target_feature(enable = "avx2")]`
///
/// # Arguments
/// * `colors` - Raw BGRA byte array
/// * `w` - Width of the region in pixels
/// * `target` - Target color as (B, G, R) tuple
///
/// # Returns
/// * `Some((x, y))` - Local coordinates within the region if found
/// * `None` - If color not found
///
/// # Performance Optimization
/// - **Main loop**: Processes 8 pixels per iteration (32 bytes) using AVX2
///   - Uses shuffle masks to extract B, G, R channels
///   - Employs bitwise operations for parallel comparison
/// - **Remainder handling**: Uses SWAR (SIMD Within A Register) technique
///   - Packs 2 pixels into a single u64 register
///   - Performs parallel channel extraction and comparison
///   - More efficient than scalar byte-by-byte comparison
///   - Handles edge cases (< 8 remaining pixels) gracefully
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn find_color_avx2(colors: &[u8], w: i32, target: (u8, u8, u8)) -> Option<(i32, i32)> {
    use std::arch::x86_64::*;

    let pixel_count = colors.len() / 4;
    let simd_pixels = pixel_count / 8; // Process 8 pixels at a time
    let remaining_pixels = pixel_count % 8;

    // Create AVX2 vectors for target color (BGRA format, repeated 8 times)
    let target_b = _mm256_set1_epi8(target.0 as i8);
    let target_g = _mm256_set1_epi8(target.1 as i8);
    let target_r = _mm256_set1_epi8(target.2 as i8);

    // Shuffle masks: extract B, G, R channels from BGRA
    let mask_b = _mm256_set_epi8(
        12, -1, -1, -1, 8, -1, -1, -1, 4, -1, -1, -1, 0, -1, -1, -1, 12, -1, -1, -1, 8, -1, -1, -1,
        4, -1, -1, -1, 0, -1, -1, -1,
    );
    let mask_g = _mm256_set_epi8(
        13, -1, -1, -1, 9, -1, -1, -1, 5, -1, -1, -1, 1, -1, -1, -1, 13, -1, -1, -1, 9, -1, -1, -1,
        5, -1, -1, -1, 1, -1, -1, -1,
    );
    let mask_r = _mm256_set_epi8(
        14, -1, -1, -1, 10, -1, -1, -1, 6, -1, -1, -1, 2, -1, -1, -1, 14, -1, -1, -1, 10, -1, -1,
        -1, 6, -1, -1, -1, 2, -1, -1, -1,
    );

    let mut base_idx = 0;

    // Process 8-pixel blocks in batch
    for _ in 0..simd_pixels {
        // Load 32 bytes (8 pixels of BGRA data)
        let data = _mm256_loadu_si256(colors.as_ptr().add(base_idx) as *const __m256i);

        // Extract B, G, R channels
        let b_channel = _mm256_shuffle_epi8(data, mask_b);
        let g_channel = _mm256_shuffle_epi8(data, mask_g);
        let r_channel = _mm256_shuffle_epi8(data, mask_r);

        // Compare B, G, R channels
        let cmp_b = _mm256_cmpeq_epi8(b_channel, target_b);
        let cmp_g = _mm256_cmpeq_epi8(g_channel, target_g);
        let cmp_r = _mm256_cmpeq_epi8(r_channel, target_r);

        // Three-way AND: B && G && R
        let matched = _mm256_and_si256(_mm256_and_si256(cmp_b, cmp_g), cmp_r);

        // Check if any pixels matched
        let mask = _mm256_movemask_epi8(matched);
        if mask != 0 {
            let bit_pos = mask.trailing_zeros() as usize;
            let pixel_idx = bit_pos / 4;
            let global_idx = base_idx / 4 + pixel_idx;
            let x = (global_idx as i32) % w;
            let y = (global_idx as i32) / w;
            return Some((x, y));
        }

        base_idx += 32; // 8 pixels × 4 bytes
    }

    // Handle remaining pixels using SWAR technique
    let remaining_start = simd_pixels * 32;
    let remaining_bytes = remaining_pixels * 4;

    find_color_swar(colors, remaining_start, remaining_bytes, w, target)
}

/// Find color in a BGRA buffer (automatically selects optimal implementation)
///
/// Searches for the specified color in a raw BGRA byte array.
/// Automatically chooses between AVX2 SIMD and scalar implementations based on
/// CPU capabilities.
///
/// # Arguments
/// * `colors` - Raw BGRA byte array
/// * `w` - Width of the region in pixels
/// * `h` - Height of the region in pixels (used for validation)
/// * `target` - Target color as (B, G, R) tuple
///
/// # Returns
/// `FindResult` containing match status and local coordinates
///
/// # Notes
/// - Coordinates are local to the buffer (not screen coordinates)
/// - For screen capture integration, see the `dxgi` feature
pub fn find_color_in_buffer(colors: &[u8], w: i32, _h: i32, target: (u8, u8, u8)) -> FindResult {
    let mut result = FindResult::default();

    // Automatically select optimal implementation
    #[cfg(target_arch = "x86_64")]
    let found = {
        if std::is_x86_feature_detected!("avx2") {
            unsafe { find_color_avx2(colors, w, target) }
        } else {
            find_color_scalar(colors, w, target)
        }
    };

    #[cfg(not(target_arch = "x86_64"))]
    let found = find_color_scalar(colors, w, target);

    if let Some((local_x, local_y)) = found {
        result.matched = true;
        result.x = local_x;
        result.y = local_y;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a BGRA pixel array from RGB values
    fn create_bgra_pixel(r: u8, g: u8, b: u8, a: u8) -> [u8; 4] {
        [b, g, r, a] // BGRA format
    }

    /// Helper function to create a solid color image buffer
    fn create_solid_image(width: i32, height: i32, color: (u8, u8, u8)) -> Vec<u8> {
        let mut buffer = Vec::with_capacity((width * height * 4) as usize);
        for _ in 0..(width * height) {
            buffer.extend_from_slice(&create_bgra_pixel(color.2, color.1, color.0, 255));
        }
        buffer
    }

    #[test]
    fn test_find_color_basic() {
        let width = 100i32;
        let height = 100i32;
        let bg_color = (100, 100, 100);
        let target_color = (255, 0, 0); // Red

        let mut buffer = create_solid_image(width, height, bg_color);

        // Place target at position (50, 50)
        let target_x = 50;
        let target_y = 50;
        let target_idx = ((target_y * width + target_x) * 4) as usize;

        buffer[target_idx] = target_color.0; // B
        buffer[target_idx + 1] = target_color.1; // G
        buffer[target_idx + 2] = target_color.2; // R
        buffer[target_idx + 3] = 255; // A

        let result = find_color_in_buffer(&buffer, width, height, target_color);

        assert!(result.matched, "Should find the color");
        assert_eq!(result.x, target_x, "X coordinate mismatch");
        assert_eq!(result.y, target_y, "Y coordinate mismatch");
    }

    #[test]
    fn test_find_color_not_found() {
        let width = 100i32;
        let height = 100i32;
        let bg_color = (100, 100, 100);

        let buffer = create_solid_image(width, height, bg_color);

        let result = find_color_in_buffer(&buffer, width, height, (255, 0, 0));

        assert!(!result.matched, "Should not find the color");
    }
}
