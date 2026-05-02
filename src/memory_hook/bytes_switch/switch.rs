//! BytesSwitch Core Implementation
//!
//! This module contains the core implementation of BytesSwitch, including:
//! - Byte switching (enable/disable)
//! - State management
//! - RAII automatic restoration

use crate::memory::{read_memory_bytes, write_memory_bytes, MemoryError};
use crate::memory_hook::utils::{ProtectionGuard, SendableHandle};
use windows::Win32::Foundation::HANDLE;

/// Bytecode Switch for quickly enabling/disabling specific machine instructions
///
/// Unlike hooking techniques, it **does not change execution flow**, but directly
/// replaces the original bytes at the target address.
///
/// # Features
/// -  **Zero Overhead**: No memory allocation, no jump instructions, no trampoline
/// -  **Fast Switching**: Simple memory write operations
/// -  **Automatic Management**: RAII auto-restoration on drop
/// -  **State Tracking**: Prevents duplicate operations
///
/// # Example
/// ```no_run
/// use win_auto_utils::memory_hook::BytesSwitch;
///
/// // Disable a feature with NOP
/// let mut switch = BytesSwitch::new_nop(handle, 0x41FAF2, 6)?;
/// switch.enable()?;   // Write NOP
/// switch.disable()?;  // Restore original instructions
/// ```
pub struct BytesSwitch {
    handle: SendableHandle,
    target_address: usize,
    original_bytes: Vec<u8>,
    patch_bytes: Vec<u8>,
    is_enabled: bool,
}

impl BytesSwitch {
    /// Create a new bytes switch
    ///
    /// # Arguments
    /// * `handle` - Process handle with PROCESS_VM_READ and PROCESS_VM_WRITE permissions
    /// * `target_address` - Target address
    /// * `original_bytes` - Original bytes (for restoration)
    /// * `patch_bytes` - Patch bytes (for replacement)
    ///
    /// # Note
    /// `original_bytes` and `patch_bytes` must have the same length
    ///
    /// # Example
    /// ```no_run
    /// let original = vec![0x01, 0x91, 0x08, 0x03, 0x00, 0x00]; // add [ecx+0x308],edx
    /// let patch = vec![0x90, 0x90, 0x90, 0x90, 0x90, 0x90];     // NOP x6
    /// let switch = BytesSwitch::new(handle, 0x41FAF2, original, patch);
    /// ```
    pub fn new(
        handle: HANDLE,
        target_address: usize,
        original_bytes: Vec<u8>,
        patch_bytes: Vec<u8>,
    ) -> Self {
        Self {
            handle: SendableHandle(handle),
            target_address,
            original_bytes,
            patch_bytes,
            is_enabled: false,
        }
    }

    /// Create a NOP patch switch (convenience method)
    ///
    /// Automatically reads the original bytes from the target address and generates
    /// a corresponding NOP sequence as the patch.
    ///
    /// # Arguments
    /// * `handle` - Process handle
    /// * `target_address` - Target address
    /// * `byte_count` - Number of bytes to replace
    ///
    /// # Errors
    /// Returns `MemoryError` if reading original bytes fails
    ///
    /// # Example
    /// ```no_run
    /// // Replace instructions at target address with 6 NOPs
    /// let switch = BytesSwitch::new_nop(handle, 0x41FAF2, 6)?;
    /// switch.enable()?;  // Disable feature
    /// ```
    pub fn new_nop(
        handle: HANDLE,
        target_address: usize,
        byte_count: usize,
    ) -> Result<Self, MemoryError> {
        let original_bytes = read_memory_bytes(handle, target_address, byte_count)?;
        let patch_bytes = vec![0x90; byte_count]; // NOP instruction

        Ok(Self::new(
            handle,
            target_address,
            original_bytes,
            patch_bytes,
        ))
    }

    /// Create a builder for precise configuration
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::BytesSwitch;
    ///
    /// let mut switch = BytesSwitch::builder()
    ///     .handle(handle)
    ///     .target_address(0x41FAF2)
    ///     .original_bytes(vec![0x01, 0x91, 0x08, 0x03, 0x00, 0x00])
    ///     .patch_bytes(vec![0x90, 0x90, 0x90, 0x90, 0x90, 0x90])
    ///     .build()?;
    ///
    /// switch.enable()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn builder() -> crate::memory_hook::bytes_switch::builder::BytesSwitchBuilder {
        crate::memory_hook::bytes_switch::builder::BytesSwitchBuilder::new()
    }

    /// Check if the switch is currently enabled
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    /// Get the process handle used by this switch instance
    pub fn get_handle(&self) -> HANDLE {
        self.handle.0
    }

    /// Enable the patch (write patch bytes to target address)
    ///
    /// # Errors
    /// If already enabled or write fails, returns `MemoryError`
    ///
    /// # Example
    /// ```no_run
    /// let mut switch = BytesSwitch::new_nop(handle, 0x41FAF2, 6)?;
    /// switch.enable()?;  // Now feature is disabled
    /// ```
    pub fn enable(&mut self) -> Result<(), MemoryError> {
        if self.is_enabled {
            return Err(MemoryError::WriteFailed(
                "Patch already enabled".to_string(),
            ));
        }

        if self.original_bytes.len() != self.patch_bytes.len() {
            return Err(MemoryError::WriteFailed(
                "Original and patch bytes length mismatch".to_string(),
            ));
        }

        let _guard =
            ProtectionGuard::new(self.handle.0, self.target_address, self.patch_bytes.len())?;

        write_memory_bytes(self.handle.0, self.target_address, &self.patch_bytes)?;
        self.is_enabled = true;

        Ok(())
    }

    /// Disable patch (restore original bytes)
    ///
    /// # Errors
    /// If not enabled or restore fails, returns `MemoryError`
    ///
    /// # Example
    /// ```no_run
    /// let mut switch = BytesSwitch::new_nop(handle, 0x41FAF2, 6)?;
    /// switch.enable()?;
    /// switch.disable()?;  // Feature is restored
    /// ```
    pub fn disable(&mut self) -> Result<(), MemoryError> {
        if !self.is_enabled {
            return Err(MemoryError::WriteFailed("Patch not enabled".to_string()));
        }

        let _guard = ProtectionGuard::new(
            self.handle.0,
            self.target_address,
            self.original_bytes.len(),
        )?;

        write_memory_bytes(self.handle.0, self.target_address, &self.original_bytes)?;
        self.is_enabled = false;

        Ok(())
    }

    /// Toggle patch state
    ///
    /// If currently enabled, disable; if disabled, enable.
    ///
    /// # Errors
    /// If toggle fails, returns `MemoryError`
    ///
    /// # Example
    /// ```no_run
    /// let mut switch = BytesSwitch::new_nop(handle, 0x41FAF2, 6)?;
    /// switch.toggle()?;  // Enable
    /// switch.toggle()?;  // Disable
    /// ```
    pub fn toggle(&mut self) -> Result<(), MemoryError> {
        if self.is_enabled {
            self.disable()
        } else {
            self.enable()
        }
    }

    /// Reset the switch to its initial state (safe for program restart)
    ///
    /// This method safely cleans up all internal state, allowing the switch
    /// to be reused after the target program restarts.
    ///
    /// # What it does:
    /// 1. If enabled, disables the patch first (restores original bytes)
    /// 2. Clears cached data (original_bytes, patch_bytes)
    /// 3. Resets all flags to initial state
    ///
    /// # Safety
    /// - Safe to call multiple times (idempotent)
    /// - Safe to call even if never enabled
    /// - After reset, the object can be used as if newly created
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::BytesSwitch;
    ///
    /// // First usage
    /// let mut switch = BytesSwitch::new_nop(handle, addr, 6)?;
    /// switch.enable()?;
    /// switch.disable()?;
    ///
    /// // Program restarted - reset state
    /// switch.reset();
    ///
    /// // Second usage (no side effects)
    /// let mut switch = BytesSwitch::new_nop(handle, new_addr, 6)?;
    /// switch.enable()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn reset(&mut self) {
        // If still enabled, disable first (this will restore bytes)
        if self.is_enabled {
            let _ = self.disable();
        }

        // Clear all cached state
        self.original_bytes.clear();
        self.patch_bytes.clear();
        self.is_enabled = false;
    }

    /// Get target address
    pub fn target_address(&self) -> usize {
        self.target_address
    }

    /// Get original bytes
    pub fn original_bytes(&self) -> &[u8] {
        &self.original_bytes
    }

    /// Get patch bytes
    pub fn patch_bytes(&self) -> &[u8] {
        &self.patch_bytes
    }
}

impl Drop for BytesSwitch {
    fn drop(&mut self) {
        // RAII: Automatically restore original bytes
        if self.is_enabled {
            let _ = self.disable();
        }
    }
}
