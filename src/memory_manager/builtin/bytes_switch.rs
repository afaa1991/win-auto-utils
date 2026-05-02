//! BytesSwitch Handler for bytecode switching

use super::address_strategy::AddressStrategy;
use crate::memory::{read_memory_bytes, MemoryError};
use crate::memory_hook::BytesSwitch;

use super::super::register::ModifierHandler;

#[derive(Clone, Debug)]
pub struct BytesSwitchConfig {
    pub name: String,
    pub byte_count: usize,
    pub patch_bytes: Vec<u8>,
}

pub struct BytesSwitchHandler {
    config: BytesSwitchConfig,
    /// Address resolution strategy - stores enough info to resolve at activation time
    address_strategy: AddressStrategy,
    cached_address: Option<usize>,
    instance: Option<BytesSwitch>,
}

impl BytesSwitchHandler {
    fn new(config: BytesSwitchConfig, address_strategy: AddressStrategy) -> Self {
        Self {
            config,
            address_strategy,
            cached_address: None,
            instance: None,
        }
    }

    /// Create a BytesSwitchHandler with static address pattern.
    ///
    /// Architecture is automatically detected at activation time from ProcessContext.
    ///
    /// # Arguments
    /// * `name` - Switch name (must be unique)
    /// * `pattern` - Static address pattern string (e.g., "game.exe+1000")
    /// * `byte_count` - Number of bytes to switch
    /// * `patch_bytes` - Bytes to write when enabled
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_manager::builtin::BytesSwitchHandler;
    ///
    /// let switch = BytesSwitchHandler::new_bytes_switch(
    ///     "nop_patch",
    ///     "game.exe+2000",
    ///     2,
    ///     vec![0x90, 0x90],
    /// )?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_bytes_switch(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
        patch_bytes: Vec<u8>,
    ) -> Result<Box<dyn ModifierHandler>, crate::memory_resolver::ParseError> {
        let config = BytesSwitchConfig {
            name: name.into(),
            byte_count,
            patch_bytes,
        };
        Ok(Box::new(Self::new(
            config,
            AddressStrategy::static_pattern(pattern),
        )))
    }

    /// Create a NOP switch with static address pattern.
    pub fn new_nop_switch(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
    ) -> Result<Box<dyn ModifierHandler>, crate::memory_resolver::ParseError> {
        Self::new_bytes_switch(name, pattern, byte_count, vec![0x90; byte_count])
    }

    /// Create a BytesSwitchHandler with AOB scanning.
    ///
    /// Uses AOB pattern scanning to find the target address at runtime.
    /// Architecture-independent.
    ///
    /// # Performance Note
    /// - First activation: Executes AOB scan (may take 10-500ms)
    /// - Subsequent activations: Reuses cached address
    /// - Cache auto-invalidates when PID changes
    ///
    /// # Arguments
    /// * `name` - Switch name
    /// * `pattern` - AOB pattern string (space-separated hex bytes)
    /// * `byte_count` - Number of bytes to switch
    /// * `patch_bytes` - Bytes to write when enabled
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_manager::builtin::BytesSwitchHandler;
    ///
    /// let switch = BytesSwitchHandler::new_bytes_switch_aob(
    ///     "nop_patch",
    ///     "90 90 90 90 90",
    ///     2,
    ///     vec![0x90, 0x90],
    /// )?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_bytes_switch_aob(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
        patch_bytes: Vec<u8>,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let config = BytesSwitchConfig {
            name: name.into(),
            byte_count,
            patch_bytes,
        };
        Ok(Box::new(Self::new(
            config,
            AddressStrategy::aob_pattern(pattern),
        )))
    }

    /// Create a NOP switch with AOB scanning.
    pub fn new_nop_switch_aob(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        Self::new_bytes_switch_aob(name, pattern, byte_count, vec![0x90; byte_count])
    }

    /// Create a BytesSwitchHandler with AOB scanning and custom range.
    ///
    /// Same as `new_bytes_switch_aob()` but allows specifying a memory range for faster scanning.
    /// Use this when you know the approximate location of the target code.
    ///
    /// # Arguments
    /// * `name` - Switch name
    /// * `pattern` - AOB pattern string
    /// * `byte_count` - Number of bytes to switch
    /// * `patch_bytes` - Bytes to write when enabled
    /// * `start_address` - Starting address for the scan (0 = from beginning)
    /// * `length` - Length of the scan range (0 = entire process memory)
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_manager::builtin::BytesSwitchHandler;
    ///
    /// // Scan only within a specific module range (much faster!)
    /// let switch = BytesSwitchHandler::new_bytes_switch_aob_with_range(
    ///     "nop_patch",
    ///     "90 90 90 90 90",
    ///     2,
    ///     vec![0x90, 0x90],
    ///     0x10000000,  // Start address (e.g., module base)
    ///     0x100000,    // Length (e.g., module size)
    /// )?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_bytes_switch_aob_with_range(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
        patch_bytes: Vec<u8>,
        start_address: usize,
        length: usize,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        Self::new_bytes_switch_aob_with_range_and_offset(
            name,
            pattern,
            byte_count,
            patch_bytes,
            start_address,
            length,
            0,
        )
    }

    /// Create a BytesSwitchHandler with AOB scanning and offset.
    ///
    /// Same as `new_bytes_switch_aob()` but supports adding an offset to the scanned address.
    /// Useful when the AOB pattern finds a nearby signature, but you need to patch at a different location.
    ///
    /// # Arguments
    /// * `name` - Switch name
    /// * `pattern` - AOB pattern string
    /// * `byte_count` - Number of bytes to switch
    /// * `patch_bytes` - Bytes to write when enabled
    /// * `offset` - Offset to add to the scanned address (can be negative)
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_manager::builtin::BytesSwitchHandler;
    ///
    /// // Scan for function prologue, but patch 0x42 bytes later
    /// let switch = BytesSwitchHandler::new_bytes_switch_aob_with_offset(
    ///     "nop_patch",
    ///     "48 89 5C 24",
    ///     2,
    ///     vec![0x90, 0x90],
    ///     0x42,
    /// )?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_bytes_switch_aob_with_offset(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
        patch_bytes: Vec<u8>,
        offset: i64,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let config = BytesSwitchConfig {
            name: name.into(),
            byte_count,
            patch_bytes,
        };
        Ok(Box::new(Self::new(
            config,
            AddressStrategy::aob_pattern_with_offset(pattern, offset),
        )))
    }

    /// Create a NOP switch with AOB scanning and offset.
    pub fn new_nop_switch_aob_with_offset(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
        offset: i64,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        Self::new_bytes_switch_aob_with_offset(
            name,
            pattern,
            byte_count,
            vec![0x90; byte_count],
            offset,
        )
    }

    /// Create a BytesSwitchHandler with AOB scanning, custom range and offset.
    ///
    /// Combines range-limited scanning with post-scan address correction.
    ///
    /// # Arguments
    /// * `name` - Switch name
    /// * `pattern` - AOB pattern string
    /// * `byte_count` - Number of bytes to switch
    /// * `patch_bytes` - Bytes to write when enabled
    /// * `start_address` - Starting address for the scan (0 = from beginning)
    /// * `length` - Length of the scan range (0 = entire process memory)
    /// * `offset` - Offset to add to the scanned address (can be negative)
    pub fn new_bytes_switch_aob_with_range_and_offset(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
        patch_bytes: Vec<u8>,
        start_address: usize,
        length: usize,
        offset: i64,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let config = BytesSwitchConfig {
            name: name.into(),
            byte_count,
            patch_bytes,
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

    /// Create a NOP switch with AOB scanning, custom range and offset.
    pub fn new_nop_switch_aob_with_range_and_offset(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
        start_address: usize,
        length: usize,
        offset: i64,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        Self::new_bytes_switch_aob_with_range_and_offset(
            name,
            pattern,
            byte_count,
            vec![0x90; byte_count],
            start_address,
            length,
            offset,
        )
    }
}

impl ModifierHandler for BytesSwitchHandler {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn activate(
        &mut self,
        ctx: &crate::memory_manager::manager::ProcessContext,
    ) -> Result<(), MemoryError> {
        // Check if instance exists and if context changed (compare handle from instance)
        let context_changed = if let Some(ref switch) = self.instance {
            // BytesSwitch wraps HANDLE internally, compare the inner HANDLE value
            switch.get_handle() != ctx.handle
        } else {
            true // No instance, need to create
        };

        // If context changed, clear address cache to force re-resolution
        if context_changed {
            self.cached_address = None;
        }

        // Resolve address (use cache if available and context unchanged)
        let target_address = if self.cached_address.is_none() {
            // Use unified AddressStrategy to resolve address
            let addr = self
                .address_strategy
                .resolve(ctx.handle, ctx.pid, ctx.architecture)?;
            self.cached_address = Some(addr);
            addr
        } else {
            self.cached_address.unwrap()
        };

        // If instance already exists and is enabled, just return (already active)
        if !context_changed && self.instance.is_some() {
            if let Some(ref switch) = self.instance {
                if switch.is_enabled() {
                    return Ok(());
                }
                // Switch exists but disabled, will re-enable below
            }
        }

        // Create new BytesSwitch instance or re-enable existing one
        if !context_changed && self.instance.is_some() {
            // Reuse existing instance (just re-enable)
            if let Some(ref mut switch) = self.instance {
                switch.enable()?;
            }
        } else {
            // Context changed or no instance, create new instance
            let original_bytes =
                read_memory_bytes(ctx.handle, target_address, self.config.byte_count)?;

            let mut switch = BytesSwitch::new(
                ctx.handle,
                target_address,
                original_bytes,
                self.config.patch_bytes.clone(),
            );

            switch.enable()?;

            self.instance = Some(switch);
        }

        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), MemoryError> {
        // Disable the switch but KEEP the instance and address cache
        // This allows fast re-activation without re-scanning
        if let Some(ref mut switch) = self.instance {
            if switch.is_enabled() {
                switch.disable()?;
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

impl BytesSwitchHandler {
    /// Manually clear the cached address (forces re-scan on next activate)
    pub fn clear_cache(&mut self) {
        self.cached_address = None;
    }

    /// Check if this handler has a cached address
    pub fn has_cached_address(&self) -> bool {
        self.cached_address.is_some()
    }
}

unsafe impl Send for BytesSwitchHandler {}
