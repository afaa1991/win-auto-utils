//! Diagnostic Tools for DLL Injection

use std::ffi::c_void;

use windows::Win32::{
    Foundation::CloseHandle,
    System::{
        Diagnostics::Debug::WriteProcessMemory,
        LibraryLoader::GetModuleHandleA,
        Memory::{VirtualAllocEx, VirtualFreeEx, MEM_RELEASE, PAGE_READWRITE},
    },
};

use super::helpers::open_process_full;
use super::DllInjectorError;

/// Diagnostic report for DLL injection feasibility
#[derive(Debug)]
pub struct DiagnosticReport {
    /// Target process ID
    pub pid: u32,
    /// Can the process be opened with PROCESS_ALL_ACCESS?
    pub process_accessible: bool,
    /// Is kernel32.dll loaded in the target?
    pub kernel32_loaded: bool,
    /// Can we allocate memory in the target process?
    pub memory_allocatable: bool,
    /// Can we write to allocated memory in the target?
    pub memory_writable: bool,
}

impl DiagnosticReport {
    /// Check if injection is likely to succeed
    pub fn is_injection_feasible(&self) -> bool {
        self.process_accessible
            && self.kernel32_loaded
            && self.memory_allocatable
            && self.memory_writable
    }

    /// Get human-readable summary
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "🔍 Injection Diagnostic Report for PID {}:",
            self.pid
        ));
        lines.push(String::from("─────────────────────────────────────────"));

        lines.push(format!(
            "Process Accessible:      {}",
            if self.process_accessible {
                "✅ Yes"
            } else {
                "❌ No"
            }
        ));

        lines.push(format!(
            "kernel32.dll Loaded:     {}",
            if self.kernel32_loaded {
                "✅ Yes"
            } else {
                "❌ No"
            }
        ));

        lines.push(format!(
            "Memory Allocatable:      {}",
            if self.memory_allocatable {
                "✅ Yes"
            } else {
                "❌ No"
            }
        ));

        lines.push(format!(
            "Memory Writable:         {}",
            if self.memory_writable {
                "✅ Yes"
            } else {
                "❌ No"
            }
        ));

        lines.push(String::from("─────────────────────────────────────────"));

        if self.is_injection_feasible() {
            lines.push(String::from("✅ Injection should succeed!"));
        } else {
            lines.push(String::from("❌ Injection will likely fail. Issues:"));

            if !self.process_accessible {
                lines.push(String::from(
                    "   • Cannot open process - Run as Administrator",
                ));
            }
            if !self.kernel32_loaded {
                lines.push(String::from(
                    "   • kernel32.dll not found - Process may be protected",
                ));
            }
            if !self.memory_allocatable {
                lines.push(String::from(
                    "   • Cannot allocate memory - Insufficient permissions",
                ));
            }
            if !self.memory_writable {
                lines.push(String::from(
                    "   • Cannot write memory - Anti-injection protection?",
                ));
            }
        }

        lines.join("\n")
    }
}

/// Perform diagnostic checks before attempting DLL injection
///
/// # Arguments
/// - `pid`: Target process ID to diagnose
///
/// # Returns
/// - `Ok(DiagnosticReport)` with detailed feasibility information
/// - `Err(DllInjectorError)` on critical failure
///
/// # Example
/// ```no_run
/// use win_auto_utils::dll_injector::diagnose_injection;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let report = diagnose_injection(12345)?;
///     println!("{}", report.summary());
///
///     if report.is_injection_feasible() {
///         println!("Safe to proceed with injection");
///     }
///     Ok(())
/// }
/// ```

pub fn diagnose_injection(pid: u32) -> Result<DiagnosticReport, DllInjectorError> {
    let mut report = DiagnosticReport {
        pid,
        process_accessible: false,
        kernel32_loaded: false,
        memory_allocatable: false,
        memory_writable: false,
    };

    // Check 1: Can we open the process?
    match open_process_full(pid) {
        Ok(handle) => {
            report.process_accessible = true;

            unsafe {
                // Check 2: Is kernel32.dll accessible?
                if GetModuleHandleA(windows::core::PCSTR::from_raw("kernel32.dll\0".as_ptr()))
                    .is_ok()
                {
                    report.kernel32_loaded = true;
                }

                // Check 3: Can we allocate and write memory?
                let test_size = 64;
                let test_addr = VirtualAllocEx(
                    handle,
                    None,
                    test_size,
                    windows::Win32::System::Memory::MEM_COMMIT,
                    PAGE_READWRITE,
                );

                if !test_addr.is_null() {
                    report.memory_allocatable = true;

                    // Try writing
                    let test_data: u32 = 0xDEADBEEF;
                    let mut bytes_written = 0;
                    let write_ok = WriteProcessMemory(
                        handle,
                        test_addr,
                        &test_data as *const u32 as *const c_void,
                        4,
                        Some(&mut bytes_written),
                    )
                    .is_ok();

                    report.memory_writable = write_ok && bytes_written == 4;

                    // Cleanup
                    let _ = VirtualFreeEx(handle, test_addr, 0, MEM_RELEASE);
                }

                let _ = CloseHandle(handle);
            }
        }
        Err(_) => {
            report.process_accessible = false;
        }
    }

    Ok(report)
}
