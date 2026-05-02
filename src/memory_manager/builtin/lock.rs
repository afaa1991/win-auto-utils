//! Lock Handler for memory locking
//!
//! Supports static address patterns with multi-level pointer resolution.
//! Example: "game.exe+1000->2FC->30" (module base + offset -> pointer dereference + offset)

use super::address_strategy::AddressStrategy;
use crate::memory::MemoryError;
use crate::memory_lock::MemoryLock;
use crate::memory_resolver::MemoryAddress;
use std::time::Duration;

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

/// Lock handler that directly uses MemoryAddress for static addresses
pub struct LockHandler {
    config: LockConfig,
    /// Parsed MemoryAddress - resolved at activation time using architecture from context
    memory_address: Option<MemoryAddress>,
    /// Address strategy used to build the MemoryAddress
    address_strategy: AddressStrategy,
    instance: Option<MemoryLock>,
}

impl LockHandler {
    /// Internal constructor
    fn new(config: LockConfig, address_strategy: AddressStrategy) -> Self {
        Self {
            config,
            address_strategy,
            memory_address: None,
            instance: None,
        }
    }

    /// Create a LockHandler with static address pattern.
    ///
    /// The architecture is automatically detected at activation time from the ProcessContext.
    /// Supports multi-level pointer chains with offsets.
    ///
    /// # Address Pattern Syntax
    /// - **Module base**: `module_name+offset` (e.g., "game.exe+1000")
    /// - **Pointer dereference**: `->offset` (e.g., "->2FC")
    /// - **Direct offset**: `+offset` (e.g., "+30")
    /// - **Hex by default**: All numbers are hexadecimal unless prefixed with `#`
    /// - **Decimal marker**: `#` prefix (e.g., "#100" = decimal 100)
    ///
    /// # Arguments
    /// * `name` - Lock name (must be unique within ModifierManager)
    /// * `pattern` - Static address pattern string
    /// * `value` - Value to lock (any type that implements AsBytes)
    /// * `scan_interval` - How often to check and restore the locked value
    ///
    /// # Examples
    /// ```no_run
    /// use win_auto_utils::memory_manager::builtin::LockHandler;
    /// use std::time::Duration;
    ///
    /// // Simple module + offset
    /// let lock1 = LockHandler::new_lock(
    ///     "health_lock",
    ///     "game.exe+1000",
    ///     100i32,
    ///     Duration::from_millis(100),
    /// )?;
    ///
    /// // Multi-level pointer chain
    /// let lock2 = LockHandler::new_lock(
    ///     "ammo_lock",
    ///     "game.exe+5000->2FC->30",
    ///     999i32,
    ///     Duration::from_millis(50),
    /// )?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_lock<T: crate::memory_lock::AsBytes>(
        name: impl Into<String>,
        pattern: &str,
        value: T,
        scan_interval: Duration,
    ) -> Result<Box<dyn ModifierHandler>, crate::memory_resolver::ParseError> {
        let config = LockConfig {
            name: name.into(),
            value: value.as_bytes().to_vec(),
            scan_interval,
        };
        Ok(Box::new(Self::new(
            config,
            AddressStrategy::static_pattern(pattern),
        )))
    }
}

impl ModifierHandler for LockHandler {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn activate(
        &mut self,
        ctx: &crate::memory_manager::manager::ProcessContext,
    ) -> Result<(), MemoryError> {
        // Check if we can reuse existing instance by comparing handle/pid from the instance itself
        let context_changed = if let Some(ref lock) = self.instance {
            lock.get_handle() != Some(ctx.handle) || lock.get_pid() != Some(ctx.pid)
        } else {
            true // No instance, need to create
        };

        if !context_changed {
            // Same process, just restart the monitoring thread
            if let Some(ref mut lock) = self.instance {
                return lock.lock_bytes(&self.config.value);
            }
        }

        // Build new MemoryLock instance (context changed or no instance)
        // Build memoryAddress from strategy using context architecture
        let mem_addr = self
            .address_strategy
            .build_memory_address(ctx.architecture)?;

        // Store the parsed MemoryAddress for potential future updates
        self.memory_address = Some(mem_addr.clone());

        // Resolve address using MemoryAddress
        let target_address = mem_addr.resolve_address(ctx.handle, ctx.pid).map_err(|e| {
            MemoryError::InvalidAddress(format!("Failed to resolve lock address: {}", e))
        })?;

        // Create new instance with current context
        let mut lock = MemoryLock::builder()
            .handle(ctx.handle)
            .pid(ctx.pid)
            .address(target_address)
            .bytes(self.config.value.clone())
            .scan_interval(self.config.scan_interval)
            .build()?;

        // Start monitoring
        lock.lock_bytes(&self.config.value)?;

        // Cache the instance
        self.instance = Some(lock);
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), MemoryError> {
        // Stop the monitoring thread but keep the instance cached
        // This allows quick re-activation without rebuilding
        if let Some(ref mut lock) = self.instance {
            lock.stop()?;
        }
        Ok(())
    }

    fn is_active(&self) -> bool {
        // Check if instance exists and is running
        self.instance
            .as_ref()
            .map_or(false, |lock| lock.is_running())
    }
}

unsafe impl Send for LockHandler {}
