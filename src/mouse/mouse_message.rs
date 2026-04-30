//! PostMessage-based mouse input (atomic high-performance API)
//!
//! This module provides minimal, zero-overhead functions for mouse input.
//! All functions accept pre-compiled coordinates with no object creation overhead.
//!
//! For user-friendly APIs, see the convenience wrapper at the end of this file.

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    PostMessageW, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP,
    WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL,
};

// ============================================================================
// Atomic High-Performance Functions (Zero Overhead)
// ============================================================================

/// Helper function to pack coordinates into LPARAM
/// Low word = x, High word = y
#[inline]
pub fn make_lparam(x: i32, y: i32) -> LPARAM {
    LPARAM(((y as u32) << 16 | (x as u32 & 0xFFFF)) as isize)
}

/// Send left button click atomically
/// 
/// # Performance
/// Two direct PostMessage calls, no allocations or lookups
#[inline]
pub fn post_click_left_atomic(hwnd: HWND, x: i32, y: i32) {
    let lparam = make_lparam(x, y);
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_LBUTTONDOWN, WPARAM(1), lparam);
        let _ = PostMessageW(Some(hwnd), WM_LBUTTONUP, WPARAM(0), lparam);
    }
}

/// Send right button click atomically
#[inline]
pub fn post_click_right_atomic(hwnd: HWND, x: i32, y: i32) {
    let lparam = make_lparam(x, y);
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_RBUTTONDOWN, WPARAM(1), lparam);
        let _ = PostMessageW(Some(hwnd), WM_RBUTTONUP, WPARAM(0), lparam);
    }
}

/// Send middle button click atomically
#[inline]
pub fn post_click_middle_atomic(hwnd: HWND, x: i32, y: i32) {
    let lparam = make_lparam(x, y);
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_MBUTTONDOWN, WPARAM(1), lparam);
        let _ = PostMessageW(Some(hwnd), WM_MBUTTONUP, WPARAM(0), lparam);
    }
}

/// Send left button press atomically
#[inline]
pub fn post_press_left_atomic(hwnd: HWND, x: i32, y: i32) {
    let lparam = make_lparam(x, y);
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_LBUTTONDOWN, WPARAM(1), lparam);
    }
}

/// Send left button release atomically
#[inline]
pub fn post_release_left_atomic(hwnd: HWND, x: i32, y: i32) {
    let lparam = make_lparam(x, y);
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_LBUTTONUP, WPARAM(0), lparam);
    }
}

/// Send mouse move atomically
#[inline]
pub fn post_move_atomic(hwnd: HWND, x: i32, y: i32) {
    let lparam = make_lparam(x, y);
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_MOUSEMOVE, WPARAM(0), lparam);
    }
}

/// Send scroll up atomically
/// 
/// # Parameters
/// * `hwnd` - Target window handle
/// * `x`, `y` - Coordinates where scroll occurs
/// * `delta` - Scroll amount (positive for up, typically 120 per notch)
#[inline]
pub fn post_scroll_up_atomic(hwnd: HWND, x: i32, y: i32, delta: i32) {
    let lparam = make_lparam(x, y);
    let wparam = ((delta as u32) << 16) as usize; // HIWORD = delta
    
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_MOUSEWHEEL, WPARAM(wparam), lparam);
    }
}

/// Send scroll down atomically
#[inline]
pub fn post_scroll_down_atomic(hwnd: HWND, x: i32, y: i32, delta: i32) {
    post_scroll_up_atomic(hwnd, x, y, -delta)
}

// ============================================================================
// Convenience Types and Errors
// ============================================================================

/// PostMessage mouse input errors
#[derive(Debug)]
pub enum PostMessageMouseError {
    InvalidWindow,
    SendMessageFailed,
}

impl std::fmt::Display for PostMessageMouseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostMessageMouseError::InvalidWindow => write!(f, "Invalid window handle"),
            PostMessageMouseError::SendMessageFailed => write!(f, "Failed to send message"),
        }
    }
}

impl std::error::Error for PostMessageMouseError {}

// ============================================================================
// Convenience Wrapper (User-Friendly API)
// ============================================================================

/// User-friendly PostMessage mouse controller
/// 
/// Provides instance-based API for convenience.
/// For maximum performance, use the atomic functions directly.
pub struct PostMessageMouse {
    hwnd: HWND,
}

impl PostMessageMouse {
    /// Create a new PostMessage mouse controller
    pub fn new(hwnd: HWND) -> Self {
        Self { hwnd }
    }

    /// Get the target window handle
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Click left button at specified coordinates
    pub fn click_left(&self, x: i32, y: i32) -> Result<(), PostMessageMouseError> {
        post_click_left_atomic(self.hwnd, x, y);
        Ok(())
    }

    /// Click right button at specified coordinates
    pub fn click_right(&self, x: i32, y: i32) -> Result<(), PostMessageMouseError> {
        post_click_right_atomic(self.hwnd, x, y);
        Ok(())
    }

    /// Click middle button at specified coordinates
    pub fn click_middle(&self, x: i32, y: i32) -> Result<(), PostMessageMouseError> {
        post_click_middle_atomic(self.hwnd, x, y);
        Ok(())
    }

    /// Press left button at specified coordinates
    pub fn press_left(&self, x: i32, y: i32) -> Result<(), PostMessageMouseError> {
        post_press_left_atomic(self.hwnd, x, y);
        Ok(())
    }

    /// Release left button at specified coordinates
    pub fn release_left(&self, x: i32, y: i32) -> Result<(), PostMessageMouseError> {
        post_release_left_atomic(self.hwnd, x, y);
        Ok(())
    }

    /// Move mouse to specified coordinates
    pub fn move_to(&self, x: i32, y: i32) -> Result<(), PostMessageMouseError> {
        post_move_atomic(self.hwnd, x, y);
        Ok(())
    }

    /// Scroll up at specified coordinates
    pub fn scroll_up(&self, x: i32, y: i32, delta: i32) -> Result<(), PostMessageMouseError> {
        post_scroll_up_atomic(self.hwnd, x, y, delta);
        Ok(())
    }

    /// Scroll down at specified coordinates
    pub fn scroll_down(&self, x: i32, y: i32, delta: i32) -> Result<(), PostMessageMouseError> {
        post_scroll_down_atomic(self.hwnd, x, y, delta);
        Ok(())
    }
}
