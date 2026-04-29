//! Clipboard operations module
//!
//! Provides clipboard read/write capabilities using Windows API.
//! This module is feature-gated and only available when the `clipboard` feature is enabled.
//!
//! # Features
//! - **Read text**: Get text content from clipboard
//! - **Write text**: Set text content to clipboard
//! - **Clear**: Clear clipboard content
//!
//! # Quick Start
//! ```no_run
//! use win_auto_utils::clipboard;
//!
//! // Write text to clipboard
//! clipboard::set_text("Hello, World!").unwrap();
//!
//! // Read text from clipboard
//! let text = clipboard::get_text().unwrap();
//! println!("Clipboard content: {}", text);
//!
//! // Clear clipboard
//! clipboard::clear().unwrap();
//! ```
//!
//! # Error Handling
//! All operations return `Result<T, ClipboardError>` for proper error handling.
//! Common errors include:
//! - Failed to open clipboard
//! - Failed to allocate global memory
//! - Failed to lock/unlock memory
//! - Unsupported clipboard format

use windows::{
    Win32::Foundation::HWND,
    Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
    },
    Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND},
};

/// Clipboard format constant for Unicode text
const CF_UNICODETEXT: u32 = 13;

/// Clipboard operation errors
#[derive(Debug)]
pub enum ClipboardError {
    /// Failed to open clipboard
    OpenFailed,
    /// Failed to close clipboard
    CloseFailed,
    /// Failed to clear clipboard
    ClearFailed,
    /// Failed to allocate global memory
    MemoryAllocationFailed,
    /// Failed to lock global memory
    MemoryLockFailed,
    /// Failed to unlock global memory
    MemoryUnlockFailed,
    /// Failed to set clipboard data
    SetDataFailed,
    /// Failed to get clipboard data
    GetDataFailed,
    /// Clipboard data is not in text format
    InvalidFormat,
    /// Failed to convert string (contains null bytes)
    StringConversionError,
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardError::OpenFailed => write!(f, "Failed to open clipboard"),
            ClipboardError::CloseFailed => write!(f, "Failed to close clipboard"),
            ClipboardError::ClearFailed => write!(f, "Failed to clear clipboard"),
            ClipboardError::MemoryAllocationFailed => {
                write!(f, "Failed to allocate global memory")
            }
            ClipboardError::MemoryLockFailed => write!(f, "Failed to lock global memory"),
            ClipboardError::MemoryUnlockFailed => write!(f, "Failed to unlock global memory"),
            ClipboardError::SetDataFailed => write!(f, "Failed to set clipboard data"),
            ClipboardError::GetDataFailed => write!(f, "Failed to get clipboard data"),
            ClipboardError::InvalidFormat => write!(f, "Clipboard data is not in text format"),
            ClipboardError::StringConversionError => {
                write!(f, "String conversion error (contains null bytes)")
            }
        }
    }
}

impl std::error::Error for ClipboardError {}

/// Set text to clipboard
///
/// # Arguments
/// * `text` - The text content to set to clipboard
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(ClipboardError)` on failure
///
/// # Example
/// ```no_run
/// use win_auto_utils::clipboard;
///
/// clipboard::set_text("Hello, World!").unwrap();
/// ```
pub fn set_text(text: &str) -> Result<(), ClipboardError> {
    // Open clipboard with NULL owner (current thread)
    unsafe {
        if OpenClipboard(Some(HWND::default())).is_err() {
            return Err(ClipboardError::OpenFailed);
        }
    }

    // Clear existing clipboard content
    unsafe {
        if EmptyClipboard().is_err() {
            CloseClipboard().ok();
            return Err(ClipboardError::ClearFailed);
        }
    }

    // Convert Rust string to UTF-16 with null terminator
    let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let byte_size = utf16.len() * std::mem::size_of::<u16>();

    // Allocate global memory
    let h_mem = unsafe { GlobalAlloc(GHND, byte_size) };
    if h_mem.is_err() || h_mem.as_ref().unwrap().0.is_null() {
        unsafe {
            CloseClipboard().ok();
        }
        return Err(ClipboardError::MemoryAllocationFailed);
    }
    let h_mem = h_mem.unwrap();

    // Lock memory and copy data
    unsafe {
        let locked_ptr = GlobalLock(h_mem);
        if locked_ptr.is_null() {
            CloseClipboard().ok();
            return Err(ClipboardError::MemoryLockFailed);
        }

        // Copy UTF-16 data to global memory
        std::ptr::copy_nonoverlapping(utf16.as_ptr(), locked_ptr as *mut u16, utf16.len());

        // Unlock memory before setting clipboard data
        // Note: We must unlock before SetClipboardData transfers ownership
        if GlobalUnlock(h_mem).is_err() {
            // GlobalUnlock may return error if lock count is already 0, which is OK
            // Check if it's a real error or just "already unlocked"
            let last_error = windows::Win32::Foundation::GetLastError().0;
            if last_error != 0 && last_error != 6 {  // ERROR_INVALID_HANDLE = 6
                CloseClipboard().ok();
                return Err(ClipboardError::MemoryUnlockFailed);
            }
        }
    }

    // Set clipboard data (convert HGLOBAL to HANDLE)
    unsafe {
        let handle = windows::Win32::Foundation::HANDLE(h_mem.0);
        if SetClipboardData(CF_UNICODETEXT, Some(handle)).is_err() {
            CloseClipboard().ok();
            return Err(ClipboardError::SetDataFailed);
        }
    }

    // Close clipboard (memory handle is now owned by clipboard)
    unsafe {
        if CloseClipboard().is_err() {
            return Err(ClipboardError::CloseFailed);
        }
    }

    Ok(())
}

/// Get text from clipboard
///
/// # Returns
/// * `Ok(String)` with clipboard text content on success
/// * `Err(ClipboardError)` on failure
///
/// # Example
/// ```no_run
/// use win_auto_utils::clipboard;
///
/// let text = clipboard::get_text().unwrap();
/// println!("Clipboard: {}", text);
/// ```
pub fn get_text() -> Result<String, ClipboardError> {
    // Open clipboard
    unsafe {
        if OpenClipboard(Some(HWND::default())).is_err() {
            return Err(ClipboardError::OpenFailed);
        }
    }

    // Get clipboard data
    let h_data = unsafe { GetClipboardData(CF_UNICODETEXT) };
    if h_data.is_err() || h_data.as_ref().unwrap().0.is_null() {
        unsafe {
            CloseClipboard().ok();
        }
        return Err(ClipboardError::GetDataFailed);
    }
    let h_data_handle = h_data.unwrap();
    // Convert HANDLE to HGLOBAL (they are compatible types)
    let h_data = windows::Win32::Foundation::HGLOBAL(h_data_handle.0);

    // Lock memory to read data
    let text = unsafe {
        let locked_ptr = GlobalLock(h_data);
        if locked_ptr.is_null() {
            CloseClipboard().ok();
            return Err(ClipboardError::MemoryLockFailed);
        }

        // Read UTF-16 string until null terminator
        let mut utf16_chars = Vec::new();
        let mut ptr = locked_ptr as *const u16;
        loop {
            let ch = *ptr;
            if ch == 0 {
                break;
            }
            utf16_chars.push(ch);
            ptr = ptr.add(1);
        }

        // Unlock memory (don't free - clipboard owns it)
        if GlobalUnlock(h_data).is_err() {
            CloseClipboard().ok();
            return Err(ClipboardError::MemoryUnlockFailed);
        }

        // Convert UTF-16 to Rust String
        match String::from_utf16(&utf16_chars) {
            Ok(s) => s,
            Err(_) => {
                CloseClipboard().ok();
                return Err(ClipboardError::StringConversionError);
            }
        }
    };

    // Close clipboard
    unsafe {
        if CloseClipboard().is_err() {
            return Err(ClipboardError::CloseFailed);
        }
    }

    Ok(text)
}

/// Clear clipboard content
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(ClipboardError)` on failure
///
/// # Example
/// ```no_run
/// use win_auto_utils::clipboard;
///
/// clipboard::clear().unwrap();
/// ```
pub fn clear() -> Result<(), ClipboardError> {
    // Open clipboard
    unsafe {
        if OpenClipboard(Some(HWND::default())).is_err() {
            return Err(ClipboardError::OpenFailed);
        }
    }

    // Clear clipboard
    unsafe {
        if EmptyClipboard().is_err() {
            CloseClipboard().ok();
            return Err(ClipboardError::ClearFailed);
        }
    }

    // Close clipboard
    unsafe {
        if CloseClipboard().is_err() {
            return Err(ClipboardError::CloseFailed);
        }
    }

    Ok(())
}

/// Check if clipboard contains text data
///
/// # Returns
/// * `true` if clipboard has text content
/// * `false` otherwise or on error
///
/// # Example
/// ```no_run
/// use win_auto_utils::clipboard;
///
/// if clipboard::has_text() {
///     let text = clipboard::get_text().unwrap();
///     println!("Got text: {}", text);
/// }
/// ```
pub fn has_text() -> bool {
    unsafe {
        if OpenClipboard(Some(HWND::default())).is_err() {
            return false;
        }

        let has_text = GetClipboardData(CF_UNICODETEXT)
            .map(|h| !h.0.is_null())
            .unwrap_or(false);

        CloseClipboard().ok();
        has_text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get_text() {
        let test_text = "Hello, Clipboard!";
        
        // Set text
        let result = set_text(test_text);
        assert!(result.is_ok(), "Failed to set text: {:?}", result);
        
        // Get text
        let retrieved = get_text().expect("Failed to get text");
        assert_eq!(retrieved, test_text);
    }

    #[test]
    fn test_clear_clipboard() {
        // Set some text first
        set_text("Test text").ok();
        
        // Clear it
        assert!(clear().is_ok(), "Failed to clear clipboard");
        
        // Note: After clearing, has_text might still return true on some systems
        // because other processes might have set clipboard content
    }

    #[test]
    fn test_unicode_text() {
        let unicode_text = "你好世界！🌍 こんにちは";
        
        let result = set_text(unicode_text);
        assert!(result.is_ok(), "Failed to set unicode text: {:?}", result);
        
        let retrieved = get_text().expect("Failed to get unicode text");
        assert_eq!(retrieved, unicode_text);
    }

    #[test]
    fn test_empty_string() {
        let result = set_text("");
        assert!(result.is_ok(), "Failed to set empty string: {:?}", result);
        
        let retrieved = get_text().expect("Failed to get empty string");
        assert_eq!(retrieved, "");
    }

    #[test]
    fn test_long_text() {
        let long_text = "A".repeat(10000);
        
        let result = set_text(&long_text);
        assert!(result.is_ok(), "Failed to set long text: {:?}", result);
        
        let retrieved = get_text().expect("Failed to get long text");
        assert_eq!(retrieved, long_text);
    }
}
