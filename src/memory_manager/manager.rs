//! Memory Manager implementation

use crate::memory::MemoryError;
use crate::memory_hook::Architecture;
use crate::utils::detect_process_architecture;
use std::collections::HashMap;
use windows::Win32::Foundation::HANDLE;

use super::register::ModifierHandler;

/// Process context for runtime binding
#[derive(Clone, Debug)]
pub struct ProcessContext {
    pub handle: HANDLE,
    pub pid: u32,
    /// Target process architecture (auto-detected on initialization)
    pub architecture: Architecture,
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
    ///
    /// # Important: Context Switching Behavior
    /// This method will:
    /// 1. Deactivate all active handlers (uninstall hooks, stop locks)
    /// 2. Clear AOB region cache for the old process
    /// 3. Set new context with auto-detected architecture
    ///
    /// # Note on Address Caching
    /// Handlers cache resolved addresses for performance. After switching context:
    /// - **AOB patterns**: Will re-scan automatically (region cache was cleared)
    /// - **Static patterns**: May use stale cached address
    ///
    /// To force re-resolution of static addresses after context switch:
    /// ```no_run
    /// # use win_auto_utils::memory_manager::ModifierManager;
    /// # let mut manager = ModifierManager::new();
    /// // Unregister and re-register handlers to clear their caches
    /// manager.unregister("my_hook")?;
    /// manager.register("my_hook", new_hook)?;
    /// manager.activate("my_hook")?;
    /// ```
    pub fn set_context(&mut self, handle: HANDLE, pid: u32) {
        // Deactivate all active handlers before changing context
        // This prevents handlers from using stale handle/pid
        let _ = self.deactivate_all();

        // Clear AOB cache for old context before setting new one
        if let Some(old_ctx) = &self.context {
            crate::memory_aobscan::clear_region_cache(old_ctx.handle);
        }

        // Auto-detect architecture when setting context
        let architecture = detect_process_architecture(pid);
        self.context = Some(ProcessContext {
            handle,
            pid,
            architecture,
        });
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

    /// Unregister all handlers and clear the manager
    pub fn clear(&mut self) -> Result<(), MemoryError> {
        // Deactivate all handlers
        for (_, mut handler) in self.handlers.drain() {
            let _ = handler.deactivate();
        }

        // Clear AOB cache for current context
        if let Some(ctx) = &self.context {
            crate::memory_aobscan::clear_region_cache(ctx.handle);
        }

        // Clear context
        self.context = None;

        Ok(())
    }

    /// Activate a specific handler by name
    pub fn activate(&mut self, name: &str) -> Result<(), MemoryError> {
        let ctx = self
            .context
            .as_ref()
            .ok_or_else(|| MemoryError::ReadFailed("No process context bound".to_string()))?;

        let handler = self
            .handlers
            .get_mut(name)
            .ok_or_else(|| MemoryError::ReadFailed(format!("Handler '{}' not found", name)))?;

        // Handler activates with full context (handle, pid, architecture)
        handler.activate(ctx)?;
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

    /// Get the current process context (if set)
    ///
    /// Returns a reference to the current ProcessContext, which contains:
    /// - `handle`: Process handle
    /// - `pid`: Process ID
    /// - `architecture`: Target process architecture (x86/x64)
    ///
    /// # Example
    /// ```no_run
    /// # use win_auto_utils::memory_manager::ModifierManager;
    /// let mut manager = ModifierManager::new();
    /// // ... set_context and register handlers ...
    ///
    /// if let Some(ctx) = manager.get_context() {
    ///     println!("PID: {}", ctx.pid);
    ///     println!("Architecture: {:?}", ctx.architecture);
    /// }
    /// ```
    pub fn get_context(&self) -> Option<&ProcessContext> {
        self.context.as_ref()
    }
}

impl Drop for ModifierManager {
    fn drop(&mut self) {
        // Clear AOB cache to prevent memory leaks
        if let Some(ctx) = &self.context {
            crate::memory_aobscan::clear_region_cache(ctx.handle);
        }

        // Deactivate all handlers (they will be dropped after this)
        for (_, mut handler) in self.handlers.drain() {
            let _ = handler.deactivate();
        }
    }
}
