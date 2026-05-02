//! Process instance module
//!
//! Defines the runtime state layer for process management.
//! Instances hold the actual system resources (handles, DCs) and manage their lifecycle.

use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND};
use windows::Win32::Graphics::Gdi::HDC;

/// Process runtime state
///
/// Contains all resources acquired during initialization.
/// Resources are optional based on InitFlags configuration.
/// Automatically cleaned up when dropped.
#[derive(Debug)]
pub struct ProcessState {
    /// Process ID (always present if initialized)
    pub pid: u32,
    /// Window handle (optional based on InitFlags)
    pub hwnd: Option<HWND>,
    /// Process handle (optional based on InitFlags)
    pub handle: Option<HANDLE>,
    /// Device context (optional based on InitFlags)
    pub hdc: Option<HDC>,
}

impl ProcessState {
    /// Check if the state is valid (PID is set and at least one resource is available)
    pub fn is_valid(&self) -> bool {
        self.pid != 0 && (self.hwnd.is_some() || self.handle.is_some() || self.hdc.is_some())
    }

    /// Check if window handle is available
    pub fn has_hwnd(&self) -> bool {
        self.hwnd.is_some()
    }

    /// Check if process handle is available
    pub fn has_handle(&self) -> bool {
        self.handle.is_some()
    }

    /// Check if device context is available
    pub fn has_dc(&self) -> bool {
        self.hdc.is_some()
    }
}

impl Drop for ProcessState {
    fn drop(&mut self) {
        // Release DC if present
        if let Some(hdc) = self.hdc.take() {
            if !hdc.is_invalid() {
                unsafe {
                    let _ = windows::Win32::Graphics::Gdi::DeleteDC(hdc);
                }
            }
        }

        // Close process handle if present
        if let Some(handle) = self.handle.take() {
            if !handle.is_invalid() {
                unsafe {
                    let _ = CloseHandle(handle);
                }
            }
        }
    }
}
