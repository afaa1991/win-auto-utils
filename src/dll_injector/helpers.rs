//! Internal Helper Functions for DLL Injection

use windows::{
    Win32::{
        Foundation::HANDLE,
        System::Threading::{OpenProcess, PROCESS_ALL_ACCESS},
    },
};

use super::DllInjectorError;

/// Get the base address of a module in the target process
/// 
/// # Arguments
/// * `pid` - Target process ID
/// * `module_name` - Module name (case-insensitive)
/// 
/// # Returns
/// Returns the module base address, or an error if not found
pub(crate) fn get_module_base_address(pid: u32, module_name: &str) -> Result<usize, DllInjectorError> {
    crate::snapshot::get_module_base_address(pid, module_name)
        .ok_or_else(|| DllInjectorError::Other(format!(
            "Module {} not found in process {}",
            module_name, pid
        )))
}

/// Check if a module is already loaded in the target process
/// 
/// # Arguments
/// * `pid` - Target process ID
/// * `module_name` - Module name to check
/// 
/// # Returns
/// Returns true if the module is loaded, false otherwise
pub(crate) fn is_module_loaded(pid: u32, module_name: &str) -> bool {
    crate::snapshot::get_module_base_address(pid, module_name).is_some()
}

/// Helper: Open process handle with full access
pub(crate) fn open_process_full(pid: u32) -> Result<HANDLE, DllInjectorError> {
    unsafe {
        OpenProcess(PROCESS_ALL_ACCESS, false, pid)
            .map_err(|e| DllInjectorError::OpenProcessFailed(e.to_string()))
            .and_then(|handle| {
                if handle.is_invalid() {
                    Err(DllInjectorError::OpenProcessFailed(
                        format!("Invalid handle for process {}", pid)
                    ))
                } else {
                    Ok(handle)
                }
            })
    }
}
