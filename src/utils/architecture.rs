//! Architecture detection utilities for Windows processes.
//!
//! Provides functions to detect whether a target process is x86 or x64,
//! which is essential for cross-architecture memory operations and hooking.

use crate::memory_hook::Architecture;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{IsWow64Process, OpenProcess, PROCESS_QUERY_INFORMATION};

/// Detect target process architecture using PID
///
/// Returns `Architecture::X64` for 64-bit processes, `Architecture::X86` for 32-bit.
/// Uses Windows API `IsWow64Process` for accurate detection.
///
/// # Arguments
/// * `pid` - Target process ID
///
/// # Returns
/// Detected architecture, defaults to host architecture if detection fails
///
/// # Example
/// ```no_run
/// use win_auto_utils::utils::detect_process_architecture;
/// use win_auto_utils::memory_hook::Architecture;
///
/// let arch = detect_process_architecture(12345);
/// match arch {
///     Architecture::X86 => println!("Target is 32-bit"),
///     Architecture::X64 => println!("Target is 64-bit"),
/// }
/// ```
pub fn detect_process_architecture(pid: u32) -> Architecture {
    unsafe {
        if let Ok(handle) = OpenProcess(PROCESS_QUERY_INFORMATION, false, pid) {
            let mut is_wow64: i32 = 0;

            if IsWow64Process(handle, &mut is_wow64 as *mut i32 as *mut _).is_ok() {
                let _ = CloseHandle(handle);

                return if is_wow64 != 0 {
                    Architecture::X86
                } else {
                    // Fallback based on host architecture
                    if cfg!(target_pointer_width = "32") {
                        Architecture::X86
                    } else {
                        Architecture::X64
                    }
                };
            }

            let _ = CloseHandle(handle);
        }

        // Fallback to host architecture
        if cfg!(target_pointer_width = "32") {
            Architecture::X86
        } else {
            Architecture::X64
        }
    }
}
