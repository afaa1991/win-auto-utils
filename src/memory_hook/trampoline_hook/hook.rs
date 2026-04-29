//! TrampolineHook Core Implementation
//!
//! This module contains the core implementation of TrampolineHook, including:
//! - Hook installation and uninstallation
//! - Memory allocation strategies
//! - Trampoline code generation
//! - Skip trampoline functionality

use crate::memory::{read_memory_bytes, write_memory_bytes, MemoryError};
use crate::memory_hook::shellcode::ShellcodeBuilder;
use crate::memory_hook::utils::{ProtectionGuard, SendableHandle};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Memory::{VirtualFreeEx, MEM_RELEASE};

/// Architecture detection for hook generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookArchitecture {
    X86,
    X64,
}

/// Trampoline hook that preserves the original function
///
/// This creates a copy of the original function's beginning (with overwritten bytes restored)
/// and a jump back to the original function after the hook point.
///
/// # Memory Management
/// 
/// TrampolineHook automatically allocates and manages both detour and trampoline memory.
/// The memory is freed when the hook is uninstalled or dropped.
///
/// ```no_run
/// use win_auto_utils::memory_hook::TrampolineHook;
///
/// // Quick constructor for x86
/// let shellcode = vec![0x01, 0xD2]; // add edx,edx
/// let mut hook = TrampolineHook::new_x86(handle, target_addr, shellcode);
/// hook.install()?; // Auto-allocates everything
/// hook.uninstall()?; // Auto-frees everything
/// ```
///
/// For advanced scenarios (custom memory addresses, pre-backed-up bytes), use [`crate::memory_hook::TrampolineHookBuilder`].

pub struct TrampolineHook {
    pub handle: SendableHandle,
    pub target_address: usize,
    pub detour_address: usize,
    pub detour_code: Option<Vec<u8>>, // For auto mode: shellcode to write
    pub trampoline_address: Option<usize>,
    pub original_bytes: Vec<u8>,
    pub bytes_to_overwrite: usize,
    pub is_installed: bool,
    pub architecture: HookArchitecture,
    pub skip_trampoline: bool,         // Skip trampoline generation, JMP directly to target+bytes
}

impl TrampolineHook {
    /// Quick constructor for x86 architecture with auto-allocated detour
    ///
    /// This is a convenience method for the most common use case:
    /// - Target process is x86 (32-bit)
    /// - You want automatic memory management
    ///
    /// # Arguments
    /// * `handle` - Process handle
    /// * `target_address` - Address to hook
    /// * `shellcode` - Your custom code bytes
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::TrampolineHook;
    ///
    /// let shellcode = vec![0x01, 0xD2, 0x01, 0x91, 0x08, 0x03, 0x00, 0x00];
    /// let mut hook = TrampolineHook::x86(handle, 0x41FAF2, shellcode);
    /// hook.install()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn x86(handle: HANDLE, target_address: usize, shellcode: Vec<u8>) -> Self {
        let mut hook = Self::auto_new(handle, target_address, shellcode);
        hook.architecture = HookArchitecture::X86;
        hook
    }

    /// Quick constructor for x64 architecture with auto-allocated detour
    ///
    /// Similar to `x86()` but for 64-bit processes.
    pub fn x64(handle: HANDLE, target_address: usize, shellcode: Vec<u8>) -> Self {
        let mut hook = Self::auto_new(handle, target_address, shellcode);
        hook.architecture = HookArchitecture::X64;
        hook
    }

    /// Create a builder for precise configuration
    /// All parameters are optional during construction - validation happens at build() time.
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
    /// For AOBScan workflows, see [`crate::memory_hook::TrampolineHookBuilder`] documentation.
    pub fn builder() -> crate::memory_hook::trampoline_hook::builder::TrampolineHookBuilder {
        crate::memory_hook::trampoline_hook::builder::TrampolineHookBuilder::new()
    }

    /// Create a new trampoline hook with automatic memory management (Recommended)
    ///
    /// TrampolineHook will automatically allocate and free both detour and trampoline memory.
    /// This is the safest and easiest mode to use - just provide your shellcode bytes!
    ///
    /// # Arguments
    /// * `handle` - Process handle with full access rights
    /// * `target_address` - Address of the function to hook
    /// * `detour_code` - Your custom shellcode bytes (without jump-back logic)
    ///
    /// # Note
    /// By default, this uses architecture-specific minimums for bytes_to_overwrite:
    /// - x86: 5 bytes (minimum for relative JMP)
    /// - x64: 14 bytes (for absolute JMP via RAX)
    /// 
    /// If your target instruction is shorter/longer, call `set_bytes_to_overwrite()` BEFORE `install()`.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::TrampolineHook;
    ///
    /// // Your custom logic: double the MP value
    /// let shellcode = vec![
    ///     0x01, 0xD2,                    // add edx,edx
    /// ];
    ///
    /// let mut hook = TrampolineHook::auto_new(handle, 0x41FAF2, shellcode);
    /// hook.set_architecture(false); // x86
    /// hook.install()?;
    /// hook.uninstall()?;
    /// ```
    pub fn auto_new(
        handle: HANDLE,
        target_address: usize,
        detour_code: Vec<u8>,
    ) -> Self {
        Self {
            handle: SendableHandle(handle),
            target_address,
            detour_address: 0, // Will be allocated during install
            detour_code: Some(detour_code),
            trampoline_address: None,
            original_bytes: Vec::new(),
            bytes_to_overwrite: 0, // Will be auto-calculated based on architecture in install()
            is_installed: false,
            architecture: HookArchitecture::X64,
            skip_trampoline: false,
        }
    }

    /// Create a new trampoline hook for x86 (32-bit) processes with auto-allocated memory
    ///
    /// This is the recommended constructor for 32-bit target processes. It automatically
    /// manages both detour and trampoline memory allocation and deallocation.
    ///
    /// # Arguments
    /// * `handle` - Process handle with full access rights
    /// * `target_address` - Address of the function to hook
    /// * `shellcode` - Your custom code bytes (without jump-back logic)
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::TrampolineHook;
    ///
    /// let shellcode = vec![0x01, 0xD2, 0x01, 0x91, 0x08, 0x03, 0x00, 0x00];
    /// let mut hook = TrampolineHook::new_x86(handle, 0x41FAF2, shellcode);
    /// hook.install()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_x86(handle: HANDLE, target_address: usize, shellcode: Vec<u8>) -> Self {
        let mut hook = Self::auto_new(handle, target_address, shellcode);
        hook.architecture = HookArchitecture::X86;
        hook
    }

    /// Create a new trampoline hook for x64 (64-bit) processes with auto-allocated memory
    ///
    /// This is the recommended constructor for 64-bit target processes. It automatically
    /// manages both detour and trampoline memory allocation and deallocation.
    ///
    /// # Arguments
    /// * `handle` - Process handle with full access rights
    /// * `target_address` - Address of the function to hook
    /// * `shellcode` - Your custom code bytes (without jump-back logic)
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::TrampolineHook;
    ///
    /// let shellcode = vec![0x01, 0xD2, 0x01, 0x91, 0x08, 0x03, 0x00, 0x00];
    /// let mut hook = TrampolineHook::new_x64(handle, 0x7FF6A1B2C3D4, shellcode);
    /// hook.install()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_x64(handle: HANDLE, target_address: usize, shellcode: Vec<u8>) -> Self {
        let mut hook = Self::auto_new(handle, target_address, shellcode);
        hook.architecture = HookArchitecture::X64;
        hook
    }

    /// Set the architecture explicitly
    ///
    /// # Arguments
    /// * `is_64bit` - true for x64, false for x86
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::TrampolineHook;
    ///
    /// let mut hook = TrampolineHook::auto_new(handle, 0x1000, shellcode);
    ///
    /// // For 32-bit process (e.g., LF2)
    /// hook.set_architecture(false);
    ///
    /// // For 64-bit process
    /// // hook.set_architecture(true);
    /// ```
    pub fn set_architecture(&mut self, is_64bit: bool) {
        self.architecture = if is_64bit {
            HookArchitecture::X64
        } else {
            HookArchitecture::X86
        };

        // Update default bytes_to_overwrite based on architecture
        if !self.is_installed {
            self.bytes_to_overwrite = if is_64bit { 14 } else { 5 };
        }
    }

    /// Set the number of bytes to overwrite at target address
    ///
    /// # Arguments
    /// * `bytes` - Number of bytes to overwrite (must be >= minimum for architecture)
    ///
    /// # Note
    /// - For x86: minimum 5 bytes (for relative JMP)
    /// - For x64: minimum 14 bytes (for absolute JMP)
    /// - Must not break instructions in the middle
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::TrampolineHook;
    ///
    /// let mut hook = TrampolineHook::auto_new(handle, 0x1000, shellcode);
    /// hook.set_bytes_to_overwrite(6); // Overwrite full 6-byte instruction
    /// ```
    pub fn set_bytes_to_overwrite(&mut self, bytes: usize) {
        self.bytes_to_overwrite = bytes;
    }

    /// Reset the hook to its initial state (safe for program restart)
    ///
    /// This method safely cleans up all internal state and allocated memory,
    /// allowing the hook to be reused after the target program restarts.
    ///
    /// # What it does:
    /// 1. If installed, uninstalls the hook first (restores original bytes)
    /// 2. Frees all auto-allocated memory (detour + trampoline)
    /// 3. Clears cached data (original_bytes, addresses, etc.)
    /// 4. Resets all flags to initial state
    ///
    /// # Safety
    /// - Safe to call multiple times (idempotent)
    /// - Safe to call even if never installed
    /// - After reset, the object can be used as if newly created
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::TrampolineHook;
    ///
    /// // First usage
    /// let mut hook = TrampolineHook::new_x86(handle, addr, shellcode);
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
        // If still installed, uninstall first (this will restore bytes and free memory)
        if self.is_installed {
            let _ = self.uninstall();
        }

        // Clear all cached state
        self.original_bytes.clear();
        self.detour_address = 0;
        self.trampoline_address = None;
        self.detour_code = None; // Clear shellcode cache too
        self.bytes_to_overwrite = 0;
        self.is_installed = false;
        self.skip_trampoline = false;         // Reset to default
    }

    /// Install the trampoline hook
    ///
    /// This will:
    /// 1. Auto-detect bytes_to_overwrite if not set (based on architecture minimums)
    /// 2. Read the original bytes from target function
    /// 3. Allocate detour memory and write shellcode (if in auto mode)
    /// 4. Allocate memory for the trampoline (unless skip_trampoline is enabled)
    /// 5. Copy original bytes to trampoline (unless skip_trampoline is enabled)
    /// 6. Add jump back to target + overwritten_bytes in trampoline (unless skip_trampoline is enabled)
    /// 7. Write jump to detour at target address
    ///
    /// # Returns
    /// The address of the trampoline (call this to invoke original function), or 0 if skip_trampoline is enabled
    ///
    /// # Errors
    /// Returns `MemoryError` if:
    /// - Hook already installed
    /// - Target address is already hooked (starts with JMP/CALL)
    /// - bytes_to_overwrite is less than minimum required for architecture
    /// - Memory allocation fails
    pub fn install(&mut self) -> Result<usize, MemoryError> {
        if self.is_installed {
            return Err(MemoryError::WriteFailed(
                "Hook already installed".to_string(),
            ));
        }

        // Step 1: Auto-calculate bytes_to_overwrite if not explicitly set
        let min_bytes = match self.architecture {
            HookArchitecture::X86 => 5,   // Minimum for JMP rel32
            HookArchitecture::X64 => 5,   // Minimum for JMP rel32 (will use absolute if needed)
        };

        if self.bytes_to_overwrite == 0 {
            // User didn't specify, use architecture minimum
            self.bytes_to_overwrite = min_bytes;
        } else if self.bytes_to_overwrite < min_bytes {
            // User specified but it's too small
            return Err(MemoryError::WriteFailed(
                format!(
                    "bytes_to_overwrite ({}) is less than minimum required for {:?} ({} bytes). \
                     JMP instruction needs at least {} bytes.",
                    self.bytes_to_overwrite,
                    self.architecture,
                    min_bytes,
                    min_bytes
                )
            ));
        }

        // Step 2: Read original bytes
        self.original_bytes =
            read_memory_bytes(self.handle.0, self.target_address, self.bytes_to_overwrite)?;

        // CRITICAL: Verify that we're reading original code, not an existing hook
        if !self.original_bytes.is_empty() {
            let first_byte = self.original_bytes[0];
            
            // Check for common hook signatures
            match first_byte {
                0xE9 => {
                    // Relative JMP (E9 xx xx xx xx)
                    return Err(MemoryError::WriteFailed(
                        format!("Target address 0x{:X} is already hooked (starts with JMP)", self.target_address)
                    ));
                }
                0xFF => {
                    // Absolute JMP or CALL (FF /4 or FF /2)
                    // Potentially already hooked, but continue anyway
                }
                _ => {
                    // Looks like normal code
                }
            }
        }

        // Step 3: Handle detour memory (auto mode vs manual mode)
        let detour_addr = if let Some(ref shellcode) = self.detour_code {
            // Auto mode: allocate detour memory and write shellcode
            let detour_size = shellcode.len() + 14; // shellcode + JMP to trampoline (or target)
            let detour_alloc = super::alloc::allocate_detour_memory(
                self.handle.0,
                self.target_address,
                detour_size,
                self.architecture,
            )?;
            
            // 🔒 CRITICAL SAFETY CHECK: Verify that Target → Detour jump will fit within bytes_to_overwrite
            // This MUST be done BEFORE writing any code to prevent silent corruption of subsequent instructions
            let jump_src = self.target_address;
            let jump_dst = detour_alloc;
            let distance = if jump_dst > jump_src {
                jump_dst - jump_src
            } else {
                jump_src - jump_dst
            };
            
            // Determine required jump instruction size based on distance
            let required_jump_size = if distance <= 0x7FFF_FFFF {
                5  // E9 relative jump (within ±2GB)
            } else {
                13 // MOV R11 + JMP R11 absolute jump (beyond ±2GB)
            };
            
            // SAFETY VALIDATION: Ensure we won't overwrite more bytes than allowed
            if required_jump_size > self.bytes_to_overwrite {
                return Err(MemoryError::WriteFailed(
                    format!(
                        "🔒 CRITICAL SAFETY CHECK FAILED:\n\
                         \n\
                         Distance from Target (0x{:X}) to Detour (0x{:X}) is {:#X} bytes ({:.2} MB).\n\
                         This {}.\n\n\
                         Required jump instruction size: {} bytes\n\
                         Your bytes_to_overwrite setting: {} bytes\n\
                         Overflow: {} extra bytes would be overwritten!\n\n\
                         ⚠️  If we proceed, we would corrupt {} bytes beyond your specified limit,\n\
                         potentially destroying subsequent instructions and causing program crashes.\n\n\
                         💡 SOLUTIONS:\n\
                         1. Increase bytes_to_overwrite to at least {} bytes:\n\
                            Call set_bytes_to_overwrite({}) before install()\n\
                         2. Ensure Detour memory allocates within ±2GB of target\n\
                            (Current allocation strategy already tries this)\n\
                         3. Find a different hook point with longer instruction sequence",
                        self.target_address,
                        detour_alloc,
                        distance,
                        distance as f64 / 1024.0 / 1024.0,
                        if distance <= 0x7FFF_FFFF { "requires relative jump" } else { "requires ABSOLUTE JUMP (>2GB)" },
                        required_jump_size,
                        self.bytes_to_overwrite,
                        required_jump_size - self.bytes_to_overwrite,
                        required_jump_size - self.bytes_to_overwrite,
                        required_jump_size,
                        required_jump_size
                    )
                ));
            }
            
            println!("[TrampolineHook] ✓ Safety check passed:");
            println!("  Distance: {:.2} MB ({})", distance as f64 / 1024.0 / 1024.0,
                     if distance <= 0x7FFF_FFFF { "relative jump OK" } else { "absolute jump needed" });
            println!("  Jump size: {} bytes ≤ bytes_to_overwrite: {} bytes", required_jump_size, self.bytes_to_overwrite);
            
            // Build complete detour code: shellcode + JMP to trampoline (or target)
            // (We'll add the JMP after we know trampoline/target address)
            write_memory_bytes(self.handle.0, detour_alloc, shellcode)?;
            
            self.detour_address = detour_alloc;
            detour_alloc
        } else {
            // Manual mode: user already provided detour_address
            self.detour_address
        };

        // Step 4: Handle trampoline (skip or allocate)
        let trampoline_addr = if self.skip_trampoline {
            // Skip trampoline: JMP directly to target + bytes_to_overwrite
            0 // No trampoline address
        } else {
            // Normal mode: allocate and build trampoline
            let trampoline_size = self.bytes_to_overwrite + 14; // Original bytes + JMP back
            let trampoline_addr = super::alloc::allocate_trampoline_nearby(
                self.handle.0,
                self.target_address,
                trampoline_size,
                self.architecture,
            )?;
            self.trampoline_address = Some(trampoline_addr);

            // Step 5: If auto mode, append JMP from detour to trampoline
            if self.detour_code.is_some() {
                let shellcode_len = self.detour_code.as_ref().unwrap().len();
                let jmp_src = detour_addr + shellcode_len;
                let jmp_dst = trampoline_addr;
                
                // Calculate distance to determine jump type
                let distance = if jmp_dst > jmp_src {
                    jmp_dst - jmp_src
                } else {
                    jmp_src - jmp_dst
                };
                
                // Use relative jump if within 2GB, otherwise absolute jump
                if distance <= 0x7FFF_FFFF {
                    // Within 2GB: use E9 relative jump (5 bytes)
                    let offset = (jmp_dst as i64).wrapping_sub((jmp_src + 5) as i64);
                    
                    if offset < i32::MIN as i64 || offset > i32::MAX as i64 {
                        return Err(MemoryError::WriteFailed("Detour and Trampoline too far for relative jump".to_string()));
                    }

                    let mut jmp_code = vec![0xE9]; // JMP rel32
                    jmp_code.extend_from_slice(&(offset as i32).to_le_bytes());
                    
                    write_memory_bytes(self.handle.0, jmp_src, &jmp_code)?;
                } else {
                    // Beyond 2GB: use absolute jump via R11 (13 bytes)
                    // Note: This writes to detour memory (which we allocated), NOT target memory
                    // So bytes_to_overwrite does NOT limit this operation
                    
                    let mut builder = ShellcodeBuilder::new_x64();
                    builder.jmp_absolute(jmp_dst);
                    let jmp_code = builder.build();
                    
                    write_memory_bytes(self.handle.0, jmp_src, &jmp_code)?;
                }
            }

            // Step 6: Build and write trampoline code
            let trampoline_code = self.build_trampoline_code(trampoline_addr)?;
            
            println!("[TrampolineHook] Trampoline code length: {} bytes", trampoline_code.len());
            println!("[TrampolineHook] Trampoline code: {:02X?}", trampoline_code);
            
            write_memory_bytes(self.handle.0, trampoline_addr, &trampoline_code)?;
            
            // Verify written bytes
            let verify_bytes = read_memory_bytes(self.handle.0, trampoline_addr, trampoline_code.len())?;
            println!("[TrampolineHook] Verified written bytes: {:02X?}", verify_bytes);
            
            if verify_bytes != trampoline_code {
                eprintln!("[TrampolineHook] ⚠️  WARNING: Written bytes don't match!");
                eprintln!("[TrampolineHook] Expected: {:02X?}", trampoline_code);
                eprintln!("[TrampolineHook] Actual:   {:02X?}", verify_bytes);
            }

            trampoline_addr
        };

        // Step 7: If skip_trampoline, append JMP from detour to target+bytes
        if self.skip_trampoline && self.detour_code.is_some() {
            let shellcode_len = self.detour_code.as_ref().unwrap().len();
            let jmp_src = detour_addr + shellcode_len;
            let jmp_dst = self.target_address + self.bytes_to_overwrite;
            
            // Calculate distance to determine jump type
            let distance = if jmp_dst > jmp_src {
                jmp_dst - jmp_src
            } else {
                jmp_src - jmp_dst
            };
            
            // Use relative jump if within 2GB, otherwise absolute jump
            if distance <= 0x7FFF_FFFF {
                // Within 2GB: use E9 relative jump (5 bytes)
                let offset = (jmp_dst as i64).wrapping_sub((jmp_src + 5) as i64);
                
                if offset < i32::MIN as i64 || offset > i32::MAX as i64 {
                    return Err(MemoryError::WriteFailed("Detour and target too far for relative jump".to_string()));
                }

                let mut jmp_code = vec![0xE9]; // JMP rel32
                jmp_code.extend_from_slice(&(offset as i32).to_le_bytes());
                
                write_memory_bytes(self.handle.0, jmp_src, &jmp_code)?;
            } else {
                // Beyond 2GB: use absolute jump via R11 (13 bytes)
                // Note: This writes to detour memory (which we allocated), NOT target memory
                // So bytes_to_overwrite does NOT limit this operation
                
                let mut builder = ShellcodeBuilder::new_x64();
                builder.jmp_absolute(jmp_dst);
                let jmp_code = builder.build();
                
                write_memory_bytes(self.handle.0, jmp_src, &jmp_code)?;
            }
        }

        // Step 8: Install inline hook at target (JMP to detour)
        self.install_inline_hook()?;

        self.is_installed = true;

        Ok(trampoline_addr)
    }

    /// Uninstall the hook and free allocated memory
    ///
    /// This will:
    /// 1. Restore original bytes at target address
    /// 2. Free trampoline memory (if allocated)
    /// 3. Free detour memory (if allocated in auto mode)
    pub fn uninstall(&mut self) -> Result<(), MemoryError> {
        if !self.is_installed {
            return Err(MemoryError::WriteFailed("Hook not installed".to_string()));
        }

        // Restore original bytes at target
        let _guard =
            ProtectionGuard::new(self.handle.0, self.target_address, self.bytes_to_overwrite)?;

        write_memory_bytes(self.handle.0, self.target_address, &self.original_bytes)?;

        // Free trampoline memory if it exists
        if let Some(addr) = self.trampoline_address {
            unsafe {
                let result = VirtualFreeEx(
                    self.handle.0, 
                    addr as *mut std::ffi::c_void, 
                    0, 
                    MEM_RELEASE
                );

                match result {
                    Ok(()) => {
                        // Successfully freed
                    }
                    Err(e) => {
                        return Err(MemoryError::WriteFailed(
                            format!("Failed to free trampoline memory at 0x{:X}: {:?}", addr, e)
                        ));
                    }
                }
            }
        }

        // Free detour memory if it was allocated (auto mode)
        if self.detour_code.is_some() && self.detour_address != 0 {
            unsafe {
                let result = VirtualFreeEx(
                    self.handle.0, 
                    self.detour_address as *mut std::ffi::c_void, 
                    0, 
                    MEM_RELEASE
                );

                match result {
                    Ok(()) => {
                        // Successfully freed
                    }
                    Err(e) => {
                        return Err(MemoryError::WriteFailed(
                            format!("Failed to free detour memory at 0x{:X}: {:?}", self.detour_address, e)
                        ));
                    }
                }
            }
        }

        self.trampoline_address = None;
        self.detour_address = 0;
        self.is_installed = false;
        self.original_bytes.clear();

        Ok(())
    }

    /// Get the trampoline address
    ///
    /// Call this after installation to get the address to call the original function
    /// Returns 0 if skip_trampoline is enabled
    pub fn get_trampoline_address(&self) -> Option<usize> {
        self.trampoline_address
    }

    /// Check if the hook is installed
    pub fn is_installed(&self) -> bool {
        self.is_installed
    }

    /// Build the trampoline code
    fn build_trampoline_code(&self, trampoline_addr: usize) -> Result<Vec<u8>, MemoryError> {
        let mut builder = match self.architecture {
            HookArchitecture::X86 => ShellcodeBuilder::new_x86(),
            HookArchitecture::X64 => ShellcodeBuilder::new_x64(),
        };

        // Copy original bytes
        builder.append_bytes(&self.original_bytes);

        // Add jump back to target + overwritten_bytes
        let return_address = self.target_address + self.bytes_to_overwrite;
        let current_address = trampoline_addr + self.original_bytes.len();

        // Calculate distance to determine jump type
        let distance = if return_address > current_address {
            return_address - current_address
        } else {
            current_address - return_address
        };

        // Use relative jump if within 2GB, otherwise absolute jump
        if distance <= 0x7FFF_FFFF {
            // Within 2GB: use E9 relative jump (5 bytes)
            builder.jmp_relative(current_address, return_address);
        } else {
            // Beyond 2GB: use absolute jump via R11 (13 bytes)
            // IMPORTANT: This should rarely happen since trampoline is allocated nearby
            builder.jmp_absolute(return_address);
        }

        Ok(builder.build())
    }

    /// Install the inline hook at target address
    fn install_inline_hook(&self) -> Result<(), MemoryError> {
        let jump_code = self.generate_jump_to_detour()?;

        // Verify jump code fits within bytes_to_overwrite
        if jump_code.len() > self.bytes_to_overwrite {
            return Err(MemoryError::WriteFailed(
                format!(
                    "Jump instruction ({} bytes) exceeds bytes_to_overwrite ({} bytes). \
                     This usually means the distance requires absolute jump but insufficient bytes were specified.",
                    jump_code.len(),
                    self.bytes_to_overwrite
                )
            ));
        }

        // If we need to overwrite more bytes than the jump instruction takes,
        // pad with NOP instructions to avoid executing partial instructions
        let jump_size = jump_code.len();
        if self.bytes_to_overwrite > jump_size {
            let padding_size = self.bytes_to_overwrite - jump_size;
            let mut final_code = jump_code.clone();
            final_code.extend_from_slice(&vec![0x90; padding_size]); // NOP padding
            
            let _guard = ProtectionGuard::new(self.handle.0, self.target_address, self.bytes_to_overwrite)?;
            write_memory_bytes(self.handle.0, self.target_address, &final_code)?;
        } else {
            let _guard = ProtectionGuard::new(self.handle.0, self.target_address, self.bytes_to_overwrite)?;
            write_memory_bytes(self.handle.0, self.target_address, &jump_code)?;
        }

        Ok(())
    }

    /// Generate jump code to detour function
    fn generate_jump_to_detour(&self) -> Result<Vec<u8>, MemoryError> {
        let mut builder = match self.architecture {
            HookArchitecture::X86 => ShellcodeBuilder::new_x86(),
            HookArchitecture::X64 => ShellcodeBuilder::new_x64(),
        };

        // Calculate distance to determine jump type
        let distance = if self.detour_address > self.target_address {
            self.detour_address - self.target_address
        } else {
            self.target_address - self.detour_address
        };

        // Use relative jump if within 2GB, otherwise absolute jump
        if distance <= 0x7FFF_FFFF {
            // Within 2GB: use E9 relative jump (5 bytes)
            builder.jmp_relative(self.target_address, self.detour_address);
        } else {
            // Beyond 2GB: use absolute jump via R11 (13 bytes)
            // IMPORTANT: Verify user specified enough bytes_to_overwrite
            if self.bytes_to_overwrite < 13 {
                return Err(MemoryError::WriteFailed(
                    format!(
                        "Absolute jump required (distance > 2GB: {:#X}), but bytes_to_overwrite ({}) is insufficient. \
                         Need at least 13 bytes for MOV R11 + JMP R11 instruction. \
                         Please call set_bytes_to_overwrite(13) before install().",
                        distance,
                        self.bytes_to_overwrite
                    )
                ));
            }
            
            builder.jmp_absolute(self.detour_address);
        }

        Ok(builder.build())
    }
}

impl Drop for TrampolineHook {
    fn drop(&mut self) {
        if self.is_installed {
            let _ = self.uninstall();
        }
    }
}