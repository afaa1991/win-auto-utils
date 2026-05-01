//! Lock Handler for memory locking

use std::time::Duration;
use windows::Win32::Foundation::HANDLE;
use crate::memory::MemoryError;
use crate::memory_lock::MemoryLock;
use crate::memory_resolver::{AddressSource, ParseError};

use super::super::register::ModifierHandler;

// ==================== Lock Config ====================

/// Lock configuration (pure data)
#[derive(Clone, Debug)]
pub struct LockConfig {
    pub name: String,
    pub value: Vec<u8>,
    pub scan_interval: Duration,
}

// ==================== Lock Handler ====================

/// Lock handler with embedded address source
pub struct LockHandler {
    config: LockConfig,
    address_source: AddressSource,
    instance: Option<MemoryLock>,
    last_handle: Option<HANDLE>,
    last_pid: Option<u32>,
}

impl LockHandler {
    /// Internal constructor (accepts AddressSource)
    fn new(config: LockConfig, address_source: AddressSource) -> Self {
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
        value: Vec<u8>,
        scan_interval: Duration,
    ) -> Box<dyn ModifierHandler> {
        let config = LockConfig {
            name: name.into(),
            value,
            scan_interval,
        };
        Box::new(Self::new(config, address_source))
    }

    /// Typed value variant
    pub fn new_with_address_typed<T: crate::memory_lock::AsBytes>(
        name: impl Into<String>,
        address_source: AddressSource,
        value: T,
        scan_interval: Duration,
    ) -> Box<dyn ModifierHandler> {
        Self::new_with_address(name, address_source, value.as_bytes().to_vec(), scan_interval)
    }

    /// Create with x86 architecture pattern
    pub fn new_lock_x86(
        name: impl Into<String>,
        pattern: &str,
        value: Vec<u8>,
        scan_interval: Duration,
    ) -> Result<Box<dyn ModifierHandler>, ParseError> {
        let addr = AddressSource::from_pattern_x86(pattern)?;
        Ok(Self::new_with_address(name, addr, value, scan_interval))
    }

    pub fn new_lock_x86_typed<T: crate::memory_lock::AsBytes>(
        name: impl Into<String>,
        pattern: &str,
        value: T,
        scan_interval: Duration,
    ) -> Result<Box<dyn ModifierHandler>, ParseError> {
        let addr = AddressSource::from_pattern_x86(pattern)?;
        Ok(Self::new_with_address_typed(name, addr, value, scan_interval))
    }

    /// Create with x64 architecture pattern
    pub fn new_lock_x64(
        name: impl Into<String>,
        pattern: &str,
        value: Vec<u8>,
        scan_interval: Duration,
    ) -> Result<Box<dyn ModifierHandler>, ParseError> {
        let addr = AddressSource::from_pattern_x64(pattern)?;
        Ok(Self::new_with_address(name, addr, value, scan_interval))
    }

    pub fn new_lock_x64_typed<T: crate::memory_lock::AsBytes>(
        name: impl Into<String>,
        pattern: &str,
        value: T,
        scan_interval: Duration,
    ) -> Result<Box<dyn ModifierHandler>, ParseError> {
        let addr = AddressSource::from_pattern_x64(pattern)?;
        Ok(Self::new_with_address_typed(name, addr, value, scan_interval))
    }

    // ===== AOB Scan Methods =====

    /// Create from AOB pattern
    pub fn new_lock_aob(
        name: impl Into<String>,
        pattern: &str,
        value: Vec<u8>,
        scan_interval: Duration,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let addr = AddressSource::from_aob(pattern)?;
        Ok(Self::new_with_address(name, addr, value, scan_interval))
    }

    pub fn new_lock_aob_typed<T: crate::memory_lock::AsBytes>(
        name: impl Into<String>,
        pattern: &str,
        value: T,
        scan_interval: Duration,
    ) -> Result<Box<dyn ModifierHandler>, String> {
        let addr = AddressSource::from_aob(pattern)?;
        Ok(Self::new_with_address_typed(name, addr, value, scan_interval))
    }
}

impl ModifierHandler for LockHandler {
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

        // Create new instance with current context
        let mut lock = MemoryLock::builder()
            .handle(handle)
            .pid(pid)
            .address(target_address)
            .bytes(self.config.value.clone())
            .scan_interval(self.config.scan_interval)
            .build()?;
        
        // Start monitoring
        lock.lock_bytes(&self.config.value)?;
        
        // Update state
        self.instance = Some(lock);
        self.last_handle = Some(handle);
        self.last_pid = Some(pid);
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), MemoryError> {
        // MemoryLock automatically stops thread via Drop
        if let Some(_lock) = self.instance.take() {
            // Drop will automatically call stop_flag, no manual operation needed
        }
        // Clear context tracking
        self.last_handle = None;
        self.last_pid = None;
        Ok(())
    }

    fn is_active(&self) -> bool {
        // MemoryLock has no is_running method, we determine by checking if instance exists
        self.instance.is_some()
    }
}

unsafe impl Send for LockHandler {}
