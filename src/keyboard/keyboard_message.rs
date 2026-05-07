//! PostMessage-based keyboard input (atomic high-performance API)
//!
//! This module provides minimal, zero-overhead functions for keyboard input.
//! All functions accept pre-compiled parameters (vk_code, scan_code) with no string lookup.

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_KEYDOWN, WM_KEYUP};

use crate::get_scan_code;
use crate::utils::key_code;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug)]
pub enum PostMessageKeyBoardError {
    SendMessageFailed,
}

impl std::fmt::Display for PostMessageKeyBoardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostMessageKeyBoardError::SendMessageFailed => write!(f, "PostMessage failed"),
        }
    }
}

impl std::error::Error for PostMessageKeyBoardError {}

// ============================================================================
// Atomic High-Performance Functions
// ============================================================================

#[inline]
pub fn make_lparam(scan_code: u16, is_keyup: bool) -> LPARAM {
    let mut lparam = ((scan_code as u32) << 16) as i32;

    if is_keyup {
        lparam |= 0xC0000000u32 as i32;
    } else {
        lparam |= 0x00000001;
    }

    LPARAM(lparam as isize)
}

#[inline]
pub fn post_key_down_atomic(hwnd: HWND, vk_code: u8, scan_code: u16) -> Result<(), PostMessageKeyBoardError> {
    let lparam = make_lparam(scan_code, false);
    unsafe {
        PostMessageW(Some(hwnd), WM_KEYDOWN, WPARAM(vk_code as usize), lparam)
            .map_err(|_| PostMessageKeyBoardError::SendMessageFailed)?;
    }
    Ok(())
}

#[inline]
pub fn post_key_up_atomic(hwnd: HWND, vk_code: u8, scan_code: u16) -> Result<(), PostMessageKeyBoardError> {
    let lparam = make_lparam(scan_code, true);
    unsafe {
        PostMessageW(Some(hwnd), WM_KEYUP, WPARAM(vk_code as usize), lparam)
            .map_err(|_| PostMessageKeyBoardError::SendMessageFailed)?;
    }
    Ok(())
}

#[inline]
pub fn post_key_click_atomic(hwnd: HWND, vk_code: u8, scan_code: u16) -> Result<(), PostMessageKeyBoardError> {
    post_key_down_atomic(hwnd, vk_code, scan_code)?;
    post_key_up_atomic(hwnd, vk_code, scan_code)?;
    Ok(())
}

// ============================================================================
// Convenience Wrapper (User-Friendly API)
// ============================================================================

#[derive(Debug)]
pub enum PostMessageKeyBoardErrorWrapper {
    InvalidWindow,
    SendMessageFailed,
}

impl std::fmt::Display for PostMessageKeyBoardErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostMessageKeyBoardErrorWrapper::InvalidWindow => write!(f, "Invalid window handle"),
            PostMessageKeyBoardErrorWrapper::SendMessageFailed => write!(f, "PostMessage failed"),
        }
    }
}

impl std::error::Error for PostMessageKeyBoardErrorWrapper {}

#[derive(Debug)]
pub struct PostMessageKeyboard {
    pub hwnd: HWND,
}

impl PostMessageKeyboard {
    #[inline]
    pub fn new(hwnd: HWND) -> Self {
        Self { hwnd }
    }

    #[inline]
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    #[inline]
    pub fn press(&self, key: &str) -> Result<(), PostMessageKeyBoardErrorWrapper> {
        let vk = key_code(key);
        if vk == 0 {
            return Err(PostMessageKeyBoardErrorWrapper::InvalidWindow);
        }
        let scan_code = get_scan_code(vk);
        post_key_down_atomic(self.hwnd, vk as u8, scan_code)
            .map_err(|_| PostMessageKeyBoardErrorWrapper::SendMessageFailed)?;
        Ok(())
    }

    #[inline]
    pub fn release(&self, key: &str) -> Result<(), PostMessageKeyBoardErrorWrapper> {
        let vk = key_code(key);
        if vk == 0 {
            return Err(PostMessageKeyBoardErrorWrapper::InvalidWindow);
        }
        let scan_code = get_scan_code(vk);
        post_key_up_atomic(self.hwnd, vk as u8, scan_code)
            .map_err(|_| PostMessageKeyBoardErrorWrapper::SendMessageFailed)?;
        Ok(())
    }

    #[inline]
    pub fn click(&self, key: &str) -> Result<(), PostMessageKeyBoardErrorWrapper> {
        let vk = key_code(key);
        if vk == 0 {
            return Err(PostMessageKeyBoardErrorWrapper::InvalidWindow);
        }
        let scan_code = get_scan_code(vk);
        post_key_click_atomic(self.hwnd, vk as u8, scan_code)
            .map_err(|_| PostMessageKeyBoardErrorWrapper::SendMessageFailed)?;
        Ok(())
    }

    #[inline]
    pub fn press_with_codes(&self, vk_code: u8, scan_code: u16) -> Result<(), PostMessageKeyBoardErrorWrapper> {
        post_key_down_atomic(self.hwnd, vk_code, scan_code)
            .map_err(|_| PostMessageKeyBoardErrorWrapper::SendMessageFailed)?;
        Ok(())
    }

    #[inline]
    pub fn release_with_codes(&self, vk_code: u8, scan_code: u16) -> Result<(), PostMessageKeyBoardErrorWrapper> {
        post_key_up_atomic(self.hwnd, vk_code, scan_code)
            .map_err(|_| PostMessageKeyBoardErrorWrapper::SendMessageFailed)?;
        Ok(())
    }

    #[inline]
    pub fn click_with_codes(&self, vk_code: u8, scan_code: u16) -> Result<(), PostMessageKeyBoardErrorWrapper> {
        post_key_click_atomic(self.hwnd, vk_code, scan_code)
            .map_err(|_| PostMessageKeyBoardErrorWrapper::SendMessageFailed)?;
        Ok(())
    }
}
