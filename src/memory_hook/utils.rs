//! Utility functions for memory hook operations
//!
//! Provides common utilities for memory protection, address validation, and error handling.

use crate::memory::MemoryError;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Memory::{
    VirtualProtectEx, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};

/// Wrapper for HANDLE that implements Send
///
/// # Safety
/// HANDLE is safe to send between threads in the context of Windows API calls.
/// Windows handles are thread-safe and can be used from any thread.
#[derive(Clone, Copy)]
pub struct SendableHandle(pub HANDLE);

// SAFETY: HANDLE is a pointer to a kernel object which is safe to share across threads.
// The Windows API guarantees that handles can be used from any thread within the same process.
unsafe impl Send for SendableHandle {}
unsafe impl Sync for SendableHandle {}

/// RAII guard for memory protection changes
///
/// Automatically restores original memory protection when dropped.
///
/// # Example
/// ```no_run
/// use win_auto_utils::memory_hook::ProtectionGuard;
/// use win_auto_utils::handle::open_process_handle;
/// use windows::Win32::System::Threading::PROCESS_ALL_ACCESS;
///
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let handle = open_process_handle(12345, PROCESS_ALL_ACCESS).unwrap();
///     let address = 0x7FF6A1B2C3D4;
///     
///     // Temporarily change protection to read-write-execute
///     let _guard = ProtectionGuard::new(handle, address, 4096)?;
///     
///     // Memory is now executable and writable
///     // ... perform hooking operations ...
///     
///     // When _guard goes out of scope, protection is restored
///     Ok(())
/// }
/// ```
pub struct ProtectionGuard {
    handle: SendableHandle,
    address: *mut std::ffi::c_void,
    size: usize,
    old_protection: PAGE_PROTECTION_FLAGS,
}

impl ProtectionGuard {
    /// Create a new protection guard
    ///
    /// Changes the memory protection to PAGE_EXECUTE_READWRITE and stores
    /// the original protection for restoration.
    ///
    /// # Arguments
    /// * `handle` - Process handle with PROCESS_VM_OPERATION rights
    /// * `address` - Base address of the memory region
    /// * `size` - Size of the memory region in bytes
    ///
    /// # Errors
    /// Returns `MemoryError` if VirtualProtectEx fails
    pub fn new(handle: HANDLE, address: usize, size: usize) -> Result<Self, MemoryError> {
        let mut old_protection = PAGE_PROTECTION_FLAGS(0);

        unsafe {
            let result = VirtualProtectEx(
                handle,
                address as *mut std::ffi::c_void,
                size,
                PAGE_EXECUTE_READWRITE,
                &mut old_protection,
            );

            if result.is_err() {
                // Get detailed error information
                let error_code = result.unwrap_err();
                return Err(MemoryError::WriteFailed(
                    format!(
                        "Failed to change memory protection at 0x{:X} (size: {} bytes). \
                         Windows error: {:?}. \
                         This may happen if the process has exited or the memory region is not accessible.",
                        address, size, error_code
                    )
                ));
            }
        }

        Ok(Self {
            handle: SendableHandle(handle),
            address: address as *mut std::ffi::c_void,
            size,
            old_protection,
        })
    }

    /// Manually restore the original protection
    pub fn restore(mut self) -> Result<(), MemoryError> {
        self.do_restore()
    }

    fn do_restore(&mut self) -> Result<(), MemoryError> {
        unsafe {
            let mut _dummy = PAGE_PROTECTION_FLAGS(0);
            let result = VirtualProtectEx(
                self.handle.0,
                self.address,
                self.size,
                self.old_protection,
                &mut _dummy,
            );

            if result.is_err() {
                return Err(MemoryError::WriteFailed(
                    "Failed to restore memory protection".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl Drop for ProtectionGuard {
    fn drop(&mut self) {
        let _ = self.do_restore();
    }
}

/// Memory protector utility for batch operations
pub struct MemoryProtector;

impl MemoryProtector {
    /// Temporarily change protection for multiple regions
    ///
    /// # Arguments
    /// * `handle` - Process handle
    /// * `regions` - Vector of (address, size) tuples
    ///
    /// # Returns
    /// Vector of ProtectionGuards that will restore protection when dropped
    pub fn protect_multiple(
        handle: HANDLE,
        regions: &[(usize, usize)],
    ) -> Result<Vec<ProtectionGuard>, MemoryError> {
        let mut guards = Vec::new();

        for &(address, size) in regions {
            let guard = ProtectionGuard::new(handle, address, size)?;
            guards.push(guard);
        }

        Ok(guards)
    }
}

#[cfg(test)]
mod tests {
    /// Validate if an address is within valid user space
    ///
    /// # Arguments
    /// * `address` - The address to validate
    /// * `is_64bit` - Whether the target process is 64-bit
    ///
    /// # Returns
    /// true if the address is potentially valid
    pub fn is_valid_user_address(address: usize, is_64bit: bool) -> bool {
        if is_64bit {
            // 64-bit: User space is typically 0x0000000000000000 to 0x00007FFFFFFFFFFF
            address < 0x0000800000000000
        } else {
            // 32-bit: User space is 0x00000000 to 0x7FFFFFFF
            address < 0x80000000
        }
    }

    /// Calculate aligned address and size for memory operations
    ///
    /// Aligns the start address down to page boundary and adjusts size accordingly.
    ///
    /// # Arguments
    /// * `address` - Starting address
    /// * `size` - Number of bytes
    /// * `page_size` - System page size (typically 4096)
    ///
    /// # Returns
    /// (aligned_address, aligned_size)
    pub fn align_to_page(address: usize, size: usize, page_size: usize) -> (usize, usize) {
        let aligned_addr = address & !(page_size - 1);
        let end_addr = address + size;
        let aligned_end = (end_addr + page_size - 1) & !(page_size - 1);
        let aligned_size = aligned_end - aligned_addr;

        (aligned_addr, aligned_size)
    }

    #[test]
    fn test_is_valid_user_address_32bit() {
        assert!(is_valid_user_address(0x00400000, false));
        assert!(is_valid_user_address(0x7FFFFFFF, false));
        assert!(!is_valid_user_address(0x80000000, false));
        assert!(!is_valid_user_address(0xFFFFFFFF, false));
    }

    #[test]
    fn test_is_valid_user_address_64bit() {
        assert!(is_valid_user_address(0x00007FFF00000000, true));
        assert!(is_valid_user_address(0x00007FFFFFFFFFFF, true));
        assert!(!is_valid_user_address(0xFFFF800000000000, true));
    }

    #[test]
    fn test_align_to_page() {
        let page_size = 4096;

        // Already aligned
        let (addr, size) = align_to_page(0x1000, 4096, page_size);
        assert_eq!(addr, 0x1000);
        assert_eq!(size, 4096);

        // Not aligned
        let (addr, size) = align_to_page(0x1005, 100, page_size);
        assert_eq!(addr, 0x1000);
        assert_eq!(size, 4096); // Rounds up to next page
    }
}
