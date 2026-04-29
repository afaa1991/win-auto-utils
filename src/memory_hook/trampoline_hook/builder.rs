//! TrampolineHook Builder - Delayed binding and fluent API
//!
//! This module provides the `TrampolineHookBuilder` for flexible hook configuration.
//!
//! # Example
//! ```no_run
//! use win_auto_utils::memory_hook::TrampolineHook;
//!
//! let mut hook = TrampolineHook::builder()
//!     .handle(handle)
//!     .target_address(0x41FAF2)
//!     .detour_code(shellcode)
//!     .x86()
//!     .build()?;
//!
//! hook.install()?;
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```
//!
//! # Runtime specification (AOBScan workflow)
//! ```no_run
//! use win_auto_utils::memory_hook::TrampolineHook;
//!
//! // Step 1: Pre-configure static parameters
//! let builder = TrampolineHook::builder()
//!     .detour_code(shellcode)
//!     .x86();
//!
//! // ... perform AOBScan or wait for user action ...
//! let target_addr = aob_scan(handle, "29 88 FC 02 00 00")?;
//!
//! // Step 2: Bind dynamic parameters and install
//! let mut hook = builder.clone()
//!     .handle(handle)
//!     .target_address(target_addr)
//!     .build()?;
//!
//! hook.install()?;
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```

use windows::Win32::Foundation::HANDLE;

use crate::memory::MemoryError;
use super::{TrampolineHook, HookArchitecture};
use crate::memory_hook::utils::SendableHandle;

/// Builder for precise TrampolineHook configuration
/// Validation happens at `build()` time.
///
/// # Example: Build-time specification
/// ```no_run
/// use win_auto_utils::memory_hook::TrampolineHook;
///
/// let mut hook = TrampolineHook::builder()
///     .handle(handle)
///     .target_address(0x41FAF2)
///     .detour_code(shellcode)
///     .x86()
///     .build()?;
///
/// hook.install()?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// For AOBScan workflows and advanced usage, see module-level documentation.
///
/// # Example 2: Install-time specification (Deferred binding)
/// ```no_run
/// use win_auto_utils::memory_hook::trampoline_hook_builder::TrampolineHookBuilder;
///
/// // Step 1: Pre-configure static parameters
/// let builder = TrampolineHookBuilder::new()
///     .detour_code(shellcode)
///     .x86();
///
/// // ... perform AOBScan or wait for user action ...
/// let target_addr = aob_scan(handle, "29 88 FC 02 00 00")?;
///
/// // Step 2: Bind dynamic parameters and install
/// let mut hook = builder.clone()
///     .handle(handle)
///     .target_address(target_addr)
///     .build()?;
///
/// hook.install()?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone)]
pub struct TrampolineHookBuilder {
    handle: Option<HANDLE>,
    target_address: Option<usize>,
    detour_code: Option<Vec<u8>>,
    trampoline_address: Option<usize>,
    original_bytes: Option<Vec<u8>>,
    bytes_to_overwrite: Option<usize>,
    architecture: Option<HookArchitecture>,
    skip_trampoline: bool,  // default: false (generate trampoline)
}

impl TrampolineHookBuilder {
    /// Create a new builder with all parameters unset
    pub fn new() -> Self {
        Self {
            handle: None,
            target_address: None,
            detour_code: None,
            trampoline_address: None,
            original_bytes: None,
            bytes_to_overwrite: None,
            architecture: None,
            skip_trampoline: false,
        }
    }

    /// Set process handle (required for install)
    ///
    /// Can be set at build-time or install-time.
    pub fn handle(mut self, handle: HANDLE) -> Self {
        self.handle = Some(handle);
        self
    }

    /// Set target address (required for install)
    ///
    /// Can be set at build-time (static address) or install-time (after AOBScan/Resolver).
    pub fn target_address(mut self, addr: usize) -> Self {
        self.target_address = Some(addr);
        self
    }

    /// Set detour shellcode (REQUIRED - TrampolineHook auto-allocates detour memory)
    ///
    /// TrampolineHook will automatically allocate memory near the target,
    /// write your shellcode, and append the JMP to trampoline.
    ///
    /// # Note
    /// TrampolineHook does NOT support custom detour addresses.
    /// If you need full control over detour, use `InlineHook` instead.
    pub fn detour_code(mut self, code: Vec<u8>) -> Self {
        self.detour_code = Some(code);
        self
    }

    /// Set custom trampoline address (advanced - user manages memory)
    ///
    /// Use this when you want to provide your own trampoline memory.
    /// TrampolineHook will NOT free this memory - you are responsible for managing it.
    ///
    /// # Important
    /// The provided memory must be:
    /// - Large enough: at least `bytes_to_overwrite + 14` bytes
    /// - Executable: PAGE_EXECUTE_READWRITE protection
    /// - Valid for the lifetime of the hook
    pub fn trampoline_address(mut self, addr: usize) -> Self {
        self.trampoline_address = Some(addr);
        self
    }

    /// Set pre-backed-up original bytes (advanced)
    ///
    /// Use this when you have already backed up the original bytes before installing any hooks.
    /// This prevents the issue of reading already-hooked bytes.
    ///
    /// If not set, TrampolineHook will read original bytes during install().
    pub fn original_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.original_bytes = Some(bytes);
        self
    }

    /// Set the number of bytes to overwrite at target address
    ///
    /// If not set, defaults to architecture minimums:
    /// - x86: 5 bytes (JMP rel32)
    /// - x64: 14 bytes (MOV RAX + JMP RAX)
    ///
    /// # Important
    /// This MUST cover complete instructions. If you split an instruction,
    /// the program will crash or behave incorrectly.
    pub fn bytes_to_overwrite(mut self, bytes: usize) -> Self {
        self.bytes_to_overwrite = Some(bytes);
        self
    }

    /// Set the target architecture
    pub fn architecture(mut self, arch: HookArchitecture) -> Self {
        self.architecture = Some(arch);
        self
    }

    /// Convenience method for x86 architecture
    pub fn x86(self) -> Self {
        self.architecture(HookArchitecture::X86)
    }

    /// Convenience method for x64 architecture
    pub fn x64(self) -> Self {
        self.architecture(HookArchitecture::X64)
    }

    /// Skip automatic trampoline generation (Advanced)
    ///
    /// When enabled:
    /// - No trampoline memory is allocated
    /// - Detour JMP goes directly to target + bytes_to_overwrite
    /// - You MUST include all logic in your shellcode (original instructions + custom logic)
    ///
    /// # Use Cases
    /// - Advanced scenarios requiring full control over execution flow
    /// - Performance-critical code (eliminates one extra jump instruction)
    ///
    /// # Warning
    /// If you skip trampoline and don't include original logic, the original functionality
    /// will be lost. This is an advanced feature - use with caution.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::TrampolineHook;
    ///
    /// // Complete shellcode: custom logic + original instruction
    /// let complete_shellcode = vec![
    ///     0x01, 0xD2,                              // custom: add edx,edx
    ///     0x01, 0x91, 0x08, 0x03, 0x00, 0x00,     // original: add [ecx+0x308],edx
    /// ];
    ///
    /// let mut hook = TrampolineHook::builder()
    ///     .handle(handle)
    ///     .target_address(target_addr)
    ///     .detour_code(complete_shellcode)
    ///     .x86()
    ///     .skip_trampoline(true)
    ///     .build()?;
    ///
    /// hook.install()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn skip_trampoline(mut self, skip: bool) -> Self {
        self.skip_trampoline = skip;
        self
    }

    /// Build the TrampolineHook with configured settings
    ///
    /// # Validation
    /// This method validates that all required parameters are set:
    /// - `handle`: Process handle
    /// - `target_address`: Address to hook
    /// - `detour_code`: Shellcode bytes
    ///
    /// # Errors
    /// Returns error if any required parameter is missing.
    pub fn build(self) -> Result<TrampolineHook, MemoryError> {
        // Validate required parameters
        let handle = self.handle.ok_or_else(|| {
            MemoryError::WriteFailed(
                "handle must be set. Call .handle(handle) before build().".to_string()
            )
        })?;

        let target_address = self.target_address.ok_or_else(|| {
            MemoryError::WriteFailed(
                "target_address must be set. Call .target_address(addr) before build(), \
                 or use .handle(handle).target_address(addr).build() chain.".to_string()
            )
        })?;

        let detour_code = self.detour_code.ok_or_else(|| {
            MemoryError::WriteFailed(
                "detour_code must be set. TrampolineHook only supports auto-allocated detour. \
                 Call .detour_code(shellcode) before build().".to_string()
            )
        })?;

        let bytes_to_overwrite = self.bytes_to_overwrite.unwrap_or_else(|| {
            match self.architecture.unwrap_or(HookArchitecture::X64) {
                HookArchitecture::X86 => 5,
                HookArchitecture::X64 => 14,
            }
        });

        Ok(TrampolineHook {
            handle: SendableHandle(handle),
            target_address,
            detour_address: 0, // Will be allocated during install
            detour_code: Some(detour_code),
            trampoline_address: self.trampoline_address,
            original_bytes: self.original_bytes.unwrap_or_default(),
            bytes_to_overwrite,
            is_installed: false,
            architecture: self.architecture.unwrap_or(HookArchitecture::X64),
            skip_trampoline: self.skip_trampoline,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Foundation::HANDLE;

    #[test]
    fn test_builder_x86_quick() {
        let shellcode = vec![0x01, 0xD2];
        let result = TrampolineHookBuilder::new()
            .handle(HANDLE::default())
            .target_address(0x1000)
            .detour_code(shellcode)
            .x86()
            .build();
        
        assert!(result.is_ok());
        // Hook created successfully
    }
    
    #[test]
    fn test_builder_fluent_api() {
        let shellcode = vec![0x01, 0xD2];
        let result = TrampolineHookBuilder::new()
            .handle(HANDLE::default())
            .target_address(0x1000)
            .detour_code(shellcode)
            .x86()
            .bytes_to_overwrite(6)
            .build();
        
        assert!(result.is_ok());
        // Hook created successfully with custom bytes_to_overwrite
    }
    
    #[test]
    fn test_builder_clone_support() {
        let builder = TrampolineHookBuilder::new()
            .detour_code(vec![0x01, 0xD2])
            .x86();
        
        // Clone and bind different parameters
        let hook1 = builder.clone()
            .handle(HANDLE::default())
            .target_address(0x1000)
            .build();
        
        let hook2 = builder.clone()
            .handle(HANDLE::default())
            .target_address(0x2000)
            .build();
        
        assert!(hook1.is_ok());
        assert!(hook2.is_ok());
    }
    
    #[test]
    fn test_builder_validation_missing_handle() {
        let result = TrampolineHookBuilder::new()
            .target_address(0x1000)
            .detour_code(vec![0x01, 0xD2])
            .x86()
            .build();
        
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("handle must be set"));
        }
    }
    
    #[test]
    fn test_builder_validation_missing_target_address() {
        let result = TrampolineHookBuilder::new()
            .handle(HANDLE::default())
            .detour_code(vec![0x01, 0xD2])
            .x86()
            .build();
        
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("target_address must be set"));
        }
    }
    
    #[test]
    fn test_builder_validation_missing_detour_code() {
        let result = TrampolineHookBuilder::new()
            .handle(HANDLE::default())
            .target_address(0x1000)
            .x86()
            .build();
        
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("detour_code must be set"));
        }
    }
}
