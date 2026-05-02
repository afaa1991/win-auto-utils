//! TrampolineHook Handler for function hooking

use super::address_strategy::AddressStrategy;
use crate::memory::MemoryError;
use crate::memory_hook::{Architecture, TrampolineHook};

use super::super::register::ModifierHandler;

// ==================== TrampolineHook Config ====================

/// TrampolineHook configuration (pure data)
#[derive(Clone, Debug)]
pub struct TrampolineHookConfig {
    pub name: String,
    pub shellcode: Vec<u8>,
    pub bytes_to_overwrite: usize,
    pub skip_trampoline: bool,
}

// ==================== TrampolineHook Handler ====================

/// TrampolineHook handler with embedded address strategy
pub struct TrampolineHookHandler {
    config: TrampolineHookConfig,
    /// Address resolution strategy
    address_strategy: AddressStrategy,
    /// Cached resolved address (avoids repeated scans)
    cached_address: Option<usize>,
    instance: Option<TrampolineHook>,
}

impl TrampolineHookHandler {
    /// Create a new TrampolineHookHandler
    fn new(config: TrampolineHookConfig, address_strategy: AddressStrategy) -> Self {
        Self {
            config,
            address_strategy,
            cached_address: None,
            instance: None,
        }
    }

    /// Create a TrampolineHookHandler with AOB scanning (recommended).
    ///
    /// Uses AOB pattern scanning to find the target address at runtime.
    /// This is the most robust approach as it works across game updates.
    ///
    /// # Performance Note
    /// - First activation: Executes AOB scan (may take 10-500ms)
    /// - Subsequent activations: Reuses cached address (instant!)
    /// - Cache auto-invalidates when PID changes
    ///
    /// # Arguments
    /// * `name` - Hook name (must be unique)
    /// * `pattern` - AOB pattern string (space-separated hex bytes, e.g., "48 89 5C 24")
    /// * `shellcode` - Your custom code bytes to execute at the hook point
    /// * `bytes_to_overwrite` - Number of bytes to overwrite (MUST equal original instruction length!)
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_manager::builtin::TrampolineHookHandler;
    ///
    /// let hook = TrampolineHookHandler::new_hook_aob(
    ///     "my_hook",
    ///     "48 89 5C 24",
    ///     vec![0x90, 0x90],  // NOP instructions
    ///     5,                  // Original instruction is 5 bytes
    /// )?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_hook_aob(
        name: impl Into<String>,
        pattern: &str,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        Self::new_hook_aob_with_range(name, pattern, shellcode, bytes_to_overwrite, 0, 0)
    }

    /// Create a TrampolineHookHandler with AOB scanning and custom search range.
    ///
    /// Same as `new_hook_aob()` but allows specifying a memory range for faster scanning.
    /// Use this when you know the approximate location of the target code.
    ///
    /// # Arguments
    /// * `name` - Hook name
    /// * `pattern` - AOB pattern string
    /// * `shellcode` - Your custom code bytes
    /// * `bytes_to_overwrite` - Number of bytes to overwrite
    /// * `start_address` - Starting address for the scan (0 = from beginning)
    /// * `length` - Length of the scan range (0 = entire process memory)
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_manager::builtin::TrampolineHookHandler;
    ///
    /// // Scan only within a specific module range (much faster!)
    /// let hook = TrampolineHookHandler::new_hook_aob_with_range(
    ///     "my_hook",
    ///     "48 89 5C 24",
    ///     vec![0x90, 0x90],
    ///     5,
    ///     0x10000000,  // Start address (e.g., module base)
    ///     0x100000,    // Length (e.g., module size)
    /// )?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_hook_aob_with_range(
        name: impl Into<String>,
        pattern: &str,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
        start_address: usize,
        length: usize,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        Self::new_hook_aob_with_range_and_offset(
            name,
            pattern,
            shellcode,
            bytes_to_overwrite,
            start_address,
            length,
            0,
        )
    }

    /// Create a TrampolineHookHandler with AOB scanning and offset.
    ///
    /// Same as `new_hook_aob()` but supports adding an offset to the scanned address.
    /// Useful when the AOB pattern finds a nearby signature, but you need to hook at a different location.
    ///
    /// # Arguments
    /// * `name` - Hook name
    /// * `pattern` - AOB pattern string
    /// * `shellcode` - Your custom code bytes
    /// * `bytes_to_overwrite` - Number of bytes to overwrite
    /// * `offset` - Offset to add to the scanned address (can be negative)
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_manager::builtin::TrampolineHookHandler;
    ///
    /// // Scan for function prologue, but hook 0x42 bytes later
    /// let hook = TrampolineHookHandler::new_hook_aob_with_offset(
    ///     "my_hook",
    ///     "48 89 5C 24",
    ///     vec![0x90, 0x90],
    ///     5,
    ///     0x42,
    /// )?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_hook_aob_with_offset(
        name: impl Into<String>,
        pattern: &str,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
        offset: i64,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let config = TrampolineHookConfig {
            name: name.into(),
            shellcode,
            bytes_to_overwrite,
            skip_trampoline: false,
        };

        Ok(Box::new(Self::new(
            config,
            AddressStrategy::aob_pattern_with_offset(pattern, offset),
        )))
    }

    /// Create a TrampolineHookHandler with AOB scanning, custom range and offset.
    ///
    /// Combines range-limited scanning with post-scan address correction.
    ///
    /// # Arguments
    /// * `name` - Hook name
    /// * `pattern` - AOB pattern string
    /// * `shellcode` - Your custom code bytes
    /// * `bytes_to_overwrite` - Number of bytes to overwrite
    /// * `start_address` - Starting address for the scan (0 = from beginning)
    /// * `length` - Length of the scan range (0 = entire process memory)
    /// * `offset` - Offset to add to the scanned address (can be negative)
    pub fn new_hook_aob_with_range_and_offset(
        name: impl Into<String>,
        pattern: &str,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
        start_address: usize,
        length: usize,
        offset: i64,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let config = TrampolineHookConfig {
            name: name.into(),
            shellcode,
            bytes_to_overwrite,
            skip_trampoline: false,
        };

        Ok(Box::new(Self::new(
            config,
            AddressStrategy::aob_pattern_with_range_and_offset(
                pattern,
                start_address,
                length,
                offset,
            ),
        )))
    }

    /// Create a TrampolineHookHandler with skip_trampoline mode and AOB scanning.
    ///
    /// In skip_trampoline mode, the hook jumps directly to target+bytes without creating a trampoline.
    /// Use this when you don't need to call the original function.
    ///
    /// # Arguments
    /// * `name` - Hook name
    /// * `pattern` - AOB pattern string
    /// * `shellcode` - Your custom code bytes
    /// * `bytes_to_overwrite` - Number of bytes to overwrite
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_manager::builtin::TrampolineHookHandler;
    ///
    /// let hook = TrampolineHookHandler::new_skip_trampoline_aob(
    ///     "nop_patch",
    ///     "90 90 90 90 90",
    ///     vec![0x90, 0x90, 0x90, 0x90, 0x90],
    ///     5,
    /// )?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_skip_trampoline_aob(
        name: impl Into<String>,
        pattern: &str,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        Self::new_skip_trampoline_aob_with_range(name, pattern, shellcode, bytes_to_overwrite, 0, 0)
    }

    /// Create a TrampolineHookHandler with skip_trampoline mode, AOB scanning and custom range.
    pub fn new_skip_trampoline_aob_with_range(
        name: impl Into<String>,
        pattern: &str,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
        start_address: usize,
        length: usize,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        Self::new_skip_trampoline_aob_with_range_and_offset(
            name,
            pattern,
            shellcode,
            bytes_to_overwrite,
            start_address,
            length,
            0,
        )
    }

    /// Create a TrampolineHookHandler with skip_trampoline mode, AOB scanning and offset.
    pub fn new_skip_trampoline_aob_with_offset(
        name: impl Into<String>,
        pattern: &str,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
        offset: i64,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let config = TrampolineHookConfig {
            name: name.into(),
            shellcode,
            bytes_to_overwrite,
            skip_trampoline: true,
        };

        Ok(Box::new(Self::new(
            config,
            AddressStrategy::aob_pattern_with_offset(pattern, offset),
        )))
    }

    /// Create a TrampolineHookHandler with skip_trampoline mode, AOB scanning, custom range and offset.
    pub fn new_skip_trampoline_aob_with_range_and_offset(
        name: impl Into<String>,
        pattern: &str,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
        start_address: usize,
        length: usize,
        offset: i64,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let config = TrampolineHookConfig {
            name: name.into(),
            shellcode,
            bytes_to_overwrite,
            skip_trampoline: true,
        };

        Ok(Box::new(Self::new(
            config,
            AddressStrategy::aob_pattern_with_range_and_offset(
                pattern,
                start_address,
                length,
                offset,
            ),
        )))
    }

    /// Create a TrampolineHookHandler with static address pattern.
    ///
    /// Architecture is automatically detected at activation time from ProcessContext.
    /// Use this when the address is stable and doesn't change across game updates.
    ///
    /// # Arguments
    /// * `name` - Hook name
    /// * `pattern` - Static address pattern string (e.g., "game.exe+1000")
    /// * `shellcode` - Your custom code bytes
    /// * `bytes_to_overwrite` - Number of bytes to overwrite
    ///
    /// # Note
    /// For static addresses, include the offset directly in the pattern string.
    /// Example: "game.exe+0x44ef1" (offset is part of the pattern)
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_manager::builtin::TrampolineHookHandler;
    ///
    /// let hook = TrampolineHookHandler::new_hook_static(
    ///     "my_hook",
    ///     "game.exe+3000",
    ///     vec![0x90, 0x90],
    ///     5,
    /// )?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_hook_static(
        name: impl Into<String>,
        pattern: &str,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
    ) -> Result<Box<dyn ModifierHandler>, crate::memory_resolver::ParseError> {
        let config = TrampolineHookConfig {
            name: name.into(),
            shellcode,
            bytes_to_overwrite,
            skip_trampoline: false,
        };

        Ok(Box::new(Self::new(
            config,
            AddressStrategy::static_pattern(pattern),
        )))
    }

    /// Create a TrampolineHookHandler with static address in skip_trampoline mode.
    ///
    /// In skip_trampoline mode, the hook jumps directly to target+bytes without creating a trampoline.
    /// Use this when you don't need to call the original function.
    ///
    /// # Note
    /// For static addresses, include the offset directly in the pattern string.
    pub fn new_skip_trampoline_static(
        name: impl Into<String>,
        pattern: &str,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
    ) -> Result<Box<dyn ModifierHandler>, crate::memory_resolver::ParseError> {
        let config = TrampolineHookConfig {
            name: name.into(),
            shellcode,
            bytes_to_overwrite,
            skip_trampoline: true,
        };

        Ok(Box::new(Self::new(
            config,
            AddressStrategy::static_pattern(pattern),
        )))
    }
}

impl ModifierHandler for TrampolineHookHandler {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn activate(
        &mut self,
        ctx: &crate::memory_manager::manager::ProcessContext,
    ) -> Result<(), MemoryError> {
        // Check if instance exists and if context changed (compare handle from instance)
        let context_changed = if let Some(ref hook) = self.instance {
            // SendableHandle wraps HANDLE, compare the inner HANDLE value
            hook.handle.0 != ctx.handle
        } else {
            true // No instance, need to create
        };

        // If context changed, clear address cache to force re-resolution
        if context_changed {
            self.cached_address = None;
        }

        // Resolve address (use cache if available and context unchanged)
        let target_address = if self.cached_address.is_none() {
            // Use unified AddressStrategy to resolve address (offset is already applied by the strategy)
            let addr = self
                .address_strategy
                .resolve(ctx.handle, ctx.pid, ctx.architecture)?;

            // Cache the resolved address
            self.cached_address = Some(addr);

            addr
        } else {
            // Use cached address (fast path!)
            self.cached_address.unwrap()
        };

        // If instance already exists and is installed, just return (already active)
        if !context_changed {
            if let Some(ref hook) = self.instance {
                if hook.is_installed() {
                    return Ok(());
                }
                // Hook exists but not installed (was deactivated), will reinstall below
            }
        }

        // Create new TrampolineHook instance or reinstall existing one
        if !context_changed && self.instance.is_some() {
            // Reuse existing instance - configuration is immutable, just reinstall
            if let Some(ref mut hook) = self.instance {
                hook.install()?;
            }
        } else {
            // Context changed or no instance, create new instance
            let detected_arch = ctx.architecture;

            // Create new TrampolineHook instance
            let mut hook =
                TrampolineHook::auto_new(ctx.handle, target_address, self.config.shellcode.clone());

            // Configure hook parameters with pre-detected architecture
            hook.set_architecture(detected_arch == Architecture::X64);
            hook.set_bytes_to_overwrite(self.config.bytes_to_overwrite);
            hook.set_skip_trampoline(self.config.skip_trampoline);

            // Install the hook
            hook.install()?;

            // Update state
            self.instance = Some(hook);
        }

        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), MemoryError> {
        // Uninstall the hook but KEEP the instance and address cache
        // This allows fast re-activation without re-scanning
        if let Some(ref mut hook) = self.instance {
            // Only uninstall if currently installed
            if hook.is_installed() {
                hook.uninstall()?;
            }
            // Keep the instance (don't take/drop it)
        }

        // Address cache is preserved for fast re-activation
        Ok(())
    }

    fn is_active(&self) -> bool {
        self.instance.is_some()
    }
}

impl TrampolineHookHandler {
    /// Manually clear the cached address (forces re-scan on next activate)
    ///
    /// This is useful when you know the target memory layout has changed
    /// (e.g., after a game update or module reload).
    pub fn clear_cache(&mut self) {
        self.cached_address = None;
    }

    /// Check if this handler has a cached address
    pub fn has_cached_address(&self) -> bool {
        self.cached_address.is_some()
    }
}

unsafe impl Send for TrampolineHookHandler {}
