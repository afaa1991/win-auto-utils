//! BytesSwitch Handler for bytecode switching

use windows::Win32::Foundation::HANDLE;
use crate::memory::{MemoryError, read_memory_bytes};
use crate::memory_hook::BytesSwitch;
use crate::memory_resolver::{AddressSource, ParseError};

use super::super::register::ModifierHandler;

// ==================== BytesSwitch Config ====================

/// BytesSwitch configuration (pure data)
#[derive(Clone, Debug)]
pub struct BytesSwitchConfig {
    pub name: String,
    pub byte_count: usize,
    pub patch_bytes: Vec<u8>,
}

// ==================== BytesSwitch Handler ====================

/// BytesSwitch handler with embedded address source
pub struct BytesSwitchHandler {
    config: BytesSwitchConfig,
    address_source: AddressSource,
    instance: Option<BytesSwitch>,
    last_handle: Option<HANDLE>,
    last_pid: Option<u32>,
}

impl BytesSwitchHandler {
    /// Internal constructor (accepts AddressSource)
    fn new(config: BytesSwitchConfig, address_source: AddressSource) -> Self {
        Self {
            config,
            address_source,
            instance: None,
            last_handle: None,
            last_pid: None,
        }
    }

    /// Generic factory method - the single source of truth
    pub fn new_with_address(
        name: impl Into<String>,
        address_source: AddressSource,
        byte_count: usize,
        patch_bytes: Vec<u8>,
    ) -> Box<dyn ModifierHandler> {
        let config = BytesSwitchConfig {
            name: name.into(),
            byte_count,
            patch_bytes,
        };
        Box::new(Self::new(config, address_source))
    }

    /// NOP variant
    pub fn new_nop_with_address(
        name: impl Into<String>,
        address_source: AddressSource,
        byte_count: usize,
    ) -> Box<dyn ModifierHandler> {
        Self::new_with_address(name, address_source, byte_count, vec![0x90; byte_count])
    }

    // ===== Static Address Methods =====
    
    pub fn new_bytes_switch_pattern(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
        patch_bytes: Vec<u8>,
    ) -> Result<Box<dyn ModifierHandler>, ParseError> {
        let addr = AddressSource::from_pattern(pattern)?;
        Ok(Self::new_with_address(name, addr, byte_count, patch_bytes))
    }

    pub fn new_nop_switch_pattern(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
    ) -> Result<Box<dyn ModifierHandler>, ParseError> {
        let addr = AddressSource::from_pattern(pattern)?;
        Ok(Self::new_nop_with_address(name, addr, byte_count))
    }

    pub fn new_bytes_switch_x86(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
        patch_bytes: Vec<u8>,
    ) -> Result<Box<dyn ModifierHandler>, ParseError> {
        let addr = AddressSource::from_pattern_x86(pattern)?;
        Ok(Self::new_with_address(name, addr, byte_count, patch_bytes))
    }

    pub fn new_nop_switch_x86(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
    ) -> Result<Box<dyn ModifierHandler>, ParseError> {
        let addr = AddressSource::from_pattern_x86(pattern)?;
        Ok(Self::new_nop_with_address(name, addr, byte_count))
    }

    pub fn new_bytes_switch_x64(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
        patch_bytes: Vec<u8>,
    ) -> Result<Box<dyn ModifierHandler>, ParseError> {
        let addr = AddressSource::from_pattern_x64(pattern)?;
        Ok(Self::new_with_address(name, addr, byte_count, patch_bytes))
    }

    pub fn new_nop_switch_x64(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
    ) -> Result<Box<dyn ModifierHandler>, ParseError> {
        let addr = AddressSource::from_pattern_x64(pattern)?;
        Ok(Self::new_nop_with_address(name, addr, byte_count))
    }

    // ===== AOB Scan Methods =====
    
    pub fn new_bytes_switch_aob(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
        patch_bytes: Vec<u8>,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let addr = AddressSource::from_aob(pattern)?;
        Ok(Self::new_with_address(name, addr, byte_count, patch_bytes))
    }

    pub fn new_nop_switch_aob(
        name: impl Into<String>,
        pattern: &str,
        byte_count: usize,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let addr = AddressSource::from_aob(pattern)?;
        Ok(Self::new_nop_with_address(name, addr, byte_count))
    }
}

impl ModifierHandler for BytesSwitchHandler {
    fn name(&self) -> &str { &self.config.name }
    
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
        let target_address = self.address_source.resolve(handle, pid)
            .map_err(|e| MemoryError::InvalidAddress(format!("Failed to resolve address: {}", e)))?;

        // Read original bytes from target address
        let original_bytes = read_memory_bytes(handle, target_address, self.config.byte_count)?;

        // Create new BytesSwitch instance
        let mut switch = BytesSwitch::new(
            handle,
            target_address,
            original_bytes,
            self.config.patch_bytes.clone(),
        );
        
        // Enable the switch (write patch bytes)
        switch.enable()?;
        
        // Update state
        self.instance = Some(switch);
        self.last_handle = Some(handle);
        self.last_pid = Some(pid);
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), MemoryError> {
        // BytesSwitch automatically restores original bytes via Drop
        if let Some(_switch) = self.instance.take() {
            // Drop will automatically call disable(), no manual operation needed
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

unsafe impl Send for BytesSwitchHandler {}
