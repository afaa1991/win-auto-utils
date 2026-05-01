//! TrampolineHook Handler for function hooking

use crate::memory::MemoryError;
use crate::memory_hook::{Architecture, TrampolineHook};
use crate::memory_resolver::AddressSource;
use windows::Win32::Foundation::HANDLE;

use super::super::register::ModifierHandler;

// ==================== TrampolineHook Config ====================

/// TrampolineHook configuration (pure data)
#[derive(Clone, Debug)]
pub struct TrampolineHookConfig {
    pub name: String,
    pub shellcode: Vec<u8>,
    pub architecture: Architecture,
    pub bytes_to_overwrite: usize,
    pub skip_trampoline: bool,
}

// ==================== TrampolineHook Handler ====================

/// TrampolineHook handler with embedded address source
pub struct TrampolineHookHandler {
    config: TrampolineHookConfig,
    address_source: AddressSource,
    instance: Option<TrampolineHook>,
    last_handle: Option<HANDLE>,
    last_pid: Option<u32>,
}

impl TrampolineHookHandler {
    /// Create a new TrampolineHookHandler with address source
    fn new(config: TrampolineHookConfig, address_source: AddressSource) -> Self {
        Self {
            config,
            address_source,
            instance: None,
            last_handle: None,
            last_pid: None,
        }
    }

    /// Factory function to create a TrampolineHookHandler with static address
    pub fn new_trampoline_hook(
        name: impl Into<String>,
        address_source: AddressSource,
        shellcode: Vec<u8>,
        architecture: Architecture,
        bytes_to_overwrite: usize,
        skip_trampoline: bool,
    ) -> Box<dyn ModifierHandler> {
        let config = TrampolineHookConfig {
            name: name.into(),
            shellcode,
            architecture,
            bytes_to_overwrite,
            skip_trampoline,
        };
        Box::new(Self::new(config, address_source))
    }

    /// Convenience function for x86 TrampolineHookHandler with static address
    pub fn new_x86_hook(
        name: impl Into<String>,
        address_source: AddressSource,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
    ) -> Box<dyn ModifierHandler> {
        let config = TrampolineHookConfig {
            name: name.into(),
            shellcode,
            architecture: Architecture::X86,
            bytes_to_overwrite,
            skip_trampoline: false,
        };
        Box::new(Self::new(config, address_source))
    }

    /// Convenience function for x64 TrampolineHookHandler with static address
    pub fn new_x64_hook(
        name: impl Into<String>,
        address_source: AddressSource,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
    ) -> Box<dyn ModifierHandler> {
        let config = TrampolineHookConfig {
            name: name.into(),
            shellcode,
            architecture: Architecture::X64,
            bytes_to_overwrite,
            skip_trampoline: false,
        };
        Box::new(Self::new(config, address_source))
    }

    /// Convenience function for x64 skip_trampoline mode with static address
    pub fn new_x64_skip_trampoline(
        name: impl Into<String>,
        address_source: AddressSource,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
    ) -> Box<dyn ModifierHandler> {
        let config = TrampolineHookConfig {
            name: name.into(),
            shellcode,
            architecture: Architecture::X64,
            bytes_to_overwrite,
            skip_trampoline: true,
        };
        Box::new(Self::new(config, address_source))
    }

    /// Convenience function for x86 skip_trampoline mode with static address
    pub fn new_x86_skip_trampoline(
        name: impl Into<String>,
        address_source: AddressSource,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
    ) -> Box<dyn ModifierHandler> {
        let config = TrampolineHookConfig {
            name: name.into(),
            shellcode,
            architecture: Architecture::X86,
            bytes_to_overwrite,
            skip_trampoline: true,
        };
        Box::new(Self::new(config, address_source))
    }

    /// Generic convenience function with configurable skip_trampoline
    pub fn new_hook_with_skip(
        name: impl Into<String>,
        address_source: AddressSource,
        shellcode: Vec<u8>,
        architecture: Architecture,
        bytes_to_overwrite: usize,
        skip_trampoline: bool,
    ) -> Box<dyn ModifierHandler> {
        let config = TrampolineHookConfig {
            name: name.into(),
            shellcode,
            architecture,
            bytes_to_overwrite,
            skip_trampoline,
        };
        Box::new(Self::new(config, address_source))
    }

    /// Convenience function to create a TrampolineHookHandler with AOB pattern
    pub fn new_trampoline_hook_aob(
        name: String,
        pattern: &str,
        shellcode: Vec<u8>,
        architecture: Architecture,
        bytes_to_overwrite: usize,
        skip_trampoline: bool,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let address_source = AddressSource::from_aob_scan(pattern)?;
        Ok(Self::new_trampoline_hook(
            name,
            address_source,
            shellcode,
            architecture,
            bytes_to_overwrite,
            skip_trampoline,
        ))
    }

    /// Convenience function for x86 TrampolineHookHandler with AOB pattern
    pub fn new_x86_hook_aob(
        name: String,
        pattern: &str,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let address_source = AddressSource::from_aob_scan(pattern)?;
        Ok(Self::new_x86_hook(
            name,
            address_source,
            shellcode,
            bytes_to_overwrite,
        ))
    }

    /// Convenience function for x64 TrampolineHookHandler with AOB pattern
    pub fn new_x64_hook_aob(
        name: String,
        pattern: &str,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let address_source = AddressSource::from_aob_scan(pattern)?;
        Ok(Self::new_x64_hook(
            name,
            address_source,
            shellcode,
            bytes_to_overwrite,
        ))
    }

    /// Convenience function for x64 skip_trampoline mode with AOB pattern
    pub fn new_x64_skip_trampoline_aob(
        name: String,
        pattern: &str,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let address_source = AddressSource::from_aob_scan(pattern)?;
        Ok(Self::new_x64_skip_trampoline(
            name,
            address_source,
            shellcode,
            bytes_to_overwrite,
        ))
    }

    /// Convenience function for x86 skip_trampoline mode with AOB pattern
    pub fn new_x86_skip_trampoline_aob(
        name: String,
        pattern: &str,
        shellcode: Vec<u8>,
        bytes_to_overwrite: usize,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let address_source = AddressSource::from_aob_scan(pattern)?;
        Ok(Self::new_x86_skip_trampoline(
            name,
            address_source,
            shellcode,
            bytes_to_overwrite,
        ))
    }
}

impl ModifierHandler for TrampolineHookHandler {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn activate(&mut self, handle: HANDLE, pid: u32) -> Result<(), MemoryError> {
        // Check if context has changed (handle or pid)
        let context_changed = self.last_handle != Some(handle) || self.last_pid != Some(pid);

        // If instance exists and context unchanged, return directly
        if self.instance.is_some() && !context_changed {
            return Ok(());
        }

        // Deactivate old instance if context changed or creating new one
        if self.instance.is_some() {
            self.deactivate()?;
        }

        // Resolve address from the address source (supports both static and AOB)
        let target_address = self.address_source.resolve(handle, pid).map_err(|e| {
            MemoryError::InvalidAddress(format!("Failed to resolve address: {}", e))
        })?;

        // Create new TrampolineHook instance
        let mut hook =
            TrampolineHook::auto_new(handle, target_address, self.config.shellcode.clone());

        // Configure hook parameters
        hook.set_architecture(self.config.architecture == Architecture::X64);
        hook.set_bytes_to_overwrite(self.config.bytes_to_overwrite);
        hook.set_skip_trampoline(self.config.skip_trampoline);

        // Install the hook
        hook.install()?;

        // Update state
        self.instance = Some(hook);
        self.last_handle = Some(handle);
        self.last_pid = Some(pid);
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), MemoryError> {
        // TrampolineHook automatically restores original bytes and frees memory via Drop
        if let Some(_hook) = self.instance.take() {
            // Drop will automatically call uninstall(), no manual operation needed
        }
        // Clear context tracking
        self.last_handle = None;
        self.last_pid = None;
        Ok(())
    }

    fn is_active(&self) -> bool {
        self.instance.is_some()
    }
}

unsafe impl Send for TrampolineHookHandler {}
