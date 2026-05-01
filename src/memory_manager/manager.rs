//! Memory Manager implementation

use std::collections::HashMap;
use windows::Win32::Foundation::HANDLE;
use crate::memory::MemoryError;

use super::register::ModifierHandler;

/// Process context for runtime binding
#[derive(Clone, Debug)]
pub struct ProcessContext {
    pub handle: HANDLE,
    pub pid: u32,
}

/// Unified manager for memory modifications (pure dispatcher)
pub struct ModifierManager {
    context: Option<ProcessContext>,
    handlers: HashMap<String, Box<dyn ModifierHandler>>,
}

impl ModifierManager {
    /// Create a new empty manager
    pub fn new() -> Self {
        Self {
            context: None,
            handlers: HashMap::new(),
        }
    }

    /// Bind process context (handle and pid)
    pub fn set_context(&mut self, handle: HANDLE, pid: u32) {
        self.context = Some(ProcessContext { handle, pid });
    }

    /// Register a handler (Handler already contains address source)
    pub fn register(&mut self, name: impl Into<String>, handler: Box<dyn ModifierHandler>) {
        let name = name.into();
        
        // Auto-uninstall old handler if exists
        if let Some(mut old_handler) = self.handlers.remove(&name) {
            let _ = old_handler.deactivate();
        }
        
        self.handlers.insert(name, handler);
    }

    /// Unregister and deactivate a handler
    pub fn unregister(&mut self, name: &str) -> Result<(), MemoryError> {
        if let Some(mut handler) = self.handlers.remove(name) {
            handler.deactivate()?;
        }
        Ok(())
    }

    /// Activate a specific handler by name
    pub fn activate(&mut self, name: &str) -> Result<(), MemoryError> {
        let ctx = self.context.as_ref()
            .ok_or_else(|| MemoryError::ReadFailed("No process context bound".to_string()))?;

        let handler = self.handlers.get_mut(name)
            .ok_or_else(|| MemoryError::ReadFailed(format!("Handler '{}' not found", name)))?;

        // Handler resolves address and activates itself
        handler.activate(ctx.handle, ctx.pid)?;
        Ok(())
    }

    /// Deactivate a specific handler
    pub fn deactivate(&mut self, name: &str) -> Result<(), MemoryError> {
        if let Some(handler) = self.handlers.get_mut(name) {
            handler.deactivate()?;
        }
        Ok(())
    }

    /// Activate all registered handlers
    pub fn activate_all(&mut self) -> Result<(), MemoryError> {
        let names: Vec<String> = self.handlers.keys().cloned().collect();
        for name in names {
            self.activate(&name)?;
        }
        Ok(())
    }

    /// Deactivate all handlers
    pub fn deactivate_all(&mut self) -> Result<(), MemoryError> {
        let names: Vec<String> = self.handlers.keys().cloned().collect();
        for name in names {
            self.deactivate(&name)?;
        }
        Ok(())
    }

    /// Check if a handler is active
    pub fn is_active(&self, name: &str) -> bool {
        self.handlers.get(name).map_or(false, |h| h.is_active())
    }

    /// Get list of registered handler names
    pub fn list_handlers(&self) -> Vec<&String> {
        self.handlers.keys().collect()
    }
}

impl Drop for ModifierManager {
    fn drop(&mut self) {
        let _ = self.deactivate_all();
    }
}