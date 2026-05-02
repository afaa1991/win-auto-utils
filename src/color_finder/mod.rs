//! Color finder module - Screen region color searching
//!
//! Provides color searching functionality within pixel buffers.
//! This module contains pure Rust algorithms that can work with any pixel data source.
//!
//! # Features
//! - Fast color matching in pixel buffers
//! - Automatic AVX2 detection and optimization
//! - Falls back to scalar implementation when AVX2 is unavailable
//! - Returns coordinates within the buffer
//! - Integrated with DXGI for screen capture (automatically enabled)
//!
//! # Example (with pixel buffer)
//! ```no_run
//! use win_auto_utils::color_finder::algorithms::{find_color_in_buffer, FindResult};
//!
//! // Assume you have BGRA pixel data from some source
//! let pixels: Vec<u8> = vec![0; 100 * 100 * 4]; // 100x100 image
//!
//! // Search for a specific color
//! let result = find_color_in_buffer(&pixels, 100, 100, (255, 0, 0));
//! if result.matched {
//!     println!("Found at local coordinates ({}, {})", result.x, result.y);
//! }
//! ```
//!
//! # Example (with screen capture - dxgi automatically enabled)
//! ```no_run
//! use win_auto_utils::color_finder::find_color;
//!
//! // Search for red color on screen at position (100, 100) with size 50x50
//! match find_color(100, 100, 50, 50, (0, 0, 255)) {
//!     Ok(result) => {
//!         if result.matched {
//!             println!("Found at screen coordinates ({}, {})", result.x, result.y);
//!         }
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```

use std::error::Error;

// Re-export algorithms module (pure Rust, always available)
pub mod algorithms;
pub use algorithms::{find_color_in_buffer, FindResult};

// dxgi feature is automatically enabled by color_finder, so we can always use it
use crate::dxgi::capture_region_bytes;

/// Find color in screen region using DXGI capture
///
/// Captures a screen region using DXGI and searches for the specified color.
/// Automatically chooses between AVX2 SIMD and scalar implementations based on
/// CPU capabilities.
///
/// # Arguments
/// * `x` - Horizontal coordinate of region's top-left corner (screen coordinates)
/// * `y` - Vertical coordinate of region's top-left corner (screen coordinates)
/// * `w` - Width of the search region in pixels
/// * `h` - Height of the search region in pixels
/// * `target` - Target color as (B, G, R) tuple
///
/// # Returns
/// * `Ok(FindResult)` - Contains match status and absolute screen coordinates
/// * `Err(Box<dyn Error>)` - If DXGI capture fails
///
/// # Coordinate System
/// - Input coordinates (`x`, `y`) are in absolute screen space
/// - Output coordinates in `FindResult` are also in absolute screen space
///
/// # Performance Notes
/// - In debug mode, prints timing information for DXGI capture
/// - AVX2 path is ~4-8x faster than scalar on supported hardware
/// - First call may be slower due to DXGI initialization
pub fn find_color(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    target: (u8, u8, u8),
) -> Result<FindResult, Box<dyn Error>> {
    // Capture region color data using DXGI (BGRA format)
    let colors = capture_region_bytes(x, y, w, h)?;

    // Use the pure algorithm module to find color
    let mut result = find_color_in_buffer(&colors, w, h, target);

    // Convert local coordinates to absolute screen coordinates
    if result.matched {
        result.x += x;
        result.y += y;
    }

    Ok(result)
}
