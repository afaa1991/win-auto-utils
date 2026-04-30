//! Process instance module
//!
//! Defines the runtime state layer for process management.
//! Instances hold the actual system resources (handles, DCs) and manage their lifecycle.

use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND};
use windows::Win32::Graphics::Gdi::HDC;

/// Process runtime state
///
/// Contains all resources acquired during initialization.
/// Automatically cleaned up when dropped.
#[derive(Debug)]
pub struct ProcessState {
    /// Process ID
    pub pid: u32,
    /// Window handle
    pub hwnd: HWND,
    /// Process handle
    pub handle: HANDLE,
    /// Device context
    pub hdc: HDC,
}

impl ProcessState {
    /// Check if the state is valid (all handles are non-null)
    pub fn is_valid(&self) -> bool {
        !self.handle.is_invalid() && !self.hwnd.is_invalid()
    }
}

impl Drop for ProcessState {
    fn drop(&mut self) {
        // Release DC
        if !self.hdc.is_invalid() {
            unsafe {
                let _ = windows::Win32::Graphics::Gdi::DeleteDC(self.hdc);
            }
        }

        // Close process handle
        if !self.handle.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.handle);
            }
        }
    }
}
