//! BytesSwitch Builder - Fluent API for precise configuration
//!
//! This module provides the `BytesSwitchBuilder` for flexible switch configuration.
//! 
//! # Example
//! ```no_run
//! use win_auto_utils::memory_hook::BytesSwitch;
//!
//! // Pre-configure, then bind dynamic parameters later
//! let builder = BytesSwitch::builder()
//!     .byte_count(6);
//!
//! // ... perform AOBScan or wait for user action ...
//! let target_addr = 0x41FAF2;
//!
//! let mut switch = builder.clone()
//!     .handle(handle)
//!     .target_address(target_addr)
//!     .nop_mode()  // Auto-read original bytes and generate NOPs
//!     .build()?;
//!
//! switch.enable()?;
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```

use windows::Win32::Foundation::HANDLE;
use crate::memory::MemoryError;
use super::BytesSwitch;

/// Builder for precise BytesSwitch configuration
///
/// # Example: Build-time specification
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
#[derive(Clone)]
pub struct BytesSwitchBuilder {
    handle: Option<HANDLE>,
    target_address: Option<usize>,
    original_bytes: Option<Vec<u8>>,
    patch_bytes: Option<Vec<u8>>,
    byte_count: Option<usize>,
    nop_mode: bool,
}

impl BytesSwitchBuilder {
    /// Create a new builder with all parameters unset
    pub fn new() -> Self {
        Self {
            handle: None,
            target_address: None,
            original_bytes: None,
            patch_bytes: None,
            byte_count: None,
            nop_mode: false,
        }
    }

    /// Set process handle (required for build)
    pub fn handle(mut self, handle: HANDLE) -> Self {
        self.handle = Some(handle);
        self
    }

    /// Set target address (required for build)
    pub fn target_address(mut self, addr: usize) -> Self {
        self.target_address = Some(addr);
        self
    }

    /// Set original bytes (required unless using nop_mode)
    pub fn original_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.original_bytes = Some(bytes);
        self
    }

    /// Set patch bytes (required unless using nop_mode)
    pub fn patch_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.patch_bytes = Some(bytes);
        self
    }

    /// Set byte count for NOP mode (alternative to providing explicit bytes)
    pub fn byte_count(mut self, count: usize) -> Self {
        self.byte_count = Some(count);
        self
    }

    /// Enable NOP mode (auto-generate NOP patch)
    ///
    /// When enabled, the builder will automatically read original bytes from
    /// the target address and generate corresponding NOP instructions as patch.
    pub fn nop_mode(mut self) -> Self {
        self.nop_mode = true;
        self
    }

    /// Build the BytesSwitch with configured settings
    ///
    /// # Validation
    /// This method validates that all required parameters are set:
    /// - `handle`: Process handle
    /// - `target_address`: Address to patch
    /// - Either (`original_bytes` + `patch_bytes`) OR (`byte_count` with `nop_mode`)
    ///
    /// # Errors
    /// Returns error if any required parameter is missing or lengths mismatch
    pub fn build(self) -> Result<BytesSwitch, MemoryError> {
        // Validate required parameters
        let handle = self.handle.ok_or_else(|| {
            MemoryError::WriteFailed(
                "handle must be set. Call .handle(handle) before build().".to_string()
            )
        })?;

        let target_address = self.target_address.ok_or_else(|| {
            MemoryError::WriteFailed(
                "target_address must be set. Call .target_address(addr) before build().".to_string()
            )
        })?;

        let (original_bytes, patch_bytes) = if self.nop_mode {
            // NOP mode: auto-read original bytes and generate NOPs
            let byte_count = self.byte_count.ok_or_else(|| {
                MemoryError::WriteFailed(
                    "byte_count must be set when using nop_mode. Call .byte_count(n) before build().".to_string()
                )
            })?;

            use crate::memory::read_memory_bytes;
            let original = read_memory_bytes(handle, target_address, byte_count)?;
            let patch = vec![0x90; byte_count];
            
            (original, patch)
        } else {
            // Manual mode: use provided bytes
            let original = self.original_bytes.ok_or_else(|| {
                MemoryError::WriteFailed(
                    "original_bytes must be set. Call .original_bytes(bytes) or enable .nop_mode().".to_string()
                )
            })?;

            let patch = self.patch_bytes.ok_or_else(|| {
                MemoryError::WriteFailed(
                    "patch_bytes must be set. Call .patch_bytes(bytes) or enable .nop_mode().".to_string()
                )
            })?;

            if original.len() != patch.len() {
                return Err(MemoryError::WriteFailed(
                    format!(
                        "Original and patch bytes length mismatch: {} vs {}",
                        original.len(),
                        patch.len()
                    )
                ));
            }

            (original, patch)
        };

        Ok(BytesSwitch::new(handle, target_address, original_bytes, patch_bytes))
    }
}
