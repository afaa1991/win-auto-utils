//! Inline Hook Implementation
//!
//! Provides functionality to redirect function execution by modifying the target function's entry point.
//! This is a simple hooking technique that directly patches the target address with a jump instruction.
//!
//! # Architecture Support
//! - **x86**: Uses 5-byte relative JMP (`E9 xx xx xx xx`)
//! - **x64**: Uses 14-byte absolute JMP (`MOV RAX, addr` + `JMP RAX`)
//!
//! # Use Cases
//! - Function interception and replacement
//! - API hooking for monitoring or modification
//! - Simple code redirection without preserving original behavior
//!
//! # Limitations
//! - Does not automatically preserve original instructions
//! - If you need to call the original function, use [`TrampolineHook`](super::TrampolineHook) instead
//! - Must manually handle register preservation in detour function if needed
//!
//! # Quick Start
//! ```no_run
//! use win_auto_utils::memory_hook::InlineHook;
//! use win_auto_utils::handle::open_process_handle;
//! use windows::Win32::System::Threading::PROCESS_ALL_ACCESS;
//!
//! fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let handle = open_process_handle(12345, PROCESS_ALL_ACCESS).unwrap();
//!     
//!     // For x86 process
//!     let mut hook = InlineHook::new_x86(handle, 0x1000, 0x2000);
//!     
//!     // For x64 process (or use InlineHook::new for default)
//!     let mut hook = InlineHook::new_x64(handle, 0x7FF6A1B2C3D4, 0x7FF6A1B2E5F6);
//!     
//!     // Install the hook
//!     hook.install()?;
//!     
//!     // Now all calls to target will jump to detour
//!     // ... your code here ...
//!     
//!     // Remove hook when done (automatically restores original bytes)
//!     hook.uninstall()?;
//!     Ok(())
//! }
//! ```
use windows::Win32::Foundation::HANDLE;
use crate::memory::{read_memory_bytes, write_memory_bytes, MemoryError};
use crate::memory_hook::shellcode::ShellcodeBuilder;
use crate::memory_hook::utils::{ProtectionGuard, SendableHandle};

/// Inline hook for redirecting function execution
///
/// # Quick Start
/// ```no_run
/// use win_auto_utils::memory_hook::InlineHook;
/// use win_auto_utils::handle::open_process_handle;
/// use windows::Win32::System::Threading::PROCESS_ALL_ACCESS;
///
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let handle = open_process_handle(12345, PROCESS_ALL_ACCESS).unwrap();
///     
///     // Hook target function at 0x1000 to redirect to 0x2000
///     let mut hook = InlineHook::new(handle, 0x1000, 0x2000);
///     hook.install()?;
///     
///     // Now calls to 0x1000 will jump to 0x2000
///     
///     // Remove hook when done
///     hook.uninstall()?;
///     Ok(())
/// }
/// ```
pub struct InlineHook {
    handle: SendableHandle,
    target_address: usize,
    detour_address: usize,
    original_bytes: Vec<u8>,
    is_installed: bool,
    architecture: HookArchitecture,
}

/// Architecture detection for hook generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookArchitecture {
    X86,
    X64,
}

impl InlineHook {
    /// Create a builder for precise configuration
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::{InlineHook, HookArchitecture};
    ///```
    pub fn builder() -> InlineHookBuilder {
        InlineHookBuilder::new()
    }

    /// Create a new inline hook for x86 (32-bit) processes
    ///
    /// This is the recommended constructor for 32-bit target processes.
    /// It creates a simple inline hook that redirects execution from the target
    /// address to the detour address.
    ///
    /// # Arguments
    /// * `handle` - Process handle with full access rights
    /// * `target_address` - Address of the function to hook
    /// * `detour_address` - Address of the replacement function
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::InlineHook;
    ///
    /// let mut hook = InlineHook::new_x86(handle, 0x1000, 0x2000);
    /// hook.install()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_x86(handle: HANDLE, target_address: usize, detour_address: usize) -> Self {
        Self {
            handle: SendableHandle(handle),
            target_address,
            detour_address,
            original_bytes: Vec::new(),
            is_installed: false,
            architecture: HookArchitecture::X86,
        }
    }

    /// Create a new inline hook for x64 (64-bit) processes
    ///
    /// This is the recommended constructor for 64-bit target processes.
    /// It creates a simple inline hook that redirects execution from the target
    /// address to the detour address using absolute jump (MOV RAX + JMP RAX).
    ///
    /// # Arguments
    /// * `handle` - Process handle with full access rights
    /// * `target_address` - Address of the function to hook
    /// * `detour_address` - Address of the replacement function
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::InlineHook;
    ///
    /// let mut hook = InlineHook::new_x64(handle, 0x7FF6A1B2C3D4, 0x7FF6A1B2E5F6);
    /// hook.install()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_x64(handle: HANDLE, target_address: usize, detour_address: usize) -> Self {
        Self {
            handle: SendableHandle(handle),
            target_address,
            detour_address,
            original_bytes: Vec::new(),
            is_installed: false,
            architecture: HookArchitecture::X64,
        }
    }

    /// Create a new inline hook (defaults to x64 architecture)
    ///
    /// This is a convenience constructor for quick hooking when you don't need
    /// to specify the architecture explicitly. Defaults to x64 for modern applications.
    ///
    /// # Arguments
    /// * `handle` - Process handle with full access rights
    /// * `target_address` - Address of the function to hook
    /// * `detour_address` - Address of the replacement function
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::InlineHook;
    ///
    /// // For 64-bit process (default)
    /// let mut hook = InlineHook::new(handle, 0x7FF6A1B2C3D4, 0x7FF6A1B2E5F6);
    /// hook.install()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(handle: HANDLE, target_address: usize, detour_address: usize) -> Self {
        Self::new_x64(handle, target_address, detour_address)
    }

    /// Set the architecture explicitly
    ///
    /// # Arguments
    /// * `is_64bit` - true for x64, false for x86
    pub fn set_architecture(&mut self, is_64bit: bool) {
        self.architecture = if is_64bit {
            HookArchitecture::X64
        } else {
            HookArchitecture::X86
        };
    }

    /// Reset the hook to its initial state (safe for program restart)
    ///
    /// This method safely cleans up all internal state, allowing the hook
    /// to be reused after the target program restarts.
    ///
    /// # What it does:
    /// 1. If installed, uninstalls the hook first (restores original bytes)
    /// 2. Clears cached data (original_bytes, etc.)
    /// 3. Resets all flags to initial state
    ///
    /// # Safety
    /// - Safe to call multiple times (idempotent)
    /// - Safe to call even if never installed
    /// - After reset, the object can be used as if newly created
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::InlineHook;
    ///
    /// // First usage
    /// let mut hook = InlineHook::new_x86(handle, target, detour);
    /// hook.install()?;
    /// hook.uninstall()?;
    ///
    /// // Program restarted - reset state
    /// hook.reset();
    ///
    /// // Second usage (no side effects)
    /// hook.install()?; // Will read fresh original bytes
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn reset(&mut self) {
        // If still installed, uninstall first (this will restore bytes)
        if self.is_installed {
            let _ = self.uninstall();
        }

        // Clear all cached state
        self.original_bytes.clear();
        self.is_installed = false;
    }

    /// Install the hook
    ///
    /// This will:
    /// 1. Save the original bytes at the target address
    /// 2. Generate jump code to the detour function
    /// 3. Write the jump code to the target address
    ///
    /// # Errors
    /// Returns `MemoryError` if reading or writing fails
    pub fn install(&mut self) -> Result<(), MemoryError> {
        if self.is_installed {
            return Err(MemoryError::WriteFailed("Hook already installed".to_string()));
        }
        
        // Determine how many bytes we need to overwrite
        let required_bytes = match self.architecture {
            HookArchitecture::X86 => 5,  // JMP rel32
            HookArchitecture::X64 => 14, // MOV RAX + JMP RAX (for absolute address)
        };
        
        // Save original bytes
        self.original_bytes = read_memory_bytes(self.handle.0, self.target_address, required_bytes)?;
        
        // Generate shellcode for the jump
        let jump_code = self.generate_jump_code()?;
        
        // Change memory protection to allow writing
        let _guard = ProtectionGuard::new(self.handle.0, self.target_address, required_bytes)?;
        
        // Write the jump code
        write_memory_bytes(self.handle.0, self.target_address, &jump_code)?;
        
        self.is_installed = true;
        Ok(())
    }
    
    /// Uninstall the hook and restore original bytes
    ///
    /// # Errors
    /// Returns `MemoryError` if restoration fails
    pub fn uninstall(&mut self) -> Result<(), MemoryError> {
        if !self.is_installed {
            return Err(MemoryError::WriteFailed("Hook not installed".to_string()));
        }
        
        if self.original_bytes.is_empty() {
            return Err(MemoryError::WriteFailed("No original bytes to restore".to_string()));
        }
        
        // Change memory protection to allow writing
        let _guard = ProtectionGuard::new(
            self.handle.0, 
            self.target_address, 
            self.original_bytes.len()
        )?;
        
        // Restore original bytes
        write_memory_bytes(self.handle.0, self.target_address, &self.original_bytes)?;
        
        self.is_installed = false;
        Ok(())
    }
    
    /// Check if the hook is currently installed
    pub fn is_installed(&self) -> bool {
        self.is_installed
    }

    /// Get the original bytes that were overwritten
    pub fn get_original_bytes(&self) -> &[u8] {
        &self.original_bytes
    }
    
    /// Generate the jump code based on architecture
    fn generate_jump_code(&self) -> Result<Vec<u8>, MemoryError> {
        let mut builder = match self.architecture {
            HookArchitecture::X86 => ShellcodeBuilder::new_x86(),
            HookArchitecture::X64 => ShellcodeBuilder::new_x64(),
        };
        
        // For x64 with large addresses, use absolute jump
        match self.architecture {
            HookArchitecture::X86 => {
                // Try relative jump first
                let offset = (self.detour_address as i64) - (self.target_address as i64) - 5;
                
                if offset >= i32::MIN as i64 && offset <= i32::MAX as i64 {
                    // Use relative jump
                    builder.jmp_relative(self.target_address, self.detour_address);
                } else {
                    // Use absolute jump (PUSH + RET)
                    builder.jmp_absolute(self.detour_address);
                }
            }
            HookArchitecture::X64 => {
                // Always use absolute jump for x64 to handle full 64-bit addresses
                builder.jmp_absolute(self.detour_address);
            }
        }
        
        Ok(builder.build())
    }
}

impl Drop for InlineHook {
    fn drop(&mut self) {
        if self.is_installed {
            let _ = self.uninstall();
        }
    }
}

/// Builder for configuring InlineHook
///
/// Provides a fluent API for building InlineHook with optional configuration.
#[derive(Debug)]
pub struct InlineHookBuilder {
    handle: Option<HANDLE>,
    target_address: Option<usize>,
    detour_address: Option<usize>,
    architecture: Option<HookArchitecture>,
}

impl InlineHookBuilder {
    fn new() -> Self {
        Self {
            handle: None,
            target_address: None,
            detour_address: None,
            architecture: None,
        }
    }

    /// Set process handle (required for install)
    pub fn handle(mut self, handle: HANDLE) -> Self {
        self.handle = Some(handle);
        self
    }

    /// Set target address (required for install)
    pub fn target_address(mut self, addr: usize) -> Self {
        self.target_address = Some(addr);
        self
    }

    /// Set detour address (required for install)
    pub fn detour_address(mut self, addr: usize) -> Self {
        self.detour_address = Some(addr);
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

    /// Build the InlineHook with configured settings
    ///
    /// # Validation
    /// This method validates that all required parameters are set:
    /// - `handle`: Process handle
    /// - `target_address`: Address to hook
    /// - `detour_address`: Address of replacement function
    ///
    /// # Errors
    /// Returns error if any required parameter is missing.
    pub fn build(self) -> Result<InlineHook, MemoryError> {
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

        let detour_address = self.detour_address.ok_or_else(|| {
            MemoryError::WriteFailed(
                "detour_address must be set. Call .detour_address(addr) before build().".to_string()
            )
        })?;

        Ok(InlineHook {
            handle: SendableHandle(handle),
            target_address,
            detour_address,
            original_bytes: Vec::new(),
            is_installed: false,
            architecture: self.architecture.unwrap_or(HookArchitecture::X64),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_hook_creation() {
        let hook = InlineHook::new_x64(HANDLE::default(), 0x1000, 0x2000);
        
        assert!(!hook.is_installed());
        assert_eq!(hook.get_original_bytes().len(), 0);
    }
    
    #[test]
    fn test_builder_api() {
        let result = InlineHook::builder()
            .handle(HANDLE::default())
            .target_address(0x1000)
            .detour_address(0x2000)
            .x86()
            .build();
        
        assert!(result.is_ok());
        let hook = result.unwrap();
        assert_eq!(hook.architecture, HookArchitecture::X86);
    }
}
