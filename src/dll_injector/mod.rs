//! DLL Injection Module
//!
//! Provides functionality to inject and unload DLLs into remote processes,
//! and call exported functions from injected DLLs.
//! Supports cross-architecture injection (x64 injector → x86/x64 target).
//!
//! # Architecture Compatibility
//! | Injector | Target Process | DLL Architecture | Status |
//! |----------|---------------|------------------|--------|
//! | x64 | x64 | x64 | ✅ Supported |
//! | x86 | x86 | x86 | ✅ Supported |
//! | x64 | x86 (WOW64) | x86 | ✅ Supported |
//! | x86 | x64 | x64 | ❌ Not possible |
//!
//! # Quick Start
//!
//! ## Basic DLL Injection
//! ```no_run
//! use win_auto_utils::dll_injector::inject_dll;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Inject a DLL into a process by PID
//!     inject_dll(12345, "C:\\path\\to\\library.dll")?;
//!     Ok(())
//! }
//! ```
//!
//! ## Call Exported Functions
//! ```no_run
//! use win_auto_utils::dll_injector::{
//!     get_exported_function_address,
//!     call_function_with_raw_bytes
//! };
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Get function address
//!     let func_addr = get_exported_function_address(12345, "game_mod.dll", "set_health")?;
//!
//!     // Call with i32 parameter (999 health)
//!     let health: i32 = 999;
//!     let param_bytes = health.to_ne_bytes();
//!     let result = call_function_with_raw_bytes(12345, func_addr, Some(&param_bytes))?;
//!
//!     println!("Function returned: {}", result);
//!     Ok(())
//! }
//! ```
//!
//! # Module Structure
//!
//! This module is organized into specialized submodules for clarity and maintainability:
//!
//! - **`core`**: Core injection/unloading logic ([`inject_dll`], [`unload_dll`])
//! - **`remote_call`**: Remote function calling APIs ([`get_exported_function_address`], [`call_function_with_raw_bytes`], [`call_function_no_params`])
//! - **`diagnostic`**: Diagnostic tools ([`diagnose_injection`])
//! - **`shellcode`**: Architecture-specific shellcode generators (x86/x64)
//!
//! # Public API Overview
//!
//! ## Core Functions
//!
//! ### `inject_dll(pid, dll_path)`
//! Injects a DLL into the target process using LoadLibraryW + CreateRemoteThread.
//! This is the primary entry point for DLL injection workflows.
//!
//! **Common Use Cases**:
//! - Loading game mods
//! - Process instrumentation
//! - Debugging and profiling
//!
//! ### `unload_dll(pid, module_name)`
//! Unloads a DLL from the target process using FreeLibrary + CreateRemoteThread.
//! Allows cleanup and dynamic unloading of injected modules.
//!
//! **Note**: May not work for system proxy DLLs (version.dll, etc.) due to reference counting.
//! For proxy DLLs, prefer configuration switches over unloading.
//!
//! ### `get_exported_function_address(pid, module_name, function_name)`
//! Resolves the virtual address of an exported function in a remote DLL.
//! Enables calling arbitrary functions without hardcoding addresses.
//!
//! **Implementation**: Uses shellcode to call GetProcAddress in target process.
//!
//! ### `call_function_with_raw_bytes(pid, function_address, param_data)`
//! Calls a remote function by passing raw byte parameters.
//! Most flexible API - supports any parameter type via serialization.
//!
//! **Parameter Types**: i8/i16/i32/i64/u8/u16/u32/u64/f32/f64/structs (via .to_ne_bytes())
//!
//! ### `call_function_no_params(pid, function_address)`
//! Calls a remote function that takes NO parameters.
//! Generates simpler and more efficient shellcode than the generic version.
//!
//! **Use Case**: For parameterless functions like `void reset_stats()`.
//!
//! ## Diagnostic Functions
//!
//! ### `diagnose_injection(pid)`
//! Performs feasibility checks before attempting injection.
//! Returns detailed report with process accessibility, memory allocation, and write permissions.
//!
//! **Usage Frequency**: Low - Only used during development/debugging.
//! **Returns**: `DiagnosticReport` with fields: process_accessible, kernel32_loaded, memory_allocatable, memory_writable.
//!
//! # Performance Characteristics
//! - Typical injection time: 50-200ms (depends on target process)
//! - Function call time: 5-30ms per call
//! - Memory overhead: ~4KB per injection (freed after completion)
//! - Shellcode size: 40-120 bytes (architecture dependent)
//!
//! # FAQ
//!
//! ## Q: Injection fails with "Access Denied"
//! A: Run your program as Administrator. Right-click → "Run as administrator"
//!
//! ## Q: How do I know if my DLL is x86 or x64?
//! A: Use the `dumpbin /headers your.dll` command or check compilation target
//!
//! ## Q: Can I inject into protected processes (e.g., games with anti-cheat)?
//! A: Generally no. Anti-cheat systems block CreateRemoteThread and memory writes.
//!
//! ## Q: Module detection shows "not loaded" after successful injection
//! A: Possible causes:
//! - DllMain returned FALSE (initialization failed)
//! - Missing dependencies prevented loading
//! - Anti-injection protection blocked loading
//! - Debug version called AllocConsole (crashes protected processes)
//!
//! Check Windows Event Viewer for application errors.
//!
//! # Feature Flag
//! Enable with: `--features "dll_injector"`

// Module declarations
mod helpers;
mod core;
mod remote_call;
mod diagnostic;
mod shellcode;

// Re-export public API
pub use core::{inject_dll, unload_dll};
use std::fmt;
pub use remote_call::{get_exported_function_address, call_function_with_raw_bytes, call_function_no_params};
pub use diagnostic::diagnose_injection;

/// DLL injection error types
#[derive(Debug)]
pub enum DllInjectorError {
    /// Failed to open target process
    OpenProcessFailed(String),
    /// Failed to allocate memory in target process
    AllocationFailed,
    /// Failed to write data to target process
    WriteFailed(String),
    /// Failed to get function address
    GetProcAddressFailed(String),
    /// Failed to create remote thread
    CreateThreadFailed(String),
    /// DLL architecture mismatch
    ArchitectureMismatch(String),
    /// Module already loaded
    AlreadyLoaded(String),
    /// Generic error
    Other(String),
}

impl fmt::Display for DllInjectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DllInjectorError::OpenProcessFailed(msg) => write!(f, "Open process failed: {}", msg),
            DllInjectorError::AllocationFailed => write!(f, "Failed to allocate memory in target process"),
            DllInjectorError::WriteFailed(msg) => write!(f, "Failed to write to target process: {}", msg),
            DllInjectorError::GetProcAddressFailed(msg) => write!(f, "Get procedure address failed: {}", msg),
            DllInjectorError::CreateThreadFailed(msg) => write!(f, "Create remote thread failed: {}", msg),
            DllInjectorError::ArchitectureMismatch(msg) => write!(f, "Architecture mismatch: {}", msg),
            DllInjectorError::AlreadyLoaded(msg) => write!(f, "DLL already loaded: {}", msg),
            DllInjectorError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for DllInjectorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_pid() {
        let result = inject_dll(0, "test.dll");
        assert!(result.is_err());
    }

    #[test]
    fn test_nonexistent_dll() {
        let result = inject_dll(std::process::id(), "C:\\nonexistent\\path.dll");
        assert!(result.is_err());
    }
}
