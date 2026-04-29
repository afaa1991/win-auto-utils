//! Color picker module - Screen pixel reading functionality
//!
//! Provides the ability to read pixel colors from screen or window device contexts.
//! This module depends on Windows GDI API for capturing pixel data.
//!
//! # Features
//! - Read pixel colors from device context (HDC)
//! - Supports both window DC and desktop DC
//!
//! # Example
//! ```no_run
//! use win_auto_utils::hdc::get_window_client_dc;
//! use win_auto_utils::hwnd::get_hwnd_by_title;
//! use win_auto_utils::color_picker::get_pixel_color;
//!
//! if let Some(hwnd) = get_hwnd_by_title("Notepad") {
//!     if let Ok(hdc) = get_window_client_dc(hwnd) {
//!         if let Some(color) = get_pixel_color(hdc, 100, 100) {
//!             println!("Pixel color at (100, 100): 0x{:06X}", color);
//!         }
//!     }
//! }
//! ```

use windows::Win32::Graphics::Gdi::{GetPixel, HDC};

/// Get pixel color from device context at specified coordinates
///
/// Reads the color value of a pixel at the given (x, y) position from the device context.
/// The returned value is in Windows COLORREF format (0x00BBGGRR).
///
/// # Arguments
/// * `hdc` - Device context handle to read from
/// * `x` - X coordinate (horizontal position)
/// * `y` - Y coordinate (vertical position)
///
/// # Returns
/// * `Some(u32)` - The pixel color in BGR format (0x00BBGGRR) if successful
/// * `None` - If the pixel cannot be read (e.g., coordinates out of bounds)
pub fn get_pixel_color(hdc: HDC, x: i32, y: i32) -> Option<u32> {
    let color_ref = unsafe { GetPixel(hdc, x, y) };

    // COLORREF is represented as u32 in windows-rs
    // 0xFFFFFFFF indicates failure
    if color_ref.0 == 0xFFFFFFFF {
        return None;
    }

    // COLORREF format is 0x00BBGGRR
    Some(color_ref.0)
}
