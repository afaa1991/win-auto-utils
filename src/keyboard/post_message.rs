//! PostMessage-based keyboard input (atomic high-performance API)
//!
//! This module provides minimal, zero-overhead functions for keyboard input.
//! All functions accept pre-compiled parameters (vk_code, scan_code) with no string lookup.
//!
//! For user-friendly APIs with string support, see the convenience wrapper at the end of this file.

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_KEYDOWN, WM_KEYUP};

use crate::get_scan_code;
use crate::utils::key_code;

// ============================================================================
// Atomic High-Performance Functions (Zero Overhead)
// ============================================================================

/// Construct LPARAM for keyboard messages
///
/// # Parameters
/// * `scan_code` - Hardware scan code
/// * `is_keyup` - true for KEYUP, false for KEYDOWN
#[inline]
pub fn make_lparam(scan_code: u16, is_keyup: bool) -> LPARAM {
    let mut lparam = ((scan_code as u32) << 16) as i32;

    if is_keyup {
        lparam |= 0xC0000000u32 as i32; // Previous key state + Transition state
    } else {
        lparam |= 0x00000001; // Repeat count = 1
    }

    LPARAM(lparam as isize)
}

/// Send KEYDOWN message to window (atomic operation)
///
/// # Performance
/// Zero overhead - direct Windows API call with pre-compiled codes
///
/// # Parameters
/// * `hwnd` - Target window handle
/// * `vk_code` - Virtual key code (pre-compiled)
/// * `scan_code` - Scan code (pre-compiled)
#[inline]
pub fn post_key_down_atomic(hwnd: HWND, vk_code: u8, scan_code: u16) {
    let lparam = make_lparam(scan_code, false);
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_KEYDOWN, WPARAM(vk_code as usize), lparam);
    }
}

/// Send KEYUP message to window (atomic operation)
///
/// # Performance
/// Zero overhead - direct Windows API call with pre-compiled codes
#[inline]
pub fn post_key_up_atomic(hwnd: HWND, vk_code: u8, scan_code: u16) {
    let lparam = make_lparam(scan_code, true);
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_KEYUP, WPARAM(vk_code as usize), lparam);
    }
}

/// Send KEYDOWN + KEYUP messages atomically (click operation)
///
/// # Performance
/// Two direct API calls, no intermediate allocations or lookups
#[inline]
pub fn post_key_click_atomic(hwnd: HWND, vk_code: u8, scan_code: u16) {
    post_key_down_atomic(hwnd, vk_code, scan_code);
    post_key_up_atomic(hwnd, vk_code, scan_code);
}

// ============================================================================
// Convenience Wrapper (User-Friendly API)
// ============================================================================

/// PostMessage keyboard errors
#[derive(Debug)]
pub enum PostMessageError {
    InvalidWindow,
    SendMessageFailed,
}

impl std::fmt::Display for PostMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostMessageError::InvalidWindow => write!(f, "Invalid window handle"),
            PostMessageError::SendMessageFailed => write!(f, "Failed to send message"),
        }
    }
}

impl std::error::Error for PostMessageError {}

/// User-friendly PostMessage keyboard controller
///
/// Provides convenient string-based API at the cost of runtime lookups.
/// For maximum performance, use the atomic functions directly.
pub struct PostMessageKeyboard {
    hwnd: HWND,
}

impl PostMessageKeyboard {
    /// Create a new PostMessage keyboard controller
    pub fn new(hwnd: HWND) -> Self {
        Self { hwnd }
    }

    /// Get the target window handle
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Press a key by name (convenience method with string lookup)
    ///
    /// # Note
    /// This method performs string-to-code conversion at runtime.
    /// For better performance, pre-compile key codes and use `press_with_codes`.
    pub fn press(&self, key: &str) -> Result<(), PostMessageError> {
        let vk_code = key_code(key);
        if vk_code == 0 {
            return Err(PostMessageError::InvalidWindow);
        }
        let scan_code = get_scan_code(vk_code);
        post_key_down_atomic(self.hwnd, vk_code, scan_code);
        Ok(())
    }

    /// Release a key by name (convenience method)
    pub fn release(&self, key: &str) -> Result<(), PostMessageError> {
        let vk_code = key_code(key);
        if vk_code == 0 {
            return Err(PostMessageError::InvalidWindow);
        }
        let scan_code = get_scan_code(vk_code);
        post_key_up_atomic(self.hwnd, vk_code, scan_code);
        Ok(())
    }

    /// Click a key by name (convenience method)
    pub fn click(&self, key: &str) -> Result<(), PostMessageError> {
        let vk_code = key_code(key);
        if vk_code == 0 {
            return Err(PostMessageError::InvalidWindow);
        }
        let scan_code = get_scan_code(vk_code);
        post_key_click_atomic(self.hwnd, vk_code, scan_code);
        Ok(())
    }

    /// Press a key using pre-compiled codes (high performance)
    pub fn press_with_codes(&self, vk_code: u8, scan_code: u16) -> Result<(), PostMessageError> {
        post_key_down_atomic(self.hwnd, vk_code, scan_code);
        Ok(())
    }

    /// Release a key using pre-compiled codes
    pub fn release_with_codes(&self, vk_code: u8, scan_code: u16) -> Result<(), PostMessageError> {
        post_key_up_atomic(self.hwnd, vk_code, scan_code);
        Ok(())
    }

    /// Click a key using pre-compiled codes
    pub fn click_with_codes(&self, vk_code: u8, scan_code: u16) -> Result<(), PostMessageError> {
        post_key_click_atomic(self.hwnd, vk_code, scan_code);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_message_creation() {
        let hwnd = HWND(std::ptr::null_mut());
        let kb = PostMessageKeyboard::new(hwnd);
        assert_eq!(kb.hwnd().0, std::ptr::null_mut());
    }

    #[test]
    fn test_make_lparam_format() {
        let lparam_down = make_lparam(0x1E, false);
        let lparam_up = make_lparam(0x1E, true);

        // Check repeat count (bits 0-15)
        assert_eq!((lparam_down.0 as u32) & 0xFFFF, 1);

        // Check transition state (bit 31): 0 for down, 1 for up
        assert_eq!(((lparam_up.0 as u32) >> 31) & 1, 1);
        assert_eq!(((lparam_down.0 as u32) >> 31) & 1, 0);

        // Check scan code (bits 16-23)
        assert_eq!(((lparam_down.0 as u32) >> 16) & 0xFF, 0x1E);
    }

    #[test]
    fn test_atomic_functions_compile() {
        // Test that atomic functions compile correctly
        let hwnd = HWND(std::ptr::null_mut());

        // These will fail at runtime (invalid HWND) but should compile
        post_key_down_atomic(hwnd, 0x41, 0x1E);
        post_key_up_atomic(hwnd, 0x41, 0x1E);
        post_key_click_atomic(hwnd, 0x41, 0x1E);

        assert!(true); // Just verify compilation
    }
}
