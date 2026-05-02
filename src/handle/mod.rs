//! Process handle management module
//!
//! Provides functions to obtain and manage process handles with proper lifecycle management.
//! Process handles are required for memory operations, thread manipulation, and other
//! low-level process interactions.
//!
//! # Important Notes
//! - Always close handles when done using them to prevent resource leaks
//! - Use RAII patterns (Process struct) for automatic cleanup
//! - Handle access rights determine what operations can be performed
//! - Higher privileges may be required for certain operations

use windows::{
    Win32::Foundation::{CloseHandle, HANDLE},
    Win32::System::Threading::{OpenProcess, PROCESS_ACCESS_RIGHTS},
};

/// Open a process handle with specified access rights
///
/// Retrieves a handle to an existing process object with the desired access permissions.
/// The handle can be used for various operations depending on the access rights granted.
///
/// # Arguments
/// * `pid` - The process ID to open
/// * `desired_access` - The access rights requested (e.g., PROCESS_ALL_ACCESS, PROCESS_VM_READ)
///
/// # Returns
/// * `Some(HANDLE)` - The process handle if successful
/// * `None` - If the operation failed (process not found, insufficient privileges, etc.)
///
/// # Common Access Rights
/// - `PROCESS_ALL_ACCESS` (0x1F0FFF): Full access to the process
/// - `PROCESS_VM_READ` (0x0010): Read memory from the process
/// - `PROCESS_VM_WRITE` (0x0020): Write memory to the process
/// - `PROCESS_VM_OPERATION` (0x0008): Perform memory operations (allocate, free, etc.)
/// - `PROCESS_QUERY_INFORMATION` (0x0400): Query process information
/// - `PROCESS_CREATE_THREAD` (0x0002): Create threads in the process
///
/// # Important
/// **You MUST call `close_handle(handle)` when done with the handle**
/// Failure to do so will cause resource leaks.
///
/// # Example
/// ```no_run
/// use win_auto_utils::snapshot::find_pid_by_name;
/// use win_auto_utils::handle::{open_process_handle, close_handle};
/// use windows::Win32::System::Threading::PROCESS_ALL_ACCESS;
///
/// if let Some(pid) = find_pid_by_name("notepad.exe") {
///     if let Some(handle) = open_process_handle(pid, PROCESS_ALL_ACCESS) {
///         // Use the handle for memory operations, etc.
///         // ...
///         
///         // IMPORTANT: Close the handle when done
///         close_handle(handle);
///     }
/// }
/// ```
pub fn open_process_handle(pid: u32, desired_access: PROCESS_ACCESS_RIGHTS) -> Option<HANDLE> {
    unsafe {
        match OpenProcess(desired_access, false, pid) {
            Ok(handle) if !handle.is_invalid() => Some(handle),
            _ => None,
        }
    }
}

/// Open a process handle with read-only access
///
/// Convenience function that opens a process with minimal read permissions.
/// Suitable for reading memory and querying process information.
///
/// # Arguments
/// * `pid` - The process ID to open
///
/// # Returns
/// * `Some(HANDLE)` - The process handle if successful
/// * `None` - If the operation failed
///
/// # Example
/// ```no_run
/// use win_auto_utils::snapshot::find_pid_by_name;
/// use win_auto_utils::handle::{open_process_read_handle, close_handle};
///
/// if let Some(pid) = find_pid_by_name("notepad.exe") {
///     if let Some(handle) = open_process_read_handle(pid) {
///         // Read memory from the process
///         // ...
///         
///         close_handle(handle);
///     }
/// }
/// ```
pub fn open_process_read_handle(pid: u32) -> Option<HANDLE> {
    use windows::Win32::System::Threading::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

    let access_rights = PROCESS_QUERY_INFORMATION | PROCESS_VM_READ;
    open_process_handle(pid, access_rights)
}

/// Open a process handle with read/write access
///
/// Convenience function that opens a process with read and write permissions.
/// Suitable for reading and writing memory.
///
/// # Arguments
/// * `pid` - The process ID to open
///
/// # Returns
/// * `Some(HANDLE)` - The process handle if successful
/// * `None` - If the operation failed
///
/// # Example
/// ```no_run
/// use win_auto_utils::snapshot::find_pid_by_name;
/// use win_auto_utils::handle::{open_process_rw_handle, close_handle};
///
/// if let Some(pid) = find_pid_by_name("notepad.exe") {
///     if let Some(handle) = open_process_rw_handle(pid) {
///         // Read and write memory
///         // ...
///         
///         close_handle(handle);
///     }
/// }
/// ```
pub fn open_process_rw_handle(pid: u32) -> Option<HANDLE> {
    use windows::Win32::System::Threading::{
        PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
    };

    let access_rights =
        PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION;

    open_process_handle(pid, access_rights)
}

/// Open a process handle with full access
///
/// Convenience function that opens a process with all possible permissions.
/// This requires elevated privileges and should be used sparingly.
///
/// # Arguments
/// * `pid` - The process ID to open
///
/// # Returns
/// * `Some(HANDLE)` - The process handle if successful
/// * `None` - If the operation failed (often due to insufficient privileges)
///
/// # Note
/// This may fail for system processes or processes running with higher privileges.
/// Consider using more specific access rights when possible.
///
/// # Example
/// ```no_run
/// use win_auto_utils::snapshot::find_pid_by_name;
/// use win_auto_utils::handle::{open_process_full_handle, close_handle};
///
/// if let Some(pid) = find_pid_by_name("notepad.exe") {
///     if let Some(handle) = open_process_full_handle(pid) {
///         // Full access to the process
///         // ...
///         
///         close_handle(handle);
///     }
/// }
/// ```
pub fn open_process_full_handle(pid: u32) -> Option<HANDLE> {
    use windows::Win32::System::Threading::PROCESS_ALL_ACCESS;
    open_process_handle(pid, PROCESS_ALL_ACCESS)
}

/// Close a process handle
///
/// Releases the handle and frees associated resources.
/// Always call this when you're done using a process handle.
///
/// # Arguments
/// * `handle` - The process handle to close
///
/// # Returns
/// `true` if successful, `false` otherwise
///
/// # Example
/// ```no_run
/// use win_auto_utils::handle::{open_process_read_handle, close_handle};
///
/// if let Some(handle) = open_process_read_handle(12345) {
///     // Use the handle...
///     close_handle(handle);
/// }
/// ```
pub fn close_handle(handle: HANDLE) -> bool {
    if handle.is_invalid() {
        return false;
    }

    unsafe { CloseHandle(handle).is_ok() }
}

/// Check if a process handle is valid
///
/// Verifies that the handle is not null or invalid.
///
/// # Arguments
/// * `handle` - The handle to check
///
/// # Returns
/// `true` if the handle appears valid, `false` otherwise
pub fn is_handle_valid(handle: HANDLE) -> bool {
    !handle.is_invalid() && !handle.0.is_null()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_process_read_handle() {
        // Try to open current process (should always work)
        let current_pid = std::process::id();

        if let Some(handle) = open_process_read_handle(current_pid) {
            assert!(is_handle_valid(handle), "Handle should be valid");

            // Clean up
            let closed = close_handle(handle);
            assert!(closed, "Should successfully close handle");
        }
    }

    #[test]
    fn test_open_process_rw_handle() {
        let current_pid = std::process::id();

        if let Some(handle) = open_process_rw_handle(current_pid) {
            assert!(is_handle_valid(handle), "Handle should be valid");
            close_handle(handle);
        }
    }

    #[test]
    fn test_open_process_full_handle() {
        let current_pid = std::process::id();

        if let Some(handle) = open_process_full_handle(current_pid) {
            assert!(is_handle_valid(handle), "Handle should be valid");
            close_handle(handle);
        }
    }

    #[test]
    fn test_close_invalid_handle() {
        // Test closing an invalid handle
        let result = close_handle(HANDLE::default());
        assert!(!result, "Should fail to close invalid handle");
    }

    #[test]
    fn test_is_handle_valid() {
        // Invalid handle
        assert!(!is_handle_valid(HANDLE::default()));
        assert!(!is_handle_valid(HANDLE(std::ptr::null_mut())));

        // Valid handle (current process)
        let current_pid = std::process::id();
        if let Some(handle) = open_process_read_handle(current_pid) {
            assert!(is_handle_valid(handle));
            close_handle(handle);
        }
    }

    #[test]
    fn test_handle_lifecycle() {
        // Test complete handle lifecycle: open -> use -> close
        let current_pid = std::process::id();

        if let Some(handle) = open_process_read_handle(current_pid) {
            println!("Opened handle: {:?}", handle);
            assert!(is_handle_valid(handle));

            // Simulate using the handle
            println!("Using handle for operations...");

            // Close it
            assert!(close_handle(handle));
            println!("Handle closed successfully");
        }
    }

    #[test]
    fn test_open_nonexistent_process() {
        // Try to open a PID that doesn't exist
        let fake_pid = 999999u32;
        let result = open_process_read_handle(fake_pid);

        // Should return None for non-existent process
        assert!(
            result.is_none(),
            "Should not get handle for non-existent process"
        );
    }
}
